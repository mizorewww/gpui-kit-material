//! Text field playground: md3.io's Text field configuration on the real
//! `TextField`, editing a Kit input bound two-way to the Value control.
//!
//! Input types other than password and multiline have no browser keyboard
//! to select on the desktop, so they edit plain text.
use super::prelude::*;

const INPUT_TYPES: &[&str] = &[
    "text",
    "password",
    "email",
    "number",
    "tel",
    "url",
    "search",
    "multiline",
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Amount",
            values: &[
                ("type", Set::Option("number")),
                ("label", Set::Text("Amount")),
                ("prefix", Set::Text("$")),
                ("suffix", Set::Text("USD")),
                ("placeholder", Set::Text("0.00")),
            ],
        },
        Scenario {
            name: "Password",
            values: &[
                ("type", Set::Option("password")),
                ("label", Set::Text("Password")),
                ("value", Set::Text("hunter2")),
                ("trailing", Set::Option("visibility")),
                ("trailing_label", Set::Text("Show password")),
                ("supporting", Set::Text("At least 8 characters")),
                ("required", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Email",
            values: &[
                ("type", Set::Option("email")),
                ("variant", Set::Option("outlined")),
                ("leading", Set::Option("mail")),
                ("label", Set::Text("Email")),
                ("value", Set::Text("not-an-email")),
                ("supporting", Set::Text("Enter a valid email address")),
                ("error", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Search",
            values: &[
                ("type", Set::Option("search")),
                ("leading", Set::Option("search")),
                ("trailing", Set::Option("cancel")),
                ("trailing_label", Set::Text("Clear")),
                ("label", Set::Text("Search")),
                ("value", Set::Text("Material")),
            ],
        },
        Scenario {
            name: "Message",
            values: &[
                ("type", Set::Option("multiline")),
                ("variant", Set::Option("outlined")),
                ("label", Set::Text("Message")),
                ("supporting", Set::Text("Say hello")),
                ("max_length", Set::Number(120.)),
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
                    options: &["filled", "outlined"],
                    default: 0,
                },
                Control::Segmented {
                    id: "density",
                    label: "Density",
                    options: &["default", "compact"],
                    default: 0,
                },
                Control::Icon {
                    id: "leading",
                    label: "Icon",
                    icons: &["", "search", "person", "mail", "calendar_today"],
                    default: 0,
                },
                Control::Icon {
                    id: "trailing",
                    label: "Trailing icon",
                    icons: &["", "cancel", "visibility", "info", "error"],
                    default: 0,
                },
                Control::Text {
                    id: "trailing_label",
                    label: "Trailing icon label",
                    default: "Clear",
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "type",
                    label: "Input type",
                    options: INPUT_TYPES,
                    default: 0,
                },
                Control::Text {
                    id: "label",
                    label: "Label",
                    default: "Label",
                },
                Control::Text {
                    id: "value",
                    label: "Value",
                    default: "",
                },
                Control::Text {
                    id: "placeholder",
                    label: "Placeholder",
                    default: "",
                },
                Control::Text {
                    id: "prefix",
                    label: "Prefix",
                    default: "",
                },
                Control::Text {
                    id: "suffix",
                    label: "Suffix",
                    default: "",
                },
                Control::Text {
                    id: "supporting",
                    label: "Supporting text",
                    default: "",
                },
                Control::Slider {
                    id: "max_length",
                    label: "Maximum length",
                    min: 0.,
                    max: 200.,
                    step: 1.,
                    default: 0.,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
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
                    id: "read_only",
                    label: "Read only",
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
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let kind = config.option("type");
    let mut field = if kind == "multiline" {
        TextField::multi_line(
            "playground-text-field",
            &preview.textarea("value", window, cx),
        )
    } else {
        TextField::new("playground-text-field", &preview.input("value", window, cx)).input_type(
            match kind {
                "password" => TextFieldInputType::Password,
                "email" => TextFieldInputType::Email,
                "number" => TextFieldInputType::Number,
                "tel" => TextFieldInputType::Tel,
                "url" => TextFieldInputType::Url,
                _ => TextFieldInputType::Text,
            },
        )
    };
    let label = config.text("label");
    let disabled = config.bool("disabled");
    field = field
        .variant(if config.option("variant") == "outlined" {
            TextFieldVariant::Outlined
        } else {
            TextFieldVariant::Filled
        })
        .density(if config.option("density") == "compact" {
            TextFieldDensity::Compact
        } else {
            TextFieldDensity::Default
        })
        .error(config.bool("error"))
        .disabled(disabled)
        .read_only(config.bool("read_only"))
        .required(config.bool("required"))
        .w(rem(280.));
    if !label.is_empty() {
        field = field.label(label);
    }
    for (id, apply) in [
        (
            "placeholder",
            TextField::placeholder as fn(TextField, SharedString) -> TextField,
        ),
        ("prefix", TextField::prefix),
        ("suffix", TextField::suffix),
        ("supporting", TextField::supporting_text),
    ] {
        let text = config.text(id);
        if !text.is_empty() {
            field = apply(field, text);
        }
    }
    let max_length = config.number("max_length") as usize;
    if max_length > 0 {
        field = field.max_length(max_length);
    }
    let leading = config.option("leading");
    if !leading.is_empty() {
        field = field.leading_icon(symbol(leading, 24.));
    }
    let trailing = config.option("trailing");
    if !trailing.is_empty() {
        let label = config.text("trailing_label");
        let label = if label.is_empty() {
            SharedString::new_static(trailing)
        } else {
            label
        };
        // A clear icon clears the value; the others are decorative actions.
        let clear = preview.setter("value", cx);
        field = field.trailing_icon(
            IconButton::new("playground-trailing", symbol(trailing, 24.), label)
                .disabled(disabled)
                .on_click(move |_, window, cx| {
                    if trailing == "cancel" {
                        clear(Value::Text(SharedString::default()), window, cx);
                    }
                }),
        );
    }
    div()
        .debug_selector(|| "playground-text-field".into())
        .child(field)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::*;
    use core::prelude::v1::test;

    #[gpui::test]
    fn every_scenario_renders_and_density_compacts_the_field(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, &PLAYGROUND);
        // The selector spans the row; the field is at its start.
        let field = rect(cx, "playground-text-field");
        cx.simulate_click(
            point(field.left() + px(40.), field.center().y),
            Modifiers::none(),
        );
        cx.simulate_input("Hi");
        settle(cx);
        assert_eq!(config(&state, cx).text("value"), "Hi");
        // Read only rejects typing; number filters it, on the real field.
        set(&state, cx, "read_only", Value::Bool(true));
        cx.simulate_input("x");
        settle(cx);
        assert_eq!(config(&state, cx).text("value"), "Hi", "read only");
        set(&state, cx, "read_only", Value::Bool(false));
        let number = PLAYGROUND
            .sections
            .iter()
            .flat_map(|section| section.controls)
            .find_map(|control| match control {
                Control::Select {
                    id: "type",
                    options,
                    ..
                }
                | Control::Segmented {
                    id: "type",
                    options,
                    ..
                } => options.iter().position(|option| *option == "number"),
                _ => None,
            })
            .expect("the type control offers number");
        set(&state, cx, "value", Value::Text(SharedString::default()));
        set(&state, cx, "type", Value::Choice(number));
        cx.simulate_input("1a2");
        settle(cx);
        assert_eq!(config(&state, cx).text("value"), "12", "number filters");
        cx.update(|window, cx| state.update(cx, |state, cx| state.reset(window, cx)));
        settle(cx);
        every_scenario_renders(&state, cx, "playground-text-field");
        cx.update(|window, cx| state.update(cx, |state, cx| state.reset(window, cx)));
        settle(cx);
        let default = rect(cx, "playground-text-field").size.height;
        set(&state, cx, "density", Value::Choice(1));
        let compact = rect(cx, "playground-text-field").size.height;
        // 8dp shorter (the test window's rem is 16px).
        let lost = f32::from(default - compact);
        assert!(
            (lost - 8. * 16. / 14.).abs() < 1.,
            "8dp shorter, not {lost}"
        );
    }
}
