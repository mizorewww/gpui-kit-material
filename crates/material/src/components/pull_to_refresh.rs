//! Pull to refresh: a scrolling area whose top, pulled down, brings in an
//! indicator and calls for a refresh when released past a threshold.
//!
//! Source: only m3e lists this component (the Figma kit and md3.io do not),
//! and there is no m3.material.io page. Values and drag rules are m3e's port
//! of Compose `PullToRefresh.kt` (`packages/m3e-motion/src/pull-to-refresh.ts`,
//! `packages/m3e-css/src/pull-to-refresh.css`,
//! `pull-to-refresh-defaults.css`, from androidx
//! `19991f2c0858aadfcb442ce02b2c8e4743c96898`). The arithmetic is held to m3e's
//! JVM reference, `conformance/fixtures/m3e-pull-to-refresh.json`.
//!
//! Copyright 2023 The Android Open Source Project. Licensed under the Apache
//! License, Version 2.0: see `LICENSE-androidx` in this directory.
//!
//! Input differs from m3e, which pulls on touch and pen only because a browser
//! mouse drag selects text. Here touch pans and trackpads pull through scroll
//! events, and a mouse drag pulls too ([`PullToRefresh::mouse_drag`]) so desktop
//! windows without a touch surface can use it. A mouse wheel never pulls.
use super::{LoadingIndicator, indeterminate_circular_progress};
use crate::{
    color::ColorRole,
    motion::{self, MotionToken, Spring},
    theme::color,
    tokens::{elevation, typography::rem},
};
use gpui::{prelude::*, *};
use std::{rc::Rc, time::Duration};

#[cfg(not(target_family = "wasm"))]
use std::time::Instant;
#[cfg(target_family = "wasm")]
use web_time::Instant;

/// How far (dp, after the drag multiplier) a pull must pass to refresh
/// (Compose `PositionalThreshold`).
pub const PULL_TO_REFRESH_THRESHOLD: f32 = 80.;
/// The drag distance counts half (Compose `DragMultiplier`).
pub const PULL_TO_REFRESH_DRAG_MULTIPLIER: f32 = 0.5;
/// How far the indicator travels at a fraction of 1 (Compose
/// `IndicatorMaxDistance`), in dp.
pub const PULL_TO_REFRESH_MAX_DISTANCE: f32 = 80.;
/// The default indicator's container (Compose `SpinnerContainerSize`), in dp.
pub const PULL_TO_REFRESH_INDICATOR_SIZE: f32 = 40.;
/// The loading indicator variant's container, in dp.
pub const PULL_TO_REFRESH_LOADING_INDICATOR_SIZE: f32 = 48.;
/// The arrow's canvas and the spinner's size (Compose `SpinnerSize`), in dp.
pub const PULL_TO_REFRESH_SPINNER_SIZE: f32 = 16.;
/// The arrow's alpha below the threshold (Compose `MinAlpha`); it is 1 from it.
pub const PULL_TO_REFRESH_MIN_ALPHA: f32 = 0.3;
/// Settling to rest or to the threshold (m3e's
/// `--m3e-comp-pull-to-refresh-state-*`, from Compose's `animateTo` spec).
const SETTLE_SPRING: Spring = Spring {
    damping: 1.,
    stiffness: 1500.,
};
const MAX_PROGRESS_ARC: f32 = 0.8;
const STROKE_WIDTH: f32 = 2.5;
const ARC_RADIUS: f32 = 5.5;
const ARROW_WIDTH: f32 = 10.;
const ARROW_HEIGHT: f32 = 5.;
/// Mouse travel before a press becomes a pull, so clicks and small movements
/// stay with the content.
const MOUSE_SLOP: f32 = 8.;
/// A pause without packets ends a stream that has no phases: the same time
/// `gpui_base`'s overscroll waits, for the same reason.
const GESTURE_GAP: Duration = Duration::from_millis(250);

/// `linearTension - linearTension.pow(2) / 4` of a progress.
fn tension_percent(progress: f32) -> f32 {
    let overshoot = progress.abs() - 1.;
    let linear = overshoot.clamp(0., 2.);
    linear - linear.powi(2) / 4.
}

/// The indicator offset for an adjusted distance pulled: linear to the
/// threshold, then on the tension curve, at most twice the threshold.
pub fn pull_vertical_offset(adjusted_distance: f32, threshold: f32) -> f32 {
    if adjusted_distance <= threshold {
        return adjusted_distance;
    }
    threshold + threshold * tension_percent(adjusted_distance / threshold)
}

/// A pull's drag bookkeeping (Compose `PullToRefreshModifierNode`), in px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PullModel {
    /// The threshold, rounded to whole pixels.
    pub threshold: f32,
    /// The drag distance the pull took.
    pub distance_pulled: f32,
    /// Where the indicator is, with the tension applied.
    pub vertical_offset: f32,
}

