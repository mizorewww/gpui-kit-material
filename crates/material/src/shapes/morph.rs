// Copyright 2022-2024 The Android Open Source Project
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// at http://www.apache.org/licenses/LICENSE-2.0 (see LICENSE-androidx next to
// this file). Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//
// Ported to Rust from androidx.graphics:graphics-shapes (Morph.kt,
// FeatureMapping.kt, FloatMapping.kt, PolygonMeasure.kt). Its `require` checks
// become fallbacks, so that no polygon pair panics.
//! Morphing: measure both outlines, map the corners of one onto the corners
//! of the other without crossings, then cut curves until both outlines have
//! the same number, matched one to one. Interpolating the matched control
//! points cannot twist the outline the way radial or index-based blending
//! does, and it needs no star-shaped outlines.
use super::polygon::{
    ANGLE_EPSILON, Cubic, DISTANCE_EPSILON, Pt, RoundedPolygon, atan2, interpolate,
    positive_modulo, pt,
};

/// Whether `progress` lies in the wrapping range from `from` to `to`.
fn progress_in_range(progress: f32, from: f32, to: f32) -> bool {
    if to >= from {
        (from..=to).contains(&progress)
    } else {
        progress >= from || progress <= to
    }
}

/// The distance between two progress values, which wrap at 1.
fn progress_distance(a: f32, b: f32) -> f32 {
    let distance = (a - b).abs();
    distance.min(1. - distance)
}

fn linear_map(xs: &[f32], ys: &[f32], x: f32) -> f32 {
    let n = xs.len();
    let start = (0..n)
        .find(|&i| progress_in_range(x, xs[i], xs[(i + 1) % n]))
        .unwrap_or(0);
    let end = (start + 1) % n;
    let size_x = positive_modulo(xs[end] - xs[start], 1.);
    let size_y = positive_modulo(ys[end] - ys[start], 1.);
    let position = if size_x < 0.001 {
        0.5
    } else {
        positive_modulo(x - xs[start], 1.) / size_x
    };
    positive_modulo(ys[start] + size_y * position, 1.)
}

/// A wrapping, piecewise-linear map between outline progress on two shapes.
struct DoubleMapper {
    source: Vec<f32>,
    target: Vec<f32>,
}

impl DoubleMapper {
    /// The mapping, or the identity where it would not be monotonic.
    fn new(mappings: &[(f32, f32)]) -> Self {
        let source: Vec<f32> = mappings.iter().map(|m| m.0).collect();
        let target: Vec<f32> = mappings.iter().map(|m| m.1).collect();
        if valid_progress(&source) && valid_progress(&target) {
            Self { source, target }
        } else {
            Self {
                source: vec![0., 0.5],
                target: vec![0., 0.5],
            }
        }
    }
    fn map(&self, x: f32) -> f32 {
        linear_map(&self.source, &self.target, x)
    }
    fn map_back(&self, x: f32) -> f32 {
        linear_map(&self.target, &self.source, x)
    }
}

/// In [0, 1), distinct, and increasing except for at most one wrap.
fn valid_progress(values: &[f32]) -> bool {
    let Some(&last) = values.last() else {
        return false;
    };
    let mut previous = last;
    let mut wraps = 0;
    for &current in values {
        if !(0. ..1.).contains(&current) || progress_distance(current, previous) <= DISTANCE_EPSILON
        {
            return false;
        }
        if current < previous {
            wraps += 1;
            if wraps > 1 {
                return false;
            }
        }
        previous = current;
    }
    true
}

/// A corner and where it sits on its outline.
#[derive(Clone, Copy)]
struct MeasuredCorner {
    progress: f32,
    representative: Pt,
    convex: bool,
}

