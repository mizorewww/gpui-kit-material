// Copyright 2025 Google LLC
// SPDX-License-Identifier: Apache-2.0
// Ported to Rust from material-color-utilities
// (typescript/dynamiccolor/color_spec_2025.ts), modified for this crate: the
// phone platform only. See licenses/material-color-utilities.txt.

//! The 2025 color spec. Roles it does not change come from the 2021 spec.
use super::color::DynamicRole::*;
use super::color::{ContrastCurve, Def, DeltaConstraint, DynamicRole, ToneDeltaPair, TonePolarity};
use super::hct::Hct;
use super::palette::TonalPalette;
use super::scheme::{DynamicScheme, Variant};
use super::spec_2021::{self, highest_surface};
use super::utils::clamp_double;

fn find_best_tone_for_chroma(hue: f64, chroma: f64, start: f64, by_decreasing: bool) -> f64 {
    let mut tone = start;
    let mut answer = tone;
    let mut best = Hct::from_hct(hue, chroma, answer);
    while best.chroma() < chroma {
        if !(0. ..=100.).contains(&tone) {
            break;
        }
        tone += if by_decreasing { -1. } else { 1. };
        let candidate = Hct::from_hct(hue, chroma, tone);
        if best.chroma() < candidate.chroma() {
            best = candidate;
            answer = tone;
        }
    }
    answer
}

/// The highest tone reaching the palette's chroma, limited to the bounds.
fn t_max_c_scaled(palette: TonalPalette, lower: f64, upper: f64, multiplier: f64) -> f64 {
    let answer =
        find_best_tone_for_chroma(palette.hue(), palette.chroma() * multiplier, 100., true);
    clamp_double(lower, upper, answer)
}

fn t_max_c(palette: TonalPalette, lower: f64, upper: f64) -> f64 {
    t_max_c_scaled(palette, lower, upper, 1.)
}

/// The lowest tone reaching the palette's chroma, limited to the bounds.
fn t_min_c(palette: TonalPalette, lower: f64, upper: f64) -> f64 {
    let answer = find_best_tone_for_chroma(palette.hue(), palette.chroma(), 0., false);
    clamp_double(lower, upper, answer)
}

fn get_curve(default_contrast: f64) -> Option<ContrastCurve> {
    let (low, normal, medium, high) = match default_contrast {
        1.5 => (1.5, 1.5, 3., 5.5),
        3. => (3., 3., 4.5, 7.),
        4.5 => (4.5, 4.5, 7., 11.),
        6. => (6., 6., 7., 11.),
        7. => (7., 7., 11., 21.),
        9. => (9., 9., 11., 21.),
        11. => (11., 11., 21., 21.),
        21. => (21., 21., 21., 21.),
        other => (other, other, 7., 21.),
    };
    Some(ContrastCurve::new(low, normal, medium, high))
}

fn farther(a: DynamicRole, b: DynamicRole, polarity: TonePolarity) -> ToneDeltaPair {
    ToneDeltaPair {
        a,
        b,
        delta: 5.,
        polarity,
        stay_together: true,
        constraint: DeltaConstraint::Farther,
    }
}

fn exact_darker(a: DynamicRole, b: DynamicRole) -> ToneDeltaPair {
    ToneDeltaPair {
        a,
        b,
        delta: 5.,
        polarity: TonePolarity::Darker,
        stay_together: true,
        constraint: DeltaConstraint::Exact,
    }
}

/// The on-surface family's chroma multiplier.
fn on_surface_multiplier(s: &DynamicScheme) -> f64 {
    match s.variant {
        Variant::Neutral => 2.2,
        Variant::TonalSpot => 1.7,
        Variant::Expressive => {
            if Hct::is_yellow(s.neutral_palette.hue()) {
                if s.is_dark { 3. } else { 2.3 }
            } else {
                1.6
            }
        }
        _ => 1.,
    }
}

