//! Tests of which nested scroll container a wheel gesture scrolls. See
//! [`crate::fast::layers::wheel_latch`].

use crate::{
    AppContext as _, Context, InteractiveElement as _, IntoElement, ListAlignment, ListState,
    ParentElement as _, PlatformInput, Render, ScrollDelta, ScrollEvent, ScrollHandle,
    StatefulInteractiveElement as _, Styled as _, TestAppContext, TouchPhase, Window, WindowHandle,
    div, list, point, px, rgb,
};

/// What the page holds above its rows.
#[derive(Clone, Copy)]
enum Inner {
    /// A div scroll container, 50 tall, of 200.
    Div,
    /// A list of 10 rows of 20, 50 tall.
    List,
    /// A div that scrolls only horizontally, 100 wide, of 300.
    Strip,
}

/// A page 100 tall that scrolls 400, holding an inner scroll container at
/// its top and rows below it.
struct Page {
    page: ScrollHandle,
    inner: ScrollHandle,
    list: ListState,
    kind: Inner,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let inner = match self.kind {
            Inner::Div => div()
                .id("inner")
                .overflow_y_scroll()
                .track_scroll(&self.inner)
                .w(px(200.))
                .h(px(50.))
                .child(div().w(px(200.)).h(px(200.)).bg(rgb(0x00ff00)))
                .into_any_element(),
            Inner::List => list(self.list.clone(), |_, _, _| {
                div()
                    .w(px(200.))
                    .h(px(20.))
                    .bg(rgb(0x00ff00))
                    .into_any_element()
            })
            .w(px(200.))
            .h(px(50.))
            .into_any_element(),
            Inner::Strip => div()
                .id("inner")
                .overflow_x_scroll()
                .track_scroll(&self.inner)
                .flex()
                .w(px(100.))
                .h(px(50.))
                .child(div().flex_none().w(px(300.)).h(px(50.)).bg(rgb(0x00ff00)))
                .into_any_element(),
        };
        div().size_full().child(
            div()
                .id("page")
                .overflow_y_scroll()
                .track_scroll(&self.page)
                .w(px(200.))
                .h(px(100.))
                .child(inner)
                .child(div().w(px(200.)).h(px(450.)).bg(rgb(0xff0000))),
        )
    }
}

fn open(cx: &mut TestAppContext, kind: Inner, layers: bool) -> WindowHandle<Page> {
    let window = cx.add_window(move |_, _| Page {
        page: ScrollHandle::new(),
        inner: ScrollHandle::new(),
        list: ListState::new(10, ListAlignment::Top, px(100.)).measure_all(),
        kind,
    });
    cx.update_window(window.into(), |_, window, cx| {
        window.set_scroll_layers(layers);
        window.draw(cx).clear(cx);
    })
    .unwrap();
    window
}

/// Dispatches a wheel of `dy` at `y` in `phase`, then draws a frame.
fn wheel(cx: &mut TestAppContext, window: WindowHandle<Page>, y: f32, dy: f32, phase: TouchPhase) {
    wheel_by(
        cx,
        window,
        y,
        ScrollDelta::Pixels(point(px(0.), px(dy))),
        phase,
    );
}

/// Dispatches a wheel of `delta` at `y` in `phase`, then draws a frame.
fn wheel_by(
    cx: &mut TestAppContext,
    window: WindowHandle<Page>,
    y: f32,
    delta: ScrollDelta,
    phase: TouchPhase,
) {
    cx.update_window(window.into(), |_, window, cx| {
        window.dispatch_event(
            PlatformInput::Scroll(ScrollEvent {
                position: point(px(20.), px(y)),
                delta,
                touch_phase: phase,
                ..Default::default()
            }),
            cx,
        );
        window.draw(cx).clear(cx);
    })
    .unwrap();
}

/// How far the page and the inner container have scrolled, down.
fn offsets(cx: &mut TestAppContext, window: WindowHandle<Page>) -> (f32, f32) {
    window
        .read_with(cx, |page, _| {
            let inner = match page.kind {
                Inner::Div => -page.inner.offset().y,
                Inner::List => page.list.scroll_px_offset_for_scrollbar().y.abs(),
                Inner::Strip => -page.inner.offset().x,
            };
            (f32::from(-page.page.offset().y), f32::from(inner))
        })
        .unwrap()
}