impl PullModel {
    /// An idle pull. A threshold that is not positive and finite uses the default.
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold: if threshold.is_finite() && threshold > 0. {
                threshold
            } else {
                PULL_TO_REFRESH_THRESHOLD
            },
            distance_pulled: 0.,
            vertical_offset: 0.,
        }
    }

    /// The drag distance after the multiplier.
    pub fn adjusted_distance(&self) -> f32 {
        self.distance_pulled * PULL_TO_REFRESH_DRAG_MULTIPLIER
    }

    /// The indicator's position as a fraction of the threshold.
    pub fn distance_fraction(&self) -> f32 {
        self.vertical_offset / self.threshold
    }

    /// Take `available` px of drag (positive down) and return what the pull
    /// consumed. Nothing is consumed while refreshing.
    pub fn consume(&mut self, available: f32, refreshing: bool) -> f32 {
        if refreshing {
            return 0.;
        }
        let offset = (self.distance_pulled + available).max(0.);
        let consumed = offset - self.distance_pulled;
        self.distance_pulled = offset;
        self.vertical_offset = pull_vertical_offset(self.adjusted_distance(), self.threshold);
        consumed
    }

    /// Whether letting go now refreshes: past the threshold, not refreshing.
    pub fn release(&self, refreshing: bool) -> bool {
        !refreshing && self.adjusted_distance() > self.threshold
    }

    /// Rest: at 0, or at the threshold while refreshing.
    pub fn reset(&mut self, refreshing: bool) {
        self.distance_pulled = if refreshing { self.threshold } else { 0. };
        self.vertical_offset = self.distance_pulled;
    }
}

/// The arrow's drawing on its canvas at a distance fraction: arc and head,
/// with the rotations Compose applies about the canvas center.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PullArrow {
    /// The canvas's side, in whole pixels.
    pub size: f32,
    /// The stroke width.
    pub stroke: f32,
    /// Degrees the whole drawing turns. Compose passes a fraction of a turn
    /// here as is, so this stays near zero; the value keeps that parity.
    pub rotation: f32,
    /// The arc's bounds as left, top, width, height.
    pub arc: [f32; 4],
    /// Where the arc starts, in degrees clockwise from three o'clock.
    pub start_angle: f32,
    /// How far the arc sweeps, in degrees.
    pub sweep_angle: f32,
    /// The arrow head's open triangle as x, y pairs.
    pub points: [f32; 6],
    /// Degrees the head turns further about the center.
    pub arrow_rotation: f32,
}

/// The arrow for a distance fraction at `density` px per dp.
pub fn pull_arrow(progress: f32, density: f32) -> PullArrow {
    // Discard the first 40% of progress and scale the rest to the full range.
    let adjusted = (progress.min(1.) - 0.4).max(0.) * 5. / 3.;
    let tension = tension_percent(progress);
    let end_trim = adjusted * MAX_PROGRESS_ARC;
    let rotation = (-0.25 + 0.4 * adjusted + tension) * 0.5;
    let start_angle = rotation * 360.;
    let end_angle = (rotation + end_trim) * 360.;
    let scale = adjusted.min(1.);

    let size = (PULL_TO_REFRESH_SPINNER_SIZE * density).round();
    let center = size / 2.;
    let stroke = STROKE_WIDTH * density;
    let arc_radius = ARC_RADIUS * density + stroke / 2.;
    let left = center - arc_radius;
    let width = arc_radius * 2.;
    let arrow_width = ARROW_WIDTH * density * scale;
    let tip_x = arrow_width / 2.;
    let tip_y = ARROW_HEIGHT * density * scale;
    let dx = arc_radius + center - arrow_width / 2.;
    let dy = center - stroke;
    PullArrow {
        size,
        stroke,
        rotation,
        arc: [left, left, width, width],
        start_angle,
        sweep_angle: end_angle - start_angle,
        points: [dx, dy, tip_x + dx, tip_y + dy, arrow_width + dx, dy],
        // Compose subtracts the stroke in px from degrees; the head sits just
        // behind the arc's end because of it.
        arrow_rotation: end_angle - stroke,
    }
}

/// The loading indicator variant's extra turn past the threshold, in degrees.
pub fn pull_loading_rotation(fraction: f32) -> f32 {
    if fraction > 1. {
        -(fraction - 1.) * 180.
    } else {
        0.
    }
}

/// The indicator a [`PullToRefresh`] shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PullToRefreshIndicator {
    /// Compose's arrow, then circular progress, on a 40dp surface.
    #[default]
    Default,
    /// Contained loading indicators on a 48dp primary container.
    LoadingIndicator,
}

/// A settle from one fraction to another.
#[derive(Clone, Copy)]
struct Settle {
    from: f32,
    to: f32,
    started: Instant,
}

/// Transient gesture state. The refreshing flag stays with the caller.
struct PullState {
    model: PullModel,
    /// The fraction shown while no pull is active.
    resting: f32,
    settle: Option<Settle>,
    pulling: bool,
    /// The refreshing flag last rendered.
    was_refreshing: bool,
    /// The mouse's press point and last position, while pressed at the top.
    mouse: Option<(f32, f32)>,
    /// A scroll gesture began with a phase, so its packets may pull.
    gesture: bool,
    last_packet: Option<Instant>,
    /// A timer will end this phaseless stream's pull.
    timer: bool,
}

impl PullState {
    fn settled_fraction(&self, now: Instant) -> f32 {
        match self.settle {
            Some(settle) => {
                let elapsed = now.saturating_duration_since(settle.started).as_secs_f32();
                let duration = SETTLE_SPRING.duration().as_secs_f32();
                if elapsed >= duration {
                    settle.to
                } else {
                    let t = (SETTLE_SPRING.ease())(elapsed / duration);
                    settle.from + (settle.to - settle.from) * t
                }
            }
            None => self.resting,
        }
    }

    fn fraction(&self, now: Instant) -> f32 {
        if self.pulling {
            self.model.distance_fraction()
        } else {
            self.settled_fraction(now)
        }
    }

