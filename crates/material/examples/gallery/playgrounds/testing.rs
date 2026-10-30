//! Test helpers for playground modules: a window showing one playground's
//! preview and configuration card, scenario rendering and interaction.
use super::super::playground::{Config, Playground, PlaygroundState, Value};
use gpui::{Bounds, Entity, Modifiers, Pixels, TestAppContext, VisualTestContext};

/// Open `playground` in a test window with Kit and Material initialized
/// (reduced motion, so state changes land at once).
pub fn open<'a>(
    cx: &'a mut TestAppContext,
    playground: &'static Playground,
) -> (Entity<PlaygroundState>, &'a mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit_material::init(cx);
        cx.set_reduce_motion(true);
    });
    let (state, cx) = cx.add_window_view(|window, cx| PlaygroundState::new(playground, window, cx));
    settle(cx);
    (state, cx)
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

/// Render the defaults and every scenario, checking each shows `selector`.
pub fn every_scenario_renders(
    state: &Entity<PlaygroundState>,
    cx: &mut VisualTestContext,
    selector: &'static str,
) {
    let count = state.read_with(cx, |state, _| state.playground.scenarios.len());
    for scenario in 0..=count {
        cx.update(|window, cx| {
            state.update(cx, |state, cx| state.select_scenario(scenario, window, cx))
        });
        settle(cx);
        assert!(
            cx.debug_bounds(selector).is_some(),
            "scenario {scenario} renders {selector}"
        );
    }
}

pub fn rect(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("{selector} is rendered"))
}

pub fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let at = rect(cx, selector).center();
    cx.simulate_click(at, Modifiers::none());
    settle(cx);
}

pub fn config(state: &Entity<PlaygroundState>, cx: &mut VisualTestContext) -> Config {
    state.read_with(cx, |state, _| state.config.clone())
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

/// Select scenario `name`.
pub fn scenario(state: &Entity<PlaygroundState>, cx: &mut VisualTestContext, name: &str) {
    let index = state.read_with(cx, |state, _| {
        state
            .playground
            .scenarios
            .iter()
            .position(|scenario| scenario.name == name)
            .unwrap_or_else(|| panic!("no scenario {name}"))
            + 1
    });
    cx.update(|window, cx| state.update(cx, |state, cx| state.select_scenario(index, window, cx)));
    settle(cx);
}
