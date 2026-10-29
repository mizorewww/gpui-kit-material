//! M3 progress indicators: linear (measured and indeterminate) and a
//! spinner. The M3 Expressive wavy and contained variants are planned.
mod circular;
pub use circular::{
    CircularProgressFrame, circular_indeterminate_frame, circular_progress,
    sample_circular_progress,
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
    spin(IconName::LoaderCircle.icon().size(px(16.)), id, cx)
}

/// A spinner standing in an icon column, at that column's icon size.
pub fn icon_spinner(id: impl Into<ElementId>, size: Rems, cx: &App) -> AnyElement {
    spin(IconName::LoaderCircle.icon().size(size), id, cx)
}

fn spin(icon: Icon, id: impl Into<ElementId>, cx: &App) -> AnyElement {
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
            move |slot, t| slot.child(icon.clone().rotate(std::f32::consts::TAU * t)),
        )
        .into_any_element()
}

/// Unmeasured work uses motion, never a fabricated completion percentage.
pub fn indeterminate_progress(id: impl Into<ElementId>, cx: &App) -> AnyElement {
    indeterminate_bar(id, px(4.), false, cx)
}

/// One active segment sweeping across the track, kept 4px apart from the
/// track on both sides as in Material 3 linear progress.
fn indeterminate_bar(
    id: impl Into<ElementId>,
    thickness: Pixels,
    muted: bool,
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
        move |bar, t| pieces(bar, t * 1.3 - 0.3),
    )
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
        return indeterminate_bar(id, thickness, muted, cx);
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
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.target = 1.);
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
