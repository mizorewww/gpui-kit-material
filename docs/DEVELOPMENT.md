# Development

Tooling around the shared gallery. Acceptance rules are in
[AGENTS.md](../AGENTS.md); none of these tools produces a conformance PASS on
its own.

## Backends

Material, the gallery and gallery-web default to Kit's **GPUI Fast** backend.
For comparison, run the upstream backend:

```sh
cargo run -p gpui-kit-material --example gallery --no-default-features --features serde,embedded-symbols
./scripts/build-gallery-web.sh --upstream   # or --fast (default)
```

## Browser gallery

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
./scripts/build-gallery-web.sh [--release]
python3 -m http.server 8080 --bind 127.0.0.1 --directory crates/gallery-web/www
```

The single-threaded build needs no cross-origin isolation headers. Generated
bindings stay out of Git. Check canvas allocation at 1×/1.25×/1.5×/2× DPR
(requires `agent-browser`; validates allocation only, not pixels):

```sh
python3 scripts/check-gallery-dpr.py http://127.0.0.1:8080 /tmp/material-gallery-dpr
```

## Frame capture

The native gallery can save a GPU-rendered frame in a reproducible state:

```sh
cargo run -p gpui-kit-material --example gallery -- \
  --capture /tmp/material-toggles.png --section toggles \
  --width 1280 --height 720 --selected --reduced-motion --settle-ms 0
```

`--help` lists theme, text scale, state (`--active`, `--disabled`,
`--no-ripple`, `--selected`, `--loading`) and scroll options. A JSON sidecar
records revision, dirty state, requested and actual viewport, device scale,
theme and motion settings. The OS may constrain the window, so compare actual
dimensions. A nonzero exit means the capture failed; an existing file is not
evidence of success. Captures are for visual review, not timed animation.

## Profiling

`scripts/profile-gallery.py` builds a release gallery and records full CPU
frame scopes (including startup), build flags, source identity and raw timings:

```sh
python3 scripts/profile-gallery.py /tmp/material-fast --backend fast --seconds 5
python3 scripts/profile-gallery.py /tmp/material-scroll --backend fast --seconds 10 \
  --scroll --scroll-input wheel
```

`--scroll` (2–120 s) disables vertical sync and scrolls continuously up and
down. `--scroll-input wheel` sends pixel wheel events through the normal input
path; the default `programmatic` mode mutates the offset directly and bypasses
wheel invalidation, so prefer `wheel` for gate evidence. Reports keep raw scroll
positions, successful new-frame counts and the lowest FPS over ≥ 250 ms windows.

The gallery FPS counter counts only successfully submitted new draws, sampled
every 500 ms; skipped submissions, repeated presents and idle time do not count,
and it is not display refresh or GPU completion. Recorded runs live under
`conformance/performance/`.

## Fonts and reference fixtures

The gallery uses the reference site's Roboto 400/500 (pinned hashes, SIL OFL)
from `crates/material/examples/gallery/fonts/roboto`; the browser build also
embeds IBM Plex Sans. Rebuild the static fonts with:

```sh
uv run --with 'fonttools[woff]==4.60.1' python scripts/build-reference-fonts.py
```

Reference theme colors (`crates/color/tests/fixtures/md3-colors.json`) are
captured from a clean checkout of the pinned md3.io source (needs `npx`, `git`,
`agent-browser`):

```sh
python3 scripts/capture-reference-colors.py /path/to/floor-material /tmp/md3-colors.json
```

The `scripts/capture-*.{py,js}` pairs record per-component reference behavior
into `conformance/reference/`.

## Vendored engine tests

```sh
cargo test -p gpui-pre --test line_layout_options
cargo test -p gpui-pre --lib text_system
```
