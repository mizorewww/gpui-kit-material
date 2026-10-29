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
}

impl Render for Harness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let disabled = self.disabled;
        div().w(px(300.)).child(
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
