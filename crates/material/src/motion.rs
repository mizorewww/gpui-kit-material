//! The motion vocabulary: M3 springs, entrances and retargetable values.
//!
//! Reference priority is Figma kit > md3.io > m3e.kyralabs.dev, per value.
//! The Figma Material 3 Design Kit defines no motion values.
//!
//! - Expressive (the default) uses md3.io's sampled CSS curves and durations
//!   for all six tokens ([`expressive`], floor/material
//!   c910072f3fc5b660276731d8a935d7186f288364). Under reduced motion it
//!   follows md3.io's captured policy (`conformance/reference/
//!   button-states-reduced-motion.json`): spatial tokens finish immediately and
//!   effects keep their own curve.
//! - Standard is absent from md3.io, so it uses m3e.kyralabs.dev's analytic
//!   springs (kyra-labs/m3exp-web `3b1e0047c0021eab7b4c0128fa1bec95c76ca746`,
//!   `packages/m3e-tokens/src/generated/tokens.css`, generated from Compose
//!   Material3 1.5.0-beta01 `StandardMotionTokens.kt` @ androidx
//!   `19991f2c0858aadfcb442ce02b2c8e4743c96898`) timed by Compose's settle
//!   estimate, and m3e's reduced-motion tokens: spatial tokens finish
//!   immediately and default/slow effects play the fast effects spring.
//!   Fixture: `conformance/fixtures/m3e-motion.json`.
//!
//! Apply the scheme at the start of a root render, like the theme; helpers
//! below resolve it on the UI thread.
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
    /// The analytic spring behind this curve: Standard's m3e spring. md3.io
    /// specifies Expressive as sampled curves, not spring parameters, so an
    /// Expressive curve has none.
    pub fn spring(self) -> Option<Spring> {
        match self {
            Self::Standard(spring) => Some(spring),
            Self::Expressive(_) => None,
        }
    }

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

    /// Position after `seconds` of a 0 → 1 transition on this curve.
    pub(crate) fn position(self, seconds: f32) -> f32 {
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
pub fn set_motion_scheme(scheme: MotionScheme) {
    SCHEME.set(scheme);
}

/// Former name of [`set_motion_scheme`]; it read like `theme::apply_scheme`,
/// which applies a color scheme.
#[deprecated(note = "renamed to `motion::set_motion_scheme`")]
pub fn apply_scheme(scheme: MotionScheme) {
    set_motion_scheme(scheme);
}

pub fn curve(token: MotionToken) -> MotionCurve {
    SCHEME.get().resolve(token)
}

/// The curve a transition plays under the current scheme and motion
/// preference. Spatial tokens finish through [`token_policy`] in both
/// schemes. Standard follows m3e's reduced-motion tokens, which replace the
/// default and slow effects springs with the fast one; Expressive follows
/// md3.io, whose effects keep their own curve.
fn effective_curve(token: MotionToken, cx: &App) -> MotionCurve {
    let scheme = SCHEME.get();
    scheme.resolve(match token {
        MotionToken::DefaultEffects | MotionToken::SlowEffects
            if scheme == MotionScheme::Standard && cx.reduce_motion() =>
        {
            MotionToken::FastEffects
        }
        token => token,
    })
}

#[derive(Clone, Copy)]
struct RunningCurve {
    curve: MotionCurve,
    running: bool,
}

impl RunningCurve {
    fn resolve(self, requested: MotionCurve) -> MotionCurve {
        if self.running { self.curve } else { requested }
    }
    fn record(&mut self, resolved: MotionCurve, status: gpui_base::MotionStatus) {
        self.curve = resolved;
        self.running = matches!(
            status,
            gpui_base::MotionStatus::Running | gpui_base::MotionStatus::Delayed
        );
    }
}

fn token_policy(resolved: MotionCurve, token: MotionToken) -> gpui_base::Transition {
    gpui_base::Transition::new(resolved.duration())
        .ease(resolved.ease())
        .reduced_motion(match token {
            MotionToken::FastEffects | MotionToken::DefaultEffects | MotionToken::SlowEffects => {
                gpui_base::ReducedMotion::Continue
            }
            _ => gpui_base::ReducedMotion::Finish,
        })
}

/// Caller-owned token channel. Keeps the active curve and Base reversal clock
/// across virtual unmounts, without retaining a window or scheduling frames.
#[derive(Clone)]
pub struct TokenTransitionPlayback<T> {
    playback: gpui_base::TransitionPlayback<T>,
    curve: std::rc::Rc<std::cell::Cell<Option<RunningCurve>>>,
    target: std::rc::Rc<std::cell::RefCell<Option<T>>>,
}
impl<T> Default for TokenTransitionPlayback<T> {
    fn default() -> Self {
        Self {
            playback: Default::default(),
            curve: Default::default(),
            target: Default::default(),
        }
    }
}
impl<T: gpui_base::Interpolate + PartialEq + 'static> TokenTransitionPlayback<T> {
    /// Resample an initialized channel's current visual target, preserving
    /// transient presentation while applying a motion preference change.
    pub fn resample(&self, token: MotionToken, cx: &App) -> Option<gpui_base::MotionValue<T>> {
        let target = self.target.borrow().clone()?;
        Some(self.sample(target, token, cx))
    }

    /// Forward actual target/preference changes while unmounted. A channel's
    /// first sample adopts its target; a fresh handle resets its lifecycle.
    pub fn sample(&self, target: T, token: MotionToken, cx: &App) -> gpui_base::MotionValue<T> {
        let requested = effective_curve(token, cx);
        let mut active = self.curve.get().unwrap_or(RunningCurve {
            curve: requested,
            running: false,
        });
        // An unmounted channel may have settled without being sampled. Let
        // Base evaluate its old target before choosing the next target's curve.
        if active.running
            && let Some(previous) = self.target.borrow().clone()
        {
            let old = self
                .playback
                .sample(previous, token_policy(active.curve, token), cx);
            active.record(active.curve, old.status);
        }
        let resolved = active.resolve(requested);
        *self.target.borrow_mut() = Some(target.clone());
        let sample = self
            .playback
            .sample(target, token_policy(resolved, token), cx);
        active.record(resolved, sample.status);
        self.curve.set(Some(active));
        sample
    }
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
/// Effects tokens preserve fades under reduced motion (Expressive keeps
/// md3.io's curves; Standard plays default and slow effects on m3e's fast
/// effects spring); spatial tokens finish immediately. Interpolation is
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
    let requested = effective_curve(token, cx);
    let state = window.use_keyed_state((id.clone(), "material-motion-curve"), cx, |_, _| {
        RunningCurve {
            curve: requested,
            running: false,
        }
    });
    let active = state.read(cx);
    let resolved = active.resolve(requested);
    let sample =
        gpui_base::transition_with_status(id, target, token_policy(resolved, token), window, cx);
    state.update(cx, |state, _| {
        state.record(resolved, sample.status);
    });
    sample.value
}

