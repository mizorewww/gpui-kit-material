//! M3 Expressive loading indicator: a shape that morphs through Compose's
//! seven Material shapes while it rotates, optionally inside a round
//! container. Reference: <https://m3e.kyralabs.dev/components/loading-indicator>
//! and the Figma kit's "Loading indicator" set (48dp container, 38dp
//! active indicator, primary or on-primary-container on primary-container).
//!
//! Each shape is a rounded polygon (straight edges joined by circular corner
//! arcs, as `androidx.graphics.shapes.RoundedPolygon` builds them). Every
//! shape is star-shaped around its center, so it is resampled once into a
//! fixed table of radii at equal angles; morphing interpolates those radii
//! and the result is painted as one GPUI path.
use crate::{color::ColorRole, motion::Spring, theme::color, tokens::typography::rem};
use gpui::{prelude::*, *};
use gpui_base::Progress;
use std::{f32::consts::TAU, sync::OnceLock, time::Duration};

/// One of the Material shapes the loading indicator morphs between.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LoadingShape {
    /// The determinate indicator's empty state.
    Circle,
    SoftBurst,
    Cookie9Sided,
    Pentagon,
    Pill,
    Sunny,
    Cookie4Sided,
    Oval,
}

impl LoadingShape {
    /// The indeterminate sequence, in the order the reference plays it.
    pub const SEQUENCE: [Self; 7] = [
        Self::SoftBurst,
        Self::Cookie9Sided,
        Self::Pentagon,
        Self::Pill,
        Self::Sunny,
        Self::Cookie4Sided,
        Self::Oval,
    ];

    fn index(self) -> usize {
        self as usize
    }
}

/// The container (48dp) and the indicator's radius inside it: the 38dp
/// active-indicator box, inset so a rotating shape stays inside it (the
/// kit's soft burst measures 33-34dp across).
pub const LOADING_INDICATOR_SIZE: f32 = 48.;
const SHAPE_RADIUS: f32 = 17.5;

/// One morph lasts this long on the reference's spring, then the next starts.
const MORPH_INTERVAL_MS: f32 = 650.;
/// The morph spring (Compose: damping ratio 0.6, stiffness 200).
const MORPH_SPRING: Spring = Spring {
    damping: 0.6,
    stiffness: 200.,
};
/// Each morph also turns the shape a quarter turn.
const MORPH_ROTATION: f32 = 90.;
/// Four passes through the sequence: 28 quarter turns (seven full turns)
/// make the loop seamless. The constant rotation turns four times per loop,
/// one turn per 4550ms (the reference's 4666ms, rounded to the loop).
const LOOP_MORPHS: usize = 28;
const LOOP_MS: f32 = MORPH_INTERVAL_MS * LOOP_MORPHS as f32;
const LOOP_TURNS: f32 = 4.;

/// A sampled indicator: the shapes being morphed, how far, and the turn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoadingFrame {
    pub from: LoadingShape,
    pub to: LoadingShape,
    /// 0 at `from`, 1 at `to`; the spring briefly overshoots past 1.
    pub morph: f32,
    pub rotation_degrees: f32,
}

impl LoadingFrame {
    /// A shape at rest, unrotated.
    pub fn still(shape: LoadingShape) -> Self {
        Self {
            from: shape,
            to: shape,
            morph: 0.,
            rotation_degrees: 0.,
        }
    }

    /// The determinate indicator: a circle at 0 that becomes the soft
    /// burst at 1, turning half a turn on the way.
    pub fn determinate(progress: f32) -> Self {
        let progress = if progress.is_finite() {
            progress.clamp(0., 1.)
        } else {
            0.
        };
        Self {
            from: LoadingShape::Circle,
            to: LoadingShape::SoftBurst,
            morph: progress,
            rotation_degrees: 180. * progress,
        }
    }
}

