//! The shared desktop/WASM gallery, organized like md3.io's component
//! pages: a top app bar (sections, search, light/dark), navigation by
//! component group, and for each component a Live preview | Specs card
//! beside a configuration card (`playground`). Only the current page is
//! built into the content list; every page keeps its state (configuration,
//! tab, scroll position) across navigation. See `catalog` for the pages
//! and `playground` for how to write a component playground.
//!
//! `cargo run -p gpui-kit-material --example gallery`
#[cfg(not(target_family = "wasm"))]
pub mod capture;
#[cfg(all(not(target_family = "wasm"), feature = "frame-profiler"))]
mod interactions;
#[cfg(all(not(target_family = "wasm"), feature = "frame-profiler"))]
mod performance;
#[cfg(all(not(target_family = "wasm"), feature = "frame-profiler"))]
mod scroll_performance;

pub(crate) mod catalog;
mod fps;
mod overview;
pub(crate) mod page;
mod pages;
pub(crate) mod playground;
mod playgrounds;
mod settings;
mod shell;
#[cfg(test)]
mod shell_tests;
pub(crate) mod spec;
#[cfg(test)]
mod styles_tests;
mod top_bar;

use gpui::{prelude::*, *};
use gpui_kit_material::icons::Assets;
use gpui_kit_material::prelude::*;
use gpui_kit_material::theme;
use page::{AnyPage, CaptureState, GalleryShared, PageCx, PageRows};
use playground::{PanelView, PlaygroundState, PreviewView, Value};
use std::{cell::RefCell, rc::Rc};

// Reveal only the newly focused control, not the entire long gallery page.
pub(crate) trait GalleryReveal: IntoElement + Sized {
    fn reveal(self, id: impl Into<ElementId>, list: &ListState) -> RevealFocus {
        RevealFocus::in_list(id, self, list.clone()).inline()
    }
    fn reveal_handle(self, id: impl Into<ElementId>, scroll: &ScrollHandle) -> RevealFocus {
        RevealFocus::new(id, self, scroll.clone()).inline()
    }
}
impl<T: IntoElement> GalleryReveal for T {}

/// The Gallery's names for the caption buttons.
fn caption_labels() -> CaptionLabels {
    CaptionLabels {
        minimize: "Minimize".into(),
        maximize: "Maximize".into(),
        restore: "Restore".into(),
        close: "Close".into(),
    }
}

/// Fixed height keeps the native traffic lights centered at every text scale.
const TITLE_BAR: Pixels = px(56.);

/// The settings whose change starts a theme transition.
type ThemeKey = (
    PaletteId,
    Option<ThemeId>,
    ThemeMode,
    Contrast,
    SharedString,
);
fn theme_key(shared: &GalleryShared) -> ThemeKey {
    (
        shared.palette,
        shared.reference_theme,
        shared.reference_mode,
        shared.contrast,
        shared.scale.clone(),
    )
}

/// A component page's card tab.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Preview,
    Specs,
}

/// What one content list row shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Row {
    /// The page header, and on component pages the card (preview tab) or
    /// the card's top (Specs tab).
    Lead,
    /// A body row of the entry's spec page.
    Spec(usize),
    /// An overview group of component cards.
    Group(catalog::Group),
}

/// A built design-kit page.
struct BuiltPage {
    page: Box<dyn AnyPage>,
    rows: PageRows,
    _observer: Subscription,
}

/// A playground's views, held here so drawing the lead row need not read the
/// state its preview writes.
#[derive(Clone)]
struct PlaygroundViews {
    preview: Entity<PreviewView>,
    panel: Entity<PanelView>,
}

/// Per catalog entry state kept across navigation.
#[derive(Default)]
struct EntryState {
    tab: Tab,
    scroll: Option<ListOffset>,
    /// The scroll offset of a page shown without the content list.
    lead_offset: Point<Pixels>,
    playground: Option<Entity<PlaygroundState>>,
    /// The playground's preview and configuration views (see
    /// `playground::PreviewView`), built with the state.
    views: Option<PlaygroundViews>,
    /// The entry's slice of its spec page, resolved once.
    slice: Option<Rc<[usize]>>,
}

