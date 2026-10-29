//! M3 list items: one-, two- and three-line rows with leading and trailing
//! elements, dividers, selection and every interaction state.
//!
//! Geometry and roles follow the Figma "Material 3 Design Kit" list item
//! (`List item/List Item: -4 Density (baseline)`) and m3e.kyralabs.dev's
//! list: a surface row with 16dp side padding and 16dp gaps, body-large
//! headline (on-surface), label-medium overline and body-medium supporting
//! text (on-surface-variant), label-small trailing text, a 40dp monogram,
//! 24dp icons, 56dp image and 114×64 video thumbnails, and an
//! outline-variant divider. The Expressive segmented list groups items 2dp
//! apart with 16dp outer and 4dp inner corners ([`segmented_list`]).
//!
//! An item that acts is a GPUI Base button ([`RingedButton`]) with the M3
//! state layers and the inner keyboard focus ring; embedded checkboxes,
//! radios and switches show the row's state and are not separate tab stops,
//! so the whole row is the one target, as M3 specifies.
use super::InteractionPreview;
use super::focus::{RingPlacement, RingedButton};
use super::{CheckboxState, MaterialCheckbox, MaterialRadio, MaterialSwitch};
use crate::{
    color::ColorRole,
    icons::Icon,
    theme::color,
    tokens::{
        shape::{OuterCorners, list_corners},
        state,
        typography::{TypeRole, rem},
    },
};
use gpui::{prelude::*, *};
use std::rc::Rc;

/// Outer corners of an Expressive segmented list.
pub const LIST_SEGMENT_RADIUS: Pixels = px(16.);
/// Corners between neighbouring items of a segmented list.
pub const LIST_SEGMENT_INNER_RADIUS: Pixels = px(4.);
/// The ground between items of a segmented list.
pub const LIST_SEGMENT_GAP: Pixels = px(2.);

/// The leading element of a list item.
// A short-lived builder value consumed by one render; boxing the icon
// would only complicate callers.
#[allow(clippy::large_enum_variant)]
pub enum ListLeading {
    None,
    /// A 40dp monogram avatar on primary-container.
    Monogram(SharedString),
    /// A 24dp icon in on-surface-variant.
    Icon(Icon),
    /// A 56dp square thumbnail; `None` paints the placeholder container.
    Image(Option<AnyElement>),
    /// A 114×64 video thumbnail; `None` paints the placeholder container.
    Video(Option<AnyElement>),
    Checkbox(bool),
    Radio(bool),
    Switch(bool),
}

/// The trailing element of a list item.
// A short-lived builder value consumed by one render; boxing the icon
// would only complicate callers.
#[allow(clippy::large_enum_variant)]
pub enum ListTrailing {
    None,
    /// A 24dp icon in on-surface-variant (e.g. an arrow).
    Icon(Icon),
    Checkbox(bool),
    Radio(bool),
    Switch(bool),
}

/// Vertical density. Standard is the M3 list (56/72/88dp); Compact is the
/// design kit's −4 density baseline (40/56/72dp, 88dp with an overline over
/// two supporting lines).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListDensity {
    #[default]
    Standard,
    Compact,
}

/// How many lines an item's text occupies, which sets its height and the
/// leading element's alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListLines {
    One,
    Two,
    Three,
}

impl ListLines {
    /// The lines of an item with these texts. Supporting text alone wraps to
    /// `supporting_lines`; an overline adds a line.
    pub fn of(overline: bool, supporting: bool, supporting_lines: usize) -> Self {
        let lines = 1 + usize::from(overline) + if supporting { supporting_lines } else { 0 };
        match lines {
            1 => Self::One,
            2 => Self::Two,
            _ => Self::Three,
        }
    }
}

