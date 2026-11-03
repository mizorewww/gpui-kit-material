//! Stepper playground: no reference defines a stepper, so its controls are the real
//! `Stepper`'s options (how many steps, which is current, how long their labels are) and the
//! width it is drawn in, where long labels wrap.
use super::prelude::*;

const SHORT: [&str; 5] = ["Source", "Service", "Account", "Review", "Done"];
const LONG: [&str; 5] = [
    "Choose the source of your documents",
    "Connect the service to convert with",
    "Sign in to your account",
    "Review what will be converted",
    "Finish and open the result",
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "First step",
            values: &[("current", Set::Number(1.))],
        },
        Scenario {
            name: "All finished",
            values: &[("current", Set::Number(6.))],
        },
        Scenario {
            name: "Narrow, long labels",
            values: &[
                ("labels", Set::Option("long")),
                ("width", Set::Number(320.)),
                ("current", Set::Number(2.)),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Progress",
            controls: &[
                Control::Slider {
                    id: "steps",
                    label: "Steps",
                    min: 1.,
                    max: 5.,
                    step: 1.,
                    default: 3.,
                },
                Control::Slider {
                    id: "current",
                    label: "Current step",
                    min: 1.,
                    max: 6.,
                    step: 1.,
                    default: 2.,
                },
            ],
        },
        Section {
            title: "Layout",
            controls: &[
                Control::Segmented {
                    id: "labels",
                    label: "Labels",
                    options: &["short", "long"],
                    default: 0,
                },
                Control::Slider {
                    id: "width",
                    label: "Width",
                    min: 240.,
                    max: 720.,
                    step: 8.,
                    default: 560.,
                },
            ],
        },
    ],
    preview,
    code: None,
};

fn preview(
    config: &Config,
    _: &mut Preview,
    _: &mut Window,
    _: &mut Context<PlaygroundState>,
) -> AnyElement {
    let labels = if config.option("labels") == "long" {
        LONG
    } else {
        SHORT
    };
    let steps = (config.number("steps") as usize).clamp(1, labels.len());
    div()
        .debug_selector(|| "playground-stepper".into())
        .w(rem(config.number("width")))
        .max_w_full()
        .child(
            Stepper::new("playground-stepper", "Setup progress")
                .steps(labels.into_iter().take(steps))
                .current((config.number("current") as usize).saturating_sub(1))
                .state_labels("Finished", "Current", "Upcoming"),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use core::prelude::v1::test;

    #[gpui::test]
    fn every_scenario_renders_and_the_width_follows_the_slider(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        render_every_scenario(&state, &mut cx);
        set(&state, &mut cx, "labels", Value::Choice(0));
        set(&state, &mut cx, "width", Value::Number(320.));
        let narrow = element_bounds(&mut cx, "playground-stepper");
        let expected = design(&mut cx, 320.);
        assert!(near(narrow.size.width, expected), "{:?}", narrow.size.width);
    }

    #[gpui::test]
    fn long_labels_wrap_where_short_ones_do_not(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        set(&state, &mut cx, "width", Value::Number(320.));
        let short = element_bounds(&mut cx, "playground-stepper").size.height;
        set(&state, &mut cx, "labels", Value::Choice(1));
        let long = element_bounds(&mut cx, "playground-stepper").size.height;
        assert!(long > short, "{long:?} is not taller than {short:?}");
    }
}