/// Which rows the content list holds: rebuilt only when this changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Layout {
    entry: usize,
    tab: Tab,
    spec_rows: usize,
}

pub(crate) struct Gallery {
    shared: Entity<GalleryShared>,
    applied_theme: ThemeKey,
    /// Design-kit pages by `PAGES` index, built when first shown.
    pages: Vec<Option<BuiltPage>>,
    entries: Vec<EntryState>,
    /// The current catalog entry.
    current: usize,
    /// The navigation filter typed into the top bar's search.
    query: SharedString,
    list: ListState,
    /// Scrolls a page whose only row is its lead (see [`Self::shows_lead_only`]).
    lead_scroll: ScrollHandle,
    rows: Vec<Row>,
    layout: Option<Layout>,
    row_focus: Vec<FocusHandle>,
    navigation_focus: FocusHandle,
    navigation_scroll: ScrollHandle,
    pending_focus: Option<(usize, bool)>,
    last_rem: Pixels,
    theme_frame_pending: bool,
    theme_menu_open: bool,
    capture: CaptureState,
    /// The rows the list builder rendered in the current frame.
    rendered_rows: Rc<RefCell<Vec<usize>>>,
    /// The lead row (header and the preview and configuration cards) as a
    /// view: see [`shell::LeadView`].
    lead: Entity<shell::LeadView>,
    _subscriptions: Vec<Subscription>,
    #[cfg(test)]
    renders: std::cell::Cell<usize>,
}

