use super::*;
use crate::color::{ColorRole, PaletteId};
use crate::theme::{color, rgba};
use core::prelude::v1::test;
use gpui_kit::component::ActiveTheme as _;
use std::cell::Cell;

fn ink() -> Scheme {
    Scheme::from_palette(PaletteId::Ink)
}
fn paper() -> Scheme {
    Scheme::from_palette(PaletteId::Paper)
}

#[test]
fn arrows_follow_the_reading_direction() {
    assert_eq!(horizontal_arrow("right", false), Some(1));
    assert_eq!(horizontal_arrow("right", true), Some(-1));
    assert_eq!(horizontal_arrow("left", true), Some(1));
    assert_eq!(horizontal_arrow("up", true), None);
}

/// A render-time reader of the scope's colors.
#[derive(gpui::IntoElement)]
struct Probe(Rc<Cell<Option<Rgba>>>);
impl RenderOnce for Probe {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.0.set(Some(color(ColorRole::SURFACE)));
        div().size(px(10.))
    }
}

struct Harness {
    renders: Rc<Cell<usize>>,
    scheme: Scheme,
    rendered: Rc<Cell<Option<Rgba>>>,
    painted: Rc<Cell<Option<Rgba>>>,
    deferred: Rc<Cell<Option<Rgba>>>,
    kit: Rc<Cell<Option<Hsla>>>,
    outside: Rc<Cell<Option<Rgba>>>,
}

impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.renders.set(self.renders.get() + 1);
        let painted = self.painted.clone();
        let deferred_ink = self.deferred.clone();
        let kit = self.kit.clone();
        let outside = self.outside.clone();
        div()
            .size_full()
            .child(Scope::new("scope").scheme(self.scheme).child_with({
                let rendered = self.rendered.clone();
                move |_, _| {
                    div()
                        .size_full()
                        .child(Probe(rendered))
                        .child(
                            canvas(
                                |_, _, _| {},
                                move |_, _, _, cx| {
                                    painted.set(Some(color(ColorRole::SURFACE)));
                                    kit.set(Some(cx.theme().background));
                                },
                            )
                            .size(px(1.)),
                        )
                        .child(deferred(current_scope(
                            canvas(
                                |_, _, _| {},
                                move |_, _, _, _| {
                                    deferred_ink.set(Some(color(ColorRole::SURFACE)));
                                },
                            )
                            .size(px(1.)),
                        )))
                        .into_any_element()
                }
            }))
            .child(
                canvas(
                    |_, _, _| {},
                    move |_, _, _, _| outside.set(Some(color(ColorRole::SURFACE))),
                )
                .size(px(1.)),
            )
    }
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

fn open(cx: &mut TestAppContext) -> (Entity<Harness>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        theme::init_component_theme(paper(), 1., cx);
        cx.set_reduce_motion(true);
    });
    cx.add_window_view(|_, _| Harness {
        renders: Rc::default(),
        scheme: ink(),
        rendered: Rc::default(),
        painted: Rc::default(),
        deferred: Rc::default(),
        kit: Rc::default(),
        outside: Rc::default(),
    })
}

#[gpui::test]
fn every_phase_and_kit_read_the_scoped_scheme_and_the_outside_does_not(cx: &mut TestAppContext) {
    let (view, cx) = open(cx);
    draw(cx);
    let want = Some(rgba(ink().get(ColorRole::SURFACE)));
    view.read_with(cx, |h, _| {
        assert_eq!(h.rendered.get(), want, "render");
        assert_eq!(h.painted.get(), want, "paint");
        assert_eq!(h.deferred.get(), want, "a deferred draw re-enters");
        assert_eq!(
            h.kit.get(),
            want.map(Hsla::from),
            "Kit's theme is projected while the scope paints"
        );
        assert_eq!(
            h.outside.get(),
            Some(rgba(paper().get(ColorRole::SURFACE))),
            "a sibling after the scope reads the application's scheme"
        );
    });
    cx.update(|_, cx| {
        assert_eq!(
            cx.theme().background,
            Hsla::from(rgba(paper().get(ColorRole::SURFACE))),
            "Kit's global is restored when the draw ends"
        );
        assert_eq!(theme::scheme(), paper());
    });
}

#[gpui::test]
fn a_settled_scope_does_not_keep_the_window_drawing(cx: &mut TestAppContext) {
    let (view, cx) = open(cx);
    draw(cx);
    cx.run_until_parked();
    let dirty_draw = |cx: &mut VisualTestContext| {
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
    };
    dirty_draw(cx);
    let settled = view.read_with(cx, |h, _| h.renders.get());
    for _ in 0..3 {
        dirty_draw(cx);
    }
    assert_eq!(
        view.read_with(cx, |h, _| h.renders.get()),
        settled,
        "swapping Kit's globals notifies no view"
    );
}

#[gpui::test]
fn a_scope_change_transitions_on_its_own_and_lands(cx: &mut TestAppContext) {
    let (view, cx) = open(cx);
    draw(cx);
    cx.update(|_, cx| cx.set_reduce_motion(false));
    view.update(cx, |h, _| h.scheme = paper());
    draw(cx);
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(50));
    draw(cx);
    let (from, to) = (
        rgba(ink().get(ColorRole::SURFACE)),
        rgba(paper().get(ColorRole::SURFACE)),
    );
    let mid = view.read_with(cx, |h, _| h.painted.get()).unwrap();
    assert!(mid != from && mid != to, "mid-transition, not {mid:?}");
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(3));
    draw(cx);
    draw(cx);
    assert_eq!(view.read_with(cx, |h, _| h.painted.get()), Some(to));
}

