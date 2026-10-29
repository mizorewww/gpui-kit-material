# Architecture

## Foundation

The foundation is [Longbridge GPUI Kit](https://github.com/longbridge/gpui-kit).
Its `gpui-kit` facade exposes GPUI, `gpui_kit::component` (`gpui-component`),
`gpui_kit::base`, platform entry points and assets.

- **Kit/Component owns behavior**: state, input normalization, keyboard
  navigation, focus, accessibility, overlays, windows and animation
  infrastructure.
- **Material owns appearance**: the reference's color roles, geometry,
  typography and motion parameters, as thin adapters over Component.
- **Base directly** only for a source-verified Component presentation gap.

A similarly named Kit component is not proof of equivalent behavior or visuals:
inspect its rendering hooks and test the replacement before deleting a local
implementation. Reuse and removal conditions are recorded in
[conformance/reuse-audit.json](../conformance/reuse-audit.json).

## Layers

```
gpui-kit-material-color   palettes → Scheme (ColorRole → Color); pure Rust
        │
gpui-kit-material
  theme       painted Scheme for this frame; transitions; text scale; Kit bridge
  tokens      shape · typography · layout · elevation · state layers
  motion      Expressive/Standard schemes; enter / value / selection_value
  icons       Material Symbols (pinned Git revision) via its Kit integration
  components  Material adapters over Kit Component; Base for verified gaps
```

Each layer depends only on those above it. Components never read stored palette
colors; they paint `ColorRole`s through `theme::color`. Material, galleries and
fixtures import `gpui` through the Kit facade so engine types always match the
selected backend.

## Lifecycle and theme bridge

Galleries use `gpui_kit::application`, `init` and `open_window`; Kit owns the
single Root and Component window extensions. Capture settings apply before the
initial theme projection and Root creation.

`theme::init_component_theme` projects 17 generic Material roles, the body font
and text scale into Component colors/tokens (and its Base projection) before
Root's first render. Palette and scale events call
`theme::advance_component_theme` before drawing; one pending callback continues
an active animation. **Render never updates Component Theme.** Full M3 roles,
reference blending, motion, spacing and elevation remain Material-owned; unmanaged
fields keep Component's mode behavior. Currently one theme per process (single
gallery window), not per-window themes.

`theme::color` is context-free and thread-local, so helpers stay plain
functions; GPUI paints on one UI thread.

## Icons

The Symbols repository owns assets, font/SVG conversion, catalog and axes.
Material's `Icon` keeps a 20px default and size/color/rotation builders over
Component Icon. `IconName` aliases use Rounded, FILL 0, weight 400, grade 0,
optical size 24 (matching the primary reference's generated icon tokens at
`3b1e004`) and return `material-symbols/` paths. `SymbolIcon` conversions
preserve all axes, including selected FILL.

`init` precomputes 103 Kit default-icon translations; resolution is a hash
lookup plus a shared-path clone. GitHub, heart-off and star-off have no
equivalent; six approximate translations await visual review. Raw SVG/data
icons bypass the resolver. Cold-load and new-size/DPR raster costs must be
measured before adding a cache beyond GPUI's.

The gallery embeds Symbols on both targets (no HTTP fallback). Consumers that
serve Symbols at runtime can disable the `embedded-symbols` default feature. An
adjacent `../gpui-material-symbols` checkout is never substituted implicitly.

## Engine patches

One GPUI and one Base implementation are shared across Material, Kit and
Symbols. Git and registry packages with equal versions are still different
identities, so the root `[patch]` tables redirect every source to `vendor/`.
**These patches apply only inside this workspace**; downstream workspaces need
the same patches until fixes land upstream. A patched Zed checkout overriding
`gpui-pre` overrides it for this kit too.

