//! Painting shapes: GPUI paths from outlines, and the [`MaterialShape`]
//! element.
//!
//! Outlines go to GPUI as cubic curves, not polylines: they are what the shape
//! is, the tessellator flattens them to its tolerance at any size, and an
//! outline of 20-100 curves needs far fewer path events than a polyline as
//! smooth would.
use super::{
    catalog::ShapeKind,
    morph::Morph,
    polygon::{Cubic, RoundedPolygon},
};
use crate::{
    color::ColorRole,
    motion::{self, MotionToken},
    theme::color,
    tokens::typography::rem,
};
use gpui::{prelude::*, *};

/// A shape's default size in design px: the kit's 48dp target.
pub const DEFAULT_SIZE: f32 = 48.;

/// A built path, reused while its key (what it draws, its size and origin) is
/// unchanged. The element that holds it, through keyed state, owns it, and a
/// changed key is the only invalidation. A key is kept only once seen twice in
/// a row, so a running morph, whose key changes every frame, stores nothing.
pub(crate) struct PathCache<K> {
    key: Option<K>,
    path: Option<Path<Pixels>>,
}

impl<K> Default for PathCache<K> {
    fn default() -> Self {
        Self {
            key: None,
            path: None,
        }
    }
}

impl<K: PartialEq> PathCache<K> {
    /// The path for `key`, from `build` unless it is already held.
    pub(crate) fn path(
        &mut self,
        key: K,
        build: impl FnOnce() -> Option<Path<Pixels>>,
    ) -> Option<Path<Pixels>> {
        if self.key.as_ref() != Some(&key) {
            self.key = Some(key);
            self.path = None;
            return build();
        }
        if self.path.is_none() {
            self.path = build();
        }
        self.path.clone()
    }
}

/// Build the path, with the outline's control-point box centered on `center`
/// (Compose's `processPath`), `side` px per outline unit, turned clockwise.
fn outline_path(
    each: impl Fn(&mut dyn FnMut(&Cubic)),
    center: Point<Pixels>,
    side: f32,
    rotation_degrees: f32,
) -> Option<Path<Pixels>> {
    let mut bounds = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    each(&mut |cubic| {
        for [x, y] in cubic.points.as_chunks::<2>().0 {
            bounds[0] = bounds[0].min(*x);
            bounds[1] = bounds[1].min(*y);
            bounds[2] = bounds[2].max(*x);
            bounds[3] = bounds[3].max(*y);
        }
    });
    let (middle_x, middle_y) = ((bounds[0] + bounds[2]) / 2., (bounds[1] + bounds[3]) / 2.);
    let (sin, cos) = rotation_degrees.to_radians().sin_cos();
    let at = |x: f32, y: f32| {
        let (x, y) = ((x - middle_x) * side, (y - middle_y) * side);
        point(
            center.x + px(x * cos - y * sin),
            center.y + px(x * sin + y * cos),
        )
    };
    let mut path = PathBuilder::fill();
    let mut first = true;
    each(&mut |cubic| {
        let [a0x, a0y, c0x, c0y, c1x, c1y, a1x, a1y] = cubic.points;
        if first {
            path.move_to(at(a0x, a0y));
            first = false;
        }
        path.cubic_bezier_to(at(a1x, a1y), at(c0x, c0y), at(c1x, c1y));
    });
    if first {
        return None;
    }
    path.close();
    path.build().ok()
}

/// The path of `polygon`: see [`morph_path`].
pub fn polygon_path(
    polygon: &RoundedPolygon,
    center: Point<Pixels>,
    side: f32,
    rotation_degrees: f32,
) -> Option<Path<Pixels>> {
    outline_path(
        |visit| polygon.cubics().iter().for_each(&mut *visit),
        center,
        side,
        rotation_degrees,
    )
}

/// The path of `morph` at `progress`, centered on `center`, `side` px per
/// outline unit (a Material shape's full size, as they are 0-1), and turned
/// clockwise about its center.
pub fn morph_path(
    morph: &Morph,
    progress: f32,
    center: Point<Pixels>,
    side: f32,
    rotation_degrees: f32,
) -> Option<Path<Pixels>> {
    outline_path(
        |visit| morph.for_each_cubic(progress, &mut *visit),
        center,
        side,
        rotation_degrees,
    )
}

