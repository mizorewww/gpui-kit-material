//! Analytic intersection of a ripple disk and the button's rounded border box.
use gpui::{Bounds, Corners, FillOptions, Path, PathBuilder, PathStyle, Pixels, Point, point, px};
use std::f64::consts::{FRAC_PI_2, TAU};

const EPS: f64 = 1e-7;
type XY = [f64; 2];

#[derive(Clone, Copy, Debug)]
enum Edge {
    Line {
        start: XY,
        end: XY,
    },
    Arc {
        center: XY,
        radius: f64,
        start: f64,
        sweep: f64,
    },
}

fn distance(a: XY, b: XY) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

impl Edge {
    fn at(self, t: f64) -> XY {
        match self {
            Self::Line { start, end } => [
                start[0] + (end[0] - start[0]) * t,
                start[1] + (end[1] - start[1]) * t,
            ],
            Self::Arc {
                center,
                radius,
                start,
                sweep,
            } => {
                let angle = start + sweep * t;
                [
                    center[0] + radius * angle.cos(),
                    center[1] + radius * angle.sin(),
                ]
            }
        }
    }

    fn parameter(self, p: XY) -> Option<f64> {
        let t = match self {
            Self::Line { start, end } => {
                let dx = end[0] - start[0];
                let dy = end[1] - start[1];
                let length_squared = dx * dx + dy * dy;
                if length_squared <= EPS * EPS {
                    return None;
                }
                ((p[0] - start[0]) * dx + (p[1] - start[1]) * dy) / length_squared
            }
            Self::Arc {
                center,
                start,
                sweep,
                ..
            } => {
                let angle = (p[1] - center[1]).atan2(p[0] - center[0]);
                let mut delta = (angle - start).rem_euclid(TAU);
                if TAU - delta < EPS {
                    delta = 0.;
                }
                delta / sweep
            }
        };
        (-EPS..=1. + EPS).contains(&t).then_some(t.clamp(0., 1.))
    }

    fn slice(self, from: f64, to: f64) -> Self {
        match self {
            Self::Line { .. } => Self::Line {
                start: self.at(from),
                end: self.at(to),
            },
            Self::Arc {
                center,
                radius,
                start,
                sweep,
            } => Self::Arc {
                center,
                radius,
                start: start + sweep * from,
                sweep: sweep * (to - from),
            },
        }
    }
}

// Return intersections with the supporting circle; the caller filters by arc span.
fn circle_intersections(edge: Edge, center: XY, radius: f64) -> Vec<XY> {
    match edge {
        Edge::Line { start, end } => {
            let d = [end[0] - start[0], end[1] - start[1]];
            let f = [start[0] - center[0], start[1] - center[1]];
            let a = d[0] * d[0] + d[1] * d[1];
            if a <= EPS * EPS {
                return vec![];
            }
            let b = f[0] * d[0] + f[1] * d[1];
            let c = f[0] * f[0] + f[1] * f[1] - radius * radius;
            let discriminant = b * b - a * c;
            if discriminant < 0. {
                return vec![];
            }
            let root = discriminant.sqrt();
            [(-b - root) / a, (-b + root) / a]
                .into_iter()
                .filter(|t| (0. ..=1.).contains(t))
                .map(|t| edge.at(t))
                .collect()
        }
        Edge::Arc {
            center: other,
            radius: other_radius,
            ..
        } => {
            let d = distance(center, other);
            if d <= EPS || d > radius + other_radius || d < (radius - other_radius).abs() {
                return vec![];
            }
            let along = (radius * radius - other_radius * other_radius + d * d) / (2. * d);
            let height = (radius * radius - along * along).max(0.).sqrt();
            let unit = [(other[0] - center[0]) / d, (other[1] - center[1]) / d];
            let base = [center[0] + along * unit[0], center[1] + along * unit[1]];
            [-1., 1.]
                .into_iter()
                .map(|sign| {
                    [
                        base[0] - sign * height * unit[1],
                        base[1] + sign * height * unit[0],
                    ]
                })
                .filter(|p| edge.parameter(*p).is_some())
                .collect()
        }
    }
}

struct RoundedRect {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
    // Clockwise, starting at top-right. GPUI clamps each to half the shorter side.
    radii: [f64; 4],
}
impl RoundedRect {
    fn radius_at(&self, p: XY) -> f64 {
        let right = p[0] >= (self.left + self.right) / 2.;
        let bottom = p[1] >= (self.top + self.bottom) / 2.;
        self.radii[match (right, bottom) {
            (true, false) => 0,
            (true, true) => 1,
            (false, true) => 2,
            (false, false) => 3,
        }]
    }

