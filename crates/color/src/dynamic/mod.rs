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
//!
//! Without the `dynamic` feature only `blend` and what it needs compile, privately, for
//! [`crate::scheme::TintFamily`]'s harmonized hues.
#![cfg_attr(not(feature = "dynamic"), allow(dead_code))]

pub mod blend;
#[cfg(feature = "dynamic")]
mod color;
#[cfg(feature = "dynamic")]
pub mod contrast;
#[cfg(feature = "dynamic")]
mod dislike;
mod hct;
#[cfg(feature = "dynamic")]
mod palette;
#[cfg(feature = "dynamic")]
mod scheme;
#[cfg(feature = "dynamic")]
mod seed;
mod solver;
#[cfg(feature = "dynamic")]
mod spec_2021;
#[cfg(feature = "dynamic")]
mod spec_2025;
#[cfg(feature = "dynamic")]
mod temperature;
pub mod utils;

#[cfg(feature = "dynamic")]
pub use color::{ContrastCurve, DynamicRole};
#[cfg(feature = "dynamic")]
pub use dislike::{fix_if_disliked, is_disliked};
#[cfg(feature = "dynamic")]
pub use hct::{Cam16, Hct, ViewingConditions};
#[cfg(feature = "dynamic")]
pub use palette::TonalPalette;
#[cfg(feature = "dynamic")]
pub use scheme::{DynamicScheme, SpecVersion, Variant};
#[cfg(feature = "dynamic")]
pub use seed::SeedScheme;
#[cfg(feature = "dynamic")]
pub use temperature::{TemperatureCache, raw_temperature};

/// `argb`'s hue and chroma at `tone` (L*), with chroma reduced where that
/// tone cannot hold it. Not part of the upstream port: the scheme uses it to
/// move a palette color to a readable tone without changing its hue.
pub(crate) fn at_tone(argb: u32, tone: f64) -> u32 {
    let hct = hct::Hct::from_argb(argb);
    hct::Hct::from_hct(hct.hue(), hct.chroma(), tone).argb()
}

/// `argb`'s hue at `tone` (L*), with its chroma up to `max_chroma` and reduced
/// further where that tone cannot hold it. Not part of the upstream port: the
/// scheme uses it for status containers.
pub(crate) fn at_tone_within(argb: u32, max_chroma: f64, tone: f64) -> u32 {
    let hct = hct::Hct::from_argb(argb);
    hct::Hct::from_hct(hct.hue(), hct.chroma().min(max_chroma), tone).argb()
}
