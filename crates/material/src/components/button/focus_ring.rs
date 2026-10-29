//! Paint outside the actual button border box without changing layout or hitboxes.
use crate::{color::ColorRole, theme::color};
use gpui::*;

pub(super) struct ButtonFocusRing {
    child: AnyElement,
    focus: FocusHandle,
    enabled: bool,
    radius: Rems,
    #[cfg(test)]
    observed: Option<std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>>,
}
impl ButtonFocusRing {
    pub(super) fn new(
        child: impl IntoElement,
        focus: FocusHandle,
        enabled: bool,
        radius: Rems,
    ) -> Self {
        Self {
            child: child.into_any_element(),
            focus,
            enabled,
            radius,
            #[cfg(test)]
            observed: None,
        }
    }
}
impl IntoElement for ButtonFocusRing {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for ButtonFocusRing {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.prepaint(window, cx);
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
        let visible =
            self.enabled && self.focus.is_focused(window) && window.last_input_was_keyboard();
        let ring_bounds = bounds.dilate(px(5.));
        #[cfg(test)]
        if let Some(observed) = &self.observed {
            observed.set(visible.then_some(ring_bounds));
        }
        if visible {
            let corners = Corners::all(self.radius.to_pixels(window.rem_size()) + px(5.))
                .clamp_radii_for_quad_size(ring_bounds.size);
            window.paint_quad(
                outline(ring_bounds, color(ColorRole::SECONDARY), BorderStyle::Solid)
                    .border_widths(px(3.))
                    .corner_radii(corners),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ButtonFocusRing;
    use crate::tokens::typography::rem;
    use gpui::{
        Bounds, Context, FocusHandle, IntoElement, Modifiers, Pixels, Render, TestAppContext,
        VisualTestContext, Window, div, prelude::*, px,
    };
    use std::{cell::Cell, rc::Rc};
    struct Harness {
        focus: FocusHandle,
        border: f32,
        disabled: bool,
        observed: Rc<Cell<Option<Bounds<Pixels>>>>,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            let button = gpui_base::Button::new("focus-ring-button")
                .track_focus(&self.focus)
                .disabled(self.disabled)
                .w(px(120.))
                .h(px(40.))
                .border(px(self.border))
                .debug_selector(|| "focus-ring-button".into())
                .child("Label");
            let mut ring =
                ButtonFocusRing::new(button, self.focus.clone(), !self.disabled, rem(20.));
            ring.observed = Some(self.observed.clone());
            div().p(px(20.)).flex().child(ring)
        }
    }
    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }
    #[gpui::test]
    fn outline_uses_border_box_and_keyboard_modality_without_affecting_layout(
        cx: &mut TestAppContext,
    ) {
        cx.update(gpui_base::init);
        let observed = Rc::new(Cell::new(None));
        let (view, cx) = cx.add_window_view({
            let observed = observed.clone();
            move |_, cx| Harness {
                focus: cx.focus_handle(),
                border: 1.,
                disabled: false,
                observed,
            }
        });
        draw(cx);
        let original = cx.debug_bounds("focus-ring-button").unwrap();
        assert_eq!(original.size, gpui::size(px(120.), px(40.)));
        assert!(observed.get().is_none());
        cx.simulate_click(original.center(), Modifiers::default());
        draw(cx);
        assert!(
            observed.get().is_none(),
            "pointer focus must not show the ring"
        );
        let keystroke =
            cx.update(|_, cx| gpui::Keystroke::parse_with_os("a", cx.platform_info().os).unwrap());
        cx.simulate_event(gpui::KeyDownEvent {
            keystroke,
            is_held: false,
            prefer_character_input: false,
        });
        draw(cx);
        for border in [1., 2., 3.] {
            view.update(cx, |this, cx| {
                this.border = border;
                cx.notify();
            });
            draw(cx);
            let root = cx.debug_bounds("focus-ring-button").unwrap();
            assert_eq!(
                root, original,
                "focus painting must not move or resize the control"
            );
            assert_eq!(
                observed.get(),
                Some(root.dilate(px(5.))),
                "outline must use the border box even with {border}px border"
            );
        }
        view.update(cx, |this, cx| {
            this.disabled = true;
            cx.notify();
        });
        draw(cx);
        assert!(
            observed.get().is_none(),
            "disabled retained focus must not draw an outline"
        );
        view.update(cx, |this, cx| {
            this.disabled = false;
            cx.notify();
        });
        draw(cx);
        assert!(observed.get().is_some());
        cx.simulate_click(original.center(), Modifiers::default());
        draw(cx);
        assert!(observed.get().is_none());
    }
}
