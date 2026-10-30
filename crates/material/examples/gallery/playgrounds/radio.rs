//! Radio buttons playground: md3.io's Radio buttons configuration
//! (direction, labels before, option sets, name, selection, a disabled
//! option, disabled) on the real `MaterialRadioGroup` with its roving
//! keyboard focus.
//!
//! md3.io's Name is the group's form name; here it names the group for
//! accessibility. Its Selected options are positions in the chosen set.
use super::prelude::*;

const SETS: [&[&str]; 6] = [
    &["Light", "Dark", "System default"],
    &["Standard", "Express", "Pick up"],
    &[
        "Relevance",
        "Lowest price",
        "Highest rating",
        "Most reviewed",
    ],
    &["None", "Callisto", "Ganymede", "Luna"],
    &["English", "Chinese (Mandarin)", "Spanish"],
    &["Allow notifications", "Turn off notifications"],
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Express delivery",
            values: &[
                ("options", Set::Option("Express delivery")),
                ("name", Set::Text("Delivery")),
                ("selected", Set::Option("2")),
                ("disable_option", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Catalog sort",
            values: &[
                ("options", Set::Option("Catalog sort")),
                ("name", Set::Text("Sort by")),
                ("selected", Set::Option("1")),
            ],
        },
        Scenario {
            name: "Phone ringtone",
            values: &[
                ("options", Set::Option("Phone ringtone")),
                ("name", Set::Text("Ringtone")),
                ("selected", Set::Option("3")),
                ("labels_before", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "App language",
            values: &[
                ("options", Set::Option("App language")),
                ("name", Set::Text("Language")),
                ("selected", Set::Option("1")),
            ],
        },
        Scenario {
            name: "Notifications",
            values: &[
                ("options", Set::Option("Notifications")),
                ("name", Set::Text("Notifications")),
                ("direction", Set::Option("horizontal")),
                ("selected", Set::Option("1")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Layout",
            controls: &[
                Control::Segmented {
                    id: "direction",
                    label: "Direction",
                    options: &["vertical", "horizontal"],
                    default: 0,
                },
                Control::Switch {
                    id: "labels_before",
                    label: "Labels before",
                    default: false,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "options",
                    label: "Options",
                    options: &[
                        "Default",
                        "Express delivery",
                        "Catalog sort",
                        "Phone ringtone",
                        "App language",
                        "Notifications",
                    ],
                    default: 0,
                },
                Control::Text {
                    id: "name",
                    label: "Name",
                    default: "Theme",
                },
                Control::Select {
                    id: "selected",
                    label: "Selected",
                    options: &["None", "1", "2", "3", "4"],
                    default: 1,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "disable_option",
                    label: "Disable an option",
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
    let options = SETS[config.choice("options")];
    let selected = config
        .choice("selected")
        .checked_sub(1)
        .filter(|&index| index < options.len());
    // The option md3.io disables: the second, or the last of two.
    let disable = config.bool("disable_option").then_some(1);
    let select = preview.setter("selected", cx);
    let mut group = MaterialRadioGroup::new("playground-radios", config.text("name"))
        .axis(if config.option("direction") == "horizontal" {
            Axis::Horizontal
        } else {
            Axis::Vertical
        })
        .labels_before(config.bool("labels_before"))
        .disabled(config.bool("disabled"))
        .selected(selected)
        .on_change(move |index: &usize, window, cx| select(Value::Choice(index + 1), window, cx));
    for (index, option) in options.iter().enumerate() {
        group = group.option(*option, disable != Some(index));
    }
    div()
        .debug_selector(|| "playground-radios".into())
        .min_w(rem(240.))
        .child(group)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::*;
    use core::prelude::v1::test;

    #[gpui::test]
    fn every_scenario_renders_and_choosing_writes_back(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, &PLAYGROUND);
        every_scenario_renders(&state, cx, "playground-radios");
        scenario(&state, cx, "Catalog sort");
        let group = rect(cx, "playground-radios");
        // The third 48dp row.
        cx.simulate_click(
            point(group.left() + px(24.), group.top() + px(2. * 48. + 24.)),
            Modifiers::none(),
        );
        settle(cx);
        assert_eq!(config(&state, cx).option("selected"), "3");
        // Arrow keys rove and select from the focused option.
        cx.simulate_keystrokes("down");
        settle(cx);
        assert_eq!(config(&state, cx).option("selected"), "4");
        // A disabled option is skipped and cannot be clicked.
        set(&state, cx, "disable_option", Value::Bool(true));
        cx.simulate_click(
            point(group.left() + px(24.), group.top() + px(48. + 24.)),
            Modifiers::none(),
        );
        settle(cx);
        assert_eq!(config(&state, cx).option("selected"), "4");
    }
}
