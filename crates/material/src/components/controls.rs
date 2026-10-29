//! Material controls composed from GPUI Base behavior.
//! Reference playgrounds: <https://md3.io/components/slider/>,
//! <https://md3.io/components/search/> and <https://md3.io/components/button/>.
#[cfg(test)]
use super::{ButtonVariant, MaterialSwitch, material_button};
use crate::{
    color::ColorRole,
    motion,
    theme::{blend, color},
    tokens::{
        state,
        typography::{TypeRole, rem},
    },
};
use gpui::{prelude::*, *};
use gpui_base::{
    input::{Input, InputState},
    slider::{Slider, SliderIndicator, SliderState, SliderThumb, SliderTrack},
};
/// Slider geometry. Classic matches the Google Fonts axis controls; Expressive
/// follows the md3.io thick track and vertical handle presentation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SliderAppearance {
    Classic,
    #[default]
    Expressive,
}
#[derive(gpui::IntoElement)]
pub struct MaterialSlider {
    state: Entity<SliderState>,
    label: SharedString,
    appearance: SliderAppearance,
    disabled: bool,
}
impl MaterialSlider {
    pub fn new(state: &Entity<SliderState>, label: impl Into<SharedString>) -> Self {
        Self {
            state: state.clone(),
            label: label.into(),
            appearance: SliderAppearance::default(),
            disabled: false,
        }
    }
    pub fn appearance(mut self, appearance: SliderAppearance) -> Self {
        self.appearance = appearance;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
fn slider_geometry(appearance: SliderAppearance) -> (f32, f32, f32) {
    match appearance {
        SliderAppearance::Classic => (4., 20., 20.),
        SliderAppearance::Expressive => (16., 4., 44.),
    }
}

/// M3 Expressive slider details, in dp: the gap between the handle and the
/// track on either side, the track's outer corners and the corners where it
/// meets the gap, and the stop indicator at the track's end.
const HANDLE_GAP: f32 = 6.;
const TRACK_CORNER: f32 = 8.;
const TRACK_INNER_CORNER: f32 = 2.;
const STOP_INDICATOR: f32 = 4.;

/// The colors of a slider in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
struct SliderPaint {
    active: Rgba,
    inactive: Rgba,
    handle: Rgba,
    /// The stop indicator on the inactive track.
    stop: Rgba,
}

fn slider_paint(disabled: bool) -> SliderPaint {
    if disabled {
        let on_surface = color(ColorRole::ON_SURFACE);
        SliderPaint {
            active: state::disabled(on_surface),
            inactive: on_surface.opacity(state::DISABLED_CONTAINER),
            // The handle is an opaque mix, so the track does not show
            // through it.
            handle: blend(
                color(ColorRole::SURFACE),
                on_surface,
                state::DISABLED_CONTENT,
            ),
            stop: state::disabled(on_surface),
        }
    } else {
        SliderPaint {
            active: color(ColorRole::PRIMARY),
            inactive: color(ColorRole::SECONDARY_CONTAINER),
            handle: color(ColorRole::PRIMARY),
            stop: color(ColorRole::PRIMARY),
        }
    }
}

impl RenderOnce for MaterialSlider {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = self.state.read(cx).percentage().end;
        let (track, width, height) = slider_geometry(self.appearance);
        let paint = slider_paint(self.disabled);
        let track_top = rem((48. - track) / 2.);
        let indicator = SliderIndicator::new(&self.state)
            .absolute()
            .left_0()
            .top(track_top)
            .w_full()
            .h(rem(track));
        let indicator = match self.appearance {
            SliderAppearance::Classic => indicator.rounded_full().bg(paint.inactive).child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .h_full()
                    .w(relative(p))
                    .rounded_full()
                    .bg(paint.active),
            ),
            SliderAppearance::Expressive => {
                // Each side of the handle stops HANDLE_GAP short of it; the
                // segment collapses rather than crossing the handle.
                let clearance = rem(width / 2. + HANDLE_GAP);
                let (outer, inner) = (rem(TRACK_CORNER), rem(TRACK_INNER_CORNER));
                indicator
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .top_0()
                            .h_full()
                            .w(relative(p))
                            .pr(clearance)
                            .child(
                                div()
                                    .when(cfg!(test), |segment| {
                                        segment.debug_selector(|| "slider-active".into())
                                    })
                                    .size_full()
                                    .rounded_l(outer)
                                    .rounded_r(inner)
                                    .bg(paint.active),
                            ),
                    )
                    .child(
                        div()
                            .absolute()
                            .right_0()
                            .top_0()
                            .h_full()
                            .w(relative(1. - p))
                            .pl(clearance)
                            .child(
                                div()
                                    .when(cfg!(test), |segment| {
                                        segment.debug_selector(|| "slider-inactive".into())
                                    })
                                    .size_full()
                                    .rounded_l(inner)
                                    .rounded_r(outer)
                                    .bg(paint.inactive),
                            ),
                    )
                    .when(p < 1., |track_view| {
                        track_view.child(
                            div()
                                .when(cfg!(test), |stop| {
                                    stop.debug_selector(|| "slider-stop".into())
                                })
                                .absolute()
                                .right(rem((track - STOP_INDICATOR) / 2.))
                                .top(rem((track - STOP_INDICATOR) / 2.))
                                .size(rem(STOP_INDICATOR))
                                .rounded_full()
                                .bg(paint.stop),
                        )
                    })
            }
        };
        Slider::new(&self.state)
            .accessibility_label(self.label)
            .disabled(self.disabled)
            .w_full()
            .h(rem(48.))
            .child(
                SliderTrack::new(&self.state)
                    .disabled(self.disabled)
                    .relative()
                    .w_full()
                    .h_full()
                    .child(indicator)
                    .child(
                        SliderThumb::new(&self.state)
                            .disabled(self.disabled)
                            .absolute()
                            .left(relative(p))
                            .ml(rem(-width / 2.))
                            .top(rem((48. - height) / 2.))
                            .w(rem(width))
                            .h(rem(height))
                            .rounded_full()
                            .bg(paint.handle),
                    ),
            )
    }
}

