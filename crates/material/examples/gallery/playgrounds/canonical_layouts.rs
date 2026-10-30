//! Canonical layouts playground: m3e's list-detail, supporting pane and feed
//! on the real `PaneScaffold` and `FeedLayout`. Destinations navigate; the width
//! slider stands in for a window larger than the preview, and also narrows the
//! frame so the feed's columns follow it.
use super::prelude::*;

const MAIL: [(&str, &str); 4] = [
    ("Ali Connors", "Brunch this weekend?"),
    ("Sandra Adams", "Summer BBQ"),
    ("Trevor Hansen", "Oui oui"),
    ("Britta Holt", "Birthday gift"),
];
const POSTS: [&str; 7] = [
    "Mountains",
    "Coast",
    "Forest",
    "Desert",
    "City",
    "Lakes",
    "Islands",
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Compact",
            values: &[("width", Set::Number(360.))],
        },
        Scenario {
            name: "Three panes",
            values: &[("width", Set::Number(1700.))],
        },
        Scenario {
            name: "Compose pane rule",
            values: &[
                ("width", Set::Number(1300.)),
                ("rule", Set::Option("compose")),
            ],
        },
        Scenario {
            name: "Supporting pane",
            values: &[("layout", Set::Option("supporting pane"))],
        },
        Scenario {
            name: "Supporting pane, tall",
            values: &[
                ("layout", Set::Option("supporting pane")),
                ("width", Set::Number(700.)),
                ("height", Set::Number(1000.)),
            ],
        },
        Scenario {
            name: "Feed",
            values: &[("layout", Set::Option("feed"))],
        },
    ],
    sections: &[
        Section {
            title: "Layout",
            controls: &[
                Control::Select {
                    id: "layout",
                    label: "Layout",
                    options: &["list-detail", "supporting pane", "feed"],
                    default: 0,
                },
                Control::Slider {
                    id: "width",
                    label: "Window width",
                    min: 320.,
                    max: 1800.,
                    step: 20.,
                    default: 1000.,
                },
                Control::Slider {
                    id: "height",
                    label: "Window height",
                    min: 400.,
                    max: 1200.,
                    step: 20.,
                    default: 800.,
                },
            ],
        },
        Section {
            title: "Panes",
            controls: &[
                Control::Segmented {
                    id: "rule",
                    label: "Pane rule",
                    options: &["recommended", "compose"],
                    default: 0,
                },
                Control::Switch {
                    id: "extra",
                    label: "Extra pane",
                    default: true,
                },
                Control::Switch {
                    id: "handle",
                    label: "Drag handle",
                    default: true,
                },
                Control::Switch {
                    id: "rtl",
                    label: "Right to left",
                    default: false,
                },
            ],
        },
    ],
    preview,
};

fn headline(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .map(|div| type_role(div, TypeRole::TitleLarge))
        .text_color(color(ColorRole::ON_SURFACE))
        .child(text.into())
}

fn body(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .map(|div| type_role(div, TypeRole::BodyMedium))
        .text_color(color(ColorRole::ON_SURFACE_VARIANT))
        .child(text.into())
}

/// A pane's surface: a bar with an optional back button, then its content.
fn screen(
    id: &'static str,
    title: impl Into<SharedString>,
    back: Option<AnyElement>,
    content: impl IntoElement,
) -> Div {
    gpui_base::v_flex()
        .debug_selector(move || id.into())
        .min_h_full()
        .rounded(rem(16.))
        .bg(color(ColorRole::SURFACE_CONTAINER))
        .p(rem(16.))
        .gap(rem(12.))
        .child(
            div()
                .flex()
                .items_center()
                .gap(rem(8.))
                .children(back)
                .child(headline(title)),
        )
        .child(content)
}

fn back_button(
    id: &'static str,
    label: &'static str,
    action: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    div()
        .debug_selector(move || id.into())
        .child(
            IconButton::new(id, symbol("arrow_back", 24.), label)
                .on_click(move |_, window, cx| action(window, cx)),
        )
        .into_any_element()
}

fn go_back<K: Clone + PartialEq + 'static>(
    navigator: &Entity<PaneNavigator<K>>,
) -> impl Fn(&mut Window, &mut App) + 'static {
    let navigator = navigator.clone();
    move |_, cx| {
        navigator.update(cx, |navigator, cx| {
            navigator.navigate_back(BackNavigationBehavior::default());
            cx.notify();
        });
    }
}

