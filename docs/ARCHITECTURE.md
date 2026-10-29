# Architecture

## Foundation

The target foundation is [Longbridge GPUI Kit](https://github.com/longbridge/gpui-kit).
Its `gpui-kit` facade exposes GPUI, `gpui_kit::component` (the `gpui-component`
crate), `gpui_kit::base`, platform entry points and assets. Prefer existing
Component behavior and presentation hooks, with thin Material adapters for the
reference-specific appearance and motion. Use Base directly only for an
identified Component presentation gap; do not build a parallel generic toolkit.

The galleries use the Kit facade for application initialization and window
creation. The Material-to-Component theme bridge runs before Root preparation.
Existing Material controls and regression fixtures still import GPUI/Base
directly, resolving to the same patched packages as Kit. Symbols and individual
control migration remain pending. Existing compatibility fixes must remain until
the same behavior is verified through the replacement dependency graph.
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

**Application lifecycle.** Galleries use `gpui_kit::application`, `init` and
`open_window`; Kit owns the single Root and Component window extensions.
Capture settings are applied before the initial theme projection and Root
creation. The frame collector starts before opening the window; its metadata
snapshot is taken afterwards, once the actual viewport and rem are established.

**Theme ownership.** The explicit bridge projects 17 generic Material roles,
the current Material body font and text scale into Component colors/render tokens
and its Base projection. Initialize it before Root's first render. Palette and
scale events advance it before drawing; a single pending callback continues an
active theme animation. Gallery render consumes the prepared values and never
updates Component Theme. Full M3 roles, reference-specific blending, motion,
spacing and elevation remain Material-owned. Unmanaged fields retain Component
mode-configuration behavior. This is currently a single-gallery-window policy,
not independent per-window themes.

**Icons.** Reuse the sibling Symbols repository's `MaterialSymbols` asset source,
Kit `IconNamed` implementations and icon conversion. Retain only Material sizing,
axis selection or API compatibility that the shared implementation cannot supply.
Do not duplicate its font parser or SVG generation. Keep the dependency direction
acyclic and preserve all styles, axes and supplemental variants.

**One GPUI.** During migration, align Kit and Symbols to the same source and
resolved revision; a Git Base and registry Base with equal version numbers are
still different package identities. Retarget necessary patches to the selected
source rather than silently dropping them. Component and assets are Git `main`
dependencies pinned by Cargo.lock to the audited `bb85882` revision. The Kit
facade is patched to the complete, unchanged implementation from that revision
in `vendor/gpui-kit`. Its normalized manifest disables implicit WASM threading;
`multithreaded-web` explicitly restores that capability and its toolchain/runtime
requirements. Native platform defaults are preserved. Source hashes and the
reproducible manifest normalization are recorded alongside the facade.
The temporary Kit asset fallback embeds desktop icons but requests
`/assets/icons/*.svg` on WASM. The Gallery web packaging does not yet supply
that fallback catalog; existing Material icons remain embedded. Component icon
coverage is unverified until the pending Symbols integration replaces it.
The vendored Base contains that
revision's source plus the compatibility patches; both crates.io and Git Base
references resolve to this one path package. GPUI remains exactly `=0.3.10`.
The workspace patches GPUI Base from `vendor/gpui-base` for CSS transition
reversal, per-transition reduced-motion policy, toggle focus policy, and busy
accessibility state on Button/Toggle. Its normalized manifest and
license retain its source provenance in `vendor/gpui-base/SOURCE.json`. These root Cargo patches apply
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
