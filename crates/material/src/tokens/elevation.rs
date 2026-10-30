//! M3 elevation shadows, painted with [`ColorRole::SHADOW`].
//!
//! Each level is the reference two-layer box shadow: a key layer at 30% and
//! an ambient layer at 15% of the shadow role. The table is m3e's
//! `--m3e-sys-elevation-level*-shadow` (kyra-labs/m3exp-web
//! `3b1e0047c0021eab7b4c0128fa1bec95c76ca746`,
//! `packages/m3e-tokens/src/generated/tokens.css`), which m3e vendors from
//! Material Web 2.5.0 because Compose draws platform shadows; md3.io agrees.
//! Fixture: `tests/fixtures/m3e-sys-tokens.json`.
//!
//! | Level | Key (x/y/blur/spread @ 30%) | Ambient (x/y/blur/spread @ 15%) |
//! |-------|-----------------------------|---------------------------------|
//! | 0     | none                        | none                            |
//! | 1     | 0/1/2/0                     | 0/1/3/1                         |
//! | 2     | 0/1/2/0                     | 0/2/6/2                         |
//! | 3     | 0/1/3/0                     | 0/4/8/3                         |
//! | 4     | 0/2/3/0                     | 0/6/10/4                        |
//! | 5     | 0/4/4/0                     | 0/8/12/6                        |
use crate::color::ColorRole;
use crate::theme::color;
use gpui::{BoxShadow, Hsla, px};

/// The highest M3 elevation level.
pub const MAX_LEVEL: u8 = 5;

const KEY_OPACITY: f32 = 0.3;
const AMBIENT_OPACITY: f32 = 0.15;

// (y, blur, spread) of the key and ambient layers for levels 1..=5.
const GEOMETRY: [[(f32, f32, f32); 2]; MAX_LEVEL as usize] = [
    [(1., 2., 0.), (1., 3., 1.)],
    [(1., 2., 0.), (2., 6., 2.)],
    [(1., 3., 0.), (4., 8., 3.)],
    [(2., 3., 0.), (6., 10., 4.)],
    [(4., 4., 0.), (8., 12., 6.)],
];

fn shadow_color(opacity: f32) -> Hsla {
    let mut shadow = color(ColorRole::SHADOW);
    shadow.a *= opacity;
    shadow.into()
}

fn layer(y: f32, blur: f32, spread: f32, opacity: f32) -> BoxShadow {
    BoxShadow::new(px(0.), px(y), shadow_color(opacity))
        .blur_radius(px(blur))
        .spread_radius(px(spread))
}

/// Both layers of `level` (clamped to [`MAX_LEVEL`]), key first.
///
/// Level 0 is two transparent, zero-geometry entries: CSS pads `none` this
/// way before interpolating shadow lists, so a transition to or from level 0
/// keeps matched entries.
pub fn layers(level: u8) -> [BoxShadow; 2] {
    match level.min(MAX_LEVEL) {
        0 => std::array::from_fn(|_| layer(0., 0., 0., 0.)),
        level => {
            let [key, ambient] = GEOMETRY[level as usize - 1];
            [
                layer(key.0, key.1, key.2, KEY_OPACITY),
                layer(ambient.0, ambient.1, ambient.2, AMBIENT_OPACITY),
            ]
        }
    }
}

/// The shadow list of `level`; level 0 paints no shadow.
pub fn level(level: u8) -> Vec<BoxShadow> {
    if level == 0 {
        Vec::new()
    } else {
        layers(level).into()
    }
}

/// M3 level 0: flat, no shadow.
pub fn level0() -> Vec<BoxShadow> {
    level(0)
}

/// M3 level 1: elevated cards, elevated buttons at rest.
pub fn level1() -> Vec<BoxShadow> {
    level(1)
}

/// M3 level 2: hovered elevated controls, menus.
pub fn level2() -> Vec<BoxShadow> {
    level(2)
}

/// M3 level 3: snackbars, dialogs, FABs at rest.
pub fn level3() -> Vec<BoxShadow> {
    level(3)
}

/// M3 level 4: hovered FABs.
pub fn level4() -> Vec<BoxShadow> {
    level(4)
}

/// M3 level 5.
pub fn level5() -> Vec<BoxShadow> {
    level(5)
}

/// Legacy single-layer overlay shadow (0/12/32 at 16% of the shadow role).
///
/// This is not an M3 elevation level. It remains only for the existing
/// snackbar, which a later change moves to [`level3`]; new surfaces should
/// use a numbered level.
pub fn popover() -> Vec<BoxShadow> {
    vec![BoxShadow::new(px(0.), px(12.), shadow_color(0.16)).blur_radius(px(32.))]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_layer(shadow: &BoxShadow, y: f32, blur: f32, spread: f32, opacity: f32) {
        let mut expected = color(ColorRole::SHADOW);
        expected.a *= opacity;
        assert_eq!(shadow.offset.x, px(0.));
        assert_eq!(shadow.offset.y, px(y));
        assert_eq!(shadow.blur_radius, px(blur));
        assert_eq!(shadow.spread_radius, px(spread));
        assert_eq!(shadow.color, Hsla::from(expected));
        assert!(!shadow.inset);
    }

    #[test]
    fn levels_match_the_m3_two_layer_shadows() {
        assert!(level0().is_empty());
        let expected = [
            [(1., 2., 0.), (1., 3., 1.)],
            [(1., 2., 0.), (2., 6., 2.)],
            [(1., 3., 0.), (4., 8., 3.)],
            [(2., 3., 0.), (6., 10., 4.)],
            [(4., 4., 0.), (8., 12., 6.)],
        ];
        let levels = [level1(), level2(), level3(), level4(), level5()];
        for (shadows, [key, ambient]) in levels.iter().zip(expected) {
            assert_eq!(shadows.len(), 2);
            assert_layer(&shadows[0], key.0, key.1, key.2, 0.3);
            assert_layer(&shadows[1], ambient.0, ambient.1, ambient.2, 0.15);
        }
        assert_eq!(level(9), level5(), "levels above 5 clamp");
    }

    #[test]
    fn levels_match_the_pinned_m3e_shadows() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/m3e-sys-tokens.json")).unwrap();
        let levels = fixture["elevation"].as_array().unwrap();
        assert_eq!(levels.len(), MAX_LEVEL as usize + 1);
        assert!(levels[0]["layers"].as_array().unwrap().is_empty());
        for level in 1..=MAX_LEVEL {
            let reference = levels[level as usize]["layers"].as_array().unwrap();
            let shadows = super::level(level);
            assert_eq!(shadows.len(), reference.len());
            for (shadow, layer) in shadows.iter().zip(reference) {
                let value = |key: &str| layer[key].as_f64().unwrap() as f32;
                assert_eq!(value("x"), 0.);
                assert_layer(
                    shadow,
                    value("y"),
                    value("blur"),
                    value("spread"),
                    value("opacity"),
                );
            }
        }
    }

    #[test]
    fn level_zero_pads_two_transparent_entries() {
        for shadow in layers(0) {
            assert_layer(&shadow, 0., 0., 0., 0.);
        }
    }
}
