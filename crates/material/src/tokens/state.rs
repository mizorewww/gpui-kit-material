//! M3 state layers and disabled emphasis, resolved over the container so a
//! selected control keeps its color under hover and press.
//!
//! Opacities are m3e's `--m3e-sys-state-*-state-layer-opacity` and
//! `--m3e-sys-state-disabled-alpha` (kyra-labs/m3exp-web
//! `3b1e0047c0021eab7b4c0128fa1bec95c76ca746`,
//! `packages/m3e-tokens/src/generated/tokens.css`, from Compose
//! `StateTokens.kt` @ androidx `19991f2c` L22-L25). Disabled containers vary
//! per component in m3e (e.g. 10% for most buttons, 12% for the plain tonal
//! button); [`DISABLED_CONTAINER`] is the classic 12% default. Fixture:
//! `tests/fixtures/m3e-sys-tokens.json`.
use gpui::Rgba;

pub const HOVER: f32 = 0.08;
pub const FOCUS: f32 = 0.10;
pub const PRESS: f32 = 0.10;
pub const DRAG: f32 = 0.16;
/// Disabled containers and content: on-surface at 12% and 38%.
pub const DISABLED_CONTAINER: f32 = 0.12;
pub const DISABLED_CONTENT: f32 = 0.38;
/// The text selection highlight: primary at 40% (Compose Material 3
/// `TextSelectionColors.kt`, `TextSelectionBackgroundOpacity = 0.4f`).
pub const TEXT_SELECTION: f32 = 0.40;

/// Composite a state layer of `content` at `opacity` over `container`
/// (source-over alpha compositing).
///
/// This is the one state-layer recipe for every component. The layer's
/// opacity multiplies `content`'s own alpha, and RGB is weighted by alpha,
/// so a layer over a transparent container keeps its own hue instead of
/// darkening toward the container's stored RGB.
pub fn layer(container: Rgba, content: Rgba, opacity: f32) -> Rgba {
    let top = content.a * opacity.clamp(0., 1.);
    let a = top + container.a * (1. - top);
    if a <= 0. {
        return Rgba {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 0.,
        };
    }
    let c = |x: f32, y: f32| (x * top + y * container.a * (1. - top)) / a;
    Rgba {
        r: c(content.r, container.r),
        g: c(content.g, container.g),
        b: c(content.b, container.b),
        a,
    }
}

/// `container` under a hover layer of its `content` color.
pub fn hovered(container: Rgba, content: Rgba) -> Rgba {
    layer(container, content, HOVER)
}

/// `container` under a press layer of its `content` color.
pub fn pressed(container: Rgba, content: Rgba) -> Rgba {
    layer(container, content, PRESS)
}

/// `color` at disabled content emphasis. The emphasis multiplies the
/// color's own alpha, so an already translucent role stays proportionally
/// fainter.
pub fn disabled(color: Rgba) -> Rgba {
    Rgba {
        a: color.a * DISABLED_CONTENT,
        ..color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba(r: f32, g: f32, b: f32, a: f32) -> Rgba {
        Rgba { r, g, b, a }
    }

    fn close(left: Rgba, right: Rgba) -> bool {
        [
            (left.r, right.r),
            (left.g, right.g),
            (left.b, right.b),
            (left.a, right.a),
        ]
        .into_iter()
        .all(|(x, y)| (x - y).abs() < 1e-6)
    }

    #[test]
    fn layer_opacities_match_the_pinned_m3e_tokens() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/m3e-sys-tokens.json")).unwrap();
        let reference = &fixture["state_layer_opacity"];
        for (opacity, name) in [
            (HOVER, "hover"),
            (FOCUS, "focus"),
            (PRESS, "pressed"),
            (DRAG, "dragged"),
        ] {
            assert_eq!(opacity, reference[name].as_f64().unwrap() as f32, "{name}");
        }
        assert_eq!(
            DISABLED_CONTENT,
            fixture["disabled_alpha"].as_f64().unwrap() as f32
        );
    }

    #[test]
    fn disabled_multiplies_alpha() {
        assert_eq!(disabled(rgba(1., 0., 0., 1.)).a, DISABLED_CONTENT);
        assert_eq!(disabled(rgba(1., 0., 0., 0.5)).a, 0.5 * DISABLED_CONTENT);
        assert_eq!(disabled(rgba(1., 0., 0., 0.)).a, 0.);
    }

    #[test]
    fn layer_over_an_opaque_container_is_a_linear_mix() {
        let container = rgba(1., 1., 1., 1.);
        let content = rgba(0., 0., 0., 1.);
        assert!(close(
            hovered(container, content),
            rgba(0.92, 0.92, 0.92, 1.)
        ));
        assert!(close(pressed(container, content), rgba(0.9, 0.9, 0.9, 1.)));
    }

    #[test]
    fn layer_over_a_transparent_container_keeps_its_hue() {
        let content = rgba(0.2, 0.4, 0.6, 1.);
        let result = layer(rgba(0., 0., 0., 0.), content, HOVER);
        assert!(close(result, rgba(0.2, 0.4, 0.6, HOVER)));
        assert_eq!(layer(rgba(0., 0., 0., 0.), content, 0.).a, 0.);
    }

    #[test]
    fn layer_opacity_multiplies_content_alpha() {
        let result = layer(rgba(0., 0., 0., 0.), rgba(1., 1., 1., 0.5), PRESS);
        assert!((result.a - 0.05).abs() < 1e-6);
    }
}
