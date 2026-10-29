//! Every extracted component on one page, in every built-in palette.
//!
//! `cargo run -p gpui-kit-material --example gallery`
#[cfg(not(target_family = "wasm"))]
pub mod capture;
#[cfg(all(not(target_family = "wasm"), feature = "frame-profiler"))]
mod performance;
#[cfg(all(not(target_family = "wasm"), feature = "frame-profiler"))]
mod scroll_performance;

mod fps;
mod rows;
mod settings;

use gpui::{prelude::*, *};
use gpui_kit_material::color::palette::ALL as PALETTES;
use gpui_kit_material::color::themes::{self, Contrast, ThemeId, ThemeMode};
use gpui_kit_material::icons::{Assets, IconName};
use gpui_kit_material::motion;
use gpui_kit_material::prelude::*;
use gpui_kit_material::theme;

// Reveal only the newly focused control, not the entire long gallery page.
trait GalleryReveal: IntoElement + Sized {
    fn reveal(self, id: impl Into<ElementId>, list: &ListState) -> RevealFocus {
        RevealFocus::in_list(id, self, list.clone()).inline()
    }
    fn reveal_handle(self, id: impl Into<ElementId>, scroll: &ScrollHandle) -> RevealFocus {
        RevealFocus::new(id, self, scroll.clone()).inline()
    }
}
impl<T: IntoElement> GalleryReveal for T {}

const GALLERY_SECTIONS: [(&str, &str); 15] = [
    ("themes", "Themes"),
    ("motion", "Motion"),
    ("buttons", "Buttons"),
    ("toggles", "Toggles"),
    ("loading", "Loading"),
    ("icons", "Icons"),
    ("labels", "Labels"),
    ("feedback", "Feedback"),
    ("colors", "Colors"),
    ("typography", "Typography"),
    ("dividers", "Dividers"),
    ("chips", "Chips"),
    ("cards", "Cards"),
    ("lists", "Lists"),
    ("window", "Window controls"),
];

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

struct Gallery {
    palette: PaletteId,
    reference_theme: Option<ThemeId>,
    reference_mode: ThemeMode,
    contrast: Contrast,
    scale: SharedString,
    density: SharedString,
    engine: usize,
    details: bool,
    list: ListState,
    row_focus: Vec<FocusHandle>,
    navigation_focus: FocusHandle,
    pending_focus: Option<(usize, bool)>,
    last_rem: Pixels,
    color_columns: usize,
    feedback_spinner_playback: RepeatAnimationPlayback,
    feedback_linear_playback: RepeatAnimationPlayback,
    feedback_circular_playback: CircularProgressPlayback,
    navigation_scroll: ScrollHandle,
    section_rows: [usize; GALLERY_SECTIONS.len()],
    selected_section: usize,
    search: Entity<gpui_base::input::InputState>,
    slider: Entity<gpui_base::slider::SliderState>,
    switch: bool,
    sheet: bool,
    snackbar: bool,
    motion_scheme: motion::MotionScheme,
    motion_target: bool,
    motion_values: [f32; 6],
    theme_frame_pending: bool,
    button_effects_disabled: bool,
    button_effects_tonal: bool,
    button_ripple: bool,
    button_active: bool,
    button_selections: [bool; 10],
    button_toggle_size: ButtonSize,
    button_loading: bool,
    button_progress: bool,
    button_loading_variant: usize,
    button_loading_clicks: usize,
    #[cfg(test)]
    renders: std::cell::Cell<usize>,
    #[cfg(test)]
    button_sync_rows: std::cell::RefCell<std::collections::BTreeSet<usize>>,
    button_loading_playbacks: [ButtonTransitionPlayback; 15],
    button_effect_playbacks: [ButtonTransitionPlayback; 5],
    button_toggle_playbacks: [ButtonTransitionPlayback; 10],
    button_progress_playbacks: [ButtonProgressPlayback; 15],
    button_progress_inputs: Option<(bool, bool, bool)>,
    /// The rows the list builder rendered in the current frame.
    rendered_rows: std::rc::Rc<std::cell::RefCell<Vec<usize>>>,
}

