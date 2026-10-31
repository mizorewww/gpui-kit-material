//! Read-only window counters for the browser profiler in `www/profile.js`.
//!
//! JavaScript times each `requestAnimationFrame` callback with
//! `performance.now()`; these exports add the engine's own successful
//! new-frame count and VSync readback. They never draw, notify or allocate a
//! frame. Call them from ordinary browser tasks, never from inside a GPUI
//! callback (App is borrowed there).
use gpui::{App, AsyncApp, VsyncStatus};
use std::cell::RefCell;
use wasm_bindgen::prelude::wasm_bindgen;

thread_local! {
    static APP: RefCell<Option<AsyncApp>> = const { RefCell::new(None) };
}

pub(super) fn install(cx: &mut App) {
    let app = cx.to_async();
    APP.with_borrow_mut(|slot| *slot = Some(app));
}

fn with_first_window<R>(read: impl FnOnce(&gpui::Window) -> R) -> Option<R> {
    APP.with_borrow(|app| {
        let app = app.as_ref()?;
        app.update(|cx| {
            let window = cx.windows().into_iter().next()?;
            window.update(cx, |_, window, _| read(window)).ok()
        })
    })
}

/// Draws every window again, after an asset arrived. Does nothing before `run`.
pub(super) fn refresh_windows() {
    APP.with_borrow(|app| {
        if let Some(app) = app {
            app.update(|cx| cx.refresh_windows());
        }
    });
}

/// Cumulative successful newly drawn frame submissions of the Gallery window,
/// or `undefined` when the backend cannot report them.
#[wasm_bindgen]
pub fn gallery_successful_new_frames() -> Option<f64> {
    with_first_window(|window| window.submission_stats().successful_new_frames)
        .flatten()
        .map(|frames| frames as f64)
}

/// Actual VSync readback: `"enabled"`, `"disabled"` or `"unsupported"`.
#[wasm_bindgen]
pub fn gallery_vsync_status() -> Option<String> {
    with_first_window(|window| {
        match window.vsync_status() {
            VsyncStatus::Enabled => "enabled",
            VsyncStatus::Disabled => "disabled",
            VsyncStatus::Unsupported => "unsupported",
        }
        .to_owned()
    })
}
