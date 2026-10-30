//! M3 Expressive navigation bar and navigation rail.
//!
//! Both are lists of [`NavItem`] destinations painted by one item renderer,
//! [`NavigationItem`], in three layouts:
//!
//! - [`NavItemLayout::Vertical`]: the flexible navigation bar's compact item
//!   and the collapsed rail's item, a 56×32dp active-indicator pill over a
//!   label-medium label.
//! - [`NavItemLayout::Horizontal`]: the flexible navigation bar's medium
//!   item, icon and label side by side inside a 40dp pill.
//! - [`NavItemLayout::Expanded`]: the expanded rail's item, a 56dp pill that
//!   hugs the icon and label-large label.
//!
//! Selecting an item grows its secondary-container pill from its center on
//! the fast spatial spring while icon and label colors cross-fade on the fast
//! effects spring; the icon switches to its filled form (Material Symbols
//! FILL 1). The hover/press state layer covers the pill wherever the pointer
//! is on the item, and a ripple spreads inside the pill.
//!
//! Behavior is GPUI Base's: each item is a Base [`gpui_base::Tab`] (tab role,
//! selection, position in set, pointer and Enter/Space activation) inside a
//! Base [`gpui_base::Tabs`] tab list, wrapped in a navigation landmark. Arrow
//! keys rove focus between items through the shared [`super::roving`]
//! helper; Enter or Space activates.
use super::button::{ButtonFocusRing, ButtonRipple};
use super::roving::{self, Change, Orientation, Roving};
use super::{InteractionPreview, large_badge, small_badge};
use crate::color::ColorRole;
use crate::icons::{Icon, Style, Symbol};
use crate::motion::{self, MotionToken};
use crate::theme::{blend, color};
use crate::tokens::{
    elevation,
    shape::RADIUS_LARGE,
    state,
    typography::{TypeRole, rem},
};
use gpui::{prelude::FluentBuilder as _, *};
use std::{collections::BTreeMap, rc::Rc};

/// The hover group every item's state layer follows. Items are siblings, so
/// one static name resolves to the enclosing item (GPUI keeps a stack per
/// group name).
const ITEM_GROUP: &str = "material-navigation-item";

/// An item's badge, drawn on its icon's top-trailing corner.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum NavBadge {
    #[default]
    None,
    /// The 6dp error dot.
    Small,
    /// A count or short label on the large badge, e.g. "3" or "999+".
    Large(SharedString),
}

/// A destination icon in its unselected (outlined) and selected (filled)
/// forms. Build it once and keep it: resolving symbol paths allocates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavIcon {
    unselected: SharedString,
    selected: SharedString,
}

impl NavIcon {
    /// Two asset paths, e.g. two separate symbols.
    pub fn new(unselected: impl Into<SharedString>, selected: impl Into<SharedString>) -> Self {
        Self {
            unselected: unselected.into(),
            selected: selected.into(),
        }
    }

    /// One Material Symbol, rounded: FILL 0 at rest and FILL 1 selected, the
    /// M3 navigation icon change.
    pub fn symbol(symbol: Symbol) -> Self {
        let icon = symbol.icon().style(Style::Rounded);
        Self::new(icon.asset_path(), icon.filled(true).asset_path())
    }

    pub(crate) fn path(&self, selected: bool) -> SharedString {
        if selected {
            self.selected.clone()
        } else {
            self.unselected.clone()
        }
    }
}

/// One destination: a unique value, the caller's label and icon, and an
/// optional badge. Cheap to clone; keep a list in view state.
#[derive(Clone, Debug)]
pub struct NavItem {
    value: SharedString,
    label: SharedString,
    icon: NavIcon,
    badge: NavBadge,
    disabled: bool,
    preview: Option<InteractionPreview>,
}

