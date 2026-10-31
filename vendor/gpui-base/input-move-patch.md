# An input a parent moves is not notified

Source: the vendored `gpui-base` (`SOURCE.json`).
File: `src/input/base/element.rs`. Test: `crates/material/tests/input_move_guard.rs`.

The input's paint notified its view whenever its bounds or caret differed from the
last paint, and those are window coordinates, so a list scrolling inputs notified
every one it painted, on every frame it painted them. Under GPUI Fast a dirty view
dirties the view holding the list, whose layer then renders the rows it shows again,
which paints their inputs at a new offset, which notifies them again: a loop that
holds itself for as long as the scroll lasts.

The input now compares its size, its text's place inside its bounds and its caret's
place inside its bounds. Its position alone still notifies while it is focused, since
only a focused input has a caret for an open menu to read. The notification is not
needed for anything else: `text_bounds` and `input_bounds` are read on demand, and
Kit's readers do so while handling events.

`component_bench`, scroll, retired instructions per frame, median of 3 runs of 300:

| | before | after |
| --- | --- | --- |
| Kit `input-k` | 107.7M | 8.0M |
| Material `text-field-m` | 115.5M (53.0M to 157.2M) | 11.5M |
| rows rendered per frame, `input-k` | 52.8 | 0.61 |

A new row's inputs still notify on their first paint, which renders that row once
more; that is what is left above the 0.25 rows per frame of the other contents.
