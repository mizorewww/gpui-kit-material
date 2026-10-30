//! Examples: the kit's example layouts — Home, Upcoming, Detailed view,
//! Reviews, Library, Gallery and Messaging — as compact phone screens
//! (412dp) and expanded browser windows (905×680), each in its own row.
//! Every control is a crate component and the screens are live:
//! navigation bars and rails switch between the app's destinations, the
//! rail's menu button expands it and its FAB opens a menu, chips filter,
//! button groups switch, list items select, the player plays and pauses,
//! and the message bars send. Photos are the kit's tonal placeholders; the
//! kit's layout-grid overlay frames are not mirrored.
use super::prelude::*;
use gpui_base::input::{InputEvent, InputState};

mod parts;
mod screens;

use parts::{PHONE_WIDTH, phone};

/// An example app's screen content (everything between the system bars and
/// the navigation component).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    Home,
    Upcoming,
    Detailed,
    Reviews,
    Library,
    Gallery,
    Messaging,
}

/// One example frame: its content, form factor, caption and — for screens
/// with a navigation bar or rail — the destinations it switches between
/// (its own content first).
#[derive(Clone, Copy, Debug)]
struct Screen {
    body: Body,
    web: bool,
    caption: &'static str,
    destinations: Option<[Body; 3]>,
}

const fn mobile(body: Body, caption: &'static str, destinations: Option<[Body; 3]>) -> Screen {
    Screen {
        body,
        web: false,
        caption,
        destinations,
    }
}

const fn web(body: Body, caption: &'static str, destinations: Option<[Body; 3]>) -> Screen {
    Screen {
        body,
        web: true,
        caption,
        destinations,
    }
}

use Body::*;

/// The kit's order: the phone screens, then the browser windows.
const SCREENS: [Screen; 14] = [
    mobile(Home, "Home · compact", Some([Home, Library, Gallery])),
    mobile(Upcoming, "Upcoming · compact", None),
    mobile(Detailed, "Detailed view · compact", None),
    mobile(Reviews, "Reviews · compact", Some([Reviews, Home, Library])),
    mobile(Library, "Library · compact", Some([Library, Home, Gallery])),
    mobile(Gallery, "Gallery · compact", Some([Gallery, Home, Library])),
    mobile(Messaging, "Messaging · compact", None),
    web(
        Gallery,
        "Gallery · expanded",
        Some([Gallery, Home, Library]),
    ),
    web(Home, "Home · expanded", Some([Home, Library, Gallery])),
    web(
        Upcoming,
        "Upcoming · expanded",
        Some([Upcoming, Home, Library]),
    ),
    web(
        Library,
        "Library · expanded",
        Some([Library, Home, Gallery]),
    ),
    web(
        Detailed,
        "Detailed view · expanded",
        Some([Detailed, Home, Library]),
    ),
    web(
        Reviews,
        "Reviews · expanded",
        Some([Reviews, Home, Library]),
    ),
    web(Messaging, "Messaging · expanded", None),
];
const COUNT: usize = SCREENS.len();

/// The navigation destinations' values; the kit labels all three "Label".
const NAV_VALUES: [&str; 3] = ["0", "1", "2"];

/// Sent messages kept per conversation; older ones scroll away in a real
/// app, here they are dropped.
const MAX_SENT: usize = 3;

/// Where the rendering screen sits and what it is.
#[derive(Clone, Copy, Debug)]
pub struct Ctx {
    /// The screen's index, which namespaces its element ids.
    pub k: usize,
    pub web: bool,
    /// The content's width in dp: the phone, or the expanded pane beside
    /// the rail.
    pub width: f32,
}

impl Ctx {
    /// The content's side inset: 16dp compact, 24dp expanded.
    pub fn inset(self) -> f32 {
        if self.web { 24. } else { 16. }
    }
}

pub struct Page {
    page: PageCx,
    /// The selected destination of each screen.
    nav: [usize; COUNT],
    rail_expanded: [bool; COUNT],
    fab_open: [bool; COUNT],
    /// Selected filter chips (Library), one bit per chip.
    filters: [u16; COUNT],
    /// The selected button-group option (Reviews).
    segment: [usize; COUNT],
    /// The selected list item (Upcoming) or conversation (Messaging).
    selected: [Option<usize>; COUNT],
    /// Favorited places (Reviews), one bit per item.
    favorites: [u8; COUNT],
    playing: [bool; COUNT],
    /// The message bars of the two Messaging screens and what each sent.
    messages: [Entity<InputState>; 2],
    sent: [Vec<SharedString>; 2],
    _subscriptions: Vec<Subscription>,
}