/// The minimum height and vertical padding of a row, in dp.
fn row_metrics(lines: ListLines, density: ListDensity) -> (f32, f32) {
    match (density, lines) {
        (ListDensity::Standard, ListLines::One) => (56., 8.),
        (ListDensity::Standard, ListLines::Two) => (72., 8.),
        (ListDensity::Standard, ListLines::Three) => (88., 12.),
        (ListDensity::Compact, ListLines::One) => (40., 0.),
        (ListDensity::Compact, ListLines::Two) => (56., 4.),
        (ListDensity::Compact, ListLines::Three) => (72., 4.),
    }
}

/// Paint of a row in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ListPaint {
    container: Rgba,
    headline: Rgba,
    support: Rgba,
    /// The state layer's content color.
    layer: Rgba,
}

/// Selected rows take the secondary container (m3e list item); disabled
/// content is on-surface at the disabled-content opacity over the surface.
fn list_paint(selected: bool, disabled: bool) -> ListPaint {
    if disabled {
        let ink = state::disabled(color(ColorRole::ON_SURFACE));
        return ListPaint {
            container: if selected {
                color(ColorRole::ON_SURFACE).opacity(state::DISABLED_CONTAINER)
            } else {
                color(ColorRole::SURFACE)
            },
            headline: ink,
            support: ink,
            layer: color(ColorRole::ON_SURFACE),
        };
    }
    if selected {
        ListPaint {
            container: color(ColorRole::SECONDARY_CONTAINER),
            headline: color(ColorRole::ON_SECONDARY_CONTAINER),
            support: color(ColorRole::ON_SECONDARY_CONTAINER),
            layer: color(ColorRole::ON_SECONDARY_CONTAINER),
        }
    } else {
        ListPaint {
            container: color(ColorRole::SURFACE),
            headline: color(ColorRole::ON_SURFACE),
            support: color(ColorRole::ON_SURFACE_VARIANT),
            layer: color(ColorRole::ON_SURFACE),
        }
    }
}

type Click = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// One M3 list item. Strings come from the caller. Give it `on_click` to make
/// the whole row a button; without one it is static content.
#[derive(gpui::IntoElement)]
pub struct ListItem {
    id: ElementId,
    headline: SharedString,
    overline: Option<SharedString>,
    supporting: Option<SharedString>,
    supporting_lines: usize,
    trailing_text: Option<SharedString>,
    leading: ListLeading,
    trailing: ListTrailing,
    divider: bool,
    selected: bool,
    disabled: bool,
    density: ListDensity,
    corners: Option<OuterCorners>,
    container: ColorRole,
    preview: Option<InteractionPreview>,
    on_click: Option<Click>,
}

impl ListItem {
    pub fn new(id: impl Into<ElementId>, headline: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            headline: headline.into(),
            overline: None,
            supporting: None,
            supporting_lines: 1,
            trailing_text: None,
            leading: ListLeading::None,
            trailing: ListTrailing::None,
            divider: false,
            selected: false,
            disabled: false,
            density: ListDensity::Standard,
            corners: None,
            container: ColorRole::SURFACE,
            preview: None,
            on_click: None,
        }
    }
    /// A label-medium line above the headline.
    pub fn overline(mut self, text: impl Into<SharedString>) -> Self {
        self.overline = Some(text.into());
        self
    }
    /// Body-medium text under the headline, on one line (ellipsized).
    pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }
    /// Let the supporting text wrap to two lines: the three-line item.
    pub fn three_line(mut self, three_line: bool) -> Self {
        self.supporting_lines = if three_line { 2 } else { 1 };
        self
    }
    /// Label-small text before the trailing element (e.g. "100+").
    pub fn trailing_text(mut self, text: impl Into<SharedString>) -> Self {
        self.trailing_text = Some(text.into());
        self
    }
    pub fn leading(mut self, leading: ListLeading) -> Self {
        self.leading = leading;
        self
    }
    pub fn trailing(mut self, trailing: ListTrailing) -> Self {
        self.trailing = trailing;
        self
    }
    /// A full-width outline-variant divider under the item.
    pub fn divider(mut self, divider: bool) -> Self {
        self.divider = divider;
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// The resting container role (surface by default), e.g. a dialog's
    /// surface-container-high so items sit on it. Selected rows keep the
    /// secondary container.
    pub fn container(mut self, role: ColorRole) -> Self {
        self.container = role;
        self
    }
    pub fn density(mut self, density: ListDensity) -> Self {
        self.density = density;
        self
    }
    /// Round the item as a segment of an Expressive segmented list
    /// (top-left, top-right, bottom-right, bottom-left; true = outside).
    pub fn segment(mut self, corners: OuterCorners) -> Self {
        self.corners = Some(corners);
        self
    }
    /// Render a static specimen in one forced interaction state: the state
    /// layer, the inner focus ring or the disabled row. See
    /// [`InteractionPreview`].
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
    /// Make the whole row a button. Rows with a checkbox, radio or switch
    /// toggle it here: the embedded control only shows the state.
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }

    /// The lines this item occupies.
    pub fn lines(&self) -> ListLines {
        ListLines::of(
            self.overline.is_some(),
            self.supporting.is_some(),
            self.supporting_lines,
        )
    }
}

