//! M3 progress indicators: linear (measured and indeterminate) and a
//! spinner. The M3 Expressive wavy and contained variants are planned.
mod circular;
pub use circular::{
    CircularProgressFrame, CircularProgressPlayback, circular_indeterminate_frame,
    circular_progress, indeterminate_circular_progress,
    indeterminate_circular_progress_with_playback, sample_circular_progress,
};

use crate::color::ColorRole;
use crate::icons::{Icon, IconName};
use crate::theme::color;
use gpui::{prelude::*, *};
use gpui_base::{Progress, h_flex};
use std::time::Duration;

const SPINNER_TURN_MS: u64 = 1000;
const INDETERMINATE_SWEEP_MS: u64 = 1400;

/// A small accent spinner (16px).
pub fn spinner(id: impl Into<ElementId>, cx: &App) -> AnyElement {
    spin(IconName::LoaderCircle.icon().size(px(16.)), id, None, cx)
}

/// A spinner standing in an icon column, at that column's icon size.
pub fn icon_spinner(id: impl Into<ElementId>, size: Rems, cx: &App) -> AnyElement {
    spin(IconName::LoaderCircle.icon().size(size), id, None, cx)
}

/// A spinner whose caller-owned clock survives virtual unmounting.
///
/// Reduced motion displays a static icon without starting an unused clock.
/// Once started, time continues during unmounting and reduced motion; disabling
/// reduced motion resumes the current phase. Use a new handle to restart.
pub fn spinner_with_playback(
    id: impl Into<ElementId>,
    playback: RepeatAnimationPlayback,
    cx: &App,
) -> AnyElement {
    spin(
        IconName::LoaderCircle.icon().size(px(16.)),
        id,
        Some(playback),
        cx,
    )
}

/// Icon-sized variant of [`spinner_with_playback`] with the same clock policy.
pub fn icon_spinner_with_playback(
    id: impl Into<ElementId>,
    size: Rems,
    playback: RepeatAnimationPlayback,
    cx: &App,
) -> AnyElement {
    spin(
        IconName::LoaderCircle.icon().size(size),
        id,
        Some(playback),
        cx,
    )
}

fn spin(
    icon: Icon,
    id: impl Into<ElementId>,
    playback: Option<RepeatAnimationPlayback>,
    cx: &App,
) -> AnyElement {
    let icon = icon.color(color(ColorRole::PRIMARY));
    if cx.reduce_motion() {
        return icon.into_any_element();
    }
    // `Icon` is not `Styled`, so the animation wraps a slot that rebuilds
    // the rotated glyph each frame.
    div()
        .flex_shrink_0()
        .with_animation(
            id,
            Animation::new(Duration::from_millis(SPINNER_TURN_MS)).repeat(),
            move |slot, t| {
                #[cfg(test)]
                tests::record_repeat_phase(t);
                slot.child(icon.clone().rotate(std::f32::consts::TAU * t))
            },
        )
        .when_some(playback, |animation, playback| {
            animation.with_repeat_playback(playback)
        })
        .repeat_when_visible()
        .into_any_element()
}

/// Unmeasured work uses motion, never a fabricated completion percentage.
///
/// Exposes a progress-indicator role without a name; use
/// [`linear_progress`] to supply the caller's accessible label.
pub fn indeterminate_progress(id: impl Into<ElementId>, cx: &App) -> AnyElement {
    indeterminate_bar(id.into(), None, px(4.), false, None, cx)
}

/// Indeterminate progress with a caller-owned clock; see
/// [`spinner_with_playback`] for unmounting and reduced-motion semantics.
pub fn indeterminate_progress_with_playback(
    id: impl Into<ElementId>,
    playback: RepeatAnimationPlayback,
    cx: &App,
) -> AnyElement {
    indeterminate_bar(id.into(), None, px(4.), false, Some(playback), cx)
}

