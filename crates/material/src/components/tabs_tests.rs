use super::*;
use core::prelude::v1::test;
use gpui::{Modifiers, TestAppContext, VisualTestContext};
use std::time::Duration;

#[test]
fn heights_and_indicators_follow_the_variant() {
    let labels = [TabItem::new("a", "A"), TabItem::new("b", "B")];
    let stacked = [TabItem::new("a", "A").icon(crate::icons::IconName::Info)];
    let icons = [TabItem::new("a", "A")
        .icon(crate::icons::IconName::Info)
        .icon_only()];
    assert_eq!(tabs_height(TabsVariant::Primary, &labels), 48.);
    assert_eq!(tabs_height(TabsVariant::Primary, &stacked), 64.);
    assert_eq!(tabs_height(TabsVariant::Primary, &icons), 48.);
    assert_eq!(tabs_height(TabsVariant::Secondary, &stacked), 48.);
    assert_eq!(indicator_height(TabsVariant::Primary), 3.);
    assert_eq!(indicator_height(TabsVariant::Secondary), 2.);
}

#[test]
fn active_tabs_use_their_variant_roles() {
    let rest = color(ColorRole::ON_SURFACE_VARIANT);
    assert_eq!(
        tab_paint(TabsVariant::Primary, false, 0., false).content,
        rest
    );
    let primary = tab_paint(TabsVariant::Primary, true, 1., false);
    assert_eq!(primary.content, color(ColorRole::PRIMARY));
    assert_eq!(
        primary.hover,
        color(ColorRole::PRIMARY).opacity(state::HOVER)
    );
    let secondary = tab_paint(TabsVariant::Secondary, true, 1., false);
    assert_eq!(secondary.content, color(ColorRole::ON_SURFACE));
    assert_eq!(
        secondary.press,
        color(ColorRole::ON_SURFACE).opacity(state::PRESS)
    );
    let disabled = tab_paint(TabsVariant::Primary, true, 1., true);
    assert_eq!(
        disabled.content,
        state::disabled(color(ColorRole::ON_SURFACE))
    );
    assert_eq!(disabled.hover.a, 0.);
}

struct Harness {
    selected: SharedString,
    changes: usize,
    variant: TabsVariant,
    auto: bool,
    disabled: Option<usize>,
    /// Optional focusable neighbours before and after the tab row.
    edges: Option<[FocusHandle; 2]>,
}

impl Render for Harness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let disabled = self.disabled;
        let edge = |edge: usize| {
            self.edges.as_ref().map(|edges| {
                div()
                    .id(("edge", edge))
                    .size(px(10.))
                    .track_focus(&edges[edge])
            })
        };
        div()
            .w(px(300.))
            .children(edge(0))
            .child(
                MaterialTabs::new("tabs", "Sections")
                    .variant(self.variant)
                    .auto_activate(self.auto)
                    .items((0..3).map(|index| {
                        TabItem::new(format!("t{index}"), "Tab").disabled(disabled == Some(index))
                    }))
                    .selected(self.selected.clone())
                    .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                        this.selected = value.clone();
                        this.changes += 1;
                        cx.notify();
                    })),
            )
            .children(edge(1))
    }
}

fn harness(
    cx: &mut TestAppContext,
    variant: TabsVariant,
) -> (Entity<Harness>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        super::roving::init(cx);
    });
    cx.add_window_view(move |_, _| Harness {
        edges: None,
        selected: "t0".into(),
        changes: 0,
        variant,
        auto: false,
        disabled: None,
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
    draw(cx);
    cx.executor().advance_clock(Duration::from_secs(2));
    draw(cx);
}

fn indicator() -> (f32, f32) {
    INDICATOR
        .with(|cell| cell.get())
        .expect("the indicator was measured")
}

fn tab(cx: &mut VisualTestContext, index: usize) -> Bounds<Pixels> {
    cx.debug_bounds(["material-tab-0", "material-tab-1", "material-tab-2"][index])
        .expect("the tab is drawn")
}

#[gpui::test]
fn the_secondary_indicator_slides_across_whole_tabs(cx: &mut TestAppContext) {
    let (view, cx) = harness(cx, TabsVariant::Secondary);
    settle(cx);
    assert_eq!(indicator(), (0., 100.));
    let at = tab(cx, 2).center();
    cx.simulate_click(at, Modifiers::default());
    cx.update(|_, cx| assert_eq!(view.read(cx).selected.as_ref(), "t2"));
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(80));
    draw(cx);
    let (left, width) = indicator();
    assert!(left > 0. && left < 200., "mid-slide at {left}");
    assert!((width - 100.).abs() < 1., "equal tabs keep their width");
    settle(cx);
    let (left, _) = indicator();
    assert!((left - 200.).abs() < 0.5, "settled at {left}");
}

#[gpui::test]
fn the_primary_indicator_matches_the_content_width(cx: &mut TestAppContext) {
    let (_, cx) = harness(cx, TabsVariant::Primary);
    settle(cx);
    let (left, width) = indicator();
    assert!(
        width > 0. && width < 100. - 2. * 16.,
        "content width {width}"
    );
    assert!((left + width / 2. - 50.).abs() < 0.5, "centered in its tab");
}

#[gpui::test]
fn reduced_motion_moves_the_indicator_at_once(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let (_, cx) = harness(cx, TabsVariant::Secondary);
    settle(cx);
    let at = tab(cx, 1).center();
    cx.simulate_click(at, Modifiers::default());
    draw(cx);
    draw(cx);
    assert_eq!(indicator(), (100., 100.));
}