/// What a [`MaterialShape`] draws: where it is between two shapes, and its
/// turn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeFrame {
    /// The shape at 0.
    pub from: ShapeKind,
    /// The shape at 1.
    pub to: ShapeKind,
    /// 0 at `from`, 1 at `to`. Values beyond extrapolate, for overshoot.
    pub progress: f32,
    /// Clockwise turn about the shape's center.
    pub rotation_degrees: f32,
}

impl ShapeFrame {
    /// A shape at rest, unrotated.
    pub const fn still(shape: ShapeKind) -> Self {
        Self::morph(shape, shape, 1.)
    }

    /// `progress` of the way from `from` to `to`, unrotated. A non-finite
    /// progress rests at `from`.
    pub const fn morph(from: ShapeKind, to: ShapeKind, progress: f32) -> Self {
        Self {
            from,
            to,
            progress: if progress.is_finite() { progress } else { 0. },
            rotation_degrees: 0.,
        }
    }

    /// The frame's path in a square of `side` px around `center`.
    pub fn path(self, center: Point<Pixels>, side: f32) -> Option<Path<Pixels>> {
        if self.from == self.to {
            polygon_path(self.to.polygon(), center, side, self.rotation_degrees)
        } else {
            morph_path(
                self.from.morph_to(self.to),
                self.progress,
                center,
                side,
                self.rotation_degrees,
            )
        }
    }
}

/// How many shape changes [`MaterialShape::animated`] remembers. A morph
/// still running after this many newer changes jumps to the oldest kept.
const HISTORY: usize = 8;

/// The shapes an animated element has been given, newest at `generation`.
struct ShapeHistory {
    kinds: [ShapeKind; HISTORY],
    generation: usize,
}

impl ShapeHistory {
    fn new(kind: ShapeKind) -> Self {
        Self {
            kinds: [kind; HISTORY],
            generation: 0,
        }
    }

    fn current(&self) -> ShapeKind {
        self.kinds[self.generation % HISTORY]
    }

    /// Record `kind`, returning the generation to animate to.
    fn advance(&mut self, kind: ShapeKind) -> usize {
        if self.current() != kind {
            self.generation += 1;
            self.kinds[self.generation % HISTORY] = kind;
        }
        self.generation
    }

    /// The frame at presentation position `shown`, in generations: a retarget
    /// carries on through the shape it was heading for, so nothing jumps.
    fn frame(&self, shown: f32) -> ShapeFrame {
        if self.generation == 0 {
            return ShapeFrame::still(self.current());
        }
        let oldest = self.generation.saturating_sub(HISTORY - 1);
        let last = (shown.ceil().max(0.) as usize).clamp(oldest + 1, self.generation);
        let progress = shown - (last - 1) as f32;
        ShapeFrame::morph(
            self.kinds[(last - 1) % HISTORY],
            self.kinds[last % HISTORY],
            // Older than the history: rest on the oldest kept shape.
            if last == oldest + 1 {
                progress.max(0.)
            } else {
                progress
            },
        )
    }
}

/// The frame an animated shape draws this render, recording `kind`.
fn animated_frame(
    id: &ElementId,
    kind: ShapeKind,
    token: MotionToken,
    window: &mut Window,
    cx: &mut App,
) -> ShapeFrame {
    let history = window.use_keyed_state((id.clone(), "material-shape-history"), cx, |_, _| {
        ShapeHistory::new(kind)
    });
    let generation = history.update(cx, |history, _| history.advance(kind));
    let shown = motion::token_value(
        (id.clone(), "material-shape-morph"),
        generation as f32,
        token,
        window,
        cx,
    );
    history.read(cx).frame(shown)
}

enum Source {
    Frame(ShapeFrame),
    Animated(ShapeKind),
}

/// A Material shape, or a morph between two, filled with a color role.
///
/// [`Self::new`] draws one shape; with [`Self::animated`] it springs to each
/// new shape it is given, retargeting from where it is, and reduced motion
/// shows the new shape at once. [`Self::frame`] draws what the caller's own
/// clock says. Without [`Self::label`] the shape is decorative and is not in
/// the accessibility tree.
#[derive(gpui::IntoElement)]
pub struct MaterialShape {
    id: ElementId,
    source: Source,
    role: ColorRole,
    size: f32,
    rotation_degrees: f32,
    label: Option<SharedString>,
    token: MotionToken,
}

impl MaterialShape {
    /// A `kind` shape: primary, 48 design px, unrotated, decorative.
    pub fn new(id: impl Into<ElementId>, kind: ShapeKind) -> Self {
        Self {
            id: id.into(),
            source: Source::Frame(ShapeFrame::still(kind)),
            role: ColorRole::PRIMARY,
            size: DEFAULT_SIZE,
            rotation_degrees: 0.,
            label: None,
            token: MotionToken::DefaultSpatial,
        }
    }

