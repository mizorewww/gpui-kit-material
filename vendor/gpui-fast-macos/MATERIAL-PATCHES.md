# gpui-fast-macos source

**Dev-only overlay.** This crate is vendored only for the gallery's and the profiler's frame observation, and its manifest turns on the engine's `frame-observation` feature. A consumer does not patch it; see "Consumer patch set" in `docs/ARCHITECTURE.md`.

gpui-fast `main` at `bbdbc42d`, packaged by upstream's own release step, as
`scripts/vendor-gpui-fast.py` reproduces it (version kept at 0.1.13). `SOURCE.json`
records every upstream file hash and the original and patched hash of each changed
file. The package-level `Cargo.lock` is not vendored.

These patches implement the GPUI backend hooks in
`vendor/gpui-pre/frame-submission-patch.md`, matching `gpui-pre-macos` plus
Fast composition surfaces:

- `src/window.rs`: `vsync_status`/`set_vsync` delegate to the composition
  helpers; `successful_frame_submissions`; `has_periodic_frame_source`
  (CVDisplayLink registered); the weak-owner `frame_waker` that requests a
  frame only when synchronization is `Disabled`, the window is visible and the
  display link is running, without forcing render/present or re-arming.
- `src/window.rs`: `GPUI_IGNORE_OCCLUSION` draws a covered window as a visible one, for
  instruction benchmarks behind a remote desktop's overlay, which covers every window.
- `src/display_link.rs`: `is_running` and `wake` (one coalesced demand request
  on the existing main-thread source).
- `src/fast/composition.rs`: `vsync_status` reads every composed Metal layer
  and reports `Unsupported` if they disagree; `set_vsync` sets all layers and
  rolls back on failure; new surfaces inherit the base layer setting or fail
  creation. A composed frame counts as one successful submission only when
  every required surface's `draw_with_status` (`gpui-fast-apple`) submits; a
  missing surface makes it incomplete. Uncomposed windows count the base draw.

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
