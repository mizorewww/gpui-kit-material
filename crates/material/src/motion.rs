//! The motion vocabulary: M3 springs, entrances and retargetable values.
//!
//! Expressive uses md3.io's sampled CSS curves and durations. Standard preserves
//! the original analytic Material springs. Apply the scheme at the start of a
//! root render, like the theme; helpers below resolve it on the UI thread.
use gpui::{prelude::*, *};
use std::cell::Cell;
use std::time::Duration;

pub mod expressive;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MotionScheme {
    Standard,
    #[default]
    Expressive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionToken {
    FastSpatial,
    DefaultSpatial,
    SlowSpatial,
    FastEffects,
    DefaultEffects,
    SlowEffects,
}

/// A resolved transition; use the same curve for timing and interpolation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MotionCurve {
    Standard(Spring),
    Expressive(expressive::SampledSpring),
}

impl MotionCurve {
    pub fn duration(self) -> Duration {
        match self {
            Self::Standard(spring) => spring.duration(),
            Self::Expressive(spring) => spring.duration,
        }
    }

    pub fn ease(self) -> impl Fn(f32) -> f32 + Copy + 'static {
        move |progress| match self {
            Self::Standard(spring) => spring.ease()(progress),
            Self::Expressive(spring) => spring.position(progress),
        }
    }

    fn position(self, seconds: f32) -> f32 {
        self.ease()(seconds / self.duration().as_secs_f32())
    }
}

impl MotionScheme {
    pub fn resolve(self, token: MotionToken) -> MotionCurve {
        use MotionToken::*;
        match self {
            Self::Standard => MotionCurve::Standard(match token {
                FastSpatial => FAST_SPATIAL,
                DefaultSpatial => DEFAULT_SPATIAL,
                SlowSpatial => SLOW_SPATIAL,
                FastEffects => FAST_EFFECTS,
                DefaultEffects => DEFAULT_EFFECTS,
                SlowEffects => SLOW_EFFECTS,
            }),
            Self::Expressive => MotionCurve::Expressive(match token {
                FastSpatial => expressive::FAST_SPATIAL,
                DefaultSpatial => expressive::DEFAULT_SPATIAL,
                SlowSpatial => expressive::SLOW_SPATIAL,
                FastEffects => expressive::FAST_EFFECTS,
                DefaultEffects => expressive::DEFAULT_EFFECTS,
                SlowEffects => expressive::SLOW_EFFECTS,
            }),
        }
    }
}

thread_local! {
    static SCHEME: Cell<MotionScheme> = const { Cell::new(MotionScheme::Expressive) };
}

/// Set the motion scheme for this root's render. Defaults to Expressive.
pub fn apply_scheme(scheme: MotionScheme) {
    SCHEME.set(scheme);
}

pub fn curve(token: MotionToken) -> MotionCurve {
    SCHEME.get().resolve(token)
}

#[derive(Clone, Copy)]
struct RunningCurve {
    curve: MotionCurve,
    running: bool,
}

/// Follow a target with a semantic token. A scheme switch takes effect after
/// the active transition (including interruptions) settles, so it cannot
/// replace the easing underneath GPUI Base's existing transition clock.
pub fn token_value(
    id: impl Into<ElementId>,
    target: f32,
    token: MotionToken,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    token_transition(id, target, token, window, cx)
}

/// Follow an interpolatable target using the selected Material motion token.
///
/// Each channel ID must retain the same value type.
/// Effects tokens preserve fades under reduced motion, matching the reference
/// CSS transition policy; spatial tokens finish immediately. Interpolation is
/// supplied by the value, so color spaces and compound geometry need not be flattened
/// into independent scalar clocks. Timing and retargeting belong to GPUI Base.
pub fn token_transition<T>(
    id: impl Into<ElementId>,
    target: T,
    token: MotionToken,
    window: &mut Window,
    cx: &mut App,
) -> T
where
    T: gpui_base::Interpolate + PartialEq + 'static,
{
    let id = id.into();
    let requested = curve(token);
    let state = window.use_keyed_state((id.clone(), "material-motion-curve"), cx, |_, _| {
        RunningCurve {
            curve: requested,
            running: false,
        }
    });
    let active = state.read(cx);
    let resolved = if active.running {
        active.curve
    } else {
        requested
    };
    let sample = gpui_base::transition_with_status(
        id,
        target,
        gpui_base::Transition::new(resolved.duration())
            .ease(resolved.ease())
            .reduced_motion(match token {
                MotionToken::FastEffects
                | MotionToken::DefaultEffects
                | MotionToken::SlowEffects => gpui_base::ReducedMotion::Continue,
                _ => gpui_base::ReducedMotion::Finish,
            }),
        window,
        cx,
    );
    state.update(cx, |state, _| {
        state.curve = resolved;
        state.running = matches!(
            sample.status,
            gpui_base::MotionStatus::Running | gpui_base::MotionStatus::Delayed
        );
    });
    sample.value
}

