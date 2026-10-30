//! Navigation suite: one set of destinations shown as the bar, rail or drawer
//! the window's size calls for, beside or above the content.
//!
//! Source: only m3e lists this component (the Figma kit and md3.io do not), so
//! its rules are m3e's port of Compose `NavigationSuiteScaffold`
//! (`packages/m3e-motion/src/window-size-class.ts`, `navigation-suite.ts`,
//! `packages/m3e-css/src/navigation-suite.css`,
//! `navigation-suite-defaults.css`, from androidx
//! `19991f2c0858aadfcb442ce02b2c8e4743c96898`). The bar, rail and drawer keep
//! their own Figma-sourced values. Showing and hiding plays the default spatial
//! token, the same as the rail and drawer it moves: m3e's fixed 0.9 / 700 spring
//! is that token under the Standard scheme.
//!
//! One deliberate difference from m3e: [`NavigationSuiteType::Drawer`] is the
//! default expanded form, so the suite moves between bar, rail and drawer. m3e
//! defaults to the expanded rail; [`NavigationSuite::expanded_type`] selects it.
use super::adaptive::{WindowHeightClass, WindowSizeClass, WindowWidthClass};
use super::navigation::{
    NavItem, NavItemLayout, NavigationBar, NavigationRail, RAIL_COLLAPSED_WIDTH,
    RAIL_EXPANDED_WIDTH,
};
use super::navigation_drawer::{DRAWER_WIDTH, NavigationDrawer};
use crate::{
    color::ColorRole,
    motion::{self, MotionToken},
    theme::color,
    tokens::typography::rem,
};
use gpui::{prelude::*, *};
use std::rc::Rc;

/// The primary action's inset above a bar (Compose
/// `PrimaryActionContentPadding`, through m3e), in dp.
pub const SUITE_PRIMARY_ACTION_PADDING: f32 = 16.;
/// The drawer's inset for a primary action, matching its item indicators.
const DRAWER_ACTION_PADDING: f32 = 12.;
/// Which navigation a suite shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NavigationSuiteType {
    /// A bar with the icon over the label.
    BarCompact,
    /// A bar with the icon beside the label, for short, wide windows.
    BarMedium,
    /// A collapsed rail.
    RailCollapsed,
    /// An expanded rail.
    RailExpanded,
    /// A standard drawer.
    Drawer,
    /// No navigation: the content alone.
    None,
}

impl NavigationSuiteType {
    /// Whether the navigation sits below the content.
    pub const fn is_bar(self) -> bool {
        matches!(self, Self::BarCompact | Self::BarMedium)
    }
}

/// The navigation for a size class: a bar below 600dp, or when the window is
/// under 480dp tall; a collapsed rail from 600dp; `expanded_type` from
/// `expanded_from`. `None` for `expanded_from` never expands.
///
/// ```
/// use gpui_kit_material::prelude::*;
///
/// let form = |width| {
///     navigation_suite_type(
///         WindowSizeClass::of(width, 800.),
///         Some(WindowWidthClass::Large),
///         NavigationSuiteType::Drawer,
///     )
/// };
/// assert_eq!(form(360.), NavigationSuiteType::BarCompact);
/// assert_eq!(form(800.), NavigationSuiteType::RailCollapsed);
/// assert_eq!(form(1300.), NavigationSuiteType::Drawer);
/// ```
pub fn navigation_suite_type(
    size: WindowSizeClass,
    expanded_from: Option<WindowWidthClass>,
    expanded_type: NavigationSuiteType,
) -> NavigationSuiteType {
    if size.width == WindowWidthClass::Compact {
        NavigationSuiteType::BarCompact
    } else if size.height == WindowHeightClass::Compact {
        NavigationSuiteType::BarMedium
    } else if expanded_from.is_some_and(|from| size.width >= from) {
        expanded_type
    } else {
        NavigationSuiteType::RailCollapsed
    }
}

type Change = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;
type TypeChange = Rc<dyn Fn(&NavigationSuiteType, &mut Window, &mut App)>;