struct DirectionHarness {
    direction: LayoutDirection,
}
impl Render for DirectionHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let row = || {
            div()
                .flex()
                .w(px(100.))
                .h(px(10.))
                .pl(px(5.))
                .child(div().w(px(10.)).h_full().debug_selector(|| "a".into()))
                .child(div().w(px(10.)).h_full().debug_selector(|| "b".into()))
                .child(
                    div()
                        .absolute()
                        .left(px(1.))
                        .w(px(2.))
                        .h_full()
                        .debug_selector(|| "abs".into()),
                )
        };
        div()
            .id("root")
            .w(px(100.))
            .child(Scope::new("dir").direction(self.direction).child(row()))
    }
}

#[gpui::test]
fn a_direction_scope_mirrors_its_subtree_and_reads_back(cx: &mut TestAppContext) {
    cx.update(|cx| gpui_kit::init(cx));
    let (view, cx) = cx.add_window_view(|_, _| DirectionHarness {
        direction: LayoutDirection::Ltr,
    });
    draw(cx);
    let x = |cx: &mut VisualTestContext, name| cx.debug_bounds(name).unwrap().left();
    assert_eq!(
        (x(cx, "a"), x(cx, "b"), x(cx, "abs")),
        (px(5.), px(15.), px(1.))
    );
    view.update(cx, |h, _| h.direction = LayoutDirection::Rtl);
    draw(cx);
    assert_eq!(
        (x(cx, "a"), x(cx, "b"), x(cx, "abs")),
        (px(85.), px(75.), px(97.)),
        "first in reading order on the right; padding and offsets flip too"
    );
}

use crate::components::{MaterialSlider, MaterialTabs, TabItem};
use gpui_base::slider::SliderState;

struct RtlControls {
    tab: SharedString,
    slider: Entity<SliderState>,
}
impl Render for RtlControls {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let this = cx.weak_entity();
        Scope::new("rtl").direction(LayoutDirection::Rtl).child(
            div()
                .w(px(320.))
                .flex()
                .flex_col()
                .child(
                    div().debug_selector(|| "tabs".into()).child(
                        MaterialTabs::new("tabs", "Tabs")
                            .items([
                                TabItem::new("a", "A"),
                                TabItem::new("b", "B"),
                                TabItem::new("c", "C"),
                            ])
                            .selected(self.tab.clone())
                            .on_change(move |value: &SharedString, _, cx| {
                                let value = value.clone();
                                let _ = this.update(cx, |this, cx| {
                                    this.tab = value;
                                    cx.notify();
                                });
                            }),
                    ),
                )
                .child(
                    div()
                        .debug_selector(|| "slider".into())
                        .child(MaterialSlider::new(&self.slider, "Level")),
                ),
        )
    }
}

fn rtl_controls(cx: &mut TestAppContext) -> (Entity<RtlControls>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    cx.add_window_view(|_, cx| RtlControls {
        tab: "a".into(),
        slider: cx.new(|_| SliderState::new().min(0.).max(100.).default_value(50.)),
    })
}

#[gpui::test]
fn arrows_move_tabs_in_the_reading_direction(cx: &mut TestAppContext) {
    let (view, cx) = rtl_controls(cx);
    draw(cx);
    let tabs = cx.debug_bounds("tabs").unwrap();
    // The first tab is on the right; pressing it focuses the row.
    let first = point(tabs.right() - px(20.), tabs.center().y);
    cx.simulate_click(first, Modifiers::default());
    draw(cx);
    assert_eq!(view.read_with(cx, |h, _| h.tab.clone()), "a");
    cx.simulate_keystrokes("left");
    draw(cx);
    assert_eq!(
        view.read_with(cx, |h, _| h.tab.clone()),
        "b",
        "left moves forward in a right-to-left row"
    );
    cx.simulate_keystrokes("right");
    draw(cx);
    assert_eq!(view.read_with(cx, |h, _| h.tab.clone()), "a");
}

#[gpui::test]
fn a_slider_grows_from_the_right(cx: &mut TestAppContext) {
    let (view, cx) = rtl_controls(cx);
    draw(cx);
    let slider = cx.debug_bounds("slider").unwrap();
    // A press near the right end is near the start of the range.
    let near_start = point(slider.right() - px(30.), slider.center().y);
    cx.simulate_click(near_start, Modifiers::default());
    draw(cx);
    let value = view.read_with(cx, |h, cx| h.slider.read(cx).value().end());
    assert!(value < 25., "pressed near the start, got {value}");
    let near_end = point(slider.left() + px(30.), slider.center().y);
    cx.simulate_click(near_end, Modifiers::default());
    draw(cx);
    let value = view.read_with(cx, |h, cx| h.slider.read(cx).value().end());
    assert!(value > 75., "pressed near the end, got {value}");
    // Keys follow the same direction.
    cx.simulate_keystrokes("left");
    draw(cx);
    let after = view.read_with(cx, |h, cx| h.slider.read(cx).value().end());
    assert!(after > value, "left increases in a right-to-left slider");
}
