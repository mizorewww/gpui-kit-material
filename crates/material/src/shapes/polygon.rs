// Copyright 2022-2024 The Android Open Source Project
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// at http://www.apache.org/licenses/LICENSE-2.0 (see LICENSE-androidx next to
// this file). Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//
// Ported to Rust from androidx.graphics:graphics-shapes (RoundedPolygon.kt,
// Cubic.kt, CornerRounding.kt, Shapes.kt, Utils.kt). Exceptions become `None`
// or a degenerate shape, so that no input panics.
//! Rounded polygons: closed outlines of cubic curves, with each vertex's
//! rounding and the corners the morph matches. All math stays in `f32`, in
//! the source's operation order, so results match the JVM reference.

/// Below this, two points are the same (the source's `DistanceEpsilon`).
pub(super) const DISTANCE_EPSILON: f32 = 1e-4;
/// The source's `AngleEpsilon`.
pub(super) const ANGLE_EPSILON: f32 = 1e-6;
const PI: f32 = std::f32::consts::PI;

// Kotlin's `cos(Float)` and its kin evaluate in double precision. Doing the
// same keeps results independent of the platform's single-precision libm.
pub(super) fn cos(x: f32) -> f32 {
    f64::from(x).cos() as f32
}
pub(super) fn sin(x: f32) -> f32 {
    f64::from(x).sin() as f32
}
pub(super) fn atan2(y: f32, x: f32) -> f32 {
    f64::from(y).atan2(f64::from(x)) as f32
}

pub(super) fn interpolate(start: f32, stop: f32, fraction: f32) -> f32 {
    (1. - fraction) * start + fraction * stop
}

pub(super) fn positive_modulo(num: f32, modulus: f32) -> f32 {
    (num % modulus + modulus) % modulus
}

/// A point in a shape's own coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Pt {
    pub x: f32,
    pub y: f32,
}

pub(super) const fn pt(x: f32, y: f32) -> Pt {
    Pt { x, y }
}

impl Pt {
    pub fn distance(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn distance_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
    fn dot_xy(self, x: f32, y: f32) -> f32 {
        self.x * x + self.y * y
    }
    fn clockwise(self, other: Self) -> bool {
        self.x * other.y - self.y * other.x > 0.
    }
    /// The unit vector, or zero for a zero vector (the source throws).
    fn direction(self) -> Self {
        let distance = self.distance();
        if distance > 0. { self / distance } else { self }
    }
    fn rotate90(self) -> Self {
        pt(-self.y, self.x)
    }
}

impl std::ops::Add for Pt {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        pt(self.x + other.x, self.y + other.y)
    }
}
impl std::ops::Sub for Pt {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        pt(self.x - other.x, self.y - other.y)
    }
}
impl std::ops::Mul<f32> for Pt {
    type Output = Self;
    fn mul(self, operand: f32) -> Self {
        pt(self.x * operand, self.y * operand)
    }
}
impl std::ops::Div<f32> for Pt {
    type Output = Self;
    fn div(self, operand: f32) -> Self {
        pt(self.x / operand, self.y / operand)
    }
}

fn direction_of(x: f32, y: f32) -> Pt {
    pt(x, y).direction()
}

fn lerp_pt(start: Pt, stop: Pt, fraction: f32) -> Pt {
    pt(
        interpolate(start.x, stop.x, fraction),
        interpolate(start.y, stop.y, fraction),
    )
}

/// One cubic Bézier curve, as anchor 0, control 0, control 1 and anchor 1,
/// each `x, y`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cubic {
    /// The curve's eight coordinates, in the order above.
    pub points: [f32; 8],
}

impl Cubic {
    pub(super) const fn new(points: [f32; 8]) -> Self {
        Self { points }
    }

    fn from_points(anchor0: Pt, control0: Pt, control1: Pt, anchor1: Pt) -> Self {
        Self::new([
            anchor0.x, anchor0.y, control0.x, control0.y, control1.x, control1.y, anchor1.x,
            anchor1.y,
        ])
    }

