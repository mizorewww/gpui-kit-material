// Copyright 2021 Google LLC
// SPDX-License-Identifier: Apache-2.0
// Ported to Rust from material-color-utilities
// (typescript/palettes/tonal_palette.ts), modified for this crate: tones are
// computed on request rather than cached in the palette, which keeps it `Copy`.
// See licenses/material-color-utilities.txt.

//! Tonal palettes: colors of constant hue and chroma at varying tone.
use super::hct::Hct;

/// A hue and chroma, from which any tone can be requested.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TonalPalette {
    hue: f64,
    chroma: f64,
    key_color: Hct,
}

impl TonalPalette {
    /// The hue and chroma of an ARGB color.
    pub fn from_argb(argb: u32) -> Self {
        Self::from_hct(Hct::from_argb(argb))
    }

    /// The hue and chroma of an HCT color, which is also the key color.
    pub fn from_hct(hct: Hct) -> Self {
        Self {
            hue: hct.hue(),
            chroma: hct.chroma(),
            key_color: hct,
        }
    }

    /// A hue and chroma; the key color is searched for.
    pub fn from_hue_and_chroma(hue: f64, chroma: f64) -> Self {
        Self {
            hue,
            chroma,
            key_color: key_color(hue, chroma),
        }
    }

    /// The palette's hue.
    pub fn hue(&self) -> f64 {
        self.hue
    }

    /// The palette's chroma.
    pub fn chroma(&self) -> f64 {
        self.chroma
    }

    /// The color that represents the palette: the tone closest to T50 that
    /// reaches its chroma.
    pub fn key_color(&self) -> Hct {
        self.key_color
    }

    /// The ARGB color at `tone`, 0 to 100.
    pub fn tone(&self, tone: f64) -> u32 {
        // Upstream: T99 on yellow hues is the average of T98 and T100.
        if tone == 99. && Hct::is_yellow(self.hue) {
            average_argb(self.tone(98.), self.tone(100.))
        } else {
            Hct::from_hct(self.hue, self.chroma, tone).argb()
        }
    }

    /// The HCT color at `tone`.
    pub fn hct(&self, tone: f64) -> Hct {
        Hct::from_argb(self.tone(tone))
    }
}

fn average_argb(a: u32, b: u32) -> u32 {
    let channel = |shift: u32| {
        let sum = ((a >> shift) & 0xff) + ((b >> shift) & 0xff);
        // Math.round((x + y) / 2): halves round up.
        sum.div_ceil(2) << shift
    };
    0xff00_0000 | channel(16) | channel(8) | channel(0)
}

/// The first tone, starting from T50 where the most chroma is available on
/// average, that matches the hue and chroma.
fn key_color(hue: f64, requested_chroma: f64) -> Hct {
    const MAX_CHROMA: f64 = 200.;
    const EPSILON: f64 = 0.01;
    const PIVOT_TONE: i32 = 50;
    // Each tone's maximum chroma is needed up to three times per step, and a
    // solve is the expensive part; 101 tones fit on the stack.
    let mut cache = [f64::NAN; 102];
    let mut max_chroma = |tone: i32| {
        let slot = &mut cache[tone.clamp(0, 101) as usize];
        if slot.is_nan() {
            *slot = Hct::from_hct(hue, MAX_CHROMA, f64::from(tone)).chroma();
        }
        *slot
    };
    let (mut lower, mut upper) = (0, 100);
    while lower < upper {
        let mid = (lower + upper) / 2;
        let is_ascending = max_chroma(mid) < max_chroma(mid + 1);
        if max_chroma(mid) >= requested_chroma - EPSILON {
            // Either side holds an answer: search the one nearer the pivot.
            if (lower - PIVOT_TONE).abs() < (upper - PIVOT_TONE).abs() {
                upper = mid;
            } else {
                if lower == mid {
                    return Hct::from_hct(hue, requested_chroma, f64::from(lower));
                }
                lower = mid;
            }
        } else if is_ascending {
            // Not enough chroma here: follow the direction to the peak.
            lower = mid + 1;
        } else {
            // Keep mid for a potential peak.
            upper = mid;
        }
    }
    Hct::from_hct(hue, requested_chroma, f64::from(lower))
}
