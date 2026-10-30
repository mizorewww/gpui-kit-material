//! Navigation bar playground: md3.io's Navigation bar configuration
//! (destination sets, selection, accessible label, item layout, ripple) on
//! the real `NavigationBar` along the bottom of a screen. "Auto" lets the
//! bar pick vertical or horizontal items from its own width.
use super::prelude::*;
use gpui_kit_material::icons::Symbol;
use std::sync::OnceLock;

const SETS: &[&str] = &["Default", "Unread", "New in music", "Ten in music"];
/// Every destination label of every set; a label outside the current set
/// selects nothing.
const LABELS: &[&str] = &[
    "Inbox",
    "Favorites",
    "Sent",
    "Home",
    "Music",
    "Podcasts",
    "Library",
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Unread",
            values: &[("destinations", Set::Option("Unread"))],
        },
        Scenario {
            name: "New in music",
            values: &[
                ("destinations", Set::Option("New in music")),
                ("selected", Set::Option("Home")),
                ("label", Set::Text("Music navigation")),
            ],
        },
        Scenario {
            name: "Ten in music",
            values: &[
                ("destinations", Set::Option("Ten in music")),
                ("selected", Set::Option("Library")),
                ("label", Set::Text("Music navigation")),
                ("layout", Set::Option("Vertical")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "destinations",
                    label: "Destinations",
                    options: SETS,
                    default: 0,
                },
                Control::Select {
                    id: "selected",
                    label: "Selected",
                    options: LABELS,
                    default: 0,
                },
                Control::Text {
                    id: "label",
                    label: "Accessible label",
                    default: "Mail navigation",
                },
            ],
        },
        Section {
            title: "Layout",
            controls: &[Control::Select {
                id: "layout",
                label: "Item layout",
                options: &["Auto", "Vertical", "Horizontal"],
                default: 0,
            }],
        },
        Section {
            title: "Behavior",
            controls: &[Control::Switch {
                id: "ripple",
                label: "Ripple",
                default: true,
            }],
        },
    ],
    preview,
};

const MAIL: &[(&str, &str)] = &[
    ("Inbox", "inbox"),
    ("Favorites", "favorite"),
    ("Sent", "send"),
];
const MUSIC: &[(&str, &str)] = &[
    ("Home", "home"),
    ("Music", "music_note"),
    ("Podcasts", "podcasts"),
    ("Library", "library_music"),
];

/// A set's icons, built once (resolving symbol paths allocates).
fn icons(set: &'static [(&'static str, &'static str)]) -> &'static [NavIcon] {
    static MAIL_ICONS: OnceLock<Vec<NavIcon>> = OnceLock::new();
    static MUSIC_ICONS: OnceLock<Vec<NavIcon>> = OnceLock::new();
    let cell = if std::ptr::eq(set, MAIL) {
        &MAIL_ICONS
    } else {
        &MUSIC_ICONS
    };
    cell.get_or_init(|| {
        set.iter()
            .map(|(_, name)| {
                NavIcon::symbol(Symbol::named(name).unwrap_or_else(|| panic!("{name} is a symbol")))
            })
            .collect()
    })
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    _: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let kind = config.option("destinations");
    let set = if kind == "Default" || kind == "Unread" {
        MAIL
    } else {
        MUSIC
    };
    let badge = |label: &str| match (kind, label) {
        ("Unread", "Inbox") => NavBadge::Large("3".into()),
        ("New in music", "Music") => NavBadge::Small,
        ("Ten in music", "Music") => NavBadge::Large("10".into()),
        _ => NavBadge::None,
    };
    let items = set
        .iter()
        .zip(icons(set))
        .map(|((label, _), icon)| NavItem::new(*label, *label, icon.clone()).badge(badge(label)));
    let selected = config.option("selected");
    let set_selected = preview.setter("selected", cx);
    let label = config.text("label");
    let mut bar = NavigationBar::new("playground-bar", label)
        .items(items)
        .selected(selected)
        .ripple(config.bool("ripple"))
        .on_change(move |value, window, cx| {
            if let Some(index) = LABELS.iter().position(|label| *label == value.as_ref()) {
                set_selected(Value::Choice(index), window, cx);
            }
        });
    bar = match config.option("layout") {
        "Vertical" => bar.layout(NavItemLayout::Vertical),
        "Horizontal" => bar.layout(NavItemLayout::Horizontal),
        _ => bar.adaptive(true),
    };
    let title = set
        .iter()
        .find(|(label, _)| *label == selected)
        .map_or("", |(label, _)| *label);
    div()
        .id("playground-navigation-bar")
        .debug_selector(|| "playground-navigation-bar".into())
        .w_full()
        .h_full()
        .min_h(rem(360.))
        .flex()
        .flex_col()
        .rounded(rem(16.))
        .overflow_hidden()
        .child(
            div()
                .flex_1()
                .min_h_0()
                .p(rem(24.))
                .bg(color(ColorRole::SURFACE))
                .child(
                    div()
                        .map(|text| type_role(text, TypeRole::TitleLarge))
                        .text_color(color(ColorRole::ON_SURFACE))
                        .child(title),
                ),
        )
        .child(bar)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::harness::Harness;
    use super::*;
    use core::prelude::v1::test;
    use gpui::TestAppContext;

    /// The center of item `index` of `count` in a bar spanning `frame`:
    /// horizontal items share at most 128dp each, centered.
    fn item(frame: Bounds<Pixels>, index: usize, count: usize, horizontal: bool) -> Point<Pixels> {
        let lane = if horizontal {
            frame.size.width.min(px(128. * count as f32))
        } else {
            frame.size.width
        };
        let left = frame.center().x - lane / 2.;
        point(
            left + lane * ((2 * index + 1) as f32 / (2 * count) as f32),
            frame.bottom() - px(32.),
        )
    }

    #[gpui::test]
    fn every_scenario_renders_the_bar(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.render_every_scenario(|harness, scenario| {
            assert!(
                harness.bounds("playground-navigation-bar").is_some(),
                "scenario {scenario}"
            );
        });
        harness.set("layout", Value::Choice(2));
        harness.expect("playground-navigation-bar");
    }

    #[gpui::test]
    fn clicking_a_destination_writes_back_in_every_layout(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        // Auto: the wide preview chooses horizontal items.
        let frame = harness.expect("playground-navigation-bar");
        harness.click_at(item(frame, 2, 3, true));
        assert_eq!(harness.config().option("selected"), "Sent");
        harness.set("layout", Value::Choice(1));
        harness.click_at(item(frame, 1, 3, false));
        assert_eq!(harness.config().option("selected"), "Favorites");
    }
}
