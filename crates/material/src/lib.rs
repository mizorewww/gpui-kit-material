//! Material 3 for GPUI.
//!
//! The kit is layered so each layer can be replaced without touching the
//! ones above it:
//!
//! | Layer | Module | Holds |
//! | --- | --- | --- |
//! | Color | [`color`] (`gpui-kit-material-color`) | palettes, roles, scheme derivation |
//! | Theme | [`theme`] | the painted scheme, palette transitions, [`theme::color`] |
//! | Tokens | [`tokens`] | shape, type scale, layout metrics, elevation, state layers |
//! | Motion | [`motion`] | M3 springs, entrances, retargetable values |
//! | Icons | [`icons`] | the kit's own glyphs and their [`icons::Assets`] |
//! | Components | [`components`] | Material components on GPUI Base behavior |
//!
//! Behavior (focus, keyboard, accessibility roles, transitions) comes from
//! [`gpui_base`]; this kit only decides how Material looks and moves.
//!
//! Call `gpui_base::init` and then [`init`] once at startup, give the application [`icons::Assets`]
//! (or chain it behind your own asset source), and call
//! [`theme::apply`] at the start of each root render.
pub use gpui_kit_material_color as color;

pub mod components;
pub mod icons;
pub mod motion;
pub mod theme;
pub mod tokens;

pub use theme::color;

/// Register the kit's key bindings. Call once, before opening windows.
pub fn init(cx: &mut gpui::App) {
    components::init(cx);
}

/// The roles, tokens and components most pages need.
pub mod prelude {
    pub use crate::color::themes::{Contrast, ThemeId, ThemeMode};
    pub use crate::color::{Appearance, ColorRole, PaletteId, ThemePreferences};
    pub use crate::components::*;
    pub use crate::theme::color;
    pub use crate::tokens::{layout::*, shape::*, typography::*};
}
