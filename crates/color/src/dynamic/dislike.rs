// Copyright 2023 Google LLC
// SPDX-License-Identifier: Apache-2.0
// Ported to Rust from material-color-utilities (typescript/dislike), modified
// for this crate. See licenses/material-color-utilities.txt.

//! Universally disliked colors: dark yellow-greens (Palmer and Schloss, 2010).
use super::hct::Hct;

/// Whether a color is a non-neutral dark yellow-green.
pub fn is_disliked(hct: Hct) -> bool {
    let hue = hct.hue().round();
    (90. ..=111.).contains(&hue) && hct.chroma().round() > 16. && hct.tone().round() < 65.
}

/// A disliked color lightened to tone 70; any other color unchanged.
pub fn fix_if_disliked(hct: Hct) -> Hct {
    if is_disliked(hct) {
        Hct::from_hct(hct.hue(), hct.chroma(), 70.)
    } else {
        hct
    }
}
