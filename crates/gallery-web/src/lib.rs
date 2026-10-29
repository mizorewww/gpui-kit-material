//! Browser entrypoint; the component examples are shared with the desktop gallery.
#[cfg(target_family = "wasm")]
#[path = "../../material/examples/gallery/mod.rs"]
mod gallery;

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run() {
    gpui_platform::web_init();
    gallery::run(gpui_platform::single_threaded_web());
}
