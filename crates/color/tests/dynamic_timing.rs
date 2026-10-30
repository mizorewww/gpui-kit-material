//! How long a whole seed scheme takes to generate. Ignored by default because
//! only a release build means anything:
//! `cargo test -p gpui-kit-material-color --release --test dynamic_timing -- --ignored --nocapture`
use gpui_kit_material_color::dynamic::{SeedScheme, SpecVersion, Variant};
use std::time::Instant;

#[test]
#[ignore = "prints release timings"]
fn seed_scheme_generation_cost() {
    for spec in [SpecVersion::Spec2021, SpecVersion::Spec2025] {
        for variant in Variant::ALL {
            let mut samples = Vec::new();
            for seed in [0x6750a4, 0xff0000, 0xffff00, 0x00ff00, 0x0000ff, 0x808080] {
                for dark in [false, true] {
                    for contrast in [0., 0.5, 1.] {
                        let seed = SeedScheme::new(seed)
                            .variant(variant)
                            .dark(dark)
                            .contrast(contrast)
                            .spec(spec);
                        let start = Instant::now();
                        std::hint::black_box(seed.scheme());
                        samples.push(start.elapsed().as_secs_f64() * 1000.);
                    }
                }
            }
            samples.sort_by(f64::total_cmp);
            println!(
                "{spec:?} {variant:?}: median {:.2} ms, max {:.2} ms over {} schemes",
                samples[samples.len() / 2],
                samples[samples.len() - 1],
                samples.len()
            );
        }
    }
}
