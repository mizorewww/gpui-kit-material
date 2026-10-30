//! The shape library against a dump of the unmodified Compose sources on the
//! JVM (provenance in the fixture's `_source`): every outline, the factories,
//! morphs at fixed progress, and all 1190 ordered pairs' cubic counts and
//! checksums, which pin how corner matching resolves near ties.
#![cfg(not(target_family = "wasm"))]
use gpui_kit_material::shapes::{
    CornerRounding, Cubic, Morph, MorphMatching, RoundedPolygon, ShapeKind,
};
use serde_json::Value;

/// As the reference's own port: one part in ten thousand of the 0-1 box.
const TOLERANCE: f64 = 1e-4;

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/compose-material-shapes.json"))
        .expect("the fixture is JSON")
}

fn kind(name: &str) -> ShapeKind {
    let named = |kind: &&ShapeKind| format!("{kind:?}") == name;
    *ShapeKind::ALL
        .iter()
        .find(named)
        .unwrap_or_else(|| panic!("{name} is a shape"))
}

fn numbers(value: &Value) -> Vec<f64> {
    value
        .as_array()
        .expect("an array of numbers")
        .iter()
        .map(|v| v.as_f64().expect("a number"))
        .collect()
}

fn assert_close(actual: &[f64], expected: &[f64], what: &str) {
    assert_eq!(actual.len(), expected.len(), "{what}: length");
    for (i, (a, e)) in actual.iter().zip(expected).enumerate() {
        assert!((a - e).abs() <= TOLERANCE, "{what}[{i}]: {a} vs {e}");
    }
}

fn assert_cubics(actual: &[Cubic], expected: &Value, what: &str) {
    let expected = expected.as_array().expect("cubics");
    assert_eq!(actual.len(), expected.len(), "{what}: cubic count");
    for (i, (cubic, expected)) in actual.iter().zip(expected).enumerate() {
        let points: Vec<f64> = cubic.points.iter().map(|v| f64::from(*v)).collect();
        assert_close(&points, &numbers(expected), &format!("{what} cubic {i}"));
    }
}

fn floats(values: [f32; 4]) -> Vec<f64> {
    values.iter().map(|v| f64::from(*v)).collect()
}

fn assert_polygon(polygon: &RoundedPolygon, reference: &Value, name: &str) {
    assert_cubics(polygon.cubics(), &reference["cubics"], name);
    let (x, y) = polygon.center();
    assert_close(
        &[f64::from(x), f64::from(y)],
        &numbers(&reference["center"]),
        &format!("{name} center"),
    );
    assert_close(
        &floats(polygon.bounds(true)),
        &numbers(&reference["bounds"]),
        &format!("{name} bounds"),
    );
    assert_close(
        &floats(polygon.max_bounds()),
        &numbers(&reference["maxBounds"]),
        &format!("{name} maxBounds"),
    );
}

#[test]
fn the_35_shapes_match_compose_in_order_and_outline() {
    let fixture = fixture();
    let presets = fixture["presets"].as_array().expect("presets");
    assert_eq!(presets.len(), 35);
    assert_eq!(ShapeKind::ALL.len(), 35);
    for (kind, reference) in ShapeKind::ALL.iter().zip(presets) {
        let name = reference["name"].as_str().expect("a name");
        assert_eq!(format!("{kind:?}"), name, "MaterialShapes.kt order");
        assert_polygon(kind.polygon(), reference, name);
    }
}

#[test]
fn the_factories_match_graphics_shapes() {
    let fixture = fixture();
    let by_name = |name: &str| {
        fixture["polygons"]
            .as_array()
            .expect("polygons")
            .iter()
            .find(|p| p["name"] == name)
            .unwrap_or_else(|| panic!("{name} in the fixture"))
    };
    // 1.0.1's default center divides by the coordinate count twice over; it
    // was fixed upstream and this port has the fix, so these two take the
    // dump's center and are held to the outline it produces.
    let dumped_center = |name: &str| {
        let center = numbers(&by_name(name)["center"]);
        Some((center[0] as f32, center[1] as f32))
    };
    let origin = (0., 0.);
    let cases = [
        (
            "pentagon.rounded",
            RoundedPolygon::regular(5, 1., origin, CornerRounding::smoothed(0.2, 0.5), None),
        ),
        ("circle.12", RoundedPolygon::circle(12, 2., (1., -1.))),
        (
            "rectangle.rounded",
            RoundedPolygon::rectangle(2., 1., CornerRounding::smoothed(0.3, 0.7), None, origin),
        ),
        (
            "rectangle.perVertex",
            RoundedPolygon::rectangle(
                2.,
                2.,
                CornerRounding::UNROUNDED,
                Some(&[
                    CornerRounding::new(0.1),
                    CornerRounding::smoothed(0.4, 1.),
                    CornerRounding::UNROUNDED,
                    CornerRounding::smoothed(0.25, 0.25),
                ]),
                origin,
            ),
        ),
        (
            "star.8",
            RoundedPolygon::star(
                8,
                1.,
                0.6,
                CornerRounding::smoothed(0.15, 0.3),
                Some(CornerRounding::new(0.05)),
                origin,
            ),
        ),
        ("pill.smooth", RoundedPolygon::pill(3., 1., 0.6, origin)),
        (
            "custom.concave",
            RoundedPolygon::from_vertices(
                &[(0., 0.), (1., 0.), (1., 1.), (0.5, 0.4), (0., 1.)],
                CornerRounding::smoothed(0.1, 0.5),
                None,
                dumped_center("custom.concave"),
            ),
        ),
        (
            "custom.negativeOnly",
            RoundedPolygon::from_vertices(
                &[(-3., -3.), (-1., -3.), (-2., -1.)],
                CornerRounding::UNROUNDED,
                None,
                dumped_center("custom.negativeOnly"),
            ),
        ),
    ];
    for (name, polygon) in cases {
        assert_polygon(&polygon.expect("valid"), by_name(name), name);
    }
}

