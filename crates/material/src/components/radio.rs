//! M3 radio button over GPUI Base's radio behavior, and a radio group with
//! roving keyboard focus.
//!
//! The radio is a 20dp ring with a 10dp dot inside a 40dp state layer and a
//! 48dp target. Selecting scales the dot in on the fast spatial spring (its
//! overshoot kept) while the ring's color follows it; reduced motion lands
//! at once.
use super::InteractionPreview;
use super::button::ButtonFocusRing;
use super::ripple::Ripple;
use super::roving::{self, Orientation, Roving, index_handles};
use crate::{
    color::ColorRole,
    motion,
    theme::{blend, color},
    tokens::{
        state,
        typography::{TypeRole, rem},
    },
};
use gpui::{prelude::*, *};
use std::rc::Rc;

/// Radio geometry, in dp.
const RING: f32 = 20.;
const RING_STROKE: f32 = 2.;
const DOT: f32 = 10.;
const STATE_LAYER: f32 = 40.;
const TARGET: f32 = 48.;

/// The colors of a radio (and of a checkbox, which shares the selection
/// control state policy) in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SelectionPaint {
    /// The ring or outline (and the dot) at rest.
    pub(crate) ink: Rgba,
    /// The ring while hovered, focused or pressed.
    pub(crate) ink_active: Rgba,
    pub(crate) hover: Rgba,
    pub(crate) focus: Rgba,
    pub(crate) press: Rgba,
}

/// The M3 selection-control state policy: selected controls are `accent`
/// with accent hover and focus layers but an on-surface press layer;
/// unselected ones are on-surface-variant (on-surface while active) with
/// on-surface hover and focus layers and an accent press layer. Disabled is
/// on-surface at [`state::DISABLED_CONTENT`] with no layers. An error
/// control passes the error role as `accent` and uses it for every layer and
/// ink.
pub(crate) fn selection_paint(
    selected: bool,
    enabled: bool,
    accent: ColorRole,
    error: bool,
) -> SelectionPaint {
    let clear = Rgba {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 0.,
    };
    if !enabled {
        let ink = state::disabled(color(ColorRole::ON_SURFACE));
        return SelectionPaint {
            ink,
            ink_active: ink,
            hover: clear,
            focus: clear,
            press: clear,
        };
    }
    let accent = color(accent);
    let on_surface = color(ColorRole::ON_SURFACE);
    let (ink, ink_active, layer, press_layer) = if error {
        (accent, accent, accent, accent)
    } else if selected {
        (accent, accent, accent, on_surface)
    } else {
        (
            color(ColorRole::ON_SURFACE_VARIANT),
            on_surface,
            on_surface,
            accent,
        )
    };
    SelectionPaint {
        ink,
        ink_active,
        hover: state::layer(clear, layer, state::HOVER),
        focus: state::layer(clear, layer, state::FOCUS),
        press: state::layer(clear, press_layer, state::PRESS),
    }
}

/// The radio's ring and dot for an animated selection `amount` (0
/// unselected, 1 selected; the spring may overshoot either end).
fn radio_indicator(amount: f32, selected: bool, enabled: bool, active: bool) -> Div {
    let amount = if amount.is_finite() { amount } else { 0. };
    let rest = selection_paint(false, enabled, ColorRole::PRIMARY, false);
    let chosen = selection_paint(true, enabled, ColorRole::PRIMARY, false);
    let from = if active { rest.ink_active } else { rest.ink };
    let tone = blend(from, chosen.ink, amount.clamp(0., 1.));
    // The dot may overshoot, but never reaches the ring.
    let dot = (DOT * amount).clamp(0., RING - 2. * RING_STROKE - 2.);
    div()
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .size(rem(RING))
        .rounded_full()
        .border(rem(RING_STROKE))
        .border_color(tone)
        .when(selected || dot > 0., |ring| {
            ring.child(
                div()
                    .when(cfg!(test), |dot| dot.debug_selector(|| "radio-dot".into()))
                    .size(rem(dot))
                    .rounded_full()
                    .bg(tone),
            )
        })
}

