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

/// Holds a harness in a cached view, as an application's root does.
struct CachedRoot(Entity<Harness>);
impl Render for CachedRoot {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        AnyView::from(self.0.clone()).cached(StyleRefinement::default().size_full())
    }
}

#[gpui::test]
fn a_settled_scope_does_not_keep_the_window_drawing(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        theme::init_component_theme(paper(), 1., cx);
        cx.set_reduce_motion(true);
    });
    let (root, cx) = cx.add_window_view(|_, cx| {
        CachedRoot(cx.new(|_| Harness {
            renders: Rc::default(),
            scheme: ink(),
            rendered: Rc::default(),
            painted: Rc::default(),
            deferred: Rc::default(),
            kit: Rc::default(),
            outside: Rc::default(),
        }))
    });
    let dirty_draw = |cx: &mut VisualTestContext| {
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
    };
    dirty_draw(cx);
    dirty_draw(cx);
    let view = root.read_with(cx, |root, _| root.0.clone());
    let notified = Rc::new(Cell::new(0));
    let _watch = cx.update(|_, cx| {
        let notified = notified.clone();
        cx.observe(&view, move |_, _| notified.set(notified.get() + 1))
    });
    for _ in 0..3 {
        dirty_draw(cx);
    }
    // Fast stamps the swapped globals, so a draw rebuilds what read them while a
    // scheme scope is on screen; but nothing is notified, so no draw is asked for.
    assert_eq!(notified.get(), 0, "swapping Kit's globals notifies no view");
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
    cx.update(gpui_kit::init);
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
                            .auto_activate(true)
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

/// Reads Kit's theme and the scope's color while rendering in a deferred draw.
#[derive(gpui::IntoElement)]
struct KitProbe(Rc<Cell<Option<(Hsla, Rgba)>>>);
impl RenderOnce for KitProbe {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        self.0
            .set(Some((cx.theme().background, color(ColorRole::SURFACE))));
        div().size(px(1.))
    }
}

struct DeferredKit(Rc<Cell<Option<(Hsla, Rgba)>>>);
impl Render for DeferredKit {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let seen = self.0.clone();
        Scope::new("kit").scheme(ink()).child_with(move |_, _| {
            div()
                .child(deferred(current_scope(KitProbe(seen))))
                .into_any_element()
        })
    }
}

#[gpui::test]
fn deferred_content_laid_out_in_place_keeps_kit_in_the_scope(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        theme::init_component_theme(paper(), 1., cx);
        cx.set_reduce_motion(true);
    });
    let seen = Rc::new(Cell::new(None));
    let (_, cx) = cx.add_window_view({
        let seen = seen.clone();
        move |_, _| DeferredKit(seen)
    });
    draw(cx);
    let want = rgba(ink().get(ColorRole::SURFACE));
    assert_eq!(seen.get(), Some((Hsla::from(want), want)));
}

/// A view that paints its scope's color and position, notified on its own.
struct Inner {
    color: Rc<Cell<Option<Rgba>>>,
}
impl Render for Inner {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.color.set(Some(color(ColorRole::SURFACE)));
        div().flex().w_full().h(px(10.)).child(
            div()
                .w(px(10.))
                .h_full()
                .debug_selector(|| "inner-cell".into()),
        )
    }
}

struct Outer(Entity<Inner>);
impl Render for Outer {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Scope::new("outer")
            .scheme(ink())
            .direction(LayoutDirection::Rtl)
            .child(
                div().w(px(100.)).child(
                    AnyView::from(self.0.clone())
                        .cached(StyleRefinement::default().w(px(100.)).h(px(10.))),
                ),
            )
    }
}

