//! The example app's screens, composed from crate components at compact
//! (phone) or expanded (tablet, desktop) size from the kit's frames of the
//! same name, and the devices around them. Everything is operable: app bar
//! actions toggle, navigate or open menus, tiles and cards open the detailed
//! view, chips filter, the player plays, and messages send.
use super::super::prelude::*;
use super::app::{Body, CONVERSATIONS, Ctx, Device, ExamplesApp, Screen, nav_items, nav_value};
use super::parts::{
    self, Bubble, art, bubble, eid, feature_item, filled, icon, line, media_card, nav_icon,
    pressable, scroll_row, section_header, text, tile,
};
use gpui_kit_material::components::{IconButtonWidth, linear_progress};

const SECTION: &str = "Section title";
const SHOW_ALL: &str = "Show all";
const SUPPORTING: &str = "Supporting line text lorem ipsum dolor sit amet, consectetur";
const DESCRIPTION: &str = "Description duis aute irure dolor in reprehenderit in voluptate velit.";
const MORE_LIKE: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
const PARAGRAPH: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.\n\nDuis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
const PLACE: &str =
    "Category • $$ • 1.2 miles away\nSupporting line text lorem ipsum dolor sit amet, consectetur";
const UPDATED: [&str; 3] = ["Updated today", "Updated yesterday", "Updated 2 days ago"];
/// The Reviews sort's segments.
const SEGMENTS: [(&str, &str); 2] = [("nearest", "Nearest"), ("top", "Top rated")];
/// The Library's filter chips.
const FILTERS: [&str; 5] = ["Albums", "Artists", "Podcasts", "Playlists", "Downloads"];
/// The tracks the player moves through.
const TRACKS: [(&str, &str); 3] = [
    ("Title", "Artist"),
    ("Headline", "Today"),
    ("Song", "Album"),
];
/// The items of the app bars' overflow menus.
const OVERFLOW: [(&str, &str); 3] = [
    ("Share", "share"),
    ("Add to list", "playlist_add"),
    ("Report", "flag"),
];
const ACCOUNT: [(&str, &str); 2] = [("Profile", "person"), ("Sign out", "logout")];
const COMPOSE: [(&str, &str); 3] = [
    ("Note", "edit_note"),
    ("Checklist", "checklist"),
    ("Photo", "image"),
];
const ATTACH: [(&str, &str); 2] = [("Photo", "image"), ("File", "attach_file")];
/// A reply the Messaging screen suggests.
const SUGGESTIONS: [&str; 3] = ["Let’s do it", "Great!", "Sounds good"];

/// The pane color of expanded content, which bars and list rows sit on.
fn pane() -> ColorRole {
    ColorRole::SURFACE_CONTAINER_LOWEST
}

/// A top app bar on the screen's pane: surface when compact, the expanded
/// content pane's surface-container-lowest when expanded. Flexible ones
/// collapse with the content's scroll.
fn app_bar(
    c: Ctx,
    name: &'static str,
    variant: TopAppBarVariant,
    title: &'static str,
) -> TopAppBar {
    TopAppBar::new(eid(name, c.k, 0), variant)
        .title(title)
        .scroll_offset(c.offset)
        .when(c.expanded, |bar| bar.container(pane()))
}

/// The room a bar takes above the content.
fn room(variant: TopAppBarVariant) -> f32 {
    variant.expanded_height(false)
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

/// The favorite symbol: filled once favorited.
fn symbol_favorite(favorite: bool) -> gpui_kit_material::icons::Icon {
    if favorite {
        filled("favorite")
    } else {
        icon("favorite")
    }
}

/// A Library card's description for the `i`th card.
fn updated(i: usize) -> &'static str {
    UPDATED[i % UPDATED.len()]
}

/// The Library's items under `filters`, in order: an item belongs to one
/// filter, and with no filter on all of them show.
pub(super) fn library_items(filters: u16, count: usize, oldest_first: bool) -> Vec<usize> {
    let mut items: Vec<usize> = (0..count)
        .filter(|item| filters == 0 || filters & (1 << (item % FILTERS.len())) != 0)
        .collect();
    if oldest_first {
        items.reverse();
    }
    items
}

