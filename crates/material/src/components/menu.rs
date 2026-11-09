//! M3 menus: the baseline menu, the M3 Expressive vertical menu (standard and
//! vibrant, with groups), a popup that opens a menu from a trigger, a context
//! menu that opens one at the pointer, and the exposed dropdown menu built from
//! a text field and a menu.
//!
//! Reference: <https://m3.material.io/components/menus/specs> and the
//! Material 3 Design Kit "Menu" page (inspected tokens: Expressive items are
//! 44dp inside 2/4dp insets with 12dp padding, 20dp icons and label-large
//! text; selected items take tertiary roles and a 12dp shape; groups have
//! 8dp corners, a 2dp gap and the menu's 16dp outer corners; baseline items
//! are 56/48/40dp on surface-container with secondary-container selection).
//!
//! Behavior comes from Kit and Base: items are Base buttons (pointer, Enter
//! and Space activation, disabled semantics) with the shared M3 focus ring;
//! arrow keys, Home/End and Escape use Base's shared actions; the popup is
//! Kit's `component::popover::Popover`, which owns the overlay, outside-click
//! and Escape dismissal, and focus capture and restoration. Material owns the
//! visuals and the open/close motion: the surface reveals from its top on the
//! default spatial token while fading on fast effects, and plays the same
//! motion back on close, retargeting from wherever it is.
//!
//! A menu taller than [`MaterialMenu::max_height`] scrolls, and a popup never
//! outgrows its window. A popup with no room on its side of the trigger opens
//! on the other side, or scrolls on the side with more room, rather than
//! cover its trigger; only a trigger that leaves less than a row on both sides
//! is covered. The keyboard's item stays in view and an open menu starts at
//! its selected item.
use super::InteractionPreview;
use super::focus::{RingPlacement, RingedButton};
use super::roving::{Direction, target};
use super::text::type_style;
#[cfg(feature = "inputs")]
use super::text_field::{TextField, TextFieldDensity, TextFieldShape, TextFieldVariant};
use crate::{
    color::ColorRole,
    icons::{Icon, Symbol},
    motion::{self, MotionToken},
    theme::color,
    tokens::{
        elevation, state,
        typography::{BASE_PX, TypeRole, rem},
    },
};
use gpui::{prelude::*, *};
use gpui_base::actions::{
    Cancel, Confirm, SelectDown, SelectFirst, SelectLast, SelectLeft, SelectRight, SelectUp,
};
use gpui_kit::component::popover::Popover;
use std::rc::Rc;

const MENU_CONTEXT: &str = "MaterialMenu";
const DROPDOWN_CONTEXT: &str = "MaterialDropdown";

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", SelectUp, Some(MENU_CONTEXT)),
        KeyBinding::new("down", SelectDown, Some(MENU_CONTEXT)),
        KeyBinding::new("home", SelectFirst, Some(MENU_CONTEXT)),
        KeyBinding::new("end", SelectLast, Some(MENU_CONTEXT)),
        KeyBinding::new("right", SelectRight, Some(MENU_CONTEXT)),
        KeyBinding::new("left", SelectLeft, Some(MENU_CONTEXT)),
        KeyBinding::new("escape", Cancel, Some(MENU_CONTEXT)),
        KeyBinding::new("enter", Confirm { secondary: false }, Some(MENU_CONTEXT)),
        KeyBinding::new("space", Confirm { secondary: false }, Some(MENU_CONTEXT)),
        KeyBinding::new("down", SelectDown, Some(DROPDOWN_CONTEXT)),
        KeyBinding::new("up", SelectUp, Some(DROPDOWN_CONTEXT)),
        KeyBinding::new(
            "enter",
            Confirm { secondary: false },
            Some(DROPDOWN_CONTEXT),
        ),
        KeyBinding::new(
            "space",
            Confirm { secondary: false },
            Some(DROPDOWN_CONTEXT),
        ),
    ]);
}

/// The menu families of the design kit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MenuStyle {
    /// The original M3 menu on surface-container.
    Baseline,
    /// M3 Expressive vertical menu on surface-container-low.
    #[default]
    Standard,
    /// M3 Expressive vertical menu on tertiary-container.
    Vibrant,
}

impl MenuStyle {
    pub const ALL: [Self; 3] = [Self::Baseline, Self::Standard, Self::Vibrant];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Baseline => "Baseline",
            Self::Standard => "Standard",
            Self::Vibrant => "Vibrant",
        }
    }
    const fn expressive(self) -> bool {
        !matches!(self, Self::Baseline)
    }
}

/// Baseline menu density: 56, 48 or 40dp items.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MenuDensity {
    #[default]
    Default,
    /// Density −2.
    Compact,
    /// Density −4.
    Dense,
}

impl MenuDensity {
    pub const ALL: [Self; 3] = [Self::Default, Self::Compact, Self::Dense];
    const fn item_height(self) -> f32 {
        match self {
            Self::Default => 56.,
            Self::Compact => 48.,
            Self::Dense => 40.,
        }
    }
    /// How much shorter an Expressive item is. The kit defines densities
    /// for the baseline menu only (8dp per two steps); md3.io offers a
    /// dense Expressive menu, so Expressive items take half those steps,
    /// keeping a 44dp item at least 36dp around its 20dp line.
    const fn expressive_reduction(self) -> f32 {
        match self {
            Self::Default => 0.,
            Self::Compact => 4.,
            Self::Dense => 8.,
        }
    }
}

type Handler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type Notify = Rc<dyn Fn(&mut Window, &mut App)>;
type Select = Rc<dyn Fn(&usize, &mut Window, &mut App)>;
type OpenChange = Rc<dyn Fn(&bool, &mut Window, &mut App)>;
type OnClosed = Box<dyn FnOnce(&mut App)>;
type MenuBuilder = Box<dyn Fn(&mut Window, &mut App) -> MaterialMenu>;

/// One menu item.
pub struct MaterialMenuItem {
    label: SharedString,
    supporting: Option<SharedString>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    trailing_text: Option<SharedString>,
    submenu: bool,
    selected: bool,
    disabled: bool,
    divider: bool,
    preview: Option<InteractionPreview>,
    on_click: Option<Handler>,
}

impl MaterialMenuItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            supporting: None,
            leading: None,
            trailing: None,
            trailing_text: None,
            submenu: false,
            selected: false,
            disabled: false,
            divider: false,
            preview: None,
            on_click: None,
        }
    }
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }
    /// A leading icon (20dp Expressive, 24dp baseline).
    pub fn leading(mut self, icon: impl IntoElement) -> Self {
        self.leading = Some(icon.into_any_element());
        self
    }
    /// A trailing icon.
    pub fn trailing(mut self, icon: impl IntoElement) -> Self {
        self.trailing = Some(icon.into_any_element());
        self
    }
    /// Trailing text such as a keyboard shortcut (supplied by the caller).
    pub fn trailing_text(mut self, text: impl Into<SharedString>) -> Self {
        self.trailing_text = Some(text.into());
        self
    }
    /// Show the sub-menu indicator; the item's activation opens the sub-menu.
    pub fn submenu(mut self, submenu: bool) -> Self {
        self.submenu = submenu;
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
    /// Draw a divider below this item.
    pub fn divider(mut self, divider: bool) -> Self {
        self.divider = divider;
        self
    }
    /// Render a static specimen in one forced interaction state.
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        if state.is_disabled() {
            self.disabled = true;
        }
        self
    }
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

/// A run of items; Expressive menus separate groups with a 2dp gap.
pub struct MenuGroup {
    label: Option<SharedString>,
    items: Vec<MaterialMenuItem>,
}

impl MenuGroup {
    pub fn new() -> Self {
        Self {
            label: None,
            items: Vec::new(),
        }
    }
    /// A section label above the group's items.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn item(mut self, item: MaterialMenuItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = MaterialMenuItem>) -> Self {
        self.items.extend(items);
        self
    }
}

impl Default for MenuGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// The roles one item paints.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ItemPaint {
    pub container: Option<Rgba>,
    pub label: Rgba,
    pub supporting: Rgba,
    pub icon: Rgba,
    pub trailing: Rgba,
    pub layer: Rgba,
}

fn alpha(mut c: Rgba, a: f32) -> Rgba {
    c.a *= a;
    c
}

/// Item colors for a style and selection (disabled content fades by 38%).
pub(crate) fn item_paint(style: MenuStyle, selected: bool) -> ItemPaint {
    use ColorRole as C;
    match (style, selected) {
        (MenuStyle::Baseline, false) | (MenuStyle::Standard, false) => ItemPaint {
            container: None,
            label: color(C::ON_SURFACE),
            supporting: color(C::ON_SURFACE_VARIANT),
            icon: color(C::ON_SURFACE_VARIANT),
            trailing: color(C::ON_SURFACE_VARIANT),
            layer: color(C::ON_SURFACE),
        },
        (MenuStyle::Baseline, true) => ItemPaint {
            container: Some(color(C::SECONDARY_CONTAINER)),
            label: color(C::ON_SECONDARY_CONTAINER),
            supporting: color(C::ON_SECONDARY_CONTAINER),
            icon: color(C::ON_SURFACE_VARIANT),
            trailing: color(C::ON_SECONDARY_CONTAINER),
            layer: color(C::ON_SECONDARY_CONTAINER),
        },
        (MenuStyle::Standard, true) => ItemPaint {
            container: Some(color(C::TERTIARY_CONTAINER)),
            label: color(C::ON_TERTIARY_CONTAINER),
            supporting: color(C::TERTIARY),
            icon: color(C::ON_TERTIARY_CONTAINER),
            trailing: color(C::ON_TERTIARY_CONTAINER),
            layer: color(C::ON_TERTIARY_CONTAINER),
        },
        (MenuStyle::Vibrant, false) => {
            let ink = color(C::ON_TERTIARY_CONTAINER);
            ItemPaint {
                container: None,
                label: ink,
                supporting: ink,
                icon: ink,
                trailing: ink,
                layer: ink,
            }
        }
        (MenuStyle::Vibrant, true) => {
            let ink = color(C::ON_TERTIARY);
            ItemPaint {
                container: Some(color(C::TERTIARY)),
                label: ink,
                supporting: ink,
                icon: ink,
                trailing: ink,
                layer: ink,
            }
        }
    }
}

