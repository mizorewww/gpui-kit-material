//! Button group playground: md3.io's Button group configuration (kind,
//! variant, size, square shape, orientation, density, equal widths, items,
//! content, labels, selection, required selection, disabled) on the real
//! `ButtonGroup` (standard: toggle buttons that widen when pressed) and
//! `ConnectedButtonGroup` (connected shapes). Clicking options selects them
//! under the chosen selection rule.
use super::prelude::*;

const VARIANTS: [ButtonVariant; 5] = [
    ButtonVariant::Filled,
    ButtonVariant::Tonal,
    ButtonVariant::Outlined,
    ButtonVariant::Elevated,
    ButtonVariant::Text,
];

/// One items preset: (label, icon) options and the initial selection.
struct Items {
    options: &'static [(&'static str, &'static str)],
    selected: &'static [&'static str],
}

const ITEMS: [Items; 3] = [
    Items {
        options: &[
            ("Bold", "format_bold"),
            ("Italic", "format_italic"),
            ("Underline", "format_underlined"),
        ],
        selected: &["Bold", "Italic"],
    },
    Items {
        options: &[
            ("XS", "looks_one"),
            ("S", "looks_two"),
            ("M", "looks_3"),
            ("L", "looks_4"),
            ("XL", "looks_5"),
        ],
        selected: &["M"],
    },
    Items {
        options: &[
            ("$", "savings"),
            ("$$", "payments"),
            ("$$$", "credit_card"),
            ("$$$$", "diamond"),
        ],
        selected: &["$$"],
    },
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Select size",
            values: &[
                ("items", Set::Option("Select size")),
                ("variant", Set::Option("tonal")),
                ("selection", Set::Option("single")),
                ("required", Set::Bool(true)),
                ("equal", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Price",
            values: &[
                ("items", Set::Option("Price")),
                ("kind", Set::Option("standard")),
                ("variant", Set::Option("outlined")),
                ("content", Set::Option("both")),
                ("labels", Set::Option("selected")),
                ("selection", Set::Option("single")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Segmented {
                    id: "kind",
                    label: "Kind",
                    options: &["standard", "connected"],
                    default: 1,
                },
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
            title: "Layout",
            controls: &[
                Control::Segmented {
                    id: "orientation",
                    label: "Orientation",
                    options: &["horizontal", "vertical"],
                    default: 0,
                },
                Control::Select {
                    id: "density",
                    label: "Density",
                    options: &["default", "comfortable", "compact"],
                    default: 0,
                },
                Control::Switch {
                    id: "equal",
                    label: "Equal widths",
                    default: false,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "items",
                    label: "Items",
                    options: &["Default", "Select size", "Price"],
                    default: 0,
                },
                Control::Segmented {
                    id: "content",
                    label: "Content",
                    options: &["text", "icons", "both"],
                    default: 0,
                },
                Control::Segmented {
                    id: "labels",
                    label: "Labels",
                    options: &["always", "selected"],
                    default: 0,
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
                    default: 2,
                },
                Control::Switch {
                    id: "required",
                    label: "Require a selection",
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

#[cfg(test)]
thread_local! {
    /// The selection the preview last rendered, for tests.
    static RENDERED: std::cell::RefCell<Vec<SharedString>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// The preview's selection and last activated action.
#[derive(Default)]
struct Demo {
    selected: Vec<SharedString>,
    status: Option<SharedString>,
}

impl Demo {
    /// Apply a click on `value` under the selection rule.
    fn toggle(&mut self, value: &SharedString, on: bool, selection: &str, required: bool) {
        match (selection, on) {
            ("single", true) => self.selected = vec![value.clone()],
            ("multi", true) => {
                if !self.selected.contains(value) {
                    self.selected.push(value.clone())
                }
            }
            (_, false) if required && self.selected.len() <= 1 => {}
            (_, false) => self.selected.retain(|selected| selected != value),
            _ => {}
        }
    }
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let demo = preview.local("button-group-demo", window, cx, |_, _| Demo::default());
    let items_index = config.choice("items");
    let items = &ITEMS[items_index];
    let selection = config.option("selection");
    let required = config.bool("required");
    if preview.changed("button-group-items", (items_index, selection)) {
        // A new item set (or rule) starts from its own selection.
        let initial: Vec<SharedString> = match selection {
            "none" => Vec::new(),
            "single" => items.selected[..1].iter().map(|v| (*v).into()).collect(),
            _ => items.selected.iter().map(|v| (*v).into()).collect(),
        };
        demo.update(cx, |demo, _| demo.selected = initial);
    }
    let state = demo.read(cx);
    let (selected, status) = (state.selected.clone(), state.status.clone());
    #[cfg(test)]
    RENDERED.with(|rendered| *rendered.borrow_mut() = selected.clone());
    let content = config.option("content");
    let (icons, text) = (content != "text", content != "icons");
    let labels_selected = config.option("labels") == "selected" && icons;
    let size = ButtonSize::ALL[config.choice("size")];
    let shape = if config.bool("square") {
        ButtonShape::Square
    } else {
        ButtonShape::Round
    };
    let vertical = config.option("orientation") == "vertical";
    let density = ButtonDensity::ALL[config.choice("density")];
    let equal = config.bool("equal");
    let disabled = config.bool("disabled");
    let variant = VARIANTS[config.choice("variant")];
    let id = ElementId::from(("playground-button-group", items_index));
    let report = {
        let demo = demo.clone();
        std::rc::Rc::new(move |value: &SharedString, cx: &mut App| {
            let value = value.clone();
            demo.update(cx, |demo, cx| {
                demo.status = Some(format!("Activated “{value}”").into());
                cx.notify();
            });
        })
    };
    let toggle = {
        let demo = demo.clone();
        std::rc::Rc::new(move |value: &SharedString, on: bool, cx: &mut App| {
            demo.update(cx, |demo, cx| {
                demo.toggle(value, on, selection, required);
                cx.notify();
            });
        })
    };
    let group = if config.option("kind") == "standard" {
        ButtonGroup::new(id, "Options")
            .button_size(size)
            .vertical(vertical)
            .density(density)
            .equal_widths(equal)
            .items(
                items
                    .options
                    .iter()
                    .enumerate()
                    .map(|(index, &(label, icon))| {
                        let value = SharedString::new_static(label);
                        let on = selected.contains(&value);
                        let show_text = text && (!labels_selected || on);
                        let mut button = MaterialButton::new(("playground-group-item", index))
                            .variant(variant)
                            .button_size(size)
                            .shape(shape)
                            .disabled(disabled)
                            .accessibility_label(label);
                        if icons {
                            button = button.icon(symbol(icon, 20.));
                        }
                        if show_text {
                            button = button.child(label);
                        }
                        if selection == "none" {
                            let report = report.clone();
                            button.on_click(move |_, _, cx| report(&value, cx))
                        } else {
                            let toggle = toggle.clone();
                            button
                                .toggle(true)
                                .selected(on)
                                .on_change(move |on, _, _, cx| toggle(&value, on, cx))
                        }
                    }),
            )
            .into_any_element()
    } else {
        let mut group = ConnectedButtonGroup::new(id, "Options")
            .variant(variant)
            .button_size(size)
            .shape(shape)
            .vertical(vertical)
            .density(density)
            .equal_widths(equal)
            .disabled(disabled)
            .options(items.options.iter().map(|&(label, _)| (label, label)))
            .labels(match (text, labels_selected) {
                (false, _) => GroupLabels::Never,
                (true, true) => GroupLabels::Selected,
                (true, false) => GroupLabels::Always,
            })
            .selection(match selection {
                "none" => GroupSelection::None,
                "single" => GroupSelection::Single,
                _ => GroupSelection::Multi,
            })
            .required(required)
            .selected_values(selected.clone());
        if icons {
            for &(label, icon) in items.options {
                group = group.icon(label, symbol(icon, 20.));
            }
        }
        if let Some(first) = selected.first() {
            group = group.selected(first.clone());
        }
        let (report, single) = (report.clone(), toggle.clone());
        group
            .on_change(move |value, _, cx| single(value, true, cx))
            .on_toggle(move |value, on, _, cx| toggle(value, on, cx))
            .on_activate(move |value, _, cx| report(value, cx))
            .into_any_element()
    };
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(rem(16.))
        .child(
            div()
                .flex()
                .debug_selector(|| "playground-button-group".into())
                .child(group),
        )
        .child(
            div()
                .h(rem(20.))
                .debug_selector(|| "playground-button-group-status".into())
                .map(|line| type_role(line, TypeRole::BodyMedium))
                .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                .children(status),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::harness::open;
    use super::*;
    use core::prelude::v1::test;

    fn selected(h: &mut super::super::harness::PlaygroundHarness) -> Vec<SharedString> {
        h.draw();
        RENDERED.with(|rendered| rendered.borrow().clone())
    }

    #[gpui::test]
    fn every_scenario_renders(cx: &mut TestAppContext) {
        let mut h = open(&PLAYGROUND, cx);
        h.renders_every_scenario("playground-button-group");
        // Both kinds, every orientation and selection rule.
        for (id, value) in [
            ("kind", Value::Choice(0)),
            ("orientation", Value::Choice(1)),
            ("selection", Value::Choice(0)),
            ("content", Value::Choice(1)),
        ] {
            h.set(id, value);
            assert!(h.cx.debug_bounds("playground-button-group").is_some());
        }
    }

    #[gpui::test]
    fn clicking_connected_options_follows_the_selection_rule(cx: &mut TestAppContext) {
        let mut h = open(&PLAYGROUND, cx);
        assert_eq!(selected(&mut h), ["Bold", "Italic"]);
        let group = h.bounds("playground-button-group");
        let underline = point(group.right() - px(20.), group.center().y);
        h.click(underline);
        assert_eq!(selected(&mut h), ["Bold", "Italic", "Underline"]);
        let bold = point(group.left() + px(20.), group.center().y);
        h.click(bold);
        assert_eq!(selected(&mut h), ["Italic", "Underline"]);
        // Single selection replaces; a required one cannot be cleared.
        h.set("selection", Value::Choice(1));
        h.set("required", Value::Bool(true));
        assert_eq!(selected(&mut h), ["Bold"]);
        let group = h.bounds("playground-button-group");
        h.click(point(group.right() - px(20.), group.center().y));
        assert_eq!(selected(&mut h), ["Underline"]);
    }

    #[gpui::test]
    fn standard_toggle_buttons_select_and_actions_report(cx: &mut TestAppContext) {
        let mut h = open(&PLAYGROUND, cx);
        h.set("kind", Value::Choice(0));
        let group = h.bounds("playground-button-group");
        h.click(point(group.left() + px(16.), group.center().y));
        assert_eq!(selected(&mut h), ["Italic"]);
        h.set("selection", Value::Choice(0));
        assert_eq!(
            h.bounds("playground-button-group-status").size.width,
            px(0.)
        );
        let group = h.bounds("playground-button-group");
        h.click(point(group.left() + px(16.), group.center().y));
        assert!(h.bounds("playground-button-group-status").size.width > px(0.));
    }

    #[gpui::test]
    fn vertical_orientation_stacks_the_options(cx: &mut TestAppContext) {
        let mut h = open(&PLAYGROUND, cx);
        let row = h.bounds("playground-button-group");
        h.set("orientation", Value::Choice(1));
        let column = h.bounds("playground-button-group");
        assert!(column.size.height > row.size.height * 2.);
        assert!(column.size.width < row.size.width);
    }
}
