# gpui-fast-macos source

Complete published 0.1.12 package, verified against its crates.io archive.
`SOURCE.json` records the archive SHA-256, every original file hash, and the
original and patched hash of each changed file. Original package license files
and the package `Cargo.lock` are retained unchanged. The archive ships no
`.cargo_vcs_info.json`.

These patches implement the GPUI backend hooks in
`vendor/gpui-pre/frame-submission-patch.md`, matching `gpui-pre-macos` plus
Fast composition surfaces:

- `src/window.rs`: `vsync_status`/`set_vsync` delegate to the composition
  helpers; `successful_frame_submissions`; `has_periodic_frame_source`
  (CVDisplayLink registered); the weak-owner `frame_waker` that requests a
  frame only when synchronization is `Disabled`, the window is visible and the
  display link is running, without forcing render/present or re-arming; and
  the unconditional `anyhow::Result` import needed by `set_vsync`.
- `src/display_link.rs`: `is_running` and `wake` (one coalesced demand request
  on the existing main-thread source).
- `src/fast/composition.rs`: `vsync_status` reads every composed Metal layer
  and reports `Unsupported` if they disagree; `set_vsync` sets all layers and
  rolls back on failure; new surfaces inherit the base layer setting or fail
  creation. A composed frame counts as one successful submission only when
  every required surface's `draw_with_status` (`gpui-fast-apple`) submits; a
  missing surface makes it incomplete. Uncomposed windows count the base draw.

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

## Window tab group built only when tabbing is in use

- `src/window.rs`: `tab_bar_visible` returns false without asking for `tabGroup`
  when the window has no tabbing identifier, was never merged or toggled, and
  has at most one tabbed window. Asking builds AppKit's window stack controller,
  which was about 40% of `open_window` in a warm sampled window open.
  `MacWindowState::tabbing_in_use` records the opening identifier,
  `set_tabbing_identifier`, `merge_all_windows` and the `toggleTabBar:` and
  `mergeAllWindows:` actions; once set, behavior is as upstream.
