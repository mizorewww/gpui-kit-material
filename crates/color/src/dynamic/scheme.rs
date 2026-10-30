// Copyright 2022 Google LLC
// SPDX-License-Identifier: Apache-2.0
// Ported to Rust from material-color-utilities
// (typescript/dynamiccolor/dynamic_scheme.ts and variant.ts), modified for this
// crate: only the phone platform, which is what desktop and web use; no CMF
// variant; a single source color. See licenses/material-color-utilities.txt.

//! A seed, a style, a mode and a contrast level, resolved to tonal palettes.
use super::color::{self, DynamicRole};
use super::hct::Hct;
use super::palette::TonalPalette;
use super::temperature::TemperatureCache;
use super::utils::sanitize_degrees_double;
use super::{dislike, utils::clamp_double};

/// The style of a generated scheme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum Variant {
    /// Grayscale; the seed only gives a trace of hue to nothing.
    Monochrome,
    /// Nearly grayscale, with a hint of the seed.
    Neutral,
    /// Pastel tokens at low chroma: Material's default.
    #[default]
    TonalSpot,
    /// Maximum chroma of the seed's hue.
    Vibrant,
    /// Hues rotated away from the seed for a playful result.
    Expressive,
    /// Places the seed in the primary container, tertiary its complement.
    Fidelity,
    /// Places the seed in the primary container, tertiary an analogue.
    Content,
    /// A playful theme in which the seed's hue does not appear in surfaces.
    Rainbow,
    /// A playful theme in which the seed's hue does not appear in accents.
    FruitSalad,
}

impl Variant {
    /// Every variant, in upstream's order.
    pub const ALL: [Self; 9] = [
        Self::Monochrome,
        Self::Neutral,
        Self::TonalSpot,
        Self::Vibrant,
        Self::Expressive,
        Self::Fidelity,
        Self::Content,
        Self::Rainbow,
        Self::FruitSalad,
    ];

    /// Whether the 2025 color spec defines this variant. Upstream resolves
    /// the others with the 2021 spec under any requested version.
    pub const fn has_2025_spec(self) -> bool {
        matches!(
            self,
            Self::Expressive | Self::Vibrant | Self::TonalSpot | Self::Neutral
        )
    }
}

/// The version of Material's color spec.
///
/// Upstream's 2026 spec only defines its CMF variant and resolves every variant
/// here as 2025, so it is not a separate version in this port.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SpecVersion {
    /// The original Material You spec: upstream's default.
    #[default]
    #[cfg_attr(feature = "serde", serde(rename = "2021"))]
    Spec2021,
    /// The Material 3 Expressive spec.
    #[cfg_attr(feature = "serde", serde(rename = "2025"))]
    Spec2025,
}

/// The state a [`DynamicRole`] resolves against.
///
/// ```
/// use gpui_kit_material_color::dynamic::{
///     DynamicRole, DynamicScheme, Hct, SpecVersion, Variant,
/// };
///
/// let scheme = DynamicScheme::new(
///     Hct::from_argb(0xff6750a4),
///     Variant::TonalSpot,
///     false,
///     0.0,
///     SpecVersion::Spec2021,
/// );
/// assert_eq!(scheme.argb(DynamicRole::Primary), 0xff65558f);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DynamicScheme {
    /// The seed.
    pub source: Hct,
    /// The style.
    pub variant: Variant,
    /// -1 is minimum contrast, 0 the design as specified, 0.5 medium, 1 maximum.
    pub contrast_level: f64,
    /// Dark mode.
    pub is_dark: bool,
    /// The spec actually in force, after [`Variant::has_2025_spec`].
    pub spec: SpecVersion,
    /// Usually colorful.
    pub primary_palette: TonalPalette,
    /// Usually less colorful.
    pub secondary_palette: TonalPalette,
    /// Usually a different hue from primary.
    pub tertiary_palette: TonalPalette,
    /// Usually not colorful: surfaces.
    pub neutral_palette: TonalPalette,
    /// Slightly more colorful than neutral: outlines and variants.
    pub neutral_variant_palette: TonalPalette,
    /// Reddish.
    pub error_palette: TonalPalette,
}

