//! Top app bar playground: md3.io's Top app bar configuration (small,
//! center-aligned, medium and large bars, the scrolled state,
//! compressible flexible bars, five contexts, title, navigation button and
//! action count) on the real `TopAppBar` heading a scrolling article. The
//! article's scroll offset drives the bar: scrolled under, it fades to
//! surface-container, and a compressible medium or large bar collapses.
use super::prelude::*;

const CONTEXTS: &[&str] = &[
    "Default",
    "Trail guide",
    "Article review",
    "Media attachment",
    "Article reader",
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Trail guide",
            values: &[
                ("context", Set::Option("Trail guide")),
                ("type", Set::Option("large")),
                ("title", Set::Text("Trail guide")),
                ("actions", Set::Option("2")),
            ],
        },
        Scenario {
            name: "Article review",
            values: &[
                ("context", Set::Option("Article review")),
                ("type", Set::Option("center")),
                ("title", Set::Text("Review article")),
                ("actions", Set::Option("1")),
            ],
        },
        Scenario {
            name: "Media attachment",
            values: &[
                ("context", Set::Option("Media attachment")),
                ("type", Set::Option("small")),
                ("title", Set::Text("Attachments")),
                ("scrolled", Set::Bool(true)),
                ("actions", Set::Option("2")),
            ],
        },
        Scenario {
            name: "Article reader",
            values: &[
                ("context", Set::Option("Article reader")),
                ("type", Set::Option("medium")),
                ("title", Set::Text("Reading list")),
                ("actions", Set::Option("2")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Select {
                    id: "type",
                    label: "Type",
                    options: &["small", "center", "medium", "large"],
                    default: 0,
                },
                Control::Switch {
                    id: "scrolled",
                    label: "Scrolled state",
                    default: false,
                },
                Control::Switch {
                    id: "compressible",
                    label: "Compressible",
                    default: true,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "context",
                    label: "Context",
                    options: CONTEXTS,
                    default: 0,
                },
                Control::Text {
                    id: "title",
                    label: "Title",
                    default: "My library",
                },
                Control::Switch {
                    id: "navigation",
                    label: "Navigation button",
                    default: true,
                },
                Control::Segmented {
                    id: "actions",
                    label: "Action count",
                    options: &["0", "1", "2"],
                    default: 1,
                },
            ],
        },
    ],
    preview,
};

/// Each context's navigation icon and two actions: (symbol, label).
const CONTEXT_ICONS: [[(&str, &str); 3]; 5] = [
    [
        ("menu", "Open navigation"),
        ("favorite", "Favorites"),
        ("more_vert", "More options"),
    ],
    [
        ("arrow_back", "Back"),
        ("map", "Show map"),
        ("bookmark", "Save trail"),
    ],
    [
        ("close", "Close"),
        ("check", "Approve"),
        ("more_vert", "More options"),
    ],
    [
        ("arrow_back", "Back"),
        ("attach_file", "Attach"),
        ("more_vert", "More options"),
    ],
    [
        ("arrow_back", "Back"),
        ("share", "Share"),
        ("bookmark", "Save article"),
    ],
];

/// The article under the bar.
const ARTICLE: &[&str] = &[
    "Scroll this content to see the bar respond: it fills with \
     surface-container once content passes beneath it, and a compressible \
     medium or large bar collapses into a single row.",
    "The trail starts at the old ranger station and climbs gently through \
     a forest of pine and fir before opening onto a ridge with views across \
     the valley.",
    "Plan for about three hours. Water is available at the trailhead, but \
     not along the route, so carry at least two liters per person.",
    "In spring, the meadows below the ridge fill with wildflowers. In \
     autumn, the larches turn gold and the light comes in low and warm.",
    "The descent follows the creek. Watch your footing on the wet rocks \
     near the falls, and keep to the marked path where it crosses private \
     land.",
    "Dogs are welcome on a leash. Camping is permitted only at the \
     designated sites past the second bridge.",
    "Return the way you came, or take the shuttle from the lower lot on \
     weekends between May and October.",
    "Share your photos and trip notes with other hikers, and report any \
     fallen trees or washed-out sections to the visitor center.",
];

/// The article's scroll position, kept across frames.
struct Article {
    scroll: ScrollHandle,
}

