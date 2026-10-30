//! M3 chips: assist, filter, input and suggestion, each flat (outlined) or
//! elevated, in every interaction state.
//!
//! All chips are 32dp tall with 8dp corners, a label-large label and 18dp
//! icons 8dp from the label. Hover, focus and press are state layers of the
//! chip's content color over its container (focus also darkens a flat
//! chip's outline); a dragged chip lifts to level 4 under the drag layer.
//! A filter chip's check slides in from its leading edge on the fast
//! spatial spring as it is selected, and out as it is cleared.
use super::InteractionPreview;
use super::focus::{RingPlacement, RingedButton};
use crate::color::ColorRole::*;
use crate::icons::{Icon, IconName};
use crate::motion;
use crate::theme::color;
use crate::tokens::{elevation, shape::RADIUS_SMALL, state, typography::*};
use gpui::{prelude::*, *};
use std::rc::Rc;

/// M3 chip geometry, in dp: 32 tall, 18dp icons 8 from the label, a 24dp
/// avatar.
pub const CHIP_HEIGHT: Rems = rem(32.);
const CHIP_ICON: f32 = 18.;
const CHIP_GAP: f32 = 8.;
const AVATAR: f32 = 24.;
/// Edge padding beside a label, and beside an icon or avatar.
const ASSIST_TEXT_EDGE: f32 = 16.;
const INPUT_TEXT_EDGE: f32 = 12.;
const ICON_EDGE: f32 = 8.;
const AVATAR_EDGE: f32 = 4.;
/// Elevated chips rest at level 1; a dragged chip lifts to level 4.
const ELEVATED_LEVEL: u8 = 1;
const HOVERED_LEVEL: u8 = 2;
const DRAGGED_LEVEL: u8 = 4;

/// The container of an M3 assist chip: its geometry, small corners and
/// outline-variant edge. Callers give it their label color and size; prefer
/// [`AssistChip`], which adds the label role, state layers and focus.
pub fn chip(leading_icon: bool) -> Div {
    gpui_base::h_flex()
        .flex_shrink_0()
        .h(CHIP_HEIGHT)
        .pl(rem(if leading_icon {
            ICON_EDGE
        } else {
            ASSIST_TEXT_EDGE
        }))
        .pr(rem(ASSIST_TEXT_EDGE))
        .gap(rem(CHIP_GAP))
        .items_center()
        .rounded(RADIUS_SMALL)
        .border_1()
        .border_color(color(OUTLINE_VARIANT))
        .whitespace_nowrap()
}

/// Whether a chip is flat with an outline or raised on a tonal container.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChipStyle {
    /// Transparent with a 1dp outline-variant outline (the kit's "Outlined").
    #[default]
    Flat,
    /// Surface-container-low at elevation level 1, without an outline.
    Elevated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChipKind {
    Assist,
    Filter,
    Input,
    Suggestion,
}

/// The colors of a chip in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ChipPaint {
    container: Rgba,
    outline: Rgba,
    label: Rgba,
    leading: Rgba,
    trailing: Rgba,
    /// The state layers' content color.
    layer: Rgba,
    elevation: u8,
}

fn clear() -> Rgba {
    Rgba {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 0.,
    }
}