/// The M3 search bar container: surface-container-high (not the highest).
const SEARCH_CONTAINER: ColorRole = ColorRole::SURFACE_CONTAINER_HIGH;

/// M3 search field. Editing, selection, clipboard and IME are owned by GPUI Base.
#[derive(gpui::IntoElement)]
pub struct SearchField {
    state: Entity<InputState>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
}
impl SearchField {
    pub fn new(state: &Entity<InputState>) -> Self {
        Self {
            state: state.clone(),
            leading: None,
            trailing: None,
        }
    }
    pub fn leading(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }
}
impl RenderOnce for SearchField {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .w_full()
            .min_w_0()
            .h(rem(56.))
            .px(rem(16.))
            .gap(rem(16.))
            .rounded_full()
            .bg(color(SEARCH_CONTAINER))
            .text_color(color(ColorRole::ON_SURFACE))
            .text_size(TypeRole::BodyLarge.scale().font_size)
            .children(self.leading)
            .child(div().flex_1().min_w_0().child(Input::new(&self.state)))
            .children(self.trailing)
    }
}

/// A nonmodal M3 standard side sheet. It leaves the main content interactive;
/// its controlled visibility retargets the spatial spring, including on close.
#[derive(gpui::IntoElement)]
pub struct MaterialSideSheet {
    id: ElementId,
    open: bool,
    width: Rems,
    children: Vec<AnyElement>,
}
impl MaterialSideSheet {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            open: false,
            width: rem(320.),
            children: vec![],
        }
    }
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    pub fn width(mut self, width: Rems) -> Self {
        self.width = width;
        self
    }
}
impl ParentElement for MaterialSideSheet {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}
/// Whether a sheet has finished closing: closed, with its reveal settled.
fn sheet_hidden(open: bool, reveal: f32) -> bool {
    !open && reveal == 0.
}

