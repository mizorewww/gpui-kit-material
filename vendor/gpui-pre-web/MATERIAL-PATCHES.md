# gpui-pre-web source

Complete published `gpui-pre-web` 0.3.10 crates.io archive (Apache-2.0,
`LICENSE-APACHE` retained). `SOURCE.json` records the archive SHA-256, every
original file hash, and the original and patched hash of each changed file.
The archive ships no `.cargo_vcs_info.json`; no Git revision is claimed.

Canvas sizing (`src/canvas_size.rs` added, `src/gpui_web.rs` declares it,
`src/window.rs` resize observer): upstream derived the CSS (logical) viewport
by dividing the `device-pixel-content-box` size by `devicePixelRatio`. Under
Chromium DPR emulation those disagree, which halved the logical viewport. The
patch keeps CSS layout size from `contentRect` and uses the device-pixel box
only for the backing store, and only when it is integral and within one pixel
of CSS × DPR (either axis order); otherwise it uses rounded CSS × DPR. A change
in DPR or logical size alone now also triggers a resize. Unit tests live in
`canvas_size.rs`.

Submission counting (`src/window.rs`): `successful_frame_submissions` counts
`WgpuRenderer::draw` calls that report a submitted frame, for the GPUI
`WindowSubmissionStats` patch (`vendor/gpui-pre/frame-submission-patch.md`).
Browser-controlled vsync stays `VsyncStatus::Unsupported`.

The package-level `Cargo.lock` is not retained; the workspace lockfile owns
resolution. `gpui-fast-web` carries the identical `canvas_size.rs` and
`window.rs`.