/// The menu container's color.
pub(crate) fn container_role(style: MenuStyle) -> ColorRole {
    match style {
        MenuStyle::Baseline => ColorRole::SURFACE_CONTAINER,
        MenuStyle::Standard => ColorRole::SURFACE_CONTAINER_LOW,
        MenuStyle::Vibrant => ColorRole::TERTIARY_CONTAINER,
    }
}

const EXPRESSIVE_ITEM: f32 = 44.;
const EXPRESSIVE_INSET_Y: f32 = 2.;
const EXPRESSIVE_INSET_X: f32 = 4.;
/// `divider_line`: a 1dp line with 2dp above and below.
const EXPRESSIVE_DIVIDER: f32 = 5.;
const GROUP_LABEL: f32 = 32.;
const GROUP_GAP: f32 = 2.;
const GROUP_RADIUS: f32 = 8.;
const MENU_RADIUS: f32 = 16.;
const BASELINE_PADDING: f32 = 8.;
const BASELINE_DIVIDER: f32 = 17.;
const BASELINE_RADIUS: f32 = 4.;

/// One item's full height in design px, including insets and a divider.
pub(crate) fn item_height(
    style: MenuStyle,
    density: MenuDensity,
    supporting: bool,
    divider: bool,
) -> f32 {
    if style.expressive() {
        let content = if supporting {
            12. + 20. + 16. + 12.
        } else {
            EXPRESSIVE_ITEM
        } - density.expressive_reduction();
        content + 2. * EXPRESSIVE_INSET_Y + if divider { EXPRESSIVE_DIVIDER } else { 0. }
    } else {
        let content = if supporting {
            (8.0_f32 + 24. + 20. + 8.).max(density.item_height())
        } else {
            density.item_height()
        };
        content + if divider { BASELINE_DIVIDER } else { 0. }
    }
}

/// A whole menu's height in design px: one `(labelled, items)` entry per
/// group, one `(supporting, divider)` entry per item.
pub(crate) fn menu_height<G, I>(style: MenuStyle, density: MenuDensity, groups: G) -> f32
where
    G: IntoIterator<Item = (bool, I)>,
    I: IntoIterator<Item = (bool, bool)>,
{
    let expressive = style.expressive();
    let mut count = 0usize;
    let mut total = 0.;
    for (label, items) in groups {
        count += 1;
        let items: f32 = items
            .into_iter()
            .map(|(supporting, divider)| item_height(style, density, supporting, divider))
            .sum();
        let inset = if expressive {
            2. * EXPRESSIVE_INSET_Y
        } else {
            0.
        };
        total += inset + if label { GROUP_LABEL } else { 0. } + items;
    }
    let between = count.saturating_sub(1) as f32;
    if expressive {
        total + GROUP_GAP * between
    } else {
        2. * BASELINE_PADDING + total + BASELINE_DIVIDER * between
    }
}

/// Focus handles for a menu's root and items, kept across renders.
struct MenuFocus {
    root: FocusHandle,
    items: Vec<FocusHandle>,
    scroll: ScrollHandle,
    /// Whether a scrolling menu has been brought to its selected item.
    placed: bool,
}

/// `pixels` in design px at the window's rem size.
fn design_px(pixels: Pixels, rem_size: Pixels) -> f32 {
    f32::from(pixels) * BASE_PX / f32::from(rem_size)
}

/// How far down a scrolling menu shows, in design px.
fn scrolled(scroll: &ScrollHandle, rem_size: Pixels) -> f32 {
    design_px(-scroll.offset().y, rem_size)
}

fn scroll_to(scroll: &ScrollHandle, offset: f32, rem_size: Pixels) {
    scroll.set_offset(point(px(0.), -rem(offset).to_pixels(rem_size)));
}

/// The offset that shows the row `top..bottom` in a `viewport` tall menu,
/// moving as little as possible.
pub(crate) fn reveal_offset(current: f32, (top, bottom): (f32, f32), viewport: f32) -> f32 {
    if top < current {
        top
    } else if bottom > current + viewport {
        bottom - viewport
    } else {
        current
    }
}

/// The offset that centers the row `top..bottom`, as far as the menu's
/// `natural` height allows.
pub(crate) fn centered_offset((top, bottom): (f32, f32), viewport: f32, natural: f32) -> f32 {
    ((top + bottom - viewport) / 2.).clamp(0., (natural - viewport).max(0.))
}

/// The space Base keeps between a popup and each edge of the content: its 8px
/// margin and the window's frame. Compose Material 3 keeps a menu 48dp from
/// the window's edges (`MenuVerticalMargin`); this kit keeps Base's margin, so
/// a menu in a short desktop window has more room.
fn popup_edge(window: &Window) -> Pixels {
    const POPUP_MARGIN: Pixels = px(8.);
    POPUP_MARGIN + window.client_inset().unwrap_or(px(0.))
}

/// How much of the window a popup can fill, in design px.
fn window_room(window: &Window) -> f32 {
    let room = window.viewport_size().height - popup_edge(window) * 2.;
    design_px(room, window.rem_size()).max(0.)
}

/// The gap between a popup and its trigger, in design px. A kit choice: Compose
/// Material 3 opens a dropdown menu flush with its anchor, and the references
/// give no gap.
const POPUP_GAP: f32 = 4.;

/// The room a popup has `(above, below)` a `trigger`, in design px, never more
/// than the window's: a trigger outside the window would otherwise promise
/// room past its edge.
fn room_beside(trigger: Bounds<Pixels>, window: &Window) -> (f32, f32) {
    let most = window_room(window);
    let rem_size = window.rem_size();
    let edge = popup_edge(window);
    let gap = rem(POPUP_GAP).to_pixels(rem_size);
    let above = trigger.top() - edge - gap;
    let below = window.viewport_size().height - edge - trigger.bottom() - gap;
    (
        design_px(above, rem_size).clamp(0., most),
        design_px(below, rem_size).clamp(0., most),
    )
}

/// An M3 menu surface. Use [`MenuPopup`] to open it from a trigger.
///
/// # Example
///
/// ```
/// use gpui_kit::prelude::*;
/// use gpui_kit_material::{icons::Symbol, prelude::*};
///
/// let icon = |symbol: Symbol| symbol.icon().component().size(rem(20.));
/// // Expressive groups, 2dp apart; a baseline menu divides items instead.
/// let menu = MaterialMenu::new("edit-menu")
///     .group(MenuGroup::new().items([
///         MaterialMenuItem::new("Cut").leading(icon(Symbol::CONTENT_CUT)),
///         MaterialMenuItem::new("Copy").leading(icon(Symbol::CONTENT_COPY)),
///     ]))
///     .group(MenuGroup::new().item(
///         MaterialMenuItem::new("Select all").on_click(|_, _, _| println!("Select all")),
///     ))
///     .density(MenuDensity::Dense);
/// # let _ = menu;
/// ```
#[derive(gpui::IntoElement)]
pub struct MaterialMenu {
    id: ElementId,
    style: MenuStyle,
    density: MenuDensity,
    groups: Vec<MenuGroup>,
    width: Option<f32>,
    max_height: Option<f32>,
    scroll: Option<ScrollHandle>,
    elevated: bool,
    focus: Option<FocusHandle>,
    on_dismiss: Option<Notify>,
    on_activate: Option<Notify>,
    on_collapse: Option<Notify>,
    focus_item: Option<usize>,
}

