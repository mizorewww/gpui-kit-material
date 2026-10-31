# gpui-pre-wgpu source

**Dev-only overlay.** This crate is vendored only for the gallery's and the profiler's frame observation, and its manifest turns on the engine's `frame-observation` feature. A consumer does not patch it; see "Consumer patch set" in `docs/ARCHITECTURE.md`.

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
  on Linux and the web, so `cargo test -p gpui-pre-wgpu --lib present_mode` runs on Linux.
- `src/gpui_wgpu.rs`: declares and exports it.
- The tests in `src/cosmic_text_system.rs` and the shader-validation tests in `src/wgpu_renderer.rs`
  are compiled out (`#[cfg(any())]`). They read fonts from Zed's repository and use a
  `naga` dev-dependency, neither of which the published package has, so the test
  target could not build and the present mode rules above could not run. They never
  ran here, so nothing that used to be checked is lost.
