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
use gpui_base::h_flex;
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
    let icon = icon.color(color(ColorRole::ACCENT));
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
pub fn indeterminate_progress(id: impl Into<ElementId>, cx: &App) -> AnyElement {
    indeterminate_bar(id, px(4.), false, None, cx)
}

/// Indeterminate progress with a caller-owned clock; see
/// [`spinner_with_playback`] for unmounting and reduced-motion semantics.
pub fn indeterminate_progress_with_playback(
    id: impl Into<ElementId>,
    playback: RepeatAnimationPlayback,
    cx: &App,
) -> AnyElement {
    indeterminate_linear_progress_with_playback(id, px(4.), false, playback, cx)
}

/// Indeterminate progress with custom thickness/color and a persistent clock.
/// This does not retain the separate transition used by measured progress.
pub fn indeterminate_linear_progress_with_playback(
    id: impl Into<ElementId>,
    thickness: Pixels,
    muted: bool,
    playback: RepeatAnimationPlayback,
    cx: &App,
) -> AnyElement {
    indeterminate_bar(id, thickness, muted, Some(playback), cx)
}

/// One active segment sweeping across the track, kept 4px apart from the
/// track on both sides as in Material 3 linear progress.
fn indeterminate_bar(
    id: impl Into<ElementId>,
    thickness: Pixels,
    muted: bool,
    playback: Option<RepeatAnimationPlayback>,
    cx: &App,
) -> AnyElement {
    let ink = color(if muted {
        ColorRole::CONTROL
    } else {
        ColorRole::ACCENT
    });
    let pieces = move |bar: gpui::Div, start: f32| {
        let end = (start + 0.3).min(1.);
        let start = start.max(0.);
        let piece = |width: f32, fill| {
            div()
                .flex_shrink_0()
                .h_full()
                .w(relative(width))
                .rounded_full()
                .bg(fill)
        };
        let track = color(ColorRole::SECONDARY_CONTAINER);
        bar.when(start > 0.005, |bar| bar.child(piece(start, track)))
            .when(end - start > 0.005, |bar| {
                bar.child(piece(end - start, ink))
            })
            .when(end < 0.995, |bar| {
                bar.child(div().flex_1().min_w_0().h_full().rounded_full().bg(track))
            })
    };
    let bar = h_flex().w_full().h(thickness).gap(px(4.)).items_center();
    if cx.reduce_motion() {
        return pieces(bar, 0.35).into_any_element();
    }
    bar.with_animation(
        id,
        Animation::new(Duration::from_millis(INDETERMINATE_SWEEP_MS)).repeat(),
        move |bar, t| {
            #[cfg(test)]
            tests::record_repeat_phase(t);
            pieces(bar, t * 1.3 - 0.3)
        },
    )
    .when_some(playback, |animation, playback| {
        animation.with_repeat_playback(playback)
    })
    .repeat_when_visible()
    .into_any_element()
}

/// A measured fraction as Material 3 linear progress, 4px thick.
pub fn progress(
    id: impl Into<ElementId>,
    progress: f32,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    linear_progress(id, Some(progress), px(4.), false, window, cx)
}

/// Material 3 linear progress: the active indicator, a 4px gap, the track
/// and a stop dot at the track's end. Only a measured fraction draws it;
/// `None` shows the indeterminate indicator at the same thickness. A paused
/// or stopping process (`muted`) keeps its fraction in the outline color.
pub fn linear_progress(
    id: impl Into<ElementId>,
    fraction: Option<f32>,
    thickness: Pixels,
    muted: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let Some(fraction) = fraction else {
        return indeterminate_bar(id, thickness, muted, None, cx);
    };
    let target = if fraction.is_finite() {
        fraction.clamp(0., 1.)
    } else {
        0.
    };
    // Spatial overshoot must not extend a measured fraction outside its track.
    let amount = crate::motion::value(id, target, window, cx).clamp(0., 1.);
    let ink = color(if muted {
        ColorRole::CONTROL
    } else {
        ColorRole::ACCENT
    });
    let dot = (thickness / 2.).max(px(4.)).min(thickness);
    h_flex()
        .w_full()
        .h(thickness)
        .gap(px(4.))
        .items_center()
        .when(amount > 0.005, |bar| {
            bar.child(
                div()
                    .flex_shrink_0()
                    .h_full()
                    .w(relative(amount))
                    .debug_selector(|| "material-progress-active".into())
                    .rounded_full()
                    .bg(ink),
            )
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
        .into_any_element()
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
            crate::motion::apply_scheme(crate::motion::MotionScheme::Expressive);
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
