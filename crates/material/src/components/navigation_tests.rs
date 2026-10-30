use super::*;
use crate::tokens::state;
use core::prelude::v1::test;
use gpui::{Modifiers, TestAppContext, VisualTestContext};
use std::time::Duration;

fn icon() -> NavIcon {
    NavIcon::new("outlined.svg", "filled.svg")
}

fn items(count: usize) -> Vec<NavItem> {
    (0..count)
        .map(|index| NavItem::new(format!("d{index}"), format!("Destination {index}"), icon()))
        .collect()
}

#[test]
fn selected_items_use_secondary_roles_and_state_layers() {
    let rest = item_paint(false, 0., 0., false);
    let ink = color(ColorRole::ON_SURFACE_VARIANT);
    assert_eq!(rest.icon, ink);
    assert_eq!(rest.label, ink);
    assert_eq!(rest.indicator.a, 0.);
    assert_eq!(rest.hover, ink.opacity(state::HOVER));
    assert_eq!(rest.press, ink.opacity(state::PRESS));

    let selected = item_paint(true, 1., 1., false);
    assert_eq!(selected.icon, color(ColorRole::ON_SECONDARY_CONTAINER));
    assert_eq!(selected.label, color(ColorRole::SECONDARY));
    assert_eq!(selected.indicator, color(ColorRole::SECONDARY_CONTAINER));
    assert_eq!(selected.indicator_scale, 1.);
    assert_eq!(
        selected.hover,
        color(ColorRole::ON_SECONDARY_CONTAINER).opacity(state::HOVER)
    );

    let disabled = item_paint(true, 1., 1., true);
    assert_eq!(disabled.icon, state::disabled(color(ColorRole::ON_SURFACE)));
    assert_eq!(disabled.hover.a, 0., "no state layer when disabled");
}

#[test]
fn layouts_follow_the_m3_item_geometry() {
    let vertical = NavItemLayout::Vertical.geometry();
    assert_eq!((vertical.pill_min_width, vertical.pill_height), (56., 32.));
    assert_eq!(vertical.item_height, 64.);
    assert_eq!(
        vertical.top + vertical.pill_height + vertical.gap + 16. + vertical.top,
        vertical.item_height,
        "6dp, pill, 4dp, label line, 6dp"
    );
    assert_eq!(NavItemLayout::Horizontal.geometry().pill_height, 40.);
    let expanded = NavItemLayout::Expanded.geometry();
    assert_eq!(
        (expanded.pill_height, expanded.label),
        (56., TypeRole::LabelLarge)
    );
}

#[test]
fn large_badges_trail_labels_beside_icons() {
    let large = NavBadge::Large("3".into());
    assert_eq!(trailing_badge(&large, NavItemLayout::Vertical, true), None);
    assert_eq!(
        trailing_badge(&large, NavItemLayout::Horizontal, true).as_deref(),
        Some("3")
    );
    assert_eq!(
        trailing_badge(&large, NavItemLayout::Horizontal, false),
        None
    );
    assert_eq!(
        trailing_badge(&large, NavItemLayout::Expanded, true).as_deref(),
        Some("3")
    );
    assert_eq!(
        trailing_badge(&NavBadge::Small, NavItemLayout::Horizontal, true),
        None
    );
}

#[test]
fn symbol_icons_fill_when_selected() {
    let icon = NavIcon::symbol(Symbol::named("home").unwrap());
    assert_ne!(icon.path(false), icon.path(true));
    let filled = crate::icons::SymbolIcon::from_asset_path(&icon.path(true)).unwrap();
    let outlined = crate::icons::SymbolIcon::from_asset_path(&icon.path(false)).unwrap();
    assert!(filled.variations().fill() > outlined.variations().fill());
}

struct BarHarness {
    selected: SharedString,
    changes: usize,
    rail: bool,
    expanded: bool,
    disabled: Option<usize>,
    /// Optional focusable neighbours before and after the bar or rail.
    edges: Option<[FocusHandle; 2]>,
}

