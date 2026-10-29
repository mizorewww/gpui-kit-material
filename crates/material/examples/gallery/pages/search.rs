//! Search: the kit's search bar states, docked and full-screen search view
//! layouts, and live docked and full-screen search views that expand on
//! focus and filter their suggestions.
use super::prelude::*;
use super::text_fields::symbol;
use gpui_base::input::{InputEvent, InputState};

const SUGGESTIONS: [(&str, &str); 6] = [
    ("Buttons", "Actions people can take"),
    ("Cards", "Content and actions about one subject"),
    ("Chips", "Compact filters and choices"),
    ("Menus", "Choices on a temporary surface"),
    ("Search", "Find relevant information"),
    ("Text fields", "Text entry in forms and dialogs"),
];

pub struct Page {
    pub(crate) search: Entity<InputState>,
    full_screen: Entity<InputState>,
    docked_open: bool,
    full_open: bool,
    picked: Option<&'static str>,
}

/// The kit's avatar: a 30dp primary circle with a monogram.
fn avatar(letter: &'static str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .size(rem(30.))
        .rounded_full()
        .bg(color(ColorRole::PRIMARY))
        .text_color(color(ColorRole::ON_PRIMARY))
        .text_size(rem(14.))
        .font_weight(FontWeight::MEDIUM)
        .child(letter)
}

/// A 40dp icon button with Base activation, keyboard focus and a hover layer.
fn icon_button(
    id: impl Into<ElementId>,
    label: &'static str,
    icon: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let ink = color(ColorRole::ON_SURFACE);
    gpui_base::Button::new(id)
        .accessibility_label(label)
        .flex()
        .items_center()
        .justify_center()
        .size(rem(40.))
        .rounded_full()
        .cursor_pointer()
        .hover(move |s| s.bg(ink.opacity(0.08)))
        .on_click(on_click)
        .child(symbol(icon, false))
}

fn suggestion_rows(query: &str) -> Vec<(&'static str, &'static str)> {
    let query = query.to_lowercase();
    SUGGESTIONS
        .into_iter()
        .filter(|(label, _)| label.to_lowercase().contains(&query))
        .collect()
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "search",
        title: "Search",
        description: "Search lets people enter a keyword or phrase to get relevant information.",
        guideline: Some("https://m3.material.io/components/search/overview"),
    };

    fn new(_: PageCx, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search components"));
        let full_screen = cx.new(|cx| InputState::new(window, cx).placeholder("Search components"));
        // The views open when their bar takes focus and refilter as people type.
        cx.subscribe_in(&search, window, |this, _, event: &InputEvent, _, cx| {
            if matches!(event, InputEvent::Focus) {
                this.docked_open = true;
            }
            cx.notify();
        })
        .detach();
        cx.subscribe_in(
            &full_screen,
            window,
            |this, _, event: &InputEvent, _, cx| {
                if matches!(event, InputEvent::Focus) {
                    this.full_open = true;
                }
                cx.notify();
            },
        )
        .detach();
        Self {
            search,
            full_screen,
            docked_open: false,
            full_open: false,
            picked: None,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        5
    }

    fn interactive(&self, row: usize) -> bool {
        row >= 3
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match row {
            0 => bar_board(),
            1 => layout_board(SearchViewLayout::Docked),
            2 => layout_board(SearchViewLayout::FullScreen),
            3 => self.live_docked(cx),
            _ => self.live_full_screen(cx),
        }
    }
}

