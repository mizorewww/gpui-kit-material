//! Helpers for playground tests: a window showing one playground's preview
//! and configuration panel, scenario sweeps and input simulation.
use super::super::playground::{Config, Playground, PlaygroundState, Value};
use gpui::{Bounds, Entity, Modifiers, Pixels, Point, TestAppContext, VisualTestContext};
use std::time::Duration;

pub fn open(
    playground: &'static Playground,
    cx: &mut TestAppContext,
) -> (Entity<PlaygroundState>, VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit_material::init(cx);
    });
    let window = cx.open_window(
        gpui::size(gpui::px(1280.), gpui::px(1600.)),
        |window, cx| PlaygroundState::new(playground, window, cx),
    );
    let state = window.root(cx).unwrap();
    let mut visual = VisualTestContext::from_window(window.into(), cx);
    settle(&mut visual);
    (state, visual)
}

pub fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    cx.run_until_parked();
}

pub fn settle(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        draw(cx);
    }
}

/// Let springs and fades finish.
pub fn settle_motion(cx: &mut VisualTestContext) {
    for _ in 0..4 {
        cx.executor().advance_clock(Duration::from_millis(500));
        draw(cx);
    }
}

pub fn element_bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("{selector} is rendered"))
}

pub fn click(cx: &mut VisualTestContext, at: Point<Pixels>) {
    cx.simulate_click(at, Modifiers::none());
    settle(cx);
}

pub fn config(state: &Entity<PlaygroundState>, cx: &mut VisualTestContext) -> Config {
    cx.read(|cx| state.read(cx).config.clone())
}

pub fn set(
    state: &Entity<PlaygroundState>,
    cx: &mut VisualTestContext,
    id: &'static str,
    value: Value,
) {
    cx.update(|window, cx| state.update(cx, |state, cx| state.set_value(id, value, window, cx)));
    settle(cx);
}

/// Select Default and every scenario in turn, drawing each.
pub fn render_every_scenario(state: &Entity<PlaygroundState>, cx: &mut VisualTestContext) {
    let count = cx.read(|cx| state.read(cx).playground.scenarios.len());
    for scenario in 0..=count {
        cx.update(|window, cx| {
            state.update(cx, |state, cx| state.select_scenario(scenario, window, cx))
        });
        settle(cx);
    }
    cx.update(|window, cx| state.update(cx, |state, cx| state.reset(window, cx)));
    settle(cx);
}

/// Draw every option of each listed choice control (one at a time).
pub fn render_every_option(
    state: &Entity<PlaygroundState>,
    cx: &mut VisualTestContext,
    ids: &[&'static str],
) {
    for id in ids {
        let count = cx.read(|cx| {
            state
                .read(cx)
                .playground
                .control(id)
                .and_then(|control| control.options())
                .map_or(0, |options| options.len())
        });
        assert!(count > 0, "{id} is a choice control");
        for index in 0..count {
            set(state, cx, id, Value::Choice(index));
        }
    }
    cx.update(|window, cx| state.update(cx, |state, cx| state.reset(window, cx)));
    settle(cx);
}

/// Click the switch of the `Control::Switch` `id` in the panel.
pub fn toggle_switch(cx: &mut VisualTestContext, id: &'static str) {
    let selector: &'static str = Box::leak(format!("playground-control-{id}").into_boxed_str());
    let row = element_bounds(cx, selector);
    click(cx, gpui::point(row.right() - gpui::px(20.), row.center().y));
}

/// `design` px at the window's current rem size.
pub fn design(cx: &mut VisualTestContext, design: f32) -> Pixels {
    cx.update(|window, _| gpui_kit_material::prelude::rem(design).to_pixels(window.rem_size()))
}

/// Whether two lengths agree to within layout rounding.
pub fn near(a: Pixels, b: Pixels) -> bool {
    (f32::from(a) - f32::from(b)).abs() < 1.
}
