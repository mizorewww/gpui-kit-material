//! Button playground: md3.io's Button configuration (variant, size, shape,
//! icon, text, toggle, selection, disabled) on the real `MaterialButton`.
use super::prelude::*;

const VARIANTS: [ButtonVariant; 5] = [
    ButtonVariant::Filled,
    ButtonVariant::Tonal,
    ButtonVariant::Outlined,
    ButtonVariant::Elevated,
    ButtonVariant::Text,
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Save",
            values: &[("text", Set::Text("Save")), ("icon", Set::Option("check"))],
        },
        Scenario {
            name: "Download",
            values: &[
                ("variant", Set::Option("tonal")),
                ("text", Set::Text("Download")),
                ("icon", Set::Option("download")),
            ],
        },
        Scenario {
            name: "Secondary",
            values: &[
                ("variant", Set::Option("outlined")),
                ("text", Set::Text("Learn more")),
            ],
        },
        Scenario {
            name: "Cancel",
            values: &[
                ("variant", Set::Option("text")),
                ("text", Set::Text("Cancel")),
            ],
        },
        Scenario {
            name: "Favorite",
            values: &[
                ("variant", Set::Option("tonal")),
                ("text", Set::Text("Favorite")),
                ("icon", Set::Option("favorite")),
                ("toggle", Set::Bool(true)),
                ("selected", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Large",
            values: &[
                ("size", Set::Option("L")),
                ("text", Set::Text("Get started")),
                ("icon", Set::Option("send")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Select {
                    id: "variant",
                    label: "Variant",
                    options: &["filled", "tonal", "outlined", "elevated", "text"],
                    default: 0,
                },
                Control::Select {
                    id: "tone",
                    label: "Tone",
                    options: &["standard", "neutral", "accent"],
                    default: 0,
                },
                Control::Segmented {
                    id: "size",
                    label: "Size",
                    options: &["XS", "S", "M", "L", "XL"],
                    default: 1,
                },
                Control::Switch {
                    id: "square",
                    label: "Square shape",
                    default: false,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Icon {
                    id: "icon",
                    label: "Icon",
                    icons: &["", "add", "check", "download", "send", "favorite"],
                    default: 0,
                },
                Control::Text {
                    id: "text",
                    label: "Text",
                    default: "Button",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "toggle",
                    label: "Toggle button",
                    default: false,
                },
                Control::Switch {
                    id: "selected",
                    label: "Selected",
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
    code: Some(code),
};

fn preview(
    config: &Config,
    preview: &mut Preview,
    _: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let text = config.text("text");
    let icon = config.option("icon");
    let mut button = MaterialButton::new("playground-button")
        .debug_selector(|| "playground-button".into())
        .variant(VARIANTS[config.choice("variant")])
        .tone(ButtonTone::ALL[config.choice("tone")])
        .button_size(ButtonSize::ALL[config.choice("size")])
        .shape(if config.bool("square") {
            ButtonShape::Square
        } else {
            ButtonShape::Round
        })
        .disabled(config.bool("disabled"))
        .accessibility_label(if text.is_empty() {
            SharedString::new_static(icon)
        } else {
            text.clone()
        });
    if !icon.is_empty() {
        button = button.icon(symbol(icon, 20.));
    }
    if config.bool("toggle") {
        // Clicking the preview flips the Selected control.
        let select = preview.setter("selected", cx);
        button = button
            .toggle(true)
            .selected(config.bool("selected"))
            .on_change(move |selected, _, window, cx| select(Value::Bool(selected), window, cx));
    }
    if !text.is_empty() {
        button = button.child(text);
    }
    button.into_any_element()
}

/// The Rust names of the Variant and Size options.
const VARIANT_NAMES: [&str; 5] = ["Filled", "Tonal", "Outlined", "Elevated", "Text"];
const SIZE_NAMES: [&str; 5] = ["ExtraSmall", "Small", "Medium", "Large", "ExtraLarge"];
const TONE_NAMES: [&str; 3] = ["Standard", "Neutral", "Accent"];

fn code(config: &Config) -> String {
    let text = config.text("text");
    let icon = config.option("icon");
    let toggle = config.bool("toggle");
    let (variant, tone, size) = (
        config.choice("variant"),
        config.choice("tone"),
        config.choice("size"),
    );
    chain(
        "MaterialButton::new(\"button\")",
        [
            (variant != 0).then(|| format!("variant(ButtonVariant::{})", VARIANT_NAMES[variant])),
            (tone != 0).then(|| format!("tone(ButtonTone::{})", TONE_NAMES[tone])),
            (size != 1).then(|| format!("button_size(ButtonSize::{})", SIZE_NAMES[size])),
            config
                .bool("square")
                .then(|| "shape(ButtonShape::Square)".into()),
            config.bool("disabled").then(|| "disabled(true)".into()),
            (!icon.is_empty()).then(|| format!("icon({})", symbol_code(icon))),
            // An icon-only button still needs a name.
            text.is_empty()
                .then(|| format!("accessibility_label({})", lit(icon))),
            toggle.then(|| "toggle(true)".into()),
            toggle.then(|| format!("selected({})", config.bool("selected"))),
            Some(if toggle {
                format!(
                    "on_change({})",
                    view_update("move |selected, _, _, cx|", "this.selected = selected;")
                )
            } else {
                format!(
                    "on_click(|_, _, _| println!({}))",
                    lit(if text.is_empty() { icon } else { &text })
                )
            }),
            (!text.is_empty()).then(|| format!("child({})", lit(&text))),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_code_follows_the_configuration() {
        let mut config = Config::defaults(&PLAYGROUND);
        let code = super::code(&config);
        assert!(
            code.starts_with("MaterialButton::new(\"button\")"),
            "{code}"
        );
        assert!(code.ends_with(".child(\"Button\")"), "{code}");
        assert!(!code.contains(".variant("));
        config.set("variant", Value::Choice(2));
        config.set("toggle", Value::Bool(true));
        let code = super::code(&config);
        assert!(code.contains(".variant(ButtonVariant::Outlined)"), "{code}");
        assert!(
            code.contains(".toggle(true)\n    .selected(false)"),
            "{code}"
        );
        assert!(!code.contains(".on_click("));
    }
}
