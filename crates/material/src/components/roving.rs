//! Roving keyboard focus shared by every Material composite widget: radio
//! groups, choice tiles, connected button groups, tabs, navigation bars and
//! rails.
//!
//! GPUI Base's `Tab`/`Tabs`, `Radio`/`RadioGroup` own pointer activation and
//! their roles but deliberately leave compound keyboard navigation to the
//! caller (`gpui-base/src/tabs.rs`: "Add compound keyboard navigation with
//! roving focus"), and Kit's `TabBar` has no arrow-key navigation either. This
//! module is that one missing behavior:
//!
//! - one tab stop per widget ([`Roving::tab_stop`]): the focused item while
//!   focus is inside the widget, otherwise the selection when it can take
//!   focus, otherwise the first item that can. GPUI's Tab and Shift+Tab pass
//!   over handles marked `tab_stop(false)` (`gpui/src/tab_stop.rs`,
//!   `next_inner`/`prev_inner`), so the stop must follow focus: a stop left
//!   on the selection would pull Shift+Tab back into the widget after an
//!   arrow moved focus past it;
//! - arrow keys along the widget's [`Orientation`] move focus with wrapping,
//!   and Home/End go to the ends, skipping items that cannot take focus;
//! - optionally the selection follows focus (radio groups, auto-activating
//!   tabs); otherwise Enter and Space activate the focused item through
//!   GPUI's own keyboard click;
//! - optionally a scroll container reveals the newly focused item.
//!
//! A widget keeps its items' focus handles across frames ([`index_handles`]
//! or [`sync_handles`]), builds a [`Roving`] each frame and installs it on
//! its container, which also sets the orientation's key context.
use gpui::*;
use std::{collections::BTreeMap, rc::Rc};

actions!(
    material_roving,
    [
        FocusNextItem,
        FocusPreviousItem,
        FocusRightItem,
        FocusLeftItem,
        FocusFirstItem,
        FocusLastItem
    ]
);

/// Key context of a widget whose items run along a row (left/right).
pub(crate) const ROW_CONTEXT: &str = "MaterialRovingRow";
/// Key context of a widget whose items run down a column (up/down).
pub(crate) const COLUMN_CONTEXT: &str = "MaterialRovingColumn";
/// Key context of a widget that answers both axes (radio groups).
pub(crate) const BOTH_CONTEXT: &str = "MaterialRovingGroup";

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("right", FocusRightItem, Some(ROW_CONTEXT)),
        KeyBinding::new("left", FocusLeftItem, Some(ROW_CONTEXT)),
        KeyBinding::new("home", FocusFirstItem, Some(ROW_CONTEXT)),
        KeyBinding::new("end", FocusLastItem, Some(ROW_CONTEXT)),
        KeyBinding::new("down", FocusNextItem, Some(COLUMN_CONTEXT)),
        KeyBinding::new("up", FocusPreviousItem, Some(COLUMN_CONTEXT)),
        KeyBinding::new("home", FocusFirstItem, Some(COLUMN_CONTEXT)),
        KeyBinding::new("end", FocusLastItem, Some(COLUMN_CONTEXT)),
        KeyBinding::new("right", FocusRightItem, Some(BOTH_CONTEXT)),
        KeyBinding::new("down", FocusNextItem, Some(BOTH_CONTEXT)),
        KeyBinding::new("left", FocusLeftItem, Some(BOTH_CONTEXT)),
        KeyBinding::new("up", FocusPreviousItem, Some(BOTH_CONTEXT)),
        KeyBinding::new("home", FocusFirstItem, Some(BOTH_CONTEXT)),
        KeyBinding::new("end", FocusLastItem, Some(BOTH_CONTEXT)),
    ]);
}

/// Which arrow keys move between a widget's items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Orientation {
    /// Left/Right (tabs, navigation bars).
    Row,
    /// Up/Down (navigation rails).
    Column,
    /// All four arrows (radio groups, connected button groups).
    Both,
}

impl Orientation {
    pub(crate) const fn context(self) -> &'static str {
        match self {
            Self::Row => ROW_CONTEXT,
            Self::Column => COLUMN_CONTEXT,
            Self::Both => BOTH_CONTEXT,
        }
    }
}

/// One arrow-key, Home or End movement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    Next,
    Previous,
    First,
    Last,
}

/// The item `direction` lands on from `current` among `count` items when
/// none is skipped. Arrows wrap; with no current item, Next starts at the
/// first and Previous at the last.
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