/// Indeterminate progress with custom thickness/color and a persistent clock.
/// This does not retain the separate transition used by measured progress.
/// The caller-supplied `label` names the indicator for assistive technology.
pub fn indeterminate_linear_progress_with_playback(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    thickness: Pixels,
    muted: bool,
    playback: RepeatAnimationPlayback,
    cx: &App,
) -> AnyElement {
    indeterminate_bar(
        id.into(),
        Some(label.into()),
        thickness,
        muted,
        Some(playback),
        cx,
    )
}

/// Base's progress-indicator semantics: 0–100 range, a value only when
/// measured, and an optional caller-supplied accessible name.
fn semantics(id: ElementId, label: Option<SharedString>, fraction: Option<f32>) -> Progress {
    Progress::new(id)
        .when_some(label, |progress, label| progress.accessibility_label(label))
        .indeterminate(fraction.is_none())
        .value(fraction.unwrap_or(0.) * 100.)
        .w_full()
}

/// A visible bar segment. Fixed-fraction segments may shrink (never below
/// zero) so the fixed 4px gaps can never push the bar beyond its track.
fn segment(width: f32, fill: Rgba) -> Div {
    div()
        .flex_shrink_1()
        .min_w_0()
        .h_full()
        .w(relative(width))
        .rounded_full()
        .bg(fill)
}

/// The indeterminate segments for one phase: leading track, active indicator
/// and trailing track, each separated by the 4px gap. `start` may be
/// negative or exceed 0.7 while the indicator enters or leaves the track.
fn indeterminate_pieces(bar: Div, start: f32, ink: Rgba) -> Div {
    let end = (start + 0.3).min(1.);
    let start = start.max(0.);
    let track = color(ColorRole::SECONDARY_CONTAINER);
    bar.when(start > 0.005, |bar| {
        bar.child(
            segment(start, track)
                .debug_selector(|| "material-progress-indeterminate-leading".into()),
        )
    })
    .when(end - start > 0.005, |bar| {
        bar.child(
            segment(end - start, ink)
                .debug_selector(|| "material-progress-indeterminate-active".into()),
        )
    })
    .when(end < 0.995, |bar| {
        bar.child(
            div()
                .flex_1()
                .min_w_0()
                .h_full()
                .rounded_full()
                .bg(track)
                .debug_selector(|| "material-progress-indeterminate-trailing".into()),
        )
    })
}

fn bar_row(thickness: Pixels) -> Div {
    h_flex().w_full().h(thickness).gap(px(4.)).items_center()
}

/// One active segment sweeping across the track, kept 4px apart from the
/// track on both sides as in Material 3 linear progress.
fn indeterminate_bar(
    id: ElementId,
    label: Option<SharedString>,
    thickness: Pixels,
    muted: bool,
    playback: Option<RepeatAnimationPlayback>,
    cx: &App,
) -> AnyElement {
    let ink = color(if muted {
        ColorRole::OUTLINE
    } else {
        ColorRole::PRIMARY
    });
    let root = semantics(id.clone(), label, None);
    if cx.reduce_motion() {
        return root
            .child(indeterminate_pieces(bar_row(thickness), 0.35, ink))
            .into_any_element();
    }
    root.child(
        bar_row(thickness)
            .with_animation(
                id,
                Animation::new(Duration::from_millis(INDETERMINATE_SWEEP_MS)).repeat(),
                move |bar, t| {
                    #[cfg(test)]
                    tests::record_repeat_phase(t);
                    indeterminate_pieces(bar, t * 1.3 - 0.3, ink)
                },
            )
            .when_some(playback, |animation, playback| {
                animation.with_repeat_playback(playback)
            })
            .repeat_when_visible(),
    )
    .into_any_element()
}

/// A measured fraction as Material 3 linear progress, 4px thick.
///
/// Exposes a progress-indicator role and value without a name; use
/// [`linear_progress`] to supply the caller's accessible label.
pub fn progress(
    id: impl Into<ElementId>,
    progress: f32,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    measured_bar(id.into(), None, progress, px(4.), false, window, cx)
}