/// Short fades, e.g. a menu's entrance.
pub const ENTER_MS: u64 = 160;

/// A Material 3 spring (damping ratio, stiffness; unit mass). The constants
/// below are the Standard motion scheme's tokens (md3.io has no Standard
/// scheme, so m3e is the source), which m3e generates from
/// Compose `StandardMotionTokens.kt` (L20-L31) and lists in `tokens.css` as
/// `--m3e-sys-motion-spring-standard-*`. Spatial springs move or reshape
/// something and may settle with a barely visible overshoot; effects springs
/// fade color and opacity and never overshoot.
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

// Bounds that keep every derived duration finite: a spring with no
// damping or no stiffness never settles, and `Duration::from_secs_f32`
// panics on the infinite or NaN time it would take.
const MIN_DAMPING: f32 = 0.01;
const MAX_DAMPING: f32 = 1000.;
const MIN_STIFFNESS: f32 = 1.;
const MAX_STIFFNESS: f32 = 1.0e6;

impl Spring {
    /// The same spring with damping and stiffness clamped to finite,
    /// positive values. NaN damping falls back to critical damping and NaN
    /// stiffness to the minimum.
    pub fn sanitized(self) -> Self {
        let damping = if self.damping.is_nan() {
            1.
        } else {
            self.damping.clamp(MIN_DAMPING, MAX_DAMPING)
        };
        let stiffness = if self.stiffness.is_nan() {
            MIN_STIFFNESS
        } else {
            self.stiffness.clamp(MIN_STIFFNESS, MAX_STIFFNESS)
        };
        Self { damping, stiffness }
    }