/// Adaptive navigation around caller-owned content.
///
/// The selection is controlled: report it through [`Self::on_change`] and pass
/// it back with [`Self::selected`]. It survives a change of navigation, and so
/// does keyboard focus, because all three forms share this element's id and
/// therefore their focus handles.
///
/// ```
/// use gpui_kit_material::prelude::*;
///
/// let suite = NavigationSuite::new("suite", "Main")
///     .items([
///         NavItem::new("home", "Home", NavIcon::new("home", "home")),
///         NavItem::new("mail", "Mail", NavIcon::new("mail", "mail")),
///     ])
///     .selected("home")
///     .expanded_type(NavigationSuiteType::RailExpanded);
/// # let _ = suite;
/// ```
#[derive(gpui::IntoElement)]
pub struct NavigationSuite {
    id: ElementId,
    label: SharedString,
    items: Vec<NavItem>,
    selected: Option<SharedString>,
    layout_type: Option<NavigationSuiteType>,
    size_class: Option<WindowSizeClass>,
    expanded_from: Option<WindowWidthClass>,
    expanded_type: NavigationSuiteType,
    hidden: bool,
    primary_action: Option<AnyElement>,
    content: Vec<AnyElement>,
    on_change: Option<Change>,
    on_type_change: Option<TypeChange>,
}

impl NavigationSuite {
    /// `label` is the navigation landmark's accessible name.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            items: Vec::new(),
            selected: None,
            layout_type: None,
            size_class: None,
            expanded_from: Some(WindowWidthClass::Large),
            expanded_type: NavigationSuiteType::Drawer,
            hidden: false,
            primary_action: None,
            content: Vec::new(),
            on_change: None,
            on_type_change: None,
        }
    }

    /// The destinations, shown by every form.
    pub fn items(mut self, items: impl IntoIterator<Item = NavItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    /// The selected destination's value.
    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.selected = Some(value.into());
        self
    }

    /// Fix the navigation, overriding the adaptive rule. `None` is adaptive.
    pub fn layout_type(mut self, layout_type: Option<NavigationSuiteType>) -> Self {
        self.layout_type = layout_type;
        self
    }

    /// Classify this size instead of the window's: for a suite framed inside a
    /// larger window, as previews are. `None` reads the window.
    pub fn size_class(mut self, size_class: Option<WindowSizeClass>) -> Self {
        self.size_class = size_class;
        self
    }

    /// The width class the navigation expands at (large by default). `None`
    /// never expands.
    pub fn expanded_from(mut self, from: Option<WindowWidthClass>) -> Self {
        self.expanded_from = from;
        self
    }

    /// What the expanded navigation is: [`NavigationSuiteType::Drawer`] by
    /// default, or [`NavigationSuiteType::RailExpanded`] as m3e defaults to.
    pub fn expanded_type(mut self, expanded_type: NavigationSuiteType) -> Self {
        self.expanded_type = expanded_type;
        self
    }

    /// Hide the navigation; the content takes the freed space. The change
    /// animates, and a hidden navigation cannot take focus.
    pub fn navigation_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// The primary action, typically a FAB: floating above a bar, heading a
    /// rail, or first in a drawer. It stays above a hidden bar.
    pub fn primary_action(mut self, action: impl IntoElement) -> Self {
        self.primary_action = Some(action.into_any_element());
        self
    }

    /// Add to the content area, which fills the space the navigation leaves.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.content.push(child.into_any_element());
        self
    }

    /// Called with a newly selected destination's value.
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Called when the size or an option changes the navigation form.
    pub fn on_type_change(
        mut self,
        handler: impl Fn(&NavigationSuiteType, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_type_change = Some(Rc::new(handler));
        self
    }

    /// The form this suite shows in `window`.
    fn resolved_type(&self, window: &Window) -> NavigationSuiteType {
        self.layout_type.unwrap_or_else(|| {
            navigation_suite_type(
                self.size_class
                    .unwrap_or_else(|| WindowSizeClass::of_window(window)),
                self.expanded_from,
                self.expanded_type,
            )
        })
    }
}

