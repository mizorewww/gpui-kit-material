# gpui-pre-web source

Complete published `gpui-pre-web` 0.3.10 crates.io archive (Apache-2.0,
`LICENSE-APACHE` retained). `SOURCE.json` records the archive SHA-256, every
original file hash, and the original and patched hash of each changed file.
The archive ships no `.cargo_vcs_info.json`; no Git revision is claimed.

Submission counting (`src/window.rs`): `successful_frame_submissions` counts
`WgpuRenderer::draw` calls that report a submitted frame, for the GPUI
`WindowSubmissionStats` patch (`vendor/gpui-pre/frame-submission-patch.md`).
Browser-controlled vsync stays `VsyncStatus::Unsupported`.

The package-level `Cargo.lock` is not retained; the workspace lockfile owns
resolution. `gpui-fast-web` carries the identical `window.rs`.