    fn omega(self) -> f32 {
        self.stiffness.sqrt()
    }

    /// Compose's settle estimate for a 0 → 1 move at a 0.1% visibility
    /// threshold, in whole milliseconds and at least 1ms: the `-duration`
    /// m3e derives for every spring token (`packages/m3e-motion/src/linear.ts`,
    /// `FloatSpringSpec.getDurationNanos`).
    pub fn duration(self) -> Duration {
        let spring = self.sanitized();
        let millis = settle_estimate_millis(spring.stiffness as f64, spring.damping as f64, 1000.);
        Duration::from_millis(millis.max(1))
    }

    /// Position from rest at 0 toward 1 after `seconds`.
    pub fn position(self, seconds: f32) -> f32 {
        let spring = self.sanitized();
        // NaN time is treated as rest; infinite time has settled.
        let t = seconds.max(0.);
        if t == f32::INFINITY {
            return 1.;
        }
        let omega = spring.omega();
        let zeta = spring.damping;
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

/// Port of Compose `SpringEstimation.kt` (animation-core @ androidx
/// `19991f2c0858aadfcb442ce02b2c8e4743c96898`, via m3e's
/// `packages/m3e-motion/src/spring-estimation.ts`): the time in whole
/// milliseconds after which a spring released `displacement` thresholds from
/// rest stays within one threshold of its target.
fn settle_estimate_millis(stiffness: f64, damping_ratio: f64, displacement: f64) -> u64 {
    let delta = 1.;
    let p0 = displacement.abs();
    let v0 = 0.;
    if p0 == 0. {
        return 0;
    }
    let coefficient = 2. * damping_ratio * stiffness.sqrt();
    let partial = coefficient * coefficient - 4. * stiffness;
    let partial_real = if partial < 0. { 0. } else { partial.sqrt() };
    let partial_imaginary = if partial < 0. {
        partial.abs().sqrt()
    } else {
        0.
    };
    let r1 = (-coefficient + partial_real) * 0.5;
    let i1 = partial_imaginary * 0.5;
    let r2 = (-coefficient - partial_real) * 0.5;
    let newton = |mut t: f64, f: &dyn Fn(f64) -> f64, df: &dyn Fn(f64) -> f64| {
        let mut change = f64::MAX;
        let mut iterations = 0;
        while change > 0.001 && iterations < 100 {
            iterations += 1;
            let last = t;
            t -= f(t) / df(t);
            change = (last - t).abs();
        }
        t
    };
    let seconds = if damping_ratio > 1. {
        let c2 = (r1 * p0 - v0) / (r1 - r2);
        let c1 = p0 - c2;
        let t1 = (delta / c1).abs().ln() / r1;
        let t2 = (delta / c2).abs().ln() / r2;
        let mut t = if !t1.is_finite() {
            t2
        } else if !t2.is_finite() {
            t1
        } else {
            t1.max(t2)
        };
        let inflection = ((c1 * r1) / (-c2 * r2)).ln() / (r2 - r1);
        let x_inflection = c1 * (r1 * inflection).exp() + c2 * (r2 * inflection).exp();
        let signed = if inflection.is_nan() || inflection <= 0. {
            -delta
        } else if -x_inflection < delta {
            if c2 > 0. && c1 < 0. {
                t = 0.;
            }
            -delta
        } else {
            t = (-(c2 * r2 * r2) / (c1 * r1 * r1)).ln() / (r1 - r2);
            delta
        };
        if (c1 * r1 * (r1 * t).exp() + c2 * r2 * (r2 * t).exp()).abs() < 0.0001 {
            t
        } else {
            newton(
                t,
                &|t| c1 * (r1 * t).exp() + c2 * (r2 * t).exp() + signed,
                &|t| c1 * r1 * (r1 * t).exp() + c2 * r2 * (r2 * t).exp(),
            )
        }
    } else if damping_ratio < 1. {
        // Under-damped: the envelope c·e^(r·t) of the oscillation.
        let c2 = (v0 - r1 * p0) / i1;
        (delta / (p0 * p0 + c2 * c2).sqrt()).ln() / r1
    } else {
        let r = r1;
        let (c1, c2) = (p0, v0 - r * p0);
        let t1 = (delta / c1).abs().ln() / r;
        let guess = (delta / c2).abs().ln();
        let mut lambert = guess;
        for _ in 0..=5 {
            lambert = guess - (lambert / r).abs().ln();
        }
        let t2 = lambert / r;
        let mut t = if !t1.is_finite() {
            t2
        } else if !t2.is_finite() {
            t1
        } else {
            t1.max(t2)
        };
        let inflection = -(r * c1 + c2) / (r * c2);
        let x_inflection = c1 * (r * inflection).exp() + c2 * inflection * (r * inflection).exp();
        let signed = if inflection.is_nan() || inflection <= 0. {
            -delta
        } else if -x_inflection < delta {
            if c2 < 0. && c1 > 0. {
                t = 0.;
            }
            -delta
        } else {
            t = -(2. / r) - c1 / c2;
            delta
        };
        newton(t, &|t| (c1 + c2 * t) * (r * t).exp() + signed, &|t| {
            (c2 * (r * t + 1.) + c1 * r) * (r * t).exp()
        })
    };
    // Kotlin's Double.toLong(): truncation, NaN → 0, saturating.
    let millis = seconds * 1000.;
    if millis.is_nan() || millis <= 0. {
        0
    } else {
        millis.min(u64::MAX as f64) as u64
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

/// Fades in while rising `distance` (100%-scale px, painted in rems so it
/// follows the text scale) into place: the offset follows the
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

#[derive(gpui::IntoElement)]
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
                        .top(crate::tokens::typography::rem(distance * (1. - offset)))
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

    #[test]
    fn degenerate_springs_have_finite_durations_and_positions() {
        let springs = [
            super::Spring {
                damping: 0.,
                stiffness: 700.,
            },
            super::Spring {
                damping: -1.,
                stiffness: 700.,
            },
            super::Spring {
                damping: 0.9,
                stiffness: 0.,
            },
            super::Spring {
                damping: f32::NAN,
                stiffness: f32::NAN,
            },
            super::Spring {
                damping: f32::INFINITY,
                stiffness: f32::INFINITY,
            },
        ];
        for spring in springs {
            let duration = spring.duration();
            assert!(duration > Duration::ZERO, "{spring:?}");
            let curve = super::MotionCurve::Standard(spring);
            assert!(curve.duration().as_secs_f32().is_finite());
            for t in [0., 0.25, 0.5, 1., f32::NAN] {
                assert!(curve.ease()(t).is_finite(), "{spring:?} at {t}");
            }
            assert!(spring.position(f32::INFINITY).is_finite());
            assert_eq!(curve.ease()(1.), 1.);
        }
        assert_eq!(super::DEFAULT_SPATIAL.sanitized(), super::DEFAULT_SPATIAL);
    }

    #[derive(gpui::IntoElement)]
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

    #[derive(gpui::IntoElement)]
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
                    Some(crate::tokens::typography::rem(0.).into()),
                    "completed entrance cannot move backward; its rise is in rems"
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
            super::set_motion_scheme(self.scheme);
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
            window.refresh();
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
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            sample.get(),
            before,
            "scheme change must not move an in-flight value"
        );
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.target = 0.);
            window.refresh();
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
            window.refresh();
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
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(sample.get(), 0.);
        super::set_motion_scheme(MotionScheme::Expressive);
    }

