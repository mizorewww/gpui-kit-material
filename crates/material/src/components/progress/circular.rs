//! Circular progress from the pinned reference's canvas and motion functions.
use crate::{color::ColorRole, theme::color};
use gpui::{prelude::*, *};
use gpui_base::{Easing, IterationCount, Progress, Timing};
use std::{f32::consts::TAU, time::Duration};

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

/// Sample independently of painting so a hidden button indicator keeps its phase.
///
/// Keep calling this after first showing the indicator, including while hidden.
/// The reference polls reduced motion in its frame callback: a running loop draws
/// one last frame then stops. Changing back does not restart it. A new ID starts
/// a fresh playback, as explicitly restarting the reference's indicator does.
pub fn sample_circular_progress(
    id: impl Into<ElementId>,
    window: &mut Window,
    cx: &mut App,
) -> CircularProgressFrame {
    let now = cx.background_executor().now();
    let reduced = cx.reduce_motion();
    let state = window.use_keyed_state((id.into(), "circular-progress-clock"), cx, |_, _| {
        (now, reduced, circular_indeterminate_frame(0.))
    });
    let (started, stopped, previous) = *state.read(cx);
    if stopped {
        return previous;
    }
    let timing = Timing::new(Duration::from_millis(6000)).iterations(IterationCount::Infinite);
    let time = timing
        .sample(now.saturating_duration_since(started))
        .directed_progress
        * 6000.;
    let frame = circular_indeterminate_frame(time);
    state.update(cx, |state, _| {
        state.1 = reduced;
        state.2 = frame;
    });
    if !reduced {
        window.request_animation_frame();
    }
    frame
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
    let (track, active) = geometry(frame);
    let track_color = color(ColorRole::SECONDARY_CONTAINER);
    let active_color = color(ColorRole::ACCENT);
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
                move |_, _, _| (),
                move |bounds, (), window, _| {
                    for (arc, ink) in [(track, track_color), (active, active_color)] {
                        if let Some(path) = arc_path(bounds, arc) {
                            window.paint_path(path, ink);
                        }
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

    struct ClockHarness {
        samples: std::rc::Rc<std::cell::RefCell<Vec<CircularProgressFrame>>>,
    }

    impl Render for ClockHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.samples
                .borrow_mut()
                .push(sample_circular_progress("clock", window, cx));
            div()
        }
    }

    #[gpui::test]
    fn clock_keeps_hidden_phase_and_stops_without_restarting_on_media_change(
        cx: &mut TestAppContext,
    ) {
        let samples = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let (_, cx) = cx.add_window_view(|_, _| ClockHarness {
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
