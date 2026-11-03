//! Checkbox playground: md3.io's Checkbox configuration (label position,
//! a parent with children, option sets, label, state, error, required,
//! disabled) on the real tri-state `MaterialCheckbox`.
//!
//! With "Parent and children" on, the parent's state follows its children
//! (indeterminate while they differ) and activating it checks or clears
//! them all, as on md3.io; the State control mirrors the parent both ways.
//! md3.io's Name and Value are HTML form fields with nothing to show and no
//! form to submit here, so they are not offered.
use super::prelude::*;

/// One option set: the parent's label and the children with their initial
/// selection.
struct OptionSet {
    label: &'static str,
    children: &'static [(&'static str, bool)],
}

const SETS: [OptionSet; 3] = [
    OptionSet {
        label: "Additions",
        children: &[
            ("Pickles", false),
            ("Tomato", true),
            ("Lettuce", false),
            ("Cheese", false),
        ],
    },
    OptionSet {
        label: "Burger additions",
        children: &[
            ("Ketchup", true),
            ("Mustard", true),
            ("Onions", false),
            ("Jalapeños", false),
        ],
    },
    OptionSet {
        label: "Email notifications",
        children: &[
            ("Comments", true),
            ("Mentions", true),
            ("Product updates", false),
            ("Newsletter", false),
        ],
    },
];

