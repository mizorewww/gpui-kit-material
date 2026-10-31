//! Tab order through the engine's lazy tab-stop index, against gpui-kit#3612. The index
//! keeps upstream's results, so a focus handle registered twice still traps reverse
//! Tab; Material registers none twice, and its roving widgets leave in both directions.
extern crate gpui_kit as gpui;

use core::prelude::v1::test;
use gpui::{prelude::*, *};
use gpui_kit::component::button::Button;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit_material::prelude::*;

struct SharedFocus {
    focus: FocusHandle,
}
impl Render for SharedFocus {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .child(Button::new("before").label("Before"))
            .child(
                div().id("outer").track_focus(&self.focus).child(
                    div()
                        .id("inner")
                        .track_focus(&self.focus)
                        .h(px(30.))
                        .child("Shared focus"),
                ),
            )
            .child(Button::new("after").label("After"))
    }
}

/// The issue's own reproduction, pinned at upstream's open behavior: when upstream
/// fixes it, the index must follow and the last assertion flips.
#[gpui::test]
fn reverse_tab_stays_on_a_shared_focus_handle_as_upstream_does(cx: &mut TestAppContext) {
    let (handle, focus) = cx.update(|cx| {
        gpui_kit::init(cx);
        let (handle, view) = gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|cx| SharedFocus {
                focus: cx.focus_handle().tab_stop(true),
            })
        })
        .expect("open minimal fixture");
        (handle, view.read(cx).focus.clone())
    });
    cx.update_window(handle, |_, window, cx| {
        window.activate_window();
        window.render_frame(cx);
        window.focus(&focus, cx);
        window.render_frame(cx);
        window.press("tab", cx);
        window.render_frame(cx);
        assert_eq!(window.find("after").focused(), Some(true));
        window.press("shift-tab", cx);
        window.render_frame(cx);
        assert_eq!(window.focused(cx), Some(focus.clone()));
        window.press("shift-tab", cx);
        window.render_frame(cx);
        assert_eq!(window.find("before").focused(), Some(false));
        assert_eq!(window.focused(cx), Some(focus.clone()));
    })
    .unwrap();
}

struct Roving {
    edges: [FocusHandle; 2],
    tab: SharedString,
}
impl Render for Roving {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let edge = |index: usize, focus: &FocusHandle| {
            div().id(("edge", index)).size(px(10.)).track_focus(focus)
        };
        div()
            .w(px(400.))
            .child(edge(0, &self.edges[0]))
            .child(
                MaterialTabs::new("tabs", "Sections")
                    .items((0..3).map(|index| TabItem::new(format!("t{index}"), "Tab")))
                    .selected(self.tab.clone())
                    .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                        this.tab = value.clone();
                        cx.notify();
                    })),
            )
            .child(edge(1, &self.edges[1]))
    }
}

/// Every stop in both directions: the row is one stop, and the arrow keys leave no
/// stale stop behind.
#[gpui::test]
fn roving_tabs_never_trap_either_direction(cx: &mut TestAppContext) {
    let (handle, edges) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit_material::init(cx);
        let (handle, view) = gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|cx| Roving {
                edges: [
                    cx.focus_handle().tab_stop(true),
                    cx.focus_handle().tab_stop(true),
                ],
                tab: "t0".into(),
            })
        })
        .expect("open fixture");
        (handle, view.read(cx).edges.clone())
    });
    cx.update_window(handle, |_, window, cx| {
        window.activate_window();
        window.render_frame(cx);
        window.focus(&edges[0], cx);
        window.render_frame(cx);
        window.focus_next(cx);
        window.render_frame(cx);
        let tab = window.focused(cx).expect("Tab enters the tab row");
        assert_ne!(tab, edges[0]);
        window.press("right", cx);
        assert_ne!(
            window.focused(cx),
            Some(tab.clone()),
            "the arrow moved focus"
        );
        window.focus_prev(cx);
        window.render_frame(cx);
        assert_eq!(
            window.focused(cx),
            Some(edges[0].clone()),
            "back out of the row"
        );
        window.focus_next(cx);
        window.render_frame(cx);
        window.focus_next(cx);
        window.render_frame(cx);
        assert_eq!(
            window.focused(cx),
            Some(edges[1].clone()),
            "forward out of the row"
        );
        window.focus_prev(cx);
        window.render_frame(cx);
        let back = window.focused(cx).expect("Shift+Tab enters the row");
        assert_ne!(back, edges[1]);
        window.focus_prev(cx);
        window.render_frame(cx);
        assert_eq!(
            window.focused(cx),
            Some(edges[0].clone()),
            "no second stop in the row"
        );
    })
    .unwrap();
}
