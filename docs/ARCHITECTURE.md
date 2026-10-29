# Architecture

## Foundation

The target foundation is [Longbridge GPUI Kit](https://github.com/longbridge/gpui-kit).
Its `gpui-kit` facade exposes GPUI, `gpui_kit::component` (the `gpui-component`
crate), `gpui_kit::base`, platform entry points and assets. Prefer existing
Component behavior and presentation hooks, with thin Material adapters for the
reference-specific appearance and motion. Use Base directly only for an
identified Component presentation gap; do not build a parallel generic toolkit.

The current implementation still imports the pinned GPUI/Base crates directly.
Facade, theme, icon and component migration is pending; this document does not
claim those runtime changes have landed. Existing compatibility fixes must remain
until the same behavior is verified through the replacement dependency graph.
The source-based reuse and removal conditions are recorded in
[conformance/reuse-audit.json](../conformance/reuse-audit.json).

## Layers

```
gpui-kit-material-color   palettes → Scheme (ColorRole → Color); pure Rust
        │
gpui-kit-material
  theme       painted Scheme for this frame; palette transitions; text scale
  tokens      shape · typography · layout · elevation · state layers
  motion      M3 springs; enter / page_enter / value / selection_value
  icons       Material Symbols via ../gpui-material-symbols and Kit integration
  components  Material adapters over Kit Component; Base for verified gaps
```

Each layer only depends on the ones above it in this list. Components never
read a palette's stored colors; they paint with `ColorRole`s through
`theme::color`.

## Decisions

**Reuse GPUI Kit.** Component is the preferred control layer; Base owns its
underlying behavior. Reuse their state, input normalization, keyboard navigation,
focus, accessibility, overlay and animation infrastructure. Material owns the
selected reference's visual roles, geometry and motion parameters. A similarly
named Kit component is not sufficient evidence that Material behavior or visuals
are preserved: inspect its rendering hooks and test the replacement before
deleting a local implementation.

**Application lifecycle.** Migrate galleries to `gpui_kit::application`, `init`
and `open_window`; Kit owns the single Root and Component window extensions.
Keep capture and complete-frame performance evidence around that lifecycle.

**Theme ownership.** Bridge Material roles to Component's semantic theme API and
Base tokens when state changes. Component's window preparation owns rem size, so
the bridge must also carry the Material text scale. Do not have competing rem
setters or unconditionally update the global theme during each render. Material
color derivation and reference-specific blending remain distinct from a generic
theme lifecycle.

**Icons.** Reuse the sibling Symbols repository's `MaterialSymbols` asset source,
Kit `IconNamed` implementations and icon conversion. Retain only Material sizing,
axis selection or API compatibility that the shared implementation cannot supply.
Do not duplicate its font parser or SVG generation. Keep the dependency direction
acyclic and preserve all styles, axes and supplemental variants.

**One GPUI.** During migration, align Kit and Symbols to the same source and
resolved revision; a Git Base and registry Base with equal version numbers are
still different package identities. Retarget necessary patches to the selected
source rather than silently dropping them. The current workspace pins the `gpui-pre` snapshot that the published
`gpui-base` requires (`=0.3.10` with `gpui-base =0.8.1`). Bump them together.
The workspace patches GPUI Base from `vendor/gpui-base` for CSS transition
reversal, per-transition reduced-motion policy, toggle focus policy, and busy
accessibility state on Button/Toggle. Its normalized manifest and
license retain the published package provenance. This root Cargo patch applies
only to builds in this workspace; downstream consumers need the same patch or
a Base release containing these fixes.
GPUI itself is patched from `vendor/gpui-pre` to expose paint-scoped passive raw
touch observation and hitbox extensions independent of layout geometry.
Extensions retain the original hitbox identity, clipping and occlusion rules.
This leaves gesture recognition, scrolling and activation
in GPUI while allowing Material to paint a ripple on touch-down and release it
on cancellation. The same workspace-only patch limitation applies.

The web backend is patched from the same `gpui-pre-web` 0.3.10 package to keep
CSS viewport dimensions separate from backing-store device pixels. When a
browser reports a device-pixel box inconsistent with its CSS size and DPR,
allocation falls back to CSS size × DPR while preserving logical layout.
The same workspace-only patch limitation applies.

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