/// The item `direction` lands on from `current`, passing over items that
/// cannot take focus (`skip`); `None` when no item can.
pub(crate) fn target(skip: &[bool], current: Option<usize>, direction: Direction) -> Option<usize> {
    let count = skip.len();
    let mut index = destination(count, current, direction)?;
    let step = match direction {
        Direction::First | Direction::Next => Direction::Next,
        Direction::Last | Direction::Previous => Direction::Previous,
    };
    for _ in 0..count {
        if !skip[index] {
            return Some(index);
        }
        index = destination(count, Some(index), step)?;
    }
    None
}

/// The item that holds a widget's one tab stop: the selection when it can
/// take focus, otherwise the first item that can.
pub(crate) fn entry(selected: Option<usize>, skip: &[bool]) -> Option<usize> {
    selected
        .filter(|index| !skip.get(*index).copied().unwrap_or(true))
        .or_else(|| skip.iter().position(|skip| !skip))
}

/// The item that holds a widget's one tab stop while `focused` holds focus:
/// the focused item when it can take focus, otherwise [`entry`].
pub(crate) fn stop(
    focused: Option<usize>,
    selected: Option<usize>,
    skip: &[bool],
) -> Option<usize> {
    focused
        .filter(|index| !skip.get(*index).copied().unwrap_or(true))
        .or_else(|| entry(selected, skip))
}

/// Called with the index of the item keyboard movement newly focuses.
pub(crate) type Select = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The roving state of one rendered widget. Clone it into each handler.
#[derive(Clone)]
pub(crate) struct Roving {
    focus: Rc<[FocusHandle]>,
    skip: Rc<[bool]>,
    selected: Option<usize>,
    orientation: Orientation,
    select_on_focus: Option<Select>,
    reveal: Option<ScrollHandle>,
    rtl: bool,
}

impl Roving {
    /// Resolve left and right arrows for this direction. Key handling runs
    /// after the draw, outside any scope, so the caller passes what it rendered in.
    pub(crate) fn rtl(mut self, rtl: bool) -> Self {
        self.rtl = rtl;
        self
    }

    /// One focus handle per item; every item can take focus until
    /// [`Roving::skip`] says otherwise.
    pub(crate) fn new(focus: impl Into<Rc<[FocusHandle]>>, orientation: Orientation) -> Self {
        let focus = focus.into();
        Self {
            skip: vec![false; focus.len()].into(),
            focus,
            selected: None,
            orientation,
            select_on_focus: None,
            reveal: None,
            rtl: false,
        }
    }

    /// Items that cannot take focus (disabled), one entry per item; arrows,
    /// Home, End and the tab stop pass over them.
    pub(crate) fn skip(mut self, skip: impl Into<Rc<[bool]>>) -> Self {
        let skip = skip.into();
        debug_assert_eq!(skip.len(), self.focus.len());
        self.skip = skip;
        self
    }

    /// Make every item unreachable (a disabled widget).
    pub(crate) fn disabled(self, disabled: bool) -> Self {
        if disabled {
            let count = self.focus.len();
            self.skip(vec![true; count])
        } else {
            self
        }
    }

    pub(crate) fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    /// Selection follows focus: `select` is called with each item keyboard
    /// movement focuses, unless it is already selected. Without it arrows
    /// only move focus and Enter/Space activates.
    pub(crate) fn select_on_focus(mut self, select: Option<Select>) -> Self {
        self.select_on_focus = select;
        self
    }

    /// A scroll container whose children are the items, scrolled to reveal
    /// the newly focused item.
    pub(crate) fn reveal(mut self, scroll: Option<ScrollHandle>) -> Self {
        self.reveal = scroll;
        self
    }

    /// The item that takes the widget's tab stop this frame (see [`stop`]).
    /// Focus changes refresh the window, so the stop follows focus.
    pub(crate) fn tab_stop(&self, window: &Window) -> Option<usize> {
        let focused = self.focus.iter().position(|focus| focus.is_focused(window));
        stop(focused, self.selected, &self.skip)
    }

    pub(crate) fn navigate(&self, direction: Direction, window: &mut Window, cx: &mut App) {
        let current = self
            .focus
            .iter()
            .position(|focus| focus.is_focused(window))
            .or(self.selected);
        let Some(index) = target(&self.skip, current, direction) else {
            return;
        };
        self.focus[index].focus(window, cx);
        if let Some(scroll) = &self.reveal {
            scroll.scroll_to_item(index);
        }
        if self.selected != Some(index)
            && let Some(select) = &self.select_on_focus
        {
            select(index, window, cx);
        }
    }

