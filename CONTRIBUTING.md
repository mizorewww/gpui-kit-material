# Contributing

Project rules for contributors and agents are in [AGENTS.md](AGENTS.md). Read it, and
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), before adding a component, a dependency
or an engine patch.

## Workspace

| Crate | Published | Holds |
| --- | --- | --- |
| [`gpui-kit-material-color`](crates/color) | crates.io | Color roles, palettes, reference themes and dynamic schemes, without GPUI. Features `serde`, `dynamic` (both default). |
| [`gpui-kit-material`](crates/material) | crates.io | The library: theme, tokens, motion, icons and components. |
| [`gpui-kit-material-gallery`](crates/gallery) | Never | The shared Gallery, its fonts and its dev-only profiler: a library and the `gallery` binary. |
| [`gpui-kit-material-gallery-web`](crates/gallery-web) | Never | The Gallery's browser entry point. |
| [`gpui-kit-material-bench`](crates/bench) | Never | `component_bench`. |

Components are Material adapters over Kit's `gpui-component` controls
(`gpui_kit::component`) and Base parts: Kit owns behavior (input, focus,
accessibility, overlays, windows), this repository owns Material visuals and motion.
The root `[patch.crates-io]` table points a few engine crates at `vendor/` for the
Gallery only; the library builds without it
([Engine patches](docs/ARCHITECTURE.md#engine-patches)).

## Gallery

One gallery module runs on desktop and in the browser. It follows md3.io's component
pages: navigation by component group, a live preview of the real component with theme
and light/dark switches, and a configuration panel for every variant, size, state and
content option. A **Specs** tab shows the Figma kit's boards. Style pages cover color,
typography, shape, motion and elevation.

```sh
cargo run -p gpui-kit-material-gallery      # desktop (GPUI Fast backend)
./scripts/build-gallery-web.sh --release    # browser, then:
python3 -m http.server 8080 --bind 127.0.0.1 --directory crates/gallery-web/www
```

Headless capture, profiling and fixture tooling are in
[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Checks

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs these on macOS, Linux and Windows, plus both GPUI backends, the WASM gallery
build, and `cargo check` of the defaults, no features, `full` with and without Fast, and
each feature group alone (`scripts/check-features.sh`).

## References

Where references disagree, priority is **Figma Material 3 Design Kit > md3.io >
m3e.kyralabs.dev**, per token family and per value; each value records its source. See
[Reference sources](docs/ARCHITECTURE.md#reference-sources).

## Performance targets

Measured on release builds from raw per-frame timings ([AGENTS.md](AGENTS.md) has the
full rules, [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md#targets) the tooling). Acceptance
evidence is still open.

| Target | Limit |
| --- | --- |
| Each comparable interaction (scrolling, hover, animations, typing, page switch, first load), native and WASM | main-thread frame time p95 and max below md3.io's best run in Chrome on the same machine |
| Idle window | draws nothing |

The profiler's finer budgets in DEVELOPMENT.md are diagnostics, not release gates.

## Documents

| Document | Contents |
| --- | --- |
| [docs/PRODUCTION.md](docs/PRODUCTION.md) | What a component and a release must meet to be called stable |
| [docs/RELEASING.md](docs/RELEASING.md) | Release order and release-day steps |
| [docs/COVERAGE.md](docs/COVERAGE.md) | Every reference component, its status and what is missing |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Status tables and remaining work |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Layers, Kit reuse, reference sources, theme bridge, engine patches |
| [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) | Gallery capture, profiling, WASM, fixtures |
| [docs/EXTRACTION.md](docs/EXTRACTION.md) | Mapping from course2md, where the kit originated |
| [scripts/README.md](scripts/README.md) | Reference capture, Figma extraction and font scripts |