impl Gallery {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let shared = cx.new(|_| GalleryShared::default());
        let subscriptions =
            vec![
                cx.observe_in(&shared, window, |this: &mut Self, shared, window, cx| {
                    let key = theme_key(shared.read(cx));
                    if key != this.applied_theme {
                        // The transition refreshes the window each step, which
                        // renders the Gallery; a notify would invalidate it twice.
                        this.applied_theme = key;
                        this.prepare_theme(window, cx);
                    } else {
                        cx.notify();
                    }
                }),
            ];
        let applied_theme = theme_key(shared.read(cx));
        let gallery = cx.entity();
        let lead = cx.new(|cx| shell::LeadView::new(gallery, cx));
        Gallery {
            lead,
            shared,
            applied_theme,
            pages: (0..pages::PAGES.len()).map(|_| None).collect(),
            entries: (0..catalog::CATALOG.len())
                .map(|_| EntryState::default())
                .collect(),
            current: 0,
            query: SharedString::default(),
            list: ListState::new(0, ListAlignment::Top, px(256.)),
            lead_scroll: ScrollHandle::new(),
            rows: Vec::new(),
            layout: None,
            row_focus: Vec::new(),
            navigation_focus: cx.focus_handle(),
            navigation_scroll: ScrollHandle::new(),
            pending_focus: None,
            last_rem: window.rem_size(),
            theme_frame_pending: false,
            theme_menu_open: false,
            capture: CaptureState::default(),
            rendered_rows: Default::default(),
            _subscriptions: subscriptions,
            #[cfg(test)]
            renders: Default::default(),
        }
    }

    // Run before Root prepares its styles, never while rendering the child view.
    fn prepare_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let shared = self.shared.read(cx);
        gpui_kit_material::motion::set_motion_scheme(shared.motion_scheme);
        let (scheme, scale) = (shared.target_scheme(), shared.text_scale());
        let running = theme::advance_component_theme(scheme, scale, window, cx);
        if running && !self.theme_frame_pending {
            self.theme_frame_pending = true;
            let view = cx.weak_entity();
            window.on_next_frame(move |window, cx| {
                // advance_component_theme refreshes the window for each step;
                // notifying the Gallery as well would invalidate it twice.
                let _ = view.update(cx, |this, cx| {
                    this.theme_frame_pending = false;
                    this.prepare_theme(window, cx);
                });
            });
        }
    }

    /// The `PAGES` index of the current entry's spec page.
    fn current_page(&self) -> Option<usize> {
        catalog::spec_page(self.current)
    }

    /// Whether page `index` (in `PAGES`) is on screen.
    pub(crate) fn shows_page(&self, index: usize) -> bool {
        self.current_page() == Some(index)
    }

    /// Build page `index` if it has not been shown yet.
    fn ensure_page(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.pages[index].is_some() {
            return;
        }
        let rows = PageRows::new(self.list.clone());
        let page = (pages::PAGES[index].build)(
            PageCx {
                shared: self.shared.clone(),
                rows: rows.clone(),
            },
            window,
            cx,
        );
        page.apply_capture(self.capture, cx);
        let observer = page.observe(index, cx);
        self.pages[index] = Some(BuiltPage {
            page,
            rows,
            _observer: observer,
        });
    }

    /// The current entry's playground state, built when first shown.
    fn ensure_playground(
        &mut self,
        entry: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Entity<PlaygroundState>> {
        let catalog::Kind::Component {
            playground: Some(playground),
        } = catalog::CATALOG[entry].kind
        else {
            return None;
        };
        if self.entries[entry].playground.is_none() {
            // The preview and panel views observe the state themselves; the
            // Gallery does not re-render for a playground change.
            let state = cx.new(|cx| PlaygroundState::new(playground, window, cx));
            let preview = cx.new(|cx| PreviewView::new(state.clone(), cx));
            let panel = cx.new(|cx| PanelView::new(state.clone(), cx));
            self.entries[entry].views = Some(PlaygroundViews { preview, panel });
            self.entries[entry].playground = Some(state);
        }
        self.entries[entry].playground.clone()
    }

    /// The spec page rows the current entry shows.
    fn spec_rows(&mut self, window: &Window, cx: &App) -> Rc<[usize]> {
        let entry = self.current;
        let spec = catalog::CATALOG[entry].spec.expect("an entry with specs");
        if let Some(rows) = spec.rows {
            return self.entries[entry]
                .slice
                .get_or_insert_with(|| rows().into())
                .clone();
        }
        let count = self.current_page().map_or(0, |index| {
            self.pages[index]
                .as_ref()
                .map_or(0, |built| built.page.row_count(window, cx))
        });
        match &self.entries[entry].slice {
            Some(slice) if slice.len() == count => slice.clone(),
            _ => {
                let rows: Rc<[usize]> = (0..count).collect();
                self.entries[entry].slice = Some(rows.clone());
                rows
            }
        }
    }

    /// Switch to entry `entry`, keeping the current entry's scroll position.
    pub(crate) fn navigate(&mut self, entry: usize, cx: &mut Context<Self>) {
        if entry == self.current {
            return;
        }
        self.entries[self.current].scroll = Some(self.list.logical_scroll_top());
        self.entries[self.current].lead_offset = self.lead_scroll.offset();
        self.current = entry;
        self.lead_scroll.set_offset(self.entries[entry].lead_offset);
        self.theme_menu_open = false;
        cx.notify();
    }

    /// Whether the current page is only its lead row: a component's preview
    /// tab. It is drawn in a plain scrolling element, not the content list.
    /// A dirty view in a list row cannot be spliced from the views around the
    /// list, as a row is a separate layout root (`Window::splice_layout`), so
    /// every animation frame in a preview would build the Gallery, its
    /// navigation and the row again.
    pub(crate) fn shows_lead_only(&self) -> bool {
        self.rows.len() == 1 && matches!(self.rows.first(), Some(Row::Lead))
    }

    /// Show `tab` on the current component page.
    pub(crate) fn set_tab(&mut self, tab: Tab, cx: &mut Context<Self>) {
        let state = &mut self.entries[self.current];
        if state.tab != tab {
            state.tab = tab;
            state.scroll = None;
            cx.notify();
        }
    }

    pub(crate) fn set_query(&mut self, query: SharedString, cx: &mut Context<Self>) {
        if self.query != query {
            self.query = query;
            cx.notify();
        }
    }

    /// Rebuild the content list when the page, tab or row count changed.
    fn sync_layout(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let entry = self.current;
        let info = &catalog::CATALOG[entry];
        if let Some(index) = self.current_page() {
            self.ensure_page(index, window, cx);
        }
        self.ensure_playground(entry, window, cx);
        let tab = self.entries[entry].tab;
        let with_specs = match info.kind {
            catalog::Kind::Overview => false,
            catalog::Kind::Document => true,
            catalog::Kind::Component { .. } => tab == Tab::Specs,
        };
        let spec_rows = if info.spec.is_some() {
            self.spec_rows(window, cx)
        } else {
            Rc::from([])
        };
        let layout = Layout {
            entry,
            tab,
            spec_rows: if with_specs { spec_rows.len() } else { 0 },
        };
        if self.layout == Some(layout) {
            return;
        }
        let previous = self.layout.replace(layout);
        let same_page = previous
            .is_some_and(|previous| previous.entry == layout.entry && previous.tab == layout.tab);
        let top = self.list.logical_scroll_top();
        self.rows.clear();
        self.rows.push(Row::Lead);
        match info.kind {
            catalog::Kind::Overview => self.rows.extend(shell::overview_groups().map(Row::Group)),
            _ if with_specs => self
                .rows
                .extend(spec_rows.iter().map(|row| Row::Spec(*row))),
            _ => {}
        }
        // Map the shown page's rows; every other built page has none.
        for (index, built) in self.pages.iter().enumerate() {
            if let Some(built) = built {
                if Some(index) == self.current_page() && with_specs {
                    built
                        .rows
                        .show(spec_rows.iter().enumerate().map(|(i, row)| (*row, i + 1)));
                } else {
                    built.rows.show([]);
                }
            }
        }
        self.row_focus = self.rows.iter().map(|_| cx.focus_handle()).collect();
        self.list.reset(0);
        self.list
            .splice_focusable(0..0, self.row_focus.iter().cloned().map(Some));
        let restore = if same_page {
            // A width-dependent row count changed: stay where the reader is.
            Some(ListOffset {
                item_ix: top.item_ix.min(self.rows.len() - 1),
                offset_in_item: top.offset_in_item,
            })
        } else {
            self.entries[entry].scroll.take()
        };
        self.list.scroll_to(restore.unwrap_or(ListOffset {
            item_ix: 0,
            offset_in_item: px(0.),
        }));
    }

    /// Show the page holding design-kit page `P` (its Specs on a component
    /// page) and return the page and the list row of its first body row.
    #[cfg(test)]
    pub(crate) fn show_page<P: page::GalleryPage>(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> (Entity<P>, usize) {
        let (entry, _) = catalog::resolve(P::INFO.id).expect("a catalog entry shows the page");
        self.navigate(entry, cx);
        self.entries[entry].tab = Tab::Specs;
        self.sync_layout(window, cx);
        let index = self.current_page().expect("the entry has specs");
        let built = self.pages[index].as_ref().expect("the page is built");
        let first = self
            .rows
            .iter()
            .position(|row| matches!(row, Row::Spec(_)))
            .unwrap();
        (built.page.any_entity().downcast::<P>().unwrap(), first)
    }

    /// Apply capture overrides to the settings and pages.
    #[cfg_attr(target_family = "wasm", allow(dead_code))]
    pub(crate) fn apply_capture(
        &mut self,
        entry: usize,
        tab: Tab,
        dark: bool,
        scale: f32,
        state: CaptureState,
        cx: &mut Context<Self>,
    ) {
        self.shared.update(cx, |shared, _| {
            shared.reference_mode = if dark {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            };
            shared.scale = scale.to_string().into();
        });
        // Capture applies the theme before the first frame (see `run`).
        self.applied_theme = theme_key(self.shared.read(cx));
        self.capture = state;
        self.current = entry;
        self.entries[entry].tab = tab;
        for built in self.pages.iter().flatten() {
            built.page.apply_capture(state, cx);
        }
        cx.notify();
    }

    /// Apply a capture's scenario and `--set` values to the current
    /// playground.
    #[cfg_attr(target_family = "wasm", allow(dead_code))]
    pub(crate) fn configure(
        &mut self,
        scenario: Option<usize>,
        values: &[(&'static str, Value)],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(state) = self.ensure_playground(self.current, window, cx) else {
            return;
        };
        state.update(cx, |state, cx| {
            if let Some(scenario) = scenario {
                state.select_scenario(scenario, window, cx);
            }
            state.apply(values, window, cx);
        });
    }
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        #[cfg(test)]
        self.renders.set(self.renders.get() + 1);
        let (motion_scheme, snackbar) = {
            let shared = self.shared.read(cx);
            (shared.motion_scheme, shared.snackbar)
        };
        gpui_kit_material::motion::set_motion_scheme(motion_scheme);
        self.sync_layout(window, cx);
        // The shown page keeps playbacks and keyed animations alive even
        // while none of its rows is mounted.
        if let Some(built) = self
            .current_page()
            .and_then(|index| self.pages[index].as_ref())
        {
            built.page.prepare(window, cx);
        }
        if self.last_rem != window.rem_size() {
            self.last_rem = window.rem_size();
            self.list.remeasure();
        }
        if let Some((row, reverse)) = self.pending_focus.take() {
            let view = cx.weak_entity();
            window.on_next_frame(move |window, cx| {
                let _ = view.update(cx, |this, cx| {
                    if gpui_base::active_focus_trap(window, cx).is_none() {
                        this.focus_row(row, reverse, window, cx);
                    }
                });
            });
        }
        let compact = is_compact(window);
        let navigation = self.render_navigation(compact, cx);
        let view = cx.weak_entity();
        let sync_view = view.clone();
        // Reused across frames: the builder records this frame's rows.
        self.rendered_rows.borrow_mut().clear();
        let rendered_rows = self.rendered_rows.clone();
        let list_rows = rendered_rows.clone();
        let lead_only = self.shows_lead_only();
        let lead_row = lead_only.then(|| self.render_row(0, window, cx));
        let content = div()
            .id("gallery-content")
            .flex_1()
            .min_w_0()
            .min_h_0()
            .h_full()
            .bg(spec::page_background())
            .text_color(color(ColorRole::ON_SURFACE))
            .font_family("Roboto")
            .when_some(lead_row, |content, row| {
                content.child(
                    div()
                        .id("gallery-lead-scroll")
                        .size_full()
                        .overflow_y_scroll()
                        .track_scroll(&self.lead_scroll)
                        .child(row),
                )
            })
            .when(!lead_only, |content| {
                content.child(
                    gpui::list(self.list.clone(), move |index, window, cx| {
                        list_rows.borrow_mut().push(index);
                        view.update(cx, |this, cx| this.render_row(index, window, cx))
                            .unwrap_or_else(|_| div().into_any_element())
                    })
                    .size_full(),
                )
            })
            .child(
                gpui::canvas(
                    move |_, _, cx| {
                        let _ = sync_view.update(cx, |this, cx| {
                            this.after_layout(&rendered_rows.borrow(), cx);
                        });
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .w(px(0.))
                .h(px(0.)),
            );
        gpui_base::v_flex()
            .id("gallery")
            .on_action(cx.listener(|this, _: &gpui_base::RootTab, window, cx| {
                this.advance_gallery_focus(false, window, cx);
            }))
            .on_action(cx.listener(|this, _: &gpui_base::RootTabPrev, window, cx| {
                this.advance_gallery_focus(true, window, cx);
            }))
            .relative()
            .size_full()
            .min_h_0()
            .bg(spec::page_background())
            .text_color(color(ColorRole::ON_SURFACE))
            .font_family("Roboto")
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .flex_row()
                    .when(compact, |layout| layout.flex_col())
                    .child(navigation)
                    .child(content),
            )
            .when(snackbar, |root| {
                let dismiss = |shared: Entity<GalleryShared>| {
                    move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
                        shared.update(cx, |shared, cx| {
                            shared.snackbar = false;
                            cx.notify();
                        })
                    }
                };
                root.child(
                    div().absolute().left(px(16.)).bottom(px(16.)).child(
                        Snackbar::new("snackbar", "Settings saved")
                            .action("Undo", dismiss(self.shared.clone()))
                            .close("Dismiss", dismiss(self.shared.clone())),
                    ),
                )
            })
    }
}

/// The window content: the title bar (the top app bar's sections, search
/// and mode toggle, the FPS counter and settings) over the cached Gallery,
/// so a counter update renders only the bar and the counter, not the page.
/// Gallery notifications and window refreshes (theme and scale steps)
/// still render the Gallery again.
struct GalleryWindow {
    gallery: Entity<Gallery>,
    top_bar: Entity<top_bar::TopBar>,
    fps: Option<Entity<fps::FpsDisplay>>,
}

impl Render for GalleryWindow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui_base::v_flex()
            .size_full()
            .bg(spec::page_background())
            .text_color(color(ColorRole::ON_SURFACE))
            .font_family("Roboto")
            .child(
                TitleBar::new("title-bar", TITLE_BAR, caption_labels())
                    .gap(rem(12.))
                    .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
                    .border_b_1()
                    .border_color(color(ColorRole::OUTLINE_VARIANT))
                    .child(
                        AnyView::from(self.top_bar.clone())
                            .cached(StyleRefinement::default().flex_1().min_w_0().h_full()),
                    )
                    .when_some(self.fps.clone(), |bar, fps| bar.child(fps))
                    .child(title_bar_control(
                        MaterialButton::new("gallery-settings-open")
                            .variant(ButtonVariant::Text)
                            .button_size(ButtonSize::ExtraSmall)
                            .accessibility_label("Settings")
                            .debug_selector(|| "gallery-settings-open".into())
                            .child("Settings")
                            .on_click(|_, window, cx| settings::open(window, cx)),
                    )),
            )
            // Not `cached`: a dirty cached view always renders again, and every
            // preview animation frame dirties the views around the preview. As
            // a plain view it is reused while clean and spliced while dirty.
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .child(self.gallery.clone()),
            )
    }
}

