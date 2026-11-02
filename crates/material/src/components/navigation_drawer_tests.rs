use super::*;
use crate::components::NavIcon;
use core::prelude::v1::test;
use gpui::{Modifiers, TestAppContext, VisualTestContext};
use std::time::Duration;

fn items(count: usize) -> Vec<NavItem> {
    (0..count)
        .map(|index| {
            NavItem::new(
                format!("d{index}"),
                format!("Destination {index}"),
                NavIcon::new("outlined.svg", "filled.svg"),
            )
        })
        .collect()
}

#[test]
fn selected_destinations_use_secondary_container_roles() {
    let rest = drawer_paint(false, 0., false);
    assert_eq!(rest.label, color(ColorRole::ON_SURFACE_VARIANT));
    assert_eq!(rest.icon, color(ColorRole::ON_SURFACE_VARIANT));
    assert_eq!(rest.indicator.a, 0.);
    assert_eq!(
        rest.hover,
        color(ColorRole::ON_SURFACE).opacity(state::HOVER)
    );
    let selected = drawer_paint(true, 1., false);
    assert_eq!(selected.label, color(ColorRole::ON_SECONDARY_CONTAINER));
    assert_eq!(selected.indicator, color(ColorRole::SECONDARY_CONTAINER));
    assert_eq!(
        selected.press,
        color(ColorRole::ON_SECONDARY_CONTAINER).opacity(state::PRESS)
    );
    let disabled = drawer_paint(false, 0., true);
    assert_eq!(
        disabled.label,
        state::disabled(color(ColorRole::ON_SURFACE))
    );
    assert_eq!(disabled.hover.a, 0.);
}

struct Harness {
    selected: SharedString,
    open: bool,
    modal: bool,
    dense: bool,
    end: bool,
    dismissed: usize,
}

impl Render for Harness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let change = cx.listener(|this, value: &SharedString, _, cx| {
            this.selected = value.clone();
            cx.notify();
        });
        let dismiss = cx.entity().downgrade();
        div()
            .relative()
            .w(px(800.))
            .h(px(600.))
            .flex()
            .child(
                NavigationDrawer::new("drawer", "Main")
                    .headline("Mail")
                    .items(items(4).into_iter().enumerate().map(|(index, item)| {
                        if index == 1 {
                            item.badge(NavBadge::Large("24".into()))
                        } else {
                            item
                        }
                    }))
                    .section(2, "Labels")
                    .selected(self.selected.clone())
                    .open(self.open)
                    .modal(self.modal)
                    .dense(self.dense)
                    .width(280.)
                    .position(if self.end {
                        DrawerPosition::End
                    } else {
                        DrawerPosition::Start
                    })
                    .on_change(change)
                    .on_dismiss(move |_, cx| {
                        let _ = dismiss.update(cx, |this, cx| {
                            this.open = false;
                            this.dismissed += 1;
                            cx.notify();
                        });
                    }),
            )
            .child(div().flex_1())
    }
}

fn harness(cx: &mut TestAppContext, modal: bool) -> (Entity<Harness>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        super::super::roving::init(cx);
    });
    cx.add_window_view(move |_, _| Harness {
        selected: "d0".into(),
        open: true,
        modal,
        dense: false,
        end: false,
        dismissed: 0,
    })
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

fn settle(cx: &mut VisualTestContext) {
    draw(cx);
    cx.executor().advance_clock(Duration::from_secs(2));
    draw(cx);
    draw(cx);
}

fn row(cx: &mut VisualTestContext, value: &str) -> Option<Bounds<Pixels>> {
    cx.debug_bounds(Box::leak(format!("drawer-item-{value}").into_boxed_str()))
}

#[gpui::test]
fn rows_follow_the_kit_geometry_and_clicks_select(cx: &mut TestAppContext) {
    let (view, cx) = harness(cx, false);
    settle(cx);
    let sheet = cx.debug_bounds("navigation-drawer-sheet").unwrap();
    assert_eq!(sheet.size.width, px(280.));
    let first = row(cx, "d0").unwrap();
    assert_eq!(first.size.height, px(56.));
    assert_eq!(first.left() - sheet.left(), px(12.), "12dp padding");
    // Headline (56dp) above the first row.
    assert_eq!(first.top() - sheet.top(), px(12. + 56.));
    // Divider (1dp) and section header (56dp) before the third row.
    let second = row(cx, "d1").unwrap();
    let third = row(cx, "d2").unwrap();
    assert_eq!(third.top() - second.bottom(), px(57.));
    assert_eq!(
        DRAWER_PAINTED.with(|p| p.borrow()["d0"].indicator),
        color(ColorRole::SECONDARY_CONTAINER)
    );
    cx.simulate_click(third.center(), Modifiers::none());
    settle(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).selected.as_ref(), "d2"));
    assert_eq!(
        DRAWER_PAINTED.with(|p| p.borrow()["d0"].indicator.a),
        0.,
        "the old indicator fades out"
    );

    view.update(cx, |view, cx| {
        view.dense = true;
        cx.notify();
    });
    settle(cx);
    assert_eq!(row(cx, "d0").unwrap().size.height, px(48.));
}

