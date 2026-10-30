//! Split button playground: md3.io's Split button configuration (variant,
//! size, icon, text, menu label, menu options, disabled) on the real
//! `SplitButton`, whose trailing segment opens a working menu. Choosing a
//! playback speed or mug color makes it the button's text; the other menus
//! report the chosen action below the button.
use super::prelude::*;

const VARIANTS: [ButtonVariant; 4] = [
    ButtonVariant::Filled,
    ButtonVariant::Tonal,
    ButtonVariant::Outlined,
    ButtonVariant::Elevated,
];

/// One menu preset: its items (label, leading icon) and whether choosing
/// an item selects it as the button's text.
struct MenuPreset {
    items: &'static [(&'static str, &'static str)],
    selects: bool,
}

const MENUS: [MenuPreset; 5] = [
    MenuPreset {
        items: &[
            ("Save as…", "save_as"),
            ("Save a copy", "file_copy"),
            ("Export as PDF", "picture_as_pdf"),
        ],
        selects: false,
    },
    MenuPreset {
        items: &[
            ("Copy link", "link"),
            ("Email", "mail"),
            ("Message", "chat"),
        ],
        selects: false,
    },
    MenuPreset {
        items: &[("0.5×", ""), ("1×", ""), ("1.5×", ""), ("2×", "")],
        selects: true,
    },
    MenuPreset {
        items: &[
            ("From beginning", "first_page"),
            ("From current slide", "slideshow"),
            ("Presenter view", "co_present"),
        ],
        selects: false,
    },
    MenuPreset {
        items: &[("Cobalt", ""), ("Sage", ""), ("Coral", ""), ("Sand", "")],
        selects: true,
    },
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Playback speed",
            values: &[
                ("variant", Set::Option("tonal")),
                ("icon", Set::Option("play_circle")),
                ("text", Set::Text("1×")),
                ("menu_label", Set::Text("Playback speed")),
                ("menu", Set::Option("Playback speed")),
            ],
        },
        Scenario {
            name: "Slideshow",
            values: &[
                ("size", Set::Option("M")),
                ("icon", Set::Option("play_circle")),
                ("text", Set::Text("Present")),
                ("menu_label", Set::Text("Presentation options")),
                ("menu", Set::Option("Slideshow")),
            ],
        },
        Scenario {
            name: "Enamel mugs",
            values: &[
                ("variant", Set::Option("outlined")),
                ("icon", Set::Option("add")),
                ("text", Set::Text("Cobalt")),
                ("menu_label", Set::Text("Mug color")),
                ("menu", Set::Option("Mug colors")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Select {
                    id: "variant",
                    label: "Variant",
                    options: &["filled", "tonal", "outlined", "elevated"],
                    default: 0,
                },
                Control::Segmented {
                    id: "size",
                    label: "Size",
                    options: &["XS", "S", "M", "L", "XL"],
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
                        "",
                        "download",
                        "bookmark",
                        "send",
                        "favorite",
                        "play_circle",
                        "add",
                    ],
                    default: 0,
                },
                Control::Text {
                    id: "text",
                    label: "Text",
                    default: "Save",
                },
                Control::Text {
                    id: "menu_label",
                    label: "Menu label",
                    default: "More save options",
                },
                Control::Select {
                    id: "menu",
                    label: "Menu options",
                    options: &["Save", "Share", "Playback speed", "Slideshow", "Mug colors"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[Control::Switch {
                id: "disabled",
                label: "Disabled",
                default: false,
            }],
        },
    ],
    preview,
};

/// The preview's own state: whether the menu is open and the last action.
#[derive(Default)]
struct Demo {
    open: bool,
    status: Option<SharedString>,
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let demo = preview.local("split-demo", window, cx, |_, _| Demo::default());
    let state = demo.read(cx);
    let (open, status) = (state.open, state.status.clone());
    let text = config.text("text");
    let preset = &MENUS[config.choice("menu")];
    let set_text = std::rc::Rc::new(preview.setter("text", cx));
    let menu = MaterialMenu::new("playground-split-menu")
        .width(200.)
        .items(preset.items.iter().map(|&(label, icon)| {
            let demo = demo.clone();
            let set_text = set_text.clone();
            let selects = preset.selects;
            let mut item = MaterialMenuItem::new(label).on_click(move |_, window, cx| {
                if selects {
                    set_text(Value::Text(label.into()), window, cx);
                }
                demo.update(cx, |demo, cx| {
                    demo.status = Some(format!("Chose “{label}”").into());
                    cx.notify();
                });
            });
            if preset.selects {
                item = item
                    .selected(text.as_ref() == label)
                    .trailing(symbol("check", 20.));
            } else if !icon.is_empty() {
                item = item.leading(symbol(icon, 20.));
            }
            item
        }));
    let icon = config.option("icon");
    let toggle = demo.clone();
    let act = demo.clone();
    let mut split = SplitButton::new("playground-split", text.clone(), config.text("menu_label"))
        .variant(VARIANTS[config.choice("variant")])
        .button_size(ButtonSize::ALL[config.choice("size")])
        .disabled(config.bool("disabled"))
        .menu(menu)
        .open(open)
        .on_toggle(move |open, _, cx| {
            let open = *open;
            toggle.update(cx, |demo, cx| {
                demo.open = open;
                cx.notify();
            });
        })
        .on_click(move |_, _, cx| {
            let text = text.clone();
            act.update(cx, |demo, cx| {
                demo.status = Some(format!("Activated “{text}”").into());
                cx.notify();
            });
        });
    if !icon.is_empty() {
        split = split.icon(symbol(icon, 20.));
    }
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(rem(16.))
        .child(
            div()
                .flex()
                .debug_selector(|| "playground-split-button".into())
                .child(split),
        )
        .child(
            div()
                .h(rem(20.))
                .debug_selector(|| "playground-split-status".into())
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

    #[gpui::test]
    fn every_scenario_renders(cx: &mut TestAppContext) {
        open(&PLAYGROUND, cx).renders_every_scenario("playground-split-button");
    }

    #[gpui::test]
    fn the_menu_opens_and_a_chosen_speed_becomes_the_text(cx: &mut TestAppContext) {
        let mut h = open(&PLAYGROUND, cx);
        h.scenario(1);
        assert!(h.cx.debug_bounds("material-menu").is_none());
        let split = h.bounds("playground-split-button");
        h.click(point(split.right() - px(20.), split.center().y));
        assert!(
            h.cx.debug_bounds("material-menu").is_some(),
            "the menu opens"
        );
        // Down focuses "0.5×"; Enter chooses it and closes the menu.
        h.cx.simulate_keystrokes("down enter");
        h.settle();
        assert_eq!(h.config().text("text"), "0.5×");
        assert!(h.cx.debug_bounds("material-menu").is_none(), "and closes");
    }

    #[gpui::test]
    fn the_leading_action_reports_itself(cx: &mut TestAppContext) {
        let mut h = open(&PLAYGROUND, cx);
        assert_eq!(h.bounds("playground-split-status").size.width, px(0.));
        let split = h.bounds("playground-split-button");
        h.click(point(split.left() + px(16.), split.center().y));
        assert!(h.cx.debug_bounds("material-menu").is_none(), "no menu");
        assert!(
            h.bounds("playground-split-status").size.width > px(0.),
            "the action is reported"
        );
    }
}
