//! Cross-control integration test: switch, search field, slider and button
//! keep Base input and disabled semantics together. The controls themselves
//! live in their own modules (`switch.rs`, `search.rs`, `slider.rs`).
#[cfg(test)]
mod tests {
    use super::super::{
        ButtonVariant, MaterialSlider, MaterialSwitch, SearchField, material_button,
    };
    use core::prelude::v1::test;
    use gpui::{prelude::*, *};
    use gpui_base::{input::InputState, slider::SliderState};

    struct ControlsHarness {
        checked: bool,
        disabled: bool,
        changes: usize,
        search: Entity<InputState>,
        slider: Entity<SliderState>,
    }
    impl Render for ControlsHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .flex_col()
                .w(px(240.))
                .gap(px(12.))
                .child(
                    div()
                        .debug_selector(|| "test-switch".into())
                        .w(px(52.))
                        .h(px(32.))
                        .child(
                            MaterialSwitch::new("switch", "Notifications")
                                .checked(self.checked)
                                .disabled(self.disabled)
                                .on_change(cx.listener(|this, value: &bool, _, cx| {
                                    this.checked = *value;
                                    this.changes += 1;
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    div()
                        .debug_selector(|| "test-search".into())
                        .child(SearchField::new(&self.search)),
                )
                .child(
                    div()
                        .debug_selector(|| "test-slider".into())
                        .child(MaterialSlider::new(&self.slider, "Weight").disabled(self.disabled)),
                )
                .child(
                    material_button("disabled-action", ButtonVariant::Filled)
                        .disabled(self.disabled)
                        .child("Action")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.changes += 1;
                            cx.notify();
                        })),
                )
        }
    }
    fn draw(cx: &mut VisualTestContext) {
        cx.update(|w, cx| {
            w.refresh();
            w.draw(cx).clear(cx);
        });
    }
    #[gpui::test]
    fn controls_keep_base_input_and_disabled_semantics(cx: &mut TestAppContext) {
        cx.update(|cx| {
            // SearchField renders Kit's Input, which reads Kit's theme.
            gpui_kit::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|w, cx| ControlsHarness {
            checked: false,
            disabled: false,
            changes: 0,
            search: cx.new(|cx| InputState::new(w, cx).placeholder("Search")),
            slider: cx.new(|_| SliderState::new().max(100.).default_value(0.)),
        });
        draw(cx);
        let toggle = cx.debug_bounds("test-switch").unwrap().center();
        cx.simulate_click(toggle, Modifiers::default());
        draw(cx);
        cx.update(|_, cx| {
            assert!(view.read(cx).checked);
            assert_eq!(view.read(cx).changes, 1);
        });
        let search = cx.debug_bounds("test-search").unwrap().center();
        cx.simulate_click(search, Modifiers::default());
        cx.simulate_input("home");
        draw(cx);
        cx.update(|_, cx| assert_eq!(view.read(cx).search.read(cx).value().as_ref(), "home"));
        let bounds = cx.debug_bounds("test-slider").unwrap();
        cx.simulate_click(
            point(bounds.left() + bounds.size.width * 0.75, bounds.center().y),
            Modifiers::default(),
        );
        draw(cx);
        cx.update(|_, cx| assert!((view.read(cx).slider.read(cx).value().end() - 75.).abs() < 2.));
        view.update(cx, |view, cx| {
            view.disabled = true;
            cx.notify();
        });
        draw(cx);
        cx.simulate_click(toggle, Modifiers::default());
        cx.simulate_click(
            point(bounds.left() + bounds.size.width * 0.1, bounds.center().y),
            Modifiers::default(),
        );
        draw(cx);
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).changes, 1);
            assert!((view.read(cx).slider.read(cx).value().end() - 75.).abs() < 2.);
        });
    }
}
