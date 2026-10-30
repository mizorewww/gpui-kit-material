//! M3 Expressive loading indicator: a shape that morphs through Compose's
//! seven Material shapes while it rotates, optionally inside a round
//! container. Reference: <https://m3e.kyralabs.dev/components/loading-indicator>
//! and the Figma kit's "Loading indicator" set (48dp container, 38dp
//! active indicator, primary or on-primary-container on primary-container).
//!
//! The shapes and their morphs come from [`crate::shapes`], with Compose's
//! `LoadingIndicator.kt` scale factor and centering; each frame is painted as
//! one GPUI path.
use crate::{
    color::ColorRole,
    motion::Spring,
    shapes::{Morph, PathCache, RoundedPolygon, ShapeKind, morph_path, polygon_path},
    theme::color,
    tokens::typography::rem,
};
use gpui::{prelude::*, *};
use gpui_base::Progress;
use std::{sync::OnceLock, time::Duration};

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

    /// This shape in the Material shape library.
    pub const fn kind(self) -> ShapeKind {
        match self {
            Self::Circle => ShapeKind::Circle,
            Self::SoftBurst => ShapeKind::SoftBurst,
            Self::Cookie9Sided => ShapeKind::Cookie9Sided,
            Self::Pentagon => ShapeKind::Pentagon,
            Self::Pill => ShapeKind::Pill,
            Self::Sunny => ShapeKind::Sunny,
            Self::Cookie4Sided => ShapeKind::Cookie4Sided,
            Self::Oval => ShapeKind::Oval,
        }
    }
}

/// The container (48dp), from the Figma kit and Compose's
/// `LoadingIndicatorTokens`.
pub const LOADING_INDICATOR_SIZE: f32 = 48.;
/// The 38dp active-indicator box inside the container (Figma kit; Compose's
/// `ActiveIndicatorScale`).
const ACTIVE_INDICATOR_SCALE: f32 = 38. / LOADING_INDICATOR_SIZE;

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

/// The polygons' scale in the active-indicator box (Compose's
/// `calculateScaleFactor`): a shape fills its box less than the circle it
/// sweeps while rotating, so all of them shrink to keep every one inside.
fn scale_factor<'a>(polygons: impl IntoIterator<Item = &'a RoundedPolygon>) -> f32 {
    let mut factor = 1_f32;
    for polygon in polygons {
        let bounds = polygon.bounds(true);
        let max = polygon.max_bounds();
        let scale_x = (bounds[2] - bounds[0]) / (max[2] - max[0]);
        let scale_y = (bounds[3] - bounds[1]) / (max[3] - max[1]);
        // The larger, so a pill does not throw off the whole set.
        factor = factor.min(scale_x.max(scale_y));
    }
    factor
}

/// The indeterminate set's scale.
fn indeterminate_factor() -> f32 {
    static FACTOR: OnceLock<f32> = OnceLock::new();
    *FACTOR.get_or_init(|| scale_factor(LoadingShape::SEQUENCE.map(|s| s.kind().polygon())))
}

/// The determinate morph and its scale. Compose turns the circle by 360°/20
/// for a smoother morph into the soft burst.
fn determinate() -> &'static (Morph, f32) {
    static DETERMINATE: OnceLock<(Morph, f32)> = OnceLock::new();
    DETERMINATE.get_or_init(|| {
        let circle = ShapeKind::Circle.polygon().rotated(360. / 20.).normalized();
        let burst = ShapeKind::SoftBurst.polygon();
        let factor = scale_factor([&circle, burst]);
        (Morph::new(&circle, burst), factor)
    })
}