/// M3 chip colors. Selected filter, input and suggestion chips sit on the
/// secondary container without an outline; disabled chips use on-surface at
/// [`state::DISABLED_CONTAINER`] for containers and outlines and at
/// [`state::DISABLED_CONTENT`] for content, and elevated ones lie flat.
fn chip_paint(kind: ChipKind, style: ChipStyle, selected: bool, disabled: bool) -> ChipPaint {
    let selected = selected && kind != ChipKind::Assist;
    let elevated = style == ChipStyle::Elevated;
    let on_surface = color(ON_SURFACE);
    if disabled {
        let content = state::disabled(on_surface);
        let faint = on_surface.opacity(state::DISABLED_CONTAINER);
        return ChipPaint {
            container: if selected || elevated { faint } else { clear() },
            outline: if selected || elevated { clear() } else { faint },
            label: content,
            leading: content,
            trailing: content,
            layer: on_surface,
            elevation: 0,
        };
    }
    let elevation = if elevated { ELEVATED_LEVEL } else { 0 };
    if selected {
        let ink = color(ON_SECONDARY_CONTAINER);
        return ChipPaint {
            container: color(SECONDARY_CONTAINER),
            outline: clear(),
            label: ink,
            leading: if kind == ChipKind::Input {
                color(PRIMARY)
            } else {
                ink
            },
            trailing: ink,
            layer: ink,
            elevation,
        };
    }
    let (label, leading) = match kind {
        ChipKind::Assist => (ON_SURFACE, PRIMARY),
        ChipKind::Filter | ChipKind::Suggestion => (ON_SURFACE_VARIANT, PRIMARY),
        ChipKind::Input => (ON_SURFACE_VARIANT, ON_SURFACE_VARIANT),
    };
    let label = color(label);
    ChipPaint {
        container: if elevated {
            color(SURFACE_CONTAINER_LOW)
        } else {
            clear()
        },
        outline: if elevated {
            clear()
        } else {
            color(OUTLINE_VARIANT)
        },
        label,
        leading: color(leading),
        trailing: color(ON_SURFACE_VARIANT),
        layer: label,
        elevation,
    }
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type RemoveHandler = Rc<dyn Fn(&mut Window, &mut App)>;
type ToggleHandler = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// What every chip carries; each public chip sets the parts it offers.
struct ChipBase {
    id: ElementId,
    kind: ChipKind,
    label: SharedString,
    style: ChipStyle,
    selected: bool,
    disabled: bool,
    dragged: bool,
    preview: Option<InteractionPreview>,
    leading: Option<Icon>,
    avatar: Option<AnyElement>,
    trailing: Option<Icon>,
    remove: Option<(SharedString, RemoveHandler)>,
    on_click: Option<ClickHandler>,
    /// Called with the next selection; resolved at render so it sees the
    /// final `selected` whatever the builder order.
    on_toggle: Option<ToggleHandler>,
}

impl ChipBase {
    fn new(id: ElementId, kind: ChipKind, label: SharedString) -> Self {
        Self {
            id,
            kind,
            label,
            style: ChipStyle::Flat,
            selected: false,
            disabled: false,
            dragged: false,
            preview: None,
            leading: None,
            avatar: None,
            trailing: None,
            remove: None,
            on_click: None,
            on_toggle: None,
        }
    }

    fn render(self, window: &mut Window, cx: &mut App) -> RingedButton {
        let focus = window
            .use_keyed_state((self.id.clone(), "chip-focus"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        let preview = self.preview;
        let disabled = self.disabled || preview.is_some_and(|p| p.is_disabled());
        let enabled = !disabled;
        let live = enabled && preview.is_none();
        let forced = |state| enabled && preview == Some(state);
        let keyboard_focus = forced(InteractionPreview::Focused)
            || (live && focus.is_focused(window) && window.last_input_was_keyboard());
        let selected = self.selected && self.kind != ChipKind::Assist;
        let paint = chip_paint(self.kind, self.style, selected, disabled);
        let dragged = self.dragged && enabled;
        // A filter chip without its own leading icon slides a check in.
        let slide = self.kind == ChipKind::Filter && self.leading.is_none();
        let check = if slide {
            let target = if selected { 1. } else { 0. };
            if preview.is_some() {
                target
            } else {
                motion::selection_value((self.id.clone(), "check"), target, window, cx)
            }
        } else {
            0.
        };
        let check = if check.is_finite() { check.max(0.) } else { 0. };
        let base = if dragged {
            // A lifted flat chip gets the elevated container, so its shadow
            // does not show through.
            let lifted = if paint.container.a > 0. {
                paint.container
            } else {
                color(SURFACE_CONTAINER_LOW)
            };
            state::layer(lifted, paint.layer, state::DRAG)
        } else if keyboard_focus {
            state::layer(paint.container, paint.layer, state::FOCUS)
        } else {
            paint.container
        };
        let (hover, press) = (
            state::hovered(paint.container, paint.layer),
            state::pressed(paint.container, paint.layer),
        );
        let elevation = if dragged {
            DRAGGED_LEVEL
        } else if paint.elevation > 0 && forced(InteractionPreview::Hovered) {
            HOVERED_LEVEL
        } else {
            paint.elevation
        };
        // M3 darkens a flat chip's outline while it has focus.
        let outline = if keyboard_focus && paint.outline.a > 0. {
            color(ON_SURFACE_VARIANT)
        } else {
            paint.outline
        };
        let left = if self.avatar.is_some() {
            AVATAR_EDGE
        } else if self.leading.is_some() {
            ICON_EDGE
        } else {
            let text = self.text_edge();
            if slide {
                // The edge narrows as the check arrives.
                text + (ICON_EDGE - text) * check.min(1.)
            } else {
                text
            }
        };
        let has_trailing = self.trailing.is_some() || self.remove.is_some();
        let right = if has_trailing {
            ICON_EDGE
        } else {
            self.text_edge()
        };
        let scale = TypeRole::LabelLarge.scale();
        let icon_size = rem(CHIP_ICON);
        let leading_icon = self.leading.map(|icon| {
            // A selected filter chip shows the check in its icon's place.
            let icon = if selected && self.kind == ChipKind::Filter {
                IconName::Check.icon()
            } else {
                icon
            };
            icon.size(icon_size).color(paint.leading)
        });
        let check_slot = slide.then(|| {
            div()
                .flex_shrink_0()
                .overflow_hidden()
                .w(rem((CHIP_ICON + CHIP_GAP) * check))
                .opacity(check.min(1.))
                .when(cfg!(test), |slot| {
                    slot.debug_selector(|| "chip-check".into())
                })
                .child(
                    div()
                        .w(rem(CHIP_ICON + CHIP_GAP))
                        .child(IconName::Check.icon().size(icon_size).color(paint.leading)),
                )
        });
        let avatar = self.avatar.map(|avatar| {
            div()
                .flex()
                .flex_shrink_0()
                .items_center()
                .justify_center()
                .size(rem(AVATAR))
                .mr(rem(CHIP_GAP))
                .rounded_full()
                .overflow_hidden()
                .child(avatar)
        });
        let trailing_icon = self.trailing.map(|icon| {
            div()
                .flex_shrink_0()
                .ml(rem(CHIP_GAP))
                .child(icon.size(icon_size).color(paint.trailing))
        });
        let key_remove = self.remove.as_ref().map(|(_, handler)| handler.clone());
        let remove = self.remove.map(|(label, handler)| {
            let on_press = handler;
            div()
                .id("chip-remove")
                .flex_shrink_0()
                .ml(rem(CHIP_GAP))
                .role(Role::Button)
                .aria_label(label)
                .when(cfg!(test), |remove| {
                    remove.debug_selector(|| "chip-remove".into())
                })
                .child(IconName::Close.icon().size(icon_size).color(paint.trailing))
                .when(live, |remove| {
                    remove
                        .cursor_pointer()
                        // The chip's own click must not see a press that
                        // belongs to the remove icon.
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        // A click pairs the press with its release, so a
                        // drag that ends over the icon removes nothing.
                        .on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            on_press(window, cx);
                        })
                })
                .into_any_element()
        });
        let label = self.label;
        let next = !self.selected;
        let on_click = self.on_toggle.map(|toggle| -> ClickHandler {
            Rc::new(move |_, window, cx| toggle(&next, window, cx))
        });
        let on_click = on_click.or(self.on_click);
        let id = self.id;
        RingedButton::new(id, RingPlacement::Outer(Corners::all(RADIUS_SMALL)))
            .track_focus(&focus)
            .map(move |button| {
                button
                    .flex_shrink_0()
                    .h(CHIP_HEIGHT)
                    .pl(rem(left))
                    .pr(rem(right))
                    .rounded(RADIUS_SMALL)
                    .border_1()
                    .border_color(outline)
                    .bg(base)
                    .shadow(elevation::level(elevation))
                    .text_color(paint.label)
                    .text_size(scale.font_size)
                    .line_height(scale.line_height)
                    .font_weight(FontWeight(scale.font_weight as f32))
                    .whitespace_nowrap()
                    .selected(selected)
                    .accessibility_label(label.clone())
                    .when(live, |button| button.cursor_pointer())
                    .children(check_slot)
                    .children(avatar)
                    .when_some(leading_icon, |button, icon| {
                        button.child(div().flex_shrink_0().mr(rem(CHIP_GAP)).child(icon))
                    })
                    .child(label)
                    .children(trailing_icon)
                    .children(remove)
                    .when_some(on_click, |button, handler| {
                        button.on_click(move |event, window, cx| handler(event, window, cx))
                    })
            })
            .when_some(key_remove.filter(|_| live), |button, remove| {
                // Backspace and Delete remove a focused input chip.
                button.on_key_down(move |event, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "backspace" | "delete") {
                        cx.stop_propagation();
                        remove(window, cx);
                    }
                })
            })
            .state_layers(hover, press)
            .disabled(disabled)
            .when_some(preview, |button, preview| button.preview_state(preview))
    }

    fn text_edge(&self) -> f32 {
        if self.kind == ChipKind::Input {
            INPUT_TEXT_EDGE
        } else {
            ASSIST_TEXT_EDGE
        }
    }
}

