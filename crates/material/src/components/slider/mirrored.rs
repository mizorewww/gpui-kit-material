//! Pointer input of a horizontal slider laid out right to left.
//!
//! Layout mirrors the track and thumbs, but Base measures a pointer from the
//! left edge of the bounds its `SliderIndicator` records
//! (`SliderState::update_value_by_position`, and the range's nearer-thumb test
//! in `SliderTrack` and `SliderThumb`), so a right-to-left slider would take
//! the mirror image of every press. Base's slider has no direction of its own.
//! Here Material turns Base's track and thumb listeners off and takes the
//! pointer itself: it reflects each position across the bounds' middle, so
//! Base's left-to-right math measures from the right edge, and leaves the value,
//! its steps, the other thumb's limit and the events to Base.
use gpui::{prelude::*, *};
use gpui_base::slider::{SliderState, SliderThumb, SliderTrack};

/// What a press on a right-to-left slider moves until it is released.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum MirroredDrag {
    /// The mouse, moving a range's start thumb or the end thumb.
    Mouse { start: bool },
    /// A touch drag the track claimed, told apart by where it started.
    Touch {
        start_position: Point<Pixels>,
        start: bool,
    },
}

/// `position` reflected across the middle of `bounds`: Base measures it from
/// the left edge, which then stands where the right edge is.
pub(super) fn mirror(position: Point<Pixels>, bounds: Bounds<Pixels>) -> Point<Pixels> {
    point(bounds.left() + bounds.right() - position.x, position.y)
}

/// Whether a press at `position`, already mirrored, is nearer a range's start
/// thumb than its end thumb, as Base decides it for a left-to-right one.
fn nearer_start(state: &SliderState, position: Point<Pixels>) -> bool {
    if !state.value().is_range() {
        return false;
    }
    let bounds = state.bounds();
    let percentage = state.percentage();
    let center = ((percentage.end - percentage.start) / 2. + percentage.start)
        * f32::from(bounds.size.width);
    f32::from(position.x - bounds.left()) < center
}

/// Moves the thumb `start` names, or with `None` the one nearer the press, to
/// `position` in window coordinates. Returns the thumb it moved.
fn press(
    state: &Entity<SliderState>,
    position: Point<Pixels>,
    start: Option<bool>,
    window: &mut Window,
    cx: &mut App,
) -> bool {
    state.update(cx, |state, cx| {
        let position = mirror(position, state.bounds());
        let start = start.unwrap_or_else(|| nearer_start(state, position));
        state.update_value_by_position(Axis::Horizontal, position, start, window, cx);
        start
    })
}

/// Sets what is dragged, telling the view that shows it when that changed.
fn set(drag: &Entity<Option<MirroredDrag>>, value: Option<MirroredDrag>, cx: &mut App) {
    drag.update(cx, |drag, cx| {
        if *drag != value {
            *drag = value;
            cx.notify();
        }
    });
}

/// The right-to-left pointer input of one slider.
#[derive(Clone)]
pub(super) struct MirroredPointer {
    pub state: Entity<SliderState>,
    pub drag: Entity<Option<MirroredDrag>>,
}

