//! Semantic button colors: state motion and the globally animated theme have
//! separate clocks. Resolve role leaves only after sampling the state animation.
use crate::{color::ColorRole, theme};
use gpui::Rgba;
use gpui_base::Interpolate;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Space {
    LegacySrgb,
    ModernOklab,
}

/// CSS color provenance matters: legacy RGB endpoints interpolate in sRGB;
/// the reference's disabled `color-mix` endpoints use Oklab in Chromium.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssColor {
    expression: Arc<Expression>,
    space: Space,
}

#[derive(Debug, PartialEq)]
enum Expression {
    Role(ColorRole, f32),
    Constant(Rgba),
    Mix(Vec<(CssColor, f64)>),
}

impl CssColor {
    pub(super) fn role(role: ColorRole) -> Self {
        Self::leaf(Expression::Role(role, 1.), Space::LegacySrgb)
    }

    /// The reference represents these values with `color-mix(in srgb, ...)`.
    pub(super) fn alpha_role(role: ColorRole, alpha: f32) -> Self {
        Self::leaf(Expression::Role(role, alpha), Space::ModernOklab)
    }

    pub(super) fn transparent() -> Self {
        Self::leaf(Expression::Constant(gpui::rgba(0)), Space::LegacySrgb)
    }

    fn leaf(expression: Expression, space: Space) -> Self {
        Self {
            expression: Arc::new(expression),
            space,
        }
    }

    pub(super) fn resolve(&self) -> Rgba {
        self.resolve_with(&theme::color)
    }

    fn resolve_with(&self, role_color: &impl Fn(ColorRole) -> Rgba) -> Rgba {
        match self.expression.as_ref() {
            Expression::Role(role, alpha) => {
                let mut color = role_color(*role);
                color.a *= alpha;
                color
            }
            Expression::Constant(color) => *color,
            Expression::Mix(terms) => mix(terms, role_color, self.space),
        }
    }

    fn append_terms(&self, terms: &mut Vec<(Self, f64)>, weight: f64, space: Space) {
        if self.space == space
            && let Expression::Mix(children) = self.expression.as_ref()
        {
            for (child, child_weight) in children {
                child.append_terms(terms, weight * child_weight, space);
            }
        } else if let Some((_, existing)) = terms.iter_mut().find(|(color, _)| color == self) {
            *existing += weight;
        } else {
            terms.push((self.clone(), weight));
        }
    }
}

impl Interpolate for CssColor {
    fn interpolate(&self, target: &Self, progress: f32) -> Self {
        if progress <= 0. || self == target {
            return self.clone();
        }
        if progress >= 1. {
            return target.clone();
        }
        let space = if self.space == Space::ModernOklab || target.space == Space::ModernOklab {
            Space::ModernOklab
        } else {
            Space::LegacySrgb
        };
        // Premultiplied mixtures are affine in their own space. Flattening and
        // merging equal terms preserves live role lookup without an ever-growing
        // history tree. A legacy mixture entering Oklab remains one conversion
        // boundary; subsequent modern mixtures flatten above it. Button targets
        // are leaves, so repeated interruptions have at most two mixture levels.
        let mut terms = Vec::new();
        self.append_terms(&mut terms, 1. - f64::from(progress), space);
        target.append_terms(&mut terms, f64::from(progress), space);
        Self {
            expression: Arc::new(Expression::Mix(terms)),
            space,
        }
    }
}

fn mix(terms: &[(CssColor, f64)], role_color: &impl Fn(ColorRole) -> Rgba, space: Space) -> Rgba {
    let mut alpha = 0.;
    let mut channels = [0.; 3];
    for (color, weight) in terms {
        let color = color.resolve_with(role_color);
        let contribution = f64::from(color.a) * weight;
        let rgb = [f64::from(color.r), f64::from(color.g), f64::from(color.b)];
        let value = match space {
            Space::LegacySrgb => rgb,
            Space::ModernOklab => to_oklab(rgb),
        };
        alpha += contribution;
        for i in 0..3 {
            channels[i] += value[i] * contribution;
        }
    }
    if alpha == 0. {
        return gpui::rgba(0);
    }
    let result = channels.map(|v| v / alpha);
    let [r, g, b] = match space {
        Space::LegacySrgb => result,
        Space::ModernOklab => from_oklab(result),
    };
    Rgba {
        r: r as f32,
        g: g as f32,
        b: b as f32,
        a: alpha as f32,
    }
}

