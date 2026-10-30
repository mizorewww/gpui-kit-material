# gpui-pre-macos source

Complete published 0.3.10 package, verified against its crates.io archive.
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
  - `anyhow::Result` is imported unconditionally (previously
    `cfg(test/test-support)` only) because `set_vsync` uses it.
- `src/display_link.rs`: `WindowFrameSource::is_running` and `wake`, which
  enqueue one demand request on the existing coalescing main-thread source
  (no timer, no self-rescheduling).

CVDisplayLink stays subscribed as the throttle and synchronized path.
Disabling synchronization permits demand-driven requests between display
ticks; it does not guarantee any target FPS. Platform/UI acceptance and
performance are verified separately.

## Off-main-thread drawable acquisition

- `src/window.rs`: `step` asks the renderer `drawable_ready_or_wake()` and skips
  the frame request while no prefetched drawable is ready; `start_display_link`
  passes the frame source's thread-safe waker to the renderer
  (`set_drawable_waker`). Transactional presents (live resize, activation)
  pause prefetching and acquire synchronously as before.
- `src/display_link.rs`: `WindowFrameSource::waker`, a `Send + Sync` closure that
  merges one request into the existing main-thread dispatch source.

See `src/drawable_prefetch.rs` in the matching Apple renderer crate.
Successful-submission counting is unchanged: only `draw_with_status`
submissions count.