    /// The point the curve starts at.
    pub fn anchor0(&self) -> (f32, f32) {
        (self.points[0], self.points[1])
    }
    /// The control point that leaves [`Self::anchor0`].
    pub fn control0(&self) -> (f32, f32) {
        (self.points[2], self.points[3])
    }
    /// The control point that arrives at [`Self::anchor1`].
    pub fn control1(&self) -> (f32, f32) {
        (self.points[4], self.points[5])
    }
    /// The point the curve ends at.
    pub fn anchor1(&self) -> (f32, f32) {
        (self.points[6], self.points[7])
    }

    /// The point at parameter `t` (0 at anchor 0, 1 at anchor 1).
    pub fn point_at(&self, t: f32) -> (f32, f32) {
        let p = self.point_on_curve(t);
        (p.x, p.y)
    }

    pub(super) fn point_on_curve(&self, t: f32) -> Pt {
        let u = 1. - t;
        let [a0x, a0y, c0x, c0y, c1x, c1y, a1x, a1y] = self.points;
        pt(
            a0x * (u * u * u) + c0x * (3. * t * u * u) + c1x * (3. * t * t * u) + a1x * (t * t * t),
            a0y * (u * u * u) + c0y * (3. * t * u * u) + c1y * (3. * t * t * u) + a1y * (t * t * t),
        )
    }

    fn zero_length(&self) -> bool {
        (self.points[0] - self.points[6]).abs() < DISTANCE_EPSILON
            && (self.points[1] - self.points[7]).abs() < DISTANCE_EPSILON
    }

