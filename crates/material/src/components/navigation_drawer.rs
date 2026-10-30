//! M3 navigation drawer, standard and modal.
//!
//! Geometry and color follow the Figma kit's Navigation Drawer (the
//! "Navigation" page): a surface-container-low sheet with 12dp padding,
//! 56dp destinations (label-large, 24dp icon, 12dp gap, 16dp leading and
//! 24dp trailing padding) on a full-pill secondary-container indicator,
//! 56dp title-small section headers and headline, and middle-inset
//! outline-variant dividers between sections. Destination badges are the
//! kit's trailing label-large text (e.g. "100+").
//!
//! - A standard drawer sits in the layout beside the content; opening and
//!   closing springs its width on the default spatial spring.
//! - A modal drawer floats over the content at level 1 with large trailing
//!   corners, over a 32% scrim. It slides in from its edge on the default
//!   spatial spring while the scrim fades; Escape or a scrim click calls
//!   [`NavigationDrawer::on_dismiss`], and opening moves focus to the
//!   selected destination.
//!
//! Selection cross-fades the indicator and colors on the fast effects
//! spring. Reduced motion finishes spatial motion at once and keeps fades.
//!
//! Behavior is shared with the navigation rail: each destination is a GPUI
//! Base `Tab` in a vertical Base `Tabs` list inside a navigation landmark;
//! arrow keys rove focus (see [`super::roving`]), Enter or Space activates.
use super::button::{ButtonFocusRing, ButtonRipple};
use super::navigation::{
    ClickHandler, ITEM_GROUP, NavBadge, NavItem, destinations, item_click, ripple_corners,
    type_role,
};
use super::roving::{Change, Orientation};
use super::{InteractionPreview, small_badge};
use crate::color::ColorRole;
use crate::icons::Icon;
use crate::motion::{self, MotionToken};
use crate::theme::{blend, color};
use crate::tokens::{
    elevation, state,
    typography::{TypeRole, rem},
};
use gpui::{prelude::FluentBuilder as _, *};
use std::rc::Rc;

/// The M3 drawer width (360dp maximum).
pub const DRAWER_WIDTH: f32 = 360.;
/// The sheet's padding around its content.
const DRAWER_PADDING: f32 = 12.;
/// A destination's, section header's and headline's height.
const ITEM_HEIGHT: f32 = 56.;
/// The dense height.
const DENSE_ITEM_HEIGHT: f32 = 48.;
const ITEM_LEADING: f32 = 16.;
const ITEM_TRAILING: f32 = 24.;
const ITEM_GAP: f32 = 12.;
/// The modal drawer's trailing corner radius (shape large).
const MODAL_RADIUS: f32 = 16.;

/// Which window edge a drawer sits on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrawerPosition {
    #[default]
    Start,
    End,
}

/// The resolved paint of one destination for the current motion frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DrawerPaint {
    pub icon: Rgba,
    pub label: Rgba,
    pub indicator: Rgba,
    pub hover: Rgba,
    pub press: Rgba,
}

/// `fade` is the fast effects amount, 0 unselected and 1 selected.
pub(crate) fn drawer_paint(selected: bool, fade: f32, disabled: bool) -> DrawerPaint {
    let fade = fade.clamp(0., 1.);
    if disabled {
        let ink = state::disabled(color(ColorRole::ON_SURFACE));
        return DrawerPaint {
            icon: ink,
            label: ink,
            indicator: color(ColorRole::ON_SURFACE).opacity(state::DISABLED_CONTAINER * fade),
            hover: Rgba::default(),
            press: Rgba::default(),
        };
    }
    let rest = color(ColorRole::ON_SURFACE_VARIANT);
    let active = color(ColorRole::ON_SECONDARY_CONTAINER);
    let layer = if selected {
        active
    } else {
        color(ColorRole::ON_SURFACE)
    };
    DrawerPaint {
        icon: blend(rest, active, fade),
        label: blend(rest, active, fade),
        indicator: color(ColorRole::SECONDARY_CONTAINER).opacity(fade),
        hover: layer.opacity(state::HOVER),
        press: layer.opacity(state::PRESS),
    }
}

#[cfg(test)]
thread_local! {
    pub(crate) static DRAWER_PAINTED: std::cell::RefCell<std::collections::BTreeMap<String, DrawerPaint>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
    /// The last rendered open amount and sheet width in design px.
    pub(crate) static DRAWER_OPEN: std::cell::Cell<(f32, f32)> = const { std::cell::Cell::new((0., 0.)) };
}

