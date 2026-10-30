// Copyright 2021 Google LLC
// SPDX-License-Identifier: Apache-2.0
// Ported to Rust from material-color-utilities (typescript/blend), modified for
// this crate. See licenses/material-color-utilities.txt.

//! Blending in HCT and CAM16.
use super::hct::{Cam16, Hct};
use super::utils::{
    difference_degrees, lstar_from_argb, rotation_direction, sanitize_degrees_double,
};

/// `design_color`'s hue shifted toward `source_color`'s: at most half their
/// distance and at most 15 degrees, so it stays recognizable.
pub fn harmonize(design_color: u32, source_color: u32) -> u32 {
    let from = Hct::from_argb(design_color);
    let to = Hct::from_argb(source_color);
    let rotation = (difference_degrees(from.hue(), to.hue()) * 0.5).min(15.);
    let hue =
        sanitize_degrees_double(from.hue() + rotation * rotation_direction(from.hue(), to.hue()));
    Hct::from_hct(hue, from.chroma(), from.tone()).argb()
}

/// `from` with its hue blended toward `to`; chroma and tone are kept.
pub fn hct_hue(from: u32, to: u32, amount: f64) -> u32 {
    let blended = Cam16::from_argb(cam16_ucs(from, to, amount));
    Hct::from_hct(
        blended.hue,
        Cam16::from_argb(from).chroma,
        lstar_from_argb(from),
    )
    .argb()
}

/// `from` blended toward `to` in CAM16-UCS; hue, chroma and tone all change.
pub fn cam16_ucs(from: u32, to: u32, amount: f64) -> u32 {
    let from = Cam16::from_argb(from);
    let to = Cam16::from_argb(to);
    Cam16::from_ucs(
        from.jstar + (to.jstar - from.jstar) * amount,
        from.astar + (to.astar - from.astar) * amount,
        from.bstar + (to.bstar - from.bstar) * amount,
    )
    .to_argb()
}
