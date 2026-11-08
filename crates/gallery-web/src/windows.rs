//! The page's handle on the Gallery's application, for `www/main.js`. Call these
//! from ordinary browser tasks, never from inside a GPUI callback (App is
//! borrowed there).
use gpui::{App, AsyncApp};
use std::cell::RefCell;
use wasm_bindgen::prelude::wasm_bindgen;

thread_local! {
    static APP: RefCell<Option<AsyncApp>> = const { RefCell::new(None) };
}

pub(super) fn install(cx: &mut App) {
    let app = cx.to_async();
    APP.with_borrow_mut(|slot| *slot = Some(app));
}

/// Draws every window again, after an asset arrived. Does nothing before `run`.
pub(super) fn refresh_windows() {
    APP.with_borrow(|app| {
        if let Some(app) = app {
            app.update(|cx| cx.refresh_windows());
        }
    });
}

/// Whether the Gallery's window is open. GPUI draws a window as it opens it,
/// and the browser shows that frame at its next animation frame.
#[wasm_bindgen]
pub fn gallery_window_open() -> bool {
    APP.with_borrow(|app| {
        app.as_ref()
            .is_some_and(|app| app.update(|cx| !cx.windows().is_empty()))
    })
}