impl MaterialMenu {
    /// Leave every item as it looks, with nothing to activate.
    fn make_inert(&mut self) {
        for group in &mut self.groups {
            for item in &mut group.items {
                item.on_click = None;
            }
        }
    }

    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: MenuStyle::Standard,
            density: MenuDensity::Default,
            groups: Vec::new(),
            width: None,
            max_height: None,
            scroll: None,
            elevated: true,
            focus: None,
            on_dismiss: None,
            on_activate: None,
            on_collapse: None,
            focus_item: None,
        }
    }
    pub fn style(mut self, style: MenuStyle) -> Self {
        self.style = style;
        self
    }
    /// Item density: the kit's baseline densities, and half those steps for
    /// Expressive items.
    pub fn density(mut self, density: MenuDensity) -> Self {
        self.density = density;
        self
    }
    /// Append an item to the last group.
    pub fn item(mut self, item: MaterialMenuItem) -> Self {
        if self.groups.is_empty() {
            self.groups.push(MenuGroup::new());
        }
        self.groups.last_mut().unwrap().items.push(item);
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = MaterialMenuItem>) -> Self {
        for item in items {
            self = self.item(item);
        }
        self
    }
    pub fn group(mut self, group: MenuGroup) -> Self {
        self.groups.push(group);
        self
    }
    /// Width in design px (default 200 baseline, 208 Expressive).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
    /// The tallest the menu grows, in design px; past it the menu scrolls.
    /// [`MenuPopup`] also keeps it inside the window without being asked.
    ///
    /// GPUI clips to rectangles, so a scrolling Expressive menu paints one
    /// rounded container behind its groups instead of one for each, which
    /// would show square corners where they are cut off.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }
    /// Lower [`Self::max_height`] to `room` if it is taller.
    fn fit(mut self, room: f32) -> Self {
        self.max_height = Some(self.max_height.map_or(room, |height| height.min(room)));
        self
    }
    /// Paint the menu's level 2 shadow (default). A popup paints it on its
    /// revealing frame instead.
    pub fn elevated(mut self, elevated: bool) -> Self {
        self.elevated = elevated;
        self
    }
    /// The handle the menu root tracks; arrow keys from it reach the items.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }
    /// Escape inside the menu.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }
    /// Runs after any enabled item (other than a sub-menu) is activated.
    pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(handler));
        self
    }
    /// Focus the item at flat `index` this frame, e.g. a sub-menu's owner as
    /// the sub-menu closes.
    pub(crate) fn focus_item(mut self, index: usize) -> Self {
        self.focus_item = Some(index);
        self
    }
    /// Left inside the menu, e.g. to close it as a cascading sub-menu.
    pub fn on_collapse(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_collapse = Some(Rc::new(handler));
        self
    }

    /// The menu's natural height in design px.
    pub fn natural_height(&self) -> f32 {
        menu_height(
            self.style,
            self.density,
            self.groups.iter().map(|group| {
                (
                    group.label.is_some(),
                    group
                        .items
                        .iter()
                        .map(|item| (item.supporting.is_some(), item.divider)),
                )
            }),
        )
    }

    /// The height the menu shows in design px: its natural height, or
    /// [`Self::max_height`] if that is less.
    pub fn visible_height(&self) -> f32 {
        let natural = self.natural_height();
        self.max_height.map_or(natural, |max| natural.min(max))
    }

    /// The bottom of the item at flat `index` (its row without a divider
    /// below it), in design px from the menu's top.
    pub fn item_bottom(&self, index: usize) -> Option<f32> {
        self.item_rows().get(index).map(|row| row.1)
    }

    /// The top of the item at flat `index` (its row, with an Expressive
    /// row's insets), in design px from the menu's top.
    pub fn item_top(&self, index: usize) -> Option<f32> {
        self.item_rows().get(index).map(|row| row.0)
    }

    /// Each item's `(top, bottom)` by flat index, as [`Self::item_top`] and
    /// [`Self::item_bottom`] give them.
    fn item_rows(&self) -> Vec<(f32, f32)> {
        let expressive = self.style.expressive();
        let mut top = if expressive { 0. } else { BASELINE_PADDING };
        let mut rows = Vec::new();
        for (group_ix, group) in self.groups.iter().enumerate() {
            if group_ix > 0 {
                top += if expressive {
                    GROUP_GAP
                } else {
                    BASELINE_DIVIDER
                };
            }
            if expressive {
                top += EXPRESSIVE_INSET_Y;
            }
            if group.label.is_some() {
                top += GROUP_LABEL;
            }
            for item in &group.items {
                let supporting = item.supporting.is_some();
                let row = item_height(self.style, self.density, supporting, false);
                rows.push((top, top + row));
                top += item_height(self.style, self.density, supporting, item.divider);
            }
            if expressive {
                top += EXPRESSIVE_INSET_Y;
            }
        }
        rows
    }
}

impl RenderOnce for MaterialMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = self.style;
        let density = self.density;
        let count: usize = self.groups.iter().map(|g| g.items.len()).sum();
        let natural = self.natural_height();
        // Only a menu that outgrows its limit scrolls, and only it needs its rows.
        let scrolling = self
            .max_height
            .filter(|max| natural > *max)
            .map(|viewport| (viewport, Rc::<[(f32, f32)]>::from(self.item_rows())));
        let selected = self
            .groups
            .iter()
            .flat_map(|g| &g.items)
            .position(|item| item.selected);
        let state =
            window.use_keyed_state((self.id.clone(), "menu-focus"), cx, |_, cx| MenuFocus {
                root: cx.focus_handle(),
                items: Vec::new(),
                scroll: ScrollHandle::new(),
                placed: false,
            });
        state.update(cx, |state, cx| {
            while state.items.len() < count {
                state.items.push(cx.focus_handle());
            }
            state.items.truncate(count);
        });
        let root_focus = self
            .focus
            .clone()
            .unwrap_or_else(|| state.read(cx).root.clone());
        let handles = state.read(cx).items.clone();
        let scroll = self
            .scroll
            .clone()
            .unwrap_or_else(|| state.read(cx).scroll.clone());
        if let Some((viewport, rows)) = &scrolling {
            let placed = state.update(cx, |state, _| std::mem::replace(&mut state.placed, true));
            if !placed && let Some(row) = selected.and_then(|ix| rows.get(ix)) {
                let offset = centered_offset(*row, *viewport, natural);
                scroll_to(&scroll, offset, window.rem_size());
            }
        }
        if let Some(handle) = self.focus_item.and_then(|ix| handles.get(ix)) {
            handle.focus(window, cx);
        }
        let skip: Rc<[bool]> = self
            .groups
            .iter()
            .flat_map(|g| g.items.iter().map(|i| i.disabled || i.preview.is_some()))
            .collect();
        // Enter and Space activate the focused item (the popover's content
        // context reserves them, so the menu answers them itself).
        let activators: Rc<Vec<Option<(Handler, bool)>>> = Rc::new(
            self.groups
                .iter()
                .flat_map(|g| {
                    g.items.iter().map(|i| {
                        i.on_click
                            .clone()
                            .filter(|_| !i.disabled && i.preview.is_none())
                            .map(|h| (h, i.submenu))
                    })
                })
                .collect(),
        );
        // Right opens a focused sub-menu item: it activates the item, whose
        // handler opens the sub-menu.
        let expand = {
            let handles = handles.clone();
            let activators = activators.clone();
            move |window: &mut Window, cx: &mut App| {
                let focused = handles.iter().position(|h| h.is_focused(window));
                match focused.and_then(|ix| activators.get(ix)) {
                    Some(Some((handler, true))) => {
                        let event = ClickEvent::Keyboard(KeyboardClickEvent {
                            button: KeyboardButton::Enter,
                            bounds: Bounds::default(),
                        });
                        handler(&event, window, cx);
                    }
                    _ => cx.propagate(),
                }
            }
        };
        let on_collapse = self.on_collapse.clone();
        let collapse = move |window: &mut Window, cx: &mut App| {
            if let Some(collapse) = on_collapse.as_ref() {
                collapse(window, cx);
            } else {
                cx.propagate();
            }
        };
        let activate = {
            let handles = handles.clone();
            let activators = activators.clone();
            let on_activate = self.on_activate.clone();
            move |window: &mut Window, cx: &mut App| {
                let Some(ix) = handles.iter().position(|h| h.is_focused(window)) else {
                    return;
                };
                if let Some(Some((handler, submenu))) = activators.get(ix) {
                    let event = ClickEvent::Keyboard(KeyboardClickEvent {
                        button: KeyboardButton::Enter,
                        bounds: Bounds::default(),
                    });
                    handler(&event, window, cx);
                    if !submenu && let Some(done) = on_activate.as_ref() {
                        done(window, cx);
                    }
                }
            }
        };
        let navigate = {
            let handles = handles.clone();
            let skip = skip.clone();
            let scrolling = scrolling.clone();
            let scroll = scroll.clone();
            move |direction: Direction, window: &mut Window, cx: &mut App| {
                // Until an item has focus, the keyboard starts where the menu shows it is: a
                // dropdown opens centered on its selected option.
                let current = handles
                    .iter()
                    .position(|h| h.is_focused(window))
                    .or(selected);
                if let Some(ix) = target(&skip, current, direction) {
                    handles[ix].focus(window, cx);
                    if let Some((viewport, rows)) = &scrolling {
                        let (top, bottom) = rows[ix];
                        // The ends take the menu's padding with them.
                        let row = (
                            if ix == 0 { 0. } else { top },
                            if ix + 1 == rows.len() {
                                natural
                            } else {
                                bottom
                            },
                        );
                        let rem_size = window.rem_size();
                        let current = scrolled(&scroll, rem_size);
                        scroll_to(&scroll, reveal_offset(current, row, *viewport), rem_size);
                    }
                }
            }
        };
        let width = self
            .width
            .unwrap_or(if style.expressive() { 208. } else { 200. });
        let container = color(container_role(style));
        let on_activate = self.on_activate.clone();
        let group_count = self.groups.len();
        let mut index = 0usize;
        let mut groups = Vec::with_capacity(group_count);
        for (group_ix, group) in self.groups.into_iter().enumerate() {
            let mut items = Vec::with_capacity(group.items.len());
            for item in group.items {
                let focus = handles.get(index).map(|handle| {
                    let keyboard = window.last_input_was_keyboard() && handle.is_focused(window);
                    (handle.clone(), keyboard)
                });
                items.push(render_item(
                    // Scoped by the menu container's own id below, so a
                    // static name is unique without formatting per frame.
                    ("material-menu-item", index),
                    item,
                    style,
                    density,
                    focus,
                    on_activate.clone(),
                    window.rem_size(),
                ));
                index += 1;
            }
            let label = group.label.map(|label| {
                let ink = if style == MenuStyle::Vibrant {
                    color(ColorRole::ON_TERTIARY_CONTAINER)
                } else {
                    color(ColorRole::ON_SURFACE_VARIANT)
                };
                type_style(div(), TypeRole::LabelLarge)
                    .h(rem(GROUP_LABEL))
                    .flex()
                    .items_center()
                    .pt(rem(6.))
                    .px(rem(12. + EXPRESSIVE_INSET_X))
                    .text_color(ink)
                    .child(label)
            });
            let group_el = if style.expressive() {
                let top = if group_ix == 0 {
                    MENU_RADIUS
                } else {
                    GROUP_RADIUS
                };
                let bottom = if group_ix + 1 == group_count {
                    MENU_RADIUS
                } else {
                    GROUP_RADIUS
                };
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .py(rem(EXPRESSIVE_INSET_Y))
                    .when(scrolling.is_none(), |group| group.bg(container))
                    .rounded_t(rem(top))
                    .rounded_b(rem(bottom))
                    .children(label)
                    .children(items)
            } else {
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .when(group_ix > 0, |this| this.child(divider_line(false)))
                    .children(label)
                    .children(items)
            };
            groups.push(group_el);
        }
        let on_dismiss = self.on_dismiss.clone();
        let nav_up = navigate.clone();
        let nav_down = navigate.clone();
        let nav_first = navigate.clone();
        let nav_last = navigate;
        div()
            .id(self.id.clone())
            .debug_selector(|| "material-menu".into())
            .role(Role::Menu)
            .key_context(MENU_CONTEXT)
            .track_focus(&root_focus)
            .on_action(move |_: &SelectUp, window, cx| nav_up(Direction::Previous, window, cx))
            .on_action(move |_: &SelectDown, window, cx| nav_down(Direction::Next, window, cx))
            .on_action(move |_: &SelectFirst, window, cx| nav_first(Direction::First, window, cx))
            .on_action(move |_: &SelectLast, window, cx| nav_last(Direction::Last, window, cx))
            .on_action(move |_: &Confirm, window, cx| activate(window, cx))
            .on_action(move |_: &SelectRight, window, cx| expand(window, cx))
            .on_action(move |_: &SelectLeft, window, cx| collapse(window, cx))
            .on_action(move |_: &Cancel, window, cx| {
                if let Some(dismiss) = on_dismiss.as_ref() {
                    dismiss(window, cx);
                } else {
                    cx.propagate();
                }
            })
            .flex()
            .flex_col()
            .flex_shrink_0()
            .w(rem(width))
            .map(|this| {
                if style.expressive() {
                    this.rounded(rem(MENU_RADIUS))
                } else {
                    this.rounded(rem(BASELINE_RADIUS)).bg(container)
                }
            })
            .when(self.elevated, |this| this.shadow(elevation::level(2)))
            .map(|this| match scrolling {
                None => group_spacing(this, style).children(groups),
                Some((viewport, _)) => this
                    .relative()
                    .h(rem(viewport))
                    .bg(container)
                    .child(
                        div()
                            .id((self.id.clone(), "menu-scroll"))
                            .size_full()
                            .overflow_y_scroll()
                            .track_scroll(&scroll)
                            .child(
                                group_spacing(div().flex().flex_col().w_full(), style)
                                    .children(groups),
                            ),
                    )
                    .child(
                        gpui_base::Scrollbar::vertical(&scroll)
                            .id((self.id.clone(), "menu-scrollbar")),
                    ),
            })
    }
}