impl NavItem {
    pub fn new(
        value: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: NavIcon,
    ) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon,
            badge: NavBadge::None,
            disabled: false,
            preview: None,
        }
    }
    pub fn badge(mut self, badge: NavBadge) -> Self {
        self.badge = badge;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Render this item as a static specimen in one forced interaction state
    /// (state layer, focus ring or disabled colors). A bar or rail holding a
    /// previewed item is a specimen and leaves the tab order. See
    /// [`InteractionPreview`].
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
    pub fn value(&self) -> &SharedString {
        &self.value
    }
    pub fn label(&self) -> &SharedString {
        &self.label
    }
    fn is_disabled(&self) -> bool {
        self.disabled || self.preview.is_some_and(InteractionPreview::is_disabled)
    }
}

/// Which M3 item an item renders as.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavItemLayout {
    /// Icon over label: the navigation bar's compact item and the collapsed
    /// rail's item.
    #[default]
    Vertical,
    /// Icon beside label in a 40dp pill: the medium navigation bar's item.
    Horizontal,
    /// Icon beside label in a 56dp pill: the expanded rail's item.
    Expanded,
}

/// Item geometry in design px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Geometry {
    pub vertical: bool,
    pub item_height: f32,
    pub pill_height: f32,
    pub pill_min_width: f32,
    pub pill_padding: f32,
    /// Between icon and label (inside the pill when horizontal).
    pub gap: f32,
    /// Above the pill of a vertical item.
    pub top: f32,
    /// Leading inset of an expanded rail item.
    pub inset: f32,
    pub label: TypeRole,
}

impl NavItemLayout {
    pub(crate) const fn geometry(self) -> Geometry {
        match self {
            Self::Vertical => Geometry {
                vertical: true,
                item_height: 64.,
                pill_height: 32.,
                pill_min_width: 56.,
                pill_padding: 0.,
                gap: 4.,
                top: 6.,
                inset: 0.,
                label: TypeRole::LabelMedium,
            },
            Self::Horizontal => Geometry {
                vertical: false,
                item_height: 64.,
                pill_height: 40.,
                pill_min_width: 0.,
                pill_padding: 16.,
                gap: 4.,
                top: 0.,
                inset: 0.,
                label: TypeRole::LabelMedium,
            },
            Self::Expanded => Geometry {
                vertical: false,
                item_height: 56.,
                pill_height: 56.,
                pill_min_width: 0.,
                pill_padding: 16.,
                gap: 8.,
                top: 0.,
                inset: 20.,
                label: TypeRole::LabelLarge,
            },
        }
    }
}

/// The resolved paint of one item for the current motion frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ItemPaint {
    pub icon: Rgba,
    pub label: Rgba,
    pub indicator: Rgba,
    /// The pill's horizontal scale, 0 hidden to 1 full (may overshoot).
    pub indicator_scale: f32,
    pub hover: Rgba,
    pub press: Rgba,
}

/// `selection` is the fast spatial amount (indicator size) and `fade` the
/// fast effects amount (colors), each 0 unselected and 1 selected.
pub(crate) fn item_paint(selected: bool, selection: f32, fade: f32, disabled: bool) -> ItemPaint {
    let fade = fade.clamp(0., 1.);
    let rest = color(ColorRole::ON_SURFACE_VARIANT);
    if disabled {
        let ink = state::disabled(color(ColorRole::ON_SURFACE));
        return ItemPaint {
            icon: ink,
            label: ink,
            indicator: color(ColorRole::ON_SURFACE).opacity(state::DISABLED_CONTAINER * fade),
            indicator_scale: selection.max(0.),
            hover: Rgba::default(),
            press: Rgba::default(),
        };
    }
    let layer = if selected {
        color(ColorRole::ON_SECONDARY_CONTAINER)
    } else {
        rest
    };
    ItemPaint {
        icon: blend(rest, color(ColorRole::ON_SECONDARY_CONTAINER), fade),
        label: blend(rest, color(ColorRole::SECONDARY), fade),
        indicator: color(ColorRole::SECONDARY_CONTAINER).opacity(fade),
        indicator_scale: selection.max(0.),
        hover: layer.opacity(state::HOVER),
        press: layer.opacity(state::PRESS),
    }
}

pub(crate) fn type_role<E: Styled>(element: E, role: TypeRole) -> E {
    let scale = role.scale();
    element
        .font_family(scale.font_family)
        .text_size(scale.font_size)
        .line_height(scale.line_height)
        .font_weight(FontWeight(scale.font_weight as f32))
}

