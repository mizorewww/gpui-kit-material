//! Material button presentation over GPUI Base activation and focus behavior.
//! Reference: floor/material c910072f3fc5b660276731d8a935d7186f288364.
//! Expanded touch targets and loading semantics are still pending.
mod control;
mod effects;
mod focus_ring;
mod ripple;
mod ripple_clip;
mod shadow;
#[cfg(test)]
mod toggle_tests;

use control::BaseControl;
use effects::CssColor;

use crate::{
    color::ColorRole,
    icons::Icon,
    motion::{self, MotionToken},
    tokens::typography::{TypeRole, rem},
};
use gpui::{prelude::*, *};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Filled,
    Elevated,
    Tonal,
    Outlined,
    Text,
}
impl ButtonVariant {
    pub const ALL: [Self; 5] = [
        Self::Filled,
        Self::Elevated,
        Self::Tonal,
        Self::Outlined,
        Self::Text,
    ];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Filled => "filled",
            Self::Elevated => "elevated",
            Self::Tonal => "tonal",
            Self::Outlined => "outlined",
            Self::Text => "text",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    ExtraSmall,
    #[default]
    Small,
    Medium,
    Large,
    ExtraLarge,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonShape {
    #[default]
    Round,
    Square,
}

impl ButtonShape {
    pub const ALL: [Self; 2] = [Self::Round, Self::Square];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Round => "round",
            Self::Square => "square",
        }
    }
}

/// Logical placement of the optional icon, relative to the label.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonIconPosition {
    #[default]
    Leading,
    Trailing,
}
/// Layout direction for logical icon placement and padding.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonDirection {
    #[default]
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug)]
struct Geometry {
    height: f32,
    padding: f32,
    gap: f32,
    outline: f32,
    square: f32,
    pressed: f32,
    typography: TypeRole,
}
impl ButtonSize {
    pub const ALL: [Self; 5] = [
        Self::ExtraSmall,
        Self::Small,
        Self::Medium,
        Self::Large,
        Self::ExtraLarge,
    ];
    pub const fn name(self) -> &'static str {
        match self {
            Self::ExtraSmall => "xs",
            Self::Small => "s",
            Self::Medium => "m",
            Self::Large => "l",
            Self::ExtraLarge => "xl",
        }
    }
    fn icon_size(self) -> f32 {
        match self {
            Self::ExtraSmall | Self::Small => 20.,
            Self::Medium => 24.,
            Self::Large => 32.,
            Self::ExtraLarge => 40.,
        }
    }
    fn geometry(self) -> Geometry {
        let (height, padding, gap, outline, square, pressed, typography) = match self {
            Self::ExtraSmall => (32., 12., 4., 1., 12., 8., TypeRole::LabelLarge),
            Self::Small => (40., 16., 8., 1., 12., 8., TypeRole::LabelLarge),
            Self::Medium => (56., 24., 8., 1., 16., 12., TypeRole::TitleMedium),
            Self::Large => (96., 48., 12., 2., 28., 16., TypeRole::HeadlineSmall),
            Self::ExtraLarge => (136., 64., 16., 3., 28., 16., TypeRole::HeadlineLarge),
        };
        Geometry {
            height,
            padding,
            gap,
            outline,
            square,
            pressed,
            typography,
        }
    }
}

fn horizontal_padding(
    size: ButtonSize,
    variant: ButtonVariant,
    icon: Option<ButtonIconPosition>,
    direction: ButtonDirection,
) -> (f32, f32) {
    let ordinary = size.geometry().padding;
    let compact_text = variant == ButtonVariant::Text
        && matches!(size, ButtonSize::ExtraSmall | ButtonSize::Small);
    let (mut start, mut end) = if compact_text {
        (12., 12.)
    } else {
        (ordinary, ordinary)
    };
    if let Some(position) = icon
        && (compact_text || size == ButtonSize::Small)
    {
        (start, end) = match position {
            ButtonIconPosition::Leading => (12., 16.),
            ButtonIconPosition::Trailing => (16., 12.),
        };
    }
    if direction == ButtonDirection::Rtl {
        (end, start)
    } else {
        (start, end)
    }
}

fn alpha(mut color: Rgba, opacity: f32) -> Rgba {
    color.a *= opacity;
    color
}
/// Composite a state layer without darkening its RGB on transparent buttons.
fn layer(container: Rgba, content: Rgba, opacity: f32) -> Rgba {
    let top = alpha(content, opacity);
    let a = top.a + container.a * (1. - top.a);
    if a == 0. {
        return gpui::rgba(0x00000000);
    }
    let c = |x: f32, y: f32| (x * top.a + y * container.a * (1. - top.a)) / a;
    Rgba {
        r: c(top.r, container.r),
        g: c(top.g, container.g),
        b: c(top.b, container.b),
        a,
    }
}
fn color_targets(variant: ButtonVariant, disabled: bool) -> (CssColor, CssColor, CssColor) {
    use ColorRole as C;
    if disabled {
        return (
            if matches!(variant, ButtonVariant::Text | ButtonVariant::Outlined) {
                CssColor::transparent()
            } else {
                CssColor::alpha_role(C::ON_SURFACE, 0.10)
            },
            CssColor::alpha_role(C::ON_SURFACE_VARIANT, 0.38),
            if variant == ButtonVariant::Outlined {
                CssColor::alpha_role(C::OUTLINE_VARIANT, 0.10)
            } else {
                CssColor::transparent()
            },
        );
    }
    let (bg, fg) = match variant {
        ButtonVariant::Filled => (CssColor::role(C::PRIMARY), C::ON_PRIMARY),
        ButtonVariant::Elevated => (CssColor::role(C::SURFACE_CONTAINER_LOW), C::PRIMARY),
        ButtonVariant::Tonal => (
            CssColor::role(C::SECONDARY_CONTAINER),
            C::ON_SECONDARY_CONTAINER,
        ),
        ButtonVariant::Outlined => (CssColor::transparent(), C::ON_SURFACE_VARIANT),
        ButtonVariant::Text => (CssColor::transparent(), C::PRIMARY),
    };
    (
        bg,
        CssColor::role(fg),
        if variant == ButtonVariant::Outlined {
            CssColor::role(C::OUTLINE_VARIANT)
        } else {
            CssColor::transparent()
        },
    )
}
fn resting_radius(size: ButtonSize, shape: ButtonShape, selected: bool) -> f32 {
    let g = size.geometry();
    if (shape == ButtonShape::Round) != selected {
        g.height / 2.
    } else {
        g.square
    }
}

fn toggle_color_targets(
    variant: ButtonVariant,
    disabled: bool,
    toggle: bool,
    selected: bool,
) -> (CssColor, CssColor, CssColor) {
    use ColorRole as C;
    if !toggle {
        return color_targets(variant, disabled);
    }
    if disabled {
        let (_, fg, border) = color_targets(variant, true);
        return (
            if variant == ButtonVariant::Outlined {
                CssColor::transparent()
            } else {
                CssColor::alpha_role(C::ON_SURFACE, 0.10)
            },
            fg,
            border,
        );
    }
    let roles = match (variant, selected) {
        (ButtonVariant::Filled, false) => Some((C::SURFACE_CONTAINER, C::ON_SURFACE_VARIANT)),
        (ButtonVariant::Filled | ButtonVariant::Elevated, true) => {
            Some((C::PRIMARY, C::ON_PRIMARY))
        }
        (ButtonVariant::Tonal, true) => Some((C::SECONDARY, C::ON_SECONDARY)),
        (ButtonVariant::Outlined, true) => Some((C::INVERSE_SURFACE, C::INVERSE_ON_SURFACE)),
        _ => None,
    };
    match roles {
        Some((bg, fg)) => (
            CssColor::role(bg),
            CssColor::role(fg),
            CssColor::transparent(),
        ),
        None => color_targets(variant, false),
    }
}

