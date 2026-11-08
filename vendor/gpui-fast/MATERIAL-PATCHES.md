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

No API is added or changed. `src/profiler.rs` and `src/profiler/journal.rs` use
`try_update` in place of the deprecated `fetch_update` (identical semantics), so
deny-warnings builds cover the crate.

Test harness: `src/style.rs` (`#[perf]` to `#[test]`), `src/svg_renderer.rs` and
`tests/fonts/` (fonts the package omits), `examples/image/` (a fixture a
`src/platform.rs` test reads).

Behavior, not yet offered upstream: `src/text_system/line_layout.rs`,
`src/text_system/line_wrapper.rs` hang a space that overflows after content
instead of starting the next wrapped row with it (`vendor/gpui-pre/hanging-space-patch.md`).
