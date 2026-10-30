// Copyright 2022 Google LLC
// SPDX-License-Identifier: Apache-2.0
// Ported to Rust from material-color-utilities (typescript/contrast), modified
// for this crate. See licenses/material-color-utilities.txt.

//! Contrast ratios between tones, and tones that reach a given ratio.
//!
//! The safe functions return `None` where upstream returns -1.
use super::utils::{clamp_double, lstar_from_y, y_from_lstar};

/// The contrast ratio of two tones, from 1 to 21. Tones are clamped to 0..=100.
pub fn ratio_of_tones(tone_a: f64, tone_b: f64) -> f64 {
    ratio_of_ys(
        y_from_lstar(clamp_double(0., 100., tone_a)),
        y_from_lstar(clamp_double(0., 100., tone_b)),
    )
}

/// The contrast ratio of two relative luminances.
pub fn ratio_of_ys(y1: f64, y2: f64) -> f64 {
    let lighter = if y1 > y2 { y1 } else { y2 };
    let darker = if lighter == y2 { y1 } else { y2 };
    (lighter + 5.) / (darker + 5.)
}

/// A tone at least as light as `tone` that reaches `ratio`, if one exists.
pub fn lighter(tone: f64, ratio: f64) -> Option<f64> {
    if !(0. ..=100.).contains(&tone) {
        return None;
    }
    let dark_y = y_from_lstar(tone);
    let light_y = ratio * (dark_y + 5.) - 5.;
    let real_contrast = ratio_of_ys(light_y, dark_y);
    if real_contrast < ratio && (real_contrast - ratio).abs() > 0.04 {
        return None;
    }
    // Gamut mapping needs a range on tone, so overshoot slightly.
    let value = lstar_from_y(light_y) + 0.4;
    (0. ..=100.).contains(&value).then_some(value)
}

/// A tone at least as dark as `tone` that reaches `ratio`, if one exists.
pub fn darker(tone: f64, ratio: f64) -> Option<f64> {
    if !(0. ..=100.).contains(&tone) {
        return None;
    }
    let light_y = y_from_lstar(tone);
    let dark_y = (light_y + 5.) / ratio - 5.;
    let real_contrast = ratio_of_ys(light_y, dark_y);
    if real_contrast < ratio && (real_contrast - ratio).abs() > 0.04 {
        return None;
    }
    let value = lstar_from_y(dark_y) - 0.4;
    (0. ..=100.).contains(&value).then_some(value)
}

/// [`lighter`], or 100 when `ratio` cannot be reached.
pub fn lighter_unsafe(tone: f64, ratio: f64) -> f64 {
    lighter(tone, ratio).unwrap_or(100.)
}

/// [`darker`], or 0 when `ratio` cannot be reached.
pub fn darker_unsafe(tone: f64, ratio: f64) -> f64 {
    darker(tone, ratio).unwrap_or(0.)
}
