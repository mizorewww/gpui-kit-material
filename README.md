# gpui-kit-material

Material 3 / **M3 Expressive** tokens, motion and components for
[Longbridge GPUI Kit](https://github.com/longbridge/gpui-kit).

Components are thin Material adapters over Kit's `gpui-component` controls
(`gpui_kit::component`): Kit owns behavior (input, focus, accessibility,
overlays, windows), this crate owns Material visuals and motion. Icons come from
[gpui-material-symbols](https://github.com/mizorewww/gpui-material-symbols).

> **Status: pre-release (0.1).** The APIs below work and are tested, but no
> component has passed visual/motion conformance review yet. Per-component
> status lives in [`conformance/md3.json`](conformance/md3.json); the plan is in
> [docs/ROADMAP.md](docs/ROADMAP.md).

| Crate | Purpose |
| --- | --- |
| [`gpui-kit-material-color`](crates/color) | Color roles, palettes, reference themes, scheme derivation. No GPUI dependency. |
| [`gpui-kit-material`](crates/material) | Theme runtime, tokens, motion, icons and components. |
| [`gpui-kit-material-gallery-web`](crates/gallery-web) | WASM entry point for the shared gallery. |

## What's included

- **Color & theme** — semantic `ColorRole`s, all 20 md3.io reference themes
  (light/dark × standard/medium/high contrast, 52 roles each), 10 legacy
  palettes (Paper/Ink, Nord, Tokyo Night, Catppuccin), animated theme
  transitions and a 100–200% text scale. Dynamic seed/HCT schemes are pending.
- **Tokens** — shape, type scale (15 roles), layout metrics, elevation, state
  layers.
- **Motion** — Expressive (default) and Standard schemes, spatial/effects
  curves, entrances, retargetable values, per-transition reduced motion.
- **Components** — `MaterialButton` (5 variants × 5 sizes × round/square,
  toggle, loading, ripple, shape morph), `ConnectedButtonGroup`, dividers,
  linear/circular progress, switch, slider, search field, side sheet, snackbar,
  focus ring/reveal and a custom title bar with caption buttons. Tiles, choice
  tiles, status badges, chips, notices and cards come from course2md and do not
  yet follow M3 geometry.

## Quick start

```toml
[dependencies]
gpui-kit = { git = "https://github.com/longbridge/gpui-kit", branch = "main" }
gpui-kit-material = { git = "https://github.com/mizorewww/gpui-kit-material" }
```

> The workspace patches GPUI, GPUI Base and GPUI Fast (see `[patch]` in
> [Cargo.toml](Cargo.toml) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md#engine-patches)).
> Downstream workspaces need the same patches until they land upstream.

```rust
use gpui_kit_material::{icons::Assets, prelude::*, theme};

gpui_kit::application()
    .with_assets(Assets::new())
    .run(|cx| {
        gpui_kit::init(cx);
        gpui_kit_material::init(cx);
        // Project the initial scheme into Kit's theme before the first Root render.
        theme::init_component_theme(initial_scheme, 1.0, cx);
        // Open windows with gpui_kit::open_window; Kit supplies the Root.
    });
```

Paint with semantic roles, never stored colors:

```rust
div().bg(color(ColorRole::SURFACE_CONTAINER)).text_color(color(ColorRole::ON_SURFACE))
```

Switch theme (animated on the slow effects spring) or motion scheme:

```rust
theme::apply_scheme(ThemeId::Ocean.scheme(ThemeMode::Dark, Contrast::High), window, cx);
gpui_kit_material::motion::apply_scheme(gpui_kit_material::motion::MotionScheme::Standard);
```

Advance theme animation from input events and pre-frame callbacks with
`theme::advance_component_theme`, never from `render`. A controlled toggle
button (the caller owns `selected`; the button never stores it):

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

Labels and accessible names come from the caller (the one remaining exception,
`CaptionLabels::default()`, is slated for removal). See the rustdoc (`cargo doc --open -p gpui-kit-material`) for the full
contracts of each component.

## Gallery

One gallery module runs on desktop and in the browser.

```sh
cargo run -p gpui-kit-material --example gallery          # desktop (GPUI Fast backend)
./scripts/build-gallery-web.sh --release                   # browser, then:
python3 -m http.server 8080 --bind 127.0.0.1 --directory crates/gallery-web/www
```

The title bar shows FPS of newly drawn frames; **Settings → Vertical sync**
toggles Metal display sync on macOS. Headless capture, profiling and fixture
tooling are described in [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Develop

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs these on macOS, Linux and Windows, plus both GPUI backends and the WASM
gallery build. Project rules for contributors and agents — Kit reuse,
conformance acceptance and the performance gates (every gallery CPU frame
≤ 2 ms, scrolling ≥ 500 FPS) — are in [AGENTS.md](AGENTS.md).

| Document | Contents |
| --- | --- |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Layers, Kit reuse, theme bridge, engine patches |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Component plan and acceptance criteria |
| [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) | Gallery capture, profiling, WASM, fixtures |
| [docs/EXTRACTION.md](docs/EXTRACTION.md) | Mapping from course2md, where the kit originated |

## License

MIT. Palette sources and licenses: [crates/color/PALETTES.md](crates/color/PALETTES.md).
Material Symbols assets are Apache-2.0, retained in the pinned
[Symbols repository](https://github.com/mizorewww/gpui-material-symbols/tree/593aa15a46076f3e2870c23c0e4fd79f94f105bd).
Roboto and IBM Plex Sans gallery fonts are SIL OFL, bundled with their licenses.
