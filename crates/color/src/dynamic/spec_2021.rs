// Copyright 2025 Google LLC
// SPDX-License-Identifier: Apache-2.0
// Ported to Rust from material-color-utilities
// (typescript/dynamiccolor/color_spec_2021.ts), modified for this crate.
// See licenses/material-color-utilities.txt.

//! The 2021 color spec: every role's palette, tone and constraints.
use super::color::{ContrastCurve, Def, DeltaConstraint, DynamicRole, ToneDeltaPair, TonePolarity};
use super::color::{DynamicRole::*, foreground_tone};
use super::dislike;
use super::hct::Hct;
use super::scheme::{DynamicScheme, Variant};

const fn curve(low: f64, normal: f64, medium: f64, high: f64) -> Option<ContrastCurve> {
    Some(ContrastCurve::new(low, normal, medium, high))
}

fn is_fidelity(s: &DynamicScheme) -> bool {
    matches!(s.variant, Variant::Fidelity | Variant::Content)
}

fn is_monochrome(s: &DynamicScheme) -> bool {
    s.variant == Variant::Monochrome
}

/// The role on which foregrounds of a surface must hold their contrast.
pub(super) fn highest_surface(s: &DynamicScheme) -> DynamicRole {
    if s.is_dark { SurfaceBright } else { SurfaceDim }
}

fn nearer_pair(a: DynamicRole, b: DynamicRole) -> ToneDeltaPair {
    ToneDeltaPair {
        a,
        b,
        delta: 10.,
        polarity: TonePolarity::Nearer,
        stay_together: false,
        constraint: DeltaConstraint::Exact,
    }
}

fn lighter_pair(a: DynamicRole, b: DynamicRole) -> ToneDeltaPair {
    ToneDeltaPair {
        a,
        b,
        delta: 10.,
        polarity: TonePolarity::Lighter,
        stay_together: true,
        constraint: DeltaConstraint::Exact,
    }
}

/// A role's constraints. The dim roles, which upstream leaves undefined before
/// the 2025 spec and which throw there, act as their base role.
pub(super) fn def(s: &DynamicScheme, role: DynamicRole) -> Def {
    let top = highest_surface(s);
    let p = s.primary_palette;
    let q = s.secondary_palette;
    let t = s.tertiary_palette;
    let e = s.error_palette;
    let n = s.neutral_palette;
    let nv = s.neutral_variant_palette;
    match role {
        PrimaryPaletteKeyColor => Def::new(p),
        SecondaryPaletteKeyColor => Def::new(q),
        TertiaryPaletteKeyColor => Def::new(t),
        NeutralPaletteKeyColor => Def::new(n),
        NeutralVariantPaletteKeyColor => Def::new(nv),
        ErrorPaletteKeyColor => Def::new(e),
        Background
        | Surface
        | SurfaceDim
        | SurfaceBright
        | SurfaceContainerLowest
        | SurfaceContainerLow
        | SurfaceContainer
        | SurfaceContainerHigh
        | SurfaceContainerHighest
        | InverseSurface => Def::new(n).is_background(),
        OnBackground => Def::new(n).on(Background, curve(3., 3., 4.5, 7.)),
        OnSurface => Def::new(n).on(top, curve(4.5, 7., 11., 21.)),
        SurfaceVariant => Def::new(nv).is_background(),
        OnSurfaceVariant => Def::new(nv).on(top, curve(3., 4.5, 7., 11.)),
        InverseOnSurface => Def::new(n).on(InverseSurface, curve(4.5, 7., 11., 21.)),
        Outline => Def::new(nv).on(top, curve(1.5, 3., 4.5, 7.)),
        OutlineVariant => Def::new(nv).on(top, curve(1., 1., 3., 4.5)),
        Shadow | Scrim => Def::new(n),
        SurfaceTint => Def::new(p).is_background(),

        Primary | PrimaryDim => Def::new(p)
            .is_background()
            .on(top, curve(3., 4.5, 7., 7.))
            .pair(nearer_pair(PrimaryContainer, Primary)),
        OnPrimary => Def::new(p).on(Primary, curve(4.5, 7., 11., 21.)),
        PrimaryContainer => Def::new(p)
            .is_background()
            .on(top, curve(1., 1., 3., 4.5))
            .pair(nearer_pair(PrimaryContainer, Primary)),
        OnPrimaryContainer => Def::new(p).on(PrimaryContainer, curve(3., 4.5, 7., 11.)),
        InversePrimary => Def::new(p).on(InverseSurface, curve(3., 4.5, 7., 7.)),

        Secondary | SecondaryDim => Def::new(q)
            .is_background()
            .on(top, curve(3., 4.5, 7., 7.))
            .pair(nearer_pair(SecondaryContainer, Secondary)),
        OnSecondary => Def::new(q).on(Secondary, curve(4.5, 7., 11., 21.)),
        SecondaryContainer => Def::new(q)
            .is_background()
            .on(top, curve(1., 1., 3., 4.5))
            .pair(nearer_pair(SecondaryContainer, Secondary)),
        OnSecondaryContainer => Def::new(q).on(SecondaryContainer, curve(3., 4.5, 7., 11.)),

        Tertiary | TertiaryDim => Def::new(t)
            .is_background()
            .on(top, curve(3., 4.5, 7., 7.))
            .pair(nearer_pair(TertiaryContainer, Tertiary)),
        OnTertiary => Def::new(t).on(Tertiary, curve(4.5, 7., 11., 21.)),
        TertiaryContainer => Def::new(t)
            .is_background()
            .on(top, curve(1., 1., 3., 4.5))
            .pair(nearer_pair(TertiaryContainer, Tertiary)),
        OnTertiaryContainer => Def::new(t).on(TertiaryContainer, curve(3., 4.5, 7., 11.)),

        Error | ErrorDim => Def::new(e)
            .is_background()
            .on(top, curve(3., 4.5, 7., 7.))
            .pair(nearer_pair(ErrorContainer, Error)),
        OnError => Def::new(e).on(Error, curve(4.5, 7., 11., 21.)),
        ErrorContainer => Def::new(e)
            .is_background()
            .on(top, curve(1., 1., 3., 4.5))
            .pair(nearer_pair(ErrorContainer, Error)),
        OnErrorContainer => Def::new(e).on(ErrorContainer, curve(3., 4.5, 7., 11.)),

        PrimaryFixed | PrimaryFixedDim => Def::new(p)
            .is_background()
            .on(top, curve(1., 1., 3., 4.5))
            .pair(lighter_pair(PrimaryFixed, PrimaryFixedDim)),
        OnPrimaryFixed => Def::new(p)
            .on(PrimaryFixedDim, curve(4.5, 7., 11., 21.))
            .second(PrimaryFixed),
        OnPrimaryFixedVariant => Def::new(p)
            .on(PrimaryFixedDim, curve(3., 4.5, 7., 11.))
            .second(PrimaryFixed),

        SecondaryFixed | SecondaryFixedDim => Def::new(q)
            .is_background()
            .on(top, curve(1., 1., 3., 4.5))
            .pair(lighter_pair(SecondaryFixed, SecondaryFixedDim)),
        OnSecondaryFixed => Def::new(q)
            .on(SecondaryFixedDim, curve(4.5, 7., 11., 21.))
            .second(SecondaryFixed),
        OnSecondaryFixedVariant => Def::new(q)
            .on(SecondaryFixedDim, curve(3., 4.5, 7., 11.))
            .second(SecondaryFixed),

        TertiaryFixed | TertiaryFixedDim => Def::new(t)
            .is_background()
            .on(top, curve(1., 1., 3., 4.5))
            .pair(lighter_pair(TertiaryFixed, TertiaryFixedDim)),
        OnTertiaryFixed => Def::new(t)
            .on(TertiaryFixedDim, curve(4.5, 7., 11., 21.))
            .second(TertiaryFixed),
        OnTertiaryFixedVariant => Def::new(t)
            .on(TertiaryFixedDim, curve(3., 4.5, 7., 11.))
            .second(TertiaryFixed),
    }
}