/// The indeterminate indicator `elapsed_ms` into its loop.
pub fn loading_indicator_frame(elapsed_ms: f32) -> LoadingFrame {
    let t = if elapsed_ms.is_finite() {
        elapsed_ms.rem_euclid(LOOP_MS)
    } else {
        0.
    };
    let index = ((t / MORPH_INTERVAL_MS).floor() as usize).min(LOOP_MORPHS - 1);
    let local = t - index as f32 * MORPH_INTERVAL_MS;
    // The spring is within 0.5% of its target when the interval ends; take
    // up that remainder linearly so the next morph starts exactly there.
    let seconds = local / 1000.;
    let rest = 1. - MORPH_SPRING.position(MORPH_INTERVAL_MS / 1000.);
    let morph = MORPH_SPRING.position(seconds) + rest * local / MORPH_INTERVAL_MS;
    let sequence = LoadingShape::SEQUENCE;
    LoadingFrame {
        from: sequence[index % sequence.len()],
        to: sequence[(index + 1) % sequence.len()],
        morph,
        rotation_degrees: (360. * LOOP_TURNS * t / LOOP_MS
            + MORPH_ROTATION * (index as f32 + morph))
            .rem_euclid(360.),
    }
}

/// Radii sampled at this many equal angles, the first at three o'clock.
const SAMPLES: usize = 120;
type Radii = [f32; SAMPLES];

/// A polygon vertex and the radius of the arc that rounds it.
#[derive(Clone, Copy)]
struct Vertex {
    x: f32,
    y: f32,
    rounding: f32,
}

fn star(points: usize, inner: f32, outer_round: f32, inner_round: f32, turn: f32) -> Vec<Vertex> {
    (0..points * 2)
        .map(|i| {
            let angle = turn + i as f32 * TAU / (points * 2) as f32;
            let (radius, rounding) = if i % 2 == 0 {
                (1., outer_round)
            } else {
                (inner, inner_round)
            };
            Vertex {
                x: radius * angle.cos(),
                y: radius * angle.sin(),
                rounding,
            }
        })
        .collect()
}

fn regular(points: usize, rounding: f32, turn: f32) -> Vec<Vertex> {
    (0..points)
        .map(|i| {
            let angle = turn + i as f32 * TAU / points as f32;
            Vertex {
                x: angle.cos(),
                y: angle.sin(),
                rounding,
            }
        })
        .collect()
}

/// Scale by (`sx`, `sy`), then turn by `turn` radians.
fn transformed(vertices: Vec<Vertex>, sx: f32, sy: f32, turn: f32) -> Vec<Vertex> {
    let (sin, cos) = turn.sin_cos();
    vertices
        .into_iter()
        .map(|v| {
            let (x, y) = (v.x * sx, v.y * sy);
            Vertex {
                x: x * cos - y * sin,
                y: x * sin + y * cos,
                rounding: v.rounding,
            }
        })
        .collect()
}

/// The vertices of each shape, from the Compose `MaterialShapes` outlines
/// rescaled to a unit outer radius.
fn vertices(shape: LoadingShape) -> Vec<Vertex> {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};
    match shape {
        LoadingShape::Circle => regular(48, 0., 0.),
        LoadingShape::SoftBurst => star(10, 0.69, 0.096, 0.096, -FRAC_PI_2),
        LoadingShape::Cookie9Sided => star(9, 0.8, 0.5, 0.5, -FRAC_PI_2),
        LoadingShape::Pentagon => regular(5, 0.31, -FRAC_PI_2),
        // A capsule: a 2 x 1.3 rectangle with fully rounded ends, on the
        // rising diagonal.
        LoadingShape::Pill => transformed(
            vec![
                Vertex {
                    x: 1.,
                    y: 0.65,
                    rounding: 0.649,
                },
                Vertex {
                    x: -1.,
                    y: 0.65,
                    rounding: 0.649,
                },
                Vertex {
                    x: -1.,
                    y: -0.65,
                    rounding: 0.649,
                },
                Vertex {
                    x: 1.,
                    y: -0.65,
                    rounding: 0.649,
                },
            ],
            1.,
            1.,
            -FRAC_PI_4,
        ),
        LoadingShape::Sunny => star(8, 0.8, 0.15, 0.15, -FRAC_PI_2),
        LoadingShape::Cookie4Sided => star(4, 0.4, 0.248, 0.224, -FRAC_PI_4),
        LoadingShape::Oval => transformed(regular(48, 0., 0.), 1., 0.64, -FRAC_PI_4),
    }
}