/// Builder methods every chip shares.
macro_rules! chip_common {
    ($chip:ident) => {
        impl $chip {
            /// Render a static specimen in one forced interaction state,
            /// painting what the live state paints. See
            /// [`InteractionPreview`].
            pub fn preview_state(mut self, state: InteractionPreview) -> Self {
                self.0.preview = Some(state);
                self
            }
            /// Paint the chip as it looks while dragged: the drag state
            /// layer and level-4 elevation.
            pub fn dragged(mut self, dragged: bool) -> Self {
                self.0.dragged = dragged;
                self
            }
            pub fn disabled(mut self, disabled: bool) -> Self {
                self.0.disabled = disabled;
                self
            }
            /// A leading icon in its 18dp slot.
            pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
                self.0.leading = Some(icon.into());
                self
            }
        }
        impl RenderOnce for $chip {
            fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
                self.0.render(window, cx)
            }
        }
    };
}

/// M3 assist chip: a smart or automated action, with an optional primary
/// leading icon.
#[derive(gpui::IntoElement)]
pub struct AssistChip(ChipBase);
chip_common!(AssistChip);

impl AssistChip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self(ChipBase::new(id.into(), ChipKind::Assist, label.into()))
    }
    pub fn elevated(mut self, elevated: bool) -> Self {
        self.0.style = if elevated {
            ChipStyle::Elevated
        } else {
            ChipStyle::Flat
        };
        self
    }
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.0.on_click = Some(Rc::new(handler));
        self
    }
}

