//! Test harness for playground modules: a window showing one playground's
//! preview and configuration card, with helpers to render every scenario,
//! drive the preview and read the configuration back.
// The helpers serve every playground's tests; each uses a subset.
#![allow(dead_code)]
use super::super::playground::{Config, Playground, PlaygroundState, Value};
use gpui::{
    AppContext, Bounds, Modifiers, Pixels, Point, TestAppContext, VisualTestContext, WindowHandle,
    px, size,
};

/// One playground in a test window.
pub struct Harness {
    pub window: WindowHandle<PlaygroundState>,
    pub cx: VisualTestContext,
}

impl Harness {
    /// Open `playground` in a 1280x900 window and draw it.
    pub fn open(cx: &mut TestAppContext, playground: &'static Playground) -> Self {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let window = cx.open_window(size(px(1280.), px(900.)), move |window, cx| {
            PlaygroundState::new(playground, window, cx)
        });
        let cx = VisualTestContext::from_window(window.into(), cx);
        let mut harness = Self { window, cx };
        harness.settle();
        harness
    }

    /// Draw a few frames and run pending work.
    pub fn settle(&mut self) {
        for _ in 0..3 {
            self.cx
                .update_window(self.window.into(), |_, window, cx| {
                    window.refresh();
                    window.draw(cx).clear(cx);
                })
                .unwrap();
            self.cx.run_until_parked();
        }
    }

    /// Select each scenario ("Default" first) and draw it; `check` runs
    /// after each with the scenario index.
    pub fn render_every_scenario(&mut self, mut check: impl FnMut(&mut Self, usize)) {
        let count = self
            .window
            .update(&mut self.cx, |state, _, _| state.playground.scenarios.len())
            .unwrap();
        for scenario in 0..=count {
            self.window
                .update(&mut self.cx, |state, window, cx| {
                    state.select_scenario(scenario, window, cx)
                })
                .unwrap();
            self.settle();
            check(self, scenario);
        }
    }

    /// The current configuration.
    pub fn config(&mut self) -> Config {
        self.window
            .update(&mut self.cx, |state, _, _| state.config.clone())
            .unwrap()
    }

    /// Set control `id`, as the configuration card would, and draw.
    pub fn set(&mut self, id: &'static str, value: Value) {
        self.window
            .update(&mut self.cx, |state, window, cx| {
                state.set_value(id, value, window, cx)
            })
            .unwrap();
        self.settle();
    }

    /// The bounds of the element tagged `selector`, if drawn.
    pub fn bounds(&mut self, selector: &'static str) -> Option<Bounds<Pixels>> {
        self.cx.debug_bounds(selector)
    }

    /// The bounds of the element tagged `selector`; panics when not drawn.
    pub fn expect(&mut self, selector: &'static str) -> Bounds<Pixels> {
        self.bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is rendered"))
    }

    /// Click at a point and draw.
    pub fn click_at(&mut self, at: Point<Pixels>) {
        self.cx.simulate_click(at, Modifiers::none());
        self.settle();
    }

    /// Click the center of the element tagged `selector` and draw.
    pub fn click(&mut self, selector: &'static str) {
        let at = self.expect(selector).center();
        self.click_at(at);
    }

    /// Type keystrokes (e.g. `"right"`, `"enter"`, `"escape"`) and draw.
    pub fn keys(&mut self, keystrokes: &str) {
        self.cx.simulate_keystrokes(keystrokes);
        self.settle();
    }

    /// Advance the test clock by `ms` (for timed motion) and draw.
    pub fn advance(&mut self, ms: u64) {
        self.cx
            .executor()
            .advance_clock(std::time::Duration::from_millis(ms));
        self.settle();
    }

    /// The value of control `id`.
    pub fn value(&mut self, id: &'static str) -> Value {
        self.config().value(id).clone()
    }
}
