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
- **Motion**: M3 standard springs, entrances, retargetable values, disclosure.
- **Components**: connected button group (radio or tab semantics, roving
  focus), tiles and tile groups, choice tiles with radio indicator, tinted
  icons, badges, chips, notices, cards, linear progress and spinner, text
  roles with accessibility semantics, focus ring and focus reveal, and
  Windows/Linux caption buttons.

## Gallery

```sh
cargo run -p gpui-kit-material --example gallery
```

## Material Symbols reference example

A native recreation of the Google Fonts icon browser, with the full symbol
catalog, axis controls, SVG/PNG export and GPUI code output. Clone the sibling
`gpui-material-symbols` repository, then run:

```sh
cargo run -p gpui-kit-material --example material_symbols
```

See [setup, component details and captures](docs/MATERIAL_SYMBOLS.md).

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