    /// The curve's bounds as `[left, top, right, bottom]`: tight, or, when
    /// `approximate`, the box of its anchors and control points.
    pub(super) fn bounds(&self, approximate: bool) -> [f32; 4] {
        let [a0x, a0y, c0x, c0y, c1x, c1y, a1x, a1y] = self.points;
        if self.zero_length() {
            return [a0x, a0y, a0x, a0y];
        }
        let (mut min_x, mut max_x) = (a0x.min(a1x), a0x.max(a1x));
        let (mut min_y, mut max_y) = (a0y.min(a1y), a0y.max(a1y));
        if approximate {
            return [
                min_x.min(c0x.min(c1x)),
                min_y.min(c0y.min(c1y)),
                max_x.max(c0x.max(c1x)),
                max_y.max(c0y.max(c1y)),
            ];
        }
        // The derivative is a quadratic; its roots are the extrema.
        let extrema = |a0: f32, c0: f32, c1: f32, a1: f32, each: &mut dyn FnMut(f32)| {
            let a = -a0 + 3. * c0 - 3. * c1 + a1;
            let b = 2. * a0 - 4. * c0 + 2. * c1;
            let c = -a0 + c0;
            if a.abs() < DISTANCE_EPSILON {
                if b != 0. {
                    each(2. * c / (-2. * b));
                }
            } else {
                let s = b * b - 4. * a * c;
                if s >= 0. {
                    each((-b + s.sqrt()) / (2. * a));
                    each((-b - s.sqrt()) / (2. * a));
                }
            }
        };
        extrema(a0x, c0x, c1x, a1x, &mut |t| {
            if (0.0..=1.0).contains(&t) {
                let x = self.point_on_curve(t).x;
                min_x = min_x.min(x);
                max_x = max_x.max(x);
            }
        });
        extrema(a0y, c0y, c1y, a1y, &mut |t| {
            if (0.0..=1.0).contains(&t) {
                let y = self.point_on_curve(t).y;
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        });
        [min_x, min_y, max_x, max_y]
    }

    pub(super) fn split(&self, t: f32) -> (Self, Self) {
        let u = 1. - t;
        let at = self.point_on_curve(t);
        let [a0x, a0y, c0x, c0y, c1x, c1y, a1x, a1y] = self.points;
        (
            Self::new([
                a0x,
                a0y,
                a0x * u + c0x * t,
                a0y * u + c0y * t,
                a0x * (u * u) + c0x * (2. * u * t) + c1x * (t * t),
                a0y * (u * u) + c0y * (2. * u * t) + c1y * (t * t),
                at.x,
                at.y,
            ]),
            Self::new([
                at.x,
                at.y,
                c0x * (u * u) + c1x * (2. * u * t) + a1x * (t * t),
                c0y * (u * u) + c1y * (2. * u * t) + a1y * (t * t),
                c1x * u + a1x * t,
                c1y * u + a1y * t,
                a1x,
                a1y,
            ]),
        )
    }

    fn reversed(&self) -> Self {
        let [a0x, a0y, c0x, c0y, c1x, c1y, a1x, a1y] = self.points;
        Self::new([a1x, a1y, c1x, c1y, c0x, c0y, a0x, a0y])
    }

    fn straight_line(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self::new([
            x0,
            y0,
            interpolate(x0, x1, 1. / 3.),
            interpolate(y0, y1, 1. / 3.),
            interpolate(x0, x1, 2. / 3.),
            interpolate(y0, y1, 2. / 3.),
            x1,
            y1,
        ])
    }

    fn circular_arc(center: Pt, x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        let p0d = direction_of(x0 - center.x, y0 - center.y);
        let p1d = direction_of(x1 - center.x, y1 - center.y);
        let rotated_p0 = p0d.rotate90();
        let rotated_p1 = p1d.rotate90();
        let clockwise = rotated_p0.dot_xy(x1 - center.x, y1 - center.y) >= 0.;
        let cosa = p0d.dot(p1d);
        if cosa > 0.999 {
            return Self::straight_line(x0, y0, x1, y1);
        }
        let k = pt(x0 - center.x, y0 - center.y).distance() * 4. / 3.
            * ((2. * (1. - cosa)).max(0.).sqrt() - (1. - cosa * cosa).max(0.).sqrt())
            / (1. - cosa)
            * if clockwise { 1. } else { -1. };
        Self::new([
            x0,
            y0,
            x0 + rotated_p0.x * k,
            y0 + rotated_p0.y * k,
            x1 - rotated_p1.x * k,
            y1 - rotated_p1.y * k,
            x1,
            y1,
        ])
    }

    fn transformed(&self, f: &impl Fn(f32, f32) -> (f32, f32)) -> Self {
        let mut points = self.points;
        for [x, y] in points.as_chunks_mut::<2>().0 {
            (*x, *y) = f(*x, *y);
        }
        Self::new(points)
    }
}

/// How a polygon's vertex is rounded.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CornerRounding {
    /// The rounding circle's radius, in the polygon's coordinates. 0 is sharp.
    pub radius: f32,
    /// 0 rounds with a circular arc alone; up to 1 stretches flanking curves
    /// from the arc along the edges.
    pub smoothing: f32,
}

impl CornerRounding {
    /// A sharp corner.
    pub const UNROUNDED: Self = Self::new(0.);

    /// A corner rounded by a circular arc of `radius`.
    pub const fn new(radius: f32) -> Self {
        Self {
            radius,
            smoothing: 0.,
        }
    }

    /// A corner rounded with `radius` and `smoothing`.
    pub const fn smoothed(radius: f32, smoothing: f32) -> Self {
        Self { radius, smoothing }
    }
}

/// A run of a polygon's outline: a corner (with whether it is convex) or an
/// edge. The morph maps corners onto corners of the same convexity.
#[derive(Clone, Debug)]
pub(super) struct Feature {
    pub cubics: Vec<Cubic>,
    /// `Some(convex)` for a corner; `None` for an edge.
    pub corner: Option<bool>,
}

/// A closed outline of cubic curves built from vertices with optional
/// rounding. See `androidx.graphics.shapes.RoundedPolygon`.
#[derive(Clone, Debug)]
pub struct RoundedPolygon {
    pub(super) features: Vec<Feature>,
    center: Pt,
    cubics: Vec<Cubic>,
}

