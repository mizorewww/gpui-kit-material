# gpui-pre-macos source

**Dev-only overlay.** This crate is vendored only for the gallery's and the profiler's frame observation, and its manifest turns on the engine's `frame-observation` feature. A consumer does not patch it; see "Consumer patch set" in `docs/ARCHITECTURE.md`.

Complete published 0.3.11 package, verified against its crates.io archive.
`SOURCE.json` records the archive SHA-256, every original file hash, and the
original and patched hash of each changed file. Original package license files
and the package `Cargo.lock` are retained unchanged. The archive ships no
`.cargo_vcs_info.json`.

These patches implement the GPUI backend hooks in
`vendor/gpui-pre/frame-submission-patch.md`:

- `src/window.rs`
  - `vsync_status`/`set_vsync` read and write the real `CAMetalLayer`
    `displaySyncEnabled`; a setting Metal does not apply is rolled back and
    reported as an error. A window without a Metal layer is `Unsupported`.
  - `successful_frame_submissions` counts draws for which the renderer's
    `draw_with_status` (`gpui-pre-apple`) reports a submitted command buffer.
  - `has_periodic_frame_source` reports whether the CVDisplayLink is
    registered, so GPUI's throttled retry waits for the next tick instead of
    self-waking.
  - `frame_waker` holds a weak window owner. It only requests a frame when
    synchronization is disabled, the window is visible and the display link is
    running; it does not force rendering or presentation and does not re-arm.
- `src/display_link.rs`: `WindowFrameSource::is_running` and `wake`, which
  enqueue one demand request on the existing coalescing main-thread source
  (no timer, no self-rescheduling).

Since 0.3.11 / 0.1.13 upstream paces a window on macOS 14 and later with a
`CAMetalDisplayLink` (`FrameSource::Metal`), which draws only into the drawables
it hands out at the display's pace, and keeps the `CVDisplayLink` registry for
earlier systems (`FrameSource::DisplayLink`). Demand frames cannot be added
between a Metal link's updates, so `set_vsync(false)` moves the window to the
`CVDisplayLink` path (`pace_for_synchronization`, which parks the Metal frames),
and `set_vsync(true)` restores the Metal link. With synchronization on, pacing is
upstream's. `has_periodic_frame_source` is `false` for a Metal link, which pauses
when GPUI goes idle and is resumed by GPUI's retry at its next tick, and the demand
`frame_waker` first calls upstream's Metal waker, returning when a link is attached.

CVDisplayLink stays subscribed as the throttle and synchronized path.
Disabling synchronization permits demand-driven requests between display
ticks; it does not guarantee any target FPS. Platform/UI acceptance and
performance are verified separately.
