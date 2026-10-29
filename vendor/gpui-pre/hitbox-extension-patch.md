# Hitbox extensions independent of layout

Source: the published Apache-2.0 `gpui-pre` 0.3.10 package (`SOURCE.json`).
Files: `src/window.rs`, `src/elements/div.rs`.

`Hitbox` gains a public field `additional_bounds: Option<Bounds<Pixels>>` and
`Hitbox::contains_pointer`, which tests the visual bounds or the extension,
each clipped to the hitbox content mask (not occlusion; `is_hovered_at` still
handles that). `Window::insert_hitbox_with_extension` inserts one hitbox with
an extra pointer rectangle; `insert_hitbox` delegates with `None`. Both
rectangles share one `HitboxId`, behavior and event handlers. Visual bounds,
layout and event anchor geometry are unchanged.

`InteractiveElement::hitbox_extension(callback)` computes the extension during
prepaint from the resolved bounds and style. Hit testing, the mouse-down-out
listener and the prepaint hover check use `contains_pointer` instead of the
visual bounds. Material uses this for minimum touch targets that must not
change layout.

Adding a public struct field is source-incompatible with code that builds
`Hitbox` literals: vendored `gpui-base` (`src/text_selection.rs` tests and
selection registration) and the `gpui-fast` retained/composition layers carry
the matching follow-ups.

Regression command:

```sh
cargo test -p gpui-kit-material --test hitbox_extension
```
