//! The README's quick start, as a downstream application compiles it.
use gpui_kit_material::{
    color::themes::{Contrast, ThemeId, ThemeMode},
    icons::Assets,
    theme,
};

fn main() {
    gpui_kit::application()
        .with_assets(Assets::new())
        .run(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
            let scheme = ThemeId::Baseline.scheme(ThemeMode::Light, Contrast::Standard);
            theme::init_component_theme(scheme, 1.0, cx);
        });
}
