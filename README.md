# gpui-kit-material

Material 3 / **M3 Expressive** tokens, motion and components for
[Longbridge GPUI Kit](https://github.com/longbridge/gpui-kit).

Components are Material adapters over Kit's `gpui-component` controls
(`gpui_kit::component`) and Base parts: Kit owns behavior (input, focus,
accessibility, overlays, windows), this crate owns Material visuals and motion.
Icons come from
[gpui-material-symbols](https://github.com/mizorewww/gpui-material-symbols).

> **Status: pre-release (0.1).** Almost every component is implemented and
> tested, but none is stable: no component has passed independent conformance
> review, and none meets the [production standard](docs/PRODUCTION.md) yet.
> [docs/COVERAGE.md](docs/COVERAGE.md) lists the state and gaps per component,
> [docs/ROADMAP.md](docs/ROADMAP.md) what remains, and
> [conformance/md3.json](conformance/md3.json) the review record.

| Crate | Purpose |
| --- | --- |
| [`gpui-kit-material-color`](crates/color) | Color roles, palettes, reference themes, scheme derivation. No GPUI dependency. |
| [`gpui-kit-material`](crates/material) | Theme runtime, tokens, motion, icons and components. |
| [`gpui-kit-material-gallery-web`](crates/gallery-web) | WASM entry point for the shared gallery. |

## What's included

- **Color and theme**: semantic `ColorRole`s, all 20 md3.io reference themes
  (light/dark × standard/medium/high contrast, 52 reference roles each, with the kit's
  10 application roles and 14 tint-family roles: 76 in all), 10 legacy
  palettes, animated theme transitions and a 100–200% text scale. Dynamic
  seed/HCT schemes are ported from material-color-utilities; image-derived seeds
  are not supported.
- **Tokens**: shape, the 15 type roles and their emphasized styles, layout
  metrics, elevation, state layers.
- **Motion**: Expressive (default) and Standard schemes, spatial/effects
  curves, entrances, retargetable values, per-transition reduced motion.
- **Components**:

| Group | Components |
| --- | --- |
| Actions | `MaterialButton` (5 variants, 5 sizes, round/square, toggle, loading), `IconButton`, `ButtonGroup`, `ConnectedButtonGroup`, `SplitButton`, `Fab`, `ExtendedFab`, `FabMenu` |
| Selection | `MaterialCheckbox`, `MaterialSwitch`, `MaterialRadio`/`MaterialRadioGroup`, `AssistChip`/`FilterChip`/`InputChip`/`SuggestionChip`, `ChipSet`, `MaterialSlider` |
| Text input | `TextField`, `ExposedDropdown` (select), `SearchField`/`SearchView`, `DatePicker`, `TimePicker` |
| Navigation | `NavigationRail`, `NavigationBar`, `NavigationDrawer`, `NavigationSuite`, `MaterialTabs`, `MaterialMenu`, `MaterialContextMenu`, `TopAppBar`, `BottomAppBar`, `Toolbar` (docked and floating) |
| Containment | `MaterialCard`, `ListItem`, `MaterialCarousel`, `MaterialDivider`, `MaterialDialog`, `MaterialBottomSheet`, `MaterialSideSheet` |
| Communication | `Badge`, `Avatar`, `LinearProgress`/`CircularProgress`, `Stepper`, `LoadingIndicator`, `Snackbar`, `MaterialTooltip`, `PullToRefresh` |
| Layout | `PaneScaffold`, `FeedLayout` |
| Application | `TitleBar`, `CaptionButtons`, tiles, choice tiles, status chips and notices (from course2md, not M3 components) |

## Crates

| Crate | Published | Holds |
| --- | --- | --- |
| [`gpui-kit-material-color`](crates/color) | Dry run in CI | Color roles, palettes, reference themes and dynamic schemes, without GPUI. Features `serde`, `dynamic` (both default). |
| [`gpui-kit-material`](crates/material) | Git revision | The library: theme, tokens, motion, icons and the components below. |
| [`gpui-kit-material-gallery`](crates/gallery) | Never | The shared Gallery, its fonts and its dev-only profiler: a library and the `gallery` binary. |
| [`gpui-kit-material-gallery-web`](crates/gallery-web) | Never | The Gallery's browser entry point. |
| [`gpui-kit-material-bench`](crates/bench) | Never | `component_bench` and `engine_bench`. |

The library's `default` features enable every group, the Fast backend, embedded symbols
and `serde`. To drop code, turn them off and name what you use; a group brings what it
needs. `full` is every group, without the backend, symbols or `serde`.

```toml
gpui-kit-material = { git = "...", rev = "...", default-features = false, features = ["buttons", "selection", "gpui-fast", "embedded-symbols"] }
```

| Feature | Components | Needs |
| --- | --- | --- |
| `buttons` | `MaterialButton`, `IconButton`, `ButtonGroup`, `ConnectedButtonGroup`, `SplitButton`, `Fab`, `ExtendedFab`, `FabMenu` | `menus`, `progress` |
| `progress` | `LinearProgress`, `CircularProgress`, `Stepper` | |
| `loading` | `LoadingIndicator`, `PullToRefresh` | `progress`, `shapes` |
| `feedback` | `Badge`, `Snackbar`, `MaterialTooltip`, `MaterialCard`, `drop_target`, status chips, notices | |
| `selection` | `MaterialCheckbox`, `MaterialRadio`, `MaterialSwitch`, chips, `ChipSet`, `MaterialSlider`, tiles, choice tiles | |
| `inputs` | `TextField`, `SearchField`, `SearchView` | |
| `menus` | `MaterialMenu`, `MenuPopup`, `MaterialContextMenu`, and `ExposedDropdown` when `inputs` is on too | |
| `pickers` | `DatePicker`, `TimePicker` | `buttons`, `inputs`, `chrono` |
| `dialogs` | `MaterialDialog`, `MaterialBottomSheet`, `MaterialSideSheet` | |
| `containment` | `Avatar`, `MaterialCarousel`, `MaterialDivider`, `ListItem` | `selection` |
| `navigation` | `NavigationBar`, `NavigationRail`, `NavigationDrawer`, `NavigationSuite`, `MaterialTabs`, `TopAppBar`, `BottomAppBar`, `Toolbar` | `adaptive`, `dialogs`, `feedback` |
| `adaptive` | Window size classes, `PaneScaffold`, `FeedLayout` | |
| `window-chrome` | `TitleBar`, `CaptionButtons` | |
| `shapes` | `shapes`: `MaterialShape`, `Morph` | |
| `dynamic-color` | `color::dynamic`: HCT and seed schemes | |

The theme, tokens, motion, icons, scopes, focus ring, ripple and text helpers are always
there.

## Quick start

The library is distributed by Git revision, and a downstream workspace needs the
engine patches below until they land upstream. All of them must use one revision:

```toml
[dependencies]
gpui-kit = { git = "https://github.com/longbridge/gpui-kit", rev = "46bf6d498fa101a3b11cc64e6d8c20d28d17b720" }
gpui-kit-material = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "3df2920f17cecb765c96f23073fff5e4a2773e86" }

[patch.crates-io]
gpui-kit = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "3df2920f17cecb765c96f23073fff5e4a2773e86" }
gpui-base = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "3df2920f17cecb765c96f23073fff5e4a2773e86" }
gpui-pre = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "3df2920f17cecb765c96f23073fff5e4a2773e86" }
gpui-pre-apple = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "3df2920f17cecb765c96f23073fff5e4a2773e86" }
gpui-pre-web = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "3df2920f17cecb765c96f23073fff5e4a2773e86" }
gpui-fast = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "3df2920f17cecb765c96f23073fff5e4a2773e86" }
gpui-fast-apple = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "3df2920f17cecb765c96f23073fff5e4a2773e86" }
gpui-fast-web = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "3df2920f17cecb765c96f23073fff5e4a2773e86" }

[patch."https://github.com/longbridge/gpui-kit"]
gpui-kit = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "3df2920f17cecb765c96f23073fff5e4a2773e86" }
gpui-base = { git = "https://github.com/mizorewww/gpui-kit-material", rev = "3df2920f17cecb765c96f23073fff5e4a2773e86" }
```

> `[patch]` is read only from the top-level manifest, so it goes in your workspace root.
> [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md#consumer-patch-set) says why each patch
> exists. The rest of `vendor/` is dev-only frame observation for the Gallery. At every pushed
> commit, CI builds this block's template from Git at that commit
> (`scripts/git-consumer/Cargo.toml.in`, `scripts/check-git-consumer.sh`), and checks that this
> block differs from the template only in its revision (`scripts/check-package.sh`). The revision
> shown here is the last release's: CI does not build that revision again.

```rust
use gpui_kit_material::{color::themes::{Contrast, ThemeId, ThemeMode}, icons::Assets, theme};

gpui_kit::application()
    .with_assets(Assets::new())
    .run(|cx| {
        gpui_kit::init(cx);
        gpui_kit_material::init(cx);
        // Project the initial scheme into Kit's theme before the first Root render.
        let scheme = ThemeId::Baseline.scheme(ThemeMode::Light, Contrast::Standard);
        theme::init_component_theme(scheme, 1.0, cx);
        // Open windows with gpui_kit::open_window; Kit supplies the Root.
    });
```

Paint with semantic roles, never stored colors:

```rust
div().bg(color(ColorRole::SURFACE_CONTAINER)).text_color(color(ColorRole::ON_SURFACE))
```

Switch theme from an input event (it animates on the slow effects spring; keep
calling `advance_component_theme` from a next-frame callback while it returns
`true`, never from `render`), or switch the motion scheme:

```rust
let ocean = ThemeId::Ocean.scheme(ThemeMode::Dark, Contrast::High);
let running = theme::advance_component_theme(ocean, 1.0, window, cx);
gpui_kit_material::motion::set_motion_scheme(gpui_kit_material::motion::MotionScheme::Standard);
```

A controlled toggle button (the caller owns `selected`; the button never
stores it):

```rust
use gpui_kit_material::icons::IconName;

let view = cx.entity();
material_button("save", ButtonVariant::Tonal)
    .button_size(ButtonSize::Medium)
    .icon(IconName::Check)
    .toggle(true)
    .selected(self.saved)
    .on_change(move |selected, _click, _window, cx| {
        view.update(cx, |this, cx| { this.saved = selected; cx.notify(); });
    })
    .child(label)
```

Labels and accessible names always come from the caller. See the rustdoc
(`cargo doc --open -p gpui-kit-material`) for each component's contract.

## Gallery

One gallery module runs on desktop and in the browser. It follows md3.io's
component pages: navigation by component group, a live preview of the real
component with theme and light/dark switches, and a configuration panel for
every variant, size, state and content option. A **Specs** tab shows the Figma
kit's boards. Style pages cover color, typography, shape, motion and elevation.

```sh
cargo run -p gpui-kit-material-gallery      # desktop (GPUI Fast backend)
./scripts/build-gallery-web.sh --release    # browser, then:
python3 -m http.server 8080 --bind 127.0.0.1 --directory crates/gallery-web/www
```

The title bar shows FPS of newly drawn frames; **Settings → Vertical sync**
toggles Metal display sync on macOS, the surface's `PresentMode` on Linux and the
DXGI sync interval on Windows. Headless capture, profiling and fixture
tooling are described in [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## References

Where references disagree, priority is **Figma Material 3 Design Kit > md3.io >
m3e.kyralabs.dev**, per token family and per value; each value records its
source. See [Reference sources](docs/ARCHITECTURE.md#reference-sources).

## Performance targets

Measured on release builds from raw per-frame timings ([AGENTS.md](AGENTS.md)
has the full rules, [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md#targets) the
tooling). Acceptance evidence is still open.

| Target | Limit |
| --- | --- |
| Each comparable interaction (scrolling, hover, animations, typing, page switch, first load), native and WASM | main-thread frame time p95 and max below md3.io's best run in Chrome on the same machine |
| Idle window | draws nothing |

The profiler's finer budgets in DEVELOPMENT.md are diagnostics, not release gates.

GPU time is reported separately.

## Develop

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs these on macOS, Linux and Windows, plus both GPUI backends, the WASM gallery build, and
`cargo check` of the defaults, no features, `full` with and without Fast, and each feature group
alone (`scripts/check-features.sh`). Project rules for contributors and agents are in
[AGENTS.md](AGENTS.md).

| Document | Contents |
| --- | --- |
| [docs/PRODUCTION.md](docs/PRODUCTION.md) | What a component and a release must meet to be called stable |
| [docs/COVERAGE.md](docs/COVERAGE.md) | Every reference component, its status and what is missing |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Status tables and remaining work |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Layers, Kit reuse, reference sources, theme bridge, engine patches |
| [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) | Gallery capture, profiling, WASM, fixtures |
| [docs/EXTRACTION.md](docs/EXTRACTION.md) | Mapping from course2md, where the kit originated |
| [scripts/README.md](scripts/README.md) | Reference capture, Figma extraction and font scripts |

## License

MIT, except files ported from Apache-2.0 sources, which keep their notices.
Palette sources and licenses: [crates/color/PALETTES.md](crates/color/PALETTES.md).
Material Symbols assets are Apache-2.0, retained in the
[gpui-material-symbols](https://crates.io/crates/gpui-material-symbols) 0.1 crates
([repository](https://github.com/mizorewww/gpui-material-symbols)).
The gallery's fonts are SIL OFL 1.1, each beside its license in
[crates/gallery/fonts](crates/gallery/fonts): Roboto
and IBM Plex Sans, which it bundles, and Noto Sans Arabic, Noto Sans Hebrew, Noto Emoji
and Noto Color Emoji, kept for conformance captures only and not bundled. None of them
is part of the library's package. The published site lists every license at
[/licenses/](https://material.mizore.blog/licenses/).