/// The radii of a segment with `outer` corners.
fn segment_radii(outer: OuterCorners) -> Corners<Pixels> {
    let radius = |outside: bool| {
        if outside {
            LIST_SEGMENT_RADIUS
        } else {
            LIST_SEGMENT_INNER_RADIUS
        }
    };
    Corners {
        top_left: radius(outer[0]),
        top_right: radius(outer[1]),
        bottom_right: radius(outer[2]),
        bottom_left: radius(outer[3]),
    }
}

fn typed<E: Styled>(element: E, role: TypeRole) -> E {
    let scale = role.scale();
    element
        .font_family(scale.font_family)
        .text_size(scale.font_size)
        .line_height(scale.line_height)
        .font_weight(FontWeight(scale.font_weight as f32))
}

/// A 24dp slot that centers a 48dp control target on the item's ruler.
fn control_slot(width: f32, control: impl IntoElement) -> Div {
    div()
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .w(rem(width))
        .h(rem(24.))
        .child(control)
}

/// The preview an embedded control shows: the row's own disabled state,
/// otherwise enabled at rest. Embedded controls never take focus.
fn control_preview(disabled: bool) -> InteractionPreview {
    if disabled {
        InteractionPreview::Disabled
    } else {
        InteractionPreview::Enabled
    }
}

/// The selection control embedded in a list item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Control {
    Checkbox,
    Radio,
    Switch,
}

fn control(
    id: &'static str,
    kind: Control,
    label: &SharedString,
    on: bool,
    disabled: bool,
) -> AnyElement {
    let preview = control_preview(disabled);
    match kind {
        Control::Checkbox => control_slot(
            24.,
            MaterialCheckbox::new(id, label.clone())
                .state(if on {
                    CheckboxState::Checked
                } else {
                    CheckboxState::Unchecked
                })
                .preview_state(preview),
        )
        .into_any_element(),
        Control::Radio => control_slot(
            24.,
            MaterialRadio::new(id, label.clone())
                .selected(on)
                .preview_state(preview),
        )
        .into_any_element(),
        Control::Switch => div()
            .flex_shrink_0()
            .child(
                MaterialSwitch::new(id, label.clone())
                    .checked(on)
                    .preview_state(preview),
            )
            .into_any_element(),
    }
}

/// A placeholder or caller thumbnail of `width`×`height` dp.
fn thumbnail(width: f32, height: f32, content: Option<AnyElement>) -> Div {
    div()
        .flex_shrink_0()
        .w(rem(width))
        .h(rem(height))
        .overflow_hidden()
        .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
        .children(content)
}