#[gpui::test]
fn arrows_rove_and_enter_activates(cx: &mut TestAppContext) {
    let (view, cx) = harness(cx, TabsVariant::Primary);
    settle(cx);
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_keystrokes("right");
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).changes, 0));
    activate(cx, "enter");
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).selected.as_ref(), "t1"));
    cx.simulate_keystrokes("end");
    activate(cx, "space");
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).selected.as_ref(), "t2"));
    cx.simulate_keystrokes("home");
    activate(cx, "enter");
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).selected.as_ref(), "t0"));
}

fn edge_focused(edges: &[FocusHandle; 2], edge: usize, cx: &mut VisualTestContext) -> bool {
    cx.update(|window, _| edges[edge].is_focused(window))
}

#[gpui::test]
fn tab_and_shift_tab_leave_after_arrowing_in_manual_mode(cx: &mut TestAppContext) {
    let (view, cx) = harness(cx, TabsVariant::Primary);
    let edges: [FocusHandle; 2] = cx.update(|_, cx| {
        [
            cx.focus_handle().tab_stop(true),
            cx.focus_handle().tab_stop(true),
        ]
    });
    view.update(cx, |view, _| view.edges = Some(edges.clone()));
    settle(cx);
    cx.update(|window, cx| edges[0].focus(window, cx));
    draw(cx);
    cx.update(|window, cx| window.focus_next(cx));
    draw(cx);
    cx.simulate_keystrokes("right right");
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).changes, 0, "manual mode"));
    cx.update(|window, cx| window.focus_prev(cx));
    draw(cx);
    assert!(edge_focused(&edges, 0, cx), "Shift+Tab leaves the tab row");
    cx.update(|window, cx| window.focus_next(cx));
    draw(cx);
    cx.simulate_keystrokes("right right");
    draw(cx);
    cx.update(|window, cx| window.focus_next(cx));
    draw(cx);
    assert!(edge_focused(&edges, 1, cx), "Tab leaves the tab row");
}

#[gpui::test]
fn auto_activation_selects_on_focus_and_skips_disabled_tabs(cx: &mut TestAppContext) {
    let (view, cx) = harness(cx, TabsVariant::Primary);
    view.update(cx, |view, _| {
        view.auto = true;
        view.disabled = Some(1);
    });
    settle(cx);
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_keystrokes("right");
    draw(cx);
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).selected.as_ref(), "t2");
        assert_eq!(view.read(cx).changes, 1);
    });
    let at = tab(cx, 1).center();
    cx.simulate_click(at, Modifiers::default());
    cx.update(|_, cx| assert_eq!(view.read(cx).changes, 1, "disabled tabs ignore clicks"));
}

struct DividerHarness {
    divider: bool,
}

impl Render for DividerHarness {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div().w(px(300.)).child(
            MaterialTabs::new("tabs", "Sections")
                .divider(self.divider)
                .items([TabItem::new("a", "A"), TabItem::new("b", "B")])
                .selected("a"),
        )
    }
}

#[gpui::test]
fn the_divider_can_be_hidden(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        super::roving::init(cx);
    });
    let (view, cx) = cx.add_window_view(|_, _| DividerHarness { divider: true });
    settle(cx);
    let line = cx
        .debug_bounds("tabs-divider")
        .expect("the divider is drawn");
    assert_eq!(line.size.height, px(1.));
    assert_eq!(line.size.width, px(300.));
    cx.update(|_, cx| view.update(cx, |view, _| view.divider = false));
    settle(cx);
    assert!(cx.debug_bounds("tabs-divider").is_none());
}

#[test]
fn rounding_of_a_measurement_does_not_redraw_the_tabs() {
    use super::spans_moved;
    assert!(!spans_moved(&[(0., 83.5)], &[(0., 84.)]));
    assert!(!spans_moved(&[(29.5, 25.)], &[(30.5, 25.)]));
    assert!(spans_moved(&[(0., 83.5)], &[(0., 90.)]));
    assert!(spans_moved(&[(0., 83.5)], &[(0., 83.5), (84., 80.)]));
    assert!(spans_moved(&[], &[(0., 83.5)]));
}

struct Narrow {
    width: f32,
    container: ColorRole,
}

impl gpui::Render for Narrow {
    fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        div().w(px(self.width)).child(
            MaterialTabs::new("narrow", "Sections")
                .variant(TabsVariant::Secondary)
                .container(self.container)
                .items([
                    TabItem::new("a", "Inbox").icon(crate::icons::IconName::Info),
                    TabItem::new("b", "Favorites").icon(crate::icons::IconName::Info),
                    TabItem::new("c", "Sent").icon(crate::icons::IconName::Info),
                ])
                .selected("a"),
        )
    }
}

/// A tab narrower than its label and the kit's padding keeps the label whole:
/// the padding gives way first, and only then the label.
#[gpui::test]
fn the_padding_gives_way_before_the_label(cx: &mut TestAppContext) {
    cx.update(crate::init);
    let (_, cx) = cx.add_window_view(|_, _| Narrow {
        width: 1200.,
        container: ColorRole::SURFACE,
    });
    let draw = |cx: &mut VisualTestContext| {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
    };
    draw(cx);
    let whole = cx.debug_bounds("tab-label-1").unwrap().size.width;
    let content = cx.debug_bounds("material-tab-1").unwrap().size.width;
    assert!(content > whole + px(32.), "a wide tab keeps its padding");
    // Three tabs narrower than the label and a whole padding on each side.
    let rem = cx.update(|window, _| window.rem_size());
    let room = whole + rem * (24. + 8. + 8.) / 16.;
    let view = cx.update(|window, _| window.root::<Narrow>().flatten().unwrap());
    view.update(cx, |view, _| view.width = f32::from(room) * 3.);
    draw(cx);
    assert_eq!(cx.debug_bounds("tab-label-1").unwrap().size.width, whole);
}