/// The outline of a rounded polygon as a closed polyline: each corner is
/// replaced by the circular arc of its rounding radius tangent to both
/// edges, shrunk where two corners would overlap on one edge.
fn rounded_outline(vertices: &[Vertex]) -> Vec<(f32, f32)> {
    const ARC_STEPS: usize = 10;
    let n = vertices.len();
    let mut outline = Vec::with_capacity(n * (ARC_STEPS + 1));
    for i in 0..n {
        let p = vertices[i];
        let a = vertices[(i + n - 1) % n];
        let b = vertices[(i + 1) % n];
        let (ax, ay) = (a.x - p.x, a.y - p.y);
        let (bx, by) = (b.x - p.x, b.y - p.y);
        let (la, lb) = (ax.hypot(ay), bx.hypot(by));
        let (ua, ub) = ((ax / la, ay / la), (bx / lb, by / lb));
        let half = (ua.0 * ub.0 + ua.1 * ub.1).clamp(-1., 1.).acos() / 2.;
        if p.rounding <= 0. || half.tan().abs() < 1e-4 || half > 1.5 {
            outline.push((p.x, p.y));
            continue;
        }
        let mut cut = p.rounding / half.tan();
        let room = la.min(lb) / 2.;
        let radius = if cut > room {
            cut = room;
            room * half.tan()
        } else {
            p.rounding
        };
        let start = (p.x + ua.0 * cut, p.y + ua.1 * cut);
        let end = (p.x + ub.0 * cut, p.y + ub.1 * cut);
        let (bisect_x, bisect_y) = (ua.0 + ub.0, ua.1 + ub.1);
        let bisect = bisect_x.hypot(bisect_y);
        let distance = radius / half.sin();
        let center = (
            p.x + bisect_x / bisect * distance,
            p.y + bisect_y / bisect * distance,
        );
        let from = (start.1 - center.1).atan2(start.0 - center.0);
        let to = (end.1 - center.1).atan2(end.0 - center.0);
        let mut sweep = to - from;
        if sweep > std::f32::consts::PI {
            sweep -= TAU;
        } else if sweep < -std::f32::consts::PI {
            sweep += TAU;
        }
        for step in 0..=ARC_STEPS {
            let angle = from + sweep * step as f32 / ARC_STEPS as f32;
            outline.push((
                center.0 + radius * angle.cos(),
                center.1 + radius * angle.sin(),
            ));
        }
    }
    outline
}

/// The outline's distance from the center along each sample angle,
/// normalized so the farthest point is 1.
fn polar(outline: &[(f32, f32)]) -> Radii {
    let mut radii = [0.; SAMPLES];
    for (i, radius) in radii.iter_mut().enumerate() {
        let angle = i as f32 * TAU / SAMPLES as f32;
        let (dx, dy) = (angle.cos(), angle.sin());
        for (j, &(px, py)) in outline.iter().enumerate() {
            let (qx, qy) = outline[(j + 1) % outline.len()];
            let (ex, ey) = (qx - px, qy - py);
            // Solve t·d = p + s·e for the ray parameter t and edge parameter s.
            let det = ex * dy - ey * dx;
            if det.abs() < 1e-9 {
                continue;
            }
            let t = (ex * py - ey * px) / det;
            let s = (dx * py - dy * px) / det;
            if (-1e-5..=1. + 1e-5).contains(&s) && t > *radius {
                *radius = t;
            }
        }
    }
    let max = radii.iter().copied().fold(0., f32::max).max(1e-6);
    radii.map(|radius| radius / max)
}

fn radii(shape: LoadingShape) -> &'static Radii {
    static TABLE: OnceLock<[Radii; 8]> = OnceLock::new();
    &TABLE.get_or_init(|| {
        [
            LoadingShape::Circle,
            LoadingShape::SoftBurst,
            LoadingShape::Cookie9Sided,
            LoadingShape::Pentagon,
            LoadingShape::Pill,
            LoadingShape::Sunny,
            LoadingShape::Cookie4Sided,
            LoadingShape::Oval,
        ]
        .map(|shape| polar(&rounded_outline(&vertices(shape))))
    })[shape.index()]
}

