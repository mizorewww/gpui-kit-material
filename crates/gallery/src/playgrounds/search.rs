//! Search playground: md3.io's Search configuration (contained or divided
//! style, bar or view state, docked or full-screen view, preset,
//! placeholder, query, suggestions, clear button, expand on focus,
//! collapse on blur, disabled) on the real `SearchField` and `SearchView`.
//!
//! The query is a Kit input bound two-way to the Query control; typing
//! filters the place suggestions and choosing one fills the query and
//! collapses the view. Focusing the bar expands it (Expand on focus); a
//! click outside the view or Escape collapses it (Collapse on blur, which
//! leaves the suggestions reachable by Tab and pointer); the view's back
//! button always collapses it. The State control follows all of these.
use super::prelude::*;
use gpui_base::input::InputState;

/// The place suggestions: city and country.
const PLACES: [(&str, &str); 8] = [
    ("Paris", "France"),
    ("Tokyo", "Japan"),
    ("New York", "United States"),
    ("Sydney", "Australia"),
    ("Cairo", "Egypt"),
    ("Rio de Janeiro", "Brazil"),
    ("Reykjavík", "Iceland"),
    ("Cape Town", "South Africa"),
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[Scenario {
        name: "Messages",
        values: &[
            ("preset", Set::Option("Messages")),
            ("style", Set::Option("divided")),
            ("placeholder", Set::Text("Search messages")),
            ("suggestions", Set::Option("none")),
        ],
    }],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Segmented {
                    id: "style",
                    label: "Style",
                    options: &["contained", "divided"],
                    default: 0,
                },
                Control::Segmented {
                    id: "field",
                    label: "Field",
                    options: &["standard", "surface", "tonal"],
                    default: 0,
                },
                Control::Segmented {
                    id: "state",
                    label: "State",
                    options: &["bar", "view"],
                    default: 0,
                },
                Control::Segmented {
                    id: "mode",
                    label: "View mode",
                    options: &["docked", "fullscreen"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "preset",
                    label: "Search",
                    options: &["Default", "Messages"],
                    default: 0,
                },
                Control::Text {
                    id: "placeholder",
                    label: "Placeholder",
                    default: "Search places",
                },
                Control::Text {
                    id: "query",
                    label: "Query",
                    default: "",
                },
                Control::Segmented {
                    id: "suggestions",
                    label: "Suggestions",
                    options: &["places", "none"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "clear",
                    label: "Clear button",
                    default: true,
                },
                Control::Switch {
                    id: "expand_on_focus",
                    label: "Expand on focus",
                    default: true,
                },
                Control::Switch {
                    id: "collapse_on_blur",
                    label: "Collapse on blur",
                    default: true,
                },
                Control::Switch {
                    id: "disabled",
                    label: "Disabled",
                    default: false,
                },
            ],
        },
    ],
    preview,
    code,
};

/// The places matching `query` (case-insensitive, city or country).
fn matching(query: &str) -> impl Iterator<Item = &'static (&'static str, &'static str)> {
    let query = query.trim().to_lowercase();
    PLACES.iter().filter(move |(city, country)| {
        query.is_empty()
            || city.to_lowercase().contains(&query)
            || country.to_lowercase().contains(&query)
    })
}

