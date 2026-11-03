//! Icon button playground: md3.io's Icon button configuration (variant,
//! size, square shape, width, icon, accessible label, toggle, selected,
//! disabled) on the real `IconButton`.
use super::prelude::*;
use gpui_kit_material::icons::{Icon, Symbol};

const WIDTHS: [IconButtonWidth; 3] = [
    IconButtonWidth::Narrow,
    IconButtonWidth::Default,
    IconButtonWidth::Wide,
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Favorite",
            values: &[
                ("icon", Set::Option("favorite")),
                ("label", Set::Text("Add to favorites")),
                ("toggle", Set::Bool(true)),
                ("selected", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Reservation date",
            values: &[
                ("variant", Set::Option("tonal")),
                ("size", Set::Option("M")),
                ("icon", Set::Option("calendar_month")),
                ("label", Set::Text("Choose a reservation date")),
            ],
        },
        Scenario {
            name: "Browse albums",
            values: &[
                ("variant", Set::Option("outlined")),
                ("width", Set::Option("narrow")),
                ("icon", Set::Option("chevron_right")),
                ("label", Set::Text("Browse albums")),
            ],
        },
        Scenario {
            name: "Raise hand",
            values: &[
                ("variant", Set::Option("filled")),
                ("size", Set::Option("L")),
                ("width", Set::Option("wide")),
                ("icon", Set::Option("front_hand")),
                ("label", Set::Text("Raise hand")),
                ("toggle", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Stop",
            values: &[
                ("variant", Set::Option("filled")),
                ("size", Set::Option("XL")),
                ("square", Set::Bool(true)),
                ("icon", Set::Option("stop")),
                ("label", Set::Text("Stop recording")),
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
                Control::Select {
                    id: "variant",
                    label: "Variant",
                    options: &["standard", "filled", "tonal", "outlined"],
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
                Control::Segmented {
                    id: "width",
                    label: "Width",
                    options: &["narrow", "default", "wide"],
                    default: 1,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Icon {
                    id: "icon",
                    label: "Icon",
                    icons: &[
                        "favorite",
                        "bookmark",
                        "download",
                        "send",
                        "add",
                        "edit",
                        "calendar_month",
                        "chevron_right",
                        "front_hand",
                        "stop",
                    ],
                    default: 0,
                },
                Control::Text {
                    id: "label",
                    label: "Accessible label",
                    default: "Add to favorites",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "toggle",
                    label: "Toggle button",
                    default: false,
                },
                Control::Switch {
                    id: "selected",
                    label: "Selected",
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
    code: Some(code),
};

/// The filled Material Symbol a selected toggle shows.
fn filled(name: &str) -> Icon {
    Icon::from(
        Symbol::named(name)
            .unwrap_or_else(|| panic!("{name} is not a Material Symbol"))
            .icon()
            .filled(true),
    )
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    _: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let icon = config.option("icon");
    let label = config.text("label");
    let label = if label.is_empty() {
        SharedString::new_static(icon)
    } else {
        label
    };
    let mut button = IconButton::new("playground-icon-button", symbol(icon, 24.), label)
        .variant(IconButtonVariant::ALL[config.choice("variant")])
        .tone(ButtonTone::ALL[config.choice("tone")])
        .button_size(ButtonSize::ALL[config.choice("size")])
        .width(WIDTHS[config.choice("width")])
        .shape(if config.bool("square") {
            ButtonShape::Square
        } else {
            ButtonShape::Round
        })
        .disabled(config.bool("disabled"));
    if config.bool("toggle") {
        // Clicking the preview flips the Selected control.
        let select = preview.setter("selected", cx);
        button = button
            .toggle(true)
            .selected(config.bool("selected"))
            .selected_icon(filled(icon))
            .on_change(move |selected, _, window, cx| select(Value::Bool(selected), window, cx));
    }
    div()
        .flex()
        .debug_selector(|| "playground-icon-button".into())
        .child(button)
        .into_any_element()
}

fn code(config: &Config) -> String {
    const VARIANT_NAMES: [&str; 4] = ["Standard", "Filled", "Tonal", "Outlined"];
    const TONE_NAMES: [&str; 3] = ["Standard", "Neutral", "Accent"];
    const SIZE_NAMES: [&str; 5] = ["ExtraSmall", "Small", "Medium", "Large", "ExtraLarge"];
    const WIDTH_NAMES: [&str; 3] = ["Narrow", "Default", "Wide"];
    let icon = config.option("icon");
    let label = config.text("label");
    let label = if label.is_empty() { icon } else { &label };
    let toggle = config.bool("toggle");
    let (variant, tone, size, width) = (
        config.choice("variant"),
        config.choice("tone"),
        config.choice("size"),
        config.choice("width"),
    );
    chain(
        format!(
            "IconButton::new(\"icon-button\", {}, {})",
            symbol_code(icon),
            lit(label)
        ),
        [
            (variant != 0)
                .then(|| format!("variant(IconButtonVariant::{})", VARIANT_NAMES[variant])),
            (tone != 0).then(|| format!("tone(ButtonTone::{})", TONE_NAMES[tone])),
            (size != 1).then(|| format!("button_size(ButtonSize::{})", SIZE_NAMES[size])),
            (width != 1).then(|| format!("width(IconButtonWidth::{})", WIDTH_NAMES[width])),
            config
                .bool("square")
                .then(|| "shape(ButtonShape::Square)".into()),
            config.bool("disabled").then(|| "disabled(true)".into()),
            toggle.then(|| "toggle(true)".into()),
            toggle.then(|| format!("selected({})", config.bool("selected"))),
            // A selected toggle shows the filled symbol.
            toggle.then(|| format!("selected_icon({}.icon().filled(true))", symbol_code(icon))),
            Some(if toggle {
                format!(
                    "on_change({})",
                    view_update("move |selected, _, _, cx|", "this.selected = selected;")
                )
            } else {
                format!("on_click(|_, _, _| println!({}))", lit(label))
            }),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::super::harness::open;
    use super::*;
    use core::prelude::v1::test;

    #[gpui::test]
    fn every_scenario_renders(cx: &mut TestAppContext) {
        open(&PLAYGROUND, cx).renders_every_scenario("playground-icon-button");
    }

    #[gpui::test]
    fn width_and_size_controls_resize_the_button(cx: &mut TestAppContext) {
        let mut h = open(&PLAYGROUND, cx);
        let initial = h.bounds("playground-icon-button").size;
        assert_eq!(initial, size(px(40.), px(40.)), "S, default width");
        h.set("width", Value::Choice(2));
        assert_eq!(
            h.bounds("playground-icon-button").size,
            size(px(52.), px(40.))
        );
        h.set("size", Value::Choice(4));
        assert_eq!(
            h.bounds("playground-icon-button").size,
            size(px(184.), px(136.))
        );
    }

    #[gpui::test]
    fn clicking_a_toggle_preview_flips_selected(cx: &mut TestAppContext) {
        let mut h = open(&PLAYGROUND, cx);
        h.click_on("playground-icon-button");
        assert!(!h.config().bool("selected"), "a plain button only acts");
        h.set("toggle", Value::Bool(true));
        h.click_on("playground-icon-button");
        assert!(h.config().bool("selected"));
        h.click_on("playground-icon-button");
        assert!(!h.config().bool("selected"));
    }

    #[test]
    fn the_code_follows_the_configuration() {
        let mut config = Config::defaults(&PLAYGROUND);
        let code = super::code(&config);
        assert!(
            code.starts_with(
                "IconButton::new(\"icon-button\", Symbol::FAVORITE, \"Add to favorites\")"
            ),
            "{code}"
        );
        assert!(!code.contains(".toggle("));
        config.set("toggle", Value::Bool(true));
        let code = super::code(&config);
        assert!(code.contains(".toggle(true)"));
        assert!(code.contains(".selected_icon(Symbol::FAVORITE.icon().filled(true))"));
    }
}
