//! Material button presentation over GPUI Base activation and focus behavior.
//! Reference: floor/material c910072f3fc5b660276731d8a935d7186f288364.
//! Ripple, animated color/state layers, expanded touch targets and toggle/loading
//! semantics are still pending; this module does not establish full conformance.
use crate::{
    color::ColorRole,
    motion::{self, MotionToken},
    theme::color,
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
fn colors(variant: ButtonVariant, disabled: bool) -> (Rgba, Rgba, Rgba) {
    let transparent = gpui::rgba(0x00000000);
    if disabled {
        return (
            if matches!(variant, ButtonVariant::Text | ButtonVariant::Outlined) {
                transparent
            } else {
                alpha(color(ColorRole::ON_SURFACE), 0.10)
            },
            alpha(color(ColorRole::ON_SURFACE_VARIANT), 0.38),
            if variant == ButtonVariant::Outlined {
                alpha(color(ColorRole::OUTLINE_VARIANT), 0.10)
            } else {
                transparent
            },
        );
    }
    let (bg, fg) = match variant {
        ButtonVariant::Filled => (color(ColorRole::PRIMARY), color(ColorRole::ON_PRIMARY)),
        ButtonVariant::Elevated => (
            color(ColorRole::SURFACE_CONTAINER_LOW),
            color(ColorRole::PRIMARY),
        ),
        ButtonVariant::Tonal => (
            color(ColorRole::SECONDARY_CONTAINER),
            color(ColorRole::ON_SECONDARY_CONTAINER),
        ),
        ButtonVariant::Outlined => (transparent, color(ColorRole::ON_SURFACE_VARIANT)),
        ButtonVariant::Text => (transparent, color(ColorRole::PRIMARY)),
    };
    (
        bg,
        fg,
        if variant == ButtonVariant::Outlined {
            color(ColorRole::OUTLINE_VARIANT)
        } else {
            transparent
        },
    )
}
fn elevation(level: u8) -> Vec<BoxShadow> {
    match level {
        1 => vec![
            BoxShadow::new(px(0.), px(1.), gpui::black().opacity(0.3)).blur_radius(px(2.)),
            BoxShadow::new(px(0.), px(1.), gpui::black().opacity(0.15))
                .blur_radius(px(3.))
                .spread_radius(px(1.)),
        ],
        2 => vec![
            BoxShadow::new(px(0.), px(1.), gpui::black().opacity(0.3)).blur_radius(px(2.)),
            BoxShadow::new(px(0.), px(2.), gpui::black().opacity(0.15))
                .blur_radius(px(6.))
                .spread_radius(px(2.)),
        ],
        _ => vec![],
    }
}

/// Passive visual state. Base remains the only activation authority.
struct PressState {
    pointer: bool,
    keyboard: bool,
    focus: FocusHandle,
    focus_out: Option<Subscription>,
    _activation: Subscription,
}
fn clear_press(state: &mut PressState) {
    state.pointer = false;
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
    mut base: gpui_base::Button,
    state: &Entity<PressState>,
    focus: &FocusHandle,
    disabled: bool,
    focusable: bool,
) -> gpui_base::Button {
    if disabled {
        return base;
    }
    let down = state.clone();
    base = base.on_mouse_down(MouseButton::Left, move |_, window, cx| {
        if !window.default_prevented() {
            down.update(cx, |state, cx| {
                state.pointer = true;
                cx.notify();
            });
        }
    });
    let up = state.clone();
    let out = state.clone();
    base = base
        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
            up.update(cx, |state, cx| {
                state.pointer = false;
                cx.notify();
            });
        })
        .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
            out.update(cx, |state, cx| {
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

/// Sized, shaped Material presentation; activation, keyboard and focus stay in Base.
#[derive(IntoElement)]
pub struct MaterialButton {
    #[cfg(test)]
    radius_observer: Option<std::rc::Rc<std::cell::Cell<f32>>>,
    id: ElementId,
    focus_handle: Option<FocusHandle>,
    focusable: bool,
    base: gpui_base::Button,
    variant: ButtonVariant,
    size: ButtonSize,
    shape: ButtonShape,
    disabled: bool,
    style: StyleRefinement,
}
impl MaterialButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            #[cfg(test)]
            radius_observer: None,
            id: id.clone(),
            focus_handle: None,
            focusable: true,
            base: gpui_base::Button::new(id),
            variant: ButtonVariant::Filled,
            size: ButtonSize::Small,
            shape: ButtonShape::Round,
            disabled: false,
            style: StyleRefinement::default(),
        }
    }
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
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
        self.base.extend(elements);
    }
}
impl InteractiveElement for MaterialButton {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }
}
impl StatefulInteractiveElement for MaterialButton {}
impl RenderOnce for MaterialButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use gpui_base::StyledExt;
        let g = self.size.geometry();
        let ty = g.typography.scale();
        let (bg, fg, border) = colors(self.variant, self.disabled);
        let radius = if self.shape == ButtonShape::Round {
            g.height / 2.
        } else {
            g.square
        };
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
        let padding = if self.variant == ButtonVariant::Text
            && matches!(self.size, ButtonSize::ExtraSmall | ButtonSize::Small)
        {
            12.
        } else {
            g.padding
        };
        let level = if self.variant == ButtonVariant::Elevated {
            1
        } else {
            0
        };
        let hover_level = match self.variant {
            ButtonVariant::Elevated => 2,
            ButtonVariant::Filled | ButtonVariant::Tonal => 1,
            _ => 0,
        };
        let weak = pressed.downgrade();
        // A window exit must clear a held press even after the pointer has left
        // the button. This canvas creates no hitbox and never handles activation.
        let exit_observer = canvas(
            |_, _, _| (),
            move |_, _, window, _| {
                window.on_mouse_event(move |_: &MouseExitEvent, phase, _, cx| {
                    if phase == DispatchPhase::Capture {
                        let _ = weak.update(cx, |state, cx| {
                            if state.pointer {
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
        let base = self.base.track_focus(&focus).child(exit_observer);
        let base = observe_press(base, &pressed, &focus, self.disabled, self.focusable);
        base.flex()
            .flex_row()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .min_w(rem(58.))
            .h(rem(g.height))
            .px(rem(padding))
            .gap(rem(g.gap))
            .rounded(rem(radius))
            .bg(bg)
            .text_color(fg)
            .text_size(ty.font_size)
            .line_height(ty.line_height)
            .font_weight(FontWeight(ty.font_weight as f32))
            .when(self.variant == ButtonVariant::Outlined, |b| {
                b.border(rem(g.outline)).border_color(border)
            })
            .shadow(elevation(if self.disabled { 0 } else { level }))
            .when(!self.disabled, |b| {
                b.cursor_pointer()
                    .hover(move |s| s.bg(layer(bg, fg, 0.08)).shadow(elevation(hover_level)))
                    .focus_visible(move |s| s.bg(layer(bg, fg, 0.10)))
                    .active(move |s| s.bg(layer(bg, fg, 0.10)).shadow(elevation(level)))
            })
            .refine_style(&self.style)
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
}
