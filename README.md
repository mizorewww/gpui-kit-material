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
  plus all 20 md3.io reference themes in light/dark and standard/medium/high
  contrast, animated palette transitions, and a text scale.
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

Both gallery targets use the reference site's Roboto 400/500 font files, with
pinned source hashes and the SIL OFL in
`crates/material/examples/gallery/fonts/roboto`. Rebuild the static fonts with:

```sh
uv run --with 'fonttools[woff]==4.60.1' python scripts/build-reference-fonts.py
```

The Typography section displays all 15 reference roles, and the text scale
selector supports 100%, 125%, 150%, and 200%. The page title uses headline-small
(24px / 32px at 100%). Reference letter-spacing values are stored in the tokens;
GPUI tracking support and full component typography alignment remain pending.

The shared gallery mounts GPUI Base's `Root` for Tab/Shift+Tab traversal and
focus containment. Focus reveal is attached to individual controls and button
options, so keyboard navigation brings the next control into view without
pinning the page during manual scrolling.

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
Reduced-motion behavior is set per transition: spatial value transitions finish
immediately, while supported non-spatial effects retain their fades.

## Buttons

`material_button(id, variant)` now returns `MaterialButton`, a presentation
wrapper over GPUI Base's button. It supports Filled, Elevated, Tonal, Outlined,
and Text variants, five `ButtonSize`s through `.button_size(...)`, and Round or
Square through `.shape(...)`. The default variant is Filled. Existing callers
with an explicit `gpui_base::Button` return annotation must update that type.
Activation, focus binding, keyboard input and disabled behavior remain in Base.
Use `.icon(...)` for a decorative SVG icon, `.icon_position(...)` for logical
Leading/Trailing placement, and `.direction(ButtonDirection::Rtl)` for mirrored
layout. Slot dimensions follow button size, and disabled icons use the disabled
content role even when the supplied icon has a custom color.
Button text defaults to its type role’s font family (Roboto); caller
`.font_family(...)` overrides remain supported. Applications must register the
font to obtain reference glyph metrics; otherwise GPUI uses its font fallback.

The gallery includes all 50 variant/size/shape combinations, disabled samples,
controlled toggles, and loading examples with no icon or either icon placement.
Press and release morph the radius with FastSpatial, including interrupted
transitions. Pointer ripples are enabled by default; `.ripple(false)` selects the
pressed state layer. XS/S retain their 32/40px visual heights while their 48px
interaction targets obey the same clipping and occlusion as the Base control.
Keyboard focus draws a Secondary outline 3px wide with a 2px gap outside the
button; pointer focus and disabled buttons do not draw it.

Use `.toggle(true).selected(selected).on_change(...)` for controlled selection.
The callback receives `(next_selected, click_event, window, app)`; update the
application's state and request a render. `.toggle_on_click(false)` suppresses
selection requests while preserving `on_click` for container-owned selection.

`.progress(visible, accessible_label)` shows a circular indicator while keeping
activation available. `.loading(loading, accessible_label)` also marks the
button busy and disables activation. Both are controlled: visible button text
comes from `.child(...)`, and `.disabled(...)` remains application-owned, so
ending loading does not clear an independent disabled state. Keep the progress
or loading configuration present with a false flag while hidden to preserve its
animation phase. Indicator labels are always supplied by the application.

These features have focused rule, input and motion tests and independent code
reviews. Full desktop/browser visual and animation conformance remains pending,
including typography tracking and the complete theme/scale/state matrix. See
[`conformance/md3.json`](conformance/md3.json) for the scoped evidence.

## Reference themes

The gallery starts with the Material baseline theme. Applications can select
one of the 20 reference presets through the same animated theme runtime:

```rust
use gpui_kit_material::{prelude::*, theme};

theme::apply_scheme(
    ThemeId::Ocean.scheme(ThemeMode::Dark, Contrast::High),
    window,
    cx,
);
```

The 120 preset configurations preserve the reference CSS's 52 colors, including
fixed, inverse, surface-container, error, and status roles. The kit's selection,
find-highlight, and status-container extensions are derived separately. Existing
Paper/Nord/etc. palettes remain adapters from their original source colors.
Dynamic seed/HCT generation is still pending.

Qualified legacy names such as `ColorRole::ACCENT` remain aliases of canonical
roles such as `ColorRole::PRIMARY`, sharing the same override storage. Code that
imported the old enum variants with `use ColorRole::{ACCENT}` or `use ColorRole::*`
must use qualified names or import the canonical variants instead.

To recapture the preset fixtures from a clean checkout of the pinned reference:

```sh
python3 scripts/capture-reference-colors.py /path/to/floor-material /tmp/md3-colors.json
```

This requires `npx`, `git`, and `agent-browser`. The script checks the reference
revision, compiles its Sass, and reads the browser's resolved CSS properties.
The source revision and capture environment are recorded in the fixture; the
reference implementation's MIT notice ships with the tables.

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
