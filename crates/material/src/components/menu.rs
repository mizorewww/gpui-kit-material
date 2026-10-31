//! M3 menus: the baseline menu, the M3 Expressive vertical menu (standard and
//! vibrant, with groups), a popup that opens a menu from a trigger, and the
//! exposed dropdown menu built from a text field and a menu.
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
use super::InteractionPreview;
use super::focus::{RingPlacement, RingedButton};
use super::roving::{Direction, target};
use super::text_field::{TextField, TextFieldDensity, TextFieldVariant, type_style};
use crate::{
    color::ColorRole,
    icons::{Icon, Symbol},
    motion::{self, MotionToken},
    theme::color,
    tokens::{
        elevation, state,
        typography::{TypeRole, rem},
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
const EXPRESSIVE_DIVIDER: f32 = 8.;
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
}

/// An M3 menu surface. Use [`MenuPopup`] to open it from a trigger.
#[derive(gpui::IntoElement)]
pub struct MaterialMenu {
    id: ElementId,
    style: MenuStyle,
    density: MenuDensity,
    groups: Vec<MenuGroup>,
    width: Option<f32>,
    elevated: bool,
    focus: Option<FocusHandle>,
    on_dismiss: Option<Notify>,
    on_activate: Option<Notify>,
    on_collapse: Option<Notify>,
    focus_item: Option<usize>,
}

impl MaterialMenu {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: MenuStyle::Standard,
            density: MenuDensity::Default,
            groups: Vec::new(),
            width: None,
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

    /// The bottom of the item at flat `index` (its row without a divider
    /// below it), in design px from the menu's top.
    pub fn item_bottom(&self, index: usize) -> Option<f32> {
        let item = self.groups.iter().flat_map(|g| &g.items).nth(index)?;
        let height = item_height(self.style, self.density, item.supporting.is_some(), false);
        Some(self.item_top(index)? + height)
    }

    /// The top of the item at flat `index` (its row, with an Expressive
    /// row's insets), in design px from the menu's top.
    pub fn item_top(&self, index: usize) -> Option<f32> {
        let expressive = self.style.expressive();
        let mut top = if expressive { 0. } else { BASELINE_PADDING };
        let mut flat = 0;
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
                if flat == index {
                    return Some(top);
                }
                top += item_height(
                    self.style,
                    self.density,
                    item.supporting.is_some(),
                    item.divider,
                );
                flat += 1;
            }
            if expressive {
                top += EXPRESSIVE_INSET_Y;
            }
        }
        None
    }
}

