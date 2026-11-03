//! Chips playground: md3.io's Chips configuration (type, elevated,
//! vertical layout, leading icons, avatars, a trailing menu, dragging,
//! chip sets, group label, multiple or required selection, disabled) on the
//! real `AssistChip`, `FilterChip`, `InputChip` and `SuggestionChip`.
//!
//! Filter, input and suggestion chips select (one at a time unless
//! "Multiple selection"; "Selection required" keeps the last one); input
//! chips remove themselves; a filter chip's trailing menu opens a real menu
//! whose choice becomes the chip's label; draggable chips reorder by drag
//! and drop, lifting under the drag layer. Input chips have no elevated
//! style in M3, so Elevated leaves them flat.
use super::prelude::*;
use std::rc::Rc;

/// A chip set: each chip's label and leading icon.
const SETS: [&[(&str, &str)]; 5] = [
    &[
        ("Event", "event"),
        ("Reminder", "alarm"),
        ("Place", "place"),
        ("Contact", "person"),
    ],
    &[
        ("Ada Lovelace", "person"),
        ("Grace Hopper", "person"),
        ("Alan Turing", "person"),
        ("Katherine Johnson", "person"),
    ],
    &[
        ("Add to calendar", "event"),
        ("Get directions", "directions"),
        ("Share", "share"),
        ("Save", "bookmark"),
    ],
    &[
        ("Sounds good!", "thumb_up"),
        ("On my way", "directions_walk"),
        ("Can we talk later?", "schedule"),
        ("Thanks!", "favorite"),
    ],
    &[
        ("Electronics", "devices"),
        ("Clothing", "checkroom"),
        ("Books", "menu_book"),
        ("Home", "home"),
        ("Toys", "toys"),
    ],
];

/// A trailing menu's choices.
const MENU: [&str; 3] = ["Any time", "Past week", "Past month"];