/// An M3 navigation drawer over caller-controlled selection and open state.
#[derive(IntoElement)]
pub struct NavigationDrawer {
    id: ElementId,
    label: SharedString,
    headline: Option<SharedString>,
    items: Vec<NavItem>,
    sections: Vec<(usize, SharedString)>,
    selected: Option<SharedString>,
    modal: bool,
    open: bool,
    dense: bool,
    position: DrawerPosition,
    width: f32,
    icons: bool,
    ripple: bool,
    on_change: Option<Change>,
    on_dismiss: Option<ClickHandler>,
}

impl NavigationDrawer {
    /// `label` is the navigation landmark's accessible name.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            headline: None,
            items: Vec::new(),
            sections: Vec::new(),
            selected: None,
            modal: false,
            open: true,
            dense: false,
            position: DrawerPosition::Start,
            width: DRAWER_WIDTH,
            icons: true,
            ripple: true,
            on_change: None,
            on_dismiss: None,
        }
    }
    /// The title-small headline above the destinations.
    pub fn headline(mut self, headline: impl Into<SharedString>) -> Self {
        self.headline = Some(headline.into());
        self
    }
    /// The destinations. Values must be unique.
    pub fn items(mut self, items: impl IntoIterator<Item = NavItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }
    /// A section label above item `before`; sections after the first are
    /// separated by a middle-inset divider.
    pub fn section(mut self, before: usize, label: impl Into<SharedString>) -> Self {
        self.sections.push((before, label.into()));
        self
    }
    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.selected = Some(value.into());
        self
    }
    /// Float over the content with a scrim instead of sitting beside it.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }
    /// Whether the drawer is shown (true by default).
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    /// 48dp rows instead of 56dp.
    pub fn dense(mut self, dense: bool) -> Self {
        self.dense = dense;
        self
    }
    pub fn position(mut self, position: DrawerPosition) -> Self {
        self.position = position;
        self
    }
    /// The sheet width in design px, clamped to 240–360dp.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.clamp(240., DRAWER_WIDTH);
        self
    }
    /// Show destination icons (true by default).
    pub fn icons(mut self, icons: bool) -> Self {
        self.icons = icons;
        self
    }
    /// Whether presses spread a ripple (on by default).
    pub fn ripple(mut self, ripple: bool) -> Self {
        self.ripple = ripple;
        self
    }
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
    /// A modal drawer's Escape key and scrim click call this; without it
    /// the drawer is not dismissible.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(move |_, window, cx| handler(window, cx)));
        self
    }
}

/// One destination row.
struct DrawerRow {
    id: ElementId,
    item: NavItem,
    selected: bool,
    height: f32,
    icon: bool,
    ripple: bool,
    focus: FocusHandle,
    tab_stop: bool,
    position: (usize, usize),
    on_click: ClickHandler,
}

