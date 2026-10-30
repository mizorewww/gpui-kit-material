//! Scheme vectors from material-color-utilities: typescript/dynamiccolor
//! (dynamic_color_test.ts, fixed_dynamic_color_test.ts),
//! typescript/scheme/dynamic_scheme_test.ts, dart/test (scheme_monochrome_test,
//! scheme_correctness_test), plus `tests/fixtures/mcu-schemes.json`: every role
//! of 416 schemes produced by running the pinned upstream TypeScript.
use gpui_kit_material_color::dynamic::DynamicRole::{self, *};
use gpui_kit_material_color::dynamic::contrast::ratio_of_tones;
use gpui_kit_material_color::dynamic::utils::lstar_from_argb;
use gpui_kit_material_color::dynamic::{
    ContrastCurve, DynamicScheme, Hct, SeedScheme, SpecVersion, Variant,
};
use gpui_kit_material_color::{ColorRole, themes};

fn scheme(seed: u32, variant: Variant, dark: bool, contrast: f64) -> DynamicScheme {
    DynamicScheme::new(
        Hct::from_argb(seed),
        variant,
        dark,
        contrast,
        SpecVersion::Spec2021,
    )
}

#[test]
fn every_role_of_every_fixture_scheme_matches_upstream() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/mcu-schemes.json")).unwrap();
    let names = |key: &str| -> Vec<String> {
        fixture["meta"][key]
            .as_array()
            .unwrap()
            .iter()
            .map(|name| name.as_str().unwrap().to_owned())
            .collect()
    };
    let role = |name: &str| {
        DynamicRole::ALL
            .into_iter()
            .find(|role| role.name() == name)
            .unwrap_or_else(|| panic!("{name}"))
    };
    let base = names("roles");
    let extra = names("roles_2025_extra");
    let rows = fixture["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 416);
    let mut mismatches = Vec::new();
    let mut compared = 0;
    for row in rows {
        let seed = u32::from_str_radix(row["seed"].as_str().unwrap(), 16).unwrap();
        let variant = match row["variant"].as_str().unwrap() {
            "monochrome" => Variant::Monochrome,
            "neutral" => Variant::Neutral,
            "tonal-spot" => Variant::TonalSpot,
            "vibrant" => Variant::Vibrant,
            "expressive" => Variant::Expressive,
            "fidelity" => Variant::Fidelity,
            "content" => Variant::Content,
            "rainbow" => Variant::Rainbow,
            "fruit-salad" => Variant::FruitSalad,
            other => panic!("{other}"),
        };
        let spec = if row["spec"] == "2025" {
            SpecVersion::Spec2025
        } else {
            SpecVersion::Spec2021
        };
        let dark = row["dark"].as_bool().unwrap();
        let contrast = row["contrast"].as_f64().unwrap();
        let dynamic = DynamicScheme::new(
            Hct::from_argb(0xff00_0000 | seed),
            variant,
            dark,
            contrast,
            spec,
        );
        let colors: Vec<&str> = row["colors"].as_str().unwrap().split(' ').collect();
        let list = base
            .iter()
            .chain(extra.iter().filter(|_| spec == SpecVersion::Spec2025));
        for (name, expected) in list.zip(&colors) {
            let actual = format!("{:06x}", dynamic.argb(role(name)) & 0xff_ffff);
            compared += 1;
            if actual != *expected {
                mismatches.push(format!(
                    "{seed:06x} {variant:?} {spec:?} dark={dark} contrast={contrast} {name}: \
                     {actual}, upstream {expected}"
                ));
            }
        }
    }
    assert!(compared > 20_000, "{compared}");
    assert!(
        mismatches.is_empty(),
        "{} of {compared} differ, first: {:#?}",
        mismatches.len(),
        &mismatches[..mismatches.len().min(20)]
    );
}

#[track_caller]
fn tones(scheme: &DynamicScheme, expected: &[(DynamicRole, f64)]) {
    for (role, tone) in expected {
        let actual = scheme.hct(*role).tone();
        // Jasmine toBeCloseTo(x, 0) means |a - x| < 0.5.
        assert!((actual - tone).abs() < 0.5, "{role:?}: {actual} vs {tone}");
    }
}