impl RoundedPolygon {
    fn from_features(features: Vec<Feature>, center: Pt) -> Self {
        let cubics = flatten(&features, center);
        Self {
            features,
            center,
            cubics,
        }
    }

    /// A polygon through `vertices`, each rounded by `rounding` or, when
    /// given, by its entry in `per_vertex`. `center` defaults to the
    /// vertices' average.
    ///
    /// `None` with fewer than three vertices, a non-finite coordinate, or a
    /// `per_vertex` of the wrong length.
    pub fn from_vertices(
        vertices: &[(f32, f32)],
        rounding: CornerRounding,
        per_vertex: Option<&[CornerRounding]>,
        center: Option<(f32, f32)>,
    ) -> Option<Self> {
        let n = vertices.len();
        if n < 3
            || vertices
                .iter()
                .any(|v| !v.0.is_finite() || !v.1.is_finite())
            || per_vertex.is_some_and(|rounds| rounds.len() != n)
        {
            return None;
        }
        let at = |i: usize| pt(vertices[i].0, vertices[i].1);
        let corners: Vec<RoundedCorner> = (0..n)
            .map(|i| {
                RoundedCorner::new(
                    at((i + n - 1) % n),
                    at(i),
                    at((i + 1) % n),
                    per_vertex.map_or(rounding, |rounds| rounds[i]),
                )
            })
            .collect();

        // For each side, how much of its corners' rounding and then smoothing
        // fits: rounding is served first.
        let cut_adjusts: Vec<(f32, f32)> = (0..n)
            .map(|i| {
                let (corner, next) = (&corners[i], &corners[(i + 1) % n]);
                let expected_round_cut = corner.expected_round_cut + next.expected_round_cut;
                let expected_cut = corner.expected_cut() + next.expected_cut();
                let side = (at(i) - at((i + 1) % n)).distance();
                if expected_round_cut > side {
                    (side / expected_round_cut, 0.)
                } else if expected_cut > side {
                    (
                        1.,
                        (side - expected_round_cut) / (expected_cut - expected_round_cut),
                    )
                } else {
                    (1., 1.)
                }
            })
            .collect();

        let corner_cubics: Vec<Vec<Cubic>> = (0..n)
            .map(|i| {
                let allowed = |delta: usize| {
                    let (round_ratio, cut_ratio) = cut_adjusts[(i + n - 1 + delta) % n];
                    corners[i].expected_round_cut * round_ratio
                        + (corners[i].expected_cut() - corners[i].expected_round_cut) * cut_ratio
                };
                corners[i].cubics(allowed(0), allowed(1))
            })
            .collect();

        let mut features = Vec::with_capacity(n * 2);
        for i in 0..n {
            let (previous, current, next) = (at((i + n - 1) % n), at(i), at((i + 1) % n));
            features.push(Feature {
                cubics: corner_cubics[i].clone(),
                corner: Some((current - previous).clockwise(next - current)),
            });
            let (end, start) = (
                corner_cubics[i].last().copied().unwrap_or_default(),
                corner_cubics[(i + 1) % n]
                    .first()
                    .copied()
                    .unwrap_or_default(),
            );
            features.push(Feature {
                cubics: vec![Cubic::straight_line(
                    end.points[6],
                    end.points[7],
                    start.points[0],
                    start.points[1],
                )],
                corner: None,
            });
        }
        let center = center.map_or_else(
            || {
                let (sum_x, sum_y) = vertices
                    .iter()
                    .fold((0_f32, 0_f32), |(x, y), v| (x + v.0, y + v.1));
                pt(sum_x / n as f32, sum_y / n as f32)
            },
            |(x, y)| pt(x, y),
        );
        Some(Self::from_features(features, center))
    }