/// What separates a menu's groups from its edges and each other.
fn group_spacing<E: Styled>(this: E, style: MenuStyle) -> E {
    if style.expressive() {
        this.gap(rem(GROUP_GAP))
    } else {
        this.py(rem(BASELINE_PADDING))
    }
}

fn divider_line(expressive: bool) -> Div {
    let line = div()
        .w_full()
        .h(px(1.))
        .bg(color(ColorRole::OUTLINE_VARIANT));
    if expressive {
        div()
            .w_full()
            .px(rem(8. + EXPRESSIVE_INSET_X))
            .py(rem(2.))
            .mt(rem(4.))
            .child(line)
    } else {
        div().w_full().py(rem(8.)).child(line)
    }
}

/// The sub-menu indicator, `arrow_right` from the Material Symbols catalog.
fn submenu_icon(size: f32) -> Icon {
    Icon::from(Symbol::named("arrow_right").expect("arrow_right is a Material Symbol"))
        .size(rem(size))
}

/// One line of an item's text: too long for the menu, it ends in an ellipsis
/// rather than stopping mid-letter at the item's edge.
fn item_text(role: TypeRole, ink: Rgba, text: SharedString) -> Div {
    type_style(div(), role)
        .text_color(ink)
        .truncate()
        .child(text)
}

fn render_item(
    id: impl Into<ElementId>,
    item: MaterialMenuItem,
    style: MenuStyle,
    density: MenuDensity,
    // And whether it has focus by keyboard.
    focus: Option<(FocusHandle, bool)>,
    on_activate: Option<Notify>,
    rem_size: Pixels,
) -> AnyElement {
    let expressive = style.expressive();
    let paint = item_paint(style, item.selected);
    let disabled = item.disabled;
    let fade = |c: Rgba| {
        if disabled {
            alpha(c, state::DISABLED_CONTENT)
        } else {
            c
        }
    };
    let icon_size = if expressive { 20. } else { 24. };
    let radius = if expressive {
        if item.selected { 12. } else { 4. }
    } else {
        0.
    };
    // The ring and ripple share the item container's rem-sized corners.
    let corners = Corners::all(rem(radius).to_pixels(rem_size));
    let content_height = item_height(style, density, item.supporting.is_some(), false)
        - if expressive {
            2. * EXPRESSIVE_INSET_Y
        } else {
            0.
        };
    let layer = paint.layer;
    // A disabled selected standard item keeps its shape but drops its
    // container; a vibrant one keeps its tertiary container.
    let container = paint
        .container
        .filter(|_| !(disabled && style == MenuStyle::Standard));
    // State layers composite over the item's own container.
    let under = container.unwrap_or(alpha(layer, 0.));
    let slot = |child: AnyElement, ink: Rgba| {
        div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .size(rem(icon_size))
            .text_color(ink)
            .child(child)
    };
    let label_role = if expressive {
        TypeRole::LabelLarge
    } else {
        TypeRole::BodyLarge
    };
    let supporting_role = if expressive {
        TypeRole::BodySmall
    } else {
        TypeRole::BodyMedium
    };
    let trailing_icon = item.trailing.or_else(|| {
        item.submenu
            .then(|| submenu_icon(icon_size).into_any_element())
    });
    let mut button = RingedButton::new(id, RingPlacement::Inner(corners))
        .state_layers(
            state::layer(under, layer, state::HOVER),
            state::layer(under, layer, state::PRESS),
        )
        .ripple(layer)
        .disabled(disabled)
        .accessibility_label(item.label.clone())
        .map(|b| {
            b.role(Role::MenuItem)
                .flex()
                .items_center()
                .w_full()
                .h(rem(content_height))
                .px(rem(12.))
                .gap(rem(if expressive { 8. } else { 12. }))
                .rounded(rem(radius))
                .when(!disabled, |b| b.cursor_pointer())
        })
        .when_some(container, |b, c| b.bg(c))
        .children(item.leading.map(|icon| slot(icon, fade(paint.icon))))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .child(item_text(label_role, fade(paint.label), item.label))
                .children(
                    item.supporting
                        .map(|text| item_text(supporting_role, fade(paint.supporting), text)),
                ),
        )
        .children(item.trailing_text.map(|text| {
            type_style(div().flex_shrink_0(), TypeRole::LabelLarge)
                .text_color(fade(paint.trailing))
                .child(text)
        }))
        .children(trailing_icon.map(|icon| slot(icon, fade(paint.trailing))));
    if let Some(preview) = item.preview {
        button = button.preview_state(preview);
        if preview.is_focused() {
            // The kit paints the focus layer as well as the ring.
            button = button.map(|b| b.bg(state::layer(under, layer, state::FOCUS)));
        }
    } else if let Some((focus, keyboard)) = focus {
        button = button.track_focus(&focus);
        if keyboard {
            button = button.map(|b| b.bg(state::layer(under, layer, state::FOCUS)));
        }
    }
    if let Some(handler) = item
        .on_click
        .filter(|_| !disabled && item.preview.is_none())
    {
        let submenu = item.submenu;
        button = button.on_click(move |event, window, cx| {
            handler(event, window, cx);
            if !submenu && let Some(activate) = on_activate.as_ref() {
                activate(window, cx);
            }
        });
    }
    let row = if expressive {
        div()
            .w_full()
            .px(rem(EXPRESSIVE_INSET_X))
            .py(rem(EXPRESSIVE_INSET_Y))
            .child(button)
    } else {
        div().w_full().child(button)
    };
    if item.divider {
        div()
            .w_full()
            .child(row)
            .child(divider_line(expressive).when(expressive, |d| d.mt(rem(0.))))
            .into_any_element()
    } else {
        row.into_any_element()
    }
}

/// The open progress `(reveal, fade)` a popup paints this frame.
fn open_motion(id: &ElementId, open: bool, window: &mut Window, cx: &mut App) -> (f32, f32) {
    let target = if open { 1. } else { 0. };
    let reveal = motion::token_value(
        (id.clone(), "menu-reveal"),
        target,
        MotionToken::DefaultSpatial,
        window,
        cx,
    );
    let fade = motion::token_value(
        (id.clone(), "menu-fade"),
        target,
        MotionToken::FastEffects,
        window,
        cx,
    )
    .clamp(0., 1.);
    (reveal, fade)
}