/// The frame's outline, in a container of `container` px around `center`.
fn frame_path(frame: LoadingFrame, center: Point<Pixels>, container: f32) -> Option<Path<Pixels>> {
    let side = container * ACTIVE_INDICATOR_SCALE;
    let turn = frame.rotation_degrees;
    let (from, to) = (frame.from, frame.to);
    if from == to {
        let factor = if from == LoadingShape::Circle {
            determinate().1
        } else {
            indeterminate_factor()
        };
        polygon_path(from.kind().polygon(), center, side * factor, turn)
    } else if (from, to) == (LoadingShape::Circle, LoadingShape::SoftBurst) {
        let (morph, factor) = determinate();
        morph_path(morph, frame.morph, center, side * factor, turn)
    } else {
        let morph = from.kind().morph_to(to.kind());
        morph_path(
            morph,
            frame.morph,
            center,
            side * indeterminate_factor(),
            turn,
        )
    }
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
    size: f32,
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
            size: LOADING_INDICATOR_SIZE,
            shape: None,
            playback: None,
        }
    }

    /// Show the round primary-container container behind the shape.
    pub fn contained(mut self, contained: bool) -> Self {
        self.contained = contained;
        self
    }

    /// The container's size in design px (48 by default); the shape scales
    /// with it.
    pub fn size(mut self, size: f32) -> Self {
        if size.is_finite() && size > 0. {
            self.size = size;
        }
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

    /// A caller-owned clock that survives virtual unmounting. This is the
    /// indicator's one use of the vendored engine's repeat-playback patch,
    /// kept for callers that pass it: without it the indicator runs on GPUI's
    /// stock repeating animation, restarting if it unmounts.
    pub fn playback(mut self, playback: RepeatAnimationPlayback) -> Self {
        self.playback = Some(playback);
        self
    }
}

/// A frame that holds still until its input changes: the path is reused while
/// frame, size and origin are unchanged. The indicator owns the cache through
/// keyed state.
fn held_canvas(
    frame: LoadingFrame,
    ink: Rgba,
    id: &ElementId,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    let paths = window.use_keyed_state((id.clone(), "loading-path"), cx, |_, _| {
        PathCache::<(LoadingFrame, f32, Point<Pixels>)>::default()
    });
    canvas(
        |_, _, _| {},
        move |bounds, _, window, cx| {
            let container = f32::from(bounds.size.width.min(bounds.size.height));
            let center = bounds.center();
            let path = paths.update(cx, |paths, _| {
                paths.path((frame, container, center), || {
                    frame_path(frame, center, container)
                })
            });
            if let Some(path) = path {
                window.paint_path(path, ink);
            }
        },
    )
    .size_full()
}