fn list_detail(
    config: &Config,
    size_class: WindowSizeClass,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let navigator = preview.local("list-detail", window, cx, |_, _| {
        PaneNavigator::<usize>::list_detail()
    });
    let state = navigator.read(cx);
    let chosen = state.content_key().copied();
    let can_back = state.can_navigate_back(BackNavigationBehavior::default());
    let both = state.is_expanded(PaneRole::Primary) && state.is_expanded(PaneRole::Secondary);
    let mail = chosen.and_then(|index| MAIL.get(index));
    let mut rows = gpui_base::v_flex().w_full();
    for (index, (from, subject)) in MAIL.into_iter().enumerate() {
        let navigator = navigator.clone();
        rows = rows.child(
            div().debug_selector(move || format!("mail-{index}")).child(
                ListItem::new(("mail-row", index), from)
                    .supporting(subject)
                    .container(ColorRole::SURFACE_CONTAINER)
                    // Selection is only meaningful while both panes show.
                    .selected(both && chosen == Some(index))
                    .on_click(move |_, _, cx| {
                        navigator.update(cx, |navigator, cx| {
                            navigator.navigate_to(PaneRole::Primary, Some(index));
                            cx.notify();
                        });
                    }),
            ),
        );
    }
    let detail_body = gpui_base::v_flex()
        .gap(rem(12.))
        .items_start()
        .child(body(mail.map_or_else(
            || "Select an item from the list.".to_owned(),
            |(_, subject)| format!("This is a description about {subject}."),
        )))
        .when(mail.is_some() && config.bool("extra"), |column| {
            let navigator = navigator.clone();
            column.child(
                div().debug_selector(|| "show-notes".into()).child(
                    MaterialButton::new("show-notes")
                        .variant(ButtonVariant::Tonal)
                        .child("Show notes")
                        .on_click(move |_, _, cx| {
                            navigator.update(cx, |navigator, cx| {
                                let key = navigator.content_key().copied();
                                navigator.navigate_to(PaneRole::Tertiary, key);
                                cx.notify();
                            });
                        }),
                ),
            )
        });
    let list_pane = screen("list-pane", "Inbox", None, rows);
    let detail_back = can_back.then(|| back_button("detail-back", "Back", go_back(&navigator)));
    let detail_pane = screen(
        "detail-pane",
        mail.map_or("No item selected", |(_, subject)| *subject),
        detail_back,
        detail_body,
    );
    let mut scaffold = PaneScaffold::list_detail("playground-list-detail", &navigator)
        .size_class(Some(size_class))
        .rule(pane_rule(config))
        .rtl(config.bool("rtl"))
        .pane(PaneRole::Secondary, "Inbox", list_pane)
        .pane(PaneRole::Primary, "Message", detail_pane);
    if config.bool("extra") {
        let notes_back = can_back.then(|| back_button("notes-back", "Back", go_back(&navigator)));
        scaffold = scaffold.pane(
            PaneRole::Tertiary,
            "Notes",
            screen(
                "notes-pane",
                "Notes",
                notes_back,
                body(mail.map_or_else(
                    || "Nothing to show.".to_owned(),
                    |(_, subject)| format!("This is extra content about {subject}."),
                )),
            ),
        );
    }
    if config.bool("handle") {
        scaffold = scaffold.drag_handle("Resize panes");
    }
    scaffold.into_any_element()
}

fn supporting(
    config: &Config,
    size_class: WindowSizeClass,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let navigator = preview.local("supporting", window, cx, |_, _| {
        PaneNavigator::<usize>::supporting_pane()
    });
    let state = navigator.read(cx);
    let related_shown = state.value().get(PaneRole::Secondary) != PaneAdaptedValue::Hidden;
    let can_back = state.can_navigate_back(BackNavigationBehavior::default());
    let open = navigator.clone();
    let main_body = gpui_base::v_flex()
        .gap(rem(12.))
        .items_start()
        .child(body(
            "The main pane holds the primary content; the supporting pane adds related material.",
        ))
        .when(!related_shown, |column| {
            column.child(
                div().debug_selector(|| "show-related".into()).child(
                    MaterialButton::new("show-related")
                        .variant(ButtonVariant::Tonal)
                        .child("Show related")
                        .on_click(move |_, _, cx| {
                            open.update(cx, |navigator, cx| {
                                navigator.navigate_to(PaneRole::Secondary, None);
                                cx.notify();
                            });
                        }),
                ),
            )
        });
    let related_back = can_back.then(|| back_button("related-back", "Back", go_back(&navigator)));
    let mut rows = gpui_base::v_flex().w_full();
    for (index, (_, subject)) in MAIL.into_iter().enumerate() {
        rows = rows.child(
            ListItem::new(("related-row", index), subject).container(ColorRole::SURFACE_CONTAINER),
        );
    }
    let mut scaffold = PaneScaffold::supporting_pane("playground-supporting", &navigator)
        .size_class(Some(size_class))
        .rule(pane_rule(config))
        .rtl(config.bool("rtl"))
        .pane(
            PaneRole::Primary,
            "Article",
            screen("main-pane", "My content", None, main_body),
        )
        .pane(
            PaneRole::Secondary,
            "Related",
            screen("related-pane", "Related", related_back, rows),
        );
    if config.bool("extra") {
        scaffold = scaffold.pane(
            PaneRole::Tertiary,
            "Notes",
            screen("notes-pane", "Notes", None, body("Extra material.")),
        );
    }
    if config.bool("handle") {
        scaffold = scaffold.drag_handle("Resize panes");
    }
    scaffold.into_any_element()
}

