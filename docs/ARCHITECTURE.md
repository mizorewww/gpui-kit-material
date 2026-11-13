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
gpui-kit-material-bench          publish = false: component_bench (on the library only)
```

`crates/consumer-check` stands outside the workspace, as a downstream manifest does.
Only the Gallery and benches select engine features beyond the backend, all of them
upstream's: `profiler` (the Gallery's `frame-profiler`) and `test-support`; the library
selects none ([Consumer patch set](#consumer-patch-set)). The Gallery's native build keeps
`gpui-kit/test-support`, whose `render_to_image` draws `--capture`.

The library's components are in Cargo feature groups, listed in the
[crate README](../crates/material/README.md#features). A component using one in another group means its group
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
Symbols. The library depends on released crates only: `gpui-kit` and `gpui-base` 0.9.1,
which name `gpui-fast` 0.1.14 and `gpui-pre` 0.3.12; Symbols depends on crates.io
`gpui-kit` 0.9 too, so a consumer resolves one Kit and its `IconNamed` is Material's.
The root `[patch.crates-io]` table redirects a few of those crates to `vendor/` for this
workspace's Gallery. **These patches apply only inside this workspace**, and none is
needed to build the library (`scripts/check-registry-consumer.sh` builds the package
without them). A patched Zed checkout overriding `gpui-pre` overrides it for this kit
too.

### Consumer patch set

A downstream workspace that wants the Gallery's engine behavior patches `gpui-kit`,
`gpui-pre`, `gpui-pre-apple`, `gpui-pre-web` and `gpui-fast` from this repository at
one revision:

```toml
[dependencies]
gpui-kit = "0.9.1"
gpui-kit-material = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "e8bb5ee18cb0520622a656a9d232d039d4211a5b" }

