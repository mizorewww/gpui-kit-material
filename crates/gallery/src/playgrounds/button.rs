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