impl DrawerRow {
    fn render(self, window: &mut Window, cx: &mut App) -> AnyElement {
        let item = self.item;
        let preview = item.preview_state_value();
        let disabled = item.is_disabled_value();
        let enabled = !disabled;
        let selected = self.selected;
        let fade = motion::token_value(
            (self.id.clone(), "drawer-fade"),
            if selected { 1. } else { 0. },
            MotionToken::FastEffects,
            window,
            cx,
        );
        let paint = drawer_paint(selected, fade, disabled);
        #[cfg(test)]
        DRAWER_PAINTED.with(|painted| painted.borrow_mut().insert(item.value().to_string(), paint));
        let focus = self
            .focus
            .tab_stop(self.tab_stop && enabled && preview.is_none());
        let layer = div()
            .id("drawer-state-layer")
            .absolute()
            .inset_0()
            .rounded_full()
            .map(|layer| match preview {
                _ if disabled => layer,
                None => layer
                    .group_hover(ITEM_GROUP, move |style| style.bg(paint.hover))
                    .group_active(ITEM_GROUP, move |style| style.bg(paint.press)),
                Some(InteractionPreview::Hovered) => layer.bg(paint.hover),
                Some(InteractionPreview::Pressed) => layer.bg(paint.press),
                Some(_) => layer,
            });
        let indicator = (paint.indicator.a > 0.001).then(|| {
            div()
                .absolute()
                .inset_0()
                .rounded_full()
                .bg(paint.indicator)
        });
        let badge = match item.badge_value() {
            NavBadge::None => None,
            NavBadge::Small => Some(small_badge().into_any_element()),
            NavBadge::Large(text) => Some(
                div()
                    .flex_shrink_0()
                    .text_color(paint.label)
                    .map(|text| type_role(text, TypeRole::LabelLarge))
                    .child(text.clone())
                    .into_any_element(),
            ),
        };
        let height = rem(self.height);
        let content = div()
            .relative()
            .w_full()
            .h(height)
            .flex()
            .items_center()
            .gap(rem(ITEM_GAP))
            .pl(rem(ITEM_LEADING))
            .pr(rem(ITEM_TRAILING))
            .rounded_full()
            .when(cfg!(test), |content| {
                let value = item.value().clone();
                content.debug_selector(move || format!("drawer-item-{value}"))
            })
            .children(indicator)
            .child(layer)
            .when(self.icon, |content| {
                content.child(
                    Icon::new(item.icon_value().path(selected))
                        .size(rem(24.))
                        .color(paint.icon),
                )
            })
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_w_0()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .text_color(paint.label)
                    .map(|label| type_role(label, TypeRole::LabelLarge))
                    .child(item.label().clone()),
            )
            .children(badge);
        let radius = height.to_pixels(window.rem_size()) / 2.;
        let ripple = ButtonRipple::new(
            content,
            (self.id.clone(), "drawer-ripple").into(),
            preview.is_none() && self.ripple,
            enabled,
            ripple_corners(radius),
            Edges::all(px(0.).into()),
            if selected {
                color(ColorRole::ON_SECONDARY_CONTAINER)
            } else {
                color(ColorRole::ON_SURFACE)
            },
            window,
            cx,
        );
        let ring =
            ButtonFocusRing::with_corners(ripple, focus.clone(), enabled, Corners::all(radius))
                .forced(preview == Some(InteractionPreview::Focused));
        let click_focus = focus.clone();
        let on_click = self.on_click;
        let (index, count) = self.position;
        gpui_base::Tab::new(self.id)
            .group(ITEM_GROUP)
            .selected(selected)
            .disabled(disabled)
            .accessibility_label(item.label().clone())
            .set_position(index + 1, count)
            .track_focus(&focus)
            .w_full()
            .h(height)
            .flex_shrink_0()
            .flex()
            .child(ring)
            .when(enabled && preview.is_none(), |tab| {
                tab.cursor_pointer().on_click(move |event, window, cx| {
                    click_focus.focus(window, cx);
                    on_click(event, window, cx)
                })
            })
            .into_any_element()
    }
}

