//! Material 3 adapters for Longbridge GPUI Kit.
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
//! | Icons | [`icons`] | Google Material Symbols through [`icons::Assets`] |
//! | Components | [`components`] | Material presentation over Kit components and Base parts |
//!
//! Reuse [`gpui_kit::component`] controls where their presentation hooks meet the
//! Material contract; [`gpui_base`] supplies behavior for uncovered parts. This
//! kit owns the Material appearance and motion, not another generic UI runtime.
//!
//! Call `gpui_kit::init` and then [`init`] once at startup. Use Kit's
//! `open_window` for the single Root, register [`icons::Assets`], and initialize [`theme::init_component_theme`] before Root's first
//! render. Advance [`theme::advance_component_theme`] from input events and
//! pre-frame callbacks. Base-only consumers can paint with
//! [`theme::apply_scheme`] at the start of each root render.
extern crate gpui_kit as gpui;

pub use gpui_kit_material_color as color;

pub mod components;
pub mod icons;
pub mod motion;
pub mod scope;
pub mod shapes;
pub mod theme;
pub mod tokens;

/// Register the kit's key bindings. Call once, before opening windows.
pub fn init(cx: &mut gpui::App) {
    icons::init(cx);
    components::init(cx);
}

/// The roles, tokens and components most pages need, including the
/// [`theme::color`] lookup. The crate root's `color` is the color module.
pub mod prelude {
    pub use crate::color::themes::{Contrast, ThemeId, ThemeMode};
    pub use crate::color::{Appearance, ColorRole, PaletteId, ThemePreferences};
    pub use crate::components::*;
    pub use crate::scope::{LayoutDirection, Scope, is_rtl};
    pub use crate::shapes::{MaterialShape, Morph, ShapeFrame, ShapeKind};
    pub use crate::theme::color;
    pub use crate::tokens::{layout::*, shape::*, typography::*};
}