/// The frame's outline: `radius` px from `center` at the farthest point.
fn frame_path(frame: LoadingFrame, center: Point<Pixels>, radius: f32) -> Option<Path<Pixels>> {
    let (from, to) = (radii(frame.from), radii(frame.to));
    let turn = frame.rotation_degrees.to_radians();
    let mut path = PathBuilder::fill();
    for i in 0..SAMPLES {
        let r = (from[i] + (to[i] - from[i]) * frame.morph).max(0.05) * radius;
        let angle = turn + i as f32 * TAU / SAMPLES as f32;
        let at = point(
            center.x + px(r * angle.cos()),
            center.y + px(r * angle.sin()),
        );
        if i == 0 {
            path.move_to(at);
        } else {
            path.line_to(at);
        }
    }
    path.close();
    path.build().ok()
}

/// M3 Expressive loading indicator.
///
/// Indeterminate by default: it morphs through [`LoadingShape::SEQUENCE`]
/// on the reference's spring while rotating. [`Self::progress`] makes it
/// determinate (a circle growing into a soft burst). Reduced motion holds
/// the first frame. The caller supplies the accessible label.
#[derive(gpui::IntoElement)]
pub struct LoadingIndicator {
    id: ElementId,
    label: SharedString,
    progress: Option<f32>,
    contained: bool,
    shape: Option<LoadingShape>,
    playback: Option<RepeatAnimationPlayback>,
}

impl LoadingIndicator {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            progress: None,
            contained: false,
            shape: None,
            playback: None,
        }
    }

    /// Show the round primary-container container behind the shape.
    pub fn contained(mut self, contained: bool) -> Self {
        self.contained = contained;
        self
    }

    /// A measured fraction (0-1), or `None` for indeterminate.
    pub fn progress(mut self, progress: Option<f32>) -> Self {
        self.progress = progress;
        self
    }

    /// A specimen: hold one shape, without animating.
    pub fn shape(mut self, shape: LoadingShape) -> Self {
        self.shape = Some(shape);
        self
    }

    /// A caller-owned clock that survives virtual unmounting.
    pub fn playback(mut self, playback: RepeatAnimationPlayback) -> Self {
        self.playback = Some(playback);
        self
    }
}

fn shape_canvas(frame: LoadingFrame, ink: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let scale =
                f32::from(bounds.size.width.min(bounds.size.height)) / LOADING_INDICATOR_SIZE;
            if let Some(path) = frame_path(frame, bounds.center(), SHAPE_RADIUS * scale) {
                window.paint_path(path, ink);
            }
        },
    )
    .size_full()
}