pub fn run(application: gpui::Application, initialize: impl FnOnce(&mut App) + 'static) {
    #[cfg(all(not(target_family = "wasm"), feature = "frame-profiler"))]
    let interactions = interactions::prepare();
    #[cfg(all(not(target_family = "wasm"), feature = "frame-profiler"))]
    let profiling = if interactions.is_some() {
        None
    } else {
        performance::prepare()
    };
    let application = application.with_assets(Assets::new());
    application.run(move |cx: &mut App| {
        #[cfg(target_family = "wasm")]
        cx.text_system()
            .add_fonts(vec![std::borrow::Cow::Borrowed(include_bytes!(
                "fonts/IBMPlexSans-Regular.ttf"
            ))])
            .expect("Failed to load the gallery font");
        cx.text_system()
            .add_fonts(vec![
                std::borrow::Cow::Borrowed(include_bytes!("fonts/roboto/Roboto-Regular.ttf")),
                std::borrow::Cow::Borrowed(include_bytes!("fonts/roboto/Roboto-Medium.ttf")),
            ])
            .expect("Failed to load the reference Roboto fonts");
        gpui_kit::init(cx);
        gpui_kit_material::init(cx);
        initialize(cx);
        let viewport = size(px(1280.), px(900.));
        #[cfg(not(target_family = "wasm"))]
        let viewport = capture::window_size(cx).unwrap_or(viewport);
        let (_window, _content) = gpui_kit::open_window(
            title_bar_window_options("gpui-kit-material", TITLE_BAR, viewport, cx),
            cx,
            move |window, cx| {
                let gallery = cx.new(|cx| Gallery::new(window, cx));
                // FPS is part of ordinary UI and profiler runs. Only deterministic
                // screenshot captures omit the changing counter and its timer.
                #[cfg(not(target_family = "wasm"))]
                let show_fps = capture::show_fps(cx);
                #[cfg(target_family = "wasm")]
                let show_fps = true;
                let fps = show_fps.then(|| cx.new(|cx| fps::FpsDisplay::new(window, cx)));
                let top_bar = cx.new(|cx| top_bar::TopBar::new(gallery.clone(), window, cx));
                #[cfg(not(target_family = "wasm"))]
                capture::install(&gallery, window, cx);
                let shared = gallery.read(cx).shared.read(cx);
                let (initial_scheme, initial_scale) = (shared.target_scheme(), shared.text_scale());
                theme::init_component_theme(initial_scheme, initial_scale, cx);
                cx.new(|_| GalleryWindow {
                    gallery,
                    top_bar,
                    fps,
                })
            },
        )
        .unwrap();
        #[cfg(all(not(target_family = "wasm"), feature = "frame-profiler"))]
        if let Some(interactions) = interactions {
            _window
                .update(cx, |_, window, cx| {
                    let gallery = _content.read(cx).gallery.clone();
                    interactions.install(gallery, window, cx);
                })
                .unwrap();
        }
        #[cfg(all(not(target_family = "wasm"), feature = "frame-profiler"))]
        if let Some(profiling) = profiling {
            _window
                .update(cx, |_, window, cx| {
                    if std::env::var_os("MATERIAL_SCROLL_PROFILE").is_some() {
                        let target = _content.read(cx).gallery.read(cx).list.clone().into();
                        profiling.install_scroll(window, cx, target);
                    } else {
                        profiling.install(window, cx);
                    }
                })
                .unwrap();
        }
        cx.activate(true);
    });
}