/// The tone at which a chroma is available, searching from `tone`.
fn find_desired_chroma_by_tone(hue: f64, chroma: f64, tone: f64, by_decreasing: bool) -> f64 {
    let mut answer = tone;
    let mut closest = Hct::from_hct(hue, chroma, tone);
    if closest.chroma() < chroma {
        let mut peak = closest.chroma();
        while closest.chroma() < chroma {
            answer += if by_decreasing { -1. } else { 1. };
            let potential = Hct::from_hct(hue, chroma, answer);
            if peak > potential.chroma() {
                break;
            }
            if (potential.chroma() - chroma).abs() < 0.4 {
                break;
            }
            if (potential.chroma() - chroma).abs() < (closest.chroma() - chroma).abs() {
                closest = potential;
            }
            peak = peak.max(potential.chroma());
        }
    }
    answer
}

/// A role's initial tone, before contrast constraints.
pub(super) fn tone(s: &DynamicScheme, role: DynamicRole) -> f64 {
    let dark = s.is_dark;
    let mono = is_monochrome(s);
    let fidelity = is_fidelity(s);
    let pick = |dark_tone: f64, light_tone: f64| if dark { dark_tone } else { light_tone };
    let level = s.contrast_level;
    match role {
        PrimaryPaletteKeyColor => s.primary_palette.key_color().tone(),
        SecondaryPaletteKeyColor => s.secondary_palette.key_color().tone(),
        TertiaryPaletteKeyColor => s.tertiary_palette.key_color().tone(),
        NeutralPaletteKeyColor => s.neutral_palette.key_color().tone(),
        NeutralVariantPaletteKeyColor => s.neutral_variant_palette.key_color().tone(),
        ErrorPaletteKeyColor => s.error_palette.key_color().tone(),
        Background | Surface => pick(6., 98.),
        OnBackground | OnSurface => pick(90., 10.),
        SurfaceDim => {
            if dark {
                6.
            } else {
                ContrastCurve::new(87., 87., 80., 75.).get(level)
            }
        }
        SurfaceBright => {
            if dark {
                ContrastCurve::new(24., 24., 29., 34.).get(level)
            } else {
                98.
            }
        }
        SurfaceContainerLowest => {
            if dark {
                ContrastCurve::new(4., 4., 2., 0.).get(level)
            } else {
                100.
            }
        }
        SurfaceContainerLow => {
            if dark {
                ContrastCurve::new(10., 10., 11., 12.).get(level)
            } else {
                ContrastCurve::new(96., 96., 96., 95.).get(level)
            }
        }
        SurfaceContainer => {
            if dark {
                ContrastCurve::new(12., 12., 16., 20.).get(level)
            } else {
                ContrastCurve::new(94., 94., 92., 90.).get(level)
            }
        }
        SurfaceContainerHigh => {
            if dark {
                ContrastCurve::new(17., 17., 21., 25.).get(level)
            } else {
                ContrastCurve::new(92., 92., 88., 85.).get(level)
            }
        }
        SurfaceContainerHighest => {
            if dark {
                ContrastCurve::new(22., 22., 26., 30.).get(level)
            } else {
                ContrastCurve::new(90., 90., 84., 80.).get(level)
            }
        }
        SurfaceVariant => pick(30., 90.),
        OnSurfaceVariant => pick(80., 30.),
        InverseSurface => pick(90., 20.),
        InverseOnSurface => pick(20., 95.),
        Outline => pick(60., 50.),
        OutlineVariant => pick(30., 80.),
        Shadow | Scrim => 0.,
        SurfaceTint => pick(80., 40.),

        Primary | PrimaryDim => {
            if mono {
                pick(100., 0.)
            } else {
                pick(80., 40.)
            }
        }
        OnPrimary => {
            if mono {
                pick(10., 90.)
            } else {
                pick(20., 100.)
            }
        }
        PrimaryContainer => {
            if fidelity {
                s.source.tone()
            } else if mono {
                pick(85., 25.)
            } else {
                pick(30., 90.)
            }
        }
        OnPrimaryContainer => {
            if fidelity {
                foreground_tone(tone(s, PrimaryContainer), 4.5)
            } else if mono {
                pick(0., 100.)
            } else {
                pick(90., 30.)
            }
        }
        InversePrimary => pick(40., 80.),

        Secondary | SecondaryDim => pick(80., 40.),
        OnSecondary => {
            if mono {
                pick(10., 100.)
            } else {
                pick(20., 100.)
            }
        }
        SecondaryContainer => {
            let initial = pick(30., 90.);
            if mono {
                pick(30., 85.)
            } else if !fidelity {
                initial
            } else {
                find_desired_chroma_by_tone(
                    s.secondary_palette.hue(),
                    s.secondary_palette.chroma(),
                    initial,
                    !dark,
                )
            }
        }
        OnSecondaryContainer => {
            if mono {
                pick(90., 10.)
            } else if !fidelity {
                pick(90., 30.)
            } else {
                foreground_tone(tone(s, SecondaryContainer), 4.5)
            }
        }

        Tertiary | TertiaryDim => {
            if mono {
                pick(90., 25.)
            } else {
                pick(80., 40.)
            }
        }
        OnTertiary => {
            if mono {
                pick(10., 90.)
            } else {
                pick(20., 100.)
            }
        }
        TertiaryContainer => {
            if mono {
                pick(60., 49.)
            } else if !fidelity {
                pick(30., 90.)
            } else {
                dislike::fix_if_disliked(s.tertiary_palette.hct(s.source.tone())).tone()
            }
        }
        OnTertiaryContainer => {
            if mono {
                pick(0., 100.)
            } else if !fidelity {
                pick(90., 30.)
            } else {
                foreground_tone(tone(s, TertiaryContainer), 4.5)
            }
        }

        Error | ErrorDim => pick(80., 40.),
        OnError => pick(20., 100.),
        ErrorContainer => pick(30., 90.),
        OnErrorContainer => {
            if mono {
                pick(90., 10.)
            } else {
                pick(90., 30.)
            }
        }

        PrimaryFixed => {
            if mono {
                40.
            } else {
                90.
            }
        }
        PrimaryFixedDim => {
            if mono {
                30.
            } else {
                80.
            }
        }
        OnPrimaryFixed => {
            if mono {
                100.
            } else {
                10.
            }
        }
        OnPrimaryFixedVariant => {
            if mono {
                90.
            } else {
                30.
            }
        }
        SecondaryFixed => {
            if mono {
                80.
            } else {
                90.
            }
        }
        SecondaryFixedDim => {
            if mono {
                70.
            } else {
                80.
            }
        }
        OnSecondaryFixed => 10.,
        OnSecondaryFixedVariant => {
            if mono {
                25.
            } else {
                30.
            }
        }
        TertiaryFixed => {
            if mono {
                40.
            } else {
                90.
            }
        }
        TertiaryFixedDim => {
            if mono {
                30.
            } else {
                80.
            }
        }
        OnTertiaryFixed => {
            if mono {
                100.
            } else {
                10.
            }
        }
        OnTertiaryFixedVariant => {
            if mono {
                90.
            } else {
                30.
            }
        }
    }
}