/// Material 3 linear progress: the active indicator, a 4px gap, the track
/// and a stop dot at the track's end. Only a measured fraction draws it;
/// `None` shows the indeterminate indicator at the same thickness. A paused
/// or stopping process (`muted`) keeps its fraction in the outline color.
/// The caller-supplied `label` names the indicator for assistive technology.
pub fn linear_progress(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    fraction: Option<f32>,
    thickness: Pixels,
    muted: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let (id, label) = (id.into(), Some(label.into()));
    match fraction {
        Some(fraction) => measured_bar(id, label, fraction, thickness, muted, window, cx),
        None => indeterminate_bar(id, label, thickness, muted, None, cx),
    }
}

fn measured_bar(
    id: ElementId,
    label: Option<SharedString>,
    fraction: f32,
    thickness: Pixels,
    muted: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let target = if fraction.is_finite() {
        fraction.clamp(0., 1.)
    } else {
        0.
    };
    // Spatial overshoot must not extend a measured fraction outside its track.
    let amount = crate::motion::value(id.clone(), target, window, cx).clamp(0., 1.);
    let ink = color(if muted {
        ColorRole::OUTLINE
    } else {
        ColorRole::PRIMARY
    });
    semantics(id, label, Some(target))
        .child(measured_pieces(bar_row(thickness), amount, thickness, ink))
        .into_any_element()
}

