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

## Reference sources

Token values follow the priority **Figma M3 Design Kit > md3.io >
m3e.kyralabs.dev**, applied per token family and per value: the first
reference that defines a value wins, and the value's rustdoc and fixture
record that source. Outranked fixtures stay as untested secondary evidence.
`reference_policy.token_sources` in
[conformance/md3.json](../conformance/md3.json) is the per-family record:

| Family | Source | Fixture |
| --- | --- | --- |
| Color | Figma kit `Schemes/*` variables for the baseline theme (6 modes); md3.io for the other 19 themes and info/success/warning | `crates/color/tests/fixtures/figma-baseline-colors.json`, `md3-colors.json` |
| Typography | Figma kit text styles and Typescale/Font theme variables (Roboto) | `crates/material/tests/fixtures/figma-typography.json` |
| Shape, elevation, state layers | Figma kit Shape variables, elevation effect styles, State Layers variables | `crates/material/tests/fixtures/figma-sys-tokens.json` |
| Motion | The kit has none: md3.io sampled curves for Expressive; m3e springs for Standard (absent from md3.io) | `conformance/fixtures/md3-expressive-browser.json`, `m3e-motion.json` |
| Button geometry and disabled colors | Figma kit Buttons page; timing from md3.io captures | `crates/material/tests/fixtures/figma-button.json`, `conformance/reference/button-*.json` |

## Lifecycle and theme bridge

Galleries use `gpui_kit::application`, `init` and `open_window`; Kit owns the
single Root and Component window extensions. Capture settings apply before the
initial theme projection and Root creation.

