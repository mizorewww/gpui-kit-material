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
  icons       Material Symbols (crates.io 0.1) via its Kit integration
  components  Material adapters over Kit Component; Base for verified gaps
```

Each layer depends only on those above it. Components never read stored palette
colors; they paint `ColorRole`s through `theme::color`. Material, galleries and
fixtures import `gpui` through the Kit facade so engine types always match the
selected backend.

## Crates and features

```
gpui-kit-material-color          no GPUI; independently usable
        ▲
gpui-kit-material                the library; nothing below is in its package
        ▲
gpui-kit-material-gallery        publish = false: Gallery, fonts, frame-profiler, instruction-bench
        ▲            ▲
        │            gpui-kit-material-gallery-web   publish = false: browser entry, wasm-bindgen
gpui-kit-material-bench          publish = false: component_bench, engine_bench (on the library only)
```

`crates/consumer-check` stands outside the workspace, as a downstream manifest does.
Dev-only engine features are selected only by the Gallery and benches; the library has
none ([Consumer patch set](#consumer-patch-set)). The Gallery's native build keeps
`gpui-kit/test-support`, whose `render_to_image` draws `--capture`.

The library's components are in Cargo feature groups, listed in the
[README](../README.md#crates). A component using one in another group means its group
declares that group in `Cargo.toml`, and `scripts/check-features.sh`, which checks each
group alone in CI, fails when it does not. `components/interaction` (focus ring, ripple),
`focus`, `ripple`, `roving`, `preview`, `labeled` and `text` are shared by the groups and
have no gate. Tests that span groups need `full`, and so run only with it.

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

`theme::init_component_theme` projects 25 generic Material roles (including the
caret and text selection colors of Kit's inputs, Base's scrollbars, and `TextView`'s
links and table heads), the body font and text scale into Component colors/tokens
(and its Base projection) before Root's first render. Palette and scale events call
`theme::advance_component_theme` before drawing; one pending callback continues
an active animation. **Render never updates Component Theme.** Full M3 roles,
reference blending, motion, spacing and elevation remain Material-owned; unmanaged
fields keep Component's mode behavior.

`scope::Scope` gives a subtree or a window's content its own scheme and direction.
It is an element that re-enters around each drawing phase of its subtree, while
`theme::color` answers from it. Material's overlays wrap what they defer in
`current_scope`, which brings it back in. The view holding the scope reads a marker
global that drawing the scope stamps, so only that view rebuilds, and a view nested in
it rebuilds through it, in the scope. Its limits are listed in `src/scope.rs`.

A scope does not touch Kit's theme globals (`cx.theme()` and Base's), so a Kit control
drawn inside one follows the application's theme. Giving it the scope's would mean
changing a global for part of a draw, and every public way to change a global notifies
its observers and, under GPUI Fast, rebuilds every view that read it. The engine-only
quiet swap that did this could not go upstream, and this regression is the accepted
price of dropping it. Material's text inputs, which draw through Kit's `Input`, give it
their scoped caret and selection colors themselves (`CaretReport::ink`).

`theme::color` is context-free and thread-local, so helpers stay plain
functions; GPUI paints on one UI thread.

## Layout direction

Layout is left to right only. GPUI has no layout direction, and like Zed this
library does not support right-to-left layout.

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

Symbols comes from crates.io (`gpui-material-symbols` 0.1), whose fonts and SVG pack
are in one data crate each. `embedded-symbols` enables its `embed-assets` (all four);
without it nothing is embedded. The desktop Gallery embeds Symbols. The browser Gallery
embeds none: `scripts/build-gallery-web.sh` copies the Outlined and Rounded fonts out of
their data crates, the page downloads them (`www/main.js`) and hands them over with
`provide_symbol_asset`, drawing a placeholder for each icon until its font arrives or
fails. Consumers that serve Symbols at runtime can disable the `embedded-symbols`
default feature. An adjacent `../gpui-material-symbols` checkout is never substituted
implicitly.

## Engine patches

One GPUI and one Base implementation are shared across Material, Kit and
Symbols. Git and registry packages with equal versions are still different
identities, so the root `[patch]` tables redirect every source to `vendor/`,
including crates.io `gpui-kit`, which Symbols depends on, so that its `IconNamed`
is Material's.
**These patches apply only inside this workspace**; a downstream workspace needs
the consumer patch set below until fixes land upstream. A patched Zed checkout
overriding `gpui-pre` overrides it for this kit too.

### Consumer patch set

A consumer patches `gpui-kit` (from both crates.io, for Symbols, and Kit's Git), `gpui-base`, `gpui-pre`, `gpui-fast`,
`gpui-pre-apple`, `gpui-fast-apple`, `gpui-pre-web` and `gpui-fast-web`, as the
[quick start](../README.md#quick-start) shows, from this repository at the same revision
as the dependency. CI builds that manifest from Git at the pushed commit
(`scripts/check-git-consumer.sh`) on every target, and
[crates/consumer-check/Cargo.toml](../crates/consumer-check/Cargo.toml), the same patches
from this checkout, on every target and both backends, beside upstream's `gpui-fps`.
Its engine has upstream's `FrameEvent`, `PlatformWindow` and frame path.

"To build" marks a patch whose API Material calls, so a consumer cannot build without
it; the others change behavior only.

| Consumer crate | To build | Why a consumer needs it |
| --- | --- | --- |
| `gpui-kit` | yes, from crates.io | Normalized manifest that keeps WASM single-threaded by default. Symbols depends on crates.io `gpui-kit` 0.9; without the crates.io patch a second Kit builds and Symbols' `IconNamed` is not Material's. |
| `gpui-base` | yes | `TransitionPlayback`, which Material calls (longbridge/gpui-kit#3841, open). Until it merges and is released, a consumer cannot build without this patch. Everything else in it equals Kit `main`, including per-transition reduced motion (#3839) and assistive-technology slider steps (#3840). |
| `gpui-pre`, `gpui-fast` | no | Behavior only. Fast is gpui-fast `main`, which has the lazy tab-stop index, `em` metrics for fonts without `m`, `record_entities_accessed` and the wheel and hover fixes (not yet released); both keep hanging overflow spaces, which upstream lacks. `gpui-pre` keeps the first two as local patches. Material calls no engine API that upstream lacks. |
| `gpui-pre-apple`, `gpui-fast-apple` | no | Non-Latin and app-private fallback fonts (upstream on gpui-fast `main`, a local patch in `gpui-pre-apple`); `sbix` color fonts drawn as color bitmaps (local in both). |
| `gpui-pre-web`, `gpui-fast-web` | no | The browser's IME panel opens at the caret, and a key or edit the IME owns stays whole. Upstream on gpui-fast `main` (unreleased); a local patch in `gpui-pre-web`. Only matters on `wasm32`. |

Everything else in `vendor/` is a **dev-only overlay**, marked in the table below,
for the Gallery's FPS display
and vsync switch, `crates/bench`'s `engine_bench` and `scripts/profile-gallery.py`: frame statistics,
vsync control, CPU scopes, presentation timing and GPU time, and the render lease the
Gallery's rows build through. `gpui-pre` and
`gpui-fast` gate them behind the `frame-observation` Cargo feature, and only the
overlay's backend manifests (`gpui-*-macos`, `-linux`, `-windows`) turn it on. The
web crates are not overlays, so their manifests leave it off and offer a
`frame-observation` feature of their own, which only `crates/gallery` selects
for its browser frame counters. No library code may use them; `crates/consumer-check` fails to build if it does.
Inside this workspace the feature is therefore always on, so upstream's `gpui-fps`
does not compile here (`E0004`, `FrameEvent::Complete`) and is not used here.

| Vendored | Source | Why |
| --- | --- | --- |
| `gpui-kit` | Git at Kit `main` `8dfdd49d`, unchanged source | Normalized manifest disables implicit WASM threading (`multithreaded-web` restores it). Hashes in `SOURCE.md`/`SOURCE.json`. |
| `gpui-base` | Git `8dfdd49d` (Kit `main`, `crates/base`) + one patch (`SOURCE.json`, `motion-patch.md`) | `TransitionPlayback` (`src/motion.rs`, `src/lib.rs`), equal to the head of the open longbridge/gpui-kit#3841. Merged upstream and no longer patched: per-transition reduced motion (#3839), assistive Increment/Decrement with `SliderState::accessibility_step` (#3840), CSS transition reversal (#3820), vertical `Toolbar` axis (#3821), slider step snapping (#3822), block surfaces in `TextView` (#3830), slider min above the default max (#3823), `Slider::track_focus` (#3824), a disabled thumb's flag (#3825), `Dialog::accessibility_label` (#3826), an input a parent moves notified only while focused (#3828), `TimeField::separator` (#3829), the input's layout published at the end of prepaint (#3831). |
| `gpui-pre` (GPUI `=0.3.11`) | crates.io + patches (`SOURCE.json`, `*-patch.md`) | Complete CPU frame scopes, presentation wait, GPU time and submission outcome in the profiler; submission statistics, vsync control and throttle retry; `em`/`ch` metrics for fonts without `m`; lazy tab-stop index (`tab-stop-index-patch.md`); `Entity::update_for_render`, dev-only behind `frame-observation` (`render-lease-patch.md`); overflowing spaces hang instead of starting a wrapped row (`hanging-space-patch.md`); test-harness repairs. |
| `gpui-pre-apple` | crates.io + patches (`SOURCE.json`, `VENDORED.md`) | Non-Latin and app-private fallback fonts; `draw_with_status` (upstream's draw, into a `CAMetalDisplayLink` drawable or `nextDrawable`) with a per-draw autorelease pool, drawable-wait and GPU timestamps. |
| `gpui-pre-web` | crates.io + patches (`SOURCE.json`, `MATERIAL-PATCHES.md`) | Moves the hidden IME input to the caret when a composition starts and on `update_ime_position`, where a browser opens its candidate window and it takes no clicks; keeps a composition's keys from GPUI and a replaced emoji whole (consumers need these); successful submission count, behind the crate's own `frame-observation` feature. |
| `gpui-fast`, `-apple`, `-web` | gpui-fast `main` `bbdbc42d` + patches (`SOURCE.json`, `MATERIAL-PATCHES.md`) | gpui-fast's repository names its crates `gpui`, `gpui_web`, ...; only its release step gives them the `gpui-fast-*` names Kit depends on, so a Git `[patch]` cannot select them. `scripts/vendor-gpui-fast.py` runs that step at a commit (version kept at 0.1.13; `gpui-fast-macros` and `-platform` are unchanged since 0.1.13 and come from crates.io) and records each file that differs. Material's Fast PRs are all upstream: wheel at its limit (#104), hover freeze for listeners (#105), released entities in a spliced view (#106), web IME (#108, #109, #110), macOS fonts (#111, #112), lazy tab stops (#113), `record_entities_accessed` (#114), `em` metrics (#115). Local: the dev-only overlay below, test-harness repairs, hanging overflow spaces (`src/text_system/line_layout.rs`, `line_wrapper.rs`) and `sbix` color fonts (`gpui-fast-apple/src/text_system.rs`). |
| `gpui-pre-macos`, `gpui-fast-macos` (dev-only) | crates.io (`gpui-pre-*`, `wgpu`) or gpui-fast `main` (`gpui-fast-*`) + patches (`SOURCE.json`, `MATERIAL-PATCHES.md`) | Vsync control, demand waker, periodic-source report and submission statistics (below). With vsync off, a window upstream paces with a `CAMetalDisplayLink` (macOS 14+) is paced by the `CVDisplayLink` path instead, so demand frames can run between ticks. |
| `gpui-pre-linux`, `gpui-fast-linux`, `gpui-pre-wgpu`, `gpui-fast-wgpu`, `wgpu` (dev-only) | crates.io (`gpui-pre-*`, `wgpu`) or gpui-fast `main` (`gpui-fast-*`) + patches (`SOURCE.json`, `MATERIAL-PATCHES.md`) | Submission statistics from the present status `wgpu` otherwise discards, and vsync control through `PresentMode` (below). |
| `gpui-pre-windows`, `gpui-fast-windows` (dev-only) | crates.io (`gpui-pre-*`, `wgpu`) or gpui-fast `main` (`gpui-fast-*`) + patches (`SOURCE.json`, `MATERIAL-PATCHES.md`) | Submission statistics counting only `S_OK` presents, and vsync control through the DXGI sync interval (below). |

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
`TransitionPlayback`, and Material's `motion::RepeatAnimationPlayback` and
`motion::RepeatWhenVisible`. No Examples
page uses a playback handle. The Gallery's are in its buttons and loading pages.

**Lazy tab stops.** `TabStopMap` records insertions during paint and builds the
order and focus lookup on the first query after a change. Upstream's per-insertion
`SumTree` update scales with the focusable elements a frame replays, which a
composited scrolling list replays every frame, in Kit's lists as in Material's.
Query results are upstream's, including gpui-kit#3612, which no Material widget
reaches (`crates/material/tests/tab_stops.rs`).

**Backend selection.** Material, the Gallery, gallery-web and the benches expose
`gpui-fast`, forwarding to Kit's backend feature; it is on by default. Material's `gpui` alias points to
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
`DwmFlush`), so vsync off cannot exceed its refresh rate there until those backends
gain a demand-driven frame source. With vsync off, real frame
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
- `motion::RepeatAnimationPlayback` keeps only a repeat's start time, for the
  indeterminate linear and circular indicators, the loading indicator and the
  spinners. `motion::RepeatWhenVisible` samples it (or its own clock) and asks for
  the next frame in paint only when its bounds meet the content mask and the
  viewport, on public GPUI API, so a clipped indicator leaves the window idle.

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
