//! Matched shadow lists for CSS-compatible button elevation interpolation.

use crate::tokens::elevation;
use gpui::BoxShadow;
use gpui_base::Interpolate;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ShadowPair([BoxShadow; 2]);

impl ShadowPair {
    /// The shared M3 elevation layers. Level 0 is padded with transparent,
    /// zero-geometry entries so transitions keep matched lists and reverse.
    pub(super) fn level(level: u8) -> Self {
        Self(elevation::layers(level))
    }

    /// What to paint: a fully transparent entry shows nothing, but would still be
    /// a primitive in the scene of every frame, as a flat button's both are.
    pub(super) fn into_shadows(self) -> Vec<BoxShadow> {
        self.0
            .into_iter()
            .filter(|shadow| shadow.color.a > 0.)
            .collect()
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
    use gpui::{black, px};

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
        assert!(
            none.clone().into_shadows().is_empty(),
            "nothing to paint at rest, while the pair stays matched to interpolate"
        );
        for shadow in &none.0 {
            assert_shadow(shadow, 0., 0., 0., 0.);
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