/// Uniform ripple clip corners.
pub(crate) fn ripple_corners(radius: Pixels) -> Corners<AbsoluteLength> {
    let radius = AbsoluteLength::from(radius);
    Corners {
        top_left: radius,
        top_right: radius,
        bottom_right: radius,
        bottom_left: radius,
    }
}

/// A 24dp icon with the item's badge on its top-trailing corner.
pub(crate) fn badged_icon(icon: Icon, badge: &NavBadge) -> Div {
    let slot = div()
        .relative()
        .flex_shrink_0()
        .size(rem(24.))
        .child(icon.size(rem(24.)));
    match badge {
        NavBadge::None => slot,
        NavBadge::Small => slot.child(small_badge().absolute().top_0().right_0()),
        NavBadge::Large(text) => slot.child(
            large_badge("badge", text.clone())
                .absolute()
                .top(rem(-2.))
                .left(rem(12.)),
        ),
    }
}

#[cfg(test)]
thread_local! {
    pub(crate) static PAINTED: std::cell::RefCell<BTreeMap<String, ItemPaint>> =
        const { std::cell::RefCell::new(BTreeMap::new()) };
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// One navigation destination as an M3 item: active-indicator pill, filled
/// icon when selected, badge and label. Bars and rails compose these; a
/// gallery may show one alone as a building block.
#[derive(IntoElement)]
pub struct NavigationItem {
    id: ElementId,
    item: NavItem,
    layout: NavItemLayout,
    selected: bool,
    show_label: bool,
    focus: Option<FocusHandle>,
    tab_stop: bool,
    position: Option<(usize, usize)>,
    fill: bool,
    on_click: Option<ClickHandler>,
}

impl NavigationItem {
    pub fn new(id: impl Into<ElementId>, item: NavItem) -> Self {
        Self {
            id: id.into(),
            item,
            layout: NavItemLayout::Vertical,
            selected: false,
            show_label: true,
            focus: None,
            tab_stop: true,
            position: None,
            fill: false,
            on_click: None,
        }
    }
    pub fn layout(mut self, layout: NavItemLayout) -> Self {
        self.layout = layout;
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    /// Hide the label, leaving the icon (its label stays the accessible name).
    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
    fn roving(mut self, focus: FocusHandle, tab_stop: bool, position: (usize, usize)) -> Self {
        self.focus = Some(focus);
        self.tab_stop = tab_stop;
        self.position = Some(position);
        self
    }
    fn fill(mut self) -> Self {
        self.fill = true;
        self
    }
}

impl RenderOnce for NavigationItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let geometry = self.layout.geometry();
        let item = self.item;
        let preview = item.preview;
        let disabled = item.is_disabled();
        let enabled = !disabled;
        let selected = self.selected;
        let target = if selected { 1. } else { 0. };
        let selection =
            motion::selection_value((self.id.clone(), "nav-indicator"), target, window, cx);
        let fade = motion::token_value(
            (self.id.clone(), "nav-fade"),
            target,
            MotionToken::FastEffects,
            window,
            cx,
        );
        let paint = item_paint(selected, selection, fade, disabled);
        #[cfg(test)]
        PAINTED.with(|painted| painted.borrow_mut().insert(format!("{:?}", self.id), paint));
        let focus = self.focus.unwrap_or_else(|| {
            window
                .use_keyed_state((self.id.clone(), "nav-focus"), cx, |_, cx| {
                    cx.focus_handle()
                })
                .read(cx)
                .clone()
        });
        let focus = focus.tab_stop(self.tab_stop && enabled && preview.is_none());

        let label = |id: &'static str| {
            div()
                .id(id)
                .flex_shrink_0()
                .whitespace_nowrap()
                .text_color(paint.label)
                .map(|label| type_role(label, geometry.label))
                .child(item.label.clone())
        };
        let scale = paint.indicator_scale;
        let inset = relative((1. - scale) / 2.);
        let indicator = (scale > 0.001 && paint.indicator.a > 0.001).then(|| {
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .left(inset)
                .right(inset)
                .rounded_full()
                .bg(paint.indicator)
        });
        let layer = div()
            .id("nav-state-layer")
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
        // The expanded rail puts a large badge after the label instead of on
        // the icon.
        let trailing = match (&item.badge, self.layout) {
            (NavBadge::Large(text), NavItemLayout::Expanded) => Some(text.clone()),
            _ => None,
        };
        let icon = badged_icon(
            Icon::new(item.icon.path(selected)).color(paint.icon),
            if trailing.is_some() {
                &NavBadge::None
            } else {
                &item.badge
            },
        );
        let pill_height = rem(geometry.pill_height);
        let pill = div()
            .relative()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .h(pill_height)
            .min_w(rem(geometry.pill_min_width))
            .px(rem(geometry.pill_padding))
            .gap(rem(geometry.gap))
            .rounded_full()
            .when(cfg!(test), |pill| {
                let value = item.value.clone();
                pill.debug_selector(move || format!("nav-pill-{value}"))
            })
            .children(indicator)
            .child(layer)
            .child(icon)
            .when(!geometry.vertical && self.show_label, |pill| {
                pill.child(label("nav-label"))
            })
            .when_some(trailing, |pill, text| {
                pill.child(large_badge("badge", text))
            });
        let radius = pill_height.to_pixels(window.rem_size()) / 2.;
        let ink = if selected {
            color(ColorRole::ON_SECONDARY_CONTAINER)
        } else {
            color(ColorRole::ON_SURFACE_VARIANT)
        };
        let ripple = ButtonRipple::new(
            pill,
            (self.id.clone(), "nav-ripple").into(),
            preview.is_none(),
            enabled,
            ripple_corners(radius),
            Edges::all(px(0.).into()),
            ink,
            window,
            cx,
        );
        let ring =
            ButtonFocusRing::with_corners(ripple, focus.clone(), enabled, Corners::all(radius))
                .forced(preview == Some(InteractionPreview::Focused));

        let click_focus = focus.clone();
        let on_click = self.on_click.filter(|_| enabled && preview.is_none());
        let vertical_top = if self.show_label {
            geometry.top
        } else {
            (geometry.item_height - geometry.pill_height) / 2.
        };
        gpui_base::Tab::new(self.id)
            .group(ITEM_GROUP)
            .selected(selected)
            .disabled(disabled)
            .accessibility_label(item.label.clone())
            .when_some(self.position, |tab, (index, count)| {
                tab.set_position(index + 1, count)
            })
            .track_focus(&focus)
            .flex_shrink_0()
            .h(rem(geometry.item_height))
            .when(self.fill, |tab| tab.flex_1().min_w_0())
            .map(|tab| match self.layout {
                NavItemLayout::Vertical => tab
                    .flex_col()
                    .justify_start()
                    .items_center()
                    .pt(rem(vertical_top))
                    .gap(rem(geometry.gap)),
                NavItemLayout::Horizontal => tab.flex_row().items_center().justify_center(),
                NavItemLayout::Expanded => tab
                    .w_full()
                    .flex_row()
                    .items_center()
                    .justify_start()
                    .px(rem(geometry.inset)),
            })
            .when(on_click.is_some(), |tab| tab.cursor_pointer())
            .child(ring)
            .when(geometry.vertical && self.show_label, |tab| {
                tab.child(label("nav-label"))
            })
            .when_some(on_click, |tab, handler| {
                tab.on_click(move |event, window, cx| {
                    click_focus.focus(window, cx);
                    handler(event, window, cx)
                })
            })
    }
}