fn variant(config: &Config) -> TopAppBarVariant {
    match config.option("type") {
        "center" => TopAppBarVariant::CenterAligned,
        "medium" => TopAppBarVariant::MediumFlexible,
        "large" => TopAppBarVariant::LargeFlexible,
        _ => TopAppBarVariant::Small,
    }
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let article = preview.local("top-app-bar-article", window, cx, |_, _| Article {
        scroll: ScrollHandle::new(),
    });
    let scroll = article.read(cx).scroll.clone();
    let variant = variant(config);
    // The article's offset in design px.
    let offset = (-f32::from(scroll.offset().y) * BASE_PX / f32::from(window.rem_size())).max(0.);
    let forced = config.bool("scrolled");
    let compressible = config.bool("compressible");
    // The scrolled state shows a compressible bar fully collapsed.
    let collapse = if !compressible {
        0.
    } else if forced {
        offset.max(variant.expanded_height(false))
    } else {
        offset
    };
    let icons = CONTEXT_ICONS[config.choice("context")];
    let mut bar = TopAppBar::new("playground-top-app-bar", variant)
        .title(config.text("title"))
        .scroll_offset(collapse)
        .scrolled_under(forced || offset > 0.)
        .actions(icons[1..=config.choice("actions")].iter().enumerate().map(
            |(index, (icon, label))| {
                IconButton::new(("top-app-bar-action", index), symbol(icon, 24.), *label)
            },
        ));
    if config.bool("navigation") {
        let (icon, label) = icons[0];
        bar = bar.navigation_icon(IconButton::new(
            "top-app-bar-navigation",
            symbol(icon, 24.),
            label,
        ));
    }
    let content = div()
        .id("top-app-bar-content")
        .debug_selector(|| "top-app-bar-content".into())
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .track_scroll(&scroll)
        // Re-render the bar as the article scrolls under it.
        .on_scroll(cx.listener(|_, _, _, cx| cx.notify()))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(rem(16.))
                .px(rem(16.))
                .pt(rem(8.))
                .pb(rem(24.))
                .children(ARTICLE.iter().map(|paragraph| {
                    type_role(div(), TypeRole::BodyLarge)
                        .text_color(color(ColorRole::ON_SURFACE))
                        .child(*paragraph)
                })),
        );
    super::toolbar::screen()
        .h(rem(420.))
        .child(
            div()
                .debug_selector(|| "playground-top-app-bar".into())
                .w_full()
                .child(bar),
        )
        .child(content)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::harness::Harness;
    use super::*;
    use core::prelude::v1::test;
    use gpui::{Modifiers, ScrollDelta, ScrollEvent, TestAppContext, point, px};

    #[gpui::test]
    fn every_scenario_renders_the_bar(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.render_every_scenario(|harness, scenario| {
            assert!(
                harness.bounds("playground-top-app-bar").is_some(),
                "scenario {scenario}"
            );
        });
    }

    #[gpui::test]
    fn scrolling_the_content_collapses_a_compressible_large_bar(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        let row = harness.expect("playground-top-app-bar").size.height;
        harness.set("type", Value::Choice(3));
        let expanded = harness.expect("playground-top-app-bar").size.height;
        assert!(expanded > row * 1.5, "large bars start expanded");
        let content = harness.expect("top-app-bar-content").center();
        harness.cx.simulate_event(ScrollEvent {
            position: content,
            delta: ScrollDelta::Pixels(point(px(0.), px(-200.))),
            modifiers: Modifiers::none(),
            ..Default::default()
        });
        harness.settle();
        let collapsed = harness.expect("playground-top-app-bar").size.height;
        assert_eq!(collapsed, row, "collapsed to one row");

        // A bar that is not compressible keeps its height when scrolled.
        harness.set("compressible", Value::Bool(false));
        assert_eq!(
            harness.expect("playground-top-app-bar").size.height,
            expanded
        );
    }

    #[gpui::test]
    fn the_scrolled_state_collapses_without_scrolling(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        let row = harness.expect("playground-top-app-bar").size.height;
        harness.set("type", Value::Choice(2));
        assert!(harness.expect("playground-top-app-bar").size.height > row);
        harness.set("scrolled", Value::Bool(true));
        assert_eq!(harness.expect("playground-top-app-bar").size.height, row);
    }
}