#[gpui::test]
fn arrows_rove_and_enter_selects(cx: &mut TestAppContext) {
    let (view, cx) = harness(cx, false);
    settle(cx);
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_keystrokes("down");
    draw(cx);
    // Enter as a full press: GPUI clicks the focused element on release.
    let keystroke =
        cx.update(|_, cx| gpui::Keystroke::parse_with_os("enter", cx.platform_info().os).unwrap());
    cx.simulate_event(gpui::KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(gpui::KeyUpEvent { keystroke });
    settle(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).selected.as_ref(), "d1"));
}

#[gpui::test]
fn standard_drawers_spring_their_width_closed(cx: &mut TestAppContext) {
    let (view, cx) = harness(cx, false);
    settle(cx);
    view.update(cx, |view, cx| {
        view.open = false;
        cx.notify();
    });
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(50));
    draw(cx);
    let (amount, _) = DRAWER_OPEN.with(|cell| cell.get());
    assert!(amount > 0. && amount < 1., "mid-motion: {amount}");
    settle(cx);
    assert!(row(cx, "d0").is_none(), "a closed drawer draws nothing");
}

#[gpui::test]
fn modal_drawers_float_over_a_dismissing_scrim(cx: &mut TestAppContext) {
    let (view, cx) = harness(cx, true);
    settle(cx);
    let scrim = cx.debug_bounds("navigation-drawer-scrim").unwrap();
    assert_eq!(scrim.size.width, px(800.));
    // Opening focused the selected destination; Escape dismisses.
    cx.simulate_keystrokes("escape");
    settle(cx);
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).dismissed, 1);
        assert!(!view.read(cx).open);
    });
    assert!(cx.debug_bounds("navigation-drawer-scrim").is_none());

    view.update(cx, |view, cx| {
        view.open = true;
        view.end = true;
        cx.notify();
    });
    settle(cx);
    let sheet = cx.debug_bounds("navigation-drawer-sheet").unwrap();
    assert_eq!(sheet.right(), px(800.), "an end drawer hugs the end edge");
    cx.simulate_click(point(px(100.), px(300.)), Modifiers::none());
    settle(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).dismissed, 2));
}

struct Trapped {
    open: bool,
    dismissed: usize,
    trigger: FocusHandle,
    focus_on_open: bool,
}

impl Render for Trapped {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let dismiss = cx.entity().downgrade();
        div()
            .relative()
            .w(px(800.))
            .h(px(600.))
            .child(
                div()
                    .id("opener")
                    .track_focus(&self.trigger)
                    .tab_stop(true)
                    .size(px(20.)),
            )
            .child(
                NavigationDrawer::new("drawer", "Main")
                    .items(items(3))
                    .selected("d0")
                    .open(self.open)
                    .modal(true)
                    .focus_on_open(self.focus_on_open)
                    .on_dismiss(move |_, cx| {
                        let _ = dismiss.update(cx, |this, cx| {
                            this.open = false;
                            this.dismissed += 1;
                            cx.notify();
                        });
                    }),
            )
    }
}

fn trapped(cx: &mut TestAppContext, reduced: bool) -> (Entity<Trapped>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        super::super::roving::init(cx);
        cx.set_reduce_motion(reduced);
    });
    cx.add_window_view(|_, cx| Trapped {
        open: false,
        dismissed: 0,
        trigger: cx.focus_handle().tab_stop(true),
        focus_on_open: true,
    })
}

fn settle_open(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        draw(cx);
        cx.run_until_parked();
    }
}

#[gpui::test]
fn a_modal_drawer_traps_focus_and_returns_it_to_its_opener(cx: &mut TestAppContext) {
    let (view, cx) = trapped(cx, true);
    settle_open(cx);
    cx.update(|window, cx| view.read(cx).trigger.clone().focus(window, cx));
    settle_open(cx);
    view.update(cx, |view, cx| {
        view.open = true;
        cx.notify();
    });
    settle_open(cx);
    cx.update(|window, cx| {
        assert!(
            gpui_base::active_focus_trap(window, cx).is_some(),
            "focus is confined to the open drawer"
        );
        assert!(!view.read(cx).trigger.is_focused(window));
    });
    view.update(cx, |view, cx| {
        view.open = false;
        cx.notify();
    });
    settle_open(cx);
    cx.update(|window, cx| {
        assert!(
            view.read(cx).trigger.is_focused(window),
            "focus went back to what opened the drawer"
        );
    });
}

