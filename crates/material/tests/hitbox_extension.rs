//! Contract tests: extra pointer targets retain Base's sole activation authority.
use std::{cell::Cell, rc::Rc};

use gpui::{
    Bounds, Context, Entity, Modifiers, Pixels, Style, TestAppContext, VisualTestContext, Window,
    div, point, prelude::*, px, size,
};

#[derive(Clone, Default)]
struct Records {
    first: Rc<Cell<usize>>,
    second: Rc<Cell<usize>>,
    hovered: Rc<Cell<bool>>,
    keyboard_height: Rc<Cell<f32>>,
}

#[derive(Default)]
struct Harness {
    records: Records,
    own_clip: bool,
    ancestor_clip: bool,
    overlap: bool,
    disabled: bool,
}

fn extension(bounds: Bounds<Pixels>, _: &Style, _: &Window) -> Option<Bounds<Pixels>> {
    Some(Bounds::new(
        point(bounds.origin.x, bounds.origin.y - px(8.)),
        size(bounds.size.width, px(48.)),
    ))
}

impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let first = self.records.first.clone();
        let second = self.records.second.clone();
        let hovered = self.records.hovered.clone();
        let keyboard_height = self.records.keyboard_height.clone();
        let button = gpui_base::Button::new("first")
            .w(px(80.))
            .h(px(32.))
            .disabled(self.disabled)
            .hitbox_extension(extension)
            .block_mouse_except_scroll()
            .when(self.own_clip, |button| button.overflow_hidden())
            .on_hover(move |value, _, _| hovered.set(*value))
            .on_click(move |event, _, _| {
                first.set(first.get() + 1);
                if let gpui::ClickEvent::Keyboard(event) = event {
                    keyboard_height.set(f32::from(event.bounds.size.height));
                }
            });
        div().size_full().p(px(20.)).child(
            div()
                .relative()
                .w(px(80.))
                .h(px(32.))
                .when(self.ancestor_clip, |parent| parent.overflow_hidden())
                .child(button)
                .when(self.overlap, |parent| {
                    parent.child(
                        gpui_base::Button::new("second")
                            .absolute()
                            .top(px(40.))
                            .w(px(80.))
                            .h(px(32.))
                            .hitbox_extension(extension)
                            .block_mouse_except_scroll()
                            .on_click(move |_, _, _| second.set(second.get() + 1)),
                    )
                }),
        )
    }
}

fn setup(cx: &mut TestAppContext) -> (Entity<Harness>, &mut VisualTestContext, Records) {
    cx.update(gpui_base::init);
    let records = Records::default();
    let copy = records.clone();
    let (view, cx) = cx.add_window_view(|_, _| Harness {
        records: copy,
        ..Default::default()
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    (view, cx, records)
}

#[gpui::test]
fn extension_activates_base_once_and_keeps_keyboard_geometry(cx: &mut TestAppContext) {
    let (_, cx, records) = setup(cx);
    cx.simulate_event(gpui::MouseMoveEvent {
        position: point(px(40.), px(14.)),
        pressed_button: None,
        modifiers: Modifiers::default(),
    });
    cx.run_until_parked();
    assert!(records.hovered.get(), "extension shares Base hover state");
    cx.simulate_click(point(px(40.), px(14.)), Modifiers::default());
    assert_eq!(records.first.get(), 1, "upper extension activates Base");
    cx.simulate_click(point(px(40.), px(58.)), Modifiers::default());
    assert_eq!(records.first.get(), 2, "lower extension activates Base");
    let keystroke =
        cx.update(|_, cx| gpui::Keystroke::parse_with_os("enter", cx.platform_info().os).unwrap());
    cx.simulate_event(gpui::KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(gpui::KeyUpEvent { keystroke });
    assert_eq!(records.first.get(), 3);
    assert_eq!(
        records.keyboard_height.get(),
        32.,
        "visual/layout bounds must remain 32px"
    );
    cx.simulate_click(point(px(40.), px(11.)), Modifiers::default());
    cx.simulate_click(point(px(40.), px(61.)), Modifiers::default());
    assert_eq!(records.first.get(), 3, "outside both target rectangles");
}

#[gpui::test]
fn extension_obeys_own_and_ancestor_clipping(cx: &mut TestAppContext) {
    let (view, cx, records) = setup(cx);
    for own_clip in [true, false] {
        view.update(cx, |view, cx| {
            view.own_clip = own_clip;
            view.ancestor_clip = !own_clip;
            cx.notify();
        });
        cx.simulate_click(point(px(40.), px(14.)), Modifiers::default());
        cx.simulate_click(point(px(40.), px(58.)), Modifiers::default());
        assert_eq!(records.first.get(), 0);
    }
    cx.simulate_click(point(px(40.), px(30.)), Modifiers::default());
    assert_eq!(records.first.get(), 1);
}

#[gpui::test]
fn overlapping_extensions_respect_occlusion_and_disabled(cx: &mut TestAppContext) {
    let (view, cx, records) = setup(cx);
    view.update(cx, |view, cx| {
        view.overlap = true;
        cx.notify();
    });
    cx.simulate_click(point(px(40.), px(55.)), Modifiers::default());
    assert_eq!(
        records.first.get(),
        0,
        "later target occludes earlier target"
    );
    assert_eq!(records.second.get(), 1);
    view.update(cx, |view, cx| {
        view.disabled = true;
        cx.notify();
    });
    cx.simulate_click(point(px(40.), px(14.)), Modifiers::default());
    assert_eq!(
        records.first.get(),
        0,
        "extension cannot bypass Base disabled behavior"
    );
}

#[gpui::test]
fn touch_extension_stays_in_base_gesture_recognizer(cx: &mut TestAppContext) {
    use gpui::InputEvent as _;
    let (_, cx, records) = setup(cx);
    for phase in [gpui::TouchPhase::Started, gpui::TouchPhase::Ended] {
        cx.update(|window, cx| {
            window.dispatch_event(
                gpui::TouchEvent {
                    id: gpui::TouchId(1),
                    phase,
                    position: point(px(40.), px(14.)),
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
        });
    }
    assert_eq!(
        records.first.get(),
        1,
        "extended raw touch is recognized as one Base tap"
    );
}
