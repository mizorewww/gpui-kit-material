# IME anchor

Source: the vendored `gpui-base` (`SOURCE.json`).
Files: `src/input/base/element.rs`, `src/input/base/state.rs`.

The input calls `Window::invalidate_character_coordinates` when it paints focused
and its caret or origin moved since it last did. GPUI then reports the selection
to the platform (`update_ime_position`), which upstream's input never asked for, so
a browser's IME panel stayed at the page's corner. Only a move schedules anything;
a focused input standing still does not. An input that loses focus forgets
where it last reported its caret, so when it is focused again with the caret
unchanged it reports it again. Test:
`a_focused_input_reports_a_moved_caret_to_the_platform`. Upstream PR: the
`material/input-ime-anchor` branch.
