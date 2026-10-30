// Copyright 2024 The Android Open Source Project
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// at http://www.apache.org/licenses/LICENSE-2.0 (see LICENSE-androidx next to
// this file). Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//
// Ported to Rust from Compose Material3's MaterialShapes.kt
// (compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/).
//! The M3 Expressive shape library: the outlines of Compose's
//! `MaterialShapes`. Every value here is that source's; neither the Figma kit
//! nor md3.io defines these outlines, and m3e ports the same file.
use super::morph::Morph;
use super::polygon::{CornerRounding, Pt, RoundedPolygon, angle_degrees, cos, pt, sin};
use std::sync::OnceLock;

/// One of the 35 shapes of the M3 Expressive shape library.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)] // Each variant is named for the shape it draws.
pub enum ShapeKind {
    Circle,
    Square,
    Slanted,
    Arch,
    Fan,
    Arrow,
    SemiCircle,
    Oval,
    Pill,
    Triangle,
    Diamond,
    ClamShell,
    Pentagon,
    Gem,
    Sunny,
    VerySunny,
    Cookie4Sided,
    Cookie6Sided,
    Cookie7Sided,
    Cookie9Sided,
    Cookie12Sided,
    Ghostish,
    Clover4Leaf,
    Clover8Leaf,
    Burst,
    SoftBurst,
    Boom,
    SoftBoom,
    Flower,
    Puffy,
    PuffyDiamond,
    PixelCircle,
    PixelTriangle,
    Bun,
    Heart,
}

const COUNT: usize = 35;

static POLYGONS: [OnceLock<RoundedPolygon>; COUNT] = [const { OnceLock::new() }; COUNT];
// Boxed: most pairs are never asked for, and a `Morph` is several words wide.
static MORPHS: [OnceLock<Box<Morph>>; COUNT * COUNT] = [const { OnceLock::new() }; COUNT * COUNT];

impl ShapeKind {
    /// Every shape, in the order of Compose's `MaterialShapes`.
    pub const ALL: [Self; COUNT] = [
        Self::Circle,
        Self::Square,
        Self::Slanted,
        Self::Arch,
        Self::Fan,
        Self::Arrow,
        Self::SemiCircle,
        Self::Oval,
        Self::Pill,
        Self::Triangle,
        Self::Diamond,
        Self::ClamShell,
        Self::Pentagon,
        Self::Gem,
        Self::Sunny,
        Self::VerySunny,
        Self::Cookie4Sided,
        Self::Cookie6Sided,
        Self::Cookie7Sided,
        Self::Cookie9Sided,
        Self::Cookie12Sided,
        Self::Ghostish,
        Self::Clover4Leaf,
        Self::Clover8Leaf,
        Self::Burst,
        Self::SoftBurst,
        Self::Boom,
        Self::SoftBoom,
        Self::Flower,
        Self::Puffy,
        Self::PuffyDiamond,
        Self::PixelCircle,
        Self::PixelTriangle,
        Self::Bun,
        Self::Heart,
    ];

    /// The outline, normalized to a 0-1 box. Built once per shape, on first
    /// use, and kept for the life of the process.
    pub fn polygon(self) -> &'static RoundedPolygon {
        POLYGONS[self as usize].get_or_init(|| outline(self).normalized())
    }

    /// The morph from this shape to `to`. Built once per ordered pair, on
    /// first use, and kept for the life of the process.
    pub fn morph_to(self, to: Self) -> &'static Morph {
        MORPHS[self as usize * COUNT + to as usize]
            .get_or_init(|| Box::new(Morph::new(self.polygon(), to.polygon())))
    }
}

const R15: CornerRounding = CornerRounding::new(0.15);
const R20: CornerRounding = CornerRounding::new(0.2);
const R30: CornerRounding = CornerRounding::new(0.3);
const R50: CornerRounding = CornerRounding::new(0.5);
const R100: CornerRounding = CornerRounding::new(1.);
const SHARP: CornerRounding = CornerRounding::UNROUNDED;
const ORIGIN: (f32, f32) = (0., 0.);
const PI: f32 = std::f32::consts::PI;

const fn p(x: f32, y: f32) -> (Pt, CornerRounding) {
    (pt(x, y), SHARP)
}
const fn r(x: f32, y: f32, radius: f32) -> (Pt, CornerRounding) {
    (pt(x, y), CornerRounding::new(radius))
}
const fn s(x: f32, y: f32, radius: f32, smoothing: f32) -> (Pt, CornerRounding) {
    (pt(x, y), CornerRounding::smoothed(radius, smoothing))
}

fn to_radians(degrees: f32) -> f32 {
    degrees / 360. * 2. * PI
}