impl ListItem {
    fn leading_element(&mut self, ink: Rgba, disabled: bool) -> Option<AnyElement> {
        let label = self.headline.clone();
        let leading = std::mem::replace(&mut self.leading, ListLeading::None);
        Some(match leading {
            ListLeading::None => return None,
            ListLeading::Monogram(text) => typed(
                div()
                    .flex()
                    .flex_shrink_0()
                    .items_center()
                    .justify_center()
                    .size(rem(40.))
                    .rounded_full()
                    .bg(if disabled {
                        color(ColorRole::ON_SURFACE).opacity(state::DISABLED_CONTAINER)
                    } else {
                        color(ColorRole::PRIMARY_CONTAINER)
                    })
                    .text_color(if disabled {
                        ink
                    } else {
                        color(ColorRole::ON_PRIMARY_CONTAINER)
                    }),
                TypeRole::TitleMedium,
            )
            .child(text)
            .into_any_element(),
            ListLeading::Icon(icon) => div()
                .flex_shrink_0()
                .size(rem(24.))
                .child(icon.size(rem(24.)).color(ink))
                .into_any_element(),
            ListLeading::Image(content) => thumbnail(56., 56., content).into_any_element(),
            ListLeading::Video(content) => thumbnail(114., 64., content).into_any_element(),
            ListLeading::Checkbox(checked) => {
                control("checkbox", Control::Checkbox, &label, checked, disabled)
            }
            ListLeading::Radio(selected) => {
                control("radio", Control::Radio, &label, selected, disabled)
            }
            ListLeading::Switch(checked) => {
                control("switch", Control::Switch, &label, checked, disabled)
            }
        })
    }

    fn trailing_element(&mut self, ink: Rgba, disabled: bool) -> Option<AnyElement> {
        let label = self.headline.clone();
        let trailing = std::mem::replace(&mut self.trailing, ListTrailing::None);
        Some(match trailing {
            ListTrailing::None => return None,
            ListTrailing::Icon(icon) => div()
                .flex_shrink_0()
                .size(rem(24.))
                .child(icon.size(rem(24.)).color(ink))
                .into_any_element(),
            ListTrailing::Checkbox(checked) => control(
                "trailing-checkbox",
                Control::Checkbox,
                &label,
                checked,
                disabled,
            ),
            ListTrailing::Radio(selected) => {
                control("trailing-radio", Control::Radio, &label, selected, disabled)
            }
            ListTrailing::Switch(checked) => control(
                "trailing-switch",
                Control::Switch,
                &label,
                checked,
                disabled,
            ),
        })
    }
}