/// Short fades, e.g. a menu's entrance.
pub const ENTER_MS: u64 = 160;

/// A Material 3 spring: the Standard motion scheme's tokens
/// (`motionSpring*` in Material Components for Android). Spatial springs
/// move or reshape something and may settle with a barely visible overshoot;
/// effects springs fade color and opacity and never overshoot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub damping: f32,
    pub stiffness: f32,
}

/// Small components: a button's shape, a switch thumb, a selection indicator.
pub const FAST_SPATIAL: Spring = Spring {
    damping: 0.9,
    stiffness: 1400.,
};
/// Partly covering surfaces: a page region arriving, a disclosure opening.
pub const DEFAULT_SPATIAL: Spring = Spring {
    damping: 0.9,
    stiffness: 700.,
};
pub const SLOW_SPATIAL: Spring = Spring {
    damping: 0.9,
    stiffness: 300.,
};
pub const FAST_EFFECTS: Spring = Spring {
    damping: 1.,
    stiffness: 3800.,
};
pub const DEFAULT_EFFECTS: Spring = Spring {
    damping: 1.,
    stiffness: 1600.,
};
/// Whole-window color changes, such as switching palettes.
pub const SLOW_EFFECTS: Spring = Spring {
    damping: 1.,
    stiffness: 800.,
};

impl Spring {
    fn omega(self) -> f32 {
        self.stiffness.sqrt()
    }

    /// Time until the remaining distance stays under 0.1% of the change.
    pub fn duration(self) -> Duration {
        let rate = self.damping.min(1.) * self.omega();
        // A critically damped spring decays as (1 + ωt)·e^(-ωt), slower than
        // its envelope alone; 9.23/ω is where that falls below 0.001.
        let seconds = if self.damping >= 1. {
            9.23 / self.omega()
        } else {
            1000_f32.ln() / rate
        };
        Duration::from_secs_f32(seconds)
    }

    /// Position from rest at 0 toward 1 after `seconds`.
    pub fn position(self, seconds: f32) -> f32 {
        let t = seconds.max(0.);
        let omega = self.omega();
        let zeta = self.damping;
        if zeta >= 1. {
            1. - (1. + omega * t) * (-omega * t).exp()
        } else {
            let damped = omega * (1. - zeta * zeta).sqrt();
            1. - (-zeta * omega * t).exp()
                * ((damped * t).cos() + zeta * omega / damped * (damped * t).sin())
        }
    }

    /// The spring as an easing over its own `duration`, ending exactly at 1.
    pub fn ease(self) -> impl Fn(f32) -> f32 + Copy + 'static {
        let seconds = self.duration().as_secs_f32();
        move |t| {
            if t >= 1. {
                1.
            } else {
                self.position(t * seconds)
            }
        }
    }
}

/// IDs belong to a logical state, not a frame or a progress value.
pub fn enter<E: IntoElement + Styled + 'static>(id: impl Into<ElementId>, view: E) -> AnyElement {
    Entrance {
        id: id.into(),
        view,
        distance: None,
    }
    .into_any_element()
}

/// A change of task state settles into place, so the new title and available
/// action read as a consequence of the user's previous action.
pub fn state_enter<E: IntoElement + Styled + 'static>(
    id: impl Into<ElementId>,
    view: E,
) -> AnyElement {
    rise_in(id, view, 8.)
}

/// A destination, settings category or secondary page arrives as one unit;
/// navigation and the app bar stay put.
pub fn page_enter<E: IntoElement + Styled + 'static>(
    id: impl Into<ElementId>,
    view: E,
) -> AnyElement {
    rise_in(id, view, 12.)
}

