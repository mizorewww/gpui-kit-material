//! Tabs playground: md3.io's Tabs configuration (primary or secondary,
//! divider, tab sets, selection, icons, badges, three or six tabs,
//! scrollable, select on arrow keys) on the real `MaterialTabs` heading a
//! content pane that shows the selected tab.
use super::prelude::*;

const SETS: &[&str] = &[
    "Default",
    "Trip planner",
    "Media gallery",
    "Recipe sections",
];

/// Each set's six tabs: label and Material Symbol.
const TABS: [[(&str, &str); 6]; 4] = [
    [
        ("Inbox", "inbox"),
        ("Favorites", "favorite"),
        ("Sent", "send"),
        ("Drafts", "draft"),
        ("Archive", "archive"),
        ("Spam", "report"),
    ],
    [
        ("Flights", "flight"),
        ("Hotels", "hotel"),
        ("Cars", "directions_car"),
        ("Activities", "hiking"),
        ("Dining", "restaurant"),
        ("Notes", "notes"),
    ],
    [
        ("Photos", "photo"),
        ("Videos", "videocam"),
        ("Music", "music_note"),
        ("Albums", "photo_album"),
        ("Shared", "group"),
        ("Trash", "delete"),
    ],
    [
        ("Overview", "info"),
        ("Ingredients", "grocery"),
        ("Steps", "format_list_numbered"),
        ("Nutrition", "nutrition"),
        ("Reviews", "reviews"),
        ("Notes", "notes"),
    ],
];

/// Every tab label of every set (once); a label outside the current tabs
/// selects nothing.
const LABELS: &[&str] = &[
    "Inbox",
    "Favorites",
    "Sent",
    "Drafts",
    "Archive",
    "Spam",
    "Flights",
    "Hotels",
    "Cars",
    "Activities",
    "Dining",
    "Notes",
    "Photos",
    "Videos",
    "Music",
    "Albums",
    "Shared",
    "Trash",
    "Overview",
    "Ingredients",
    "Steps",
    "Nutrition",
    "Reviews",
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Trip planner",
            values: &[
                ("tabs", Set::Option("Trip planner")),
                ("selected", Set::Option("Hotels")),
                ("count", Set::Option("6")),
                ("scrollable", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Media gallery",
            values: &[
                ("tabs", Set::Option("Media gallery")),
                ("selected", Set::Option("Photos")),
                ("badges", Set::Bool(true)),
                ("auto", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Recipe sections",
            values: &[
                ("tabs", Set::Option("Recipe sections")),
                ("selected", Set::Option("Ingredients")),
                ("variant", Set::Option("secondary")),
                ("icons", Set::Bool(false)),
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
                    options: &["primary", "secondary"],
                    default: 0,
                },
                Control::Switch {
                    id: "divider",
                    label: "Divider",
                    default: true,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "tabs",
                    label: "Tabs",
                    options: SETS,
                    default: 0,
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
                Control::Segmented {
                    id: "count",
                    label: "Tab count",
                    options: &["3", "6"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "scrollable",
                    label: "Scrollable",
                    default: false,
                },
                Control::Switch {
                    id: "auto",
                    label: "Select on arrow keys",
                    default: false,
                },
            ],
        },
    ],
    preview,
};

fn preview(
    config: &Config,
    preview: &mut Preview,
    _: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let tabs = &TABS[config.choice("tabs")];
    let count = if config.option("count") == "6" { 6 } else { 3 };
    let icons = config.bool("icons");
    let badges = config.bool("badges");
    let selected = config.option("selected");
    let items = tabs[..count]
        .iter()
        .enumerate()
        .map(|(index, (label, icon))| {
            let mut item = TabItem::new(*label, *label);
            if icons {
                item = item.icon(symbol(icon, 24.));
            }
            match index {
                0 if badges => item.badge(NavBadge::Large("3".into())),
                2 if badges => item.badge(NavBadge::Small),
                _ => item,
            }
        });
    let set_selected = preview.setter("selected", cx);
    let tabs_row = MaterialTabs::new("playground-tabs", "Sections")
        .variant(if config.option("variant") == "secondary" {
            TabsVariant::Secondary
        } else {
            TabsVariant::Primary
        })
        .divider(config.bool("divider"))
        .scrollable(config.bool("scrollable"))
        .auto_activate(config.bool("auto"))
        .items(items)
        .selected(selected)
        .on_change(move |value, window, cx| {
            if let Some(index) = LABELS.iter().position(|label| *label == value.as_ref()) {
                set_selected(Value::Choice(index), window, cx);
            }
        });
    let current = tabs[..count]
        .iter()
        .find(|(label, _)| *label == selected)
        .map_or("", |(label, _)| *label);
    div()
        .id("playground-tabs")
        .debug_selector(|| "playground-tabs".into())
        .w_full()
        .max_w(rem(640.))
        .flex()
        .flex_col()
        .rounded(rem(16.))
        .overflow_hidden()
        .bg(color(ColorRole::SURFACE))
        .child(tabs_row)
        .child(
            div()
                .p(rem(24.))
                .h(rem(160.))
                .map(|text| type_role(text, TypeRole::BodyLarge))
                .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                .child(current),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::nav_harness::Harness;
    use super::*;
    use core::prelude::v1::test;
    use gpui::TestAppContext;

    #[gpui::test]
    fn every_scenario_renders_the_tabs(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.render_every_scenario(|harness, scenario| {
            assert!(
                harness.bounds("playground-tabs").is_some(),
                "scenario {scenario}"
            );
        });
    }

    #[gpui::test]
    fn clicks_and_arrow_keys_select_tabs(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        // Three fixed tabs share the width; icon-over-label tabs are 64dp.
        let frame = harness.expect("playground-tabs");
        let third = point(
            frame.left() + frame.size.width * (5. / 6.),
            frame.top() + px(32.),
        );
        harness.click_at(third);
        assert_eq!(harness.config().option("selected"), "Sent");
        // Manual activation: arrows only move focus.
        harness.keys("left");
        assert_eq!(harness.config().option("selected"), "Sent");
        // Select on arrow keys: the focused tab is selected as it moves.
        harness.set("auto", Value::Bool(true));
        harness.keys("left");
        assert_eq!(harness.config().option("selected"), "Inbox");
    }
}