    fn settle_to(&mut self, to: f32, now: Instant, reduced: bool) {
        let from = self.fraction(now);
        self.resting = to;
        self.settle = (!reduced && from != to).then_some(Settle {
            from,
            to,
            started: now,
        });
    }
}

type Refresh = Rc<dyn Fn(&mut Window, &mut App)>;

/// Let go of an active pull: refresh when `commit` and past the threshold.
fn finish_pull(
    state: &Entity<PullState>,
    commit: bool,
    refreshing: bool,
    on_refresh: Option<&Refresh>,
    window: &mut Window,
    cx: &mut App,
) {
    let now = cx.background_executor().now();
    let reduced = cx.reduce_motion();
    let refresh = state.update(cx, |state, cx| {
        let refresh = commit && state.model.release(refreshing);
        let from = state.model.distance_fraction();
        state.pulling = false;
        state.resting = from;
        state.settle = None;
        state.settle_to(0., now, reduced);
        state.model.reset(false);
        cx.notify();
        refresh
    });
    if refresh && let Some(on_refresh) = on_refresh {
        on_refresh(window, cx);
    }
}

/// Pull to refresh around caller-owned scrolling content.
///
/// The refreshing flag is controlled: [`Self::on_refresh`] reports a release
/// past the threshold, and the caller sets [`Self::refreshing`] until its work
/// is done. The gesture has no keyboard form, so give the screen a visible
/// refresh control that calls the same handler.
///
/// ```
/// use gpui_kit_material::prelude::*;
///
/// let list = PullToRefresh::new("inbox", "Refreshing the inbox")
///     .indicator(PullToRefreshIndicator::LoadingIndicator)
///     .on_refresh(|_, _| { /* start the work and set `refreshing` */ });
/// # let _ = list;
/// ```
#[derive(gpui::IntoElement)]
pub struct PullToRefresh {
    id: ElementId,
    label: SharedString,
    refreshing: bool,
    disabled: bool,
    indicator: PullToRefreshIndicator,
    threshold: f32,
    mouse_drag: bool,
    scroll: Option<ScrollHandle>,
    content: Vec<AnyElement>,
    on_refresh: Option<Refresh>,
}

impl PullToRefresh {
    /// `label` names the indicator for assistive technology.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            refreshing: false,
            disabled: false,
            indicator: PullToRefreshIndicator::Default,
            threshold: PULL_TO_REFRESH_THRESHOLD,
            mouse_drag: true,
            scroll: None,
            content: Vec::new(),
            on_refresh: None,
        }
    }

    /// Whether a refresh is running: the indicator rests at the threshold,
    /// spinning, and pulls are ignored.
    pub fn refreshing(mut self, refreshing: bool) -> Self {
        self.refreshing = refreshing;
        self
    }

    /// Ignore pulls.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Which indicator to show.
    pub fn indicator(mut self, indicator: PullToRefreshIndicator) -> Self {
        self.indicator = indicator;
        self
    }

    /// The distance (dp, after the drag multiplier) a pull must pass.
    pub fn threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    /// Whether a mouse drag pulls (on by default).
    pub fn mouse_drag(mut self, mouse_drag: bool) -> Self {
        self.mouse_drag = mouse_drag;
        self
    }

    /// The scroll position pulls start from. By default the element keeps its
    /// own; pass one to read or move the content's position.
    pub fn track_scroll(mut self, scroll: &ScrollHandle) -> Self {
        self.scroll = Some(scroll.clone());
        self
    }

    /// Add to the scrolling content.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.content.push(child.into_any_element());
        self
    }

    /// Called when a pull is released past the threshold.
    pub fn on_refresh(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_refresh = Some(Rc::new(handler));
        self
    }
}

/// The arrow's built paths and what they were built from. Owned by the
/// indicator's keyed state: any change of fraction or bounds replaces them.
#[derive(Default)]
struct ArrowPaths {
    key: Option<(u32, Bounds<Pixels>)>,
    arc: Option<Path<Pixels>>,
    head: Option<Path<Pixels>>,
}

impl ArrowPaths {
    /// Build the paths unless they are already current.
    fn update(&mut self, fraction: f32, bounds: Bounds<Pixels>) {
        let key = (fraction.to_bits(), bounds);
        if self.key != Some(key) {
            let (arc, head) = arrow_paths(fraction, bounds);
            *self = Self {
                key: Some(key),
                arc,
                head,
            };
        }
    }
}

/// Build the arrow's arc and head for `bounds`.
fn arrow_paths(
    fraction: f32,
    bounds: Bounds<Pixels>,
) -> (Option<Path<Pixels>>, Option<Path<Pixels>>) {
    let scale = f32::from(bounds.size.width) / PULL_TO_REFRESH_SPINNER_SIZE;
    let arrow = pull_arrow(fraction, 1.);
    let center = arrow.size / 2.;
    let at = |x: f32, y: f32, degrees: f32| {
        let (sin, cos) = degrees.to_radians().sin_cos();
        let (dx, dy) = (x - center, y - center);
        point(
            bounds.origin.x + px((center + dx * cos - dy * sin) * scale),
            bounds.origin.y + px((center + dx * sin + dy * cos) * scale),
        )
    };
    let mut arc = None;
    if arrow.sweep_angle > 0. {
        const STEPS: usize = 32;
        let [left, top, width, height] = arrow.arc;
        let (rx, ry) = (width / 2., height / 2.);
        let mut path = PathBuilder::stroke(px(arrow.stroke * scale));
        for step in 0..=STEPS {
            let angle =
                (arrow.start_angle + arrow.sweep_angle * step as f32 / STEPS as f32).to_radians();
            let point = at(
                left + rx + rx * angle.cos(),
                top + ry + ry * angle.sin(),
                arrow.rotation,
            );
            if step == 0 {
                path.move_to(point);
            } else {
                path.line_to(point);
            }
        }
        arc = path.build().ok();
    }
    let mut head = None;
    let [ax, ay, bx, by, cx, cy] = arrow.points;
    if ax != cx || ay != by {
        let turn = arrow.rotation + arrow.arrow_rotation;
        let mut path = PathBuilder::stroke(px(arrow.stroke * scale));
        path.move_to(at(ax, ay, turn));
        path.line_to(at(bx, by, turn));
        path.line_to(at(cx, cy, turn));
        head = path.build().ok();
    }
    (arc, head)
}

