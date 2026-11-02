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
use gpui_base::input::{InputEvent, InputState, MoveDown};
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

/// Switch the Gallery's theme mode, keeping the palette or theme it paints.
pub(crate) fn set_dark(shared: &Entity<GalleryShared>, dark: bool, cx: &mut App) {
    shared.update(cx, |shared, cx| {
        shared.set_dark(dark);
        cx.notify();
    })
}

/// Makes a control in the title bar its own: [`title_bar_control`] takes its
/// left presses, and this also takes its right presses, which the bar would
/// answer with the system's window menu, and occludes the bar beneath it. The
/// bar is a drag area, which Windows hit-tests as the caption wherever the
/// pointer reaches it: a field there could not be drag-selected.
pub(crate) fn bar_control<E: InteractiveElement>(element: E) -> E {
    title_bar_control(element)
        .occlude()
        .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
}

pub(crate) struct TopBar {
    gallery: Entity<Gallery>,
    shared: Entity<GalleryShared>,
    pub(crate) search: Entity<InputState>,
    /// What the bar shows of the Gallery.
    pub(crate) shown: Shown,
    /// Whether the compact bar has traded its brand for the search field.
    pub(crate) search_open: bool,
    /// Whether the last frame was compact: a query typed in the medium layout
    /// opens the search when the window narrows, as the results are in the
    /// drawer.
    was_compact: bool,
    /// The compact search toggle, which closing the search gives focus back to.
    pub(crate) toggle_focus: FocusHandle,
    /// Whether focus the closed search's drawer gives back goes to the toggle.
    restore_toggle: bool,
    /// Whether focus last left the drawer, which gives it back to the field.
    from_drawer: bool,
    /// Keeps Tab in the bar while the compact search's results cover the page.
    trap_focus: FocusHandle,
    /// A caption style forced by a capture or test, which the bar leaves room for.
    caption: Option<Caption>,
    _subscriptions: Vec<Subscription>,
}

/// What the bar shows of the Gallery: its section, the mode it paints, and
/// whether the compact drawer covers the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Shown {
    section: TopSection,
    pub(crate) mode: ThemeMode,
    drawer_open: bool,
}

fn shown(gallery: &Entity<Gallery>, cx: &App) -> Shown {
    let gallery = gallery.read(cx);
    Shown {
        section: catalog::CATALOG[gallery.current].group.section(),
        mode: gallery.shared.read(cx).painted_mode(),
        drawer_open: gallery.drawer_open,
    }
}

