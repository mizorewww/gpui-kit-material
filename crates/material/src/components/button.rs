//! Material button presentation over GPUI Base activation and focus behavior.
//! Reference priority is Figma kit > md3.io > m3e.kyralabs.dev, per value.
//! Sizes, paddings, gaps, icon sizes, corners, outline widths and disabled
//! colors come from the Figma Material 3 Design Kit's Buttons page (Button
//! 57994:2227 and its text/elevated/outline/tonal/toggle sets; fixture
//! `tests/fixtures/figma-button.json`). The kit has no motion, so
//! state-layer, shadow, ripple and color-transition timing follow the md3.io
//! captures of floor/material c910072f3fc5b660276731d8a935d7186f288364
//! (`conformance/reference/button-*.json`). m3e's button tokens
//! (`tests/fixtures/m3e-sys-tokens.json`) are outranked secondary evidence.
//! Controlled toggle and loading presentation over Base behavior.
#[cfg(test)]
mod active_tests;
mod control;
mod playback;
pub use playback::{ButtonTransitionInputs, ButtonTransitionPlayback};
mod effects;
mod focus_ring;
mod form;
pub(crate) use focus_ring::ButtonFocusRing;
pub use form::{FabSize, IconButtonWidth};
pub(crate) use form::{Form, Palette, Roles, Segment, segment_metrics};
mod hit_target;
#[cfg(test)]
mod loading_tests;
#[cfg(test)]
mod preview_tests;
mod ripple;
pub(crate) use ripple::ButtonRipple;
#[cfg(test)]
pub(crate) use ripple::{ripple_writes, waves_painted};
mod ripple_clip;
mod shadow;
#[cfg(test)]
mod toggle_tests;

use super::InteractionPreview;
use control::BaseControl;
use effects::CssColor;

use crate::{
    color::ColorRole,
    icons::Icon,
    motion::{self, MotionToken},
    tokens::{
        state,
        typography::{TypeRole, rem},
    },
};
use gpui::{prelude::*, *};

/// Persistent progress/loading presentation for one logical button.
///
/// Keep this handle across virtual unmounts. Forward actual progress, loading,
/// and disabled changes with [`Self::sync`] while unmounted; do not report a
/// virtual unmount as hiding progress. Clones share playback, not interaction
/// state. A fresh handle starts a new lifecycle.
#[derive(Clone, Default)]
pub struct ButtonProgressPlayback {
    state: std::rc::Rc<std::cell::RefCell<ButtonProgressState>>,
}

#[derive(Default)]
struct ButtonProgressState {
    started: bool,
    visible: bool,
    generation: u64,
    circular: super::progress::CircularProgressPlayback,
    opacity: gpui_base::TransitionPlayback<f32>,
}

impl ButtonProgressPlayback {
    /// Record real input changes at their event time without requesting frames.
    /// `disabled` is the caller's independent flag; loading also disables.
    /// First showing progress starts its circular clock. Hiding preserves that
    /// clock, but the next show adopts the new opacity rather than resuming an
    /// old hidden transition, matching the ordinary button API.
    pub fn sync(&self, progress: bool, loading: bool, disabled: bool, cx: &App) {
        self.sample_inputs(progress, loading, disabled, cx);
    }

    /// Forward a preference change after updating the app, including offscreen.
    /// This does not start unused progress or restart a previously stopped loop.
    /// Opacity retains its existing reduced-motion Continue policy.
    pub fn observe_reduced_motion(&self, cx: &App) {
        self.state.borrow().circular.observe_reduced_motion(cx);
    }