/// What a menu popup keeps between frames, in one entity, as a page holds many closed ones
/// and a frame records each read. The channels keep their own clocks.
struct PopupState {
    focus: FocusHandle,
    /// The trigger's size in design px, measured as it prepaints.
    trigger_size: Rc<std::cell::Cell<(f32, f32)>>,
    /// The trigger's window bounds, once it has prepainted.
    trigger_bounds: Rc<std::cell::Cell<Option<Bounds<Pixels>>>>,
    /// Closing the sub-menu by keyboard returns focus to its owner item.
    restore: Rc<std::cell::Cell<bool>>,
    reveal: motion::TokenTransitionState<f32>,
    fade: motion::TokenTransitionState<f32>,
}

/// Whether a popup still paints its surface: open, or closing until both
/// channels settle.
pub(crate) fn popup_visible(open: bool, reveal: f32, fade: f32) -> bool {
    open || reveal > 0.001 || fade > 0.001
}

#[derive(gpui::IntoElement)]
struct Trigger {
    child: AnyElement,
    open: bool,
}

impl gpui_kit::component::Selectable for Trigger {
    fn selected(mut self, selected: bool) -> Self {
        self.open = selected;
        self
    }
    fn is_selected(&self) -> bool {
        self.open
    }
}

impl RenderOnce for Trigger {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.child
    }
}

/// A trigger that opens a [`MaterialMenu`] in Kit's popover overlay.
///
/// `open` is controlled: Kit's popover reports pointer toggles, outside
/// clicks and Escape through `on_open_change`, and an activated item closes
/// the menu the same way (unless [`Self::close_on_activate`] is off).
/// Keyboard activation of the trigger itself is the caller's (e.g. its
/// button's click handler).
///
/// A cascading sub-menu ([`Self::submenu`]) opens beside the item that owns
/// it, on the side away from the popup's anchor edge, with the same reveal
/// motion. Right on a sub-menu item activates it (the caller opens the
/// sub-menu, which takes focus); Left or Escape inside the sub-menu reports
/// `false` through [`Self::on_submenu_change`] and returns focus to the
/// parent menu.
///
/// # Example
///
/// ```
/// use gpui_kit::{ClickEvent, Window, prelude::*};
/// use gpui_kit_material::prelude::*;
///
/// struct Editor {
///     menu_open: bool,
/// }
///
/// impl Render for Editor {
///     fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
///         let button = material_button("format", ButtonVariant::Tonal).child("Format").on_click(
///             // Pointer presses reach the popup; keys arrive here.
///             cx.listener(|this, event: &ClickEvent, _, cx| {
///                 if matches!(event, ClickEvent::Keyboard(_)) {
///                     this.menu_open = !this.menu_open;
///                     cx.notify();
///                 }
///             }),
///         );
///         let menu = MaterialMenu::new("format-menu")
///             .items([MaterialMenuItem::new("Bold"), MaterialMenuItem::new("Italic")]);
///         MenuPopup::new("format-popup", button, menu)
///             .open(self.menu_open)
///             .on_open_change(cx.listener(|this, open: &bool, _, cx| {
///                 this.menu_open = *open;
///                 cx.notify();
///             }))
///     }
/// }
/// ```
#[derive(gpui::IntoElement)]
pub struct MenuPopup {
    id: ElementId,
    trigger: AnyElement,
    menu: MaterialMenu,
    open: bool,
    on_open_change: Option<OpenChange>,
    anchor: Anchor,
    close_on_activate: bool,
    submenu: Option<(usize, MaterialMenu)>,
    submenu_open: bool,
    on_submenu_change: Option<OpenChange>,
    full_width: bool,
    on_closed: Option<OnClosed>,
}

impl MenuPopup {
    pub fn new(id: impl Into<ElementId>, trigger: impl IntoElement, menu: MaterialMenu) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            menu,
            open: false,
            on_open_change: None,
            anchor: Anchor::TopLeft,
            close_on_activate: true,
            submenu: None,
            submenu_open: false,
            on_submenu_change: None,
            full_width: false,
            on_closed: None,
        }
    }
    /// Fill the parent's width, with the menu as wide as the trigger.
    #[cfg(feature = "inputs")]
    fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }
    /// Runs on a frame that finds the menu closed and settled.
    fn on_closed(mut self, handler: impl FnOnce(&mut App) + 'static) -> Self {
        self.on_closed = Some(Box::new(handler));
        self
    }
    /// Close the menu when an item (other than a sub-menu) is activated
    /// (default). Off, the menu stays open, e.g. to toggle several items.
    pub fn close_on_activate(mut self, close: bool) -> Self {
        self.close_on_activate = close;
        self
    }
    /// A cascading sub-menu for the main menu's item at flat `index`
    /// (counting across groups), shown while `open`.
    pub fn submenu(mut self, index: usize, menu: MaterialMenu, open: bool) -> Self {
        self.submenu = Some((index, menu));
        self.submenu_open = open;
        self
    }
    /// Left or Escape inside the open sub-menu reports `false`.
    pub fn on_submenu_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_submenu_change = Some(Rc::new(handler));
        self
    }
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    pub fn on_open_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(handler));
        self
    }
    /// The popup's own anchor corner (Kit's convention; default top-left,
    /// below the trigger). `LeftCenter` and `RightCenter` place the menu
    /// beside the trigger, to its right or left, with their tops aligned
    /// (M3's "right start" and "left start" placements). A menu above or
    /// below the trigger that does not fit there opens on the other side
    /// when it fits there, and otherwise scrolls on the side with more room.
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }
}

/// Whether a popup anchored at `anchor` sits above its trigger, so it
/// reveals from its bottom edge.
pub(crate) fn opens_upward(anchor: Anchor) -> bool {
    matches!(
        anchor,
        Anchor::BottomLeft | Anchor::BottomCenter | Anchor::BottomRight
    )
}

/// The side of its trigger a popup opens on: `Some(true)` above, `Some(false)`
/// below, or `None` when the trigger leaves no usable room on either side.
///
/// `need` is how tall the popup is `(above, below)` the trigger (a sub-menu's
/// column can make the two differ), `room` the space there, and `least` the
/// height under which a popup is no use (about one row), all in design px.
///
/// A popup opens on its side of the trigger, on the other side when it fits
/// only there, and otherwise on whichever side has more room, where it
/// scrolls. Compose Material 3's `DropdownMenuPositionProvider` tries below,
/// then above; its `ExposedDropdownMenu` caps the menu at the larger of the
/// heights above and below the anchor, and at the whole window when the anchor
/// is off it (`calculateMaxHeight`). Base's side positioner follows the same
/// order. Kit's popover places a popup by its corner and only clamps it into
/// the window, which slides one too tall for its side back over its trigger.
/// The one-row threshold, and keeping that clamping with `None` for a trigger
/// that leaves less, are a kit choice: Compose gives its menu the whole window
/// only when the anchor is off it.
pub(crate) fn popup_side(
    asks_above: bool,
    (need_above, need_below): (f32, f32),
    (above, below): (f32, f32),
    least: f32,
) -> Option<bool> {
    let ((asked, asked_need), (other, other_need)) = if asks_above {
        ((above, need_above), (below, need_below))
    } else {
        ((below, need_below), (above, need_above))
    };
    let flip = asked_need > asked && (other_need <= other || other > asked);
    let (room, need) = if flip {
        (other, other_need)
    } else {
        (asked, asked_need)
    };
    (room >= need || room >= least).then_some(asks_above != flip)
}

/// The anchor on the other side of the trigger, at the same corner of it.
fn vertically_flipped(anchor: Anchor) -> Anchor {
    match anchor {
        Anchor::TopLeft => Anchor::BottomLeft,
        Anchor::TopCenter => Anchor::BottomCenter,
        Anchor::TopRight => Anchor::BottomRight,
        Anchor::BottomLeft => Anchor::TopLeft,
        Anchor::BottomCenter => Anchor::TopCenter,
        Anchor::BottomRight => Anchor::TopRight,
        other => other,
    }
}

/// Whether a sub-menu cascades to the left: popups anchored on their right
/// edge (end placements, or beside the trigger's left) open leftward.
pub(crate) fn cascades_left(anchor: Anchor) -> bool {
    matches!(
        anchor,
        Anchor::TopRight | Anchor::BottomRight | Anchor::RightCenter
    )
}

/// The space between a menu and its cascading sub-menu.
const CASCADE_GAP: f32 = 4.;

/// A menu's revealing frame: `height * shown` tall, its shadow and opacity
/// following `fade`, revealing from the top (or the bottom when `upward`).
fn reveal_surface(
    menu: impl IntoElement,
    style: MenuStyle,
    height: f32,
    (shown, fade): (f32, f32),
    upward: bool,
) -> Div {
    let radius = if style.expressive() {
        MENU_RADIUS
    } else {
        BASELINE_RADIUS
    };
    let ring = fade * fade * fade;
    let shadow: Vec<BoxShadow> = elevation::level(2)
        .into_iter()
        .map(|mut s| {
            s.color.a *= ring;
            s
        })
        .collect();
    div()
        .flex()
        .flex_col()
        .when(upward, |surface| surface.justify_end())
        .flex_shrink_0()
        .h(rem(height * shown.clamp(0., 1.)))
        .overflow_hidden()
        .rounded(rem(radius))
        .shadow(shadow)
        .opacity(fade)
        .child(menu)
}