#[cfg(test)]
fn colors(variant: ButtonVariant, disabled: bool) -> (Rgba, Rgba, Rgba) {
    let (bg, fg, border) = color_targets(variant, disabled);
    (bg.resolve(), fg.resolve(), border.resolve())
}

/// Passive visual state. Base remains the only activation authority.
struct PressState {
    pointer: bool,
    hovered: bool,
    caller_hovered: bool,
    native_hovered: bool,
    outside_window: bool,
    keyboard: bool,
    focus: FocusHandle,
    focus_out: Option<Subscription>,
    _activation: Subscription,
}
fn clear_press(state: &mut PressState) {
    state.pointer = false;
    state.hovered = false;
    state.outside_window = true;
    state.keyboard = false;
}
fn press_state(
    id: ElementId,
    focus: &FocusHandle,
    window: &mut Window,
    cx: &mut App,
) -> Entity<PressState> {
    let state = window.use_keyed_state((id, "material-press"), cx, |window, cx| {
        let activation =
            cx.observe_window_activation(window, |state: &mut PressState, window, cx| {
                if !window.is_window_active() {
                    clear_press(state);
                    cx.notify();
                }
            });
        PressState {
            pointer: false,
            hovered: false,
            caller_hovered: false,
            native_hovered: false,
            outside_window: false,
            keyboard: false,
            focus: focus.clone(),
            focus_out: None,
            _activation: activation,
        }
    });
    if state.read(cx).focus_out.is_none() || state.read(cx).focus != *focus {
        let weak = state.downgrade();
        let subscription = window.on_focus_out(focus, cx, move |_, _, cx| {
            let _ = weak.update(cx, |state, cx| {
                state.keyboard = false;
                cx.notify();
            });
        });
        state.update(cx, |state, _| {
            state.keyboard = false;
            state.focus = focus.clone();
            state.focus_out = Some(subscription);
        });
    }
    state
}
fn observe_press(
    mut base: BaseControl,
    state: &Entity<PressState>,
    focus: &FocusHandle,
    disabled: bool,
    focusable: bool,
) -> BaseControl {
    if disabled {
        return base;
    }
    let down = state.clone();
    base = base.on_mouse_down(MouseButton::Left, move |_, window, cx| {
        if !window.default_prevented() {
            down.update(cx, |state, cx| {
                state.outside_window = false;
                state.pointer = true;
                state.hovered = !window.last_input_was_touch();
                cx.notify();
            });
        }
    });
    let up = state.clone();
    let out = state.clone();
    base = base
        .on_mouse_up(MouseButton::Left, move |_, window, cx| {
            up.update(cx, |state, cx| {
                state.pointer = false;
                state.hovered = !window.last_input_was_touch();
                cx.notify();
            });
        })
        .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
            out.update(cx, |state, cx| {
                state.hovered = false;
                state.pointer = false;
                cx.notify();
            });
        });
    if focusable {
        let down = state.clone();
        let up = state.clone();
        let focus = focus.clone();
        base = base
            .on_key_down(move |event, window, cx| {
                let stroke = &event.keystroke;
                let pressed = !window.default_prevented()
                    && focus.is_focused(window)
                    && !stroke.modifiers.modified()
                    && matches!(stroke.key.as_str(), "enter" | "space");
                down.update(cx, |state, cx| {
                    state.keyboard = pressed;
                    cx.notify();
                });
            })
            .on_key_up(move |event, _, cx| {
                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                    up.update(cx, |state, cx| {
                        state.keyboard = false;
                        cx.notify();
                    });
                }
            });
    }
    base
}