/// Shared list behavior of bars and rails.
struct Destinations {
    handles: Rc<[FocusHandle]>,
    selected: Option<usize>,
    entry: Option<usize>,
    roving: Roving,
}

fn destinations(
    id: &ElementId,
    items: &[NavItem],
    selected: Option<&SharedString>,
    orientation: Orientation,
    window: &mut Window,
    cx: &mut App,
) -> Destinations {
    let state = window.use_keyed_state((id.clone(), "nav-focus-handles"), cx, |_, _| {
        BTreeMap::<SharedString, FocusHandle>::new()
    });
    let handles: Rc<[FocusHandle]> = state
        .update(cx, |handles, cx| {
            roving::sync_handles(handles, items, |item| &item.value, || cx.focus_handle())
        })
        .into();
    let selected = items.iter().position(|item| Some(&item.value) == selected);
    let specimen = items.iter().any(|item| item.preview.is_some());
    let skip: Rc<[bool]> = items.iter().map(NavItem::is_disabled).collect();
    // Arrows only move focus; Enter or Space activates.
    let roving = Roving::new(handles.clone(), orientation)
        .skip(skip)
        .selected(selected);
    let entry = if specimen {
        None
    } else {
        roving.tab_stop(window)
    };
    Destinations {
        handles,
        selected,
        entry,
        roving,
    }
}