fn feed() -> AnyElement {
    let mut layout = FeedLayout::new("playground-feed");
    for (index, name) in POSTS.into_iter().enumerate() {
        let card = gpui_base::v_flex()
            .h_full()
            .p(rem(16.))
            .gap(rem(4.))
            .rounded(rem(12.))
            .bg(color(ColorRole::SURFACE_CONTAINER_HIGHEST))
            .child(headline(name))
            .child(body(format!("Feed item {}", index + 1)));
        layout = if index == 0 {
            layout.wide_item(card)
        } else {
            layout.item(card)
        };
    }
    div()
        .id("playground-feed-scroll")
        .size_full()
        .overflow_y_scroll()
        .py(rem(16.))
        .child(layout)
        .into_any_element()
}

fn pane_rule(config: &Config) -> PaneRule {
    if config.option("rule") == "compose" {
        PaneRule::Compose
    } else {
        PaneRule::Recommended
    }
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let size_class = WindowSizeClass::of(config.number("width"), config.number("height"));
    let content = match config.option("layout") {
        "supporting pane" => supporting(config, size_class, preview, window, cx),
        "feed" => feed(),
        _ => list_detail(config, size_class, preview, window, cx),
    };
    div()
        .id("playground-canonical-frame")
        .debug_selector(|| "playground-canonical-layouts".into())
        .w_full()
        .max_w(rem(config.number("width")))
        .h(rem(560.))
        .rounded(rem(16.))
        .overflow_hidden()
        .border_1()
        .border_color(color(ColorRole::OUTLINE_VARIANT))
        .bg(color(ColorRole::SURFACE))
        .child(content)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use core::prelude::v1::test;

    #[gpui::test]
    fn every_scenario_and_layout_renders(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        render_every_scenario(&state, &mut cx);
        render_every_option(&state, &mut cx, &["layout", "rule"]);
        assert!(cx.debug_bounds("playground-canonical-layouts").is_some());
    }

    #[gpui::test]
    fn the_window_width_changes_the_pane_count(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        settle_motion(&mut cx);
        assert!(cx.debug_bounds("list-pane").is_some());
        assert!(cx.debug_bounds("detail-pane").is_some());
        assert!(cx.debug_bounds("notes-pane").is_none());
        set(&state, &mut cx, "width", Value::Number(360.));
        settle_motion(&mut cx);
        assert!(cx.debug_bounds("list-pane").is_some());
        assert!(cx.debug_bounds("detail-pane").is_none());
        set(&state, &mut cx, "width", Value::Number(1700.));
        settle_motion(&mut cx);
        assert!(cx.debug_bounds("notes-pane").is_some());
    }

    #[gpui::test]
    fn a_row_navigates_to_the_detail_and_back_returns(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        set(&state, &mut cx, "width", Value::Number(360.));
        settle_motion(&mut cx);
        let row = element_bounds(&mut cx, "mail-1").center();
        click(&mut cx, row);
        settle_motion(&mut cx);
        assert!(cx.debug_bounds("detail-pane").is_some());
        assert!(cx.debug_bounds("list-pane").is_none());
        let back = element_bounds(&mut cx, "detail-back").center();
        click(&mut cx, back);
        settle_motion(&mut cx);
        assert!(cx.debug_bounds("list-pane").is_some());
        assert!(cx.debug_bounds("detail-pane").is_none());
    }

    #[gpui::test]
    fn the_supporting_pane_opens_as_a_destination_or_reflows_when_tall(
        cx: &mut gpui::TestAppContext,
    ) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        set(&state, &mut cx, "layout", Value::Choice(1));
        set(&state, &mut cx, "width", Value::Number(700.));
        settle_motion(&mut cx);
        assert!(cx.debug_bounds("related-pane").is_none());
        let open_it = element_bounds(&mut cx, "show-related").center();
        click(&mut cx, open_it);
        settle_motion(&mut cx);
        assert!(cx.debug_bounds("related-pane").is_some());
        let back = element_bounds(&mut cx, "related-back").center();
        click(&mut cx, back);
        settle_motion(&mut cx);
        // Tall enough: it stacks under the main pane without navigating.
        set(&state, &mut cx, "height", Value::Number(1000.));
        settle_motion(&mut cx);
        let main = element_bounds(&mut cx, "main-pane");
        let related = element_bounds(&mut cx, "related-pane");
        assert!(related.top() >= main.top(), "{related:?} {main:?}");
    }

    #[gpui::test]
    fn the_feed_lays_out_in_columns(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        set(&state, &mut cx, "layout", Value::Choice(2));
        settle_motion(&mut cx);
        assert!(cx.debug_bounds("feed-layout").is_some());
    }
}
