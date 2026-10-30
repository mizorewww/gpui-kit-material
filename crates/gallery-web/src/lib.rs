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

/// Supplies one symbol asset by its file name. The page downloads the styles the
/// Gallery draws instead of embedding the 52 MB pack of every style in the module.
/// Icons draw empty until then, and a call after `run` redraws the window.
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn provide_symbol_asset(file_name: &str, bytes: Vec<u8>) -> Result<(), wasm_bindgen::JsError> {
    use gpui_kit_material::icons::Asset;
    Asset::ALL
        .into_iter()
        .find(|asset| asset.file_name() == file_name)
        .ok_or_else(|| wasm_bindgen::JsError::new(&format!("unknown asset {file_name}")))?
        .provide(bytes)
        .map_err(|error| wasm_bindgen::JsError::new(&format!("{error:?}")))?;
    frame_counters::refresh_windows();
    Ok(())
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run() {
    gpui_kit::platform::web_init();
    // The Gallery's fonts have no CJK, and a phone's IME types it: draw those
    // characters with the browser's fonts, as it does for emoji by default.
    let platform = std::rc::Rc::new(
        gpui_kit::web::WebPlatform::new_with_backend_and_font_fallback(
            false,
            gpui_kit::web::WebBackendPreference::Auto,
            gpui_kit::web::CanvasFontFallback::EmojiAndCjk,
        ),
    );
    let http_client = std::sync::Arc::new(platform.fetch_http_client());
    let application = gpui::Application::with_platform(platform).with_http_client(http_client);
    gallery::run(application, |cx| {
        motion_preference::install(cx);
        frame_counters::install(cx);
    });
}