impl RenderOnce for ListItem {
    fn render(mut self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let preview = self.preview;
        let disabled = self.disabled || preview.is_some_and(InteractionPreview::is_disabled);
        let lines = self.lines();
        let (min_height, pad_y) = row_metrics(lines, self.density);
        let mut paint = list_paint(self.selected, disabled);
        if !self.selected {
            paint.container = color(self.container);
        }
        let leading = self.leading_element(paint.support, disabled);
        let trailing = self.trailing_element(paint.support, disabled);
        let top_aligned = lines == ListLines::Three;
        let supporting_lines = self.supporting_lines;

        let text = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .when_some(self.overline.clone(), |column, overline| {
                column.child(
                    typed(div().id("overline"), TypeRole::LabelMedium)
                        .text_color(paint.support)
                        .truncate()
                        .child(overline),
                )
            })
            .child(
                typed(div().id("headline"), TypeRole::BodyLarge)
                    .text_color(paint.headline)
                    .truncate()
                    .child(self.headline.clone()),
            )
            .when_some(self.supporting.clone(), |column, supporting| {
                column.child(
                    typed(div().id("supporting"), TypeRole::BodyMedium)
                        .text_color(paint.support)
                        .whitespace_normal()
                        .line_clamp(supporting_lines)
                        .text_ellipsis()
                        .child(supporting),
                )
            });

        let trailing_text = self.trailing_text.clone().map(|value| {
            typed(div().flex_shrink_0(), TypeRole::LabelSmall)
                .text_color(paint.support)
                .child(value)
        });

        let row = div()
            .flex()
            .flex_1()
            .w_full()
            .min_w_0()
            .gap(rem(16.))
            .px(rem(16.))
            .py(rem(pad_y))
            .min_h(rem(min_height))
            .map(|row| {
                if top_aligned {
                    row.items_start()
                } else {
                    row.items_center()
                }
            })
            .children(leading)
            .child(text)
            .when(trailing_text.is_some() || trailing.is_some(), |row| {
                row.child(
                    div()
                        .flex()
                        .flex_shrink_0()
                        .items_center()
                        .gap(rem(10.))
                        .children(trailing_text)
                        .children(trailing),
                )
            });

        let radii = self.corners.map(segment_radii).unwrap_or_default();
        let divider = self.divider;
        let container = paint.container;
        let decorate = move |element: Div| {
            element
                .w_full()
                .min_w_0()
                .flex()
                .flex_col()
                .bg(container)
                .rounded_tl(radii.top_left)
                .rounded_tr(radii.top_right)
                .rounded_br(radii.bottom_right)
                .rounded_bl(radii.bottom_left)
                .when(divider, |item| {
                    item.border_b_1()
                        .border_color(color(ColorRole::OUTLINE_VARIANT))
                })
        };

        let Some(on_click) = self.on_click.take() else {
            // Static content: a plain row that still shows a forced state's
            // layer for specimens.
            let layer = preview.map(|p| p.layer_opacity()).unwrap_or(0.);
            return decorate(div())
                .id(self.id)
                .when(layer > 0. && !disabled, |item| {
                    item.bg(state::layer(container, paint.layer, layer))
                })
                .child(row)
                .into_any_element();
        };

        let (hover, press) = (
            state::hovered(container, paint.layer),
            state::pressed(container, paint.layer),
        );
        let mut button = RingedButton::new(self.id, RingPlacement::Inner(radii))
            .map(|button| {
                button
                    .w_full()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .items_stretch()
                    .text_left()
                    .whitespace_normal()
                    .bg(container)
                    .rounded_tl(radii.top_left)
                    .rounded_tr(radii.top_right)
                    .rounded_br(radii.bottom_right)
                    .rounded_bl(radii.bottom_left)
                    .when(divider, |item| {
                        item.border_b_1()
                            .border_color(color(ColorRole::OUTLINE_VARIANT))
                    })
            })
            .accessibility_label(self.headline)
            .state_layers(hover, press)
            .disabled(disabled)
            .on_click(move |event, window, cx| on_click(event, window, cx))
            .child(row);
        if let Some(state) = preview {
            button = button.preview_state(state);
        }
        button.into_any_element()
    }
}