    /// Advance lifecycle and opacity without requesting frames. A started
    /// clock is sampled even while hidden or unmounted, keeping its phase and
    /// reduced-motion stop; only a painted indicator requests frames.
    fn sample_inputs(
        &self,
        progress: bool,
        loading: bool,
        disabled: bool,
        cx: &App,
    ) -> gpui_base::MotionValue<f32> {
        let mut state = self.state.borrow_mut();
        let visible = progress || loading;
        if visible && !state.visible {
            state.generation += 1;
            state.opacity = gpui_base::TransitionPlayback::default();
        }
        state.started |= visible;
        state.visible = visible;
        if state.started {
            state.circular.sample(cx);
        }
        if visible {
            state.opacity.sample(
                if disabled || loading { 0.7 } else { 1. },
                playback::opacity_policy(),
                cx,
            )
        } else {
            gpui_base::MotionValue {
                value: 1.,
                status: gpui_base::MotionStatus::Idle,
            }
        }
    }
}

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
    /// The container height in design px.
    pub(crate) fn height(self) -> f32 {
        self.geometry().height
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
            Self::ExtraLarge => (136., 64., 16., 2., 28., 16., TypeRole::HeadlineLarge),
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

/// Inline padding: the kit pads every variant, with or without an icon, by
/// the size's State-layer padding (12/16/24/48/64), unlike md3.io's compact
/// text and small-icon paddings. Both sides are equal, so logical direction
/// does not change them.
fn horizontal_padding(size: ButtonSize) -> (f32, f32) {
    let padding = size.geometry().padding;
    (padding, padding)
}

#[cfg(test)]
fn alpha(mut color: Rgba, opacity: f32) -> Rgba {
    color.a *= opacity;
    color
}
/// Disabled colors follow the Figma kit (every Button*/Toggle button* set at
/// State=Disabled): every variant, including text and outlined, fills its
/// container with `State Layers/On Surface/Opacity-10` and draws its content
/// in on-surface under a 0.38 layer; the outlined button keeps its full
/// outline-variant outline. Fixture: `tests/fixtures/figma-button.json`.
fn disabled_targets(outlined: bool) -> (CssColor, CssColor, CssColor) {
    use ColorRole as C;
    (
        CssColor::alpha_role(C::ON_SURFACE, 0.10),
        CssColor::alpha_role(C::ON_SURFACE, crate::tokens::state::DISABLED_CONTENT),
        if outlined {
            CssColor::role(C::OUTLINE_VARIANT)
        } else {
            CssColor::transparent()
        },
    )
}
fn color_targets(variant: ButtonVariant, disabled: bool) -> (CssColor, CssColor, CssColor) {
    use ColorRole as C;
    if disabled {
        return disabled_targets(variant == ButtonVariant::Outlined);
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
        // The kit's disabled toggles match the plain buttons, except that a
        // selected outlined toggle (inverse surface when enabled) drops its
        // outline.
        return disabled_targets(variant == ButtonVariant::Outlined && !selected);
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

/// The resolved container and content of an enabled toggle button of
/// `variant`, for composites (connected groups) that paint toggle colors
/// on their own options.
pub(crate) fn toggle_colors(variant: ButtonVariant, selected: bool) -> (Rgba, Rgba) {
    let (bg, fg, _) = toggle_color_targets(variant, false, true, selected);
    (bg.resolve(), fg.resolve())
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
    /// Counts touches and mouse presses, so a touch's hold limit ends only that touch.
    press_generation: u64,
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
            press_generation: 0,
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
                state.press_generation += 1;
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
                // Only activation keys change the pressed visual; an unrelated
                // key (including a modifier) during a held Space keeps it.
                let stroke = &event.keystroke;
                if !matches!(stroke.key.as_str(), "enter" | "space") {
                    return;
                }
                let pressed = !window.default_prevented()
                    && focus.is_focused(window)
                    && !stroke.modifiers.modified();
                down.update(cx, |state, cx| {
                    if state.keyboard != pressed {
                        state.keyboard = pressed;
                        cx.notify();
                    }
                });
            })
            .on_key_up(move |event, _, cx| {
                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                    up.update(cx, |state, cx| {
                        if state.keyboard {
                            state.keyboard = false;
                            cx.notify();
                        }
                    });
                }
            });
    }
    base
}