    /// Draw exactly `frame`, for a caller's own clock.
    pub fn frame(mut self, frame: ShapeFrame) -> Self {
        self.rotation_degrees = frame.rotation_degrees;
        self.source = Source::Frame(frame);
        self
    }

    /// Whether a new shape morphs from the last one on a spring, rather than
    /// replacing it. The shape is the one given to [`Self::new`].
    pub fn animated(mut self, animated: bool) -> Self {
        let kind = match self.source {
            Source::Frame(frame) => frame.to,
            Source::Animated(kind) => kind,
        };
        self.source = if animated {
            Source::Animated(kind)
        } else {
            Source::Frame(ShapeFrame::still(kind))
        };
        self
    }

    /// The motion token [`Self::animated`] follows. Default spatial by
    /// default.
    pub fn motion(mut self, token: MotionToken) -> Self {
        self.token = token;
        self
    }

    /// The fill.
    pub fn color(mut self, role: ColorRole) -> Self {
        self.role = role;
        self
    }

    /// The square's side in design px, following the text scale. Ignored
    /// unless finite and positive.
    pub fn size(mut self, size: f32) -> Self {
        if size.is_finite() && size > 0. {
            self.size = size;
        }
        self
    }

    /// A clockwise turn about the center. A non-finite turn is ignored.
    pub fn rotation(mut self, degrees: f32) -> Self {
        if degrees.is_finite() {
            self.rotation_degrees = degrees;
        }
        self
    }

