//! Drawer playground: md3.io's Drawer configuration (standard or modal,
//! dense, start or end position, width, destination sets, headline,
//! selection, icons, badges, section labels, open, dismissible) on the
//! real `NavigationDrawer` beside or over a content pane whose menu button
//! opens and closes it.
use super::super::playground::PaneHint;
use super::super::shell::{panes_stacked, preview_area};
use super::prelude::*;
use gpui_kit_material::icons::Symbol;
use std::sync::OnceLock;

const SETS: &[&str] = &["Default", "Mailbox", "Mailbox folders", "Files"];
/// Every destination label of every set; a label outside the current set
/// selects nothing.
const LABELS: &[&str] = &[
    "Photos",
    "Albums",
    "Recents",
    "Trash",
    "Inbox",
    "Outbox",
    "Favorites",
    "Family",
    "School",
    "Work",
    "My files",
    "Shared",
    "Starred",
    "Offline",
    "Uploads",
];

/// The least a standard drawer leaves its content, in design px.
const MIN_CONTENT: f32 = 200.;

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Mailbox",
            values: &[
                ("destinations", Set::Option("Mailbox")),
                ("headline", Set::Text("Mail")),
                ("selected", Set::Option("Inbox")),
                ("badges", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Mailbox folders",
            values: &[
                ("destinations", Set::Option("Mailbox folders")),
                ("headline", Set::Text("Mail")),
                ("selected", Set::Option("Inbox")),
                ("badges", Set::Bool(true)),
                ("sections", Set::Bool(true)),
                ("variant", Set::Option("modal")),
            ],
        },
        Scenario {
            name: "Files",
            values: &[
                ("destinations", Set::Option("Files")),
                ("headline", Set::Text("Drive")),
                ("selected", Set::Option("My files")),
                ("sections", Set::Bool(true)),
                ("dense", Set::Bool(true)),
                ("position", Set::Option("end")),
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
                    options: &["standard", "modal"],
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
            controls: &[
                Control::Segmented {
                    id: "position",
                    label: "Position",
                    options: &["start", "end"],
                    default: 0,
                },
                Control::Slider {
                    id: "width",
                    label: "Width",
                    min: 240.,
                    max: 360.,
                    step: 4.,
                    default: 280.,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "destinations",
                    label: "Destinations",
                    options: SETS,
                    default: 0,
                },
                Control::Text {
                    id: "headline",
                    label: "Headline",
                    default: "Photos",
                },
                Control::Select {
                    id: "selected",
                    label: "Selected",
                    options: LABELS,
                    default: 0,
                },
                Control::Switch {
                    id: "icons",
                    label: "Icons",
                    default: true,
                },
                Control::Switch {
                    id: "badges",
                    label: "Badges",
                    default: false,
                },
                Control::Switch {
                    id: "sections",
                    label: "Section labels",
                    default: false,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "open",
                    label: "Open",
                    default: true,
                },
                Control::Switch {
                    id: "dismissible",
                    label: "Dismissible",
                    default: true,
                },
            ],
        },
    ],
    preview,
};

/// One destination: label, Material Symbol and badge text.
type Destination = (&'static str, &'static str, Option<&'static str>);

/// A destination set: its destinations and section labels (before index).
struct DestinationSet {
    destinations: &'static [Destination],
    sections: &'static [(usize, &'static str)],
}

const PHOTOS: DestinationSet = DestinationSet {
    destinations: &[
        ("Photos", "photo", None),
        ("Albums", "photo_album", Some("12")),
        ("Recents", "schedule", Some("3")),
        ("Trash", "delete", None),
    ],
    sections: &[(0, "Library")],
};

const MAILBOX: DestinationSet = DestinationSet {
    destinations: &[
        ("Inbox", "inbox", Some("24")),
        ("Outbox", "outbox", None),
        ("Favorites", "favorite", None),
        ("Trash", "delete", None),
    ],
    sections: &[(0, "Folders")],
};

const FOLDERS: DestinationSet = DestinationSet {
    destinations: &[
        ("Inbox", "inbox", Some("100+")),
        ("Outbox", "outbox", None),
        ("Favorites", "favorite", None),
        ("Trash", "delete", None),
        ("Family", "label", None),
        ("School", "label", Some("4")),
        ("Work", "label", None),
    ],
    sections: &[(0, "Folders"), (4, "Labels")],
};

const FILES: DestinationSet = DestinationSet {
    destinations: &[
        ("My files", "folder", None),
        ("Shared", "folder_shared", Some("2")),
        ("Starred", "star", None),
        ("Recents", "schedule", None),
        ("Offline", "offline_pin", None),
        ("Uploads", "upload", Some("1")),
    ],
    sections: &[(0, "Files"), (4, "On this device")],
};

const ALL: [&DestinationSet; 4] = [&PHOTOS, &MAILBOX, &FOLDERS, &FILES];