impl RenderOnce for PullToRefresh {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let now = cx.background_executor().now();
        let reduced = cx.reduce_motion();
        let refreshing = self.refreshing;
        let threshold = self.threshold;
        let state = window.use_keyed_state((self.id.clone(), "pull-state"), cx, |_, _| PullState {
            model: PullModel::new(threshold),
            resting: if refreshing { 1. } else { 0. },
            settle: None,
            pulling: false,
            was_refreshing: refreshing,
            mouse: None,
            gesture: false,
            last_packet: None,
            timer: false,
        });
        let scroll = match self.scroll.clone() {
            Some(scroll) => scroll,
            None => window
                .use_keyed_state((self.id.clone(), "pull-scroll"), cx, |_, _| {
                    ScrollHandle::default()
                })
                .read(cx)
                .clone(),
        };
        let (fraction, pulling, settling) = state.update(cx, |state, _| {
            state.model.threshold = PullModel::new(threshold).threshold;
            if state.was_refreshing != refreshing {
                state.was_refreshing = refreshing;
                state.pulling = false;
                state.settle_to(if refreshing { 1. } else { 0. }, now, reduced);
                state.model.reset(refreshing);
            }
            if let Some(settle) = state.settle {
                let elapsed = now.saturating_duration_since(settle.started);
                if elapsed >= SETTLE_SPRING.duration() {
                    state.settle = None;
                }
            }
            (state.fraction(now), state.pulling, state.settle.is_some())
        });
        if settling {
            window.request_animation_frame();
        }

        let enabled = !self.disabled && !refreshing;
        let on_refresh = self.on_refresh;
        let mouse_drag = self.mouse_drag;
        // Nothing to listen for when no pull can start or is running.
        let events = (enabled || pulling).then(|| {
            let state = state.clone();
            let scroll = scroll.clone();
            canvas(
                move |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
                move |_, hitbox, window, _| {
                    pull_listeners(
                        hitbox, state, scroll, enabled, refreshing, mouse_drag, on_refresh, window,
                    );
                },
            )
            .absolute()
            .inset_0()
        });

        let visible = pulling || refreshing || fraction > 0.;
        let loading = self.indicator == PullToRefreshIndicator::LoadingIndicator;
        let box_size = if loading {
            PULL_TO_REFRESH_LOADING_INDICATOR_SIZE
        } else {
            PULL_TO_REFRESH_INDICATOR_SIZE
        };
        let id = self.id.clone();
        let label = self.label.clone();
        let indicator = visible.then(|| {
            let spinner_alpha = motion::token_value(
                (id.clone(), "pull-crossfade"),
                if refreshing { 1. } else { 0. },
                MotionToken::DefaultEffects,
                window,
                cx,
            )
            .clamp(0., 1.);
            let arrow_alpha = motion::token_value(
                (id.clone(), "pull-arrow-alpha"),
                if fraction >= 1. {
                    1.
                } else {
                    PULL_TO_REFRESH_MIN_ALPHA
                },
                MotionToken::DefaultEffects,
                window,
                cx,
            );
            let face: AnyElement = if loading {
                if refreshing {
                    LoadingIndicator::new((id.clone(), "pull-refreshing"), label.clone())
                        .contained(true)
                        .into_any_element()
                } else {
                    LoadingIndicator::new((id.clone(), "pull-progress"), label.clone())
                        .contained(true)
                        .progress(Some(fraction.min(1.)))
                        .turn(pull_loading_rotation(fraction))
                        .into_any_element()
                }
            } else {
                let ink = color(ColorRole::ON_SURFACE_VARIANT);
                let spinner_size = size(
                    rem(PULL_TO_REFRESH_SPINNER_SIZE),
                    rem(PULL_TO_REFRESH_SPINNER_SIZE),
                );
                div()
                    .relative()
                    .size(rem(PULL_TO_REFRESH_SPINNER_SIZE))
                    .when(spinner_alpha < 1., |face| {
                        let alpha = arrow_alpha * (1. - spinner_alpha);
                        let paths =
                            window.use_keyed_state((id.clone(), "pull-arrow-paths"), cx, |_, _| {
                                ArrowPaths::default()
                            });

                        face.child(
                            gpui_base::Progress::new((id.clone(), "pull-arrow"))
                                .accessibility_label(label.clone())
                                .value(fraction.min(1.) * 100.)
                                .absolute()
                                .inset_0()
                                .child(
                                    canvas(
                                        |_, _, _| {},
                                        move |bounds, _, window, cx| {
                                            let ink = ink.opacity(alpha);
                                            paths.update(cx, |paths, _| {
                                                paths.update(fraction, bounds);
                                                for path in
                                                    [&paths.arc, &paths.head].into_iter().flatten()
                                                {
                                                    window.paint_path(path.clone(), ink);
                                                }
                                            });
                                        },
                                    )
                                    .size_full(),
                                ),
                        )
                    })
                    .when(spinner_alpha > 0., |face| {
                        face.child(indeterminate_circular_progress(
                            (id.clone(), "pull-spinner"),
                            label.clone(),
                            spinner_size,
                            spinner_alpha,
                            window,
                            cx,
                        ))
                    })
                    .into_any_element()
            };
            div()
                .debug_selector(|| "pull-to-refresh-indicator".into())
                .absolute()
                .top(rem(fraction * PULL_TO_REFRESH_MAX_DISTANCE - box_size))
                .left_0()
                .right_0()
                .flex()
                .justify_center()
                .child(
                    div()
                        .size(rem(box_size))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .bg(color(if loading {
                            ColorRole::PRIMARY_CONTAINER
                        } else {
                            ColorRole::SURFACE_CONTAINER_HIGH
                        }))
                        .when(!loading, |surface| surface.shadow(elevation::level(2)))
                        .child(face),
                )
        });
        div()
            .id((self.id.clone(), "pull"))
            .relative()
            .size_full()
            .overflow_hidden()
            .child(
                div()
                    .id((self.id, "pull-content"))
                    .debug_selector(|| "pull-to-refresh-content".into())
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .children(self.content),
            )
            .children(events)
            // During a pull nothing beneath takes a hover or the release's click.
            .when(pulling, |root| {
                root.child(div().absolute().inset_0().occlude())
            })
            .children(indicator)
    }
}