/// Picks a surface multiplier by variant: neutral, tonal spot, expressive on a
/// yellow neutral, expressive otherwise, vibrant.
fn surface_multiplier(s: &DynamicScheme, values: [f64; 5]) -> f64 {
    match s.variant {
        Variant::Neutral => values[0],
        Variant::TonalSpot => values[1],
        Variant::Expressive => {
            if Hct::is_yellow(s.neutral_palette.hue()) {
                values[2]
            } else {
                values[3]
            }
        }
        Variant::Vibrant => values[4],
        _ => 1.,
    }
}

/// A light surface tone by neutral hue and variant: yellow, vibrant, other.
fn light_surface_tone(s: &DynamicScheme, yellow: f64, vibrant: f64, other: f64) -> f64 {
    if Hct::is_yellow(s.neutral_palette.hue()) {
        yellow
    } else if s.variant == Variant::Vibrant {
        vibrant
    } else {
        other
    }
}

pub(super) fn def(s: &DynamicScheme, role: DynamicRole) -> Def {
    let top = highest_surface(s);
    let dark = s.is_dark;
    let p = s.primary_palette;
    let q = s.secondary_palette;
    let t = s.tertiary_palette;
    let e = s.error_palette;
    let n = s.neutral_palette;
    let raised = (s.contrast_level > 0.).then_some(()).and(get_curve(1.5));
    match role {
        // Remapped colors share their target's constraints.
        SurfaceVariant => def(s, SurfaceContainerHighest),
        SurfaceTint => def(s, Primary),
        Background => def(s, Surface),
        OnBackground => def(s, OnSurface),

        Surface | InverseSurface | SurfaceContainerLowest => Def::new(n).is_background(),
        SurfaceDim => Def::new(n).is_background().multiplier(if dark {
            1.
        } else {
            surface_multiplier(s, [2.5, 1.7, 2.7, 1.75, 1.36])
        }),
        SurfaceBright => Def::new(n).is_background().multiplier(if dark {
            surface_multiplier(s, [2.5, 1.7, 2.7, 1.75, 1.36])
        } else {
            1.
        }),
        SurfaceContainerLow => Def::new(n)
            .is_background()
            .multiplier(surface_multiplier(s, [1.3, 1.25, 1.3, 1.15, 1.08])),
        SurfaceContainer => Def::new(n)
            .is_background()
            .multiplier(surface_multiplier(s, [1.6, 1.4, 1.6, 1.3, 1.15])),
        SurfaceContainerHigh => Def::new(n)
            .is_background()
            .multiplier(surface_multiplier(s, [1.9, 1.5, 1.95, 1.45, 1.22])),
        SurfaceContainerHighest => Def::new(n)
            .is_background()
            .multiplier(surface_multiplier(s, [2.2, 1.7, 2.3, 1.6, 1.29])),
        OnSurface => Def::new(n)
            .multiplier(on_surface_multiplier(s))
            .on(top, get_curve(if dark { 11. } else { 9. })),
        OnSurfaceVariant => Def::new(n)
            .multiplier(on_surface_multiplier(s))
            .on(top, get_curve(if dark { 6. } else { 4.5 })),
        Outline => Def::new(n)
            .multiplier(on_surface_multiplier(s))
            .on(top, get_curve(3.)),
        OutlineVariant => Def::new(n)
            .multiplier(on_surface_multiplier(s))
            .on(top, get_curve(1.5)),
        InverseOnSurface => Def::new(n).on(InverseSurface, get_curve(7.)),

        Primary => Def::new(p)
            .is_background()
            .on(top, get_curve(4.5))
            .pair(farther(
                PrimaryContainer,
                Primary,
                TonePolarity::RelativeLighter,
            )),
        PrimaryDim => Def::new(p)
            .is_background()
            .on(SurfaceContainerHigh, get_curve(4.5))
            .pair(farther(PrimaryDim, Primary, TonePolarity::Darker)),
        OnPrimary => Def::new(p).on(Primary, get_curve(6.)),
        PrimaryContainer => Def::new(p).is_background().on(top, raised),
        OnPrimaryContainer => Def::new(p).on(PrimaryContainer, get_curve(6.)),
        PrimaryFixed => Def::new(p).is_background().on(top, raised),
        PrimaryFixedDim => Def::new(p)
            .is_background()
            .pair(exact_darker(PrimaryFixedDim, PrimaryFixed)),
        OnPrimaryFixed => Def::new(p).on(PrimaryFixedDim, get_curve(7.)),
        OnPrimaryFixedVariant => Def::new(p).on(PrimaryFixedDim, get_curve(4.5)),
        InversePrimary => Def::new(p).on(InverseSurface, get_curve(6.)),

        Secondary => Def::new(q)
            .is_background()
            .on(top, get_curve(4.5))
            .pair(farther(
                SecondaryContainer,
                Secondary,
                TonePolarity::RelativeLighter,
            )),
        SecondaryDim => Def::new(q)
            .is_background()
            .on(SurfaceContainerHigh, get_curve(4.5))
            .pair(farther(SecondaryDim, Secondary, TonePolarity::Darker)),
        OnSecondary => Def::new(q).on(Secondary, get_curve(6.)),
        SecondaryContainer => Def::new(q).is_background().on(top, raised),
        OnSecondaryContainer => Def::new(q).on(SecondaryContainer, get_curve(6.)),
        SecondaryFixed => Def::new(q).is_background().on(top, raised),
        SecondaryFixedDim => Def::new(q)
            .is_background()
            .pair(exact_darker(SecondaryFixedDim, SecondaryFixed)),
        OnSecondaryFixed => Def::new(q).on(SecondaryFixedDim, get_curve(7.)),
        OnSecondaryFixedVariant => Def::new(q).on(SecondaryFixedDim, get_curve(4.5)),

        Tertiary => Def::new(t)
            .is_background()
            .on(top, get_curve(4.5))
            .pair(farther(
                TertiaryContainer,
                Tertiary,
                TonePolarity::RelativeLighter,
            )),
        TertiaryDim => Def::new(t)
            .is_background()
            .on(SurfaceContainerHigh, get_curve(4.5))
            .pair(farther(TertiaryDim, Tertiary, TonePolarity::Darker)),
        OnTertiary => Def::new(t).on(Tertiary, get_curve(6.)),
        TertiaryContainer => Def::new(t).is_background().on(top, raised),
        OnTertiaryContainer => Def::new(t).on(TertiaryContainer, get_curve(6.)),
        TertiaryFixed => Def::new(t).is_background().on(top, raised),
        TertiaryFixedDim => Def::new(t)
            .is_background()
            .pair(exact_darker(TertiaryFixedDim, TertiaryFixed)),
        OnTertiaryFixed => Def::new(t).on(TertiaryFixedDim, get_curve(7.)),
        OnTertiaryFixedVariant => Def::new(t).on(TertiaryFixedDim, get_curve(4.5)),

        Error => Def::new(e)
            .is_background()
            .on(top, get_curve(4.5))
            .pair(farther(
                ErrorContainer,
                Error,
                TonePolarity::RelativeLighter,
            )),
        ErrorDim => Def::new(e)
            .is_background()
            .on(SurfaceContainerHigh, get_curve(4.5))
            .pair(farther(ErrorDim, Error, TonePolarity::Darker)),
        OnError => Def::new(e).on(Error, get_curve(6.)),
        ErrorContainer => Def::new(e).is_background().on(top, raised),
        OnErrorContainer => Def::new(e).on(ErrorContainer, get_curve(4.5)),

        PrimaryPaletteKeyColor
        | SecondaryPaletteKeyColor
        | TertiaryPaletteKeyColor
        | NeutralPaletteKeyColor
        | NeutralVariantPaletteKeyColor
        | ErrorPaletteKeyColor
        | Shadow
        | Scrim => spec_2021::def(s, role),
    }
}

