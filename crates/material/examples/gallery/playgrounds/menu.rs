//! Menu playground: md3.io's Menu configuration (standard, vibrant, gap
//! and baseline menus, density, six placements, three menus, the button's
//! label, icons, supporting text, a cascading sub-menu, closing on
//! selection) on the real `MenuPopup` opened from a tonal button. Items
//! activate, Text actions toggle and select, and the sub-menu opens by
//! pointer or the Right key.
use super::prelude::*;
use gpui::Anchor;

const POSITIONS: &[&str] = &[
    "bottom start",
    "bottom end",
    "top start",
    "top end",
    "right start",
    "left start",
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Link context",
            values: &[
                ("menu", Set::Option("Link context")),
                ("label", Set::Text("Link options")),
                ("variant", Set::Option("baseline")),
                ("supporting", Set::Bool(false)),
            ],
        },
        Scenario {
            name: "Text actions",
            values: &[
                ("menu", Set::Option("Text actions")),
                ("label", Set::Text("Format")),
                ("variant", Set::Option("gap")),
                ("close", Set::Bool(false)),
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
                    options: &["standard", "vibrant", "gap", "baseline"],
                    default: 0,
                },
                Control::Switch {
                    id: "dense",
                    label: "Dense",
                    default: false,
                },
            ],
        },
        Section {
            title: "Layout",
            controls: &[Control::Select {
                id: "position",
                label: "Position",
                options: POSITIONS,
                default: 0,
            }],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "menu",
                    label: "Menu",
                    options: &["Default", "Link context", "Text actions"],
                    default: 0,
                },
                Control::Text {
                    id: "label",
                    label: "Button label",
                    default: "Open menu",
                },
                Control::Switch {
                    id: "icons",
                    label: "Icons",
                    default: true,
                },
                Control::Switch {
                    id: "supporting",
                    label: "Supporting text",
                    default: false,
                },
                Control::Switch {
                    id: "submenu",
                    label: "Submenu",
                    default: false,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[Control::Switch {
                id: "close",
                label: "Close on selection",
                default: true,
            }],
        },
    ],
    preview,
};

