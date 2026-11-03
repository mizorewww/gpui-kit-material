//! FAB menu playground: md3.io's FAB menu configuration (presentation,
//! color, size, placement, items, item count, item icons, open) on the real
//! `FabMenu`. The FAB opens and closes the menu with its animation (the
//! Open control follows), and choosing an item closes it and reports the
//! action.
use super::fab::{position as fab_position, stage};
use super::prelude::*;

/// One item preset: the FAB's icon and accessible name, and up to six (label, icon) items.
struct Items {
    fab: &'static str,
    name: &'static str,
    items: [(&'static str, &'static str); 6],
}

const ITEMS: [Items; 4] = [
    Items {
        fab: "edit",
        name: "Message actions",
        items: [
            ("Reply", "reply"),
            ("Forward", "forward"),
            ("Archive", "archive"),
            ("Snooze", "snooze"),
            ("Label", "label"),
            ("Delete", "delete"),
        ],
    },
    Items {
        fab: "add",
        name: "New music",
        items: [
            ("Album", "album"),
            ("Playlist", "queue_music"),
            ("Station", "radio"),
            ("Podcast", "podcasts"),
            ("Mix", "shuffle"),
            ("Upload", "upload"),
        ],
    },
    Items {
        fab: "photo_library",
        name: "Photo categories",
        items: [
            ("People", "person"),
            ("Places", "place"),
            ("Things", "category"),
            ("Videos", "videocam"),
            ("Favorites", "favorite"),
            ("Screenshots", "screenshot"),
        ],
    },
    Items {
        fab: "share",
        name: "Share",
        items: [
            ("Message", "chat"),
            ("Email", "mail"),
            ("Copy link", "link"),
            ("Nearby", "nearby"),
            ("Print", "print"),
            ("More", "more_horiz"),
        ],
    },
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "New music",
            values: &[
                ("items", Set::Option("New music")),
                ("count", Set::Option("4")),
                ("placement", Set::Option("bottom end")),
            ],
        },
        Scenario {
            name: "Photo categories",
            values: &[
                ("items", Set::Option("Photo categories")),
                ("color", Set::Option("tertiary")),
                ("size", Set::Option("medium")),
                ("count", Set::Option("5")),
                ("open", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Share",
            values: &[
                ("items", Set::Option("Share")),
                ("presentation", Set::Option("menu")),
                ("color", Set::Option("secondary")),
                ("placement", Set::Option("bottom start")),
                ("count", Set::Option("4")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Segmented {
                    id: "presentation",
                    label: "Presentation",
                    options: &["list", "menu", "auto"],
                    default: 0,
                },
                Control::Segmented {
                    id: "color",
                    label: "Color",
                    options: &["primary", "secondary", "tertiary"],
                    default: 0,
                },
                Control::Select {
                    id: "size",
                    label: "Size",
                    options: &["default", "medium", "large"],
                    default: 0,
                },
                Control::Select {
                    id: "placement",
                    label: "Placement",
                    options: &["none", "bottom end", "bottom start"],
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
                    options: &["Default", "New music", "Photo categories", "Share"],
                    default: 0,
                },
                Control::Segmented {
                    id: "count",
                    label: "Item count",
                    options: &["2", "3", "4", "5", "6"],
                    default: 1,
                },
                Control::Switch {
                    id: "icons",
                    label: "Item icons",
                    default: true,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[Control::Switch {
                id: "open",
                label: "Open",
                default: false,
            }],
        },
    ],
    preview,
    code,
};

/// The last chosen item, reported under the menu.
#[derive(Default)]
struct Demo {
    status: Option<SharedString>,
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let demo = preview.local("fab-menu-demo", window, cx, |_, _| Demo::default());
    let status = demo.read(cx).status.clone();
    let preset = &ITEMS[config.choice("items")];
    let count = config.choice("count") + 2;
    let icons = config.bool("icons");
    let set_open = std::rc::Rc::new(preview.on_bool("open", cx));
    let items = preset.items[..count]
        .iter()
        .enumerate()
        .map(|(index, &(label, icon))| {
            let id = ("playground-fab-menu-item", index);
            let item = if icons {
                FabMenuItem::new(id, symbol(icon, 24.), label)
            } else {
                FabMenuItem::label_only(id, label)
            };
            let demo = demo.clone();
            let set_open = set_open.clone();
            item.on_click(move |_, window, cx| {
                demo.update(cx, |demo, cx| {
                    demo.status = Some(format!("Chose “{label}”").into());
                    cx.notify();
                });
                set_open(&false, window, cx);
            })
        });
    let placement = config.option("placement");
    let toggle = set_open.clone();
    let menu = FabMenu::new("playground-fab-menu", symbol(preset.fab, 24.), preset.name)
        .presentation(FabMenuPresentation::ALL[config.choice("presentation")])
        .color(FabMenuColor::ALL[config.choice("color")])
        .size([FabSize::Regular, FabSize::Medium, FabSize::Large][config.choice("size")])
        .align(if placement == "bottom start" {
            FabMenuAlign::Start
        } else {
            FabMenuAlign::End
        })
        .open(config.bool("open"))
        .on_toggle(move |open, window, cx| toggle(open, window, cx))
        .items(items);
    let position = match placement {
        "bottom end" => "bottom right",
        "bottom start" => "bottom left",
        _ => "center",
    };
    div()
        .w_full()
        .flex_1()
        .flex()
        .flex_col()
        .child(
            div()
                .h(rem(20.))
                .debug_selector(|| "playground-fab-menu-status".into())
                .map(|line| type_role(line, TypeRole::BodyMedium))
                .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                .children(status),
        )
        .child(stage(
            fab_position(position).place(
                div()
                    .flex()
                    .debug_selector(|| "playground-fab-menu".into())
                    .child(menu),
            ),
        ))
        .into_any_element()
}

fn code(config: &Config) -> String {
    const PRESENTATION_NAMES: [&str; 3] = ["List", "Menu", "Auto"];
    const COLOR_NAMES: [&str; 3] = ["Primary", "Secondary", "Tertiary"];
    let preset = &ITEMS[config.choice("items")];
    let count = config.choice("count") + 2;
    let icons = config.bool("icons");
    let (presentation, color, size) = (
        config.choice("presentation"),
        config.choice("color"),
        config.choice("size"),
    );
    let placement = config.option("placement");
    let close = listener("|this, _: &ClickEvent, _, cx|", "this.open = false;");
    let items = preset.items[..count].iter().map(|&(label, icon)| {
        let id = label.to_lowercase().replace(' ', "-");
        let item = if icons {
            format!(
                "FabMenuItem::new({}, {}, {})",
                lit(&id),
                symbol_code(icon),
                lit(label)
            )
        } else {
            format!("FabMenuItem::label_only({}, {})", lit(&id), lit(label))
        };
        // Choosing an item closes the menu.
        let item = chain(item, [Some(format!("on_click({close})"))]);
        Some(call("item", &item))
    });
    let menu = chain(
        format!(
            "FabMenu::new(\"fab-menu\", {}, {})",
            symbol_code(preset.fab),
            lit(preset.name)
        ),
        [
            (presentation != 0).then(|| {
                format!(
                    "presentation(FabMenuPresentation::{})",
                    PRESENTATION_NAMES[presentation]
                )
            }),
            (color != 0).then(|| format!("color(FabMenuColor::{})", COLOR_NAMES[color])),
            (size != 0).then(|| format!("size(FabSize::{})", super::fab::SIZE_NAMES[size])),
            (placement == "bottom start").then(|| "align(FabMenuAlign::Start)".into()),
            Some(format!("open({})", config.bool("open"))),
            Some(format!(
                "on_toggle({})",
                listener("|this, open: &bool, _, cx|", "this.open = *open;")
            )),
        ]
        .into_iter()
        .chain(items),
    );
    match placement {
        "bottom end" => format!("FabPosition::BottomEnd.place(\n    {}\n)", indent(&menu)),
        "bottom start" => format!("FabPosition::BottomStart.place(\n    {}\n)", indent(&menu)),
        _ => menu,
    }
}

#[cfg(test)]
mod tests {
    use super::super::harness::open;
    use super::*;
    use core::prelude::v1::test;

    #[gpui::test]
    fn every_scenario_renders(cx: &mut TestAppContext) {
        open(&PLAYGROUND, cx).renders_every_scenario("playground-fab-menu");
    }

    #[gpui::test]
    fn the_fab_opens_the_list_and_an_item_closes_it(cx: &mut TestAppContext) {
        let mut h = open(&PLAYGROUND, cx);
        let closed = h.bounds("playground-fab-menu");
        assert_eq!(closed.size, size(px(56.), px(56.)));
        h.click(closed.center());
        assert!(h.config().bool("open"), "the FAB opens the menu");
        let menu = h.bounds("playground-fab-menu");
        assert_eq!(menu.size.height, px(56. + 8. + 56. * 3. + 4. * 2.));
        // The first (top) item, at the end edge.
        h.click(point(menu.right() - px(28.), menu.top() + px(28.)));
        assert!(!h.config().bool("open"), "an item closes the menu");
        assert!(h.bounds("playground-fab-menu-status").size.width > px(0.));
        assert_eq!(h.bounds("playground-fab-menu").size, closed.size);
    }

    #[gpui::test]
    fn count_size_and_placement_controls_change_the_menu(cx: &mut TestAppContext) {
        let mut h = open(&PLAYGROUND, cx);
        h.set("count", Value::Choice(4));
        h.set("open", Value::Bool(true));
        let menu = h.bounds("playground-fab-menu");
        assert_eq!(menu.size.height, px(56. + 8. + 56. * 6. + 4. * 5.));
        h.set("open", Value::Bool(false));
        h.set("size", Value::Choice(2));
        h.set("placement", Value::Choice(2));
        let stage = h.bounds("playground-stage");
        let fab = h.bounds("playground-fab-menu");
        assert_eq!(fab.size, size(px(96.), px(96.)));
        assert_eq!(fab.left(), stage.left() + px(16.));
        assert_eq!(fab.bottom(), stage.bottom() - px(16.));
    }

    #[test]
    fn the_code_follows_the_configuration() {
        let mut config = Config::defaults(&PLAYGROUND);
        let code = super::code(&config);
        assert!(
            code.starts_with("FabMenu::new(\"fab-menu\", Symbol::EDIT, \"Message actions\")"),
            "{code}"
        );
        assert!(code.contains("FabMenuItem::new(\"archive\", Symbol::ARCHIVE, \"Archive\")"));
        assert!(!code.contains("\"Snooze\""));
        config.set("count", Value::Choice(2));
        assert!(super::code(&config).contains("\"Snooze\""));
        config.set("placement", Value::Choice(2));
        let code = super::code(&config);
        assert!(
            code.starts_with("FabPosition::BottomStart.place("),
            "{code}"
        );
        assert!(code.contains(".align(FabMenuAlign::Start)"));
    }
}
