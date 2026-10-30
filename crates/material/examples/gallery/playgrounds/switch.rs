//! Switch playground: md3.io's Switch configuration (icons, label position,
//! label, supporting text, checked, error, required, disabled) on the real
//! `MaterialSwitch` in a labelled settings row.
use super::prelude::*;

const ICONS: [SwitchIcons; 3] = [SwitchIcons::None, SwitchIcons::Selected, SwitchIcons::Both];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Play over notifications",
            values: &[
                ("label", Set::Text("Play over notifications")),
                ("supporting", Set::Text("")),
                ("checked", Set::Bool(false)),
                ("icons", Set::Option("none")),
            ],
        },
        Scenario {
            name: "Camera access",
            values: &[
                ("label", Set::Text("Camera access")),
                ("supporting", Set::Text("Let apps use the camera")),
                ("icons", Set::Option("both")),
            ],
        },
        Scenario {
            name: "Show password",
            values: &[
                ("label", Set::Text("Show password")),
                ("supporting", Set::Text("")),
                ("checked", Set::Bool(false)),
                ("position", Set::Option("end")),
            ],
        },
        Scenario {
            name: "Airplane mode",
            values: &[
                ("label", Set::Text("Airplane mode")),
                ("supporting", Set::Text("Turns off wireless connections")),
                ("checked", Set::Bool(false)),
                ("icons", Set::Option("none")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Segmented {
                    id: "icons",
                    label: "Icons",
                    options: &["none", "selected", "both"],
                    default: 1,
                },
                Control::Segmented {
                    id: "position",
                    label: "Label position",
                    options: &["start", "end"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Text {
                    id: "label",
                    label: "Label",
                    default: "Notifications",
                },
                Control::Text {
                    id: "supporting",
                    label: "Supporting text",
                    default: "Stay up to date",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "checked",
                    label: "Checked",
                    default: true,
                },
                Control::Switch {
                    id: "error",
                    label: "Error",
                    default: false,
                },
                Control::Switch {
                    id: "required",
                    label: "Required",
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
    preview: &mut Preview,
    _: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let label = config.text("label");
    let supporting = config.text("supporting");
    let (error, disabled, checked) = (
        config.bool("error"),
        config.bool("disabled"),
        config.bool("checked"),
    );
    let accessible: SharedString = if config.bool("required") {
        format!("{label} *").into()
    } else {
        label.clone()
    };
    let ink = |role: ColorRole| {
        let ink = color(role);
        if disabled { ink.opacity(0.38) } else { ink }
    };
    let text_role = if error {
        ColorRole::ERROR
    } else {
        ColorRole::ON_SURFACE
    };
    let supporting_role = if error {
        ColorRole::ERROR
    } else {
        ColorRole::ON_SURFACE_VARIANT
    };
    let toggle = preview.on_bool("checked", cx);
    let text = div()
        .id("switch-text")
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        // The label toggles the switch, as an HTML label would.
        .when(!disabled, |text| text.cursor_pointer())
        .on_click(move |_, window, cx| {
            if !disabled {
                toggle(&!checked, window, cx)
            }
        })
        .child(
            accessible_text("switch-label", accessible.clone())
                .map(|text| type_role(text, TypeRole::BodyLarge))
                .text_color(ink(text_role)),
        )
        .when(!supporting.is_empty(), |text| {
            text.child(
                accessible_text("switch-supporting", supporting)
                    .map(|text| type_role(text, TypeRole::BodyMedium))
                    .text_color(ink(supporting_role)),
            )
        });
    let switch = MaterialSwitch::new("playground-switch", accessible)
        .checked(checked)
        .icons(ICONS[config.choice("icons")])
        .error(error)
        .disabled(disabled)
        .on_change(preview.on_bool("checked", cx));
    let switch = div()
        .debug_selector(|| "playground-switch".into())
        .child(switch);
    let end = config.option("position") == "end";
    div()
        .w_full()
        .max_w(rem(320.))
        .flex()
        .items_center()
        .gap(rem(16.))
        .flex_row()
        .when(end, |row| row.flex_row_reverse())
        .child(text)
        .child(switch)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::*;
    use core::prelude::v1::test;

    #[gpui::test]
    fn every_scenario_renders_and_the_switch_writes_back(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, &PLAYGROUND);
        every_scenario_renders(&state, cx, "playground-switch");
        scenario(&state, cx, "Camera access");
        assert!(config(&state, cx).bool("checked"));
        click(cx, "playground-switch");
        assert!(!config(&state, cx).bool("checked"));
        // The error state renders on the real switch.
        set(&state, cx, "error", Value::Bool(true));
        click(cx, "playground-switch");
        assert!(config(&state, cx).bool("checked"));
    }
}
