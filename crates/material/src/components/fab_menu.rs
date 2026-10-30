//! M3 Expressive FAB menu: a FAB that opens a stack of related actions.
//!
//! Opening morphs the FAB into the round close button (container → solid
//! color, 16dp → full corners, icon turning into the close icon) while the
//! items, end-aligned above it, expand from the FAB's size into labelled
//! pills, nearest first, on the default spatial spring. Closing reverses
//! the stagger from the farthest item. Under reduced motion every change
//! finishes at once.
//!
//! The toggle and items are [`MaterialButton`]s, so activation, focus,
//! state layers, ripples and focus rings are the button's. The caller owns
//! `open`; Escape inside the menu requests closing.
//!
//! [`FabMenuPresentation::Menu`] presents the same items as a baseline
//! [`MaterialMenu`] opening from the FAB in Kit's popover (the menu the web
//! uses); `Auto` picks the expressive list in compact windows (under
//! 600dp) and the menu in wider ones.
use super::{
    FabColor, FabSize, InteractionPreview, MaterialButton, MaterialMenu, MaterialMenuItem,
    MenuPopup, MenuStyle,
    button::{Form, Palette, Roles},
    fab::{FAB_ELEVATION, revealed_label},
};
use crate::{
    icons::{Icon, IconName},
    motion::{self, MotionToken},
    tokens::typography::{BASE_PX, TypeRole, rem},
};
use gpui::{prelude::*, *};
use std::{rc::Rc, time::Duration};

/// The FAB menu color styles: the closed FAB and the items use the
/// container color, the close button the solid color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FabMenuColor {
    #[default]
    Primary,
    Secondary,
    Tertiary,
}

impl FabMenuColor {
    pub const ALL: [Self; 3] = [Self::Primary, Self::Secondary, Self::Tertiary];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Tertiary => "tertiary",
        }
    }
    /// (container style, solid style).
    const fn styles(self) -> (FabColor, FabColor) {
        match self {
            Self::Primary => (FabColor::PrimaryContainer, FabColor::Primary),
            Self::Secondary => (FabColor::SecondaryContainer, FabColor::Secondary),
            Self::Tertiary => (FabColor::TertiaryContainer, FabColor::Tertiary),
        }
    }
    fn item_palette(self) -> Palette {
        let (container, content) = self.styles().0.roles();
        Palette {
            rest: Roles {
                container: Some(container),
                content,
            },
            selected: None,
            disabled_container: true,
            elevation: (0, 0),
        }
    }
    fn toggle_palette(self, open: bool) -> Palette {
        let (closed, opened) = self.styles();
        if open { opened } else { closed }.palette(FAB_ELEVATION)
    }
}

/// How a FAB menu presents its items.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FabMenuPresentation {
    /// The M3 Expressive list of pills stacked above the FAB.
    #[default]
    List,
    /// A baseline menu opening from the FAB.
    Menu,
    /// The list in compact windows, the menu from 600dp up.
    Auto,
}

impl FabMenuPresentation {
    pub const ALL: [Self; 3] = [Self::List, Self::Menu, Self::Auto];
    /// The presentation used in a window `width` design px wide.
    pub fn resolve(self, width: f32) -> Self {
        match self {
            Self::Auto if width < COMPACT_WIDTH => Self::List,
            Self::Auto => Self::Menu,
            other => other,
        }
    }
}

/// The compact window class's upper bound, in dp.
const COMPACT_WIDTH: f32 = 600.;

/// Which side the items align to: the FAB's end (a bottom-end FAB) or
/// start (a bottom-start FAB).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FabMenuAlign {
    #[default]
    End,
    Start,
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type ToggleHandler = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// One action of a FAB menu. Rendered on its own (e.g. a building-block
/// specimen) it is the open item in its `color`; inside a menu it takes the
/// menu's color.
#[derive(gpui::IntoElement)]
pub struct FabMenuItem {
    id: ElementId,
    icon: Option<Icon>,
    label: SharedString,
    color: FabMenuColor,
    preview: Option<InteractionPreview>,
    on_click: Option<ClickHandler>,
}

