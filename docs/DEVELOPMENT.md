# Development

Tooling around the shared gallery. Acceptance rules are in
[AGENTS.md](../AGENTS.md); none of these tools produces a conformance PASS on
its own.

## Backends

Material, the Gallery, gallery-web and the benches default to Kit's **GPUI Fast** backend.
For comparison, run the upstream backend:

```sh
cargo run -p gpui-kit-material-gallery --no-default-features --features embedded-symbols
./scripts/build-gallery-web.sh --upstream   # or --fast (default)
```

## Feature combinations

```sh
scripts/check-features.sh                                  # defaults, nothing, full, each group alone
scripts/check-features.sh --target wasm32-unknown-unknown
```

CI's `features` job runs both. Adding a component to a group, or making one use another
group, is where this fails; the groups are in the [README](../README.md#crates).

## Browser gallery

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
./scripts/build-gallery-web.sh [--release]
python3 -m http.server 8080 --bind 127.0.0.1 --directory crates/gallery-web/www
```

The single-threaded build needs no cross-origin isolation headers. Generated
bindings stay out of Git. Check canvas allocation at real 1×/1.25×/1.5×/2×/3× device
scale factors (requires `agent-browser` and Chromium; validates allocation only,
not pixels). CDP device-metrics emulation reports a device-pixel box that
disagrees with `devicePixelRatio`, so do not use it to judge canvas size:

```sh
python3 scripts/check-gallery-dpr.py http://127.0.0.1:8080 /tmp/material-gallery-dpr
```

The page shows a splash until the engine submits its first frame, and an error
message when neither WebGPU nor WebGL2 is available, startup fails or a phone takes
the WebGL context back. Latin text fonts are embedded in the module; CJK and emoji
draw with the browser's fonts (`CanvasFontFallback::EmojiAndCjk`), so they add no
download. The Outlined and Rounded icon fonts, the symbol styles the Gallery draws,
are fetched after the first frame with a thin progress bar and passed in through
`provide_symbol_asset`; icons draw empty until then. Keeping them out of the module
also keeps it under the 25 MiB per-file hosting limit.

On a phone the page follows the visual viewport, so the canvas lays out above an
on-screen keyboard and the Gallery scrolls the caret back into view, and the shell
stays inside the safe-area insets. `#button` or `#button/specs` opens a page, and
the browser's Back returns to the previous one. `--release` also strips names and runs Binaryen's `wasm-opt` when it is on
`PATH` (`brew install binaryen`); publishing needs it to stay under that limit.

## Publishing the site

`scripts/build-site.sh` writes the static site to `target/site`: this README and
the docs at `/`, the release Gallery at `/gallery/` with content-hashed assets, and
`cargo doc` output for the two library crates at `/api/`. It also writes `_headers`
(immutable hashed assets, revalidated HTML) and fails on a broken internal link or
heading anchor. Links to repository files that are not published go to GitHub. It
needs [`uv`](https://docs.astral.sh/uv/) for the Markdown dependency; pass
`--skip-wasm` to reuse the bindings already in `crates/gallery-web/www/wasm`.

`--release` builds with the `wasm-release` profile (fat LTO, one codegen unit, source
paths remapped); `release` stays what performance profiles measure. A `.br` copy at
highest quality sits beside the module, its bindings and the icon fonts, and
`deploy/site/worker.js` serves it for `/gallery/wasm/*` with `encodeBody: "manual"`,
which Cloudflare's own compression loses to. Everything else stays a plain asset, and
only those requests count as Worker invocations. `node --test deploy/site/worker.test.js` tests the
Worker's rules, and `npx cf dev` in `deploy/site` with `SITE_DIR` set runs it locally;
its dev proxy rewrites `Accept-Encoding`, so a client without Brotli cannot be tried there.

`scripts/deploy-site.sh` refuses a site not built from the current commit: it reads the
record `scripts/site_stamp.py` writes beside `target/site`. HEAD and a clean tree must
match the site's record, the module must be a `wasm-release` build whose compiled inputs
have not changed since (so `--skip-wasm` still serves a documentation change), and the
directory must be unchanged since the build. `--check` stops after these checks.

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
cargo run -p gpui-kit-material-gallery -- \
  --capture /tmp/material-button.png --section button \
  --set variant=tonal --set size=L --width 1280 --height 900 --reduced-motion
```

`--section` takes a page id (`button`, `text-field`, `color`, …) or a
design-kit page id (`buttons`, `text-fields`, …, which opens that page's
Specs tab). `--tab preview|specs` picks the component card's tab;
`--scenario <name>` and repeated `--set <control>=<value>` configure the
page's playground (control ids are in `playgrounds/<component>.rs`); invalid
names are errors. `--palette <name>` paints a built-in palette (`Paper`,
`Ink`, `Nord`, …) instead of the baseline theme, with `--dark` for a dark
one. `--help` lists the ids, theme, text scale, state (`--active`,
`--disabled`, `--no-ripple`, `--selected`, `--loading`) and scroll options.
A JSON sidecar records revision, dirty state, requested and actual
viewport, device scale, theme and motion settings. The OS may constrain the
window, so compare actual dimensions. A nonzero exit means the capture
failed; an existing file is not evidence of success. Captures are for visual
review, not timed animation.

## Profiling

The engine is upstream's: Material adds no frame instrumentation to it. Frame timing
comes from upstream's own `profiler` feature (the Gallery's `frame-profiler`), which
records each frame's `Window::draw` (with its request-layout, prepaint and paint
phases) and the platform submission of what it drew. Everything else is measured
outside the engine:

| What | How |
| --- | --- |
| Native frame time per interaction | `scripts/profile-gallery.py` (below): draw + submission per frame, from upstream's profiler |
| Native main-thread work outside draw and submission (input dispatch, the frame callback, cleanup) | a sampling profiler on the release build: Instruments' Time Profiler, or `sample <pid>` |
| Native GPU time | Instruments' Metal System Trace |
| Instructions per frame, independent of load | `--features instruction-bench` (below) and `component_bench` |
| Browser frame time | `www/profile.js` (each `requestAnimationFrame` callback) and Chrome DevTools performance traces |
| md3.io | the same probe and DevTools traces ([below](#chrome-comparison-with-md3io)) |

What can no longer be measured, since the dev-only engine patches that did it were
dropped (gpui-fast does not take API additions):

- the whole synchronous frame callback as one CPU scope, from the engine;
- the drawable wait (`nextDrawable`) separately from submission, and Metal GPU
  timestamps in the report;
- successfully submitted new frames, so neither the title bar's FPS counter nor a
  drawn-frame count: "idle windows draw nothing" is checked in a trace (Instruments,
  DevTools), not by a counter;
- frames with vertical sync off (the switch and `--vsync` are gone): every frame is
  paced by the display, so a frame's time is what it costs, not a frame rate;
- full-traversal scroll runs with per-frame position association (`--scroll`), and
  the standalone `engine_bench`.

### Interaction profiles

```sh
python3 scripts/profile-gallery.py /tmp/material-interactions --backend fast --script interactions
```

builds a release Gallery with `frame-profiler`, runs one interaction script
(`crates/gallery/src/interactions.rs`) and writes `frames.json` (every frame, labelled
with the step that was active, raw), `targets.json` and the build's provenance.
Scripts: `interactions` (page switches between Button, Text field, Date picker and
Examples, hover and press in the Button preview, 8 px wheel scrolling of Overview and
Button's Specs), and `hover`, `switch`, `scroll-specs` and `scroll-overview`, which
repeat one of them. The binary runs the same scripts with
`MATERIAL_PROFILE_SCRIPT=<script> MATERIAL_PROFILE_OUTPUT=<frames.json>`.

A frame's time is its draw plus its submission; the first frame after a page switch is
that switch's cost and is kept out of the steady state. An unfocused window is
throttled, so keep the Gallery's window in front while it runs, and compare runs taken
under the same conditions: the report records the load average. macOS gives a covered
window no frames, and a remote desktop's overlay covers every window: such a run
submits nothing, and the runner reports it incomplete (exit 2). Run on a local display.

### Targets

The release target is AGENTS.md's: beat md3.io's best run on the same machine
([the comparison below](#chrome-comparison-with-md3io)), and draw nothing when idle.
The budgets in this table are diagnostics for catching regressions, not release
gates. Every run writes `targets.json` and prints one verdict per budget (PASS,
FAIL or UNVERIFIED when the data to judge it is missing), from raw per-frame data
with nearest-rank quantiles; an average never decides one:

| Target | Limit | Source |
| --- | --- | --- |
| steady-state frame p95 / max | 1 ms / 2 ms (WASM 2 / 4 ms) | interaction frames other than the first and those after a page switch; `www/profile.js` for WASM |
| Material list against an equivalent Kit list | ≤ 2× the instructions per frame, worst of p50 and p95 | `component_bench --content material-list` and `kit-list`, same mode; UNVERIFIED without instruction counters, which read 0 outside native macOS |
| first frame | ≤ 1.2 × Kit's stories baseline on this machine | the initial draw plus the first submitted frame, against `--kit-first-frame-ms`; UNVERIFIED without that baseline |
| page switch | 8 ms | the first frame after each switch |
| idle window | 0 frames drawn | not judged by the tool: a trace of an untouched window |
| WASM against md3.io | main-thread frame time below md3.io's | stays UNVERIFIED in the tool; [the comparison below](#chrome-comparison-with-md3io) is by hand |

Re-judge saved evidence without running anything:

```sh
python3 scripts/profile-gallery.py --summarize /tmp/material-interactions/frames.json --kit-first-frame-ms 42
cargo run --release -p gpui-kit-material-bench --bin component_bench -- \
  --content material-list --mode scroll --frames 300 --out /tmp/material-list.json
cargo run --release -p gpui-kit-material-bench --bin component_bench -- \
  --content kit-list --mode scroll --frames 300 --out /tmp/kit-list.json
python3 scripts/profile-gallery.py --summarize /tmp/material-list.json /tmp/kit-list.json
```

Runner exit status: `0` for a complete run with no failing budget, `1` when a budget
failed, `2` for incomplete or unstable evidence. Evidence for a release commit goes
under [conformance/performance/releases/](../conformance/performance/releases/README.md).

### Instruction counts

`--features instruction-bench` counts retired instructions per main-thread frame, which
load and Low Power Mode leave alone (`src/instr_bench.rs`). `MATERIAL_INSTR_MODE` is
`scroll`, `redraw`, `idle`, `switch` or `tour`; `tour` visits each entry of
`MATERIAL_INSTR_TOUR` in turn (`button,text-field,button:specs`) for
`MATERIAL_INSTR_TOUR_ROUNDS`, recording what each switch cost on its first and later
visits, with phase times and lines shaped. Every report also has `startup`, the
instructions at each step from `main` to the first presented frame.

A window that something covers gets no frames from macOS, and a benchmark then waits for
ever; a remote desktop's overlay covers every window, so run it on a local display.
Counts stay comparable; a stalled main thread can still add frames, as a wheel's hover
freeze expiring refreshes the window, so compare the frames' `row_builds` too.

### Chrome comparison with md3.io

AGENTS.md asks for main-thread frame time p95 and max below md3.io's best run in
Chrome, natively and as WASM, for equivalent pages and interactions. Measure the
browser sides the same way, release WASM, same Chrome build, flags, viewport
(1280×800) and DPR, with the machine otherwise idle. Natively, the interaction
profiles above give draw and submission per frame; a Time Profiler trace of the same
interaction shows the main-thread work around them, which they leave out.

1. Automated probe. `www/profile.js` contains `runFrameProbe`, a
   self-contained function that times each animation frame from its
   `requestAnimationFrame` callback to a `MessageChannel` task posted there,
   which runs after that frame's style, layout, paint and commit, while it
   sends 8 px wheel steps (and scrolls the nearest scroller) at 1000 Hz to what is under the
   viewport's centre once the page has settled. The report's `moved` says whether they moved
   anything, from `scrolled_px` on a DOM page; a report with `moved` false measured an
   idle page:

   ```sh
   python3 scripts/profile-gallery.py /tmp/md3io --md3io https://md3.io/components/buttons --seconds 10
   # same probe on the Gallery, served as in "Browser gallery":
   #   http://127.0.0.1:8080/?profile=probe&seconds=10  -> window.__galleryProfile.report
   # (the Gallery's canvas reports `moved` null: the engine no longer counts drawn frames)
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

### Browser (WASM)

`--web` builds the release WASM Gallery, serves it with cross-origin isolation
(finest `performance.now()` resolution) on a free local port and runs it in
headless Chromium through `agent-browser` with
`--disable-gpu-vsync,--disable-frame-rate-limit`, then closes the browser and
server:

```sh
python3 scripts/profile-gallery.py /tmp/material-web --web --backend fast --seconds 15 --web-mode scroll
```

The same page works by hand: open `http://127.0.0.1:8080/?profile=scroll&seconds=15`
(or `profile=startup`, `profile=idle`; optional `hz`, `legpx`) and read
`window.__galleryProfile.report` when `done`. `www/profile.js` times every
`requestAnimationFrame` callback (one whole GPUI frame when it draws) and the
synchronous `run()` startup with `performance.now()` into preallocated buffers. The
engine does not say which callbacks drew, so the report counts callbacks, not frames,
and leaves the idle target UNVERIFIED: read drawn frames from a DevTools performance
trace. The scroll driver dispatches 8 px DOM wheel events from a `MessageChannel` loop
independent of animation frames. It cannot read the list position, so legs scroll a
fixed distance and coverage stays unverified; GPU time is null (no WebGPU timestamp
queries). Browser reports are never acceptance-eligible.

## Fonts and reference fixtures

The gallery uses Roboto 400/500/600 (the Figma kit's emphasized styles need 600),
instantiated from the reference site's pinned variable font (hashes in
`fonts/SOURCES.json`, SIL OFL), from
`crates/gallery/fonts/roboto`; the browser build also
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
into `conformance/reference/`. Figma kit token fixtures (`figma-*.json`) come from
`scripts/figma/figma.py` ([scripts/README.md](../scripts/README.md#figma-kit-first-priority-reference)).

## Vendored engine tests

```sh
cargo test -p gpui-pre-apple --test font_fallback
cargo test -p gpui-pre --lib text_system
cargo test -p gpui-kit-material -p gpui-fast --features gpui-kit-material/gpui-fast --lib
cargo test -p gpui-fast-apple --test font_fallback
```