    /// A regular polygon of `sides` (at least 3) vertices on a circle of
    /// `radius`, the first at three o'clock.
    pub fn regular(
        sides: usize,
        radius: f32,
        center: (f32, f32),
        rounding: CornerRounding,
        per_vertex: Option<&[CornerRounding]>,
    ) -> Option<Self> {
        let vertices: Vec<(f32, f32)> = (0..sides)
            .map(|i| {
                let angle = PI / sides as f32 * 2. * i as f32;
                (
                    cos(angle) * radius + center.0,
                    sin(angle) * radius + center.1,
                )
            })
            .collect();
        Self::from_vertices(&vertices, rounding, per_vertex, Some(center))
    }

    /// A circle of `radius`, made by fully rounding `sides` (at least 3)
    /// vertices.
    pub fn circle(sides: usize, radius: f32, center: (f32, f32)) -> Option<Self> {
        if sides < 3 {
            return None;
        }
        let polygon_radius = radius / cos(PI / sides as f32);
        Self::regular(
            sides,
            polygon_radius,
            center,
            CornerRounding::new(radius),
            None,
        )
    }

    /// A `width` by `height` rectangle. Vertices run bottom right, bottom
    /// left, top left, top right.
    pub fn rectangle(
        width: f32,
        height: f32,
        rounding: CornerRounding,
        per_vertex: Option<&[CornerRounding]>,
        center: (f32, f32),
    ) -> Option<Self> {
        let (left, top) = (center.0 - width / 2., center.1 - height / 2.);
        let (right, bottom) = (center.0 + width / 2., center.1 + height / 2.);
        Self::from_vertices(
            &[(right, bottom), (left, bottom), (left, top), (right, top)],
            rounding,
            per_vertex,
            Some(center),
        )
    }

    /// A star of `points` (at least 2) alternating between `radius` and
    /// `inner_radius`, which must be positive and smaller. `inner_rounding`
    /// defaults to `rounding`.
    pub fn star(
        points: usize,
        radius: f32,
        inner_radius: f32,
        rounding: CornerRounding,
        inner_rounding: Option<CornerRounding>,
        center: (f32, f32),
    ) -> Option<Self> {
        if radius.is_nan() || inner_radius.is_nan() {
            return None;
        }
        if radius <= 0. || inner_radius <= 0. || inner_radius >= radius {
            return None;
        }
        let mut vertices = Vec::with_capacity(points * 2);
        for i in 0..points {
            let angle = PI / points as f32 * 2. * i as f32;
            vertices.push((
                cos(angle) * radius + center.0,
                sin(angle) * radius + center.1,
            ));
            let angle = PI / points as f32 * (2 * i + 1) as f32;
            vertices.push((
                cos(angle) * inner_radius + center.0,
                sin(angle) * inner_radius + center.1,
            ));
        }
        let per_vertex: Option<Vec<CornerRounding>> =
            inner_rounding.map(|inner| (0..points).flat_map(|_| [rounding, inner]).collect());
        Self::from_vertices(&vertices, rounding, per_vertex.as_deref(), Some(center))
    }

    /// A capsule: a `width` by `height` rectangle whose short ends round
    /// fully, with `smoothing` on the corners.
    pub fn pill(width: f32, height: f32, smoothing: f32, center: (f32, f32)) -> Option<Self> {
        if width.is_nan() || height.is_nan() || width <= 0. || height <= 0. {
            return None;
        }
        let (half_w, half_h) = (width / 2., height / 2.);
        Self::from_vertices(
            &[
                (half_w + center.0, half_h + center.1),
                (-half_w + center.0, half_h + center.1),
                (-half_w + center.0, -half_h + center.1),
                (half_w + center.0, -half_h + center.1),
            ],
            CornerRounding::smoothed(half_w.min(half_h), smoothing),
            None,
            Some(center),
        )
    }

    /// The outline, as contiguous curves.
    pub fn cubics(&self) -> &[Cubic] {
        &self.cubics
    }

    /// The center the polygon was built around.
    pub fn center(&self) -> (f32, f32) {
        (self.center.x, self.center.y)
    }