fn each_case(test: impl Fn(&mut TestAppContext, Inner, bool)) {
    for kind in [Inner::Div, Inner::List] {
        for layers in [false, true] {
            let mut cx = TestAppContext::single();
            test(&mut cx, kind, layers);
        }
    }
}

/// A wheel over a nested container moves it, and not the page around it.
#[test]
fn a_wheel_over_a_nested_container_moves_only_it() {
    each_case(|cx, kind, layers| {
        let window = open(cx, kind, layers);
        wheel(cx, window, 20., -30., TouchPhase::Started);
        wheel(cx, window, 20., -30., TouchPhase::Moved);
        assert_eq!(offsets(cx, window), (0., 60.), "layers {layers}");
    });
}

/// A gesture that brings the nested container to its end keeps it: the
/// rest of the swipe, and its momentum, do not move the page. The next
/// gesture does.
#[test]
fn a_nested_container_at_its_end_keeps_the_gesture() {
    each_case(|cx, kind, layers| {
        let window = open(cx, kind, layers);
        wheel(cx, window, 20., -100., TouchPhase::Started);
        wheel(cx, window, 20., -100., TouchPhase::Moved);
        wheel(cx, window, 20., -100., TouchPhase::Ended);
        wheel(cx, window, 20., -100., TouchPhase::Moved);
        assert_eq!(offsets(cx, window), (0., 150.), "layers {layers}");

        wheel(cx, window, 20., -30., TouchPhase::Started);
        assert_eq!(offsets(cx, window), (30., 150.), "layers {layers}");
    });
}

/// A page scrolling under a still pointer keeps the gesture when a nested
/// container comes under the pointer.
#[test]
fn the_page_keeps_its_gesture_over_a_nested_container() {
    each_case(|cx, kind, layers| {
        let window = open(cx, kind, layers);
        // The nested container to its end, then the page by a new gesture.
        wheel(cx, window, 20., -200., TouchPhase::Started);
        wheel(cx, window, 20., -60., TouchPhase::Started);
        assert_eq!(offsets(cx, window), (60., 150.), "layers {layers}");

        // Back up from below the nested container, until it is under the
        // pointer and could move.
        wheel(cx, window, 20., 15., TouchPhase::Started);
        wheel(cx, window, 20., 30., TouchPhase::Moved);
        assert_eq!(offsets(cx, window), (15., 150.), "layers {layers}");
        wheel(cx, window, 20., 10., TouchPhase::Moved);
        assert_eq!(offsets(cx, window), (5., 150.), "layers {layers}");
    });
}

/// A pointer that moved starts a new gesture, as a new touch does.
#[test]
fn a_moved_pointer_starts_a_new_gesture() {
    each_case(|cx, kind, layers| {
        let window = open(cx, kind, layers);
        wheel(cx, window, 20., -200., TouchPhase::Moved);
        wheel(cx, window, 20., -30., TouchPhase::Moved);
        assert_eq!(offsets(cx, window), (0., 150.), "layers {layers}");
        wheel(cx, window, 21., -30., TouchPhase::Moved);
        assert_eq!(offsets(cx, window), (30., 150.), "layers {layers}");
    });
}

/// A vertical wheel over a container that scrolls only horizontally moves it
/// onto its one axis, and the page with it, as before, for the whole gesture.
#[test]
fn a_mapped_wheel_moves_the_page_too() {
    for layers in [false, true] {
        let mut cx = TestAppContext::single();
        let window = open(&mut cx, Inner::Strip, layers);
        wheel(&mut cx, window, 10., -30., TouchPhase::Started);
        assert_eq!(offsets(&mut cx, window), (30., 30.), "layers {layers}");
        // The page keeps the gesture, and the wheel still moves the strip.
        wheel(&mut cx, window, 10., -10., TouchPhase::Moved);
        assert_eq!(offsets(&mut cx, window), (40., 40.), "layers {layers}");
    }
}

