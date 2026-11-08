//! Concrete button presentation channels; timing/retargeting remain in Base.
use super::*;

/// Controlled presentation inputs for an unmounted logical button.
/// `disabled` includes loading's disabled presentation. Transient interaction
/// is deliberately absent: sync uses resting hover/focus/pointer state.
#[derive(Clone, Copy, PartialEq)]
pub struct ButtonTransitionInputs {
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub shape: ButtonShape,
    pub toggle: bool,
    pub selected: bool,
    pub disabled: bool,
    pub active: bool,
    pub ripple: bool,
}

impl Default for ButtonTransitionInputs {
    fn default() -> Self {
        Self {
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            shape: ButtonShape::default(),
            toggle: false,
            selected: false,
            disabled: false,
            active: false,
            ripple: true,
        }
    }
}

/// One logical button's visual transitions, independent of virtual mounting.
/// Clones share only samples/curves. No focus, pointer, hover, ripple, view or
/// window is retained. Forward offscreen controlled input changes through
/// `sync` while drawing, as after layout decides which rows are mounted; call it
/// after updating the Material motion scheme/theme.
#[derive(Clone, Default)]
pub struct ButtonTransitionPlayback {
    last_sync: std::rc::Rc<std::cell::Cell<Option<SyncInputs>>>,
    pub(super) background: motion::TokenTransitionState<CssColor>,
    pub(super) foreground: motion::TokenTransitionState<CssColor>,
    pub(super) border: motion::TokenTransitionState<CssColor>,
    pub(super) radius: motion::TokenTransitionState<f32>,
    pub(super) shadow: motion::TokenTransitionState<shadow::ShadowPair>,
    pub(super) opacity: gpui_base::TransitionState<f32>,
    opacity_target: std::rc::Rc<std::cell::Cell<f32>>,
}
#[derive(Clone, Copy, PartialEq)]
struct SyncInputs {
    inputs: ButtonTransitionInputs,
    reduced: bool,
    spatial: motion::MotionCurve,
    effects: motion::MotionCurve,
}
impl ButtonTransitionPlayback {
    pub(super) fn record_inputs(&self, inputs: ButtonTransitionInputs, cx: &App) {
        self.last_sync.set(Some(Self::observed(inputs, cx)));
    }

    fn observed(inputs: ButtonTransitionInputs, cx: &App) -> SyncInputs {
        SyncInputs {
            inputs,
            reduced: cx.reduce_motion(),
            spatial: motion::curve(MotionToken::FastSpatial),
            effects: motion::curve(MotionToken::DefaultEffects),
        }
    }
    /// Sample actual inputs while drawing; a channel that starts running
    /// requests the next frame. First sync adopts the target, so initializing
    /// unused handles does not create an animation.
    pub fn sync(&self, inputs: ButtonTransitionInputs, window: &mut Window, cx: &mut App) {
        let observed = Self::observed(inputs, cx);
        // Repeated owner renders must not reset the visible button's transient
        // targets to rest or allocate new color expressions every frame.
        if self.last_sync.get() == Some(observed) {
            return;
        }
        let only_preferences = self.last_sync.get().is_some_and(|old| old.inputs == inputs);
        self.last_sync.set(Some(observed));
        if only_preferences {
            self.background
                .resample(MotionToken::DefaultEffects, window, cx);
            self.foreground
                .resample(MotionToken::DefaultEffects, window, cx);
            self.border
                .resample(MotionToken::DefaultEffects, window, cx);
            self.radius.resample(MotionToken::FastSpatial, window, cx);
            self.shadow
                .resample(MotionToken::DefaultEffects, window, cx);
            self.opacity
                .sample(self.opacity_target.get(), opacity_policy(), window, cx);
            return;
        }
        let (bg, fg, border) = toggle_color_targets(
            inputs.variant,
            inputs.disabled,
            inputs.toggle,
            inputs.selected,
        );
        self.background
            .sample(bg, MotionToken::DefaultEffects, window, cx);
        self.foreground
            .sample(fg, MotionToken::DefaultEffects, window, cx);
        self.border
            .sample(border, MotionToken::DefaultEffects, window, cx);
        let radius = if inputs.active {
            inputs.size.geometry().pressed
        } else {
            resting_radius(inputs.size, inputs.shape, inputs.toggle && inputs.selected)
        };
        self.radius
            .sample(radius, MotionToken::FastSpatial, window, cx);
        self.shadow.sample(
            shadow::ShadowPair::level(shadow_level(
                inputs.variant,
                inputs.disabled,
                inputs.toggle,
                false,
                inputs.active,
            )),
            MotionToken::DefaultEffects,
            window,
            cx,
        );
        let opacity = state_opacity(
            inputs.active,
            inputs.disabled,
            inputs.ripple,
            inputs.active,
            false,
            false,
        );
        self.opacity_target.set(opacity);
        self.opacity.sample(opacity, opacity_policy(), window, cx);
    }
}

