# Frame submission statistics, vsync control and throttle retry

Source: the published Apache-2.0 `gpui-pre` 0.3.10 package (`SOURCE.json`).
Files: `src/platform.rs`, `src/window.rs`, `src/platform/test/window.rs`.

`PlatformWindow` gains three defaulted methods, so unpatched backends compile
and report honestly:

- `successful_frame_submissions() -> Option<u64>`: a monotonic backend count of
  submissions that obtained a drawable and submitted render work; `None` means
  the backend cannot establish success. Never GPU completion or scanout.
- `vsync_status()` / `set_vsync(bool)`: `VsyncStatus::{Unsupported, Enabled,
  Disabled}` read from the real backend setting. The default is `Unsupported`
  and `set_vsync` returns an error rather than pretending.
- `has_periodic_frame_source()`: whether a running periodic source (for example
  a display link) will request another frame without an explicit wake.

`Window` exposes `submission_stats() -> WindowSubmissionStats` (returned
`draw`/`draw_composed` attempts, the time of the last one, and successful
submissions of distinct `Window::draw` generations), `vsync_status()` and
`set_vsync()`. Each `draw` advances a generation; a submission counts as a new
frame only when the backend counter increases and that generation has not
already succeeded, so re-presenting the same scene is not counted and a failed
first attempt can still be followed by a successful retry. A counter reset
yields "unknown". The same decision feeds `PresentTiming::successful_new_frame`
(`presentation-timing-patch.md`).

Throttled frame retry: upstream always called `schedule_frame()` and
`wake_platform()` when a frame was deferred by the inactive-window throttle.
On a backend that has both a display link and an unpaced demand waker (the
patched macOS backends with vsync disabled) that immediate wake busy-loops.
The patch skips the self-wake when `has_periodic_frame_source()` is true; the
next periodic tick after the deadline serves the pending work. Demand-only
backends keep the default `false` and retain the upstream retry. The test
window exposes `periodic_frame_source` for
`test_periodic_source_throttle_waits_and_resumes`.

Backend implementations: `gpui-pre-macos`/`gpui-fast-macos` (Metal display
sync, demand waker, counts), `gpui-pre-web`/`gpui-fast-web` (counts only;
browser-controlled vsync stays `Unsupported`), `gpui-*-linux` through `gpui-*-wgpu`
and `wgpu` (present-status counts, `PresentMode` control) and `gpui-*-windows`
(`S_OK` counts, DXGI sync interval). Each has its own `MATERIAL-PATCHES.md`.

`src/platform/frame_submission.rs` (added) holds what those backends share, so
it is tested on every host: `FrameSubmissionCounter` counts a frame once and only
when every surface it required presented, and `dxgi_present_succeeded` accepts only
`S_OK`, because `DXGI_STATUS_OCCLUDED` is a success code that presented nothing.

Regression command:

```sh
cargo test -p gpui-pre --lib window::tests
cargo test -p gpui-pre --lib frame_submission
cargo test -p gpui-pre-wgpu --lib present_mode # Linux hosts: wgpu is inactive elsewhere
```