const FIXED_NON_MONOCHROME: &[(DynamicRole, f64)] = &[
    (PrimaryFixed, 90.),
    (PrimaryFixedDim, 80.),
    (OnPrimaryFixed, 10.),
    (OnPrimaryFixedVariant, 30.),
    (SecondaryFixed, 90.),
    (SecondaryFixedDim, 80.),
    (OnSecondaryFixed, 10.),
    (OnSecondaryFixedVariant, 30.),
    (TertiaryFixed, 90.),
    (TertiaryFixedDim, 80.),
    (OnTertiaryFixed, 10.),
    (OnTertiaryFixedVariant, 30.),
];

const FIXED_MONOCHROME: &[(DynamicRole, f64)] = &[
    (PrimaryFixed, 40.),
    (PrimaryFixedDim, 30.),
    (OnPrimaryFixed, 100.),
    (OnPrimaryFixedVariant, 90.),
    (SecondaryFixed, 80.),
    (SecondaryFixedDim, 70.),
    (OnSecondaryFixed, 10.),
    (OnSecondaryFixedVariant, 25.),
    (TertiaryFixed, 40.),
    (TertiaryFixedDim, 30.),
    (OnTertiaryFixed, 100.),
    (OnTertiaryFixedVariant, 90.),
];

#[test]
fn fixed_colors() {
    let tonal = scheme(0xffff0000, Variant::TonalSpot, true, 0.);
    tones(&tonal, FIXED_NON_MONOCHROME);
    for (role, argb) in [
        (PrimaryFixed, 0xFFFFDAD4),
        (PrimaryFixedDim, 0xFFFFB4A8),
        (OnPrimaryFixed, 0xFF3A0905),
        (OnPrimaryFixedVariant, 0xFF73342A),
        (SecondaryFixed, 0xFFFFDAD4),
        (SecondaryFixedDim, 0xFFE7BDB6),
        (OnSecondaryFixed, 0xFF2C1512),
        (OnSecondaryFixedVariant, 0xFF5D3F3B),
        (TertiaryFixed, 0xFFFBDFA6),
        (TertiaryFixedDim, 0xFFDEC48C),
        (OnTertiaryFixed, 0xFF251A00),
        (OnTertiaryFixedVariant, 0xFF564419),
    ] {
        assert_eq!(tonal.argb(role), argb, "{role:?}");
    }
    tones(
        &scheme(0xffff0000, Variant::Monochrome, false, 0.),
        FIXED_MONOCHROME,
    );
    tones(
        &scheme(0xffff0000, Variant::Monochrome, true, 0.),
        FIXED_MONOCHROME,
    );
}

/// dart closeTo(x, 1) means |a - x| <= 1.
#[track_caller]
fn tones_within_one(scheme: &DynamicScheme, expected: &[(DynamicRole, f64)]) {
    for (role, tone) in expected {
        let actual = scheme.hct(*role).tone();
        assert!((actual - tone).abs() <= 1., "{role:?}: {actual} vs {tone}");
    }
}

#[test]
fn monochrome_spec() {
    tones_within_one(
        &scheme(0xff0000ff, Variant::Monochrome, true, 0.),
        &[
            (Primary, 100.),
            (OnPrimary, 10.),
            (PrimaryContainer, 85.),
            (OnPrimaryContainer, 0.),
            (Secondary, 80.),
            (OnSecondary, 10.),
            (SecondaryContainer, 30.),
            (OnSecondaryContainer, 90.),
            (Tertiary, 90.),
            (OnTertiary, 10.),
            (TertiaryContainer, 60.),
            (OnTertiaryContainer, 0.),
        ],
    );
    tones_within_one(
        &scheme(0xff0000ff, Variant::Monochrome, false, 0.),
        &[
            (Primary, 0.),
            (OnPrimary, 90.),
            (PrimaryContainer, 25.),
            (OnPrimaryContainer, 100.),
            (Secondary, 40.),
            (OnSecondary, 100.),
            (SecondaryContainer, 85.),
            (OnSecondaryContainer, 10.),
            (Tertiary, 25.),
            (OnTertiary, 90.),
            (TertiaryContainer, 49.),
            (OnTertiaryContainer, 100.),
        ],
    );
}

