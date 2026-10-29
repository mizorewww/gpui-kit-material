//! Focus presentation for composed controls, and scrolling that reveals
//! newly focused rows without pinning them during manual scrolling.
use crate::color::ColorRole;
use gpui::{prelude::*, *};

/// Draw a visible outline around a component whose focus handle is internal.
/// The wrapper observes descendants without adding a keyboard stop of its own.
#[derive(IntoElement)]
pub struct FocusRing {
    id: ElementId,
    child: AnyElement,
}

impl FocusRing {
    pub fn new(id: impl Into<ElementId>, child: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            child: child.into_any_element(),
        }
    }
}

struct RingState {
    container: FocusHandle,
    active: bool,
    /// The window focus when containment was last resolved.
    seen: Option<FocusHandle>,
}

impl RenderOnce for FocusRing {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| RingState {
            container: cx.focus_handle(),
            active: false,
            seen: None,
        });
        let (focus, active) = (state.read(cx).container.clone(), state.read(cx).active);
        let focused = window.focused(cx);
        // Containment is only reliable after the frame commits, so resolve it
        // then, and only when window focus has moved since the last check.
        if state.read(cx).seen != focused {
            let observed = state.clone();
            window.defer(cx, move |window, cx| {
                let active = observed.read(cx).container.contains_focused(window, cx);
                let changed = active != observed.read(cx).active;
                observed.update(cx, |state, _| {
                    state.active = active;
                    state.seen = focused;
                });
                if changed {
                    window.refresh();
                }
            });
        }
        div()
            .id(self.id)
            .track_focus(&focus)
            .tab_stop(false)
            .flex_shrink_0()
            .border_2()
            .border_color(if active {
                crate::theme::color(ColorRole::INK).into()
            } else {
                transparent_black()
            })
            .rounded_full()
            .child(self.child)
    }
}

#[derive(Clone)]
enum RevealScroll {
    Handle(ScrollHandle),
    List(ListState),
}

impl RevealScroll {
    fn viewport(&self) -> Bounds<Pixels> {
        match self {
            RevealScroll::Handle(scroll) => scroll.bounds(),
            RevealScroll::List(state) => state.viewport_bounds(),
        }
    }

    fn scroll_by(&self, delta: Pixels) {
        match self {
            RevealScroll::Handle(scroll) => {
                scroll.set_offset(scroll.offset() + point(px(0.), delta));
            }
            // Reveal deltas move the content; ListState distances move the
            // viewport through the content, so their signs are opposite.
            RevealScroll::List(state) => state.scroll_by(-delta),
        }
    }
}

/// Scrolls `scroll` so a newly focused descendant is visible, once per focus
/// change or geometry change, never while the person scrolls by hand.
#[derive(IntoElement)]
pub struct RevealFocus {
    id: ElementId,
    child: AnyElement,
    scroll: RevealScroll,
    full_width: bool,
}

struct State {
    container: FocusHandle,
    focused: Option<FocusHandle>,
    /// Window focus and geometry when the reveal was last resolved.
    seen: Option<(Option<FocusHandle>, Pixels, Size<Pixels>)>,
}

impl RevealFocus {
    pub fn new(id: impl Into<ElementId>, child: impl IntoElement, scroll: ScrollHandle) -> Self {
        Self {
            id: id.into(),
            child: child.into_any_element(),
            scroll: RevealScroll::Handle(scroll),
            full_width: true,
        }
    }

    /// Same reveal behavior against a variable-height `list` viewport.
    pub fn in_list(id: impl Into<ElementId>, child: impl IntoElement, list: ListState) -> Self {
        Self {
            id: id.into(),
            child: child.into_any_element(),
            scroll: RevealScroll::List(list),
            full_width: true,
        }
    }

    pub fn inline(mut self) -> Self {
        self.full_width = false;
        self
    }
}

impl RenderOnce for RevealFocus {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| State {
            container: cx.focus_handle(),
            focused: None,
            seen: None,
        });
        let focus = state.read(cx).container.clone();
        div()
            .on_children_prepainted(move |bounds, window, cx| {
                let Some(bounds) = bounds.first().copied() else {
                    return;
                };
                let seen = (window.focused(cx), window.rem_size(), window.bounds().size);
                if state.read(cx).seen.as_ref() == Some(&seen) {
                    return;
                }
                let state = state.clone();
                let scroll = self.scroll.clone();
                // Focus ancestry is only reliable after this frame has committed:
                // prepaint moves reused subtrees out of the previous dispatch tree.
                window.defer(cx, move |window, cx| {
                    let focused = window
                        .focused(cx)
                        .filter(|_| state.read(cx).container.contains_focused(window, cx));
                    let geometry_changed = state
                        .read(cx)
                        .seen
                        .as_ref()
                        .is_none_or(|(_, rem, size)| (*rem, *size) != (seen.1, seen.2));
                    state.update(cx, |state, _| {
                        let reveal =
                            focused.is_some() && (state.focused != focused || geometry_changed);
                        state.focused = focused;
                        state.seen = Some(seen);
                        if !reveal {
                            return;
                        }
                        let viewport = scroll.viewport();
                        if viewport.size.height <= px(0.) {
                            return;
                        }
                        let delta = reveal_delta(
                            f32::from(bounds.top()),
                            f32::from(bounds.bottom()),
                            f32::from(viewport.top()) + 8.,
                            f32::from(viewport.bottom()) - 8.,
                        );
                        if delta != 0. {
                            scroll.scroll_by(px(delta));
                            window.refresh();
                        }
                    });
                });
            })
            .id(self.id)
            .track_focus(&focus)
            .tab_stop(false)
            .flex_shrink_0()
            .when(self.full_width, |view| view.w_full())
            .min_w_0()
            .child(self.child)
    }
}

