//! M3 progress indicators: linear and circular, flat or M3 Expressive
//! wavy, 4dp or thick, measured or indeterminate, plus icon-sized spinners.
//! [`LinearProgress`] and [`CircularProgress`] are the full components; the
//! free functions are shorthands kept for existing callers.
mod circular;
mod indicator;
mod linear;
mod paint;
pub use circular::{
    CircularProgressFrame, CircularProgressPlayback, circular_indeterminate_frame,
    circular_progress, indeterminate_circular_progress,
    indeterminate_circular_progress_with_playback, sample_circular_progress,
};
pub use indicator::{CircularProgress, circular_progress_size};
pub use linear::{
    LINEAR_INDETERMINATE_MS, LinearProgress, PROGRESS_THICK_THICKNESS, PROGRESS_THICKNESS,
    ProgressShape, linear_indeterminate_lines,
};

use crate::color::ColorRole;
use crate::icons::{Icon, IconName};
use crate::theme::color;
use gpui::{prelude::*, *};
use gpui_base::Progress;
use std::time::Duration;

const SPINNER_TURN_MS: u64 = 1000;

/// A small accent spinner (16px), announced as indeterminate progress
/// named by the caller's `label`.
pub fn spinner(id: impl Into<ElementId>, label: impl Into<SharedString>, cx: &App) -> AnyElement {
    spin(
        IconName::LoaderCircle.icon().size(px(16.)),
        id,
        label,
        None,
        cx,
    )
}

/// A spinner standing in an icon column, at that column's icon size.
pub fn icon_spinner(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    size: Rems,
    cx: &App,
) -> AnyElement {
    spin(
        IconName::LoaderCircle.icon().size(size),
        id,
        label,
        None,
        cx,
    )
}

/// A spinner whose caller-owned clock survives virtual unmounting.
///
/// Reduced motion displays a static icon without starting an unused clock.
/// Once started, time continues during unmounting and reduced motion; disabling
/// reduced motion resumes the current phase. Use a new handle to restart.
pub fn spinner_with_playback(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    playback: RepeatAnimationPlayback,
    cx: &App,
) -> AnyElement {
    spin(
        IconName::LoaderCircle.icon().size(px(16.)),
        id,
        label,
        Some(playback),
        cx,
    )
}

/// Icon-sized variant of [`spinner_with_playback`] with the same clock policy.
pub fn icon_spinner_with_playback(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    size: Rems,
    playback: RepeatAnimationPlayback,
    cx: &App,
) -> AnyElement {
    spin(
        IconName::LoaderCircle.icon().size(size),
        id,
        label,
        Some(playback),
        cx,
    )
}

fn spin(
    icon: Icon,
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    playback: Option<RepeatAnimationPlayback>,
    cx: &App,
) -> AnyElement {
    let id = id.into();
    let icon = icon.color(color(ColorRole::PRIMARY));
    // The spinner is decorative motion; the progress node carries its meaning.
    let root = Progress::new(id.clone())
        .accessibility_label(label)
        .indeterminate(true)
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center();
    if cx.reduce_motion() {
        return root.child(icon).into_any_element();
    }
    // `Icon` is not `Styled`, so the animation wraps a slot that rebuilds
    // the rotated glyph each frame.
    root.child(
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
            .repeat_when_visible(),
    )
    .into_any_element()
}

/// Unmeasured work uses motion, never a fabricated completion percentage.
///
/// Exposes a progress-indicator role without a name; prefer
/// [`indeterminate_progress_with_label`], which names it.
pub fn indeterminate_progress(id: impl Into<ElementId>, _cx: &App) -> AnyElement {
    LinearProgress::unlabelled(id).into_any_element()
}

/// [`indeterminate_progress`] named by the caller's `label` for assistive
/// technology.
pub fn indeterminate_progress_with_label(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    _cx: &App,
) -> AnyElement {
    LinearProgress::new(id, label).into_any_element()
}

/// Indeterminate progress with a caller-owned clock; see
/// [`spinner_with_playback`] for unmounting and reduced-motion semantics.
pub fn indeterminate_progress_with_playback(
    id: impl Into<ElementId>,
    playback: RepeatAnimationPlayback,
    _cx: &App,
) -> AnyElement {
    LinearProgress::unlabelled(id)
        .playback(playback)
        .into_any_element()
}

/// Indeterminate progress with custom thickness/color and a persistent clock.
/// The caller-supplied `label` names the indicator for assistive technology.
pub fn indeterminate_linear_progress_with_playback(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    thickness: Pixels,
    muted: bool,
    playback: RepeatAnimationPlayback,
    _cx: &App,
) -> AnyElement {
    LinearProgress::new(id, label)
        .thickness(thickness)
        .muted(muted)
        .playback(playback)
        .into_any_element()
}