type HoverCallback = std::rc::Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// Sized, shaped Material presentation; activation, keyboard and focus stay in Base.
#[derive(gpui::IntoElement)]
pub struct MaterialButton {
    #[cfg(test)]
    progress_opacity_observer: Option<std::rc::Rc<std::cell::Cell<f32>>>,
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
    active_state: bool,
    progress: Option<(bool, SharedString)>,
    loading: Option<(bool, SharedString)>,
    progress_playback: Option<ButtonProgressPlayback>,
    transition_playback: Option<ButtonTransitionPlayback>,
    selector: Option<String>,
    preview: Option<InteractionPreview>,
    form: Form,
    palette: Option<Palette>,
    open: bool,
    press_probe: Option<std::rc::Rc<std::cell::Cell<bool>>>,
    style: StyleRefinement,
}
impl MaterialButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            #[cfg(test)]
            progress_opacity_observer: None,
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
            base: BaseControl::new(id).block_mouse_except_scroll(),
            children: vec![],
            icon: None,
            icon_position: ButtonIconPosition::Leading,
            direction: ButtonDirection::Ltr,
            selector: None,
            variant: ButtonVariant::Filled,
            size: ButtonSize::Small,
            shape: ButtonShape::Round,
            disabled: false,
            ripple: true,
            active_state: false,
            progress: None,
            loading: None,
            progress_playback: None,
            transition_playback: None,
            preview: None,
            form: Form::Label,
            palette: None,
            open: false,
            press_probe: None,
            style: StyleRefinement::default(),
        }
    }
    /// Use a non-label form's geometry (icon button, FAB, split segment).
    pub(crate) fn form(mut self, form: Form) -> Self {
        self.form = form;
        self
    }
    /// Paint with these roles instead of the variant's.
    pub(crate) fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }
    /// A trailing split segment whose menu is open: fully round.
    /// Report whether this button is pressed (pointer, key or forced
    /// preview) to a composite such as a button group. A change requests
    /// one more frame so the composite reads it.
    pub(crate) fn press_probe(mut self, probe: std::rc::Rc<std::cell::Cell<bool>>) -> Self {
        self.press_probe = Some(probe);
        self
    }
    pub(crate) fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    fn form_geometry(&self) -> form::FormGeometry {
        let geometry = self.form.geometry(self.size, self.direction);
        if self.open && self.form == Form::Segment(Segment::Trailing) {
            // The open, fully round trailing segment centers its chevron.
            let side = (geometry.padding.0 + geometry.padding.1) / 2.;
            form::FormGeometry {
                padding: (side, side),
                ..geometry
            }
        } else {
            geometry
        }
    }
    fn color_targets(&self) -> (CssColor, CssColor, CssColor) {
        match &self.palette {
            Some(palette) => palette.targets(self.disabled, self.base.toggle, self.base.selected),
            None => toggle_color_targets(
                self.variant,
                self.disabled,
                self.base.toggle,
                self.base.selected,
            ),
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
    /// Show an indeterminate indicator without changing activation or busy state.
    /// The label names the progress indicator for assistive technology. Keep this
    /// configuration present while hidden to retain its animation phase.
    /// When combined with `loading`, either true flag shows the indicator.
    pub fn progress(mut self, visible: bool, label: impl Into<SharedString>) -> Self {
        self.progress = Some((visible, label.into()));
        self
    }
    /// Show progress, expose busy, and suppress activation while `loading` is true.
    ///
    /// This is controlled presentation: the caller owns the loading flag, label
    /// children, and independent `disabled` flag. Update children to show a loading
    /// label; no previous label or disabled state is saved or restored internally.
    /// `loading(false, label)` retains progress configuration and its clock. An
    /// explicitly disabled button stays disabled when loading ends. The label
    /// names the indicator for assistive technology, not the button's visible text.
    pub fn loading(mut self, loading: bool, label: impl Into<SharedString>) -> Self {
        self.loading = Some((loading, label.into()));
        self
    }
    /// Keep visual transition clocks across virtual unmounting. Forward real
    /// offscreen controlled changes through the handle's `sync` method.
    /// Interaction state and ripple remain mount-local.
    pub fn transition_playback(mut self, playback: ButtonTransitionPlayback) -> Self {
        self.transition_playback = Some(playback);
        self
    }

    /// Retain progress lifecycle and opacity across virtual unmounting.
    /// While unmounted, the owner must forward real inputs and reduced motion
    /// to the handle. Hover, pressed, focus and ripple remain mount-local.
    pub fn progress_playback(mut self, playback: ButtonProgressPlayback) -> Self {
        self.progress_playback = Some(playback);
        self
    }
    /// Keep the reference active appearance, for example while a menu is open.
    /// This only changes presentation: it does not focus, select, activate, or
    /// create a ripple. The caller owns this state independently of `disabled`.
    /// The name leaves GPUI's `active(style)` customization available.
    pub fn active_state(mut self, active: bool) -> Self {
        self.active_state = active;
        self
    }
    /// Render a static specimen in one forced interaction state, painting
    /// exactly what the live state paints: its state-layer opacity, elevation,
    /// pressed shape (with the press layer at rest instead of a pointer
    /// ripple) and focus ring. The specimen leaves the tab order and ignores
    /// live hover and press. Combine with `toggle`/`selected` for toggle
    /// specimens. See [`InteractionPreview`].
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
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
    /// Names what the button paints, which a compact button's target root is not.
    fn debug_selector(mut self, f: impl FnOnce() -> String) -> Self {
        self.selector = Some(f());
        self
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
/// Transition-sampled visual channels for one render.
struct SampledColors {
    bg: Rgba,
    fg: Rgba,
    border: Rgba,
    /// Disabled icon CSS is an explicit child color without a transition.
    disabled_icon: Rgba,
}

/// Transient interaction state resolved from the passive press observer.
struct Interaction {
    focus: FocusHandle,
    pressed: Entity<PressState>,
    active: bool,
    hovered: bool,
    focused: bool,
}

/// State-layer opacity, elevation and shape sampled for one render.
struct Effects {
    opacity: f32,
    shadows: shadow::ShadowPair,
    radius: f32,
}

impl MaterialButton {
    fn progress_visible(&self) -> bool {
        self.progress.as_ref().is_some_and(|(visible, _)| *visible)
    }

    /// Keep the owner's no-op guard aligned with targets sampled by a
    /// visible render, without injecting any resting interaction target.
    fn record_transition_inputs(&self, cx: &App) {
        if let Some(playback) = &self.transition_playback {
            playback.record_inputs(
                ButtonTransitionInputs {
                    variant: self.variant,
                    size: self.size,
                    shape: self.shape,
                    toggle: self.base.toggle,
                    selected: self.base.selected,
                    disabled: self.disabled,
                    active: self.active_state,
                    ripple: self.ripple,
                },
                cx,
            );
        }
    }

    /// One lifecycle implementation serves both caller-owned and keyed
    /// playback. The keyed handle lives while the button renders each frame.
    /// Returns the playback (when progress is configured) and its opacity.
    fn sample_progress(
        &self,
        loading: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> (Option<ButtonProgressPlayback>, f32) {
        if self.loading.is_none() && self.progress.is_none() {
            return (None, 1.);
        }
        let playback = self.progress_playback.clone().unwrap_or_else(|| {
            window
                .use_keyed_state((self.id.clone(), "progress-playback"), cx, |_, _| {
                    ButtonProgressPlayback::default()
                })
                .read(cx)
                .clone()
        });
        let opacity = playback.sample_inputs(self.progress_visible(), loading, self.disabled, cx);
        if matches!(
            opacity.status,
            gpui_base::MotionStatus::Running | gpui_base::MotionStatus::Delayed
        ) {
            window.request_animation_frame();
        }
        (Some(playback), opacity.value)
    }

    fn sample_colors(&self, window: &mut Window, cx: &mut App) -> SampledColors {
        let (bg_target, fg_target, border_target) = self.color_targets();
        let disabled_icon = fg_target.resolve();
        let transitions = self.transition_playback.as_ref();
        let bg = playback::token_sample(
            transitions.map(|p| &p.background),
            (self.id.clone(), "background-color"),
            bg_target,
            MotionToken::DefaultEffects,
            window,
            cx,
        )
        .resolve();
        let fg = playback::token_sample(
            transitions.map(|p| &p.foreground),
            (self.id.clone(), "foreground-color"),
            fg_target,
            MotionToken::DefaultEffects,
            window,
            cx,
        )
        .resolve();
        let border = playback::token_sample(
            transitions.map(|p| &p.border),
            (self.id.clone(), "border-color"),
            border_target,
            MotionToken::DefaultEffects,
            window,
            cx,
        )
        .resolve();
        #[cfg(test)]
        if let Some(observer) = &self.colors_observer {
            observer.set((bg, fg, border));
        }
        SampledColors {
            bg,
            fg,
            border,
            disabled_icon,
        }
    }

    fn resolve_interaction(&self, window: &mut Window, cx: &mut App) -> Interaction {
        let focus = self.focus_handle.clone().unwrap_or_else(|| {
            window
                .use_keyed_state((self.id.clone(), "material-focus"), cx, |_, cx| {
                    cx.focus_handle()
                })
                .read(cx)
                .clone()
        });
        let pressed = press_state(self.id.clone(), &focus, window, cx);
        if self.disabled || !self.focusable || !focus.is_focused(window) {
            // Nothing to clear on almost every render: only read, as a write
            // would mark what read this state as changed.
            let held = {
                let state = pressed.read(cx);
                state.keyboard || (self.disabled && state.pointer)
            };
            if held {
                pressed.update(cx, |state, _| {
                    if self.disabled {
                        state.pointer = false;
                    }
                    state.keyboard = false;
                });
            }
        }
        if let Some(preview) = self.preview {
            return Interaction {
                focus,
                pressed,
                active: self.active_state || preview.is_pressed(),
                hovered: preview.is_hovered(),
                focused: preview.is_focused(),
            };
        }
        let press = pressed.read(cx);
        let active = self.active_state || press.pointer || press.keyboard;
        let hovered = press.hovered && !window.last_input_was_touch();
        let focused =
            self.focusable && focus.is_focused(window) && window.last_input_was_keyboard();
        Interaction {
            focus,
            pressed,
            active,
            hovered,
            focused,
        }
    }

    fn sample_effects(
        &self,
        interaction: &Interaction,
        window: &mut Window,
        cx: &mut App,
    ) -> Effects {
        let Interaction {
            active,
            hovered,
            focused,
            ..
        } = *interaction;
        let transitions = self.transition_playback.as_ref();
        let opacity = playback::opacity_sample(
            transitions,
            (self.id.clone(), "state-opacity"),
            playback::state_opacity(
                self.active_state,
                self.disabled,
                // A forced press shows the ripple at rest: the press layer
                // covering the whole container.
                self.ripple && self.preview.is_none(),
                active,
                focused,
                hovered,
            ),
            playback::opacity_policy(),
            window,
            cx,
        );
        let level = match &self.palette {
            Some(palette) => palette.shadow_level(self.disabled, hovered, active),
            // Icon buttons never elevate.
            None if matches!(self.form, Form::Icon(_)) => 0,
            None => playback::shadow_level(
                self.variant,
                self.disabled,
                self.base.toggle,
                hovered,
                active,
            ),
        };
        let shadows = playback::token_sample(
            transitions.map(|p| &p.shadow),
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
        let target = match self.form {
            // Split segments morph their inner corners; outer ends stay round.
            Form::Segment(segment) => {
                let metrics = segment_metrics(self.size);
                if self.open && segment == Segment::Trailing {
                    self.size.geometry().height / 2.
                } else if active || hovered {
                    metrics.inner_active
                } else {
                    metrics.inner
                }
            }
            form if !form.sized_shape() => form.fixed_radius().unwrap_or(0.),
            _ if active => self.size.geometry().pressed,
            _ => resting_radius(
                self.size,
                self.shape,
                self.base.toggle && self.base.selected,
            ),
        };
        let radius = playback::token_sample(
            transitions.map(|p| &p.radius),
            (self.id.clone(), "pressed-radius"),
            target,
            MotionToken::FastSpatial,
            window,
            cx,
        );
        #[cfg(test)]
        if let Some(observer) = &self.radius_observer {
            observer.set(radius);
        }
        Effects {
            opacity,
            shadows,
            radius,
        }
    }

    fn icon_slot_box(&self) -> Div {
        div()
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .size(rem(self.form_geometry().icon))
    }

    /// The leading/trailing slots: a visible progress indicator always takes
    /// the start slot and replaces the icon.
    fn slots(
        &mut self,
        loading: bool,
        progress: Option<(&ButtonProgressPlayback, f32)>,
        disabled_icon: Rgba,
        cx: &App,
    ) -> (Option<Div>, Option<Div>) {
        if let Some((playback, opacity)) = progress {
            let label = if loading {
                self.loading.as_ref()
            } else {
                self.progress.as_ref()
            }
            .expect("visible progress has caller configuration")
            .1
            .clone();
            // Frame requests stay limited to a painted, on-screen indicator.
            let indicator = super::progress::indeterminate_circular_progress_with_playback(
                (self.id.clone(), "button-progress-indicator"),
                label,
                size(rem(self.form_geometry().icon.min(24.)), rem(24.)),
                opacity,
                &playback.state.borrow().circular,
                cx,
            );
            let slot = self
                .icon_slot_box()
                .debug_selector(|| "material-button-progress-slot".into())
                .child(indicator);
            return (Some(slot), None);
        }
        let icon_size = self.form_geometry().icon;
        let slot = self.icon.take().map(|icon| {
            let icon = if self.disabled {
                icon.color(disabled_icon)
            } else {
                icon
            };
            self.icon_slot_box().child(icon.size(rem(icon_size)))
        });
        match self.icon_position {
            ButtonIconPosition::Leading => (slot, None),
            ButtonIconPosition::Trailing => (None, slot),
        }
    }
}

/// A touch came down on a button: press it at once, as a tap's own mouse-down comes
/// only with the finger's lift, and bound how long it can hold. GPUI reports a tap's
/// end and a pan's scroll publicly, but not a system cancel of a still finger.
fn touch_press(state: &mut PressState, cx: &mut Context<PressState>) {
    state.press_generation += 1;
    state.pointer = true;
    state.hovered = false;
    state.outside_window = false;
    cx.notify();
    let generation = state.press_generation;
    let timer = cx.background_executor().timer(ripple::TOUCH_HOLD_LIMIT);
    cx.spawn(async move |this, cx| {
        timer.await;
        let _ = this.update(cx, |state, cx| {
            if state.press_generation == generation && state.pointer {
                touch_release(state, cx);
            }
        });
    })
    .detach();
}

fn touch_release(state: &mut PressState, cx: &mut Context<PressState>) {
    state.pointer = false;
    state.hovered = false;
    cx.notify();
}

/// Passive pointer/touch observer. A window exit must clear a held press even
/// after the pointer has left the button. Its non-occluding hitbox, the Base root,
/// only locates passive touch presses.
fn pointer_observer(pressed: &Entity<PressState>, touch_enabled: bool) -> impl IntoElement {
    let weak = pressed.downgrade();
    canvas(
        move |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
        move |_, hitbox, window, cx| {
            let moving_hitbox = hitbox.clone();
            let touch = weak.clone();
            // GPUI offers each new touch as an unclaimed drag. Observing the offer
            // without claiming it leaves tap and scroll recognition untouched. A tap
            // ends in the mouse-up `observe_press` installs, a pan in the scroll below
            // and a claimed drag here.
            window.on_mouse_event(move |event: &TouchDragEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble {
                    return;
                }
                match event.phase {
                    TouchPhase::Started
                        if touch_enabled
                            && !window.default_prevented()
                            && hitbox.is_hovered(window) =>
                    {
                        let _ = touch.update(cx, touch_press);
                    }
                    TouchPhase::Ended | TouchPhase::Cancelled
                        if touch.upgrade().is_some_and(|state| state.read(cx).pointer) =>
                    {
                        let _ = touch.update(cx, touch_release);
                    }
                    _ => {}
                }
            });
            // Installed only while held: every wheel event calls every wheel listener.
            if weak.upgrade().is_some_and(|state| state.read(cx).pointer) {
                let scrolled = weak.clone();
                window.on_mouse_event(move |_: &ScrollEvent, phase, window, cx| {
                    if phase == DispatchPhase::Capture
                        && window.last_input_was_touch()
                        && scrolled.upgrade().is_some_and(|state| state.read(cx).pointer)
                    {
                        let _ = scrolled.update(cx, touch_release);
                    }
                });
            }
            let moving = weak.clone();
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                // Every button hears every move: only a held or returning one writes.
                if phase == DispatchPhase::Capture
                    && moving.upgrade().is_some_and(|state| {
                        let state = state.read(cx);
                        state.pointer || state.outside_window
                    })
                {
                    let _ = moving.update(cx, |state, cx| {
                        state.outside_window = false;
                        let hovered = moving_hitbox.is_hovered_at(event.position, window)
                            && !window.last_input_was_touch();
                        if state.hovered != hovered {
                            state.hovered = hovered;
                            cx.notify();
                        }
                    });
                }
            });
            window.on_mouse_event(move |_: &MouseExitEvent, phase, _, cx| {
                if phase == DispatchPhase::Capture
                    && weak.upgrade().is_some_and(|state| {
                        let state = state.read(cx);
                        !state.outside_window || state.pointer || state.hovered
                    })
                {
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
    .size_full()
}

/// CSS :hover is independent of keyboard modality. Preserve the caller's
/// GPUI modality preference separately, sharing the one native listener.
fn bridge_hover(
    base: BaseControl,
    pressed: &Entity<PressState>,
    callback: Option<HoverCallback>,
    caller_mode: HoverListenerMode,
    window: &mut Window,
    cx: &mut App,
) -> BaseControl {
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
    let hover_state = pressed.clone();
    base.hover_listener_mode(HoverListenerMode::InputModalityIndependent)
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
        })
}

/// The ripple's ink and visibility from the final (caller-refined) style.
/// Base's opacity scope ends after child.paint, so apply the same local
/// multiplier to the overlay (ancestor opacity remains in Window).
fn ripple_ink(painted: &Style, fg: Rgba) -> (Rgba, bool) {
    let mut ink = painted.text.color.map(Rgba::from).unwrap_or(fg);
    ink.a *= painted.opacity.unwrap_or(1.);
    let visible = painted.visibility == Visibility::Visible;
    if !visible {
        ink.a = 0.;
    }
    (ink, visible)
}

impl RenderOnce for MaterialButton {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use gpui_base::StyledExt;
        // Layout already mirrors under an inherited right-to-left direction, so
        // the logical order stays the left-to-right one here.
        if crate::scope::is_rtl(window) {
            self.direction = ButtonDirection::Ltr;
        }
        let loading = self.loading.as_ref().is_some_and(|(visible, _)| *visible);
        let show_progress = loading || self.progress_visible();
        self.disabled |= loading;
        if let Some(preview) = self.preview {
            self.disabled |= preview.is_disabled();
            self.base = self.base.tab_stop(false);
        }
        self.record_transition_inputs(cx);
        // `disabled()` already set Base; loading additionally disables it.
        self.base = self.base.disabled(self.disabled).busy(loading);
        let (progress_playback, progress_opacity) = self.sample_progress(loading, window, cx);
        #[cfg(test)]
        if let Some(observer) = &self.progress_opacity_observer {
            observer.set(progress_opacity);
        }
        let g = self.form_geometry();
        let ty = g.typography.scale();
        let colors = self.sample_colors(window, cx);
        let interaction = self.resolve_interaction(window, cx);
        if let Some(probe) = &self.press_probe
            && probe.get() != interaction.active
        {
            probe.set(interaction.active);
            window.request_animation_frame();
        }
        let effects = self.sample_effects(&interaction, window, cx);
        let (left_padding, right_padding) = g.padding;
        let progress = progress_playback
            .as_ref()
            .filter(|_| show_progress)
            .map(|playback| (playback, progress_opacity));
        let (leading, trailing) = self.slots(loading, progress, colors.disabled_icon, cx);
        let Interaction { focus, pressed, .. } = interaction;
        let expanded_target = matches!(self.size, ButtonSize::ExtraSmall | ButtonSize::Small)
            && matches!(self.form, Form::Label | Form::Icon(_) | Form::Segment(_));
        let target = expanded_target
            .then(|| {
                hit_target::Target::resolve(g.height, &self.style, window.rem_size(), |p| {
                    window.pixel_snap(p)
                })
            })
            .flatten();
        let base = self.base.track_focus(&focus);
        let base = observe_press(base, &pressed, &focus, self.disabled, self.focusable);
        let base = bridge_hover(
            base,
            &pressed,
            self.hover_callback,
            self.caller_hover_mode,
            window,
            cx,
        )
        .when(!self.disabled && self.preview.is_none(), |b| {
            b.cursor_pointer()
        });
        let observer = pointer_observer(&pressed, !self.disabled);
        let SampledColors { bg, fg, border, .. } = colors;
        // A split segment: full outer ends, animated inner corners.
        let corners = match self.form {
            Form::Segment(segment) => {
                let rem_size = window.rem_size();
                let outer = rem(g.height / 2.).to_pixels(rem_size);
                let inner = rem(effects.radius).to_pixels(rem_size);
                let leading_side =
                    (segment == Segment::Leading) == (self.direction == ButtonDirection::Ltr);
                let (left, right) = if leading_side {
                    (outer, inner)
                } else {
                    (inner, outer)
                };
                Some(Corners {
                    top_left: left,
                    bottom_left: left,
                    top_right: right,
                    bottom_right: right,
                })
            }
            _ => None,
        };
        // A compact button's root is its 48dp target, so what it paints moves to a
        // child; the caller's layout stays on the root.
        let root_layout = target.is_some().then(|| hit_target::take_layout(&mut self.style));
        let variant = self.variant;
        let palette = self.palette;
        let direction = self.direction;
        let style = self.style;
        // The painted surface, whichever element it is.
        macro_rules! surface {
            ($element:expr) => {
                $element
                    .flex()
                    .flex_row()
                    .when(direction == ButtonDirection::Rtl, |b| b.flex_row_reverse())
                    .items_center()
                    .justify_center()
                    .pl(rem(left_padding))
                    .pr(rem(right_padding))
                    .gap(rem(g.gap))
                    .map(|b| match corners {
                        Some(corners) => b
                            .rounded_tl(corners.top_left)
                            .rounded_tr(corners.top_right)
                            .rounded_br(corners.bottom_right)
                            .rounded_bl(corners.bottom_left),
                        None => b.rounded(rem(effects.radius)),
                    })
                    .bg(state::layer(bg, fg, effects.opacity))
                    .text_color(fg)
                    .font_family(ty.font_family)
                    .text_size(ty.font_size)
                    .letter_spacing(ty.letter_spacing)
                    .line_height(ty.line_height)
                    .font_weight(FontWeight(ty.font_weight as f32))
                    .when(
                        variant == ButtonVariant::Outlined && palette.is_none(),
                        |b| b.border(rem(g.outline)).border_color(border),
                    )
                    .shadow(effects.shadows.into_shadows())
                    .refine_style(&style)
            };
        }
        let ripple_on = self.ripple && self.preview.is_none();
        let forced_focus = self.preview == Some(InteractionPreview::Focused);
        let ring_enabled = !self.disabled && (self.focusable || forced_focus);
        let radius = rem(effects.radius);
        let selector = self.selector;
        let ring = move |child: ripple::ButtonRipple, focus: FocusHandle| {
            match corners {
                Some(corners) => ButtonFocusRing::with_corners(child, focus, ring_enabled, corners),
                None => ButtonFocusRing::new(child, focus, ring_enabled, radius),
            }
            .forced(forced_focus)
        };
        let id = self.id;
        let enabled = !self.disabled;
        if let (Some(target), Some(layout)) = (target, root_layout) {
            let visual = surface!(div().flex_grow(1.).min_w(px(0.)))
                .h(rem(g.height))
                .refine_style(&hit_target::height_of(&layout))
                .children(leading)
                .children(self.children)
                .children(trailing)
                .when_some(selector, |b, name| b.debug_selector(move || name));
            let mut painted_style = Style::default();
            let mut visual = visual;
            painted_style.refine(visual.style());
            let (ripple_color, ripple_visible) = ripple_ink(&painted_style, fg);
            let surface = ripple::ButtonRipple::new(
                visual,
                id,
                ripple_on,
                enabled && ripple_visible,
                painted_style.corner_radii,
                painted_style.border_widths,
                ripple_color,
                window,
                cx,
            )
            .hit_extension(target.extension());
            base.flex_shrink_0()
                .min_w(rem(g.min_width))
                .when_some(g.width, |b, width| b.w(rem(width)))
                .refine_style(&layout)
                .h(target.height)
                .mt(target.margin_top)
                .mb(target.margin_bottom)
                .child(ring(surface, focus))
                .child(observer)
                .into_any_element()
        } else {
            let mut base = surface!(
                base.flex_shrink_0()
                    .min_w(rem(g.min_width))
                    .when_some(g.width, |b, width| b.w(rem(width)))
                    .h(rem(g.height))
            )
            .children(leading)
            .children(self.children)
            .children(trailing)
            .child(observer)
            .when_some(selector, |b, name| b.debug_selector(move || name));
            let mut painted_style = Style::default();
            painted_style.refine(base.style());
            let (ripple_color, ripple_visible) = ripple_ink(&painted_style, fg);
            let surface = ripple::ButtonRipple::new(
                base,
                id,
                ripple_on,
                enabled && ripple_visible,
                painted_style.corner_radii,
                painted_style.border_widths,
                ripple_color,
                window,
                cx,
            );
            ring(surface, focus).into_any_element()
        }
    }
}
/// Convenience constructor with caller-supplied content and activation.
pub fn material_button(id: impl Into<ElementId>, variant: ButtonVariant) -> MaterialButton {
    MaterialButton::new(id).variant(variant)
}

#[cfg(test)]
mod tests {
    use super::{ButtonSize, ButtonVariant, MaterialButton, alpha, colors};
    use crate::tokens::state::layer;
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
            crate::motion::set_motion_scheme(crate::motion::MotionScheme::Expressive);
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
                .position(0.1),
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
            crate::motion::set_motion_scheme(crate::motion::MotionScheme::Expressive);
            crate::theme::apply_scheme(
                crate::color::Scheme::from_palette(self.palette),
                window,
                cx,
            );
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
            window.refresh();
            window.draw(cx).clear(cx);
            cx.set_reduce_motion(false);
        });
        let initial = sample.get();
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.variant = ButtonVariant::Tonal);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_colors_close(sample.get(), initial);
        cx.executor().advance_clock(Duration::from_millis(125));
        draw(cx);
        let filled = colors(ButtonVariant::Filled, false);
        let tonal = colors(ButtonVariant::Tonal, false);
        let progress = crate::motion::MotionScheme::Expressive
            .resolve(crate::motion::MotionToken::DefaultEffects)
            .position(0.125);
        assert_colors_close(
            sample.get(),
            (
                crate::theme::blend(filled.0, tonal.0, progress),
                crate::theme::blend(filled.1, tonal.1, progress),
                tonal.2,
            ),
        );
        let midpoint = sample.get();
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.palette = PaletteId::Ink);
            window.refresh();
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
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.executor().advance_clock(Duration::from_millis(50));
        draw(cx);
        let disabling = sample.get();
        assert!(disabling.0.a > 0.10 && disabling.0.a < 1.);
        assert!(disabling.1.a > 0.38 && disabling.1.a < 1.);
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.disabled = false);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_colors_close(sample.get(), disabling);
        cx.executor().advance_clock(Duration::from_millis(250));
        draw(cx);
        assert_colors_close(sample.get(), colors(ButtonVariant::Tonal, false));
        cx.update(|window, cx| {
            cx.set_reduce_motion(true);
            view.update(cx, |view, _| view.disabled = true);
            window.refresh();
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
    fn figma_buttons() -> serde_json::Value {
        serde_json::from_str(include_str!("../../tests/fixtures/figma-button.json")).unwrap()
    }

    /// A kit color variable name as the role (and opacity) it paints.
    pub(super) fn figma_role(variable: &str) -> (ColorRole, f32) {
        match variable {
            "Schemes/On Surface" => (ColorRole::ON_SURFACE, 1.),
            "Schemes/Outline Variant" => (ColorRole::OUTLINE_VARIANT, 1.),
            "State Layers/On Surface/Opacity-10" => (ColorRole::ON_SURFACE, 0.10),
            other => panic!("unexpected kit variable {other}"),
        }
    }

    #[test]
    fn size_geometry_matches_the_figma_kit_buttons() {
        let fixture = figma_buttons();
        let px = |value: &serde_json::Value| value.as_f64().unwrap() as f32;
        for (size, name) in ButtonSize::ALL
            .into_iter()
            .zip(["XSmall", "Small", "Medium", "Large", "XLarge"])
        {
            let reference = &fixture["sizes"][name];
            let g = size.geometry();
            assert_eq!(g.height, px(&reference["container_height"]), "{name}");
            assert_eq!(size.icon_size(), px(&reference["icon_size"]), "{name}");
            assert_eq!(g.outline, px(&reference["outline_width"]), "{name}");
            assert_eq!(g.square, px(&reference["shape_square"]), "{name}");
            assert_eq!(g.pressed, px(&reference["shape_pressed"]), "{name}");
            assert_eq!(g.gap, px(&reference["icon_label_gap"]), "{name}");
            let padding = reference["padding"].as_array().unwrap();
            // [top, right, bottom, left]; the label's line box fills the rest.
            assert_eq!(padding[1], padding[3], "{name}");
            assert_eq!(
                super::horizontal_padding(size),
                (px(&padding[3]), px(&padding[1])),
                "{name}"
            );
            let style = reference["label_style"].as_str().unwrap();
            let role = g.typography.name();
            assert_eq!(
                style,
                format!("M3/{}", role.replacen('-', "/", 1)),
                "{name} label style"
            );
        }
    }

    #[test]
    fn disabled_buttons_match_the_figma_kit() {
        let fixture = figma_buttons();
        let disabled = &fixture["disabled"];
        for variant in ButtonVariant::ALL {
            let (bg, fg, border) = colors(variant, true);
            let reference = &disabled[match variant {
                ButtonVariant::Filled => "filled",
                ButtonVariant::Elevated => "elevated",
                ButtonVariant::Tonal => "tonal",
                ButtonVariant::Outlined => "outlined",
                ButtonVariant::Text => "text",
            }];
            let (container, opacity) =
                figma_role(reference["container"]["variable"].as_str().unwrap());
            assert_eq!(
                bg,
                alpha(color(container), opacity),
                "{variant:?} container"
            );
            let (content, _) = figma_role(reference["content"]["variable"].as_str().unwrap());
            let content_opacity = reference["content"]["opacity"].as_f64().unwrap() as f32;
            assert_eq!(
                reference["content"]["icon_opacity"], reference["content"]["opacity"],
                "{variant:?}"
            );
            assert_eq!(fg, alpha(color(content), content_opacity), "{variant:?}");
            match reference["outline"].as_object() {
                Some(outline) => {
                    let (role, opacity) = figma_role(outline["variable"].as_str().unwrap());
                    assert_eq!(border, alpha(color(role), opacity), "{variant:?} outline");
                }
                None => assert_eq!(border.a, 0., "{variant:?} outline"),
            }
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
            crate::motion::set_motion_scheme(crate::motion::MotionScheme::Expressive);
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
                    .position(0.1);
        assert!((sample.get() - expected).abs() < 0.001);
        cx.update(|_, cx| assert_eq!(view.read(cx).activations, 0));
        cx.simulate_event(TouchEvent {
            id: TouchId(1),
            phase: TouchPhase::Cancelled,
            position: center,
            ..Default::default()
        });
        // A system cancel of a still finger has no public signal: the hold limit ends it.
        cx.executor().advance_clock(super::ripple::TOUCH_HOLD_LIMIT);
        cx.run_until_parked();
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
                    .position(0.1);
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
        for other in ["shift", "a", "shift-a"] {
            key(cx, other, true);
            draw(cx);
            assert_eq!(
                sample.get(),
                16.,
                "{other} pressed while holding Space keeps the pressed visual"
            );
            key(cx, other, false);
        }
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
                        // The kit: the size's State-layer padding on both sides.
                        let (icon_side, label_side) = (pad, pad);
                        let border = if variant == ButtonVariant::Outlined {
                            match size {
                                ButtonSize::Large | ButtonSize::ExtraLarge => 2.,
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
