//! The Expressive motion scheme's springs, from m3e.kyralabs.dev.
//!
//! Source: kyra-labs/m3exp-web (<https://github.com/kyra-labs/m3exp-web>) at
//! `3b1e0047c0021eab7b4c0128fa1bec95c76ca746`,
//! `packages/m3e-tokens/src/generated/tokens.css`
//! (`--m3e-sys-motion-spring-expressive-{fast,default,slow}-{spatial,effects}-*`),
//! generated from Compose Material3 1.5.0-beta01
//! `tokens/ExpressiveMotionTokens.kt` @ androidx
//! `19991f2c0858aadfcb442ce02b2c8e4743c96898` (L22-L33).
//!
//! m3e evaluates these as analytic springs (its `SpringSimulation` port) over
//! Compose's settle estimate; its CSS `linear()` easings are ≤ 40-stop
//! approximations of the same curves. The crate evaluates the springs exactly
//! through [`Spring`], and `conformance/fixtures/m3e-motion.json` (written by
//! `scripts/extract-m3e-tokens.py`) checks parameters, durations and every
//! `linear()` stop. The md3.io (floor/material) sampled curves these replaced
//! are retained only as secondary evidence in
//! `conformance/fixtures/md3-expressive-browser.json`.
//!
//! Effects springs are identical to the Standard scheme's.
use super::Spring;

/// Small components: a button's shape, a switch thumb, a selection indicator.
/// Overshoots visibly (damping ratio 0.6).
pub const FAST_SPATIAL: Spring = Spring {
    damping: 0.6,
    stiffness: 800.,
};
/// Partly covering surfaces.
pub const DEFAULT_SPATIAL: Spring = Spring {
    damping: 0.8,
    stiffness: 380.,
};
/// Full-screen surfaces.
pub const SLOW_SPATIAL: Spring = Spring {
    damping: 0.8,
    stiffness: 200.,
};
pub const FAST_EFFECTS: Spring = super::FAST_EFFECTS;
pub const DEFAULT_EFFECTS: Spring = super::DEFAULT_EFFECTS;
pub const SLOW_EFFECTS: Spring = super::SLOW_EFFECTS;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion::{MotionScheme, MotionToken};

    fn reference() -> serde_json::Value {
        serde_json::from_str(include_str!(
            "../../../../conformance/fixtures/m3e-motion.json"
        ))
        .unwrap()
    }

    fn token(name: &str) -> MotionToken {
        match name {
            "fast-spatial" => MotionToken::FastSpatial,
            "default-spatial" => MotionToken::DefaultSpatial,
            "slow-spatial" => MotionToken::SlowSpatial,
            "fast-effects" => MotionToken::FastEffects,
            "default-effects" => MotionToken::DefaultEffects,
            "slow-effects" => MotionToken::SlowEffects,
            _ => panic!("unexpected reference token {name}"),
        }
    }

    #[test]
    fn both_schemes_match_the_pinned_m3e_springs() {
        let reference = reference();
        assert_eq!(
            reference["source_commit"],
            "3b1e0047c0021eab7b4c0128fa1bec95c76ca746"
        );
        for (name, scheme) in [
            ("standard", MotionScheme::Standard),
            ("expressive", MotionScheme::Expressive),
        ] {
            let springs = reference["springs"][name].as_array().unwrap();
            assert_eq!(springs.len(), 6);
            let mut seen = std::collections::BTreeSet::new();
            for entry in springs {
                let label = entry["token"].as_str().unwrap();
                assert!(seen.insert(label), "duplicate {name} {label}");
                let curve = scheme.resolve(token(label));
                let spring = curve.spring();
                assert_eq!(
                    spring.damping,
                    entry["damping_ratio"].as_f64().unwrap() as f32,
                    "{name} {label} damping"
                );
                assert_eq!(
                    spring.stiffness,
                    entry["stiffness"].as_f64().unwrap() as f32,
                    "{name} {label} stiffness"
                );
                assert_eq!(
                    curve.duration().as_millis(),
                    entry["duration_ms"].as_u64().unwrap() as u128,
                    "{name} {label} Compose settle duration"
                );
                // Each linear() stop is the spring sampled on a whole
                // millisecond and rounded to 4 digits.
                let stops = entry["linear_stops"].as_array().unwrap();
                assert!(stops.len() >= 2);
                for stop in stops {
                    let progress = stop[0].as_f64().unwrap() as f32;
                    let expected = stop[1].as_f64().unwrap() as f32;
                    let actual = curve.ease()(progress);
                    assert!(
                        (actual - expected).abs() < 0.0005,
                        "{name} {label} at {progress}: {actual} vs m3e {expected}"
                    );
                }
            }
        }
    }

    #[test]
    fn expressive_spatial_overshoots_and_effects_stay_bounded() {
        for spring in [FAST_SPATIAL, DEFAULT_SPATIAL, SLOW_SPATIAL] {
            let ease = spring.ease();
            assert!((0..=1000).any(|t| ease(t as f32 / 1000.) > 1.01));
            assert_eq!(ease(0.), 0.);
            assert_eq!(ease(1.), 1.);
            assert_eq!(ease(2.), 1.);
            assert_eq!(ease(f32::NAN), 0.);
        }
        for spring in [FAST_EFFECTS, DEFAULT_EFFECTS, SLOW_EFFECTS] {
            let values = (0..=1000)
                .map(|t| spring.ease()(t as f32 / 1000.))
                .collect::<Vec<_>>();
            assert!(values.iter().all(|v| (0. ..=1.).contains(v)));
            assert!(values.windows(2).all(|p| p[0] <= p[1]));
        }
    }
}