/// Maps corners, nearest first, keeping the mapping free of crossings.
fn feature_mapper(corners1: &[MeasuredCorner], corners2: &[MeasuredCorner]) -> DoubleMapper {
    let mut pairs: Vec<(f32, usize, usize)> = Vec::new();
    for (i, a) in corners1.iter().enumerate() {
        for (j, b) in corners2.iter().enumerate() {
            // Corners only map onto corners of the same convexity.
            if a.convex == b.convex {
                let d = (a.representative - b.representative).distance_squared();
                if d != f32::MAX {
                    pairs.push((d, i, j));
                }
            }
        }
    }
    pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
    match pairs.as_slice() {
        [] => DoubleMapper::new(&[(0., 0.), (0.5, 0.5)]),
        [(_, i, j)] => {
            let (f1, f2) = (corners1[*i].progress, corners2[*j].progress);
            DoubleMapper::new(&[(f1, f2), ((f1 + 0.5) % 1., (f2 + 0.5) % 1.)])
        }
        _ => {
            let mut mapping: Vec<(f32, f32)> = Vec::new();
            let mut used1 = vec![false; corners1.len()];
            let mut used2 = vec![false; corners2.len()];
            for &(_, i, j) in &pairs {
                if used1[i] || used2[j] {
                    continue;
                }
                let (f1, f2) = (corners1[i].progress, corners2[j].progress);
                let Err(at) = mapping.binary_search_by(|m| m.0.total_cmp(&f1)) else {
                    continue;
                };
                let n = mapping.len();
                if n >= 1 {
                    let (before1, before2) = mapping[(at + n - 1) % n];
                    let (after1, after2) = mapping[at % n];
                    if progress_distance(f1, before1) < DISTANCE_EPSILON
                        || progress_distance(f1, after1) < DISTANCE_EPSILON
                        || progress_distance(f2, before2) < DISTANCE_EPSILON
                        || progress_distance(f2, after2) < DISTANCE_EPSILON
                    {
                        continue;
                    }
                    if n > 1 && !progress_in_range(f2, before2, after2) {
                        continue;
                    }
                }
                mapping.insert(at, (f1, f2));
                used1[i] = true;
                used2[j] = true;
            }
            DoubleMapper::new(&mapping)
        }
    }
}

/// `graphics-shapes` 1.0.1's `doMapping`: for each of `f1`'s corners in turn,
/// the nearest of `f2`'s that leaves enough for the corners still to come.
fn greedy_picks(f1: &[MeasuredCorner], f2: &[MeasuredCorner]) -> Vec<MeasuredCorner> {
    let distance = |a: &MeasuredCorner, b: &MeasuredCorner| {
        if a.convex == b.convex {
            (a.representative - b.representative).distance_squared()
        } else {
            f32::MAX
        }
    };
    let nearest = |corner: &MeasuredCorner, from: usize, to: usize| {
        let mut best = from;
        let mut best_distance = distance(corner, &f2[from % f2.len()]);
        for i in from + 1..=to {
            let d = distance(corner, &f2[i % f2.len()]);
            if d < best_distance {
                best = i;
                best_distance = d;
            }
        }
        best
    };
    let (m, n) = (f1.len(), f2.len());
    let first = nearest(&f1[0], 0, n - 1);
    let mut picks = vec![f2[first]];
    let mut last_picked = first;
    for (i, corner) in f1.iter().enumerate().skip(1) {
        let mut last = first as isize - (m - i) as isize;
        if last <= last_picked as isize {
            last += n as isize;
        }
        let best = nearest(corner, last_picked + 1, last.max(0) as usize);
        picks.push(f2[best % n]);
        last_picked = best;
    }
    picks
}