impl RenderOnce for MaterialMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = self.style;
        let density = self.density;
        let count: usize = self.groups.iter().map(|g| g.items.len()).sum();
        let state =
            window.use_keyed_state((self.id.clone(), "menu-focus"), cx, |_, cx| MenuFocus {
                root: cx.focus_handle(),
                items: Vec::new(),
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
        let collapse = Rc::new(move |window: &mut Window, cx: &mut App| {
            if let Some(collapse) = on_collapse.as_ref() {
                collapse(window, cx);
            } else {
                cx.propagate();
            }
        });
        let rtl = crate::scope::is_rtl(window);
        let expand = Rc::new(expand);
        let (expand_right, expand_left) = (expand.clone(), expand);
        let (collapse_right, collapse_left) = (collapse.clone(), collapse);
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
            move |direction: Direction, window: &mut Window, cx: &mut App| {
                let current = handles.iter().position(|h| h.is_focused(window));
                if let Some(ix) = target(&skip, current, direction) {
                    handles[ix].focus(window, cx);
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
                items.push(render_item(
                    // Scoped by the menu container's own id below, so a
                    // static name is unique without formatting per frame.
                    ("material-menu-item", index),
                    item,
                    style,
                    density,
                    handles.get(index).cloned(),
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
                    .bg(container)
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
            .on_action(move |_: &SelectRight, window, cx| {
                if rtl {
                    collapse_right(window, cx)
                } else {
                    expand_right(window, cx)
                }
            })
            .on_action(move |_: &SelectLeft, window, cx| {
                if rtl {
                    expand_left(window, cx)
                } else {
                    collapse_left(window, cx)
                }
            })
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
                    this.gap(rem(GROUP_GAP)).rounded(rem(MENU_RADIUS))
                } else {
                    this.py(rem(BASELINE_PADDING))
                        .rounded(rem(BASELINE_RADIUS))
                        .bg(container)
                }
            })
            .when(self.elevated, |this| this.shadow(elevation::level(2)))
            .children(groups)
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

fn render_item(
    id: impl Into<ElementId>,
    item: MaterialMenuItem,
    style: MenuStyle,
    density: MenuDensity,
    focus: Option<FocusHandle>,
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
                .child(
                    type_style(div(), label_role)
                        .text_color(fade(paint.label))
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .child(item.label),
                )
                .children(item.supporting.map(|text| {
                    type_style(div(), supporting_role)
                        .text_color(fade(paint.supporting))
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .child(text)
                })),
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
    } else if let Some(focus) = focus {
        button = button.track_focus(&focus);
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
        }
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
    /// (M3's "right start" and "left start" placements).
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }
}

/// The same anchor with its sides exchanged. Kit places popups physically, so
/// right-to-left layout asks for the mirrored corner.
pub(crate) fn mirrored_anchor(anchor: Anchor) -> Anchor {
    match anchor {
        Anchor::TopLeft => Anchor::TopRight,
        Anchor::TopRight => Anchor::TopLeft,
        Anchor::BottomLeft => Anchor::BottomRight,
        Anchor::BottomRight => Anchor::BottomLeft,
        Anchor::LeftCenter => Anchor::RightCenter,
        Anchor::RightCenter => Anchor::LeftCenter,
        other => other,
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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (reveal, fade) = open_motion(&self.id, self.open, window, cx);
        let visible = popup_visible(self.open, reveal, fade);
        let focus = window
            .use_keyed_state((self.id.clone(), "menu-popup-focus"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        // The trigger's height in design px, measured as it prepaints,
        // places a side popup's top at the trigger's top.
        let trigger_height = window
            .use_keyed_state((self.id.clone(), "menu-popup-trigger"), cx, |_, _| {
                Rc::new(std::cell::Cell::new(0f32))
            })
            .read(cx)
            .clone();
        let style = self.menu.style;
        let height = self.menu.natural_height();
        let anchor = self.anchor;
        let upward = opens_upward(anchor);
        let side = matches!(anchor, Anchor::LeftCenter | Anchor::RightCenter);
        let change = self.on_open_change.clone();
        let close = self.on_open_change.clone();
        let close_on_activate = self.close_on_activate;
        let close_menu: Notify = Rc::new(move |window: &mut Window, cx: &mut App| {
            if let Some(close) = close.as_ref() {
                close(&false, window, cx);
            }
        });
        let submenu_index = self.submenu.as_ref().map(|(index, _)| *index);
        let submenu_top = submenu_index.and_then(|index| self.menu.item_top(index));
        let submenu_bottom = submenu_index.and_then(|index| self.menu.item_bottom(index));
        // The reserved cascade column's height below the trigger.
        let cascade_height = self.submenu.as_ref().map_or(0., |(_, submenu)| {
            let first_row = if submenu.style.expressive() {
                EXPRESSIVE_INSET_Y
            } else {
                BASELINE_PADDING
            };
            (submenu_top.unwrap_or(0.) - first_row).max(0.) + submenu.natural_height()
        });
        // Closing the sub-menu by keyboard returns focus to its owner item.
        let restore = window
            .use_keyed_state((self.id.clone(), "menu-submenu-restore"), cx, |_, _| {
                Rc::new(std::cell::Cell::new(false))
            })
            .read(cx)
            .clone();
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
            let sub_height = submenu.natural_height();
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
            let first_row = if sub_style.expressive() {
                EXPRESSIVE_INSET_Y
            } else {
                BASELINE_PADDING
            };
            let visible = popup_visible(open, sub_reveal, sub_fade);
            // The column is reserved while closed, so opening the sub-menu
            // never moves the menu Kit anchored. Below the trigger the
            // sub-menu's first item lines up with its owner and it extends
            // down; above the trigger its last item lines up with the
            // owner's bottom and it extends up, within the popup's box.
            let column = div().flex_shrink_0().flex().flex_col();
            let column = if upward {
                let under = (height - submenu_bottom.unwrap_or(height) - first_row).max(0.);
                column.h(rem(sub_height + under)).pb(rem(under))
            } else {
                let top = (submenu_top.unwrap_or(0.) - first_row).max(0.);
                column.h(rem(top + sub_height)).pt(rem(top))
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
            ((height * shown).max(cascade_height) - trigger_height.get()).max(0.)
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
        let measure = trigger_height;
        // Material buttons block the mouse for the elements under them, so
        // Kit's popover never sees a press on such a trigger. A window
        // listener over the trigger's bounds toggles the menu instead (a
        // press while open closes it before the popover's outside-press
        // dismissal, which then has nothing to do).
        let toggle = self.on_open_change.clone();
        let press = canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let toggle = toggle.clone();
                window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                    if phase == DispatchPhase::Capture
                        && event.button == MouseButton::Left
                        && bounds.contains(&event.position)
                        && let Some(toggle) = toggle.as_ref()
                    {
                        toggle(&!open, window, cx);
                    }
                });
            },
        )
        .absolute()
        .size_full();
        let trigger = div()
            .relative()
            .child(press)
            .on_children_prepainted(move |bounds, window, _| {
                if let Some(bounds) = bounds.first() {
                    measure.set(
                        f32::from(bounds.size.height) * crate::tokens::typography::BASE_PX
                            / f32::from(window.rem_size()),
                    );
                }
            })
            .child(self.trigger);
        Popover::new(self.id.clone())
            .appearance(false)
            .anchor(if crate::scope::is_rtl(window) {
                mirrored_anchor(anchor)
            } else {
                anchor
            })
            .offset(rem(4.).to_pixels(window.rem_size()))
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

/// The M3 exposed dropdown menu: a text field that opens a menu of options.
#[derive(gpui::IntoElement)]
pub struct ExposedDropdown {
    id: ElementId,
    variant: TextFieldVariant,
    density: TextFieldDensity,
    label: SharedString,
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
    on_open_change: Option<OpenChange>,
    on_select: Option<Select>,
}

impl ExposedDropdown {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            variant: TextFieldVariant::Filled,
            density: TextFieldDensity::Default,
            label: label.into(),
            options: Vec::new(),
            disabled_options: Vec::new(),
            selected: None,
            open: false,
            disabled: false,
            error: false,
            required: false,
            leading: None,
            supporting: None,
            style: MenuStyle::Standard,
            width: 210.,
            on_open_change: None,
            on_select: None,
        }
    }
    /// Mark the choice as required: the label and accessible name gain the
    /// required mark. Enforcing it belongs to the form that owns the value.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }
    /// The field's density (default 56dp; compact 48dp).
    pub fn density(mut self, density: TextFieldDensity) -> Self {
        self.density = density;
        self
    }
    pub fn options<S: Into<SharedString>>(mut self, options: impl IntoIterator<Item = S>) -> Self {
        self.options = options.into_iter().map(Into::into).collect();
        self
    }
    /// Options (by index) shown disabled: they cannot be chosen.
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
    pub fn menu_style(mut self, style: MenuStyle) -> Self {
        self.style = style;
        self
    }
    /// Field and menu width in design px (default 210).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    pub fn on_open_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(handler));
        self
    }
    pub fn on_select(mut self, handler: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

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
        let value = self
            .selected
            .and_then(|ix| self.options.get(ix).cloned())
            .unwrap_or_default();
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
            .variant(self.variant)
            .density(self.density)
            .label(self.label.clone())
            .required(self.required)
            .trailing_icon(arrow)
            .error(self.error)
            .disabled(self.disabled)
            .w(rem(self.width));
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
            .role(Role::ComboBox)
            .aria_label(super::labeled::required_label(&self.label, self.required))
            .aria_value(value)
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
        let selected = self.selected;
        let on_select = self.on_select.clone();
        let disabled_options = self.disabled_options;
        let menu = MaterialMenu::new((id.clone(), "menu"))
            .style(self.style)
            .width(self.width)
            .items(self.options.into_iter().enumerate().map(|(ix, option)| {
                let on_select = on_select.clone();
                MaterialMenuItem::new(option)
                    .selected(selected == Some(ix))
                    .disabled(disabled_options.contains(&ix))
                    .on_click(move |_, window, cx| {
                        if let Some(select) = on_select.as_ref() {
                            select(&ix, window, cx);
                        }
                    })
            }));
        if self.disabled {
            return trigger.into_any_element();
        }
        MenuPopup::new((id, "popup"), trigger, menu)
            .open(open)
            .on_open_change(move |next, window, cx| toggle(*next, window, cx))
            .into_any_element()
    }
}

#[cfg(test)]
#[path = "menu_tests.rs"]
mod tests;
