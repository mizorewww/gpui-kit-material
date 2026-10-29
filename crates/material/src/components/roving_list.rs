//! Roving keyboard focus shared by Material composite widgets (connected
//! button groups, tabs, navigation bars and rails).
//!
//! GPUI Base's `Tab`/`Tabs` own pointer activation and tab/tablist
//! semantics but deliberately leave compound keyboard navigation to the
//! caller (`gpui-base/src/tabs.rs`: "Add compound keyboard navigation with
//! roving focus"), and Kit's `TabBar` has no arrow-key navigation either. This
//! module is that one missing behavior: one tab stop per widget (the current
//! item), arrow keys that move focus between items with wrapping, and
//! Home/End. Enter and Space activate the focused item through GPUI's own
//! keyboard click on the focused element.
use gpui::*;
use std::{collections::BTreeMap, rc::Rc};

actions!(
    material_roving,
    [
        FocusNextItem,
        FocusPreviousItem,
        FocusFirstItem,
        FocusLastItem
    ]
);

/// Key context of a widget whose items run along a row (left/right).
pub(crate) const ROW_CONTEXT: &str = "MaterialRovingRow";
/// Key context of a widget whose items run down a column (up/down).
pub(crate) const COLUMN_CONTEXT: &str = "MaterialRovingColumn";

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("right", FocusNextItem, Some(ROW_CONTEXT)),
        KeyBinding::new("left", FocusPreviousItem, Some(ROW_CONTEXT)),
        KeyBinding::new("home", FocusFirstItem, Some(ROW_CONTEXT)),
        KeyBinding::new("end", FocusLastItem, Some(ROW_CONTEXT)),
        KeyBinding::new("down", FocusNextItem, Some(COLUMN_CONTEXT)),
        KeyBinding::new("up", FocusPreviousItem, Some(COLUMN_CONTEXT)),
        KeyBinding::new("home", FocusFirstItem, Some(COLUMN_CONTEXT)),
        KeyBinding::new("end", FocusLastItem, Some(COLUMN_CONTEXT)),
    ]);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    Next,
    Previous,
    First,
    Last,
}

/// Arrow keys wrap; with no current item, Next starts at the first and
/// Previous at the last.
pub(crate) fn destination(
    count: usize,
    current: Option<usize>,
    direction: Direction,
) -> Option<usize> {
    if count == 0 {
        return None;
    }
    Some(match direction {
        Direction::First => 0,
        Direction::Last => count - 1,
        Direction::Next => current.map_or(0, |index| (index + 1) % count),
        Direction::Previous => current.map_or(count - 1, |index| (index + count - 1) % count),
    })
}

pub(crate) type Change = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// The roving state of one rendered widget: its item values, their focus
/// handles and the current selection. Clone it into each action handler.
#[derive(Clone)]
pub(crate) struct Roving {
    pub values: Vec<SharedString>,
    pub focus: Vec<FocusHandle>,
    /// Items that cannot take focus (disabled), skipped by arrow keys.
    pub skip: Vec<bool>,
    pub selected: Option<usize>,
    pub on_change: Option<Change>,
    /// Selection follows focus (radio groups and auto-activating tabs).
    /// Otherwise arrows only move focus and Enter/Space activates.
    pub follow_focus: bool,
    /// A scroll container whose children are the items, scrolled to reveal
    /// the newly focused item.
    pub reveal: Option<ScrollHandle>,
}

impl Roving {
    /// The next focusable index from `current` in `direction`, skipping
    /// disabled items; `None` when no item can take focus.
    fn target(&self, current: Option<usize>, direction: Direction) -> Option<usize> {
        let count = self.values.len();
        let mut index = destination(count, current, direction)?;
        let step = match direction {
            Direction::First | Direction::Next => Direction::Next,
            Direction::Last | Direction::Previous => Direction::Previous,
        };
        for _ in 0..count {
            if !self.skip.get(index).copied().unwrap_or(false) {
                return Some(index);
            }
            index = destination(count, Some(index), step)?;
        }
        None
    }