impl FabMenuItem {
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl Into<Icon>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            icon: Some(icon.into()),
            ..Self::label_only(id, label)
        }
    }
    /// An item showing only its label.
    pub fn label_only(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            label: label.into(),
            color: FabMenuColor::Primary,
            preview: None,
            on_click: None,
        }
    }
    /// The color of a stand-alone item (a menu's items take its color).
    pub fn color(mut self, color: FabMenuColor) -> Self {
        self.color = color;
        self
    }
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
    pub fn on_click(
        mut self,
        callback: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(callback));
        self
    }
}

impl FabMenuItem {
    /// This item as a baseline menu entry.
    fn menu_item(self) -> MaterialMenuItem {
        let mut item = MaterialMenuItem::new(self.label);
        if let Some(icon) = self.icon {
            item = item.leading(icon.size(rem(24.)));
        }
        if let Some(callback) = self.on_click {
            item = item.on_click(move |event, window, cx| callback(event, window, cx));
        }
        item
    }
    /// The item at a 0 … 1 `expansion` from FAB size to labelled pill;
    /// only an `open` menu's items can be activated.
    fn button(self, expansion: f32, open: bool, window: &Window) -> MaterialButton {
        let label = revealed_label(&self.label, TypeRole::TitleMedium, expansion, window);
        let mut button = MaterialButton::new(self.id)
            .form(Form::FabMenuItem { expansion })
            .palette(self.color.item_palette())
            .when_some(self.icon, |button, icon| button.icon(icon))
            .accessibility_label(self.label)
            .children(label)
            .opacity(expansion.clamp(0., 1.));
        if let Some(state) = self.preview {
            button = button.preview_state(state);
        }
        if let Some(callback) = self.on_click.filter(|_| open) {
            button = button.on_click(move |event, window, cx| callback(event, window, cx));
        }
        button
    }
}

impl RenderOnce for FabMenuItem {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        self.button(1., true, window)
    }
}

#[derive(gpui::IntoElement)]
pub struct FabMenu {
    id: ElementId,
    icon: Icon,
    close_icon: Icon,
    label: SharedString,
    color: FabMenuColor,
    size: FabSize,
    presentation: FabMenuPresentation,
    align: FabMenuAlign,
    open: bool,
    items: Vec<FabMenuItem>,
    preview: Option<InteractionPreview>,
    on_toggle: Option<ToggleHandler>,
}

/// Delay between neighbouring items' entrances and exits.
const STAGGER: Duration = Duration::from_millis(30);
/// Space between the items and the button, and between items.
const BUTTON_GAP: f32 = 8.;
const ITEM_GAP: f32 = 4.;

impl FabMenu {
    /// `icon` is the closed FAB's; `label` names the toggle for assistive
    /// technology.
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl Into<Icon>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            close_icon: IconName::Close.into(),
            label: label.into(),
            color: FabMenuColor::Primary,
            size: FabSize::Regular,
            presentation: FabMenuPresentation::List,
            align: FabMenuAlign::End,
            open: false,
            items: Vec::new(),
            preview: None,
            on_toggle: None,
        }
    }
    pub fn close_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.close_icon = icon.into();
        self
    }
    pub fn color(mut self, color: FabMenuColor) -> Self {
        self.color = color;
        self
    }
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    /// The FAB's size (regular by default); open, it is the round close
    /// button of the same size.
    pub fn size(mut self, size: FabSize) -> Self {
        self.size = size;
        self
    }
    pub fn presentation(mut self, presentation: FabMenuPresentation) -> Self {
        self.presentation = presentation;
        self
    }
    /// Align the items to the FAB's end (default) or start edge.
    pub fn align(mut self, align: FabMenuAlign) -> Self {
        self.align = align;
        self
    }
    pub fn item(mut self, item: FabMenuItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = FabMenuItem>) -> Self {
        self.items.extend(items);
        self
    }
    /// Render the toggle as a static specimen in one interaction state.
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
    /// The requested open state (toggle activation, Escape).
    pub fn on_toggle(mut self, callback: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(callback));
        self
    }
}