#[test]
fn rotated_hue_vectors() {
    let hct = Hct::from_hct(43., 16., 16.);
    // Jasmine toBeCloseTo(x, 0.4) means |a - x| < 10^-0.4 / 2.
    let close = |actual: f64, expected: f64| {
        assert!(
            (actual - expected).abs() < 10f64.powf(-0.4) / 2.,
            "{actual} vs {expected}"
        );
    };
    close(DynamicScheme::rotated_hue(hct, &[], &[]), 43.);
    close(DynamicScheme::rotated_hue(hct, &[0.], &[0.]), 43.);
    close(DynamicScheme::rotated_hue(hct, &[0.], &[0., 1.]), 43.);
    close(
        DynamicScheme::rotated_hue(hct, &[0., 42., 360.], &[0., 15., 0.]),
        58.,
    );
    close(
        DynamicScheme::rotated_hue(hct, &[0., 42., 360.], &[0., 480., 0.]),
        163.,
    );
}

const SEEDS: [u32; 4] = [0xFFFF0000, 0xFFFFFF00, 0xFF00FF00, 0xFF0000FF];

#[test]
fn schemes_respect_text_surface_contrast() {
    let pairs = [
        (OnPrimary, Primary),
        (OnPrimaryContainer, PrimaryContainer),
        (OnSecondary, Secondary),
        (OnSecondaryContainer, SecondaryContainer),
        (OnTertiary, Tertiary),
        (OnTertiaryContainer, TertiaryContainer),
        (OnError, Error),
        (OnErrorContainer, ErrorContainer),
        (OnBackground, Background),
        (OnSurfaceVariant, SurfaceBright),
        (OnSurfaceVariant, SurfaceDim),
    ];
    for seed in SEEDS {
        for level in [-1., -0.75, -0.5, -0.25, 0., 0.25, 0.5, 0.75, 1.] {
            for dark in [false, true] {
                for variant in [
                    Variant::Content,
                    Variant::Expressive,
                    Variant::Fidelity,
                    Variant::Monochrome,
                    Variant::Neutral,
                    Variant::TonalSpot,
                    Variant::Vibrant,
                ] {
                    let s = scheme(seed, variant, dark, level);
                    for (foreground, background) in pairs {
                        let ratio =
                            ratio_of_tones(s.hct(foreground).tone(), s.hct(background).tone());
                        let minimum = if level >= 0. { 4.5 } else { 3. };
                        assert!(
                            ratio >= minimum,
                            "{seed:x} {variant:?} {dark} {level} {foreground:?} on \
                             {background:?}: {ratio}"
                        );
                    }
                }
            }
        }
    }
}

enum Constraint {
    Contrast(DynamicRole, DynamicRole, ContrastCurve),
    /// `a` farther from the surface than `b` by 10.
    Farther(DynamicRole, DynamicRole),
    /// `a` darker than `b` by 10.
    Darker(DynamicRole, DynamicRole),
    Background(DynamicRole),
}

fn constraints() -> Vec<Constraint> {
    use Constraint::{Contrast, Darker, Farther};
    let on_text = ContrastCurve::new(4.5, 7., 11., 21.);
    let on_variant = ContrastCurve::new(3., 4.5, 7., 11.);
    let accent = ContrastCurve::new(3., 4.5, 7., 7.);
    let outline = ContrastCurve::new(1.5, 3., 4.5, 7.);
    let container = ContrastCurve::new(0., 0., 3., 4.5);
    let mut list = Vec::new();
    for surface in [SurfaceDim, SurfaceBright] {
        list.push(Contrast(OnSurface, surface, on_text));
        list.push(Contrast(OnSurfaceVariant, surface, on_variant));
        list.push(Contrast(Outline, surface, outline));
        for role in [Primary, Secondary, Tertiary, Error] {
            list.push(Contrast(role, surface, accent));
        }
        for role in [
            PrimaryContainer,
            PrimaryFixed,
            PrimaryFixedDim,
            SecondaryContainer,
            SecondaryFixed,
            SecondaryFixedDim,
            TertiaryContainer,
            TertiaryFixed,
            TertiaryFixedDim,
            ErrorContainer,
            OutlineVariant,
        ] {
            list.push(Contrast(role, surface, container));
        }
    }
    list.push(Contrast(InverseOnSurface, InverseSurface, on_text));
    list.push(Contrast(InversePrimary, InverseSurface, accent));
    for (on, role) in [
        (OnPrimary, Primary),
        (OnSecondary, Secondary),
        (OnTertiary, Tertiary),
        (OnError, Error),
    ] {
        list.push(Contrast(on, role, on_text));
    }
    for (on, role) in [
        (OnPrimaryContainer, PrimaryContainer),
        (OnSecondaryContainer, SecondaryContainer),
        (OnTertiaryContainer, TertiaryContainer),
        (OnErrorContainer, ErrorContainer),
    ] {
        list.push(Contrast(on, role, on_variant));
    }
    for (on, on_variant_role, fixed, dim) in [
        (
            OnPrimaryFixed,
            OnPrimaryFixedVariant,
            PrimaryFixed,
            PrimaryFixedDim,
        ),
        (
            OnSecondaryFixed,
            OnSecondaryFixedVariant,
            SecondaryFixed,
            SecondaryFixedDim,
        ),
        (
            OnTertiaryFixed,
            OnTertiaryFixedVariant,
            TertiaryFixed,
            TertiaryFixedDim,
        ),
    ] {
        for background in [fixed, dim] {
            list.push(Contrast(on, background, on_text));
            list.push(Contrast(on_variant_role, background, on_variant));
        }
        list.push(Darker(dim, fixed));
    }
    for (role, container_role) in [
        (Primary, PrimaryContainer),
        (Secondary, SecondaryContainer),
        (Tertiary, TertiaryContainer),
        (Error, ErrorContainer),
    ] {
        list.push(Farther(role, container_role));
    }
    for role in [
        Background,
        Error,
        ErrorContainer,
        Primary,
        PrimaryContainer,
        PrimaryFixed,
        PrimaryFixedDim,
        Secondary,
        SecondaryContainer,
        SecondaryFixed,
        SecondaryFixedDim,
        Surface,
        SurfaceBright,
        SurfaceContainer,
        SurfaceContainerHigh,
        SurfaceContainerHighest,
        SurfaceContainerLow,
        SurfaceContainerLowest,
        SurfaceDim,
        SurfaceTint,
        SurfaceVariant,
        Tertiary,
        TertiaryContainer,
        TertiaryFixed,
        TertiaryFixedDim,
    ] {
        list.push(Constraint::Background(role));
    }
    list
}