    /// The accessible name, exposing the shape as an image.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for MaterialShape {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut frame = match self.source {
            Source::Frame(frame) => frame,
            Source::Animated(kind) => animated_frame(&self.id, kind, self.token, window, cx),
        };
        frame.rotation_degrees = self.rotation_degrees;
        let ink = color(self.role);
        let paths = window.use_keyed_state((self.id.clone(), "material-shape-path"), cx, |_, _| {
            PathCache::<(ShapeFrame, f32, Point<Pixels>)>::default()
        });
        div()
            .id(self.id)
            .flex_shrink_0()
            .size(rem(self.size))
            .when_some(self.label, |node, label| {
                node.role(Role::Image).aria_label(label)
            })
            .child(
                canvas(
                    |_, _, _| {},
                    move |bounds, _, window, cx| {
                        let side = f32::from(bounds.size.width.min(bounds.size.height));
                        let center = bounds.center();
                        let path = paths.update(cx, |paths, _| {
                            paths.path((frame, side, center), || frame.path(center, side))
                        });
                        if let Some(path) = path {
                            window.paint_path(path, ink);
                        }
                    },
                )
                .size_full(),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_history_walks_through_each_shape_in_turn() {
        let mut history = ShapeHistory::new(ShapeKind::Circle);
        assert_eq!(history.frame(0.), ShapeFrame::still(ShapeKind::Circle));
        assert_eq!(
            history.advance(ShapeKind::Circle),
            0,
            "same shape: no morph"
        );
        assert_eq!(history.advance(ShapeKind::Heart), 1);
        assert_eq!(history.advance(ShapeKind::Bun), 2);
        // Interrupted a third of the way into the first morph.
        let early = history.frame(0.33);
        assert_eq!(
            (early.from, early.to),
            (ShapeKind::Circle, ShapeKind::Heart)
        );
        assert!((early.progress - 0.33).abs() < 1e-6);
        // Past it, the second morph; the two meet with the same shape.
        let late = history.frame(1.25);
        assert_eq!((late.from, late.to), (ShapeKind::Heart, ShapeKind::Bun));
        assert!((late.progress - 0.25).abs() < 1e-6);
        // A spring's overshoot extrapolates the last morph.
        let over = history.frame(2.1);
        assert_eq!((over.from, over.to), (ShapeKind::Heart, ShapeKind::Bun));
        assert!((over.progress - 1.1).abs() < 1e-5);
        assert_eq!(history.frame(2.).progress, 1.);
    }

    #[test]
    fn long_bursts_of_changes_stay_inside_the_history() {
        let mut history = ShapeHistory::new(ShapeKind::Circle);
        for i in 0..100 {
            history.advance(ShapeKind::ALL[1 + i % 34]);
        }
        // A presentation far behind every kept shape rests on the oldest.
        let stale = history.frame(0.);
        assert_eq!(stale.progress, 0.);
        assert_eq!(
            stale.from,
            history.kinds[(history.generation - 7) % HISTORY]
        );
        // NaN from a broken clock lands on a kept morph, not out of bounds.
        assert!(history.frame(f32::NAN).progress.is_finite());
    }

    #[test]
    fn frames_build_paths_and_refuse_bad_progress() {
        let center = point(px(24.), px(24.));
        for kind in ShapeKind::ALL {
            assert!(
                ShapeFrame::still(kind).path(center, 48.).is_some(),
                "{kind:?}"
            );
        }
        let frame = ShapeFrame::morph(ShapeKind::Circle, ShapeKind::Boom, f32::NAN);
        assert_eq!(frame.progress, 0.);
        assert!(frame.path(center, 48.).is_some());
        // A zero-sized box is reachable mid-layout; it must simply not panic.
        let _ = frame.path(center, 0.);
    }

    #[test]
    fn the_path_cache_keeps_only_a_key_seen_twice_in_a_row() {
        let mut cache = PathCache::<u32>::default();
        let builds = std::cell::Cell::new(0);
        let build = || {
            builds.set(builds.get() + 1);
            ShapeFrame::still(ShapeKind::Bun).path(point(px(24.), px(24.)), 48.)
        };
        // A running morph: every key new, nothing stored, built each time.
        assert!(cache.path(1, build).is_some());
        assert!(cache.path(2, build).is_some());
        assert_eq!(builds.get(), 2);
        assert!(cache.path.is_none());
        // At rest: built once more, then reused.
        assert!(cache.path(2, build).is_some());
        assert!(cache.path(2, build).is_some());
        assert!(cache.path(2, build).is_some());
        assert_eq!(builds.get(), 3);
        // A changed key invalidates it.
        assert!(cache.path(3, build).is_some());
        assert!(cache.path.is_none());
        assert_eq!(builds.get(), 4);
    }

    struct Harness {
        kind: ShapeKind,
        drawn: Option<ShapeFrame>,
    }

    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            self.drawn = Some(animated_frame(
                &ElementId::from("probe"),
                self.kind,
                MotionToken::DefaultSpatial,
                window,
                cx,
            ));
            div()
                .flex()
                .items_start()
                .child(
                    div().debug_selector(|| "shape".into()).child(
                        MaterialShape::new("shape", self.kind)
                            .label("Favorite")
                            .size(96.)
                            .animated(true),
                    ),
                )
                .child(MaterialShape::new("decorative", ShapeKind::Bun))
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }

    #[gpui::test]
    fn a_shape_fills_its_rem_sized_square(cx: &mut gpui::TestAppContext) {
        let (_, cx) = cx.add_window_view(|_, _| Harness {
            kind: ShapeKind::Heart,
            drawn: None,
        });
        draw(cx);
        assert_eq!(
            cx.debug_bounds("shape").expect("rendered").size,
            size(px(96.), px(96.))
        );
    }

    /// Draw a new shape and return the frame drawn just after the change.
    fn change_shape(cx: &mut gpui::TestAppContext, reduced: bool) -> ShapeFrame {
        cx.update(|cx| cx.set_reduce_motion(reduced));
        let (view, cx) = cx.add_window_view(|_, _| Harness {
            kind: ShapeKind::Heart,
            drawn: None,
        });
        draw(cx);
        let first = view.read_with(cx, |view, _| view.drawn).expect("drawn");
        assert_eq!(
            first,
            ShapeFrame::still(ShapeKind::Heart),
            "no morph on mount"
        );
        view.update(cx, |view, cx| {
            view.kind = ShapeKind::Bun;
            cx.notify();
        });
        draw(cx);
        view.read_with(cx, |view, _| view.drawn).expect("drawn")
    }

    #[gpui::test]
    fn a_new_shape_morphs_from_the_last_one(cx: &mut gpui::TestAppContext) {
        let frame = change_shape(cx, false);
        assert_eq!((frame.from, frame.to), (ShapeKind::Heart, ShapeKind::Bun));
        assert!(frame.progress < 1., "still on its way: {}", frame.progress);
    }

    #[gpui::test]
    fn reduced_motion_shows_a_new_shape_at_once(cx: &mut gpui::TestAppContext) {
        let frame = change_shape(cx, true);
        assert_eq!((frame.to, frame.progress), (ShapeKind::Bun, 1.));
    }
}
