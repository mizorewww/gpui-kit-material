//! Matched shadow lists for CSS-compatible button elevation interpolation.

use gpui::{BoxShadow, black, px};
use gpui_base::Interpolate;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ShadowPair([BoxShadow; 2]);

impl ShadowPair {
    pub(super) fn level(level: u8) -> Self {
        match level {
            1 => Self([
                BoxShadow::new(px(0.), px(1.), black().opacity(0.3)).blur_radius(px(2.)),
                BoxShadow::new(px(0.), px(1.), black().opacity(0.15))
                    .blur_radius(px(3.))
                    .spread_radius(px(1.)),
            ]),
            2 => Self([
                BoxShadow::new(px(0.), px(1.), black().opacity(0.3)).blur_radius(px(2.)),
                BoxShadow::new(px(0.), px(2.), black().opacity(0.15))
                    .blur_radius(px(6.))
                    .spread_radius(px(2.)),
            ]),
            // CSS pads `none` with transparent, zero-geometry shadows before
            // interpolating lists. Keeping both entries also permits reversal.
            _ => Self(std::array::from_fn(|_| {
                BoxShadow::new(px(0.), px(0.), black().opacity(0.))
            })),
        }
    }

    pub(super) fn into_shadows(self) -> Vec<BoxShadow> {
        self.0.into_iter().collect()
    }
}

impl Interpolate for ShadowPair {
    fn interpolate(&self, target: &Self, progress: f32) -> Self {
        Self(std::array::from_fn(|index| {
            let from = &self.0[index];
            let to = &target.0[index];
            BoxShadow {
                color: from.color.interpolate(&to.color, progress),
                offset: from.offset.interpolate(&to.offset, progress),
                blur_radius: from.blur_radius.interpolate(&to.blur_radius, progress),
                spread_radius: from.spread_radius.interpolate(&to.spread_radius, progress),
                inset: false,
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_shadow(shadow: &BoxShadow, y: f32, blur: f32, spread: f32, alpha: f32) {
        assert_eq!(shadow.offset.x, px(0.));
        assert_eq!(shadow.offset.y, px(y));
        assert_eq!(shadow.blur_radius, px(blur));
        assert_eq!(shadow.spread_radius, px(spread));
        assert_eq!(shadow.color, black().opacity(alpha));
        assert!(!shadow.inset);
    }

    #[test]
    fn none_to_level_one_interpolates_geometry_and_alpha() {
        let none = ShadowPair::level(0);
        for shadow in none.clone().into_shadows() {
            assert_shadow(&shadow, 0., 0., 0., 0.);
        }
        let middle = none.interpolate(&ShadowPair::level(1), 0.5).into_shadows();
        assert_eq!(middle.len(), 2);
        assert_shadow(&middle[0], 0.5, 1., 0., 0.15);
        assert_shadow(&middle[1], 0.5, 1.5, 0.5, 0.075);
    }

    #[test]
    fn level_one_to_two_interpolates_geometry_and_preserves_alpha() {
        let middle = ShadowPair::level(1)
            .interpolate(&ShadowPair::level(2), 0.5)
            .into_shadows();
        assert_eq!(middle.len(), 2);
        assert_shadow(&middle[0], 1., 2., 0., 0.3);
        assert_shadow(&middle[1], 1.5, 4.5, 1.5, 0.15);
    }
}
