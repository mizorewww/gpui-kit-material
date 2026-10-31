//! Loading indicator playground: md3.io's Loading indicator configuration
//! (contained, size, value, accessible label, indeterminate) on the real
//! `LoadingIndicator`.
use super::prelude::*;

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Contained",
            values: &[("contained", Set::Bool(true))],
        },
        Scenario {
            name: "Pull to refresh",
            values: &[
                ("contained", Set::Bool(true)),
                ("indeterminate", Set::Bool(false)),
                ("value", Set::Number(70.)),
                ("label", Set::Text("Pull to refresh")),
            ],
        },
        Scenario {
            name: "Large",
            values: &[("size", Set::Number(96.))],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Switch {
                    id: "contained",
                    label: "Contained",
                    default: false,
                },
                Control::Slider {
                    id: "size",
                    label: "Size",
                    min: 24.,
                    max: 240.,
                    step: 4.,
                    default: 48.,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Slider {
                    id: "value",
                    label: "Value",
                    min: 0.,
                    max: 100.,
                    step: 1.,
                    default: 50.,
                },
                Control::Text {
                    id: "label",
                    label: "Accessible label",
                    default: "Loading your content",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[Control::Switch {
                id: "indeterminate",
                label: "Indeterminate",
                default: true,
            }],
        },
    ],
    preview,
};

fn preview(
    config: &Config,
    _: &mut Preview,
    _: &mut Window,
    _: &mut Context<PlaygroundState>,
) -> AnyElement {
    let progress = (!config.bool("indeterminate")).then(|| config.number("value") / 100.);
    div()
        .debug_selector(|| "playground-loading-indicator".into())
        .child(
            LoadingIndicator::new("playground-loading-indicator", config.text("label"))
                .contained(config.bool("contained"))
                .size(config.number("size"))
                .progress(progress),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use core::prelude::v1::test;

    #[gpui::test]
    fn every_scenario_renders_and_size_follows_the_slider(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        render_every_scenario(&state, &mut cx);
        let small = element_bounds(&mut cx, "playground-loading-indicator");
        let expected = design(&mut cx, 48.);
        assert!(near(small.size.height, expected), "{:?}", small.size.height);
        set(&state, &mut cx, "size", Value::Number(120.));
        let large = element_bounds(&mut cx, "playground-loading-indicator");
        let expected = design(&mut cx, 120.);
        assert!(near(large.size.height, expected), "{:?}", large.size.height);
        // Determinate and contained render on the same node.
        toggle_switch(&mut cx, "indeterminate");
        assert!(!config(&state, &mut cx).bool("indeterminate"));
        toggle_switch(&mut cx, "contained");
        assert!(config(&state, &mut cx).bool("contained"));
        set(&state, &mut cx, "value", Value::Number(100.));
        settle_motion(&mut cx);
        assert_eq!(
            element_bounds(&mut cx, "playground-loading-indicator").size,
            large.size
        );
    }
}