/// `graphics-shapes` 1.0.1's `featureMapper`: the shorter list maps into the
/// longer. No corners at all maps by identity, where the source throws.
fn greedy_mapper(corners1: &[MeasuredCorner], corners2: &[MeasuredCorner]) -> DoubleMapper {
    if corners1.is_empty() || corners2.is_empty() {
        return DoubleMapper::new(&[(0., 0.), (0.5, 0.5)]);
    }
    let pairs: Vec<(f32, f32)> = if corners1.len() > corners2.len() {
        greedy_picks(corners2, corners1)
            .iter()
            .zip(corners2)
            .map(|(a, b)| (a.progress, b.progress))
            .collect()
    } else {
        corners1
            .iter()
            .zip(greedy_picks(corners1, corners2))
            .map(|(a, b)| (a.progress, b.progress))
            .collect()
    };
    DoubleMapper::new(&pairs)
}

const MEASURE_SEGMENTS: u32 = 3;

/// How a [`Morph`] measures its outlines and maps their corners.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MorphMatching {
    /// `graphics-shapes` since 1.1: progress is curve length, and corners map
    /// nearest first without crossing. Handles outlines that are not
    /// star-shaped around their center.
    #[default]
    Current,
    /// `graphics-shapes` 1.0.1, the version Compose Material3 1.5.0-beta01 and
    /// m3e pin: progress is the angle swept around the center, and corners map
    /// by a greedy walk.
    Compose1_5,
}

/// The measure in use, with the center the angle sweeps around.
#[derive(Clone, Copy)]
enum Measurer {
    Length,
    Angle(Pt),
}

impl Measurer {
    fn of(matching: MorphMatching, polygon: &RoundedPolygon) -> Self {
        match matching {
            MorphMatching::Current => Self::Length,
            MorphMatching::Compose1_5 => {
                let (x, y) = polygon.center();
                Self::Angle(pt(x, y))
            }
        }
    }

    fn measure(self, cubic: &Cubic) -> f32 {
        match self {
            Self::Length => closest_progress_to(cubic, f32::INFINITY).1,
            Self::Angle(center) => {
                let sweep = positive_modulo(
                    angle_around(center, cubic.anchor1()) - angle_around(center, cubic.anchor0()),
                    TWO_PI,
                );
                // An empty curve must not measure almost a full turn.
                if sweep > TWO_PI - DISTANCE_EPSILON {
                    0.
                } else {
                    sweep
                }
            }
        }
    }

    fn cut_point(self, cubic: &Cubic, measure: f32) -> f32 {
        match self {
            Self::Length => closest_progress_to(cubic, measure).0,
            Self::Angle(center) => {
                let start = angle_around(center, cubic.anchor0());
                find_minimum(|t| {
                    let at = cubic.point_at(t);
                    (positive_modulo(angle_around(center, at) - start, TWO_PI) - measure).abs()
                })
            }
        }
    }
}

const TWO_PI: f32 = 2. * std::f32::consts::PI;

fn angle_around(center: Pt, point: (f32, f32)) -> f32 {
    (atan2(point.1 - center.y, point.0 - center.x) + TWO_PI) % TWO_PI
}

/// The source's `findMinimum`: a ternary search over 0-1.
fn find_minimum(f: impl Fn(f32) -> f32) -> f32 {
    let (mut a, mut b) = (0_f32, 1_f32);
    while b - a > 1e-5 {
        let c1 = (2. * a + b) / 3.;
        let c2 = (2. * b + a) / 3.;
        if f(c1) < f(c2) {
            b = c2;
        } else {
            a = c1;
        }
    }
    (a + b) / 2.
}

/// The length of `cubic` up to `threshold`, on three chords: the parameter
/// reached, and the length walked. The source's `LengthMeasurer`.
fn closest_progress_to(cubic: &Cubic, threshold: f32) -> (f32, f32) {
    let mut total = 0_f32;
    let mut remainder = threshold;
    let mut previous = pt(cubic.points[0], cubic.points[1]);
    for i in 1..=MEASURE_SEGMENTS {
        let progress = i as f32 / MEASURE_SEGMENTS as f32;
        let point = cubic.point_on_curve(progress);
        let segment = (point - previous).distance();
        if segment >= remainder {
            return (
                progress - (1. - remainder / segment) / MEASURE_SEGMENTS as f32,
                threshold,
            );
        }
        remainder -= segment;
        total += segment;
        previous = point;
    }
    (1., total)
}

