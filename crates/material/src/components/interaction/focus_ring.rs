//! Paint the M3 focus indicator around (or, for tightly connected items,
//! inside) a control without changing layout or hitboxes.
//!
//! The indicator is 3dp of secondary. Standalone controls use the M3 outer
//! offset (2dp outside the border box). Items packed edge to edge, such as
//! connected tiles, use the M3 inner offset (3dp inside) so a neighbour
//! painted later cannot cover it. Where the item's own fill can be primary,
//! as a selected connected button group option is, an inner secondary ring
//! cannot be seen; such a group keeps the outer offset and hands its rings to
//! a [`RingLayer`] that it paints after every item.
use crate::{color::ColorRole, theme::color};
use gpui::*;

/// M3 focus indicator thickness.
pub(crate) const RING_WIDTH: Pixels = px(3.);
/// M3 focus indicator outer offset: the gap between the border box and the ring.
pub(crate) const OUTER_OFFSET: Pixels = px(2.);
/// M3 focus indicator inner offset: how far inside the border box the ring's
/// outer edge sits.
pub(crate) const INNER_OFFSET: Pixels = px(3.);

enum Visibility {
    /// Visible while this handle has keyboard focus.
    Focus(FocusHandle),
    /// Visible while the caller's resolved state says focus is inside, e.g. a
    /// wrapper around a component whose own focus handle is private.
    Resolved(bool),
}

enum Radius {
    Rems(Rems),
    Corners(Corners<Pixels>),
}

pub(crate) struct ButtonFocusRing {
    child: AnyElement,
    visibility: Visibility,
    enabled: bool,
    radius: Radius,
    inward: bool,
    /// Where the ring is painted instead of in place, when set.
    layer: Option<RingLayer>,
    /// A specimen's forced focus: shown regardless of input modality.
    forced: bool,
    #[cfg(test)]
    observed: Option<std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>>,
    #[cfg(test)]
    observed_corners: Option<std::rc::Rc<std::cell::Cell<Option<Corners<Pixels>>>>>,
}
impl ButtonFocusRing {
    pub(crate) fn new(
        child: impl IntoElement,
        focus: FocusHandle,
        enabled: bool,
        radius: Rems,
    ) -> Self {
        Self {
            child: child.into_any_element(),
            visibility: Visibility::Focus(focus),
            enabled,
            radius: Radius::Rems(radius),
            inward: false,
            layer: None,
            forced: false,
            #[cfg(test)]
            observed: None,
            #[cfg(test)]
            observed_corners: None,
        }
    }

    /// The outer ring around a control whose border box has these corners.
    pub(crate) fn with_corners(
        child: impl IntoElement,
        focus: FocusHandle,
        enabled: bool,
        corners: Corners<Pixels>,
    ) -> Self {
        Self {
            radius: Radius::Corners(corners),
            ..Self::new(child, focus, enabled, Rems(0.))
        }
    }

    /// The outer ring, shown while `active` (focus resolved by the caller to
    /// be inside the child) and the last input came from the keyboard.
    pub(crate) fn resolved(child: impl IntoElement, active: bool, radius: Rems) -> Self {
        Self {
            child: child.into_any_element(),
            visibility: Visibility::Resolved(active),
            enabled: true,
            radius: Radius::Rems(radius),
            inward: false,
            layer: None,
            forced: false,
            #[cfg(test)]
            observed: None,
            #[cfg(test)]
            observed_corners: None,
        }
    }

    /// Always draw the ring (while enabled), for a static focused specimen.
    pub(crate) fn forced(mut self, forced: bool) -> Self {
        self.forced = forced;
        self
    }

    /// Draw the ring at the M3 inner offset instead of the outer one.
    pub(crate) fn inward(mut self) -> Self {
        self.inward = true;
        self
    }

    /// Hand the ring to `layer`, which paints it after its later siblings,
    /// instead of painting it in place, where they would cover an outer ring.
    pub(crate) fn painted_by(mut self, layer: &RingLayer) -> Self {
        self.layer = Some(layer.clone());
        self
    }