fn matrix(m: [[f64; 3]; 3], value: [f64; 3]) -> [f64; 3] {
    m.map(|row| row.iter().zip(value).map(|(a, b)| a * b).sum())
}

// W3C CSS Color 4 sample conversion code (D65), retrieved 2026-10-29:
// https://www.w3.org/TR/css-color-4/#color-conversion-code
// Extended sRGB transfer functions and signed cube roots preserve out-of-gamut
// intermediates. Do not clip intermediate colors before premultiplied mixing.
#[allow(clippy::excessive_precision)] // Keep the published W3C coefficients verbatim.
fn to_oklab(rgb: [f64; 3]) -> [f64; 3] {
    let linear = rgb.map(|v| {
        if v.abs() <= 0.04045 {
            v / 12.92
        } else {
            v.signum() * ((v.abs() + 0.055) / 1.055).powf(2.4)
        }
    });
    let xyz = matrix(
        [
            [506752. / 1228815., 87881. / 245763., 12673. / 70218.],
            [87098. / 409605., 175762. / 245763., 12673. / 175545.],
            [7918. / 409605., 87881. / 737289., 1001167. / 1053270.],
        ],
        linear,
    );
    let lms = matrix(
        [
            [0.8190224379967030, 0.3619062600528904, -0.1288737815209879],
            [0.0329836539323885, 0.9292868615863434, 0.0361446663506424],
            [0.0481771893596242, 0.2642395317527308, 0.6335478284694309],
        ],
        xyz,
    );
    matrix(
        [
            [0.2104542683093140, 0.7936177747023054, -0.0040720430116193],
            [1.9779985324311684, -2.4285922420485799, 0.4505937096174110],
            [0.0259040424655478, 0.7827717124575296, -0.8086757549230774],
        ],
        lms.map(f64::cbrt),
    )
}