fn measured_pieces(bar: Div, amount: f32, thickness: Pixels, ink: Rgba) -> Div {
    let dot = (thickness / 2.).max(px(4.)).min(thickness);
    bar.when(amount > 0.005, |bar| {
        bar.child(segment(amount, ink).debug_selector(|| "material-progress-active".into()))
    })
    .when(amount < 0.995, |bar| {
        bar.child(
            div()
                .relative()
                .flex_1()
                .min_w_0()
                .h_full()
                .rounded_full()
                .bg(color(ColorRole::SECONDARY_CONTAINER))
                .debug_selector(|| "material-progress-track".into())
                .child(
                    div()
                        .absolute()
                        .top((thickness - dot) / 2.)
                        .right((thickness - dot) / 2.)
                        .size(dot)
                        .rounded_full()
                        .bg(ink),
                ),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::progress;
    use gpui::{Context, IntoElement, Render, TestAppContext, Window, div, prelude::*, px};
    use std::time::Duration;

    thread_local! {
        static REPEAT_PHASES: std::cell::RefCell<Vec<f32>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    pub(super) fn record_repeat_phase(phase: f32) {
        REPEAT_PHASES.with_borrow_mut(|phases| phases.push(phase));
    }

    struct RepeatHarness {
        kind: usize,
        mounted: bool,
        playback: gpui::RepeatAnimationPlayback,
    }

    impl Render for RepeatHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let mut root = div().w(px(200.)).h(px(40.));
            if self.mounted {
                let playback = self.playback.clone();
                root = root.child(match self.kind {
                    0 => super::spinner_with_playback("repeat", playback, cx),
                    1 => super::icon_spinner_with_playback("repeat", gpui::rems(1.), playback, cx),
                    2 => super::indeterminate_progress_with_playback("repeat", playback, cx),
                    _ => super::indeterminate_linear_progress_with_playback(
                        "repeat",
                        "Loading",
                        px(6.),
                        true,
                        playback,
                        cx,
                    ),
                });
            }
            root
        }
    }

    #[gpui::test]
    fn explicit_repeat_clocks_survive_unmount_and_reduced_motion(cx: &mut TestAppContext) {
        for kind in 0..4 {
            let (view, cx) = cx.add_window_view(|_, _| RepeatHarness {
                kind,
                mounted: false,
                playback: gpui::RepeatAnimationPlayback::default(),
            });
            REPEAT_PHASES.with_borrow_mut(Vec::clear);
            // Neither unmounted nor initially reduced indicators start playback.
            cx.executor().advance_clock(Duration::from_secs(2));
            cx.update(|window, cx| {
                cx.set_reduce_motion(true);
                view.update(cx, |view, _| view.mounted = true);
                window.refresh();
                window.draw(cx).clear(cx);
            });
            assert!(REPEAT_PHASES.with_borrow(|phases| phases.is_empty()));
            cx.executor().advance_clock(Duration::from_secs(1));
            cx.update(|window, cx| {
                cx.set_reduce_motion(false);
                window.refresh();
                window.draw(cx).clear(cx);
            });
            assert_eq!(
                REPEAT_PHASES.with_borrow(|phases| *phases.last().unwrap()),
                0.
            );
            cx.update(|window, cx| {
                view.update(cx, |view, _| view.mounted = false);
                window.refresh();
                window.draw(cx).clear(cx);
                window.refresh();
                window.draw(cx).clear(cx);
            });
            let count = REPEAT_PHASES.with_borrow(Vec::len);
            cx.executor().advance_clock(Duration::from_millis(350));
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
            assert_eq!(REPEAT_PHASES.with_borrow(Vec::len), count);
            cx.update(|window, cx| {
                view.update(cx, |view, _| view.mounted = true);
                window.refresh();
                window.draw(cx).clear(cx);
            });
            let duration = if kind < 2 { 1000. } else { 1400. };
            let phase = REPEAT_PHASES.with_borrow(|phases| *phases.last().unwrap());
            assert!(
                (phase - 350. / duration).abs() < 0.001,
                "kind {kind}: {phase}"
            );
            cx.update(|window, cx| {
                cx.set_reduce_motion(true);
                window.refresh();
                window.draw(cx).clear(cx);
            });
            let count = REPEAT_PHASES.with_borrow(Vec::len);
            cx.executor().advance_clock(Duration::from_millis(350));
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
            assert_eq!(REPEAT_PHASES.with_borrow(Vec::len), count);
            cx.update(|window, cx| {
                cx.set_reduce_motion(false);
                window.refresh();
                window.draw(cx).clear(cx);
            });
            let phase = REPEAT_PHASES.with_borrow(|phases| *phases.last().unwrap());
            assert!(
                (phase - 700. / duration).abs() < 0.001,
                "kind {kind}: {phase}"
            );
            // Leave no mounted animation from this case for the next case.
            cx.update(|window, cx| {
                view.update(cx, |view, _| view.mounted = false);
                window.refresh();
                window.draw(cx).clear(cx);
            });
        }
    }

    struct ProgressHarness {
        target: f32,
    }
    impl Render for ProgressHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            crate::motion::set_motion_scheme(crate::motion::MotionScheme::Expressive);
            div()
                .w(px(100.))
                .child(progress("bounded-progress", self.target, window, cx))
        }
    }

    #[gpui::test]
    fn expressive_overshoot_never_paints_beyond_the_progress_track(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| ProgressHarness { target: 0. });
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.target = 1.);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        // At 60%, the reference default spatial curve is above its endpoint.
        cx.executor().advance_clock(Duration::from_millis(270));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            cx.debug_bounds("material-progress-active")
                .unwrap()
                .size
                .width,
            px(100.)
        );
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.target = 0.);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.executor().advance_clock(Duration::from_millis(270));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert!(
            cx.debug_bounds("material-progress-active").is_none(),
            "negative overshoot must not create a negative-width indicator"
        );
    }
}

#[cfg(test)]
mod layout_tests {
    use gpui::{
        Bounds, Context, IntoElement, Pixels, Render, TestAppContext, VisualTestContext, Window,
        accesskit, div, prelude::*, px,
    };

    const TRACK: f32 = 300.;

