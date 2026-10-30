//! Circular progress from the pinned reference's canvas and motion functions.
use crate::{color::ColorRole, theme::color};
use gpui::{prelude::*, *};
use gpui_base::{Easing, IterationCount, Progress, Timing};
#[cfg(not(target_family = "wasm"))]
use std::time::Instant;
use std::{cell::RefCell, f32::consts::TAU, rc::Rc, time::Duration};
#[cfg(target_family = "wasm")]
use web_time::Instant;

/// A circular arc's start in clockwise degrees from three o'clock and its fraction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CircularProgressFrame {
    pub rotation_degrees: f32,
    pub sweep: f32,
}

impl CircularProgressFrame {
    /// Measured progress begins at twelve o'clock. This does not animate the value.
    pub fn determinate(fraction: f32) -> Self {
        Self {
            rotation_degrees: -90.,
            sweep: normalized_fraction(fraction),
        }
    }
}

fn normalized_fraction(fraction: f32) -> f32 {
    if fraction.is_finite() {
        fraction.clamp(0., 1.)
    } else {
        0.
    }
}

/// The reference's six-second growing arc, including four eased rotation kicks.
pub fn circular_indeterminate_frame(elapsed_ms: f32) -> CircularProgressFrame {
    let t = if elapsed_ms.is_finite() {
        elapsed_ms.rem_euclid(6000.)
    } else {
        0.
    };
    let completed = (t / 1500.).floor();
    let kick = ((t - completed * 1500.) / 300.).min(1.);
    let decelerate = Easing::cubic_bezier(0.05, 0.7, 0.1, 1.).unwrap();
    let standard = Easing::cubic_bezier(0.2, 0., 0., 1.).unwrap();
    CircularProgressFrame {
        rotation_degrees: t * 0.18 + completed * 90. + 90. * decelerate.sample(kick),
        sweep: if t <= 3000. {
            0.1 + 0.77 * t / 3000.
        } else {
            0.87 - 0.77 * standard.sample((t - 3000.) / 3000.)
        },
    }
}

/// Sample independently of painting so a hidden indicator keeps its phase.
///
/// Keep calling this after first showing the indicator, including while hidden.
/// The reference polls reduced motion in its frame callback: a running loop draws
/// one last frame then stops. Changing back does not restart it. A new ID starts
/// a fresh playback, as explicitly restarting the reference's indicator does.
///
/// `visible` states whether the caller paints the returned frame. Only a visible,
/// running sample requests the next animation frame; a hidden sample advances
/// and observes reduced motion without keeping the window out of idle.
pub fn sample_circular_progress(
    id: impl Into<ElementId>,
    visible: bool,
    window: &mut Window,
    cx: &mut App,
) -> CircularProgressFrame {
    let (frame, running) = sample_clock(id.into(), window, cx);
    if visible && running {
        window.request_animation_frame();
    }
    frame
}

/// Caller-owned circular playback that survives virtual unmounting.
///
/// A new handle starts on its first sample (normally the first mounted render).
/// Clones share one logical indicator's start time, stopped flag and last frame;
/// no window, pointer state or scheduled callbacks are retained. Use within one
/// app, and create a new handle to restart a stopped playback.
///
/// Reduced motion stops an existing playback permanently after one final sample.
/// An owner whose indicator is unmounted must call [`Self::observe_reduced_motion`]
/// when the preference changes, so an offscreen enable/disable is not missed.
#[derive(Clone, Default)]
pub struct CircularProgressPlayback {
    state: Rc<RefCell<Option<CircularClock>>>,
}

struct CircularClock {
    started: Instant,
    stopped: bool,
    last_frame: CircularProgressFrame,
}

impl CircularProgressPlayback {
    /// Sample the reference clock without requesting a frame. Calling this for
    /// the first time starts playback, even when no drawing element is mounted.
    pub fn sample(&self, cx: &App) -> CircularProgressFrame {
        self.sample_running(cx).0
    }

    /// Observe a reduced-motion preference change without starting an unused
    /// indicator or requesting frames. Call after updating the app preference,
    /// including while unmounted. Disabling reduced motion never restarts it.
    pub fn observe_reduced_motion(&self, cx: &App) {
        if cx.reduce_motion() && self.state.borrow().is_some() {
            self.sample_running(cx);
        }
    }