#[test]
fn morphs_match_compose_at_fixed_progress() {
    let fixture = fixture();
    for reference in fixture["morphs"].as_array().expect("morphs") {
        let (start, end) = (
            reference["start"].as_str().expect("a name"),
            reference["end"].as_str().expect("a name"),
        );
        let morph = Morph::with_matching(
            kind(start).polygon(),
            kind(end).polygon(),
            MorphMatching::Compose1_5,
        );
        let label = format!("{start} -> {end}");
        for step in reference["steps"].as_array().expect("steps") {
            let progress = step["progress"].as_f64().expect("a number") as f32;
            assert_cubics(
                &morph.as_cubics(progress),
                &step["cubics"],
                &format!("{label} @ {progress}"),
            );
        }
        assert_close(
            &floats(morph.bounds(true)),
            &numbers(&reference["bounds"]),
            &format!("{label} bounds"),
        );
        assert_close(
            &floats(morph.max_bounds()),
            &numbers(&reference["maxBounds"]),
            &format!("{label} maxBounds"),
        );
    }
}

#[test]
fn every_ordered_pair_matches_compose_in_count_and_checksum() {
    let fixture = fixture();
    let pairs = fixture["morphPairs"].as_array().expect("pairs");
    assert_eq!(pairs.len(), 35 * 34);
    let mut mismatches = Vec::new();
    for pair in pairs {
        let (start, end) = (
            pair[0].as_str().expect("a name"),
            pair[1].as_str().expect("a name"),
        );
        let (count, sum) = (
            pair[2].as_u64().expect("a count") as usize,
            pair[3].as_f64().expect("a checksum"),
        );
        let cubics = Morph::with_matching(
            kind(start).polygon(),
            kind(end).polygon(),
            MorphMatching::Compose1_5,
        )
        .as_cubics(0.5);
        let actual: f64 = cubics.iter().flat_map(|c| c.points).map(f64::from).sum();
        if cubics.len() != count || (actual - sum).abs() > 0.01 {
            mismatches.push(format!(
                "{start} -> {end}: {}/{count} cubics, sum {actual}/{sum}",
                cubics.len()
            ));
        }
    }
    assert!(mismatches.is_empty(), "{mismatches:#?}");
}

/// Twice the signed area enclosed by a flattened outline.
fn signed_area(cubics: &[Cubic]) -> f32 {
    let mut area = 0.;
    let mut previous = cubics[0].anchor0();
    for cubic in cubics {
        for step in 1..=8 {
            let at = cubic.point_at(step as f32 / 8.);
            area += previous.0 * at.1 - at.0 * previous.1;
            previous = at;
        }
    }
    area
}

/// Morphing must not twist: the outline keeps its winding, and so encloses
/// real area, all the way through every pair.
#[test]
fn no_morph_turns_inside_out_at_any_progress() {
    let mut smallest = f32::MAX;
    for from in ShapeKind::ALL {
        let start = signed_area(from.polygon().cubics()).signum();
        for to in ShapeKind::ALL {
            for step in 0..=10 {
                let progress = step as f32 / 10.;
                let cubics = from.morph_to(to).as_cubics(progress);
                let area = signed_area(&cubics);
                assert_eq!(
                    area.signum(),
                    start,
                    "{from:?} -> {to:?} at {progress} winds the other way"
                );
                smallest = smallest.min(area.abs());
            }
        }
    }
    assert!(
        smallest > 0.05,
        "a morph collapsed: twice its area is {smallest}"
    );
}

/// A morph's outline, flattened for geometry checks.
fn flattened(cubics: &[Cubic]) -> Vec<(f32, f32)> {
    let mut outline = Vec::with_capacity(cubics.len() * 8);
    for cubic in cubics {
        outline.extend((0..8).map(|i| cubic.point_at(i as f32 / 8.)));
    }
    outline
}