fn item_click(on_change: Option<Change>, value: SharedString, selected: bool) -> ClickHandler {
    Rc::new(move |_, window, cx| {
        if !selected && let Some(on_change) = &on_change {
            on_change(&value, window, cx);
        }
    })
}

/// M3 Expressive flexible navigation bar: 3–5 primary destinations along
/// the bottom of a compact or medium window, on surface-container.
#[derive(IntoElement)]
pub struct NavigationBar {
    id: ElementId,
    label: SharedString,
    items: Vec<NavItem>,
    selected: Option<SharedString>,
    layout: NavItemLayout,
    show_labels: bool,
    on_change: Option<Change>,
}

impl NavigationBar {
    /// `label` is the navigation landmark's accessible name.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            items: Vec::new(),
            selected: None,
            layout: NavItemLayout::Vertical,
            show_labels: true,
            on_change: None,
        }
    }
    /// The destinations. Values must be unique.
    pub fn items(mut self, items: impl IntoIterator<Item = NavItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }
    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.selected = Some(value.into());
        self
    }
    /// Horizontal items (icon beside label) for medium windows; vertical
    /// (the default) for compact ones. `Expanded` is a rail layout and
    /// renders as horizontal here.
    pub fn layout(mut self, layout: NavItemLayout) -> Self {
        self.layout = match layout {
            NavItemLayout::Expanded => NavItemLayout::Horizontal,
            other => other,
        };
        self
    }
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

/// The widest share a horizontal bar item takes, so a few items gather in
/// the middle of a wide bar instead of spreading to its edges.
const HORIZONTAL_ITEM_MAX: f32 = 128.;

impl RenderOnce for NavigationBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let list = destinations(
            &self.id,
            &self.items,
            self.selected.as_ref(),
            Orientation::Row,
            window,
            cx,
        );
        let count = self.items.len();
        let geometry = self.layout.geometry();
        let horizontal = self.layout == NavItemLayout::Horizontal;
        let lane = gpui_base::Tabs::new((self.id.clone(), "nav-items"))
            .aria_label(self.label.clone())
            .flex()
            .flex_row()
            .w_full()
            .h(rem(geometry.item_height))
            .when(horizontal, |lane| {
                lane.max_w(rem(HORIZONTAL_ITEM_MAX * count as f32))
            })
            .children(self.items.into_iter().enumerate().map(|(index, item)| {
                let selected = list.selected == Some(index);
                let click = item_click(self.on_change.clone(), item.value.clone(), selected);
                NavigationItem::new(
                    ElementId::NamedChild(self.id.clone().into(), item.value.clone()),
                    item,
                )
                .layout(self.layout)
                .selected(selected)
                .show_label(self.show_labels)
                .fill()
                .roving(
                    list.handles[index].clone(),
                    list.entry == Some(index),
                    (index, count),
                )
                .on_click(move |event, window, cx| click(event, window, cx))
            }));
        let lane = list.roving.install(lane);
        div()
            .id(self.id)
            .role(Role::Navigation)
            .aria_label(self.label)
            .w_full()
            .flex()
            .justify_center()
            .bg(color(ColorRole::SURFACE_CONTAINER))
            .child(lane)
    }
}

/// Where a rail's destinations sit in its height.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RailAlignment {
    #[default]
    Top,
    Middle,
}

