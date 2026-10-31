# Quiet global swap

Source: the published Apache-2.0 `gpui-pre` 0.3.10 package (`SOURCE.json`), the
same in `vendor/gpui-fast`. File: `src/app.rs`.

**A needed feature, not a bug fix.** It can be reverted alone: it depends on no
other local patch.

`App::swap_global_quietly` exchanges a global with a value, for a caller that puts
it back before the draw returns. `global_mut` would notify observers and, under
GPUI Fast, mark the global changed for every retained view that read it.

Material's `Scope` needs this to give a subtree its own Kit theme while it draws.
The global's value is only different inside the scope, so to anything outside it
has not changed. With `global_mut`, one animation frame inside a scope rebuilt every
view in the window that read Kit's theme. The scope states its own invalidation:
its holder reads a marker global that drawing the scope stamps, so a view nested in
it is rebuilt through the holder, in the scope, and not alone outside it.
