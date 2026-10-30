# gpui-pre-linux source

Complete published package, verified against its crates.io archive. `SOURCE.json`
records the archive SHA-256, every original file hash, and the original and patched
hash of each changed file.

These patches implement the GPUI backend hooks in
`vendor/gpui-pre/frame-submission-patch.md`:

- `src/linux/x11/window.rs`, `src/linux/wayland/window.rs`
  - `successful_frame_submissions` reports a window-owned
    `gpui::FrameSubmissionCounter`, because device-loss recovery replaces the
    renderer. `draw` moved unchanged into the inherent `draw_frame`, which records
    the renderer's `take_present_succeeded`. Device-loss returns, skipped frames and
    rejected presentations do not count; GPUI decides whether the draw generation
    is new.
  - `vsync_status` and `set_vsync` read and change the renderer's present mode.

Frame requests stay paced by the X11 refresh timer and Wayland frame callbacks, as
upstream: disabling vsync removes presentation blocking but does not raise the frame
rate above the display's refresh rate. Runtime behavior awaits verification on Linux.