/// A drawer that opens without taking focus (a search field's results)
/// leaves it where it was: neither the drawer nor its focus trap takes it.
#[gpui::test]
fn a_drawer_that_does_not_focus_on_open_leaves_focus_alone(cx: &mut TestAppContext) {
    let (view, cx) = trapped(cx, true);
    view.update(cx, |view, _| view.focus_on_open = false);
    settle_open(cx);
    cx.update(|window, cx| view.read(cx).trigger.clone().focus(window, cx));
    settle_open(cx);
    view.update(cx, |view, cx| {
        view.open = true;
        cx.notify();
    });
    settle_open(cx);
    cx.update(|window, cx| {
        assert!(
            view.read(cx).trigger.is_focused(window),
            "focus stayed where it was"
        );
        assert!(gpui_base::active_focus_trap(window, cx).is_none());
    });
    view.update(cx, |view, cx| {
        view.open = false;
        cx.notify();
    });
    settle_open(cx);
    cx.update(|window, cx| assert!(view.read(cx).trigger.is_focused(window)));
}

/// A field above an open drawer of its results, as a search shows them.
struct Results {
    field: FocusHandle,
    container: FocusHandle,
}

impl Render for Results {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let field = self.field.clone();
        div()
            .relative()
            .w(px(800.))
            .h(px(600.))
            .child(div().id("field").track_focus(&self.field).size(px(20.)))
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .track_focus(&self.container)
                    .child(
                        NavigationDrawer::new("results", "Results")
                            .items(items(3))
                            // The selection is not first: focus_first still goes first.
                            .selected("d1")
                            .modal(true)
                            .focus_on_open(false)
                            .on_exit_start(move |window, cx| field.focus(window, cx)),
                    ),
            )
    }
}

/// A field's Down goes to the first result and Up from there comes back, so
/// the results are reachable from the keyboard while focus starts in the field.
#[gpui::test]
fn a_field_reaches_the_first_destination_and_up_comes_back(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        super::super::roving::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|_, cx| Results {
        field: cx.focus_handle(),
        container: cx.focus_handle(),
    });
    settle_open(cx);
    cx.update(|window, cx| view.read(cx).field.clone().focus(window, cx));
    settle_open(cx);
    cx.update(|window, cx| {
        let container = view.read(cx).container.clone();
        NavigationDrawer::focus_first(&container, window, cx);
    });
    settle_open(cx);
    cx.update(|window, cx| {
        let view = view.read(cx);
        assert!(view.container.contains_focused(window, cx));
        assert!(!view.field.is_focused(window));
    });
    // Down and Up rove between the results.
    cx.simulate_keystrokes("down up");
    settle_open(cx);
    cx.update(|window, cx| assert!(!view.read(cx).field.is_focused(window)));
    // Up on the first result leaves for the field instead of wrapping.
    cx.simulate_keystrokes("up");
    settle_open(cx);
    cx.update(|window, cx| {
        assert!(
            view.read(cx).field.is_focused(window),
            "Up from the first result is back in the field"
        );
    });
}

#[gpui::test]
fn a_closing_drawer_does_not_dismiss_again(cx: &mut TestAppContext) {
    let (view, cx) = trapped(cx, false);
    settle_open(cx);
    view.update(cx, |view, cx| {
        view.open = true;
        cx.notify();
    });
    settle_open(cx);
    cx.executor().advance_clock(Duration::from_secs(2));
    settle_open(cx);
    let scrim = gpui::point(px(700.), px(300.));
    cx.simulate_click(scrim, Modifiers::default());
    draw(cx);
    cx.simulate_click(scrim, Modifiers::default());
    draw(cx);
    assert_eq!(view.read_with(cx, |view, _| view.dismissed), 1);
}

struct LongDrawer;
impl Render for LongDrawer {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div().relative().w(px(800.)).h(px(300.)).flex().child(
            NavigationDrawer::new("drawer", "Main")
                .items(items(30))
                .selected("d0")
                .open(true)
                .width(280.),
        )
    }
}

/// A row keyboard focus lands on is scrolled into the sheet, and the sheet stays where a
/// wheel takes it afterwards, with focus still on that row.
#[gpui::test]
fn keyboard_focus_scrolls_its_row_into_the_sheet(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        super::super::roving::init(cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(|_, _| LongDrawer);
    settle(cx);
    let sheet = cx.debug_bounds("navigation-drawer-sheet").unwrap();
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_keystrokes("end");
    settle(cx);
    let last = row(cx, "d29").unwrap();
    assert!(
        last.bottom() <= sheet.bottom() && last.top() >= sheet.top(),
        "end: {last:?} in {sheet:?}"
    );
    cx.simulate_keystrokes("home");
    settle(cx);
    let first = row(cx, "d0").unwrap();
    assert!(
        first.top() >= sheet.top() && first.bottom() <= sheet.bottom(),
        "home: {first:?} in {sheet:?}"
    );
    for _ in 0..3 {
        cx.simulate_keystrokes("down");
    }
    settle(cx);
    cx.simulate_event(gpui::ScrollEvent {
        position: sheet.center(),
        delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.), px(-400.))),
        ..Default::default()
    });
    settle(cx);
    assert!(
        row(cx, "d3").unwrap().bottom() < sheet.top(),
        "the wheel took the focused row away and it stays away"
    );
}

