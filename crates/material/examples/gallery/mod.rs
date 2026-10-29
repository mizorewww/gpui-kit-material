//! The shared desktop/WASM gallery, organized like the component pages of
//! Google's Figma "Material 3 Design Kit": one panel per page, listed in
//! `pages::PAGES`. See `page` for how to add a page.
//!
//! `cargo run -p gpui-kit-material --example gallery`
#[cfg(not(target_family = "wasm"))]
pub mod capture;
#[cfg(all(not(target_family = "wasm"), feature = "frame-profiler"))]
mod performance;
#[cfg(all(not(target_family = "wasm"), feature = "frame-profiler"))]
mod scroll_performance;

mod fps;
pub(crate) mod page;
mod pages;
mod settings;
mod shell;
#[cfg(test)]
mod shell_tests;
pub(crate) mod spec;

use gpui::{prelude::*, *};
use gpui_kit_material::icons::Assets;
use gpui_kit_material::prelude::*;
use gpui_kit_material::theme;
use page::{AnyPage, CaptureState, GalleryShared, PageCx, PageRows};

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

pub(crate) struct Gallery {
    shared: Entity<GalleryShared>,
    applied_theme: ThemeKey,
    pages: Vec<Box<dyn AnyPage>>,
    page_rows: Vec<PageRows>,
    /// The list row of each page's header.
    headers: Vec<usize>,
    /// Each page's body rows.
    counts: Vec<usize>,
    list: ListState,
    row_focus: Vec<FocusHandle>,
    navigation_focus: FocusHandle,
    navigation_scroll: ScrollHandle,
    pending_focus: Option<(usize, bool)>,
    last_rem: Pixels,
    selected_section: usize,
    theme_frame_pending: bool,
    /// The rows the list builder rendered in the current frame.
    rendered_rows: std::rc::Rc<std::cell::RefCell<Vec<usize>>>,
    _subscriptions: Vec<Subscription>,
    #[cfg(test)]
    renders: std::cell::Cell<usize>,
}

impl Gallery {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let shared = cx.new(|_| GalleryShared::default());
        let list = ListState::new(0, ListAlignment::Top, px(128.));
        let mut pages = Vec::with_capacity(pages::PAGES.len());
        let mut page_rows = Vec::with_capacity(pages::PAGES.len());
        let mut headers = Vec::with_capacity(pages::PAGES.len());
        let mut counts = Vec::with_capacity(pages::PAGES.len());
        let mut next = 0;
        for entry in pages::PAGES {
            let rows = PageRows::new(list.clone(), next);
            let page = (entry.build)(
                PageCx {
                    shared: shared.clone(),
                    rows: rows.clone(),
                },
                window,
                cx,
            );
            let count = page.row_count(window, cx);
            headers.push(next);
            counts.push(count);
            page_rows.push(rows);
            pages.push(page);
            next += 1 + count;
        }
        let row_focus: Vec<_> = (0..next).map(|_| cx.focus_handle()).collect();
        list.splice_focusable(0..0, row_focus.iter().cloned().map(Some));
        let mut subscriptions: Vec<_> = pages.iter().map(|page| page.observe(cx)).collect();
        subscriptions.push(cx.observe_in(
            &shared,
            window,
            |this: &mut Self, shared, window, cx| {
                let key = theme_key(shared.read(cx));
                if key != this.applied_theme {
                    // The transition refreshes the window each step, which
                    // renders the Gallery; a notify would invalidate it twice.
                    this.applied_theme = key;
                    this.prepare_theme(window, cx);
                } else {
                    cx.notify();
                }
            },
        ));
        let applied_theme = theme_key(shared.read(cx));
        Gallery {
            shared,
            applied_theme,
            pages,
            page_rows,
            headers,
            counts,
            list,
            row_focus,
            navigation_focus: cx.focus_handle(),
            navigation_scroll: ScrollHandle::new(),
            pending_focus: None,
            last_rem: window.rem_size(),
            selected_section: 0,
            theme_frame_pending: false,
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

    /// Apply capture overrides to the settings and every page.
    #[cfg_attr(target_family = "wasm", allow(dead_code))]
    pub(crate) fn apply_capture(
        &mut self,
        section: usize,
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
        self.selected_section = section;
        for page in &self.pages {
            page.apply_capture(state, cx);
        }
        cx.notify();
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
        // Pages keep playbacks and keyed animations alive even while none of
        // their rows is mounted.
        for page in &self.pages {
            page.prepare(window, cx);
        }
        self.sync_row_counts(window, cx);
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
        let content = div()
            .id("gallery-content")
            .flex_1()
            .min_w_0()
            .min_h_0()
            .h_full()
            .bg(spec::page_background())
            .text_color(color(ColorRole::ON_SURFACE))
            .font_family("Roboto")
            .child(
                gpui::list(self.list.clone(), move |index, window, cx| {
                    list_rows.borrow_mut().push(index);
                    view.update(cx, |this, cx| this.render_row(index, window, cx))
                        .unwrap_or_else(|_| div().into_any_element())
                })
                .size_full(),
            )
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

/// The window content: the title bar and its FPS counter over the cached
/// Gallery, so a counter update renders only this bar and the counter, not
/// every Gallery row. Gallery notifications and window refreshes (theme and
/// scale steps) still render the Gallery again.
struct GalleryWindow {
    gallery: Entity<Gallery>,
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
                    .child(section_title("window-title", "gpui-kit-material"))
                    .child(div().flex_1())
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
            .child(
                AnyView::from(self.gallery.clone())
                    .cached(StyleRefinement::default().flex_1().min_h_0().w_full()),
            )
    }
}

pub fn run(application: gpui::Application, initialize: impl FnOnce(&mut App) + 'static) {
    #[cfg(all(not(target_family = "wasm"), feature = "frame-profiler"))]
    let profiling = performance::prepare();
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
        let viewport = size(px(960.), px(900.));
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
                #[cfg(not(target_family = "wasm"))]
                capture::install(&gallery, window, cx);
                let shared = gallery.read(cx).shared.read(cx);
                let (initial_scheme, initial_scale) = (shared.target_scheme(), shared.text_scale());
                theme::init_component_theme(initial_scheme, initial_scale, cx);
                cx.new(|_| GalleryWindow { gallery, fps })
            },
        )
        .unwrap();
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
    use super::{Gallery, GalleryWindow, fps};
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
    fn fps_updates_render_only_the_title_bar(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let mut parts = None;
        let window = cx.open_window(size(px(960.), px(600.)), |window, cx| {
            let gallery = cx.new(|cx| Gallery::new(window, cx));
            let fps = cx.new(|cx| fps::FpsDisplay::new(window, cx));
            parts = Some((gallery.clone(), fps.clone()));
            GalleryWindow {
                gallery,
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