/// M3 filter chip: a toggle among filter options. Selecting slides a check
/// in before the label (or shows it in place of the leading icon).
#[derive(gpui::IntoElement)]
pub struct FilterChip(ChipBase);
chip_common!(FilterChip);

impl FilterChip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self(ChipBase::new(id.into(), ChipKind::Filter, label.into()))
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.0.selected = selected;
        self
    }
    pub fn elevated(mut self, elevated: bool) -> Self {
        self.0.style = if elevated {
            ChipStyle::Elevated
        } else {
            ChipStyle::Flat
        };
        self
    }
    /// A trailing icon, e.g. a drop-down arrow for a chip that opens a menu.
    pub fn trailing_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.0.trailing = Some(icon.into());
        self
    }
    /// Called with the chip's next selection when it is activated.
    pub fn on_toggle(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.0.on_toggle = Some(Rc::new(handler));
        self
    }
}

/// M3 input chip: a piece of information the person entered, with an
/// optional leading icon or avatar and a remove icon.
#[derive(gpui::IntoElement)]
pub struct InputChip(ChipBase);
chip_common!(InputChip);

impl InputChip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self(ChipBase::new(id.into(), ChipKind::Input, label.into()))
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.0.selected = selected;
        self
    }
    /// A 24dp avatar before the label, clipped to a circle.
    pub fn avatar(mut self, avatar: impl IntoElement) -> Self {
        self.0.avatar = Some(avatar.into_any_element());
        self
    }
    /// A trailing remove icon named `label` for accessibility; `handler`
    /// runs when it is clicked.
    pub fn on_remove(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.0.remove = Some((label.into(), Rc::new(handler)));
        self
    }
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.0.on_click = Some(Rc::new(handler));
        self
    }
}

