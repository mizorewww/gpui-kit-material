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
  (light/dark × standard/medium/high contrast, 52 roles each), 10 legacy
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
| Navigation | `NavigationRail`, `NavigationBar`, `NavigationDrawer`, `NavigationSuite`, `MaterialTabs`, `MaterialMenu`, `TopAppBar`, `BottomAppBar`, `Toolbar` (docked and floating) |
| Containment | `MaterialCard`, `ListItem`, `MaterialCarousel`, `MaterialDivider`, `MaterialDialog`, `MaterialBottomSheet`, `MaterialSideSheet` |
| Communication | `Badge`, `Avatar`, `LinearProgress`/`CircularProgress`, `LoadingIndicator`, `Snackbar`, `MaterialTooltip`, `PullToRefresh` |
| Layout | `PaneScaffold`, `FeedLayout` |
| Application | `TitleBar`, `CaptionButtons`, tiles, choice tiles, status chips and notices (from course2md, not M3 components) |

## Quick start

```toml
[dependencies]
gpui-kit = { git = "https://github.com/longbridge/gpui-kit", branch = "main" }
gpui-kit-material = { git = "https://github.com/mizorewww/gpui-kit-material" }
```

> The workspace patches GPUI, GPUI Base and GPUI Fast (see `[patch]` in
> [Cargo.toml](Cargo.toml) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md#engine-patches)).
> Downstream workspaces need only the patches listed in
> [crates/consumer-check/Cargo.toml](crates/consumer-check/Cargo.toml) until they
> land upstream; the rest of `vendor/` is dev-only frame observation for the Gallery.

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
cargo run -p gpui-kit-material --example gallery          # desktop (GPUI Fast backend)
./scripts/build-gallery-web.sh --release                   # browser, then:
python3 -m http.server 8080 --bind 127.0.0.1 --directory crates/gallery-web/www
```

The title bar shows FPS of newly drawn frames; **Settings → Vertical sync**
toggles Metal display sync on macOS. Headless capture, profiling and fixture
tooling are described in [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## References

Where references disagree, priority is **Figma Material 3 Design Kit > md3.io >
m3e.kyralabs.dev**, per token family and per value; each value records its
source. See [Reference sources](docs/ARCHITECTURE.md#reference-sources).

## Performance targets

Measured on release builds from raw per-frame timings ([AGENTS.md](AGENTS.md)
has the full rules, [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md#targets) the
tooling). These are targets; acceptance evidence is still open.

| Target | Limit |
| --- | --- |
| Steady-state main-thread CPU | p95 ≤ 1 ms, max ≤ 2 ms (WASM ×2) |
| Main-thread drawable wait | none |
| Idle window | draws nothing |
| Forced redraw of an unchanged page | ≤ 0.3 ms |
| First frame / page switch | ≤ 16 ms CPU / ≤ 8 ms |
| GPU time | p95 ≤ 2 ms, reported separately |
| Scrolling, vsync off | ≥ 500 new frames/s |
| Scrolling, vsync on | no missed refreshes at 120 Hz |

## Develop

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs these on macOS, Linux and Windows, plus both GPUI backends and the WASM
gallery build. Project rules for contributors and agents are in
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
Material Symbols assets are Apache-2.0, retained in the pinned
[Symbols repository](https://github.com/mizorewww/gpui-material-symbols/tree/593aa15a46076f3e2870c23c0e4fd79f94f105bd).
Roboto and IBM Plex Sans gallery fonts are SIL OFL, bundled with their licenses.