const CHIP_IDS: [&str; 5] = ["chip-0", "chip-1", "chip-2", "chip-3", "chip-4"];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Email recipients",
            values: &[
                ("type", Set::Option("input")),
                ("set", Set::Option("Email recipients")),
                ("avatar", Set::Bool(true)),
                ("group_label", Set::Text("Recipients")),
            ],
        },
        Scenario {
            name: "Experience actions",
            values: &[
                ("type", Set::Option("assist")),
                ("set", Set::Option("Experience actions")),
                ("elevated", Set::Bool(true)),
                ("group_label", Set::Text("Actions")),
            ],
        },
        Scenario {
            name: "Quick replies",
            values: &[
                ("type", Set::Option("suggestion")),
                ("set", Set::Option("Quick replies")),
                ("icons", Set::Bool(false)),
                ("group_label", Set::Text("Suggested replies")),
            ],
        },
        Scenario {
            name: "Category filters",
            values: &[
                ("type", Set::Option("filter")),
                ("set", Set::Option("Category filters")),
                ("multiple", Set::Bool(true)),
                ("menu", Set::Bool(true)),
                ("group_label", Set::Text("Categories")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Select {
                    id: "type",
                    label: "Type",
                    options: &["assist", "filter", "input", "suggestion"],
                    default: 1,
                },
                Control::Switch {
                    id: "elevated",
                    label: "Elevated",
                    default: false,
                },
                Control::Switch {
                    id: "vertical",
                    label: "Vertical layout",
                    default: false,
                },
                Control::Switch {
                    id: "icons",
                    label: "Leading icons",
                    default: true,
                },
                Control::Switch {
                    id: "avatar",
                    label: "Avatar",
                    default: false,
                },
                Control::Switch {
                    id: "menu",
                    label: "Trailing menu",
                    default: false,
                },
                Control::Switch {
                    id: "draggable",
                    label: "Draggable",
                    default: false,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "set",
                    label: "Chip set",
                    options: &[
                        "Default",
                        "Email recipients",
                        "Experience actions",
                        "Quick replies",
                        "Category filters",
                    ],
                    default: 0,
                },
                Control::Text {
                    id: "group_label",
                    label: "Group label",
                    default: "Reminders",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "multiple",
                    label: "Multiple selection",
                    default: false,
                },
                Control::Switch {
                    id: "required",
                    label: "Selection required",
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

/// The chip set's live state: display order (indices into the set), which
/// chips are selected or removed, each chip's menu choice and the open menu.
#[derive(Clone, Debug, Default, PartialEq)]
struct ChipsState {
    order: Vec<usize>,
    selected: Vec<bool>,
    removed: Vec<bool>,
    choice: Vec<Option<usize>>,
    open_menu: Option<usize>,
}

impl ChipsState {
    fn new(len: usize) -> Self {
        Self {
            order: (0..len).collect(),
            selected: vec![false; len],
            removed: vec![false; len],
            choice: vec![None; len],
            open_menu: None,
        }
    }

    /// Move the chip at display position `from` to position `to`. Display positions
    /// count only the chips still shown; `order` still holds the removed ones.
    fn reorder(&mut self, from: usize, to: usize) {
        let position = |shown| {
            self.order
                .iter()
                .enumerate()
                .filter(|(_, chip)| !self.removed[**chip])
                .map(|(position, _)| position)
                .nth(shown)
        };
        let (Some(from), Some(to)) = (position(from), position(to)) else {
            return;
        };
        if from != to {
            let chip = self.order.remove(from);
            self.order.insert(to, chip);
        }
    }
}

/// What a chip shows, shared by the chip and its drag preview.
#[derive(Clone)]
struct ChipLook {
    kind: &'static str,
    label: SharedString,
    icon: Option<&'static str>,
    avatar: bool,
    elevated: bool,
    menu: bool,
    selected: bool,
    disabled: bool,
}

impl ChipLook {
    fn dragged(&self, position: usize) -> AnyElement {
        chip(("dragged-chip", position), self, true, None, None, None)
    }
}

type Toggle = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type Remove = Rc<dyn Fn(&mut Window, &mut App)>;

/// An avatar: the label's initial on a primary container.
fn avatar(label: &SharedString) -> Div {
    let initial: String = label.chars().take(1).collect();
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(color(ColorRole::PRIMARY_CONTAINER))
        .text_color(color(ColorRole::ON_PRIMARY_CONTAINER))
        .map(|text| type_role(text, TypeRole::LabelMedium))
        .child(initial)
}

/// The real chip of `look.kind`.
fn chip(
    id: impl Into<ElementId>,
    look: &ChipLook,
    dragged: bool,
    toggle: Option<Toggle>,
    remove: Option<(SharedString, Remove)>,
    click: Option<Remove>,
) -> AnyElement {
    let icon = look.icon.map(|name| symbol(name, 18.));
    let label = look.label.clone();
    match look.kind {
        "assist" => AssistChip::new(id, label)
            .elevated(look.elevated)
            .disabled(look.disabled)
            .dragged(dragged)
            .when_some(icon, |chip, icon| chip.icon(icon))
            .when_some(click, |chip, click| {
                chip.on_click(move |_, window, cx| click(window, cx))
            })
            .into_any_element(),
        "input" => InputChip::new(id, label.clone())
            .selected(look.selected)
            .disabled(look.disabled)
            .dragged(dragged)
            .when(look.avatar, |chip| chip.avatar(avatar(&label)))
            .when(!look.avatar, |chip| match icon {
                Some(icon) => chip.icon(icon),
                None => chip,
            })
            .when_some(remove, |chip, (name, remove)| {
                chip.on_remove(name, move |window, cx| remove(window, cx))
            })
            .when_some(toggle, |chip, toggle| {
                let next = !look.selected;
                chip.on_click(move |_, window, cx| toggle(next, window, cx))
            })
            .into_any_element(),
        "suggestion" => SuggestionChip::new(id, label)
            .selected(look.selected)
            .elevated(look.elevated)
            .disabled(look.disabled)
            .dragged(dragged)
            .when_some(icon, |chip, icon| chip.icon(icon))
            .when_some(toggle, |chip, toggle| {
                let next = !look.selected;
                chip.on_click(move |_, window, cx| toggle(next, window, cx))
            })
            .into_any_element(),
        _ => FilterChip::new(id, label)
            .selected(look.selected)
            .elevated(look.elevated)
            .disabled(look.disabled)
            .dragged(dragged)
            .when_some(icon, |chip, icon| chip.icon(icon))
            .when(look.menu, |chip| {
                chip.trailing_icon(symbol("arrow_drop_down", 18.))
            })
            .when_some(toggle, |chip, toggle| {
                chip.on_toggle(move |next, window, cx| toggle(*next, window, cx))
            })
            .into_any_element(),
    }
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let set = SETS[config.choice("set")];
    let kind = config.option("type");
    let state = preview.local("chips", window, cx, |_, _| ChipsState::new(set.len()));
    // A new set or type starts fresh.
    if preview.changed("chips-reset", (config.choice("set"), kind)) {
        state.update(cx, |state, _| *state = ChipsState::new(set.len()));
    }
    let snapshot = state.read(cx).clone();
    let rules = ChipSelection {
        multiple: config.bool("multiple"),
        required: config.bool("required"),
    };
    let disabled = config.bool("disabled");
    let draggable = config.bool("draggable") && !disabled;
    let menu = config.bool("menu") && kind == "filter";
    let elevated = config.bool("elevated") && kind != "input";
    let notify = |state: &Entity<ChipsState>, cx: &mut App, f: &dyn Fn(&mut ChipsState)| {
        state.update(cx, |state, cx| {
            f(state);
            cx.notify();
        })
    };
    let chips = snapshot
        .order
        .iter()
        .enumerate()
        .filter(|(_, index)| !snapshot.removed[**index])
        .map(|(position, &index)| {
            let (name, icon) = set[index];
            let label: SharedString = match snapshot.choice[index] {
                Some(choice) if menu => MENU[choice].into(),
                _ => name.into(),
            };
            let look = ChipLook {
                kind,
                label: label.clone(),
                icon: config.bool("icons").then_some(icon),
                avatar: config.bool("avatar"),
                elevated,
                menu,
                selected: snapshot.selected[index],
                disabled,
            };
            let toggle: Option<Toggle> = (kind != "assist" && !menu).then(|| {
                let state = state.clone();
                Rc::new(move |on: bool, _: &mut Window, cx: &mut App| {
                    notify(&state, cx, &|s| {
                        rules.apply(&mut s.selected, index, on);
                    })
                }) as Toggle
            });
            let remove = (kind == "input").then(|| {
                let state = state.clone();
                let remove: Remove = Rc::new(move |_: &mut Window, cx: &mut App| {
                    notify(&state, cx, &|s| s.removed[index] = true)
                });
                (SharedString::from(format!("Remove {label}")), remove)
            });
            let element = chip(("chip", index), &look, false, toggle, remove, None);
            let element = if menu {
                let open = snapshot.open_menu == Some(index);
                let opener = state.clone();
                let chooser = state.clone();
                MenuPopup::new(
                    ("chip-menu", index),
                    element,
                    MaterialMenu::new(("chip-menu-items", index)).items(
                        MENU.iter().enumerate().map(|(choice, item)| {
                            let chooser = chooser.clone();
                            MaterialMenuItem::new(*item)
                                .selected(snapshot.choice[index] == Some(choice))
                                .on_click(move |_, _, cx| {
                                    notify(&chooser, cx, &|s| {
                                        s.choice[index] = Some(choice);
                                        s.selected[index] = choice > 0;
                                        s.open_menu = None;
                                    })
                                })
                        }),
                    ),
                )
                .open(open && !disabled)
                .on_open_change(move |next: &bool, _, cx| {
                    let next = *next;
                    notify(&opener, cx, &|s| s.open_menu = next.then_some(index))
                })
                .into_any_element()
            } else {
                element
            };
            let slot = div()
                .debug_selector(move || CHIP_IDS[position].into())
                .child(element);
            ChipSetItem::new(("chip-item", index), slot)
                .drag_preview(move |_, _| look.dragged(position))
        });
    let reorder = state.clone();
    let set = ChipSet::new("chip-set", config.text("group_label"))
        .vertical(config.bool("vertical"))
        .max_w(rem(360.))
        .justify_center()
        .items(chips)
        .when(draggable, |set| {
            set.on_reorder(move |from, to, _, cx| notify(&reorder, cx, &|s| s.reorder(from, to)))
        });
    div()
        .debug_selector(|| "playground-chips".into())
        .child(set)
        .into_any_element()
}

fn code(config: &Config) -> String {
    let set = SETS[config.choice("set")];
    let kind = config.option("type");
    let disabled = config.bool("disabled");
    let draggable = config.bool("draggable") && !disabled;
    let menu = config.bool("menu") && kind == "filter";
    let elevated = config.bool("elevated") && kind != "input";
    let avatar = config.bool("avatar") && kind == "input";
    let icons = config.bool("icons") && !avatar;
    let rules = format!(
        "ChipSelection {{ multiple: {}, required: {} }}",
        config.bool("multiple"),
        config.bool("required")
    );
    let head = match kind {
        "assist" => "AssistChip",
        "input" => "InputChip",
        "suggestion" => "SuggestionChip",
        _ => "FilterChip",
    };
    let chip = chain(
        format!("{head}::new((\"chip\", index), label)"),
        [
            elevated.then(|| "elevated(true)".into()),
            (kind != "assist").then(|| "selected(self.selected[index])".into()),
            disabled.then(|| "disabled(true)".into()),
            icons.then(|| "icon(icon)".into()),
            avatar.then(|| {
                "avatar(Avatar::new((\"avatar\", index), label).monogram(monogram_initials(label)).size(24.))"
                    .into()
            }),
            menu.then(|| "trailing_icon(Symbol::ARROW_DROP_DOWN)".into()),
            (kind == "input").then(|| {
                format!(
                    "on_remove(format!(\"Remove {{label}}\"), {})",
                    view_update("move |_, cx|", "this.removed[index] = true;")
                )
            }),
            match kind {
                "assist" => Some("on_click(move |_, _, _| println!(\"{label}\"))".into()),
                "filter" if menu => None,
                "filter" => Some(format!(
                    "on_toggle({})",
                    listener(
                        "move |this, on: &bool, _, cx|",
                        &format!("{rules}.apply(&mut this.selected, index, *on);")
                    )
                )),
                _ => Some(format!(
                    "on_click({})",
                    listener(
                        "move |this, _: &ClickEvent, _, cx|",
                        &format!(
                            "let on = !this.selected[index];\n{rules}.apply(&mut this.selected, index, on);"
                        )
                    )
                )),
            },
        ],
    );
    // A filter chip with a menu opens it; a choice selects the chip.
    let chip = if menu {
        let item = chain(
            "MaterialMenuItem::new(choice)",
            [
                Some("selected(self.choice[index] == Some(position))".into()),
                Some(format!(
                    "on_click({})",
                    listener(
                        "move |this, _: &ClickEvent, _, cx|",
                        "this.choice[index] = Some(position);\nthis.selected[index] = position > 0;"
                    )
                )),
            ],
        );
        let choices: Vec<String> = MENU.iter().map(|choice| lit(choice)).collect();
        chain(
            format!(
                "MenuPopup::new(\n    (\"chip-menu\", index),\n    {},\n    MaterialMenu::new((\"chip-menu-items\", index)).items(\n        [{}].into_iter().enumerate().map(|(position, choice)| {{\n            {}\n        }}),\n    ),\n)",
                indent(&chip),
                choices.join(", "),
                item.replace('\n', "\n            ")
            ),
            [
                Some("open(self.open_menu == Some(index))".into()),
                Some(format!(
                    "on_open_change({})",
                    listener(
                        "move |this, open: &bool, _, cx|",
                        "this.open_menu = open.then_some(index);"
                    )
                )),
            ],
        )
    } else {
        chip
    };
    let mut item = format!("ChipSetItem::new(\n    label,\n    {},\n)", indent(&chip));
    if draggable {
        // The chip under the pointer while it is dragged.
        item = chain(
            item,
            [Some(format!(
                "drag_preview(move |_, _| {head}::new(\"dragged\", label).dragged(true).into_any_element())"
            ))],
        );
    }
    let chips: Vec<String> = set
        .iter()
        .map(|(label, icon)| {
            if icons {
                format!("({}, {})", lit(label), symbol_code(icon))
            } else {
                lit(label)
            }
        })
        .collect();
    let binding = if icons { "(label, icon)" } else { "label" };
    // A set that reorders shows its chips in the view's order.
    let items = if draggable {
        format!(
            "items(self.order.iter().map(|&index| {{\n    let {binding} = CHIPS[index];\n    {}\n}}))",
            indent(&item)
        )
    } else {
        format!(
            "items([{}].into_iter().enumerate().map(|(index, {binding})| {{\n    {}\n}}))",
            chips.join(", "),
            indent(&item)
        )
    };
    let chip_set = chain(
        format!(
            "ChipSet::new(\"chips\", {})",
            lit(&config.text("group_label"))
        ),
        [
            config.bool("vertical").then(|| "vertical(true)".into()),
            Some(items),
            draggable.then(|| {
                format!(
                    "on_reorder({})",
                    view_update(
                        "move |from, to, _, cx|",
                        "let chip = this.order.remove(from);\nthis.order.insert(to, chip);"
                    )
                )
            }),
        ],
    );
    if draggable {
        let element = if icons { "(&str, Symbol)" } else { "&str" };
        format!(
            "// const CHIPS: [{element}; {}] = [{}];\n{chip_set}",
            chips.len(),
            chips.join(", ")
        )
    } else {
        chip_set
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn reordering_moves_a_chip_and_ignores_out_of_range() {
        let mut state = ChipsState::new(3);
        state.reorder(2, 0);
        assert_eq!(state.order, [2, 0, 1]);
        state.reorder(0, 9);
        assert_eq!(state.order, [2, 0, 1], "out of range is ignored");
    }

    #[test]
    fn reordering_counts_only_the_chips_shown() {
        let mut state = ChipsState::new(4);
        state.removed[0] = true;
        // Shown: 1, 2, 3. The last goes first.
        state.reorder(2, 0);
        let shown = |state: &ChipsState| -> Vec<usize> {
            let chips = state.order.iter().copied();
            chips.filter(|chip| !state.removed[*chip]).collect()
        };
        assert_eq!(shown(&state), [3, 1, 2]);
        state.reorder(0, 2);
        assert_eq!(shown(&state), [1, 2, 3]);
        state.reorder(0, 3);
        assert_eq!(shown(&state), [1, 2, 3], "past the chips shown is ignored");
    }

    #[gpui::test]
    fn reset_forgets_what_the_preview_holds(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, &PLAYGROUND);
        click(cx, "chip-0");
        let chips = |state: &Entity<PlaygroundState>, cx: &mut VisualTestContext| {
            state.read_with(cx, |state, cx| {
                state
                    .local_entity::<ChipsState>("chips")
                    .unwrap()
                    .read(cx)
                    .selected
                    .clone()
            })
        };
        assert!(chips(&state, cx)[0], "selected");
        cx.update(|window, cx| state.update(cx, |state, cx| state.reset(window, cx)));
        settle(cx);
        assert!(chips(&state, cx).iter().all(|selected| !selected), "reset");
    }

    #[gpui::test]
    fn every_scenario_renders_and_chips_select_and_remove(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, &PLAYGROUND);
        every_scenario_renders(&state, cx, "playground-chips");
        cx.update(|window, cx| state.update(cx, |state, cx| state.reset(window, cx)));
        settle(cx);
        // Without its own icon, a selected filter chip slides a check in and
        // widens.
        set(&state, cx, "icons", Value::Bool(false));
        let width = |cx: &mut VisualTestContext, chip| rect(cx, chip).size.width;
        let (rest0, rest1) = (width(cx, "chip-0"), width(cx, "chip-1"));
        click(cx, "chip-0");
        assert!(width(cx, "chip-0") > rest0, "selected");
        click(cx, "chip-1");
        assert_eq!(width(cx, "chip-0"), rest0, "one at a time");
        assert!(width(cx, "chip-1") > rest1);
        set(&state, cx, "multiple", Value::Bool(true));
        click(cx, "chip-0");
        assert!(width(cx, "chip-0") > rest0 && width(cx, "chip-1") > rest1);

        // Input chips remove themselves from their remove icon.
        scenario(&state, cx, "Email recipients");
        let chip = rect(cx, "chip-0");
        cx.simulate_click(
            point(chip.right() - px(16.), chip.center().y),
            Modifiers::none(),
        );
        settle(cx);
        assert!(cx.debug_bounds("chip-0").is_none(), "removed");
        assert!(cx.debug_bounds("chip-3").is_some());
    }

    #[test]
    fn the_code_follows_the_configuration() {
        let mut config = Config::defaults(&PLAYGROUND);
        let code = super::code(&config);
        assert!(
            code.starts_with("ChipSet::new(\"chips\", \"Reminders\")"),
            "{code}"
        );
        assert!(
            code.contains("FilterChip::new((\"chip\", index), label)"),
            "{code}"
        );
        assert!(code.contains("(\"Event\", Symbol::EVENT)"), "{code}");
        assert!(!code.contains(".elevated("));
        config.set("elevated", Value::Bool(true));
        assert!(super::code(&config).contains(".elevated(true)"));
        config.set("menu", Value::Bool(true));
        assert!(super::code(&config).contains("MenuPopup::new("));
        config.set("draggable", Value::Bool(true));
        assert!(super::code(&config).contains(".on_reorder("));
    }
}