/// How many pairs of non-adjacent segments of `outline` properly cross.
fn crossings(outline: &[(f32, f32)]) -> usize {
    let side = |a: (f32, f32), b: (f32, f32), c: (f32, f32)| {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    };
    let n = outline.len();
    let mut count = 0;
    for i in 0..n {
        let (a, b) = (outline[i], outline[(i + 1) % n]);
        for j in i + 2..n {
            if i == 0 && j == n - 1 {
                continue;
            }
            let (c, d) = (outline[j], outline[(j + 1) % n]);
            if (a.0.max(b.0) < c.0.min(d.0))
                || (c.0.max(d.0) < a.0.min(b.0))
                || (a.1.max(b.1) < c.1.min(d.1))
                || (c.1.max(d.1) < a.1.min(b.1))
            {
                continue;
            }
            let (d1, d2) = (side(a, b, c), side(a, b, d));
            let (d3, d4) = (side(c, d, a), side(c, d, b));
            if d1 * d2 < -1e-9 && d3 * d4 < -1e-9 {
                count += 1;
            }
        }
    }
    count
}

/// How many ordered pairs self-intersect somewhere in the morph.
fn self_intersecting_pairs(matching: MorphMatching) -> Vec<(ShapeKind, ShapeKind, f32, usize)> {
    let mut found = Vec::new();
    for from in ShapeKind::ALL {
        for to in ShapeKind::ALL {
            if from == to {
                continue;
            }
            let morph = Morph::with_matching(from.polygon(), to.polygon(), matching);
            let worst = [0.25_f32, 0.5, 0.75]
                .into_iter()
                .map(|progress| (progress, crossings(&flattened(&morph.as_cubics(progress)))))
                .max_by_key(|entry| entry.1);
            if let Some((progress, count)) = worst
                && count > 0
            {
                found.push((from, to, progress, count));
            }
        }
    }
    found
}

#[test]
#[ignore = "survey: prints self-intersections per matching; run with --nocapture"]
fn survey_self_intersections() {
    for matching in [MorphMatching::Current, MorphMatching::Compose1_5] {
        let found = self_intersecting_pairs(matching);
        let total: usize = found.iter().map(|entry| entry.3).sum();
        println!(
            "{matching:?}: {} of 1190 pairs, {total} crossings",
            found.len()
        );
        for entry in found.iter().take(40) {
            println!("  {entry:?}");
        }
    }
}

/// The reason `Current` is the default: 1.0.1's matching self-intersects on 13
/// pairs (the survey above), and it does on none.
#[test]
fn no_morph_of_the_library_self_intersects() {
    for from in ShapeKind::ALL {
        for to in ShapeKind::ALL {
            let cubics = from.morph_to(to).as_cubics(0.5);
            let mut outline = Vec::with_capacity(cubics.len() * 3);
            for cubic in &cubics {
                outline.extend((0..3).map(|i| cubic.point_at(i as f32 / 3.)));
            }
            assert_eq!(crossings(&outline), 0, "{from:?} -> {to:?}");
        }
    }
}

#[test]
fn morph_endpoints_reproduce_both_shapes() {
    // Sampling each outline at many points, a morph's ends must lie on it.
    fn nearest(point: (f32, f32), outline: &[(f32, f32)]) -> f32 {
        outline
            .iter()
            .map(|q| (q.0 - point.0).hypot(q.1 - point.1))
            .fold(f32::MAX, f32::min)
    }
    let flatten = |cubics: &[Cubic]| -> Vec<(f32, f32)> {
        cubics
            .iter()
            .flat_map(|c| {
                (0..48)
                    .map(|i| c.point_at(i as f32 / 48.))
                    .collect::<Vec<_>>()
            })
            .collect()
    };
    let pairs = [
        (ShapeKind::Circle, ShapeKind::Heart),
        (ShapeKind::PixelTriangle, ShapeKind::Puffy),
        (ShapeKind::Boom, ShapeKind::Bun),
        (ShapeKind::Ghostish, ShapeKind::Clover8Leaf),
    ];
    for (from, to) in pairs {
        let morph = from.morph_to(to);
        for (progress, polygon) in [(0., from.polygon()), (1., to.polygon())] {
            let shape = flatten(polygon.cubics());
            for point in flatten(&morph.as_cubics(progress)) {
                let distance = nearest(point, &shape);
                assert!(
                    distance < 0.01,
                    "{from:?} -> {to:?} at {progress}: {distance}"
                );
            }
        }
    }
}

#[test]
fn shapes_are_distinct_and_normalized() {
    for (i, a) in ShapeKind::ALL.iter().enumerate() {
        let [left, top, right, bottom] = a.polygon().bounds(true);
        assert!(
            (right - left).max(bottom - top) > 0.999 && (right - left).max(bottom - top) < 1.001,
            "{a:?} fills its box"
        );
        assert!(
            left > -0.001 && top > -0.001 && right < 1.001 && bottom < 1.001,
            "{a:?} stays in its box"
        );
        for b in &ShapeKind::ALL[i + 1..] {
            assert_ne!(a.polygon().cubics(), b.polygon().cubics(), "{a:?} {b:?}");
        }
    }
    let morph = ShapeKind::Circle.morph_to(ShapeKind::Circle);
    assert!(!morph.is_empty(), "a shape morphs to itself");
}