fn check(s: &DynamicScheme, constraint: &Constraint) -> Result<(), String> {
    match *constraint {
        Constraint::Contrast(foreground, background, curve) => {
            let (fore, back) = (s.hct(foreground).tone(), s.hct(background).tone());
            let actual = ratio_of_tones(fore, back);
            let desired = curve.get(s.contrast_level);
            const TOLERANCE: f64 = 0.05;
            let failed = if desired <= 4.5 {
                actual < desired - TOLERANCE
            } else {
                actual < 4.5 - TOLERANCE
                    || (actual < desired - TOLERANCE && fore != 100. && fore != 0.)
            };
            if failed {
                return Err(format!(
                    "{foreground:?} on {background:?}: {actual}, desired {desired}"
                ));
            }
        }
        Constraint::Farther(a, b) => {
            let (a_tone, b_tone) = (s.hct(a).tone(), s.hct(b).tone());
            let actual = if s.is_dark {
                a_tone - b_tone
            } else {
                b_tone - a_tone
            };
            if actual < 10. - 0.5 {
                return Err(format!("{a:?} farther than {b:?} by {actual}"));
            }
        }
        Constraint::Darker(a, b) => {
            let actual = s.hct(b).tone() - s.hct(a).tone();
            if actual < 10. - 0.5 {
                return Err(format!("{a:?} darker than {b:?} by {actual}"));
            }
        }
        Constraint::Background(role) => {
            let tone = s.hct(role).tone();
            if (50.5..59.5).contains(&tone) {
                return Err(format!("{role:?} has tone {tone}, in the forbidden zone"));
            }
        }
    }
    Ok(())
}

