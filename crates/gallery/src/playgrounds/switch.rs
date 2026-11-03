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
    code,
};

fn preview(
    config: &Config,
    preview: &mut Preview,
    _: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let supporting = config.text("supporting");
    let switch = MaterialSwitch::new("playground-switch", config.text("label"))
        .show_label(true)
        .required(config.bool("required"))
        .label_position(if config.option("position") == "end" {
            LabelPosition::End
        } else {
            LabelPosition::Start
        })
        .when(!supporting.is_empty(), |switch| {
            switch.supporting_text(supporting)
        })
        .checked(config.bool("checked"))
        .icons(ICONS[config.choice("icons")])
        .error(config.bool("error"))
        .disabled(config.bool("disabled"))
        .on_change(preview.on_bool("checked", cx));
    div()
        .debug_selector(|| "playground-switch".into())
        .w_full()
        .max_w(rem(320.))
        .child(switch)
        .into_any_element()
}

fn code(config: &Config) -> String {
    let supporting = config.text("supporting");
    chain(
        format!("MaterialSwitch::new(\"switch\", {})", lit(&config.text("label"))),
        [
            Some("show_label(true)".into()),
            (!supporting.is_empty()).then(|| format!("supporting_text({})", lit(&supporting))),
            (config.option("position") == "start")
                .then(|| "label_position(LabelPosition::Start)".into()),
            Some(format!("checked({})", config.bool("checked"))),
            match config.option("icons") {
                "selected" => Some("icons(SwitchIcons::Selected)".into()),
                "both" => Some("icons(SwitchIcons::Both)".into()),
                _ => None,
            },
            config.bool("required").then(|| "required(true)".into()),
            config.bool("error").then(|| "error(true)".into()),
            config.bool("disabled").then(|| "disabled(true)".into()),
            Some(
                "on_change(cx.listener(|this, checked: &bool, _, cx| {\n    this.checked = *checked;\n    cx.notify();\n}))"
                    .into(),
            ),
        ],
    )
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

    #[test]
    fn the_code_follows_the_configuration() {
        let mut config = Config::defaults(&PLAYGROUND);
        let code = super::code(&config);
        assert!(code.starts_with("MaterialSwitch::new(\"switch\", \"Notifications\")"));
        assert!(code.contains(".icons(SwitchIcons::Selected)"));
        assert!(!code.contains(".disabled("));
        config.set("disabled", Value::Bool(true));
        assert!(super::code(&config).contains(".disabled(true)"));
    }
}
