# gpui-kit-material

Material 3 design tokens, motion and components for [GPUI](https://www.gpui.rs),
built on the headless behavior of [GPUI Base](https://crates.io/crates/gpui-base).

The kit started as the shared design primitives of the
[course2md](https://github.com/mizorewww/course2md) desktop app. It is being
grown into a complete **Material 3 Expressive** implementation; see
[docs/ROADMAP.md](docs/ROADMAP.md).

| Crate | Purpose |
| --- | --- |
| [`gpui-kit-material-color`](crates/color) | Color roles, built-in palettes, scheme derivation. No GPUI dependency. |
| [`gpui-kit-material`](crates/material) | Theme runtime, tokens, motion, icons and components. |

## Use

```rust
use gpui_kit_material::{icons::Assets, prelude::*, theme};

gpui_platform::application()
    .with_assets(Assets::with(MyAssets)) // the kit's glyphs, then yours
    .run(|cx| {
        gpui_base::init(cx);
        gpui_kit_material::init(cx);
        // open windows…
    });

// At the start of each root render:
theme::apply(&preferences, window, cx);
```

Components paint with semantic roles (`color(ColorRole::SURFACE)`), so a palette
change animates the whole window on the slow effects spring.

## What's included

- **Theme**: 10 built-in palettes (Paper/Ink, Nord, Tokyo Night, Catppuccin),
  separate light/dark choices, animated palette transitions, a text scale.
- **Tokens**: shape (connected tile corners), type scale, layout metrics,
  elevation, state layers.
- **Motion**: md3.io Expressive sampled curves (spatial and effects, fast/default/slow),
  selectable Standard springs, entrances, retargetable values, disclosure.
- **Components**: connected button group (radio or tab semantics, roving
  focus), tiles and tile groups, choice tiles with radio indicator, tinted
  icons, badges, chips, notices, cards, linear progress and spinner, text
  roles with accessibility semantics, focus ring and focus reveal, and
  Windows/Linux caption buttons.

## Gallery

```sh
cargo run -p gpui-kit-material --example gallery
```

The browser entrypoint uses the same Rust gallery module and components:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
./scripts/build-gallery-web.sh
python3 -m http.server 8080 --bind 127.0.0.1 --directory crates/gallery-web/www
```

Open `http://localhost:8080`. The single-threaded browser build runs without
cross-origin isolation headers. It embeds IBM Plex Sans (SIL OFL, bundled beside
the font) for the web platform's default text measurements. Use
`./scripts/build-gallery-web.sh --release` for an optimized build. Generated
bindings stay out of Git.

Component alignment is tracked in [`conformance/md3.json`](conformance/md3.json).
An available gallery example does not imply visual or motion conformance;
each component needs independent review and desktop/browser evidence.

The gallery includes six motion previews and a Standard/Expressive selector.
Applications can select their scheme at the start of each root render, before
applying the theme:

```rust
gpui_kit_material::motion::apply_scheme(
    gpui_kit_material::motion::MotionScheme::Expressive,
);
```

Expressive is the default. A value transition retains its curve until it settles,
including reversals, so changing schemes cannot jump an in-flight value. A mounted
entrance retains its initial scheme; a new palette target uses the current scheme.
Reduced motion still paints the final state immediately.

## Material Symbols consumer example

The Google Fonts icon browser belongs to
[gpui-material-symbols](https://github.com/mizorewww/gpui-material-symbols).
It consumes this kit's Material components; this repository retains the reusable
controls and their gallery demonstrations.

## Develop

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## License

MIT. Palette sources and their licenses are listed in
[crates/color/PALETTES.md](crates/color/PALETTES.md); Lucide glyphs are ISC
([LICENSE-lucide](crates/material/assets/icons/LICENSE-lucide)).