    // Exact exterior distance and signed interior containment. Because GPUI
    // limits every radius to half the shorter side, a corner arc never leaves
    // its quadrant; only that quadrant's radius affects exterior projection.
    fn distance(&self, p: XY) -> f64 {
        let radius = self.radius_at(p);
        let x =
            (p[0] - (self.left + self.right) / 2.).abs() - (self.right - self.left) / 2. + radius;
        let y =
            (p[1] - (self.top + self.bottom) / 2.).abs() - (self.bottom - self.top) / 2. + radius;
        x.max(0.).hypot(y.max(0.)) + x.max(y).min(0.) - radius
    }

    fn project(&self, p: XY) -> XY {
        if self.distance(p) <= 0. {
            return p;
        }
        let radius = self.radius_at(p);
        let core = [
            p[0].clamp(self.left + radius, self.right - radius),
            p[1].clamp(self.top + radius, self.bottom - radius),
        ];
        let scale = radius / distance(core, p);
        [
            core[0] + (p[0] - core[0]) * scale,
            core[1] + (p[1] - core[1]) * scale,
        ]
    }

    fn edges(&self) -> Vec<Edge> {
        let [tr, br, bl, tl] = self.radii;
        let corners = [
            [self.right - tr, self.top + tr],
            [self.right - br, self.bottom - br],
            [self.left + bl, self.bottom - bl],
            [self.left + tl, self.top + tl],
        ];
        let mut edges = Vec::with_capacity(8);
        for i in 0..4 {
            let arc = Edge::Arc {
                center: corners[i],
                radius: self.radii[i],
                start: (i as f64 - 1.) * FRAC_PI_2,
                sweep: FRAC_PI_2,
            };
            let previous = Edge::Arc {
                center: corners[(i + 3) % 4],
                radius: self.radii[(i + 3) % 4],
                start: (i as f64 - 2.) * FRAC_PI_2,
                sweep: FRAC_PI_2,
            };
            let line = Edge::Line {
                start: previous.at(1.),
                end: arc.at(0.),
            };
            if distance(line.at(0.), line.at(1.)) > EPS {
                edges.push(line);
            }
            if self.radii[i] > EPS {
                edges.push(arc);
            }
        }
        edges
    }
}

fn split_inside(
    edge: Edge,
    mut cuts: Vec<f64>,
    inside: impl Fn(XY) -> bool,
    output: &mut Vec<Edge>,
) {
    cuts.extend([0., 1.]);
    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|a, b| (*a - *b).abs() < EPS);
    for pair in cuts.windows(2) {
        if pair[1] - pair[0] > EPS && inside(edge.at((pair[0] + pair[1]) / 2.)) {
            output.push(edge.slice(pair[0], pair[1]));
        }
    }
}

