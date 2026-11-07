//! Where the focused Material text input's caret was drawn, for a page that keeps
//! it above an on-screen keyboard.
//!
//! GPUI keeps the focused input handler private to its window, so a view cannot
//! ask it where editing happens. Material's [`TextField`](super::TextField) and
//! [`SearchBar`](super::SearchBar) wrap Kit's input in [`CaretReport`], which
//! records the caret Kit laid out this frame while that input has focus. The
//! record is per window and lives outside GPUI's globals, so writing it while
//! painting invalidates nothing.
//!
//! The same wrapper gives Kit's input Material's caret and selection colors
//! ([`CaretReport::ink`]) where they differ from Kit's theme: an error field's
//! caret, or any field inside a [`Scope`](crate::scope::Scope), which leaves Kit's
//! theme alone.
use gpui::*;
use gpui_base::input::{DiagnosticColors, InputBaseState, InputEditorStyle, InputModeKind};
use gpui_kit::component::Theme as ComponentTheme;
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
    /// The caret and selection colors to give the input, when not Kit's.
    ink: Option<(Hsla, Hsla)>,
}

impl<M: InputModeKind> CaretReport<M> {
    pub(crate) fn around(child: AnyElement, state: Entity<InputBaseState<M>>) -> Self {
        Self {
            child,
            state,
            ink: None,
        }
    }

    /// Paint the input's caret and selection in these colors. Kit's input takes
    /// them from Kit's theme as it renders; where they differ, the input is given
    /// them right after, and the rest of its style is what Kit's gives it.
    pub(crate) fn ink(mut self, caret: Hsla, selection: Hsla, cx: &App) -> Self {
        let theme = ComponentTheme::global(cx);
        if theme.caret != caret || theme.selection != selection {
            self.ink = Some((caret, selection));
        }
        self
    }
}

/// The style Kit's `Input` gives its state as it renders (`Input::render` at Kit
/// `b881cadb`), with Material's caret and selection. A Material field never folds,
/// so it has no fold icons.
fn editor_style(caret: Hsla, selection: Hsla, cx: &App) -> InputEditorStyle {
    let theme = ComponentTheme::global(cx);
    let highlight = &theme.highlight_theme;
    let status = &highlight.style.status;
    InputEditorStyle {
        foreground: theme.foreground,
        muted_foreground: theme.muted_foreground,
        background: highlight
            .style
            .editor_background
            .unwrap_or_else(|| theme.input_background()),
        border: theme.border,
        selection,
        caret,
        diagnostics: DiagnosticColors {
            error: status.error(cx),
            warning: status.warning(cx),
            info: status.info(cx),
            hint: status.hint(cx),
        },
        highlight_styles: highlight.clone(),
        editor_invisible: highlight.style.editor_invisible,
        editor_active_line: highlight.style.editor_active_line,
        editor_gutter_background: highlight.style.editor_gutter_background,
        fold_icon_renderer: None,
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
        let layout = self.child.request_layout(window, cx);
        // Kit's input has rendered, writing its style; this one replaces it for
        // the frame, and an input drawn from last frame keeps it.
        if let Some((caret, selection)) = self.ink {
            let style = editor_style(caret, selection, cx);
            self.state
                .update(cx, |state, _| state.set_editor_style(style));
        }
        (layout, ())
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
