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
Material controls, galleries and regression fixtures resolve their `gpui` import
through the Kit facade; Base remains a direct dependency for the reviewed
presentation gaps. This keeps their public engine types aligned when selecting
a Kit backend. Symbols provide the shared icon source; individual
control migration remains pending. Existing compatibility fixes must remain until
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
  icons       Material Symbols via its pinned Git revision and Kit integration
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

**Icons.** The Symbols repository owns the asset source, font/SVG
conversion, catalog and axes. Material's `Icon` only preserves its 20px default
and existing size/color/rotation builders over Component Icon. The semantic
`IconName` aliases use Rounded, FILL=0, weight 400, grade 0 and optical size 24,
matching the primary reference site source (`apps/site/scripts/icons.js` and
generated icon tokens at `3b1e004`). They return canonical `material-symbols/` paths; old literal
`material-kit/icons/` paths are removed. Configured `SymbolIcon` conversions
preserve all axes and styles, including selected FILL.

Material initialization precomputes 103 Kit default path translations. The
resolver only performs a hash lookup and clones the shared path during drawing.
GitHub, heart-off and star-off have no equivalent and remain caller-owned; six
approximate translations remain explicitly pending visual review. Raw SVG/data
icons bypass the resolver. This is not blanket icon-conformance acceptance.
Symbols SVG generation currently relies on GPUI's raster cache; cold-load and
new-size/DPR costs still require measurement before adding another cache.

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
The Gallery uses embedded Material Symbols on both targets, without Kit's
HTTP icon fallback. Material's `embedded-symbols` default feature can be disabled
by consumers that provide Symbols assets at runtime. The Symbols repository's
Web consumer does this to retain its incremental downloads. Symbols is pinned
to a published Git commit containing the required features and embedded assets;
the default build never substitutes an adjacent checkout. Engine compatibility
patches are all inside this repository. Downstream workspaces need the same
engine patch policy until the compatibility fixes are upstream.
The vendored Base contains that
revision's source plus the compatibility patches; both crates.io and Git Base
references resolve to this one path package. GPUI remains exactly `=0.3.10`.
The workspace patches GPUI Base from `vendor/gpui-base` for CSS transition
reversal, per-transition reduced-motion policy, toggle focus policy, and busy
accessibility state on Button/Toggle. Its normalized manifest and
license retain its source provenance in `vendor/gpui-base/SOURCE.json`. These root Cargo patches apply
only to builds in this workspace; downstream consumers need the same patch or
a Base release containing these fixes.

**GPUI Fast.** Material and Gallery-web expose `gpui-fast`, forwarding to Kit's
official backend feature. Kit selects Fast for its public core, platform, Base,
Component and assets types. Material's `gpui` crate alias points to Kit, so it
cannot accidentally combine direct upstream types with the Fast facade.
Cargo's additive dependencies still compile upstream packages; these do not
define the selected facade's public types. Fast is enabled by default for
Material and both galleries; the upstream backend remains available with
default features disabled for comparison.

Fast 0.1.12 core, Apple and Web packages are patched in `vendor/gpui-fast*`.
Their `MATERIAL-MIGRATION.json` files preserve published archive and source
hashes. Compatibility patches cover full CPU scopes, repeat playback, detailed
text layout, font registration, raw touch observation and expanded hit targets.
Retained subtree and scroll-layer replay carry touch observers and independently
clipped hit targets. Font generation invalidates retained views, layers and text
measurements, while successful registration keeps unrelated App instances
independent. Material paint changes synchronously refresh the window before
drawing, including when Component also queues a global refresh. No-op theme
updates keep retained content reusable. The Web dependency edges retain explicit
single-threaded defaults; the package's multithreaded opt-in is unchanged.
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

**Frame observation and presentation settings.** Window submission statistics
retain attempts for diagnostics and separately count successfully submitted new
draw generations. FPS uses only the latter: no-drawable failures and repeated
presentation of the same draw do not inflate it. Backends without a success
signal report unavailable. These counters do not claim GPU completion. The Gallery samples these counters in a separate view every
500 ms; performance profiling includes this view, while explicit static captures
disable it and record that choice. `VsyncStatus` reports backend capability and
actual state. The vendored macOS backends forward settings to Metal's display
synchronization property, including existing and newly created Fast composition
surfaces. Unsupported backends return an error instead of storing a pretend
setting. With synchronization disabled, macOS forwards real frame demand into
the existing coalescing main-thread source. The display link remains a fallback
for safety throttling; clean windows do not acquire an extra rendering loop.

An application on a patched Zed checkout overrides `gpui-pre` for the whole
graph, including this kit.

**Thread-local paint.** `theme::color` is context-free so helpers stay plain
functions. GPUI renders on one UI thread; the paint is per thread.

**Rems follow text.** 14px = 1rem at 100%. Geometry that shares a ruler with
text is in rems (`typography::rem(px)`); platform geometry (caption buttons,
hairlines, tile gaps) stays in px.

**Virtualized animation state.** Kit/Base's keyed transitions live only while
their elements remain mounted. Base's `TransitionPlayback` exposes the same
transition sampler to callers that need to retain a channel across unmounts;
it preserves interruption and reversal state without scheduling frames. The
keyed API continues to own scheduling for its mounted elements. Owners forward
actual target changes and reduced-motion observations, while visible elements
request frames for active transitions. Material's `CircularProgressPlayback`
similarly retains only the indicator clock and its reduced-motion stop state.
These handles do not preserve transient pointer or focus state, nor rendered
subtrees. A new handle explicitly starts a new logical playback.

**No locale.** The kit ships no translated strings. Components that need
accessible names (e.g. `CaptionLabels`) take them from the application.

**Tests next to code.** Pure rules (corners, scheme derivation, focus reveal
math, keyboard destinations) have unit tests; layout and motion behavior use
GPUI's `TestAppContext` with `debug_selector` bounds.