/// The Messaging screens' indices in [`SCREENS`] (compact, expanded).
const MESSAGING: [usize; 2] = [6, 13];

impl Page {
    fn screen_body(&self, k: usize) -> Body {
        let screen = SCREENS[k];
        screen
            .destinations
            .map_or(screen.body, |destinations| destinations[self.nav[k]])
    }

    fn set_nav(&mut self, k: usize, value: &SharedString, cx: &mut Context<Self>) {
        let Some(index) = NAV_VALUES.iter().position(|v| *v == value.as_ref()) else {
            return;
        };
        if self.nav[k] != index {
            self.nav[k] = index;
            if !SCREENS[k].web {
                // Compact screens are as tall as their content.
                self.page.rows.remeasure(k..=k);
            }
            cx.notify();
        }
    }

    fn send(&mut self, which: usize, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.messages[which].clone();
        let message = input.read(cx).value().trim().to_string();
        if message.is_empty() {
            return;
        }
        input.update(cx, |input, cx| input.set_value("", window, cx));
        let sent = &mut self.sent[which];
        if sent.len() == MAX_SENT {
            sent.remove(0);
        }
        sent.push(message.into());
        let k = MESSAGING[which];
        if !SCREENS[k].web {
            self.page.rows.remeasure(k..=k);
        }
        cx.notify();
    }

    fn nav_bar(&self, k: usize, cx: &mut Context<Self>) -> NavigationBar {
        NavigationBar::new(parts::eid("nav-bar", k, 0), "Destinations")
            .items(screens::nav_items().iter().cloned())
            .selected(NAV_VALUES[self.nav[k]])
            .on_change(
                cx.listener(move |this, value: &SharedString, _, cx| this.set_nav(k, value, cx)),
            )
    }

    fn phone_screen(
        &mut self,
        k: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let screen = SCREENS[k];
        let body = self.screen_body(k);
        let c = Ctx {
            k,
            web: false,
            width: PHONE_WIDTH,
        };
        let has_nav = screen.destinations.is_some();
        // The gesture bar continues the surface-container of a navigation
        // bar, player or message bar above it.
        let gesture = if has_nav || matches!(body, Detailed | Messaging) {
            ColorRole::SURFACE_CONTAINER
        } else {
            ColorRole::SURFACE
        };
        let content = div()
            .w(rem(PHONE_WIDTH))
            .flex()
            .flex_col()
            .child(self.body(body, c, window, cx))
            .when(has_nav, |content| content.child(self.nav_bar(k, cx)));
        phone(parts::eid("phone", k, 0), content, gesture, body == Reviews).into_any_element()
    }

