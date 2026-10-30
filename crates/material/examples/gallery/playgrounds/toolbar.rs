//! Toolbar playground: md3.io's Toolbar configuration (docked or floating,
//! standard or vibrant, elevation, orientation, docked item placement, six
//! action sets, item count and formatting toggles) on the real `Toolbar`
//! inside a preview screen: a floating toolbar centered over it, a docked
//! one along its bottom edge.
use super::prelude::*;
use gpui_kit_material::icons::Symbol;
use std::sync::OnceLock;

const ACTION_SETS: &[&str] = &[
    "Default",
    "Video call",
    "Text formatting",
    "Browser",
    "Document editing",
    "Step navigation",
    "Email actions",
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Video call",
            values: &[
                ("actions", Set::Option("Video call")),
                ("color", Set::Option("vibrant")),
                ("count", Set::Option("5")),
            ],
        },
        Scenario {
            name: "Text formatting",
            values: &[
                ("actions", Set::Option("Text formatting")),
                ("count", Set::Option("5")),
            ],
        },
        Scenario {
            name: "Browser",
            values: &[
                ("actions", Set::Option("Browser")),
                ("variant", Set::Option("docked")),
                ("elevated", Set::Bool(false)),
                ("count", Set::Option("5")),
            ],
        },
        Scenario {
            name: "Document editing",
            values: &[
                ("actions", Set::Option("Document editing")),
                ("orientation", Set::Option("vertical")),
            ],
        },
        Scenario {
            name: "Step navigation",
            values: &[
                ("actions", Set::Option("Step navigation")),
                ("variant", Set::Option("docked")),
                ("elevated", Set::Bool(false)),
                ("count", Set::Option("3")),
            ],
        },
        Scenario {
            name: "Email actions",
            values: &[
                ("actions", Set::Option("Email actions")),
                ("color", Set::Option("vibrant")),
                ("variant", Set::Option("docked")),
                ("docked_items", Set::Option("center")),
                ("elevated", Set::Bool(false)),
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
                    options: &["docked", "floating"],
                    default: 1,
                },
                Control::Segmented {
                    id: "color",
                    label: "Color",
                    options: &["standard", "vibrant"],
                    default: 0,
                },
                Control::Switch {
                    id: "elevated",
                    label: "Elevated",
                    default: true,
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
                Control::Segmented {
                    id: "docked_items",
                    label: "Docked items",
                    options: &["spread", "center"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "actions",
                    label: "Actions",
                    options: ACTION_SETS,
                    default: 0,
                },
                Control::Segmented {
                    id: "count",
                    label: "Item count",
                    options: &["2", "3", "4", "5"],
                    default: 2,
                },
                Control::Switch {
                    id: "toggles",
                    label: "Formatting toggles",
                    default: true,
                },
            ],
        },
    ],
    preview,
};

/// One toolbar item: a Material Symbol (or a label when `icon` is empty),
/// its accessible label, and whether it is a formatting toggle.
#[derive(Clone, Copy)]
struct Action {
    icon: &'static str,
    label: &'static str,
    toggle: bool,
    danger: bool,
}

const fn icon(icon: &'static str, label: &'static str) -> Action {
    Action {
        icon,
        label,
        toggle: false,
        danger: false,
    }
}
const fn toggle(icon: &'static str, label: &'static str) -> Action {
    Action {
        icon,
        label,
        toggle: true,
        danger: false,
    }
}
const fn text(label: &'static str) -> Action {
    Action {
        icon: "",
        label,
        toggle: false,
        danger: false,
    }
}

/// Each action set's five items; the item count shows the first N.
const ACTIONS: [[Action; 5]; 7] = [
    [
        toggle("format_bold", "Bold"),
        toggle("format_italic", "Italic"),
        toggle("format_underlined", "Underline"),
        icon("edit", "Edit"),
        icon("more_vert", "More options"),
    ],
    [
        toggle("mic", "Microphone"),
        toggle("videocam", "Camera"),
        icon("screen_share", "Share screen"),
        icon("back_hand", "Raise hand"),
        Action {
            icon: "call_end",
            label: "Leave call",
            toggle: false,
            danger: true,
        },
    ],
    [
        toggle("format_bold", "Bold"),
        toggle("format_italic", "Italic"),
        toggle("format_underlined", "Underline"),
        toggle("strikethrough_s", "Strikethrough"),
        icon("format_color_text", "Text color"),
    ],
    [
        icon("arrow_back", "Back"),
        icon("arrow_forward", "Forward"),
        icon("refresh", "Reload"),
        icon("home", "Home"),
        icon("tab", "Tabs"),
    ],
    [
        icon("undo", "Undo"),
        icon("redo", "Redo"),
        toggle("format_list_bulleted", "Bulleted list"),
        icon("image", "Insert image"),
        icon("link", "Insert link"),
    ],
    [
        text("Back"),
        text("Skip"),
        text("Next"),
        text("Save draft"),
        text("Finish"),
    ],
    [
        icon("archive", "Archive"),
        icon("delete", "Delete"),
        icon("mark_email_unread", "Mark as unread"),
        icon("schedule", "Snooze"),
        icon("more_vert", "More options"),
    ],
];