impl RenderOnce for MaterialSideSheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reveal = motion::value(
            (self.id.clone(), "reveal"),
            if self.open { 1. } else { 0. },
            window,
            cx,
        );
        let progress = reveal.clamp(0., 1.);
        let width = self.width.to_pixels(window.rem_size());
        div()
            .id(self.id)
            .absolute()
            .top_0()
            .bottom_0()
            .right(width * (progress - 1.))
            .w(width)
            .opacity(progress)
            .bg(color(ColorRole::SURFACE))
            // Once the close has settled, the sheet's content leaves the tree,
            // so it offers no tab stops or accessibility nodes off screen. The
            // spring lands exactly on its target; an overshoot does not.
            .when(!sheet_hidden(self.open, reveal), |sheet| {
                sheet.children(self.children)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[core::prelude::v1::test]
    fn slider_variants_have_distinct_handle_and_track() {
        assert_eq!(slider_geometry(SliderAppearance::Classic), (4., 20., 20.));
        assert_eq!(
            slider_geometry(SliderAppearance::Expressive),
            (16., 4., 44.)
        );
    }

    #[core::prelude::v1::test]
    fn slider_uses_m3_expressive_details_and_disabled_roles() {
        assert_eq!(
            (HANDLE_GAP, TRACK_CORNER, TRACK_INNER_CORNER, STOP_INDICATOR),
            (6., 8., 2., 4.)
        );
        let enabled = slider_paint(false);
        assert_eq!(enabled.active, color(ColorRole::PRIMARY));
        assert_eq!(enabled.inactive, color(ColorRole::SECONDARY_CONTAINER));
        assert_eq!(enabled.stop, color(ColorRole::PRIMARY));
        let disabled = slider_paint(true);
        let on_surface = color(ColorRole::ON_SURFACE);
        assert_eq!(disabled.active, state::disabled(on_surface));
        assert_eq!(disabled.inactive.a, on_surface.a * 0.12);
        assert_eq!(disabled.handle.a, 1., "the disabled handle is opaque");
    }

    struct SliderHarness(Entity<SliderState>);
    impl Render for SliderHarness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div()
                .w(px(200.))
                .child(MaterialSlider::new(&self.0, "Level"))
        }
    }

    #[gpui::test]
    fn the_stop_indicator_sits_at_the_track_end_until_the_value_reaches_it(
        cx: &mut TestAppContext,
    ) {
        cx.update(gpui_base::init);
        let (_, window) = cx.add_window_view(|_, cx| {
            SliderHarness(cx.new(|_| SliderState::new().max(100.).default_value(30.)))
        });
        draw(window);
        let stop = window
            .debug_bounds("slider-stop")
            .expect("a stop indicator");
        assert_eq!(stop.size, size(px(4.), px(4.)));
        assert_eq!(stop.right(), px(200. - 6.));
        // The 4dp handle is centered at 60px; each side stops 6dp short.
        let active = window.debug_bounds("slider-active").unwrap();
        let inactive = window.debug_bounds("slider-inactive").unwrap();
        assert_eq!(active.right(), px(60. - 2. - 6.));
        assert_eq!(inactive.left(), px(60. + 2. + 6.));
        let (_, window) = cx.add_window_view(|_, cx| {
            SliderHarness(cx.new(|_| SliderState::new().max(100.).default_value(100.)))
        });
        draw(window);
        assert!(window.debug_bounds("slider-stop").is_none());
    }

    #[core::prelude::v1::test]
    fn search_bar_sits_on_surface_container_high() {
        assert_eq!(SEARCH_CONTAINER, ColorRole::SURFACE_CONTAINER_HIGH);
        assert_ne!(SEARCH_CONTAINER, ColorRole::SURFACE_CONTAINER_HIGHEST);
    }

    struct ControlsHarness {
        checked: bool,
        disabled: bool,
        changes: usize,
        search: Entity<InputState>,
        slider: Entity<SliderState>,
    }
    impl Render for ControlsHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .flex_col()
                .w(px(240.))
                .gap(px(12.))
                .child(
                    div()
                        .debug_selector(|| "test-switch".into())
                        .w(px(52.))
                        .h(px(32.))
                        .child(
                            MaterialSwitch::new("switch", "Notifications")
                                .checked(self.checked)
                                .disabled(self.disabled)
                                .on_change(cx.listener(|this, value: &bool, _, cx| {
                                    this.checked = *value;
                                    this.changes += 1;
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    div()
                        .debug_selector(|| "test-search".into())
                        .child(SearchField::new(&self.search)),
                )
                .child(
                    div()
                        .debug_selector(|| "test-slider".into())
                        .child(MaterialSlider::new(&self.slider, "Weight").disabled(self.disabled)),
                )
                .child(
                    material_button("disabled-action", ButtonVariant::Filled)
                        .disabled(self.disabled)
                        .child("Action")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.changes += 1;
                            cx.notify();
                        })),
                )
        }
    }
    fn draw(cx: &mut VisualTestContext) {
        cx.update(|w, cx| {
            w.refresh();
            w.draw(cx).clear(cx);
        });
    }
    #[gpui::test]
    fn controls_keep_base_input_and_disabled_semantics(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|w, cx| ControlsHarness {
            checked: false,
            disabled: false,
            changes: 0,
            search: cx.new(|cx| InputState::new(w, cx).placeholder("Search")),
            slider: cx.new(|_| SliderState::new().max(100.).default_value(0.)),
        });
        draw(cx);
        let toggle = cx.debug_bounds("test-switch").unwrap().center();
        cx.simulate_click(toggle, Modifiers::default());
        draw(cx);
        cx.update(|_, cx| {
            assert!(view.read(cx).checked);
            assert_eq!(view.read(cx).changes, 1);
        });
        let search = cx.debug_bounds("test-search").unwrap().center();
        cx.simulate_click(search, Modifiers::default());
        cx.simulate_input("home");
        draw(cx);
        cx.update(|_, cx| assert_eq!(view.read(cx).search.read(cx).value().as_ref(), "home"));
        let bounds = cx.debug_bounds("test-slider").unwrap();
        cx.simulate_click(
            point(bounds.left() + bounds.size.width * 0.75, bounds.center().y),
            Modifiers::default(),
        );
        draw(cx);
        cx.update(|_, cx| assert!((view.read(cx).slider.read(cx).value().end() - 75.).abs() < 2.));
        view.update(cx, |view, cx| {
            view.disabled = true;
            cx.notify();
        });
        draw(cx);
        cx.simulate_click(toggle, Modifiers::default());
        cx.simulate_click(
            point(bounds.left() + bounds.size.width * 0.1, bounds.center().y),
            Modifiers::default(),
        );
        draw(cx);
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).changes, 1);
            assert!((view.read(cx).slider.read(cx).value().end() - 75.).abs() < 2.);
        });
    }

    struct SheetHarness(bool);
    impl Render for SheetHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().relative().size_full().overflow_hidden().child(
                MaterialSideSheet::new("sheet")
                    .open(self.0)
                    .width(rem(100.))
                    .child(div().debug_selector(|| "sheet-body".into()).size_full()),
            )
        }
    }
    #[gpui::test]
    fn reduced_motion_sheet_opens_and_closes_outside_viewport(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let (view, cx) = cx.add_window_view(|_, _| SheetHarness(false));
        draw(cx);
        assert!(cx.debug_bounds("sheet-body").is_none());
        view.update(cx, |v, cx| {
            v.0 = true;
            cx.notify();
        });
        draw(cx);
        let open = cx.debug_bounds("sheet-body").unwrap();
        let viewport = cx.update(|window, _| window.viewport_size());
        assert_eq!(open.right(), viewport.width, "an open sheet meets the edge");
        view.update(cx, |v, cx| {
            v.0 = false;
            cx.notify();
        });
        draw(cx);
        assert!(
            cx.debug_bounds("sheet-body").is_none(),
            "a settled closed sheet renders no content"
        );
    }

    #[core::prelude::v1::test]
    fn the_sheet_keeps_its_content_until_the_close_settles() {
        assert!(
            !sheet_hidden(true, 0.),
            "opening starts from 0 with content"
        );
        assert!(
            !sheet_hidden(false, 0.4),
            "closing animates its content out"
        );
        assert!(!sheet_hidden(false, -0.01), "an overshoot is still moving");
        assert!(sheet_hidden(false, 0.));
    }

    struct FocusSheetHarness {
        open: bool,
        focus: FocusHandle,
    }
    impl Render for FocusSheetHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().relative().size_full().child(
                MaterialSideSheet::new("focus-sheet").open(self.open).child(
                    div()
                        .id("sheet-control")
                        .track_focus(&self.focus)
                        .debug_selector(|| "sheet-control".into())
                        .size(px(20.)),
                ),
            )
        }
    }

    #[gpui::test]
    fn a_closed_sheet_offers_no_tab_stops(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let (view, cx) = cx.add_window_view(|_, cx| FocusSheetHarness {
            open: true,
            focus: cx.focus_handle(),
        });
        draw(cx);
        assert!(cx.debug_bounds("sheet-control").is_some());
        view.update(cx, |v, cx| {
            v.open = false;
            cx.notify();
        });
        draw(cx);
        assert!(cx.debug_bounds("sheet-control").is_none());
        cx.update(|window, cx| {
            window.focus_next(cx);
            let focus = view.read(cx).focus.clone();
            assert!(!focus.is_focused(window));
        });
    }
}
