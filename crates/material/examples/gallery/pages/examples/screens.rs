//! The example apps' screen content, composed from crate components. Each
//! body renders at compact (phone) or expanded (browser) size from the
//! kit's frames of the same name.
use super::super::prelude::*;
use super::parts::{
    Bubble, art, bar_button, bubble, eid, feature_item, filled, icon, line, media_card, nav_icon,
    scroll_row, section_header, text, tile,
};
use super::{Body, Ctx, NAV_VALUES, Page};
use gpui_kit_material::components::{IconButtonWidth, linear_progress};
use std::sync::OnceLock;

const SECTION: &str = "Section title";
const SHOW_ALL: &str = "Show all";
const SUPPORTING: &str = "Supporting line text lorem ipsum dolor sit amet, consectetur";
const DESCRIPTION: &str = "Description duis aute irure dolor in reprehenderit in voluptate velit.";
const MORE_LIKE: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
const PARAGRAPH: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.\n\nDuis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
const PLACE: &str =
    "Category • $$ • 1.2 miles away\nSupporting line text lorem ipsum dolor sit amet, consectetur";
const UPDATED: [&str; 3] = ["Updated today", "Updated yesterday", "Updated 2 days ago"];
const SEGMENTS: [&str; 2] = ["0", "1"];

/// The three destinations every navigation bar and rail shows: the kit's
/// "Label" with its star.
pub fn nav_items() -> &'static [NavItem] {
    static ITEMS: OnceLock<Vec<NavItem>> = OnceLock::new();
    ITEMS.get_or_init(|| {
        NAV_VALUES
            .iter()
            .map(|value| NavItem::new(*value, "Label", nav_icon("stars")))
            .collect()
    })
}

/// The pane color of expanded content, which bars and list rows sit on.
fn pane() -> ColorRole {
    ColorRole::SURFACE_CONTAINER_LOWEST
}

/// A top app bar on the screen's pane: surface when compact, the
/// expanded content pane's surface-container-lowest when expanded.
fn app_bar(
    c: Ctx,
    name: &'static str,
    variant: TopAppBarVariant,
    title: &'static str,
) -> TopAppBar {
    TopAppBar::new(eid(name, c.k, 0), variant)
        .title(title)
        .when(c.web, |bar| bar.container(pane()))
}

fn back(c: Ctx, name: &'static str) -> ToolbarButton {
    bar_button(eid(name, c.k, 1), "arrow_back", "Back", true)
}

/// The kit's hero carousel: one large item and a small one on a phone;
/// on the expanded pane the kit's tablet hero shows several 316dp items
/// stepping down to a small one, which is the multi-browse arrangement.
fn hero_carousel(c: Ctx, name: &'static str) -> MaterialCarousel {
    MaterialCarousel::new(eid(name, c.k, 0), "Featured")
        .layout(if c.web {
            CarouselLayout::MultiBrowse
        } else {
            CarouselLayout::Hero
        })
        .large_item_width(316.)
        .item_height(205.)
        .width(c.width)
        .items((0..6).map(|_| CarouselItem::new().content(art(96.).size_full())))
}

/// White text over the scrim of a hero image, as the kit paints it.
fn on_image(role: TypeRole, content: &'static str) -> Div {
    spec::type_role(div(), role)
        .text_color(gpui::white())
        .truncate()
        .child(content)
}

/// The scrim gradient under a hero image's text.
fn scrim(from: f32, to: f32) -> Background {
    linear_gradient(
        180.,
        linear_color_stop(color(ColorRole::SCRIM).opacity(from), 0.),
        linear_color_stop(color(ColorRole::SCRIM).opacity(to), 1.),
    )
}

impl Page {
    pub(super) fn body(
        &mut self,
        body: Body,
        c: Ctx,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match body {
            Body::Home => self.home(c),
            Body::Upcoming => self.upcoming(c, cx),
            Body::Detailed => self.detailed(c, window, cx),
            Body::Reviews => self.reviews(c, cx),
            Body::Library => self.library(c, cx),
            Body::Gallery => self.gallery(c),
            Body::Messaging => self.messaging(c, cx),
        }
    }

