//! Material dynamic color: HCT, tonal palettes and seed-generated schemes.
//!
//! A port of Google's [material-color-utilities] at commit
//! `bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3`, licensed under Apache-2.0
//! (`licenses/material-color-utilities.txt`). Each file keeps its notice and
//! states that it was modified.
//!
//! [material-color-utilities]: https://github.com/material-foundation/material-color-utilities
pub mod blend;
pub mod contrast;
mod dislike;
mod hct;
mod palette;
mod solver;
mod temperature;
pub mod utils;

pub use dislike::{fix_if_disliked, is_disliked};
pub use hct::{Cam16, Hct, ViewingConditions};
pub use palette::TonalPalette;
pub use temperature::{TemperatureCache, raw_temperature};