impl RenderOnce for MenuPopup {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Base keeps a popup inside the window; a menu taller than the window
        // would still lose its end there, so it scrolls instead.
        let room = window_room(window);
        self.menu = self.menu.fit(room);
        let state =
            window.use_keyed_state((self.id.clone(), "menu-popup"), cx, |_, cx| PopupState {
                focus: cx.focus_handle(),
                trigger_size: Default::default(),
                trigger_bounds: Default::default(),
                restore: Default::default(),
                reveal: Default::default(),
                fade: Default::default(),
            });
        let state = state.read(cx);
        let (focus, trigger_size, trigger_bounds, restore) = (
            state.focus.clone(),
            state.trigger_size.clone(),
            state.trigger_bounds.clone(),
            state.restore.clone(),
        );
        let (reveal_channel, fade_channel) = (state.reveal.clone(), state.fade.clone());
        let target = if self.open { 1. } else { 0. };
        let reveal = reveal_channel.follow(target, MotionToken::DefaultSpatial, window, cx);
        let fade = fade_channel
            .follow(target, MotionToken::FastEffects, window, cx)
            .clamp(0., 1.);
        let visible = popup_visible(self.open, reveal, fade);
        // A menu on its way out shows its items, and activates none of them.
        if !self.open {
            self.menu.make_inert();
            if let Some((_, submenu)) = self.submenu.as_mut() {
                submenu.make_inert();
            }
            // A sub-menu goes with its parent, and does not come back with it.
            if self.submenu_open
                && let Some(change) = self.on_submenu_change.clone()
            {
                window.defer(cx, move |window, cx| change(&false, window, cx));
            }
        }
        if !visible && let Some(closed) = self.on_closed.take() {
            closed(cx);
        }
        // The trigger's height places a side popup's top at the trigger's top, and its
        // width is a full-width menu's.
        let full_width = self.full_width;
        if full_width {
            self.menu = self.menu.width(trigger_size.get().0);
        }
        let style = self.menu.style;
        let mut anchor = self.anchor;
        let side = matches!(anchor, Anchor::LeftCenter | Anchor::RightCenter);
        // A sub-menu's column counts: below the trigger it reaches from its owner's
        // top down, above it from its owner's bottom up.
        let submenu_need = |menu: &MaterialMenu, above: bool| {
            self.submenu.as_ref().map_or(0., |(index, submenu)| {
                let first_row = if submenu.style.expressive() {
                    EXPRESSIVE_INSET_Y
                } else {
                    BASELINE_PADDING
                };
                let height = menu.visible_height();
                let inset = if above {
                    height - menu.item_bottom(*index).unwrap_or(height)
                } else {
                    menu.item_top(*index).unwrap_or(0.)
                };
                (inset - first_row).max(0.) + submenu.visible_height()
            })
        };
        // The room left for the sub-menu's column past its owner: the window's,
        // or the chosen side's.
        let mut side_room = room;
        // A menu too tall for its side of the trigger opens on the other side if it
        // fits there, and otherwise scrolls on the side with more room. Base would
        // only slide it back into the window, over its own trigger.
        // A trigger that leaves less than a row on both sides keeps that clamping.
        if !side && let Some(bounds) = trigger_bounds.get() {
            let beside = room_beside(bounds, window);
            let asks_above = opens_upward(anchor);
            let height = self.menu.visible_height();
            let need = |above: bool| height.max(submenu_need(&self.menu, above));
            let least = self.menu.item_bottom(0).unwrap_or(height).min(height);
            if let Some(above) = popup_side(asks_above, (need(true), need(false)), beside, least) {
                if above != asks_above {
                    anchor = vertically_flipped(anchor);
                }
                side_room = if above { beside.0 } else { beside.1 };
                self.menu = self.menu.fit(side_room);
            }
        }
        let height = self.menu.visible_height();
        let upward = opens_upward(anchor);
        let change = self.on_open_change.clone();
        let close = self.on_open_change.clone();
        let close_on_activate = self.close_on_activate;
        let close_menu: Notify = Rc::new(move |window: &mut Window, cx: &mut App| {
            if let Some(close) = close.as_ref() {
                close(&false, window, cx);
            }
        });
        let submenu_index = self.submenu.as_ref().map(|(index, _)| *index);
        // A sub-menu stays beside its owner wherever the menu has scrolled to, so
        // a popup with one holds the scroll handle where it can read it.
        let scrolled_by = if submenu_index.is_some() {
            let scroll = window
                .use_keyed_state((self.id.clone(), "menu-popup-scroll"), cx, |_, _| {
                    ScrollHandle::new()
                })
                .read(cx)
                .clone();
            if !visible {
                scroll.set_offset(Point::default());
            }
            let offset = scrolled(&scroll, window.rem_size());
            self.menu.scroll = Some(scroll);
            offset
        } else {
            0.
        };
        let submenu_top = submenu_index
            .and_then(|index| self.menu.item_top(index))
            .map(|top| top - scrolled_by);
        let submenu_bottom = submenu_index
            .and_then(|index| self.menu.item_bottom(index))
            .map(|bottom| bottom - scrolled_by);
        // A sub-menu's column starts from its owner, so it has what is left of the room past
        // that: told the whole room, the column outgrows the window and Base's clamp, which
        // can only pin an oversized box, leaves its end outside.
        let column_inset = |style: MenuStyle| {
            let first_row = if style.expressive() {
                EXPRESSIVE_INSET_Y
            } else {
                BASELINE_PADDING
            };
            if upward {
                (height - submenu_bottom.unwrap_or(height) - first_row).max(0.)
            } else {
                (submenu_top.unwrap_or(0.) - first_row).max(0.)
            }
        };
        self.submenu = self.submenu.map(|(index, menu)| {
            let room = (side_room - column_inset(menu.style)).max(0.);
            (index, menu.fit(room))
        });
        // The reserved cascade column's height below the trigger.
        let cascade_height = self.submenu.as_ref().map_or(0., |(_, submenu)| {
            column_inset(submenu.style) + submenu.visible_height()
        });
        let mut menu = self.menu;
        if let Some(index) = submenu_index
            && restore.replace(false)
        {
            menu = menu.focus_item(index);
        }
        let menu = menu
            .elevated(false)
            .track_focus(&focus)
            .on_dismiss({
                let close = close_menu.clone();
                move |window, cx| close(window, cx)
            })
            .on_activate({
                let close = close_menu.clone();
                move |window, cx| {
                    if close_on_activate {
                        close(window, cx)
                    }
                }
            });
        let shown = reveal.clamp(0., 1.);
        let surface = reveal_surface(menu, style, height, (shown, fade), upward)
            .debug_selector(|| "menu-popup-surface".into());

        // The cascading sub-menu, revealed on its own channels.
        let cascade = self.submenu.map(|(_, submenu)| {
            let open = self.open && self.submenu_open;
            let cascade_id = ElementId::from((self.id.clone(), "submenu"));
            let (sub_reveal, sub_fade) = open_motion(&cascade_id, open, window, cx);
            let sub_focus = window
                .use_keyed_state((self.id.clone(), "menu-submenu-focus"), cx, |_, cx| {
                    cx.focus_handle()
                })
                .read(cx)
                .clone();
            // The sub-menu takes focus as it opens.
            let was_open = window
                .use_keyed_state((self.id.clone(), "menu-submenu-open"), cx, |_, _| {
                    Rc::new(std::cell::Cell::new(false))
                })
                .read(cx)
                .clone();
            if open && !was_open.get() {
                sub_focus.focus(window, cx);
            }
            was_open.set(open);
            let sub_style = submenu.style;
            let sub_height = submenu.visible_height();
            let collapse: Notify = {
                let change = self.on_submenu_change.clone();
                let parent = focus.clone();
                let restore = restore.clone();
                Rc::new(move |window: &mut Window, cx: &mut App| {
                    parent.focus(window, cx);
                    restore.set(true);
                    if let Some(change) = change.as_ref() {
                        change(&false, window, cx);
                    }
                })
            };
            let submenu = submenu
                .elevated(false)
                .track_focus(&sub_focus)
                .on_dismiss({
                    let collapse = collapse.clone();
                    move |window, cx| collapse(window, cx)
                })
                .on_collapse(move |window, cx| collapse(window, cx))
                .on_activate({
                    let close = close_menu.clone();
                    move |window, cx| {
                        if close_on_activate {
                            close(window, cx)
                        }
                    }
                });
            let inset = column_inset(sub_style);
            let visible = popup_visible(open, sub_reveal, sub_fade);
            // The column is reserved while closed, so opening the sub-menu
            // never moves the menu Kit anchored. Below the trigger the
            // sub-menu's first item lines up with its owner and it extends
            // down; above the trigger its last item lines up with the
            // owner's bottom and it extends up, within the popup's box.
            let column = div().flex_shrink_0().flex().flex_col();
            let column = if upward {
                column.h(rem(sub_height + inset)).pb(rem(inset))
            } else {
                column.h(rem(inset + sub_height)).pt(rem(inset))
            };
            column.when(!visible, |cascade| cascade.invisible()).child(
                reveal_surface(
                    submenu,
                    sub_style,
                    sub_height,
                    (sub_reveal.clamp(0., 1.), sub_fade),
                    false,
                )
                .debug_selector(|| "menu-popup-submenu".into()),
            )
        });
        // Kit centers a side popup on the trigger; the margin moves the
        // menu's top to the trigger's top.
        let side_offset = if side {
            ((height * shown).max(cascade_height) - trigger_size.get().1).max(0.)
        } else {
            0.
        };
        let content = div()
            .flex()
            .map(|content| {
                if upward {
                    content.items_end()
                } else {
                    content.items_start()
                }
            })
            .gap(rem(CASCADE_GAP))
            .when(cascades_left(anchor), |content| content.flex_row_reverse())
            .when(side_offset > 0., |content| content.mt(rem(side_offset)))
            .child(surface)
            .children(cascade);
        let surface = std::cell::RefCell::new(Some(crate::scope::current_scope(content)));
        let open = self.open;
        let measure = trigger_size;
        let measure_bounds = trigger_bounds;
        let placed = visible;
        // Material buttons block the mouse for the elements under them, so
        // Kit's popover never sees a press on such a trigger. A window
        // listener toggles the menu instead (a press while open closes it
        // before the popover's outside-press dismissal, which then has nothing
        // to do). A passive hitbox over the trigger keeps what covers the
        // trigger, such as a dialog or an app bar, in charge of its own press;
        // it has to come after the trigger, which would otherwise occlude it.
        let toggle = self.on_open_change.clone();
        let press = canvas(
            |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
            move |_, hitbox, window, _| {
                let toggle = toggle.clone();
                window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                    if phase == DispatchPhase::Capture
                        && event.button == MouseButton::Left
                        && hitbox.is_hovered(window)
                        && let Some(toggle) = toggle.as_ref()
                    {
                        toggle(&!open, window, cx);
                    }
                });
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();
        let trigger = div()
            .relative()
            .when(full_width, |trigger| trigger.w_full())
            .child(self.trigger)
            .child(super::prepaint_bounds(press, move |bounds, window, _| {
                let design = |pixels: Pixels| {
                    f32::from(pixels) * crate::tokens::typography::BASE_PX
                        / f32::from(window.rem_size())
                };
                measure.set((design(bounds.size.width), design(bounds.size.height)));
                // The side was chosen from the bounds the last frame measured: a
                // trigger that has moved since asks for a frame that chooses again.
                if measure_bounds.replace(Some(bounds)) != Some(bounds) && placed {
                    window.request_animation_frame();
                }
            }));
        Popover::new(self.id.clone())
            .when(full_width, |popover| popover.w_full())
            .appearance(false)
            .anchor(anchor)
            .offset(rem(POPUP_GAP).to_pixels(window.rem_size()))
            .track_focus(&focus)
            .open(visible)
            .on_open_change(move |next, window, cx| {
                if let Some(change) = change.as_ref()
                    && *next != open
                {
                    change(next, window, cx);
                }
            })
            .trigger(Trigger {
                child: trigger.into_any_element(),
                open: self.open,
            })
            .content(move |_, _, _| {
                surface
                    .borrow_mut()
                    .take()
                    .unwrap_or_else(|| div().into_any_element())
            })
    }
}