    fn home(&self, c: Ctx) -> AnyElement {
        let (k, inset) = (c.k, c.inset());
        let gap = if c.web { 24. } else { 8. };
        let bar = app_bar(
            c,
            "home-bar",
            if c.web {
                TopAppBarVariant::LargeFlexible
            } else {
                TopAppBarVariant::MediumFlexible
            },
            "Label",
        )
        .navigation_icon(bar_button(
            eid("home-account", k, 0),
            "account_circle",
            "Account",
            true,
        ))
        .action(bar_button(
            eid("home-notifications", k, 0),
            "notifications",
            "Notifications",
            false,
        ))
        .action(bar_button(
            eid("home-settings", k, 0),
            "settings",
            "Settings",
            false,
        ));
        let header = |i: usize| section_header(eid("home-header", k, i), SECTION, SHOW_ALL, inset);
        let artists = div().pb(rem(16.)).child(header(0)).child(
            scroll_row(eid("home-artists", k, 0), inset, gap)
                .children((0..if c.web { 13 } else { 8 }).map(|_| tile(true, "Label"))),
        );
        let hero = div()
            .child(header(1))
            .child(hero_carousel(c, "home-hero"))
            .child(
                div().h(rem(48.)).px(rem(inset)).flex().child(
                    div()
                        .w(rem(316.))
                        .flex()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .child(line(TypeRole::BodyLarge, ColorRole::ON_SURFACE, "Artist"))
                                .child(line(
                                    TypeRole::BodyMedium,
                                    ColorRole::ON_SURFACE_VARIANT,
                                    "Title",
                                )),
                        )
                        .child(
                            IconButton::new(eid("home-play", k, 0), filled("play_arrow"), "Play")
                                .variant(IconButtonVariant::Tonal),
                        ),
                ),
            );
        let albums = div().child(header(2)).child(
            scroll_row(eid("home-albums", k, 0), inset, gap)
                .py(rem(8.))
                .children((0..if c.web { 13 } else { 8 }).map(|_| tile(false, "Label"))),
        );
        let cards = div().pb(rem(16.)).child(header(3)).child(
            scroll_row(eid("home-cards", k, 0), inset, gap)
                .children((0..9).map(|_| media_card("Artist", "Song"))),
        );
        div()
            .flex()
            .flex_col()
            .child(bar)
            .child(artists)
            .child(hero)
            .child(albums)
            .child(cards)
            .when(!c.web, |home| {
                home.child(
                    div()
                        .px(rem(16.))
                        .pb(rem(8.))
                        .child(section_header(
                            eid("home-header", k, 4),
                            SECTION,
                            SHOW_ALL,
                            0.,
                        ))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(rem(8.))
                                .children((0..2).map(|i| {
                                    feature_item(
                                        eid("home-feature", k, i),
                                        "Headline",
                                        DESCRIPTION,
                                        "Today • 23 min",
                                        "Play",
                                    )
                                })),
                        ),
                )
                .child(
                    div()
                        .pt(rem(24.))
                        .pb(rem(16.))
                        .flex()
                        .flex_col()
                        .gap(rem(16.))
                        .child(
                            div()
                                .h(rem(48.))
                                .px(rem(16.))
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(rem(12.))
                                        .child(art(24.).size(rem(40.)).rounded_full())
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .child(text(
                                                    TypeRole::BodySmall,
                                                    ColorRole::ON_SURFACE_VARIANT,
                                                    "More like",
                                                ))
                                                .child(text(
                                                    TypeRole::TitleMedium,
                                                    ColorRole::ON_SURFACE,
                                                    "Title",
                                                )),
                                        ),
                                )
                                .child(IconButton::new(
                                    eid("home-more-like", k, 0),
                                    icon("arrow_forward"),
                                    SHOW_ALL,
                                )),
                        )
                        .child(
                            scroll_row(eid("home-more", k, 0), 16., 8.)
                                .children((0..8).map(|_| tile(false, "Label"))),
                        )
                        .child(
                            text(
                                TypeRole::BodyMedium,
                                ColorRole::ON_SURFACE_VARIANT,
                                MORE_LIKE,
                            )
                            .px(rem(16.))
                            .line_clamp(2),
                        ),
                )
            })
            .into_any_element()
    }

    fn upcoming(&self, c: Ctx, cx: &mut Context<Self>) -> AnyElement {
        let (k, inset) = (c.k, c.inset());
        let selected = self.selected[k];
        let bar = app_bar(c, "upcoming-bar", TopAppBarVariant::MediumFlexible, "Label")
            .navigation_icon(back(c, "upcoming-back"))
            .action(bar_button(
                eid("upcoming-attach", k, 0),
                "attach_file",
                "Attach",
                false,
            ))
            .action(bar_button(
                eid("upcoming-today", k, 0),
                "today",
                "Calendar",
                false,
            ))
            .action(bar_button(
                eid("upcoming-more", k, 0),
                "more_vert",
                "More options",
                false,
            ));
        div()
            .flex()
            .flex_col()
            .child(bar)
            .child(hero_carousel(c, "upcoming-hero"))
            .child(
                div().px(rem(inset)).flex().justify_end().child(
                    material_button(eid("upcoming-show-all", k, 0), ButtonVariant::Text)
                        .icon(filled("stars"))
                        .child(SHOW_ALL),
                ),
            )
            .child(section_header(
                eid("upcoming-header", k, 0),
                if c.web { "Featured" } else { SECTION },
                SHOW_ALL,
                inset,
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .pb(rem(16.))
                    .when(c.web, |list| list.px(rem(8.)))
                    .children((0..5).map(|i| {
                        ListItem::new(eid("upcoming-item", k, i), "List item")
                            .supporting(SUPPORTING)
                            .three_line(true)
                            .leading(ListLeading::Image(None))
                            .trailing(ListTrailing::Icon(icon("more_vert")))
                            .selected(selected == Some(i))
                            .when(c.web, |item| item.container(pane()))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.selected[k] = (this.selected[k] != Some(i)).then_some(i);
                                cx.notify();
                            }))
                    })),
            )
            .into_any_element()
    }

    fn detailed(&self, c: Ctx, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let (k, inset) = (c.k, c.inset());
        let bar = app_bar(c, "detail-bar", TopAppBarVariant::Small, "Label")
            .navigation_icon(back(c, "detail-back"))
            .action(bar_button(
                eid("detail-favorite", k, 0),
                "stars",
                "Favorite",
                false,
            ))
            .action(bar_button(
                eid("detail-save", k, 0),
                "bookmark",
                "Save",
                false,
            ))
            .action(bar_button(
                eid("detail-more", k, 0),
                "more_vert",
                "More options",
                false,
            ));
        let published = || {
            text(
                TypeRole::LabelSmall,
                ColorRole::ON_SURFACE_VARIANT,
                "Published date",
            )
        };
        let paragraph = || text(TypeRole::BodyMedium, ColorRole::ON_SURFACE, PARAGRAPH);
        let header = div()
            .flex()
            .gap(rem(24.))
            .px(rem(inset))
            .py(rem(8.))
            .child(
                art(if c.web { 112. } else { 72. })
                    .size(rem(if c.web { 216. } else { 136. }))
                    .rounded(rem(28.)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(rem(if c.web { 16. } else { 24. }))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(rem(4.))
                            .child(text(
                                TypeRole::HeadlineSmall,
                                ColorRole::ON_SURFACE,
                                "Headline",
                            ))
                            .child(text(
                                TypeRole::TitleMedium,
                                ColorRole::ON_SURFACE_VARIANT,
                                "supporting text",
                            ))
                            .when(c.web, |column| {
                                column.child(published()).child(paragraph().line_clamp(5))
                            }),
                    )
                    .child(
                        material_button(eid("detail-download", k, 0), ButtonVariant::Filled)
                            .icon(filled("stars"))
                            .child("Download"),
                    ),
            );
        let cards = div()
            .pb(rem(32.))
            .child(
                div()
                    .h(rem(if c.web { 64. } else { 48. }))
                    .px(rem(inset))
                    .flex()
                    .items_center()
                    .child(text(
                        TypeRole::HeadlineSmall,
                        ColorRole::ON_SURFACE,
                        SECTION,
                    ))
                    .child(IconButton::new(
                        eid("detail-header", k, 0),
                        icon("arrow_forward"),
                        SHOW_ALL,
                    )),
            )
            .child(
                div()
                    .px(rem(inset))
                    .flex()
                    .flex_col()
                    .gap(rem(16.))
                    .children((0..3).map(|i| {
                        feature_item(
                            eid("detail-feature", k, i),
                            "Title",
                            DESCRIPTION,
                            "Today • 23 min",
                            "Play",
                        )
                    })),
            );
        let content = div()
            .flex()
            .flex_col()
            .child(bar)
            .child(header)
            .when(!c.web, |content| {
                content.child(
                    div()
                        .px(rem(16.))
                        .py(rem(8.))
                        .flex()
                        .flex_col()
                        .gap(rem(8.))
                        .child(published())
                        .child(paragraph()),
                )
            })
            .child(cards);
        if c.web {
            // The expanded player floats over the pane (`pane_overlay`).
            content.pb(rem(68.)).into_any_element()
        } else {
            content.child(self.player(c, window, cx)).into_any_element()
        }
    }

    /// What floats over an expanded screen's scrolling pane: the Detailed
    /// view's media player.
    pub(super) fn pane_overlay(
        &self,
        body: Body,
        c: Ctx,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        (body == Body::Detailed).then(|| self.player(c, window, cx).into_any_element())
    }

    /// The compact media player: playback progress over the artwork, track
    /// and play / skip controls.
    fn player(&self, c: Ctx, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let k = c.k;
        let playing = self.playing[k];
        div()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .bg(color(ColorRole::SURFACE))
            .child(linear_progress(
                eid("player-progress", k, 0),
                "Playback position",
                Some(if c.web { 0.5 } else { 0.2 }),
                px(4.),
                false,
                window,
                cx,
            ))
            .child(
                div()
                    .h(rem(64.))
                    .pr(rem(if c.web { 8. } else { 12. }))
                    .flex()
                    .items_center()
                    .gap(rem(16.))
                    .bg(color(ColorRole::SURFACE_CONTAINER))
                    .child(art(32.).size(rem(64.)))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(rem(2.))
                            .child(line(TypeRole::BodyMedium, ColorRole::ON_SURFACE, "Title"))
                            .child(line(
                                TypeRole::BodySmall,
                                ColorRole::ON_SURFACE_VARIANT,
                                "Artist",
                            )),
                    )
                    .child(
                        IconButton::new(
                            eid("player-play", k, 0),
                            filled(if playing { "pause" } else { "play_arrow" }),
                            if playing { "Pause" } else { "Play" },
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.playing[k] = !this.playing[k];
                            cx.notify();
                        })),
                    )
                    .child(IconButton::new(
                        eid("player-next", k, 0),
                        filled("skip_next"),
                        "Next",
                    )),
            )
            .when(c.web, |player| {
                player
                    .absolute()
                    .left(rem(8.))
                    .right(rem(8.))
                    .bottom_0()
                    .rounded_t(rem(12.))
                    .overflow_hidden()
            })
    }

    fn reviews(&self, c: Ctx, cx: &mut Context<Self>) -> AnyElement {
        let k = c.k;
        let chips = || {
            div()
                .flex()
                .gap(rem(8.))
                .child(
                    AssistChip::new(eid("review-itinerary", k, 0), "Add to my itinerary")
                        .icon(icon("today"))
                        .elevated(true),
                )
                .child(
                    AssistChip::new(eid("review-walk", k, 0), "12 mins from hotel")
                        .icon(icon("directions_walk"))
                        .elevated(true),
                )
        };
        let caption = |padding: f32| {
            div()
                .absolute()
                .left_0()
                .right_0()
                .bottom_0()
                .p(rem(padding))
                .flex()
                .flex_col()
                .gap(rem(8.))
                .bg(scrim(0., 0.5))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(on_image(TypeRole::DisplayMedium, "Title"))
                        .child(on_image(TypeRole::BodySmall, "Subtitle").opacity(0.8)),
                )
                .child(chips())
        };
        let segment = self.segment[k];
        let group = ConnectedButtonGroup::new(eid("review-sort", k, 0), "Sort places")
            .options(SEGMENTS.map(|value| (value, "Label")))
            .selected(SEGMENTS[segment])
            .full_width()
            .on_change(cx.listener(move |this, value: &SharedString, _, cx| {
                if let Some(index) = SEGMENTS.iter().position(|v| *v == value.as_ref()) {
                    this.segment[k] = index;
                    cx.notify();
                }
            }));
        let favorites = self.favorites[k];
        let list = div()
            .flex()
            .flex_col()
            .when(c.web, |list| list.px(rem(8.)).gap(rem(4.)))
            .children((0..3).map(|i| {
                let favorite = favorites & (1 << i) != 0;
                div()
                    .child(
                        ListItem::new(eid("review-place", k, i), "List item")
                            .supporting(PLACE)
                            .three_line(true)
                            .leading(ListLeading::Image(None))
                            .trailing_text("100+")
                            .trailing(ListTrailing::Icon(symbol_favorite(favorite)))
                            .selected(favorite)
                            .when(c.web, |item| item.container(pane()))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.favorites[k] ^= 1 << i;
                                cx.notify();
                            })),
                    )
                    .child(
                        MaterialDivider::new(eid("review-divider", k, i))
                            .variant(DividerVariant::MiddleInset),
                    )
            }));
        if c.web {
            return div()
                .flex()
                .flex_col()
                .gap(rem(4.))
                .child(
                    app_bar(c, "review-bar", TopAppBarVariant::Small, "Label")
                        .navigation_icon(back(c, "review-back"))
                        .action(bar_button(
                            eid("review-favorite", k, 0),
                            "stars",
                            "Favorite",
                            false,
                        ))
                        .action(bar_button(
                            eid("review-search", k, 0),
                            "search",
                            "Search",
                            false,
                        )),
                )
                .child(
                    div()
                        .px(rem(24.))
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(rem(12.))
                        .child(
                            div()
                                .relative()
                                .w_full()
                                .h(rem(224.))
                                .rounded(rem(28.))
                                .overflow_hidden()
                                .child(art(120.).size_full())
                                .child(caption(24.)),
                        )
                        .child(div().w(rem(253.)).pb(rem(12.)).child(group)),
                )
                .child(list)
                .into_any_element();
        }
        let hero = div()
            .relative()
            .w_full()
            .h(rem(364.))
            .pb(rem(8.))
            .child(art(160.).size_full())
            .child(
                // The status bar and back button sit on a top scrim.
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(rem(116.))
                    .bg(scrim(0.4, 0.)),
            )
            .child(
                div().absolute().top(rem(60.)).left(rem(4.)).child(
                    IconButton::new(eid("review-back", k, 1), icon("arrow_back"), "Back")
                        .variant(IconButtonVariant::Tonal)
                        .width(IconButtonWidth::Narrow),
                ),
            )
            .child(caption(16.).bottom(rem(8.)));
        div()
            .flex()
            .flex_col()
            .child(hero)
            .child(
                div()
                    .px(rem(16.))
                    .py(rem(8.))
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(text(TypeRole::TitleLarge, ColorRole::ON_SURFACE, SECTION))
                    .child(text(
                        TypeRole::TitleSmall,
                        ColorRole::ON_SURFACE_VARIANT,
                        "Within 5 miles  • $$-$$$",
                    )),
            )
            .child(div().px(rem(16.)).py(rem(12.)).child(group))
            .child(list)
            .child(
                div().p(rem(16.)).child(
                    material_button(eid("review-all", k, 0), ButtonVariant::Outlined)
                        .button_size(ButtonSize::Medium)
                        .icon(filled("stars"))
                        .w_full()
                        .child("View 231 restaurants"),
                ),
            )
            .into_any_element()
    }

    fn library(&self, c: Ctx, cx: &mut Context<Self>) -> AnyElement {
        let (k, inset) = (c.k, c.inset());
        let filters = self.filters[k];
        let bar = app_bar(
            c,
            "library-bar",
            if c.web {
                TopAppBarVariant::MediumFlexible
            } else {
                TopAppBarVariant::Small
            },
            "Label",
        )
        .navigation_icon(back(c, "library-back"))
        .action(bar_button(
            eid("library-favorite", k, 0),
            "stars",
            "Favorite",
            false,
        ))
        .action(bar_button(
            eid("library-more", k, 0),
            "more_vert",
            "More options",
            false,
        ));
        let chips = scroll_row(
            eid("library-chips", k, 0),
            if c.web { 24. } else { 8. },
            if c.web { 8. } else { 4. },
        )
        .py(rem(if c.web { 16. } else { 8. }))
        .children((0..if c.web { 5 } else { 10 }).map(|i| {
            FilterChip::new(eid("library-chip", k, i), "Label")
                .selected(filters & (1 << i) != 0)
                .on_toggle(cx.listener(move |this, selected: &bool, _, cx| {
                    if *selected {
                        this.filters[k] |= 1 << i;
                    } else {
                        this.filters[k] &= !(1 << i);
                    }
                    cx.notify();
                }))
        }));
        let card = |i: usize| {
            div()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(rem(4.))
                .child(art(56.).w_full().h(rem(120.)).rounded(rem(16.)))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(line(TypeRole::TitleSmall, ColorRole::ON_SURFACE, "Title"))
                        .child(line(
                            TypeRole::BodySmall,
                            ColorRole::ON_SURFACE_VARIANT,
                            UPDATED[i % UPDATED.len()],
                        )),
                )
        };
        div()
            .flex()
            .flex_col()
            .child(bar)
            .child(chips)
            .when(!c.web, |library| {
                library.child(
                    div().p(rem(8.)).child(
                        div()
                            .py(rem(8.))
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                material_button(eid("library-sort", k, 0), ButtonVariant::Text)
                                    .icon(icon("swap_vert"))
                                    .child("Label"),
                            )
                            .child(IconButton::new(
                                eid("library-view", k, 0),
                                icon("list"),
                                "List view",
                            )),
                    ),
                )
            })
            .child(
                div()
                    .px(rem(inset))
                    .pb(rem(32.))
                    .grid()
                    .grid_cols(if c.web { 5 } else { 3 })
                    .gap(rem(if c.web { 16. } else { 8. }))
                    .children((0..if c.web { 20 } else { 12 }).map(card)),
            )
            .into_any_element()
    }

    fn gallery(&self, c: Ctx) -> AnyElement {
        let k = c.k;
        let bar = app_bar(c, "gallery-bar", TopAppBarVariant::LargeFlexible, "Label")
            .navigation_icon(bar_button(
                eid("gallery-account", k, 0),
                "account_circle",
                "Account",
                true,
            ))
            .action(bar_button(
                eid("gallery-notifications", k, 0),
                "notifications",
                "Notifications",
                false,
            ))
            .action(bar_button(
                eid("gallery-settings", k, 0),
                "settings",
                "Settings",
                false,
            ));
        let image = |glyph: f32| art(glyph).h(rem(176.)).rounded(rem(12.));
        if c.web {
            // Columns share the row in the kit's proportions.
            let cell = |basis: f32| {
                div()
                    .flex_basis(rem(basis))
                    .flex_grow(1.)
                    .flex_shrink(1.)
                    .min_w_0()
            };
            let pair = |basis: f32| {
                cell(basis)
                    .flex()
                    .flex_col()
                    .gap(rem(4.))
                    .child(image(72.).w_full())
                    .child(image(72.).w_full())
            };
            return div()
                .flex()
                .flex_col()
                .child(bar)
                .child(
                    div()
                        .px(rem(24.))
                        .py(rem(16.))
                        .flex()
                        .flex_col()
                        .gap(rem(4.))
                        .child(
                            div()
                                .h(rem(356.))
                                .flex()
                                .gap(rem(4.))
                                .child(cell(460.).child(art(144.).size_full().rounded(rem(12.))))
                                .child(
                                    cell(305.)
                                        .flex()
                                        .flex_col()
                                        .gap(rem(4.))
                                        .child(
                                            div()
                                                .flex()
                                                .gap(rem(4.))
                                                .child(cell(117.).child(image(56.).w_full()))
                                                .child(cell(185.).child(image(72.).w_full())),
                                        )
                                        .child(image(72.).w_full()),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(rem(4.))
                                .child(pair(250.))
                                .child(pair(370.))
                                .child(pair(141.)),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(rem(4.))
                                .child(pair(383.))
                                .child(pair(383.)),
                        ),
                )
                .into_any_element();
        }
        div()
            .flex()
            .flex_col()
            .child(bar)
            .child(
                div().py(rem(4.)).flex().flex_wrap().gap(rem(4.)).children(
                    [412., 158., 250., 412., 250., 158.]
                        .into_iter()
                        .map(|width| image(72.).w(rem(width))),
                ),
            )
            .child(
                div()
                    .pb(rem(16.))
                    .child(
                        div()
                            .h(rem(48.))
                            .px(rem(16.))
                            .flex()
                            .items_center()
                            .child(text(TypeRole::TitleLarge, ColorRole::ON_SURFACE, SECTION)),
                    )
                    .child(
                        scroll_row(eid("gallery-albums", k, 0), 16., 8.)
                            .py(rem(8.))
                            .children((0..9).map(|_| tile(false, "Label"))),
                    ),
            )
            .into_any_element()
    }

    fn messaging(&self, c: Ctx, cx: &mut Context<Self>) -> AnyElement {
        let k = c.k;
        let which = usize::from(c.web);
        let received = |message: &'static str| {
            div()
                .px(rem(16.))
                .py(rem(8.))
                .flex()
                .items_center()
                .gap(rem(8.))
                .child(art(20.).size(rem(36.)).rounded_full())
                .child(bubble(Bubble::Received, message))
        };
        let link = div()
            .px(rem(16.))
            .py(rem(8.))
            .flex()
            .flex_col()
            .items_end()
            .gap(rem(4.))
            .child(
                CardSurface::new(CardVariant::Filled)
                    .w(rem(224.))
                    .rounded(rem(12.))
                    .p(rem(12.))
                    .gap(rem(6.))
                    .child(art(96.).size(rem(200.)).rounded(rem(8.)))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(line(
                                TypeRole::BodyMedium,
                                ColorRole::ON_SURFACE,
                                "Homemade Dumplings",
                            ))
                            .child(line(
                                TypeRole::BodyMedium,
                                ColorRole::ON_SURFACE_VARIANT,
                                "everydumplingever.com",
                            )),
                    ),
            )
            .child(bubble(Bubble::Sent, "or we could make this?"));
        let sent = div()
            .px(rem(16.))
            .flex()
            .flex_col()
            .items_end()
            .gap(rem(4.))
            .children(
                self.sent[which]
                    .iter()
                    .map(|message| bubble(Bubble::Sent, message.clone())),
            );
        let suggestions = div()
            .px(rem(16.))
            .pt(rem(16.))
            .pb(rem(12.))
            .flex()
            .justify_end()
            .gap(rem(8.))
            .child(bubble(Bubble::Suggested, "Let’s do it"))
            .child(bubble(Bubble::Suggested, "Great!"))
            .child(bubble(Bubble::Suggested, "Great!"));
        let composer = div()
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap(rem(if c.web { 8. } else { 4. }))
            .child(IconButton::new(
                eid("message-attach", k, 0),
                icon("add_circle"),
                "Add attachment",
            ))
            .child(IconButton::new(
                eid("message-emoji", k, 0),
                icon("mood"),
                "Emoji",
            ))
            .child(
                SearchField::new(&self.messages[which])
                    .id(eid("message-field", k, 0))
                    .leading(IconButton::new(
                        eid("message-menu", k, 0),
                        icon("menu"),
                        "Menu",
                    ))
                    .trailing(IconButton::new(
                        eid("message-search", k, 0),
                        icon("search"),
                        "Search",
                    ))
                    .flex_1()
                    .min_w_0(),
            );
        let bar = |title: &'static str| {
            app_bar(c, "message-bar", TopAppBarVariant::Small, title)
                .navigation_icon(back(c, "message-back"))
                .action(bar_button(
                    eid("message-attach-bar", k, 0),
                    "attach_file",
                    "Attach",
                    false,
                ))
                .when(!c.web, |bar| {
                    bar.action(bar_button(
                        eid("message-favorite", k, 0),
                        "stars",
                        "Favorite",
                        false,
                    ))
                })
                .action(bar_button(
                    eid("message-more", k, 0),
                    "more_vert",
                    "More options",
                    false,
                ))
        };
        if !c.web {
            return div()
                .flex()
                .flex_col()
                .child(bar("Name"))
                .child(received("So excited!").pt(rem(16.)))
                .child(
                    div()
                        .px(rem(16.))
                        .py(rem(4.))
                        .flex()
                        .justify_center()
                        .child(text(TypeRole::BodyMedium, ColorRole::OUTLINE, "Yesterday")),
                )
                .child(received("What should we make?"))
                .child(received("Pasta?"))
                .child(link)
                .child(received("Sounds good!"))
                .child(sent)
                .child(suggestions)
                .child(
                    composer
                        .p(rem(16.))
                        .rounded_t(rem(12.))
                        .bg(color(ColorRole::SURFACE_CONTAINER)),
                )
                .into_any_element();
        }
        let selected = self.selected[k];
        div()
            .size_full()
            .flex()
            .child(
                div()
                    .w(rem(313.))
                    .h_full()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .h(rem(56.))
                            .px(rem(16.))
                            .flex()
                            .items_center()
                            .child(text(
                                TypeRole::TitleSmall,
                                ColorRole::ON_SURFACE_VARIANT,
                                "Conversations",
                            )),
                    )
                    .child(
                        div()
                            .id(eid("conversations", k, 0))
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .children((0..8).map(|i| {
                                ListItem::new(eid("conversation", k, i), "Name")
                                    .supporting(SUPPORTING)
                                    .leading(ListLeading::Monogram("A".into()))
                                    .trailing_text("10 min")
                                    .selected(selected == Some(i))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.selected[k] = Some(i);
                                        cx.notify();
                                    }))
                            })),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .pt(rem(16.))
                    .pr(rem(16.))
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .rounded_t(rem(28.))
                            .overflow_hidden()
                            .bg(color(pane()))
                            .flex()
                            .flex_col()
                            .child(bar("Name"))
                            .child(
                                div()
                                    .id(eid("conversation-scroll", k, 0))
                                    .flex_1()
                                    .min_h_0()
                                    .overflow_y_scroll()
                                    .child(link)
                                    .child(received("That looks so good!"))
                                    .child(sent)
                                    .child(suggestions),
                            )
                            .child(composer.h(rem(80.)).px(rem(16.))),
                    ),
            )
            .into_any_element()
    }
}

/// The favorite symbol: filled once favorited.
fn symbol_favorite(favorite: bool) -> gpui_kit_material::icons::Icon {
    if favorite {
        filled("favorite")
    } else {
        icon("favorite")
    }
}