    /// This polygon with `f` applied to every point. Corners keep their
    /// convexity.
    pub fn transformed(&self, f: impl Fn(f32, f32) -> (f32, f32)) -> Self {
        let (x, y) = f(self.center.x, self.center.y);
        let features = self
            .features
            .iter()
            .map(|feature| Feature {
                cubics: feature.cubics.iter().map(|c| c.transformed(&f)).collect(),
                corner: feature.corner,
            })
            .collect();
        Self::from_features(features, pt(x, y))
    }

    /// This polygon turned `degrees` about the origin.
    pub fn rotated(&self, degrees: f32) -> Self {
        let radians = f64::from(degrees) * (std::f64::consts::PI / 180.);
        let (s, c) = (radians.sin() as f32, radians.cos() as f32);
        self.transformed(move |x, y| (c * x + -s * y, s * x + c * y))
    }

    /// This polygon moved and scaled so its [`Self::bounds`] are centered in
    /// a 0-1 box, which the longer side fills.
    pub fn normalized(&self) -> Self {
        let [left, top, right, bottom] = self.bounds(true);
        let (width, height) = (right - left, bottom - top);
        let side = width.max(height);
        if side.is_nan() || side <= 0. {
            return self.clone();
        }
        let offset_x = (side - width) / 2. - left;
        let offset_y = (side - height) / 2. - top;
        self.transformed(move |x, y| ((x + offset_x) / side, (y + offset_y) / side))
    }

    /// The outline's `[left, top, right, bottom]`: tight, or, when
    /// `approximate`, the box of all anchors and control points.
    ///
    /// As in the source, the result's right and bottom start from the
    /// smallest positive float, so a polygon wholly in negative coordinates
    /// reports them just above zero.
    pub fn bounds(&self, approximate: bool) -> [f32; 4] {
        let mut result = [f32::MAX, f32::MAX, f32::from_bits(1), f32::from_bits(1)];
        for cubic in &self.cubics {
            let bounds = cubic.bounds(approximate);
            result[0] = result[0].min(bounds[0]);
            result[1] = result[1].min(bounds[1]);
            result[2] = result[2].max(bounds[2]);
            result[3] = result[3].max(bounds[3]);
        }
        result
    }

    /// The square around [`Self::center`] that holds the outline at any
    /// rotation, as `[left, top, right, bottom]`.
    pub fn max_bounds(&self) -> [f32; 4] {
        let mut max_squared = 0_f32;
        for cubic in &self.cubics {
            let anchor = (pt(cubic.points[0], cubic.points[1]) - self.center).distance_squared();
            let middle = (cubic.point_on_curve(0.5) - self.center).distance_squared();
            max_squared = max_squared.max(anchor.max(middle));
        }
        let distance = max_squared.sqrt();
        [
            self.center.x - distance,
            self.center.y - distance,
            self.center.x + distance,
            self.center.y + distance,
        ]
    }
}

/// The features' curves, starting mid-way through the first corner, without
/// zero-length curves, and closed exactly on the first anchor.
fn flatten(features: &[Feature], center: Pt) -> Vec<Cubic> {
    let mut out = Vec::new();
    let mut first: Option<Cubic> = None;
    let mut last: Option<Cubic> = None;
    let split = features
        .first()
        .filter(|feature| feature.cubics.len() == 3)
        .map(|feature| {
            let (start, end) = feature.cubics[1].split(0.5);
            ([feature.cubics[0], start], [end, feature.cubics[2]])
        });
    for i in 0..=features.len() {
        let run: &[Cubic] = match &split {
            Some(split) if i == 0 => &split.1,
            Some(split) if i == features.len() => &split.0,
            None if i == features.len() => break,
            _ => &features[i].cubics,
        };
        for cubic in run {
            if !cubic.zero_length() {
                if let Some(last) = last {
                    out.push(last);
                }
                last = Some(*cubic);
                first.get_or_insert(*cubic);
            } else if let Some(last) = &mut last {
                // Dropping several tiny curves in a row must not leave a gap.
                last.points[6] = cubic.points[6];
                last.points[7] = cubic.points[7];
            }
        }
    }
    match (first, last) {
        (Some(first), Some(mut last)) => {
            last.points[6] = first.points[0];
            last.points[7] = first.points[1];
            out.push(last);
        }
        // A zero-sized polygon.
        _ => out.push(Cubic::new([
            center.x, center.y, center.x, center.y, center.x, center.y, center.x, center.y,
        ])),
    }
    out
}

