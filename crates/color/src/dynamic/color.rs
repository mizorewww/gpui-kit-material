// Copyright 2022 Google LLC
// SPDX-License-Identifier: Apache-2.0
// Ported to Rust from material-color-utilities (typescript/dynamiccolor:
// dynamic_color.ts, contrast_curve.ts, tone_delta_pair.ts), modified for this
// crate: colors are an enum resolved through match tables instead of objects of
// closures, and the tone delegates cover the phone platform only.
// See licenses/material-color-utilities.txt.

//! Dynamic colors: roles whose tone follows mode, contrast and background.
use super::contrast;
use super::hct::Hct;
use super::palette::TonalPalette;
use super::scheme::{DynamicScheme, SpecVersion};
use super::spec_2021;
use super::spec_2025;
use super::utils::{clamp_double, lerp};
use std::cell::RefCell;
use std::collections::HashMap;

/// A role of a [`DynamicScheme`].
#[allow(missing_docs, reason = "each is the Material role of the same name")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DynamicRole {
    PrimaryPaletteKeyColor,
    SecondaryPaletteKeyColor,
    TertiaryPaletteKeyColor,
    NeutralPaletteKeyColor,
    NeutralVariantPaletteKeyColor,
    ErrorPaletteKeyColor,
    Background,
    OnBackground,
    Surface,
    SurfaceDim,
    SurfaceBright,
    SurfaceContainerLowest,
    SurfaceContainerLow,
    SurfaceContainer,
    SurfaceContainerHigh,
    SurfaceContainerHighest,
    OnSurface,
    SurfaceVariant,
    OnSurfaceVariant,
    InverseSurface,
    InverseOnSurface,
    Outline,
    OutlineVariant,
    Shadow,
    Scrim,
    SurfaceTint,
    Primary,
    PrimaryDim,
    OnPrimary,
    PrimaryContainer,
    OnPrimaryContainer,
    InversePrimary,
    Secondary,
    SecondaryDim,
    OnSecondary,
    SecondaryContainer,
    OnSecondaryContainer,
    Tertiary,
    TertiaryDim,
    OnTertiary,
    TertiaryContainer,
    OnTertiaryContainer,
    Error,
    ErrorDim,
    OnError,
    ErrorContainer,
    OnErrorContainer,
    PrimaryFixed,
    PrimaryFixedDim,
    OnPrimaryFixed,
    OnPrimaryFixedVariant,
    SecondaryFixed,
    SecondaryFixedDim,
    OnSecondaryFixed,
    OnSecondaryFixedVariant,
    TertiaryFixed,
    TertiaryFixedDim,
    OnTertiaryFixed,
    OnTertiaryFixedVariant,
}

impl DynamicRole {
    /// Every role.
    pub const ALL: [Self; 59] = {
        use DynamicRole::*;
        [
            PrimaryPaletteKeyColor,
            SecondaryPaletteKeyColor,
            TertiaryPaletteKeyColor,
            NeutralPaletteKeyColor,
            NeutralVariantPaletteKeyColor,
            ErrorPaletteKeyColor,
            Background,
            OnBackground,
            Surface,
            SurfaceDim,
            SurfaceBright,
            SurfaceContainerLowest,
            SurfaceContainerLow,
            SurfaceContainer,
            SurfaceContainerHigh,
            SurfaceContainerHighest,
            OnSurface,
            SurfaceVariant,
            OnSurfaceVariant,
            InverseSurface,
            InverseOnSurface,
            Outline,
            OutlineVariant,
            Shadow,
            Scrim,
            SurfaceTint,
            Primary,
            PrimaryDim,
            OnPrimary,
            PrimaryContainer,
            OnPrimaryContainer,
            InversePrimary,
            Secondary,
            SecondaryDim,
            OnSecondary,
            SecondaryContainer,
            OnSecondaryContainer,
            Tertiary,
            TertiaryDim,
            OnTertiary,
            TertiaryContainer,
            OnTertiaryContainer,
            Error,
            ErrorDim,
            OnError,
            ErrorContainer,
            OnErrorContainer,
            PrimaryFixed,
            PrimaryFixedDim,
            OnPrimaryFixed,
            OnPrimaryFixedVariant,
            SecondaryFixed,
            SecondaryFixedDim,
            OnSecondaryFixed,
            OnSecondaryFixedVariant,
            TertiaryFixed,
            TertiaryFixedDim,
            OnTertiaryFixed,
            OnTertiaryFixedVariant,
        ]
    };