/// Fades in while rising `distance` px into place: the offset follows the
/// default spatial spring and the opacity its effects spring, on one clock.
fn rise_in<E: IntoElement + Styled + 'static>(
    id: impl Into<ElementId>,
    view: E,
    distance: f32,
) -> AnyElement {
    Entrance {
        id: id.into(),
        view,
        distance: Some(distance),
    }
    .into_any_element()
}

#[derive(IntoElement)]
struct Entrance<E: IntoElement + Styled + 'static> {
    id: ElementId,
    view: E,
    distance: Option<f32>,
}

impl<E: IntoElement + Styled + 'static> RenderOnce for Entrance<E> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // A mounted entrance retains its curve even if a root switches schemes.
        // Keep the wrapper and IDs stable when reduced motion changes, too.
        let state =
            window.use_keyed_state((self.id.clone(), "material-entrance-scheme"), cx, |_, _| {
                (SCHEME.get(), false)
            });
        let (scheme, completed) = state.update(cx, |state, cx| {
            state.1 |= cx.reduce_motion();
            *state
        });
        let effects = scheme.resolve(MotionToken::DefaultEffects);
        if let Some(distance) = self.distance {
            let spatial = scheme.resolve(MotionToken::DefaultSpatial);
            let total = spatial.duration().max(effects.duration());
            let seconds = total.as_secs_f32();
            self.view
                .with_animation(self.id, Animation::new(total), move |view, t| {
                    let t = if completed { 1. } else { t };
                    let offset = spatial.position(t * seconds);
                    let opacity = effects.position(t * seconds);
                    view.relative()
                        .top(px(distance * (1. - offset)))
                        .opacity(opacity.clamp(0., 1.))
                })
                .into_any_element()
        } else {
            self.view
                .with_animation(
                    self.id,
                    Animation::new(effects.duration()).with_easing(effects.ease()),
                    move |view, t| view.opacity(if completed { 1. } else { t }),
                )
                .into_any_element()
        }
    }
}

/// A value that follows `target` on the default spatial spring, retargeting
/// from its current presentation when the target changes again.
pub fn value(id: impl Into<ElementId>, target: f32, window: &mut Window, cx: &mut App) -> f32 {
    token_value(id, target, MotionToken::DefaultSpatial, window, cx)
}

/// A selection moves and reshapes on the fast spatial spring, retargeting
/// from wherever it is when the choice changes again.
pub fn selection_value(
    id: impl Into<ElementId>,
    target: f32,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    token_value(id, target, MotionToken::FastSpatial, window, cx)
}

