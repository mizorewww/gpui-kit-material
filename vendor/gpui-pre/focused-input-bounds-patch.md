# Focused input selection bounds

Source: the published Apache-2.0 `gpui-pre` 0.3.10 package (`SOURCE.json`).
Files: `src/window.rs`; `gpui-fast` carries the same method.

`Window::focused_input_selection_bounds` returns the focused input handler's
`selected_bounds`, the query `invalidate_character_coordinates` already makes for
the IME panel, in window coordinates. A view can then keep the caret above an
on-screen keyboard without knowing which of its descendants is being edited. The
Gallery does that when the browser's visible area shrinks.
