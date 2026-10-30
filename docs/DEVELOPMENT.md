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

The page shows a splash until the engine submits its first frame, and an error
message when neither WebGPU nor WebGL2 is available or startup fails. Fonts for
text are embedded in the module. The Outlined and Rounded icon fonts, the symbol
styles the Gallery draws, are downloaded next to it and passed in through
`provide_symbol_asset`, which keeps the module under the 25 MiB per-file hosting
limit. `--release` also strips names and runs Binaryen's `wasm-opt` when it is on
`PATH` (`brew install binaryen`); publishing needs it to stay under that limit.

## Publishing the site

`scripts/build-site.sh` writes the static site to `target/site`: this README and
the docs at `/`, the release Gallery at `/gallery/` with content-hashed assets, and
`cargo doc` output for the two library crates at `/api/`. It also writes `_headers`
(immutable hashed assets, revalidated HTML) and fails on a broken internal link or
heading anchor. Links to repository files that are not published go to GitHub. It
needs [`uv`](https://docs.astral.sh/uv/) for the Markdown dependency; pass
`--skip-wasm` to reuse the bindings already in `crates/gallery-web/www/wasm`.

```sh
./scripts/build-site.sh
python3 -m http.server 8080 --bind 127.0.0.1 --directory target/site  # local preview
cf auth login                                                         # once
./scripts/deploy-site.sh
```

`scripts/deploy-site.sh` publishes `target/site` to <https://material.mizore.blog>
and checks the live URLs. Deployment is manual: CI holds no Cloudflare credentials.
The site is an assets-only Worker configured in `deploy/site/`, because the `cf` CLI
no longer uploads to classic Pages projects. `cf deploy` creates the DNS record for
that hostname and changes nothing else in the zone.

## Frame capture

The native gallery can save a GPU-rendered frame in a reproducible state:

```sh
cargo run -p gpui-kit-material --example gallery -- \
  --capture /tmp/material-button.png --section button \
  --set variant=tonal --set size=L --width 1280 --height 900 --reduced-motion
```

`--section` takes a page id (`button`, `text-field`, `color`, …) or a
design-kit page id (`buttons`, `text-fields`, …, which opens that page's
Specs tab). `--tab preview|specs` picks the component card's tab;
`--scenario <name>` and repeated `--set <control>=<value>` configure the
page's playground. `--help` lists the ids, theme, text scale, state
(`--active`, `--disabled`, `--no-ripple`, `--selected`, `--loading`) and
scroll options. A JSON sidecar
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

Drawable acquisition runs off the main thread: each vendored Metal renderer
(`gpui-{pre,fast}-apple`, `src/drawable_prefetch.rs`) keeps one drawable
acquired ahead by its own thread, and the macOS window skips a frame request
until one is ready instead of blocking in `nextDrawable` (see the crates'
`MATERIAL-PATCHES.md`/`MATERIAL-MIGRATION.json`). Any remaining main-thread
wait (live resize, a stale drawable size, a failed prefetch) is still timed
and reported separately as `presentation_wait_*`/`drawable_wait`. GPU time
(Metal `GPUStartTime`/`GPUEndTime`, macOS only) is reported per frame under
`gpu` and never counts toward the CPU budget; other backends report null with
a reason.

### Targets

Every run writes `targets.json` and prints one verdict per AGENTS.md target
(PASS, FAIL or UNVERIFIED when the data to judge it is missing), from raw
per-frame data with nearest-rank quantiles; an average never decides one:

| Target | Limit | Source |
| --- | --- | --- |
| steady-state CPU p95 / max, excluding drawable wait | 1 ms / 2 ms (WASM 2 / 4 ms) | scroll frames after a 750 ms settle |
| main-thread drawable wait | none (> 0.1 ms in any frame fails) | `drawable_wait` per frame |
| idle window | 0 frames drawn or presented | `--bench` idle scenario, `?profile=idle` |
| forced redraw of an unchanged page | 0.3 ms every frame | `--bench` redraw scenario (root notified each frame) |
| first frame | 16 ms main-thread CPU | thread CPU clock from `open_window` to after the first submission |
| page switch | 8 ms | hook: the Gallery shell reports `page_switch_ns` (native) or calls `window.__galleryPageSwitch(ms)` (web) |
| GPU p95 | 2 ms | Metal timestamps |
| new frames/s, vsync off | ≥ 500 in every 100 ms window | successful submissions only |
| missed refreshes, vsync on | 0 at 120 Hz | intervals > 1.5 refreshes; UNVERIFIED on other displays |

Re-judge saved evidence without running anything:

```sh
python3 scripts/profile-gallery.py --summarize /tmp/material-bench/*.json /tmp/material-scroll/frames.json
```

### Engine benchmark

`crates/material/examples/engine_bench.rs` validates engine changes without
the Gallery: a 4000-row `list` of Material components (`--content material`),
plain quads and text (`simple`), or Gallery-like sections whose boards scroll
horizontally inside their rows (`boards`). It drives one 8 px wheel event per
frame (scroll), nothing (idle), or a root notify per frame (redraw), and
writes raw per-frame CPU scopes, phases, drawable waits, submissions, GPU
executions, engine counters and main-thread CPU time. `--bench` builds it and
runs startup ×3, scroll with vsync off and on, idle and redraw:

```sh
python3 scripts/profile-gallery.py /tmp/material-bench --bench --seconds 5 [--bench-content boards]
cargo run --release -p gpui-kit-material --example engine_bench --features frame-profiler -- \
  --scenario scroll --vsync off --seconds 5 --content material --out /tmp/scroll.json
```

`--open-delay-ms N` opens a small plain window first and the measured window
N ms later, so `sample <pid>` can attach in between (attaching during
launch blocks the process on dyld notifications). Close other Gallery and
build processes first: the reports record the load average, and a loaded
machine inflates every timing and caps FPS.

### Chrome comparison with md3.io

AGENTS.md asks for WASM main-thread frame time ≥ 5× lower than md3.io in
Chrome for equivalent pages and interactions. Measure both sides the same way,
release WASM, same Chrome build, flags, viewport (1280×800) and DPR, with the
machine otherwise idle.

1. Automated probe. `www/profile.js` contains `runFrameProbe`, a
   self-contained function that times each animation frame from its
   `requestAnimationFrame` callback to a `MessageChannel` task posted there,
   which runs after that frame's style, layout, paint and commit, while it
   sends 8 px wheel steps (and scrolls the nearest scroller) at 1000 Hz:

   ```sh
   python3 scripts/profile-gallery.py /tmp/md3io --md3io https://md3.io/components/buttons --seconds 10
   # same probe on the Gallery, served as in "Browser gallery":
   #   http://127.0.0.1:8080/?profile=probe&seconds=10  -> window.__galleryProfile.report
   ```

   `probe.json` keeps the raw per-frame durations. The probe does not see
   input handlers that run before the frame or compositor-thread work, and
   Chrome scrolls md3.io on the compositor thread, so its scroll frames cost
   the main thread almost nothing; compare interactions that need main-thread
   work on both sides as well (hover sweeps over buttons, ripples, switching
   theme or page) with the DevTools trace below.
2. DevTools trace (authoritative breakdown). Start Chrome with
   `--disable-gpu-vsync --disable-frame-rate-limit`, open md3.io, DevTools →
   Performance (Screenshots and CPU throttling off), record while
   repeating the interaction for 10 s (paste `runFrameProbe` into the console
   to drive the same wheel input), then for each frame read the main-thread
   *Animation Frame* duration (Task → Animation frame fired → Style/Layout/
   Paint/Commit) from the Summary/Bottom-Up panels or export the trace JSON
   and sum `RunTask` durations per `BeginMainFrame`. Record the same
   interaction on the Gallery page, where the per-frame work is the
   `requestAnimationFrame` callback (`?profile=scroll` reports it directly).
3. Report p50, p95 and max per side with raw durations, and the ratio of the
   Gallery's p95 to md3.io's. A probe that rounds to zero (100 µs timer
   resolution without cross-origin isolation) means the reference interaction
   does no main-thread work, not that the comparison passed.

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
(or `profile=startup`, `profile=idle`; optional `hz`, `legpx`) and read
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