/// Build the filled ripple intersection, preserving circular outline segments.
/// Degenerate or invalid inputs have no painted area.
pub(super) fn paint_path(
    bounds: Bounds<Pixels>,
    corners: Corners<Pixels>,
    center: Point<Pixels>,
    radius: Pixels,
) -> Option<Path<Pixels>> {
    let [left, top, width, height, x, y, radius] = [
        bounds.origin.x,
        bounds.origin.y,
        bounds.size.width,
        bounds.size.height,
        center.x,
        center.y,
        radius,
    ]
    .map(|v| f64::from(f32::from(v)));
    if ![left, top, width, height, x, y, radius]
        .into_iter()
        .all(f64::is_finite)
        || width <= 0.
        || height <= 0.
        || radius <= 0.
    {
        return None;
    }
    if ![
        corners.top_left,
        corners.top_right,
        corners.bottom_left,
        corners.bottom_right,
    ]
    .into_iter()
    .all(|r| f32::from(r).is_finite())
    {
        return None;
    }
    let corners = corners
        .map(|r| (*r).max(px(0.)))
        .clamp_radii_for_quad_size(bounds.size);
    let rect = RoundedRect {
        left,
        top,
        right: left + width,
        bottom: top + height,
        radii: [
            corners.top_right,
            corners.bottom_right,
            corners.bottom_left,
            corners.top_left,
        ]
        .map(|r| f64::from(f32::from(r))),
    };
    let center = [x, y];
    if rect.distance(center) >= radius {
        return None;
    }
    let outline = rect.edges();
    let mut pieces = Vec::new();
    let mut crossings = Vec::new();
    for edge in outline {
        let intersections = circle_intersections(edge, center, radius);
        let cuts = intersections
            .iter()
            .filter_map(|p| edge.parameter(*p))
            .collect();
        crossings.extend(intersections);
        split_inside(
            edge,
            cuts,
            |p| distance(p, center) <= radius + EPS,
            &mut pieces,
        );
    }
    for quadrant in 0..4 {
        let edge = Edge::Arc {
            center,
            radius,
            start: quadrant as f64 * FRAC_PI_2,
            sweep: FRAC_PI_2,
        };
        let cuts = crossings
            .iter()
            .filter_map(|p| edge.parameter(*p))
            .collect();
        // Strict containment avoids doubling coincident corner/ripple arcs.
        split_inside(edge, cuts, |p| rect.distance(p) < -EPS, &mut pieces);
    }
    if pieces.is_empty() {
        return None;
    }
    // Convexity guarantees the average of boundary points is inside the intersection.
    let interior = pieces
        .iter()
        .map(|e| e.at(0.5))
        .fold([0., 0.], |a, p| [a[0] + p[0], a[1] + p[1]])
        .map(|v| v / pieces.len() as f64);
    pieces.sort_by(|a, b| {
        let angle = |edge: &Edge| {
            let p = edge.at(0.5);
            (p[1] - interior[1]).atan2(p[0] - interior[0])
        };
        angle(a).total_cmp(&angle(b))
    });
    let to_point = |p: XY| point(px(p[0] as f32), px(p[1] as f32));
    let mut builder = PathBuilder::fill()
        .with_style(PathStyle::Fill(FillOptions::default().with_tolerance(0.01)));
    builder.move_to(to_point(pieces[0].at(0.)));
    for edge in pieces {
        match edge {
            Edge::Line { end, .. } => builder.line_to(to_point(end)),
            Edge::Arc { radius, sweep, .. } => {
                // Lyon converts SVG arcs to quadratic curves. Short circular arcs
                // keep that conversion's outward error below a subpixel, while
                // preserving analytic arc geometry instead of polygon clipping.
                let steps = (sweep / (std::f64::consts::PI / 24.)).ceil() as usize;
                for step in 1..=steps {
                    builder.arc_to(
                        point(px(radius as f32), px(radius as f32)),
                        px(0.),
                        false,
                        true,
                        to_point(edge.at(step as f64 / steps as f64)),
                    );
                }
            }
        }
    }
    builder.close();
    let mut path = builder.build().ok()?;
    if path.vertices.is_empty() {
        return None;
    }
    // Tessellation can slightly move arc endpoints (even beyond the requested
    // tolerance). Project those vertices back onto the analytic convex intersection.
    // A nearest boundary point lies on either outline or at an outline crossing.
    for vertex in &mut path.vertices {
        let p = [
            f64::from(f32::from(vertex.xy_position.x)),
            f64::from(f32::from(vertex.xy_position.y)),
        ];
        if rect.distance(p) <= 0. && distance(p, center) <= radius {
            continue;
        }
        let d = distance(p, center);
        let scale = if d > radius { radius / d } else { 1. };
        let on_circle = [
            center[0] + (p[0] - center[0]) * scale,
            center[1] + (p[1] - center[1]) * scale,
        ];
        let candidates = [rect.project(p), on_circle];
        if let Some(nearest) = candidates
            .iter()
            .chain(crossings.iter())
            .filter(|q| rect.distance(**q) <= EPS && distance(**q, center) <= radius + EPS)
            .min_by(|a, b| distance(**a, p).total_cmp(&distance(**b, p)))
        {
            vertex.xy_position = to_point(*nearest);
        }
    }
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::size;

    fn paint_path(
        bounds: Bounds<Pixels>,
        corner: Pixels,
        center: Point<Pixels>,
        radius: Pixels,
    ) -> Option<Path<Pixels>> {
        super::paint_path(bounds, Corners::all(corner), center, radius)
    }

    fn bounds() -> Bounds<Pixels> {
        Bounds::new(point(px(10.), px(20.)), size(px(120.), px(40.)))
    }

    #[test]
    fn contained_disjoint_and_degenerate_disks() {
        let b = bounds();
        assert!(paint_path(b, px(20.), b.center(), px(5.)).is_some());
        assert!(paint_path(b, px(20.), b.center(), px(200.)).is_some());
        assert!(paint_path(b, px(20.), point(px(-50.), px(0.)), px(3.)).is_none());
        assert!(paint_path(b, px(20.), b.center(), px(0.)).is_none());
        assert!(paint_path(b, px(20.), b.center(), px(f32::NAN)).is_none());
        assert!(
            paint_path(
                Bounds::new(b.origin, size(px(0.), px(0.))),
                px(0.),
                b.center(),
                px(5.)
            )
            .is_none()
        );
    }

    fn area(path: &Path<Pixels>) -> f64 {
        path.vertices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|triangle| {
                let p = triangle
                    .iter()
                    .map(|v| {
                        [
                            f64::from(f32::from(v.xy_position.x)),
                            f64::from(f32::from(v.xy_position.y)),
                        ]
                    })
                    .collect::<Vec<_>>();
                ((p[1][0] - p[0][0]) * (p[2][1] - p[0][1])
                    - (p[1][1] - p[0][1]) * (p[2][0] - p[0][0]))
                    .abs()
                    / 2.
            })
            .sum()
    }

    #[test]
    fn intersection_retains_the_expected_area() {
        let b = bounds();
        let disk = paint_path(b, px(20.), b.center(), px(10.)).unwrap();
        assert!((area(&disk) - std::f64::consts::PI * 100.).abs() < 3.);
        let half_disk = paint_path(b, px(0.), point(px(10.), px(40.)), px(10.)).unwrap();
        assert!((area(&half_disk) - std::f64::consts::PI * 50.).abs() < 2.);
        let whole = paint_path(b, px(20.), b.center(), px(200.)).unwrap();
        assert!((area(&whole) - (80. * 40. + std::f64::consts::PI * 400.)).abs() < 8.);
        // The left cap coincides with the ripple's left semicircle.
        let coincident = paint_path(b, px(20.), point(px(30.), px(40.)), px(20.)).unwrap();
        assert!((area(&coincident) - std::f64::consts::PI * 400.).abs() < 8.);
    }

    #[test]
    fn asymmetric_corners_preserve_area_and_clip_every_vertex() {
        let b = bounds();
        for raw in [
            [0_f32, 4., 12., 20.],
            [20., 0., 4., 12.],
            [12., 20., 0., 4.],
            [4., 12., 20., 0.],
            [-4., 200., 8., 3.],
        ] {
            let corners = Corners {
                top_right: px(raw[0]),
                bottom_right: px(raw[1]),
                bottom_left: px(raw[2]),
                top_left: px(raw[3]),
            };
            let radii = raw.map(|r| f64::from(r.clamp(0., 20.)));
            let whole = super::paint_path(b, corners, b.center(), px(200.)).unwrap();
            let expected = 120. * 40.
                - (1. - std::f64::consts::PI / 4.) * radii.iter().map(|r| r * r).sum::<f64>();
            assert!((area(&whole) - expected).abs() < 3.);
            for center in [[10., 20.], [130., 20.], [130., 60.], [10., 60.], [70., 40.]] {
                for radius in [5., 20., 35., 200.] {
                    if let Some(path) = super::paint_path(
                        b,
                        corners,
                        point(px(center[0] as f32), px(center[1] as f32)),
                        px(radius as f32),
                    ) {
                        for vertex in path.vertices {
                            let p = [
                                f64::from(f32::from(vertex.xy_position.x)),
                                f64::from(f32::from(vertex.xy_position.y)),
                            ];
                            assert!((10. - 0.001..=130. + 0.001).contains(&p[0]));
                            assert!((20. - 0.001..=60. + 0.001).contains(&p[1]));
                            assert!(distance(p, center) <= radius + 0.001);
                            // Independently check each rounded cutout's quadrant.
                            for (i, r) in radii.into_iter().enumerate() {
                                let right = i < 2;
                                let bottom = i == 1 || i == 2;
                                let c = [
                                    if right { 130. - r } else { 10. + r },
                                    if bottom { 60. - r } else { 20. + r },
                                ];
                                let in_x = if right { p[0] > c[0] } else { p[0] < c[0] };
                                let in_y = if bottom { p[1] > c[1] } else { p[1] < c[1] };
                                if in_x && in_y {
                                    assert!(distance(p, c) <= r + 0.001);
                                }
                            }
                        }
                    }
                }
            }
        }
        let invalid = Corners {
            top_left: px(f32::NAN),
            ..Corners::all(px(0.))
        };
        assert!(super::paint_path(b, invalid, b.center(), px(20.)).is_none());
        let zero = Bounds::new(b.origin, size(px(0.), px(40.)));
        assert!(super::paint_path(zero, Corners::all(px(200.)), b.center(), px(20.)).is_none());
    }

    #[test]
    fn corner_and_edge_intersections_stay_inside_both_shapes() {
        let b = bounds();
        for corner in [0_f32, 4., 20., 200.] {
            let rect = RoundedRect {
                left: 10.,
                top: 20.,
                right: 130.,
                bottom: 60.,
                radii: [f64::from(corner.min(20.)); 4],
            };
            for center in [
                [10., 20.],
                [10., 40.],
                [30., 40.],
                [70., 20.],
                [70., 40.],
                [130., 60.],
            ] {
                for radius in [1., 5., 20., 35., 80., 200.] {
                    let path = paint_path(
                        b,
                        px(corner),
                        point(px(center[0] as f32), px(center[1] as f32)),
                        px(radius as f32),
                    );
                    if let Some(path) = path {
                        for vertex in path.vertices {
                            let p = [
                                f64::from(f32::from(vertex.xy_position.x)),
                                f64::from(f32::from(vertex.xy_position.y)),
                            ];
                            assert!(
                                rect.distance(p) <= 0.001,
                                "outside rounded rect: {p:?}, center={center:?}, radius={radius}, corner={corner}"
                            );
                            assert!(
                                distance(p, center) <= radius + 0.001,
                                "outside disk: {p:?}, center={center:?}, radius={radius}, corner={corner}"
                            );
                        }
                    } else {
                        assert!(rect.distance(center) >= radius);
                    }
                }
            }
        }
    }

    #[test]
    fn independent_vertical_integration_matches_varied_intersections() {
        // An independent reference integrates vertical disk/rounded-box slices;
        // it does not use Edge, RoundedRect::distance, intersections or sorting.
        let b = bounds();
        let mut seed = 0x91b5_16afu32;
        let mut next = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            f64::from(seed) / f64::from(u32::MAX)
        };
        for case in 0..160 {
            let raw = std::array::from_fn::<_, 4, _>(|_| (next() * 20.) as f32);
            let radii = raw.map(f64::from);
            let center = [(next() * 180. - 20.) as f32, (next() * 100. - 10.) as f32];
            let radius = (next() * 100. + 0.1) as f32;
            let corners = Corners {
                top_right: px(raw[0]),
                bottom_right: px(raw[1]),
                bottom_left: px(raw[2]),
                top_left: px(raw[3]),
            };
            let path =
                super::paint_path(b, corners, point(px(center[0]), px(center[1])), px(radius));
            let [cx, cy] = center.map(f64::from);
            let r = f64::from(radius);
            let mut expected = 0.;
            let dx = 120. / 4096.;
            for column in 0..4096 {
                let x = 10. + (f64::from(column) + 0.5) * dx;
                let disk_squared = r * r - (x - cx).powi(2);
                if disk_squared <= 0. {
                    continue;
                }
                let mut top = 20_f64;
                let mut bottom = 60_f64;
                for (i, cr) in radii.into_iter().enumerate() {
                    let right = i < 2;
                    let lower = i == 1 || i == 2;
                    let corner_x = if right { 130. - cr } else { 10. + cr };
                    if (right && x > corner_x) || (!right && x < corner_x) {
                        let inset = cr - (cr * cr - (x - corner_x).powi(2)).max(0.).sqrt();
                        if lower {
                            bottom = bottom.min(60. - inset);
                        } else {
                            top = top.max(20. + inset);
                        }
                    }
                }
                let half = disk_squared.sqrt();
                expected += (bottom.min(cy + half) - top.max(cy - half)).max(0.) * dx;
            }
            let actual = path.as_ref().map(area).unwrap_or(0.);
            assert!(
                (actual - expected).abs() < 2. + expected * 0.003,
                "case={case}, radii={radii:?}, center={center:?}, radius={radius}, actual={actual}, expected={expected}"
            );
            if let Some(path) = path {
                assert!(path.vertices.iter().all(|v| v.st_position == point(0., 1.)));
            }
        }
    }

    #[test]
    fn complete_coincidence_and_external_tangency_have_expected_area() {
        let disk_bounds = Bounds::new(point(px(10.), px(20.)), size(px(40.), px(40.)));
        let disk = paint_path(disk_bounds, px(20.), disk_bounds.center(), px(20.)).unwrap();
        assert!((area(&disk) - std::f64::consts::PI * 400.).abs() < 3.);
        // Tangency on a straight edge and at the rounded cap has zero area.
        assert!(paint_path(bounds(), px(0.), point(px(-10.), px(40.)), px(20.)).is_none());
        assert!(paint_path(bounds(), px(20.), point(px(-10.), px(40.)), px(20.)).is_none());
        assert!(paint_path(bounds(), px(20.), point(px(30.), px(0.)), px(20.)).is_none());
    }
}
