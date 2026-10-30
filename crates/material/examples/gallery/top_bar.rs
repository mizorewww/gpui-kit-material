//! The top app bar inside the window's title bar, as on md3.io: the brand,
//! the top-level sections, a search field that filters the navigation, and
//! the light/dark toggle. A separate view, so the FPS counter beside it and
//! page changes do not render one another.
//!
//! In the compact layout a menu button opens the navigation drawer, which
//! holds the sections too, and the search field collapses to a button that
//! trades the brand for the field.
use super::catalog::{self, TopSection};
use super::spec::type_role;
use super::{
    Gallery,
    page::GalleryShared,
    playground::symbol,
    shell::{ShellLayout, shell_width},
    spec::RemScaled,
};
use gpui::{prelude::*, *};
use gpui_base::input::{InputEvent, InputState};
use gpui_kit_material::prelude::*;
use gpui_kit_material::tokens::typography::BASE_PX;

/// Below these window widths (in design px) the brand's wordmark gives way:
/// to the sections and search in the medium layout, to the bar's buttons in
/// the compact one.
const MEDIUM_WORDMARK: f32 = 1040.;
const COMPACT_WORDMARK: f32 = 600.;
/// The bar is a fixed height, so its controls stop following the text scale
/// where a 40dp control would no longer fit in it.
const MAX_CHROME_SCALE: f32 = 1.25;

/// The rem the title bar's controls lay out at.
pub(crate) fn chrome_rem(window: &Window) -> Pixels {
    window.rem_size().min(px(BASE_PX * MAX_CHROME_SCALE))
}

/// The search field's width, and the least the medium layout's field shrinks to.
const SEARCH_WIDTH: f32 = 240.;
const SEARCH_MIN_WIDTH: f32 = 96.;

pub(crate) struct TopBar {
    gallery: Entity<Gallery>,
    shared: Entity<GalleryShared>,
    pub(crate) search: Entity<InputState>,
    /// What the bar shows of the Gallery: its section and mode.
    shown: (TopSection, ThemeMode),
    /// Whether the compact bar has traded its brand for the search field.
    search_open: bool,
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
            search_open: false,
            _subscriptions: subscriptions,
        }
    }
}

impl Render for TopBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (section, mode) = self.shown;
        let dark = mode == ThemeMode::Dark;
        let compact = ShellLayout::of(window) == ShellLayout::Compact;
        let width = shell_width(window);
        let wordmark = width
            >= if compact {
                COMPACT_WORDMARK
            } else {
                MEDIUM_WORDMARK
            };
        let searching = compact && self.search_open;
        let field = title_bar_control(
            div()
                .debug_selector(|| "top-search".into())
                .when(compact, |field| field.flex_1().min_w_0())
                .when(!compact, |field| {
                    field
                        .w(rem(SEARCH_WIDTH))
                        .flex_shrink(1.)
                        .min_w(rem(SEARCH_MIN_WIDTH))
                })
                .child(
                    SearchField::new(&self.search)
                        .id("gallery-search")
                        .leading(symbol("search", 20.))
                        .h(rem(40.)),
                ),
        );
        let rem_size = chrome_rem(window);
        let bar = div()
            .id("top-bar")
            .size_full()
            .flex()
            .items_center()
            .gap(rem(if compact { 4. } else { 8. }))
            // The field takes the whole bar on a phone: its placeholder is
            // cut off beside the menu and mode buttons.
            .when(compact && !searching, |bar| {
                bar.child(title_bar_control(
                    div().debug_selector(|| "top-menu".into()).child(
                        IconButton::new("top-menu", symbol("menu", 24.), "Open navigation")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.gallery.update(cx, |gallery, cx| {
                                    let open = gallery.drawer_open;
                                    gallery.set_drawer_open(!open, cx);
                                })
                            })),
                    ),
                ))
            })
            .when(!searching, |bar| {
                bar.child(
                    div()
                        .flex()
                        .flex_shrink_0()
                        .items_center()
                        .gap(rem(10.))
                        .when(!compact, |brand| brand.pr(rem(16.)))
                        .child(brand_mark())
                        .when(wordmark, |brand| {
                            brand.child(
                                accessible_text("brand", "material")
                                    .map(|text| type_role(text, TypeRole::TitleLarge))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(color(ColorRole::ON_SURFACE)),
                            )
                        }),
                )
            })
            .when(!compact, |bar| bar.child(self.render_sections(section, cx)))
            .when(!searching, |bar| bar.child(div().flex_1()))
            .when(!compact || searching, |bar| bar.child(field))
            .when(compact, |bar| {
                bar.child(title_bar_control(
                    div().debug_selector(|| "top-search-toggle".into()).child(
                        IconButton::new(
                            "top-search-toggle",
                            symbol(if searching { "close" } else { "search" }, 24.),
                            if searching {
                                "Close search"
                            } else {
                                "Search components"
                            },
                        )
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.search_open = !this.search_open;
                            if this.search_open {
                                this.search
                                    .update(cx, |search, cx| search.focus(window, cx));
                            } else {
                                // Closing the search ends the filter as well.
                                this.search
                                    .update(cx, |search, cx| search.set_value("", window, cx));
                                let gallery = this.gallery.clone();
                                gallery.update(cx, |gallery, cx| {
                                    gallery.set_query(SharedString::default(), cx);
                                    gallery.set_drawer_open(false, cx);
                                });
                            }
                            cx.notify();
                        })),
                    ),
                ))
            })
            .when(!searching, |bar| {
                bar.child(title_bar_control(
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
            });
        RemScaled::new(rem_size, bar.into_any_element())
    }
}

impl TopBar {
    /// The top-level sections, which the compact layout's drawer holds instead.
    fn render_sections(&self, section: TopSection, cx: &mut Context<Self>) -> impl IntoElement {
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
            }))
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