/// M3 suggestion chip: a dynamically generated suggestion, such as a reply.
#[derive(gpui::IntoElement)]
pub struct SuggestionChip(ChipBase);
chip_common!(SuggestionChip);

impl SuggestionChip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self(ChipBase::new(id.into(), ChipKind::Suggestion, label.into()))
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.0.selected = selected;
        self
    }
    pub fn elevated(mut self, elevated: bool) -> Self {
        self.0.style = if elevated {
            ChipStyle::Elevated
        } else {
            ChipStyle::Flat
        };
        self
    }
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.0.on_click = Some(Rc::new(handler));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn chips_use_m3_geometry() {
        assert_eq!(CHIP_HEIGHT, rem(32.));
        assert_eq!(RADIUS_SMALL, px(8.));
        assert_eq!((CHIP_ICON, CHIP_GAP, AVATAR), (18., 8., 24.));
        assert_eq!(
            (ASSIST_TEXT_EDGE, INPUT_TEXT_EDGE, ICON_EDGE, AVATAR_EDGE),
            (16., 12., 8., 4.)
        );
    }

    #[test]
    fn chips_use_m3_roles() {
        let c = color;
        let assist = chip_paint(ChipKind::Assist, ChipStyle::Flat, false, false);
        assert_eq!(assist.outline, c(OUTLINE_VARIANT));
        assert_eq!(assist.label, c(ON_SURFACE));
        assert_eq!(assist.leading, c(PRIMARY));
        assert_eq!(assist.container.a, 0.);
        let elevated = chip_paint(ChipKind::Assist, ChipStyle::Elevated, false, false);
        assert_eq!(elevated.container, c(SURFACE_CONTAINER_LOW));
        assert_eq!(elevated.outline.a, 0.);
        assert_eq!(elevated.elevation, 1);
        let filter = chip_paint(ChipKind::Filter, ChipStyle::Flat, true, false);
        assert_eq!(filter.container, c(SECONDARY_CONTAINER));
        assert_eq!(filter.label, c(ON_SECONDARY_CONTAINER));
        assert_eq!(filter.outline.a, 0.);
        let input = chip_paint(ChipKind::Input, ChipStyle::Flat, false, false);
        assert_eq!(input.label, c(ON_SURFACE_VARIANT));
        assert_eq!(input.trailing, c(ON_SURFACE_VARIANT));
        let suggestion = chip_paint(ChipKind::Suggestion, ChipStyle::Flat, false, false);
        assert_eq!(suggestion.label, c(ON_SURFACE_VARIANT));
        let disabled = chip_paint(ChipKind::Assist, ChipStyle::Flat, false, true);
        assert_eq!(disabled.label, state::disabled(c(ON_SURFACE)));
        assert_eq!(
            disabled.outline,
            c(ON_SURFACE).opacity(state::DISABLED_CONTAINER)
        );
        let disabled = chip_paint(ChipKind::Filter, ChipStyle::Elevated, true, true);
        assert_eq!(
            disabled.container,
            c(ON_SURFACE).opacity(state::DISABLED_CONTAINER)
        );
        assert_eq!(disabled.elevation, 0, "disabled chips lie flat");
        assert_eq!(
            chip_paint(ChipKind::Assist, ChipStyle::Flat, true, false),
            assist,
            "assist chips have no selection"
        );
    }

    struct ChipHarness {
        selected: bool,
        removed: usize,
        clicks: usize,
    }
    impl Render for ChipHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div()
                .flex()
                .items_start()
                .child(
                    div()
                        .debug_selector(|| "plain-chip".into())
                        .child(AssistChip::new("plain-chip", "Plain")),
                )
                .child(
                    div()
                        .debug_selector(|| "icon-chip".into())
                        .child(AssistChip::new("icon-chip", "Plain").icon(IconName::Info)),
                )
                .child(
                    div().debug_selector(|| "filter-chip".into()).child(
                        // The toggle comes before `selected` so the next
                        // value must be resolved at render.
                        FilterChip::new("filter-chip", "Plain")
                            .on_toggle(cx.listener(|this, next: &bool, _, cx| {
                                this.selected = *next;
                                cx.notify();
                            }))
                            .selected(self.selected),
                    ),
                )
                .child(
                    div().debug_selector(|| "input-chip".into()).child(
                        InputChip::new("input-chip", "Plain")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.clicks += 1;
                                cx.notify();
                            }))
                            .on_remove("Remove", {
                                let view = cx.entity();
                                move |_, cx| {
                                    view.update(cx, |this, cx| {
                                        this.removed += 1;
                                        cx.notify();
                                    })
                                }
                            }),
                    ),
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
    fn a_leading_icon_takes_the_chip_padding_it_replaces(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let (_, cx) = cx.add_window_view(|_, _| ChipHarness {
            selected: false,
            removed: 0,
            clicks: 0,
        });
        draw(cx);
        let plain = cx.debug_bounds("plain-chip").unwrap();
        let icon = cx.debug_bounds("icon-chip").unwrap();
        assert_eq!(plain.size.height, px(32.));
        // 18dp icon + 8dp gap replace 8dp of the 16dp leading padding.
        assert_eq!(icon.size.width - plain.size.width, px(18. + 8. - 8.));
    }

    #[gpui::test]
    fn selecting_a_filter_chip_slides_its_check_in(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| ChipHarness {
            selected: false,
            removed: 0,
            clicks: 0,
        });
        draw(cx);
        let rest = cx.debug_bounds("filter-chip").unwrap();
        assert_eq!(cx.debug_bounds("chip-check").unwrap().size.width, px(0.));
        cx.simulate_click(rest.center(), Modifiers::default());
        draw(cx);
        view.read_with(cx, |h, _| assert!(h.selected));
        let selected = cx.debug_bounds("filter-chip").unwrap();
        // The 16dp edge becomes 8dp, plus the 18dp check and its 8dp gap.
        assert_eq!(selected.size.width - rest.size.width, px(18. + 8. - 8.));
        cx.simulate_click(selected.center(), Modifiers::default());
        draw(cx);
        view.read_with(cx, |h, _| assert!(!h.selected, "a second click clears it"));
    }

    #[gpui::test]
    fn the_remove_icon_does_not_click_the_chip(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| ChipHarness {
            selected: false,
            removed: 0,
            clicks: 0,
        });
        draw(cx);
        let remove = cx.debug_bounds("chip-remove").unwrap();
        cx.simulate_click(remove.center(), Modifiers::default());
        draw(cx);
        view.read_with(cx, |h, _| assert_eq!((h.removed, h.clicks), (1, 0)));
        let chip = cx.debug_bounds("input-chip").unwrap();
        cx.simulate_click(
            point(chip.left() + px(14.), chip.center().y),
            Modifiers::default(),
        );
        draw(cx);
        view.read_with(cx, |h, _| assert_eq!((h.removed, h.clicks), (1, 1)));
    }

    #[gpui::test]
    fn a_press_released_over_the_remove_icon_removes_nothing(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| ChipHarness {
            selected: false,
            removed: 0,
            clicks: 0,
        });
        draw(cx);
        let remove = cx.debug_bounds("chip-remove").unwrap().center();
        let chip = cx.debug_bounds("input-chip").unwrap();
        let label = point(chip.left() + px(14.), chip.center().y);
        cx.simulate_mouse_down(label, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(remove, Some(MouseButton::Left), Modifiers::default());
        cx.simulate_mouse_up(remove, MouseButton::Left, Modifiers::default());
        draw(cx);
        view.read_with(cx, |h, _| assert_eq!(h.removed, 0));
    }
}