    #[cfg(test)]
    pub(crate) fn observe(
        mut self,
        observed: std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>,
    ) -> Self {
        self.observed = Some(observed);
        self
    }
}

/// Ring bounds and corner radii for a border box with `corners`.
pub(crate) fn ring_geometry(
    bounds: Bounds<Pixels>,
    corners: Corners<Pixels>,
    inward: bool,
) -> (Bounds<Pixels>, Corners<Pixels>) {
    if inward {
        let ring = bounds.inset(INNER_OFFSET);
        let shrink = |radius: Pixels| (radius - INNER_OFFSET).max(px(0.));
        let corners = Corners {
            top_left: shrink(corners.top_left),
            top_right: shrink(corners.top_right),
            bottom_right: shrink(corners.bottom_right),
            bottom_left: shrink(corners.bottom_left),
        };
        (ring, corners.clamp_radii_for_quad_size(ring.size))
    } else {
        let grow = OUTER_OFFSET + RING_WIDTH;
        let ring = bounds.dilate(grow);
        let corners = Corners {
            top_left: corners.top_left + grow,
            top_right: corners.top_right + grow,
            bottom_right: corners.bottom_right + grow,
            bottom_left: corners.bottom_left + grow,
        };
        (ring, corners.clamp_radii_for_quad_size(ring.size))
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
        let focused = match &self.visibility {
            Visibility::Focus(focus) => focus.is_focused(window),
            Visibility::Resolved(active) => *active,
        };
        let visible =
            self.enabled && (self.forced || (focused && window.last_input_was_keyboard()));
        let corners = match self.radius {
            Radius::Rems(radius) => Corners::all(radius.to_pixels(window.rem_size())),
            Radius::Corners(corners) => corners,
        };
        #[cfg(test)]
        if let Some(observed) = &self.observed_corners {
            observed.set(Some(corners));
        }
        let (ring_bounds, ring_corners) = ring_geometry(bounds, corners, self.inward);
        #[cfg(test)]
        if let Some(observed) = &self.observed {
            observed.set(visible.then_some(ring_bounds));
        }
        #[cfg(test)]
        PAINTED_RING_CHILDREN.with(|count| count.set(count.get() + 1));
        if visible {
            match &self.layer {
                Some(layer) => layer.ring.set(Some((ring_bounds, ring_corners))),
                None => paint_ring(ring_bounds, ring_corners, window),
            }
        }
    }
}

fn paint_ring(bounds: Bounds<Pixels>, corners: Corners<Pixels>, window: &mut Window) {
    window.paint_quad(
        outline(bounds, color(ColorRole::SECONDARY), BorderStyle::Solid)
            .border_widths(RING_WIDTH)
            .corner_radii(corners),
    );
}

#[cfg(test)]
thread_local! {
    /// How many rings' children have painted, for checking a layer's order.
    pub(crate) static PAINTED_RING_CHILDREN: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// The last ring a layer painted, and how many rings' children had
    /// painted before it.
    pub(crate) static LAYERED_RING: std::cell::Cell<Option<(Bounds<Pixels>, usize)>> =
        const { std::cell::Cell::new(None) };
}

/// A ring handed to a [`RingLayer`]: its bounds and corners, until painted.
type PendingRing = std::rc::Rc<std::cell::Cell<Option<(Bounds<Pixels>, Corners<Pixels>)>>>;

/// Paints, above a group of tightly packed items, the focus ring that one of
/// them hands it with [`ButtonFocusRing::painted_by`]. Make one per render and
/// add [`RingLayer::element`] to the group after every item: it is absolutely
/// positioned and takes no space. The ring stays in the group's paint order and
/// content mask, so overlays opened above the group still cover it.
#[derive(Clone, Default)]
pub(crate) struct RingLayer {
    ring: PendingRing,
}

impl RingLayer {
    pub(crate) fn element(&self) -> RingLayerElement {
        RingLayerElement {
            ring: self.ring.clone(),
        }
    }
}