/// The entrance (exit) delay of item `index` of `count`, counted from the
/// top: opening starts nearest the FAB, closing farthest from it.
fn stagger(index: usize, count: usize, open: bool) -> Duration {
    let steps = if open { count - 1 - index } else { index };
    STAGGER * steps as u32
}

/// The toggle icon for a 0 (FAB icon) … 1 (close icon) morph amount: the
/// outgoing icon turns a quarter away as the incoming one turns in.
fn toggle_icon(icon: Icon, close: Icon, morph: f32) -> Icon {
    let quarter = std::f32::consts::FRAC_PI_2;
    if morph >= 0.5 {
        close.rotate(-(1. - morph) * quarter)
    } else {
        icon.rotate(morph * quarter)
    }
}

impl FabMenu {
    /// The baseline presentation: the FAB opens a menu of the items in
    /// Kit's popover, above the FAB and aligned to its `align` edge.
    fn render_menu(self) -> AnyElement {
        let open = self.open;
        let mut toggle = MaterialButton::new((self.id.clone(), "fab-menu-toggle"))
            .form(Form::FabMenuToggle {
                open: false,
                size: self.size,
            })
            .palette(self.color.toggle_palette(false))
            .icon(self.icon)
            .accessibility_label(self.label)
            .aria_expanded(open);
        if let Some(state) = self.preview {
            toggle = toggle.preview_state(state);
        }
        if let Some(callback) = self.on_toggle.clone() {
            // The FAB (a blocking Base button) takes pointer and key
            // activation itself; the popover's trigger toggle could only
            // request the same `!open`.
            toggle = toggle.on_click(move |_, window, cx| callback(&!open, window, cx));
        }
        let menu = MaterialMenu::new((self.id.clone(), "fab-menu-list"))
            .style(MenuStyle::Baseline)
            .items(self.items.into_iter().map(FabMenuItem::menu_item));
        let mut popup = MenuPopup::new(self.id.clone(), toggle, menu)
            .open(open)
            .anchor(match self.align {
                FabMenuAlign::End => Anchor::BottomRight,
                FabMenuAlign::Start => Anchor::BottomLeft,
            });
        if let Some(callback) = self.on_toggle {
            popup = popup.on_open_change(move |open, window, cx| callback(open, window, cx));
        }
        popup.into_any_element()
    }
}