    /// Upstream's snake_case name of the role.
    pub const fn name(self) -> &'static str {
        use DynamicRole::*;
        match self {
            PrimaryPaletteKeyColor => "primary_palette_key_color",
            SecondaryPaletteKeyColor => "secondary_palette_key_color",
            TertiaryPaletteKeyColor => "tertiary_palette_key_color",
            NeutralPaletteKeyColor => "neutral_palette_key_color",
            NeutralVariantPaletteKeyColor => "neutral_variant_palette_key_color",
            ErrorPaletteKeyColor => "error_palette_key_color",
            Background => "background",
            OnBackground => "on_background",
            Surface => "surface",
            SurfaceDim => "surface_dim",
            SurfaceBright => "surface_bright",
            SurfaceContainerLowest => "surface_container_lowest",
            SurfaceContainerLow => "surface_container_low",
            SurfaceContainer => "surface_container",
            SurfaceContainerHigh => "surface_container_high",
            SurfaceContainerHighest => "surface_container_highest",
            OnSurface => "on_surface",
            SurfaceVariant => "surface_variant",
            OnSurfaceVariant => "on_surface_variant",
            InverseSurface => "inverse_surface",
            InverseOnSurface => "inverse_on_surface",
            Outline => "outline",
            OutlineVariant => "outline_variant",
            Shadow => "shadow",
            Scrim => "scrim",
            SurfaceTint => "surface_tint",
            Primary => "primary",
            PrimaryDim => "primary_dim",
            OnPrimary => "on_primary",
            PrimaryContainer => "primary_container",
            OnPrimaryContainer => "on_primary_container",
            InversePrimary => "inverse_primary",
            Secondary => "secondary",
            SecondaryDim => "secondary_dim",
            OnSecondary => "on_secondary",
            SecondaryContainer => "secondary_container",
            OnSecondaryContainer => "on_secondary_container",
            Tertiary => "tertiary",
            TertiaryDim => "tertiary_dim",
            OnTertiary => "on_tertiary",
            TertiaryContainer => "tertiary_container",
            OnTertiaryContainer => "on_tertiary_container",
            Error => "error",
            ErrorDim => "error_dim",
            OnError => "on_error",
            ErrorContainer => "error_container",
            OnErrorContainer => "on_error_container",
            PrimaryFixed => "primary_fixed",
            PrimaryFixedDim => "primary_fixed_dim",
            OnPrimaryFixed => "on_primary_fixed",
            OnPrimaryFixedVariant => "on_primary_fixed_variant",
            SecondaryFixed => "secondary_fixed",
            SecondaryFixedDim => "secondary_fixed_dim",
            OnSecondaryFixed => "on_secondary_fixed",
            OnSecondaryFixedVariant => "on_secondary_fixed_variant",
            TertiaryFixed => "tertiary_fixed",
            TertiaryFixedDim => "tertiary_fixed_dim",
            OnTertiaryFixed => "on_tertiary_fixed",
            OnTertiaryFixedVariant => "on_tertiary_fixed_variant",
        }
    }
}

/// A value that changes with the contrast level, usually the contrast a color
/// needs against its background.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContrastCurve {
    /// The value at contrast level -1.
    pub low: f64,
    /// The value at contrast level 0.
    pub normal: f64,
    /// The value at contrast level 0.5.
    pub medium: f64,
    /// The value at contrast level 1.
    pub high: f64,
}

impl ContrastCurve {
    /// A curve from its values at contrast levels -1, 0, 0.5 and 1.
    pub const fn new(low: f64, normal: f64, medium: f64, high: f64) -> Self {
        Self {
            low,
            normal,
            medium,
            high,
        }
    }

    /// The value at a contrast level, interpolating between the stops.
    pub fn get(&self, level: f64) -> f64 {
        if level <= -1. {
            self.low
        } else if level < 0. {
            lerp(self.low, self.normal, level + 1.)
        } else if level < 0.5 {
            lerp(self.normal, self.medium, level / 0.5)
        } else if level < 1. {
            lerp(self.medium, self.high, (level - 0.5) / 0.5)
        } else {
            self.high
        }
    }
}

/// How the tones of a pair relate; `a` is described compared with `b`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TonePolarity {
    Darker,
    Lighter,
    Nearer,
    RelativeLighter,
}

/// How to fulfill a tone delta constraint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DeltaConstraint {
    Exact,
    Farther,
}

