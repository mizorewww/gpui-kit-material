# gpui-fast source

`gpui-fast` 0.1.14 as published on crates.io, with the local changes below, recorded by
`scripts/vendor-gpui-fast.py --published 0.1.14`. `SOURCE.json` records the archive's
SHA-256, every upstream file hash and the original and patched hash of each changed
file. The package-level `Cargo.lock` is not vendored. The other gpui-fast crates
(`-macos`, `-linux`, `-windows`, `-wgpu`, `-web`, `-platform`, `-macros`) come from
crates.io 0.1.14 unchanged.

Released in 0.1.14 and no longer patched here (each was a Material PR):
wheel at its scroll limit (#104), released entities in a spliced view (#106),
hover freeze for listeners (#105), `record_entities_accessed` by what entered and
left (#114), `em` metrics through fallback shaping (#115), lazy tab-stop index
(#113, upstream's `src/fast/tab_stop.rs`; `src/tab_stop.rs` is unused upstream code).

No API is added or changed. `src/profiler.rs` and `src/profiler/journal.rs` use
`try_update` in place of the deprecated `fetch_update` (identical semantics), so
deny-warnings builds cover the crate.

Test harness: `src/style.rs` (`#[perf]` to `#[test]`), `src/svg_renderer.rs` and
`tests/fonts/` (fonts the package omits), `examples/image/` (a fixture a
`src/platform.rs` test reads), and `src/profiler/hang.rs`'s
`slow_views_are_journaled_with_their_own_time`, which takes the draw that contains the
slow views rather than the last one: a later frame can replay them without the work.

Behavior, not yet released upstream: `src/text_system/line_layout.rs`,
`src/text_system/line_wrapper.rs` hang a space that overflows after content
instead of starting the next wrapped row with it (`vendor/gpui-pre/hanging-space-patch.md`; zed-industries/zed#66644).
`src/elements/list.rs`: after a width change (since 0.1.14, a change of the width
inside the list's padding and border), a `measure_all` list lays out only the items it
shows in that frame, keeping the others' heights as estimates scaled by how
those items reflowed, and measures the rest over later frames, 4 ms per frame
(`vendor/gpui-pre/list-width-remeasure-patch.md`; offered as longbridge/gpui-fast#129).