fn shape_canvas(frame: LoadingFrame, ink: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let container = f32::from(bounds.size.width.min(bounds.size.height));
            if let Some(path) = frame_path(frame, bounds.center(), container) {
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
            .size(rem(self.size))
            .rounded_full()
            .when(self.contained, |root| {
                root.bg(color(ColorRole::PRIMARY_CONTAINER))
            });
        if let Some(shape) = self.shape {
            let canvas = held_canvas(LoadingFrame::still(shape), ink, &self.id, window, cx);
            return root.child(canvas);
        }
        if let Some(target) = measured {
            let shown =
                crate::motion::value((self.id.clone(), "loading-progress"), target, window, cx)
                    .clamp(0., 1.);
            let frame = LoadingFrame::determinate(shown);
            let canvas = held_canvas(frame, ink, &self.id, window, cx);
            return root.child(canvas);
        }
        let animation = Animation::new(Duration::from_secs_f32(LOOP_MS / 1000.)).repeat();
        let paint =
            move |slot: Div, t| slot.child(shape_canvas(loading_indicator_frame(t * LOOP_MS), ink));
        let slot = div().size_full();
        match self.playback {
            // GPUI's stock repeating animation: its clock, and holding the
            // first frame under reduced motion, are the element's own.
            None => root.child(slot.with_animation(self.id, animation, paint)),
            Some(playback) => root.child(
                slot.with_animation(self.id, animation, paint)
                    .with_repeat_playback(playback)
                    .repeat_when_visible(),
            ),
        }
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

    const SAMPLES: usize = 120;

    /// The outline's distance from the center along equal angles, normalized
    /// so the farthest point is 1.
    fn radii(shape: LoadingShape) -> [f32; SAMPLES] {
        // Compose's normalization centers the control-point box, so a shape's
        // own center can sit slightly off the box's (0.504 for the circle).
        let (cx, cy) = shape.kind().polygon().center();
        let mut outline = Vec::new();
        for cubic in shape.kind().polygon().cubics() {
            outline.extend((0..24).map(|i| cubic.point_at(i as f32 / 24.)));
        }
        let mut radii = [0_f32; SAMPLES];
        for (i, radius) in radii.iter_mut().enumerate() {
            let angle = i as f32 * std::f32::consts::TAU / SAMPLES as f32;
            let (dx, dy) = (angle.cos(), angle.sin());
            for (j, &(px, py)) in outline.iter().enumerate() {
                let (qx, qy) = outline[(j + 1) % outline.len()];
                let (px, py, ex, ey) = (px - cx, py - cy, qx - px, qy - py);
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
        assert!((0..SAMPLES).any(|i| (pill[i] - pill[(i + quarter) % SAMPLES]).abs() > 0.1));
    }

    #[test]
    fn determinate_grows_a_circle_into_the_soft_burst() {
        assert_eq!(LoadingFrame::determinate(0.).from, LoadingShape::Circle);
        assert_eq!(LoadingFrame::determinate(f32::NAN).morph, 0.);
        let full = LoadingFrame::determinate(2.);
        assert_eq!((full.to, full.morph), (LoadingShape::SoftBurst, 1.));
        assert_eq!(full.rotation_degrees, 180.);
        assert!(frame_path(full, point(px(24.), px(24.)), 48.).is_some());
    }

    #[test]
    fn every_morph_of_the_sequence_stays_in_the_active_box_while_it_turns() {
        // Compose's scale keeps the turning shape inside the 38dp box, up to
        // the outline's corners that the longer of its two axes lets out.
        let half = 48. * ACTIVE_INDICATOR_SCALE / 2. * indeterminate_factor();
        let sequence = LoadingShape::SEQUENCE;
        for (i, from) in sequence.into_iter().enumerate() {
            let to = sequence[(i + 1) % sequence.len()];
            let morph = from.kind().morph_to(to.kind());
            for cubic in morph.as_cubics(0.5) {
                let (x, y) = cubic.anchor0();
                let reach = (x - 0.5).hypot(y - 0.5) * 2. * half;
                assert!(reach < 48. / 2., "{from:?} -> {to:?}: {reach}");
            }
        }
        // The soft burst, the indicator's face, is the size the kit draws
        // (33-34dp across at rest, Figma "Loading indicator").
        let burst = ShapeKind::SoftBurst.polygon().bounds(false);
        let across = (burst[2] - burst[0]) * half * 2.;
        assert!((30. ..=38.).contains(&across), "{across}");
    }

    #[test]
    fn the_determinate_morph_is_the_rotated_circle_into_the_soft_burst() {
        let (morph, factor) = determinate();
        assert!((0.5..=1.).contains(factor), "{factor}");
        let start = morph.as_cubics(0.);
        // At 0 it is a circle: every anchor is the same distance out.
        for cubic in &start {
            let (x, y) = cubic.anchor0();
            assert!(((x - 0.5).hypot(y - 0.5) - 0.5).abs() < 0.02, "{x} {y}");
        }
    }

    #[test]
    fn shapes_map_onto_the_library() {
        assert_eq!(LoadingShape::Pill.kind(), ShapeKind::Pill);
        for shape in LoadingShape::SEQUENCE {
            assert_ne!(shape.kind(), ShapeKind::Circle);
        }
    }

    struct Harness;
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div()
                .flex()
                .items_start()
                .child(
                    div()
                        .debug_selector(|| "loading".into())
                        .child(LoadingIndicator::new("li", "Loading").contained(true)),
                )
                .child(LoadingIndicator::new("li-measured", "Uploading").progress(Some(0.4)))
                .child(
                    div()
                        .debug_selector(|| "loading-large".into())
                        .child(LoadingIndicator::new("li-large", "Loading").size(96.)),
                )
        }
    }

    #[gpui::test]
    fn the_indicator_is_a_48dp_progress_node(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let (_, cx) = cx.add_window_view(|_, _| Harness);
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let bounds = cx.debug_bounds("loading").unwrap();
        assert_eq!(bounds.size, size(px(48.), px(48.)));
        let large = cx.debug_bounds("loading-large").unwrap();
        assert_eq!(large.size, size(px(96.), px(96.)), "a sized container");
        // The rendered LoadingIndicator itself carries the progress role and
        // the caller's label, indeterminate or measured.
        cx.update(|window, _| {
            for (id, label) in [("li", "Loading"), ("li-measured", "Uploading")] {
                let node = gpui_base::test_support::find(window, &[], &ElementId::from(id))
                    .unwrap_or_else(|| panic!("{id} was not rendered"));
                assert_eq!(node.role(), Some(Role::ProgressIndicator), "{id}");
                assert_eq!(node.label(), Some(label), "{id}");
                assert_eq!(node.bounds().size, size(px(48.), px(48.)), "{id}");
            }
        });
    }
}