/// Each set's icons, built once (resolving symbol paths allocates).
fn icons(set: usize) -> &'static [NavIcon] {
    static ICONS: OnceLock<Vec<Vec<NavIcon>>> = OnceLock::new();
    &ICONS.get_or_init(|| {
        ALL.iter()
            .map(|set| {
                set.destinations
                    .iter()
                    .map(|(_, name, _)| {
                        NavIcon::symbol(
                            Symbol::named(name).unwrap_or_else(|| panic!("{name} is a symbol")),
                        )
                    })
                    .collect()
            })
            .collect()
    })[set]
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let index = config.choice("destinations");
    let set = ALL[index];
    let badges = config.bool("badges");
    // Where a standard drawer would leave its content no usable pane, it overlays
    // the content instead, as a modal one does, and a press beside it closes it.
    let room = preview_area(window, panes_stacked(window, PaneHint::default())).0;
    let overlays = room < config.number("width") + MIN_CONTENT;
    let modal = overlays || config.option("variant") == "modal";
    let end = config.option("position") == "end";
    let open = config.bool("open");
    let selected = config.option("selected");
    let headline = config.text("headline");
    let items = set
        .destinations
        .iter()
        .zip(icons(index))
        .map(|((label, _, badge), icon)| {
            let item = NavItem::new(*label, *label, icon.clone());
            match badge {
                Some(text) if badges => item.badge(NavBadge::Large((*text).into())),
                _ => item,
            }
        });
    let set_selected = preview.setter("selected", cx);
    let set_open = std::rc::Rc::new(preview.setter("open", cx));
    let mut drawer = NavigationDrawer::new("playground-drawer", "Main navigation")
        .items(items)
        .selected(selected)
        .modal(modal)
        .open(open)
        .dense(config.bool("dense"))
        .position(if end {
            DrawerPosition::End
        } else {
            DrawerPosition::Start
        })
        .width(config.number("width"))
        .icons(config.bool("icons"))
        .on_change(move |value, window, cx| {
            if let Some(index) = LABELS.iter().position(|label| *label == value.as_ref()) {
                set_selected(Value::Choice(index), window, cx);
            }
        });
    if !headline.is_empty() {
        drawer = drawer.headline(headline);
    }
    if config.bool("sections") {
        for (before, label) in set.sections {
            drawer = drawer.section(*before, *label);
        }
    }
    if overlays || config.bool("dismissible") {
        let close = set_open.clone();
        drawer = drawer.on_dismiss(move |window, cx| close(Value::Bool(false), window, cx));
    }
    let toggle = set_open.clone();
    let title = set
        .destinations
        .iter()
        .find(|(label, _, _)| *label == selected)
        .map_or("", |(label, _, _)| *label);
    let content = div()
        .flex_1()
        .min_w_0()
        .h_full()
        .p(rem(12.))
        .flex()
        .flex_col()
        .gap(rem(8.))
        .bg(color(ColorRole::SURFACE))
        .child(
            div()
                .flex()
                .items_center()
                .gap(rem(8.))
                .child(
                    div().debug_selector(|| "drawer-menu".into()).child(
                        IconButton::new(
                            "drawer-menu",
                            symbol(if open { "menu_open" } else { "menu" }, 24.),
                            if open {
                                "Close navigation"
                            } else {
                                "Open navigation"
                            },
                        )
                        .on_click(move |_, window, cx| toggle(Value::Bool(!open), window, cx)),
                    ),
                )
                .child(
                    div()
                        .map(|text| type_role(text, TypeRole::TitleLarge))
                        .text_color(color(ColorRole::ON_SURFACE))
                        .child(title),
                ),
        );
    let frame = div()
        .id("playground-navigation-drawer")
        .debug_selector(|| "playground-navigation-drawer".into())
        .relative()
        .w_full()
        .h_full()
        .min_h(rem(360.))
        .flex()
        .rounded(rem(16.))
        .overflow_hidden()
        .border_1()
        .border_color(color(ColorRole::OUTLINE_VARIANT));
    // A modal drawer overlays the content; a standard one sits on its edge.
    if modal || end {
        frame.child(content).child(drawer).into_any_element()
    } else {
        frame.child(drawer).child(content).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::super::nav_harness::Harness;
    use super::*;
    use core::prelude::v1::test;
    use gpui::TestAppContext;

    #[gpui::test]
    fn every_scenario_renders_the_drawer(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.render_every_scenario(|harness, scenario| {
            assert!(
                harness.bounds("playground-navigation-drawer").is_some(),
                "scenario {scenario}"
            );
        });
    }

    #[gpui::test]
    fn clicks_select_and_a_modal_drawer_dismisses(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        // A standard start drawer: 1dp frame border, 12dp padding, the 56dp
        // headline, then 56dp rows.
        let frame = harness.expect("playground-navigation-drawer");
        let recents =
            frame.origin + point(px(1. + 12. + 100.), px(1. + 12. + 56. + 56. * 2. + 28.));
        harness.click_at(recents);
        assert_eq!(harness.config().option("selected"), "Recents");

        // The menu button closes and reopens the drawer.
        harness.click("drawer-menu");
        assert!(!harness.config().bool("open"));
        harness.advance(2000);
        harness.set("variant", Value::Choice(1));
        harness.click("drawer-menu");
        assert!(harness.config().bool("open"));
        harness.advance(2000);
        // Opening the modal drawer focused it: Escape dismisses.
        harness.keys("escape");
        assert!(!harness.config().bool("open"));

        // Not dismissible: Escape and the scrim leave it open.
        harness.set("dismissible", Value::Bool(false));
        harness.set("open", Value::Bool(true));
        harness.advance(2000);
        harness.keys("escape");
        harness.click_at(frame.origin + point(frame.size.width - px(20.), px(200.)));
        assert!(harness.config().bool("open"));
    }
}