impl RenderOnce for FabMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let width =
            f32::from(window.viewport_size().width) * BASE_PX / f32::from(window.rem_size());
        if self.presentation.resolve(width) == FabMenuPresentation::Menu {
            return self.render_menu();
        }
        let open = self.open;
        let count = self.items.len();
        let curve = motion::curve(MotionToken::DefaultSpatial);
        let start = self.align == FabMenuAlign::Start;
        let mut column = div()
            .flex()
            .flex_col()
            .map(|column| {
                if start {
                    column.items_start()
                } else {
                    column.items_end()
                }
            })
            .gap(rem(ITEM_GAP));
        let mut any_item = false;
        for (index, item) in self.items.into_iter().enumerate() {
            // Render runs in the parent's id scope, so key each item's
            // expansion under the menu's own id: two menus with the same
            // item ids must not share a transition.
            let expansion = window.with_element_namespace(self.id.clone(), |window| {
                gpui_base::transition(
                    ElementId::from((item.id.clone(), "fab-menu-expansion")),
                    if open { 1f32 } else { 0. },
                    gpui_base::Transition::new(curve.duration())
                        .ease(curve.ease())
                        .delay(stagger(index, count, open)),
                    window,
                    cx,
                )
            });
            // A settled closed menu lays out no items.
            if !open && expansion <= 0. {
                continue;
            }
            any_item = true;
            column = column.child(
                item.color(self.color)
                    .button(expansion, open, window)
                    .tab_stop(open),
            );
        }
        let morph = motion::token_value(
            (self.id.clone(), "fab-menu-icon"),
            if open { 1. } else { 0. },
            MotionToken::FastSpatial,
            window,
            cx,
        );
        let mut toggle = MaterialButton::new((self.id.clone(), "fab-menu-toggle"))
            .form(Form::FabMenuToggle {
                open,
                size: self.size,
            })
            .palette(self.color.toggle_palette(open))
            .icon(toggle_icon(self.icon, self.close_icon, morph))
            .accessibility_label(self.label)
            .aria_expanded(open);
        if let Some(state) = self.preview {
            toggle = toggle.preview_state(state);
        }
        let on_toggle = self.on_toggle;
        if let Some(callback) = on_toggle.clone() {
            toggle = toggle.on_click(move |_, window, cx| callback(&!open, window, cx));
        }
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .map(|menu| {
                if start {
                    menu.items_start()
                } else {
                    menu.items_end()
                }
            })
            .gap(rem(BUTTON_GAP))
            .when_some(on_toggle.filter(|_| open), |menu, callback| {
                menu.on_key_down(move |event, window, cx| {
                    if event.keystroke.key == "escape" {
                        cx.stop_propagation();
                        callback(&false, window, cx);
                    }
                })
            })
            .when(any_item, |menu| menu.child(column))
            .child(toggle)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{FabMenu, FabMenuColor, FabMenuItem, stagger};
    use crate::{color::ColorRole, icons::IconName, motion};
    use gpui::{
        Context, InteractiveElement as _, IntoElement, ParentElement as _, Render, Styled as _,
        TestAppContext, VisualTestContext, Window, div, prelude::FluentBuilder as _, px,
    };
    use std::time::Duration;

    #[core::prelude::v1::test]
    fn items_stagger_from_the_fab_and_close_from_the_top() {
        assert_eq!(stagger(2, 3, true), Duration::ZERO, "nearest opens first");
        assert_eq!(stagger(0, 3, true), Duration::from_millis(60));
        assert_eq!(
            stagger(0, 3, false),
            Duration::ZERO,
            "farthest closes first"
        );
        assert_eq!(stagger(2, 3, false), Duration::from_millis(60));
    }

    #[core::prelude::v1::test]
    fn the_close_button_takes_the_solid_color() {
        let closed = FabMenuColor::Tertiary.toggle_palette(false);
        let open = FabMenuColor::Tertiary.toggle_palette(true);
        assert_eq!(closed.rest.container, Some(ColorRole::TERTIARY_CONTAINER));
        assert_eq!(open.rest.container, Some(ColorRole::TERTIARY));
        assert_eq!(
            FabMenuColor::Tertiary.item_palette().rest.container,
            Some(ColorRole::TERTIARY_CONTAINER)
        );
    }

    struct Harness {
        open: bool,
        twin: bool,
        clicks: usize,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            let click = cx.listener(|this, _: &gpui::ClickEvent, _, cx| {
                this.clicks += 1;
                cx.notify();
            });
            let click = std::rc::Rc::new(click);
            div()
                .flex()
                .items_start()
                .p(px(40.))
                .child(
                    div().flex().debug_selector(|| "menu".into()).child(
                        FabMenu::new("menu", IconName::Check, "Actions")
                            .open(self.open)
                            .items((0..3usize).map(|index| {
                                let click = click.clone();
                                FabMenuItem::new(("item", index), IconName::Check, "Action")
                                    .on_click(move |event, window, cx| click(event, window, cx))
                            })),
                    ),
                )
                .when(self.twin, |row| {
                    // A closed sibling menu whose items reuse the same ids.
                    row.child(div().flex().debug_selector(|| "twin".into()).child(
                        FabMenu::new("twin", IconName::Check, "Actions").items((0..3usize).map(
                            |index| FabMenuItem::new(("item", index), IconName::Check, "Action"),
                        )),
                    ))
                })
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }

    #[gpui::test]
    fn opening_grows_the_stack_and_closing_removes_it(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(false);
            motion::set_motion_scheme(motion::MotionScheme::Expressive);
        });
        let (view, cx) = cx.add_window_view(|_, _| Harness {
            open: false,
            twin: false,
            clicks: 0,
        });
        draw(cx);
        let closed = cx.debug_bounds("menu").unwrap().size;
        assert_eq!(closed.height, px(56.));
        view.update(cx, |view, cx| {
            view.open = true;
            cx.notify();
        });
        draw(cx);
        cx.executor().advance_clock(Duration::from_secs(2));
        draw(cx);
        draw(cx);
        let open = cx.debug_bounds("menu").unwrap().size;
        assert_eq!(open.height, px(56. * 4. + 8. + 4. * 2.));
        assert!(open.width > px(56.));
        view.update(cx, |view, cx| {
            view.open = false;
            cx.notify();
        });
        draw(cx);
        cx.executor().advance_clock(Duration::from_secs(2));
        draw(cx);
        draw(cx);
        assert_eq!(cx.debug_bounds("menu").unwrap().size.height, px(56.));
    }

    #[gpui::test]
    fn sibling_menus_with_equal_item_ids_open_independently(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(false);
            motion::set_motion_scheme(motion::MotionScheme::Expressive);
        });
        let (view, cx) = cx.add_window_view(|_, _| Harness {
            open: false,
            twin: true,
            clicks: 0,
        });
        draw(cx);
        view.update(cx, |view, cx| {
            view.open = true;
            cx.notify();
        });
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(60));
        draw(cx);
        // A transition shared with the closed twin would be recreated at its
        // target every frame and jump open instead of growing.
        let growing = cx.debug_bounds("menu").unwrap().size.width;
        cx.executor().advance_clock(Duration::from_secs(2));
        draw(cx);
        draw(cx);
        let open = cx.debug_bounds("menu").unwrap().size.width;
        assert!(
            growing > px(56.) && growing < open,
            "{growing:?} of {open:?}"
        );
        assert_eq!(cx.debug_bounds("twin").unwrap().size.height, px(56.));
    }

    #[gpui::test]
    fn a_closing_menu_ignores_clicks_on_its_fading_items(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(false);
        });
        let (view, cx) = cx.add_window_view(|_, _| Harness {
            open: true,
            twin: false,
            clicks: 0,
        });
        draw(cx);
        cx.executor().advance_clock(Duration::from_secs(2));
        draw(cx);
        draw(cx);
        let menu = cx.debug_bounds("menu").unwrap();
        let first = gpui::point(menu.right() - px(28.), menu.top() + px(28.));
        cx.simulate_click(first, gpui::Modifiers::default());
        draw(cx);
        view.read_with(cx, |view, _| {
            assert_eq!(view.clicks, 1, "an open item clicks")
        });
        view.update(cx, |view, cx| {
            view.open = false;
            cx.notify();
        });
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(20));
        draw(cx);
        cx.simulate_click(first, gpui::Modifiers::default());
        draw(cx);
        view.read_with(cx, |view, _| assert_eq!(view.clicks, 1));
    }
}