#[derive(Clone, Copy)]
struct MeasuredCubic {
    cubic: Cubic,
    start: f32,
    end: f32,
    size: f32,
    measurer: Measurer,
}

impl MeasuredCubic {
    fn new(measurer: Measurer, cubic: Cubic, start: f32, end: f32) -> Self {
        Self {
            cubic,
            start,
            end,
            size: measurer.measure(&cubic),
            measurer,
        }
    }

    fn cut_at(&self, progress: f32) -> (Self, Self) {
        // Rounding upstream can land just outside this curve's range.
        let bounded = if progress < self.start {
            self.start
        } else if progress > self.end {
            self.end
        } else {
            progress
        };
        let relative = (bounded - self.start) / (self.end - self.start);
        let t = self.measurer.cut_point(&self.cubic, relative * self.size);
        // A zero-length chord divides zero by zero.
        let t = if t.is_nan() { 0. } else { t.clamp(0., 1.) };
        let (c1, c2) = self.cubic.split(t);
        (
            Self::new(self.measurer, c1, self.start, bounded),
            Self::new(self.measurer, c2, bounded, self.end),
        )
    }
}

fn measured_cubics(
    measurer: Measurer,
    cubics: &[Cubic],
    outline_progress: &[f32],
) -> Vec<MeasuredCubic> {
    let mut measured: Vec<MeasuredCubic> = Vec::with_capacity(cubics.len());
    let mut start = 0.;
    for (i, cubic) in cubics.iter().enumerate() {
        if outline_progress[i + 1] - outline_progress[i] > DISTANCE_EPSILON {
            measured.push(MeasuredCubic::new(
                measurer,
                *cubic,
                start,
                outline_progress[i + 1],
            ));
            start = outline_progress[i + 1];
        }
    }
    match measured.last_mut() {
        Some(last) => last.end = 1.,
        // A zero-length outline: one point, covering all of its progress.
        None => measured.push(MeasuredCubic::new(
            measurer,
            cubics.first().copied().unwrap_or_default(),
            0.,
            1.,
        )),
    }
    measured
}

fn measure_polygon(
    measurer: Measurer,
    polygon: &RoundedPolygon,
) -> (Vec<MeasuredCubic>, Vec<MeasuredCorner>) {
    let mut cubics = Vec::new();
    let mut corner_cubics: Vec<(usize, Pt, bool)> = Vec::new();
    for feature in &polygon.features {
        for (index, cubic) in feature.cubics.iter().enumerate() {
            if let Some(convex) = feature.corner
                && index == feature.cubics.len() / 2
            {
                let first = feature.cubics.first().copied().unwrap_or_default();
                let last = feature.cubics.last().copied().unwrap_or_default();
                let representative = pt(
                    (first.points[0] + last.points[6]) / 2.,
                    (first.points[1] + last.points[7]) / 2.,
                );
                corner_cubics.push((cubics.len(), representative, convex));
            }
            cubics.push(*cubic);
        }
    }
    let mut measures = Vec::with_capacity(cubics.len() + 1);
    measures.push(0_f32);
    let mut sum = 0_f32;
    for cubic in &cubics {
        sum += measurer.measure(cubic);
        measures.push(sum);
    }
    let outline_progress: Vec<f32> = measures.iter().map(|m| m / sum).collect();
    let corners = corner_cubics
        .into_iter()
        .map(|(i, representative, convex)| MeasuredCorner {
            progress: {
                let middle = (outline_progress[i] + outline_progress[i + 1]) / 2.;
                // 1.0.1 has no wrap here, and the wrap's rounding decides ties.
                match measurer {
                    Measurer::Length => positive_modulo(middle, 1.),
                    Measurer::Angle(_) => middle,
                }
            },
            representative,
            convex,
        })
        .collect();
    (
        measured_cubics(measurer, &cubics, &outline_progress),
        corners,
    )
}

