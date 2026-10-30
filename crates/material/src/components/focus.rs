//! Focus presentation for composed controls, and scrolling that reveals
//! newly focused rows without pinning them during manual scrolling.
use super::InteractionPreview;
use super::button::ButtonFocusRing;
use crate::tokens::{shape::RADIUS_FULL, typography::rem};
use gpui::{prelude::*, *};

/// Draw the M3 keyboard focus indicator (the button's 3dp secondary ring, 2dp
/// outside) around a component whose focus handle is internal. The wrapper
/// observes descendants without adding a keyboard stop of its own, and shows
/// the ring only for keyboard focus.
#[derive(gpui::IntoElement)]
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
        let wrapper = div()
            .id(self.id)
            .track_focus(&focus)
            .tab_stop(false)
            .flex_shrink_0()
            .rounded_full()
            .child(self.child);
        ButtonFocusRing::resolved(wrapper, active, rem(RADIUS_FULL.as_f32()))
    }
}

/// How a [`RingedButton`] draws its keyboard focus ring.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum RingPlacement {
    /// The M3 outer offset, for standalone controls.
    Outer(Corners<Pixels>),
    /// The M3 inner offset, for items packed edge to edge with neighbours.
    Inner(Corners<Pixels>),
}

/// A GPUI Base button that draws the shared M3 keyboard focus ring. It
/// forwards the builder methods callers chain on a button, and resolves its
/// focus handle when it renders.
#[derive(gpui::IntoElement)]
pub struct RingedButton {
    id: ElementId,
    button: gpui_base::Button,
    focus: Option<FocusHandle>,
    placement: RingPlacement,
    disabled: bool,
    /// Hover and press state-layer backgrounds, installed as live styles or
    /// painted directly for a forced preview.
    layers: Option<(Rgba, Rgba)>,
    ripple: Option<Rgba>,
    preview: Option<InteractionPreview>,
}

impl RingedButton {
    pub(crate) fn new(id: impl Into<ElementId>, placement: RingPlacement) -> Self {
        let id = id.into();
        Self {
            button: gpui_base::Button::new(id.clone()),
            id,
            focus: None,
            placement,
            disabled: false,
            layers: None,
            ripple: None,
            preview: None,
        }
    }

    /// Restyle the inner button.
    pub(crate) fn map(mut self, f: impl FnOnce(gpui_base::Button) -> gpui_base::Button) -> Self {
        self.button = f(self.button);
        self
    }

    /// The container under its hover and press state layers. Live, they are
    /// the pointer hover and active styles; a preview paints the one it forces.
    pub(crate) fn state_layers(mut self, hover: Rgba, press: Rgba) -> Self {
        self.layers = Some((hover, press));
        self
    }

    /// Press feedback: the shared ripple, in this content color, clipped to the
    /// button's corners.
    pub(crate) fn ripple(mut self, ink: Rgba) -> Self {
        self.ripple = Some(ink);
        self
    }

    /// Render a static specimen in one forced interaction state: the hover or
    /// press layer, the focus ring, or the disabled control. The specimen
    /// leaves the tab order and ignores live hover and press. See
    /// [`InteractionPreview`].
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        if state.is_disabled() {
            self = self.disabled(true);
        }
        self
    }

    /// Use a caller-owned focus handle, e.g. for roving focus in a group.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    /// The activation handler for pointer, Enter and Space input.
    pub fn on_click(self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.map(|button| button.on_click(handler))
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self.map(|button| button.disabled(disabled))
    }

    pub fn accessibility_label(self, label: impl Into<SharedString>) -> Self {
        self.map(|button| button.accessibility_label(label))
    }

    pub fn tab_stop(self, tab_stop: bool) -> Self {
        self.map(|button| button.tab_stop(tab_stop))
    }
}

impl Styled for RingedButton {
    fn style(&mut self) -> &mut StyleRefinement {
        self.button.style()
    }
}

impl ParentElement for RingedButton {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.button.extend(elements)
    }
}

impl InteractiveElement for RingedButton {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.button.interactivity()
    }
}

impl StatefulInteractiveElement for RingedButton {}

impl RenderOnce for RingedButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus = self.focus.unwrap_or_else(|| {
            window
                .use_keyed_state(
                    ElementId::NamedChild(self.id.clone().into(), "focus".into()),
                    cx,
                    |_, cx| cx.focus_handle(),
                )
                .read(cx)
                .clone()
        });
        #[cfg(test)]
        let key = format!("{:?}", self.id);
        let enabled = !self.disabled;
        // A disabled button is inert and a specimen is not operated: neither
        // shows the pointer cursor (nor, below, live state layers).
        let pointer = shows_pointer(enabled, self.preview);
        let mut button = self.button.track_focus(&focus).map(|button| {
            if pointer {
                button.cursor_pointer()
            } else {
                button.cursor_default()
            }
        });
        if let Some((hover, press)) = self.layers.filter(|_| enabled) {
            button = match self.preview {
                None => button
                    .hover(move |style| style.bg(hover))
                    .active(move |style| style.bg(press)),
                Some(InteractionPreview::Hovered) => button.bg(hover),
                Some(InteractionPreview::Pressed) => button.bg(press),
                Some(_) => button,
            };
        }
        if self.preview.is_some() {
            button = button.tab_stop(false);
        }
        let forced = self.preview == Some(InteractionPreview::Focused);
        let corners = match self.placement {
            RingPlacement::Outer(corners) | RingPlacement::Inner(corners) => corners,
        };
        let content = match self.ripple {
            Some(ink) => super::Ripple::new(
                ElementId::NamedChild(self.id.clone().into(), "ripple".into()),
                button,
            )
            .ink(ink)
            .corners(corners)
            .disabled(!enabled)
            .specimen(self.preview.is_some())
            .into_any_element(),
            None => button.into_any_element(),
        };
        let ring = ButtonFocusRing::with_corners(content, focus, enabled, corners);
        let ring = match self.placement {
            RingPlacement::Outer(_) => ring,
            RingPlacement::Inner(_) => ring.inward(),
        }
        .forced(forced);
        #[cfg(test)]
        let ring = ring.observe(test_rings::cell(key));
        ring
    }
}

/// Whether a [`RingedButton`] shows the pointer cursor and live hover and
/// press layers: only when it is enabled and not a forced specimen.
pub(crate) fn shows_pointer(enabled: bool, preview: Option<InteractionPreview>) -> bool {
    enabled && preview.is_none()
}

/// Test hook: the focus ring each [`RingedButton`] last painted, by the
/// debug form of its element id.
#[cfg(test)]
pub(crate) mod test_rings {
    use gpui::{Bounds, Pixels};
    use std::{cell::Cell, cell::RefCell, collections::HashMap, rc::Rc};
    type Ring = Rc<Cell<Option<Bounds<Pixels>>>>;
    thread_local! {
        static RINGS: RefCell<HashMap<String, Ring>> = RefCell::new(HashMap::new());
    }
    pub(crate) fn cell(key: String) -> Ring {
        RINGS.with(|rings| rings.borrow_mut().entry(key).or_default().clone())
    }
    /// The ring painted for a button whose id's debug form contains `part`.
    pub(crate) fn painted(part: &str) -> Option<Bounds<Pixels>> {
        RINGS.with(|rings| {
            rings
                .borrow()
                .iter()
                .filter(|(key, _)| key.contains(part))
                .find_map(|(_, ring)| ring.get())
        })
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
#[derive(gpui::IntoElement)]
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
