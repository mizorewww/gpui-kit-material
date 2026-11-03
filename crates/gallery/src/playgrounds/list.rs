//! List playground: md3.io's List configuration (standard or segmented,
//! one to three text lines, leading and trailing elements, dividers, a
//! subheader, item content presets, selection and per-item state) on the
//! real `ListItem`s. Clicking a row selects it (single), toggles it
//! (multi) or toggles its trailing control, writing back to the
//! "… selected" switches. The pill shape makes the rows a sidebar's
//! destinations, as the Sidebar destinations scenario shows.
use super::prelude::*;

const SWITCH_ICONS: [SwitchIcons; 3] =
    [SwitchIcons::None, SwitchIcons::Selected, SwitchIcons::Both];
const ITEM_SETS: &[&str] = &[
    "Default",
    "Inbox threads",
    "Color picker",
    "Plant cart",
    "Address book",
    "Folders",
];
const SELECTED: [&str; 5] = [
    "selected_0",
    "selected_1",
    "selected_2",
    "selected_3",
    "selected_4",
];

/// One preset's content: subheader, overline, then (headline, icon,
/// trailing text) per item.
struct ItemSet {
    subheader: &'static str,
    overline: &'static str,
    items: [(&'static str, &'static str, &'static str); 5],
}

const SETS: [ItemSet; 6] = [
    ItemSet {
        subheader: "Ideas for today",
        overline: "Routine",
        items: [
            ("Morning walk", "favorite", "5 min"),
            ("Read a chapter", "bookmark", "10 min"),
            ("Try a new recipe", "download", "15 min"),
            ("Call a friend", "call", "20 min"),
            ("Stretch", "self_improvement", "25 min"),
        ],
    },
    ItemSet {
        subheader: "Inbox",
        overline: "Unread",
        items: [
            ("Ali Connors", "mail", "10:42"),
            ("Jennifer Lee", "mail", "9:15"),
            ("Sandra Adams", "mail", "Yesterday"),
            ("Trevor Hansen", "mail", "Mon"),
            ("Britta Holt", "mail", "Sun"),
        ],
    },
    ItemSet {
        subheader: "Accent colors",
        overline: "Palette",
        items: [
            ("Crimson", "palette", "#B3261E"),
            ("Ocean", "palette", "#0061A4"),
            ("Forest", "palette", "#386A20"),
            ("Sunflower", "palette", "#7C5800"),
            ("Lavender", "palette", "#6750A4"),
        ],
    },
    ItemSet {
        subheader: "Your cart",
        overline: "Houseplant",
        items: [
            ("Monstera", "potted_plant", "$32"),
            ("Fiddle leaf fig", "potted_plant", "$48"),
            ("Snake plant", "potted_plant", "$24"),
            ("Pothos", "potted_plant", "$18"),
            ("Calathea", "potted_plant", "$29"),
        ],
    },
    ItemSet {
        subheader: "Contacts",
        overline: "Favorite",
        items: [
            ("Abbey Christensen", "call", "Mobile"),
            ("Alex Nelson", "call", "Work"),
            ("Anna Lindström", "call", "Home"),
            ("Ben Okafor", "call", "Mobile"),
            ("Carla Ruiz", "call", "Work"),
        ],
    },
    ItemSet {
        subheader: "Folders",
        overline: "Folder",
        items: [
            ("All items", "inbox", "128"),
            ("Projects", "folder", "64"),
            ("Reading", "folder", "31"),
            ("Shared", "folder_shared", "18"),
            ("Archive", "inventory_2", "15"),
        ],
    },
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Inbox threads",
            values: &[
                ("items", Set::Option("Inbox threads")),
                ("lines", Set::Option("3")),
                ("leading", Set::Option("avatar")),
                ("trailing", Set::Option("text")),
                ("dividers", Set::Option("inset")),
                ("count", Set::Option("5")),
                (
                    "supporting",
                    Set::Text(
                        "I'll be in your neighborhood doing errands this weekend. Do you want to grab brunch?",
                    ),
                ),
                ("label", Set::Text("Inbox")),
                ("selection", Set::Option("none")),
            ],
        },
        Scenario {
            name: "Color picker",
            values: &[
                ("items", Set::Option("Color picker")),
                ("lines", Set::Option("1")),
                ("leading", Set::Option("icon")),
                ("trailing", Set::Option("control")),
                ("count", Set::Option("5")),
                ("label", Set::Text("Accent color")),
                ("selection", Set::Option("single")),
                ("selected_0", Set::Bool(false)),
                ("selected_1", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Plant cart",
            values: &[
                ("items", Set::Option("Plant cart")),
                ("variant", Set::Option("segmented")),
                ("lines", Set::Option("2")),
                ("leading", Set::Option("image")),
                ("trailing", Set::Option("control")),
                ("supporting", Set::Text("In stock · Ships tomorrow")),
                ("label", Set::Text("Cart")),
                ("selection", Set::Option("multi")),
                ("selected_2", Set::Bool(true)),
                ("disable_last", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Address book",
            values: &[
                ("items", Set::Option("Address book")),
                ("lines", Set::Option("2")),
                ("leading", Set::Option("avatar")),
                ("trailing", Set::Option("icon")),
                ("dividers", Set::Option("full width")),
                ("subheader", Set::Bool(true)),
                ("count", Set::Option("5")),
                ("supporting", Set::Text("+1 555 0100")),
                ("label", Set::Text("Contacts")),
                ("selection", Set::Option("none")),
                ("selected_0", Set::Bool(false)),
            ],
        },
        Scenario {
            name: "Sidebar destinations",
            values: &[
                ("items", Set::Option("Folders")),
                ("density", Set::Option("−3")),
                ("shape", Set::Option("pill")),
                ("lines", Set::Option("1")),
                ("leading", Set::Option("icon")),
                ("trailing", Set::Option("text")),
                ("count", Set::Option("5")),
                ("label", Set::Text("Folders")),
                ("selection", Set::Option("single")),
                ("selected_0", Set::Bool(false)),
                ("selected_1", Set::Bool(true)),
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
                    options: &["standard", "segmented"],
                    default: 0,
                },
                Control::Segmented {
                    id: "density",
                    label: "Density",
                    options: &["standard", "compact", "−3"],
                    default: 0,
                },
                Control::Segmented {
                    id: "shape",
                    label: "Shape",
                    options: &["rectangle", "pill"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Layout",
            controls: &[
                Control::Segmented {
                    id: "lines",
                    label: "Text lines",
                    options: &["1", "2", "3"],
                    default: 1,
                },
                Control::Select {
                    id: "leading",
                    label: "Leading",
                    options: &["none", "icon", "tinted", "avatar", "image", "video"],
                    default: 1,
                },
                Control::Select {
                    id: "trailing",
                    label: "Trailing",
                    options: &["none", "text", "icon", "control"],
                    default: 1,
                },
                Control::Select {
                    id: "dividers",
                    label: "Dividers",
                    options: &["none", "full width", "inset"],
                    default: 0,
                },
                Control::Switch {
                    id: "subheader",
                    label: "Subheader",
                    default: false,
                },
                Control::Switch {
                    id: "expressive",
                    label: "Expressive",
                    default: false,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "switch_icons",
                    label: "Switch icons",
                    options: &["none", "selected", "both"],
                    default: 0,
                },
                Control::Select {
                    id: "items",
                    label: "Items",
                    options: ITEM_SETS,
                    default: 0,
                },
                Control::Segmented {
                    id: "count",
                    label: "Item count",
                    options: &["3", "5"],
                    default: 0,
                },
                Control::Text {
                    id: "supporting",
                    label: "Supporting text",
                    default: "Make a little time for yourself",
                },
                Control::Switch {
                    id: "overline",
                    label: "Overline",
                    default: false,
                },
                Control::Text {
                    id: "label",
                    label: "Accessible label",
                    default: "Ideas for today",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Segmented {
                    id: "selection",
                    label: "Selection",
                    options: &["none", "single", "multi"],
                    default: 1,
                },
                Control::Switch {
                    id: "disable_last",
                    label: "Disable last item",
                    default: false,
                },
                Control::Switch {
                    id: "selected_0",
                    label: "First selected",
                    default: true,
                },
                Control::Switch {
                    id: "selected_1",
                    label: "Second selected",
                    default: false,
                },
                Control::Switch {
                    id: "selected_2",
                    label: "Third selected",
                    default: false,
                },
                Control::Switch {
                    id: "selected_3",
                    label: "Fourth selected",
                    default: false,
                },
                Control::Switch {
                    id: "selected_4",
                    label: "Fifth selected",
                    default: false,
                },
            ],
        },
    ],
    preview,
    code: None,
};

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let set = &SETS[config.choice("items")];
    let count: usize = if config.option("count") == "5" { 5 } else { 3 };
    let lines = config.option("lines");
    let supporting = config.text("supporting");
    let overline = lines == "3" && config.bool("overline");
    let selection = config.option("selection");
    let trailing = config.option("trailing");
    let segmented = config.option("variant") == "segmented";
    let density = match config.option("density") {
        "compact" => ListDensity::Compact,
        "−3" => ListDensity::Minus3,
        _ => ListDensity::Standard,
    };
    let pill = config.option("shape") == "pill";
    let mut selected: Vec<bool> = SELECTED.iter().map(|id| config.bool(id)).collect();
    // The selection the last frame showed, to tell which row was just turned on.
    let shown = preview.local("shown-selection", window, cx, |_, _| {
        vec![false; SELECTED.len()]
    });
    // One choice: the row just turned on stays, or with none (a multiple selection
    // left by the selection mode) the first. The others go from the controls too,
    // after this frame, which shows it already.
    let keep = (selection == "single")
        .then(|| {
            let previous = shown.read(cx);
            selected
                .iter()
                .zip(previous)
                .position(|(now, before)| *now && !*before)
                .or_else(|| selected.iter().position(|selected| *selected))
        })
        .flatten();
    if let Some(keep) = keep {
        let extra: Vec<&'static str> = (0..SELECTED.len())
            .filter(|&index| index != keep && std::mem::take(&mut selected[index]))
            .map(|index| SELECTED[index])
            .collect();
        if !extra.is_empty() {
            cx.defer_in(window, move |state, window, cx| {
                for id in extra {
                    state.set_value(id, Value::Bool(false), window, cx);
                }
            });
        }
    }
    // Recorded without a notification: it only answers the next frame's question.
    if *shown.read(cx) != selected {
        shown.update(cx, |shown, _| shown.clone_from(&selected));
    }
    // Rows act when they select or toggle their trailing control.
    let interactive = selection != "none" || trailing == "control";
    let mut setters = Vec::with_capacity(SELECTED.len());
    for id in SELECTED {
        setters.push(preview.setter(id, cx));
    }
    let setters = std::rc::Rc::new(setters);

    let items = set.items[..count]
        .iter()
        .enumerate()
        .map(|(index, (headline, icon, trailing_text))| {
            let on = selected[index];
            // Segments stand on their own container so their shapes read
            // against the surface.
            let mut item =
                ListItem::new(SharedString::from(format!("list-item-{index}")), *headline)
                    .switch_icons(SWITCH_ICONS[config.choice("switch_icons")])
                    .selected(on && selection != "none")
                    .expressive(config.bool("expressive"))
                    .density(density)
                    .pill(pill)
                    .disabled(config.bool("disable_last") && index + 1 == count)
                    .container(if segmented {
                        ColorRole::SURFACE_CONTAINER
                    } else {
                        ColorRole::SURFACE
                    });
            if overline {
                item = item.overline(set.overline);
            }
            if lines != "1" && !supporting.is_empty() {
                item = item.supporting(supporting.clone());
                if lines == "3" && !overline {
                    item = item.three_line(true);
                }
            }
            item = item.leading(match config.option("leading") {
                "icon" => ListLeading::Icon(symbol(icon, 24.)),
                "tinted" => ListLeading::Tinted(symbol(icon, 24.), Tint::Teal, 40.),
                "avatar" => ListLeading::Monogram(
                    headline
                        .chars()
                        .next()
                        .map(String::from)
                        .unwrap_or_default()
                        .into(),
                ),
                "image" => ListLeading::Image(Some(
                    super::card::scene("list-image")
                        .size_full()
                        .into_any_element(),
                )),
                "video" => ListLeading::Video(Some(
                    super::card::scene("list-video")
                        .size_full()
                        .into_any_element(),
                )),
                _ => ListLeading::None,
            });
            item = match trailing {
                "text" => item.trailing_text(*trailing_text),
                "icon" => item.trailing(ListTrailing::Icon(symbol(
                    if *icon == "call" {
                        "call"
                    } else {
                        "chevron_right"
                    },
                    24.,
                ))),
                "control" => item.trailing(match selection {
                    "single" => ListTrailing::Radio(on),
                    "multi" => ListTrailing::Checkbox(on),
                    _ => ListTrailing::Switch(on),
                }),
                _ => item,
            };
            match config.option("dividers") {
                "full width" if index + 1 < count => item = item.divider(true),
                "inset" if index + 1 < count => item = item.divider(true).divider_inset(true),
                _ => {}
            }
            if interactive {
                let single = selection == "single";
                let setters = setters.clone();
                item = item.on_click(move |_, window, cx| {
                    if single {
                        // One choice: select this row, clear the others.
                        for (other, set) in setters[..count].iter().enumerate() {
                            set(Value::Bool(other == index), window, cx);
                        }
                    } else {
                        setters[index](Value::Bool(!on), window, cx);
                    }
                });
            }
            item
        })
        .collect::<Vec<_>>();
    let list = if segmented {
        segmented_list(items)
    } else {
        // A pill stands in from the list's edges, as a navigation drawer's 12dp padding sets it.
        div()
            .flex()
            .flex_col()
            .w_full()
            .when(pill, |list| list.px(rem(12.)))
            .children(items)
    };
    let label = config.text("label");
    div()
        .id("list-preview")
        .debug_selector(|| "playground-list".into())
        .w(rem(560.))
        .max_w_full()
        .flex()
        .flex_col()
        .role(Role::List)
        .when(!label.is_empty(), |list| list.aria_label(label))
        .when(config.bool("subheader"), |column| {
            column.child(list_subheader("list-subheader", set.subheader))
        })
        .child(list)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::super::playground::Value;
    use super::super::nav_harness::Harness;
    use super::PLAYGROUND;
    use gpui::TestAppContext;

    #[gpui::test]
    fn every_scenario_renders(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.render_every_scenario(|harness, _| {
            harness.expect("playground-list");
        });
    }

    #[gpui::test]
    fn pill_destinations_select_by_click(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.expect("playground-list");
        harness.set("shape", Value::Choice(1));
        let list = harness.expect("playground-list");
        // Default: three rows, the first selected; the pills stand 12dp in.
        let third = gpui::point(list.center().x, list.origin.y + list.size.height * 5. / 6.);
        harness.click_at(third);
        let config = harness.config();
        assert!(!config.bool("selected_0"));
        assert!(config.bool("selected_2"));
    }

    #[gpui::test]
    fn clicking_rows_selects_one_or_toggles_many(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        let list = harness.expect("playground-list");
        // Default: single selection, first item selected; rows are 72dp.
        let third = gpui::point(list.center().x, list.origin.y + list.size.height * 5. / 6.);
        harness.click_at(third);
        let config = harness.config();
        assert!(!config.bool("selected_0"));
        assert!(config.bool("selected_2"));

        harness.set("selection", Value::Choice(2));
        let first = gpui::point(list.center().x, list.origin.y + list.size.height / 6.);
        harness.click_at(first);
        let config = harness.config();
        assert!(config.bool("selected_0"));
        assert!(
            config.bool("selected_2"),
            "multi selection keeps the others"
        );

        // Back to one choice, one stays selected.
        harness.set("selection", Value::Choice(1));
        let config = harness.config();
        assert!(config.bool("selected_0"));
        assert!(!config.bool("selected_2"), "single selection keeps one");
        harness.set("selection", Value::Choice(2));

        // Back to one choice from a multiple selection: the first stays.
        harness.set("selection", Value::Choice(2));
        harness.set("selected_2", Value::Bool(true));
        harness.set("selection", Value::Choice(1));
        let config = harness.config();
        assert!(config.bool("selected_0") && !config.bool("selected_2"));
        harness.set("selection", Value::Choice(2));

        // More items and a subheader grow the list.
        harness.set("count", Value::Choice(1));
        harness.set("subheader", Value::Bool(true));
        assert!(harness.expect("playground-list").size.height > list.size.height);
    }

    #[gpui::test]
    fn density_minus3_rows_are_44dp(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.expect("playground-list");
        harness.set("lines", Value::Choice(0));
        let standard = harness.expect("playground-list").size.height;
        harness.set("density", Value::Choice(2));
        // One-line rows, 44dp where a standard item is 56dp
        // (the Gallery's rem follows its text scale).
        let minus3 = harness.expect("playground-list").size.height;
        assert!(
            (minus3 / standard - 44. / 56.).abs() < 0.01,
            "{minus3:?} against {standard:?}"
        );
    }

    #[gpui::test]
    fn a_later_rows_switch_takes_the_single_selection(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.expect("playground-list");
        // Single selection with the first row selected: turning on the third
        // row's switch selects it, rather than being reverted.
        harness.set("selected_2", Value::Bool(true));
        let config = harness.config();
        assert!(config.bool("selected_2"), "the row just turned on stays");
        assert!(!config.bool("selected_0"), "the other goes");
        // And an earlier row's switch takes it back.
        harness.set("selected_0", Value::Bool(true));
        let config = harness.config();
        assert!(config.bool("selected_0"));
        assert!(!config.bool("selected_2"));
    }
}
