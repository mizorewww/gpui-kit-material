# gpui-fast-wgpu source

Complete published package, verified against its crates.io archive. `SOURCE.json`
records the archive SHA-256, every original file hash, and the original and patched
hash of each changed file.

These patches give the Linux backends what `vendor/gpui-pre/frame-submission-patch.md`
needs:

- `src/wgpu_renderer.rs`
  - `draw` presents with `Queue::present_checked` (`vendor/wgpu/MATERIAL-PATCHES.md`).
    Its result is unchanged, since Wayland paces its frame loop on "a present was
    issued"; `take_present_succeeded` reports whether the last draw's presentation
    was accepted, and every skip, acquire failure or render failure leaves it `false`.
  - `presentation_synchronized` reads the configured present mode;
    `set_presentation_synchronized` reconfigures the surface for a mode it reported
    supporting, keeps a mode that already matches, and errors when there is none. The
    mode survives device-loss recovery, which recreates the surface from the
    current configuration.
- `src/present_mode.rs` (added): which modes synchronize and which to select,
  separate from any surface so they need no GPU. wgpu is only an active dependency
  on Linux and the web, so `cargo test -p gpui-fast-wgpu --lib present_mode` runs on Linux.
- `src/gpui_wgpu.rs`: declares and exports it.
