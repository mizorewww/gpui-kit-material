//! Test harness for playground modules: one playground's preview in a
//! canvas sized like the Gallery's preview pane (a centering flex column),
//! with helpers to change controls, click and settle motion.
use super::super::playground::{Config, Playground, PlaygroundState, Value};
use gpui::{prelude::*, *};
use std::time::Duration;

/// The preview canvas, as the Gallery hosts it.
pub struct Canvas {
    pub state: Entity<PlaygroundState>,
}

impl Render for Canvas {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // One design px per pixel.
        window.set_rem_size(px(gpui_kit_material::tokens::typography::BASE_PX));
        let preview = self
            .state
            .update(cx, |state, cx| state.render_preview(window, cx));
        div()
            .size(px(800.))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .p(px(24.))
            .child(preview)
    }
}

pub fn open(playground: &'static Playground, cx: &mut TestAppContext) -> PlaygroundHarness {
    cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit_material::init(cx);
        cx.set_reduce_motion(false);
    });
    let window = cx.add_window(|window, cx| Canvas {
        state: cx.new(|cx| PlaygroundState::new(playground, window, cx)),
    });
    let state = window
        .read_with(cx, |canvas, _| canvas.state.clone())
        .unwrap();
    let mut harness = PlaygroundHarness {
        cx: VisualTestContext::from_window(window.into(), cx),
        state,
    };
    harness.settle();
    harness
}

pub struct PlaygroundHarness {
    pub cx: VisualTestContext,
    pub state: Entity<PlaygroundState>,
}

impl PlaygroundHarness {
    pub fn draw(&mut self) {
        self.cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        self.cx.run_until_parked();
    }

    /// Draw, let every transition finish, and draw again.
    pub fn settle(&mut self) {
        self.draw();
        self.cx.executor().advance_clock(Duration::from_secs(3));
        for _ in 0..3 {
            self.draw();
        }
    }

    pub fn set(&mut self, id: &'static str, value: Value) {
        let state = self.state.clone();
        self.cx.update(|window, cx| {
            state.update(cx, |state, cx| state.set_value(id, value, window, cx))
        });
        self.settle();
    }

    pub fn scenario(&mut self, index: usize) {
        let state = self.state.clone();
        self.cx.update(|window, cx| {
            state.update(cx, |state, cx| state.select_scenario(index, window, cx))
        });
        self.settle();
    }

    pub fn config(&mut self) -> Config {
        self.state
            .read_with(&self.cx, |state, _| state.config.clone())
    }

    pub fn bounds(&mut self, selector: &'static str) -> Bounds<Pixels> {
        self.cx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is rendered"))
    }

    pub fn click(&mut self, at: Point<Pixels>) {
        self.cx.simulate_click(at, Modifiers::none());
        self.settle();
    }

    pub fn click_on(&mut self, selector: &'static str) {
        let at = self.bounds(selector).center();
        self.click(at);
    }

    /// Render every scenario (Default first) and check `selector` shows.
    pub fn renders_every_scenario(&mut self, selector: &'static str) {
        let count = self
            .state
            .read_with(&self.cx, |state, _| state.playground.scenarios.len());
        for index in 0..=count {
            self.scenario(index);
            assert!(
                self.cx.debug_bounds(selector).is_some(),
                "scenario {index} renders {selector}"
            );
        }
    }
}
