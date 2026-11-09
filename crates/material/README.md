# gpui-kit-material

Material 3 / M3 Expressive tokens, motion and components for
[Longbridge GPUI Kit](https://github.com/longbridge/gpui-kit).

Components are Material adapters over Kit's `gpui-component` controls
(`gpui_kit::component`) and Base parts: Kit owns behavior (input, focus,
accessibility, overlays, windows), this crate owns Material visuals and motion.
Icons come from [gpui-material-symbols](https://crates.io/crates/gpui-material-symbols),
color from [gpui-kit-material-color](https://crates.io/crates/gpui-kit-material-color).

> **Pre-release (0.1).** Almost every component is implemented and tested, but
> none is stable yet. The repository's
> [coverage table](https://github.com/mizorewww/gpui-kit-material/blob/main/docs/COVERAGE.md)
> lists the state and gaps per component.

## Install

```toml
[dependencies]
gpui-kit = "0.9.1"
gpui-kit-material = "0.1"
```

Use the `gpui-kit` release this crate depends on (0.9.1 or a later 0.9), so that Kit,
Base, GPUI and the Symbols crate resolve to one implementation; no `[patch]` is
needed. Minimum Rust: **1.98**.

In the browser, published Kit 0.9.1 builds its Web platform multithreaded, which needs the
nightly toolchain its [WebAssembly guide](https://github.com/longbridge/gpui-kit/blob/main/website/docs/webassembly.md)
names (`wasm_thread` does not build on stable).

```rust,no_run
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

Paint with semantic `ColorRole`s through `theme::color`; labels and accessible
names always come from the caller. The rustdoc describes each component's contract.

## Features

`default` enables every component group, the GPUI Fast backend, embedded symbols
and `serde`. `full` is every group, without the backend, symbols or `serde`. To drop
code, disable the defaults and name what you use; a group brings what it needs.

| Feature | Contents |
| --- | --- |
| `buttons` | `MaterialButton`, `IconButton`, `ButtonGroup`, `ConnectedButtonGroup`, `SplitButton`, `Fab`, `ExtendedFab`, `FabMenu` |
| `progress` | `LinearProgress`, `CircularProgress`, `Stepper` |
| `loading` | `LoadingIndicator`, `PullToRefresh` |
| `feedback` | `Badge`, `Snackbar`, `MaterialTooltip`, `MaterialCard`, `drop_target`, status chips, notices |
| `selection` | `MaterialCheckbox`, `MaterialRadio`, `MaterialSwitch`, chips, `ChipSet`, `MaterialSlider`, tiles, choice tiles |
| `inputs` | `TextField`, `SearchField`, `SearchView` |
| `menus` | `MaterialMenu`, `MenuPopup`, `MaterialContextMenu`, and `ExposedDropdown` with `inputs` |
| `pickers` | `DatePicker`, `TimePicker` |
| `dialogs` | `MaterialDialog`, `MaterialBottomSheet`, `MaterialSideSheet` |
| `containment` | `Avatar`, `MaterialCarousel`, `MaterialDivider`, `ListItem` |
| `navigation` | `NavigationBar`, `NavigationRail`, `NavigationDrawer`, `NavigationSuite`, `MaterialTabs`, `TopAppBar`, `BottomAppBar`, `Toolbar` |
| `adaptive` | Window size classes, `PaneScaffold`, `FeedLayout` |
| `window-chrome` | `TitleBar`, `CaptionButtons` |
| `shapes` | `MaterialShape`, `Morph` |
| `dynamic-color` | `color::dynamic`: HCT and seed schemes |
| `gpui-fast` | Kit's GPUI Fast backend; off selects upstream GPUI |
| `embedded-symbols` | Material Symbols fonts and SVGs compiled into the binary |
| `serde` | Serializable theme preferences |

The theme, tokens, motion, icons, scopes, focus ring, ripple and text helpers are
always there.

## License

MIT, except the rules ported from androidx, which are Apache-2.0
(`src/components/LICENSE-androidx`, `src/shapes/LICENSE-androidx`); the motion
curves keep their MIT notice in `src/motion/LICENSE-material`. Material Symbols
assets are Apache-2.0, in the gpui-material-symbols crates.
