//! The top app bar inside the window's title bar, as on md3.io: the brand,
//! the top-level sections, a search field that filters the navigation, and
//! the light/dark toggle. A separate view, so the FPS counter beside it and
//! page changes do not render one another.
//!
//! In the compact layout a menu button opens the navigation drawer, which
//! holds the sections too, and the search field collapses to a button that
//! trades the brand for the field. Its icon controls keep their size at larger
//! text scales, having no text to scale, and where a platform's caption buttons
//! leave too little room the brand gives way, then the mode toggle, which the
//! settings dialog holds too: see [`CompactBar`].
use super::catalog::{self, TopSection};
use super::spec::{RemScaled, type_role};
use super::{
    Gallery,
    page::GalleryShared,
    playground::symbol,
    shell::{ShellLayout, shell_width},
};
use gpui::{prelude::*, *};
use gpui_base::input::{InputEvent, InputState};
use gpui_kit_material::prelude::*;

/// Below these window widths (in design px) the brand's wordmark gives way:
/// to the sections and search in the medium layout, to the bar's buttons in
/// the compact one.
const MEDIUM_WORDMARK: f32 = 1040.;
const COMPACT_WORDMARK: f32 = 600.;
/// The search field's width, and the least the medium layout's field shrinks to.
const SEARCH_WIDTH: f32 = 240.;
const SEARCH_MIN_WIDTH: f32 = 96.;
/// The compact bar's icon buttons and brand, and the gap between its controls.
const COMPACT_CONTROL: f32 = 40.;
const COMPACT_BRAND: f32 = 28.;
const COMPACT_GAP: f32 = 4.;

/// What fits a compact bar beside the menu, search and settings buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CompactBar {
    pub(crate) brand: bool,
    pub(crate) mode: bool,
}

impl CompactBar {
    /// What fits in `room` px, the width the title bar leaves for its children
    /// at the compact controls' size.
    fn fitting(room: f32) -> Self {
        // The menu, search and settings buttons, with their gaps.
        let required = 3. * COMPACT_CONTROL + 2. * COMPACT_GAP;
        let mode = required + COMPACT_CONTROL + COMPACT_GAP;
        // The spacer between the brand and the search button adds a gap too.
        let brand = mode + COMPACT_BRAND + 2. * COMPACT_GAP;
        Self {
            brand: room >= brand,
            mode: room >= mode,
        }
    }

    /// What fits this window's title bar, with its caption style forced or
    /// detected.
    pub(crate) fn of(height: Pixels, caption: Option<Caption>, window: &Window) -> Self {
        let insets = window.insets().effective();
        let width = window.viewport_size().width - insets.left - insets.right;
        Self::fitting(f32::from(title_bar_room(width, height, caption, window)))
    }
}

/// An icon control of the compact bar, at its size for the 100% text scale at
/// any larger one. An icon has no text to scale, and a phone at 200% has no
/// room for controls twice the size.
pub(crate) fn compact_control(child: impl IntoElement, window: &Window) -> AnyElement {
    let rem = window.rem_size().min(px(BASE_PX));
    RemScaled::new(rem, child.into_any_element()).into_any_element()
}

/// Switch the Gallery's theme mode.
pub(crate) fn set_dark(shared: &Entity<GalleryShared>, dark: bool, cx: &mut App) {
    shared.update(cx, |shared, cx| {
        shared.reference_mode = if dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        shared.reference_theme.get_or_insert(ThemeId::Baseline);
        cx.notify();
    })
}

pub(crate) struct TopBar {
    gallery: Entity<Gallery>,
    shared: Entity<GalleryShared>,
    pub(crate) search: Entity<InputState>,
    /// What the bar shows of the Gallery: its section and mode.
    shown: (TopSection, ThemeMode),
    /// Whether the compact bar has traded its brand for the search field.
    search_open: bool,
    /// A caption style forced by a capture or test, which the bar leaves room for.
    caption: Option<Caption>,
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
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search"));
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
            caption: None,
            _subscriptions: subscriptions,
        }
    }

    /// Leave room for a forced caption style's buttons.
    pub(crate) fn caption(mut self, caption: Option<Caption>) -> Self {
        self.caption = caption;
        self
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
        let fit = CompactBar::of(super::title_bar_height(window), self.caption, window);
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
        div()
            .id("top-bar")
            .size_full()
            .flex()
            .items_center()
            .when(compact, |bar| bar.gap(px(COMPACT_GAP)))
            .when(!compact, |bar| bar.gap(rem(8.)))
            // The field takes the whole bar on a phone: its placeholder is
            // cut off beside the menu and mode buttons.
            .when(compact && !searching, |bar| {
                bar.child(compact_control(
                    title_bar_control(
                        div().debug_selector(|| "top-menu".into()).child(
                            IconButton::new("top-menu", symbol("menu", 24.), "Open navigation")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.gallery.update(cx, |gallery, cx| {
                                        let open = gallery.drawer_open;
                                        gallery.set_drawer_open(!open, cx);
                                    })
                                })),
                        ),
                    ),
                    window,
                ))
            })
            .when(!searching && (!compact || fit.brand), |bar| {
                let brand = div()
                    .debug_selector(|| "top-brand".into())
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
                    });
                if compact {
                    bar.child(compact_control(brand, window))
                } else {
                    bar.child(brand)
                }
            })
            .when(!compact, |bar| bar.child(self.render_sections(section, cx)))
            .when(!searching, |bar| bar.child(div().flex_1()))
            .when(!compact || searching, |bar| bar.child(field))
            .when(compact, |bar| {
                bar.child(compact_control(
                    title_bar_control(
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
                            .on_click(cx.listener(
                                |this, _, window, cx| {
                                    this.search_open = !this.search_open;
                                    if this.search_open {
                                        this.search
                                            .update(cx, |search, cx| search.focus(window, cx));
                                    } else {
                                        // Closing the search ends the filter as well.
                                        this.search.update(cx, |search, cx| {
                                            search.set_value("", window, cx)
                                        });
                                        let gallery = this.gallery.clone();
                                        gallery.update(cx, |gallery, cx| {
                                            gallery.set_query(SharedString::default(), cx);
                                            gallery.set_drawer_open(false, cx);
                                        });
                                    }
                                    cx.notify();
                                },
                            )),
                        ),
                    ),
                    window,
                ))
            })
            .when(!searching && (!compact || fit.mode), |bar| {
                let toggle = title_bar_control(
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
                        .on_click(
                            cx.listener(move |this, _, _, cx| set_dark(&this.shared, !dark, cx)),
                        ),
                    ),
                );
                if compact {
                    bar.child(compact_control(toggle, window))
                } else {
                    bar.child(toggle)
                }
            })
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
