//! GPUI Fast's wheel listener ignores a wheel that cannot move its box: no notify,
//! which would rebuild the view, and no scrolled mark, which would repaint every
//! scroll layer holding the box. A vertical wheel over a horizontal box arrives on
//! each event of a page scroll, so this is what keeps those boxes out of the frame.
//! `vendor/gpui-fast/src/elements/div.rs`, `paint_scroll_listener`.
#![cfg(feature = "gpui-fast")]
extern crate gpui_kit as gpui;

use core::prelude::v1::test;
use gpui::{prelude::*, *};
use std::{cell::Cell, rc::Rc};

const BOX: f32 = 100.;
const CONTENT: f32 = 160.;

struct Scroller {
    renders: Rc<Cell<usize>>,
    scroll: ScrollHandle,
}

impl Render for Scroller {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.renders.set(self.renders.get() + 1);
        div().size_full().child(
            div()
                .id("scroller")
                .track_scroll(&self.scroll)
                .overflow_x_scroll()
                .flex()
                .w(px(BOX))
                .h(px(50.))
                .child(div().flex_none().w(px(CONTENT)).h(px(50.))),
        )
    }
}

/// A draw always rebuilds the window's root, so the box's view sits below one.
struct Root(Entity<Scroller>);

impl Render for Root {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.0.clone())
    }
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.run_until_parked();
}

#[gpui::test]
fn a_wheel_that_cannot_move_the_box_rebuilds_nothing(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let renders = Rc::new(Cell::new(0));
    let scroll = ScrollHandle::new();
    let (_view, cx) = cx.add_window_view({
        let (renders, scroll) = (renders.clone(), scroll.clone());
        move |_, cx| Root(cx.new(|_| Scroller { renders, scroll }))
    });
    let inside = point(px(BOX / 2.), px(25.));
    draw(cx);
    cx.simulate_mouse_move(inside, None, Modifiers::default());
    draw(cx);

    let mut wheel = |dy: f32| {
        let before = renders.get();
        cx.simulate_event(ScrollEvent {
            position: inside,
            delta: ScrollDelta::Pixels(point(px(0.), px(dy))),
            ..Default::default()
        });
        draw(cx);
        renders.get() - before
    };

    // The window's first wheel changes its input modality, which refreshes it.
    wheel(20.);

    // A vertical wheel maps onto the horizontal box. At the start, one pointing
    // back cannot move it.
    assert_eq!(wheel(20.), 0, "a wheel towards the end it is at");
    assert_eq!(scroll.offset().x, px(0.));

    // One with room moves it, so the guard is not swallowing every wheel.
    assert_eq!(wheel(-20.), 1, "a wheel with room");
    assert_eq!(scroll.offset().x, px(-20.));

    // Past the far end the offset clamps, and then there is no room again.
    assert_eq!(wheel(-(CONTENT - BOX)), 1, "a wheel reaching the end");
    assert_eq!(scroll.offset().x, -px(CONTENT - BOX));
    assert_eq!(wheel(-20.), 0, "a wheel past the end");
    assert_eq!(scroll.offset().x, -px(CONTENT - BOX));
}