pub(super) fn opacity_policy() -> gpui_base::Transition {
    gpui_base::Transition::new(std::time::Duration::from_millis(100))
        .reduced_motion(gpui_base::ReducedMotion::Continue)
        .easing(gpui_base::Easing::cubic_bezier(0.2, 0., 0., 1.).expect("valid Material easing"))
}
pub(super) fn state_opacity(
    controlled_active: bool,
    disabled: bool,
    ripple: bool,
    active: bool,
    focused: bool,
    hovered: bool,
) -> f32 {
    if controlled_active && !ripple {
        state::PRESS
    } else if disabled {
        0.
    } else if active && !ripple {
        state::PRESS
    } else if focused {
        state::FOCUS
    } else if hovered {
        state::HOVER
    } else {
        0.
    }
}
pub(super) fn shadow_level(
    variant: ButtonVariant,
    disabled: bool,
    toggle: bool,
    hovered: bool,
    active: bool,
) -> u8 {
    if disabled {
        0
    } else if variant == ButtonVariant::Elevated {
        if hovered && !active { 2 } else { 1 }
    } else if !toggle
        && hovered
        && !active
        && matches!(variant, ButtonVariant::Filled | ButtonVariant::Tonal)
    {
        1
    } else {
        0
    }
}
pub(super) fn token_sample<T: gpui_base::Interpolate + PartialEq + 'static>(
    state: &motion::TokenTransitionState<T>,
    target: T,
    token: MotionToken,
    window: &mut Window,
    cx: &mut App,
) -> T {
    state.follow(target, token, window, cx)
}
pub(super) fn opacity_sample(
    playback: &ButtonTransitionPlayback,
    target: f32,
    policy: gpui_base::Transition,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    playback.opacity_target.set(target);
    playback.opacity.sample(target, policy, window, cx).value
}

#[cfg(test)]
mod tests {
    use super::{ButtonTransitionInputs, ButtonTransitionPlayback, opacity_policy, shadow};
    use crate::motion::{self, MotionToken};
    use gpui::{App, Context, Empty, IntoElement, Render, TestAppContext, Window};

    /// Runs its closure while drawing, where a sample may request a frame.
    type Run = Box<dyn FnOnce(&mut Window, &mut App)>;
    struct WhileDrawing(Option<Run>);
    impl Render for WhileDrawing {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if let Some(run) = self.0.take() {
                run(window, cx);
            }
            Empty
        }
    }

    #[gpui::test]
    fn preference_sync_keeps_last_visual_targets(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(false);
        });
        let (view, cx) = cx.add_window_view(|_, _| {
            WhileDrawing(Some(Box::new(|window, cx| {
                motion::set_motion_scheme(motion::MotionScheme::Expressive);
                let playback = ButtonTransitionPlayback::default();
                let inputs = ButtonTransitionInputs::default();
                playback.sync(inputs, window, cx);
                // Simulate the last targets produced by live transient presentation,
                // without putting pointer/focus flags into the persistent handle.
                let pressed = inputs.size.geometry().pressed;
                playback
                    .radius
                    .sample(pressed, MotionToken::FastSpatial, window, cx);
                playback.shadow.sample(
                    shadow::ShadowPair::level(1),
                    MotionToken::DefaultEffects,
                    window,
                    cx,
                );
                playback.opacity_target.set(crate::tokens::state::HOVER);
                playback
                    .opacity
                    .sample(crate::tokens::state::HOVER, opacity_policy(), window, cx);
                cx.set_reduce_motion(true);
                motion::set_motion_scheme(motion::MotionScheme::Standard);
                playback.sync(inputs, window, cx);
                assert_eq!(
                    playback
                        .radius
                        .resample(MotionToken::FastSpatial, window, cx)
                        .unwrap()
                        .value,
                    pressed
                );
                assert_eq!(playback.opacity_target.get(), crate::tokens::state::HOVER);
                // Same target remains a continuation, not a rest-target reversal.
                assert_eq!(
                    playback
                        .radius
                        .resample(MotionToken::FastSpatial, window, cx)
                        .unwrap()
                        .status,
                    gpui_base::MotionStatus::Finished
                );
            })))
        });
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert!(
            view.read_with(cx, |view, _| view.0.is_none()),
            "ran while drawing"
        );
    }
}
