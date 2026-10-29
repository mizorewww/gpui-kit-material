# Architecture

## Layers

```
gpui-kit-material-color   palettes → Scheme (ColorRole → Color); pure Rust
        │
gpui-kit-material
  theme       painted Scheme for this frame; palette transitions; text scale
  tokens      shape · typography · layout · elevation · state layers
  motion      M3 springs; enter / page_enter / value / selection_value
  icons       Icon element + the kit's glyphs (AssetSource with fallback)
  components  Material looks on GPUI Base behavior
```

Each layer only depends on the ones above it in this list. Components never
read a palette's stored colors; they paint with `ColorRole`s through
`theme::color`.

## Decisions

**GPUI Base for behavior.** Buttons, radios, tabs, focus, keyboard navigation,
accessibility roles and transitions come from `gpui-base`. The kit decides only
how Material looks and moves. It does not depend on `gpui-component`; apps that
use both mirror `theme::scheme()` into gpui-component's `Theme` when
`theme::apply` reports a change.

**One GPUI.** The workspace pins the `gpui-pre` snapshot that the published
`gpui-base` requires (`=0.3.10` with `gpui-base =0.8.1`). Bump them together.
An application on a patched Zed checkout overrides `gpui-pre` for the whole
graph, including this kit.

**Thread-local paint.** `theme::color` is context-free so helpers stay plain
functions. GPUI renders on one UI thread; the paint is per thread.

**Rems follow text.** 14px = 1rem at 100%. Geometry that shares a ruler with
text is in rems (`typography::rem(px)`); platform geometry (caption buttons,
hairlines, tile gaps) stays in px.

**No locale.** The kit ships no translated strings. Components that need
accessible names (e.g. `CaptionLabels`) take them from the application.

**Tests next to code.** Pure rules (corners, scheme derivation, focus reveal
math, keyboard destinations) have unit tests; layout and motion behavior use
GPUI's `TestAppContext` with `debug_selector` bounds.