#[gpui::test]
fn a_view_notified_alone_stays_in_its_scope(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        theme::init_component_theme(paper(), 1., cx);
        cx.set_reduce_motion(true);
    });
    let seen = Rc::new(Cell::new(None));
    let inner_seen = seen.clone();
    let (root, cx) = cx.add_window_view(move |_, cx| {
        let inner = cx.new(|_| Inner { color: inner_seen });
        let outer = cx.new(|_| Outer(inner));
        CachedOuter(outer)
    });
    draw(cx);
    let want = Some(rgba(ink().get(ColorRole::SURFACE)));
    assert_eq!(seen.get(), want);
    assert_eq!(cx.debug_bounds("inner-cell").unwrap().left(), px(90.));
    let inner = root.read_with(cx, |root, cx| root.0.read(cx).0.clone());
    seen.set(None);
    inner.update(cx, |_, cx| cx.notify());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.run_until_parked();
    assert_eq!(seen.get(), want, "rebuilt in the scope's scheme");
    assert_eq!(
        cx.debug_bounds("inner-cell").unwrap().left(),
        px(90.),
        "and direction"
    );
}

struct CachedOuter(Entity<Outer>);
impl Render for CachedOuter {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        AnyView::from(self.0.clone()).cached(StyleRefinement::default().size_full())
    }
}

#[gpui::test]
fn an_application_direction_reaches_the_window_that_set_it(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let (_, cx) = cx.add_window_view(|_, _| DirectionHarness {
        direction: LayoutDirection::Ltr,
    });
    draw(cx);
    cx.update(|window, cx| {
        set_app_direction(LayoutDirection::Rtl, cx);
        assert_eq!(cx.layout_direction(), LayoutDirection::Rtl);
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(window.layout_direction(), LayoutDirection::Rtl);
        set_window_direction(LayoutDirection::Ltr, window);
        set_app_direction(LayoutDirection::Rtl, cx);
        window.draw(cx).clear(cx);
        assert_eq!(
            window.layout_direction(),
            LayoutDirection::Ltr,
            "a window that chose keeps its own"
        );
    });
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

struct ScrolledRow {
    scroll: gpui::ScrollHandle,
    rtl: bool,
}
impl Render for ScrolledRow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let direction = if self.rtl {
            LayoutDirection::Rtl
        } else {
            LayoutDirection::Ltr
        };
        Scope::new("scrolled").direction(direction).child(
            div()
                .id("list")
                .w(px(100.))
                .h(px(40.))
                .overflow_y_scroll()
                .track_scroll(&self.scroll)
                .child(
                    div()
                        .h(px(200.))
                        .flex()
                        .child(div().w(px(10.)).h_full().debug_selector(|| "cell".into())),
                ),
        )
    }
}

#[gpui::test]
fn mirrored_layout_survives_retained_frames_and_scrolling(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let (view, cx) = cx.add_window_view(|_, _| ScrolledRow {
        scroll: gpui::ScrollHandle::new(),
        rtl: true,
    });
    let dirty = |cx: &mut VisualTestContext| {
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
    };
    draw(cx);
    let at = cx.debug_bounds("cell").unwrap();
    assert_eq!(at.left(), px(90.), "mirrored");
    // Scroll without a refresh: Fast replays the layer rather than laying out.
    cx.simulate_event(ScrollEvent {
        position: point(px(50.), px(20.)),
        delta: ScrollDelta::Pixels(point(px(0.), px(-30.))),
        modifiers: Modifiers::default(),
        touch_phase: TouchPhase::Moved,
        ..Default::default()
    });
    dirty(cx);
    let moved = cx.debug_bounds("cell").unwrap();
    assert_eq!(moved.left(), px(90.), "still mirrored after the replay");
    assert_eq!(moved.top(), at.top() - px(30.), "and scrolled");
    // A view notification reuses retained nodes: the record must hold.
    view.update(cx, |_, cx| cx.notify());
    dirty(cx);
    assert_eq!(cx.debug_bounds("cell").unwrap().left(), px(90.));
    // Changing direction lays the same retained nodes out the other way.
    view.update(cx, |view, cx| {
        view.rtl = false;
        cx.notify();
    });
    dirty(cx);
    dirty(cx);
    assert_eq!(
        cx.debug_bounds("cell").unwrap().left(),
        px(0.),
        "unmirrored"
    );
}