/// One menu item: label, Material Symbol and supporting text.
type Item = (&'static str, &'static str, &'static str);

/// Each menu's sections (groups in the gap variant, divided otherwise).
const MENUS: [&[&[Item]]; 3] = [
    &[
        &[
            ("Cut", "content_cut", "Remove the selection"),
            ("Copy", "content_copy", "Copy the selection"),
            ("Paste", "content_paste", "Insert the clipboard"),
        ],
        &[("Select all", "select_all", "Select the whole document")],
    ],
    &[
        &[
            ("Open in new tab", "open_in_new", "Keep this page open"),
            (
                "Open in new window",
                "open_in_browser",
                "Start a new window",
            ),
        ],
        &[
            ("Copy link address", "link", "Copy the URL"),
            ("Save link as", "download", "Download the page"),
        ],
    ],
    &[
        &[
            ("Bold", "format_bold", "Heavier strokes"),
            ("Italic", "format_italic", "Slanted letters"),
            ("Underline", "format_underlined", "A line below"),
        ],
        &[
            ("Align left", "format_align_left", "Ragged right edge"),
            ("Align center", "format_align_center", "Centered lines"),
            ("Align right", "format_align_right", "Ragged left edge"),
        ],
    ],
];

/// The cascading sub-menu's items, after a "Share" item.
const SHARE: Item = ("Share", "share", "Send to others");
const SUBMENU: &[Item] = &[
    ("Email", "mail", "Send as an attachment"),
    ("Message", "chat", "Send in a conversation"),
    ("Copy link", "link", "Paste it anywhere"),
];

/// The preview's menu state.
struct MenuState {
    open: bool,
    submenu: bool,
    /// Text actions: the toggled styles (bit per item) and the alignment.
    styles: u32,
    align: usize,
    /// The last activated item, for the caption under the button.
    last: Option<&'static str>,
}

#[cfg(test)]
thread_local! {
    /// The menu state the last preview drew: (open, submenu, styles, align, last).
    static DRAWN: std::cell::Cell<(bool, bool, u32, usize, Option<&'static str>)> =
        const { std::cell::Cell::new((false, false, 0, 0, None)) };
}

fn anchor(position: &str) -> Anchor {
    match position {
        "bottom end" => Anchor::TopRight,
        "top start" => Anchor::BottomLeft,
        "top end" => Anchor::BottomRight,
        "right start" => Anchor::LeftCenter,
        "left start" => Anchor::RightCenter,
        _ => Anchor::TopLeft,
    }
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let state = preview.local("menu-state", window, cx, |_, _| MenuState {
        open: false,
        submenu: false,
        styles: 0b001,
        align: 0,
        last: None,
    });
    let (open, submenu_open, styles, align, last) = {
        let s = state.read(cx);
        (s.open, s.submenu, s.styles, s.align, s.last)
    };
    #[cfg(test)]
    DRAWN.set((open, submenu_open, styles, align, last));
    let variant = config.option("variant");
    let style = match variant {
        "vibrant" => MenuStyle::Vibrant,
        "baseline" => MenuStyle::Baseline,
        _ => MenuStyle::Standard,
    };
    let density = if config.bool("dense") {
        MenuDensity::Dense
    } else {
        MenuDensity::Default
    };
    let icons = config.bool("icons");
    let supporting = config.bool("supporting");
    let with_submenu = config.bool("submenu");
    let which = config.choice("menu");
    let text_actions = which == 2;
    let width = if supporting { 240. } else { 208. };
    // The kit's icons: 24dp baseline, 20dp Expressive.
    let icon_size = if style == MenuStyle::Baseline {
        24.
    } else {
        20.
    };

    // One item; `index` is its flat position (sections in order).
    let item = |(label, icon, support): Item, index: usize, section: usize| {
        let mut item = MaterialMenuItem::new(label);
        if icons {
            item = item.leading(symbol(icon, icon_size));
        }
        if supporting {
            item = item.supporting_text(support);
        }
        if text_actions {
            // Styles toggle; the alignment is a single choice.
            let selected = match section {
                0 => styles & (1 << index) != 0,
                1 => align + 3 == index,
                _ => false,
            };
            item = item.selected(selected);
        }
        let state = state.clone();
        item.on_click(move |_, _, cx| {
            state.update(cx, |state, cx| {
                state.last = Some(label);
                if text_actions {
                    match section {
                        0 => state.styles ^= 1 << index,
                        1 => state.align = index - 3,
                        _ => {}
                    }
                }
                cx.notify();
            })
        })
    };
    let sections = MENUS[which];
    let mut menu = MaterialMenu::new("playground-menu")
        .style(style)
        .density(density)
        .width(width);
    let mut index = 0;
    let section_count = sections.len();
    for (section, items) in sections.iter().enumerate() {
        let last_section = section + 1 == section_count;
        let mut built: Vec<MaterialMenuItem> = items
            .iter()
            .map(|entry| {
                let built = item(*entry, index, section);
                index += 1;
                built
            })
            .collect();
        if last_section && with_submenu {
            let state = state.clone();
            let mut share = MaterialMenuItem::new(SHARE.0)
                .submenu(true)
                .selected(submenu_open);
            if icons {
                share = share.leading(symbol(SHARE.1, icon_size));
            }
            if supporting {
                share = share.supporting_text(SHARE.2);
            }
            built.push(share.on_click(move |_, _, cx| {
                state.update(cx, |state, cx| {
                    state.submenu = !state.submenu;
                    cx.notify();
                })
            }));
        }
        if variant == "gap" {
            menu = menu.group(MenuGroup::new().items(built));
        } else {
            if !last_section && let Some(last) = built.pop() {
                built.push(last.divider(true));
            }
            menu = menu.items(built);
        }
    }
    let share_index = index;

    let label = config.text("label");
    let trigger = MaterialButton::new("menu-trigger")
        .debug_selector(|| "menu-trigger".into())
        .variant(ButtonVariant::Tonal)
        .accessibility_label(label.clone())
        .child(label)
        .on_click({
            let state = state.clone();
            move |event, _, cx| {
                // Pointer presses reach the popover; keys arrive here.
                if matches!(event, ClickEvent::Keyboard(_)) {
                    state.update(cx, |state, cx| {
                        state.open = !state.open;
                        cx.notify();
                    })
                }
            }
        });
    let mut popup = MenuPopup::new("playground-menu-popup", trigger, menu)
        .open(open)
        .anchor(anchor(config.option("position")))
        .close_on_activate(config.bool("close"))
        .on_open_change({
            let state = state.clone();
            move |open, _, cx| {
                state.update(cx, |state, cx| {
                    state.open = *open;
                    if !*open {
                        state.submenu = false;
                    }
                    cx.notify();
                })
            }
        });
    if with_submenu {
        let submenu = MaterialMenu::new("playground-submenu")
            .style(style)
            .density(density)
            .width(width)
            .items(
                SUBMENU
                    .iter()
                    .enumerate()
                    .map(|(offset, entry)| item(*entry, share_index + 1 + offset, usize::MAX)),
            );
        let state = state.clone();
        popup = popup
            .submenu(share_index, submenu, submenu_open)
            .on_submenu_change(move |open, _, cx| {
                state.update(cx, |state, cx| {
                    state.submenu = *open;
                    cx.notify();
                })
            });
    }
    div()
        .debug_selector(|| "playground-menu".into())
        .flex()
        .flex_col()
        .items_center()
        .gap(rem(12.))
        .child(popup)
        .child(
            accessible_text("menu-last", last.unwrap_or(""))
                .map(|text| type_role(text, TypeRole::BodyMedium))
                .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::harness::Harness;
    use super::*;
    use core::prelude::v1::test;
    use gpui::TestAppContext;

    fn drawn() -> (bool, bool, u32, usize, Option<&'static str>) {
        DRAWN.get()
    }

    #[gpui::test]
    fn every_scenario_renders_the_trigger(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.render_every_scenario(|harness, scenario| {
            assert!(
                harness.bounds("menu-trigger").is_some(),
                "scenario {scenario}"
            );
        });
    }

    #[gpui::test]
    fn the_button_opens_the_menu_and_an_item_closes_it(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.cx.update(|_, cx| cx.set_reduce_motion(true));
        harness.click("menu-trigger");
        harness.advance(1000);
        assert!(drawn().0, "the button opens the menu");
        assert!(harness.bounds("menu-popup-surface").is_some());
        // Down focuses Cut; Down again Copy; Enter activates it and closes.
        harness.keys("down down enter");
        harness.advance(1000);
        let (open, _, _, _, last) = drawn();
        assert!(!open);
        assert_eq!(last, Some("Copy"));
    }

    #[gpui::test]
    fn text_actions_toggle_and_the_submenu_cascades(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.cx.update(|_, cx| cx.set_reduce_motion(true));
        harness
            .window
            .update(&mut harness.cx, |state, window, cx| {
                state.select_scenario(2, window, cx)
            })
            .unwrap();
        harness.set("submenu", Value::Bool(true));
        harness.click("menu-trigger");
        harness.advance(1000);
        // Italic toggles on and the menu stays open (Close on selection off).
        harness.keys("down down enter");
        assert_eq!(drawn().2, 0b011);
        assert!(drawn().0);
        // Past Underline and Align left, Align center selects the second
        // alignment.
        harness.keys("down down down enter");
        assert_eq!(drawn().3, 1);
        // Share (the last item) opens its sub-menu with Right.
        harness.keys("end right");
        harness.advance(1000);
        assert!(drawn().1, "the sub-menu opens");
        assert!(harness.bounds("menu-popup-submenu").is_some());
        harness.keys("down enter");
        assert_eq!(drawn().4, Some("Email"));
        harness.keys("left");
        assert!(!drawn().1, "Left closes the sub-menu");
        assert!(drawn().0, "the menu stays open");
    }
}
