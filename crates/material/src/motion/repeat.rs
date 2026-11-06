//! A repeating decorative animation that draws frames only while it can be seen.
//!
//! GPUI's `with_animation(...).repeat()` asks for a frame on every draw, so an
//! indeterminate indicator scrolled out of a list, or clipped by its parent,
//! keeps an otherwise idle window drawing. [`RepeatWhenVisible`] samples the
//! same clock (the executor's, from the element's first layout or from a
//! caller-owned [`RepeatAnimationPlayback`]) but requests the next frame only
//! when its bounds intersect the content mask and the window viewport.
//!
//! Use it only for content that draws inside its own bounds and whose motion
//! cannot carry it into view from outside: overflow, shadows past the bounds
//! and entrance animations need the stock scheduling. Opacity is not
//! considered: a transparent indicator in view keeps its clock moving.
//!
//! Scheduling is committed in paint, after a successful prepaint, so a list's
//! retried prepaint never queues a frame for bounds it then discards. One frame
//! callback queued while visible may still notify its view once after the
//! element leaves the viewport; that draw does not queue another.
use gpui::*;
use std::cell::Cell;
#[cfg(not(target_family = "wasm"))]
use std::time::Instant;
use std::{rc::Rc, time::Duration};
#[cfg(target_family = "wasm")]
use web_time::Instant;

/// A repeating animation's clock, owned by the caller rather than the mounted
/// element.
///
/// Keep this in the view that owns a virtualized row: the clock starts on the
/// first layout of an element using it and keeps running while that element is
/// unmounted, so a row scrolled back into view resumes its phase instead of
/// restarting. It holds no view, pending callback, layout or paint state. Clones
/// share one clock; a new handle starts a new playback. Use one handle per
/// logical playback, with an unchanged period, within one app.
#[derive(Clone, Default)]
pub struct RepeatAnimationPlayback {
    start: Rc<Cell<Option<Instant>>>,
}

impl RepeatAnimationPlayback {
    /// Whether a mounted element has started this clock.
    pub fn is_started(&self) -> bool {
        self.start.get().is_some()
    }
}

/// Wraps `E` in a repeating animation scheduled only while visible.
pub trait RepeatWhenVisibleExt: IntoElement + Sized + 'static {
    /// Repeat `animator` every `period`, passing the linear phase in `0..=1`,
    /// and request frames only while this element is visible. Under reduced
    /// motion the phase stays at 0 and no frames are requested.
    ///
    /// # Panics
    /// Panics if `period` is zero.
    fn repeat_when_visible(
        self,
        id: impl Into<ElementId>,
        period: Duration,
        animator: impl Fn(Self, f32) -> Self + 'static,
    ) -> RepeatWhenVisible<Self> {
        assert!(!period.is_zero(), "a repeating animation needs a period");
        RepeatWhenVisible {
            id: id.into(),
            element: Some(self),
            period,
            playback: None,
            animator: Box::new(animator),
            next_frame: None,
        }
    }
}

impl<E: IntoElement + 'static> RepeatWhenVisibleExt for E {}

/// The element [`RepeatWhenVisibleExt::repeat_when_visible`] returns.
pub struct RepeatWhenVisible<E> {
    id: ElementId,
    element: Option<E>,
    period: Duration,
    playback: Option<RepeatAnimationPlayback>,
    animator: Box<dyn Fn(E, f32) -> E + 'static>,
    /// The view to notify for the next frame, while the animation runs. Set in
    /// layout, used in paint when the element turned out to be visible.
    next_frame: Option<EntityId>,
}

impl<E> RepeatWhenVisible<E> {
    /// Sample a caller-owned clock instead of the element's own.
    pub fn playback(mut self, playback: Option<RepeatAnimationPlayback>) -> Self {
        self.playback = playback;
        self
    }
}

/// The element's own clock, kept while it stays mounted.
struct RepeatState {
    start: Instant,
}

/// Where a repeat of `period` stands `elapsed` after it started, as GPUI's
/// repeating animation samples it.
fn phase(elapsed: Duration, period: Duration) -> f32 {
    let delta = elapsed.as_secs_f32() / period.as_secs_f32();
    if delta > 1. { delta % 1. } else { delta }
}

/// Whether `bounds` has a positive area inside both `mask` and the viewport.
fn visible(bounds: Bounds<Pixels>, mask: Bounds<Pixels>, viewport: Size<Pixels>) -> bool {
    let left = bounds.left().max(mask.left()).max(px(0.));
    let top = bounds.top().max(mask.top()).max(px(0.));
    let right = bounds.right().min(mask.right()).min(viewport.width);
    let bottom = bounds.bottom().min(mask.bottom()).min(viewport.height);
    right > left && bottom > top
}

impl<E: IntoElement + 'static> IntoElement for RepeatWhenVisible<E> {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl<E: IntoElement + 'static> Element for RepeatWhenVisible<E> {
    type RequestLayoutState = AnyElement;
    type PrepaintState = bool;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, AnyElement) {
        window.with_element_state(global_id.unwrap(), |state, window| {
            let now = cx.background_executor().now();
            let mut state = state.unwrap_or(RepeatState { start: now });
            if let Some(playback) = &self.playback {
                state.start = playback.start.get().unwrap_or_else(|| {
                    playback.start.set(Some(now));
                    now
                });
            }
            let running = !cx.reduce_motion();
            let t = if running {
                phase(now.duration_since(state.start), self.period)
            } else {
                0.
            };
            let element = self.element.take().expect("laid out once");
            let mut element = (self.animator)(element, t).into_any_element();
            self.next_frame = running.then(|| window.current_view());
            ((element.request_layout(window, cx), element), state)
        })
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        element: &mut AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> bool {
        // Prepaint bounds already include scroll offsets.
        let shown = self.next_frame.is_some()
            && visible(bounds, window.content_mask().bounds, window.viewport_size());
        element.prepaint(window, cx);
        shown
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        element: &mut AnyElement,
        shown: &mut bool,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
        if *shown && let Some(view) = self.next_frame.take() {
            window.on_next_frame(move |_, cx| cx.notify(view));
        }
    }
}

#[cfg(test)]
mod tests;