/// A menu that opens at the pointer on a right press inside its parent.
///
/// Add it as the first child of the element it serves, which must be
/// `relative`: it takes no room, and it builds nothing until the press, so a
/// long list pays only for a hitbox and a listener per row. The menu behaves as
/// a [`MenuPopup`] does: it reveals on the same motion, keeps inside the window,
/// takes focus and returns it, and closes on an activated item, Escape or a press
/// outside. A right press elsewhere in the parent moves it there. Of nested
/// context menus, the innermost under the pointer opens.
///
/// A control that blocks the mouse, such as a button in the row, keeps its own
/// right press, as it does for Kit's `context_menu`. Give the same commands a
/// way in from the keyboard, usually a [`MenuPopup`] behind a "more" button.
///
/// ```
/// use gpui_kit::{div, prelude::*};
/// use gpui_kit_material::prelude::*;
///
/// # let row_content = div();
/// let row = div()
///     .relative()
///     .child(MaterialContextMenu::new("row-menu", |_, _| {
///         MaterialMenu::new("row-menu-items").item(MaterialMenuItem::new("Rename"))
///     }))
///     .child(row_content);
/// # let _ = row;
/// ```
#[derive(gpui::IntoElement)]
pub struct MaterialContextMenu {
    id: ElementId,
    menu: MenuBuilder,
}

impl MaterialContextMenu {
    /// `menu` builds the menu on each frame it is shown, and no other.
    pub fn new(
        id: impl Into<ElementId>,
        menu: impl Fn(&mut Window, &mut App) -> MaterialMenu + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            menu: Box::new(menu),
        }
    }
}

#[derive(Default)]
struct ContextMenuState {
    /// Whether the popup is in the tree: from the press until it has closed.
    mounted: bool,
    /// The popup enters the tree closed for one frame, so that it has a closed
    /// state to reveal from and a trigger position to open at.
    entering: bool,
    open: bool,
    /// The pointer's offset from the parent's start edge.
    position: Point<Pixels>,
}

impl RenderOnce for MaterialContextMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state((self.id.clone(), "context-menu"), cx, |_, _| {
            ContextMenuState::default()
        });
        let press = canvas(
            |bounds, window, _| (bounds, window.insert_hitbox(bounds, HitboxBehavior::Normal)),
            {
                let state = state.clone();
                move |_, (bounds, hitbox): (Bounds<Pixels>, Hitbox), window, _| {
                    window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                        if phase.bubble()
                            && event.button == MouseButton::Right
                            && hitbox.is_hovered(window)
                        {
                            // Bubbling reaches the innermost context menu first.
                            cx.stop_propagation();
                            let from_left = event.position - bounds.origin;
                            state.update(cx, |state, cx| {
                                state.entering = !state.mounted;
                                state.mounted = true;
                                state.open = true;
                                state.position = from_left;
                                cx.notify();
                            });
                        }
                    });
                }
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();
        let mut popup = None;
        if state.read(cx).mounted {
            let entering = state.update(cx, |state, _| std::mem::take(&mut state.entering));
            if entering {
                window.request_animation_frame();
            }
            let open = state.read(cx).open && !entering;
            let position = state.read(cx).position;
            let close = state.clone();
            let mut menu = MenuPopup::new((self.id, "popup"), div(), (self.menu)(window, cx))
                .open(open)
                .on_open_change(move |next, _, cx| {
                    close.update(cx, |state, cx| {
                        state.open = *next;
                        cx.notify();
                    })
                });
            // The frame that finds it closed still draws it, so its popover
            // returns focus before it leaves the tree.
            if !entering {
                let leave = state.clone();
                menu = menu.on_closed(move |cx| leave.update(cx, |state, _| state.mounted = false));
            }
            popup = Some(
                div()
                    .absolute()
                    .left(position.x)
                    .top(position.y)
                    .child(menu),
            );
        }
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .child(press)
            .children(popup)
    }
}

/// The M3 exposed dropdown menu: a text field that opens a menu of options.
///
/// The menu is built from [`Self::options`], or is the caller's own
/// [`Self::menu`] when its items need more than a label: supporting text,
/// leading icons, labelled groups or an action that is not a value. This
/// follows Jetpack Compose Material 3, whose `ExposedDropdownMenuBox` pairs
/// the text field with menu content the caller supplies
/// (`ExposedDropdownMenu { DropdownMenuItem(...) }`). Items in a caller's menu
/// that do not choose a value, such as an action that manages the values, are
/// the caller's responsibility: their handlers decide what they do, and the
/// field shows only what [`Self::value`] says.
///
/// # Example
///
/// ```
/// # use gpui_kit::{prelude::*, *};
/// use gpui_kit_material::prelude::*;
///
/// struct Order {
///     fruit: Option<usize>,
///     open: bool,
/// }
///
/// impl Render for Order {
///     fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
///         ExposedDropdown::new("fruit", "Fruit")
///             .options(["Apple", "Banana", "Cherry"])
///             .selected(self.fruit)
///             .open(self.open)
///             .on_open_change(cx.listener(|this, open: &bool, _, cx| {
///                 this.open = *open;
///                 cx.notify();
///             }))
///             .on_select(cx.listener(|this, index: &usize, _, cx| {
///                 (this.fruit, this.open) = (Some(*index), false);
///                 cx.notify();
///             }))
///     }
/// }
/// ```
#[cfg(feature = "inputs")]
#[derive(gpui::IntoElement)]
pub struct ExposedDropdown {
    id: ElementId,
    variant: TextFieldVariant,
    density: TextFieldDensity,
    shape: TextFieldShape,
    label: SharedString,
    show_label: bool,
    options: Vec<SharedString>,
    disabled_options: Vec<usize>,
    selected: Option<usize>,
    open: bool,
    disabled: bool,
    error: bool,
    required: bool,
    leading: Option<AnyElement>,
    supporting: Option<SharedString>,
    style: MenuStyle,
    width: f32,
    menu_max_height: Option<f32>,
    full_width: bool,
    menu: Option<MaterialMenu>,
    value: Option<SharedString>,
    on_open_change: Option<OpenChange>,
    on_select: Option<Select>,
}

