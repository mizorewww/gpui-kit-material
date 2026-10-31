# gpui-pre-web source

**A consumer patches this crate** for the IME position below; see "Consumer patch set" in `docs/ARCHITECTURE.md`. Its manifest leaves the engine's `frame-observation` off. Only `crates/gallery-web` selects this crate's `frame-observation` feature.

Complete published `gpui-pre-web` 0.3.10 crates.io archive (Apache-2.0,
`LICENSE-APACHE` retained). `SOURCE.json` records the archive SHA-256, every
original file hash, and the original and patched hash of each changed file.
The archive ships no `.cargo_vcs_info.json`; no Git revision is claimed.

Submission counting (`src/window.rs`, `Cargo.toml`), behind the
`frame-observation` feature: `successful_frame_submissions` counts
`WgpuRenderer::draw` calls that report a submitted frame, for the GPUI
`WindowSubmissionStats` patch (`vendor/gpui-pre/frame-submission-patch.md`).
Browser-controlled vsync stays `VsyncStatus::Unsupported`.

IME position (`src/window.rs`, `src/ime_mirror.rs`): upstream's
`update_ime_position` did nothing, so a browser's IME panel opened at the hidden
input in the page's corner. It now moves that input to the caret.

The package-level `Cargo.lock` is not retained; the workspace lockfile owns
resolution. `gpui-fast-web` carries the identical `window.rs`, `ime_mirror.rs` and
feature. Canvas sizing is upstream's: `scripts/check-gallery-dpr.py` validates it.
