//! Select playground: md3.io's Select configuration (variant, density,
//! option set, label and whether it shows, selection, supporting text, a disabled option, error,
//! required, disabled) on the real `ExposedDropdown`, whose menu opens and
//! really selects.
//!
//! md3.io's Selected options are its default set's values; here they are
//! positions in the chosen option set, so they work for every set. The Models
//! set opens a menu the caller builds, whose items carry supporting text and
//! icons under a group label, followed by an action that is not a value.
use super::prelude::*;
use std::rc::Rc;

/// The Models set: each model's name, its supporting text and its icon.
const MODELS: [(&str, &str, &str); 3] = [
    ("Fast", "Quick replies to short questions", "bolt"),
    ("Balanced", "Good summaries at a moderate cost", "balance"),
    ("Deep", "Careful reasoning for long documents", "psychology"),
];

const SETS: [&[&str]; 4] = [
    &["Apple", "Banana", "Cherry"],
    &["Alaska", "California", "Colorado", "New York", "Texas"],
    &[
        "No restrictions",
        "Vegetarian",
        "Vegan",
        "Gluten-free",
        "Halal",
    ],
    &["Fast", "Balanced", "Deep"],
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
        Scenario {
            name: "Model with a rich menu",
            values: &[
                ("set", Set::Option("Models")),
                ("label", Set::Text("Model")),
                ("selected", Set::Option("3")),
                ("supporting", Set::Text("Used for new requests")),
                ("full_width", Set::Bool(true)),
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
                Control::Switch {
                    id: "full_width",
                    label: "Full width",
                    default: false,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Switch {
                    id: "limit_height",
                    label: "Limit menu height",
                    default: false,
                },
                Control::Select {
                    id: "set",
                    label: "Select",
                    options: &["Default", "State", "Diet", "Models"],
                    default: 0,
                },
                Control::Text {
                    id: "label",
                    label: "Label",
                    default: "Fruit",
                },
                Control::Switch {
                    id: "show_label",
                    label: "Show label",
                    default: true,
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
                    label: "Disable the second option",
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
    code: None,
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
    let select = Rc::new(preview.setter("selected", cx));
    // The Models menu's action shows that it ran, in a snackbar under the field.
    let managed = preview.local("managed", window, cx, |_, _| false);
    let is_managed = *managed.read(cx);
    let set_managed = {
        let managed = managed.clone();
        move |on: bool, cx: &mut App| {
            managed.update(cx, |managed, cx| {
                *managed = on;
                cx.notify();
            })
        }
    };
    let models = config.option("set") == "Models";
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
        .show_label(config.bool("show_label"))
        .open(is_open)
        .error(config.bool("error"))
        .required(config.bool("required"))
        .disabled(config.bool("disabled"))
        .width(280.)
        .when(config.bool("limit_height"), |dropdown| {
            dropdown.menu_max_height(120.)
        })
        .on_open_change({
            let open = open.clone();
            move |next: &bool, _, cx| {
                open.update(cx, |open, cx| {
                    *open = *next;
                    cx.notify();
                })
            }
        });
    // A custom menu carries its items' selection, disabled state and
    // handlers itself, so the Models set sets none of the options' own.
    dropdown = if models {
        let set_managed = set_managed.clone();
        dropdown
            .menu(models_menu(
                selected,
                select,
                config.bool("disable_second"),
                move |_, cx| set_managed(true, cx),
            ))
            .value(selected.map_or("", |index| MODELS[index].0))
    } else {
        dropdown
            .options(options.iter().copied())
            .disabled_options(config.bool("disable_second").then_some(1))
            .selected(selected)
            .on_select(move |index: &usize, window, cx| {
                open.update(cx, |open, cx| {
                    *open = false;
                    cx.notify();
                });
                select(Value::Choice(index + 1), window, cx);
            })
    };
    if !supporting.is_empty() {
        dropdown = dropdown.supporting_text(supporting);
    }
    let full_width = config.bool("full_width");
    if full_width {
        dropdown = dropdown.full_width();
    }
    div()
        .flex()
        .flex_col()
        .gap(rem(16.))
        .when(full_width, |column| column.w_full())
        .child(
            div()
                .debug_selector(|| "playground-select".into())
                .when(full_width, |select| select.w_full())
                .child(dropdown),
        )
        .when(models, |column| {
            column.child(
                Snackbar::new("playground-select-managed", "Opened model management")
                    .debug_name("playground-select-managed")
                    .open(is_managed)
                    .duration(SnackbarDuration::Short)
                    .on_dismiss(move |_, cx| set_managed(false, cx)),
            )
        })
        .into_any_element()
}

/// A menu of models with what each is for, under a group label, then an
/// action that manages them rather than choosing one.
fn models_menu(
    selected: Option<usize>,
    select: Rc<impl Fn(Value, &mut Window, &mut App) + 'static>,
    disable_second: bool,
    on_manage: impl Fn(&mut Window, &mut App) + 'static,
) -> MaterialMenu {
    let models = MODELS
        .iter()
        .enumerate()
        .map(|(index, &(name, supporting, icon))| {
            let select = select.clone();
            MaterialMenuItem::new(name)
                .supporting_text(supporting)
                .leading(symbol(icon, 24.))
                .selected(selected == Some(index))
                .disabled(disable_second && index == 1)
                .on_click(move |_, window, cx| select(Value::Choice(index + 1), window, cx))
        });
    MaterialMenu::new("playground-select-models")
        .style(MenuStyle::Baseline)
        .group(MenuGroup::new().label("Models").items(models))
        .group(
            MenuGroup::new().item(
                MaterialMenuItem::new("Manage models…")
                    .leading(symbol("settings", 24.))
                    .on_click(move |_, window, cx| on_manage(window, cx)),
            ),
        )
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

    #[gpui::test]
    fn the_models_set_opens_a_rich_menu_that_chooses_a_model(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, &PLAYGROUND);
        set(&state, cx, "set", Value::Choice(3));
        set(&state, cx, "selected", Value::Choice(0));
        open_menu(cx);
        let surface = rect(cx, "menu-popup-surface");
        // Under the group label, the second model's two-line row.
        let menu = models_menu(
            None,
            Rc::new(|_, _: &mut Window, _: &mut App| {}),
            false,
            |_, _| {},
        );
        let (top, bottom) = (menu.item_top(1).unwrap(), menu.item_bottom(1).unwrap());
        cx.simulate_click(
            point(surface.center().x, surface.top() + px((top + bottom) / 2.)),
            Modifiers::none(),
        );
        settle(cx);
        assert_eq!(config(&state, cx).option("selected"), "2");
        assert!(cx.debug_bounds("menu-popup-surface").is_none(), "closed");
    }

    #[gpui::test]
    fn the_models_menus_action_shows_a_snackbar(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, &PLAYGROUND);
        set(&state, cx, "set", Value::Choice(3));
        assert!(cx.debug_bounds("playground-select-managed").is_none());
        open_menu(cx);
        let surface = rect(cx, "menu-popup-surface");
        let menu = models_menu(
            None,
            Rc::new(|_, _: &mut Window, _: &mut App| {}),
            false,
            |_, _| {},
        );
        // The action after the three models; the Gallery's rem follows its text
        // scale, so design px scale to the surface.
        let (top, bottom) = (menu.item_top(3).unwrap(), menu.item_bottom(3).unwrap());
        let scale = f32::from(surface.size.height) / menu.visible_height();
        cx.simulate_click(
            point(
                surface.center().x,
                surface.top() + px((top + bottom) / 2. * scale),
            ),
            Modifiers::none(),
        );
        settle(cx);
        assert!(cx.debug_bounds("menu-popup-surface").is_none(), "closed");
        assert!(
            cx.debug_bounds("playground-select-managed").is_some(),
            "the snackbar says the action ran"
        );
        assert_eq!(
            config(&state, cx).option("selected"),
            "1",
            "no model chosen"
        );
    }
}