    /// Set the orientation's key context on `element` and install the four
    /// movement actions.
    pub(crate) fn install<E: InteractiveElement>(self, element: E) -> E {
        let next = self.clone();
        let previous = self.clone();
        let right = self.clone();
        let left = self.clone();
        let (toward_right, toward_left) = if self.rtl {
            (Direction::Previous, Direction::Next)
        } else {
            (Direction::Next, Direction::Previous)
        };
        let first = self.clone();
        let context = self.orientation.context();
        let last = self;
        element
            .key_context(context)
            .on_action(move |_: &FocusNextItem, window, cx| {
                next.navigate(Direction::Next, window, cx)
            })
            .on_action(move |_: &FocusPreviousItem, window, cx| {
                previous.navigate(Direction::Previous, window, cx)
            })
            .on_action(move |_: &FocusRightItem, window, cx| {
                right.navigate(toward_right, window, cx)
            })
            .on_action(move |_: &FocusLeftItem, window, cx| left.navigate(toward_left, window, cx))
            .on_action(move |_: &FocusFirstItem, window, cx| {
                first.navigate(Direction::First, window, cx)
            })
            .on_action(move |_: &FocusLastItem, window, cx| {
                last.navigate(Direction::Last, window, cx)
            })
    }
}

/// A value-keyed widget's change callback.
pub(crate) type Change = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// Adapt a value callback to [`Select`] for widgets keyed by value.
pub(crate) fn select_value(
    values: impl IntoIterator<Item = SharedString>,
    on_change: Change,
) -> Select {
    let values: Rc<[SharedString]> = values.into_iter().collect();
    Rc::new(move |index, window, cx| {
        if let Some(value) = values.get(index) {
            on_change(value, window, cx);
        }
    })
}

