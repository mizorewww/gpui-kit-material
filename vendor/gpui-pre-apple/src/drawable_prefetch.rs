//! Material patch: acquire the next `CAMetalDrawable` off the main thread.
//!
//! `-[CAMetalLayer nextDrawable]` blocks until the compositor returns a
//! drawable to the layer's pool (up to a display refresh, with or without
//! display sync). Upstream called it on the main thread inside every draw, so
//! that wait stalled input, layout and painting. Here a thread owned by the
//! renderer keeps at most one drawable acquired ahead of the next frame. The
//! main thread takes it without blocking, and asks for the next one as soon
//! as it has taken one, so the wait overlaps the following frame's CPU work.
//!
//! The platform window asks [`DrawablePrefetch::ready_or_wake`] before running
//! a frame: when no drawable is ready yet, it skips that request, and the
//! prefetch thread calls the window's waker once one is. Frames are therefore
//! never drawn into a drawable the main thread waits for, unless prefetching
//! is unavailable (paused for transactional presentation during live resize,
//! failed, or the drawable size changed), where the renderer falls back to
//! upstream's synchronous acquisition and reports that wait.
//!
//! A drawable that is never presented returns to the layer's pool when it is
//! released, so a prefetched drawable of a stale size is simply dropped.

use std::{
    sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError},
    thread,
};

/// Wakes the window that owns the renderer, from any thread.
pub type DrawableWaker = Arc<dyn Fn() + Send + Sync>;

#[derive(Default)]
struct State {
    /// The drawable acquired for the next frame, if any.
    ready: Option<metal::MetalDrawable>,
    /// The main thread wants a drawable acquired.
    requested: bool,
    /// The prefetch thread is inside `nextDrawable`.
    acquiring: bool,
    /// Prefetching is paused: transactional presentation (live resize) and
    /// headless renderers acquire synchronously.
    paused: bool,
    /// The last acquisition returned no drawable (zero-sized or detached
    /// layer); the main thread acquires synchronously until one succeeds.
    failed: bool,
    /// Call the waker once a drawable is ready.
    wake_when_ready: bool,
    waker: Option<DrawableWaker>,
    shutdown: bool,
}

struct Shared {
    state: Mutex<State>,
    changed: Condvar,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// One renderer's drawable prefetch thread and its hand-off slot.
pub struct DrawablePrefetch {
    shared: Arc<Shared>,
}

impl DrawablePrefetch {
    /// Starts the prefetch thread for `layer`. It acquires nothing until the
    /// first request.
    pub fn new(layer: metal::MetalLayer) -> Self {
        let shared = Arc::new(Shared {
            state: Mutex::new(State::default()),
            changed: Condvar::new(),
        });
        let thread_shared = shared.clone();
        let spawned = thread::Builder::new()
            .name("gpui-drawable-prefetch".into())
            .spawn(move || run(&thread_shared, &layer));
        if let Err(error) = spawned {
            log::error!("failed to start the drawable prefetch thread: {error}");
            let mut state = shared.lock();
            state.paused = true;
            state.shutdown = true;
        }
        Self { shared }
    }

    /// Sets the window waker called when a drawable a frame waited for is
    /// ready. `None` stops waking.
    pub fn set_waker(&self, waker: Option<DrawableWaker>) {
        self.shared.lock().waker = waker;
    }

    /// Pauses prefetching (dropping any prefetched drawable) or resumes it.
    pub fn set_paused(&self, paused: bool) {
        let mut state = self.shared.lock();
        state.paused = paused;
        if paused {
            state.ready = None;
        } else {
            state.requested = true;
        }
        drop(state);
        self.shared.changed.notify_all();
    }

    /// Asks for a drawable ahead of the first frame (or after a resize).
    pub fn request(&self) {
        self.shared.lock().requested = true;
        self.shared.changed.notify_all();
    }

    /// Whether a frame started now gets its drawable without blocking the
    /// main thread. If not, the window waker is called once it would.
    pub fn ready_or_wake(&self) -> bool {
        let mut state = self.shared.lock();
        if state.ready.is_some() || state.paused || state.failed || state.shutdown {
            return true;
        }
        state.requested = true;
        state.wake_when_ready = true;
        drop(state);
        self.shared.changed.notify_all();
        false
    }

    /// Takes the prefetched drawable if one of `size` is ready, and asks for
    /// the next. `None` means the caller acquires synchronously; a drawable
    /// still being acquired is then left to the next frame.
    pub fn take(&self, width: u64, height: u64) -> Option<metal::MetalDrawable> {
        let mut state = self.shared.lock();
        if state.paused || state.shutdown {
            return None;
        }
        let drawable = state.ready.take();
        state.requested = true;
        drop(state);
        self.shared.changed.notify_all();
        let drawable = drawable?;
        let texture = drawable.texture();
        (texture.width() == width && texture.height() == height).then_some(drawable)
    }

    /// Waits for the requested or in-progress acquisition and takes its
    /// drawable, for the rare frame that runs before it finished. Returns
    /// `None` if prefetching is paused or the acquisition failed.
    pub fn wait_take(&self, width: u64, height: u64) -> Option<metal::MetalDrawable> {
        let mut state = self.shared.lock();
        while (state.acquiring || state.requested)
            && state.ready.is_none()
            && !state.paused
            && !state.shutdown
        {
            state = self
                .shared
                .changed
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
        }
        let drawable = state.ready.take();
        state.requested = true;
        drop(state);
        self.shared.changed.notify_all();
        let drawable = drawable?;
        let texture = drawable.texture();
        (texture.width() == width && texture.height() == height).then_some(drawable)
    }
}

impl Drop for DrawablePrefetch {
    fn drop(&mut self) {
        let mut state = self.shared.lock();
        state.shutdown = true;
        state.ready = None;
        state.waker = None;
        drop(state);
        self.shared.changed.notify_all();
        // Not joined: the thread may be inside `nextDrawable`, and exits when
        // that returns. It owns a reference to the layer until then.
    }
}

fn run(shared: &Shared, layer: &metal::MetalLayer) {
    loop {
        let mut state = shared.lock();
        while !state.shutdown && (state.paused || !state.requested || state.ready.is_some()) {
            state = shared
                .changed
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
        }
        if state.shutdown {
            return;
        }
        state.acquiring = true;
        state.requested = false;
        drop(state);

        // `nextDrawable` returns an autoreleased drawable; retain it before
        // this thread's pool drains.
        let drawable = objc2::rc::autoreleasepool(|_| {
            layer.next_drawable().map(|drawable| drawable.to_owned())
        });

        let mut state = shared.lock();
        state.acquiring = false;
        state.failed = drawable.is_none();
        if !state.paused && !state.shutdown {
            state.ready = drawable;
        }
        // A failed acquisition wakes too: the frame then acquires
        // synchronously instead of waiting for a drawable that won't come.
        let waker = if std::mem::take(&mut state.wake_when_ready) && !state.shutdown {
            state.waker.clone()
        } else {
            None
        };
        drop(state);
        shared.changed.notify_all();
        if let Some(waker) = waker {
            waker();
        }
    }
}