pub(super) fn tone(s: &DynamicScheme, role: DynamicRole) -> f64 {
    let dark = s.is_dark;
    let p = s.primary_palette;
    let q = s.secondary_palette;
    let t = s.tertiary_palette;
    let e = s.error_palette;
    let variant = s.variant;
    // The tone of the background, for roles with no tone of their own.
    let from_background = || {
        def(s, role)
            .background
            .map_or(50., |background| s.tone(background))
    };
    // Fixed roles take their container's tone in light mode at standard contrast.
    let light_standard = || DynamicScheme {
        is_dark: false,
        contrast_level: 0.,
        ..*s
    };
    match role {
        SurfaceVariant => tone(s, SurfaceContainerHighest),
        SurfaceTint => tone(s, Primary),
        Background => tone(s, Surface),
        OnBackground => s.tone(OnSurface),

        Surface => {
            if dark {
                4.
            } else {
                light_surface_tone(s, 99., 97., 98.)
            }
        }
        SurfaceDim => {
            if dark {
                4.
            } else {
                light_surface_tone(s, 90., 85., 87.)
            }
        }
        SurfaceBright => {
            if dark {
                18.
            } else {
                light_surface_tone(s, 99., 97., 98.)
            }
        }
        SurfaceContainerLowest => {
            if dark {
                0.
            } else {
                100.
            }
        }
        SurfaceContainerLow => {
            if dark {
                6.
            } else {
                light_surface_tone(s, 98., 95., 96.)
            }
        }
        SurfaceContainer => {
            if dark {
                9.
            } else {
                light_surface_tone(s, 96., 92., 94.)
            }
        }
        SurfaceContainerHigh => {
            if dark {
                12.
            } else {
                light_surface_tone(s, 94., 90., 92.)
            }
        }
        SurfaceContainerHighest => {
            if dark {
                15.
            } else {
                light_surface_tone(s, 92., 88., 90.)
            }
        }
        OnSurface => {
            if variant == Variant::Vibrant {
                t_max_c_scaled(s.neutral_palette, 0., 100., 1.1)
            } else {
                from_background()
            }
        }
        InverseSurface => {
            if dark {
                98.
            } else {
                4.
            }
        }

        Primary => match variant {
            Variant::Neutral => {
                if dark {
                    80.
                } else {
                    40.
                }
            }
            Variant::TonalSpot => {
                if dark {
                    80.
                } else {
                    t_max_c(p, 0., 100.)
                }
            }
            Variant::Expressive => {
                let upper = if dark {
                    if Hct::is_cyan(p.hue()) { 88. } else { 98. }
                } else if Hct::is_yellow(p.hue()) {
                    25.
                } else {
                    98.
                };
                t_max_c(p, 0., upper)
            }
            _ => t_max_c(p, 0., if Hct::is_cyan(p.hue()) { 88. } else { 98. }),
        },
        PrimaryDim => match variant {
            Variant::Neutral => 85.,
            Variant::TonalSpot => t_max_c(p, 0., 90.),
            _ => t_max_c(p, 0., 100.),
        },
        PrimaryContainer => match variant {
            Variant::Neutral => {
                if dark {
                    30.
                } else {
                    90.
                }
            }
            Variant::TonalSpot => {
                if dark {
                    t_min_c(p, 35., 93.)
                } else {
                    t_max_c(p, 0., 90.)
                }
            }
            Variant::Expressive => {
                if dark {
                    t_min_c(p, 30., 93.)
                } else {
                    t_max_c(p, 78., if Hct::is_cyan(p.hue()) { 88. } else { 90. })
                }
            }
            _ => {
                if dark {
                    t_min_c(p, 66., 93.)
                } else {
                    t_max_c(p, 66., if Hct::is_cyan(p.hue()) { 88. } else { 93. })
                }
            }
        },
        PrimaryFixed => light_standard().tone(PrimaryContainer),
        PrimaryFixedDim => s.tone(PrimaryFixed),
        InversePrimary => t_max_c(p, 0., 100.),

        Secondary => match variant {
            Variant::Neutral => {
                if dark {
                    t_min_c(q, 0., 98.)
                } else {
                    t_max_c(q, 0., 100.)
                }
            }
            Variant::Vibrant => t_max_c(q, 0., if dark { 90. } else { 98. }),
            _ => {
                if dark {
                    80.
                } else {
                    t_max_c(q, 0., 100.)
                }
            }
        },
        SecondaryDim => {
            if variant == Variant::Neutral {
                85.
            } else {
                t_max_c(q, 0., 90.)
            }
        }
        SecondaryContainer => match variant {
            Variant::Vibrant => {
                if dark {
                    t_min_c(q, 30., 40.)
                } else {
                    t_max_c(q, 84., 90.)
                }
            }
            Variant::Expressive => {
                if dark {
                    15.
                } else {
                    t_max_c(q, 90., 95.)
                }
            }
            _ => {
                if dark {
                    25.
                } else {
                    90.
                }
            }
        },
        SecondaryFixed => light_standard().tone(SecondaryContainer),
        SecondaryFixedDim => s.tone(SecondaryFixed),

        Tertiary => match variant {
            Variant::Expressive | Variant::Vibrant => {
                let upper = if Hct::is_cyan(t.hue()) {
                    88.
                } else if dark {
                    98.
                } else {
                    100.
                };
                t_max_c(t, 0., upper)
            }
            _ => {
                if dark {
                    t_max_c(t, 0., 98.)
                } else {
                    t_max_c(t, 0., 100.)
                }
            }
        },
        TertiaryDim => {
            if variant == Variant::TonalSpot {
                t_max_c(t, 0., 90.)
            } else {
                t_max_c(t, 0., 100.)
            }
        }
        TertiaryContainer => match variant {
            Variant::Neutral => t_max_c(t, 0., if dark { 93. } else { 96. }),
            Variant::TonalSpot => t_max_c(t, 0., if dark { 93. } else { 100. }),
            Variant::Expressive => {
                let upper = if Hct::is_cyan(t.hue()) {
                    88.
                } else if dark {
                    93.
                } else {
                    100.
                };
                t_max_c(t, 75., upper)
            }
            _ => {
                if dark {
                    t_max_c(t, 0., 93.)
                } else {
                    t_max_c(t, 72., 100.)
                }
            }
        },
        TertiaryFixed => light_standard().tone(TertiaryContainer),
        TertiaryFixedDim => s.tone(TertiaryFixed),

        Error => {
            if dark {
                t_min_c(e, 0., 98.)
            } else {
                t_max_c(e, 0., 100.)
            }
        }
        ErrorDim => t_min_c(e, 0., 100.),
        ErrorContainer => {
            if dark {
                t_min_c(e, 30., 93.)
            } else {
                t_max_c(e, 0., 90.)
            }
        }

        OnSurfaceVariant
        | Outline
        | OutlineVariant
        | InverseOnSurface
        | OnPrimary
        | OnPrimaryContainer
        | OnPrimaryFixed
        | OnPrimaryFixedVariant
        | OnSecondary
        | OnSecondaryContainer
        | OnSecondaryFixed
        | OnSecondaryFixedVariant
        | OnTertiary
        | OnTertiaryContainer
        | OnTertiaryFixed
        | OnTertiaryFixedVariant
        | OnError
        | OnErrorContainer => from_background(),

        PrimaryPaletteKeyColor
        | SecondaryPaletteKeyColor
        | TertiaryPaletteKeyColor
        | NeutralPaletteKeyColor
        | NeutralVariantPaletteKeyColor
        | ErrorPaletteKeyColor
        | Shadow
        | Scrim => spec_2021::tone(s, role),
    }
}
