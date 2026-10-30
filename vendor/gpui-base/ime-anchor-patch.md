# IME anchor

Source: the vendored `gpui-base` (`SOURCE.json`).
Files: `src/input/base/element.rs`, `src/input/base/state.rs`.

The input calls `Window::invalidate_character_coordinates` when it paints focused
and its caret or origin moved since it last did. GPUI then reports the selection
to the platform (`update_ime_position`), which upstream's input never asked for, so
a browser's IME panel stayed at the page's corner. Only a move schedules anything;
a focused input standing still does not.