pub(crate) struct RingLayerElement {
    ring: PendingRing,
}

impl IntoElement for RingLayerElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for RingLayerElement {
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
        let style = Style {
            position: Position::Absolute,
            size: size(px(0.).into(), px(0.).into()),
            ..Style::default()
        };
        (window.request_layout(style, [], cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut Window,
        _: &mut App,
    ) {
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        _: &mut App,
    ) {
        // Taken, so a ring from an earlier frame is never painted again.
        let ring = self.ring.take();
        #[cfg(test)]
        LAYERED_RING.with(|layered| {
            layered.set(
                ring.map(|(bounds, _)| (bounds, PAINTED_RING_CHILDREN.with(|count| count.get()))),
            )
        });
        if let Some((bounds, corners)) = ring {
            paint_ring(bounds, corners, window);
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
        forced: bool,
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
                ButtonFocusRing::new(button, self.focus.clone(), !self.disabled, rem(20.))
                    .forced(self.forced);
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
                forced: false,
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

    struct Asymmetric {
        focus: FocusHandle,
        observed: Rc<Cell<Option<gpui::Corners<Pixels>>>>,
    }
    impl Render for Asymmetric {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let mut ring = ButtonFocusRing::with_corners(
                div().size(px(40.)),
                self.focus.clone(),
                true,
                gpui::Corners {
                    top_left: px(1.),
                    top_right: px(2.),
                    bottom_right: px(3.),
                    bottom_left: px(4.),
                },
            )
            .forced(true);
            ring.observed_corners = Some(self.observed.clone());
            ring
        }
    }

    #[gpui::test]
    fn the_ring_keeps_each_corner_of_the_control(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let observed = Rc::new(Cell::new(None));
        let (_, cx) = cx.add_window_view({
            let observed = observed.clone();
            move |_, cx| Asymmetric {
                focus: cx.focus_handle(),
                observed,
            }
        });
        draw(cx);
        assert_eq!(
            observed.get(),
            Some(gpui::Corners {
                top_left: px(1.),
                top_right: px(2.),
                bottom_right: px(3.),
                bottom_left: px(4.),
            }),
            "each corner stays where the control has it"
        );
    }

    #[gpui::test]
    fn forced_ring_paints_without_focus_but_not_while_disabled(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let observed = Rc::new(Cell::new(None));
        let (view, cx) = cx.add_window_view({
            let observed = observed.clone();
            move |_, cx| Harness {
                focus: cx.focus_handle(),
                border: 0.,
                disabled: false,
                forced: true,
                observed,
            }
        });
        draw(cx);
        let root = cx.debug_bounds("focus-ring-button").unwrap();
        assert_eq!(observed.get(), Some(root.dilate(px(5.))));
        view.update(cx, |this, cx| {
            this.disabled = true;
            cx.notify();
        });
        draw(cx);
        assert!(observed.get().is_none());
    }

    #[test]
    fn outer_and_inner_offsets_follow_m3() {
        use super::ring_geometry;
        use gpui::{Corners, bounds, point, size};
        let b = bounds(point(px(10.), px(10.)), size(px(100.), px(80.)));
        let corners = Corners {
            top_left: px(24.),
            top_right: px(24.),
            bottom_right: px(6.),
            bottom_left: px(2.),
        };
        let (outer, outer_corners) = ring_geometry(b, corners, false);
        assert_eq!(outer, b.dilate(px(5.)), "2dp offset plus 3dp ring");
        assert_eq!(outer_corners.top_left, px(29.));
        assert_eq!(outer_corners.bottom_right, px(11.));
        let (inner, inner_corners) = ring_geometry(b, corners, true);
        assert_eq!(inner, b.inset(px(3.)), "the ring's outer edge 3dp inside");
        assert_eq!(inner_corners.top_left, px(21.));
        assert_eq!(inner_corners.bottom_right, px(3.));
        assert_eq!(inner_corners.bottom_left, px(0.), "radii never go negative");
    }
}