/// The source's `doRepeat`: turn the section's points around (0.5, 0.5),
/// reflecting every other section when `mirroring`.
fn repeated(
    points: &[(Pt, CornerRounding)],
    reps: usize,
    mirroring: bool,
) -> Vec<(Pt, CornerRounding)> {
    let center = pt(0.5, 0.5);
    let mut out = Vec::new();
    if mirroring {
        let angles: Vec<f32> = points.iter().map(|q| angle_degrees(q.0 - center)).collect();
        let distances: Vec<f32> = points.iter().map(|q| (q.0 - center).distance()).collect();
        let actual_reps = reps * 2;
        let section = 360. / actual_reps as f32;
        for it in 0..actual_reps {
            for index in 0..points.len() {
                let even = it % 2 == 0;
                let i = if even {
                    index
                } else {
                    points.len() - 1 - index
                };
                if i > 0 || even {
                    let turned = if even {
                        angles[i]
                    } else {
                        section - angles[i] + 2. * angles[0]
                    };
                    let a = to_radians(section * it as f32 + turned);
                    out.push((pt(cos(a), sin(a)) * distances[i] + center, points[i].1));
                }
            }
        }
    } else {
        let np = points.len();
        for it in 0..np * reps {
            let a = to_radians((it / np) as f32 * 360. / reps as f32);
            let off = points[it % np].0 - center;
            out.push((
                pt(
                    off.x * cos(a) - off.y * sin(a),
                    off.x * sin(a) + off.y * cos(a),
                ) + center,
                points[it % np].1,
            ));
        }
    }
    out
}

fn custom(points: &[(Pt, CornerRounding)], reps: usize, mirroring: bool) -> RoundedPolygon {
    let actual = repeated(points, reps, mirroring);
    let vertices: Vec<(f32, f32)> = actual.iter().map(|q| (q.0.x, q.0.y)).collect();
    let rounding: Vec<CornerRounding> = actual.iter().map(|q| q.1).collect();
    RoundedPolygon::from_vertices(&vertices, SHARP, Some(&rounding), Some((0.5, 0.5)))
        .unwrap_or_else(unbuildable)
}

/// A single point, standing in for an outline that cannot be built. The
/// tables below all have finite vertices and matching lengths, which is all
/// the constructors refuse, and the fixture test builds every one of them.
fn unbuildable() -> RoundedPolygon {
    RoundedPolygon::from_vertices(&[(0.5, 0.5); 3], SHARP, None, None).unwrap_or_else(|| {
        unreachable!("three finite vertices with no per-vertex rounding are always accepted")
    })
}

fn star(points: usize, inner: f32, rounding: CornerRounding) -> RoundedPolygon {
    RoundedPolygon::star(points, 1., inner, rounding, None, ORIGIN).unwrap_or_else(unbuildable)
}