impl DynamicScheme {
    /// Resolve the palettes of a seed. A non-finite contrast level counts as 0;
    /// others are limited to -1..=1.
    pub fn new(
        source: Hct,
        variant: Variant,
        is_dark: bool,
        contrast_level: f64,
        spec: SpecVersion,
    ) -> Self {
        let spec = if variant.has_2025_spec() {
            spec
        } else {
            SpecVersion::Spec2021
        };
        let contrast_level = if contrast_level.is_finite() {
            clamp_double(-1., 1., contrast_level)
        } else {
            0.
        };
        let palettes = match spec {
            SpecVersion::Spec2021 => palettes_2021(variant, source),
            SpecVersion::Spec2025 => palettes_2025(variant, source, is_dark),
        };
        Self {
            source,
            variant,
            contrast_level,
            is_dark,
            spec,
            primary_palette: palettes[0],
            secondary_palette: palettes[1],
            tertiary_palette: palettes[2],
            neutral_palette: palettes[3],
            neutral_variant_palette: palettes[4],
            error_palette: palettes[5],
        }
    }

    /// The hue in `hues` at the range of `breakpoints` holding the source's
    /// hue, or the source's hue when no range does.
    pub fn piecewise_hue(source: Hct, breakpoints: &[f64], hues: &[f64]) -> f64 {
        let size = breakpoints.len().saturating_sub(1).min(hues.len());
        for i in 0..size {
            if source.hue() >= breakpoints[i] && source.hue() < breakpoints[i + 1] {
                return sanitize_degrees_double(hues[i]);
            }
        }
        source.hue()
    }

    /// The source's hue rotated by the entry of `rotations` at the range of
    /// `breakpoints` holding it.
    pub fn rotated_hue(source: Hct, breakpoints: &[f64], rotations: &[f64]) -> f64 {
        let rotation = if breakpoints.len().saturating_sub(1).min(rotations.len()) == 0 {
            0.
        } else {
            Self::piecewise_hue(source, breakpoints, rotations)
        };
        sanitize_degrees_double(source.hue() + rotation)
    }

    /// The ARGB color of a role.
    pub fn argb(&self, role: DynamicRole) -> u32 {
        self.hct(role).argb()
    }

    /// The HCT color of a role.
    pub fn hct(&self, role: DynamicRole) -> Hct {
        color::hct(self, role)
    }

    /// The tone of a role. This resolves contrast constraints; for the color
    /// use [`Self::hct`].
    pub fn tone(&self, role: DynamicRole) -> f64 {
        color::tone(self, role)
    }
}

fn hue_chroma(hue: f64, chroma: f64) -> TonalPalette {
    TonalPalette::from_hue_and_chroma(hue, chroma)
}

const DEFAULT_ERROR: (f64, f64) = (25., 84.);

fn palettes_2021(variant: Variant, source: Hct) -> [TonalPalette; 6] {
    [
        primary_2021(variant, source),
        secondary_2021(variant, source),
        tertiary_2021(variant, source),
        neutral_2021(variant, source),
        neutral_variant_2021(variant, source),
        hue_chroma(DEFAULT_ERROR.0, DEFAULT_ERROR.1),
    ]
}

fn primary_2021(variant: Variant, source: Hct) -> TonalPalette {
    let hue = source.hue();
    match variant {
        Variant::Content | Variant::Fidelity => hue_chroma(hue, source.chroma()),
        Variant::FruitSalad => hue_chroma(sanitize_degrees_double(hue - 50.), 48.),
        Variant::Monochrome => hue_chroma(hue, 0.),
        Variant::Neutral => hue_chroma(hue, 12.),
        Variant::Rainbow => hue_chroma(hue, 48.),
        Variant::TonalSpot => hue_chroma(hue, 36.),
        Variant::Expressive => hue_chroma(sanitize_degrees_double(hue + 240.), 40.),
        Variant::Vibrant => hue_chroma(hue, 200.),
    }
}