/// The outline, restarted at `cut`: its progress runs from 0 there.
fn cut_and_shift(cubics: Vec<MeasuredCubic>, cut: f32) -> Vec<MeasuredCubic> {
    if cut < DISTANCE_EPSILON {
        return cubics;
    }
    let target_index = cubics
        .iter()
        .position(|c| (c.start..=c.end).contains(&cut))
        .unwrap_or(0);
    let measurer = cubics[target_index].measurer;
    let (b1, b2) = cubics[target_index].cut_at(cut);
    let n = cubics.len();
    let mut shifted = Vec::with_capacity(n + 1);
    shifted.push(b2.cubic);
    for i in 1..n {
        shifted.push(cubics[(i + target_index) % n].cubic);
    }
    shifted.push(b1.cubic);
    let mut progress = Vec::with_capacity(n + 2);
    for index in 0..n + 2 {
        progress.push(if index == 0 {
            0.
        } else if index == n + 1 {
            1.
        } else {
            positive_modulo(cubics[(target_index + index - 1) % n].end - cut, 1.)
        });
    }
    measured_cubics(measurer, &shifted, &progress)
}

fn match_cubics(
    p1: &RoundedPolygon,
    p2: &RoundedPolygon,
    matching: MorphMatching,
) -> Vec<(Cubic, Cubic)> {
    let (bs1, corners1) = measure_polygon(Measurer::of(matching, p1), p1);
    let (measured2, corners2) = measure_polygon(Measurer::of(matching, p2), p2);
    let mapper = match matching {
        MorphMatching::Current => feature_mapper(&corners1, &corners2),
        MorphMatching::Compose1_5 => greedy_mapper(&corners1, &corners2),
    };
    let cut = mapper.map(0.);
    let bs2 = cut_and_shift(measured2, cut);

    let mut matched = Vec::with_capacity(bs1.len().max(bs2.len()) * 2);
    let (mut i1, mut i2) = (1, 1);
    let mut b1 = bs1.first().copied();
    let mut b2 = bs2.first().copied();
    while let (Some(c1), Some(c2)) = (b1, b2) {
        let b1a = if i1 == bs1.len() { 1. } else { c1.end };
        let b2a = if i2 == bs2.len() {
            1.
        } else {
            mapper.map_back(positive_modulo(c2.end + cut, 1.))
        };
        let min = b1a.min(b2a);
        let (segment1, next1) = if b1a > min + ANGLE_EPSILON {
            let (a, b) = c1.cut_at(min);
            (a, Some(b))
        } else {
            let next = bs1.get(i1).copied();
            i1 += 1;
            (c1, next)
        };
        let (segment2, next2) = if b2a > min + ANGLE_EPSILON {
            let (a, b) = c2.cut_at(positive_modulo(mapper.map(min) - cut, 1.));
            (a, Some(b))
        } else {
            let next = bs2.get(i2).copied();
            i2 += 1;
            (c2, next)
        };
        matched.push((segment1.cubic, segment2.cubic));
        b1 = next1;
        b2 = next2;
    }
    matched
}

/// An animation between two [`RoundedPolygon`]s of any structure: both
/// outlines cut into matching curves. See `androidx.graphics.shapes.Morph`.
#[derive(Clone, Debug)]
pub struct Morph {
    start: RoundedPolygon,
    end: RoundedPolygon,
    matched: Vec<(Cubic, Cubic)>,
}

impl Morph {
    /// Match `start` and `end`. This measures both outlines, so hold on to
    /// the result: [`ShapeKind::morph_to`](super::ShapeKind::morph_to) keeps
    /// one per pair of Material shapes.
    pub fn new(start: &RoundedPolygon, end: &RoundedPolygon) -> Self {
        Self::with_matching(start, end, MorphMatching::default())
    }

