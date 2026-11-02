//! Browser entrypoint; the component examples are shared with the desktop gallery.
#[cfg(target_family = "wasm")]
extern crate gpui_kit as gpui;

#[cfg(target_family = "wasm")]
mod frame_counters;
#[cfg(target_family = "wasm")]
mod motion_preference;

/// Supplies one symbol asset by its file name. The page downloads the styles the
/// Gallery draws instead of embedding the 52 MB pack of every style in the module.
/// Icons draw empty until then, and a call after `run` redraws the window.
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn provide_symbol_asset(file_name: &str, bytes: Vec<u8>) -> Result<(), wasm_bindgen::JsError> {
    named_asset(file_name)?
        .provide(bytes)
        .map_err(|error| wasm_bindgen::JsError::new(&format!("{error:?}")))?;
    settle_symbol_asset(file_name)
}

/// The page gave up on a symbol asset. Icons of its style stop waiting for it and draw
/// nothing; the styles that arrived are not held back by it.
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn symbol_asset_failed(file_name: &str) -> Result<(), wasm_bindgen::JsError> {
    settle_symbol_asset(file_name)
}

#[cfg(target_family = "wasm")]
fn named_asset(file_name: &str) -> Result<gpui_kit_material::icons::Asset, wasm_bindgen::JsError> {
    use gpui_kit_material::icons::Asset;
    Asset::ALL
        .into_iter()
        .find(|asset| asset.file_name() == file_name)
        .ok_or_else(|| wasm_bindgen::JsError::new(&format!("unknown asset {file_name}")))
}

#[cfg(target_family = "wasm")]
fn settle_symbol_asset(file_name: &str) -> Result<(), wasm_bindgen::JsError> {
    if let gpui_kit_material::icons::Asset::Font(style) = named_asset(file_name)? {
        gpui_kit_material::icons::set_style_pending(style, false);
    }
    frame_counters::refresh_windows();
    Ok(())
}

/// Until the font of a style the Gallery draws has arrived, its icons draw a
/// placeholder: on a phone the download outlasts several page visits.
#[cfg(target_family = "wasm")]
fn await_drawn_symbols() {
    use gpui_kit_material::icons::{Asset, Style, set_style_pending};
    for style in [Style::Outlined, Style::Rounded] {
        set_style_pending(style, !Asset::Font(style).is_available());
    }
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run() {
    gpui_kit::platform::web_init();
    await_drawn_symbols();
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
    gpui_kit_material_gallery::run(application, |cx| {
        motion_preference::install(cx);
        frame_counters::install(cx);
    });
}