    enum Case {
        Measured(f32),
        Indeterminate(f32),
    }
    struct LayoutHarness {
        case: Case,
    }
    impl Render for LayoutHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let ink = crate::theme::color(crate::color::ColorRole::PRIMARY);
            let bar = super::bar_row(px(4.));
            let bar = match self.case {
                Case::Measured(amount) => super::measured_pieces(bar, amount, px(4.), ink),
                Case::Indeterminate(start) => super::indeterminate_pieces(bar, start, ink),
            };
            div()
                .w(px(TRACK))
                .child(bar.debug_selector(|| "row".into()))
        }
    }

    fn bounds(cx: &mut VisualTestContext, names: &[&'static str]) -> Vec<Bounds<Pixels>> {
        names
            .iter()
            .filter_map(|name| cx.debug_bounds(name))
            .collect()
    }

    fn assert_within_track(cx: &mut VisualTestContext, names: &[&'static str], label: &str) {
        let row = cx.debug_bounds("row").unwrap();
        assert_eq!(row.size.width, px(TRACK));
        let pieces = bounds(cx, names);
        assert!(!pieces.is_empty(), "{label}");
        let used: f32 = pieces.iter().map(|b| f32::from(b.size.width)).sum::<f32>()
            + 4. * (pieces.len() - 1) as f32;
        assert!(used <= TRACK + 0.01, "{label}: {used}");
        for piece in pieces {
            assert!(piece.size.width >= px(0.), "{label}");
            assert!(piece.right() <= row.right() + px(0.01), "{label}");
            assert!(piece.left() >= row.left() - px(0.01), "{label}");
        }
    }

    #[gpui::test]
    fn linear_segments_and_gaps_never_exceed_the_track(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| LayoutHarness {
            case: Case::Measured(0.),
        });
        let draw = |cx: &mut VisualTestContext| {
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
        };
        let measured = ["material-progress-active", "material-progress-track"];
        for amount in [0., 0.004, 0.006, 0.5, 0.98, 0.99, 0.994, 0.996, 1.] {
            view.update(cx, |view, cx| {
                view.case = Case::Measured(amount);
                cx.notify();
            });
            draw(cx);
            assert_within_track(cx, &measured, &format!("measured {amount}"));
        }
        // A measured fraction keeps its nominal width while the gap fits.
        view.update(cx, |view, cx| {
            view.case = Case::Measured(0.5);
            cx.notify();
        });
        draw(cx);
        assert_eq!(
            cx.debug_bounds("material-progress-active")
                .unwrap()
                .size
                .width,
            px(150.)
        );
        let indeterminate = [
            "material-progress-indeterminate-leading",
            "material-progress-indeterminate-active",
            "material-progress-indeterminate-trailing",
        ];
        // Entering, middle, end clamped to 1, and leaving the track.
        for start in [-0.3, -0.29, 0., 0.35, 0.68, 0.69, 0.7, 0.75, 0.99] {
            view.update(cx, |view, cx| {
                view.case = Case::Indeterminate(start);
                cx.notify();
            });
            draw(cx);
            assert_within_track(cx, &indeterminate, &format!("indeterminate {start}"));
        }
    }

    struct A11yHarness {
        nodes: std::rc::Rc<std::cell::RefCell<Vec<accesskit::Node>>>,
    }
    impl Render for A11yHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            for (label, fraction) in [("Upload", Some(0.25)), ("Sync", None)] {
                let mut node = accesskit::Node::new(accesskit::Role::GenericContainer);
                super::semantics(label.into(), Some(label.into()), fraction)
                    .render(window, cx)
                    .into_element()
                    .write_a11y_info(&mut node);
                self.nodes.borrow_mut().push(node);
            }
            div()
                .w(px(200.))
                .child(super::linear_progress(
                    "measured",
                    "Upload",
                    Some(0.25),
                    px(4.),
                    false,
                    window,
                    cx,
                ))
                .child(super::linear_progress(
                    "unmeasured",
                    "Sync",
                    None,
                    px(4.),
                    false,
                    window,
                    cx,
                ))
        }
    }

    #[gpui::test]
    fn linear_progress_exposes_role_range_value_and_caller_label(cx: &mut TestAppContext) {
        let nodes = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let (_, cx) = cx.add_window_view({
            let nodes = nodes.clone();
            move |_, _| A11yHarness { nodes }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let nodes = nodes.borrow();
        let (measured, unmeasured) = (&nodes[0], &nodes[1]);
        assert_eq!(measured.label(), Some("Upload"));
        assert_eq!(measured.numeric_value(), Some(25.));
        assert_eq!(measured.min_numeric_value(), Some(0.));
        assert_eq!(measured.max_numeric_value(), Some(100.));
        assert_eq!(unmeasured.label(), Some("Sync"));
        assert_eq!(unmeasured.numeric_value(), None);
    }
}
