//! Material dynamic color: HCT, tonal palettes and seed-generated schemes.
//!
//! A port of Google's [material-color-utilities] at commit
//! `bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3`, licensed under Apache-2.0
//! (`licenses/material-color-utilities.txt`). Each file keeps its notice and
//! states that it was modified.
//!
//! [material-color-utilities]: https://github.com/material-foundation/material-color-utilities
mod hct;
mod solver;
pub mod utils;

pub use hct::{Cam16, Hct, ViewingConditions};