#[cfg(test)]
mod presentation_tests {
    use super::{FabMenu, FabMenuAlign, FabMenuItem, FabMenuPresentation};
    use crate::{components::FabSize, icons::IconName};
    use gpui::{
        Context, InteractiveElement as _, IntoElement, Modifiers, ParentElement as _, Render,
        Styled as _, TestAppContext, VisualTestContext, Window, div, prelude::FluentBuilder as _,
        px, size,
    };
    use std::{cell::RefCell, rc::Rc};

    #[core::prelude::v1::test]
    fn auto_lists_in_compact_windows_and_menus_in_wider_ones() {
        let auto = FabMenuPresentation::Auto;
        assert_eq!(auto.resolve(412.), FabMenuPresentation::List);
        assert_eq!(auto.resolve(840.), FabMenuPresentation::Menu);
        assert_eq!(
            FabMenuPresentation::List.resolve(1200.),
            FabMenuPresentation::List
        );
        assert_eq!(
            FabMenuPresentation::Menu.resolve(300.),
            FabMenuPresentation::Menu
        );
    }

    struct Harness {
        open: bool,
        presentation: FabMenuPresentation,
        align: FabMenuAlign,
        size: FabSize,
        picked: Rc<RefCell<Vec<usize>>>,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div()
                .size_full()
                .flex()
                .items_end()
                .map(|row| match self.align {
                    FabMenuAlign::End => row.justify_end(),
                    FabMenuAlign::Start => row.justify_start(),
                })
                .p(px(40.))
                .child(
                    div().flex().debug_selector(|| "menu".into()).child(
                        FabMenu::new("menu", IconName::Check, "Actions")
                            .presentation(self.presentation)
                            .align(self.align)
                            .size(self.size)
                            .open(self.open)
                            .on_toggle(cx.listener(|this, open: &bool, _, cx| {
                                this.open = *open;
                                cx.notify();
                            }))
                            .items((0..3usize).map(|index| {
                                let picked = self.picked.clone();
                                let item = if index == 0 {
                                    FabMenuItem::label_only(("item", index), "First action")
                                } else {
                                    FabMenuItem::new(("item", index), IconName::Check, "Action")
                                };
                                item.on_click(move |_, _, _| picked.borrow_mut().push(index))
                            })),
                    ),
                )
        }
    }

    fn settle(cx: &mut VisualTestContext) {
        for _ in 0..3 {
            cx.executor()
                .advance_clock(std::time::Duration::from_millis(800));
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        presentation: FabMenuPresentation,
        align: FabMenuAlign,
        size: FabSize,
    ) -> (gpui::Entity<Harness>, &mut VisualTestContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            crate::init(cx);
            cx.set_reduce_motion(true);
        });
        cx.add_window_view(move |_, _| Harness {
            open: false,
            presentation,
            align,
            size,
            picked: Rc::default(),
        })
    }

    #[gpui::test]
    fn the_menu_presentation_opens_a_baseline_menu_whose_items_act(cx: &mut TestAppContext) {
        let (view, cx) = setup(
            cx,
            FabMenuPresentation::Menu,
            FabMenuAlign::End,
            FabSize::Regular,
        );
        settle(cx);
        let fab = cx.debug_bounds("menu").unwrap();
        assert_eq!(fab.size, size(px(56.), px(56.)));
        cx.simulate_click(fab.center(), Modifiers::default());
        settle(cx);
        cx.update(|_, cx| assert!(view.read(cx).open));
        let surface = cx.debug_bounds("menu-popup-surface").unwrap();
        assert!(
            surface.bottom() <= fab.top(),
            "the menu opens above the FAB"
        );
        assert_eq!(surface.right(), fab.right(), "end-aligned");
        assert_eq!(
            cx.debug_bounds("menu").unwrap().size,
            fab.size,
            "the FAB stays"
        );
        cx.simulate_keystrokes("down down enter");
        settle(cx);
        cx.update(|_, cx| {
            assert_eq!(*view.read(cx).picked.borrow(), vec![1]);
            assert!(!view.read(cx).open, "an item closes the menu");
        });
    }

    #[gpui::test]
    fn a_start_aligned_list_grows_from_a_sized_fab(cx: &mut TestAppContext) {
        let (view, cx) = setup(
            cx,
            FabMenuPresentation::List,
            FabMenuAlign::Start,
            FabSize::Large,
        );
        settle(cx);
        let closed = cx.debug_bounds("menu").unwrap();
        assert_eq!(closed.size, size(px(96.), px(96.)));
        view.update(cx, |view, cx| {
            view.open = true;
            cx.notify();
        });
        settle(cx);
        let open = cx.debug_bounds("menu").unwrap();
        assert_eq!(open.left(), closed.left(), "items align to the start");
        assert_eq!(open.size.height, px(96. + 8. + 56. * 3. + 4. * 2.));
        // The label-only first item sits at the top, start-aligned.
        let first = gpui::point(open.left() + px(28.), open.top() + px(28.));
        cx.simulate_click(first, Modifiers::default());
        settle(cx);
        cx.update(|_, cx| assert_eq!(*view.read(cx).picked.borrow(), vec![0]));
    }
}