/// The navigation's size along the axis it hides on, in design px.
fn navigation_extent(suite_type: NavigationSuiteType) -> f32 {
    match suite_type {
        NavigationSuiteType::BarCompact => NavItemLayout::Vertical.geometry().item_height,
        NavigationSuiteType::BarMedium => NavItemLayout::Horizontal.geometry().item_height,
        NavigationSuiteType::RailCollapsed => RAIL_COLLAPSED_WIDTH,
        NavigationSuiteType::RailExpanded => RAIL_EXPANDED_WIDTH,
        NavigationSuiteType::Drawer => DRAWER_WIDTH,
        NavigationSuiteType::None => 0.,
    }
}

/// Wrap an optional change handler for the navigation forms' `on_change`.
fn forward(handler: &Option<Change>) -> impl Fn(&SharedString, &mut Window, &mut App) + 'static {
    let handler = handler.clone();
    move |value, window, cx| {
        if let Some(handler) = &handler {
            handler(value, window, cx);
        }
    }
}

impl RenderOnce for NavigationSuite {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let suite_type = self.resolved_type(window);
        let shown = motion::token_value(
            (self.id.clone(), "suite-visibility"),
            if self.hidden { 0. } else { 1. },
            MotionToken::DefaultSpatial,
            window,
            cx,
        )
        .clamp(0., 1.);

        let previous =
            window.use_keyed_state((self.id.clone(), "suite-type"), cx, |_, _| suite_type);
        if *previous.read(cx) != suite_type {
            previous.update(cx, |previous, _| *previous = suite_type);
            if let Some(handler) = self.on_type_change.clone() {
                window.defer(cx, move |window, cx| handler(&suite_type, window, cx));
            }
        }