impl Gallery {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| {
            gpui_base::input::InputState::new(window, cx).placeholder("Search components")
        });
        let slider = cx.new(|_| {
            gpui_base::slider::SliderState::new()
                .min(0.)
                .max(100.)
                .default_value(40.)
        });
        cx.observe(&slider, |_, _, cx| cx.notify()).detach();
        let color_columns = rows::color_columns(window);
        let row_focus: Vec<_> = (0..rows::row_count(color_columns))
            .map(|_| cx.focus_handle())
            .collect();
        let list = ListState::new(0, ListAlignment::Top, px(128.));
        list.splice_focusable(0..0, row_focus.iter().cloned().map(Some));
        let section_rows = std::array::from_fn(|section| {
            rows::physical_row(
                rows::ROWS
                    .iter()
                    .position(|row| row.section == Some(GALLERY_SECTIONS[section].0))
                    .expect("each Gallery section has a heading row"),
                color_columns,
            )
        });
        Gallery {
            search,
            slider,
            switch: false,
            sheet: false,
            snackbar: false,
            motion_scheme: motion::MotionScheme::Expressive,
            motion_target: false,
            motion_values: [0.; 6],
            theme_frame_pending: false,
            button_effects_disabled: false,
            button_effects_tonal: false,
            button_ripple: true,
            button_active: false,
            button_selections: [false; 10],
            button_toggle_size: ButtonSize::Small,
            button_loading: false,
            button_progress: false,
            button_loading_variant: 0,
            button_loading_clicks: 0,
            #[cfg(test)]
            renders: Default::default(),
            #[cfg(test)]
            button_sync_rows: Default::default(),
            button_loading_playbacks: std::array::from_fn(|_| ButtonTransitionPlayback::default()),
            button_effect_playbacks: std::array::from_fn(|_| ButtonTransitionPlayback::default()),
            button_toggle_playbacks: std::array::from_fn(|_| ButtonTransitionPlayback::default()),
            button_progress_playbacks: std::array::from_fn(|_| ButtonProgressPlayback::default()),
            button_progress_inputs: None,
            rendered_rows: Default::default(),
            palette: PaletteId::Paper,
            reference_theme: Some(ThemeId::Baseline),
            reference_mode: ThemeMode::Light,
            contrast: Contrast::Standard,
            scale: "1".into(),
            density: "auto".into(),
            engine: 0,
            details: false,
            list,
            row_focus,
            navigation_focus: cx.focus_handle(),
            pending_focus: None,
            last_rem: window.rem_size(),
            color_columns,
            feedback_spinner_playback: RepeatAnimationPlayback::default(),
            feedback_linear_playback: RepeatAnimationPlayback::default(),
            feedback_circular_playback: CircularProgressPlayback::default(),
            section_rows,
            navigation_scroll: ScrollHandle::new(),
            selected_section: 0,
        }
    }

    fn target_scheme(&self) -> gpui_kit_material::color::Scheme {
        self.reference_theme.map_or_else(
            || gpui_kit_material::color::Scheme::from_palette(self.palette),
            |id| id.scheme(self.reference_mode, self.contrast),
        )
    }

    // Run before Root prepares its styles, never while rendering the child view.
    fn prepare_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        motion::set_motion_scheme(self.motion_scheme);
        let running = theme::advance_component_theme(
            self.target_scheme(),
            self.scale.parse().unwrap_or(1.),
            window,
            cx,
        );
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
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        #[cfg(test)]
        self.renders.set(self.renders.get() + 1);
        self.feedback_circular_playback.observe_reduced_motion(cx);
        let progress_inputs = (
            self.button_progress,
            self.button_loading,
            self.button_effects_disabled,
        );
        for playback in &self.button_progress_playbacks {
            if self.button_progress_inputs != Some(progress_inputs) {
                playback.sync(progress_inputs.0, progress_inputs.1, progress_inputs.2, cx);
            }
            playback.observe_reduced_motion(cx);
        }
        self.button_progress_inputs = Some(progress_inputs);
        motion::set_motion_scheme(self.motion_scheme);
        // Initialize all previews with the Gallery, even before their first visit.
        // Keep Base's keyed retargeting and pinned curves alive independently of
        // virtual rows. An offscreen target change still runs its finite transition
        // (and requests frames until settled); remounting never restarts it.
        self.motion_values = std::array::from_fn(|index| {
            motion::token_value(
                ("motion-preview", index),
                if self.motion_target { 1. } else { 0. },
                [
                    motion::MotionToken::FastSpatial,
                    motion::MotionToken::DefaultSpatial,
                    motion::MotionToken::SlowSpatial,
                    motion::MotionToken::FastEffects,
                    motion::MotionToken::DefaultEffects,
                    motion::MotionToken::SlowEffects,
                ][index],
                window,
                cx,
            )
        });
        let compact = is_compact(window);
        let columns = rows::color_columns(window);
        if columns != self.color_columns {
            let previous = self.list.logical_scroll_top();
            let source = rows::source_row(previous.item_ix, self.color_columns);
            self.color_columns = columns;
            self.row_focus
                .resize_with(rows::row_count(columns), || cx.focus_handle());
            self.list.splice_focusable(
                0..self.list.item_count(),
                self.row_focus.iter().cloned().map(Some),
            );
            self.section_rows = std::array::from_fn(|section| {
                rows::physical_row(
                    rows::ROWS
                        .iter()
                        .position(|row| row.section == Some(GALLERY_SECTIONS[section].0))
                        .unwrap(),
                    columns,
                )
            });
            self.list.scroll_to(ListOffset {
                item_ix: rows::physical_row(source, columns),
                offset_in_item: previous.offset_in_item,
            });
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
        let navigation = div()
            .id("gallery-navigation")
            .role(Role::Navigation)
            .aria_label("Gallery sections")
            .track_focus(&self.navigation_focus)
            .tab_stop(false)
            .flex_shrink_0()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.navigation_scroll)
            .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
            .when(compact, |nav| nav.w_full().max_h(rem(160.)).border_b_1())
            .when(!compact, |nav| nav.w(rem(176.)).h_full().border_r_1())
            .border_color(color(ColorRole::OUTLINE_VARIANT))
            .child(
                gpui_base::v_flex()
                    .p(rem(12.))
                    .pt(rem(32.))
                    .gap(rem(12.))
                    .child(
                        accessible_text("gallery-navigation-title", "Browse components")
                            .text_size(TEXT_AUX),
                    )
                    .child(
                        gpui_base::v_flex()
                            .gap(rem(8.))
                            .when(compact, |items| items.flex_row().flex_wrap())
                            .children(GALLERY_SECTIONS.into_iter().enumerate().map(
                                |(index, (_, label))| {
                                    MaterialButton::new(("navigate", index))
                                        .variant(ButtonVariant::Tonal)
                                        .button_size(ButtonSize::ExtraSmall)
                                        .toggle(true)
                                        .selected(self.selected_section == index)
                                        .toggle_on_click(false)
                                        .child(label)
                                        .when(!compact, |button| button.w_full().justify_start())
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            this.selected_section = index;
                                            this.scroll_to_section(index);
                                            let _ = window;
                                            cx.notify();
                                        }))
                                        .reveal_handle(
                                            ("reveal-navigation", index),
                                            &self.navigation_scroll,
                                        )
                                },
                            )),
                    ),
            );
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
            .bg(color(ColorRole::SURFACE_CONTAINER))
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
                            this.sync_button_transitions(&rendered_rows.borrow(), cx);
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
            .bg(color(ColorRole::SURFACE_CONTAINER))
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
            .when(self.snackbar, |root| {
                root.child(
                    div().absolute().left(px(16.)).bottom(px(16.)).child(
                        Snackbar::new("snackbar", "Settings saved")
                            .action(
                                "Undo",
                                cx.listener(|this, _, _, cx| {
                                    this.snackbar = false;
                                    cx.notify();
                                }),
                            )
                            .close(
                                "Dismiss",
                                cx.listener(|this, _, _, cx| {
                                    this.snackbar = false;
                                    cx.notify();
                                }),
                            ),
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
            .bg(color(ColorRole::SURFACE_CONTAINER))
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
                let initial = gallery.read(cx);
                let initial_scheme = initial.target_scheme();
                let initial_scale = initial.scale.parse().unwrap_or(1.);
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