impl Render for BarHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let mut destinations = items(3);
        if let Some(index) = self.disabled {
            destinations[index] = destinations[index].clone().disabled(true);
        }
        let change = cx.listener(|this, value: &SharedString, _, cx| {
            this.selected = value.clone();
            this.changes += 1;
            cx.notify();
        });
        let edge = |edge: usize| {
            self.edges.as_ref().map(|edges| {
                div()
                    .id(("edge", edge))
                    .size(px(10.))
                    .track_focus(&edges[edge])
            })
        };
        let (before, after) = (edge(0), edge(1));
        let widget = if self.rail {
            div()
                .h(px(600.))
                .child(
                    NavigationRail::new("rail", "Main")
                        .items(destinations)
                        .section(1, "Section")
                        .selected(self.selected.clone())
                        .expanded(self.expanded)
                        .on_change(change),
                )
                .into_any_element()
        } else {
            div()
                .w(px(360.))
                .child(
                    NavigationBar::new("bar", "Main")
                        .items(destinations)
                        .selected(self.selected.clone())
                        .on_change(change),
                )
                .into_any_element()
        };
        div().children(before).child(widget).children(after)
    }
}

fn harness(cx: &mut TestAppContext, rail: bool) -> (Entity<BarHarness>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        super::roving::init(cx);
    });
    cx.add_window_view(move |_, _| BarHarness {
        selected: "d0".into(),
        changes: 0,
        rail,
        expanded: false,
        disabled: None,
        edges: None,
    })
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

/// Enter or Space as a full press: GPUI clicks the focused element on the
/// key's release.
fn activate(cx: &mut VisualTestContext, key: &str) {
    draw(cx);
    let keystroke =
        cx.update(|_, cx| gpui::Keystroke::parse_with_os(key, cx.platform_info().os).unwrap());
    cx.simulate_event(gpui::KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(gpui::KeyUpEvent { keystroke });
    draw(cx);
}

fn settle(cx: &mut VisualTestContext) {
    draw(cx);
    cx.executor().advance_clock(Duration::from_secs(2));
    draw(cx);
}

fn pill(cx: &mut VisualTestContext, value: &str) -> Bounds<Pixels> {
    cx.debug_bounds(Box::leak(format!("nav-pill-{value}").into_boxed_str()))
        .expect("the item's pill is drawn")
}

fn painted(part: &str) -> ItemPaint {
    PAINTED.with(|painted| {
        painted
            .borrow()
            .iter()
            .find(|(key, _)| key.contains(part))
            .map(|(_, paint)| *paint)
            .expect("the item painted")
    })
}

#[gpui::test]
fn clicking_selects_and_the_indicator_grows_on_the_new_item(cx: &mut TestAppContext) {
    let (view, cx) = harness(cx, false);
    settle(cx);
    assert_eq!(pill(cx, "d0").size, size(px(56.), px(32.)));
    assert_eq!(painted("d0").indicator_scale, 1.);
    assert_eq!(painted("d1").indicator_scale, 0.);
    let at = pill(cx, "d1").center();
    cx.simulate_click(at, Modifiers::default());
    draw(cx);
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).selected.as_ref(), "d1");
        assert_eq!(view.read(cx).changes, 1);
    });
    // Mid-motion the new pill is still growing.
    cx.executor().advance_clock(Duration::from_millis(40));
    draw(cx);
    let growing = painted("d1").indicator_scale;
    assert!(growing > 0. && growing < 1., "{growing}");
    settle(cx);
    assert!((painted("d1").indicator_scale - 1.).abs() < 1e-3);
    assert!(painted("d0").indicator_scale.abs() < 1e-3);
    // Re-clicking the current destination changes nothing.
    let at = pill(cx, "d1").center();
    cx.simulate_click(at, Modifiers::default());
    cx.update(|_, cx| assert_eq!(view.read(cx).changes, 1));
}