/// Wheel listeners other than the containers' own scrolling still hear the
/// event: the nested container's `on_scroll` and the page's.
#[test]
fn other_wheel_listeners_still_hear_the_event() {
    use std::{cell::Cell, rc::Rc};

    struct Heard {
        inner: Rc<Cell<u32>>,
        page: Rc<Cell<u32>>,
        handle: ScrollHandle,
        page_handle: ScrollHandle,
    }
    impl Render for Heard {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let (inner, page) = (self.inner.clone(), self.page.clone());
            div()
                .id("page")
                .overflow_y_scroll()
                .track_scroll(&self.page_handle)
                .on_scroll(move |_, _, _| page.set(page.get() + 1))
                .w(px(200.))
                .h(px(100.))
                .child(
                    div()
                        .id("inner")
                        .overflow_y_scroll()
                        .track_scroll(&self.handle)
                        .on_scroll(move |_, _, _| inner.set(inner.get() + 1))
                        .w(px(200.))
                        .h(px(50.))
                        .child(div().w(px(200.)).h(px(200.))),
                )
                .child(div().w(px(200.)).h(px(450.)))
        }
    }

    let mut cx = TestAppContext::single();
    let (inner, page) = (Rc::new(Cell::new(0)), Rc::new(Cell::new(0)));
    let (handle, page_handle) = (ScrollHandle::new(), ScrollHandle::new());
    let window = cx.add_window({
        let (inner, page) = (inner.clone(), page.clone());
        let (handle, page_handle) = (handle.clone(), page_handle.clone());
        move |_, _| Heard {
            inner,
            page,
            handle,
            page_handle,
        }
    });
    cx.update_window(window.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
        window.dispatch_event(
            PlatformInput::Scroll(ScrollEvent {
                position: point(px(20.), px(20.)),
                delta: ScrollDelta::Pixels(point(px(0.), px(-30.))),
                touch_phase: TouchPhase::Started,
                ..Default::default()
            }),
            cx,
        );
    })
    .unwrap();
    assert_eq!(f32::from(-handle.offset().y), 30.);
    assert_eq!(f32::from(-page_handle.offset().y), 0.);
    assert_eq!((inner.get(), page.get()), (1, 1));
}

/// After a wheel's last event, or momentum's, the gesture ends once
/// `LATCH_IDLE` passes without another, and the next wheel scrolls the page.
/// While a trackpad's fingers are down it lasts however long they pause.
#[test]
fn a_gesture_ends_when_its_events_stop() {
    let idle = crate::fast::layers::wheel_latch::LATCH_IDLE + std::time::Duration::from_millis(1);
    for layers in [false, true] {
        let mut cx = TestAppContext::single();
        let window = open(&mut cx, Inner::Div, layers);
        wheel(&mut cx, window, 20., -200., TouchPhase::Moved);
        cx.executor().advance_clock(idle / 2);
        wheel(&mut cx, window, 20., -30., TouchPhase::Moved);
        assert_eq!(offsets(&mut cx, window), (0., 150.), "layers {layers}");
        cx.executor().advance_clock(idle);
        wheel(&mut cx, window, 20., -30., TouchPhase::Moved);
        assert_eq!(offsets(&mut cx, window), (30., 150.), "layers {layers}");

        let mut cx = TestAppContext::single();
        let window = open(&mut cx, Inner::Div, layers);
        wheel(&mut cx, window, 20., -200., TouchPhase::Started);
        cx.executor().advance_clock(idle * 4);
        wheel(&mut cx, window, 20., -30., TouchPhase::Moved);
        assert_eq!(offsets(&mut cx, window), (0., 150.), "layers {layers}");
    }
}

/// A mouse wheel's clicks, in lines, keep their gesture as a swipe does.
#[test]
fn a_mouse_wheel_keeps_its_gesture() {
    each_case(|cx, kind, layers| {
        let window = open(cx, kind, layers);
        for _ in 0..4 {
            wheel_by(
                cx,
                window,
                20.,
                ScrollDelta::Lines(point(0., -3.)),
                TouchPhase::Moved,
            );
        }
        let (page, inner) = offsets(cx, window);
        assert_eq!((page, inner), (0., 150.), "layers {layers}");
    });
}

/// A vertical swipe with a little sideways movement over a strip that
/// scrolls only horizontally scrolls the page, and does not hand the gesture
/// to the strip.
#[test]
fn a_strip_does_not_take_a_mainly_vertical_swipe() {
    for layers in [false, true] {
        let mut cx = TestAppContext::single();
        let window = open(&mut cx, Inner::Strip, layers);
        let swipe = ScrollDelta::Pixels(point(px(-0.5), px(-12.)));
        wheel_by(&mut cx, window, 10., swipe, TouchPhase::Started);
        wheel_by(&mut cx, window, 10., swipe, TouchPhase::Moved);
        assert_eq!(offsets(&mut cx, window), (24., 1.), "layers {layers}");
    }
}