fn secondary_2021(variant: Variant, source: Hct) -> TonalPalette {
    let hue = source.hue();
    match variant {
        Variant::Content | Variant::Fidelity => {
            hue_chroma(hue, (source.chroma() - 32.).max(source.chroma() * 0.5))
        }
        Variant::FruitSalad => hue_chroma(sanitize_degrees_double(hue - 50.), 36.),
        Variant::Monochrome => hue_chroma(hue, 0.),
        Variant::Neutral => hue_chroma(hue, 8.),
        Variant::Rainbow | Variant::TonalSpot => hue_chroma(hue, 16.),
        Variant::Expressive => hue_chroma(
            DynamicScheme::rotated_hue(
                source,
                &[0., 21., 51., 121., 151., 191., 271., 321., 360.],
                &[45., 95., 45., 20., 45., 90., 45., 45., 45.],
            ),
            24.,
        ),
        Variant::Vibrant => hue_chroma(
            DynamicScheme::rotated_hue(
                source,
                &[0., 41., 61., 101., 131., 181., 251., 301., 360.],
                &[18., 15., 10., 12., 15., 18., 15., 12., 12.],
            ),
            24.,
        ),
    }
}

fn tertiary_2021(variant: Variant, source: Hct) -> TonalPalette {
    let hue = source.hue();
    match variant {
        Variant::Content => TonalPalette::from_hct(dislike::fix_if_disliked(
            TemperatureCache::new(source).analogous(3, 6)[2],
        )),
        Variant::Fidelity => TonalPalette::from_hct(dislike::fix_if_disliked(
            TemperatureCache::new(source).complement(),
        )),
        Variant::FruitSalad => hue_chroma(hue, 36.),
        Variant::Monochrome => hue_chroma(hue, 0.),
        Variant::Neutral => hue_chroma(hue, 16.),
        Variant::Rainbow | Variant::TonalSpot => {
            hue_chroma(sanitize_degrees_double(hue + 60.), 24.)
        }
        Variant::Expressive => hue_chroma(
            DynamicScheme::rotated_hue(
                source,
                &[0., 21., 51., 121., 151., 191., 271., 321., 360.],
                &[120., 120., 20., 45., 20., 15., 20., 120., 120.],
            ),
            32.,
        ),
        Variant::Vibrant => hue_chroma(
            DynamicScheme::rotated_hue(
                source,
                &[0., 41., 61., 101., 131., 181., 251., 301., 360.],
                &[35., 30., 20., 25., 30., 35., 30., 25., 25.],
            ),
            32.,
        ),
    }
}

fn neutral_2021(variant: Variant, source: Hct) -> TonalPalette {
    let hue = source.hue();
    match variant {
        Variant::Content | Variant::Fidelity => hue_chroma(hue, source.chroma() / 8.),
        Variant::FruitSalad => hue_chroma(hue, 10.),
        Variant::Monochrome | Variant::Rainbow => hue_chroma(hue, 0.),
        Variant::Neutral => hue_chroma(hue, 2.),
        Variant::TonalSpot => hue_chroma(hue, 6.),
        Variant::Expressive => hue_chroma(sanitize_degrees_double(hue + 15.), 8.),
        Variant::Vibrant => hue_chroma(hue, 10.),
    }
}

fn neutral_variant_2021(variant: Variant, source: Hct) -> TonalPalette {
    let hue = source.hue();
    match variant {
        Variant::Content | Variant::Fidelity => hue_chroma(hue, source.chroma() / 8. + 4.),
        Variant::FruitSalad => hue_chroma(hue, 16.),
        Variant::Monochrome | Variant::Rainbow => hue_chroma(hue, 0.),
        Variant::Neutral => hue_chroma(hue, 2.),
        Variant::TonalSpot => hue_chroma(hue, 8.),
        Variant::Expressive => hue_chroma(sanitize_degrees_double(hue + 15.), 12.),
        Variant::Vibrant => hue_chroma(hue, 12.),
    }
}

fn expressive_neutral_hue(source: Hct) -> f64 {
    DynamicScheme::rotated_hue(
        source,
        &[0., 71., 124., 253., 278., 300., 360.],
        &[10., 0., 10., 0., 10., 0.],
    )
}