impl TopBar {
    pub(crate) fn new(
        gallery: Entity<Gallery>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search"));
        let shared = gallery.read(cx).shared.clone();
        let search_focus = search.read(cx).focus_handle(cx);
        gallery.update(cx, |gallery, _| gallery.search_focus = Some(search_focus));
        let subscriptions = vec![
            cx.subscribe_in(&search, window, |this: &mut Self, search, event, _, cx| {
                match event {
                    InputEvent::Change => {
                        let query = search.read(cx).value();
                        this.gallery
                            .update(cx, |gallery, cx| gallery.set_query(query, cx));
                    }
                    InputEvent::PressEnter { .. } => {
                        this.gallery
                            .update(cx, |gallery, cx| gallery.open_first_match(cx));
                    }
                    // A compact window shows the results the query still
                    // has, unless focus is coming back from them: the
                    // drawer gives it back as it closes.
                    InputEvent::Focus => {
                        if !std::mem::take(&mut this.from_drawer) {
                            this.gallery
                                .update(cx, |gallery, cx| gallery.show_results(cx));
                        }
                    }
                    InputEvent::Blur => {}
                }
            }),
            // Focus leaving the drawer (a result, or the holder a scrim
            // press focuses) comes back to the field as the drawer closes,
            // which is no reason to show the results again.
            cx.on_focus_out(
                &gallery.read(cx).drawer_focus.clone(),
                window,
                |this: &mut Self, _, _, _| this.from_drawer = true,
            ),
            // Closing the search gives focus back to its toggle. The drawer
            // gives it back to the field it took it from once it closes, which
            // the closed search no longer shows.
            cx.on_focus_lost(window, |this: &mut Self, window, cx| {
                if std::mem::take(&mut this.restore_toggle) {
                    this.toggle_focus.focus(window, cx);
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
            was_compact: false,
            toggle_focus: cx.focus_handle(),
            restore_toggle: false,
            from_drawer: false,
            trap_focus: cx.focus_handle(),
            caption: None,
            _subscriptions: subscriptions,
        }
    }

    /// Leave room for a forced caption style's buttons.
    pub(crate) fn caption(mut self, caption: Option<Caption>) -> Self {
        self.caption = caption;
        self
    }

    fn has_query(&self, cx: &App) -> bool {
        !self.search.read(cx).value().trim().is_empty()
    }

    /// Open the compact search and focus its field.
    fn open_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.search_open = true;
        self.restore_toggle = false;
        self.search
            .update(cx, |search, cx| search.focus(window, cx));
        cx.notify();
    }

    /// Close the compact search, which ends the filter as well, and give focus
    /// back to its toggle.
    pub(crate) fn close_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.search_open = false;
        self.clear_query(window, cx);
        self.toggle_focus.focus(window, cx);
        self.restore_toggle = true;
        cx.notify();
    }

    /// Down in the compact search: to the first result, showing the
    /// results first if they were dismissed.
    fn down_to_results(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.search_open || ShellLayout::of(window) != ShellLayout::Compact {
            return;
        }
        let gallery = self.gallery.read(cx);
        if gallery.drawer_open {
            let container = gallery.drawer_focus.clone();
            cx.stop_propagation();
            NavigationDrawer::focus_first(&container, window, cx);
        } else if self.has_query(cx) {
            cx.stop_propagation();
            self.gallery
                .update(cx, |gallery, cx| gallery.show_results(cx));
        }
    }

    fn clear_query(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.search
            .update(cx, |search, cx| search.set_value("", window, cx));
        self.gallery.update(cx, |gallery, cx| {
            gallery.set_query(SharedString::default(), cx);
            gallery.set_drawer_open(false, cx);
        });
    }
}

impl Render for TopBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Shown {
            section,
            mode,
            drawer_open,
        } = self.shown;
        let dark = mode == ThemeMode::Dark;
        let compact = ShellLayout::of(window) == ShellLayout::Compact;
        // The compact bar shows a query typed before the window narrowed.
        if compact && !self.was_compact && self.has_query(cx) {
            self.search_open = true;
        }
        self.was_compact = compact;
        let width = shell_width(window);
        let wordmark = width
            >= if compact {
                COMPACT_WORDMARK
            } else {
                MEDIUM_WORDMARK
            };
        let searching = compact && self.search_open;
        let fit = CompactBar::of(super::title_bar_height(window), self.caption, window);
        let field = bar_control(
            div()
                .id("top-search")
                .debug_selector(|| "top-search".into())
                // Down goes to the compact search's results, before the
                // single-line field takes it.
                .capture_action(
                    cx.listener(|this, _: &MoveDown, window, cx| this.down_to_results(window, cx)),
                )
                // Escape closes the compact search, or clears the query.
                .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                    if event.keystroke.key != "escape" {
                        return;
                    }
                    if this.search_open && ShellLayout::of(window) == ShellLayout::Compact {
                        cx.stop_propagation();
                        this.close_search(window, cx);
                    } else if this.has_query(cx) {
                        cx.stop_propagation();
                        this.clear_query(window, cx);
                    }
                }))
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
        let bar = div()
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
                    bar_control(
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
                    bar_control(
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
                            .track_focus(&self.toggle_focus)
                            .on_click(cx.listener(
                                |this, _, window, cx| {
                                    if this.search_open {
                                        this.close_search(window, cx);
                                    } else {
                                        this.open_search(window, cx);
                                    }
                                },
                            )),
                        ),
                    ),
                    window,
                ))
            })
            .when(!searching && (!compact || fit.mode), |bar| {
                let toggle = bar_control(
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
            });
        // The results cover the page: Tab stays with the field and its toggle.
        if searching && drawer_open {
            gpui_base::FocusTrapElement::focus_trap(
                bar.track_focus(&self.trap_focus).tab_stop(false),
                "top-search-trap",
                &self.trap_focus,
            )
            .into_any_element()
        } else {
            bar.into_any_element()
        }
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
                bar_control(
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

#[cfg(test)]
mod tests {
    use super::bar_control;
    use core::prelude::v1::test;
    use gpui::{prelude::*, *};
    use std::{cell::Cell, rc::Rc};

    /// A bar that records its right presses and hover, around one control.
    struct Bar {
        right: Rc<Cell<bool>>,
        hovered: Rc<Cell<bool>>,
    }

    impl Render for Bar {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let (right, hovered) = (self.right.clone(), self.hovered.clone());
            div()
                .id("bar")
                .size_full()
                .on_mouse_down(MouseButton::Right, move |_, _, _| right.set(true))
                .on_hover(move |over, _, _| hovered.set(*over))
                .child(bar_control(
                    div()
                        .id("control")
                        .debug_selector(|| "control".into())
                        .size(px(40.)),
                ))
        }
    }

    /// A title-bar control takes its right presses, which would open the
    /// window menu, and hides the bar beneath it from the pointer, which
    /// Windows would hit-test as the caption.
    #[gpui::test]
    fn a_bar_control_takes_its_presses_and_hides_the_bar(cx: &mut TestAppContext) {
        let (right, hovered) = (Rc::new(Cell::new(false)), Rc::new(Cell::new(false)));
        let bar = Bar {
            right: right.clone(),
            hovered: hovered.clone(),
        };
        let (_, cx) = cx.add_window_view(|_, _| bar);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let control = cx.debug_bounds("control").unwrap();
        let outside = point(px(200.), px(200.));
        cx.simulate_mouse_move(outside, None, Modifiers::none());
        assert!(hovered.get(), "the bar's own space is the bar's");
        cx.simulate_mouse_move(control.center(), None, Modifiers::none());
        assert!(!hovered.get(), "the control occludes the bar");
        cx.simulate_mouse_down(control.center(), MouseButton::Right, Modifiers::none());
        assert!(!right.get(), "the control takes its right press");
        cx.simulate_mouse_down(outside, MouseButton::Right, Modifiers::none());
        assert!(right.get());
    }
}