    pub fn navigate(&self, direction: Direction, window: &mut Window, cx: &mut App) {
        let current = self
            .focus
            .iter()
            .position(|focus| focus.is_focused(window))
            .or(self.selected);
        let Some(index) = self.target(current, direction) else {
            return;
        };
        self.focus[index].focus(window, cx);
        if let Some(scroll) = &self.reveal {
            scroll.scroll_to_item(index);
        }
        if self.follow_focus
            && self.selected != Some(index)
            && let Some(on_change) = &self.on_change
        {
            on_change(&self.values[index], window, cx);
        }
    }

    /// Install the four roving actions on `element`.
    pub fn install<E: InteractiveElement>(self, element: E) -> E {
        let next = self.clone();
        let previous = self.clone();
        let first = self.clone();
        let last = self;
        element
            .on_action(move |_: &FocusNextItem, window, cx| {
                next.navigate(Direction::Next, window, cx)
            })
            .on_action(move |_: &FocusPreviousItem, window, cx| {
                previous.navigate(Direction::Previous, window, cx)
            })
            .on_action(move |_: &FocusFirstItem, window, cx| {
                first.navigate(Direction::First, window, cx)
            })
            .on_action(move |_: &FocusLastItem, window, cx| {
                last.navigate(Direction::Last, window, cx)
            })
    }
}

/// The focus handle of each item, created on first sight. Handles of items
/// that are gone are dropped, so the map tracks the current items instead of
/// every value ever shown. Keys must be unique.
pub(crate) fn sync_handles<T>(
    handles: &mut BTreeMap<SharedString, FocusHandle>,
    items: &[T],
    key: impl Fn(&T) -> &SharedString,
    mut create: impl FnMut() -> FocusHandle,
) -> Vec<FocusHandle> {
    let current = items
        .iter()
        .map(|item| {
            handles
                .entry(key(item).clone())
                .or_insert_with(&mut create)
                .clone()
        })
        .collect::<Vec<_>>();
    // Keys are unique, so any extra entry is stale.
    if handles.len() > items.len() {
        handles.retain(|value, _| items.iter().any(|item| key(item) == value));
    }
    current
}

/// The item that holds the widget's one tab stop: the selection, or the
/// first focusable item.
pub(crate) fn entry(selected: Option<usize>, skip: &[bool]) -> Option<usize> {
    selected
        .filter(|index| !skip.get(*index).copied().unwrap_or(false))
        .or_else(|| skip.iter().position(|skip| !skip))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roving(skip: Vec<bool>) -> Roving {
        Roving {
            values: (0..skip.len()).map(|i| i.to_string().into()).collect(),
            focus: Vec::new(),
            skip,
            selected: None,
            on_change: None,
            follow_focus: false,
            reveal: None,
        }
    }

    #[core::prelude::v1::test]
    fn arrows_skip_disabled_items_and_wrap() {
        let r = roving(vec![false, true, false, true]);
        assert_eq!(r.target(Some(0), Direction::Next), Some(2));
        assert_eq!(r.target(Some(2), Direction::Next), Some(0));
        assert_eq!(r.target(Some(0), Direction::Previous), Some(2));
        assert_eq!(r.target(None, Direction::Last), Some(2));
        let r = roving(vec![true, false]);
        assert_eq!(r.target(None, Direction::First), Some(1));
        assert_eq!(roving(vec![true, true]).target(None, Direction::Next), None);
    }

    #[core::prelude::v1::test]
    fn the_tab_stop_prefers_an_enabled_selection() {
        assert_eq!(entry(Some(1), &[false, false]), Some(1));
        assert_eq!(entry(Some(1), &[true, true, false]), Some(2));
        assert_eq!(entry(None, &[true, false]), Some(1));
        assert_eq!(entry(None, &[true]), None);
    }
}
