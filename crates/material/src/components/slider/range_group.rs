//! The node a range slider's thumbs are grouped under.
//!
//! Base's `Slider` root is always a `Role::Slider` with the end thumb's value and
//! Increment/Decrement (`vendor/gpui-base/src/slider.rs`, `Slider::render`). A
//! range's thumbs are its sliders here, each with its own value, bounds and
//! actions, so that root would make assistive technology find three sliders. A
//! range is rendered under this group instead: it names the range, carries its
//! orientation, ends a pointer interaction as Base's root does
//! (`SliderState::handle_release`), and is marked disabled with the slider.
//! GPUI has no disabled setter on a `Div`, and Base's wrapper for it
//! (`a11y_disabled.rs`, `A11yDisabledExt`) is crate-private, so
//! [`DisabledNode`] adds the flag.
use gpui::{prelude::*, *};
use gpui_base::slider::SliderState;

/// The group a range's thumbs are rendered in, holding `content`.
pub(super) fn range_group(
    state: &Entity<SliderState>,
    label: SharedString,
    orientation: Orientation,
    disabled: bool,
    content: impl IntoElement,
) -> impl IntoElement {
    let group = div()
        .id(("slider", state.entity_id()))
        .role(Role::Group)
        .aria_label(label)
        .aria_orientation(orientation)
        .size_full()
        .when(!disabled, |group| {
            let (up, up_out) = (state.clone(), state.clone());
            group
                .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                    up.update(cx, |state, cx| state.handle_release(cx));
                })
                .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                    up_out.update(cx, |state, cx| state.handle_release(cx));
                })
        })
        .child(content);
    DisabledNode {
        inner: group,
        disabled,
    }
    .test_support()
}

/// An element whose accessibility node is marked disabled while `disabled`.
pub(super) struct DisabledNode<E> {
    inner: E,
    disabled: bool,
}

impl<E: Element> IntoElement for DisabledNode<E> {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl<E: InteractiveElement> InteractiveElement for DisabledNode<E> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl<E: Element> Element for DisabledNode<E> {
    type RequestLayoutState = E::RequestLayoutState;
    type PrepaintState = E::PrepaintState;

    fn id(&self) -> Option<ElementId> {
        self.inner.id()
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        self.inner.source_location()
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.inner.request_layout(id, inspector, window, cx)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.inner
            .prepaint(id, inspector, bounds, layout, window, cx)
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.inner
            .paint(id, inspector, bounds, layout, prepaint, window, cx)
    }

    fn a11y_role(&self) -> Option<Role> {
        self.inner.a11y_role()
    }

    fn write_a11y_info(&self, node: &mut accesskit::Node) {
        self.inner.write_a11y_info(node);
        if self.disabled {
            node.set_disabled();
        }
    }

    fn a11y_synthetic_children(
        &mut self,
        prepaint: &mut Self::PrepaintState,
        builder: &mut A11ySubtreeBuilder,
    ) {
        self.inner.a11y_synthetic_children(prepaint, builder);
    }
}