/// A measured fraction as Material 3 linear progress, 4px thick.
///
/// Exposes a progress-indicator role and value without a name; prefer
/// [`progress_with_label`], which names it.
pub fn progress(
    id: impl Into<ElementId>,
    progress: f32,
    _window: &mut Window,
    _cx: &mut App,
) -> AnyElement {
    LinearProgress::unlabelled(id)
        .value(Some(progress))
        .into_any_element()
}

/// [`progress`] named by the caller's `label` for assistive technology.
pub fn progress_with_label(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    progress: f32,
    _window: &mut Window,
    _cx: &mut App,
) -> AnyElement {
    LinearProgress::new(id, label)
        .value(Some(progress))
        .into_any_element()
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
    _window: &mut Window,
    _cx: &mut App,
) -> AnyElement {
    LinearProgress::new(id, label)
        .value(fraction)
        .thickness(thickness)
        .muted(muted)
        .into_any_element()
}

#[cfg(test)]
mod tests {
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
                    0 => super::spinner_with_playback("repeat", "Loading", playback, cx),
                    1 => super::icon_spinner_with_playback(
                        "repeat",
                        "Loading",
                        gpui::rems(1.),
                        playback,
                        cx,
                    ),
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

    /// One wavy measured indicator, linear or circular.
    struct WavyHarness {
        value: f32,
        circular: bool,
    }

    impl Render for WavyHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let value = Some(self.value);
            div().w(px(200.)).child(if self.circular {
                super::CircularProgress::new("wavy", "Upload")
                    .wavy()
                    .value(value)
                    .into_any_element()
            } else {
                super::LinearProgress::new("wavy", "Upload")
                    .wavy()
                    .value(value)
                    .into_any_element()
            })
        }
    }

    /// Frames requested after drawing a settled wavy indicator at `value`.
    fn frames_after_settling(cx: &mut TestAppContext, value: f32, circular: bool) -> usize {
        let (_, cx) = cx.add_window_view(|_, _| WavyHarness { value, circular });
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            // Let any first-frame motion settle, then draw once more.
            window.simulate_next_frame(cx);
            window.refresh();
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx)
        })
    }

    #[gpui::test]
    fn a_flat_wave_requests_no_frames(cx: &mut TestAppContext) {
        for circular in [false, true] {
            for value in [0.05, 0.1, 0.95, 0.97, 1.] {
                assert_eq!(
                    frames_after_settling(cx, value, circular),
                    0,
                    "circular {circular}, {value}: a flat wave kept the window busy"
                );
            }
            assert!(
                frames_after_settling(cx, 0.5, circular) > 0,
                "a visible wave moves"
            );
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
            let duration = if kind < 2 { 1000. } else { 1750. };
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
}

#[cfg(test)]
mod layout_tests {
    use gpui::{
        Context, ElementId, IntoElement, Render, Role, TestAppContext, Window, div, prelude::*, px,
    };
    use gpui_base::test_support::{ElementSnapshot, find};

    /// Every shorthand and component, as an application renders them.
    struct Rendered;
    impl Render for Rendered {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(px(200.))
                .flex()
                .flex_col()
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
                .child(super::progress_with_label("named", "Copy", 0.5, window, cx))
                .child(super::indeterminate_progress_with_label(
                    "named-busy",
                    "Index",
                    cx,
                ))
                .child(super::progress("legacy", 0.5, window, cx))
                .child(super::CircularProgress::new("ring", "Render").value(Some(0.75)))
        }
    }

    fn node(window: &Window, id: &'static str) -> ElementSnapshot {
        find(window, &[], &ElementId::from(id)).unwrap_or_else(|| panic!("{id} was not rendered"))
    }

    #[gpui::test]
    fn rendered_progress_nodes_carry_role_and_caller_label(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let (_, cx) = cx.add_window_view(|_, _| Rendered);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|window, _| {
            for (id, label) in [
                ("measured", "Upload"),
                ("unmeasured", "Sync"),
                ("named", "Copy"),
                ("named-busy", "Index"),
                ("ring", "Render"),
            ] {
                let node = node(window, id);
                assert_eq!(node.role(), Some(Role::ProgressIndicator), "{id}");
                assert_eq!(node.label(), Some(label), "{id}");
                assert!(node.visible(), "{id}");
            }
            // The 4dp linear indicator fills its row.
            let measured = node(window, "measured").bounds();
            assert_eq!(measured.size.width, px(200.));
            assert_eq!(measured.size.height, px(4.));
            // The legacy shorthand stays unnamed.
            let legacy = node(window, "legacy");
            assert_eq!(legacy.role(), Some(Role::ProgressIndicator));
            assert_eq!(legacy.label(), None);
        });
    }
}
