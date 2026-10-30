//! Progress playground: md3.io's Progress configuration (variant, shape,
//! thickness, circular size, stop indicator, value, buffer, percentage,
//! accessible label, indeterminate, disabled) on the real `LinearProgress`
//! and `CircularProgress`.
//!
//! The buffer ends the linear track: past it only the stop indicator
//! remains (the Figma kit has no buffer; md3.io's option is kept). A
//! circular indicator has no buffer or stop indicator, as in M3.
use super::prelude::*;

/// md3.io's linear preview width.
const LINEAR_WIDTH: f32 = 352.;

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Wavy upload",
            values: &[
                ("shape", Set::Option("wavy")),
                ("value", Set::Number(62.)),
                ("percentage", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Streaming buffer",
            values: &[
                ("thickness", Set::Option("thick")),
                ("value", Set::Number(30.)),
                ("buffer", Set::Number(55.)),
                ("label", Set::Text("Buffering video")),
            ],
        },
        Scenario {
            name: "Circular loading",
            values: &[
                ("variant", Set::Option("circular")),
                ("shape", Set::Option("wavy")),
                ("indeterminate", Set::Bool(true)),
                ("label", Set::Text("Loading")),
            ],
        },
        Scenario {
            name: "Paused download",
            values: &[
                ("variant", Set::Option("circular")),
                ("value", Set::Number(80.)),
                ("percentage", Set::Bool(true)),
                ("disabled", Set::Bool(true)),
                ("label", Set::Text("Download paused")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Segmented {
                    id: "variant",
                    label: "Variant",
                    options: &["linear", "circular"],
                    default: 0,
                },
                Control::Segmented {
                    id: "shape",
                    label: "Shape",
                    options: &["flat", "wavy"],
                    default: 0,
                },
                Control::Segmented {
                    id: "thickness",
                    label: "Thickness",
                    options: &["thin", "thick"],
                    default: 0,
                },
                Control::Slider {
                    id: "size",
                    label: "Circular size",
                    min: 24.,
                    max: 96.,
                    step: 4.,
                    default: 48.,
                },
                Control::Switch {
                    id: "stop",
                    label: "Stop indicator",
                    default: true,
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
                    default: 45.,
                },
                Control::Slider {
                    id: "buffer",
                    label: "Buffer",
                    min: 0.,
                    max: 100.,
                    step: 1.,
                    // md3.io starts at 70 yet previews a full track; the
                    // full buffer reproduces its preview.
                    default: 100.,
                },
                Control::Switch {
                    id: "percentage",
                    label: "Show percentage",
                    default: false,
                },
                Control::Text {
                    id: "label",
                    label: "Accessible label",
                    default: "Uploading files",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "indeterminate",
                    label: "Indeterminate",
                    default: false,
                },
                Control::Switch {
                    id: "disabled",
                    label: "Disabled",
                    default: false,
                },
            ],
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
    let value = config.number("value");
    let indeterminate = config.bool("indeterminate");
    let fraction = (!indeterminate).then_some(value / 100.);
    let shape = if config.option("shape") == "wavy" {
        ProgressShape::Wavy
    } else {
        ProgressShape::Flat
    };
    let thickness = if config.option("thickness") == "thick" {
        PROGRESS_THICK_THICKNESS
    } else {
        PROGRESS_THICKNESS
    };
    let label = config.text("label");
    let disabled = config.bool("disabled");
    let circular = config.option("variant") == "circular";
    // The playground keys each variant apart so a switch starts fresh.
    let indicator = if circular {
        CircularProgress::new("playground-progress-circular", label)
            .value(fraction)
            .shape(shape)
            .thickness(thickness)
            .size(config.number("size"))
            .disabled(disabled)
            .into_any_element()
    } else {
        div()
            .debug_selector(|| "playground-progress-track".into())
            .w_full()
            .max_w(rem(LINEAR_WIDTH))
            .child(
                LinearProgress::new("playground-progress-linear", label)
                    .value(fraction)
                    .shape(shape)
                    .thickness(thickness)
                    .stop_indicator(config.bool("stop"))
                    .buffer(Some(config.number("buffer") / 100.))
                    .disabled(disabled),
            )
            .into_any_element()
    };
    let percentage = (config.bool("percentage") && !indeterminate).then(|| {
        div()
            .debug_selector(|| "playground-progress-percentage".into())
            .map(|text| type_role(text, TypeRole::LabelLarge))
            .text_color(color(if disabled {
                ColorRole::ON_SURFACE_VARIANT
            } else {
                ColorRole::ON_SURFACE
            }))
            .child(SharedString::from(format!("{}%", value.round())))
    });
    div()
        .debug_selector(|| "playground-progress".into())
        .flex()
        .items_center()
        .gap(rem(16.))
        .when(circular, |row| row.flex_col())
        .child(indicator)
        .children(percentage)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use core::prelude::v1::test;

    #[gpui::test]
    fn every_scenario_and_option_renders(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        render_every_scenario(&state, &mut cx);
        render_every_option(&state, &mut cx, &["variant", "shape", "thickness"]);
        for indeterminate in [true, false] {
            set(&state, &mut cx, "variant", Value::Choice(1));
            set(&state, &mut cx, "indeterminate", Value::Bool(indeterminate));
            set(&state, &mut cx, "variant", Value::Choice(0));
        }
    }

    #[gpui::test]
    fn size_and_percentage_change_the_preview(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        let linear = element_bounds(&mut cx, "playground-progress-track");
        let expected = design(&mut cx, LINEAR_WIDTH);
        assert!(near(linear.size.width, expected), "{:?}", linear.size.width);
        assert!(cx.debug_bounds("playground-progress-percentage").is_none());
        // The percentage label follows the track.
        toggle_switch(&mut cx, "percentage");
        assert!(config(&state, &mut cx).bool("percentage"));
        let label = element_bounds(&mut cx, "playground-progress-percentage");
        assert!(label.left() >= linear.right());
        set(&state, &mut cx, "percentage", Value::Bool(false));
        // Circular follows the size slider.
        set(&state, &mut cx, "variant", Value::Choice(1));
        assert_eq!(
            element_bounds(&mut cx, "playground-progress").size.height,
            gpui::px(48.)
        );
        set(&state, &mut cx, "size", Value::Number(96.));
        assert_eq!(
            element_bounds(&mut cx, "playground-progress").size.height,
            gpui::px(96.)
        );
    }
}