/// Content shown in place when `open`, fading in as it arrives.
pub fn disclosure(id: impl Into<ElementId>, open: bool, content: Div) -> AnyElement {
    if open {
        // Keep intrinsic measurement, padding and child layout in the normal
        // tree. Measuring in prepaint and clipping to a previous frame's height
        // cuts off controls when the content or available width changes.
        enter(id, content)
    } else {
        div().hidden().into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{enter, value};
    use gpui::{
        App, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _, Pixels,
        Render, RenderOnce, Styled as _, TestAppContext, VisualTestContext, Window, div, px, size,
    };
    use std::{cell::Cell, rc::Rc, time::Duration};

    #[derive(IntoElement)]
    struct StatefulChild {
        target: f32,
        mounts: Rc<Cell<usize>>,
        sample: Rc<Cell<f32>>,
    }

    impl RenderOnce for StatefulChild {
        fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
            window.use_keyed_state("motion-child-state", cx, |_, _| {
                self.mounts.set(self.mounts.get() + 1);
            });
            self.sample
                .set(value("motion-child-value", self.target, window, cx));
            div().size(px(40.))
        }
    }

    struct PreferenceHarness {
        target: f32,
        mounts: Rc<Cell<usize>>,
        sample: Rc<Cell<f32>>,
    }

    impl Render for PreferenceHarness {
        fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            enter(
                "motion-preference-parent",
                div().child(StatefulChild {
                    target: self.target,
                    mounts: self.mounts.clone(),
                    sample: self.sample.clone(),
                }),
            )
        }
    }

    #[gpui::test]
    fn reduce_motion_preserves_child_state_and_later_value_transitions(cx: &mut TestAppContext) {
        let mounts = Rc::new(Cell::new(0));
        let sample = Rc::new(Cell::new(-1.));
        let (view, cx) = cx.add_window_view({
            let mounts = mounts.clone();
            let sample = sample.clone();
            move |_, _| PreferenceHarness {
                target: 0.,
                mounts,
                sample,
            }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(mounts.get(), 1);
        assert_eq!(sample.get(), 0.);

        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.target = 1.;
                cx.set_reduce_motion(true);
                cx.notify();
            });
            window.draw(cx).clear(cx);
        });
        assert_eq!(mounts.get(), 1, "preference must not remount the child");
        assert_eq!(
            sample.get(),
            1.,
            "reduced motion paints the target immediately"
        );

        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.target = 0.;
                cx.set_reduce_motion(false);
                cx.notify();
            });
            window.draw(cx).clear(cx);
        });
        assert_eq!(mounts.get(), 1);
        assert_eq!(
            sample.get(),
            1.,
            "restored motion retains the previous target"
        );
        cx.executor().advance_clock(Duration::from_millis(50));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert!(
            sample.get() > 0. && sample.get() < 1.,
            "the next change must paint an intermediate value"
        );
        assert_eq!(mounts.get(), 1);
    }

    struct RevealHarness {
        open: bool,
        width: Pixels,
        content_height: Pixels,
    }

    impl Render for RevealHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let content = div()
                .debug_selector(|| "motion-reveal-surface".into())
                .flex()
                .flex_col()
                .p(px(16.))
                .child(
                    div()
                        .debug_selector(|| "motion-reveal-content".into())
                        .h(self.content_height),
                );
            let reveal = super::disclosure("geometry-reveal", self.open, content);
            div()
                .debug_selector(|| "motion-layout-root".into())
                .flex()
                .flex_col()
                .w(self.width)
                .gap(px(8.))
                .child(div().h(px(20.)))
                .child(reveal)
                .child(
                    div()
                        .debug_selector(|| "motion-layout-footer".into())
                        .h(px(20.)),
                )
        }
    }

    fn draw_open_reveal(
        view: &Entity<RevealHarness>,
        cx: &mut VisualTestContext,
        width: Pixels,
        content_height: Pixels,
    ) {
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.open = true;
                view.width = width;
                view.content_height = content_height;
                cx.notify();
            });
            window.draw(cx).clear(cx);
        });
    }

    fn assert_reveal_geometry(cx: &mut VisualTestContext, width: Pixels, height: Pixels) {
        let root = cx.debug_bounds("motion-layout-root").unwrap();
        let surface = cx.debug_bounds("motion-reveal-surface").unwrap();
        let content = cx.debug_bounds("motion-reveal-content").unwrap();
        let footer = cx.debug_bounds("motion-layout-footer").unwrap();
        assert_eq!(surface.left(), root.left());
        assert_eq!(surface.top(), root.top() + px(28.));
        assert_eq!(surface.size, size(width, height + px(32.)));
        assert_eq!(content.top() - surface.top(), px(16.));
        assert_eq!(surface.bottom() - content.bottom(), px(16.));
        assert_eq!(footer.top() - surface.bottom(), px(8.));
        assert!(footer.bottom() <= root.bottom());
    }

    #[gpui::test]
    fn disclosure_first_frame_preserves_natural_size_and_padding(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| RevealHarness {
            open: false,
            width: px(240.),
            content_height: px(40.),
        });
        draw_open_reveal(&view, cx, px(240.), px(40.));
        assert_reveal_geometry(cx, px(240.), px(40.));
    }

    #[gpui::test]
    fn disclosure_content_resize_updates_following_rows_in_the_same_frame(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| RevealHarness {
            open: false,
            width: px(240.),
            content_height: px(40.),
        });
        draw_open_reveal(&view, cx, px(240.), px(40.));
        draw_open_reveal(&view, cx, px(180.), px(96.));
        assert_reveal_geometry(cx, px(180.), px(96.));
    }

    #[gpui::test]
    fn disclosure_closes_without_leaving_height_or_children(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| RevealHarness {
            open: false,
            width: px(240.),
            content_height: px(40.),
        });
        draw_open_reveal(&view, cx, px(240.), px(40.));
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.open = false;
                cx.notify();
            });
            window.draw(cx).clear(cx);
        });
        let root = cx.debug_bounds("motion-layout-root").unwrap();
        let footer = cx.debug_bounds("motion-layout-footer").unwrap();
        assert_eq!(footer.top(), root.top() + px(28.));
        assert!(cx.debug_bounds("motion-reveal-surface").is_none());
        assert!(cx.debug_bounds("motion-reveal-content").is_none());
    }

    #[derive(IntoElement)]
    struct EntranceProbe {
        style: gpui::StyleRefinement,
        sample: Rc<std::cell::RefCell<gpui::StyleRefinement>>,
    }
    impl gpui::Styled for EntranceProbe {
        fn style(&mut self) -> &mut gpui::StyleRefinement {
            &mut self.style
        }
    }
    impl RenderOnce for EntranceProbe {
        fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
            *self.sample.borrow_mut() = self.style;
            div().size(px(40.))
        }
    }
    struct EntranceHarness {
        rise: bool,
        sample: Rc<std::cell::RefCell<gpui::StyleRefinement>>,
    }
    impl Render for EntranceHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let probe = EntranceProbe {
                style: Default::default(),
                sample: self.sample.clone(),
            };
            if self.rise {
                super::page_enter("probe", probe)
            } else {
                super::enter("probe", probe)
            }
        }
    }
    #[gpui::test]
    fn completed_entrances_do_not_rewind_when_reduced_motion_is_restored(cx: &mut TestAppContext) {
        for rise in [false, true] {
            let sample = Rc::new(std::cell::RefCell::new(gpui::StyleRefinement::default()));
            let (_, cx) = cx.add_window_view({
                let sample = sample.clone();
                move |_, _| EntranceHarness { rise, sample }
            });
            cx.update(|window, cx| window.draw(cx).clear(cx));
            cx.update(|window, cx| {
                cx.set_reduce_motion(true);
                window.refresh();
                window.draw(cx).clear(cx);
            });
            assert_eq!(sample.borrow().opacity, Some(1.));
            cx.update(|window, cx| {
                cx.set_reduce_motion(false);
                window.refresh();
                window.draw(cx).clear(cx);
            });
            assert_eq!(
                sample.borrow().opacity,
                Some(1.),
                "completed entrance cannot fade backward"
            );
            if rise {
                assert_eq!(
                    sample.borrow().inset.top,
                    Some(px(0.).into()),
                    "completed entrance cannot move backward"
                );
            }
        }
    }

    struct SchemeHarness {
        target: f32,
        scheme: super::MotionScheme,
        sample: Rc<Cell<f32>>,
    }

    impl Render for SchemeHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            super::apply_scheme(self.scheme);
            self.sample
                .set(super::value("scheme-transition", self.target, window, cx));
            div().size(px(40.))
        }
    }

    #[gpui::test]
    fn switching_schemes_and_reversing_preserve_the_running_presentation(cx: &mut TestAppContext) {
        use super::{MotionScheme, MotionToken};
        let sample = Rc::new(Cell::new(-1.));
        let (view, cx) = cx.add_window_view({
            let sample = sample.clone();
            move |_, _| SchemeHarness {
                target: 0.,
                scheme: MotionScheme::Expressive,
                sample,
            }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.target = 1.);
            window.draw(cx).clear(cx);
        });
        cx.executor().advance_clock(Duration::from_millis(120));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let before = sample.get();
        assert!(
            (before
                - MotionScheme::Expressive
                    .resolve(MotionToken::DefaultSpatial)
                    .position(0.120))
            .abs()
                < 0.00001
        );
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.scheme = MotionScheme::Standard);
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            sample.get(),
            before,
            "scheme change must not move an in-flight value"
        );
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.target = 0.);
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            sample.get(),
            before,
            "reversal must begin at the presentation value"
        );
        cx.executor().advance_clock(Duration::from_millis(50));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert!(sample.get() < before && sample.get() > 0.);
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(sample.get(), 0.);
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.target = 1.);
            window.draw(cx).clear(cx);
        });
        cx.executor().advance_clock(Duration::from_millis(50));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let standard = MotionScheme::Standard
            .resolve(MotionToken::DefaultSpatial)
            .position(0.050);
        assert!(
            (sample.get() - standard).abs() < 0.00001,
            "settled values adopt the new scheme"
        );
        cx.update(|window, cx| {
            cx.set_reduce_motion(true);
            view.update(cx, |view, _| view.target = 0.);
            window.draw(cx).clear(cx);
        });
        assert_eq!(sample.get(), 0.);
        super::apply_scheme(MotionScheme::Expressive);
    }

    #[test]
    fn springs_settle_on_their_target() {
        use super::{DEFAULT_EFFECTS, DEFAULT_SPATIAL, FAST_SPATIAL, SLOW_EFFECTS};
        for spring in [FAST_SPATIAL, DEFAULT_SPATIAL, DEFAULT_EFFECTS, SLOW_EFFECTS] {
            let ease = spring.ease();
            assert_eq!(ease(0.), 0.);
            assert_eq!(ease(1.), 1.);
            assert!((ease(0.999) - 1.).abs() < 0.002, "{spring:?}");
            let duration = spring.duration().as_secs_f32();
            assert!((0.1..0.5).contains(&duration), "{spring:?}: {duration}");
        }
        // Effects springs never overshoot; spatial ones barely do.
        let effects = (0..=200).map(|i| DEFAULT_EFFECTS.ease()(i as f32 / 200.));
        assert!(effects.clone().all(|v| v <= 1.));
        assert!(effects.collect::<Vec<_>>().windows(2).all(|p| p[0] <= p[1]));
        assert!((0..=200).all(|i| FAST_SPATIAL.ease()(i as f32 / 200.) < 1.003));
    }

    // Heap ownership deliberately prevents Copy: callers can animate compound values.
    #[derive(Clone, Debug, PartialEq)]
    struct CompoundEffects(Vec<f32>);

    impl gpui_base::Interpolate for CompoundEffects {
        fn interpolate(&self, target: &Self, progress: f32) -> Self {
            Self(
                self.0
                    .iter()
                    .zip(&target.0)
                    .map(|(from, to)| from + (to - from) * progress)
                    .collect(),
            )
        }
    }

    struct CompoundEffectsHarness {
        target: CompoundEffects,
        sample: Rc<std::cell::RefCell<CompoundEffects>>,
    }

    impl Render for CompoundEffectsHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            super::apply_scheme(super::MotionScheme::Expressive);
            *self.sample.borrow_mut() = super::token_transition(
                "compound-effects-transition",
                self.target.clone(),
                super::MotionToken::DefaultEffects,
                window,
                cx,
            );
            div().size(px(40.))
        }
    }

    #[gpui::test]
    fn generic_effects_interpolate_reverse_and_preserve_fades_under_reduced_motion(
        cx: &mut TestAppContext,
    ) {
        let start = CompoundEffects(vec![0., 10.]);
        let target = CompoundEffects(vec![1., 30.]);
        let sample = Rc::new(std::cell::RefCell::new(start.clone()));
        let (view, cx) = cx.add_window_view({
            let sample = sample.clone();
            let start = start.clone();
            move |_, _| CompoundEffectsHarness {
                target: start,
                sample,
            }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(*sample.borrow(), start);
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.target = target.clone());
            window.draw(cx).clear(cx);
        });
        assert_eq!(*sample.borrow(), start);
        cx.executor().advance_clock(Duration::from_millis(125));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        // The 250ms DefaultEffects curve has a documented midpoint sample of 0.96.
        let midpoint = sample.borrow().clone();
        assert!((midpoint.0[0] - 0.96).abs() < 0.00001);
        assert!((midpoint.0[1] - 29.2).abs() < 0.00001);
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.target = start.clone());
            window.draw(cx).clear(cx);
        });
        assert_eq!(*sample.borrow(), midpoint, "reversal must not jump");
        cx.executor().advance_clock(Duration::from_millis(50));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let reversing = sample.borrow().clone();
        assert!(reversing.0[0] > 0. && reversing.0[0] < midpoint.0[0]);
        assert!(reversing.0[1] > 10. && reversing.0[1] < midpoint.0[1]);
        cx.update(|window, cx| {
            cx.set_reduce_motion(true);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            *sample.borrow(),
            reversing,
            "reduced motion preserves the in-flight fade"
        );
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.target = target.clone());
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            *sample.borrow(),
            reversing,
            "retargeting still begins continuously"
        );
        cx.executor().advance_clock(Duration::from_millis(250));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            *sample.borrow(),
            target,
            "fade completes under reduced motion"
        );
        cx.update(|window, cx| {
            cx.set_reduce_motion(false);
            view.update(cx, |view, _| view.target = start.clone());
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            *sample.borrow(),
            target,
            "restoring motion retains presentation"
        );
        cx.executor().advance_clock(Duration::from_millis(250));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(*sample.borrow(), start);
    }
}