fn header(id: (&'static str, usize), text: SharedString, height: f32) -> AnyElement {
    div()
        .id(id)
        .role(Role::Heading)
        .aria_label(text.clone())
        .flex_shrink_0()
        .h(rem(height))
        .flex()
        .items_center()
        .px(rem(16.))
        .whitespace_nowrap()
        .text_color(color(ColorRole::ON_SURFACE_VARIANT))
        .map(|text| type_role(text, TypeRole::TitleSmall))
        .child(text)
        .into_any_element()
}

/// The middle-inset divider between sections.
fn divider(index: usize) -> AnyElement {
    div()
        .id(("drawer-divider", index))
        .flex_shrink_0()
        .px(rem(16.))
        .child(div().h(px(1.)).bg(color(ColorRole::OUTLINE_VARIANT)))
        .into_any_element()
}

impl RenderOnce for NavigationDrawer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let open = self.open;
        let amount = motion::token_value(
            (self.id.clone(), "drawer-open"),
            if open { 1. } else { 0. },
            MotionToken::DefaultSpatial,
            window,
            cx,
        );
        let visible = amount.clamp(0., 1.);
        let width = self.width;
        #[cfg(test)]
        DRAWER_OPEN.with(|cell| cell.set((visible, width)));
        let modal = self.modal;
        let end = self.position == DrawerPosition::End;
        // Fully closed: nothing to draw, focus or block the content with.
        if !open && visible < 0.001 {
            return div()
                .id(self.id)
                .when(!modal, |gap| gap.h_full().w(px(0.)))
                .into_any_element();
        }
        let list = destinations(
            &self.id,
            &self.items,
            self.selected.as_ref(),
            Orientation::Column,
            window,
            cx,
        );
        // Opening a modal drawer moves focus into it.
        let was_open =
            window.use_keyed_state((self.id.clone(), "drawer-was-open"), cx, |_, _| false);
        let opened = open && !*was_open.read(cx);
        was_open.update(cx, |was, _| *was = open);
        if modal
            && opened
            && let Some(handle) = list
                .selected
                .or(list.entry)
                .map(|i| list.handles[i].clone())
        {
            window.defer(cx, move |window, cx| handle.focus(window, cx));
        }

        let row_height = if self.dense {
            DENSE_ITEM_HEIGHT
        } else {
            ITEM_HEIGHT
        };
        let count = self.items.len();
        let mut sections = self.sections;
        sections.sort_by_key(|(before, _)| *before);
        let mut sections = sections.into_iter().peekable();
        let mut lane = gpui_base::Tabs::new((self.id.clone(), "drawer-items"))
            .aria_label(self.label.clone())
            .aria_orientation(accesskit::Orientation::Vertical)
            .w_full()
            .flex()
            .flex_col();
        for (index, item) in self.items.into_iter().enumerate() {
            while let Some((_, text)) = sections.next_if(|(before, _)| *before <= index) {
                if index > 0 {
                    lane = lane.child(divider(index));
                }
                lane = lane.child(header(("drawer-section", index), text, row_height));
            }
            let selected = list.selected == Some(index);
            let on_click = item_click(self.on_change.clone(), item.value().clone(), selected);
            lane = lane.child(
                DrawerRow {
                    id: ElementId::NamedChild(self.id.clone().into(), item.value().clone()),
                    item,
                    selected,
                    height: row_height,
                    icon: self.icons,
                    ripple: self.ripple,
                    focus: list.handles[index].clone(),
                    tab_stop: list.entry == Some(index),
                    position: (index, count),
                    on_click,
                }
                .render(window, cx),
            );
        }
        let lane = list.roving.install(lane);
        let on_dismiss = self.on_dismiss.filter(|_| modal);
        let sheet = div()
            .id((self.id.clone(), "drawer-sheet"))
            .when(cfg!(test), |sheet| {
                sheet.debug_selector(|| "navigation-drawer-sheet".into())
            })
            .w(rem(width))
            .h_full()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .p(rem(DRAWER_PADDING))
            .overflow_y_scroll()
            .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
            .when(modal, |sheet| {
                let radius = rem(MODAL_RADIUS);
                sheet
                    .map(|sheet| {
                        if end {
                            sheet.rounded_l(radius)
                        } else {
                            sheet.rounded_r(radius)
                        }
                    })
                    .shadow(elevation::level(1))
            })
            .when_some(self.headline, |sheet, text| {
                sheet.child(header(("drawer-headline", 0), text, row_height))
            })
            .child(lane)
            .when_some(on_dismiss.clone(), |sheet, dismiss| {
                sheet.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape" {
                        cx.stop_propagation();
                        dismiss(&ClickEvent::default(), window, cx);
                    }
                })
            });
        let landmark = |element: Stateful<Div>| {
            element
                .role(Role::Navigation)
                .aria_label(self.label.clone())
        };
        if modal {
            let offset = rem((-(1. - amount) * width).min(0.));
            let scrim = div()
                .id((self.id.clone(), "drawer-scrim"))
                .when(cfg!(test), |scrim| {
                    scrim.debug_selector(|| "navigation-drawer-scrim".into())
                })
                .absolute()
                .inset_0()
                .bg(color(ColorRole::SCRIM).opacity(0.32 * visible))
                .when_some(on_dismiss, |scrim, dismiss| {
                    scrim.on_click(move |event, window, cx| dismiss(event, window, cx))
                });
            landmark(div().id(self.id))
                .absolute()
                .inset_0()
                .overflow_hidden()
                .child(scrim)
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .map(|frame| {
                            if end {
                                frame.right(offset)
                            } else {
                                frame.left(offset)
                            }
                        })
                        .child(sheet),
                )
                .into_any_element()
        } else {
            landmark(div().id(self.id))
                .h_full()
                .w(rem(width * visible))
                .flex_shrink_0()
                .flex()
                .when(end, |gap| gap.justify_end())
                .overflow_hidden()
                .child(sheet)
                .into_any_element()
        }
    }
}

#[cfg(test)]
#[path = "navigation_drawer_tests.rs"]
mod tests;
