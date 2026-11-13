//! Which scroll container a wheel gesture scrolls, when containers nest.
//!
//! Every scroll container under the pointer hears a wheel event, innermost
//! first, and each used to move by it: a list inside a scrolling page moved
//! together with the page. Now the innermost container that the wheel moves
//! takes the event, and the scroll containers around it leave it alone. The
//! event still propagates: other wheel listeners, a container's own
//! `on_scroll` among them, hear it as before.
//!
//! It keeps the gesture too, as a platform scroll view does. Once a container
//! took a wheel, the gesture's later events are its own while it lasts, even
//! when it reaches its end: a list scrolled to its bottom does not hand the
//! rest of a swipe, or its momentum, to the page around it, and a page
//! scrolling under a still pointer is not taken over by a list that comes
//! under it. A gesture lasts at the pointer position it started at while a
//! trackpad's fingers are down, and after they lift, or for a mouse wheel,
//! while its events keep coming less than [`LATCH_IDLE`] apart, as momentum
//! and a wheel's clicks do; a trackpad touch that starts ends it at once. When
//! the container that keeps it is no longer under the pointer, the gesture
//! ends.
//!
//! A container that scrolls on one axis takes only a wheel that goes mainly
//! along it. One going mainly across it, such as a vertical wheel over a strip
//! that scrolls only horizontally, moves it as before, onto its one axis or by
//! its small part along it, without taking the event: what is around it
//! scrolls by the same wheel.
//!
//! The containers under the pointer note themselves in the capture phase,
//! before any of them hears the event in the bubble phase, so that each knows
//! whether the container keeping the gesture is among them.

use scheduler::Instant;
use std::time::Duration;

use crate::{App, IsZero as _, Pixels, Point, ScrollDelta, ScrollEvent, TouchPhase, Window};

/// How long after a wheel event a gesture lasts without another. A wheel's
/// clicks, a swipe's events and its momentum come far closer together.
pub(crate) const LATCH_IDLE: Duration = Duration::from_millis(250);

/// A scroll container, by the address of the state its offset lives in.
pub(crate) type Container = usize;

/// The wheel gesture under way, and the containers under the pointer.
#[derive(Default)]
pub(crate) struct WheelLatch {
    latched: Option<Latched>,
    /// The containers under the pointer for the event being dispatched,
    /// outermost first, as the capture phase met them.
    under_pointer: Vec<Container>,
    /// The container that took the event being dispatched.
    taken_by: Option<Container>,
    /// Whether a trackpad's fingers are down and have moved.
    fingers_down: bool,
}

struct Latched {
    container: Container,
    at: Instant,
    position: Point<Pixels>,
}

/// Who a wheel event belongs to, as a container hears it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Owner {
    /// The gesture is this container's: it moves if it can and nothing
    /// around it does.
    This,
    /// A container inside this one took the event.
    Taken,
    /// The gesture is another container's, around this one.
    Other,
    /// No gesture is under way: the innermost container the wheel moves
    /// takes it.
    Nobody,
}

/// Ends the gesture if `event` does not continue it, and forgets which
/// containers were under the pointer. Called before `event` is dispatched.
pub(crate) fn begin_dispatch(window: &mut Window, cx: &App, event: &ScrollEvent) {
    let now = cx.background_executor().now();
    let latch = &mut window.fast_layers.wheel_latch;
    latch.under_pointer.clear();
    latch.taken_by = None;
    // As `hover_freeze`'s `Fingers`: a touch that lifts without moving sends
    // a `Moved` of nothing and never `Ended`, and momentum is `Moved` too.
    let (precise, moved) = match event.delta {
        ScrollDelta::Pixels(delta) => (true, !delta.x.is_zero() || !delta.y.is_zero()),
        ScrollDelta::Lines(_) => (false, false),
    };
    let fingers_were_down = latch.fingers_down;
    latch.fingers_down = match event.touch_phase {
        TouchPhase::Started => precise,
        TouchPhase::Moved => precise && moved && fingers_were_down,
        TouchPhase::Ended | TouchPhase::Cancelled => false,
    };
    if latch.latched.as_ref().is_some_and(|latched| {
        event.touch_phase == TouchPhase::Started
            || latched.position != event.position
            || (!fingers_were_down && now - latched.at >= LATCH_IDLE)
    }) {
        latch.latched = None;
    }
}

/// Notes `container` as under the pointer, in the capture phase.
pub(crate) fn note_under_pointer(window: &mut Window, container: Container) {
    window.fast_layers.wheel_latch.under_pointer.push(container);
}

/// Who the wheel event being dispatched belongs to, as `container` hears it
/// in the bubble phase.
pub(crate) fn owner(window: &Window, container: Container) -> Owner {
    let latch = &window.fast_layers.wheel_latch;
    if latch.taken_by.is_some_and(|taken_by| taken_by != container) {
        return Owner::Taken;
    }
    match &latch.latched {
        Some(latched) if latched.container == container => Owner::This,
        Some(latched) if latch.under_pointer.contains(&latched.container) => Owner::Other,
        _ => Owner::Nobody,
    }
}

/// Gives `event`, the gesture it belongs to and the events that continue it,
/// to `container`.
pub(crate) fn hold(window: &mut Window, cx: &App, container: Container, event: &ScrollEvent) {
    window.fast_layers.wheel_latch.taken_by = Some(container);
    window.fast_layers.wheel_latch.latched = Some(Latched {
        container,
        at: cx.background_executor().now(),
        position: event.position,
    });
}