    #[test]
    fn standard_scheme_matches_the_pinned_m3e_springs() {
        use super::{MotionScheme, MotionToken};
        let reference: serde_json::Value = serde_json::from_str(include_str!(
            "../../../conformance/fixtures/m3e-motion.json"
        ))
        .unwrap();
        assert_eq!(
            reference["source_commit"],
            "3b1e0047c0021eab7b4c0128fa1bec95c76ca746"
        );
        let springs = reference["springs"]["standard"].as_array().unwrap();
        assert_eq!(springs.len(), 6);
        let mut seen = std::collections::BTreeSet::new();
        for entry in springs {
            let label = entry["token"].as_str().unwrap();
            assert!(seen.insert(label), "duplicate {label}");
            let token = match label {
                "fast-spatial" => MotionToken::FastSpatial,
                "default-spatial" => MotionToken::DefaultSpatial,
                "slow-spatial" => MotionToken::SlowSpatial,
                "fast-effects" => MotionToken::FastEffects,
                "default-effects" => MotionToken::DefaultEffects,
                "slow-effects" => MotionToken::SlowEffects,
                _ => panic!("unexpected reference token {label}"),
            };
            let curve = MotionScheme::Standard.resolve(token);
            let spring = curve.spring().unwrap();
            let number = |key: &str| entry[key].as_f64().unwrap() as f32;
            assert_eq!(spring.damping, number("damping_ratio"), "{label} damping");
            assert_eq!(spring.stiffness, number("stiffness"), "{label} stiffness");
            assert_eq!(
                curve.duration().as_millis(),
                entry["duration_ms"].as_u64().unwrap() as u128,
                "{label} Compose settle duration"
            );
            // Each linear() stop is the spring sampled on a whole millisecond
            // and rounded to 4 digits.
            let stops = entry["linear_stops"].as_array().unwrap();
            assert!(stops.len() >= 2);
            for stop in stops {
                let progress = stop[0].as_f64().unwrap() as f32;
                let expected = stop[1].as_f64().unwrap() as f32;
                let actual = curve.ease()(progress);
                assert!(
                    (actual - expected).abs() < 0.0005,
                    "{label} at {progress}: {actual} vs m3e {expected}"
                );
            }
        }
        assert_eq!(
            reference["reduced_motion"]["default-effects"]["replaced_by"],
            "fast-effects"
        );
        assert_eq!(
            reference["reduced_motion"]["slow-effects"]["replaced_by"],
            "fast-effects"
        );
    }