#[gpui::test]
fn reduced_motion_settles_the_indicator_on_the_changed_frame(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let (_, cx) = harness(cx, false);
    settle(cx);
    let at = pill(cx, "d2").center();
    cx.simulate_click(at, Modifiers::default());
    draw(cx);
    draw(cx);
    assert_eq!(painted("d2").indicator_scale, 1.);
}

#[gpui::test]
fn arrows_rove_focus_and_enter_activates(cx: &mut TestAppContext) {
    let (view, cx) = harness(cx, false);
    settle(cx);
    cx.update(|window, cx| window.focus_next(cx));
    draw(cx);
    cx.simulate_keystrokes("right");
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).changes, 0, "arrows only move focus"));
    activate(cx, "enter");
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).selected.as_ref(), "d1"));
    cx.simulate_keystrokes("left left");
    activate(cx, "space");
    draw(cx);
    cx.update(|_, cx| {
        assert_eq!(
            view.read(cx).selected.as_ref(),
            "d2",
            "left wraps to the last"
        );
    });
}

#[gpui::test]
fn arrows_skip_disabled_items(cx: &mut TestAppContext) {
    let (view, cx) = harness(cx, false);
    view.update(cx, |view, _| view.disabled = Some(1));
    settle(cx);
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_keystrokes("right");
    activate(cx, "enter");
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).selected.as_ref(), "d2"));
    let at = pill(cx, "d1").center();
    cx.simulate_click(at, Modifiers::default());
    cx.update(|_, cx| assert_eq!(view.read(cx).selected.as_ref(), "d2"));
}

fn tab_leaves_after_arrowing(cx: &mut TestAppContext, rail: bool) {
    let (view, cx) = harness(cx, rail);
    let edges: [FocusHandle; 2] = cx.update(|_, cx| {
        [
            cx.focus_handle().tab_stop(true),
            cx.focus_handle().tab_stop(true),
        ]
    });
    view.update(cx, |view, _| view.edges = Some(edges.clone()));
    settle(cx);
    let arrows = if rail { "down down" } else { "right right" };
    let is_focused = |edge: &FocusHandle, cx: &mut VisualTestContext| {
        cx.update(|window, _| edge.is_focused(window))
    };
    cx.update(|window, cx| edges[0].focus(window, cx));
    draw(cx);
    cx.update(|window, cx| window.focus_next(cx));
    draw(cx);
    cx.simulate_keystrokes(arrows);
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).changes, 0, "arrows only move focus"));
    cx.update(|window, cx| window.focus_prev(cx));
    draw(cx);
    assert!(is_focused(&edges[0], cx), "Shift+Tab leaves backward");
    cx.update(|window, cx| window.focus_next(cx));
    draw(cx);
    cx.simulate_keystrokes(arrows);
    draw(cx);
    cx.update(|window, cx| window.focus_next(cx));
    draw(cx);
    assert!(is_focused(&edges[1], cx), "Tab leaves forward");
}

#[gpui::test]
fn tab_leaves_the_bar_after_arrowing(cx: &mut TestAppContext) {
    tab_leaves_after_arrowing(cx, false);
}

#[gpui::test]
fn tab_leaves_the_rail_after_arrowing(cx: &mut TestAppContext) {
    tab_leaves_after_arrowing(cx, true);
}

#[gpui::test]
fn the_bar_has_one_tab_stop(cx: &mut TestAppContext) {
    let (_, cx) = harness(cx, false);
    settle(cx);
    cx.update(|window, cx| window.focus_next(cx));
    let first = cx.update(|window, cx| window.focused(cx));
    assert!(first.is_some());
    cx.update(|window, cx| window.focus_next(cx));
    assert_eq!(
        cx.update(|window, cx| window.focused(cx)),
        first,
        "tab cycles back to the selected item"
    );
}