#[cfg(feature = "inputs")]
impl ExposedDropdown {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            variant: TextFieldVariant::Filled,
            shape: TextFieldShape::Standard,
            density: TextFieldDensity::Default,
            label: label.into(),
            show_label: true,
            options: Vec::new(),
            disabled_options: Vec::new(),
            selected: None,
            open: false,
            disabled: false,
            error: false,
            required: false,
            leading: None,
            supporting: None,
            // The kit's menus anchored to a text field are baseline menus.
            style: MenuStyle::Baseline,
            width: 210.,
            menu_max_height: None,
            full_width: false,
            menu: None,
            value: None,
            on_open_change: None,
            on_select: None,
        }
    }
    /// Open the caller's menu instead of one built from [`Self::options`],
    /// whose items carry their own click handlers, selection and disabled
    /// state; [`Self::options`], [`Self::selected`], [`Self::disabled_options`]
    /// and [`Self::on_select`] then go unused, and [`Self::value`] gives the
    /// field's text. The menu keeps its own style and density, and
    /// [`Self::menu_max_height`] applies unless it set a height limit. Its
    /// width is the field's [`Self::width`] unless it set one; with
    /// [`Self::full_width`] the field's width always wins, over a width the
    /// menu set too. Activating an item closes it through
    /// [`Self::on_open_change`], as an option does.
    pub fn menu(mut self, menu: MaterialMenu) -> Self {
        self.menu = Some(menu);
        self
    }
    /// The field's text, and its accessible value, in place of the selected
    /// option's label: what a [`Self::menu`] has chosen.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }
    /// Whether options-menu configuration is set alongside a [`Self::menu`],
    /// which overrides it and leaves it inert.
    fn mixes_menu_and_options(&self) -> bool {
        self.menu.is_some()
            && (!self.options.is_empty()
                || self.selected.is_some()
                || !self.disabled_options.is_empty()
                || self.on_select.is_some()
                || self.style != MenuStyle::Baseline)
    }
    /// The text the field shows: [`Self::value`], else the selected option.
    fn value_text(&self) -> SharedString {
        self.value
            .clone()
            .or_else(|| self.selected.and_then(|ix| self.options.get(ix).cloned()))
            .unwrap_or_default()
    }
    /// Mark the choice as required: the label and accessible name gain the
    /// required mark. Enforcing it belongs to the form that owns the value.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
    /// Whether the field draws its label (the default). A dropdown that a list item's
    /// headline already names, beside it, draws none; the label still names it to
    /// assistive technology. With [`Self::required`] the required mark is then only in
    /// that accessible name, so the row that names the dropdown shows the mark itself.
    /// A [`TextFieldVariant::Hero`] dropdown draws no floating label anyway: it shows
    /// its label as the placeholder whether or not this is set.
    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }
    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }
    /// The field's corners.
    pub fn shape(mut self, shape: TextFieldShape) -> Self {
        self.shape = shape;
        self
    }
    /// The field's density (default 56dp; compact 48dp).
    pub fn density(mut self, density: TextFieldDensity) -> Self {
        self.density = density;
        self
    }
    /// The values the menu offers, one item each. A [`Self::menu`] overrides them.
    pub fn options<S: Into<SharedString>>(mut self, options: impl IntoIterator<Item = S>) -> Self {
        self.options = options.into_iter().map(Into::into).collect();
        self
    }
    /// Options (by index) shown disabled: they cannot be chosen. A [`Self::menu`]
    /// overrides them; its items carry their own disabled state.
    pub fn disabled_options(mut self, options: impl IntoIterator<Item = usize>) -> Self {
        self.disabled_options = options.into_iter().collect();
        self
    }
    /// Paint the field's error colors (with the supporting text as the
    /// error message).
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }
    /// The chosen option (by index), shown in the field and marked in the menu.
    /// A [`Self::menu`] overrides it; [`Self::value`] gives the field's text.
    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn leading_icon(mut self, icon: impl IntoElement) -> Self {
        self.leading = Some(icon.into_any_element());
        self
    }
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }
    /// The style of the menu built from [`Self::options`]. A [`Self::menu`]
    /// overrides it with its own style.
    pub fn menu_style(mut self, style: MenuStyle) -> Self {
        self.style = style;
        self
    }
    /// Field and menu width in design px (default 210).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    /// The tallest the menu grows, in design px; see [`MaterialMenu::max_height`].
    pub fn menu_max_height(mut self, height: f32) -> Self {
        self.menu_max_height = Some(height);
        self
    }
    /// Fill the parent's width instead of [`Self::width`], with the menu as
    /// wide as the field, like a text field in the same form.
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }
    pub fn on_open_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(handler));
        self
    }
    /// Called with an option's index when it is chosen. A [`Self::menu`]
    /// overrides it; its items carry their own click handlers.
    pub fn on_select(mut self, handler: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

#[cfg(feature = "inputs")]
impl RenderOnce for ExposedDropdown {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let focus = window
            .use_keyed_state((id.clone(), "dropdown-focus"), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            })
            .read(cx)
            .clone();
        let open = self.open && !self.disabled;
        let keyboard_focus = focus.is_focused(window) && window.last_input_was_keyboard();
        if self.mixes_menu_and_options() {
            static WARNED: std::sync::Once = std::sync::Once::new();
            WARNED.call_once(|| {
                log::warn!(
                    "an ExposedDropdown with a custom `.menu()` ignores `options`, `selected`, \
                     `disabled_options`, `on_select` and `menu_style`: set them on the menu's items"
                )
            });
        }
        let value = self.value_text();
        // Whether the field's value ended in an ellipsis when it was last drawn:
        // then its whole text is a tooltip on the field.
        let clipped = window
            .use_keyed_state((id.clone(), "dropdown-value-clipped"), cx, |_, _| {
                Rc::new(std::cell::Cell::new(false))
            })
            .read(cx)
            .clone();
        // The arrow turns over with the menu on the fast spatial token.
        let turn = motion::token_value(
            (id.clone(), "dropdown-arrow"),
            if open { 1. } else { 0. },
            MotionToken::FastSpatial,
            window,
            cx,
        );
        let arrow = Icon::from(
            Symbol::named("arrow_drop_down").expect("arrow_drop_down is a Material Symbol"),
        )
        .size(rem(24.))
        .rotate(std::f32::consts::PI * turn);
        let mut field = TextField::specimen((id.clone(), "field"), value.clone())
            .report_value_clipped(clipped.clone())
            .variant(self.variant)
            .shape(self.shape)
            .density(self.density)
            .when(self.show_label, |field| field.label(self.label.clone()))
            // A hero field draws no label, and a dropdown has no placeholder of its own.
            .when(self.variant == TextFieldVariant::Hero, |field| {
                field.placeholder(self.label.clone())
            })
            .required(self.required)
            .trailing_icon(arrow)
            .error(self.error)
            .disabled(self.disabled)
            .map(|field| {
                if self.full_width {
                    field.w_full()
                } else {
                    field.w(rem(self.width))
                }
            });
        if let Some(leading) = self.leading {
            field = field.leading_icon(leading);
        }
        if let Some(text) = self.supporting.clone() {
            field = field.supporting_text(text);
        }
        if open || keyboard_focus {
            field = field.preview_state(InteractionPreview::Focused);
        }
        let change = self.on_open_change.clone();
        let toggle = {
            let change = change.clone();
            move |next: bool, window: &mut Window, cx: &mut App| {
                if let Some(change) = change.as_ref() {
                    change(&next, window, cx);
                }
            }
        };
        let open_down = toggle.clone();
        let open_up = toggle.clone();
        let confirm = toggle.clone();
        let trigger = div()
            .id((id.clone(), "trigger"))
            .debug_selector(|| "dropdown-trigger".into())
            .when(self.full_width, |trigger| trigger.w_full())
            .role(Role::ComboBox)
            .aria_label(super::labeled::required_label(&self.label, self.required))
            .aria_value(value.clone())
            .aria_expanded(open)
            .key_context(DROPDOWN_CONTEXT)
            .when(!self.disabled, |this| this.track_focus(&focus))
            .when(!self.disabled, |this| this.cursor_pointer())
            .on_action(move |_: &SelectDown, window, cx| {
                if !open {
                    open_down(true, window, cx)
                }
            })
            .on_action(move |_: &SelectUp, window, cx| {
                if !open {
                    open_up(true, window, cx)
                }
            })
            .on_action(move |_: &Confirm, window, cx| confirm(!open, window, cx))
            .child(field);
        let menu = match self.menu {
            Some(menu) => menu,
            None => options_menu(
                MaterialMenu::new((id.clone(), "menu")).style(self.style),
                self.options,
                self.selected,
                &self.disabled_options,
                self.on_select,
            ),
        };
        // A caller's menu keeps a width or height limit it set itself.
        let menu = MaterialMenu {
            width: menu.width.or(Some(self.width)),
            max_height: menu.max_height.or(self.menu_max_height),
            ..menu
        };
        // Only while the value is cut short: a tooltip that repeated what the field
        // shows would only be noise, to the pointer and to assistive technology.
        // Not over the open menu, which lists the values itself. The tooltip is
        // the `feedback` feature's.
        #[cfg(feature = "feedback")]
        let trigger = if clipped.get() {
            super::MaterialTooltip::plain((id.clone(), "value-tip"), value)
                .map(|tip| if open { tip.force_visible(false) } else { tip })
                .trigger(trigger)
                .into_any_element()
        } else {
            trigger.into_any_element()
        };
        #[cfg(not(feature = "feedback"))]
        let trigger = trigger.into_any_element();
        if self.disabled {
            return trigger;
        }
        MenuPopup::new((id, "popup"), trigger, menu)
            .full_width(self.full_width)
            .open(open)
            .on_open_change(move |next, window, cx| toggle(*next, window, cx))
            .into_any_element()
    }
}

/// `menu` with an item for each of an [`ExposedDropdown`]'s options, each choosing itself.
#[cfg(feature = "inputs")]
fn options_menu(
    menu: MaterialMenu,
    options: Vec<SharedString>,
    selected: Option<usize>,
    disabled: &[usize],
    on_select: Option<Select>,
) -> MaterialMenu {
    menu.items(options.into_iter().enumerate().map(|(ix, option)| {
        let on_select = on_select.clone();
        MaterialMenuItem::new(option)
            .selected(selected == Some(ix))
            .disabled(disabled.contains(&ix))
            .on_click(move |_, window, cx| {
                if let Some(select) = on_select.as_ref() {
                    select(&ix, window, cx);
                }
            })
    }))
}

#[cfg(test)]
#[path = "menu_tests.rs"]
mod tests;
