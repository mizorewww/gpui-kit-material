//! Bridge the browser preference into GPUI before opening the gallery window.
use gpui::{App, Global};
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{MediaQueryList, MediaQueryListEvent};

struct MotionPreference {
    query: MediaQueryList,
    listener: Closure<dyn FnMut(MediaQueryListEvent)>,
}
impl Global for MotionPreference {}
impl Drop for MotionPreference {
    fn drop(&mut self) {
        let _ = self
            .query
            .remove_event_listener_with_callback("change", self.listener.as_ref().unchecked_ref());
    }
}

pub(super) fn install(cx: &mut App) {
    let query = web_sys::window()
        .expect("gallery requires a browser window")
        .match_media("(prefers-reduced-motion: reduce)")
        .expect("browser media queries must be available")
        .expect("browser must return a media query list");
    cx.set_reduce_motion(query.matches());
    let app = cx.to_async();
    let listener = Closure::wrap(Box::new(move |event: MediaQueryListEvent| {
        app.update(|cx| cx.set_reduce_motion(event.matches()));
    }) as Box<dyn FnMut(MediaQueryListEvent)>);
    query
        .add_event_listener_with_callback("change", listener.as_ref().unchecked_ref())
        .expect("failed to observe the browser motion preference");
    // The App owns the listener, and removes it before its Closure is dropped.
    cx.set_global(MotionPreference { query, listener });
}