/// A required tone distance between two roles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ToneDeltaPair {
    pub a: DynamicRole,
    pub b: DynamicRole,
    pub delta: f64,
    pub polarity: TonePolarity,
    pub stay_together: bool,
    pub constraint: DeltaConstraint,
}

/// A role's constraints under one scheme. Its initial tone is separate
/// (`spec_*::tone`) because computing it can be expensive and can read other
/// roles.
#[derive(Clone, Copy, Debug)]
pub(super) struct Def {
    pub palette: TonalPalette,
    pub is_background: bool,
    pub chroma_multiplier: f64,
    pub background: Option<DynamicRole>,
    pub second_background: Option<DynamicRole>,
    pub curve: Option<ContrastCurve>,
    pub pair: Option<ToneDeltaPair>,
}

impl Def {
    pub fn new(palette: TonalPalette) -> Self {
        Self {
            palette,
            is_background: false,
            chroma_multiplier: 1.,
            background: None,
            second_background: None,
            curve: None,
            pair: None,
        }
    }

    pub fn is_background(mut self) -> Self {
        self.is_background = true;
        self
    }

    pub fn multiplier(mut self, multiplier: f64) -> Self {
        self.chroma_multiplier = multiplier;
        self
    }

    pub fn on(mut self, background: DynamicRole, curve: Option<ContrastCurve>) -> Self {
        self.background = Some(background);
        self.curve = curve;
        self
    }

    pub fn second(mut self, background: DynamicRole) -> Self {
        self.second_background = Some(background);
        self
    }

    pub fn pair(mut self, pair: ToneDeltaPair) -> Self {
        self.pair = Some(pair);
        self
    }
}

fn def(s: &DynamicScheme, role: DynamicRole) -> Def {
    match s.spec {
        SpecVersion::Spec2021 => spec_2021::def(s, role),
        SpecVersion::Spec2025 => spec_2025::def(s, role),
    }
}

fn initial_tone(s: &DynamicScheme, role: DynamicRole) -> f64 {
    memoized(s, role, true, || match s.spec {
        SpecVersion::Spec2021 => spec_2021::tone(s, role),
        SpecVersion::Spec2025 => spec_2025::tone(s, role),
    })
}

/// A tone: its role, the scheme's mode and contrast (fixed roles resolve under
/// a light, standard copy of the scheme), and whether it is the initial tone.
type MemoKey = (DynamicRole, bool, u64, bool);

thread_local! {
    // Resolving roles one at a time re-solves shared tones many times over:
    // measured at a 162 ms median for a 2025 vibrant scheme on an Apple M2.
    // The owner is `with_tone_memo`: the memo lives for one call over one
    // scheme, so it never has anything to invalidate.
    static TONE_MEMO: RefCell<Option<HashMap<MemoKey, f64>>> = const { RefCell::new(None) };
}

/// Run `f`, sharing resolved tones between the roles it reads. Every role read
/// inside must belong to one scheme.
pub(super) fn with_tone_memo<R>(f: impl FnOnce() -> R) -> R {
    struct Clear;
    impl Drop for Clear {
        fn drop(&mut self) {
            TONE_MEMO.with_borrow_mut(|memo| *memo = None);
        }
    }
    TONE_MEMO.with_borrow_mut(|memo| *memo = Some(HashMap::with_capacity(128)));
    let _clear = Clear;
    f()
}

fn memoized(
    s: &DynamicScheme,
    role: DynamicRole,
    initial: bool,
    compute: impl FnOnce() -> f64,
) -> f64 {
    let key = (role, s.is_dark, s.contrast_level.to_bits(), initial);
    if let Some(tone) =
        TONE_MEMO.with_borrow(|memo| memo.as_ref().and_then(|memo| memo.get(&key).copied()))
    {
        return tone;
    }
    // Not borrowed while computing: tones read other tones.
    let tone = compute();
    TONE_MEMO.with_borrow_mut(|memo| {
        if let Some(memo) = memo {
            memo.insert(key, tone);
        }
    });
    tone
}