/// The collapsed rail's width.
pub const RAIL_COLLAPSED_WIDTH: f32 = 96.;
/// The expanded rail's default width (M3: 220–360dp).
pub const RAIL_EXPANDED_WIDTH: f32 = 220.;

/// M3 Expressive navigation rail: a 96dp column of vertical items that
/// expands to a 220–360dp column of horizontal items, with an optional menu
/// button and FAB above the destinations and, when expanded, section
/// headers. The width moves on the default spatial spring.
///
/// A modal expanded rail floats on surface-container at level 2 with large
/// trailing corners over a scrim ([`navigation_scrim`]); Escape dismisses it.
#[derive(IntoElement)]
pub struct NavigationRail {
    id: ElementId,
    label: SharedString,
    items: Vec<NavItem>,
    sections: Vec<(usize, SharedString)>,
    selected: Option<SharedString>,
    expanded: bool,
    modal: bool,
    alignment: RailAlignment,
    expanded_width: f32,
    menu: Option<AnyElement>,
    fab: Option<AnyElement>,
    on_change: Option<Change>,
    on_dismiss: Option<ClickHandler>,
}

impl NavigationRail {
    /// `label` is the navigation landmark's accessible name.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            items: Vec::new(),
            sections: Vec::new(),
            selected: None,
            expanded: false,
            modal: false,
            alignment: RailAlignment::Top,
            expanded_width: RAIL_EXPANDED_WIDTH,
            menu: None,
            fab: None,
            on_change: None,
            on_dismiss: None,
        }
    }
    /// The destinations. Values must be unique.
    pub fn items(mut self, items: impl IntoIterator<Item = NavItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }
    /// A section header shown above item `before` while expanded.
    pub fn section(mut self, before: usize, header: impl Into<SharedString>) -> Self {
        self.sections.push((before, header.into()));
        self
    }
    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.selected = Some(value.into());
        self
    }
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
    /// Paint the floating modal container (meaningful while expanded).
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }
    pub fn alignment(mut self, alignment: RailAlignment) -> Self {
        self.alignment = alignment;
        self
    }
    /// The expanded width in design px, clamped to M3's 220–360dp.
    pub fn expanded_width(mut self, width: f32) -> Self {
        self.expanded_width = width.clamp(220., 360.);
        self
    }
    /// The menu button at the top (the caller's icon button that toggles
    /// `expanded`).
    pub fn menu(mut self, menu: impl IntoElement) -> Self {
        self.menu = Some(menu.into_any_element());
        self
    }
    /// The FAB under the menu: a FAB when collapsed, an extended FAB when
    /// expanded (the caller supplies whichever matches `expanded`).
    pub fn fab(mut self, fab: impl IntoElement) -> Self {
        self.fab = Some(fab.into_any_element());
        self
    }
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
    /// Escape inside a modal rail calls this (the scrim's click is the
    /// caller's, see [`navigation_scrim`]).
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(move |_, window, cx| handler(window, cx)));
        self
    }
}

/// Top padding above a rail's menu button.
const RAIL_TOP: f32 = 44.;
/// Space between the menu/FAB block and the destinations.
const RAIL_HEADER_GAP: f32 = 40.;
const RAIL_ITEM_GAP: f32 = 4.;
const SECTION_HEIGHT: f32 = 36.;