    #[gpui::test]
    fn reduced_motion_follows_each_schemes_source(cx: &mut TestAppContext) {
        use super::{MotionScheme, MotionToken, effective_curve};
        cx.update(|cx| {
            for reduce in [false, true] {
                cx.set_reduce_motion(reduce);
                for token in [MotionToken::DefaultEffects, MotionToken::SlowEffects] {
                    super::set_motion_scheme(MotionScheme::Standard);
                    // m3e: default/slow effects play the fast effects spring.
                    assert_eq!(
                        effective_curve(token, cx),
                        MotionScheme::Standard.resolve(if reduce {
                            MotionToken::FastEffects
                        } else {
                            token
                        }),
                        "Standard {token:?} reduce={reduce}"
                    );
                    super::set_motion_scheme(MotionScheme::Expressive);
                    // md3.io: effects keep their own curve.
                    assert_eq!(
                        effective_curve(token, cx),
                        MotionScheme::Expressive.resolve(token),
                        "Expressive {token:?} reduce={reduce}"
                    );
                }
            }
            cx.set_reduce_motion(false);
        });
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
            super::set_motion_scheme(super::MotionScheme::Expressive);
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
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(*sample.borrow(), start);
        cx.executor().advance_clock(Duration::from_millis(125));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        // Expressive DefaultEffects is md3.io's 250ms sampled curve, whose
        // midpoint stop is 0.96.
        let expected = super::MotionScheme::Expressive
            .resolve(super::MotionToken::DefaultEffects)
            .position(0.125);
        let midpoint = sample.borrow().clone();
        assert!((expected - 0.96).abs() < 0.00001, "{expected}");
        assert!((midpoint.0[0] - expected).abs() < 0.00001);
        assert!((midpoint.0[1] - (10. + 20. * expected)).abs() < 0.0001);
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.target = start.clone());
            window.refresh();
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
            window.refresh();
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
            window.refresh();
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