/// Install the gesture's window listeners for this frame. They run in the
/// capture phase, because the scroller would otherwise take a packet that
/// should first unwind the pull.
#[allow(clippy::too_many_arguments)]
fn pull_listeners(
    hitbox: Hitbox,
    state: Entity<PullState>,
    scroll: ScrollHandle,
    enabled: bool,
    refreshing: bool,
    mouse_drag: bool,
    on_refresh: Option<Refresh>,
    window: &mut Window,
) {
    {
        let state = state.clone();
        let scroll = scroll.clone();
        let on_refresh = on_refresh.clone();
        let hitbox = hitbox.clone();
        window.on_mouse_event(move |event: &ScrollEvent, phase, window, cx| {
            if phase != DispatchPhase::Capture {
                return;
            }
            let now = cx.background_executor().now();
            let current = state.read(cx);
            let pulling = current.pulling;
            if !pulling && !hitbox.should_handle_scroll(window) {
                return;
            }
            match event.touch_phase {
                TouchPhase::Started => {
                    state.update(cx, |state, _| {
                        state.gesture = true;
                        state.last_packet = Some(now);
                    });
                    return;
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    state.update(cx, |state, _| {
                        state.gesture = false;
                        state.last_packet = Some(now);
                    });
                    if pulling {
                        let commit = event.touch_phase == TouchPhase::Ended;
                        finish_pull(&state, commit, refreshing, on_refresh.as_ref(), window, cx);
                        cx.stop_propagation();
                    }
                    return;
                }
                TouchPhase::Moved => {}
            }
            // A wheel notch never pulls.
            let ScrollDelta::Pixels(delta) = event.delta else {
                return;
            };
            let dy = f32::from(delta.y);
            let current = state.read(cx);
            // Packets after a phased gesture ended are momentum: they may
            // scroll, but a fling arriving at the top is not a pull. A stream
            // with no phases starts fresh after a pause.
            let fresh = current.gesture
                || current
                    .last_packet
                    .is_none_or(|last| now.saturating_duration_since(last) > GESTURE_GAP);
            let phaseless = !current.gesture;
            let at_top = scroll.offset().y >= px(0.);
            let start = !pulling && enabled && at_top && dy > 0. && fresh;
            state.update(cx, |state, _| state.last_packet = Some(now));
            if !pulling && !start {
                return;
            }
            let arm_timer = state.update(cx, |state, cx| {
                state.model.consume(dy, refreshing);
                state.pulling = state.model.distance_pulled > 0.;
                state.settle = None;
                cx.notify();
                let arm = state.pulling && phaseless && !state.timer;
                if arm {
                    state.timer = true;
                }
                arm
            });
            cx.stop_propagation();
            if arm_timer {
                let state = state.clone();
                let on_refresh = on_refresh.clone();
                window
                    .spawn(cx, async move |cx| {
                        loop {
                            cx.background_executor().timer(GESTURE_GAP).await;
                            let done = cx
                                .update(|window, cx| {
                                    let current = state.read(cx);
                                    let now = cx.background_executor().now();
                                    let idle = current.last_packet.is_none_or(|last| {
                                        now.saturating_duration_since(last) >= GESTURE_GAP
                                    });
                                    if !current.pulling || current.gesture {
                                        state.update(cx, |state, _| state.timer = false);
                                        return true;
                                    }
                                    if idle {
                                        state.update(cx, |state, _| state.timer = false);
                                        finish_pull(
                                            &state,
                                            true,
                                            refreshing,
                                            on_refresh.as_ref(),
                                            window,
                                            cx,
                                        );
                                    }
                                    idle
                                })
                                .unwrap_or(true);
                            if done {
                                break;
                            }
                        }
                    })
                    .detach();
            }
        });
    }
    if !mouse_drag {
        return;
    }
    {
        let state = state.clone();
        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
            if phase == DispatchPhase::Capture
                && event.button == MouseButton::Left
                && enabled
                && scroll.offset().y >= px(0.)
                && hitbox.is_hovered(window)
            {
                let y = f32::from(event.position.y);
                state.update(cx, |state, _| state.mouse = Some((y, y)));
            }
        });
    }
    // These listen whether or not a press is in progress: a press changes
    // nothing on screen, so a frame painted after it may never come.
    {
        let state = state.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase != DispatchPhase::Capture {
                return;
            }
            let Some((press, last)) = state.read(cx).mouse else {
                return;
            };
            if !event.dragging() {
                // The release went elsewhere. Nothing confirms it was meant
                // as one, so a pull in progress lets go without refreshing.
                let pulling = state.read(cx).pulling;
                state.update(cx, |state, _| state.mouse = None);
                if pulling {
                    finish_pull(&state, false, refreshing, None, window, cx);
                }
                return;
            }
            let y = f32::from(event.position.y);
            let pulling = state.read(cx).pulling;
            if !pulling && y - press < MOUSE_SLOP {
                return;
            }
            state.update(cx, |state, cx| {
                state.model.consume(y - last, refreshing);
                state.pulling = state.model.distance_pulled > 0.;
                state.settle = None;
                state.mouse = Some((press, y));
                cx.notify();
            });
            cx.stop_propagation();
        });
    }
    window.on_mouse_event(move |_: &MouseUpEvent, phase, window, cx| {
        if phase != DispatchPhase::Capture {
            return;
        }
        let current = state.read(cx);
        if current.mouse.is_none() {
            return;
        }
        let pulling = current.pulling;
        state.update(cx, |state, _| state.mouse = None);
        if pulling {
            finish_pull(&state, true, refreshing, on_refresh.as_ref(), window, cx);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use serde_json::Value;
    use std::cell::Cell;

    const REFERENCE: &str =
        include_str!("../../../../conformance/fixtures/m3e-pull-to-refresh.json");

    fn number(value: &Value) -> f32 {
        value.as_f64().expect("a number") as f32
    }

    #[track_caller]
    fn close(actual: f32, expected: f32, what: &str) {
        // The port rounds to f32 as Kotlin does; allow for operation order.
        let tolerance = 2e-4_f32.max(expected.abs() * 2e-6);
        assert!(
            (actual - expected).abs() <= tolerance,
            "{what}: {actual} != {expected}"
        );
    }

    #[test]
    fn the_constants_are_the_references() {
        let reference: Value = serde_json::from_str(REFERENCE).unwrap();
        close(
            PULL_TO_REFRESH_DRAG_MULTIPLIER,
            number(&reference["dragMultiplier"]),
            "multiplier",
        );
        close(
            PULL_TO_REFRESH_MIN_ALPHA,
            number(&reference["minAlpha"]),
            "alpha",
        );
        close(
            PULL_TO_REFRESH_INDICATOR_SIZE,
            number(&reference["spinnerContainerSize"]),
            "container",
        );
    }

    #[test]
    fn the_arrow_matches_every_reference_sample() {
        let reference: Value = serde_json::from_str(REFERENCE).unwrap();
        let arrows = reference["arrows"].as_array().unwrap();
        assert!(arrows.len() > 40, "{}", arrows.len());
        for sample in arrows {
            let progress = number(&sample["progress"]);
            let density = number(&sample["density"]);
            let arrow = pull_arrow(progress, density);
            let what = format!("arrow at {progress} x{density}");
            close(arrow.size, number(&sample["size"]), &what);
            close(arrow.rotation, number(&sample["rotation"]), &what);
            let arc = sample["arc"].as_array().unwrap();
            for (index, expected) in arc.iter().take(4).enumerate() {
                close(arrow.arc[index], number(expected), &what);
            }
            close(arrow.start_angle, number(&arc[4]), &what);
            close(arrow.sweep_angle, number(&arc[5]), &what);
            close(arrow.stroke, number(&arc[6]), &what);
            for (index, expected) in sample["points"].as_array().unwrap().iter().enumerate() {
                close(arrow.points[index], number(expected), &what);
            }
            close(
                arrow.arrow_rotation,
                number(&sample["arrowRotations"][1]),
                &what,
            );
        }
    }

    #[test]
    fn drags_match_every_reference_sequence() {
        let reference: Value = serde_json::from_str(REFERENCE).unwrap();
        let drags = reference["drags"].as_array().unwrap();
        assert!(drags.len() >= 7, "{}", drags.len());
        for drag in drags {
            let name = drag["name"].as_str().unwrap();
            let refreshing = drag["refreshing"].as_bool().unwrap();
            let mut model = PullModel::new(number(&drag["thresholdPx"]));
            for step in drag["steps"].as_array().unwrap() {
                let consumed = model.consume(number(&step["available"]), refreshing);
                close(consumed, number(&step["consumed"]), name);
                close(model.distance_pulled, number(&step["distancePulled"]), name);
                close(model.vertical_offset, number(&step["verticalOffset"]), name);
                close(model.distance_fraction(), number(&step["fraction"]), name);
            }
            assert_eq!(
                model.release(refreshing),
                drag["refreshed"].as_bool().unwrap(),
                "{name}"
            );
        }
    }

    #[test]
    fn a_bad_threshold_uses_the_default_and_nothing_divides_by_zero() {
        for threshold in [0., -5., f32::NAN, f32::INFINITY] {
            let mut model = PullModel::new(threshold);
            assert_eq!(model.threshold, PULL_TO_REFRESH_THRESHOLD);
            model.consume(40., false);
            assert!(model.distance_fraction().is_finite());
        }
    }

    #[test]
    fn the_loading_indicator_turns_back_only_past_the_threshold() {
        assert_eq!(pull_loading_rotation(0.5), 0.);
        assert_eq!(pull_loading_rotation(1.), 0.);
        assert_eq!(pull_loading_rotation(1.5), -90.);
    }

    struct Screen {
        refreshing: bool,
        disabled: bool,
        refreshes: Rc<Cell<usize>>,
        clicks: Rc<Cell<usize>>,
        scroll: ScrollHandle,
    }

    impl Render for Screen {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(crate::tokens::typography::BASE_PX));
            let refreshes = self.refreshes.clone();
            let clicks = self.clicks.clone();
            div().size(px(400.)).child(
                PullToRefresh::new("ptr", "Refreshing")
                    .refreshing(self.refreshing)
                    .disabled(self.disabled)
                    .track_scroll(&self.scroll)
                    .on_refresh(move |_, _| refreshes.set(refreshes.get() + 1))
                    .child(
                        div()
                            .id("row")
                            .debug_selector(|| "row".into())
                            .h(px(2000.))
                            .w_full()
                            .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
                    ),
            )
        }
    }

    fn open(cx: &mut gpui::TestAppContext) -> (Entity<Screen>, &mut gpui::VisualTestContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let (view, cx) = cx.add_window_view(|_, _| Screen {
            refreshing: false,
            disabled: false,
            refreshes: Rc::default(),
            clicks: Rc::default(),
            scroll: ScrollHandle::default(),
        });
        draw(cx);
        (view, cx)
    }

    fn draw(cx: &mut gpui::VisualTestContext) {
        for _ in 0..2 {
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
            cx.run_until_parked();
        }
    }

    fn scroll(cx: &mut gpui::VisualTestContext, phase: TouchPhase, dy: f32) {
        cx.simulate_event(ScrollEvent {
            position: point(px(200.), px(200.)),
            delta: ScrollDelta::Pixels(point(px(0.), px(dy))),
            touch_phase: phase,
            ..Default::default()
        });
        draw(cx);
    }

    fn indicator_top(cx: &mut gpui::VisualTestContext) -> Option<f32> {
        cx.debug_bounds("pull-to-refresh-indicator")
            .map(|bounds| f32::from(bounds.top()))
    }

    #[gpui::test]
    fn a_pull_past_the_threshold_refreshes_once_and_a_short_one_does_not(
        cx: &mut gpui::TestAppContext,
    ) {
        let (view, cx) = open(cx);
        assert_eq!(indicator_top(cx), None, "nothing shows at rest");
        scroll(cx, TouchPhase::Started, 0.);
        scroll(cx, TouchPhase::Moved, 80.);
        // 80px of drag counts half: a fraction of 0.5 moves the box 40dp from
        // just above the top, which leaves it level with the top edge.
        assert_eq!(indicator_top(cx), Some(0.));
        scroll(cx, TouchPhase::Ended, 0.);
        let refreshes = view.read_with(cx, |view, _| view.refreshes.clone());
        assert_eq!(refreshes.get(), 0);
        assert_eq!(indicator_top(cx), None, "it returns to rest");

        scroll(cx, TouchPhase::Started, 0.);
        scroll(cx, TouchPhase::Moved, 200.);
        scroll(cx, TouchPhase::Ended, 0.);
        assert_eq!(refreshes.get(), 1);
    }

    #[gpui::test]
    fn refreshing_holds_the_indicator_at_the_threshold_and_ignores_pulls(
        cx: &mut gpui::TestAppContext,
    ) {
        let (view, cx) = open(cx);
        cx.update(|_, cx| view.update(cx, |view, _| view.refreshing = true));
        draw(cx);
        let at_threshold = PULL_TO_REFRESH_MAX_DISTANCE - PULL_TO_REFRESH_INDICATOR_SIZE;
        assert_eq!(indicator_top(cx), Some(at_threshold));
        scroll(cx, TouchPhase::Started, 0.);
        scroll(cx, TouchPhase::Moved, 300.);
        scroll(cx, TouchPhase::Ended, 0.);
        assert_eq!(indicator_top(cx), Some(at_threshold));
        let refreshes = view.read_with(cx, |view, _| view.refreshes.clone());
        assert_eq!(refreshes.get(), 0);
        cx.update(|_, cx| view.update(cx, |view, _| view.refreshing = false));
        draw(cx);
        assert_eq!(indicator_top(cx), None);
    }

    #[gpui::test]
    fn a_scrolled_list_scrolls_instead_of_pulling(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx);
        let handle = view.read_with(cx, |view, _| view.scroll.clone());
        scroll(cx, TouchPhase::Started, 0.);
        scroll(cx, TouchPhase::Moved, -300.);
        assert!(handle.offset().y < px(0.), "the content scrolled");
        scroll(cx, TouchPhase::Moved, 100.);
        assert_eq!(indicator_top(cx), None, "not at the top: no pull");
        assert!(handle.offset().y < px(0.));
    }

    #[gpui::test]
    fn pulling_back_unwinds_the_pull_before_the_content_scrolls(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx);
        let handle = view.read_with(cx, |view, _| view.scroll.clone());
        scroll(cx, TouchPhase::Started, 0.);
        scroll(cx, TouchPhase::Moved, 100.);
        scroll(cx, TouchPhase::Moved, -40.);
        assert_eq!(handle.offset().y, px(0.), "the pull took the packet");
        assert!(indicator_top(cx).is_some());
    }

    #[gpui::test]
    fn momentum_after_a_gesture_is_not_a_pull(cx: &mut gpui::TestAppContext) {
        let (_, cx) = open(cx);
        scroll(cx, TouchPhase::Started, 0.);
        scroll(cx, TouchPhase::Ended, 0.);
        scroll(cx, TouchPhase::Moved, 50.);
        assert_eq!(indicator_top(cx), None);
    }

    #[gpui::test]
    fn a_phaseless_stream_pulls_and_a_pause_releases_it(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx);
        scroll(cx, TouchPhase::Moved, 200.);
        assert!(indicator_top(cx).is_some());
        cx.executor().advance_clock(GESTURE_GAP * 2);
        draw(cx);
        let refreshes = view.read_with(cx, |view, _| view.refreshes.clone());
        assert_eq!(refreshes.get(), 1);
        assert_eq!(indicator_top(cx), None);
    }

    #[gpui::test]
    fn a_mouse_drag_pulls_and_its_release_clicks_nothing(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx);
        let start = point(px(200.), px(100.));
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        draw(cx);
        let end = point(px(200.), px(320.));
        cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
        draw(cx);
        assert!(indicator_top(cx).is_some());
        cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
        draw(cx);
        let (refreshes, clicks) =
            view.read_with(cx, |view, _| (view.refreshes.clone(), view.clicks.clone()));
        assert_eq!(refreshes.get(), 1);
        assert_eq!(clicks.get(), 0, "a pull's release is not a click");
        // A plain click still reaches the content.
        cx.simulate_click(start, Modifiers::none());
        assert_eq!(clicks.get(), 1);
    }

    /// Its content takes every press for itself, so a press draws nothing: the
    /// pull's own stateful elements ask for a frame only as a press bubbles to
    /// them, and it never does.
    struct PressTaken {
        refreshes: Rc<Cell<usize>>,
    }
    impl Render for PressTaken {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let refreshes = self.refreshes.clone();
            div().size(px(400.)).child(
                PullToRefresh::new("pull", "Refreshing")
                    .on_refresh(move |_, _| refreshes.set(refreshes.get() + 1))
                    .child(
                        div()
                            .h(px(2000.))
                            .w_full()
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation()),
                    ),
            )
        }
    }

    #[gpui::test]
    fn a_mouse_pull_starts_on_content_that_takes_the_press(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let refreshes: Rc<Cell<usize>> = Rc::default();
        let (_, cx) = cx.add_window_view({
            let refreshes = refreshes.clone();
            move |_, _| PressTaken { refreshes }
        });
        draw(cx);
        // The pointer is already there, as it is before any press.
        let start = point(px(200.), px(100.));
        cx.simulate_mouse_move(start, None::<MouseButton>, Modifiers::none());
        draw(cx);
        // A test window draws as soon as it is dirty, so a press that asked for
        // a frame would hide the listeners missing from the frame before it.
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        let end = point(px(200.), px(320.));
        cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
        draw(cx);
        assert!(indicator_top(cx).is_some(), "the move started a pull");
        cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
        draw(cx);
        assert_eq!(refreshes.get(), 1);
    }

    #[gpui::test]
    fn a_lost_release_ends_the_pull_without_refreshing(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx);
        let start = point(px(200.), px(100.));
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        draw(cx);
        let end = point(px(200.), px(320.));
        cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
        draw(cx);
        assert!(indicator_top(cx).is_some());
        // The release landed outside the window: the next move has no button.
        cx.simulate_mouse_move(end, None::<MouseButton>, Modifiers::none());
        draw(cx);
        assert_eq!(indicator_top(cx), None, "the pull let go");
        let (refreshes, clicks) =
            view.read_with(cx, |view, _| (view.refreshes.clone(), view.clicks.clone()));
        assert_eq!(refreshes.get(), 0, "a lost release does not commit");
        // Nothing occludes the content and the next click is only a click.
        cx.simulate_click(start, Modifiers::none());
        draw(cx);
        assert_eq!(refreshes.get(), 0, "the click after it does not commit");
        assert_eq!(clicks.get(), 1, "the click reaches the content");
    }

    #[gpui::test]
    fn disabled_ignores_pulls(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx);
        cx.update(|_, cx| view.update(cx, |view, _| view.disabled = true));
        draw(cx);
        scroll(cx, TouchPhase::Started, 0.);
        scroll(cx, TouchPhase::Moved, 300.);
        assert_eq!(indicator_top(cx), None);
    }

    #[gpui::test]
    fn the_settle_animates_with_motion_and_jumps_under_reduced_motion(
        cx: &mut gpui::TestAppContext,
    ) {
        let (view, cx) = open(cx);
        cx.update(|_, cx| cx.set_reduce_motion(false));
        cx.update(|_, cx| view.update(cx, |view, _| view.refreshing = true));
        draw(cx);
        let target = PULL_TO_REFRESH_MAX_DISTANCE - PULL_TO_REFRESH_INDICATOR_SIZE;
        let first = indicator_top(cx).expect("the settle starts");
        assert!(first < target, "{first}");
        cx.executor().advance_clock(SETTLE_SPRING.duration());
        draw(cx);
        assert_eq!(indicator_top(cx), Some(target));
    }
}
