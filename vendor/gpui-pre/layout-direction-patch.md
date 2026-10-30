# Layout direction

Source: the published Apache-2.0 `gpui-pre` 0.3.10 package (`SOURCE.json`), the
same in `vendor/gpui-fast`. Files: `src/geometry.rs`, `src/taffy.rs`,
`src/window.rs`, `src/app.rs`, `src/style.rs`, `src/elements/text.rs`, and in Fast
`src/fast/layout.rs` and `src/fast/rounded_clip.rs`.

**A needed feature, not a bug fix.** Upstream GPUI has no direction concept, so
nothing here corrects upstream behavior. It can be reverted alone or proposed
upstream alone: it depends on no other local patch.

GPUI has no layout direction. Styles name physical sides (`pl`, `left`,
`rounded_tl`), so a right-to-left interface would need every element restyled.
Taffy 0.15 has a CSS `direction`, but as in CSS it only flips inline flow:
`padding-left` and `left` stay physical. That is the wrong model for an API that
only has physical names, so this patch mirrors instead:

- `LayoutDirection` (`Ltr`, `Rtl`), `Window::layout_direction`,
  `set_layout_direction`, `with_layout_direction`, and an application default
  (`App::set_layout_direction`) that windows follow until they choose their own.
- `Window::request_layout` records a node laid out under `Rtl`.
  `TaffyLayoutEngine::layout_bounds` then places each child of a recorded node at
  `parent_width - x - width`, so flex order, padding, margins, gaps and absolute
  offsets flip together. A node whose parent is not recorded stays where it is, so
  the root of a right-to-left subtree does not move.
- A horizontally scrolling parent mirrors across its content
  (`scrollable_overflow_rect.right`), keeping children inside its scrollable range.
- `Style::paint` exchanges an element's own left and right corner radii and
  border widths, and Fast's rounded clip does the same. Layout computed the border
  on one side and the mirrored child sits against the other, so the paint follows.
- `TextAlign::in_direction`: `Left` means the start of the line and `Right` its
  end when text elements paint. The bidirectional order of the text itself is the
  platform shaper's and is untouched.

Left-to-right trees take one extra branch per `request_layout` and one empty-set
check per `layout_bounds`, and allocate nothing.

Fast keeps layout nodes across frames, so its record outlives the frame like them:
each request rewrites the node's entry and releasing a transient node removes it.

Not covered, and left to the caller: painted paths and quads inside an element,
pointer arithmetic against `bounds.left()`, and key handling, which runs after the
draw. `gpui-kit-material` handles these per component.

Left-to-right behavior is unchanged: no node is recorded, so `child_offset` returns
the location as it is, and `in_direction` and the paint flips are identity.

Regression command:

```sh
cargo test -p gpui-kit-material --lib scope
```