/// One focus handle per position of the widget keyed by `id`, kept across
/// frames. Extra handles go with removed items.
pub(crate) fn index_handles(
    id: impl Into<ElementId>,
    count: usize,
    window: &mut Window,
    cx: &mut App,
) -> Rc<[FocusHandle]> {
    let handles = window.use_keyed_state(id, cx, |_, _| Vec::<FocusHandle>::new());
    // Focus on an item that goes would fall out of the widget, and its arrow
    // keys with it: it moves to the nearest item that stays.
    let current = handles.read(cx);
    if count > 0
        && current.len() > count
        && current[count..]
            .iter()
            .any(|handle| handle.is_focused(window))
    {
        let neighbor = current[count - 1].clone();
        window.defer(cx, move |window, cx| neighbor.focus(window, cx));
    }
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

/// The focus handle of each item keyed by value, created on first sight.
/// Handles of items that are gone are dropped, so the map tracks the current
/// items instead of every value ever shown. Keys must be unique.
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

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use std::cell::RefCell;

    #[test]
    fn arrows_wrap_and_start_when_the_current_item_is_unknown() {
        assert_eq!(destination(3, Some(0), Direction::Next), Some(1));
        assert_eq!(destination(3, Some(2), Direction::Next), Some(0));
        assert_eq!(destination(3, Some(0), Direction::Previous), Some(2));
        assert_eq!(destination(3, None, Direction::Next), Some(0));
        assert_eq!(destination(3, None, Direction::Previous), Some(2));
        assert_eq!(destination(3, Some(1), Direction::First), Some(0));
        assert_eq!(destination(3, Some(1), Direction::Last), Some(2));
        assert_eq!(destination(0, Some(0), Direction::Next), None);
    }

    #[test]
    fn arrows_skip_items_that_cannot_take_focus() {
        let skip = [false, true, false, true];
        assert_eq!(target(&skip, Some(0), Direction::Next), Some(2));
        assert_eq!(target(&skip, Some(2), Direction::Next), Some(0));
        assert_eq!(target(&skip, Some(0), Direction::Previous), Some(2));
        assert_eq!(target(&skip, None, Direction::Last), Some(2));
        assert_eq!(target(&[true, false], None, Direction::First), Some(1));
        assert_eq!(target(&[true, true], None, Direction::Next), None);
        assert_eq!(target(&[], None, Direction::First), None);
    }

    #[test]
    fn the_tab_stop_prefers_a_focusable_selection() {
        assert_eq!(entry(Some(1), &[false, false]), Some(1));
        assert_eq!(entry(Some(1), &[true, true, false]), Some(2));
        assert_eq!(entry(None, &[true, false]), Some(1));
        assert_eq!(entry(Some(5), &[false]), Some(0));
        assert_eq!(entry(None, &[true]), None);
    }

    #[test]
    fn the_tab_stop_follows_a_focusable_focused_item() {
        assert_eq!(stop(Some(2), Some(0), &[false; 3]), Some(2));
        assert_eq!(stop(Some(1), Some(0), &[false, true, false]), Some(0));
        assert_eq!(stop(None, Some(1), &[false; 3]), Some(1));
        assert_eq!(stop(Some(0), None, &[true]), None);
    }

    #[test]
    fn each_orientation_has_its_own_key_context() {
        assert_eq!(Orientation::Row.context(), ROW_CONTEXT);
        assert_eq!(Orientation::Column.context(), COLUMN_CONTEXT);
        assert_eq!(Orientation::Both.context(), BOTH_CONTEXT);
    }

    actions!(roving_harness, [TabNext, TabPrev]);

    struct Harness {
        focus: Rc<[FocusHandle]>,
        orientation: Orientation,
        skip: Vec<bool>,
        selected: Option<usize>,
        follow: bool,
        picked: Rc<RefCell<Vec<usize>>>,
        before: FocusHandle,
        after: FocusHandle,
    }

    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let picked = self.picked.clone();
            let entity = cx.entity().downgrade();
            let select: Option<Select> = self.follow.then(|| {
                Rc::new(move |index, _: &mut Window, cx: &mut App| {
                    picked.borrow_mut().push(index);
                    entity
                        .update(cx, |this, cx| {
                            this.selected = Some(index);
                            cx.notify();
                        })
                        .ok();
                }) as Select
            });
            let roving = Roving::new(self.focus.clone(), self.orientation)
                .skip(self.skip.clone())
                .selected(self.selected)
                .select_on_focus(select);
            let entry = roving.tab_stop(window);
            div()
                .key_context("RovingHarness")
                .on_action(|_: &TabNext, window, cx| window.focus_next(cx))
                .on_action(|_: &TabPrev, window, cx| window.focus_prev(cx))
                .child(
                    div()
                        .id("before")
                        .size(px(20.))
                        .track_focus(&self.before.clone().tab_stop(true)),
                )
                .child(roving.install(div().id("group")).children(
                    self.focus.iter().enumerate().map(|(index, focus)| {
                        div()
                            .id(index)
                            .size(px(20.))
                            .track_focus(&focus.clone().tab_stop(entry == Some(index)))
                    }),
                ))
                .child(
                    div()
                        .id("after")
                        .size(px(20.))
                        .track_focus(&self.after.clone().tab_stop(true)),
                )
        }
    }

    type Rig<'a> = (
        Entity<Harness>,
        Rc<[FocusHandle]>,
        Rc<RefCell<Vec<usize>>>,
        &'a mut gpui::VisualTestContext,
    );

    fn harness(
        cx: &mut gpui::TestAppContext,
        orientation: Orientation,
        skip: Vec<bool>,
        follow: bool,
    ) -> Rig<'_> {
        cx.update(|cx| {
            init(cx);
            cx.bind_keys([
                KeyBinding::new("tab", TabNext, Some("RovingHarness")),
                KeyBinding::new("shift-tab", TabPrev, Some("RovingHarness")),
            ]);
        });
        let picked = Rc::new(RefCell::new(Vec::new()));
        let focus: Rc<[FocusHandle]> =
            cx.update(|cx| (0..skip.len()).map(|_| cx.focus_handle()).collect());
        let (view, cx) = cx.add_window_view({
            let (focus, picked) = (focus.clone(), picked.clone());
            move |_, cx| Harness {
                before: cx.focus_handle(),
                after: cx.focus_handle(),
                focus,
                orientation,
                skip,
                selected: None,
                follow,
                picked,
            }
        });
        cx.run_until_parked();
        (view, focus, picked, cx)
    }

    fn focused(focus: &[FocusHandle], cx: &mut gpui::VisualTestContext) -> Option<usize> {
        cx.update(|window, _| focus.iter().position(|focus| focus.is_focused(window)))
    }

    #[gpui::test]
    fn a_row_moves_with_left_and_right_only(cx: &mut gpui::TestAppContext) {
        let (_, focus, picked, cx) = harness(cx, Orientation::Row, vec![false, true, false], false);
        cx.update(|window, cx| focus[0].focus(window, cx));
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&focus, cx), Some(2), "the disabled item is skipped");
        cx.simulate_keystrokes("down");
        assert_eq!(focused(&focus, cx), Some(2), "a row ignores Up/Down");
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&focus, cx), Some(0), "arrows wrap");
        cx.simulate_keystrokes("end");
        assert_eq!(focused(&focus, cx), Some(2));
        assert!(picked.borrow().is_empty(), "focus alone selects nothing");
    }

    #[gpui::test]
    fn a_column_moves_with_up_and_down_only(cx: &mut gpui::TestAppContext) {
        let (_, focus, _, cx) = harness(cx, Orientation::Column, vec![false; 3], false);
        cx.update(|window, cx| focus[1].focus(window, cx));
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&focus, cx), Some(1), "a column ignores Left/Right");
        cx.simulate_keystrokes("up");
        assert_eq!(focused(&focus, cx), Some(0));
        cx.simulate_keystrokes("up");
        assert_eq!(focused(&focus, cx), Some(2));
        cx.simulate_keystrokes("home");
        assert_eq!(focused(&focus, cx), Some(0));
    }

    #[gpui::test]
    fn selection_follows_focus_on_both_axes(cx: &mut gpui::TestAppContext) {
        let (view, focus, picked, cx) = harness(cx, Orientation::Both, vec![false; 3], true);
        cx.update(|window, cx| focus[0].focus(window, cx));
        cx.simulate_keystrokes("down");
        cx.simulate_keystrokes("right");
        cx.simulate_keystrokes("left");
        assert_eq!(focused(&focus, cx), Some(1));
        assert_eq!(*picked.borrow(), [1, 2, 1]);
        assert_eq!(view.read_with(cx, |view, _| view.selected), Some(1));
    }

    #[gpui::test]
    fn a_disabled_widget_cannot_be_entered(cx: &mut gpui::TestAppContext) {
        let focus: Rc<[FocusHandle]> = cx.update(|cx| (0..2).map(|_| cx.focus_handle()).collect());
        let roving = Roving::new(focus, Orientation::Row)
            .selected(Some(0))
            .disabled(true);
        assert_eq!(entry(roving.selected, &roving.skip), None);
    }

    #[gpui::test]
    fn tab_and_shift_tab_leave_after_arrowing_in_manual_mode(cx: &mut gpui::TestAppContext) {
        let (view, focus, _, cx) = harness(cx, Orientation::Row, vec![false; 3], false);
        view.update(cx, |view, _| view.selected = Some(0));
        let (before, after) =
            view.read_with(cx, |view, _| (view.before.clone(), view.after.clone()));
        cx.update(|window, cx| before.focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&focus, cx), Some(0), "Tab enters on the selection");
        cx.simulate_keystrokes("right right");
        assert_eq!(focused(&focus, cx), Some(2));
        cx.simulate_keystrokes("shift-tab");
        assert!(
            cx.update(|window, _| before.is_focused(window)),
            "Shift+Tab leaves the widget backward"
        );
        cx.simulate_keystrokes("tab");
        assert_eq!(
            focused(&focus, cx),
            Some(0),
            "the stop returns to the selection"
        );
        cx.simulate_keystrokes("right");
        cx.simulate_keystrokes("tab");
        assert!(
            cx.update(|window, _| after.is_focused(window)),
            "Tab leaves the widget forward"
        );
    }

    struct Shrinking {
        count: usize,
        handles: Rc<RefCell<Rc<[FocusHandle]>>>,
    }
    impl gpui::Render for Shrinking {
        fn render(
            &mut self,
            window: &mut Window,
            cx: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            let handles = index_handles("items", self.count, window, cx);
            *self.handles.borrow_mut() = handles.clone();
            div().flex().flex_col().children(
                handles
                    .iter()
                    .enumerate()
                    .map(|(i, handle)| div().id(i).track_focus(handle).size(px(10.))),
            )
        }
    }

    #[gpui::test]
    fn focus_moves_to_a_neighbor_when_its_item_goes(cx: &mut gpui::TestAppContext) {
        let handles: Rc<RefCell<Rc<[FocusHandle]>>> = Rc::new(RefCell::new(Rc::from([])));
        let (view, cx) = cx.add_window_view({
            let handles = handles.clone();
            move |_, _| Shrinking { count: 3, handles }
        });
        let draw = |cx: &mut gpui::VisualTestContext| {
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
            cx.run_until_parked();
        };
        draw(cx);
        let last = handles.borrow()[2].clone();
        cx.update(|window, cx| last.focus(window, cx));
        draw(cx);
        drop(last);
        view.update(cx, |view, cx| {
            view.count = 2;
            cx.notify();
        });
        draw(cx);
        draw(cx);
        let kept = handles.borrow()[1].clone();
        cx.update(|window, _| assert!(kept.is_focused(window), "focus stays in the widget"));
    }
}