    fn web_screen(&mut self, k: usize, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let body = self.screen_body(k);
        let rail = if body == Messaging {
            0.
        } else if self.rail_expanded[k] {
            RAIL_EXPANDED_WIDTH
        } else {
            RAIL_COLLAPSED_WIDTH
        };
        // The kit's 905dp window, narrowed to fit the page: the Gallery
        // never scrolls sideways, and the screens' panes reflow.
        let web_width = parts::WEB_WIDTH.min(spec::content_width(window) - 2.);
        let c = Ctx {
            k,
            web: true,
            // The pane keeps a 16dp margin to the window's trailing edge.
            width: web_width - rail - 16.,
        };
        let content = if body == Messaging {
            self.body(body, c, window, cx)
        } else {
            div()
                .size_full()
                .flex()
                .child(self.rail(k, cx))
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
                                .relative()
                                .flex_1()
                                .min_h_0()
                                .rounded_t(rem(28.))
                                .overflow_hidden()
                                .bg(color(ColorRole::SURFACE_CONTAINER_LOWEST))
                                .child(
                                    div()
                                        .id(parts::eid("pane", k, 0))
                                        .size_full()
                                        .overflow_y_scroll()
                                        .child(self.body(body, c, window, cx)),
                                )
                                .children(self.pane_overlay(body, c, window, cx)),
                        ),
                )
                .into_any_element()
        };
        parts::browser(parts::eid("browser", k, 0), web_width, content).into_any_element()
    }

    /// The expanded screens' navigation rail: a menu button that expands
    /// it, a FAB (extended while expanded) that opens a compose menu, and
    /// the destinations.
    fn rail(&self, k: usize, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let expanded = self.rail_expanded[k];
        let open = self.fab_open[k];
        let menu = MaterialMenu::new(parts::eid("compose-menu", k, 0))
            .width(200.)
            .item(
                MaterialMenuItem::new("Note")
                    .leading(parts::icon("edit_note"))
                    .on_click(close_menu(k, cx)),
            )
            .item(
                MaterialMenuItem::new("Checklist")
                    .leading(parts::icon("checklist"))
                    .on_click(close_menu(k, cx)),
            )
            .item(
                MaterialMenuItem::new("Photo")
                    .leading(parts::icon("image"))
                    .on_click(close_menu(k, cx)),
            );
        let fab = MenuPopup::new(
            parts::eid("compose", k, 0),
            ExtendedFab::new(parts::eid("fab", k, 0), parts::filled("edit"), "Compose")
                .expanded(expanded)
                .on_click(cx.listener(move |this, event: &ClickEvent, _, cx| {
                    // Pointer presses reach the popover; keys arrive here.
                    if matches!(event, ClickEvent::Keyboard(_)) {
                        this.fab_open[k] = !open;
                        cx.notify();
                    }
                })),
            menu,
        )
        .open(open)
        .on_open_change(cx.listener(move |this, open: &bool, _, cx| {
            this.fab_open[k] = *open;
            cx.notify();
        }));
        div().h_full().flex_shrink_0().child(
            NavigationRail::new(parts::eid("rail", k, 0), "Destinations")
                .items(screens::nav_items().iter().cloned())
                .selected(NAV_VALUES[self.nav[k]])
                .expanded(expanded)
                .menu(
                    IconButton::new(
                        parts::eid("rail-menu", k, 0),
                        parts::icon(if expanded { "menu_open" } else { "menu" }),
                        if expanded {
                            "Collapse navigation"
                        } else {
                            "Expand navigation"
                        },
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.rail_expanded[k] = !expanded;
                        cx.notify();
                    })),
                )
                .fab(fab)
                .on_change(
                    cx.listener(move |this, value: &SharedString, _, cx| {
                        this.set_nav(k, value, cx)
                    }),
                ),
        )
    }
}