struct RoundedCorner {
    p0: Pt,
    p1: Pt,
    p2: Pt,
    d1: Pt,
    d2: Pt,
    radius: f32,
    smoothing: f32,
    expected_round_cut: f32,
}

impl RoundedCorner {
    fn new(p0: Pt, p1: Pt, p2: Pt, rounding: CornerRounding) -> Self {
        let (v01, v21) = (p0 - p1, p2 - p1);
        let (d01, d21) = (v01.distance(), v21.distance());
        if d01 > 0. && d21 > 0. {
            let (d1, d2) = (v01 / d01, v21 / d21);
            let cos_angle = d1.dot(d2);
            let sin_angle = (1. - cos_angle * cos_angle).max(0.).sqrt();
            let expected_round_cut = if f64::from(sin_angle) > 1e-3 {
                rounding.radius * (cos_angle + 1.) / sin_angle
            } else {
                0.
            };
            Self {
                p0,
                p1,
                p2,
                d1,
                d2,
                radius: rounding.radius,
                smoothing: rounding.smoothing,
                expected_round_cut,
            }
        } else {
            Self {
                p0,
                p1,
                p2,
                d1: Pt::default(),
                d2: Pt::default(),
                radius: 0.,
                smoothing: 0.,
                expected_round_cut: 0.,
            }
        }
    }

    fn expected_cut(&self) -> f32 {
        (1. + self.smoothing) * self.expected_round_cut
    }

    fn cubics(&self, allowed_cut0: f32, allowed_cut1: f32) -> Vec<Cubic> {
        let allowed_cut = allowed_cut0.min(allowed_cut1);
        if self.expected_round_cut < DISTANCE_EPSILON
            || allowed_cut < DISTANCE_EPSILON
            || self.radius < DISTANCE_EPSILON
        {
            let p = self.p1;
            return vec![Cubic::straight_line(p.x, p.y, p.x, p.y)];
        }
        let round_cut = allowed_cut.min(self.expected_round_cut);
        let smoothing0 = self.actual_smoothing(allowed_cut0);
        let smoothing1 = self.actual_smoothing(allowed_cut1);
        let radius = self.radius * round_cut / self.expected_round_cut;
        let center_distance = (radius * radius + round_cut * round_cut).sqrt();
        let center = self.p1 + ((self.d1 + self.d2) / 2.).direction() * center_distance;
        let intersection0 = self.p1 + self.d1 * round_cut;
        let intersection2 = self.p1 + self.d2 * round_cut;
        let flanking0 = flanking_curve(
            round_cut,
            smoothing0,
            self.p1,
            self.p0,
            intersection0,
            intersection2,
            center,
            radius,
        );
        let flanking2 = flanking_curve(
            round_cut,
            smoothing1,
            self.p1,
            self.p2,
            intersection2,
            intersection0,
            center,
            radius,
        )
        .reversed();
        let arc = Cubic::circular_arc(
            center,
            flanking0.points[6],
            flanking0.points[7],
            flanking2.points[0],
            flanking2.points[1],
        );
        vec![flanking0, arc, flanking2]
    }

    fn actual_smoothing(&self, allowed_cut: f32) -> f32 {
        if allowed_cut > self.expected_cut() {
            self.smoothing
        } else if allowed_cut > self.expected_round_cut {
            self.smoothing * (allowed_cut - self.expected_round_cut)
                / (self.expected_cut() - self.expected_round_cut)
        } else {
            0.
        }
    }
}