#[cfg(test)]
mod window_tests {
    use super::{Gallery, GalleryWindow, fps, top_bar};
    // GPUI's attribute expands to an unqualified #[test]. Keep that name
    // bound to Rust's test attribute rather than the parent's GPUI glob import.
    use core::prelude::v1::test;
    use gpui::{AnyWindowHandle, AppContext as _, TestAppContext, px, size};

    fn draw(window: AnyWindowHandle, cx: &mut TestAppContext) {
        cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
            .unwrap();
        cx.run_until_parked();
    }

    #[gpui::test]
    fn preview_updates_render_neither_the_gallery_nor_the_lead_row(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let mut parts = None;
        let window = cx.open_window(size(px(1280.), px(800.)), |window, cx| {
            let gallery = cx.new(|cx| Gallery::new(window, cx));
            let top_bar = cx.new(|cx| top_bar::TopBar::new(gallery.clone(), window, cx));
            parts = Some(gallery.clone());
            GalleryWindow {
                gallery,
                top_bar,
                fps: None,
            }
        });
        let gallery = parts.unwrap();
        let entry = crate::gallery::catalog::position("button").unwrap();
        cx.update(|cx| gallery.update(cx, |gallery, cx| gallery.navigate(entry, cx)));
        for _ in 0..3 {
            draw(window.into(), cx);
        }
        let (preview, lead) = cx.read(|cx| {
            let gallery = gallery.read(cx);
            let views = gallery.entries[entry].views.clone().unwrap();
            (views.preview, gallery.lead.clone())
        });
        let counts = |cx: &TestAppContext| {
            cx.read(|cx| (gallery.read(cx).renders.get(), lead.read(cx).renders.get()))
        };
        let settled = counts(cx);
        // What every animation frame of a component in the preview does:
        // `Window::request_animation_frame` notifies the view, without updating it.
        for _ in 0..3 {
            cx.update(|cx| cx.notify(preview.entity_id()));
            draw(window.into(), cx);
        }
        assert_eq!(counts(cx), settled, "a preview frame rebuilt the page");
        // The lead row still follows the Gallery, its invalidation owner.
        cx.update(|cx| gallery.update(cx, |_, cx| cx.notify()));
        draw(window.into(), cx);
        let after = counts(cx);
        assert!(after.0 > settled.0 && after.1 > settled.1);
    }

