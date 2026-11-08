//! Kit's `crates/base/src/slider.rs`, whose crate is outside this workspace, so that its own
//! tests do not run here. What `MaterialSlider` relies on from its state.
extern crate gpui_kit as gpui;

use core::prelude::v1::test;
use gpui::{prelude::*, *};
use gpui_base::slider::{SliderEvent, SliderState, SliderValue};
use std::{cell::Cell, rc::Rc};

struct Holder(Entity<SliderState>);

impl Render for Holder {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

/// A slider of `state`, 100 px wide, in a window.
fn sized(
    state: SliderState,
    cx: &mut TestAppContext,
) -> (Entity<SliderState>, &mut VisualTestContext) {
    let (holder, cx) = cx.add_window_view(|_, cx| Holder(cx.new(|_| state)));
    let state = holder.read_with(cx, |holder, _| holder.0.clone());
    state.update(cx, |state, _| {
        state.set_bounds(Bounds::new(point(px(0.), px(0.)), size(px(100.), px(10.))));
    });
    (state, cx)
}

#[test]
fn min_can_be_set_above_the_default_max_before_max() {
    let state = SliderState::new().min(150.).max(200.).default_value(175.);
    assert_eq!(state.value(), SliderValue::Single(175.));
    assert_eq!(state.percentage(), 0.0..0.5);
}

#[gpui::test]
fn a_dropped_handle_stands_at_its_value(cx: &mut TestAppContext) {
    let (state, cx) = sized(SliderState::new().step(25.), cx);
    let drag = |x: f32, start: bool, cx: &mut VisualTestContext| {
        state.update_in(cx, |state, window, cx| {
            let position = point(px(x), px(0.));
            state.update_value_by_position(Axis::Horizontal, position, start, window, cx);
        })
    };
    drag(30., false, cx);
    state.read_with(cx, |state, _| {
        assert_eq!(state.value(), SliderValue::Single(25.));
        assert_eq!(state.percentage(), 0.0..0.25);
    });

    state.update_in(cx, |state, window, cx| {
        state.set_value((25., 75.), window, cx);
    });
    drag(40., true, cx);
    drag(60., false, cx);
    state.read_with(cx, |state, _| {
        assert_eq!(state.value(), SliderValue::Range(50., 50.));
        assert_eq!(state.percentage(), 0.5..0.5);
    });
}

#[gpui::test]
fn an_accessibility_step_reports_a_change(cx: &mut TestAppContext) {
    let (state, cx) = sized(SliderState::new().default_value(99.), cx);
    let changes = Rc::new(Cell::new(0));
    cx.update(|_, cx| {
        let changes = changes.clone();
        cx.subscribe(&state, move |_, event: &SliderEvent, _| {
            assert!(matches!(event, SliderEvent::Change(_)));
            changes.set(changes.get() + 1);
        })
        .detach();
    });
    let step = |delta: f32, cx: &mut VisualTestContext| {
        state.update_in(cx, |state, window, cx| {
            state.step_thumb(false, delta, window, cx)
        })
    };
    step(1., cx);
    assert_eq!(changes.get(), 1);
    assert_eq!(state.read_with(cx, |state, _| state.value().end()), 100.);
    // At its end the action changes nothing and reports nothing.
    step(1., cx);
    assert_eq!(changes.get(), 1);
    step(-1., cx);
    assert_eq!(changes.get(), 2);
}

#[gpui::test]
fn each_range_thumb_steps_by_itself(cx: &mut TestAppContext) {
    let (state, cx) = sized(SliderState::new().default_value((20., 60.)), cx);
    let step = |start: bool, delta: f32, cx: &mut VisualTestContext| {
        state.update_in(cx, |state, window, cx| {
            state.step_thumb(start, delta, window, cx)
        })
    };
    step(true, 5., cx);
    assert_eq!(
        state.read_with(cx, |state, _| state.value()),
        SliderValue::Range(25., 60.)
    );
    step(false, -5., cx);
    assert_eq!(
        state.read_with(cx, |state, _| state.value()),
        SliderValue::Range(25., 55.)
    );
    step(true, 100., cx);
    assert_eq!(
        state.read_with(cx, |state, _| state.value()),
        SliderValue::Range(55., 55.),
        "a thumb stops at the other"
    );
}

#[gpui::test]
fn a_zero_step_drags_continuously(cx: &mut TestAppContext) {
    for step in [0., -5., f32::NAN, f32::INFINITY] {
        let (state, cx) = sized(SliderState::new().step(step), cx);
        state.update_in(cx, |state, window, cx| {
            let position = point(px(30.), px(0.));
            state.update_value_by_position(Axis::Horizontal, position, false, window, cx);
        });
        state.read_with(cx, |state, _| {
            // It used to divide by zero, and the handle jumped to the max.
            let value = state.value().end();
            assert!((value - 30.).abs() < 1e-3, "step {step}: {value}");
            assert!((state.percentage().end - 0.3).abs() < 1e-5, "step {step}");
        });
    }
}

#[gpui::test]
fn an_accessibility_step_lands_on_the_step_grid(cx: &mut TestAppContext) {
    let (state, cx) = sized(SliderState::new().step(10.).default_value(33.), cx);
    let step = |delta: f32, cx: &mut VisualTestContext| {
        state.update_in(cx, |state, window, cx| {
            state.step_thumb(false, delta, window, cx)
        });
        state.read_with(cx, |state, _| state.value().end())
    };
    // As the keyboard's arrows do: 33 + 10 snaps to 40, not 43.
    assert_eq!(step(10., cx), 40.);
    state.update_in(cx, |state, window, cx| state.set_value(33., window, cx));
    assert_eq!(step(-10., cx), 20.);
}

#[test]
fn a_continuous_slider_steps_by_a_hundredth_of_its_range() {
    for step in [0., -1., f32::NAN] {
        let state = SliderState::new().min(0.).max(200.).step(step);
        assert_eq!(state.accessibility_step(), 2., "step {step}");
    }
    assert_eq!(SliderState::new().step(5.).accessibility_step(), 5.);
}

#[gpui::test]
fn stepping_leaves_an_invalid_value_or_range_alone(cx: &mut TestAppContext) {
    let cases = [
        SliderState::new().default_value(f32::NAN),
        SliderState::new().min(100.).max(0.).default_value(50.),
        SliderState::new().default_value((60., 20.)),
    ];
    for state in cases {
        let (state, cx) = sized(state, cx);
        let before = state.read_with(cx, |state, _| state.value());
        let changes = Rc::new(Cell::new(0));
        cx.update(|_, cx| {
            let changes = changes.clone();
            cx.subscribe(&state, move |_, _: &SliderEvent, _| {
                changes.set(changes.get() + 1)
            })
            .detach();
        });
        for start in [false, true] {
            state.update_in(cx, |state, window, cx| {
                state.step_thumb(start, 1., window, cx)
            });
        }
        let after = state.read_with(cx, |state, _| state.value());
        assert_eq!(
            (after.start().to_bits(), after.end().to_bits()),
            (before.start().to_bits(), before.end().to_bits()),
            "{before:?} became {after:?}"
        );
        assert_eq!(changes.get(), 0, "{before:?} reported a change");
    }
}

#[gpui::test]
fn a_range_thumb_snaps_to_the_grid_and_stops_at_the_other(cx: &mut TestAppContext) {
    let (state, cx) = sized(SliderState::new().step(10.).default_value((20., 25.)), cx);
    let step = |start: bool, delta: f32, cx: &mut VisualTestContext| {
        state.update_in(cx, |state, window, cx| {
            state.set_value((20., 25.), window, cx);
            state.step_thumb(start, delta, window, cx);
        });
        state.read_with(cx, |state, _| state.value())
    };
    // The end leaves 25 for the grid: 35 rounds to 40.
    assert_eq!(step(false, 10., cx), SliderValue::Range(20., 40.));
    // The start would land on 30, past the end: it stops at 25.
    assert_eq!(step(true, 10., cx), SliderValue::Range(25., 25.));
    // The end would snap to 20, onto the start, and stops there.
    assert_eq!(step(false, -10., cx), SliderValue::Range(20., 20.));
}