`theme::init_component_theme` projects 19 generic Material roles (including the
caret and text selection colors of Kit's inputs), the body font
and text scale into Component colors/tokens (and its Base projection) before
Root's first render. Palette and scale events call
`theme::advance_component_theme` before drawing; one pending callback continues
an active animation. **Render never updates Component Theme.** Full M3 roles,
reference blending, motion, spacing and elevation remain Material-owned; unmanaged
fields keep Component's mode behavior.

`scope::Scope` gives a subtree or a window's content its own scheme and direction.
It is an element that re-enters around each drawing phase of its subtree, while
`theme::color` answers from it and Kit's theme globals hold its projection.
Material's overlays wrap what they defer in `current_scope`, which brings it back in.
Swapping the globals notifies their observers, none of which redraws, but GPUI Fast
stamps them as changed, so while a scheme scope is on screen a draw rebuilds retained
subtrees that read Kit's theme. Its limits are listed in `src/scope.rs`.

`theme::color` is context-free and thread-local, so helpers stay plain
functions; GPUI paints on one UI thread.

## Layout direction

Direction is inherited: the application's, overridden by a window's, overridden by
a `scope::Scope`. Layout mirrors in the engine after Taffy runs, so a component's
flex order, padding and absolute offsets flip without restyling, and its own
asymmetric corners and borders flip when it paints. Components handle only what
layout cannot reach: painted paths (`LinearProgress`, the slider track), pointer
math, and arrow keys, which run after the draw and so use the direction captured
while rendering. Directional icons flip by name (`icons.rs`). A clock dial and window
controls stay left to right.

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
**These patches apply only inside this workspace**; a downstream workspace needs
the consumer patch set below until fixes land upstream. A patched Zed checkout
overriding `gpui-pre` overrides it for this kit too.

### Consumer patch set

A consumer patches `gpui-kit`, `gpui-base`, `gpui-pre`, `gpui-fast`,
`gpui-pre-apple`, `gpui-fast-apple`, `gpui-pre-web` and `gpui-fast-web`, exactly as
[crates/consumer-check/Cargo.toml](../crates/consumer-check/Cargo.toml) does; CI
builds that crate on every target and both backends, beside upstream's `gpui-fps`.
Its engine has upstream's `FrameEvent`, `PlatformWindow` and frame path.

| Consumer crate | Why a consumer needs it |
| --- | --- |
| `gpui-kit` | Normalized manifest that keeps WASM single-threaded by default. |
| `gpui-base` | Slider, dialog and toolbar corrections, transition behavior, and the input reporting its caret. |
| `gpui-pre`, `gpui-fast` | Layout direction, letter spacing, the lazy tab-stop index, repeat playback and visibility, and the other engine rules in the table below. |
| `gpui-pre-apple`, `gpui-fast-apple` | Non-Latin and app-private fallback fonts. |
| `gpui-pre-web`, `gpui-fast-web` | The browser's IME panel opens at the caret; upstream's `update_ime_position` does nothing. Only matters on `wasm32`. |

Everything else in `vendor/` is a **dev-only overlay**, marked in the table below,
for the Gallery's FPS display
and vsync switch, `engine_bench` and `scripts/profile-gallery.py`: frame statistics,
vsync control, CPU scopes, presentation timing and GPU time. `gpui-pre` and
`gpui-fast` gate them behind the `frame-observation` Cargo feature, and only the
overlay's backend manifests (`gpui-*-macos`, `-linux`, `-windows`) turn it on. The
web crates are not overlays, so their manifests leave it off and offer a
`frame-observation` feature of their own, which only `crates/gallery-web` selects
for its browser frame counters. No library code may use them; `crates/consumer-check` fails to build if it does.
Inside this workspace the feature is therefore always on, so upstream's `gpui-fps`
does not compile here (`E0004`, `FrameEvent::Complete`) and is not used here.

| Vendored | Source | Why |
| --- | --- | --- |
| `gpui-kit` | Git `main` at audited `bb85882`, unchanged source | Normalized manifest disables implicit WASM threading (`multithreaded-web` restores it). Hashes in `SOURCE.md`/`SOURCE.json`. |
| `gpui-base` | Git `bb85882` (`crates/base`) + patches (`SOURCE.json`, `motion-patch.md`) | CSS transition reversal, per-transition reduced motion, `TransitionPlayback`; vertical `Toolbar` axis; slider step snapping from min, assistive Increment/Decrement that move only a range's end, `Slider::track_focus`, right-to-left pointer math; `Dialog::accessibility_label`; the input reports its caret to the platform when it moves (`ime-anchor-patch.md`). |
| `gpui-pre` (GPUI `=0.3.10`) | crates.io + patches (`SOURCE.json`, `*-patch.md`) | Layout direction; letter spacing; animation visibility and repeat playback; complete CPU frame scopes, presentation wait, GPU time and submission outcome in the profiler; submission statistics, vsync control and throttle retry; `em`/`ch` metrics for fonts without `m`; lazy tab-stop index (`tab-stop-index-patch.md`); `Entity::update_for_render` (`render-lease-patch.md`); overflowing spaces hang instead of starting a wrapped row (`hanging-space-patch.md`); test-harness repairs; `Window::focused_input_selection_bounds` (`focused-input-bounds-patch.md`). |
| `gpui-pre-apple` | crates.io + patches (`SOURCE.json`, `VENDORED.md`) | Non-Latin and app-private fallback fonts; `draw_with_status` with a per-draw autorelease pool, drawable-wait and GPU timestamps. |
| `gpui-pre-web` | crates.io + patches (`SOURCE.json`, `MATERIAL-PATCHES.md`) | Moves the hidden IME input to the caret, where a browser opens its candidate window (consumers need this); successful submission count, behind the crate's own `frame-observation` feature. |
| `gpui-fast`, `-apple`, `-web` | 0.1.12 + patches (`MATERIAL-MIGRATION.json`, per-file `local_changes`) | The GPUI, Apple and web patches above carried through Fast retained/scroll replay and composition; explicit single-threaded web defaults. |
| `gpui-pre-macos`, `gpui-fast-macos` (dev-only) | crates.io + patches (`SOURCE.json`, `MATERIAL-PATCHES.md`) | Vsync control, demand waker, periodic-source report and submission statistics (below). |
| `gpui-pre-linux`, `gpui-fast-linux`, `gpui-pre-wgpu`, `gpui-fast-wgpu`, `wgpu` (dev-only) | crates.io + patches (`SOURCE.json`, `MATERIAL-PATCHES.md`) | Submission statistics from the present status `wgpu` otherwise discards, and vsync control through `PresentMode` (below). |
| `gpui-pre-windows`, `gpui-fast-windows` (dev-only) | crates.io + patches (`SOURCE.json`, `MATERIAL-PATCHES.md`) | Submission statistics counting only `S_OK` presents, and vsync control through the DXGI sync interval (below). |

**Replaced by Material code.** No engine patch remains for these:

- **Touch press.** A button and its ripple observe the unclaimed `TouchDragEvent` GPUI
  offers at touch start. GPUI has no public signal for a system cancel of a still
  finger, so that press ends at a 5 s hold limit.
- **48dp target of XS and S buttons.** The Base root is the target, negative margins
  keep it out of layout, and the painted surface is its child
  (`components/button/hit_target.rs`). A caller's `.hover()`, `.focus()` or `.active()`
  style lands on the root, not the surface. A button with `overflow_hidden`, or a height
  only layout can resolve, has no extended target.
- **Toggle focus policy.** Base Button with `aria_toggled`.
- **Error caret.** The field lays out Kit's Input with its caret in the theme.
- **Busy state.** Material marks its own control root, which only `accesskit_windows`
  reads.

**Audit results for transition extras.** These stay, because removing them changes
reference parity or the budget: reversal, per-transition reduced motion,
`TransitionPlayback`, `RepeatAnimationPlayback` and `repeat_when_visible`. No Examples
page uses a playback handle. The Gallery's are in its buttons and loading pages.

**Lazy tab stops.** `TabStopMap` records insertions during paint and builds the
order and focus lookup on the first query after a change. Upstream's per-insertion
`SumTree` update scales with the focusable elements a frame replays, which a
composited scrolling list replays every frame, in Kit's lists as in Material's.
Query results are upstream's, including gpui-kit#3612, which no Material widget
reaches (`crates/material/tests/tab_stops.rs`).

**Backend selection.** Material and gallery-web expose `gpui-fast`, forwarding
to Kit's backend feature; it is on by default. Material's `gpui` alias points to
Kit, so upstream and Fast types cannot mix. Cargo still compiles upstream
packages additively; they do not define the facade's public types. Disable
default features to compare against upstream.

## Frame observation

Dev-only: see [Consumer patch set](#consumer-patch-set).

Submission statistics count attempts and, separately, successfully submitted
**new** draw generations; FPS uses only the latter. Backends without a success
signal report "unavailable"; nothing claims GPU completion. The gallery samples
counters every 500 ms in its own view (included in profiles, disabled for static
captures).

`VsyncStatus` reports capability and actual state. The macOS backends forward
it to Metal's display-sync property for existing and new surfaces; Linux to the
surface's `wgpu::PresentMode` (X11 starts on `Fifo`, Wayland on `Mailbox`); Windows
to the DXGI sync interval (upstream's 0, so it starts `Disabled`). Unsupported
backends return an error rather than a pretend value. On Linux and Windows frame
requests stay paced by the display (X11 refresh timer, Wayland frame callback,
`DwmFlush`), so vsync off cannot exceed its refresh rate there and the 500 FPS gate
is not measurable until those backends gain a demand-driven frame source. With vsync off, real frame
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
