//! Contract tests for the small gpui-pre raw-touch observation patch.
//! Observing a press must not take ownership of Base's tap recognizer.
extern crate gpui_kit as gpui;

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    Context, Entity, InputEvent, Modifiers, StyleRefinement, TestAppContext, TouchEvent, TouchId,
    TouchPhase, VisualTestContext, Window, canvas, div, point, prelude::*, px,
};

#[derive(Clone, Default)]
struct Records {
    events: Rc<RefCell<Vec<(usize, TouchPhase, bool)>>>,
    clicks: Rc<Cell<usize>>,
    paints: Rc<Cell<usize>>,
}

struct ObservedButton {
    records: Records,
    interfere: bool,
}

impl Render for ObservedButton {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let clicks = self.records.clicks.clone();
        let records = self.records.clone();
        let interfere = self.interfere;
        gpui_base::Button::new("observed")
            .w(px(120.))
            .h(px(40.))
            .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
            .child(
                canvas(
                    |_, _, _| (),
                    move |_, _, window, _| {
                        records.paints.set(records.paints.get() + 1);
                        for observer in 0..2 {
                            let events = records.events.clone();
                            window.observe_touch_event(move |event, window, cx| {
                                events.borrow_mut().push((
                                    observer,
                                    event.phase,
                                    window.default_prevented(),
                                ));
                                if interfere && observer == 0 {
                                    window.prevent_default();
                                    cx.stop_propagation();
                                }
                            });
                        }
                    },
                )
                .size_full(),
            )
    }
}

struct Harness {
    child: Entity<ObservedButton>,
    visible: bool,
}
impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().when(self.visible, |root| {
            root.child(
                self.child
                    .clone()
                    .cached(StyleRefinement::default().w(px(120.)).h(px(40.))),
            )
        })
    }
}

fn setup(
    cx: &mut TestAppContext,
    interfere: bool,
) -> (Entity<Harness>, &mut VisualTestContext, Records) {
    cx.update(gpui_base::init);
    let records = Records::default();
    let copied = records.clone();
    let (view, cx) = cx.add_window_view(|_, cx| Harness {
        child: cx.new(|_| ObservedButton {
            records: copied,
            interfere,
        }),
        visible: true,
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    (view, cx, records)
}

fn touch(cx: &mut VisualTestContext, id: u64, phase: TouchPhase) {
    cx.update(|window, cx| {
        let result = window.dispatch_event(
            TouchEvent {
                id: TouchId(id),
                phase,
                position: point(px(20.), px(20.)),
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        let _ = result;
    });
}

#[gpui::test]
fn all_raw_phases_are_observed_once_without_claiming_base_taps(cx: &mut TestAppContext) {
    let (_, cx, records) = setup(cx, false);
    for (id, phase) in [
        (1, TouchPhase::Started),
        (1, TouchPhase::Moved),
        (1, TouchPhase::Ended),
        (2, TouchPhase::Started),
        (2, TouchPhase::Cancelled),
    ] {
        touch(cx, id, phase);
    }
    let phases = [
        TouchPhase::Started,
        TouchPhase::Moved,
        TouchPhase::Ended,
        TouchPhase::Started,
        TouchPhase::Cancelled,
    ];
    let expected: Vec<_> = phases
        .into_iter()
        .flat_map(|phase| [(0, phase, false), (1, phase, false)])
        .collect();
    assert_eq!(*records.events.borrow(), expected);
    assert_eq!(
        records.clicks.get(),
        1,
        "ended tap activates once; cancellation never activates"
    );
}

#[gpui::test]
fn observer_consumption_is_discarded_and_mouse_keyboard_are_unaffected(cx: &mut TestAppContext) {
    let (_, cx, records) = setup(cx, true);
    touch(cx, 1, TouchPhase::Started);
    touch(cx, 1, TouchPhase::Ended);
    assert_eq!(
        records.clicks.get(),
        1,
        "observer cannot cancel Base's synthesized tap"
    );
    assert_eq!(
        records.events.borrow().len(),
        4,
        "stop_propagation cannot hide the event from another observer"
    );
    assert!(
        records
            .events
            .borrow()
            .iter()
            .all(|(_, _, prevented)| !prevented)
    );
    let count = records.events.borrow().len();
    cx.simulate_click(point(px(20.), px(20.)), Modifiers::default());
    assert_eq!(records.clicks.get(), 2);
    let keystroke =
        cx.update(|_, cx| gpui::Keystroke::parse_with_os("enter", cx.platform_info().os).unwrap());
    cx.simulate_event(gpui::KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(gpui::KeyUpEvent { keystroke });
    assert_eq!(records.clicks.get(), 3);
    assert_eq!(
        records.events.borrow().len(),
        count,
        "mouse and keyboard never enter the raw touch observer path"
    );
}

#[gpui::test]
fn cached_paint_keeps_one_registration_and_removal_drops_it(cx: &mut TestAppContext) {
    let (view, cx, records) = setup(cx, false);
    // Warm the touch input modality; its first transition deliberately refreshes the window.
    touch(cx, 1, TouchPhase::Started);
    touch(cx, 1, TouchPhase::Cancelled);
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let paints = records.paints.get();
    records.events.borrow_mut().clear();
    for id in 2..5 {
        view.update(cx, |_, cx| cx.notify());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        touch(cx, id, TouchPhase::Started);
        touch(cx, id, TouchPhase::Cancelled);
    }
    assert_eq!(
        records.paints.get(),
        paints,
        "cached child must genuinely reuse paint for this test"
    );
    assert_eq!(
        records.events.borrow().len(),
        12,
        "two callbacks per raw event, even after repeated cached reuse"
    );
    view.update(cx, |view, cx| {
        view.visible = false;
        cx.notify();
    });
    // Dispatch must flush the dirty frame before observing, without an explicit draw here.
    touch(cx, 5, TouchPhase::Started);
    touch(cx, 5, TouchPhase::Cancelled);
    assert_eq!(
        records.events.borrow().len(),
        12,
        "removed paint registrations must not survive"
    );
}
