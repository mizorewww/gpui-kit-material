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

`--scroll` (2–120 s) disables vertical sync and scrolls the whole Gallery
top → bottom → top repeatedly, turning only at the real ends. Input comes from
a foreground timer (`--input-hz`, default 1000), never from frame callbacks,
so the driver cannot pace frames. `--scroll-input wheel` sends 8 px wheel
events through the normal input path; `programmatic` mutates the offset
directly and bypasses wheel invalidation, so prefer `wheel` for gate evidence.

Reports keep raw positions, laps and row coverage, successful new-frame
counts, the lowest FPS over ≥ 250 ms windows, and draw-associated submission
attempt gaps as a diagnostic. Runs whose rate sits near or below the display
refresh while VSync reads disabled are marked (`observed_rate_near_or_below_refresh`).
On macOS a windowed `CAMetalLayer` with `displaySyncEnabled = NO` was observed
blocking in `nextDrawable` (dispatch semaphore) for most of each ~16.6 ms
frame, so VSync readback alone does not prove unpaced presentation.

The 2 ms judgement uses the whole synchronous CPU scope, including drawable
acquisition. The elapsed time of each `nextDrawable` call is reported
separately (`presentation_wait_*`, `diagnostic_*_excluding_drawable_call`) and
never subtracted from acceptance. GPU time (Metal `GPUStartTime`/`GPUEndTime`,
macOS only) is reported per frame under `gpu` and never counts toward the CPU
budget; other backends report null with a reason.

Runner exit status: `0` only for a complete run the gallery reports eligible
with the CPU and FPS gates both passing; `1` when a measured gate failed; `2`
for incomplete, unstable or unverifiable evidence. Startup and browser runs
cannot establish the FPS gate, so they return `1` or `2`, never `0`.

### Browser (WASM)

`--web` builds the release WASM Gallery, serves it with cross-origin isolation
(finest `performance.now()` resolution) on a free local port and runs it in
headless Chromium through `agent-browser` with
`--disable-gpu-vsync,--disable-frame-rate-limit`, then closes the browser and
server:

```sh
python3 scripts/profile-gallery.py /tmp/material-web --web --backend fast --seconds 15 --scroll
```

The same page works by hand: open `http://127.0.0.1:8080/?profile=scroll&seconds=15`
(or `profile=startup`; optional `hz`, `legpx`) and read
`window.__galleryProfile.report` when `done`. `www/profile.js` times every
`requestAnimationFrame` callback (one whole GPUI frame) and the synchronous
`run()` startup with `performance.now()` into preallocated buffers, and reads
the engine's successful new-frame counter after each scope. The scroll driver
dispatches 8 px DOM wheel events from a `MessageChannel` loop independent of
animation frames. It cannot read the list position, so legs scroll a fixed
distance and coverage stays unverified; GPU time is null (no WebGPU timestamp
queries). Browser reports are never acceptance-eligible.

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