#[allow(clippy::too_many_arguments)] // Mirrors the source's computeFlankingCurve.
fn flanking_curve(
    round_cut: f32,
    smoothing: f32,
    corner: Pt,
    side_start: Pt,
    intersection: Pt,
    other_intersection: Pt,
    center: Pt,
    radius: f32,
) -> Cubic {
    let side_direction = (side_start - corner).direction();
    let curve_start = corner + side_direction * round_cut * (1. + smoothing);
    let p = lerp_pt(
        intersection,
        (intersection + other_intersection) / 2.,
        smoothing,
    );
    let curve_end = center + direction_of(p.x - center.x, p.y - center.y) * radius;
    let tangent = (curve_end - center).rotate90();
    let anchor_end =
        line_intersection(side_start, side_direction, curve_end, tangent).unwrap_or(intersection);
    // Two thirds, as the source: it "seems to come from design tools".
    let anchor_start = (curve_start + anchor_end * 2.) / 3.;
    Cubic::from_points(curve_start, anchor_start, anchor_end, curve_end)
}

fn line_intersection(p0: Pt, d0: Pt, p1: Pt, d1: Pt) -> Option<Pt> {
    let rotated = d1.rotate90();
    let den = d0.dot(rotated);
    if den.abs() < DISTANCE_EPSILON {
        return None;
    }
    let num = (p1 - p0).dot(rotated);
    if den.abs() < DISTANCE_EPSILON * num.abs() {
        return None;
    }
    Some(p0 + d0 * (num / den))
}

/// The angle of a vector, in degrees.
pub(super) fn angle_degrees(p: Pt) -> f32 {
    atan2(p.y, p.x) * 180. / PI
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn bad_input_is_refused_instead_of_panicking() {
        let none = CornerRounding::UNROUNDED;
        assert!(RoundedPolygon::from_vertices(&[(0., 0.), (1., 0.)], none, None, None).is_none());
        let nan = [(0., 0.), (1., f32::NAN), (1., 1.)];
        assert!(RoundedPolygon::from_vertices(&nan, none, None, None).is_none());
        assert!(RoundedPolygon::circle(2, 1., (0., 0.)).is_none());
        assert!(RoundedPolygon::regular(0, 1., (0., 0.), none, None).is_none());
        assert!(RoundedPolygon::star(5, 1., 2., none, None, (0., 0.)).is_none());
        assert!(RoundedPolygon::star(5, f32::NAN, 0.5, none, None, (0., 0.)).is_none());
        assert!(RoundedPolygon::pill(0., 1., 0., (0., 0.)).is_none());
        let short = [CornerRounding::new(0.1)];
        let triangle = [(0., 0.), (1., 0.), (0., 1.)];
        assert!(RoundedPolygon::from_vertices(&triangle, none, Some(&short), None).is_none());
    }

    #[test]
    fn a_zero_sized_polygon_is_a_single_point_curve() {
        let point = [(1., 1.); 4];
        let polygon = RoundedPolygon::from_vertices(&point, CornerRounding::new(0.2), None, None)
            .expect("finite");
        assert_eq!(polygon.cubics().len(), 1);
        assert_eq!(polygon.cubics()[0].anchor0(), (1., 1.));
        // Normalizing has no side to divide by; it must not produce NaN.
        assert_eq!(polygon.normalized().cubics()[0].anchor0(), (1., 1.));
    }

    #[test]
    fn the_outline_closes_exactly() {
        let polygon = RoundedPolygon::star(
            6,
            1.,
            0.5,
            CornerRounding::smoothed(0.1, 0.5),
            None,
            (0., 0.),
        )
        .expect("valid");
        let cubics = polygon.cubics();
        for pair in cubics.windows(2) {
            let (a, b) = (pair[0].anchor1(), pair[1].anchor0());
            assert!((a.0 - b.0).abs() < DISTANCE_EPSILON && (a.1 - b.1).abs() < DISTANCE_EPSILON);
        }
        assert_eq!(
            cubics.last().map(Cubic::anchor1),
            cubics.first().map(Cubic::anchor0)
        );
    }
}