    fn sample_running(&self, cx: &App) -> (CircularProgressFrame, bool) {
        let now = cx.background_executor().now();
        let reduced = cx.reduce_motion();
        let mut state = self.state.borrow_mut();
        let state = state.get_or_insert_with(|| CircularClock {
            started: now,
            stopped: reduced,
            last_frame: circular_indeterminate_frame(0.),
        });
        if state.stopped {
            return (state.last_frame, false);
        }
        let timing = Timing::new(Duration::from_millis(6000)).iterations(IterationCount::Infinite);
        let time = timing
            .sample(now.saturating_duration_since(state.started))
            .directed_progress
            * 6000.;
        state.last_frame = circular_indeterminate_frame(time);
        state.stopped = reduced;
        (state.last_frame, !reduced)
    }
}

// The default API keeps its existing consecutive-frame keyed lifetime.
fn sample_clock(id: ElementId, window: &mut Window, cx: &mut App) -> (CircularProgressFrame, bool) {
    let state = window.use_keyed_state((id, "circular-progress-clock"), cx, |_, _| {
        CircularProgressPlayback::default()
    });
    state.read(cx).clone().sample_running(cx)
}

#[derive(Clone, Copy, Debug)]
struct ArcGeometry {
    start: f32,
    sweep: f32,
}

fn geometry(frame: CircularProgressFrame) -> (ArcGeometry, ArcGeometry) {
    let sweep = frame.sweep.clamp(0., 1.) * TAU;
    // The fixed 24-unit canvas has a 2-unit stroke and a 4-unit clear gap.
    let gap = sweep.min(0.5);
    let start = frame.rotation_degrees.to_radians();
    (
        ArcGeometry {
            start: start + sweep + gap,
            sweep: (TAU - sweep - 2. * gap).max(0.),
        },
        ArcGeometry { start, sweep },
    )
}

/// Filled outline of a radius-11, thickness-2 arc with circular end caps.
/// Filling the outline before scaling preserves the reference's elliptical
/// strokes and caps when its 24-unit bitmap is squeezed into a 20×24 CSS box.
fn arc_path(bounds: Bounds<Pixels>, arc: ArcGeometry) -> Option<Path<Pixels>> {
    if arc.sweep <= 0. || bounds.size.width <= px(0.) || bounds.size.height <= px(0.) {
        return None;
    }
    let scale_x = f32::from(bounds.size.width) / 24.;
    let scale_y = f32::from(bounds.size.height) / 24.;
    let point_at = |radius: f32, angle: f32| {
        point(
            px(12. + radius * angle.cos()),
            px(12. + radius * angle.sin()),
        )
    };
    let mut path = PathBuilder::fill();
    let start = arc.start;
    let end = start + arc.sweep;
    path.move_to(point_at(12., start));
    // SVG arcs with coincident endpoints omit a full circle, so split it.
    for i in 1..=2 {
        path.arc_to(
            point(px(12.), px(12.)),
            px(0.),
            false,
            true,
            point_at(12., start + arc.sweep * i as f32 / 2.),
        );
    }
    path.arc_to(
        point(px(1.), px(1.)),
        px(0.),
        false,
        true,
        point_at(10., end),
    );
    for i in 1..=2 {
        path.arc_to(
            point(px(10.), px(10.)),
            px(0.),
            false,
            false,
            point_at(10., end - arc.sweep * i as f32 / 2.),
        );
    }
    path.arc_to(
        point(px(1.), px(1.)),
        px(0.),
        false,
        true,
        point_at(12., start),
    );
    path.close();
    path.transform(Transform::new(
        scale_x,
        0.,
        0.,
        scale_y,
        f32::from(bounds.origin.x),
        f32::from(bounds.origin.y),
    ));
    path.build().ok()
}

/// Paint a sampled circular frame in a caller-sized box, using the reference's
/// 24-unit canvas and 2-unit stroke. Pass a 20×24 rem box for xs/s button slots,
/// or 24×24 for larger buttons. The parent owns the surrounding icon slot.
///
/// `fraction` supplies the measured accessibility value; `None` is indeterminate.
/// The caller supplies the accessible label and opacity (0.7 for loading buttons).
pub fn circular_progress(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    frame: CircularProgressFrame,
    fraction: Option<f32>,
    display_size: Size<Rems>,
    opacity: f32,
) -> AnyElement {
    paint_progress(
        id.into(),
        label.into(),
        frame,
        fraction,
        display_size,
        opacity,
        false,
    )
}

/// Animate a bounded circular indicator while its box intersects the viewport
/// and ancestor clip. Its original clock continues offscreen, so scrolling back
/// does not restart playback. Reduced motion retains the sampler's stop policy.
pub fn indeterminate_circular_progress(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    display_size: Size<Rems>,
    opacity: f32,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = id.into();
    let (frame, running) = sample_clock(id.clone(), window, cx);
    paint_progress(
        id,
        label.into(),
        frame,
        None,
        display_size,
        opacity,
        running,
    )
}

