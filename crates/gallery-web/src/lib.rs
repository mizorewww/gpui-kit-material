//! Browser entrypoint; the component examples are shared with the desktop gallery.
#[cfg(target_family = "wasm")]
extern crate gpui_kit as gpui;

#[cfg(target_family = "wasm")]
mod frame_counters;
#[cfg(target_family = "wasm")]
#[path = "../../material/examples/gallery/mod.rs"]
mod gallery;
#[cfg(target_family = "wasm")]
mod motion_preference;

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run() {
    gpui_kit::platform::web_init();
    gallery::run(gpui_kit::platform::single_threaded_web(), |cx| {
        motion_preference::install(cx);
        frame_counters::install(cx);
    });
}