| Vendored | Source | Why |
| --- | --- | --- |
| `gpui-kit` | Git `main` at audited `bb85882`, unchanged source | Normalized manifest disables implicit WASM threading (`multithreaded-web` restores it). Hashes in `SOURCE.md`/`SOURCE.json`. |
| `gpui-base` | 0.8.1 + patches (`SOURCE.json`) | CSS transition reversal, per-transition reduced motion, toggle focus policy, busy accessibility state on Button/Toggle, `TransitionPlayback`; `Hitbox.additional_bounds` follow-up in `text_selection.rs`. |
| `gpui-pre` (GPUI `=0.3.10`) | + patches (`*-patch.md`; no `SOURCE.json` yet, and profiler, line-layout, window and SVG changes are undocumented) | Paint-scoped passive raw touch observation; hitbox extensions independent of layout (keeping identity, clipping, occlusion); animation visibility. |
| `gpui-pre-apple` | 0.3.10 + patches (`VENDORED.md`) | Detailed CoreText layout (glyph positions, clusters, carets); `draw_with_status` submission reporting with a per-draw autorelease pool. |
| `gpui-pre-web` | 0.3.10 + patch (no provenance note yet) | Keeps CSS viewport size separate from backing-store pixels; falls back to CSS size × DPR on inconsistent reports. |
| `gpui-fast`, `-apple`, `-web` | 0.1.12 + patches (`MATERIAL-MIGRATION.json`) | Full CPU scopes, repeat playback, detailed text layout, font registration, raw touch and expanded hit targets through retained replay; synchronous refresh on Material paint changes; explicit single-threaded web defaults. |
| `gpui-pre-macos`, `gpui-fast-macos` | + patches (`MATERIAL-PATCHES.md`, `SOURCE.json`) | Vsync control and submission statistics (below). |

Touch patches leave gesture recognition, scrolling and activation in GPUI while
letting Material start a ripple on touch-down and release it on cancellation.

**Backend selection.** Material and gallery-web expose `gpui-fast`, forwarding
to Kit's backend feature; it is on by default. Material's `gpui` alias points to
Kit, so upstream and Fast types cannot mix. Cargo still compiles upstream
packages additively; they do not define the facade's public types. Disable
default features to compare against upstream.

## Frame observation

Submission statistics count attempts and, separately, successfully submitted
**new** draw generations; FPS uses only the latter. Backends without a success
signal report "unavailable"; nothing claims GPU completion. The gallery samples
counters every 500 ms in its own view (included in profiles, disabled for static
captures).

`VsyncStatus` reports capability and actual state. The macOS backends forward
it to Metal's display-sync property for existing and new surfaces; unsupported
backends return an error rather than a pretend value. With vsync off, real frame
demand drives the coalescing main-thread source; the display link remains only
as a safety throttle, so clean windows gain no render loop.

## Virtualized animation state

Kit/Base keyed transitions live only while their element is mounted. For
virtualized rows, owners retain playback handles instead:

- Base `TransitionPlayback` keeps a transition sampler (with interruption and
  reversal) across unmounts without scheduling frames.
- `ButtonTransitionPlayback` keeps a button's color, shape, elevation and
  state-layer channels, combining that sampler with Material's motion curve. A
  completed offscreen transition settles before the next target's curve is
  chosen. Owners sync controlled inputs for unmounted buttons after layout
  determines which rows participated.
- `CircularProgressPlayback` keeps only the indicator clock and reduced-motion
  stop state.

Handles do not keep pointer/focus state or rendered subtrees; a new handle starts
a new logical playback. Visible elements request frames for active transitions.

## Conventions

- **Rems follow text.** 14px = 1rem at 100%. Geometry sharing a ruler with text
  uses `typography::rem(px)`; platform geometry (caption buttons, hairlines,
  tile gaps) stays in px.
- **No locale.** Components ship no strings; accessible names (e.g.
  `CaptionLabels`) come from the application.
- **Tests next to code.** Pure rules (corners, schemes, focus-reveal math,
  keyboard destinations) have unit tests; layout and motion use GPUI's
  `TestAppContext` with `debug_selector` bounds.
