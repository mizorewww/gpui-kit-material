# gpui-pre-windows source

**Dev-only overlay.** This crate is vendored only for the gallery's and the profiler's frame observation, and its manifest turns on the engine's `frame-observation` feature. A consumer does not patch it; see "Consumer patch set" in `docs/ARCHITECTURE.md`.

Complete published package, verified against its crates.io archive. `SOURCE.json`
records the archive SHA-256, every original file hash, and the original and patched
hash of each changed file.

These patches implement the GPUI backend hooks in
`vendor/gpui-pre/frame-submission-patch.md`:

- `src/directx_renderer.rs`
  - `draw` and `present` return whether the frame was presented. Only `S_OK` counts
    (`gpui::dxgi_present_succeeded`): `DXGI_STATUS_OCCLUDED` and the mode-change
    statuses are success codes for which nothing was presented, and the frame skipped
    after device loss presents nothing either.
  - `Present` uses `present_sync_interval` in place of upstream's literal 0. It keeps
    upstream's default of 0, which vsync reports as `Disabled`; 1 is `Enabled`.
    Tearing is not requested: it needs a different swap chain flag and is unavailable
    to composition swap chains.
- `src/window.rs`: `successful_frame_submissions` reports a `Cell` kept outside the
  renderer, and `vsync_status`/`set_vsync` the sync interval.

The VSync thread keeps pacing frame requests with `DwmFlush`, as upstream: disabling
vsync does not raise the frame rate above the display's refresh rate. Runtime behavior
awaits verification on Windows.