/// A scroll area inside a scrolling page keeps the wheel while it has content
/// to scroll, so the page does not move with it. Its own scroll listener runs
/// first, then this one stops the event before it reaches the page.
pub fn contain_scroll(view: Stateful<Div>, handle: &ScrollHandle) -> Stateful<Div> {
    let observed = handle.clone();
    view.track_scroll(handle).on_scroll(move |_, _, cx| {
        if observed.max_offset().y > px(0.) {
            cx.stop_propagation();
        }
    })
}

fn reveal_delta(top: f32, bottom: f32, visible_top: f32, visible_bottom: f32) -> f32 {
    if top < visible_top || bottom - top > visible_bottom - visible_top {
        visible_top - top
    } else if bottom > visible_bottom {
        visible_bottom - bottom
    } else {
        0.
    }
}

#[cfg(test)]
mod tests {
    use super::{RevealFocus, reveal_delta};
    use gpui::{
        Context, FocusHandle, IntoElement, ListAlignment, ListOffset, ListState, Render,
        TestAppContext, Window, div, list, prelude::*, px, size,
    };

    struct ListRevealHarness {
        list: ListState,
        focus: FocusHandle,
        inline: bool,
    }

    impl Render for ListRevealHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let state = self.list.clone();
            let focus = self.focus.clone();
            let inline = self.inline;
            list(self.list.clone(), move |_, _, _| {
                let control = div()
                    .id("focused-control")
                    .debug_selector(|| "list-reveal-control".into())
                    .track_focus(&focus)
                    .w(px(40.))
                    .h(px(30.));
                let reveal = RevealFocus::in_list("list-reveal", control, state.clone());
                let reveal = if inline { reveal.inline() } else { reveal };
                div()
                    .relative()
                    .w_full()
                    .h(px(400.))
                    .child(div().absolute().top(px(150.)).w_full().child(reveal))
                    .into_any_element()
            })
            .w(px(100.))
            .h(px(100.))
        }
    }

    #[gpui::test]
    fn list_focus_reveal_scrolls_both_edges_in_content_coordinates(cx: &mut TestAppContext) {
        for inline in [false, true] {
            for initial_offset in [0., 170.] {
                let (view, cx) = cx.add_window_view(|_, cx| {
                    let state = ListState::new(1, ListAlignment::Top, px(0.));
                    state.scroll_to(ListOffset {
                        item_ix: 0,
                        offset_in_item: px(initial_offset),
                    });
                    ListRevealHarness {
                        list: state,
                        focus: cx.focus_handle(),
                        inline,
                    }
                });
                cx.run_until_parked();
                let before = cx.debug_bounds("list-reveal-control").unwrap();
                cx.update(|window, cx| {
                    let focus = view.read(cx).focus.clone();
                    window.focus(&focus, cx);
                    window.refresh();
                });
                cx.run_until_parked();
                let after = cx.debug_bounds("list-reveal-control").unwrap();
                let viewport = cx.update(|_, cx| view.read(cx).list.viewport_bounds());
                if initial_offset == 0. {
                    assert!(before.bottom() > viewport.bottom());
                    assert!(after.top() < before.top(), "bottom reveal must scroll down");
                } else {
                    assert!(before.top() < viewport.top());
                    assert!(after.top() > before.top(), "top reveal must scroll up");
                }
                assert!(after.top() >= viewport.top() + px(8.));
                assert!(after.bottom() <= viewport.bottom() - px(8.));
                assert_eq!(after.size, size(px(40.), px(30.)));
            }
        }
    }

    #[test]
    fn reveals_clipped_controls_and_the_start_of_oversized_rows() {
        assert_eq!(reveal_delta(100., 160., 80., 400.), 0.);
        assert_eq!(reveal_delta(30., 100., 80., 400.), 50.);
        assert_eq!(reveal_delta(380., 440., 80., 400.), -40.);
        assert_eq!(reveal_delta(300., 900., 80., 400.), -220.);
    }
}
