//! Color for gpui-kit-material, independent of GPUI.
//!
//! - [`Color`]: an sRGB RGBA value with the blend the kit uses for state
//!   layers, containers and animated palette changes.
//! - [`palette`]: the built-in source palettes and the serializable
//!   light/dark [`ThemePreferences`].
//! - [`scheme`]: the semantic [`ColorRole`]s components paint with, and the
//!   derivation of a [`Scheme`] from a palette.
//! - [`themes`]: the 20 md3.io reference presets, with explicit light/dark mode
//!   and standard/medium/high contrast. Values come from the reference CSS;
//!   these tables do not implement dynamic seed-color generation.
//!
//! Keeping this crate free of GPUI lets the scheme rules be tested on their
//! own, and leaves room for Material's dynamic color (HCT tonal palettes)
//! to arrive here without touching components.
mod color;
pub mod palette;
pub mod scheme;
pub mod themes;

pub use color::Color;
pub use palette::{Appearance, Palette, PaletteId, ThemePreferences};
pub use scheme::{ColorRole, Scheme};