/// The kit's `Building Blocks / Nav item`, selected and not, with and without an
/// icon, focused.
#[test]
fn items_follow_the_kits_nav_item() {
    assert_eq!(
        drawer_label_scale(true).font_weight,
        TypeRole::LabelLarge.emphasized().font_weight
    );
    assert_eq!(
        drawer_label_scale(false).font_weight,
        TypeRole::LabelLarge.scale().font_weight
    );
    assert_eq!(drawer_leading(true), 16.);
    assert_eq!(drawer_leading(false), 24.);
    let selected = drawer_paint(true, 1., false);
    let ink = color(ColorRole::ON_SECONDARY_CONTAINER);
    assert_eq!(selected.focus, ink.opacity(0.10));
    let rest = drawer_paint(false, 0., false);
    assert_eq!(rest.focus, color(ColorRole::ON_SURFACE).opacity(0.10));
}

#[gpui::test]
fn keyboard_focus_paints_a_rows_focus_layer_and_a_pointer_does_not(cx: &mut TestAppContext) {
    let (_, cx) = harness(cx, false);
    settle(cx);
    let focus = gpui::Background::from(drawer_paint(false, 0., false).focus);
    let painted = |cx: &mut VisualTestContext| {
        cx.update(|window, _| {
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.background == focus)
        })
    };
    assert!(!painted(cx), "no row has focus");
    let second = row(cx, "d1").unwrap();
    cx.simulate_click(second.center(), Modifiers::none());
    settle(cx);
    assert!(!painted(cx), "a press focuses the row but shows no focus");
    cx.simulate_keystrokes("down");
    settle(cx);
    assert!(painted(cx), "the keyboard does");
}

#[test]
fn a_drawer_narrows_to_its_cap_at_any_text_scale() {
    assert_eq!(fitted_width(360., None, px(28.)), 360.);
    // 200% text: 360 design px would draw 720 px; a 264 px cap allows 132.
    assert_eq!(fitted_width(360., Some(px(264.)), px(28.)), 132.);
    assert_eq!(fitted_width(200., Some(px(1000.)), px(14.)), 200.);
    assert_eq!(fitted_width(360., Some(px(-10.)), px(14.)), 0.);
}

/// A drawer at 200% text in a phone-width window.
struct Scaled {
    max_width: Option<Pixels>,
    modal: bool,
}

impl Render for Scaled {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(28.));
        let drawer = NavigationDrawer::new("scaled", "Main")
            .items(items(3))
            .selected("d0")
            .modal(self.modal)
            .width(360.)
            .on_dismiss(|_, _| {});
        let drawer = match self.max_width {
            Some(max) => drawer.max_width(max),
            None => drawer,
        };
        div().relative().size_full().flex().child(drawer)
    }
}

#[gpui::test]
fn a_scaled_modal_drawer_leaves_its_scrim_in_a_narrow_window(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        super::super::roving::init(cx);
    });
    let (view, cx) = cx.add_window_view(|_, _| Scaled {
        max_width: None,
        modal: true,
    });
    cx.simulate_resize(size(px(320.), px(640.)));
    settle(cx);
    let sheet = cx.debug_bounds("navigation-drawer-sheet").unwrap();
    assert!(
        sheet.size.width <= px(320. - DRAWER_SCRIM_MARGIN + 0.5),
        "the sheet fits beside its scrim: {sheet:?}"
    );
    assert!(
        sheet.size.width > px(200.),
        "and still takes its share: {sheet:?}"
    );

    // A caller's narrower cap wins over the window's.
    view.update(cx, |view, cx| {
        view.max_width = Some(px(150.));
        cx.notify();
    });
    settle(cx);
    let sheet = cx.debug_bounds("navigation-drawer-sheet").unwrap();
    assert!((sheet.size.width - px(150.)).abs() < px(0.5), "{sheet:?}");

    // A standard drawer honours a cap too.
    view.update(cx, |view, cx| {
        view.modal = false;
        cx.notify();
    });
    settle(cx);
    let sheet = cx.debug_bounds("navigation-drawer-sheet").unwrap();
    assert!((sheet.size.width - px(150.)).abs() < px(0.5), "{sheet:?}");
}