fn outline(kind: ShapeKind) -> RoundedPolygon {
    use ShapeKind as K;
    match kind {
        K::Circle => RoundedPolygon::circle(10, 1., ORIGIN).unwrap_or_else(unbuildable),
        K::Square => {
            RoundedPolygon::rectangle(1., 1., R30, None, ORIGIN).unwrap_or_else(unbuildable)
        }
        K::Slanted => custom(
            &[
                s(0.926, 0.970, 0.189, 0.811),
                s(-0.021, 0.967, 0.187, 0.057),
            ],
            2,
            false,
        ),
        K::Arch => RoundedPolygon::regular(4, 1., ORIGIN, SHARP, Some(&[R100, R100, R20, R20]))
            .unwrap_or_else(unbuildable)
            .rotated(-135.),
        K::Fan => custom(
            &[
                s(1.004, 1.000, 0.148, 0.417),
                r(0.000, 1.000, 0.151),
                r(0.000, -0.003, 0.148),
                r(0.978, 0.020, 0.803),
            ],
            1,
            false,
        ),
        K::Arrow => custom(
            &[
                r(0.500, 0.892, 0.313),
                r(-0.216, 1.050, 0.207),
                s(0.499, -0.160, 0.215, 1.000),
                r(1.225, 1.060, 0.211),
            ],
            1,
            false,
        ),
        K::SemiCircle => {
            RoundedPolygon::rectangle(1.6, 1., SHARP, Some(&[R20, R20, R100, R100]), ORIGIN)
                .unwrap_or_else(unbuildable)
        }
        K::Oval => RoundedPolygon::circle(8, 1., ORIGIN)
            .unwrap_or_else(unbuildable)
            .transformed(|x, y| (x, y * 0.64))
            .rotated(-45.),
        K::Pill => custom(
            &[
                r(0.961, 0.039, 0.426),
                p(1.001, 0.428),
                r(1.000, 0.609, 1.000),
            ],
            2,
            true,
        ),
        K::Triangle => RoundedPolygon::regular(3, 1., ORIGIN, R20, None)
            .unwrap_or_else(unbuildable)
            .rotated(-90.),
        K::Diamond => custom(
            &[s(0.500, 1.096, 0.151, 0.524), r(0.040, 0.500, 0.159)],
            2,
            false,
        ),
        K::ClamShell => custom(
            &[
                r(0.171, 0.841, 0.159),
                r(-0.020, 0.500, 0.140),
                r(0.170, 0.159, 0.159),
            ],
            2,
            false,
        ),
        K::Pentagon => custom(
            &[
                r(0.500, -0.009, 0.172),
                r(1.030, 0.365, 0.164),
                r(0.828, 0.970, 0.169),
            ],
            1,
            true,
        ),
        K::Gem => custom(
            &[
                s(0.499, 1.023, 0.241, 0.778),
                r(-0.005, 0.792, 0.208),
                r(0.073, 0.258, 0.228),
                r(0.433, -0.000, 0.491),
            ],
            1,
            true,
        ),
        K::Sunny => star(8, 0.8, R15),
        K::VerySunny => custom(&[r(0.500, 1.080, 0.085), r(0.358, 0.843, 0.085)], 8, false),
        K::Cookie4Sided => custom(&[r(1.237, 1.236, 0.258), r(0.500, 0.918, 0.233)], 4, false),
        K::Cookie6Sided => custom(&[r(0.723, 0.884, 0.394), r(0.500, 1.099, 0.398)], 6, false),
        K::Cookie7Sided => star(7, 0.75, R50).rotated(-90.),
        K::Cookie9Sided => star(9, 0.8, R50).rotated(-90.),
        K::Cookie12Sided => star(12, 0.8, R50).rotated(-90.),
        K::Ghostish => custom(
            &[
                r(0.500, 0., 1.000),
                r(1., 0., 1.000),
                s(1., 1.140, 0.254, 0.106),
                r(0.575, 0.906, 0.253),
            ],
            1,
            true,
        ),
        K::Clover4Leaf => custom(&[p(0.500, 0.074), r(0.725, -0.099, 0.476)], 4, true),
        K::Clover8Leaf => custom(&[p(0.500, 0.036), r(0.758, -0.101, 0.209)], 8, false),
        K::Burst => custom(
            &[r(0.500, -0.006, 0.006), r(0.592, 0.158, 0.006)],
            12,
            false,
        ),
        K::SoftBurst => custom(&[r(0.193, 0.277, 0.053), r(0.176, 0.055, 0.053)], 10, false),
        K::Boom => custom(
            &[r(0.457, 0.296, 0.007), r(0.500, -0.051, 0.007)],
            15,
            false,
        ),
        K::SoftBoom => custom(
            &[
                p(0.733, 0.454),
                r(0.839, 0.437, 0.532),
                s(0.949, 0.449, 0.439, 1.000),
                r(0.998, 0.478, 0.174),
            ],
            16,
            true,
        ),
        K::Flower => custom(
            &[
                p(0.370, 0.187),
                r(0.416, 0.049, 0.381),
                r(0.479, 0.001, 0.095),
            ],
            8,
            true,
        ),
        K::Puffy => custom(
            &[
                p(0.500, 0.053),
                r(0.545, -0.040, 0.405),
                r(0.670, -0.035, 0.426),
                r(0.717, 0.066, 0.574),
                p(0.722, 0.128),
                r(0.777, 0.002, 0.360),
                r(0.914, 0.149, 0.660),
                r(0.926, 0.289, 0.660),
                p(0.881, 0.346),
                r(0.940, 0.344, 0.126),
                r(1.003, 0.437, 0.255),
            ],
            2,
            true,
        )
        .transformed(|x, y| (x, y * 0.742)),
        K::PuffyDiamond => custom(
            &[
                r(0.870, 0.130, 0.146),
                p(0.818, 0.357),
                r(1.000, 0.332, 0.853),
            ],
            4,
            true,
        ),
        K::PixelCircle => custom(
            &[
                p(0.500, 0.000),
                p(0.704, 0.000),
                p(0.704, 0.065),
                p(0.843, 0.065),
                p(0.843, 0.148),
                p(0.926, 0.148),
                p(0.926, 0.296),
                p(1.000, 0.296),
            ],
            2,
            true,
        ),
        K::PixelTriangle => custom(
            &[
                p(0.110, 0.500),
                p(0.113, 0.000),
                p(0.287, 0.000),
                p(0.287, 0.087),
                p(0.421, 0.087),
                p(0.421, 0.170),
                p(0.560, 0.170),
                p(0.560, 0.265),
                p(0.674, 0.265),
                p(0.675, 0.344),
                p(0.789, 0.344),
                p(0.789, 0.439),
                p(0.888, 0.439),
            ],
            1,
            true,
        ),
        K::Bun => custom(
            &[
                p(0.796, 0.500),
                r(0.853, 0.518, 1.),
                r(0.992, 0.631, 1.),
                r(0.968, 1.000, 1.),
            ],
            2,
            true,
        ),
        K::Heart => custom(
            &[
                r(0.500, 0.268, 0.016),
                r(0.792, -0.066, 0.958),
                r(1.064, 0.276, 1.000),
                r(0.501, 0.946, 0.129),
            ],
            1,
            true,
        ),
    }
}
