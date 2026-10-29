//! Roving keyboard focus for single-selection groups (radio groups, choice
//! tiles, connected button groups): one tab stop at the selection, arrow
//! keys, Home and End move focus among the available items and select them.
//!
//! A group owns one focus handle per position ([`roving_handles`]), builds a
//! [`RovingFocus`] each frame from its items' availability and selection, and
//! installs it on its container with a key context bound by
//! [`bind_roving_keys`].
use gpui::*;
use std::rc::Rc;

pub(crate) use super::button_group::{FirstChoice, LastChoice, NextChoice, PreviousChoice};

/// One arrow-key, Home or End movement within a group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RovingStep {
    Next,
    Previous,
    First,
    Last,
}

/// The item `step` lands on from `current`, among `enabled` items. Arrows
/// wrap; with no current item, Next starts at the first available item and
/// Previous at the last.
pub(crate) fn roving_step(
    enabled: &[bool],
    current: Option<usize>,
    step: RovingStep,
) -> Option<usize> {
    let mut available = (0..enabled.len()).filter(|&i| enabled[i]);
    let first = available.next()?;
    let last = (0..enabled.len()).rev().find(|&i| enabled[i])?;
    let after = |current: usize| (current + 1..enabled.len()).find(|&i| enabled[i]);
    let before = |current: usize| (0..current).rev().find(|&i| enabled[i]);
    Some(match (step, current) {
        (RovingStep::First, _) | (RovingStep::Next, None) => first,
        (RovingStep::Last, _) | (RovingStep::Previous, None) => last,
        (RovingStep::Next, Some(current)) => after(current).unwrap_or(first),
        (RovingStep::Previous, Some(current)) => before(current).unwrap_or(last),
    })
}

/// The single tab stop of a group: the selected item when it is available,
/// otherwise the first available one.
pub(crate) fn roving_entry(enabled: &[bool], selected: Option<usize>) -> Option<usize> {
    selected
        .filter(|&i| enabled.get(i).copied().unwrap_or(false))
        .or_else(|| enabled.iter().position(|&e| e))
}

/// Bind the arrow keys, Home and End to the roving actions in `context`.
pub(crate) fn bind_roving_keys(cx: &mut App, context: &'static str) {
    cx.bind_keys([
        KeyBinding::new("right", NextChoice, Some(context)),
        KeyBinding::new("down", NextChoice, Some(context)),
        KeyBinding::new("left", PreviousChoice, Some(context)),
        KeyBinding::new("up", PreviousChoice, Some(context)),
        KeyBinding::new("home", FirstChoice, Some(context)),
        KeyBinding::new("end", LastChoice, Some(context)),
    ]);
}

/// One focus handle per position of the group keyed by `id`, kept across
/// frames. Extra handles go with removed items.
pub(crate) fn roving_handles(
    id: impl Into<ElementId>,
    count: usize,
    window: &mut Window,
    cx: &mut App,
) -> Rc<[FocusHandle]> {
    let handles = window.use_keyed_state(id, cx, |_, _| Vec::<FocusHandle>::new());
    handles.update(cx, |handles, cx| {
        if handles.len() != count {
            handles.truncate(count);
            while handles.len() < count {
                handles.push(cx.focus_handle());
            }
        }
        handles.as_slice().into()
    })
}

type Select = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// A group's roving focus for one frame.
#[derive(Clone)]
pub(crate) struct RovingFocus {
    focus: Rc<[FocusHandle]>,
    enabled: Rc<[bool]>,
    selected: Option<usize>,
    on_select: Option<Select>,
}

impl RovingFocus {
    /// `focus` and `enabled` have one entry per item; `on_select` is called
    /// with the index of an item that keyboard movement newly selects.
    pub(crate) fn new(
        focus: Rc<[FocusHandle]>,
        enabled: Rc<[bool]>,
        selected: Option<usize>,
        on_select: Option<Select>,
    ) -> Self {
        debug_assert_eq!(focus.len(), enabled.len());
        Self {
            focus,
            enabled,
            selected,
            on_select,
        }
    }

    /// The item that takes the group's tab stop.
    pub(crate) fn entry(&self) -> Option<usize> {
        roving_entry(&self.enabled, self.selected)
    }

    /// Move focus by `step` and select the item it lands on.
    pub(crate) fn go(&self, step: RovingStep, window: &mut Window, cx: &mut App) {
        let current = self
            .focus
            .iter()
            .position(|focus| focus.is_focused(window))
            .or(self.selected);
        let Some(index) = roving_step(&self.enabled, current, step) else {
            return;
        };
        self.focus[index].focus(window, cx);
        if self.selected != Some(index)
            && let Some(on_select) = &self.on_select
        {
            on_select(index, window, cx);
        }
    }

    /// Install the movement actions on the group's container, under the key
    /// `context` given to [`bind_roving_keys`].
    pub(crate) fn install<E: InteractiveElement>(self, element: E, context: &'static str) -> E {
        let (next, previous, first, last) = (self.clone(), self.clone(), self.clone(), self);
        element
            .key_context(context)
            .on_action(move |_: &NextChoice, window, cx| next.go(RovingStep::Next, window, cx))
            .on_action(move |_: &PreviousChoice, window, cx| {
                previous.go(RovingStep::Previous, window, cx)
            })
            .on_action(move |_: &FirstChoice, window, cx| first.go(RovingStep::First, window, cx))
            .on_action(move |_: &LastChoice, window, cx| last.go(RovingStep::Last, window, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn arrows_skip_unavailable_items_and_wrap() {
        let enabled = [true, false, true, true];
        assert_eq!(roving_step(&enabled, Some(0), RovingStep::Next), Some(2));
        assert_eq!(roving_step(&enabled, Some(3), RovingStep::Next), Some(0));
        assert_eq!(
            roving_step(&enabled, Some(2), RovingStep::Previous),
            Some(0)
        );
        assert_eq!(
            roving_step(&enabled, Some(0), RovingStep::Previous),
            Some(3)
        );
        assert_eq!(roving_step(&enabled, None, RovingStep::Next), Some(0));
        assert_eq!(roving_step(&enabled, None, RovingStep::Previous), Some(3));
        assert_eq!(roving_step(&enabled, Some(2), RovingStep::Last), Some(3));
        assert_eq!(roving_step(&enabled, Some(3), RovingStep::First), Some(0));
        assert_eq!(
            roving_step(&[false, false], Some(0), RovingStep::Next),
            None
        );
        assert_eq!(roving_step(&[], None, RovingStep::First), None);
    }

    #[test]
    fn an_all_available_group_moves_like_a_ring() {
        let enabled = [true; 3];
        assert_eq!(roving_step(&enabled, Some(1), RovingStep::Next), Some(2));
        assert_eq!(roving_step(&enabled, Some(2), RovingStep::Next), Some(0));
        assert_eq!(
            roving_step(&enabled, Some(0), RovingStep::Previous),
            Some(2)
        );
    }

    #[test]
    fn tab_enters_at_the_selected_available_item() {
        assert_eq!(roving_entry(&[true, true, true], Some(2)), Some(2));
        assert_eq!(roving_entry(&[true, true, false], Some(2)), Some(0));
        assert_eq!(roving_entry(&[false, true], None), Some(1));
        assert_eq!(roving_entry(&[true], Some(5)), Some(0));
        assert_eq!(roving_entry(&[false], None), None);
    }
}