[patch.crates-io]
gpui-kit = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "e8bb5ee18cb0520622a656a9d232d039d4211a5b" }
gpui-pre = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "e8bb5ee18cb0520622a656a9d232d039d4211a5b" }
gpui-pre-apple = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "e8bb5ee18cb0520622a656a9d232d039d4211a5b" }
gpui-pre-web = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "e8bb5ee18cb0520622a656a9d232d039d4211a5b" }
gpui-fast = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "e8bb5ee18cb0520622a656a9d232d039d4211a5b" }
```

`[patch]` is read only from the top-level manifest, so it goes in your workspace root.
The revision shown is `gpui-kit-material` 0.1.2's source; the engine patches in it are
unchanged since they moved to Kit 0.9.1 (`98c72b8`). CI does not build that revision
again, and `scripts/check-package.sh` checks that this block differs from the template CI
builds (`scripts/git-consumer/Cargo.toml.in`) only in its revision.

CI builds that
manifest from Git at the pushed commit (`scripts/check-git-consumer.sh`) on every target,
and [crates/consumer-check/Cargo.toml](../crates/consumer-check/Cargo.toml), the same
patches from this checkout, on every target and both backends, beside upstream's
`gpui-fps`. Every patch changes behavior only: Material calls no engine API that the
releases lack.

| Consumer crate | Why a consumer may want it |
| --- | --- |
| `gpui-kit` | Kit 0.9.1's unchanged source with a normalized manifest that keeps WASM single-threaded by default (`multithreaded-web` restores upstream's workers); published Kit selects its Web platforms' default `multithreaded` feature. |
| `gpui-pre`, `gpui-fast` | Spaces that overflow a wrapped row hang instead of starting the next one (upstream lacks this), and a `measure_all` list measures its off-screen items over later frames after a width change, so a long `TextView` resizes without laying out every block per frame. On `gpui-fast`, a wheel over nested scroll containers scrolls only the innermost one it moves, for the whole gesture, instead of every container under the pointer. `gpui-pre` also keeps the lazy tab-stop index and `em` metrics for fonts without `m`, which gpui-fast 0.1.14 has upstream. |
| `gpui-pre-apple` | Non-Latin and app-private fallback fonts, and `sbix` color fonts drawn as color bitmaps, on the upstream backend; gpui-fast 0.1.14 has them upstream. |
| `gpui-pre-web` | The browser's IME panel opens at the caret, and a key or edit the IME owns stays whole, on the upstream backend; gpui-fast 0.1.14 has them upstream. Only matters on `wasm32`. |

The other gpui-fast crates (`-apple` apart, below, `-macos`, `-linux`, `-windows`,
`-wgpu`, `-web`, `-platform`, `-macros`) and the upstream backend's platform crates come
from crates.io unchanged. No vendored crate adds or changes engine API: gpui-fast takes
no API additions, so the dev-only frame observation the Gallery's FPS counter, vsync
switch and old profiler used was dropped. `crates/consumer-check` builds the library
beside upstream's `gpui-fps`; so does this workspace.

| Vendored | Source | Why |
| --- | --- | --- |
| `gpui-kit` | crates.io 0.9.1 (tag `v0.9.1`, `39ab849a`), unchanged source | Normalized manifest disables implicit WASM threading (`multithreaded-web` restores it). Hashes in `SOURCE.md`/`SOURCE.json`. |
| `gpui-pre` (GPUI `=0.3.12`) | crates.io + patches (`SOURCE.json`, `*-patch.md`) | `em`/`ch` metrics for fonts without `m` (`em-fallback-patch.md`); lazy tab-stop index (`tab-stop-index-patch.md`); overflowing spaces hang instead of starting a wrapped row (`hanging-space-patch.md`); a width change lays out a `measure_all` list's shown items first and the rest over later frames (`list-width-remeasure-patch.md`); test-harness repairs; `try_update` for the deprecated `fetch_update` in the profiler. |
| `gpui-pre-apple` | crates.io 0.3.12 + patches (`SOURCE.json`, `VENDORED.md`) | Non-Latin and app-private fallback fonts; `sbix` color fonts. |
| `gpui-pre-web` | crates.io 0.3.12 + patches (`SOURCE.json`, `MATERIAL-PATCHES.md`) | Moves the hidden IME input to the caret when a composition starts and on `update_ime_position`, where a browser opens its candidate window and it takes no clicks; keeps a composition's keys from GPUI and a replaced emoji whole. |
| `gpui-fast` | crates.io 0.1.14 + patches (`SOURCE.json`, `MATERIAL-PATCHES.md`), recorded by `scripts/vendor-gpui-fast.py --published 0.1.14` | Material's Fast PRs are all released in 0.1.14: wheel at its limit (#104), hover freeze for listeners (#105), released entities in a spliced view (#106), web IME (#108, #109, #110), macOS fonts (#111, #112), lazy tab stops (#113), `record_entities_accessed` (#114), `em` metrics (#115), `sbix` color fonts (#119). Local: hanging overflow spaces (`src/text_system/line_layout.rs`, `line_wrapper.rs`), a `measure_all` list's measurement after a width change (`src/elements/list.rs`), a wheel scrolls only the innermost nested scroll container it moves and stays with it for the gesture (`src/fast/layers/wheel_latch.rs`), test-harness repairs, `try_update` for the deprecated `fetch_update`. |
| `gpui-fast-apple` | crates.io 0.1.14 + a test (`SOURCE.json`, `MATERIAL-PATCHES.md`) | Workspace only, no runtime change: the `font_fallback` regression test with the Gallery's fonts, and a warning fix. Not in the consumer set. |

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
`TransitionState`, and Material's `motion::RepeatAnimationPlayback` and
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

## Rows built from an entity

A Gallery page's rows, and a playground's preview and configuration panel, need the
entity's `Context` for their listeners and keep what they build in it. Under GPUI
Fast, an `Entity::update` while the window draws counts as a write: every other
retained subtree that read the entity (the other rows, the panel, the Gallery around
a preview) would be built again on the next frame. So the entity is the view: it
implements `Render` for the part asked for, and the caller renders it in place with
upstream's `gpui::View::render`, whose update is drawing the view rather than writing
it (`crates/gallery/src/page.rs`, "How a row is built"). A notify while it builds
still marks it changed. Upstream GPUI has no retained subtrees and uses `update`.

## Virtualized animation state

Kit/Base keyed transitions live only while their element is mounted. For
virtualized rows, owners retain playback handles instead:

- Base `TransitionState` keeps a transition sampler (with interruption and
  reversal) across unmounts. It holds no view or window; sampling it while
  drawing requests the next frame while the transition runs.
- `ButtonTransitionPlayback` keeps a button's color, shape, elevation and
  state-layer channels, combining that sampler with Material's motion curve. A
  completed offscreen transition settles before the next target's curve is
  chosen. Owners sync controlled inputs for unmounted buttons after layout
  determines which rows participated, in the same frame (a list prepaint), so a
  forwarded transition requests one more frame and no more: the next sync with
  the same inputs samples nothing.
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
