# Source

This directory vendors `gpui-pre-apple` 0.3.10 from crates.io:
<https://crates.io/crates/gpui-pre-apple/0.3.10>.

Published archive SHA-256:
`e97cd068d8134b4e073321b946491760aa1e4511e486084a92e73b9174bff78a`.
The upstream Apache 2.0 license is retained in `LICENSE-APACHE`. The nested
`vendor/gpui` source is part of the published crate and supplies Metal shader
build definitions.

`SOURCE.json` records every original file hash and the original and patched
hash of each changed or added file. The archive ships no `.cargo_vcs_info.json`.
The package-level `Cargo.lock` is not retained; the workspace lockfile owns
resolution.

Text patches (`src/text_system.rs`, `src/open_type.rs`, added
`tests/detailed_layout.rs`; `Cargo.toml` adds `unicode-bidi = "0.3.18"` and the
`detailed_layout` `[[test]]`):

- `layout_line_with_options` implements the opt-in GPUI API
  (`vendor/gpui-pre/line-layout-options-patch.md`): it preserves CoreText's
  glyph positions, explicit paragraph direction (`CTParagraphStyle`), shaping
  clusters and visual caret intervals (`CTLineEnumerateCaretOffsets`).
  Existing layout calls continue using the upstream path.
- Fonts without an `m` glyph are no longer skipped at load (valid non-Latin
  fonts); GPUI measures `em`/`ch` through fallback instead.
- App-private fallback fonts are placed in the CoreText cascade from their
  concrete descriptors, in declared order, and previously selected fonts get
  their cascades rebuilt after a later font registration without dropping font
  IDs still used by shaped frames.
- Color-bitmap glyph support is detected once per font ID from the presence of
  an `sbix` table instead of matching the Apple Color Emoji PostScript names.
- `unicode-bidi` resolves bidi levels for the detailed path.

Presentation patches (`src/metal_renderer.rs`). `draw_with_status` reports
actual command-buffer submission (`draw` delegates to it). Its per-call autorelease pool drains temporary Metal
references before returning to the presentation timer, including early returns;
demand frames need not wait for the enclosing Cocoa event-loop pool.
Synchronization, drawable timeout, buffering, and counting policy are unchanged.

Profiler patch: while profiler tracing is enabled, `draw_with_status` times
`CAMetalLayer nextDrawable` and reports it through
`gpui::profiler::record_presentation_wait`, so frame evidence can separate
blocked drawable waits from CPU work. Presentation policy is unchanged.

GPU timing patch: while profiler tracing is enabled, `attach_gpu_timing` adds a
command-buffer completion handler that reports `GPUStartTime`, `GPUEndTime`,
status and the CPU commit time through `gpui::profiler::record_gpu_execution`.
Rendering and presentation are unchanged.
