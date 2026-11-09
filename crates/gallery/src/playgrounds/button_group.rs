//! Button group playground: md3.io's Button group configuration (kind,
//! variant, size, square shape, orientation, density, equal widths, width, items,
//! content, labels, selection, required selection, disabled) on the real
//! `ButtonGroup` (standard: toggle buttons that widen when pressed) and
//! `ConnectedButtonGroup` (connected shapes, or segments that fill the preview
//! or, as a toolbar's view switch, fit their options). Clicking options selects them
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
                Control::Select {
                    id: "tone",
                    label: "Tone",
                    options: &["standard", "neutral", "accent"],
                    default: 0,
                },
                Control::Segmented {
                    id: "kind",
                    label: "Kind",
                    options: &["standard", "connected", "segmented"],
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
                // A connected group fits its options unless it fills; a segmented
                // one fills unless it fits.
                Control::Segmented {
                    id: "width",
                    label: "Width",
                    options: &["content", "fill"],
                    default: 0,
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
    code,
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
    let tone = ButtonTone::ALL[config.choice("tone")];
    let id = ElementId::from(("playground-button-group", items_index));
    let segmented = config.option("kind") == "segmented";
    let fill = config.option("width") == "fill" && config.option("kind") != "standard";
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
                            .tone(tone)
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
            .tone(tone)
            .button_size(size)
            .shape(shape)
            .vertical(vertical)
            .density(density)
            .equal_widths(equal)
            .disabled(disabled)
            .options(items.options.iter().map(|&(label, _)| (label, label)))
            .map(|group| match (segmented, fill) {
                (true, true) => group.segmented(),
                (true, false) => group.segmented().fit_content(),
                (false, true) => group.full_width(),
                (false, false) => group,
            })
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
        .when(fill, |preview| preview.w_full())
        .gap(rem(16.))
        .child(
            div()
                .flex()
                .when(fill, |frame| frame.w_full())
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

/// The Rust names of the Variant, Tone, Size and Density options.
const VARIANT_NAMES: [&str; 5] = ["Filled", "Tonal", "Outlined", "Elevated", "Text"];
const TONE_NAMES: [&str; 3] = ["Standard", "Neutral", "Accent"];
const SIZE_NAMES: [&str; 5] = ["ExtraSmall", "Small", "Medium", "Large", "ExtraLarge"];
const DENSITY_NAMES: [&str; 3] = ["Default", "Comfortable", "Compact"];

fn code(config: &Config) -> String {
    let items = &ITEMS[config.choice("items")];
    let selection = config.option("selection");
    let required = config.bool("required");
    let content = config.option("content");
    let (icons, text) = (content != "text", content != "icons");
    let labels_selected = config.option("labels") == "selected" && icons;
    let (variant, tone, size) = (
        config.choice("variant"),
        config.choice("tone"),
        config.choice("size"),
    );
    let square = config.bool("square");
    let disabled = config.bool("disabled");
    let density = config.choice("density");
    let layout = [
        (config.option("orientation") == "vertical").then(|| "vertical(true)".into()),
        (density != 0).then(|| format!("density(ButtonDensity::{})", DENSITY_NAMES[density])),
        config.bool("equal").then(|| "equal_widths(true)".into()),
        match (config.option("kind"), config.option("width")) {
            ("segmented", "fill") => Some("segmented()".into()),
            ("segmented", _) => Some("segmented()\n.fit_content()".into()),
            ("connected", "fill") => Some("full_width()".into()),
            _ => None,
        },
    ];
    let initial = |count: usize| {
        let values: Vec<String> = items.selected[..count.min(items.selected.len())]
            .iter()
            .map(|value| lit(value))
            .collect();
        values.join(", ")
    };
    if config.option("kind") == "standard" {
        let toggle_body = match (selection, required) {
            ("single", true) => "if on {\n    this.selected = vec![label];\n}",
            ("single", false) => {
                "if on {\n    this.selected = vec![label];\n} else {\n    this.selected.clear();\n}"
            }
            ("multi", true) => {
                "if on {\n    this.selected.push(label);\n} else if this.selected.len() > 1 {\n    this.selected.retain(|v| *v != label);\n}"
            }
            _ => {
                "this.selected.retain(|v| *v != label);\nif on {\n    this.selected.push(label);\n}"
            }
        };
        let button = chain(
            "MaterialButton::new(label)",
            [
                (variant != 0)
                    .then(|| format!("variant(ButtonVariant::{})", VARIANT_NAMES[variant])),
                (tone != 0).then(|| format!("tone(ButtonTone::{})", TONE_NAMES[tone])),
                (size != 1).then(|| format!("button_size(ButtonSize::{})", SIZE_NAMES[size])),
                square.then(|| "shape(ButtonShape::Square)".into()),
                disabled.then(|| "disabled(true)".into()),
                (!text || labels_selected).then(|| "accessibility_label(label)".into()),
                icons.then(|| "icon(icon)".into()),
                (selection != "none").then(|| "toggle(true)".into()),
                (selection != "none").then(|| "selected(self.selected.contains(&label))".into()),
                Some(if selection == "none" {
                    "on_click(move |_, _, _| println!(\"{label}\"))".into()
                } else {
                    format!(
                        "on_change({})",
                        view_update("move |on, _, _, cx|", toggle_body)
                    )
                }),
                match (text, labels_selected) {
                    (false, _) => None,
                    (true, true) => Some(
                        "when(self.selected.contains(&label), |button| button.child(label))".into(),
                    ),
                    (true, false) => Some("child(label)".into()),
                },
            ],
        );
        let options: Vec<String> = items
            .options
            .iter()
            .map(|(label, icon)| {
                if icons {
                    format!("({}, {})", lit(label), symbol_code(icon))
                } else {
                    lit(label)
                }
            })
            .collect();
        let map = if icons { "|(label, icon)|" } else { "|label|" };
        return chain(
            "ButtonGroup::new(\"button-group\", \"Options\")",
            [(size != 1).then(|| format!("button_size(ButtonSize::{})", SIZE_NAMES[size]))]
                .into_iter()
                .chain(layout)
                .chain([Some(format!(
                    "items([{}].map({map} {{\n    {}\n}}))",
                    options.join(", "),
                    indent(&button)
                ))]),
        );
    }
    let options: Vec<String> = items
        .options
        .iter()
        .map(|(label, _)| format!("({}, {})", lit(label), lit(label)))
        .collect();
    let selection_calls = match (selection, required) {
        ("none", _) => vec![
            "selection(GroupSelection::None)".into(),
            "on_activate(|value, _, _| println!(\"{value}\"))".into(),
        ],
        ("single", true) => vec![
            format!("selected({})", initial(1)),
            format!(
                "on_change({})",
                listener(
                    "|this, value: &SharedString, _, cx|",
                    "this.selected = value.clone();"
                )
            ),
        ],
        _ => {
            let multi = selection == "multi";
            let count = if multi { items.selected.len() } else { 1 };
            [
                multi.then(|| "selection(GroupSelection::Multi)".to_string()),
                (!required).then(|| "required(false)".into()),
                Some(format!("selected_values([{}])", initial(count))),
                Some(format!(
                    "on_toggle({})",
                    view_update(
                        "move |value, on, _, cx|",
                        "this.selected.retain(|v| v != value);\nif on {\n    this.selected.push(value.clone());\n}"
                    )
                )),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
    };
    chain(
        "ConnectedButtonGroup::new(\"button-group\", \"Options\")",
        [
            (variant != 1).then(|| format!("variant(ButtonVariant::{})", VARIANT_NAMES[variant])),
            (tone != 0).then(|| format!("tone(ButtonTone::{})", TONE_NAMES[tone])),
            Some(format!("button_size(ButtonSize::{})", SIZE_NAMES[size])),
            square.then(|| "shape(ButtonShape::Square)".into()),
        ]
        .into_iter()
        .chain(layout)
        .chain([
            disabled.then(|| "disabled(true)".into()),
            Some(format!("options([{}])", options.join(", "))),
        ])
        .chain(items.options.iter().map(|(label, icon)| {
            icons.then(|| format!("icon({}, {})", lit(label), symbol_code(icon)))
        }))
        .chain([match (text, labels_selected) {
            (false, _) => Some("labels(GroupLabels::Never)".into()),
            (true, true) => Some("labels(GroupLabels::Selected)".into()),
            (true, false) => None,
        }])
        .chain(selection_calls.into_iter().map(Some)),
    )
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

    /// A segmented group fills the preview unless it fits its options, the
    /// width a connected group has.
    #[gpui::test]
    fn segments_fill_or_fit_their_options(cx: &mut TestAppContext) {
        let mut h = open(&PLAYGROUND, cx);
        let connected = h.bounds("playground-button-group");
        h.set("kind", Value::Choice(2));
        let fitted = h.bounds("playground-button-group");
        assert_eq!(fitted.size.width, connected.size.width);
        h.set("width", Value::Choice(1));
        let filled = h.bounds("playground-button-group");
        assert!(filled.size.width > fitted.size.width * 1.5, "{filled:?}");
        h.set("kind", Value::Choice(1));
        assert_eq!(
            h.bounds("playground-button-group").size.width,
            filled.size.width
        );
    }

    #[test]
    fn the_code_follows_the_configuration() {
        let mut config = Config::defaults(&PLAYGROUND);
        let code = super::code(&config);
        assert!(
            code.starts_with("ConnectedButtonGroup::new(\"button-group\", \"Options\")"),
            "{code}"
        );
        assert!(code.contains(".variant(ButtonVariant::Filled)"), "{code}");
        assert!(
            code.contains(".selected_values([\"Bold\", \"Italic\"])"),
            "{code}"
        );
        assert!(!code.contains(".vertical("));
        assert!(
            !code.contains("width") && !code.contains("segmented"),
            "{code}"
        );
        config.set("kind", Value::Choice(2));
        assert!(
            super::code(&config).contains(".segmented()\n    .fit_content()"),
            "{}",
            super::code(&config)
        );
        config.set("width", Value::Choice(1));
        let code = super::code(&config);
        assert!(
            code.contains(".segmented()") && !code.contains("fit_content"),
            "{code}"
        );
        config.set("kind", Value::Choice(1));
        assert!(super::code(&config).contains(".full_width()"));
        config.set("width", Value::Choice(0));
        config.set("orientation", Value::Choice(1));
        assert!(super::code(&config).contains(".vertical(true)"));
        config.set("kind", Value::Choice(0));
        let code = super::code(&config);
        assert!(
            code.starts_with("ButtonGroup::new(\"button-group\", \"Options\")"),
            "{code}"
        );
        assert!(code.contains(".toggle(true)"), "{code}");
    }
}