/// Draw with a caller-owned clock, retaining phase across virtual unmounts.
/// Frame requests remain limited to visible paint. The owner must forward
/// offscreen reduced-motion changes to [`CircularProgressPlayback::observe_reduced_motion`].
pub fn indeterminate_circular_progress_with_playback(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    display_size: Size<Rems>,
    opacity: f32,
    playback: &CircularProgressPlayback,
    cx: &App,
) -> AnyElement {
    let (frame, running) = playback.sample_running(cx);
    paint_progress(
        id.into(),
        label.into(),
        frame,
        None,
        display_size,
        opacity,
        running,
    )
}

fn paint_progress(
    id: ElementId,
    label: SharedString,
    frame: CircularProgressFrame,
    fraction: Option<f32>,
    display_size: Size<Rems>,
    opacity: f32,
    request_frame: bool,
) -> AnyElement {
    let (track, active) = geometry(frame);
    let track_color = color(ColorRole::SECONDARY_CONTAINER);
    let active_color = color(ColorRole::PRIMARY);
    Progress::new(id)
        .accessibility_label(label)
        .indeterminate(fraction.is_none())
        .value(normalized_fraction(fraction.unwrap_or(0.)) * 100.)
        .flex_shrink_0()
        .w(display_size.width)
        .h(display_size.height)
        .opacity(opacity.clamp(0., 1.))
        .child(
            canvas(
                move |bounds, window, _| {
                    let visible = bounds
                        .intersect(&window.content_mask().bounds)
                        .intersect(&Bounds::new(point(px(0.), px(0.)), window.viewport_size()));
                    request_frame && visible.size.width > px(0.) && visible.size.height > px(0.)
                },
                move |bounds, request_frame, window, _| {
                    for (arc, ink) in [(track, track_color), (active, active_color)] {
                        if let Some(path) = arc_path(bounds, arc) {
                            window.paint_path(path, ink);
                        }
                    }
                    if request_frame {
                        window.request_animation_frame();
                    }
                },
            )
            .size_full(),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{
        ArcGeometry, CircularProgressFrame, arc_path, circular_indeterminate_frame, geometry,
        normalized_fraction, sample_circular_progress,
    };
    use gpui::{
        Bounds, Context, IntoElement, Render, TestAppContext, Window, div, point, px, size,
    };
    use std::f32::consts::{FRAC_PI_2, PI, TAU};
    use std::time::Duration;

    #[test]
    fn matches_pinned_browser_motion_samples() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../conformance/reference/button-loading.json"
        ))
        .unwrap();
        for item in fixture["modes"]["ordinary"]["motion"].as_array().unwrap() {
            let frame = circular_indeterminate_frame(item["time"].as_f64().unwrap() as f32);
            assert!(
                (f64::from(frame.rotation_degrees) - item["rotation"].as_f64().unwrap()).abs()
                    < 0.002
            );
            assert!((f64::from(frame.sweep) - item["sweep"].as_f64().unwrap()).abs() < 0.00002);
        }
    }

    struct RetainedClockHarness {
        mounted: bool,
        playback: super::CircularProgressPlayback,
    }
    impl Render for RetainedClockHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            use gpui::prelude::*;
            div().size(px(100.)).when(self.mounted, |root| {
                root.child(super::indeterminate_circular_progress_with_playback(
                    "retained-circular",
                    "Loading",
                    size(gpui::rems(1.), gpui::rems(1.)),
                    1.,
                    &self.playback,
                    cx,
                ))
            })
        }
    }

    #[gpui::test]
    fn caller_clock_survives_unmount_and_observes_offscreen_reduced_motion(
        cx: &mut TestAppContext,
    ) {
        let playback = super::CircularProgressPlayback::default();
        let (view, cx) = cx.add_window_view(|_, _| RetainedClockHarness {
            mounted: false,
            playback: playback.clone(),
        });
        cx.executor().advance_clock(Duration::from_millis(200));
        cx.update(|window, cx| {
            cx.set_reduce_motion(true);
            playback.observe_reduced_motion(cx);
            assert!(
                playback.state.borrow().is_none(),
                "preference events must not start an unused clock"
            );
            cx.set_reduce_motion(false);
            view.update(cx, |view, _| view.mounted = true);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            playback.state.borrow().as_ref().unwrap().last_frame,
            circular_indeterminate_frame(0.)
        );
        cx.executor().advance_clock(Duration::from_millis(150));
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.mounted = false);
            window.refresh();
            window.draw(cx).clear(cx);
            // Drain the one previously scheduled visible-frame callback.
            window.simulate_next_frame(cx);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(window.simulate_next_frame(cx), 0);
        });
        cx.executor().advance_clock(Duration::from_millis(350));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(window.simulate_next_frame(cx), 0);
            view.update(cx, |view, _| view.mounted = true);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let frame = playback.state.borrow().as_ref().unwrap().last_frame;
        let expected = circular_indeterminate_frame(500.);
        assert!((frame.rotation_degrees - expected.rotation_degrees).abs() < 0.002);
        assert!((frame.sweep - expected.sweep).abs() < 0.00002);
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.mounted = false);
            window.refresh();
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.executor().advance_clock(Duration::from_millis(200));
        cx.update(|window, cx| {
            cx.set_reduce_motion(true);
            playback.observe_reduced_motion(cx);
            assert_eq!(
                window.simulate_next_frame(cx),
                0,
                "preference observation does not schedule"
            );
            cx.set_reduce_motion(false);
            playback.observe_reduced_motion(cx);
        });
        let stopped = playback.state.borrow().as_ref().unwrap().last_frame;
        assert!(
            (stopped.rotation_degrees - circular_indeterminate_frame(700.).rotation_degrees).abs()
                < 0.002
        );
        assert!(playback.state.borrow().as_ref().unwrap().stopped);
        cx.executor().advance_clock(Duration::from_secs(2));
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.mounted = true);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(window.simulate_next_frame(cx), 0);
        });
        assert_eq!(
            playback.state.borrow().as_ref().unwrap().last_frame,
            stopped
        );
    }

    #[gpui::test]
    fn caller_clock_initial_reduced_mount_stays_stopped_and_new_handle_restarts(
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let playback = super::CircularProgressPlayback::default();
        let (view, cx) = cx.add_window_view(|_, _| RetainedClockHarness {
            mounted: true,
            playback: playback.clone(),
        });
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.update(|window, cx| {
            cx.set_reduce_motion(false);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(window.simulate_next_frame(cx), 0);
        });
        assert_eq!(
            playback.state.borrow().as_ref().unwrap().last_frame,
            circular_indeterminate_frame(0.)
        );
        let fresh = super::CircularProgressPlayback::default();
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.playback = fresh.clone());
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.executor().advance_clock(Duration::from_millis(150));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert!(!fresh.state.borrow().as_ref().unwrap().stopped);
        assert!(
            (fresh
                .state
                .borrow()
                .as_ref()
                .unwrap()
                .last_frame
                .rotation_degrees
                - circular_indeterminate_frame(150.).rotation_degrees)
                .abs()
                < 0.002
        );
        assert!(playback.state.borrow().as_ref().unwrap().stopped);
    }

    struct ClockHarness {
        visible: bool,
        samples: std::rc::Rc<std::cell::RefCell<Vec<CircularProgressFrame>>>,
    }

    impl Render for ClockHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.samples.borrow_mut().push(sample_circular_progress(
                "clock",
                self.visible,
                window,
                cx,
            ));
            div()
        }
    }

    #[gpui::test]
    fn hidden_samples_advance_without_requesting_frames(cx: &mut TestAppContext) {
        let samples = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let (view, cx) = cx.add_window_view(|_, _| ClockHarness {
            visible: true,
            samples: samples.clone(),
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            // Backends differ in how many requests one visible draw queues
            // (upstream also schedules its own frame); hidden must be zero.
            assert!(window.simulate_next_frame(cx) >= 1);
            view.update(cx, |view, _| view.visible = false);
            window.refresh();
            window.draw(cx).clear(cx);
            // Drain the request queued by the last visible sample.
            window.simulate_next_frame(cx);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(window.simulate_next_frame(cx), 0);
        });
        cx.executor().advance_clock(Duration::from_millis(150));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(window.simulate_next_frame(cx), 0);
        });
        let sampled = *samples.borrow().last().unwrap();
        assert!(
            (sampled.rotation_degrees - circular_indeterminate_frame(150.).rotation_degrees).abs()
                < 0.002
        );
    }

    #[gpui::test]
    fn clock_keeps_hidden_phase_and_stops_without_restarting_on_media_change(
        cx: &mut TestAppContext,
    ) {
        let samples = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let (_, cx) = cx.add_window_view(|_, _| ClockHarness {
            visible: false,
            samples: samples.clone(),
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.executor().advance_clock(Duration::from_millis(150));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let sampled = *samples.borrow().last().unwrap();
        assert!(
            (sampled.rotation_degrees - circular_indeterminate_frame(150.).rotation_degrees).abs()
                < 0.002
        );
        // No paint element was mounted: the sampled clock still advances.
        cx.update(|window, cx| {
            cx.set_reduce_motion(true);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let stopped = *samples.borrow().last().unwrap();
        cx.executor().advance_clock(Duration::from_millis(800));
        cx.update(|window, cx| {
            cx.set_reduce_motion(false);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(*samples.borrow().last().unwrap(), stopped);
    }

    struct VisibleClockHarness {
        visible: bool,
        samples: std::rc::Rc<std::cell::RefCell<Vec<CircularProgressFrame>>>,
    }
    impl Render for VisibleClockHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            use gpui::prelude::*;
            let indicator = super::indeterminate_circular_progress(
                "visible-clock",
                "Loading",
                size(gpui::rems(1.), gpui::rems(1.)),
                1.,
                window,
                cx,
            );
            self.samples
                .borrow_mut()
                .push(super::sample_clock("visible-clock".into(), window, cx).0);
            div().relative().size(px(100.)).overflow_hidden().child(
                div()
                    .absolute()
                    .top(px(if self.visible { 10. } else { 150. }))
                    .child(indicator),
            )
        }
    }

    #[gpui::test]
    fn clipped_indicator_stops_requests_preserves_phase_and_reduced_stop(cx: &mut TestAppContext) {
        let samples = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let (view, cx) = cx.add_window_view(|_, _| VisibleClockHarness {
            visible: false,
            samples: samples.clone(),
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert_eq!(window.simulate_next_frame(cx), 0);
        });
        cx.executor().advance_clock(Duration::from_millis(150));
        cx.update(|window, cx| {
            view.update(cx, |this, _| this.visible = true);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(window.simulate_next_frame(cx), 1);
        });
        let frame = *samples.borrow().last().unwrap();
        assert!(
            (frame.rotation_degrees - circular_indeterminate_frame(150.).rotation_degrees).abs()
                < 0.002
        );
        cx.update(|window, cx| {
            view.update(cx, |this, _| this.visible = false);
            cx.set_reduce_motion(true);
            window.refresh();
            window.draw(cx).clear(cx);
            // The prior visible frame already queued one callback. It may fire
            // once after clipping; the stopped clock must not enqueue another.
            assert_eq!(window.simulate_next_frame(cx), 1);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(window.simulate_next_frame(cx), 0);
        });
        let stopped = *samples.borrow().last().unwrap();
        cx.executor().advance_clock(Duration::from_secs(2));
        cx.update(|window, cx| {
            view.update(cx, |this, _| this.visible = true);
            cx.set_reduce_motion(false);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(window.simulate_next_frame(cx), 0);
        });
        assert_eq!(*samples.borrow().last().unwrap(), stopped);
    }

    #[test]
    fn measured_accessibility_and_paint_normalize_nonfinite_values_together() {
        for (input, expected) in [
            (f32::NAN, 0.),
            (f32::INFINITY, 0.),
            (f32::NEG_INFINITY, 0.),
            (-1., 0.),
            (2., 1.),
            (0.5, 0.5),
        ] {
            assert_eq!(normalized_fraction(input), expected);
            assert_eq!(CircularProgressFrame::determinate(input).sweep, expected);
        }
    }

    #[test]
    fn track_clears_round_caps_and_measured_arc_starts_at_twelve() {
        let (track, active) = geometry(circular_indeterminate_frame(0.));
        assert_eq!(active.start, 0.);
        assert!((active.sweep - TAU * 0.1).abs() < 0.00001);
        assert!((track.start - active.sweep - 0.5).abs() < 0.00001);
        assert!((track.sweep + active.sweep + 1. - TAU).abs() < 0.00001);
        assert_eq!(geometry(CircularProgressFrame::determinate(1.)).0.sweep, 0.);
        assert_eq!(geometry(CircularProgressFrame::determinate(0.)).1.sweep, 0.);
        assert!(
            (geometry(CircularProgressFrame::determinate(0.5)).1.start + FRAC_PI_2).abs() < 0.00001
        );
        for width in [20., 24.] {
            assert!(
                arc_path(
                    Bounds::new(point(px(0.), px(0.)), size(px(width), px(24.))),
                    active
                )
                .is_some()
            );
            assert!(
                arc_path(
                    Bounds::new(point(px(0.), px(0.)), size(px(width), px(24.))),
                    ArcGeometry {
                        start: -FRAC_PI_2,
                        sweep: 2. * PI
                    }
                )
                .is_some()
            );
        }
    }
}