#[allow(clippy::excessive_precision)] // Keep the published W3C coefficients verbatim.
fn from_oklab(lab: [f64; 3]) -> [f64; 3] {
    let lms = matrix(
        [
            [1., 0.3963377773761749, 0.2158037573099136],
            [1., -0.1055613458156586, -0.0638541728258133],
            [1., -0.0894841775298119, -1.2914855480194092],
        ],
        lab,
    );
    let xyz = matrix(
        [
            [1.2268798758459243, -0.5578149944602171, 0.2813910456659647],
            [-0.0405757452148008, 1.1122868032803170, -0.0717110580655164],
            [-0.0763729366746601, -0.4214933324022432, 1.5869240198367816],
        ],
        lms.map(|v| v.powi(3)),
    );
    matrix(
        [
            [12831. / 3959., -329. / 214., -1974. / 3959.],
            [-851781. / 878810., 1648619. / 878810., 36519. / 878810.],
            [705. / 12673., -2585. / 12673., 705. / 667.],
        ],
        xyz,
    )
    .map(|v| {
        if v.abs() > 0.0031308 {
            v.signum() * (1.055 * v.abs().powf(1. / 2.4) - 0.055)
        } else {
            12.92 * v
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(role: ColorRole) -> Rgba {
        if role == ColorRole::PRIMARY {
            gpui::rgb(0x6750a4)
        } else if role == ColorRole::SECONDARY_CONTAINER {
            gpui::rgb(0xe8def8)
        } else if role == ColorRole::ON_PRIMARY {
            gpui::rgb(0xffffff)
        } else if role == ColorRole::ON_SURFACE {
            gpui::rgb(0x1d1b20)
        } else if role == ColorRole::ON_SURFACE_VARIANT {
            gpui::rgb(0x49454f)
        } else {
            panic!("unknown fixture role")
        }
    }

    fn browser_lab(group: &str, property: &str) -> [f64; 4] {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../conformance/reference/button-effects.json"
        ))
        .unwrap();
        let value = fixture[group][property]["value"].as_str().unwrap();
        let values: Vec<f64> = value
            .strip_prefix("oklab(")
            .unwrap()
            .strip_suffix(')')
            .unwrap()
            .split_whitespace()
            .filter(|v| *v != "/")
            .map(|v| v.parse().unwrap())
            .collect();
        values.try_into().unwrap()
    }

    fn assert_lab(value: Rgba, expected: [f64; 4]) {
        let lab = to_oklab([value.r as f64, value.g as f64, value.b as f64]);
        for (actual, expected) in lab.into_iter().chain([value.a as f64]).zip(expected) {
            assert!(
                // Chromium155 converts white to (0.999994, .0000455678,
                // .0000200868); the published W3C matrices yield (1, 0, 0).
                // Account for that measured conversion difference, not timing.
                (actual - expected).abs() < 0.00006,
                "{actual} != {expected}"
            );
        }
    }

    // Chromium155 + pinned c910072 CSS, paused at125ms/250ms, progress .96.
    #[test]
    fn browser_legacy_variant_sample_uses_srgb() {
        let result = CssColor::role(ColorRole::PRIMARY)
            .interpolate(&CssColor::role(ColorRole::SECONDARY_CONTAINER), 0.96)
            .resolve_with(&fixture);
        assert_eq!(
            [result.r, result.g, result.b].map(|v| (v * 255.).round() as u8),
            [227, 216, 245]
        );
    }

    #[test]
    fn browser_disabled_samples_use_premultiplied_oklab() {
        let content = CssColor::role(ColorRole::ON_PRIMARY)
            .interpolate(
                &CssColor::alpha_role(ColorRole::ON_SURFACE_VARIANT, 0.38),
                0.96,
            )
            .resolve_with(&fixture);
        assert_lab(content, browser_lab("disabled", "color"));
        let background = CssColor::role(ColorRole::PRIMARY)
            .interpolate(&CssColor::alpha_role(ColorRole::ON_SURFACE, 0.1), 0.96)
            .resolve_with(&fixture);
        assert_lab(background, browser_lab("disabled", "background-color"));
    }

    #[test]
    fn transparent_endpoint_does_not_darken_content() {
        for color in [
            CssColor::role(ColorRole::PRIMARY),
            CssColor::alpha_role(ColorRole::PRIMARY, 0.38),
        ] {
            let result = color
                .interpolate(&CssColor::transparent(), 0.5)
                .resolve_with(&fixture);
            let original = color.resolve_with(&fixture);
            for (a, b) in [result.r, result.g, result.b]
                .into_iter()
                .zip([original.r, original.g, original.b])
            {
                assert!((a - b).abs() < 0.00001);
            }
            assert_eq!(result.a, original.a * 0.5);
        }
        assert_eq!(CssColor::transparent().resolve(), gpui::rgba(0));
    }

    #[test]
    fn interrupted_colors_preserve_modern_space_and_completion_releases_history() {
        let enabled = CssColor::role(ColorRole::ON_PRIMARY);
        let disabled = CssColor::alpha_role(ColorRole::ON_SURFACE_VARIANT, 0.38);
        let sampled = enabled.interpolate(&disabled, 0.96);
        let reversed = sampled.interpolate(&enabled, 0.279);
        // Chromium: 25ms into the shortened 240ms reverse.
        assert_lab(
            reversed.resolve_with(&fixture),
            browser_lab("reversed", "color"),
        );
        assert_eq!(reversed.space, Space::ModernOklab);
        assert_eq!(reversed.interpolate(&enabled, 1.), enabled);
        assert_eq!(reversed.interpolate(&enabled, 0.), reversed);
    }

    #[test]
    fn theme_changes_resolve_leaves_without_changing_state_progress() {
        let expression = CssColor::role(ColorRole::PRIMARY)
            .interpolate(&CssColor::alpha_role(ColorRole::ON_SURFACE, 0.1), 0.5);
        let a = expression.resolve_with(&|_| gpui::rgb(0xff0000));
        let b = expression.resolve_with(&|_| gpui::rgb(0x0000ff));
        assert!((a.r - 1.).abs() < 0.00001 && a.b.abs() < 0.00001);
        assert!((b.b - 1.).abs() < 0.00001 && b.r.abs() < 0.00001);
        assert_eq!(a.a, b.a);
    }

    #[test]
    fn repeated_interruptions_keep_live_roles_and_bounded_history() {
        let target = CssColor::alpha_role(ColorRole::ON_SURFACE_VARIANT, 0.38);
        let mut current = CssColor::role(ColorRole::ON_PRIMARY);
        for index in 0..1000 {
            let next = if index % 2 == 0 {
                target.clone()
            } else {
                CssColor::role(ColorRole::ON_PRIMARY)
            };
            current = current.interpolate(&next, 0.1);
            let Expression::Mix(terms) = current.expression.as_ref() else {
                panic!("expected mix")
            };
            assert_eq!(terms.len(), 2);
            assert!(
                terms
                    .iter()
                    .all(|(color, _)| !matches!(color.expression.as_ref(), Expression::Mix(_)))
            );
        }
        let changed = current.resolve_with(&|_| gpui::rgb(0xff0000));
        assert!((changed.r - 1.).abs() < 0.00001 && changed.b.abs() < 0.00001);
        assert_eq!(current.interpolate(&target, 1.), target);
    }

    #[test]
    fn legacy_mixture_entering_modern_preserves_conversion_and_live_theme() {
        let legacy = CssColor::role(ColorRole::PRIMARY)
            .interpolate(&CssColor::role(ColorRole::SECONDARY_CONTAINER), 0.4);
        let disabled = CssColor::alpha_role(ColorRole::ON_SURFACE, 0.1);
        let mut current = legacy.interpolate(&disabled, 0.3);

        // Independently evaluate the sRGB mixture first, then convert that
        // result to Oklab for the premultiplied outer mixture.
        let expected = |roles: &dyn Fn(ColorRole) -> Rgba| {
            let first = roles(ColorRole::PRIMARY);
            let second = roles(ColorRole::SECONDARY_CONTAINER);
            let disabled = roles(ColorRole::ON_SURFACE);
            let legacy_rgb = [
                first.r + (second.r - first.r) * 0.4,
                first.g + (second.g - first.g) * 0.4,
                first.b + (second.b - first.b) * 0.4,
            ];
            let from = to_oklab(legacy_rgb.map(f64::from));
            let to = to_oklab([disabled.r, disabled.g, disabled.b].map(f64::from));
            let alpha = 0.7 + 0.3 * 0.1;
            let lab = std::array::from_fn(|i| (from[i] * 0.7 + to[i] * 0.03) / alpha);
            let rgb = from_oklab(lab);
            [rgb[0], rgb[1], rgb[2], alpha]
        };
        let changed_theme = |role| {
            if role == ColorRole::PRIMARY {
                gpui::rgb(0xf03020)
            } else if role == ColorRole::SECONDARY_CONTAINER {
                gpui::rgb(0x2040f0)
            } else {
                gpui::rgb(0x20c060)
            }
        };
        for roles in [&fixture as &dyn Fn(ColorRole) -> Rgba, &changed_theme] {
            let actual = current.resolve_with(&|role| roles(role));
            for (actual, expected) in [actual.r, actual.g, actual.b, actual.a]
                .into_iter()
                .zip(expected(roles))
            {
                assert!((f64::from(actual) - expected).abs() < 0.00001);
            }
        }
        assert_ne!(
            current.resolve_with(&fixture),
            current.resolve_with(&changed_theme)
        );

        // Keep the legacy contribution significant after 1000 interruptions
        // so a frozen conversion boundary cannot hide behind underflow.
        for index in 0..1000 {
            let next = if index % 2 == 0 {
                disabled.clone()
            } else {
                CssColor::role(ColorRole::ON_PRIMARY)
            };
            current = current.interpolate(&next, 0.0001);
            let Expression::Mix(terms) = current.expression.as_ref() else {
                panic!("expected modern mixture")
            };
            assert_eq!(current.space, Space::ModernOklab);
            assert!(terms.len() <= 3);
            let nested: Vec<_> = terms
                .iter()
                .filter(|(color, _)| matches!(color.expression.as_ref(), Expression::Mix(_)))
                .collect();
            assert_eq!(nested.len(), 1);
            assert_eq!(nested[0].0, legacy);
            assert!(nested[0].1 > 0.6);
        }
        // Only roles inside the nested legacy mixture change here. This
        // specifically detects freezing that boundary while other leaves live.
        let changed_legacy = |role| {
            if role == ColorRole::PRIMARY || role == ColorRole::SECONDARY_CONTAINER {
                gpui::rgb(0xff0000)
            } else {
                fixture(role)
            }
        };
        let original = current.resolve_with(&fixture);
        let changed = current.resolve_with(&changed_legacy);
        assert!((changed.r - original.r).abs() > 0.1);
        assert!((changed.b - original.b).abs() > 0.1);
        assert_eq!(current.interpolate(&disabled, 1.), disabled);
    }
}
