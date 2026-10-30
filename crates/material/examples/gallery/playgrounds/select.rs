//! Select playground: md3.io's Select configuration (variant, density,
//! option set, label, selection, supporting text, a disabled option, error,
//! required, disabled) on the real `ExposedDropdown`, whose menu opens and
//! really selects.
//!
//! md3.io's Selected options are its default set's values; here they are
//! positions in the chosen option set, so they work for every set.
use super::prelude::*;

const SETS: [&[&str]; 3] = [
    &["Apple", "Banana", "Cherry"],
    &["Alaska", "California", "Colorado", "New York", "Texas"],
    &[
        "No restrictions",
        "Vegetarian",
        "Vegan",
        "Gluten-free",
        "Halal",
    ],
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "State",
            values: &[
                ("set", Set::Option("State")),
                ("label", Set::Text("State")),
                ("selected", Set::Option("None")),
                ("supporting", Set::Text("Where you live")),
                ("required", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Dietary options",
            values: &[
                ("set", Set::Option("Diet")),
                ("variant", Set::Option("filled")),
                ("label", Set::Text("Diet")),
                ("selected", Set::Option("2")),
                ("supporting", Set::Text("We'll plan the menu around it")),
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
                    default: 1,
                },
                Control::Segmented {
                    id: "density",
                    label: "Density",
                    options: &["default", "compact"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "set",
                    label: "Select",
                    options: &["Default", "State", "Diet"],
                    default: 0,
                },
                Control::Text {
                    id: "label",
                    label: "Label",
                    default: "Fruit",
                },
                Control::Select {
                    id: "selected",
                    label: "Selected",
                    options: &["None", "1", "2", "3", "4", "5"],
                    default: 1,
                },
                Control::Text {
                    id: "supporting",
                    label: "Supporting text",
                    default: "Choose a favorite",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "disable_second",
                    label: "Disable banana",
                    default: false,
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
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let options = SETS[config.choice("set")];
    // "1".. are positions; a position past the set's end selects nothing.
    let selected = config
        .choice("selected")
        .checked_sub(1)
        .filter(|&index| index < options.len());
    let open = preview.local("open", window, cx, |_, _| false);
    let is_open = *open.read(cx);
    let label = config.text("label");
    let label: SharedString = if config.bool("required") && !label.is_empty() {
        format!("{label}*").into()
    } else {
        label
    };
    let select = preview.setter("selected", cx);
    let supporting = config.text("supporting");
    let mut dropdown = ExposedDropdown::new("playground-select", label)
        .variant(if config.option("variant") == "filled" {
            TextFieldVariant::Filled
        } else {
            TextFieldVariant::Outlined
        })
        .density(if config.option("density") == "compact" {
            TextFieldDensity::Compact
        } else {
            TextFieldDensity::Default
        })
        .options(options.iter().copied())
        .disabled_options(config.bool("disable_second").then_some(1))
        .selected(selected)
        .open(is_open)
        .error(config.bool("error"))
        .disabled(config.bool("disabled"))
        .width(280.)
        .on_open_change({
            let open = open.clone();
            move |next: &bool, _, cx| {
                open.update(cx, |open, cx| {
                    *open = *next;
                    cx.notify();
                })
            }
        })
        .on_select(move |index: &usize, window, cx| {
            open.update(cx, |open, cx| {
                *open = false;
                cx.notify();
            });
            select(Value::Choice(index + 1), window, cx);
        });
    if !supporting.is_empty() {
        dropdown = dropdown.supporting_text(supporting);
    }
    div()
        .debug_selector(|| "playground-select".into())
        .child(dropdown)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::*;
    use core::prelude::v1::test;

    /// Click the preview's field (the configuration card has dropdowns too).
    fn open_menu(cx: &mut VisualTestContext) {
        let field = rect(cx, "playground-select");
        cx.simulate_click(
            point(field.center().x, field.top() + px(24.)),
            Modifiers::none(),
        );
        settle(cx);
    }

    #[gpui::test]
    fn every_scenario_renders_and_the_menu_selects(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, &PLAYGROUND);
        every_scenario_renders(&state, cx, "playground-select");
        cx.update(|window, cx| state.update(cx, |state, cx| state.reset(window, cx)));
        settle(cx);
        assert!(
            rect(cx, "playground-select").size.height > px(56.),
            "with supporting text"
        );
        open_menu(cx);
        let surface = rect(cx, "menu-popup-surface");
        // The third of three rows.
        cx.simulate_click(
            point(
                surface.center().x,
                surface.top() + surface.size.height * (2.5 / 3.),
            ),
            Modifiers::none(),
        );
        settle(cx);
        assert_eq!(config(&state, cx).option("selected"), "3");
        assert!(cx.debug_bounds("menu-popup-surface").is_none(), "closed");

        // A disabled banana cannot be chosen.
        set(&state, cx, "disable_second", Value::Bool(true));
        open_menu(cx);
        let surface = rect(cx, "menu-popup-surface");
        cx.simulate_click(
            point(
                surface.center().x,
                surface.top() + surface.size.height * (1.5 / 3.),
            ),
            Modifiers::none(),
        );
        settle(cx);
        assert_eq!(config(&state, cx).option("selected"), "3");
    }
}
