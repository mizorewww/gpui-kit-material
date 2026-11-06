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
fn horizontal_arrows_move_right_forward() {
    assert_eq!(horizontal_arrow("right"), Some(1));
    assert_eq!(horizontal_arrow("left"), Some(-1));
    assert_eq!(horizontal_arrow("up"), None);
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

/// A view that paints its scope's color, notified on its own.
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
    let inner = root.read_with(cx, |root, cx| root.0.read(cx).0.clone());
    seen.set(None);
    inner.update(cx, |_, cx| cx.notify());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.run_until_parked();
    assert_eq!(seen.get(), want, "rebuilt in the scope's scheme");
}

struct CachedOuter(Entity<Outer>);
impl Render for CachedOuter {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        AnyView::from(self.0.clone()).cached(StyleRefinement::default().size_full())
    }
}