type HoverCallback = std::rc::Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// Sized, shaped Material presentation; activation, keyboard and focus stay in Base.
#[derive(IntoElement)]
pub struct MaterialButton {
    #[cfg(test)]
    radius_observer: Option<std::rc::Rc<std::cell::Cell<f32>>>,
    #[cfg(test)]
    colors_observer: Option<std::rc::Rc<std::cell::Cell<(Rgba, Rgba, Rgba)>>>,
    hover_callback: Option<HoverCallback>,
    caller_hover_mode: HoverListenerMode,
    #[cfg(test)]
    effects_observer: Option<std::rc::Rc<std::cell::RefCell<(f32, shadow::ShadowPair)>>>,
    id: ElementId,
    focus_handle: Option<FocusHandle>,
    focusable: bool,
    base: BaseControl,
    children: Vec<AnyElement>,
    icon: Option<Icon>,
    icon_position: ButtonIconPosition,
    direction: ButtonDirection,
    variant: ButtonVariant,
    size: ButtonSize,
    shape: ButtonShape,
    disabled: bool,
    ripple: bool,
    style: StyleRefinement,
}
impl MaterialButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            #[cfg(test)]
            radius_observer: None,
            #[cfg(test)]
            colors_observer: None,
            hover_callback: None,
            caller_hover_mode: HoverListenerMode::InputModalityAware,
            #[cfg(test)]
            effects_observer: None,
            id: id.clone(),
            focus_handle: None,
            focusable: true,
            base: BaseControl::new(id),
            children: vec![],
            icon: None,
            icon_position: ButtonIconPosition::Leading,
            direction: ButtonDirection::Ltr,
            variant: ButtonVariant::Filled,
            size: ButtonSize::Small,
            shape: ButtonShape::Round,
            disabled: false,
            ripple: true,
            style: StyleRefinement::default(),
        }
    }
    /// Enable controlled toggle semantics and Material selected presentation.
    pub fn toggle(mut self, enabled: bool) -> Self {
        self.base.toggle = enabled;
        self
    }
    /// Set the application-owned selection. This does not emit a change request.
    pub fn selected(mut self, selected: bool) -> Self {
        self.base.selected = selected;
        self
    }
    /// Allow activation to request a selection change (enabled by default).
    /// Disable this when a containing group owns selection; `on_click` still runs.
    pub fn toggle_on_click(mut self, enabled: bool) -> Self {
        self.base.toggle_on_click = enabled;
        self
    }
    /// Handle Base's request to change selection. The caller must update the
    /// value supplied to `selected`; the button never stores selection itself.
    pub fn on_change(
        mut self,
        callback: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.base.on_change = Some(std::rc::Rc::new(callback));
        self
    }
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Set a decorative icon; the button supplies its reference slot size.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    pub fn icon_position(mut self, position: ButtonIconPosition) -> Self {
        self.icon_position = position;
        self
    }
    pub fn direction(mut self, direction: ButtonDirection) -> Self {
        self.direction = direction;
        self
    }
    pub fn button_size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }
    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self.base = self.base.disabled(disabled);
        self
    }
    /// Enable the pointer ripple (enabled by default).
    pub fn ripple(mut self, enabled: bool) -> Self {
        self.ripple = enabled;
        self
    }
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.base = self.base.accessibility_label(label);
        self
    }
    pub fn on_click(
        mut self,
        callback: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.base = self.base.on_click(callback);
        self
    }
    pub fn track_focus(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self.base = self.base.track_focus(handle);
        self
    }
    pub fn tab_index(mut self, index: isize) -> Self {
        self.base = self.base.tab_index(index);
        self
    }
    pub fn tab_stop(mut self, stop: bool) -> Self {
        self.base = self.base.tab_stop(stop);
        self
    }
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self.base = self.base.focusable(focusable);
        self
    }
}
impl Styled for MaterialButton {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl ParentElement for MaterialButton {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}
impl InteractiveElement for MaterialButton {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }
}
impl StatefulInteractiveElement for MaterialButton {
    fn hover_listener_mode(mut self, mode: HoverListenerMode) -> Self {
        self.caller_hover_mode = mode;
        self
    }
    fn on_hover(mut self, listener: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.hover_callback = Some(std::rc::Rc::new(listener));
        self
    }
}
impl RenderOnce for MaterialButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use gpui_base::StyledExt;
        let g = self.size.geometry();
        let ty = g.typography.scale();
        let (bg_target, fg_target, border_target) = toggle_color_targets(
            self.variant,
            self.disabled,
            self.base.toggle,
            self.base.selected,
        );
        // Disabled icon CSS is an explicit child color without a transition.
        let disabled_icon_color = fg_target.resolve();
        let bg = motion::token_transition(
            (self.id.clone(), "background-color"),
            bg_target,
            MotionToken::DefaultEffects,
            window,
            cx,
        )
        .resolve();
        let fg = motion::token_transition(
            (self.id.clone(), "foreground-color"),
            fg_target,
            MotionToken::DefaultEffects,
            window,
            cx,
        )
        .resolve();
        let border = motion::token_transition(
            (self.id.clone(), "border-color"),
            border_target,
            MotionToken::DefaultEffects,
            window,
            cx,
        )
        .resolve();
        let radius = resting_radius(
            self.size,
            self.shape,
            self.base.toggle && self.base.selected,
        );
        #[cfg(test)]
        if let Some(observer) = &self.colors_observer {
            observer.set((bg, fg, border));
        }
        let focus = self.focus_handle.unwrap_or_else(|| {
            window
                .use_keyed_state((self.id.clone(), "material-focus"), cx, |_, cx| {
                    cx.focus_handle()
                })
                .read(cx)
                .clone()
        });
        let pressed = press_state(self.id.clone(), &focus, window, cx);
        if self.disabled || !self.focusable || !focus.is_focused(window) {
            pressed.update(cx, |state, _| {
                if self.disabled {
                    state.pointer = false;
                }
                state.keyboard = false;
            });
        }
        let active = pressed.read(cx).pointer || pressed.read(cx).keyboard;
        let hovered = pressed.read(cx).hovered && !window.last_input_was_touch();
        let focused =
            self.focusable && focus.is_focused(window) && window.last_input_was_keyboard();
        let opacity = gpui_base::transition(
            (self.id.clone(), "state-opacity"),
            if self.disabled {
                0.
            } else if (active && !self.ripple) || focused {
                0.10
            } else if hovered {
                0.08
            } else {
                0.
            },
            gpui_base::Transition::new(std::time::Duration::from_millis(100))
                .reduced_motion(gpui_base::ReducedMotion::Continue)
                .easing(
                    gpui_base::Easing::cubic_bezier(0.2, 0., 0., 1.)
                        .expect("valid Material easing"),
                ),
            window,
            cx,
        );
        let level = if self.disabled {
            0
        } else if self.variant == ButtonVariant::Elevated {
            if hovered && !active { 2 } else { 1 }
        } else if !self.base.toggle
            && hovered
            && !active
            && matches!(self.variant, ButtonVariant::Filled | ButtonVariant::Tonal)
        {
            1
        } else {
            0
        };
        let shadows = motion::token_transition(
            (self.id.clone(), "box-shadow"),
            shadow::ShadowPair::level(level),
            MotionToken::DefaultEffects,
            window,
            cx,
        );
        #[cfg(test)]
        if let Some(observer) = &self.effects_observer {
            *observer.borrow_mut() = (opacity, shadows.clone());
        }
        let radius = motion::token_value(
            (self.id.clone(), "pressed-radius"),
            if pressed.read(cx).pointer || pressed.read(cx).keyboard {
                g.pressed
            } else {
                radius
            },
            MotionToken::FastSpatial,
            window,
            cx,
        );
        #[cfg(test)]
        if let Some(observer) = &self.radius_observer {
            observer.set(radius);
        }
        let (left_padding, right_padding) = horizontal_padding(
            self.size,
            self.variant,
            self.icon.as_ref().map(|_| self.icon_position),
            self.direction,
        );
        let slot = self.icon.map(|icon| {
            let icon = if self.disabled {
                icon.color(disabled_icon_color)
            } else {
                icon
            };
            div()
                .flex()
                .items_center()
                .justify_center()
                .flex_shrink_0()
                .size(rem(self.size.icon_size()))
                .child(icon.size(rem(self.size.icon_size())))
        });
        let (leading, trailing) = match self.icon_position {
            ButtonIconPosition::Leading => (slot, None),
            ButtonIconPosition::Trailing => (None, slot),
        };
        let weak = pressed.downgrade();
        // A window exit must clear a held press even after the pointer has left
        // the button. Its non-occluding hitbox only locates passive touch presses.
        let touch_enabled = !self.disabled;
        let exit_observer = canvas(
            |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
            move |bounds, hitbox, window, _| {
                let touch = weak.clone();
                window.observe_touch_event(move |event, window, cx| match event.phase {
                    TouchPhase::Started
                        if touch_enabled && hitbox.is_hovered_at(event.position, window) =>
                    {
                        let _ = touch.update(cx, |state, cx| {
                            state.pointer = true;
                            state.hovered = false;
                            state.outside_window = false;
                            cx.notify();
                        });
                    }
                    TouchPhase::Ended | TouchPhase::Cancelled => {
                        let _ = touch.update(cx, |state, cx| {
                            if state.pointer {
                                state.pointer = false;
                                state.hovered = false;
                                cx.notify();
                            }
                        });
                    }
                    _ => {}
                });
                let moving = weak.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                    if phase == DispatchPhase::Capture {
                        let _ = moving.update(cx, |state, cx| {
                            if state.pointer || state.outside_window {
                                state.outside_window = false;
                                let hovered = bounds.contains(&event.position)
                                    && !window.last_input_was_touch();
                                if state.hovered != hovered {
                                    state.hovered = hovered;
                                    cx.notify();
                                }
                            }
                        });
                    }
                });
                window.on_mouse_event(move |_: &MouseExitEvent, phase, _, cx| {
                    if phase == DispatchPhase::Capture {
                        let _ = weak.update(cx, |state, cx| {
                            // A deferred native hover check can still see the last
                            // in-window mouse position after exit. Keep it suppressed
                            // until an actual pointer event enters the window again.
                            state.outside_window = true;
                            if state.pointer || state.hovered {
                                state.hovered = false;
                                state.pointer = false;
                                cx.notify();
                            }
                        });
                    }
                });
            },
        )
        .absolute()
        .size_full();
        let base = self
            .base
            .track_focus(&focus)
            .children(leading)
            .children(self.children)
            .children(trailing)
            .child(exit_observer);
        let base = observe_press(base, &pressed, &focus, self.disabled, self.focusable);
        let hover_state = pressed.clone();
        let callback = self.hover_callback;
        let caller_mode = self.caller_hover_mode;
        // CSS :hover is independent of keyboard modality. Preserve the caller's
        // GPUI modality preference separately, sharing the one native listener.
        if let Some(callback) = callback.clone() {
            let caller_hovered = pressed.read(cx).native_hovered
                && !pressed.read(cx).outside_window
                && (!window.last_input_was_keyboard()
                    || matches!(caller_mode, HoverListenerMode::InputModalityIndependent));
            if caller_hovered != pressed.read(cx).caller_hovered {
                pressed.update(cx, |state, _| state.caller_hovered = caller_hovered);
                let weak = pressed.downgrade();
                window.defer(cx, move |window, cx| {
                    if let Some(state) = weak.upgrade()
                        && state.read(cx).caller_hovered == caller_hovered
                    {
                        callback(&caller_hovered, window, cx);
                    }
                });
            }
        }
        let base = base
            .hover_listener_mode(HoverListenerMode::InputModalityIndependent)
            .on_hover(move |hovered, window, cx| {
                let caller_hovered = *hovered
                    && !hover_state.read(cx).outside_window
                    && (!window.last_input_was_keyboard()
                        || matches!(caller_mode, HoverListenerMode::InputModalityIndependent));
                let changed = hover_state.update(cx, |state, cx| {
                    // GPUI reports hover=false during a held mouse down; the passive
                    // move observer maintains pointer location for that interval.
                    if !state.pointer {
                        let hovered =
                            *hovered && !state.outside_window && !window.last_input_was_touch();
                        if state.hovered != hovered {
                            state.hovered = hovered;
                            cx.notify();
                        }
                    }
                    state.native_hovered = *hovered;
                    let changed = state.caller_hovered != caller_hovered;
                    state.caller_hovered = caller_hovered;
                    changed
                });
                if changed && let Some(callback) = &callback {
                    callback(&caller_hovered, window, cx);
                }
            });
        let mut base = base
            .flex()
            .flex_row()
            .when(self.direction == ButtonDirection::Rtl, |b| {
                b.flex_row_reverse()
            })
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .min_w(rem(58.))
            .h(rem(g.height))
            .pl(rem(left_padding))
            .pr(rem(right_padding))
            .gap(rem(g.gap))
            .rounded(rem(radius))
            .bg(layer(bg, fg, opacity))
            .text_color(fg)
            .text_size(ty.font_size)
            .line_height(ty.line_height)
            .font_weight(FontWeight(ty.font_weight as f32))
            .when(self.variant == ButtonVariant::Outlined, |b| {
                b.border(rem(g.outline)).border_color(border)
            })
            .shadow(shadows.into_shadows())
            .when(!self.disabled, |b| b.cursor_pointer())
            .refine_style(&self.style);
        let mut painted_style = Style::default();
        painted_style.refine(base.style());
        let mut ripple_color = painted_style.text.color.map(Rgba::from).unwrap_or(fg);
        // Base's opacity scope ends after child.paint; apply the same local
        // multiplier to the overlay (ancestor opacity remains in Window).
        ripple_color.a *= painted_style.opacity.unwrap_or(1.);
        let ripple_visible = painted_style.visibility == Visibility::Visible;
        if !ripple_visible {
            ripple_color.a = 0.;
        }
        let base = ripple::ButtonRipple::new(
            base,
            self.id,
            self.ripple,
            !self.disabled && ripple_visible,
            painted_style.corner_radii,
            painted_style.border_widths,
            ripple_color,
            window,
            cx,
        );
        focus_ring::ButtonFocusRing::new(base, focus, !self.disabled && self.focusable, rem(radius))
    }
}
/// Convenience constructor with caller-supplied content and activation.
pub fn material_button(id: impl Into<ElementId>, variant: ButtonVariant) -> MaterialButton {
    MaterialButton::new(id).variant(variant)
}

