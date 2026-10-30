//! Bottom app bar playground: md3.io's Bottom app bar configuration (FAB
//! shown, centered or trailing, three action sets, action count, the FAB's
//! label, visibility) on the real `BottomAppBar` under scrolling content.
//! As on Android, scrolling the content down hides the bar and scrolling up
//! shows it again, both through the Visible control.
use super::prelude::*;
use gpui::ScrollDelta;

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Notes",
            values: &[
                ("actions", Set::Option("Notes")),
                ("count", Set::Option("3")),
                ("fab_label", Set::Text("New note")),
            ],
        },
        Scenario {
            name: "Document",
            values: &[
                ("actions", Set::Option("Document")),
                ("position", Set::Option("center")),
                ("count", Set::Option("2")),
                ("fab_label", Set::Text("Edit document")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Layout",
            controls: &[
                Control::Switch {
                    id: "fab",
                    label: "Show FAB",
                    default: true,
                },
                Control::Segmented {
                    id: "position",
                    label: "FAB position",
                    options: &["center", "end"],
                    default: 1,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "actions",
                    label: "Actions",
                    options: &["Default", "Notes", "Document"],
                    default: 0,
                },
                Control::Segmented {
                    id: "count",
                    label: "Action count",
                    options: &["1", "2", "3"],
                    default: 1,
                },
                Control::Text {
                    id: "fab_label",
                    label: "FAB label",
                    default: "Compose",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[Control::Switch {
                id: "visible",
                label: "Visible",
                default: true,
            }],
        },
    ],
    preview,
};

/// Each action set's actions `(symbol, label)` and FAB symbol.
const ACTIONS: [([(&str, &str); 3], &str); 3] = [
    (
        [
            ("favorite", "Favorite"),
            ("bookmark", "Bookmark"),
            ("share", "Share"),
        ],
        "add",
    ),
    (
        [
            ("check_box", "New checklist"),
            ("brush", "New drawing"),
            ("mic", "New voice note"),
        ],
        "add",
    ),
    (
        [
            ("search", "Find in document"),
            ("print", "Print"),
            ("share", "Share"),
        ],
        "edit",
    ),
];

const CONTENT: &[&str] = &[
    "Scroll down to hide the bar and up to bring it back.",
    "Bottom app bars keep frequent actions within reach of the thumb on \
     mobile screens, with an optional floating action button for the most \
     important one.",
    "Use up to four actions. Put the most common action first and keep the \
     set the same across screens, so people can find them without looking.",
    "When the content scrolls, the bar can step aside to give it more room \
     and return as soon as people scroll back up.",
    "The FAB stays inside the bar, on its trailing edge or in its center.",
    "Notes, documents and messages all benefit from quick access to \
     creation and sharing actions at the bottom of the screen.",
];

fn preview(
    config: &Config,
    preview: &mut Preview,
    _: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let (actions, fab_icon) = ACTIONS[config.choice("actions")];
    let count = config.choice("count") + 1;
    let mut bar = BottomAppBar::new("playground-bottom-app-bar", "Actions")
        .actions(
            actions[..count]
                .iter()
                .enumerate()
                .map(|(index, (icon, label))| {
                    IconButton::new(("bottom-app-bar-action", index), symbol(icon, 24.), *label)
                }),
        )
        .fab_centered(config.option("position") == "center")
        .visible(config.bool("visible"));
    if config.bool("fab") {
        let label = config.text("fab_label");
        let label = if label.is_empty() {
            SharedString::new_static(fab_icon)
        } else {
            label
        };
        bar = bar.fab(Fab::new("bottom-app-bar-fab", symbol(fab_icon, 24.), label));
    }
    let show = preview.setter("visible", cx);
    let content = div()
        .id("bottom-app-bar-content")
        .debug_selector(|| "bottom-app-bar-content".into())
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        // Down hides the bar, up shows it.
        .on_scroll(move |event, window, cx| {
            let dy = match event.delta {
                ScrollDelta::Pixels(delta) => f32::from(delta.y),
                ScrollDelta::Lines(delta) => delta.y,
            };
            if dy != 0. {
                show(Value::Bool(dy > 0.), window, cx);
            }
        })
        .child(
            div()
                .flex()
                .flex_col()
                .gap(rem(16.))
                .p(rem(16.))
                .children(CONTENT.iter().map(|paragraph| {
                    type_role(div(), TypeRole::BodyLarge)
                        .text_color(color(ColorRole::ON_SURFACE))
                        .child(*paragraph)
                })),
        );
    super::toolbar::screen()
        .h(rem(420.))
        .child(content)
        .child(
            div()
                .debug_selector(|| "playground-bottom-app-bar".into())
                .w_full()
                .child(bar),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::nav_harness::Harness;
    use super::*;
    use core::prelude::v1::test;
    use gpui::{Modifiers, ScrollEvent, TestAppContext, point, px};

    #[gpui::test]
    fn every_scenario_renders_the_bar(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.render_every_scenario(|harness, scenario| {
            assert!(
                harness.bounds("playground-bottom-app-bar").is_some(),
                "scenario {scenario}"
            );
        });
    }

    fn scroll(harness: &mut Harness, dy: f32) {
        let at = harness.expect("bottom-app-bar-content").center();
        harness.cx.simulate_event(ScrollEvent {
            position: at,
            delta: ScrollDelta::Pixels(point(px(0.), px(dy))),
            modifiers: Modifiers::none(),
            ..Default::default()
        });
        harness.settle();
    }

    #[gpui::test]
    fn scrolling_down_hides_the_bar_and_up_shows_it(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        let shown = harness.expect("playground-bottom-app-bar").size.height;
        assert!(shown > px(0.));
        scroll(&mut harness, -40.);
        assert!(!harness.config().bool("visible"));
        harness.advance(2000);
        assert_eq!(
            harness.expect("playground-bottom-app-bar").size.height,
            px(0.)
        );
        scroll(&mut harness, 40.);
        assert!(harness.config().bool("visible"));
        harness.advance(2000);
        assert_eq!(
            harness.expect("playground-bottom-app-bar").size.height,
            shown
        );
    }
}