impl ExamplesApp {
    /// A click listener on the app. Unlike `cx.listener`'s, its type holds no
    /// borrow of `cx`, so element helpers can return it inside iterators.
    fn act<F>(
        cx: &mut Context<Self>,
        action: F,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static + use<F>
    where
        F: Fn(&mut Self, &ClickEvent, &mut Window, &mut Context<Self>) + 'static,
    {
        let app = cx.weak_entity();
        move |event, window, cx| {
            let _ = app.update(cx, |app, cx| action(app, event, window, cx));
        }
    }

    /// A listener navigating to `body`.
    fn to(
        &self,
        body: Body,
        cx: &mut Context<Self>,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static + use<> {
        Self::act(cx, move |this, _, window, cx| this.go(body, window, cx))
    }

    /// A listener putting `message` on the snackbar.
    fn tell(
        &self,
        message: &'static str,
        cx: &mut Context<Self>,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static + use<> {
        Self::act(cx, move |this, _, _, cx| this.say(message, cx))
    }

    fn back_button(&self, c: Ctx, name: &'static str, cx: &mut Context<Self>) -> ToolbarButton {
        ToolbarButton::icon(eid(name, c.k, 1), nav_icon("arrow_back"), "Back")
            .ink(ColorRole::ON_SURFACE)
            .on_click(cx.listener(|this, _, window, cx| this.go_back(window, cx)))
    }

    /// An app bar action that toggles, keeping its state in `name`.
    fn mark(
        &self,
        c: Ctx,
        name: &'static str,
        symbol: &'static str,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> ToolbarButton {
        ToolbarButton::icon(eid(name, c.k, 0), nav_icon(symbol), label)
            .toggle(true)
            .selected(self.marks.contains(name))
            .on_click(cx.listener(move |this, _, _, cx| this.toggle_mark(name, cx)))
    }

    /// An app bar action that goes to `body`.
    fn link(
        &self,
        c: Ctx,
        name: &'static str,
        symbol: &'static str,
        label: &'static str,
        body: Body,
        cx: &mut Context<Self>,
    ) -> ToolbarButton {
        ToolbarButton::icon(eid(name, c.k, 0), nav_icon(symbol), label).on_click(self.to(body, cx))
    }

    /// An app bar action that reports `message`.
    fn notice(
        &self,
        c: Ctx,
        name: &'static str,
        symbol: &'static str,
        label: &'static str,
        message: &'static str,
        cx: &mut Context<Self>,
    ) -> ToolbarButton {
        ToolbarButton::icon(eid(name, c.k, 0), nav_icon(symbol), label)
            .on_click(self.tell(message, cx))
    }

    /// A menu opened by `trigger`: one popup is open at a time, a choice
    /// reports itself on the snackbar.
    fn menu_of(
        &self,
        c: Ctx,
        key: &'static str,
        items: &'static [(&'static str, &'static str)],
        trigger: impl IntoElement,
        cx: &mut Context<Self>,
    ) -> MenuPopup {
        let mut menu = MaterialMenu::new(eid(key, c.k, 1)).width(200.);
        for (label, symbol) in items {
            let label = *label;
            menu = menu.item(MaterialMenuItem::new(label).leading(icon(symbol)).on_click(
                cx.listener(move |this, _: &ClickEvent, _, cx| {
                    this.open_menu = None;
                    this.say(label, cx);
                }),
            ));
        }
        MenuPopup::new(eid(key, c.k, 2), trigger, menu)
            .open(self.open_menu == Some(key))
            .on_open_change(cx.listener(move |this, open: &bool, _, cx| {
                if *open {
                    this.open_menu = Some(key);
                } else if this.open_menu == Some(key) {
                    this.open_menu = None;
                }
                cx.notify();
            }))
    }

    /// Opens a menu from the keyboard; pointer presses reach the popover.
    fn menu_keys(
        &self,
        key: &'static str,
        cx: &mut Context<Self>,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static + use<> {
        Self::act(cx, move |this, event, _, cx| {
            if matches!(event, ClickEvent::Keyboard(_)) {
                this.open_menu = if this.open_menu == Some(key) {
                    None
                } else {
                    Some(key)
                };
                cx.notify();
            }
        })
    }

    /// An app bar action opening a menu.
    fn bar_menu(
        &self,
        c: Ctx,
        key: &'static str,
        symbol: &'static str,
        label: &'static str,
        items: &'static [(&'static str, &'static str)],
        cx: &mut Context<Self>,
    ) -> MenuPopup {
        let trigger = ToolbarButton::icon(eid(key, c.k, 0), nav_icon(symbol), label)
            .on_click(self.menu_keys(key, cx));
        self.menu_of(c, key, items, trigger, cx)
    }

    fn overflow(&self, c: Ctx, key: &'static str, cx: &mut Context<Self>) -> MenuPopup {
        self.bar_menu(c, key, "more_vert", "More options", &OVERFLOW, cx)
    }

    /// Everything a scrolling screen draws: the content under its bar, what
    /// floats over it, and what stays pinned below.
    pub(super) fn scaffold(
        &self,
        screen: Screen,
        body: Body,
        c: Ctx,
        cx: &mut Context<Self>,
    ) -> Div {
        let Screen {
            bar,
            content,
            footer,
            overlay,
        } = screen;
        let (bar, room) = bar.map_or((None, 0.), |(bar, room)| (Some(bar), room));
        div()
            .flex_1()
            .min_h_0()
            .w_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .child(
                        div()
                            .id(eid("screen", c.k, body.index()))
                            .debug_selector(|| "example-scroll".into())
                            .size_full()
                            .overflow_y_scroll()
                            .track_scroll(&self.scrolls[body.index()])
                            // The offset collapses the bar.
                            .on_scroll(cx.listener(|_, _, _, cx| cx.notify()))
                            .child(div().w_full().pt(rem(room)).child(content)),
                    )
                    .when_some(bar, |frame, bar| {
                        frame.child(
                            div()
                                .id(eid("screen-bar", c.k, 0))
                                .absolute()
                                .top_0()
                                .left_0()
                                .right_0()
                                .occlude()
                                .child(bar),
                        )
                    })
                    .children(overlay)
                    .when_some(self.snackbar.clone(), |frame, message| {
                        frame.child(
                            div()
                                .debug_selector(|| "example-snackbar".into())
                                .absolute()
                                .left(rem(16.))
                                .right(rem(16.))
                                .bottom(rem(16.))
                                .child(Snackbar::new(eid("snackbar", c.k, 0), message).close(
                                    "Dismiss",
                                    cx.listener(|this, _: &ClickEvent, _, cx| {
                                        this.dismiss_snackbar(cx)
                                    }),
                                )),
                        )
                    }),
            )
            .children(footer)
    }

    /// The phone: status bar, the screen, its navigation bar and gesture bar.
    pub(super) fn phone(
        &mut self,
        body: Body,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let device = Device::Phone;
        let c = Ctx {
            k: device.index(),
            expanded: false,
            width: device.screen().0,
            offset: self.offset(body, window),
        };
        let has_nav = !nav_value(body).is_empty();
        // The gesture bar continues the surface-container of a navigation
        // bar, player or message bar above it.
        let gesture = if has_nav || matches!(body, Body::Detailed | Body::Messaging) {
            ColorRole::SURFACE_CONTAINER
        } else {
            ColorRole::SURFACE
        };
        let screen = self.screen(body, c, window, cx);
        let content = self.scaffold(screen, body, c, cx).when(has_nav, |content| {
            content.child(
                NavigationBar::new(eid("nav-bar", c.k, 0), "Destinations")
                    .items(nav_items().iter().cloned())
                    .selected(nav_value(body))
                    .on_change(self.navigate_to(cx)),
            )
        });
        parts::phone(
            eid("device", c.k, 0),
            content,
            gesture,
            body == Body::Reviews,
        )
        .into_any_element()
    }

    /// A destination's value navigates to its screen.
    fn navigate_to(
        &self,
        cx: &mut Context<Self>,
    ) -> impl Fn(&SharedString, &mut Window, &mut App) + 'static + use<> {
        let app = cx.weak_entity();
        move |value, window, cx| {
            if let Some((body, ..)) = super::app::NAV
                .iter()
                .find(|(_, candidate, ..)| *candidate == value.as_ref())
            {
                let _ = app.update(cx, |app, cx| app.go(*body, window, cx));
            }
        }
    }

    /// A tablet or desktop window: its chrome around the navigation rail and
    /// the content pane. Messaging has panes of its own and no rail.
    pub(super) fn expanded(
        &mut self,
        device: Device,
        body: Body,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let screen_width = device.screen().0;
        let rail_expanded = self.rail_expanded.unwrap_or(device == Device::Desktop);
        let rail_width = if body == Body::Messaging {
            0.
        } else if rail_expanded {
            RAIL_EXPANDED_WIDTH
        } else {
            RAIL_COLLAPSED_WIDTH
        };
        let c = Ctx {
            k: device.index(),
            expanded: true,
            // The pane keeps a 16dp margin to the window's trailing edge.
            width: screen_width - rail_width - 16.,
            offset: self.offset(body, window),
        };
        let content = if body == Body::Messaging {
            self.messaging_panes(c, window, cx)
        } else {
            let screen = self.screen(body, c, window, cx);
            let pane_content = self.scaffold(screen, body, c, cx);
            div()
                .size_full()
                .flex()
                .child(self.rail(c, body, rail_expanded, cx))
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
                                .flex()
                                .flex_col()
                                .rounded_t(rem(28.))
                                .overflow_hidden()
                                .bg(color(pane()))
                                .child(pane_content),
                        ),
                )
                .into_any_element()
        };
        let id = eid("device", c.k, 0);
        match device {
            Device::Tablet => parts::tablet(id, content).into_any_element(),
            _ => {
                let bar = self.browser_bar(c, body, cx);
                parts::browser(id, device.screen(), bar, content).into_any_element()
            }
        }
    }