impl RenderOnce for LoadingIndicator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let ink = color(if self.contained {
            ColorRole::ON_PRIMARY_CONTAINER
        } else {
            ColorRole::PRIMARY
        });
        let measured = self
            .progress
            .map(|p| if p.is_finite() { p.clamp(0., 1.) } else { 0. });
        let root = Progress::new(self.id.clone())
            .accessibility_label(self.label)
            .indeterminate(measured.is_none())
            .value(measured.unwrap_or(0.) * 100.)
            .flex_shrink_0()
            .size(rem(LOADING_INDICATOR_SIZE))
            .rounded_full()
            .when(self.contained, |root| {
                root.bg(color(ColorRole::PRIMARY_CONTAINER))
            });
        if let Some(shape) = self.shape {
            return root.child(shape_canvas(LoadingFrame::still(shape), ink));
        }
        if let Some(target) = measured {
            let shown = crate::motion::value((self.id, "loading-progress"), target, window, cx)
                .clamp(0., 1.);
            return root.child(shape_canvas(LoadingFrame::determinate(shown), ink));
        }
        let playback = self.playback;
        root.child(
            div()
                .size_full()
                .with_animation(
                    self.id,
                    Animation::new(Duration::from_secs_f32(LOOP_MS / 1000.)).repeat(),
                    move |slot, t| {
                        slot.child(shape_canvas(loading_indicator_frame(t * LOOP_MS), ink))
                    },
                )
                .when_some(playback, |animation, playback| {
                    animation.with_repeat_playback(playback)
                })
                .repeat_when_visible(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_loop_morphs_through_seven_shapes_and_closes_seamlessly() {
        let start = loading_indicator_frame(0.);
        assert_eq!(start.from, LoadingShape::SoftBurst);
        assert_eq!(start.to, LoadingShape::Cookie9Sided);
        assert_eq!(start.morph, 0.);
        let shapes: Vec<_> = (0..7)
            .map(|i| loading_indicator_frame(i as f32 * MORPH_INTERVAL_MS + 1.).from)
            .collect();
        assert_eq!(shapes, LoadingShape::SEQUENCE);
        // Each morph settles on its target before the next starts.
        let settled = loading_indicator_frame(MORPH_INTERVAL_MS - 0.01);
        assert!((settled.morph - 1.).abs() < 0.01, "{}", settled.morph);
        // The end of the loop meets its start, shape and rotation alike.
        let end = loading_indicator_frame(LOOP_MS - 0.01);
        assert_eq!(end.to, LoadingShape::SoftBurst);
        let gap = (end.rotation_degrees - 360.)
            .abs()
            .min(end.rotation_degrees);
        assert!(gap < 0.1, "rotation jumps by {gap}°");
    }

    #[test]
    fn the_spring_overshoots_and_each_morph_adds_a_quarter_turn() {
        let peak = (0..65)
            .map(|i| loading_indicator_frame(i as f32 * 10.).morph)
            .fold(0., f32::max);
        assert!(peak > 1.05 && peak < 1.2, "{peak}");
        let a = loading_indicator_frame(MORPH_INTERVAL_MS - 0.01).rotation_degrees;
        let b = loading_indicator_frame(0.).rotation_degrees;
        let constant = 360. * LOOP_TURNS * MORPH_INTERVAL_MS / LOOP_MS;
        assert!((a - b - 90. - constant).abs() < 0.5, "{a} {b}");
    }

    #[test]
    fn shapes_are_normalized_star_shaped_and_distinct() {
        let circle = radii(LoadingShape::Circle);
        assert!(circle.iter().all(|r| (r - 1.).abs() < 0.01));
        for shape in LoadingShape::SEQUENCE {
            let r = radii(shape);
            let max = r.iter().copied().fold(0., f32::max);
            let min = r.iter().copied().fold(1., f32::min);
            assert!((max - 1.).abs() < 1e-4, "{shape:?}");
            assert!(min > 0.3 && min < 0.97, "{shape:?}: {min}");
        }
        // The cookie's four lobes repeat every quarter turn; the pill does not.
        let cookie = radii(LoadingShape::Cookie4Sided);
        let pill = radii(LoadingShape::Pill);
        let quarter = SAMPLES / 4;
        assert!((0..SAMPLES).all(|i| (cookie[i] - cookie[(i + quarter) % SAMPLES]).abs() < 0.02));
        assert!((0..SAMPLES).any(|i| (pill[i] - pill[(i + quarter) % SAMPLES]).abs() > 0.2));
    }

    #[test]
    fn determinate_grows_a_circle_into_the_soft_burst() {
        assert_eq!(LoadingFrame::determinate(0.).from, LoadingShape::Circle);
        assert_eq!(LoadingFrame::determinate(f32::NAN).morph, 0.);
        let full = LoadingFrame::determinate(2.);
        assert_eq!((full.to, full.morph), (LoadingShape::SoftBurst, 1.));
        assert_eq!(full.rotation_degrees, 180.);
        assert!(frame_path(full, point(px(24.), px(24.)), 17.5).is_some());
    }

    struct Harness;
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            let mut node = accesskit::Node::new(accesskit::Role::GenericContainer);
            RenderOnce::render(
                Progress::new("probe")
                    .accessibility_label("Loading")
                    .indeterminate(true),
                window,
                cx,
            )
            .into_element()
            .write_a11y_info(&mut node);
            assert_eq!(node.label(), Some("Loading"));
            div().flex().items_start().child(
                div()
                    .debug_selector(|| "loading".into())
                    .child(LoadingIndicator::new("li", "Loading").contained(true)),
            )
        }
    }

    #[gpui::test]
    fn the_indicator_is_a_48dp_progress_node(cx: &mut gpui::TestAppContext) {
        let (_, cx) = cx.add_window_view(|_, _| Harness);
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let bounds = cx.debug_bounds("loading").unwrap();
        assert_eq!(bounds.size, size(px(48.), px(48.)));
    }
}