const STATES: [CheckboxState; 3] = [
    CheckboxState::Unchecked,
    CheckboxState::Checked,
    CheckboxState::Indeterminate,
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Burger additions",
            values: &[
                ("options", Set::Option("Burger additions")),
                ("label", Set::Text("Burger additions")),
            ],
        },
        Scenario {
            name: "Email notifications",
            values: &[
                ("options", Set::Option("Email notifications")),
                ("parent", Set::Bool(false)),
                ("label", Set::Text("Email me about replies")),
                ("state", Set::Option("checked")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[Control::Segmented {
                id: "position",
                label: "Label position",
                options: &["start", "end"],
                default: 1,
            }],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Switch {
                    id: "parent",
                    label: "Parent and children",
                    default: true,
                },
                Control::Select {
                    id: "options",
                    label: "Options",
                    options: &["Default", "Burger additions", "Email notifications"],
                    default: 0,
                },
                Control::Text {
                    id: "label",
                    label: "Label",
                    default: "Additions",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Select {
                    id: "state",
                    label: "State",
                    options: &["unchecked", "checked", "indeterminate"],
                    default: 2,
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

/// The parent's state for its children's selection.
fn parent_state(children: &[bool]) -> CheckboxState {
    match (children.iter().all(|c| *c), children.iter().any(|c| *c)) {
        (true, _) => CheckboxState::Checked,
        (false, true) => CheckboxState::Indeterminate,
        _ => CheckboxState::Unchecked,
    }
}

/// The children's selection for a parent state set from the State control.
fn children_for(state: CheckboxState, set: &OptionSet) -> Vec<bool> {
    match state {
        CheckboxState::Checked => vec![true; set.children.len()],
        CheckboxState::Unchecked => vec![false; set.children.len()],
        CheckboxState::Indeterminate => {
            let initial: Vec<bool> = set.children.iter().map(|(_, on)| *on).collect();
            if parent_state(&initial) == CheckboxState::Indeterminate {
                initial
            } else {
                (0..set.children.len()).map(|i| i == 0).collect()
            }
        }
    }
}

fn state_value(state: CheckboxState) -> Value {
    Value::Choice(STATES.iter().position(|s| *s == state).unwrap_or(0))
}

/// Shared row options.
#[derive(Clone, Copy)]
struct RowStyle {
    position: LabelPosition,
    required: bool,
    error: bool,
    disabled: bool,
}

/// A labelled checkbox row.
fn row(
    id: &'static str,
    label: SharedString,
    state: CheckboxState,
    style: RowStyle,
    on_change: impl Fn(&CheckboxState, &mut Window, &mut App) + 'static,
) -> Div {
    // Hugging the row, so the selector's center is on the control or its label.
    div()
        .flex()
        .self_start()
        .debug_selector(move || id.into())
        .child(
            MaterialCheckbox::new(id, label)
                .show_label(true)
                .label_position(style.position)
                .required(style.required)
                .state(state)
                .error(style.error)
                .disabled(style.disabled)
                .on_change(on_change),
        )
}

const CHILD_IDS: [&str; 4] = [
    "checkbox-child-0",
    "checkbox-child-1",
    "checkbox-child-2",
    "checkbox-child-3",
];

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let set = &SETS[config.choice("options")];
    let wanted = STATES[config.choice("state")];
    let style = RowStyle {
        position: if config.option("position") == "start" {
            LabelPosition::Start
        } else {
            LabelPosition::End
        },
        required: config.bool("required"),
        error: config.bool("error"),
        disabled: config.bool("disabled"),
    };
    let label = config.text("label");
    let set_state = std::rc::Rc::new(preview.setter("state", cx));
    if !config.bool("parent") {
        let set_state = set_state.clone();
        return div()
            .flex()
            .debug_selector(|| "playground-checkbox".into())
            .child(row(
                "checkbox-parent",
                label,
                wanted,
                style,
                move |next, window, cx| set_state(state_value(*next), window, cx),
            ))
            .into_any_element();
    }
    let children = preview.local("children", window, cx, |_, _| children_for(wanted, set));
    // A new option set starts from its own selection.
    if preview.changed("options", config.choice("options")) {
        let initial = set.children.iter().map(|(_, on)| *on).collect();
        children.update(cx, |children, _| *children = initial);
    }
    let derived = parent_state(children.read(cx));
    // Setting the State control checks or clears the children; otherwise
    // the control follows the children.
    if preview.changed("state", config.choice("state")) && wanted != derived {
        let next = children_for(wanted, set);
        children.update(cx, |children, _| *children = next);
    } else if wanted != derived {
        cx.defer_in(window, move |state, window, cx| {
            state.set_value("state", state_value(derived), window, cx)
        });
    }
    let selection = children.read(cx).clone();
    let parent_state_now = parent_state(&selection);
    let parent = {
        let children = children.clone();
        let set_state = set_state.clone();
        row(
            "checkbox-parent",
            label,
            parent_state_now,
            style,
            move |next, window, cx| {
                let on = *next == CheckboxState::Checked;
                children.update(cx, |children, cx| {
                    children.iter_mut().for_each(|c| *c = on);
                    cx.notify();
                });
                set_state(state_value(*next), window, cx);
            },
        )
    };
    let child_rows = set.children.iter().enumerate().map(|(index, (name, _))| {
        let children = children.clone();
        let set_state = set_state.clone();
        let state = if selection.get(index).copied().unwrap_or(false) {
            CheckboxState::Checked
        } else {
            CheckboxState::Unchecked
        };
        row(
            CHILD_IDS[index],
            SharedString::new_static(name),
            state,
            RowStyle {
                required: false,
                ..style
            },
            move |next, window, cx| {
                let on = *next == CheckboxState::Checked;
                let derived = children.update(cx, |children, cx| {
                    if let Some(child) = children.get_mut(index) {
                        *child = on;
                    }
                    cx.notify();
                    parent_state(children)
                });
                set_state(state_value(derived), window, cx);
            },
        )
        .pl(rem(24.))
    });
    div()
        .debug_selector(|| "playground-checkbox".into())
        .id("checkbox-group")
        .role(Role::Group)
        .aria_label(SETS[config.choice("options")].label)
        .flex()
        .flex_col()
        .child(parent)
        .children(child_rows)
        .into_any_element()
}

fn code(config: &Config) -> String {
    const STATE_NAMES: [&str; 3] = ["Unchecked", "Checked", "Indeterminate"];
    let style = |required: bool| {
        [
            Some("show_label(true)".to_string()),
            (config.option("position") == "start")
                .then(|| "label_position(LabelPosition::Start)".into()),
            (required && config.bool("required")).then(|| "required(true)".into()),
            config.bool("error").then(|| "error(true)".into()),
            config.bool("disabled").then(|| "disabled(true)".into()),
        ]
    };
    let state = config.choice("state");
    let label = lit(&config.text("label"));
    if !config.bool("parent") {
        return chain(
            format!("MaterialCheckbox::new(\"checkbox\", {label})"),
            style(true).into_iter().chain([
                (state != 0).then(|| format!("state(CheckboxState::{})", STATE_NAMES[state])),
                Some(format!(
                    "on_change({})",
                    listener(
                        "|this, state: &CheckboxState, _, cx|",
                        "this.state = *state;"
                    )
                )),
            ]),
        );
    }
    let set = &SETS[config.choice("options")];
    let children: Vec<String> = set.children.iter().map(|(name, _)| lit(name)).collect();
    // The parent is checked when every child is, indeterminate when some are.
    let parent = chain(
        format!("MaterialCheckbox::new(\"all\", {label})"),
        style(true).into_iter().chain([
            Some(
                "state(match self.checked.iter().filter(|checked| **checked).count() {\n    0 => CheckboxState::Unchecked,\n    n if n == self.checked.len() => CheckboxState::Checked,\n    _ => CheckboxState::Indeterminate,\n})"
                    .into(),
            ),
            Some(format!(
                "on_change({})",
                listener(
                    "|this, state: &CheckboxState, _, cx|",
                    "this.checked.fill(*state == CheckboxState::Checked);"
                )
            )),
        ]),
    );
    let child = chain(
        "MaterialCheckbox::new((\"option\", index), name)",
        style(false).into_iter().chain([
            Some("checked(self.checked[index])".into()),
            Some(format!(
                "on_change({})",
                listener(
                    "move |this, state: &CheckboxState, _, cx|",
                    "this.checked[index] = *state == CheckboxState::Checked;"
                )
            )),
        ]),
    );
    chain(
        "div()",
        [
            Some("flex()".into()),
            Some("flex_col()".into()),
            Some(call("child", &parent)),
            Some(format!(
                "children([{}].into_iter().enumerate().map(|(index, name)| {{\n    {}\n}}))",
                children.join(", "),
                indent(&child)
            )),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::*;
    use core::prelude::v1::test;

    #[gpui::test]
    fn every_scenario_renders_and_the_parent_follows_its_children(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, &PLAYGROUND);
        every_scenario_renders(&state, cx, "playground-checkbox");
        cx.update(|window, cx| state.update(cx, |state, cx| state.reset(window, cx)));
        settle(cx);
        assert_eq!(config(&state, cx).option("state"), "indeterminate");
        // Activating the mixed parent checks every child.
        click(cx, "checkbox-parent");
        assert_eq!(config(&state, cx).option("state"), "checked");
        // Clearing one child makes the parent mixed again.
        click(cx, "checkbox-child-0");
        assert_eq!(config(&state, cx).option("state"), "indeterminate");
        // The State control clears them all.
        set(&state, cx, "state", Value::Choice(0));
        click(cx, "checkbox-child-1");
        assert_eq!(config(&state, cx).option("state"), "indeterminate");

        // A lone checkbox writes its own state back.
        scenario(&state, cx, "Email notifications");
        click(cx, "checkbox-parent");
        assert_eq!(config(&state, cx).option("state"), "unchecked");
    }

    #[test]
    fn the_parent_state_summarizes_its_children() {
        assert_eq!(parent_state(&[true, true]), CheckboxState::Checked);
        assert_eq!(parent_state(&[true, false]), CheckboxState::Indeterminate);
        assert_eq!(parent_state(&[false, false]), CheckboxState::Unchecked);
        for set in &SETS {
            for state in STATES {
                assert_eq!(parent_state(&children_for(state, set)), state);
            }
        }
    }

    #[test]
    fn the_code_follows_the_configuration() {
        let mut config = Config::defaults(&PLAYGROUND);
        let code = super::code(&config);
        assert!(code.starts_with("div()"), "{code}");
        assert!(
            code.contains("MaterialCheckbox::new(\"all\", \"Additions\")"),
            "{code}"
        );
        assert!(code.contains("\"Pickles\", \"Tomato\""), "{code}");
        config.set("parent", Value::Bool(false));
        let code = super::code(&config);
        assert!(
            code.starts_with("MaterialCheckbox::new(\"checkbox\", \"Additions\")"),
            "{code}"
        );
        assert!(code.contains(".state(CheckboxState::Indeterminate)"));
        assert!(!code.contains(".error("));
        config.set("error", Value::Bool(true));
        assert!(super::code(&config).contains(".error(true)"));
    }
}