/// The M3 radio of a choice: an outlined ring, filled with a dot when
/// selected. A static indicator for controls that are themselves the radio
/// (e.g. a choice tile).
pub fn choice_radio(selected: bool, enabled: bool) -> Div {
    radio_indicator(if selected { 1. } else { 0. }, selected, enabled, false)
}

/// [`choice_radio`] at an animated selection amount.
pub(crate) fn animated_choice_radio(amount: f32, selected: bool, enabled: bool) -> Div {
    radio_indicator(amount, selected, enabled, false)
}

type Select = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// Controlled M3 radio button. It has no visible label unless
/// [`MaterialRadio::visible_label`] is set; its label is always its
/// accessible name.
#[derive(gpui::IntoElement)]
pub struct MaterialRadio {
    id: ElementId,
    label: SharedString,
    visible_label: bool,
    label_before: bool,
    selected: bool,
    disabled: bool,
    preview: Option<InteractionPreview>,
    focus: Option<FocusHandle>,
    tab_stop: bool,
    position: Option<(usize, usize)>,
    on_select: Option<Select>,
}

impl MaterialRadio {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            visible_label: false,
            label_before: false,
            selected: false,
            disabled: false,
            preview: None,
            focus: None,
            tab_stop: true,
            position: None,
            on_select: None,
        }
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Show the label after the radio, inside its target.
    pub fn visible_label(mut self, visible: bool) -> Self {
        self.visible_label = visible;
        self
    }
    /// Put the visible label before the radio instead of after it, the
    /// radio pushed to the row's far end (md3.io's "Labels before").
    pub fn label_before(mut self, before: bool) -> Self {
        self.label_before = before;
        self
    }
    /// Render a static specimen in one forced interaction state. See
    /// [`InteractionPreview`].
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
    /// Use a caller-owned focus handle, e.g. for roving focus in a group.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }
    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }
    /// The radio's one-based position in its group of `size`.
    pub fn position(mut self, position: usize, size: usize) -> Self {
        self.position = Some((position, size));
        self
    }
    /// Called with `true` when an unselected radio is chosen; a radio cannot
    /// deselect itself.
    pub fn on_change(mut self, f: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

/// Hover and press styles resolve against this group.
const RADIO_GROUP_STYLE: &str = "material-radio";

impl RenderOnce for MaterialRadio {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus = self.focus.clone().unwrap_or_else(|| {
            window
                .use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        let preview = self.preview;
        let disabled = self.disabled || preview.is_some_and(|p| p.is_disabled());
        let enabled = !disabled;
        let live = enabled && preview.is_none();
        let forced = |state| enabled && preview == Some(state);
        let keyboard_focus = forced(InteractionPreview::Focused)
            || (live && focus.is_focused(window) && window.last_input_was_keyboard());
        let amount = if preview.is_some() {
            if self.selected { 1. } else { 0. }
        } else {
            motion::selection_value(
                (self.id.clone(), "dot"),
                if self.selected { 1. } else { 0. },
                window,
                cx,
            )
        };
        let paint = selection_paint(self.selected, enabled, ColorRole::PRIMARY, false);
        let active = keyboard_focus
            || forced(InteractionPreview::Hovered)
            || forced(InteractionPreview::Pressed);
        let ring = radio_indicator(amount, self.selected, enabled, active).when(
            live && !self.selected,
            |ring| {
                let ink = paint.ink_active;
                ring.group_hover(RADIO_GROUP_STYLE, move |style| style.border_color(ink))
            },
        );
        let layer = div()
            .id("state-layer")
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .size(rem(STATE_LAYER))
            .rounded_full()
            .when(keyboard_focus, |layer| layer.bg(paint.focus))
            .when(forced(InteractionPreview::Hovered), |layer| {
                layer.bg(paint.hover)
            })
            .when(forced(InteractionPreview::Pressed), |layer| {
                layer.bg(paint.press)
            })
            .when(live && !keyboard_focus, |layer| {
                let (hover, press) = (paint.hover, paint.press);
                layer
                    .group_hover(RADIO_GROUP_STYLE, move |style| style.bg(hover))
                    .group_active(RADIO_GROUP_STYLE, move |style| style.bg(press))
            })
            .child(ring);
        let layer = Ripple::new((self.id.clone(), "ripple"), layer)
            .round()
            .centered()
            .hit_extension(Edges::all(rem((TARGET - STATE_LAYER) / 2.)))
            .ink(paint.ink_active)
            .disabled(!enabled)
            .specimen(preview.is_some());
        let ringed = ButtonFocusRing::new(layer, focus.clone(), enabled, rem(STATE_LAYER / 2.))
            .forced(forced(InteractionPreview::Focused));
        let label_color = if enabled {
            color(ColorRole::ON_SURFACE)
        } else {
            state::disabled(color(ColorRole::ON_SURFACE))
        };
        let scale = TypeRole::BodyLarge.scale();
        let on_select = self.on_select;
        let before = self.label_before && self.visible_label;
        let (near, far) = (
            (TARGET - STATE_LAYER) / 2.,
            if self.visible_label {
                8.
            } else {
                (TARGET - STATE_LAYER) / 2.
            },
        );
        let mut radio = gpui_base::Radio::new(self.id)
            .track_focus(&focus)
            .accessibility_label(self.label.clone())
            .checked(self.selected)
            .disabled(disabled)
            .tab_stop(self.tab_stop && preview.is_none());
        if let Some((position, size)) = self.position {
            radio = radio.set_position(position, size);
        }
        radio
            .when_some(on_select, |radio, on_select| {
                radio.on_change(move |value, _, window, cx| on_select(&value, window, cx))
            })
            .group(RADIO_GROUP_STYLE)
            .flex()
            .flex_shrink_0()
            .items_center()
            .h(rem(TARGET))
            .min_w(rem(TARGET))
            .map(|radio| {
                if before {
                    radio
                        .flex_row_reverse()
                        .justify_between()
                        .gap(rem(16.))
                        .pr(rem(near))
                        .pl(rem(far))
                } else {
                    radio.pl(rem(near)).pr(rem(far))
                }
            })
            .when(live, |radio| radio.cursor_pointer())
            .child(ringed)
            .when(self.visible_label, |radio| {
                radio.child(
                    div()
                        .when(!before, |label| label.pl(rem(4.)))
                        .whitespace_nowrap()
                        .text_color(label_color)
                        .text_size(scale.font_size)
                        .line_height(scale.line_height)
                        .child(self.label),
                )
            })
    }
}

type GroupChange = Rc<dyn Fn(&usize, &mut Window, &mut App)>;

struct GroupOption {
    label: SharedString,
    enabled: bool,
}

/// A labelled set of [`MaterialRadio`]s with one selection: Tab enters at
/// the selected option; arrow keys, Home and End move focus among the
/// available options and select them.
#[derive(gpui::IntoElement)]
pub struct MaterialRadioGroup {
    id: ElementId,
    label: SharedString,
    axis: Axis,
    options: Vec<GroupOption>,
    selected: Option<usize>,
    labels_before: bool,
    disabled: bool,
    on_change: Option<GroupChange>,
}

impl MaterialRadioGroup {
    /// `label` names the group for accessibility.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            axis: Axis::Vertical,
            options: Vec::new(),
            selected: None,
            labels_before: false,
            disabled: false,
            on_change: None,
        }
    }
    /// Lay the options out in a row instead of a column.
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }
    /// Append an option with its visible label.
    pub fn option(mut self, label: impl Into<SharedString>, enabled: bool) -> Self {
        self.options.push(GroupOption {
            label: label.into(),
            enabled,
        });
        self
    }
    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }
    /// Put each label before its radio. See [`MaterialRadio::label_before`].
    pub fn labels_before(mut self, before: bool) -> Self {
        self.labels_before = before;
        self
    }
    /// Disable every option: the group shows its selection but takes no
    /// focus or input.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Called with the index of a newly chosen option.
    pub fn on_change(mut self, f: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for MaterialRadioGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let count = self.options.len();
        let focus = index_handles((self.id.clone(), "radio-focus"), count, window, cx);
        let disabled = self.disabled;
        let skip: Rc<[bool]> = self
            .options
            .iter()
            .map(|option| disabled || !option.enabled)
            .collect();
        let selected = self.selected.filter(|&index| index < count);
        let on_select = self.on_change.clone().map(|on_change| {
            Rc::new(move |index: usize, window: &mut Window, cx: &mut App| {
                on_change(&index, window, cx)
            }) as roving::Select
        });
        let roving = Roving::new(focus.clone(), Orientation::Both)
            .skip(skip)
            .selected(selected)
            .select_on_focus(on_select);
        let entry = roving.tab_stop(window);
        let radios = self.options.into_iter().enumerate().map(|(index, option)| {
            let on_change = self.on_change.clone();
            MaterialRadio::new(("radio", index), option.label)
                .visible_label(true)
                .label_before(self.labels_before)
                .selected(selected == Some(index))
                .disabled(disabled || !option.enabled)
                .track_focus(&focus[index])
                .tab_stop(entry == Some(index))
                .position(index + 1, count)
                .on_change(move |_, window, cx| {
                    if let Some(on_change) = &on_change {
                        on_change(&index, window, cx);
                    }
                })
        });
        let group = gpui_base::RadioGroup::new(self.id)
            .axis(self.axis)
            .aria_label(self.label)
            .flex()
            .map(|group| match self.axis {
                Axis::Vertical => group.flex_col(),
                Axis::Horizontal => group.flex_row().flex_wrap(),
            })
            .children(radios);
        roving.install(group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn radio_uses_the_m3_selection_state_policy() {
        let c = color;
        let on = selection_paint(true, true, ColorRole::PRIMARY, false);
        assert_eq!(on.ink, c(ColorRole::PRIMARY));
        let clear = Rgba {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 0.,
        };
        assert_eq!(
            on.hover,
            state::layer(clear, c(ColorRole::PRIMARY), state::HOVER)
        );
        assert_eq!(
            on.press,
            state::layer(clear, c(ColorRole::ON_SURFACE), state::PRESS),
            "a selected press layer is on-surface"
        );
        let off = selection_paint(false, true, ColorRole::PRIMARY, false);
        assert_eq!(off.ink, c(ColorRole::ON_SURFACE_VARIANT));
        assert_eq!(off.ink_active, c(ColorRole::ON_SURFACE));
        assert_eq!(
            off.press,
            state::layer(clear, c(ColorRole::PRIMARY), state::PRESS),
            "an unselected press layer is primary"
        );
        let disabled = selection_paint(true, false, ColorRole::PRIMARY, false);
        assert_eq!(disabled.ink, state::disabled(c(ColorRole::ON_SURFACE)));
        assert_eq!(disabled.hover.a, 0.);
        let error = selection_paint(false, true, ColorRole::ERROR, true);
        assert_eq!(error.ink, c(ColorRole::ERROR));
        assert_eq!(
            error.press,
            state::layer(clear, c(ColorRole::ERROR), state::PRESS)
        );
        assert_eq!((RING, DOT, STATE_LAYER, TARGET), (20., 10., 40., 48.));
    }

    struct GroupHarness {
        selected: Option<usize>,
        changes: Vec<usize>,
    }
    impl Render for GroupHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().w(px(400.)).child(
                MaterialRadioGroup::new("fruit", "Fruit")
                    .option("Apple", true)
                    .option("Banana", false)
                    .option("Cherry", true)
                    .selected(self.selected)
                    .on_change(cx.listener(|this, index: &usize, _, cx| {
                        this.selected = Some(*index);
                        this.changes.push(*index);
                        cx.notify();
                    })),
            )
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }

    #[gpui::test]
    fn a_radio_group_roves_with_arrow_keys(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            crate::components::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| GroupHarness {
            selected: Some(0),
            changes: Vec::new(),
        });
        draw(cx);
        cx.update(|window, cx| window.focus_next(cx));
        let first = cx.update(|window, cx| window.focused(cx));
        assert!(first.is_some());
        cx.update(|window, cx| window.focus_next(cx));
        assert_eq!(
            cx.update(|window, cx| window.focused(cx)),
            first,
            "the group is one tab stop"
        );
        cx.simulate_keystrokes("down");
        draw(cx);
        view.read_with(cx, |h, _| assert_eq!(h.changes, [2]));
        cx.simulate_keystrokes("down");
        draw(cx);
        view.read_with(cx, |h, _| assert_eq!(h.changes, [2, 0]));
        cx.simulate_keystrokes("end");
        draw(cx);
        view.read_with(cx, |h, _| assert_eq!(h.selected, Some(2)));
    }

    struct DotHarness(bool);
    impl Render for DotHarness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().child(MaterialRadio::new("dot", "Dot").selected(self.0))
        }
    }

    #[gpui::test]
    fn the_dot_scales_in_and_lands_at_10dp(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let (view, cx) = cx.add_window_view(|_, _| DotHarness(false));
        draw(cx);
        assert!(cx.debug_bounds("radio-dot").is_none());
        view.update(cx, |view, cx| {
            view.0 = true;
            cx.notify();
        });
        draw(cx);
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(60));
        draw(cx);
        let growing = cx.debug_bounds("radio-dot").expect("the dot is growing");
        assert!(growing.size.width < px(10.), "mid-animation: {growing:?}");
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(1));
        draw(cx);
        assert_eq!(
            cx.debug_bounds("radio-dot").unwrap().size,
            size(px(10.), px(10.))
        );
    }

    struct BeforeHarness(bool);
    impl Render for BeforeHarness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().flex().flex_col().w(px(240.)).child(
                div().debug_selector(|| "before-radio".into()).child(
                    MaterialRadio::new("before", "Express")
                        .visible_label(true)
                        .label_before(self.0)
                        .selected(true),
                ),
            )
        }
    }

    #[gpui::test]
    fn a_label_before_puts_the_radio_at_the_far_end(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| BeforeHarness(false));
        draw(cx);
        let row = cx.debug_bounds("before-radio").unwrap();
        let dot = cx.debug_bounds("radio-dot").unwrap();
        assert_eq!(
            dot.center().x - row.left(),
            px(24.),
            "after: the radio leads"
        );
        view.update(cx, |view, cx| {
            view.0 = true;
            cx.notify();
        });
        draw(cx);
        let dot = cx.debug_bounds("radio-dot").unwrap();
        assert_eq!(
            row.right() - dot.center().x,
            px(24.),
            "before: the radio trails"
        );
    }

    struct ClickHarness {
        selected: bool,
        disabled: bool,
    }
    impl Render for ClickHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().flex().items_start().child(
                div().debug_selector(|| "click-radio".into()).child(
                    MaterialRadio::new("click", "Click")
                        .selected(self.selected)
                        .disabled(self.disabled)
                        .on_change(cx.listener(|this, _: &bool, _, cx| {
                            this.selected = true;
                            cx.notify();
                        })),
                ),
            )
        }
    }

    #[gpui::test]
    fn clicking_selects_unless_disabled(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| ClickHarness {
            selected: false,
            disabled: true,
        });
        draw(cx);
        let target = cx.debug_bounds("click-radio").unwrap();
        assert_eq!(target.size.height, px(48.));
        cx.simulate_click(target.center(), Modifiers::default());
        draw(cx);
        view.read_with(cx, |h, _| assert!(!h.selected));
        view.update(cx, |h, cx| {
            h.disabled = false;
            cx.notify();
        });
        draw(cx);
        cx.simulate_click(target.center(), Modifiers::default());
        draw(cx);
        view.read_with(cx, |h, _| assert!(h.selected));
    }
}
