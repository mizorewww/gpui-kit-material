//! Where the focused Material text input's caret was drawn, for a page that keeps
//! it above an on-screen keyboard.
//!
//! GPUI keeps the focused input handler private to its window, so a view cannot
//! ask it where editing happens. Material's [`TextField`](super::TextField) and
//! [`SearchBar`](super::SearchBar) wrap Kit's input in [`CaretReport`], which
//! records the caret Kit laid out this frame while that input has focus. The
//! record is per window and lives outside GPUI's globals, so writing it while
//! painting invalidates nothing.
use gpui::*;
use gpui_base::input::{InputBaseState, InputModeKind};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    /// Per window: the focused input's handle and its caret, last frame drawn.
    static CARETS: RefCell<HashMap<WindowId, (WeakFocusHandle, Bounds<Pixels>)>> =
        RefCell::new(HashMap::new());
}

/// The caret of the Material text input focused in `window`, in window
/// coordinates, as it was last drawn. `None` when focus is elsewhere, including
/// in an input that is not Material's.
pub fn focused_caret_bounds(window: &Window) -> Option<Bounds<Pixels>> {
    let id = window.window_handle().window_id();
    CARETS.with_borrow_mut(|carets| {
        let (handle, bounds) = carets.get(&id)?;
        match handle.upgrade() {
            Some(handle) if handle.is_focused(window) => Some(*bounds),
            _ => {
                carets.remove(&id);
                None
            }
        }
    })
}

/// Draws a Kit input and records its caret while it has focus.
pub(crate) struct CaretReport<M: InputModeKind> {
    child: AnyElement,
    state: Entity<InputBaseState<M>>,
}

impl<M: InputModeKind> CaretReport<M> {
    pub(crate) fn around(child: AnyElement, state: Entity<InputBaseState<M>>) -> Self {
        Self { child, state }
    }
}

impl<M: InputModeKind> IntoElement for CaretReport<M> {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl<M: InputModeKind> Element for CaretReport<M> {
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
        let state = self.state.read(cx);
        let handle = state.focus_handle(cx);
        if !handle.is_focused(window) {
            return;
        }
        // Kit publishes the caret at the end of its prepaint; before its first
        // layout, the input's own box stands in.
        let caret = state.cursor_layout().map_or(bounds, |(caret, _)| caret);
        let id = window.window_handle().window_id();
        CARETS.with_borrow_mut(|carets| carets.insert(id, (handle.downgrade(), caret)));
    }
}