impl MirroredPointer {
    /// The track, with Base's listeners off and these on. A press sets the
    /// nearer thumb; on a single slider it also drags it, as Base's track does.
    pub fn track(&self, track: SliderTrack, is_range: bool) -> SliderTrack {
        let (state, drag) = (self.state.clone(), self.drag.clone());
        track
            .disabled(true)
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                let start = press(&state, event.position, None, window, cx);
                if !is_range {
                    set(&drag, Some(MirroredDrag::Mouse { start }), cx);
                }
            })
            .child(self.touch_layer())
    }

    /// A thumb that drags itself, ahead of Base's listeners, which never hear
    /// the press: Base's thumb also stops it there.
    pub fn thumb(&self, thumb: SliderThumb, start: bool) -> SliderThumb {
        let drag = self.drag.clone();
        thumb.capture_any_mouse_down(move |event, _, cx| {
            if event.button == MouseButton::Left {
                cx.stop_propagation();
                set(&drag, Some(MirroredDrag::Mouse { start }), cx);
            }
        })
    }

    /// Follows a mouse drag across the window while one is held, as Base's
    /// drag does, and lets go when the button is no longer down.
    pub fn mouse_drag(&self, cx: &App) -> Option<impl IntoElement + use<>> {
        let Some(MirroredDrag::Mouse { start }) = *self.drag.read(cx) else {
            return None;
        };
        let (state, drag) = (self.state.clone(), self.drag.clone());
        Some(
            canvas(
                |_, _, _| {},
                move |_, _, window, _| {
                    let (state, drag) = (state.clone(), drag.clone());
                    window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                        if phase != DispatchPhase::Capture
                            || *drag.read(cx) != Some(MirroredDrag::Mouse { start })
                        {
                            return;
                        }
                        if event.pressed_button == Some(MouseButton::Left) {
                            press(&state, event.position, Some(start), window, cx);
                        } else {
                            set(&drag, None, cx);
                            state.update(cx, |state, cx| state.handle_release(cx));
                        }
                    });
                },
            )
            .absolute()
            .size_0(),
        )
    }

    /// Ends a mouse drag at its release.
    pub fn release(&self, cx: &mut App) {
        if matches!(*self.drag.read(cx), Some(MirroredDrag::Mouse { .. })) {
            set(&self.drag, None, cx);
        }
    }

    /// Claims a finger drag that starts on the track, as Base's track does
    /// (`SliderTrack`'s touch layer, off with its other listeners).
    fn touch_layer(&self) -> impl IntoElement {
        let (state, drag) = (self.state.clone(), self.drag.clone());
        canvas(
            |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
            move |_, hitbox, window, _| {
                let (state, drag) = (state.clone(), drag.clone());
                window.on_mouse_event(move |event: &TouchDragEvent, phase, window, cx| {
                    if !phase.bubble() {
                        return;
                    }
                    if event.phase == TouchPhase::Started {
                        // A drag whose end never arrived ends when another begins.
                        if matches!(*drag.read(cx), Some(MirroredDrag::Touch { .. })) {
                            set(&drag, None, cx);
                            state.update(cx, |state, cx| state.handle_release(cx));
                        }
                        // The hitbox, not the bounds, so a layer covering the
                        // track keeps its own drags.
                        if window.default_prevented() || !hitbox.is_hovered(window) {
                            return;
                        }
                        window.prevent_default();
                        let state_now = state.read(cx);
                        let start = nearer_start(
                            state_now,
                            mirror(event.start_position, state_now.bounds()),
                        );
                        let claim = MirroredDrag::Touch {
                            start_position: event.start_position,
                            start,
                        };
                        set(&drag, Some(claim), cx);
                    }
                    let Some(MirroredDrag::Touch {
                        start_position,
                        start,
                    }) = *drag.read(cx)
                    else {
                        return;
                    };
                    if start_position != event.start_position {
                        return;
                    }
                    cx.stop_propagation();
                    match event.phase {
                        TouchPhase::Started | TouchPhase::Moved => {
                            press(&state, event.position, Some(start), window, cx);
                        }
                        TouchPhase::Ended | TouchPhase::Cancelled => {
                            set(&drag, None, cx);
                            state.update(cx, |state, cx| state.handle_release(cx));
                        }
                    }
                });
            },
        )
        .absolute()
        .size_full()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn a_mirrored_position_measures_from_the_right_edge() {
        let bounds = Bounds::new(point(px(100.), px(0.)), size(px(200.), px(10.)));
        // 30px in from the right edge is 30px in from the left once mirrored.
        assert_eq!(
            mirror(point(px(270.), px(5.)), bounds),
            point(px(130.), px(5.))
        );
        assert_eq!(mirror(point(px(100.), px(5.)), bounds).x, px(300.));
        assert_eq!(mirror(point(px(200.), px(5.)), bounds).x, px(200.));
    }
}
