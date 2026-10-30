//! Slider playground: md3.io's Slider configuration (size, orientation,
//! color, tick marks, value indicator, label, value and range end, inset
//! icon, standard/centered/range variants, step, disabled) on the real
//! `MaterialSlider` over a Base slider state per step.
//!
//! Dragging or keying the preview writes Value (and Range end) back; the
//! controls move the preview's thumbs. The inset icon shows from the
//! medium size up, as M3 Expressive draws it.
use super::prelude::*;
use gpui_base::slider::{SliderEvent, SliderState, SliderValue};

const SIZES: [SliderSize; 5] = SliderSize::ALL;
const COLORS: [SliderColor; 4] = SliderColor::ALL;
const STEPS: [f32; 3] = [1., 5., 10.];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Call volume",
            values: &[
                ("size", Set::Option("M")),
                ("label", Set::Text("Call volume")),
                ("value", Set::Number(70.)),
                ("icon", Set::Bool(true)),
                ("step", Set::Option("1")),
            ],
        },
        Scenario {
            name: "Font size",
            values: &[
                ("size", Set::Option("S")),
                ("label", Set::Text("Font size")),
                ("value", Set::Number(50.)),
                ("ticks", Set::Bool(true)),
                ("color", Set::Option("secondary")),
            ],
        },
        Scenario {
            name: "Price budget",
            values: &[
                ("label", Set::Text("Price budget")),
                ("variant", Set::Option("range")),
                ("value", Set::Number(20.)),
                ("range_end", Set::Number(80.)),
                ("step", Set::Option("5")),
            ],
        },
        Scenario {
            name: "Living room climate",
            values: &[
                ("size", Set::Option("L")),
                ("orientation", Set::Option("vertical")),
                ("label", Set::Text("Temperature")),
                ("variant", Set::Option("centered")),
                ("value", Set::Number(65.)),
                ("color", Set::Option("tertiary")),
                ("step", Set::Option("5")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Segmented {
                    id: "size",
                    label: "Size",
                    options: &["XS", "S", "M", "L", "XL"],
                    default: 0,
                },
                Control::Segmented {
                    id: "orientation",
                    label: "Orientation",
                    options: &["horizontal", "vertical"],
                    default: 0,
                },
                Control::Select {
                    id: "color",
                    label: "Color",
                    options: &["primary", "secondary", "tertiary", "error"],
                    default: 0,
                },
                Control::Switch {
                    id: "ticks",
                    label: "Tick marks",
                    default: false,
                },
                Control::Switch {
                    id: "indicator",
                    label: "Value indicator",
                    default: true,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Text {
                    id: "label",
                    label: "Label",
                    default: "Volume",
                },
                Control::Slider {
                    id: "value",
                    label: "Value",
                    min: 0.,
                    max: 100.,
                    step: 1.,
                    default: 40.,
                },
                Control::Slider {
                    id: "range_end",
                    label: "Range end",
                    min: 0.,
                    max: 100.,
                    step: 1.,
                    default: 80.,
                },
                Control::Switch {
                    id: "icon",
                    label: "Inset icon",
                    default: false,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Segmented {
                    id: "variant",
                    label: "Variant",
                    options: &["standard", "centered", "range"],
                    default: 0,
                },
                Control::Segmented {
                    id: "step",
                    label: "Step",
                    options: &["1", "5", "10"],
                    default: 2,
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

/// The slider state for each step, created on first use.
const STATE_KEYS: [&str; 3] = ["slider-1", "slider-5", "slider-10"];

/// The value the configuration asks the slider for.
fn wanted(config: &Config) -> SliderValue {
    let start = config.number("value");
    if config.option("variant") == "range" {
        let end = config.number("range_end");
        SliderValue::from((start.min(end), start.max(end)))
    } else {
        SliderValue::from(start)
    }
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let step_index = config.choice("step");
    let step = STEPS[step_index];
    let key = STATE_KEYS[step_index];
    let value = wanted(config);
    let slider = preview.local(key, window, cx, |_, _| {
        SliderState::new()
            .min(0.)
            .max(100.)
            .step(step)
            .default_value(value)
    });
    if preview.changed(key, ()) {
        // Moving the thumbs writes Value and Range end back.
        cx.subscribe_in(
            &slider,
            window,
            |state: &mut PlaygroundState, _, event: &SliderEvent, window, cx| {
                let (SliderEvent::Change(value) | SliderEvent::Release(value)) = event;
                state.set_value("value", Value::Number(value.start()), window, cx);
                if value.is_range() {
                    state.set_value("range_end", Value::Number(value.end()), window, cx);
                }
            },
        )
        .detach();
    }
    if slider.read(cx).value() != value {
        slider.update(cx, |slider, cx| slider.set_value(value, window, cx));
    }
    let size = SIZES[config.choice("size")];
    let vertical = config.option("orientation") == "vertical";
    let label = config.text("label");
    let mut material = MaterialSlider::new(&slider, label.clone())
        .id("playground-slider")
        .size(size)
        .axis(if vertical {
            Axis::Vertical
        } else {
            Axis::Horizontal
        })
        .color(COLORS[config.choice("color")])
        .centered(config.option("variant") == "centered")
        .stops(config.bool("ticks"))
        .value_indicator(config.bool("indicator"))
        .disabled(config.bool("disabled"));
    if config.bool("icon") {
        material = material.icon(symbol("volume_up", 24.));
    }
    let track = div()
        .debug_selector(|| "playground-slider".into())
        .map(|track| {
            if vertical {
                track.h(rem(240.))
            } else {
                track.w_full().max_w(rem(360.))
            }
        })
        .child(material);
    div()
        .flex()
        .flex_col()
        .items_start()
        .gap(rem(8.))
        .when(!label.is_empty(), |column| {
            column.child(
                accessible_text("slider-label", label)
                    .map(|text| type_role(text, TypeRole::BodyLarge))
                    .text_color(color(if config.bool("disabled") {
                        ColorRole::ON_SURFACE_VARIANT
                    } else {
                        ColorRole::ON_SURFACE
                    })),
            )
        })
        .child(track)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::*;
    use core::prelude::v1::test;

    #[gpui::test]
    fn every_scenario_renders_and_the_thumb_writes_back(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, &PLAYGROUND);
        every_scenario_renders(&state, cx, "playground-slider");
        cx.update(|window, cx| state.update(cx, |state, cx| state.reset(window, cx)));
        settle(cx);
        // Clicking near the track's end moves the value there (step 10).
        let track = rect(cx, "playground-slider");
        cx.simulate_click(
            point(track.left() + track.size.width * 0.9, track.center().y),
            Modifiers::none(),
        );
        settle(cx);
        let value = config(&state, cx).number("value");
        assert!((value - 90.).abs() < 0.5, "moved to {value}");
        // Arrow keys step from the focused thumb.
        cx.simulate_keystrokes("left");
        settle(cx);
        let value = config(&state, cx).number("value");
        assert!((value - 80.).abs() < 0.5, "stepped to {value}");

        // A range writes both ends; the control moves its thumb.
        scenario(&state, cx, "Price budget");
        set(&state, cx, "range_end", Value::Number(60.));
        assert_eq!(config(&state, cx).number("range_end"), 60.);
        assert_eq!(config(&state, cx).number("value"), 20.);
    }
}