const BAR: Value = Value::Choice(0);
const VIEW: Value = Value::Choice(1);

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let input: Entity<InputState> = preview.input("query", window, cx);
    let disabled = config.bool("disabled");
    let placeholder = config.text("placeholder");
    if preview.changed("input-mode", (&placeholder, disabled)) {
        input.update(cx, |input, cx| {
            input.set_placeholder(placeholder.clone(), window, cx);
            input.set_disabled(disabled, cx);
        });
    }
    let expanded = config.option("state") == "view" && !disabled;
    let messages = config.option("preset") == "Messages";
    let set_state = std::rc::Rc::new(preview.setter("state", cx));
    let set_query = std::rc::Rc::new(preview.setter("query", cx));
    let query = config.text("query");

    let leading = if expanded {
        let collapse = preview.setter("state", cx);
        IconButton::new("search-back", symbol("arrow_back", 24.), "Back")
            .on_click(move |_, window, cx| collapse(BAR, window, cx))
            .into_any_element()
    } else if messages {
        IconButton::new("search-menu", symbol("menu", 24.), "Menu")
            .disabled(disabled)
            .into_any_element()
    } else {
        symbol("search", 24.).into_any_element()
    };
    let mut bar = SearchField::new(&input)
        .id("playground-search-bar")
        .field_style(SearchFieldStyle::ALL[config.choice("field")])
        .leading(leading)
        .disabled(disabled);
    if config.bool("clear") && !query.is_empty() {
        let clear = set_query.clone();
        bar = bar.trailing(
            IconButton::new("search-clear", symbol("close", 24.), "Clear search")
                .disabled(disabled)
                .on_click(move |_, window, cx| {
                    clear(Value::Text(SharedString::default()), window, cx)
                }),
        );
    }
    if messages && !expanded {
        bar = bar.trailing(
            div()
                .size(rem(30.))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(color(ColorRole::PRIMARY_CONTAINER))
                .text_color(color(ColorRole::ON_PRIMARY_CONTAINER))
                .map(|text| type_role(text, TypeRole::LabelLarge))
                .child("A"),
        );
    }
    let suggestions = (config.option("suggestions") == "places")
        .then(|| {
            matching(&query).map(|(city, country)| {
                let choose_query = set_query.clone();
                let choose_state = set_state.clone();
                SearchSuggestion::new(*city)
                    .supporting_text(*country)
                    .leading(symbol("location_on", 24.))
                    .on_click(move |_, window, cx| {
                        choose_query(Value::Text(SharedString::new_static(city)), window, cx);
                        choose_state(BAR, window, cx);
                    })
            })
        })
        .into_iter()
        .flatten();
    let full_screen = config.option("mode") == "fullscreen";
    let view = SearchView::new("playground-search", bar)
        .layout(if full_screen {
            SearchViewLayout::FullScreen
        } else {
            SearchViewLayout::Docked
        })
        .view_style(if config.option("style") == "divided" {
            SearchViewStyle::Divided
        } else {
            SearchViewStyle::Contained
        })
        .expanded(expanded)
        .expand_on_focus(config.bool("expand_on_focus") && config.option("state") == "bar")
        .collapse_on_blur(config.bool("collapse_on_blur"))
        .on_expanded_change({
            let set_state = set_state.clone();
            move |expanded, window, cx| set_state(if *expanded { VIEW } else { BAR }, window, cx)
        })
        .suggestions(suggestions);
    div()
        .id("search-frame")
        .debug_selector(|| "playground-search".into())
        .map(|frame| {
            if full_screen {
                // A phone-sized area for the view to cover.
                frame
                    .w_full()
                    .max_w(rem(360.))
                    .h(rem(480.))
                    .rounded(rem(16.))
                    .overflow_hidden()
                    .border_1()
                    .border_color(color(ColorRole::OUTLINE_VARIANT))
                    .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
                    .when(!expanded, |frame| frame.p(rem(12.)))
            } else {
                frame
                    .w_full()
                    .max_w(rem(360.))
                    .min_h(rem(56. + 2. + 5. * 64.))
            }
        })
        .child(view)
        .into_any_element()
}

