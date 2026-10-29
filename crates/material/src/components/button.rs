//! Material button presentation over GPUI Base activation and focus behavior.
//! Reference: floor/material c910072f3fc5b660276731d8a935d7186f288364.
//! Ripple, animated state layers/shadows, expanded touch targets and toggle/loading
//! semantics are still pending. Color transitions currently inherit Base's reduced
//! motion and repeated-reversal policies; this is not full reference conformance.
mod effects;
mod focus_ring;
mod shadow;

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
#[cfg(test)]
fn colors(variant: ButtonVariant, disabled: bool) -> (Rgba, Rgba, Rgba) {
    let (bg, fg, border) = color_targets(variant, disabled);
    (bg.resolve(), fg.resolve(), border.resolve())
}
fn elevation(level: u8) -> Vec<BoxShadow> {
    shadow::ShadowPair::level(level).into_shadows()
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
    #[cfg(test)]
    colors_observer: Option<std::rc::Rc<std::cell::Cell<(Rgba, Rgba, Rgba)>>>,
    id: ElementId,
    focus_handle: Option<FocusHandle>,
    focusable: bool,
    base: gpui_base::Button,
    children: Vec<AnyElement>,
    icon: Option<Icon>,
    icon_position: ButtonIconPosition,
    direction: ButtonDirection,
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
            #[cfg(test)]
            colors_observer: None,
            id: id.clone(),
            focus_handle: None,
            focusable: true,
            base: gpui_base::Button::new(id),
            children: vec![],
            icon: None,
            icon_position: ButtonIconPosition::Leading,
            direction: ButtonDirection::Ltr,
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
impl StatefulInteractiveElement for MaterialButton {}
impl RenderOnce for MaterialButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use gpui_base::StyledExt;
        let g = self.size.geometry();
        let ty = g.typography.scale();
        let (bg_target, fg_target, border_target) = color_targets(self.variant, self.disabled);
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
        let radius = if self.shape == ButtonShape::Round {
            g.height / 2.
        } else {
            g.square
        };
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
        let base = self
            .base
            .track_focus(&focus)
            .children(leading)
            .children(self.children)
            .children(trailing)
            .child(exit_observer);
        let base = observe_press(base, &pressed, &focus, self.disabled, self.focusable);
        let base = base
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
            .refine_style(&self.style);
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