#[gpui::test]
fn the_rail_width_springs_between_collapsed_and_expanded(cx: &mut TestAppContext) {
    let (view, cx) = harness(cx, true);
    settle(cx);
    assert_eq!(pill(cx, "d0").size, size(px(56.), px(32.)));
    let collapsed = cx.debug_bounds("nav-pill-d0").unwrap();
    view.update(cx, |view, cx| {
        view.expanded = true;
        cx.notify();
    });
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(60));
    draw(cx);
    settle(cx);
    let expanded = pill(cx, "d0");
    assert_eq!(expanded.size.height, px(56.), "expanded items are 56dp");
    assert!(expanded.size.width > collapsed.size.width);
    assert_eq!(expanded.left(), px(20.), "20dp leading inset");
}

#[gpui::test]
fn previewed_items_are_specimens(cx: &mut TestAppContext) {
    struct Specimen;
    impl Render for Specimen {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            NavigationBar::new("specimen", "Specimen")
                .items(
                    items(2)
                        .into_iter()
                        .map(|item| item.preview_state(InteractionPreview::Hovered)),
                )
                .selected("d0")
        }
    }
    cx.update(gpui_base::init);
    let (_, cx) = cx.add_window_view(|_, _| Specimen);
    draw(cx);
    cx.update(|window, cx| window.focus_next(cx));
    assert!(cx.update(|window, cx| window.focused(cx)).is_none());
    assert_eq!(painted("d0").hover.a, painted("d0").hover.a);
}

#[test]
fn adaptive_bars_switch_layout_at_the_medium_class() {
    assert_eq!(adaptive_layout(599.), NavItemLayout::Vertical);
    assert_eq!(adaptive_layout(MEDIUM_WIDTH), NavItemLayout::Horizontal);
}

struct Sized {
    width: f32,
    hide: bool,
    expanded: bool,
}

impl Render for Sized {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div()
            .flex()
            .flex_col()
            .child(
                div().w(px(self.width)).child(
                    NavigationBar::new("bar", "Main")
                        .items(items(3))
                        .adaptive(true)
                        .ripple(false)
                        .selected("d0"),
                ),
            )
            .child(
                div().h(px(400.)).child(
                    NavigationRail::new("rail", "Main")
                        .items(items(2).into_iter().map(|item| {
                            let value = item.value().clone();
                            NavItem::new(format!("r{value}"), item.label().clone(), icon())
                        }))
                        .hide_collapsed(self.hide)
                        .expanded(self.expanded)
                        .selected("rd0"),
                ),
            )
    }
}

#[gpui::test]
fn adaptive_bars_measure_their_width(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        super::roving::init(cx);
    });
    let (view, cx) = cx.add_window_view(|_, _| Sized {
        width: 360.,
        hide: false,
        expanded: false,
    });
    settle(cx);
    assert_eq!(pill(cx, "d0").size.height, px(32.), "compact: vertical");
    view.update(cx, |view, cx| {
        view.width = 900.;
        cx.notify();
    });
    settle(cx);
    assert_eq!(pill(cx, "d0").size.height, px(40.), "medium: horizontal");
    view.update(cx, |view, cx| {
        view.width = 360.;
        cx.notify();
    });
    settle(cx);
    assert_eq!(pill(cx, "d0").size.height, px(32.));
}

#[gpui::test]
fn hidden_collapsed_rails_open_from_zero_width(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        super::roving::init(cx);
    });
    let (view, cx) = cx.add_window_view(|_, _| Sized {
        width: 360.,
        hide: true,
        expanded: false,
    });
    settle(cx);
    assert!(
        cx.debug_bounds("nav-pill-rd0").is_none(),
        "a hidden collapsed rail draws no destinations"
    );
    view.update(cx, |view, cx| {
        view.expanded = true;
        cx.notify();
    });
    settle(cx);
    assert_eq!(pill(cx, "rd0").size.height, px(56.));
    view.update(cx, |view, cx| {
        view.expanded = false;
        cx.notify();
    });
    settle(cx);
    assert!(cx.debug_bounds("nav-pill-rd0").is_none());
}