fn code(config: &Config) -> String {
    let disabled = config.bool("disabled");
    let expanded = config.option("state") == "view" && !disabled;
    let messages = config.option("preset") == "Messages";
    let query = config.text("query");
    let leading = if expanded {
        chain(
            format!("IconButton::new(\"search-back\", {}, \"Back\")", symbol_code("arrow_back")),
            [Some(
                "on_click(cx.listener(|this, _: &ClickEvent, _, cx| {\n    this.search_expanded = false;\n    cx.notify();\n}))"
                    .into(),
            )],
        )
    } else if messages {
        chain(
            format!(
                "IconButton::new(\"search-menu\", {}, \"Menu\")",
                symbol_code("menu")
            ),
            [disabled.then(|| "disabled(true)".into())],
        )
    } else {
        format!("{}.icon().component()", symbol_code("search"))
    };
    let clear = (config.bool("clear") && !query.is_empty()).then(|| {
        format!(
            "trailing({})",
            chain(
                format!(
                    "IconButton::new(\"search-clear\", {}, \"Clear search\")",
                    symbol_code("close")
                ),
                [
                    disabled.then(|| "disabled(true)".into()),
                    Some(
                        "on_click(cx.listener(|this, _: &ClickEvent, window, cx| {\n    this.query.update(cx, |query, cx| query.set_value(\"\", window, cx));\n}))"
                            .into(),
                    ),
                ],
            )
        )
    });
    let field = config.choice("field");
    let bar = chain(
        "SearchField::new(&self.query)",
        [
            (field != 0).then(|| {
                format!(
                    "field_style(SearchFieldStyle::{:?})",
                    SearchFieldStyle::ALL[field]
                )
            }),
            Some(format!("leading({leading})")),
            clear,
            // The account's monogram.
            (messages && !expanded).then(|| {
                format!(
                    "trailing({})",
                    chain(
                        "div()",
                        [
                            Some("size(rem(30.))".into()),
                            Some("rounded_full()".into()),
                            Some("flex()".into()),
                            Some("items_center()".into()),
                            Some("justify_center()".into()),
                            Some("bg(color(ColorRole::PRIMARY_CONTAINER))".into()),
                            Some("text_color(color(ColorRole::ON_PRIMARY_CONTAINER))".into()),
                            Some("child(\"A\")".into()),
                        ],
                    )
                )
            }),
            disabled.then(|| "disabled(true)".into()),
        ],
    );
    let suggestions = (config.option("suggestions") == "places").then(|| {
        format!(
            "suggestions(PLACES.iter().map(|&(city, country)| {{\n    {}\n}}))",
            chain(
                "SearchSuggestion::new(city)",
                [
                    Some("supporting_text(country)".into()),
                    Some(format!("leading({}.icon().component())", symbol_code("location_on"))),
                    Some(
                        "on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {\n    this.query.update(cx, |query, cx| query.set_value(city, window, cx));\n    this.search_expanded = false;\n    cx.notify();\n}))"
                            .into(),
                    ),
                ],
            )
            .replace('\n', "\n    ")
        )
    });
    // The placeholder belongs to the query's input state, which the view
    // creates once.
    with_imports(format!(
        "// self.query = cx.new(|cx| InputState::new(window, cx).placeholder({}));\n{}",
        lit(&config.text("placeholder")),
        chain(
            format!(
                "SearchView::new(\n    \"search\",\n    {},\n)",
                bar.replace('\n', "\n    ")
            ),
            [
                (config.option("mode") == "fullscreen")
                    .then(|| "layout(SearchViewLayout::FullScreen)".into()),
                (config.option("style") == "divided")
                    .then(|| "view_style(SearchViewStyle::Divided)".into()),
                Some("expanded(self.search_expanded)".into()),
                config
                    .bool("expand_on_focus")
                    .then(|| "expand_on_focus(true)".into()),
                config
                    .bool("collapse_on_blur")
                    .then(|| "collapse_on_blur(true)".into()),
                Some(
                    "on_expanded_change(cx.listener(|this, expanded: &bool, _, cx| {\n    this.search_expanded = *expanded;\n    cx.notify();\n}))"
                        .into(),
                ),
                suggestions,
            ],
        )
    ))
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn typing_filters_the_places() {
        assert_eq!(matching("").count(), PLACES.len());
        assert_eq!(matching("tok").map(|p| p.0).collect::<Vec<_>>(), ["Tokyo"]);
        assert_eq!(matching("JAPAN").count(), 1);
        assert_eq!(matching("zzz").count(), 0);
    }

    #[gpui::test]
    fn every_scenario_renders_and_typing_filters_then_chooses(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, &PLAYGROUND);
        every_scenario_renders(&state, cx, "playground-search");
        cx.update(|window, cx| state.update(cx, |state, cx| state.reset(window, cx)));
        settle(cx);
        // Focusing the bar expands the view.
        click(cx, "search-bar");
        assert_eq!(config(&state, cx).option("state"), "view");
        // Five rows show before scrolling.
        let row = rect(cx, "search-results").size.height / 5.;
        cx.simulate_input("tok");
        settle(cx);
        assert_eq!(config(&state, cx).text("query"), "tok");
        let results = rect(cx, "search-results");
        assert!(
            (results.size.height - row).abs() < px(1.),
            "only Tokyo matches"
        );
        cx.simulate_click(results.center(), Modifiers::none());
        settle(cx);
        assert_eq!(config(&state, cx).text("query"), "Tokyo");
        assert_eq!(config(&state, cx).option("state"), "bar");

        // The clear button empties the query.
        let bar = rect(cx, "search-bar");
        cx.simulate_click(
            point(bar.right() - px(24.), bar.center().y),
            Modifiers::none(),
        );
        settle(cx);
        assert_eq!(config(&state, cx).text("query"), "");

        // A click outside collapses an expanded view.
        set(&state, cx, "state", VIEW);
        assert!(cx.debug_bounds("search-results").is_some());
        let frame = rect(cx, "playground-search");
        cx.simulate_click(
            point(frame.right() + px(100.), frame.top() + px(10.)),
            Modifiers::none(),
        );
        settle(cx);
        assert_eq!(config(&state, cx).option("state"), "bar");
    }

    #[test]
    fn the_code_follows_the_configuration() {
        let mut config = Config::defaults(&PLAYGROUND);
        let code = super::code(&config);
        assert!(code.starts_with(
            "use gpui_kit_material::icons::Symbol;\nuse gpui_kit::base::input::InputState;\n\n// self.query = cx.new(|cx| InputState::new(window, cx).placeholder(\"Search places\"));\nSearchView::new(\n    \"search\",\n    SearchField::new(&self.query)"
        ), "{code}");
        assert!(code.contains(".leading(Symbol::SEARCH.icon().component())"));
        assert!(code.contains("SearchSuggestion::new(city)"));
        assert!(!code.contains(".view_style("));
        config.set("style", Value::Choice(1));
        config.set("suggestions", Value::Choice(1));
        let code = super::code(&config);
        assert!(code.contains(".view_style(SearchViewStyle::Divided)"));
        assert!(!code.contains(".suggestions("));
    }
}
