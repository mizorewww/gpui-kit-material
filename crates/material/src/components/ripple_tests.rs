//! The shared ripple across controls, and its promise not to take input.
use super::*;
use crate::components::button::waves_painted;
use core::prelude::v1::test;
use gpui::{
    AnyElement, Context, Entity, Modifiers, MouseButton, Render, ScrollDelta, ScrollEvent,
    ScrollHandle, TestAppContext, TouchEvent, TouchId, TouchPhase, VisualTestContext, Window, div,
    point, prelude::*, px,
};
use std::cell::Cell;
use std::rc::Rc;

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

fn touch(cx: &mut VisualTestContext, phase: TouchPhase, position: gpui::Point<gpui::Pixels>) {
    cx.simulate_event(TouchEvent {
        id: TouchId(1),
        phase,
        position,
        ..Default::default()
    });
}

/// One slot per control, each under a selector.
struct Controls;
impl Render for Controls {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let slot = |name: &'static str, child: AnyElement| {
            div()
                .w(px(200.))
                .debug_selector(move || name.into())
                .child(child)
        };
        div()
            .flex()
            .flex_col()
            .child(slot(
                "checkbox",
                MaterialCheckbox::new("checkbox", "Check").into_any_element(),
            ))
            .child(slot(
                "switch",
                MaterialSwitch::new("switch", "Switch").into_any_element(),
            ))
            .child(slot(
                "radio",
                MaterialRadio::new("radio", "Radio").into_any_element(),
            ))
            .child(slot(
                "chip",
                AssistChip::new("chip", "Chip")
                    .on_click(|_, _, _| {})
                    .into_any_element(),
            ))
            .child(slot(
                "row",
                ListItem::new("row", "Row")
                    .on_click(|_, _, _| {})
                    .into_any_element(),
            ))
    }
}

#[gpui::test]
fn every_control_answers_a_press_with_a_wave(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
    });
    let (_, cx) = cx.add_window_view(|_, _| Controls);
    draw(cx);
    for name in ["checkbox", "switch", "radio", "chip", "row"] {
        let bounds = cx.debug_bounds(name).unwrap();
        // The control sits at the slot's start.
        let at = point(bounds.left() + px(20.), bounds.center().y);
        let before = waves_painted();
        cx.simulate_mouse_move(at, None, Modifiers::default());
        cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::default());
        draw(cx);
        // A wave has no size until its expansion has started.
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(100));
        draw(cx);
        assert!(waves_painted() > before, "{name} paints a wave while held");
        cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::default());
        draw(cx);
    }
}

struct ScrollHarness {
    scroll: ScrollHandle,
    pressed: Rc<Cell<Option<Entity<bool>>>>,
    clicks: Rc<Cell<usize>>,
}
impl Render for ScrollHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pressed = window.use_keyed_state("pressed", cx, |_, _| false);
        self.pressed.set(Some(pressed.clone()));
        let clicks = self.clicks.clone();
        div()
            .id("list")
            .debug_selector(|| "list".into())
            .w(px(100.))
            .h(px(100.))
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .child(
                Ripple::new(
                    "item-ripple",
                    div()
                        .id("item")
                        .h(px(400.))
                        .debug_selector(|| "item".into())
                        .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
                )
                .touch_pressed(&pressed),
            )
    }
}

fn scroll_harness(cx: &mut TestAppContext) -> (Entity<ScrollHarness>, &mut VisualTestContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.add_window_view(|_, _| ScrollHarness {
        scroll: ScrollHandle::new(),
        pressed: Rc::default(),
        clicks: Rc::default(),
    })
}

fn pressed(view: &Entity<ScrollHarness>, cx: &mut VisualTestContext) -> bool {
    let pressed = view.read_with(cx, |h, _| {
        let entity = h.pressed.take();
        h.pressed.set(entity.clone());
        entity.expect("drawn")
    });
    pressed.read_with(cx, |pressed, _| *pressed)
}

#[gpui::test]
fn a_touch_scroll_reaches_the_container_and_cancels_the_press(cx: &mut TestAppContext) {
    let (view, cx) = scroll_harness(cx);
    draw(cx);
    let at = cx.debug_bounds("list").unwrap().center();
    touch(cx, TouchPhase::Started, at);
    draw(cx);
    assert!(pressed(&view, cx), "touch-down gives press feedback");
    cx.simulate_event(ScrollEvent {
        position: at,
        delta: ScrollDelta::Pixels(point(px(0.), px(-50.))),
        modifiers: Modifiers::default(),
        touch_phase: TouchPhase::Moved,
        ..Default::default()
    });
    draw(cx);
    let offset = view.read_with(cx, |h, _| h.scroll.offset());
    assert_eq!(offset.y, px(-50.), "the ripple did not swallow the scroll");
    assert!(!pressed(&view, cx), "a scroll cancels the press");
}