impl RenderOnce for NavigationRail {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let expanded = self.expanded;
        let width = motion::token_value(
            (self.id.clone(), "rail-width"),
            if expanded {
                self.expanded_width
            } else {
                RAIL_COLLAPSED_WIDTH
            },
            MotionToken::DefaultSpatial,
            window,
            cx,
        );
        let list = destinations(
            &self.id,
            &self.items,
            self.selected.as_ref(),
            Orientation::Column,
            window,
            cx,
        );
        let layout = if expanded {
            NavItemLayout::Expanded
        } else {
            NavItemLayout::Vertical
        };
        let count = self.items.len();
        let modal = self.modal && expanded;
        let mut sections = self.sections;
        sections.sort_by_key(|(before, _)| *before);
        let mut sections = sections.into_iter().peekable();
        let mut lane = gpui_base::Tabs::new((self.id.clone(), "rail-items"))
            .aria_label(self.label.clone())
            .aria_orientation(accesskit::Orientation::Vertical)
            .w_full()
            .flex()
            .flex_col()
            .when(!expanded, |lane| {
                lane.items_center().gap(rem(RAIL_ITEM_GAP))
            });
        for (index, item) in self.items.into_iter().enumerate() {
            while let Some((_, header)) = sections.next_if(|(before, _)| *before <= index) {
                if expanded {
                    lane = lane.child(section_header(index, header));
                }
            }
            let selected = list.selected == Some(index);
            let click = item_click(self.on_change.clone(), item.value.clone(), selected);
            lane = lane.child(
                NavigationItem::new(
                    ElementId::NamedChild(self.id.clone().into(), item.value.clone()),
                    item,
                )
                .layout(layout)
                .selected(selected)
                .roving(
                    list.handles[index].clone(),
                    list.entry == Some(index),
                    (index, count),
                )
                .on_click(move |event, window, cx| click(event, window, cx)),
            );
        }
        if expanded {
            for (index, header) in sections {
                lane = lane.child(section_header(index, header));
            }
        }
        let lane = list.roving.install(lane);
        // The entering layout fades in while the width moves.
        let lane = motion::enter(
            (
                self.id.clone(),
                if expanded {
                    "rail-expanded"
                } else {
                    "rail-collapsed"
                },
            ),
            lane,
        );
        let has_header = self.menu.is_some() || self.fab.is_some();
        let header = div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .gap(rem(RAIL_ITEM_GAP))
            .map(|header| {
                if expanded {
                    header.items_start().pl(rem(20.))
                } else {
                    header.w_full().items_center()
                }
            })
            .children(self.menu)
            .children(self.fab);
        let on_dismiss = self.on_dismiss.filter(|_| modal);
        div()
            .id(self.id)
            .role(Role::Navigation)
            .aria_label(self.label)
            .relative()
            .h_full()
            .w(rem(width))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .overflow_hidden()
            .pt(rem(RAIL_TOP))
            .pb(rem(20.))
            .map(|rail| {
                if modal {
                    rail.bg(color(ColorRole::SURFACE_CONTAINER))
                        .rounded_r(RADIUS_LARGE)
                        .shadow(elevation::level(2))
                } else {
                    rail.bg(color(ColorRole::SURFACE))
                }
            })
            .when(has_header, |rail| {
                rail.child(header)
                    .child(div().h(rem(RAIL_HEADER_GAP)).flex_shrink_0())
            })
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .flex()
                    .flex_col()
                    .when(self.alignment == RailAlignment::Middle, |column| {
                        column.justify_center()
                    })
                    .child(lane),
            )
            .when_some(on_dismiss, |rail, dismiss| {
                rail.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape" {
                        cx.stop_propagation();
                        dismiss(&ClickEvent::default(), window, cx);
                    }
                })
            })
    }
}

fn section_header(index: usize, header: SharedString) -> impl IntoElement {
    div()
        .id(("rail-section", index))
        .role(Role::Heading)
        .aria_label(header.clone())
        .flex_shrink_0()
        .h(rem(SECTION_HEIGHT))
        .flex()
        .items_center()
        .pl(rem(36.))
        .pr(rem(20.))
        .whitespace_nowrap()
        .text_color(color(ColorRole::ON_SURFACE_VARIANT))
        .map(|text| type_role(text, TypeRole::TitleSmall))
        .child(header)
}

/// The scrim behind a modal expanded rail: scrim at 32%, fading in. Its
/// click dismisses the rail.
pub fn navigation_scrim(
    id: impl Into<ElementId>,
    on_dismiss: impl Fn(&mut Window, &mut App) + 'static,
) -> impl IntoElement {
    motion::enter(
        id,
        div()
            .id("navigation-scrim")
            .absolute()
            .inset_0()
            .bg(color(ColorRole::SCRIM).opacity(0.32))
            .on_click(move |_, window, cx| on_dismiss(window, cx)),
    )
}

#[cfg(test)]
#[path = "navigation_tests.rs"]
mod tests;