/// Every action's icon, resolved once (resolving symbol paths allocates).
fn nav_icon(name: &'static str) -> NavIcon {
    static ICONS: OnceLock<Vec<(&'static str, NavIcon)>> = OnceLock::new();
    ICONS
        .get_or_init(|| {
            let mut icons: Vec<(&'static str, NavIcon)> = Vec::new();
            for action in ACTIONS.iter().flatten() {
                if !action.icon.is_empty() && !icons.iter().any(|(n, _)| *n == action.icon) {
                    let symbol = Symbol::named(action.icon)
                        .unwrap_or_else(|| panic!("{} is a Material Symbol", action.icon));
                    icons.push((action.icon, NavIcon::symbol(symbol)));
                }
            }
            icons
        })
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, icon)| icon.clone())
        .expect("an action icon")
}

/// The selected formatting toggles, one bit per item; Bold starts on.
struct Toggles(u32);

#[cfg(test)]
thread_local! {
    /// The toggles the last preview drew.
    static SELECTED: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// The preview screen: the app surface the component sits on.
pub(super) fn screen() -> Div {
    div()
        .relative()
        .w_full()
        .flex_1()
        .min_h(rem(360.))
        .flex()
        .flex_col()
        .rounded(rem(12.))
        .overflow_hidden()
        .border_1()
        .border_color(color(ColorRole::OUTLINE_VARIANT))
        .bg(color(ColorRole::SURFACE))
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let docked = config.option("variant") == "docked";
    let scheme = if config.option("color") == "vibrant" {
        ToolbarColor::Vibrant
    } else {
        ToolbarColor::Standard
    };
    let vertical = config.option("orientation") == "vertical";
    let count = config.choice("count") + 2;
    let toggles_on = config.bool("toggles");
    let toggles = preview.local("toolbar-toggles", window, cx, |_, _| Toggles(1));
    let selected = toggles.read(cx).0;
    #[cfg(test)]
    SELECTED.set(selected);
    let set = config.choice("actions");
    let items = ACTIONS[set][..count]
        .iter()
        .enumerate()
        .map(|(index, action)| {
            let id = ("toolbar-item", index);
            let mut button = if action.icon.is_empty() {
                ToolbarButton::label(id, action.label)
            } else {
                ToolbarButton::icon(id, nav_icon(action.icon), action.label)
            }
            .color(scheme);
            if action.danger {
                button = button.ink(ColorRole::ERROR);
            }
            if action.toggle && toggles_on {
                let toggles = toggles.clone();
                button = button
                    .toggle(true)
                    .selected(selected & (1 << index) != 0)
                    .on_click(move |_, _, cx| {
                        toggles.update(cx, |toggles, cx| {
                            toggles.0 ^= 1 << index;
                            cx.notify();
                        })
                    });
            }
            button
        });
    let toolbar = Toolbar::new("playground-toolbar", "Actions")
        .configuration(if docked {
            ToolbarConfiguration::Docked
        } else {
            ToolbarConfiguration::Floating
        })
        .orientation(if vertical {
            ToolbarOrientation::Vertical
        } else {
            ToolbarOrientation::Horizontal
        })
        .color(scheme)
        .elevated(config.bool("elevated"))
        .docked_centered(config.option("docked_items") == "center")
        .items(items);
    let toolbar = div()
        .debug_selector(|| "playground-toolbar".into())
        .when(docked, |toolbar| toolbar.w_full())
        .child(toolbar);
    screen()
        .map(|screen| {
            if docked {
                screen.justify_end()
            } else {
                screen.items_center().justify_center()
            }
        })
        .child(toolbar)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::harness::Harness;
    use super::*;
    use core::prelude::v1::test;
    use gpui::TestAppContext;

    #[gpui::test]
    fn every_scenario_renders_the_toolbar(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.render_every_scenario(|harness, scenario| {
            assert!(
                harness.bounds("playground-toolbar").is_some(),
                "scenario {scenario}"
            );
        });
    }

    #[gpui::test]
    fn docked_spans_the_screen_and_vertical_stands_up(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        let floating = harness.expect("playground-toolbar");
        assert!(floating.size.width > floating.size.height);
        harness.set("orientation", Value::Choice(1));
        let vertical = harness.expect("playground-toolbar");
        assert!(vertical.size.height > vertical.size.width);
        harness.set("variant", Value::Choice(0));
        let docked = harness.expect("playground-toolbar");
        assert!(docked.size.width > floating.size.width * 2.);
    }

    #[gpui::test]
    fn formatting_toggles_select_on_click(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        let toolbar = harness.expect("playground-toolbar");
        assert_eq!(SELECTED.get(), 0b001, "Bold starts selected");
        // Italic is the second 48dp item, after 8dp padding and a 4dp gap.
        let scale = f32::from(harness.cx.update(|window, _| window.rem_size())) / BASE_PX;
        let italic = gpui::point(
            toolbar.left() + gpui::px((8. + 48. + 4. + 24.) * scale),
            toolbar.center().y,
        );
        harness.click_at(italic);
        assert_eq!(SELECTED.get(), 0b011);
        harness.click_at(italic);
        assert_eq!(SELECTED.get(), 0b001);
        // Without formatting toggles the items are plain actions.
        harness.set("toggles", Value::Bool(false));
        harness.click_at(italic);
        assert_eq!(SELECTED.get(), 0b001);
    }
}