/// An Expressive segmented list: each item on its own surface, 2dp apart,
/// with 16dp outer and 4dp inner corners.
pub fn segmented_list(items: impl IntoIterator<Item = ListItem>) -> Div {
    let items: Vec<ListItem> = items.into_iter().collect();
    let count = items.len();
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(LIST_SEGMENT_GAP)
        .children(
            items
                .into_iter()
                .enumerate()
                .map(move |(index, item)| item.segment(list_corners(index, count))),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::focus::test_rings;
    use core::prelude::v1::test;
    use gpui::{Context, TestAppContext, VisualTestContext};
    use std::cell::Cell;

    #[test]
    fn lines_follow_the_overline_and_supporting_text() {
        assert_eq!(ListLines::of(false, false, 1), ListLines::One);
        assert_eq!(ListLines::of(false, true, 1), ListLines::Two);
        assert_eq!(ListLines::of(true, false, 1), ListLines::Two);
        assert_eq!(ListLines::of(false, true, 2), ListLines::Three);
        assert_eq!(ListLines::of(true, true, 1), ListLines::Three);
    }

    #[test]
    fn heights_match_m3_and_the_kit_density() {
        assert_eq!(row_metrics(ListLines::One, ListDensity::Standard).0, 56.);
        assert_eq!(row_metrics(ListLines::Two, ListDensity::Standard).0, 72.);
        assert_eq!(row_metrics(ListLines::Three, ListDensity::Standard).0, 88.);
        assert_eq!(row_metrics(ListLines::One, ListDensity::Compact).0, 40.);
        assert_eq!(row_metrics(ListLines::Two, ListDensity::Compact).0, 56.);
        assert_eq!(row_metrics(ListLines::Three, ListDensity::Compact).0, 72.);
    }

    #[test]
    fn selected_and_disabled_rows_use_their_roles() {
        let enabled = list_paint(false, false);
        assert_eq!(enabled.container, color(ColorRole::SURFACE));
        assert_eq!(enabled.headline, color(ColorRole::ON_SURFACE));
        assert_eq!(enabled.support, color(ColorRole::ON_SURFACE_VARIANT));
        let selected = list_paint(true, false);
        assert_eq!(selected.container, color(ColorRole::SECONDARY_CONTAINER));
        assert_eq!(selected.headline, color(ColorRole::ON_SECONDARY_CONTAINER));
        let disabled = list_paint(false, true);
        assert_eq!(
            disabled.headline,
            state::disabled(color(ColorRole::ON_SURFACE))
        );
    }

    #[test]
    fn segments_round_only_their_outer_corners() {
        let radii = segment_radii(list_corners(0, 3));
        assert_eq!(radii.top_left, LIST_SEGMENT_RADIUS);
        assert_eq!(radii.bottom_left, LIST_SEGMENT_INNER_RADIUS);
        let last = segment_radii(list_corners(2, 3));
        assert_eq!(last.bottom_right, LIST_SEGMENT_RADIUS);
        assert_eq!(last.top_right, LIST_SEGMENT_INNER_RADIUS);
    }

    struct Harness {
        clicks: Rc<Cell<usize>>,
        disabled: bool,
        checked: bool,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(16.));
            let clicks = self.clicks.clone();
            div()
                .w(px(360.))
                .child(
                    ListItem::new("item", "List item")
                        .supporting("Supporting")
                        .leading(ListLeading::Monogram("A".into()))
                        .trailing(ListTrailing::Switch(self.checked))
                        .disabled(self.disabled)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            clicks.set(clicks.get() + 1);
                            this.checked = !this.checked;
                            cx.notify();
                        })),
                )
                .child(
                    div()
                        .debug_selector(|| "one".into())
                        .child(ListItem::new("one", "One")),
                )
                .child(
                    div().debug_selector(|| "three".into()).child(
                        ListItem::new("three", "Three")
                            .supporting("Short")
                            .three_line(true),
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
    fn the_row_toggles_its_switch_and_ignores_input_while_disabled(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let clicks = Rc::new(Cell::new(0));
        let (view, cx) = cx.add_window_view({
            let clicks = clicks.clone();
            move |_, _| Harness {
                clicks,
                disabled: false,
                checked: false,
            }
        });
        draw(cx);
        let three = cx.debug_bounds("three").unwrap();
        let one = cx.debug_bounds("one").unwrap();
        // The test text system's line boxes differ from the platform's, so
        // only the minimum heights are exact here.
        assert!(
            one.size.height >= px(56.),
            "one-line items are at least 56dp"
        );
        assert!(
            three.size.height >= px(88.),
            "three-line items are at least 88dp"
        );
        let row = point(px(100.), one.top() - px(20.));
        cx.simulate_click(row, Modifiers::default());
        draw(cx);
        assert_eq!(clicks.get(), 1);
        assert!(view.read_with(cx, |v, _| v.checked));
        // Keyboard: the row is one tab stop with the inner ring.
        cx.update(|window, cx| window.focus_next(cx));
        let keystroke =
            cx.update(|_, cx| gpui::Keystroke::parse_with_os("a", cx.platform_info().os).unwrap());
        cx.simulate_event(gpui::KeyDownEvent {
            keystroke,
            is_held: false,
            prefer_character_input: false,
        });
        draw(cx);
        assert!(test_rings::painted("item").is_some());
        view.update(cx, |v, cx| {
            v.disabled = true;
            cx.notify();
        });
        draw(cx);
        cx.simulate_click(row, Modifiers::default());
        draw(cx);
        assert_eq!(clicks.get(), 1, "a disabled row is inert");
    }
}
