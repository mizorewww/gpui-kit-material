//! A list scrolling an input changes nothing the input draws, so it must not mark the
//! input's view dirty: every row with one would be rendered again on every frame, and
//! the list would never stay on its scroll layer. Only what reads the caret needs to
//! know, as an open menu does, and only a focused input has one of those. A new size is
//! a change for everyone. `vendor/gpui-base/src/input/base/element.rs`, the input's
//! paint. Fast alone can reuse a view, so only it can tell.
#![cfg(feature = "gpui-fast")]
extern crate gpui_kit as gpui;

use core::prelude::v1::test;
use gpui::{prelude::*, *};
use gpui_kit::component::input::{Input, InputState};
use std::{cell::Cell, rc::Rc};

struct Moved {
    input: Entity<InputState>,
    top: Rc<Cell<f32>>,
    width: Rc<Cell<f32>>,
}

impl Render for Moved {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .pt(px(self.top.get()))
            .child(div().w(px(self.width.get())).child(Input::new(&self.input)))
    }
}

#[gpui::test]
fn an_input_moved_by_its_parent_is_dirty_only_while_focused(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let (top, width) = (Rc::new(Cell::new(0.)), Rc::new(Cell::new(200.)));
    let mut input = None;
    let window = cx.open_window(size(px(400.), px(300.)), |window, cx| {
        let state = cx.new(|cx| InputState::new(window, cx).default_value("a"));
        input = Some(state.clone());
        Moved {
            input: state,
            top: top.clone(),
            width: width.clone(),
        }
    });
    let input = input.unwrap();
    let mut cx = VisualTestContext::from_window(window.into(), cx);
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.run_until_parked();
    // Paints the input where it now is, then counts the views a draw with nothing else
    // to do builds: a notify during a draw only marks a view dirty, for the next draw.
    let dirty_after_painting_at = |y: f32, cx: &mut VisualTestContext| {
        top.set(y);
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let built = window.layout_stats().views_built;
            window.draw(cx).clear(cx);
            window.layout_stats().views_built - built
        })
    };

    for y in [10., 20., 30.] {
        assert_eq!(dirty_after_painting_at(y, &mut cx), 0, "moved, not focused");
    }

    width.set(240.);
    assert_ne!(dirty_after_painting_at(30., &mut cx), 0, "a new size");

    cx.update(|window, cx| input.update(cx, |state, cx| state.focus(window, cx)));
    dirty_after_painting_at(30., &mut cx);
    assert_ne!(dirty_after_painting_at(40., &mut cx), 0, "moved, focused");
}
