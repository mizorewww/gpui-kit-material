//! Material controls composed from GPUI Base behavior.
//! Reference playgrounds: https://md3.io/components/{switch,slider,search,button}/.
#[cfg(test)]
use super::{ButtonVariant, material_button};
use crate::{
    color::ColorRole,
    motion,
    theme::color,
    tokens::typography::{TypeRole, rem},
};
use gpui::{prelude::*, *};
use gpui_base::{
    input::{Input, InputState},
    slider::{Slider, SliderIndicator, SliderState, SliderThumb, SliderTrack},
};
use std::rc::Rc;

/// Controlled M3 switch: 52×32 track and an animated 16→24 thumb.
#[derive(gpui::IntoElement)]
pub struct MaterialSwitch {
    id: ElementId,
    label: SharedString,
    checked: bool,
    disabled: bool,
    on_change: Option<SwitchChange>,
}
type SwitchChange = Rc<dyn Fn(&bool, &mut Window, &mut App)>;
impl MaterialSwitch {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            checked: false,
            disabled: false,
            on_change: None,
        }
    }
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn on_change(mut self, f: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}
fn switch_geometry(t: f32) -> (f32, f32) {
    let t = t.clamp(0., 1.);
    (16. + 8. * t, 8. + 16. * t)
}
impl RenderOnce for MaterialSwitch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = motion::selection_value(
            (self.id.clone(), "thumb"),
            if self.checked { 1. } else { 0. },
            window,
            cx,
        );
        let (size, left) = switch_geometry(t);
        let action = self.on_change;
        gpui_base::Switch::new(self.id)
            .accessibility_label(self.label)
            .checked(self.checked)
            .disabled(self.disabled)
            .on_change(move |value, _, window, cx| {
                if let Some(f) = &action {
                    f(&value, window, cx);
                }
            })
            .relative()
            .w(rem(52.))
            .h(rem(32.))
            .rounded_full()
            .border_2()
            .border_color(color(if self.checked {
                ColorRole::ACCENT
            } else {
                ColorRole::CONTROL
            }))
            .bg(color(if self.checked {
                ColorRole::ACCENT
            } else {
                ColorRole::SURFACE_HIGH
            }))
            .when(self.disabled, |e| e.opacity(0.38))
            .child(
                div()
                    .absolute()
                    .left(rem(left - 2.))
                    .top(rem((32. - size) / 2. - 2.))
                    .size(rem(size))
                    .rounded_full()
                    .bg(color(if self.checked {
                        ColorRole::ON_PRIMARY
                    } else {
                        ColorRole::CONTROL
                    })),
            )
    }
}

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
impl RenderOnce for MaterialSlider {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let p = self.state.read(cx).percentage().end;
        let (track, width, height) = slider_geometry(self.appearance);
        Slider::new(&self.state)
            .accessibility_label(self.label)
            .disabled(self.disabled)
            .w_full()
            .h(rem(48.))
            .when(self.disabled, |e| e.opacity(0.38))
            .child(
                SliderTrack::new(&self.state)
                    .disabled(self.disabled)
                    .relative()
                    .w_full()
                    .h_full()
                    .child(
                        SliderIndicator::new(&self.state)
                            .absolute()
                            .left_0()
                            .top(rem((48. - track) / 2.))
                            .w_full()
                            .h(rem(track))
                            .rounded_full()
                            .bg(color(ColorRole::SECONDARY_CONTAINER))
                            .child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .top_0()
                                    .h_full()
                                    .w(relative(p))
                                    .rounded_full()
                                    .bg(color(ColorRole::ACCENT)),
                            ),
                    )
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
                            .bg(color(ColorRole::ACCENT)),
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
impl RenderOnce for MaterialSideSheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let progress = motion::value(
            (self.id.clone(), "reveal"),
            if self.open { 1. } else { 0. },
            window,
            cx,
        )
        .clamp(0., 1.);
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
            .children(self.children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    #[core::prelude::v1::test]
    fn switch_motion_preserves_track_clearance() {
        for t in [0., 0.25, 0.5, 0.75, 1.] {
            let (size, left) = switch_geometry(t);
            assert!(left >= 4.);
            assert!(left + size <= 48.);
        }
        assert_eq!(switch_geometry(0.), (16., 8.));
        assert_eq!(switch_geometry(1.), (24., 24.));
    }
    #[core::prelude::v1::test]
    fn slider_variants_have_distinct_handle_and_track() {
        assert_eq!(slider_geometry(SliderAppearance::Classic), (4., 20., 20.));
        assert_eq!(
            slider_geometry(SliderAppearance::Expressive),
            (16., 4., 44.)
        );
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
        let closed = cx.debug_bounds("sheet-body").unwrap();
        view.update(cx, |v, cx| {
            v.0 = true;
            cx.notify();
        });
        draw(cx);
        let open = cx.debug_bounds("sheet-body").unwrap();
        assert_eq!(closed.left() - open.left(), open.size.width);
        view.update(cx, |v, cx| {
            v.0 = false;
            cx.notify();
        });
        draw(cx);
        assert_eq!(cx.debug_bounds("sheet-body").unwrap(), closed);
    }
}