/// The foreground tone nearest `ratio` against `bg_tone`.
pub(super) fn foreground_tone(bg_tone: f64, ratio: f64) -> f64 {
    let lighter_tone = contrast::lighter_unsafe(bg_tone, ratio);
    let darker_tone = contrast::darker_unsafe(bg_tone, ratio);
    let lighter_ratio = contrast::ratio_of_tones(lighter_tone, bg_tone);
    let darker_ratio = contrast::ratio_of_tones(darker_tone, bg_tone);
    if tone_prefers_light_foreground(bg_tone) {
        // Where the initial ratio is high and neither direction reaches it,
        // staying light avoids a momentary black at some contrast levels.
        let negligible = (lighter_ratio - darker_ratio).abs() < 0.1
            && lighter_ratio < ratio
            && darker_ratio < ratio;
        if lighter_ratio >= ratio || lighter_ratio >= darker_ratio || negligible {
            lighter_tone
        } else {
            darker_tone
        }
    } else if darker_ratio >= ratio || darker_ratio >= lighter_ratio {
        darker_tone
    } else {
        lighter_tone
    }
}

/// White reads best on tones below about 60.
fn tone_prefers_light_foreground(tone: f64) -> bool {
    tone.round() < 60.
}

pub(super) fn hct(s: &DynamicScheme, role: DynamicRole) -> Hct {
    let d = def(s, role);
    let tone = tone(s, role);
    match s.spec {
        SpecVersion::Spec2021 => d.palette.hct(tone),
        SpecVersion::Spec2025 => {
            if d.chroma_multiplier == 1. {
                return d.palette.hct(tone);
            }
            let chroma = d.palette.chroma() * d.chroma_multiplier;
            if tone == 99. && Hct::is_yellow(d.palette.hue()) {
                TonalPalette::from_hue_and_chroma(d.palette.hue(), chroma).hct(tone)
            } else {
                Hct::from_hct(d.palette.hue(), chroma, tone)
            }
        }
    }
}

pub(super) fn tone(s: &DynamicScheme, role: DynamicRole) -> f64 {
    memoized(s, role, false, || match s.spec {
        SpecVersion::Spec2021 => tone_2021(s, role),
        SpecVersion::Spec2025 => tone_2025(s, role),
    })
}

/// A tone satisfying `desired` against two backgrounds.
fn dual_background_tone(
    s: &DynamicScheme,
    answer: f64,
    first: DynamicRole,
    second: DynamicRole,
    desired: f64,
) -> f64 {
    let (tone_1, tone_2) = (tone(s, first), tone(s, second));
    let (upper, lower) = (tone_1.max(tone_2), tone_1.min(tone_2));
    if contrast::ratio_of_tones(upper, answer) >= desired
        && contrast::ratio_of_tones(lower, answer) >= desired
    {
        return answer;
    }
    let light = contrast::lighter(upper, desired);
    let dark = contrast::darker(lower, desired);
    if tone_prefers_light_foreground(tone_1) || tone_prefers_light_foreground(tone_2) {
        return light.unwrap_or(100.);
    }
    match (light, dark) {
        (Some(only), None) | (None, Some(only)) => only,
        (_, dark) => dark.unwrap_or(0.),
    }
}