    #[gpui::test]
    fn fps_updates_render_only_the_title_bar(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let mut parts = None;
        let window = cx.open_window(size(px(1280.), px(800.)), |window, cx| {
            let gallery = cx.new(|cx| Gallery::new(window, cx));
            let fps = cx.new(|cx| fps::FpsDisplay::new(window, cx));
            let top_bar = cx.new(|cx| top_bar::TopBar::new(gallery.clone(), window, cx));
            parts = Some((gallery.clone(), fps.clone()));
            GalleryWindow {
                gallery,
                top_bar,
                fps: Some(fps),
            }
        });
        let (gallery, fps) = parts.unwrap();
        draw(window.into(), cx);
        // Settle first-frame work (playback and focus setup notify once).
        draw(window.into(), cx);
        draw(window.into(), cx);
        let rendered = cx.read(|cx| gallery.read(cx).renders.get());
        assert!(rendered > 0);
        cx.update(|cx| fps.update(cx, |_, cx| cx.notify()));
        draw(window.into(), cx);
        assert_eq!(
            cx.read(|cx| gallery.read(cx).renders.get()),
            rendered,
            "a counter update must reuse the cached Gallery"
        );
        cx.update(|cx| gallery.update(cx, |_, cx| cx.notify()));
        draw(window.into(), cx);
        assert_eq!(cx.read(|cx| gallery.read(cx).renders.get()), rendered + 1);
    }
}
