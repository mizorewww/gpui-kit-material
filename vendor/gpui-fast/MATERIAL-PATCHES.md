# gpui-fast source

gpui-fast `main` at `bbdbc42d`, packaged by upstream's own release step, as
`scripts/vendor-gpui-fast.py` reproduces it (version kept at 0.1.13). `SOURCE.json`
records every upstream file hash and the original and patched hash of each changed
file. The package-level `Cargo.lock` is not vendored.

Merged upstream and no longer patched here (each was a Material PR):
wheel at its scroll limit (#104), released entities in a spliced view (#106),
hover freeze for listeners (#105), `record_entities_accessed` by what entered and
left (#114), `em` metrics through fallback shaping (#115), lazy tab-stop index
(#113, upstream's `src/fast/tab_stop.rs`; `src/tab_stop.rs` is unused upstream code).

Dev-only, behind the `frame-observation` feature that only the dev overlay's
backend manifests turn on (`vendor/gpui-pre/*-patch.md` describe the same patches):

- `Cargo.toml`: the `frame-observation` feature.
- `src/app.rs`, `src/app/bench_context.rs`: CPU frame scopes around initial and
  test draws; legacy phase metrics ignore `FrameEvent::Complete`.
- `src/profiler.rs`, `src/profiler/hang.rs`, `src/profiler/journal.rs`: presentation
  wait, GPU time and submission outcome; `fetch_update` renamed `try_update`.
- `src/platform.rs`, `src/platform/frame_submission.rs` (added),
  `src/platform/test/window.rs`, `src/window.rs`: submission statistics, vsync control,
  periodic frame source and throttle retry.
- `src/fast/dependencies.rs`, `src/fast/tests/dependencies.rs`:
  `Entity::update_for_render` (render lease).

Test harness: `src/style.rs` (`#[perf]` to `#[test]`), `src/svg_renderer.rs` and
`tests/fonts/` (fonts the package omits), `examples/image/` (a fixture a
`src/platform.rs` test reads).

Behavior, not yet offered upstream: `src/text_system/line_layout.rs`,
`src/text_system/line_wrapper.rs` hang a space that overflows after content
instead of starting the next wrapped row with it (`vendor/gpui-pre/hanging-space-patch.md`).