    /// [`Self::new`], matching the outlines by `matching`.
    pub fn with_matching(
        start: &RoundedPolygon,
        end: &RoundedPolygon,
        matching: MorphMatching,
    ) -> Self {
        Self {
            start: start.clone(),
            end: end.clone(),
            matched: match_cubics(start, end, matching),
        }
    }

    /// How many curves the outline has at any progress.
    pub fn len(&self) -> usize {
        self.matched.len()
    }

    /// Whether the morph has no curves. Polygons always give it at least one.
    pub fn is_empty(&self) -> bool {
        self.matched.is_empty()
    }

    /// Visit the outline at `progress`, without allocating: 0 is the start
    /// shape, 1 the end shape, and values beyond them extrapolate, for a
    /// spring's overshoot.
    pub fn for_each_cubic(&self, progress: f32, mut each: impl FnMut(&Cubic)) {
        for (a, b) in &self.matched {
            let mut cubic = Cubic::default();
            for (value, (from, to)) in cubic.points.iter_mut().zip(a.points.iter().zip(&b.points)) {
                *value = interpolate(*from, *to, progress);
            }
            each(&cubic);
        }
    }

    /// The outline at `progress`, closed exactly on its first anchor.
    pub fn as_cubics(&self, progress: f32) -> Vec<Cubic> {
        let mut cubics = Vec::with_capacity(self.matched.len());
        self.for_each_cubic(progress, |cubic| cubics.push(*cubic));
        if let [first, .., last] = cubics.as_mut_slice() {
            last.points[6] = first.points[0];
            last.points[7] = first.points[1];
        }
        cubics
    }

    /// The box covering both shapes' [`RoundedPolygon::bounds`].
    pub fn bounds(&self, approximate: bool) -> [f32; 4] {
        union(self.start.bounds(approximate), self.end.bounds(approximate))
    }

    /// The box covering both shapes' [`RoundedPolygon::max_bounds`].
    pub fn max_bounds(&self) -> [f32; 4] {
        union(self.start.max_bounds(), self.end.max_bounds())
    }
}

fn union(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        a[0].min(b[0]),
        a[1].min(b[1]),
        a[2].max(b[2]),
        a[3].max(b[3]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::polygon::CornerRounding;
    use core::prelude::v1::test;

    #[test]
    fn degenerate_and_mismatched_polygons_morph_without_panicking() {
        let point =
            RoundedPolygon::from_vertices(&[(1., 1.); 3], CornerRounding::UNROUNDED, None, None)
                .expect("finite");
        let star = RoundedPolygon::star(5, 1., 0.4, CornerRounding::new(0.1), None, (0., 0.))
            .expect("valid");
        let triangle = RoundedPolygon::regular(3, 1., (0., 0.), CornerRounding::UNROUNDED, None)
            .expect("valid");
        for (a, b) in [
            (&point, &star),
            (&star, &point),
            (&point, &point),
            (&star, &triangle),
            (&triangle, &star),
        ] {
            let morph = Morph::new(a, b);
            assert!(!morph.is_empty());
            for progress in [0., 0.5, 1., 1.2] {
                assert!(
                    morph
                        .as_cubics(progress)
                        .iter()
                        .all(|c| c.points.iter().all(|v| v.is_finite())),
                    "{progress}"
                );
            }
        }
    }

    #[test]
    fn the_mapper_rejects_repeats_and_double_wraps() {
        assert!(valid_progress(&[0.3, 0.6, 0.]));
        assert!(!valid_progress(&[0.5, 0., 0.7]));
        assert!(!valid_progress(&[0.5]));
        assert!(!valid_progress(&[]));
        assert!(!valid_progress(&[0.2, 1.2]));
        let identity = DoubleMapper::new(&[(0.5, 0.5)]);
        assert_eq!(identity.map(0.25), 0.25);
    }
}