        let bar = suite_type.is_bar();
        let offset = rem(-navigation_extent(suite_type) * (1. - shown));
        let present = !(self.hidden && shown <= 0.) && suite_type != NavigationSuiteType::None;
        let on_change = self.on_change;
        let mut action = self.primary_action;
        let selected = self.selected.unwrap_or_default();
        let navigation: Option<AnyElement> = present.then(|| match suite_type {
            NavigationSuiteType::RailCollapsed | NavigationSuiteType::RailExpanded => {
                let rail = NavigationRail::new(self.id.clone(), self.label.clone())
                    .items(self.items)
                    .selected(selected)
                    .expanded(suite_type == NavigationSuiteType::RailExpanded)
                    .on_change(forward(&on_change));
                match action.take() {
                    Some(action) => rail.fab(action).into_any_element(),
                    None => rail.into_any_element(),
                }
            }
            NavigationSuiteType::Drawer => {
                let drawer = NavigationDrawer::new(self.id.clone(), self.label.clone())
                    .items(self.items)
                    .selected(selected)
                    .open(true)
                    .on_change(forward(&on_change));
                match action.take() {
                    Some(action) => gpui_base::v_flex()
                        .h_full()
                        .flex_shrink_0()
                        .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
                        .child(div().p(rem(DRAWER_ACTION_PADDING)).child(action))
                        .child(div().flex_1().min_h_0().child(drawer))
                        .into_any_element(),
                    None => drawer.into_any_element(),
                }
            }
            // Bars, and `None`, which `present` already excludes.
            _ => NavigationBar::new(self.id.clone(), self.label.clone())
                .items(self.items)
                .selected(selected)
                .layout(if suite_type == NavigationSuiteType::BarMedium {
                    NavItemLayout::Horizontal
                } else {
                    NavItemLayout::Vertical
                })
                .on_change(forward(&on_change))
                .into_any_element(),
        });
        // The bar's action floats over the content, so a hidden bar keeps it.
        // A rail's or drawer's leaves with its navigation.
        let floating = action.filter(|_| bar).map(|action| {
            div()
                .absolute()
                .bottom(rem(SUITE_PRIMARY_ACTION_PADDING))
                .right(rem(SUITE_PRIMARY_ACTION_PADDING))
                .child(action)
        });
        let content = div()
            .relative()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .debug_selector(|| "navigation-suite-content".into())
            .children(self.content)
            .children(floating);
        let navigation = navigation.map(|navigation| {
            div()
                .flex_shrink_0()
                .debug_selector(|| "navigation-suite-navigation".into())
                .map(|frame| {
                    if bar {
                        frame.w_full().mb(offset)
                    } else {
                        frame.h_full().ml(offset)
                    }
                })
                .child(navigation)
        });
        div()
            .id((self.id, "suite"))
            .size_full()
            .flex()
            .overflow_hidden()
            .bg(color(ColorRole::SURFACE))
            .text_color(color(ColorRole::ON_SURFACE))
            .map(|root| {
                if bar {
                    root.flex_col().child(content).children(navigation)
                } else {
                    root.flex_row().children(navigation).child(content)
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::NavIcon;
    use core::prelude::v1::test;

    fn class(width: f32, height: f32) -> WindowSizeClass {
        WindowSizeClass::of(width, height)
    }

    #[test]
    fn the_rule_moves_between_bar_rail_and_drawer_by_width() {
        use NavigationSuiteType::*;
        let large = Some(WindowWidthClass::Large);
        for (width, height, expected) in [
            (360., 800., BarCompact),
            (599., 800., BarCompact),
            (600., 800., RailCollapsed),
            (1199., 800., RailCollapsed),
            (1200., 800., Drawer),
            (1800., 800., Drawer),
            // A short window keeps a bar, with its items laid out sideways.
            (900., 479., BarMedium),
            (1400., 479., BarMedium),
            (360., 479., BarCompact),
        ] {
            assert_eq!(
                navigation_suite_type(class(width, height), large, Drawer),
                expected,
                "{width}x{height}"
            );
        }
    }

    #[test]
    fn the_expanded_form_and_its_breakpoint_are_options() {
        use NavigationSuiteType::*;
        let wide = class(1300., 800.);
        assert_eq!(
            navigation_suite_type(wide, Some(WindowWidthClass::Large), RailExpanded),
            RailExpanded,
        );
        assert_eq!(
            navigation_suite_type(wide, Option::None, Drawer),
            RailCollapsed
        );
        assert_eq!(
            navigation_suite_type(class(900., 800.), Some(WindowWidthClass::Expanded), Drawer),
            Drawer,
        );
    }

    struct Shell {
        layout_type: Option<NavigationSuiteType>,
        size_class: WindowSizeClass,
        selected: SharedString,
        hidden: bool,
        changes: Vec<NavigationSuiteType>,
    }

    impl Render for Shell {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(crate::tokens::typography::BASE_PX));
            div().size(px(1000.)).child(
                NavigationSuite::new("suite", "Main")
                    .items(
                        ["home", "mail"]
                            .map(|value| NavItem::new(value, value, NavIcon::new("home", "home"))),
                    )
                    .selected(self.selected.clone())
                    .layout_type(self.layout_type)
                    .size_class(Some(self.size_class))
                    .navigation_hidden(self.hidden)
                    .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                        this.selected = value.clone();
                        cx.notify();
                    }))
                    .on_type_change(cx.listener(|this, kind: &NavigationSuiteType, _, cx| {
                        this.changes.push(*kind);
                        cx.notify();
                    }))
                    .child(div().size_full().child("Content")),
            )
        }
    }

    fn open(cx: &mut gpui::TestAppContext) -> (Entity<Shell>, &mut gpui::VisualTestContext) {
        cx.update(|cx| {
            cx.set_reduce_motion(true);
        });
        cx.add_window_view(|_, _| Shell {
            layout_type: None,
            size_class: class(360., 800.),
            selected: "home".into(),
            hidden: false,
            changes: Vec::new(),
        })
    }

    fn draw(cx: &mut gpui::VisualTestContext) {
        for _ in 0..3 {
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
            cx.run_until_parked();
        }
    }

    #[gpui::test]
    fn each_form_sits_on_its_edge_and_the_content_takes_the_rest(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx);
        draw(cx);
        let nav = cx.debug_bounds("navigation-suite-navigation").unwrap();
        let content = cx.debug_bounds("navigation-suite-content").unwrap();
        assert!(nav.top() >= content.bottom(), "a bar is below the content");
        assert_eq!(nav.size.width, px(1000.));

        for (size_class, width) in [
            (class(800., 800.), RAIL_COLLAPSED_WIDTH),
            (class(1300., 800.), DRAWER_WIDTH),
        ] {
            cx.update(|_, cx| view.update(cx, |view, _| view.size_class = size_class));
            draw(cx);
            let nav = cx.debug_bounds("navigation-suite-navigation").unwrap();
            let content = cx.debug_bounds("navigation-suite-content").unwrap();
            assert_eq!(nav.size.width, px(width), "{size_class:?}");
            assert_eq!(content.left(), nav.right(), "{size_class:?}");
            assert_eq!(content.size.width, px(1000. - width), "{size_class:?}");
        }
        let changes = view.read_with(cx, |view, _| view.changes.clone());
        assert_eq!(
            changes,
            [
                NavigationSuiteType::RailCollapsed,
                NavigationSuiteType::Drawer
            ]
        );
    }

    /// The suite has no direction of its own: its rail and action follow the
    /// layout's mirroring, like everything else in a scope.
    #[gpui::test]
    fn right_to_left_puts_the_rail_on_the_right(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx);
        cx.update(|window, cx| {
            window.set_layout_direction(LayoutDirection::Rtl);
            view.update(cx, |view, _| view.size_class = class(800., 800.));
        });
        draw(cx);
        let nav = cx.debug_bounds("navigation-suite-navigation").unwrap();
        let content = cx.debug_bounds("navigation-suite-content").unwrap();
        assert_eq!(nav.size.width, px(RAIL_COLLAPSED_WIDTH));
        assert_eq!(content.left(), px(0.));
        assert_eq!(nav.left(), content.right());
    }

    #[gpui::test]
    fn hiding_gives_the_content_the_space_and_removes_the_navigation(
        cx: &mut gpui::TestAppContext,
    ) {
        let (view, cx) = open(cx);
        cx.update(|_, cx| {
            view.update(cx, |view, _| {
                view.size_class = class(800., 800.);
                view.hidden = true;
            })
        });
        draw(cx);
        assert!(cx.debug_bounds("navigation-suite-navigation").is_none());
        let content = cx.debug_bounds("navigation-suite-content").unwrap();
        assert_eq!(content.size.width, px(1000.));
    }

    #[gpui::test]
    fn a_fixed_type_wins_and_none_shows_only_content(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx);
        cx.update(|_, cx| {
            view.update(cx, |view, _| {
                view.layout_type = Some(NavigationSuiteType::None)
            })
        });
        draw(cx);
        assert!(cx.debug_bounds("navigation-suite-navigation").is_none());
        cx.update(|_, cx| {
            view.update(cx, |view, _| {
                view.layout_type = Some(NavigationSuiteType::RailExpanded)
            })
        });
        draw(cx);
        let nav = cx.debug_bounds("navigation-suite-navigation").unwrap();
        assert_eq!(nav.size.width, px(RAIL_EXPANDED_WIDTH));
    }

    #[gpui::test]
    fn selection_and_keyboard_focus_survive_a_change_of_form(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx);
        draw(cx);
        // Keyboard focus enters the bar, and stays on that destination when
        // the navigation becomes a rail.
        cx.update(|window, cx| window.focus_next(cx));
        draw(cx);
        let focused = cx.update(|window, cx| window.focused(cx));
        assert!(focused.is_some(), "tab reaches a destination");
        cx.update(|_, cx| view.update(cx, |view, _| view.size_class = class(800., 800.)));
        draw(cx);
        assert_eq!(cx.update(|window, cx| window.focused(cx)), focused);
        assert_eq!(
            view.read_with(cx, |view, _| view.selected.clone()).as_ref(),
            "home"
        );
    }
}