impl Page {
    fn suggestions(
        &self,
        query: &str,
        full: bool,
        cx: &mut Context<Self>,
    ) -> Vec<SearchSuggestion> {
        suggestion_rows(query)
            .into_iter()
            .map(|(label, supporting)| {
                SearchSuggestion::new(label)
                    .supporting_text(supporting)
                    .leading(symbol("category", false))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.picked = Some(label);
                        if full {
                            this.full_open = false;
                        } else {
                            this.docked_open = false;
                        }
                        window.blur(cx);
                        cx.notify();
                    }))
            })
            .collect()
    }

    fn live_docked(&self, cx: &mut Context<Self>) -> AnyElement {
        let query = self.search.read(cx).value();
        let open = self.docked_open;
        let leading = if open {
            icon_button(
                "docked-back",
                "Close search",
                "arrow_back",
                cx.listener(|this, _, window, cx| {
                    this.docked_open = false;
                    window.blur(cx);
                    cx.notify();
                }),
            )
            .into_any_element()
        } else {
            symbol("search", false).into_any_element()
        };
        let state = self.search.clone();
        let mut bar = SearchField::new(&self.search).leading(leading);
        if !query.is_empty() {
            bar = bar.trailing(icon_button(
                "docked-clear",
                "Clear query",
                "close",
                move |_, window, cx| {
                    state.update(cx, |state, cx| {
                        state.set_value("", window, cx);
                        state.focus(window, cx);
                    })
                },
            ));
        }
        let bar = bar.trailing(avatar("A"));
        let picked = self
            .picked
            .map(|label| format!("Last chosen: {label}"))
            .unwrap_or_else(|| "Focus the bar to open the view".to_owned());
        spec::demo(
            "live-docked-search",
            "Docked search view",
            div()
                .flex()
                .flex_col()
                .gap(rem(12.))
                .h(rem(56. + 2. + 5. * 64. + 32.))
                .child(accessible_text("docked-status", picked))
                .child(
                    SearchView::new("live-docked", bar)
                        .expanded(open)
                        .suggestions(self.suggestions(&query, false, cx)),
                ),
        )
        .into_any_element()
    }

    fn live_full_screen(&self, cx: &mut Context<Self>) -> AnyElement {
        let query = self.full_screen.read(cx).value();
        let open = self.full_open;
        let leading = if open {
            icon_button(
                "full-back",
                "Close search",
                "arrow_back",
                cx.listener(|this, _, window, cx| {
                    this.full_open = false;
                    window.blur(cx);
                    cx.notify();
                }),
            )
            .into_any_element()
        } else {
            symbol("menu", false).into_any_element()
        };
        let bar = SearchField::new(&self.full_screen)
            .leading(leading)
            .trailing(symbol("mic", false));
        spec::demo(
            "live-full-screen-search",
            "Full-screen search view",
            div()
                .w(rem(412.))
                .h(rem(480.))
                .rounded(rem(28.))
                .overflow_hidden()
                .border_1()
                .border_color(color(ColorRole::OUTLINE_VARIANT))
                .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
                .pt(rem(12.))
                .child(
                    SearchView::new("live-full", bar)
                        .layout(SearchViewLayout::FullScreen)
                        .expanded(open)
                        .suggestions(self.suggestions(&query, true, cx)),
                ),
        )
        .into_any_element()
    }
}

fn bar_board() -> AnyElement {
    let states = [
        InteractionPreview::Enabled,
        InteractionPreview::Hovered,
        InteractionPreview::Pressed,
    ];
    let cells = [false, true].into_iter().flat_map(move |with_avatar| {
        states.into_iter().enumerate().map(move |(ix, state)| {
            let mut bar = SearchField::specimen(
                ("search-bar-specimen", ix + if with_avatar { 3 } else { 0 }),
                "Hinted search text",
                true,
            )
            .leading(symbol("menu", false))
            .trailing(symbol("search", false))
            .preview_state(state)
            .w(rem(360.));
            if with_avatar {
                bar = bar.trailing(avatar("A"));
            }
            bar
        })
    });
    scrolling_board(
        "search-bar-board",
        88. + 3. * 360. + 2. * 16. + 2. * 20. + 2.,
        board("search-bars")
            .caption("Search bar")
            .columns(3)
            .column_width(360.)
            .column_labels(states.map(InteractionPreview::name))
            .row_labels(["Icons", "Avatar"])
            .cells(cells),
    )
}

fn layout_board(layout: SearchViewLayout) -> AnyElement {
    let full = layout == SearchViewLayout::FullScreen;
    let width = if full { 412. } else { 360. };
    let cells = [true, false]
        .into_iter()
        .enumerate()
        .map(move |(ix, input)| {
            let mut bar = SearchField::specimen(
                ("search-layout-bar", ix + if full { 2 } else { 0 }),
                if input {
                    "Input text"
                } else {
                    "Hinted search text"
                },
                !input,
            )
            .leading(symbol("arrow_back", false));
            if input {
                bar = bar.trailing(symbol("close", false));
            }
            bar = bar.trailing(symbol("mic", false));
            let view = SearchView::new(("search-layout", ix + if full { 2 } else { 0 }), bar)
                .layout(layout)
                .expanded(true)
                .max_visible(3)
                .suggestions((0..3).map(|_| {
                    SearchSuggestion::new("Label text")
                        .supporting_text("Supporting line text, lorem ipsum dolor")
                        .leading(symbol("category", false))
                }));
            div()
                .w(rem(width))
                .when(full, |this| this.h(rem(56. + 3. * 64.)))
                .child(view)
        });
    scrolling_board(
        if full {
            "search-full-board"
        } else {
            "search-docked-board"
        },
        2. * width + 16. + 2. * 20. + 2.,
        board(if full {
            "search-full-layouts"
        } else {
            "search-docked-layouts"
        })
        .caption(if full {
            "Search full-screen layout"
        } else {
            "Search docked layout"
        })
        .columns(2)
        .column_width(width)
        .column_labels(["Input text", "Supporting text"])
        .cells(cells),
    )
}

/// A board at its natural width, scrolling horizontally in narrow windows.
fn scrolling_board(id: &'static str, width: f32, board: impl IntoElement) -> AnyElement {
    div()
        .id(id)
        .w_full()
        .overflow_x_scroll()
        .child(div().w(rem(width)).child(board))
        .into_any_element()
}
