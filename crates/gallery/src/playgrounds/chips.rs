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

    /// Move the chip at display position `from` to position `to`.
    fn reorder(&mut self, from: usize, to: usize) {
        if from >= self.order.len() || to >= self.order.len() || from == to {
            return;
        }
        let chip = self.order.remove(from);
        self.order.insert(to, chip);
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
            ChipSetItem::new(slot).drag_preview(move |_, _| look.dragged(position))
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
}
