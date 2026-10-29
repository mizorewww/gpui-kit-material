//! Color for gpui-kit-material, independent of GPUI.
//!
//! - [`Color`]: a linear RGBA value with the blend the kit uses for state
//!   layers, containers and animated palette changes.
//! - [`palette`]: the built-in source palettes and the serializable
//!   light/dark [`ThemePreferences`].
//! - [`scheme`]: the semantic [`ColorRole`]s components paint with, and the
//!   derivation of a [`Scheme`] from a palette.
//!
//! Keeping this crate free of GPUI lets the scheme rules be tested on their
//! own, and leaves room for Material's dynamic color (HCT tonal palettes)
//! to arrive here without touching components.
mod color;
pub mod palette;
pub mod scheme;

pub use color::Color;
pub use palette::{Appearance, Palette, PaletteId, ThemePreferences};
pub use scheme::{ColorRole, Scheme};