#[gpui::test]
fn a_still_touch_the_system_cancels_does_not_stay_pressed(cx: &mut TestAppContext) {
    let (view, cx) = scroll_harness(cx);
    draw(cx);
    let at = cx.debug_bounds("list").unwrap().center();
    touch(cx, TouchPhase::Started, at);
    draw(cx);
    assert!(pressed(&view, cx));
    // GPUI publishes no event for this cancel.
    touch(cx, TouchPhase::Cancelled, at);
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(6));
    cx.run_until_parked();
    draw(cx);
    assert!(!pressed(&view, cx), "the hold limit released it");
}

#[gpui::test]
fn a_touch_moving_past_the_slop_cancels_and_one_that_stays_holds(cx: &mut TestAppContext) {
    let (view, cx) = scroll_harness(cx);
    draw(cx);
    let at = cx.debug_bounds("list").unwrap().center();
    touch(cx, TouchPhase::Started, at);
    draw(cx);
    touch(cx, TouchPhase::Moved, at + point(px(3.), px(3.)));
    draw(cx);
    assert!(pressed(&view, cx), "a small move is still a press");
    touch(cx, TouchPhase::Moved, at + point(px(0.), px(30.)));
    draw(cx);
    assert!(!pressed(&view, cx), "a long move is a scroll");
    touch(cx, TouchPhase::Ended, at + point(px(0.), px(30.)));
    draw(cx);
    assert!(!pressed(&view, cx));
}

#[gpui::test]
fn a_specimen_or_disabled_ripple_starts_no_wave(cx: &mut TestAppContext) {
    struct Inert;
    impl Render for Inert {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .flex_col()
                .child(
                    Ripple::new("off", div().size(px(40.)).debug_selector(|| "off".into()))
                        .disabled(true),
                )
                .child(
                    Ripple::new("spec", div().size(px(40.)).debug_selector(|| "spec".into()))
                        .specimen(true),
                )
        }
    }
    let (_, cx) = cx.add_window_view(|_, _| Inert);
    draw(cx);
    for name in ["off", "spec"] {
        let at = cx.debug_bounds(name).unwrap().center();
        let before = waves_painted();
        cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::default());
        draw(cx);
        assert_eq!(waves_painted(), before, "{name} stays still");
        cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::default());
    }
}

#[gpui::test]
fn a_long_press_ending_releases_the_wave(cx: &mut TestAppContext) {
    let (view, cx) = scroll_harness(cx);
    draw(cx);
    let at = cx.debug_bounds("list").unwrap().center();
    touch(cx, TouchPhase::Started, at);
    draw(cx);
    assert!(pressed(&view, cx));
    cx.simulate_event(gpui::LongPressEvent {
        phase: TouchPhase::Ended,
        start_position: at,
        position: at,
    });
    draw(cx);
    assert!(!pressed(&view, cx));
}

struct IdleRipple;
impl Render for IdleRipple {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div()
            .size_full()
            .child(Ripple::new("idle", div().size(px(40.))).into_any_element())
    }
}

/// Every ripple hears every wheel, release and exit in the window. Updating an
/// entity marks what read it as changed, so one with nothing to end must not
/// write: a scrolling list would render all the rows it holds on each frame.
#[gpui::test]
fn an_idle_ripple_writes_nothing_when_drawn_or_on_global_events(cx: &mut TestAppContext) {
    use crate::components::button::ripple_writes;
    cx.update(gpui_base::init);
    let (_, cx) = cx.add_window_view(|_, _| IdleRipple);
    let at = point(px(20.), px(20.));
    let away = point(px(300.), px(300.));
    draw(cx);
    let settled = ripple_writes();
    draw(cx);
    cx.simulate_event(ScrollEvent {
        position: at,
        delta: ScrollDelta::Pixels(point(px(0.), px(-8.))),
        ..Default::default()
    });
    cx.simulate_mouse_up(away, MouseButton::Left, Modifiers::default());
    cx.simulate_event(gpui::MouseExitEvent {
        position: away,
        pressed_button: None,
        modifiers: Modifiers::default(),
    });
    draw(cx);
    assert_eq!(ripple_writes(), settled, "nothing to reconcile or release");
    // The control: a held press still writes, and its release ends it.
    cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::default());
    draw(cx);
    let pressed = ripple_writes();
    assert!(pressed > settled, "a press reconciles its wave");
    cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::default());
    assert!(ripple_writes() > pressed, "a held press is released");
}

#[gpui::test]
fn deactivating_the_window_releases_a_held_press(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(|_, _| Controls);
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    draw(cx);
    let slot = cx.debug_bounds("checkbox").unwrap();
    let on_box = point(slot.left() + px(20.), slot.center().y);
    cx.simulate_mouse_down(on_box, MouseButton::Left, Modifiers::default());
    draw(cx);
    let painted = waves_painted();
    draw(cx);
    assert!(waves_painted() > painted, "the press holds its wave");
    cx.deactivate_window();
    // The release may never come; the wave fades and goes without it.
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(2));
    cx.run_until_parked();
    draw(cx);
    let settled = waves_painted();
    draw(cx);
    assert_eq!(waves_painted(), settled, "no wave is left held");
}