#[test]
fn scheme_correctness_holds_for_every_variant_seed_mode_and_contrast() {
    let constraints = constraints();
    let mut failures = Vec::new();
    for variant in Variant::ALL {
        for level in [-1., 0., 0.5, 1.] {
            for seed in [0xFF0000FF, 0xFF00FF00, 0xFFFFFF00, 0xFFFF0000] {
                for dark in [false, true] {
                    let s = scheme(seed, variant, dark, level);
                    for constraint in &constraints {
                        if let Err(message) = check(&s, constraint) {
                            failures
                                .push(format!("{variant:?} {level} {seed:x} {dark}: {message}"));
                        }
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:#?}",
        failures.len(),
        &failures[..failures.len().min(20)]
    );
}

#[test]
fn the_conversion_covers_every_role_and_keeps_upstream_values() {
    for spec in [SpecVersion::Spec2021, SpecVersion::Spec2025] {
        for variant in Variant::ALL {
            for dark in [false, true] {
                let seed = SeedScheme::new(0x6750a4)
                    .variant(variant)
                    .dark(dark)
                    .spec(spec);
                let converted = seed.scheme();
                let dynamic = seed.dynamic();
                assert_eq!(converted.is_dark(), dark);
                for (role, upstream) in [
                    (ColorRole::PRIMARY, Primary),
                    (ColorRole::ON_SURFACE, OnSurface),
                    (ColorRole::SURFACE_CONTAINER, SurfaceContainer),
                    (ColorRole::TERTIARY_FIXED_DIM, TertiaryFixedDim),
                    (ColorRole::SHADOW, Shadow),
                ] {
                    assert_eq!(
                        converted.get(role),
                        gpui_kit_material_color::Color::hex(dynamic.argb(upstream) & 0xff_ffff),
                        "{role:?}"
                    );
                }
                // Every reference role resolves and is opaque.
                for role in themes::REFERENCE_ROLES {
                    assert_eq!(converted.get(role).a, 1., "{role:?}");
                }
            }
        }
    }
}

#[test]
fn status_roles_stay_readable_at_every_contrast() {
    let tone = |color: gpui_kit_material_color::Color| {
        let channel = |value: f32| (value * 255.).round() as u32;
        lstar_from_argb(
            0xff00_0000 | channel(color.r) << 16 | channel(color.g) << 8 | channel(color.b),
        )
    };
    for seed in [
        0x6750a4, 0xff0000, 0xffff00, 0x00ff00, 0x0000ff, 0x000000, 0xffffff,
    ] {
        for dark in [false, true] {
            for level in [-1., 0., 0.5, 1.] {
                let scheme = SeedScheme::new(seed).dark(dark).contrast(level).scheme();
                let minimum = if level >= 0. { 4.5 } else { 3. };
                for (role, on) in [
                    (ColorRole::SUCCESS, ColorRole::ON_SUCCESS),
                    (ColorRole::WARNING, ColorRole::ON_WARNING),
                    (ColorRole::INFO, ColorRole::ON_INFO),
                ] {
                    let ratio = ratio_of_tones(tone(scheme.get(role)), tone(scheme.get(on)));
                    assert!(
                        ratio >= minimum,
                        "{seed:x} {dark} {level} {role:?}: {ratio}"
                    );
                }
            }
        }
    }
}

#[test]
fn hex_seeds_parse() {
    assert_eq!(SeedScheme::parse_hex("#6750A4"), Some(0x6750a4));
    assert_eq!(SeedScheme::parse_hex(" 6750a4 "), Some(0x6750a4));
    assert_eq!(SeedScheme::parse_hex("#abc"), Some(0xaabbcc));
    assert_eq!(SeedScheme::parse_hex("abc"), Some(0xaabbcc));
    for invalid in ["", "#", "12", "1234", "12345", "1234567", "#ggg", "+12345"] {
        assert_eq!(SeedScheme::parse_hex(invalid), None, "{invalid}");
    }
}

#[test]
fn out_of_range_contrast_is_limited_and_never_panics() {
    let at_one = scheme(0xff6750a4, Variant::TonalSpot, false, 1.);
    assert_eq!(scheme(0xff6750a4, Variant::TonalSpot, false, 5.), at_one);
    let at_zero = scheme(0xff6750a4, Variant::TonalSpot, false, 0.);
    assert_eq!(
        scheme(0xff6750a4, Variant::TonalSpot, false, f64::NAN),
        at_zero
    );
}

#[test]
fn variants_without_a_2025_spec_resolve_as_2021() {
    for variant in Variant::ALL {
        let s = DynamicScheme::new(
            Hct::from_argb(0xff6750a4),
            variant,
            false,
            0.,
            SpecVersion::Spec2025,
        );
        assert_eq!(
            s.spec == SpecVersion::Spec2025,
            variant.has_2025_spec(),
            "{variant:?}"
        );
    }
}

#[test]
fn dim_roles_before_the_2025_spec_act_as_their_base_role() {
    let s = scheme(0xff6750a4, Variant::TonalSpot, false, 0.);
    for (dim, base) in [
        (PrimaryDim, Primary),
        (SecondaryDim, Secondary),
        (TertiaryDim, Tertiary),
        (ErrorDim, Error),
    ] {
        assert_eq!(s.argb(dim), s.argb(base));
    }
}