    /// The browser's bar: history, reload, the address, and its own bookmark
    /// and menu.
    fn browser_bar(&self, c: Ctx, body: Body, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let bookmarked = self.marks.contains("browser-star");
        div()
            .flex_shrink_0()
            .h(rem(68.))
            .px(rem(12.))
            .flex()
            .items_center()
            .gap(rem(8.))
            .border_b_1()
            .border_color(color(ColorRole::OUTLINE_VARIANT))
            .child(
                IconButton::new(eid("browser-back", c.k, 0), icon("arrow_back"), "Back")
                    .disabled(!self.can_go_back())
                    .on_click(cx.listener(|this, _, window, cx| this.go_back(window, cx))),
            )
            .child(
                IconButton::new(
                    eid("browser-forward", c.k, 0),
                    icon("arrow_forward"),
                    "Forward",
                )
                .disabled(!self.can_go_forward())
                .on_click(cx.listener(|this, _, window, cx| this.go_forward(window, cx))),
            )
            .child(
                IconButton::new(eid("browser-reload", c.k, 0), icon("refresh"), "Reload")
                    .on_click(cx.listener(|this, _, _, cx| this.reload(cx))),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h(rem(36.))
                    .pl(rem(16.))
                    .pr(rem(4.))
                    .rounded_full()
                    .bg(color(ColorRole::SURFACE_CONTAINER))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .min_w_0()
                            .items_center()
                            .gap(rem(8.))
                            .child(
                                filled("lock")
                                    .size(rem(15.))
                                    .color(color(ColorRole::ON_SURFACE_VARIANT)),
                            )
                            .child(
                                div()
                                    .debug_selector(|| "browser-address".into())
                                    .flex_1()
                                    .min_w_0()
                                    .child(line(
                                        TypeRole::BodyLarge,
                                        ColorRole::ON_SURFACE,
                                        body.path(),
                                    )),
                            ),
                    )
                    .child(
                        IconButton::new(
                            eid("browser-star", c.k, 0),
                            if bookmarked {
                                filled("star")
                            } else {
                                icon("star")
                            },
                            "Bookmark this page",
                        )
                        .on_click(
                            cx.listener(|this, _, _, cx| this.toggle_mark("browser-star", cx)),
                        ),
                    ),
            )
            .child(
                self.menu_of(
                    c,
                    "browser-account",
                    &ACCOUNT,
                    IconButton::new(
                        eid("browser-account", c.k, 0),
                        filled("account_circle"),
                        "Account",
                    )
                    .on_click(self.menu_keys("browser-account", cx)),
                    cx,
                ),
            )
            .child(
                self.menu_of(
                    c,
                    "browser-more",
                    &OVERFLOW,
                    IconButton::new(eid("browser-more", c.k, 0), icon("more_vert"), "More")
                        .on_click(self.menu_keys("browser-more", cx)),
                    cx,
                ),
            )
    }

    /// The expanded screens' navigation rail: a menu button that expands it,
    /// a FAB (extended while expanded) that opens a compose menu, and the
    /// destinations.
    fn rail(
        &self,
        c: Ctx,
        body: Body,
        expanded: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let fab = self.menu_of(
            c,
            "compose",
            &COMPOSE,
            ExtendedFab::new(eid("fab", c.k, 0), filled("edit"), "Compose")
                .expanded(expanded)
                .on_click(self.menu_keys("compose", cx)),
            cx,
        );
        div().h_full().flex_shrink_0().child(
            NavigationRail::new(eid("rail", c.k, 0), "Destinations")
                .items(nav_items().iter().cloned())
                .selected(nav_value(body))
                .expanded(expanded)
                .menu(
                    IconButton::new(
                        eid("rail-menu", c.k, 0),
                        icon(if expanded { "menu_open" } else { "menu" }),
                        if expanded {
                            "Collapse navigation"
                        } else {
                            "Expand navigation"
                        },
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.rail_expanded = Some(!expanded);
                        cx.notify();
                    })),
                )
                .fab(fab)
                .on_change(self.navigate_to(cx)),
        )
    }

    /// The kit's hero carousel: one large item and a small one on a phone;
    /// on the expanded pane the kit's tablet hero shows several 316dp items
    /// stepping down to a small one, the multi-browse arrangement. Its items
    /// open the detailed view.
    fn hero_carousel(
        &self,
        c: Ctx,
        name: &'static str,
        cx: &mut Context<Self>,
    ) -> MaterialCarousel {
        MaterialCarousel::new(eid(name, c.k, 0), "Featured")
            .layout(if c.expanded {
                CarouselLayout::MultiBrowse
            } else {
                CarouselLayout::Hero
            })
            .large_item_width(316.)
            .item_height(205.)
            .width(c.width)
            .items((0..6).map(|i| {
                CarouselItem::new().content(pressable(
                    eid(name, c.k, 10 + i),
                    "Open",
                    art(96.).size_full(),
                    self.to(Body::Detailed, cx),
                ))
            }))
    }

    /// `body`'s screen. Messaging's expanded panes are [`Self::messaging_panes`].
    pub(super) fn screen(
        &mut self,
        body: Body,
        c: Ctx,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Screen {
        match body {
            Body::Home => self.home(c, cx),
            Body::Upcoming => self.upcoming(c, cx),
            Body::Detailed => self.detailed(c, window, cx),
            Body::Reviews => self.reviews(c, cx),
            Body::Library => self.library(c, cx),
            Body::Gallery => self.gallery(c, cx),
            Body::Messaging => self.messaging(c, cx),
        }
    }

    /// A play button for `track`, showing pause while it plays.
    fn play_icon(&self, track: usize) -> (&'static str, &'static str) {
        if self.playing && self.track == track {
            ("pause", "Pause")
        } else {
            ("play_arrow", "Play")
        }
    }

    fn home(&self, c: Ctx, cx: &mut Context<Self>) -> Screen {
        let (k, inset) = (c.k, c.inset());
        let gap = if c.expanded { 24. } else { 8. };
        let variant = if c.expanded {
            TopAppBarVariant::LargeFlexible
        } else {
            TopAppBarVariant::MediumFlexible
        };
        let bar = app_bar(c, "home-bar", variant, "Home")
            .navigation_icon(self.bar_menu(
                c,
                "home-account",
                "account_circle",
                "Account",
                &ACCOUNT,
                cx,
            ))
            .action(self.link(
                c,
                "home-notifications",
                "notifications",
                "Messages",
                Body::Messaging,
                cx,
            ))
            .action(self.notice(c, "home-settings", "settings", "Settings", "Settings", cx));
        let header = |i: usize, target: Body, cx: &mut Context<Self>| {
            section_header(
                eid("home-header", k, i),
                SECTION,
                SHOW_ALL,
                inset,
                self.to(target, cx),
            )
        };
        let artists = div()
            .pb(rem(16.))
            .child(header(0, Body::Library, cx))
            .child(scroll_row(eid("home-artists", k, 0), inset, gap).children(
                (0..if c.expanded { 13 } else { 8 }).map(|i| {
                    tile(
                        eid("home-artist", k, i),
                        true,
                        "Label",
                        self.to(Body::Detailed, cx),
                    )
                }),
            ));
        let (play_symbol, play_label) = self.play_icon(0);
        let hero = div()
            .child(header(1, Body::Upcoming, cx))
            .child(self.hero_carousel(c, "home-hero", cx))
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
                            IconButton::new(
                                eid("home-play", k, 0),
                                filled(play_symbol),
                                play_label,
                            )
                            .variant(IconButtonVariant::Tonal)
                            .on_click(cx.listener(|this, _, _, cx| this.play(0, cx))),
                        ),
                ),
            );
        let albums = div().child(header(2, Body::Library, cx)).child(
            scroll_row(eid("home-albums", k, 0), inset, gap)
                .py(rem(8.))
                .children((0..if c.expanded { 13 } else { 8 }).map(|i| {
                    tile(
                        eid("home-album", k, i),
                        false,
                        "Label",
                        self.to(Body::Detailed, cx),
                    )
                })),
        );
        let cards = div()
            .pb(rem(16.))
            .child(header(3, Body::Library, cx))
            .child(
                scroll_row(eid("home-cards", k, 0), inset, gap).children((0..9).map(|i| {
                    media_card(
                        eid("home-card", k, i),
                        "Artist",
                        "Song",
                        self.to(Body::Detailed, cx),
                    )
                })),
            );
        let features = (!c.expanded).then(|| {
            div()
                .px(rem(16.))
                .pb(rem(8.))
                .child(section_header(
                    eid("home-header", k, 4),
                    SECTION,
                    SHOW_ALL,
                    0.,
                    self.to(Body::Upcoming, cx),
                ))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(rem(8.))
                        .children((0..2).map(|i| self.feature(c, "home-feature", i, cx))),
                )
        });
        let more = (!c.expanded).then(|| {
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
                        .child(
                            IconButton::new(
                                eid("home-more-like", k, 0),
                                icon("arrow_forward"),
                                SHOW_ALL,
                            )
                            .on_click(self.to(Body::Library, cx)),
                        ),
                )
                .child(
                    scroll_row(eid("home-more", k, 0), 16., 8.).children((0..8).map(|i| {
                        tile(
                            eid("home-more-tile", k, i),
                            false,
                            "Label",
                            self.to(Body::Detailed, cx),
                        )
                    })),
                )
                .child(
                    text(
                        TypeRole::BodyMedium,
                        ColorRole::ON_SURFACE_VARIANT,
                        MORE_LIKE,
                    )
                    .px(rem(16.))
                    .line_clamp(2),
                )
        });
        Screen {
            bar: Some((bar, room(variant))),
            content: div()
                .flex()
                .flex_col()
                .child(artists)
                .child(hero)
                .child(albums)
                .child(cards)
                .children(features)
                .children(more)
                .into_any_element(),
            footer: None,
            overlay: None,
        }
    }

    /// A feature item playing track `i`.
    fn feature(&self, c: Ctx, name: &'static str, i: usize, cx: &mut Context<Self>) -> Div {
        let (symbol, label) = self.play_icon(i);
        feature_item(
            eid(name, c.k, i),
            "Headline",
            DESCRIPTION,
            "Today • 23 min",
            symbol,
            label,
            cx.listener(move |this, _: &ClickEvent, _, cx| this.play(i, cx)),
        )
    }

    fn upcoming(&self, c: Ctx, cx: &mut Context<Self>) -> Screen {
        let (k, inset) = (c.k, c.inset());
        let selected = self.selected;
        let variant = TopAppBarVariant::MediumFlexible;
        let bar = app_bar(c, "upcoming-bar", variant, "Upcoming")
            .navigation_icon(self.back_button(c, "upcoming-back", cx))
            .action(self.bar_menu(c, "upcoming-attach", "attach_file", "Attach", &ATTACH, cx))
            .action(self.mark(c, "upcoming-today", "today", "Add to calendar", cx))
            .action(self.overflow(c, "upcoming-more", cx));
        let content = div()
            .flex()
            .flex_col()
            .child(self.hero_carousel(c, "upcoming-hero", cx))
            .child(
                div().px(rem(inset)).flex().justify_end().child(
                    material_button(eid("upcoming-show-all", k, 0), ButtonVariant::Text)
                        .icon(filled("stars"))
                        .child(SHOW_ALL)
                        .on_click(self.to(Body::Library, cx)),
                ),
            )
            .child(section_header(
                eid("upcoming-header", k, 0),
                if c.expanded { "Featured" } else { SECTION },
                SHOW_ALL,
                inset,
                self.to(Body::Library, cx),
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .pb(rem(16.))
                    .when(c.expanded, |list| list.px(rem(8.)))
                    .children((0..5).map(|i| {
                        let checked = selected & (1 << i) != 0;
                        ListItem::new(eid("upcoming-item", k, i), "List item")
                            .supporting(SUPPORTING)
                            .three_line(true)
                            .leading(ListLeading::Image(None))
                            .trailing(ListTrailing::Checkbox(checked))
                            .selected(checked)
                            .when(c.expanded, |item| item.container(pane()))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.selected ^= 1 << i;
                                cx.notify();
                            }))
                    })),
            )
            .into_any_element();
        Screen {
            bar: Some((bar, room(variant))),
            content,
            footer: None,
            overlay: None,
        }
    }

    fn detailed(&self, c: Ctx, window: &mut Window, cx: &mut Context<Self>) -> Screen {
        let (k, inset) = (c.k, c.inset());
        let variant = TopAppBarVariant::Small;
        let bar = app_bar(c, "detail-bar", variant, "Detailed view")
            .navigation_icon(self.back_button(c, "detail-back", cx))
            .action(self.mark(c, "detail-favorite", "stars", "Favorite", cx))
            .action(self.mark(c, "detail-save", "bookmark", "Save", cx))
            .action(self.overflow(c, "detail-more", cx));
        let published = || {
            text(
                TypeRole::LabelSmall,
                ColorRole::ON_SURFACE_VARIANT,
                "Published date",
            )
        };
        let paragraph = || text(TypeRole::BodyMedium, ColorRole::ON_SURFACE, PARAGRAPH);
        let downloaded = self.marks.contains("detail-download");
        let header = div()
            .flex()
            .gap(rem(24.))
            .px(rem(inset))
            .py(rem(8.))
            .child(
                art(if c.expanded { 112. } else { 72. })
                    .size(rem(if c.expanded { 216. } else { 136. }))
                    .rounded(rem(28.)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(rem(if c.expanded { 16. } else { 24. }))
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
                            .when(c.expanded, |column| {
                                column.child(published()).child(paragraph().line_clamp(5))
                            }),
                    )
                    .child(
                        material_button(
                            eid("detail-download", k, 0),
                            if downloaded {
                                ButtonVariant::Tonal
                            } else {
                                ButtonVariant::Filled
                            },
                        )
                        .icon(filled(if downloaded { "check" } else { "stars" }))
                        .child(if downloaded { "Downloaded" } else { "Download" })
                        .on_click(
                            cx.listener(|this, _, _, cx| this.toggle_mark("detail-download", cx)),
                        ),
                    ),
            );
        let cards = div()
            .pb(rem(32.))
            .child(
                div()
                    .h(rem(if c.expanded { 64. } else { 48. }))
                    .px(rem(inset))
                    .flex()
                    .items_center()
                    .child(text(
                        TypeRole::HeadlineSmall,
                        ColorRole::ON_SURFACE,
                        SECTION,
                    ))
                    .child(
                        IconButton::new(
                            eid("detail-header", k, 0),
                            icon("arrow_forward"),
                            SHOW_ALL,
                        )
                        .on_click(self.to(Body::Library, cx)),
                    ),
            )
            .child(
                div()
                    .px(rem(inset))
                    .flex()
                    .flex_col()
                    .gap(rem(16.))
                    .children((0..3).map(|i| self.feature(c, "detail-feature", i, cx))),
            );
        let content = div()
            .flex()
            .flex_col()
            .child(header)
            .when(!c.expanded, |content| {
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
            .child(cards)
            // The expanded player floats over the end of the content.
            .when(c.expanded, |content| content.pb(rem(68.)))
            .into_any_element();
        let player = self.player(c, window, cx).into_any_element();
        let (footer, overlay) = if c.expanded {
            (None, Some(player))
        } else {
            (Some(player), None)
        };
        Screen {
            bar: Some((bar, room(variant))),
            content,
            footer,
            overlay,
        }
    }

    /// The compact media player: playback progress over the artwork, track
    /// and play / skip controls.
    fn player(&self, c: Ctx, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let k = c.k;
        let (title, artist) = TRACKS[self.track];
        let (symbol, label) = self.play_icon(self.track);
        let track = self.track;
        div()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .bg(color(ColorRole::SURFACE))
            .child(linear_progress(
                eid("player-progress", k, 0),
                "Playback position",
                Some(self.position),
                px(4.),
                false,
                window,
                cx,
            ))
            .child(
                div()
                    .h(rem(64.))
                    .pr(rem(if c.expanded { 8. } else { 12. }))
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
                            .child(line(TypeRole::BodyMedium, ColorRole::ON_SURFACE, title))
                            .child(line(
                                TypeRole::BodySmall,
                                ColorRole::ON_SURFACE_VARIANT,
                                artist,
                            )),
                    )
                    .child(
                        IconButton::new(eid("player-play", k, 0), filled(symbol), label)
                            .on_click(cx.listener(move |this, _, _, cx| this.play(track, cx))),
                    )
                    .child(
                        IconButton::new(eid("player-next", k, 0), filled("skip_next"), "Next")
                            .on_click(cx.listener(|this, _, _, cx| this.next_track(cx))),
                    ),
            )
            .when(c.expanded, |player| {
                player
                    .absolute()
                    .left(rem(8.))
                    .right(rem(8.))
                    .bottom_0()
                    .rounded_t(rem(12.))
                    .overflow_hidden()
            })
    }

    fn reviews(&self, c: Ctx, cx: &mut Context<Self>) -> Screen {
        let k = c.k;
        let itinerary = self.marks.contains("review-itinerary");
        let chips = |cx: &mut Context<Self>| {
            div()
                .flex()
                .gap(rem(8.))
                .child(
                    AssistChip::new(
                        eid("review-itinerary", k, 0),
                        if itinerary {
                            "In my itinerary"
                        } else {
                            "Add to my itinerary"
                        },
                    )
                    .icon(icon(if itinerary {
                        "event_available"
                    } else {
                        "today"
                    }))
                    .elevated(true)
                    .on_click(
                        cx.listener(|this, _, _, cx| this.toggle_mark("review-itinerary", cx)),
                    ),
                )
                .child(
                    AssistChip::new(eid("review-walk", k, 0), "12 mins from hotel")
                        .icon(icon("directions_walk"))
                        .elevated(true)
                        .on_click(self.tell("Directions from the hotel", cx)),
                )
        };
        let caption = |padding: f32, cx: &mut Context<Self>| {
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
                .child(chips(cx))
        };
        let group = ConnectedButtonGroup::new(eid("review-sort", k, 0), "Sort places")
            .options(SEGMENTS)
            .selected(SEGMENTS[self.segment].0)
            .full_width()
            .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                if let Some(index) = SEGMENTS.iter().position(|(v, _)| *v == value.as_ref()) {
                    this.segment = index;
                    cx.notify();
                }
            }));
        let favorites = self.favorites;
        // Top rated reverses the nearest-first order.
        let order: [usize; 3] = if self.segment == 1 {
            [2, 1, 0]
        } else {
            [0, 1, 2]
        };
        let list = div()
            .flex()
            .flex_col()
            .when(c.expanded, |list| list.px(rem(8.)).gap(rem(4.)))
            .children(order.into_iter().map(|i| {
                let favorite = favorites & (1 << i) != 0;
                div()
                    .child(
                        ListItem::new(eid("review-place", k, i), PLACE_NAMES[i])
                            .supporting(PLACE)
                            .three_line(true)
                            .leading(ListLeading::Image(None))
                            .trailing_text("100+")
                            .trailing(ListTrailing::Icon(symbol_favorite(favorite)))
                            .selected(favorite)
                            .when(c.expanded, |item| item.container(pane()))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.favorites ^= 1 << i;
                                cx.notify();
                            })),
                    )
                    .child(
                        MaterialDivider::new(eid("review-divider", k, i))
                            .variant(DividerVariant::MiddleInset),
                    )
            }));
        if c.expanded {
            let variant = TopAppBarVariant::Small;
            let bar = app_bar(c, "review-bar", variant, "Reviews")
                .navigation_icon(self.back_button(c, "review-back", cx))
                .action(self.mark(c, "review-favorite", "stars", "Favorite", cx))
                .action(self.notice(c, "review-search", "search", "Search", "Search", cx));
            let content = div()
                .flex()
                .flex_col()
                .gap(rem(4.))
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
                                .child(caption(24., cx)),
                        )
                        .child(div().w(rem(253.)).pb(rem(12.)).child(group)),
                )
                .child(list)
                .into_any_element();
            return Screen {
                bar: Some((bar, room(variant))),
                content,
                footer: None,
                overlay: None,
            };
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
                        .width(IconButtonWidth::Narrow)
                        .on_click(cx.listener(|this, _, window, cx| this.go_back(window, cx))),
                ),
            )
            .child(caption(16., cx).bottom(rem(8.)));
        let content = div()
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
                        .child("View 231 restaurants")
                        .on_click(self.to(Body::Library, cx)),
                ),
            )
            .into_any_element();
        Screen {
            bar: None,
            content,
            footer: None,
            overlay: None,
        }
    }

    fn library(&self, c: Ctx, cx: &mut Context<Self>) -> Screen {
        let (k, inset) = (c.k, c.inset());
        let filters = self.filters;
        let variant = if c.expanded {
            TopAppBarVariant::MediumFlexible
        } else {
            TopAppBarVariant::Small
        };
        let bar = app_bar(c, "library-bar", variant, "Library")
            .navigation_icon(self.back_button(c, "library-back", cx))
            .action(self.mark(c, "library-favorite", "stars", "Favorite", cx))
            .action(self.overflow(c, "library-more", cx));
        let chips = scroll_row(
            eid("library-chips", k, 0),
            if c.expanded { 24. } else { 8. },
            if c.expanded { 8. } else { 4. },
        )
        .py(rem(if c.expanded { 16. } else { 8. }))
        .children(FILTERS.iter().enumerate().map(|(i, label)| {
            FilterChip::new(eid("library-chip", k, i), *label)
                .selected(filters & (1 << i) != 0)
                .on_toggle(cx.listener(move |this, selected: &bool, _, cx| {
                    if *selected {
                        this.filters |= 1 << i;
                    } else {
                        this.filters &= !(1 << i);
                    }
                    cx.notify();
                }))
        }));
        let oldest_first = self.oldest_first;
        let list_view = self.list_view;
        let controls = div().px(rem(if c.expanded { 16. } else { 8. })).child(
            div()
                .py(rem(8.))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    material_button(eid("library-sort", k, 0), ButtonVariant::Text)
                        .icon(icon("swap_vert"))
                        .child(if oldest_first { "Oldest" } else { "Newest" })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.oldest_first = !this.oldest_first;
                            cx.notify();
                        })),
                )
                .child(
                    IconButton::new(
                        eid("library-view", k, 0),
                        icon(if list_view { "grid_view" } else { "list" }),
                        if list_view { "Grid view" } else { "List view" },
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.list_view = !this.list_view;
                        cx.notify();
                    })),
                ),
        );
        let count = if c.expanded { 20 } else { 12 };
        let items = library_items(filters, count, oldest_first);
        let body = if list_view {
            div()
                .pb(rem(32.))
                .flex()
                .flex_col()
                .when(c.expanded, |list| list.px(rem(8.)))
                .children(items.into_iter().map(|i| {
                    ListItem::new(eid("library-row", k, i), "Title")
                        .supporting(updated(i))
                        .leading(ListLeading::Image(None))
                        .trailing(ListTrailing::Icon(icon("chevron_right")))
                        .when(c.expanded, |item| item.container(pane()))
                        .on_click(self.to(Body::Detailed, cx))
                }))
                .into_any_element()
        } else {
            let gap = if c.expanded { 16. } else { 8. };
            let columns = if c.expanded {
                (((c.width - 2. * inset + gap) / (140. + gap)).floor() as u16).max(2)
            } else {
                3
            };
            div()
                .px(rem(inset))
                .pb(rem(32.))
                .grid()
                .grid_cols(columns)
                .gap(rem(gap))
                .children(items.into_iter().map(|i| {
                    pressable(
                        eid("library-card", k, i),
                        "Title",
                        div()
                            .w_full()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(rem(4.))
                            .child(art(56.).w_full().h(rem(120.)).rounded(rem(16.)))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(line(
                                        TypeRole::TitleSmall,
                                        ColorRole::ON_SURFACE,
                                        "Title",
                                    ))
                                    .child(line(
                                        TypeRole::BodySmall,
                                        ColorRole::ON_SURFACE_VARIANT,
                                        updated(i),
                                    )),
                            ),
                        self.to(Body::Detailed, cx),
                    )
                }))
                .into_any_element()
        };
        Screen {
            bar: Some((bar, room(variant))),
            content: div()
                .flex()
                .flex_col()
                .child(chips)
                .child(controls)
                .child(body)
                .into_any_element(),
            footer: None,
            overlay: None,
        }
    }

    /// A photo that a press picks.
    fn photo(
        &self,
        c: Ctx,
        i: usize,
        height: f32,
        glyph: f32,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let picked = self.picked == Some(i);
        pressable(
            eid("gallery-photo", c.k, i),
            "Photo",
            art(glyph)
                .relative()
                .w_full()
                .h(rem(height))
                .rounded(rem(12.))
                .when(picked, |photo| {
                    photo
                        .border_2()
                        .border_color(color(ColorRole::PRIMARY))
                        .child(
                            div()
                                .absolute()
                                .top(rem(8.))
                                .right(rem(8.))
                                .size(rem(24.))
                                .rounded_full()
                                .bg(color(ColorRole::PRIMARY))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    filled("check")
                                        .size(rem(18.))
                                        .color(color(ColorRole::ON_PRIMARY)),
                                ),
                        )
                }),
            Self::act(cx, move |this, _, _, cx| {
                this.picked = (this.picked != Some(i)).then_some(i);
                cx.notify();
            }),
        )
    }

    fn gallery(&self, c: Ctx, cx: &mut Context<Self>) -> Screen {
        let k = c.k;
        let variant = TopAppBarVariant::LargeFlexible;
        let bar = app_bar(c, "gallery-bar", variant, "Gallery")
            .navigation_icon(self.bar_menu(
                c,
                "gallery-account",
                "account_circle",
                "Account",
                &ACCOUNT,
                cx,
            ))
            .action(self.link(
                c,
                "gallery-notifications",
                "notifications",
                "Messages",
                Body::Messaging,
                cx,
            ))
            .action(self.notice(
                c,
                "gallery-settings",
                "settings",
                "Settings",
                "Settings",
                cx,
            ));
        let cell = |i: usize, basis: f32, height: f32, glyph: f32, cx: &mut Context<Self>| {
            div()
                .flex_basis(rem(basis))
                .flex_grow(1.)
                .flex_shrink(1.)
                .min_w_0()
                .h(rem(height))
                .child(self.photo(c, i, height, glyph, cx))
        };
        let content = if c.expanded {
            // Columns share the row in the kit's proportions.
            let pair = |first: usize, basis: f32, cx: &mut Context<Self>| {
                div()
                    .flex_basis(rem(basis))
                    .flex_grow(1.)
                    .flex_shrink(1.)
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(rem(4.))
                    .child(cell(first, basis, 176., 72., cx).w_full())
                    .child(cell(first + 1, basis, 176., 72., cx).w_full())
            };
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
                        .child(cell(0, 460., 356., 144., cx))
                        .child(
                            div()
                                .flex_basis(rem(305.))
                                .flex_grow(1.)
                                .flex_shrink(1.)
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .gap(rem(4.))
                                .child(
                                    div()
                                        .flex()
                                        .gap(rem(4.))
                                        .child(cell(1, 117., 176., 56., cx))
                                        .child(cell(2, 185., 176., 72., cx)),
                                )
                                .child(cell(3, 305., 176., 72., cx).w_full()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .gap(rem(4.))
                        .child(pair(4, 250., cx))
                        .child(pair(6, 370., cx))
                        .child(pair(8, 141., cx)),
                )
                .child(
                    div()
                        .flex()
                        .gap(rem(4.))
                        .child(pair(10, 383., cx))
                        .child(pair(12, 383., cx)),
                )
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_col()
                .child(
                    div().py(rem(4.)).flex().flex_wrap().gap(rem(4.)).children(
                        [412., 158., 250., 412., 250., 158.]
                            .into_iter()
                            .enumerate()
                            .map(|(i, width)| {
                                div()
                                    .w(rem(width))
                                    .h(rem(176.))
                                    .child(self.photo(c, i, 176., 72., cx))
                            }),
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
                                .children((0..9).map(|i| {
                                    tile(
                                        eid("gallery-album", k, i),
                                        false,
                                        "Label",
                                        self.to(Body::Detailed, cx),
                                    )
                                })),
                        ),
                )
                .into_any_element()
        };
        Screen {
            bar: Some((bar, room(variant))),
            content,
            footer: None,
            overlay: None,
        }
    }

    /// A row of reply suggestions; a press sends one.
    fn suggestions(&self, c: Ctx, cx: &mut Context<Self>) -> Div {
        div()
            .px(rem(16.))
            .pt(rem(16.))
            .pb(rem(12.))
            .flex()
            .flex_wrap()
            .justify_end()
            .gap(rem(8.))
            .children(SUGGESTIONS.into_iter().enumerate().map(|(i, reply)| {
                pressable(
                    eid("message-suggestion", c.k, i),
                    reply,
                    bubble(Bubble::Suggested, reply),
                    Self::act(cx, move |this, _, _, cx| this.send_text(reply, cx)),
                )
            }))
    }

    /// The message bar: attachments, emoji, the field and send.
    fn composer(&self, c: Ctx, cx: &mut Context<Self>) -> Div {
        let k = c.k;
        div()
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap(rem(if c.expanded { 8. } else { 4. }))
            .child(
                self.menu_of(
                    c,
                    "message-attach",
                    &ATTACH,
                    IconButton::new(
                        eid("message-attach", k, 0),
                        icon("add_circle"),
                        "Add attachment",
                    )
                    .on_click(self.menu_keys("message-attach", cx)),
                    cx,
                ),
            )
            .child(
                IconButton::new(eid("message-emoji", k, 0), icon("mood"), "Insert a smile")
                    .on_click(cx.listener(|this, _, window, cx| {
                        let message = this.message.clone();
                        message.update(cx, |input, cx| {
                            let text = format!("{}:)", input.value());
                            input.set_value(text, window, cx);
                        });
                    })),
            )
            .child(
                SearchField::new(&self.message)
                    .id(eid("message-field", k, 0))
                    .trailing(
                        IconButton::new(eid("message-send", k, 0), filled("send"), "Send")
                            .on_click(cx.listener(|this, _, window, cx| this.send(window, cx))),
                    )
                    .flex_1()
                    .min_w_0(),
            )
    }

    /// The messages of the selected conversation.
    fn sent_bubbles(&self) -> Div {
        div()
            .px(rem(16.))
            .flex()
            .flex_col()
            .items_end()
            .gap(rem(4.))
            .children(
                self.sent[self.conversation]
                    .iter()
                    .map(|message| bubble(Bubble::Sent, message.clone())),
            )
    }

    fn messaging_bar(&self, c: Ctx, title: &'static str, cx: &mut Context<Self>) -> TopAppBar {
        app_bar(c, "message-bar", TopAppBarVariant::Small, title)
            .navigation_icon(self.back_button(c, "message-back", cx))
            .action(self.bar_menu(
                c,
                "message-attach-bar",
                "attach_file",
                "Attach",
                &ATTACH,
                cx,
            ))
            .when(!c.expanded, |bar| {
                bar.action(self.mark(c, "message-favorite", "stars", "Favorite", cx))
            })
            .action(self.overflow(c, "message-more", cx))
    }

    /// The compact conversation.
    fn messaging(&self, c: Ctx, cx: &mut Context<Self>) -> Screen {
        let variant = TopAppBarVariant::Small;
        let bar = self.messaging_bar(c, "Name", cx);
        let content = div()
            .flex()
            .flex_col()
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
            .child(link_message())
            .child(received("Sounds good!"))
            .child(self.sent_bubbles())
            .child(self.suggestions(c, cx))
            .into_any_element();
        Screen {
            bar: Some((bar, room(variant))),
            content,
            footer: Some(
                self.composer(c, cx)
                    .p(rem(16.))
                    .rounded_t(rem(12.))
                    .bg(color(ColorRole::SURFACE_CONTAINER))
                    .into_any_element(),
            ),
            overlay: None,
        }
    }

    /// The expanded screen: the conversation list beside the conversation.
    fn messaging_panes(&mut self, c: Ctx, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let k = c.k;
        let selected = self.conversation;
        let conversation = Screen {
            bar: Some((self.messaging_bar(c, "Name", cx), 64.)),
            content: div()
                .flex()
                .flex_col()
                .child(link_message())
                .child(received("That looks so good!"))
                .child(self.sent_bubbles())
                .child(self.suggestions(c, cx))
                .into_any_element(),
            footer: Some(
                self.composer(c, cx)
                    .h(rem(80.))
                    .px(rem(16.))
                    .into_any_element(),
            ),
            overlay: None,
        };
        let scaffold = self.scaffold(conversation, Body::Messaging, c, cx);
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
                            .children((0..CONVERSATIONS).map(|i| {
                                ListItem::new(eid("conversation", k, i), CONVERSATION_NAMES[i])
                                    .supporting(SUPPORTING)
                                    .leading(ListLeading::Monogram(
                                        CONVERSATION_NAMES[i][..1].to_string().into(),
                                    ))
                                    .trailing_text("10 min")
                                    .selected(selected == i)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.conversation = i;
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
                            .flex()
                            .flex_col()
                            .rounded_t(rem(28.))
                            .overflow_hidden()
                            .bg(color(pane()))
                            .child(scaffold),
                    ),
            )
            .into_any_element()
    }
}

/// The places on the Reviews screen.
const PLACE_NAMES: [&str; 3] = ["Noodle house", "Corner bistro", "Harbor grill"];
/// The people on the Messaging screen.
const CONVERSATION_NAMES: [&str; CONVERSATIONS] = [
    "Ana", "Ben", "Chloe", "Dev", "Elena", "Farid", "Grace", "Hugo",
];

/// A received message: the sender's avatar and the bubble.
fn received(message: &'static str) -> Div {
    div()
        .px(rem(16.))
        .py(rem(8.))
        .flex()
        .items_center()
        .gap(rem(8.))
        .child(art(20.).size(rem(36.)).rounded_full())
        .child(bubble(Bubble::Received, message))
}

/// A sent link preview and its message.
fn link_message() -> Div {
    div()
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
        .child(bubble(Bubble::Sent, "or we could make this?"))
}