fn expressive_neutral_chroma(source: Hct, is_dark: bool) -> f64 {
    if is_dark {
        if Hct::is_yellow(expressive_neutral_hue(source)) {
            6.
        } else {
            14.
        }
    } else {
        18.
    }
}

fn vibrant_neutral_hue(source: Hct) -> f64 {
    DynamicScheme::rotated_hue(
        source,
        &[0., 38., 105., 140., 333., 360.],
        &[-14., 10., -14., 10., -14.],
    )
}

const VIBRANT_NEUTRAL_CHROMA: f64 = 28.;

// Only called for the four variants of `Variant::has_2025_spec`; the others
// resolve with `palettes_2021`.
fn palettes_2025(variant: Variant, source: Hct, is_dark: bool) -> [TonalPalette; 6] {
    let hue = source.hue();
    let error_hue = DynamicScheme::piecewise_hue(
        source,
        &[0., 3., 13., 23., 33., 43., 153., 273., 360.],
        &[12., 22., 32., 12., 22., 32., 22., 12.],
    );
    match variant {
        Variant::Neutral => {
            let blue = Hct::is_blue(hue);
            [
                hue_chroma(hue, if blue { 12. } else { 8. }),
                hue_chroma(hue, if blue { 6. } else { 4. }),
                hue_chroma(
                    DynamicScheme::rotated_hue(
                        source,
                        &[0., 38., 105., 161., 204., 278., 333., 360.],
                        &[-32., 26., 10., -39., 24., -15., -32.],
                    ),
                    20.,
                ),
                hue_chroma(hue, 1.4),
                hue_chroma(hue, 1.4 * 2.2),
                hue_chroma(error_hue, 50.),
            ]
        }
        Variant::TonalSpot => [
            hue_chroma(hue, if is_dark { 26. } else { 32. }),
            hue_chroma(hue, 16.),
            hue_chroma(
                DynamicScheme::rotated_hue(
                    source,
                    &[0., 20., 71., 161., 333., 360.],
                    &[-40., 48., -32., 40., -32.],
                ),
                28.,
            ),
            hue_chroma(hue, 5.),
            hue_chroma(hue, 5. * 1.7),
            hue_chroma(error_hue, 60.),
        ],
        Variant::Expressive => {
            let neutral_hue = expressive_neutral_hue(source);
            let neutral_chroma = expressive_neutral_chroma(source, is_dark);
            [
                hue_chroma(hue, if is_dark { 36. } else { 48. }),
                hue_chroma(
                    DynamicScheme::rotated_hue(
                        source,
                        &[0., 105., 140., 204., 253., 278., 300., 333., 360.],
                        &[-160., 155., -100., 96., -96., -156., -165., -160.],
                    ),
                    if is_dark { 16. } else { 24. },
                ),
                hue_chroma(
                    DynamicScheme::rotated_hue(
                        source,
                        &[0., 105., 140., 204., 253., 278., 300., 333., 360.],
                        &[-165., 160., -105., 101., -101., -160., -170., -165.],
                    ),
                    48.,
                ),
                hue_chroma(neutral_hue, neutral_chroma),
                hue_chroma(
                    neutral_hue,
                    neutral_chroma
                        * if (105. ..125.).contains(&neutral_hue) {
                            1.6
                        } else {
                            2.3
                        },
                ),
                hue_chroma(error_hue, 64.),
            ]
        }
        Variant::Vibrant => {
            let neutral_hue = vibrant_neutral_hue(source);
            [
                hue_chroma(hue, 74.),
                hue_chroma(neutral_hue, 56.),
                hue_chroma(
                    DynamicScheme::rotated_hue(
                        source,
                        &[0., 38., 71., 105., 140., 161., 253., 333., 360.],
                        &[-72., 35., 24., -24., 62., 50., 62., -72.],
                    ),
                    56.,
                ),
                hue_chroma(neutral_hue, VIBRANT_NEUTRAL_CHROMA),
                hue_chroma(neutral_hue, VIBRANT_NEUTRAL_CHROMA * 1.29),
                hue_chroma(error_hue, 80.),
            ]
        }
        _ => palettes_2021(variant, source),
    }
}