#[cfg(test)]
mod tests {
    use super::{ButtonSize, ButtonVariant, MaterialButton, alpha, colors, layer};
    use crate::{color::ColorRole, theme::color};
    use gpui::{
        Context, IntoElement, Modifiers, Render, TestAppContext, VisualTestContext, Window, div,
        prelude::*, px, rgba,
    };

    type PaintedColors = (gpui::Rgba, gpui::Rgba, gpui::Rgba);

    struct EffectsHarness {
        disabled: bool,
        ripple: bool,
        sample: std::rc::Rc<std::cell::RefCell<(f32, super::shadow::ShadowPair)>>,
        focus: gpui::FocusHandle,
        hovers: Vec<bool>,
        hover_mode: gpui::HoverListenerMode,
        activations: usize,
    }
    impl Render for EffectsHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            crate::motion::apply_scheme(crate::motion::MotionScheme::Expressive);
            let mut button = MaterialButton::new("effects")
                .ripple(self.ripple)
                .disabled(self.disabled)
                .track_focus(&self.focus)
                .debug_selector(|| "effects".into())
                .child("Effects")
                .on_click(cx.listener(|this, _, _, _| this.activations += 1));
            button = StatefulInteractiveElement::on_hover(
                button,
                cx.listener(|this, hovered, _, _| this.hovers.push(*hovered)),
            );
            button = StatefulInteractiveElement::hover_listener_mode(button, self.hover_mode);
            button.effects_observer = Some(self.sample.clone());
            div().p(px(30.)).child(button)
        }
    }
    #[gpui::test]
    fn default_ripple_suppresses_pressed_layer_but_retains_hover_and_keyboard_focus(
        cx: &mut TestAppContext,
    ) {
        use super::shadow::ShadowPair;
        use gpui::{MouseButton, MouseMoveEvent};
        use std::{cell::RefCell, rc::Rc, time::Duration};
        cx.update(gpui_base::init);
        assert!(MaterialButton::new("default").ripple);
        let sample = Rc::new(RefCell::new((0., ShadowPair::level(0))));
        let (_, cx) = cx.add_window_view({
            let sample = sample.clone();
            move |_, cx| EffectsHarness {
                disabled: false,
                ripple: true,
                sample,
                focus: cx.focus_handle(),
                hovers: vec![],
                hover_mode: gpui::HoverListenerMode::InputModalityAware,
                activations: 0,
            }
        });
        draw(cx);
        let center = cx.debug_bounds("effects").unwrap().center();
        cx.simulate_event(MouseMoveEvent {
            position: center,
            pressed_button: None,
            modifiers: Modifiers::default(),
        });
        cx.run_until_parked();
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        assert_eq!(sample.borrow().0, 0.08);
        cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        assert_eq!(
            sample.borrow().0,
            0.08,
            "a pointer wave coexists with hover, without adding the pressed state layer"
        );
        cx.simulate_mouse_up(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        key(cx, "space", true);
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        assert_eq!(
            sample.borrow().0,
            0.1,
            "keyboard focus keeps its separate state layer"
        );
        key(cx, "space", false);
    }
    #[gpui::test]
    fn state_opacity_and_shadow_have_independent_event_clocks(cx: &mut TestAppContext) {
        use super::shadow::ShadowPair;
        use gpui::{MouseButton, MouseMoveEvent, point};
        use gpui_base::Interpolate;
        use std::{cell::RefCell, rc::Rc, time::Duration};
        cx.update(gpui_base::init);
        let sample = Rc::new(RefCell::new((0., ShadowPair::level(0))));
        let (view, cx) = cx.add_window_view({
            let sample = sample.clone();
            move |_, cx| EffectsHarness {
                ripple: false,
                disabled: false,
                sample,
                focus: cx.focus_handle(),
                hovers: vec![],
                hover_mode: gpui::HoverListenerMode::InputModalityAware,
                activations: 0,
            }
        });
        draw(cx);
        let center = cx.debug_bounds("effects").unwrap().center();
        cx.simulate_event(MouseMoveEvent {
            position: center,
            pressed_button: None,
            modifiers: Modifiers::default(),
        });
        draw(cx);
        cx.run_until_parked();
        draw(cx);
        assert_eq!(sample.borrow().0, 0.);
        cx.executor().advance_clock(Duration::from_millis(50));
        draw(cx);
        // Independently sampled from the CSS reference in Chromium.
        assert!((sample.borrow().0 - 0.0702267).abs() < 0.00001);
        cx.executor().advance_clock(Duration::from_millis(50));
        draw(cx);
        assert_eq!(sample.borrow().0, 0.08);
        let expected = ShadowPair::level(0).interpolate(
            &ShadowPair::level(1),
            crate::motion::MotionScheme::Expressive
                .resolve(crate::motion::MotionToken::DefaultEffects)
                .ease()(0.4),
        );
        assert_eq!(sample.borrow().1, expected);
        assert_ne!(sample.borrow().1, ShadowPair::level(1));
        cx.executor().advance_clock(Duration::from_millis(150));
        draw(cx);
        assert_eq!(sample.borrow().1, ShadowPair::level(1));
        // A stationary pointer stays hovered across disable/re-enable, even
        // though the native hover listener need not emit another true event.
        view.update(cx, |view, cx| {
            view.disabled = true;
            cx.notify();
        });
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(250));
        draw(cx);
        assert_eq!(*sample.borrow(), (0., ShadowPair::level(0)));
        view.update(cx, |view, cx| {
            view.disabled = false;
            cx.notify();
        });
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(250));
        draw(cx);
        assert_eq!(*sample.borrow(), (0.08, ShadowPair::level(1)));
        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        cx.simulate_mouse_move(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(50));
        draw(cx);
        assert!(
            (sample.borrow().0 - 0.0975567).abs() < 0.00001,
            "reduced motion preserves CSS state fade"
        );
        assert_ne!(
            sample.borrow().1,
            ShadowPair::level(0),
            "reduced motion preserves CSS shadow transition"
        );
        cx.executor().advance_clock(Duration::from_millis(50));
        draw(cx);
        assert_eq!(sample.borrow().0, 0.10);
        cx.simulate_mouse_up(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(250));
        draw(cx);
        assert_eq!(
            sample.borrow().0,
            0.08,
            "held move's GPUI hover=false must not lose hover on release"
        );
        assert_eq!(sample.borrow().1, ShadowPair::level(1));
        cx.update(|_, cx| {
            assert!(!view.read(cx).hovers.is_empty());
            assert_eq!(view.read(cx).activations, 1);
        });
        cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        cx.simulate_mouse_up(
            point(px(900.), px(800.)),
            MouseButton::Left,
            Modifiers::default(),
        );
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(250));
        draw(cx);
        assert_eq!(*sample.borrow(), (0., ShadowPair::level(0)));
        // Pointer focus becomes visible only when keyboard modality takes over.
        key(cx, "a", true);
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        assert_eq!(sample.borrow().0, 0.10);
        view.update(cx, |view, cx| {
            view.disabled = true;
            cx.notify();
        });
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(250));
        draw(cx);
        assert_eq!(*sample.borrow(), (0., ShadowPair::level(0)));
        view.update(cx, |view, cx| {
            view.disabled = false;
            cx.notify();
        });
        cx.update(|window, cx| {
            window.blur(cx);
            window.activate_window();
        });
        cx.run_until_parked();
        cx.simulate_event(MouseMoveEvent {
            position: center,
            pressed_button: None,
            modifiers: Modifiers::default(),
        });
        draw(cx);
        cx.run_until_parked();
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(250));
        draw(cx);
        assert_eq!(*sample.borrow(), (0.08, ShadowPair::level(1)));
        cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        assert_eq!(sample.borrow().0, 0.10);
        cx.deactivate_window();
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(250));
        for _ in 0..3 {
            cx.run_until_parked();
            draw(cx);
            assert_eq!(
                *sample.borrow(),
                (0., ShadowPair::level(0)),
                "deactivation must not revive hover from the cached pointer position"
            );
            cx.update(|_, cx| assert_eq!(view.read(cx).hovers.last(), Some(&false)));
        }
        cx.simulate_mouse_up(
            point(px(900.), px(800.)),
            MouseButton::Left,
            Modifiers::default(),
        );
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        cx.simulate_event(MouseMoveEvent {
            position: center,
            pressed_button: None,
            modifiers: Modifiers::default(),
        });
        draw(cx);
        cx.run_until_parked();
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(250));
        draw(cx);
        assert_eq!(
            *sample.borrow(),
            (0.08, ShadowPair::level(1)),
            "a real pointer move after activation restores hover"
        );
        cx.update(|_, cx| assert_eq!(view.read(cx).hovers.last(), Some(&true)));
    }

    #[gpui::test]
    fn keyboard_hover_modes_preserve_caller_events_and_material_elevation(cx: &mut TestAppContext) {
        use super::shadow::ShadowPair;
        use gpui::{HoverListenerMode, MouseMoveEvent, point};
        use std::{cell::RefCell, rc::Rc, time::Duration};
        cx.update(gpui_base::init);
        let sample = Rc::new(RefCell::new((0., ShadowPair::level(0))));
        let (view, cx) = cx.add_window_view({
            let sample = sample.clone();
            move |_, cx| EffectsHarness {
                ripple: false,
                disabled: false,
                sample,
                focus: cx.focus_handle(),
                hovers: vec![],
                hover_mode: HoverListenerMode::InputModalityAware,
                activations: 0,
            }
        });
        draw(cx);
        let center = cx.debug_bounds("effects").unwrap().center();
        cx.simulate_event(MouseMoveEvent {
            position: center,
            pressed_button: None,
            modifiers: Modifiers::default(),
        });
        draw(cx);
        cx.run_until_parked();
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(250));
        draw(cx);
        assert_eq!(*sample.borrow(), (0.08, ShadowPair::level(1)));
        cx.update(|window, cx| {
            let focus = view.read(cx).focus.clone();
            window.focus(&focus, cx);
        });
        key(cx, "a", true);
        draw(cx);
        cx.run_until_parked();
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        assert_eq!(*sample.borrow(), (0.10, ShadowPair::level(1)));
        cx.update(|_, cx| assert_eq!(view.read(cx).hovers, [true, false]));
        view.update(cx, |view, cx| {
            view.hover_mode = HoverListenerMode::InputModalityIndependent;
            cx.notify();
        });
        draw(cx);
        cx.run_until_parked();
        draw(cx);
        cx.update(|_, cx| assert_eq!(view.read(cx).hovers, [true, false, true]));
        key(cx, "b", true);
        draw(cx);
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(view.read(cx).hovers, [true, false, true]));
        cx.simulate_event(gpui::TouchEvent {
            id: gpui::TouchId(1),
            phase: gpui::TouchPhase::Started,
            position: center,
            ..Default::default()
        });
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(250));
        draw(cx);
        assert_eq!(
            *sample.borrow(),
            (0.1, ShadowPair::level(0)),
            "without ripple, held touch uses its pressed layer and drops hover elevation"
        );
        cx.simulate_event(gpui::TouchEvent {
            id: gpui::TouchId(1),
            phase: gpui::TouchPhase::Ended,
            position: center,
            ..Default::default()
        });
        cx.update(|window, cx| window.blur(cx));
        cx.simulate_event(gpui::MouseExitEvent {
            position: point(px(1200.), px(1000.)),
            pressed_button: None,
            modifiers: Modifiers::default(),
        });
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(250));
        draw(cx);
        assert_eq!(*sample.borrow(), (0., ShadowPair::level(0)));
        cx.run_until_parked();
        draw(cx);
        cx.run_until_parked();
        draw(cx);
        assert_eq!(*sample.borrow(), (0., ShadowPair::level(0)));
        cx.update(|_, cx| assert_eq!(view.read(cx).hovers, [true, false, true, false]));
    }

    struct ColorsHarness {
        variant: ButtonVariant,
        disabled: bool,
        palette: crate::color::PaletteId,
        sample: std::rc::Rc<std::cell::Cell<PaintedColors>>,
    }

    impl Render for ColorsHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            crate::motion::apply_scheme(crate::motion::MotionScheme::Expressive);
            crate::theme::apply_palette(self.palette, window, cx);
            let mut button = MaterialButton::new("colors-button")
                .variant(self.variant)
                .disabled(self.disabled)
                .child("Colors");
            button.colors_observer = Some(self.sample.clone());
            button
        }
    }

    fn assert_colors_close(actual: PaintedColors, expected: PaintedColors) {
        for (actual, expected) in [
            (actual.0, expected.0),
            (actual.1, expected.1),
            (actual.2, expected.2),
        ] {
            for (actual, expected) in [
                (actual.r, expected.r),
                (actual.g, expected.g),
                (actual.b, expected.b),
                (actual.a, expected.a),
            ] {
                assert!(
                    (actual - expected).abs() < 0.00001,
                    "{actual} != {expected}"
                );
            }
        }
    }

    #[gpui::test]
    fn rendered_button_colors_keep_state_timing_across_theme_changes(cx: &mut TestAppContext) {
        use crate::color::PaletteId;
        use std::{cell::Cell, rc::Rc, time::Duration};
        cx.update(gpui_base::init);
        let sample = Rc::new(Cell::new((rgba(0), rgba(0), rgba(0))));
        let (view, cx) = cx.add_window_view({
            let sample = sample.clone();
            move |_, _| ColorsHarness {
                variant: ButtonVariant::Filled,
                disabled: false,
                palette: PaletteId::Paper,
                sample,
            }
        });
        cx.update(|window, cx| {
            cx.set_reduce_motion(true);
            window.draw(cx).clear(cx);
            cx.set_reduce_motion(false);
        });
        let initial = sample.get();
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.variant = ButtonVariant::Tonal);
            window.draw(cx).clear(cx);
        });
        assert_colors_close(sample.get(), initial);
        cx.executor().advance_clock(Duration::from_millis(125));
        draw(cx);
        let filled = colors(ButtonVariant::Filled, false);
        let tonal = colors(ButtonVariant::Tonal, false);
        assert_colors_close(
            sample.get(),
            (
                crate::theme::blend(filled.0, tonal.0, 0.96),
                crate::theme::blend(filled.1, tonal.1, 0.96),
                tonal.2,
            ),
        );
        let midpoint = sample.get();
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.palette = PaletteId::Ink);
            window.draw(cx).clear(cx);
        });
        assert_colors_close(sample.get(), midpoint);
        cx.executor().advance_clock(Duration::from_millis(125));
        draw(cx);
        assert_ne!(
            crate::theme::scheme(),
            crate::color::Scheme::from_palette(PaletteId::Paper)
        );
        assert_colors_close(sample.get(), colors(ButtonVariant::Tonal, false));

        cx.update(|window, cx| {
            view.update(cx, |view, _| view.disabled = true);
            window.draw(cx).clear(cx);
        });
        cx.executor().advance_clock(Duration::from_millis(50));
        draw(cx);
        let disabling = sample.get();
        assert!(disabling.0.a > 0.10 && disabling.0.a < 1.);
        assert!(disabling.1.a > 0.38 && disabling.1.a < 1.);
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.disabled = false);
            window.draw(cx).clear(cx);
        });
        assert_colors_close(sample.get(), disabling);
        cx.executor().advance_clock(Duration::from_millis(250));
        draw(cx);
        assert_colors_close(sample.get(), colors(ButtonVariant::Tonal, false));
        cx.update(|window, cx| {
            cx.set_reduce_motion(true);
            view.update(cx, |view, _| view.disabled = true);
            window.draw(cx).clear(cx);
        });
        assert_colors_close(sample.get(), colors(ButtonVariant::Tonal, false));
        cx.executor().advance_clock(Duration::from_millis(250));
        draw(cx);
        assert_colors_close(sample.get(), colors(ButtonVariant::Tonal, true));
    }

    #[test]
    fn transparent_state_layers_preserve_content_rgb_and_opacity() {
        let content = rgba(0x6750a4ff);
        let hovered = layer(rgba(0), content, 0.08);
        for (a, b) in [
            (hovered.r, content.r),
            (hovered.g, content.g),
            (hovered.b, content.b),
        ] {
            assert!((a - b).abs() < 1e-6);
        }
        assert_eq!(hovered.a, 0.08);
        let filled = layer(rgba(0xffffffff), content, 0.10);
        assert_eq!(filled.a, 1.);
        assert!((filled.r - (1. * 0.9 + content.r * 0.1)).abs() < 0.00001);
    }
    #[test]
    fn disabled_buttons_use_independent_content_container_and_outline_roles() {
        for variant in ButtonVariant::ALL {
            let (bg, fg, border) = colors(variant, true);
            assert_eq!(fg, alpha(color(ColorRole::ON_SURFACE_VARIANT), 0.38));
            assert_eq!(
                bg.a,
                if matches!(variant, ButtonVariant::Outlined | ButtonVariant::Text) {
                    0.
                } else {
                    0.10
                }
            );
            assert_eq!(
                border.a,
                if variant == ButtonVariant::Outlined {
                    0.10
                } else {
                    0.
                }
            );
        }
        assert_eq!(
            colors(ButtonVariant::Text, false).1,
            color(ColorRole::PRIMARY)
        );
        assert_eq!(
            colors(ButtonVariant::Outlined, false).1,
            color(ColorRole::ON_SURFACE_VARIANT)
        );
        assert_eq!(
            colors(ButtonVariant::Outlined, false).2,
            color(ColorRole::OUTLINE_VARIANT)
        );
    }

    struct Harness {
        disabled: bool,
        activations: usize,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().flex().flex_col().items_start().gap(px(10.)).children(
                ButtonSize::ALL.into_iter().map(|size| {
                    MaterialButton::new(size.name())
                        .button_size(size)
                        .debug_selector(move || size.name().into())
                        .disabled(self.disabled)
                        .child("Go")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.activations += 1;
                            cx.notify();
                        }))
                }),
            )
        }
    }
    fn draw(cx: &mut VisualTestContext) {
        cx.update(|w, cx| {
            w.refresh();
            w.draw(cx).clear(cx);
        });
    }
    #[gpui::test]
    fn all_sizes_keep_visual_height_and_base_keyboard_disabled_behavior(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| Harness {
            disabled: false,
            activations: 0,
        });
        draw(cx);
        for (name, height) in [
            ("xs", 32.),
            ("s", 40.),
            ("m", 56.),
            ("l", 96.),
            ("xl", 136.),
        ] {
            let bounds = cx.debug_bounds(name).unwrap();
            assert_eq!(bounds.size.height, px(height));
            assert!(bounds.size.width >= px(58.));
        }
        let location = cx.debug_bounds("s").unwrap().center();
        cx.simulate_click(location, Modifiers::default());
        draw(cx);
        for key in ["enter", "space"] {
            let keystroke = cx.update(|_, cx| {
                gpui::Keystroke::parse_with_os(key, cx.platform_info().os).unwrap()
            });
            cx.simulate_event(gpui::KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            cx.simulate_event(gpui::KeyUpEvent { keystroke });
        }
        draw(cx);
        cx.update(|_, cx| assert_eq!(view.read(cx).activations, 3));
        view.update(cx, |this, cx| {
            this.disabled = true;
            cx.notify();
        });
        draw(cx);
        cx.simulate_click(location, Modifiers::default());
        draw(cx);
        for key in ["enter", "space"] {
            let keystroke = cx.update(|_, cx| {
                gpui::Keystroke::parse_with_os(key, cx.platform_info().os).unwrap()
            });
            cx.simulate_event(gpui::KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            cx.simulate_event(gpui::KeyUpEvent { keystroke });
        }
        draw(cx);
        cx.update(|_, cx| assert_eq!(view.read(cx).activations, 3));
    }
    struct MotionHarness {
        disabled: bool,
        shown: bool,
        sample: std::rc::Rc<std::cell::Cell<f32>>,
        focus: gpui::FocusHandle,
        other: gpui::FocusHandle,
        activations: usize,
    }
    impl Render for MotionHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            crate::motion::apply_scheme(crate::motion::MotionScheme::Expressive);
            let mut button = MaterialButton::new("motion-button")
                .track_focus(&self.focus)
                .button_size(ButtonSize::Large)
                .disabled(self.disabled)
                .child("Hold")
                .debug_selector(|| "motion-button".into())
                .on_click(cx.listener(|this, _, _, _| this.activations += 1));
            button.radius_observer = Some(self.sample.clone());
            div()
                .flex()
                .gap(px(30.))
                .children(self.shown.then_some(button))
                .child(
                    gpui_base::Button::new("other")
                        .track_focus(&self.other)
                        .child("Other"),
                )
        }
    }
    fn key(cx: &mut VisualTestContext, name: &str, down: bool) {
        let keystroke =
            cx.update(|_, cx| gpui::Keystroke::parse_with_os(name, cx.platform_info().os).unwrap());
        if down {
            cx.simulate_event(gpui::KeyDownEvent {
                keystroke,
                is_held: false,
                prefer_character_input: false,
            });
        } else {
            cx.simulate_event(gpui::KeyUpEvent { keystroke });
        }
    }
    #[gpui::test]
    fn raw_touch_animates_held_shape_and_cancellation_preserves_base_activation(
        cx: &mut TestAppContext,
    ) {
        use gpui::{TouchEvent, TouchId, TouchPhase};
        use std::{cell::Cell, rc::Rc, time::Duration};
        cx.update(gpui_base::init);
        let sample = Rc::new(Cell::new(0.));
        let (view, cx) = cx.add_window_view({
            let sample = sample.clone();
            move |_, cx| MotionHarness {
                disabled: false,
                shown: true,
                sample,
                focus: cx.focus_handle(),
                other: cx.focus_handle(),
                activations: 0,
            }
        });
        draw(cx);
        let center = cx.debug_bounds("motion-button").unwrap().center();
        cx.simulate_event(TouchEvent {
            id: TouchId(1),
            phase: TouchPhase::Started,
            position: center,
            ..Default::default()
        });
        draw(cx);
        assert_eq!(sample.get(), 48.);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        let expected = 48.
            + (16. - 48.)
                * crate::motion::MotionScheme::Expressive
                    .resolve(crate::motion::MotionToken::FastSpatial)
                    .ease()(0.1 / 0.425);
        assert!((sample.get() - expected).abs() < 0.001);
        cx.update(|_, cx| assert_eq!(view.read(cx).activations, 0));
        cx.simulate_event(TouchEvent {
            id: TouchId(1),
            phase: TouchPhase::Cancelled,
            position: center,
            ..Default::default()
        });
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(500));
        draw(cx);
        assert_eq!(sample.get(), 48.);
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).activations, 0);
            cx.set_reduce_motion(true);
        });
        cx.simulate_event(TouchEvent {
            id: TouchId(2),
            phase: TouchPhase::Started,
            position: center,
            ..Default::default()
        });
        draw(cx);
        assert_eq!(sample.get(), 16.);
        cx.simulate_event(TouchEvent {
            id: TouchId(2),
            phase: TouchPhase::Ended,
            position: center,
            ..Default::default()
        });
        draw(cx);
        assert_eq!(sample.get(), 48.);
        cx.update(|_, cx| assert_eq!(view.read(cx).activations, 1));
    }
    #[gpui::test]
    fn press_radius_matches_reference_and_retargets_without_jumps(cx: &mut TestAppContext) {
        use gpui::{MouseButton, point};
        use std::{cell::Cell, rc::Rc, time::Duration};
        cx.update(gpui_base::init);
        let sample = Rc::new(Cell::new(0.));
        let (_, cx) = cx.add_window_view({
            let sample = sample.clone();
            move |_, cx| MotionHarness {
                disabled: false,
                shown: true,
                sample,
                focus: cx.focus_handle(),
                other: cx.focus_handle(),
                activations: 0,
            }
        });
        draw(cx);
        let center = cx.debug_bounds("motion-button").unwrap().center();
        assert_eq!(sample.get(), 48.);
        cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        assert_eq!(sample.get(), 48.);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        let expected = 48.
            + (16. - 48.)
                * crate::motion::MotionScheme::Expressive
                    .resolve(crate::motion::MotionToken::FastSpatial)
                    .ease()(0.1 / 0.425);
        assert!((sample.get() - expected).abs() < 0.001);
        let before = sample.get();
        cx.simulate_mouse_move(
            center + point(px(1.), px(1.)),
            MouseButton::Left,
            Modifiers::default(),
        );
        draw(cx);
        assert!(
            (sample.get() - before).abs() < 0.001,
            "held move must not release"
        );
        cx.simulate_mouse_up(
            point(px(900.), px(800.)),
            MouseButton::Left,
            Modifiers::default(),
        );
        draw(cx);
        assert!(
            (sample.get() - before).abs() < 0.001,
            "release must retarget continuously"
        );
        cx.executor().advance_clock(Duration::from_secs(1));
        draw(cx);
        assert_eq!(sample.get(), 48.);
        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        assert_eq!(sample.get(), 16.);
        cx.simulate_mouse_up(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        assert_eq!(sample.get(), 48.);
        cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        cx.simulate_mouse_move(
            point(px(900.), px(800.)),
            MouseButton::Left,
            Modifiers::default(),
        );
        draw(cx);
        assert_eq!(sample.get(), 16.);
        cx.simulate_event(gpui::MouseExitEvent {
            position: point(px(1200.), px(1000.)),
            pressed_button: Some(MouseButton::Left),
            modifiers: Modifiers::default(),
        });
        draw(cx);
        assert_eq!(
            sample.get(),
            48.,
            "leaving window after moving outside must clear press"
        );
    }
    #[gpui::test]
    fn pressed_visuals_clear_on_blur_disable_and_remount(cx: &mut TestAppContext) {
        use gpui::MouseButton;
        use std::{cell::Cell, rc::Rc};
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let sample = Rc::new(Cell::new(0.));
        let (view, cx) = cx.add_window_view({
            let sample = sample.clone();
            move |_, cx| MotionHarness {
                disabled: false,
                shown: true,
                sample,
                focus: cx.focus_handle(),
                other: cx.focus_handle(),
                activations: 0,
            }
        });
        draw(cx);
        cx.update(|window, cx| {
            let handle = view.read(cx).focus.clone();
            window.focus(&handle, cx);
        });
        draw(cx);
        key(cx, "space", true);
        draw(cx);
        assert_eq!(sample.get(), 16.);
        key(cx, "a", false);
        draw(cx);
        assert_eq!(
            sample.get(),
            16.,
            "unrelated key release does not release Space"
        );
        cx.update(|window, cx| {
            let handle = view.read(cx).other.clone();
            window.focus(&handle, cx);
        });
        draw(cx);
        assert_eq!(sample.get(), 48.);
        cx.update(|window, cx| {
            let handle = view.read(cx).focus.clone();
            window.focus(&handle, cx);
        });
        draw(cx);
        key(cx, "space", false);
        draw(cx);
        cx.update(|_, cx| assert_eq!(view.read(cx).activations, 0));
        key(cx, "space", true);
        draw(cx);
        view.update(cx, |this, cx| {
            this.disabled = true;
            cx.notify();
        });
        draw(cx);
        assert_eq!(sample.get(), 48.);
        key(cx, "space", false);
        view.update(cx, |this, cx| {
            this.disabled = false;
            cx.notify();
        });
        draw(cx);
        let center = cx.debug_bounds("motion-button").unwrap().center();
        cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        assert_eq!(sample.get(), 16.);
        view.update(cx, |this, cx| {
            this.shown = false;
            cx.notify();
        });
        draw(cx);
        cx.simulate_mouse_up(center, MouseButton::Left, Modifiers::default());
        view.update(cx, |this, cx| {
            this.shown = true;
            cx.notify();
        });
        draw(cx);
        assert_eq!(sample.get(), 48.);
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        assert_eq!(sample.get(), 16.);
        cx.deactivate_window();
        draw(cx);
        assert_eq!(sample.get(), 48., "window deactivation clears a held press");
    }
    struct IconHarness {
        size: ButtonSize,
        variant: ButtonVariant,
        position: super::ButtonIconPosition,
        direction: super::ButtonDirection,
    }
    impl Render for IconHarness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().flex().items_start().child(
                MaterialButton::new("icon-button")
                    .button_size(self.size)
                    .variant(self.variant)
                    .icon_position(self.position)
                    .direction(self.direction)
                    .icon(crate::icons::IconName::Check.icon().size(px(100.)))
                    .debug_selector(|| "icon-button".into())
                    .child(
                        div()
                            .w(px(40.))
                            .h(px(10.))
                            .debug_selector(|| "icon-label".into()),
                    ),
            )
        }
    }
    #[gpui::test]
    fn icon_slots_and_logical_padding_match_all_sizes_variants_and_directions(
        cx: &mut TestAppContext,
    ) {
        use super::{ButtonDirection, ButtonIconPosition};
        cx.update(gpui_base::init);
        let (view, cx) = cx.add_window_view(|_, _| IconHarness {
            size: ButtonSize::Small,
            variant: ButtonVariant::Filled,
            position: ButtonIconPosition::Leading,
            direction: ButtonDirection::Ltr,
        });
        for (size, icon, gap, pad) in [
            (ButtonSize::ExtraSmall, 20., 4., 12.),
            (ButtonSize::Small, 20., 8., 16.),
            (ButtonSize::Medium, 24., 8., 24.),
            (ButtonSize::Large, 32., 12., 48.),
            (ButtonSize::ExtraLarge, 40., 16., 64.),
        ] {
            for variant in ButtonVariant::ALL {
                for direction in [ButtonDirection::Ltr, ButtonDirection::Rtl] {
                    for position in [ButtonIconPosition::Leading, ButtonIconPosition::Trailing] {
                        view.update(cx, |this, cx| {
                            this.size = size;
                            this.variant = variant;
                            this.position = position;
                            this.direction = direction;
                            cx.notify();
                        });
                        draw(cx);
                        let button = cx.debug_bounds("icon-button").unwrap();
                        let label = cx.debug_bounds("icon-label").unwrap();
                        let special = size == ButtonSize::Small
                            || (size == ButtonSize::ExtraSmall && variant == ButtonVariant::Text);
                        let (icon_side, label_side) = if special { (12., 16.) } else { (pad, pad) };
                        let border = if variant == ButtonVariant::Outlined {
                            match size {
                                ButtonSize::Large => 2.,
                                ButtonSize::ExtraLarge => 3.,
                                _ => 1.,
                            }
                        } else {
                            0.
                        };
                        let expected = icon_side + label_side + icon + gap + 40. + 2. * border;
                        assert!(
                            (f32::from(button.size.width) - expected).abs() < 0.01,
                            "{size:?} {variant:?} {position:?} {direction:?}"
                        );
                        let icon_left = (position == ButtonIconPosition::Leading)
                            == (direction == ButtonDirection::Ltr);
                        let expected_label = if icon_left {
                            icon_side + icon + gap + border
                        } else {
                            label_side + border
                        };
                        assert!(
                            (f32::from(label.left() - button.left()) - expected_label).abs() < 0.01,
                            "label: {size:?} {variant:?} {position:?} {direction:?}"
                        );
                    }
                }
            }
        }
    }
}