fn tone_2021(s: &DynamicScheme, role: DynamicRole) -> f64 {
    let d = def(s, role);
    let decreasing = s.contrast_level < 0.;
    if let Some(pair) = d.pair {
        let a_is_nearer = pair.polarity == TonePolarity::Nearer
            || (pair.polarity == TonePolarity::Lighter && !s.is_dark)
            || (pair.polarity == TonePolarity::Darker && s.is_dark);
        let (nearer, farther) = if a_is_nearer {
            (pair.a, pair.b)
        } else {
            (pair.b, pair.a)
        };
        let am_nearer = role == nearer;
        let expansion = if s.is_dark { 1. } else { -1. };
        let delta = pair.delta;
        let mut n_tone = initial_tone(s, nearer);
        let mut f_tone = initial_tone(s, farther);
        // 1st round: solve to the minimum of each.
        if let (Some(bg), Some(n_curve), Some(f_curve)) =
            (d.background, def(s, nearer).curve, def(s, farther).curve)
        {
            let bg_tone = tone(s, bg);
            let n_contrast = n_curve.get(s.contrast_level);
            let f_contrast = f_curve.get(s.contrast_level);
            if contrast::ratio_of_tones(bg_tone, n_tone) < n_contrast {
                n_tone = foreground_tone(bg_tone, n_contrast);
            }
            if contrast::ratio_of_tones(bg_tone, f_tone) < f_contrast {
                f_tone = foreground_tone(bg_tone, f_contrast);
            }
            if decreasing {
                n_tone = foreground_tone(bg_tone, n_contrast);
                f_tone = foreground_tone(bg_tone, f_contrast);
            }
        }
        if (f_tone - n_tone) * expansion < delta {
            // 2nd round: expand farther to match delta.
            f_tone = clamp_double(0., 100., n_tone + delta * expansion);
            if (f_tone - n_tone) * expansion < delta {
                // 3rd round: contract nearer to match delta.
                n_tone = clamp_double(0., 100., f_tone - delta * expansion);
            }
        }
        // Avoid the 50-59 awkward zone.
        let move_both = |n: &mut f64, f: &mut f64| {
            if expansion > 0. {
                *n = 60.;
                *f = f.max(*n + delta * expansion);
            } else {
                *n = 49.;
                *f = f.min(*n + delta * expansion);
            }
        };
        if (50. ..60.).contains(&n_tone) {
            move_both(&mut n_tone, &mut f_tone);
        } else if (50. ..60.).contains(&f_tone) {
            if pair.stay_together {
                move_both(&mut n_tone, &mut f_tone);
            } else {
                f_tone = if expansion > 0. { 60. } else { 49. };
            }
        }
        return if am_nearer { n_tone } else { f_tone };
    }

    let mut answer = initial_tone(s, role);
    let (Some(bg), Some(curve)) = (d.background, d.curve) else {
        return answer;
    };
    let bg_tone = tone(s, bg);
    let desired = curve.get(s.contrast_level);
    if contrast::ratio_of_tones(bg_tone, answer) < desired || decreasing {
        answer = foreground_tone(bg_tone, desired);
    }
    if d.is_background && (50. ..60.).contains(&answer) {
        answer = if contrast::ratio_of_tones(49., bg_tone) >= desired {
            49.
        } else {
            60.
        };
    }
    match d.second_background {
        Some(second) => dual_background_tone(s, answer, bg, second, desired),
        None => answer,
    }
}

/// Fixed-dim accents keep the tones the awkward-zone rule would move.
fn is_fixed_dim(role: DynamicRole) -> bool {
    matches!(
        role,
        DynamicRole::PrimaryFixedDim
            | DynamicRole::SecondaryFixedDim
            | DynamicRole::TertiaryFixedDim
    )
}

fn avoid_awkward_tones(tone: f64) -> f64 {
    if tone >= 57. {
        clamp_double(65., 100., tone)
    } else {
        clamp_double(0., 49., tone)
    }
}

fn tone_2025(s: &DynamicScheme, role: DynamicRole) -> f64 {
    let d = def(s, role);
    if let Some(pair) = d.pair {
        let absolute = match pair.polarity {
            TonePolarity::Darker => -pair.delta,
            TonePolarity::RelativeLighter if s.is_dark => -pair.delta,
            _ => pair.delta,
        };
        let am_a = role == pair.a;
        let (self_role, ref_role) = if am_a {
            (pair.a, pair.b)
        } else {
            (pair.b, pair.a)
        };
        let mut self_tone = initial_tone(s, self_role);
        let ref_tone = tone(s, ref_role);
        let relative = absolute * if am_a { 1. } else { -1. };
        match pair.constraint {
            DeltaConstraint::Exact => {
                self_tone = clamp_double(0., 100., ref_tone + relative);
            }
            DeltaConstraint::Farther => {
                self_tone = if relative > 0. {
                    clamp_double(ref_tone + relative, 100., self_tone)
                } else {
                    clamp_double(0., ref_tone + relative, self_tone)
                };
            }
        }
        if let (Some(bg), Some(curve)) = (d.background, d.curve) {
            let bg_tone = tone(s, bg);
            let wanted = curve.get(s.contrast_level);
            if contrast::ratio_of_tones(bg_tone, self_tone) < wanted || s.contrast_level < 0. {
                self_tone = foreground_tone(bg_tone, wanted);
            }
        }
        if d.is_background && !is_fixed_dim(role) {
            self_tone = avoid_awkward_tones(self_tone);
        }
        return self_tone;
    }

    let mut answer = initial_tone(s, role);
    let (Some(bg), Some(curve)) = (d.background, d.curve) else {
        return answer;
    };
    let bg_tone = tone(s, bg);
    let desired = curve.get(s.contrast_level);
    if contrast::ratio_of_tones(bg_tone, answer) < desired || s.contrast_level < 0. {
        answer = foreground_tone(bg_tone, desired);
    }
    if d.is_background && !is_fixed_dim(role) {
        answer = avoid_awkward_tones(answer);
    }
    match d.second_background {
        Some(second) => dual_background_tone(s, answer, bg, second, desired),
        None => answer,
    }
}