/// A compose-menu item's action: the item's choice would open its editor;
/// here it closes the menu.
fn close_menu(
    k: usize,
    cx: &mut Context<Page>,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    cx.listener(move |this, _: &ClickEvent, _, cx| {
        this.fab_open[k] = false;
        cx.notify();
    })
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "examples",
        title: "Examples",
        description: "Example layouts are meant to be a starting point, not a final destination. Feel free to modify them, experiment, and add your own unique touch to create a design that is truly your own.",
        guideline: Some(
            "https://m3.material.io/foundations/layout/applying-layout/window-size-classes",
        ),
    };

    fn new(page: PageCx, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let messages = [(); 2]
            .map(|_| cx.new(|cx| InputState::new(window, cx).placeholder("Hinted search text")));
        let _subscriptions = messages
            .iter()
            .enumerate()
            .map(|(which, input)| {
                cx.subscribe_in(
                    input,
                    window,
                    move |this, _, event: &InputEvent, window, cx| {
                        if matches!(event, InputEvent::PressEnter { .. }) {
                            this.send(which, window, cx);
                        }
                    },
                )
            })
            .collect();
        let mut selected = [None; COUNT];
        selected[MESSAGING[1]] = Some(0);
        Self {
            page,
            nav: [0; COUNT],
            rail_expanded: [false; COUNT],
            fab_open: [false; COUNT],
            // The kit selects the second chip.
            filters: [0b10; COUNT],
            segment: [0; COUNT],
            selected,
            favorites: [0; COUNT],
            playing: [true; COUNT],
            messages,
            sent: [Vec::new(), Vec::new()],
            _subscriptions,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        COUNT
    }

    fn interactive(&self, _: usize) -> bool {
        true
    }

    fn render_row(
        &mut self,
        row: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let screen = SCREENS[row];
        let frame = if screen.web {
            self.web_screen(row, window, cx)
        } else {
            self.phone_screen(row, window, cx)
        };
        spec::demo(
            ("example", row),
            screen.caption,
            div().w_full().flex().child(frame),
        )
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn every_kit_example_appears_once_per_form_factor() {
        for web in [false, true] {
            let mut bodies: Vec<_> = SCREENS
                .iter()
                .filter(|screen| screen.web == web)
                .map(|screen| format!("{:?}", screen.body))
                .collect();
            bodies.sort();
            assert_eq!(
                bodies,
                [
                    "Detailed",
                    "Gallery",
                    "Home",
                    "Library",
                    "Messaging",
                    "Reviews",
                    "Upcoming"
                ]
            );
        }
    }

    #[test]
    fn destinations_start_with_the_screen_and_are_distinct() {
        for screen in SCREENS {
            if let Some(destinations) = screen.destinations {
                assert_eq!(destinations[0], screen.body, "{}", screen.caption);
                assert!(
                    destinations[0] != destinations[1]
                        && destinations[1] != destinations[2]
                        && destinations[0] != destinations[2],
                    "{}",
                    screen.caption
                );
            }
        }
    }

    #[test]
    fn expanded_screens_navigate_with_a_rail_except_messaging() {
        for screen in SCREENS.iter().filter(|screen| screen.web) {
            assert_eq!(
                screen.destinations.is_some(),
                screen.body != Messaging,
                "{}",
                screen.caption
            );
        }
    }

    #[test]
    fn messaging_indices_point_at_the_messaging_screens() {
        assert_eq!(SCREENS[MESSAGING[0]].body, Messaging);
        assert!(!SCREENS[MESSAGING[0]].web);
        assert_eq!(SCREENS[MESSAGING[1]].body, Messaging);
        assert!(SCREENS[MESSAGING[1]].web);
    }

    type GalleryWindow = WindowHandle<super::super::super::Gallery>;

    fn open(cx: &mut TestAppContext) -> (GalleryWindow, Entity<Page>, usize) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let window = cx.open_window(size(px(1280.), px(900.)), super::super::super::Gallery::new);
        let (page, first) = window
            .update(cx, |gallery, window, cx| {
                gallery.show_page::<Page>(window, cx)
            })
            .unwrap();
        (window, page, first)
    }

    fn draw(window: &GalleryWindow, cx: &mut TestAppContext) {
        cx.update_window((*window).into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    }

    fn row_height(window: &GalleryWindow, cx: &mut TestAppContext, row: usize) -> Pixels {
        window
            .update(cx, |gallery, _, cx| {
                gallery.list.scroll_to(ListOffset {
                    item_ix: row,
                    offset_in_item: px(0.),
                });
                cx.notify();
            })
            .unwrap();
        draw(window, cx);
        window
            .update(cx, |gallery, _, _| gallery.list.bounds_for_item(row))
            .unwrap()
            .expect("the screen's row is mounted")
            .size
            .height
    }

    #[gpui::test]
    fn a_phone_destination_switch_shows_that_destination_and_remeasures(cx: &mut TestAppContext) {
        let (window, page, first) = open(cx);
        draw(&window, cx);
        let home = row_height(&window, cx, first);
        page.update(cx, |page, cx| page.set_nav(0, &"1".into(), cx));
        assert_eq!(page.read_with(cx, |page, _| page.screen_body(0)), Library);
        let library = row_height(&window, cx, first);
        assert!(
            library < home - px(400.),
            "the shorter Library screen replaces Home ({home:?} → {library:?})"
        );
        // An unknown value leaves the destination alone.
        page.update(cx, |page, cx| page.set_nav(0, &"9".into(), cx));
        assert_eq!(page.read_with(cx, |page, _| page.nav[0]), 1);
    }

    #[gpui::test]
    fn the_message_bar_sends_and_clears(cx: &mut TestAppContext) {
        let (window, page, _) = open(cx);
        draw(&window, cx);
        for message in ["One", "  ", "Two", "Three", "Four"] {
            window
                .update(cx, |_, window, cx| {
                    page.update(cx, |page, cx| {
                        page.messages[0]
                            .update(cx, |input, cx| input.set_value(message, window, cx));
                        page.send(0, window, cx);
                    })
                })
                .unwrap();
        }
        page.read_with(cx, |page, cx| {
            // Blank messages are not sent; only the latest are kept.
            assert_eq!(page.sent[0], ["Two", "Three", "Four"]);
            assert!(page.sent[1].is_empty());
            assert_eq!(page.messages[0].read(cx).value(), "");
        });
    }
}
