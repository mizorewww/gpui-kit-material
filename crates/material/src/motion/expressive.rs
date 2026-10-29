//! Expressive CSS spring curves from floor/material at c910072f3fc5b660276731d8a935d7186f288364.
//! Source: src/styles/abstract/_variables.scss. See LICENSE-material for the MIT notice.
//! Keep the sampled linear interpolation: an analytic spring is not the same curve.
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampledSpring {
    pub duration: Duration,
    samples: [f32; 25],
}

impl SampledSpring {
    /// CSS linear() interpolation between uniformly spaced reference samples.
    pub fn position(self, progress: f32) -> f32 {
        let progress = progress.clamp(0., 1.);
        if progress >= 1. {
            return 1.;
        }
        let position = progress * 24.;
        let index = position.floor() as usize;
        let fraction = position - index as f32;
        self.samples[index] + (self.samples[index + 1] - self.samples[index]) * fraction
    }
}

pub const FAST_SPATIAL: SampledSpring = SampledSpring {
    duration: Duration::from_millis(425),
    samples: [
        0.0, 0.102, 0.323, 0.57, 0.784, 0.942, 1.039, 1.085, 1.094, 1.082, 1.06, 1.037, 1.018,
        1.004, 0.995, 0.992, 0.991, 0.992, 0.995, 0.997, 0.999, 1.0, 1.001, 1.001, 1.0,
    ],
};

pub const DEFAULT_SPATIAL: SampledSpring = SampledSpring {
    duration: Duration::from_millis(450),
    samples: [
        0.0, 0.055, 0.18, 0.332, 0.483, 0.619, 0.733, 0.823, 0.891, 0.94, 0.973, 0.994, 1.007,
        1.013, 1.015, 1.015, 1.013, 1.011, 1.009, 1.006, 1.005, 1.003, 1.002, 1.001, 1.0,
    ],
};

pub const SLOW_SPATIAL: SampledSpring = SampledSpring {
    duration: Duration::from_millis(625),
    samples: [
        0.0, 0.056, 0.182, 0.335, 0.488, 0.624, 0.738, 0.827, 0.895, 0.943, 0.975, 0.996, 1.007,
        1.013, 1.015, 1.015, 1.013, 1.011, 1.008, 1.006, 1.004, 1.003, 1.002, 1.001, 1.0,
    ],
};

pub const FAST_EFFECTS: SampledSpring = SampledSpring {
    duration: Duration::from_millis(175),
    samples: [
        0.0, 0.075, 0.227, 0.39, 0.537, 0.657, 0.751, 0.822, 0.874, 0.912, 0.939, 0.958, 0.971,
        0.98, 0.987, 0.991, 0.994, 0.996, 0.997, 0.998, 0.999, 0.999, 0.999, 1.0, 1.0,
    ],
};

pub const DEFAULT_EFFECTS: SampledSpring = SampledSpring {
    duration: Duration::from_millis(250),
    samples: [
        0.0, 0.066, 0.203, 0.355, 0.496, 0.616, 0.713, 0.788, 0.845, 0.888, 0.92, 0.943, 0.96,
        0.971, 0.98, 0.986, 0.99, 0.993, 0.995, 0.997, 0.998, 0.998, 0.999, 0.999, 1.0,
    ],
};

pub const SLOW_EFFECTS: SampledSpring = SampledSpring {
    duration: Duration::from_millis(350),
    samples: [
        0.0, 0.065, 0.2, 0.351, 0.491, 0.611, 0.708, 0.783, 0.841, 0.885, 0.917, 0.941, 0.958,
        0.97, 0.979, 0.985, 0.99, 0.993, 0.995, 0.997, 0.998, 0.998, 0.999, 0.999, 1.0,
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curves_match_browser_computed_transforms() {
        let reference: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../conformance/fixtures/md3-expressive-browser.json"
        ))
        .unwrap();
        let curves = reference["curves"].as_array().unwrap();
        assert_eq!(curves.len(), 6);
        let mut names = std::collections::BTreeSet::new();
        for curve in curves {
            let token = curve["token"].as_str().unwrap();
            assert!(names.insert(token), "duplicate reference curve {token}");
            assert_eq!(curve["samples"].as_array().unwrap().len(), 29);
            let spring = match token {
                "fast-spatial" => FAST_SPATIAL,
                "default-spatial" => DEFAULT_SPATIAL,
                "slow-spatial" => SLOW_SPATIAL,
                "fast-effects" => FAST_EFFECTS,
                "default-effects" => DEFAULT_EFFECTS,
                "slow-effects" => SLOW_EFFECTS,
                _ => panic!("unexpected reference token {token}"),
            };
            assert_eq!(
                spring.duration.as_millis(),
                curve["duration_ms"].as_u64().unwrap() as u128
            );
            for sample in curve["samples"].as_array().unwrap() {
                let progress = sample[0].as_f64().unwrap() as f32;
                let expected = sample[1].as_f64().unwrap() as f32;
                let actual = spring.position(progress);
                assert!(
                    (actual - expected).abs() < 0.00001,
                    "{token} at {progress}: {actual} vs browser {expected}"
                );
            }
        }
    }

    #[test]
    fn spatial_overshoot_is_preserved_and_effects_are_bounded() {
        for spring in [FAST_SPATIAL, DEFAULT_SPATIAL, SLOW_SPATIAL] {
            assert!((0..=1000).any(|t| spring.position(t as f32 / 1000.) > 1.01));
            assert_eq!(spring.position(-1.), 0.);
            assert_eq!(spring.position(1.), 1.);
            assert_eq!(spring.position(2.), 1.);
        }
        for spring in [FAST_EFFECTS, DEFAULT_EFFECTS, SLOW_EFFECTS] {
            let values = (0..=1000)
                .map(|t| spring.position(t as f32 / 1000.))
                .collect::<Vec<_>>();
            assert!(values.iter().all(|v| (0. ..=1.).contains(v)));
            assert!(values.windows(2).all(|p| p[0] <= p[1]));
        }
    }
}
