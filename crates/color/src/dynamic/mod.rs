//! Material dynamic color: HCT, tonal palettes and seed-generated schemes.
//!
//! A port of Google's [material-color-utilities] at commit
//! `bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3`, licensed under Apache-2.0
//! (`licenses/material-color-utilities.txt`). Each file keeps its notice and
//! states that it was modified.
//!
//! Not ported: the image quantizers and score (schemes start from a known
//! seed), the watch platform and the CMF variant, the deprecated `CorePalette`
//! and `Scheme` classes, and multiple source colors.
//!
//! [material-color-utilities]: https://github.com/material-foundation/material-color-utilities
pub mod blend;
mod color;
pub mod contrast;
mod dislike;
mod hct;
mod palette;
mod scheme;
mod seed;
mod solver;
mod spec_2021;
mod spec_2025;
mod temperature;
pub mod utils;

pub use color::{ContrastCurve, DynamicRole};
pub use dislike::{fix_if_disliked, is_disliked};
pub use hct::{Cam16, Hct, ViewingConditions};
pub use palette::TonalPalette;
pub use scheme::{DynamicScheme, SpecVersion, Variant};
pub use seed::SeedScheme;
pub use temperature::{TemperatureCache, raw_temperature};
