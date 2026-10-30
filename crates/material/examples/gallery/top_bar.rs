//! The top app bar inside the window's title bar, as on md3.io: the brand,
//! the top-level sections, a search field that filters the navigation, and
//! the light/dark toggle. A separate view, so the FPS counter beside it and
//! page changes do not render one another.
use super::catalog::{self, TopSection};
use super::spec::type_role;
use super::{Gallery, page::GalleryShared, playground::symbol};
use gpui::{prelude::*, *};
use gpui_base::input::{InputEvent, InputState};
use gpui_kit_material::prelude::*;

pub(crate) struct TopBar {
    gallery: Entity<Gallery>,
    shared: Entity<GalleryShared>,
    pub(crate) search: Entity<InputState>,
    /// What the bar shows of the Gallery: its section and mode.
    shown: (TopSection, ThemeMode),
    _subscriptions: Vec<Subscription>,
}

fn shown(gallery: &Entity<Gallery>, cx: &App) -> (TopSection, ThemeMode) {
    let gallery = gallery.read(cx);
    (
        catalog::CATALOG[gallery.current].group.section(),
        gallery.shared.read(cx).reference_mode,
    )
}

impl TopBar {
    pub(crate) fn new(
        gallery: Entity<Gallery>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search components"));
        let shared = gallery.read(cx).shared.clone();
        let subscriptions = vec![
            cx.subscribe_in(&search, window, |this: &mut Self, search, event, _, cx| {
                match event {
                    InputEvent::Change => {
                        let query = search.read(cx).value();
                        this.gallery
                            .update(cx, |gallery, cx| gallery.set_query(query, cx));
                    }
                    // Enter opens the first match.
                    InputEvent::PressEnter { .. } => {
                        let gallery = this.gallery.clone();
                        gallery.update(cx, |gallery, cx| {
                            let first = gallery.matches().next();
                            if let Some(first) = first {
                                gallery.navigate(first, cx);
                            }
                        });
                    }
                    _ => {}
                }
            }),
            // Only a section or mode change renders the bar.
            cx.observe(&gallery, |this: &mut Self, gallery, cx| {
                let now = shown(&gallery, cx);
                if now != this.shown {
                    this.shown = now;
                    cx.notify();
                }
            }),
            cx.observe(&shared, |this: &mut Self, _, cx| {
                let now = shown(&this.gallery, cx);
                if now != this.shown {
                    this.shown = now;
                    cx.notify();
                }
            }),
        ];
        let shown = shown(&gallery, cx);
        Self {
            gallery,
            shared,
            search,
            shown,
            _subscriptions: subscriptions,
        }
    }
}

impl Render for TopBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (section, mode) = self.shown;
        let dark = mode == ThemeMode::Dark;
        div()
            .id("top-bar")
            .size_full()
            .flex()
            .items_center()
            .gap(rem(8.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(rem(10.))
                    .pr(rem(16.))
                    .child(brand_mark())
                    .child(
                        accessible_text("brand", "material")
                            .map(|text| type_role(text, TypeRole::TitleLarge))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(color(ColorRole::ON_SURFACE)),
                    ),
            )
            .child(
                div()
                    .id("sections")
                    .role(Role::Navigation)
                    .aria_label("Sections")
                    .flex()
                    .items_center()
                    .gap(rem(4.))
                    .children(TopSection::ALL.into_iter().map(|item| {
                        let selected = item == section;
                        title_bar_control(
                            MaterialButton::new(item.id())
                                .variant(if selected {
                                    ButtonVariant::Tonal
                                } else {
                                    ButtonVariant::Text
                                })
                                .button_size(ButtonSize::ExtraSmall)
                                .debug_selector(move || format!("top-section-{}", item.id()))
                                .child(item.title())
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.gallery.update(cx, |gallery, cx| {
                                        gallery.navigate(catalog::first_of(item), cx)
                                    });
                                })),
                        )
                    })),
            )
            .child(div().flex_1())
            .child(title_bar_control(
                div().w(rem(240.)).child(
                    SearchField::new(&self.search)
                        .id("gallery-search")
                        .leading(symbol("search", 20.))
                        .h(rem(40.)),
                ),
            ))
            .child(title_bar_control(
                div().debug_selector(|| "mode-toggle".into()).child(
                    IconButton::new(
                        "mode-toggle",
                        symbol(if dark { "light_mode" } else { "dark_mode" }, 24.),
                        if dark {
                            "Use light theme"
                        } else {
                            "Use dark theme"
                        },
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.shared.update(cx, |shared, cx| {
                            shared.reference_mode = if dark {
                                ThemeMode::Light
                            } else {
                                ThemeMode::Dark
                            };
                            shared.reference_theme.get_or_insert(ThemeId::Baseline);
                            cx.notify();
                        })
                    })),
                ),
            ))
    }
}

/// The brand mark: an on-primary "m" in a primary circle.
fn brand_mark() -> impl IntoElement {
    div()
        .size(rem(28.))
        .flex_shrink_0()
        .rounded_full()
        .bg(color(ColorRole::PRIMARY))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .map(|text| type_role(text, TypeRole::TitleMedium))
                .font_weight(FontWeight::BOLD)
                .text_color(color(ColorRole::ON_PRIMARY))
                .child("m"),
        )
}
