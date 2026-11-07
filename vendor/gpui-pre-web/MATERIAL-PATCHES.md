# gpui-pre-web source

**A consumer patches this crate** for the IME position below; see "Consumer patch set" in `docs/ARCHITECTURE.md`. Its manifest leaves the engine's `frame-observation` off. Only `crates/gallery-web` selects this crate's `frame-observation` feature.

Complete published `gpui-pre-web` 0.3.11 crates.io archive (Apache-2.0,
`LICENSE-APACHE` retained). `SOURCE.json` records the archive SHA-256, every
original file hash, and the original and patched hash of each changed file.
The archive ships no `.cargo_vcs_info.json`; no Git revision is claimed.

Submission counting (`src/window.rs`, `Cargo.toml`), behind the
`frame-observation` feature: `successful_frame_submissions` counts
`WgpuRenderer::draw` calls that report a submitted frame, for the GPUI
`WindowSubmissionStats` patch (`vendor/gpui-pre/frame-submission-patch.md`).
Browser-controlled vsync stays `VsyncStatus::Unsupported`.

IME position (`src/window.rs`, `src/ime_mirror.rs`, `src/events.rs`): upstream's
`update_ime_position` did nothing, so a browser's IME panel opened at the hidden
input in the page's corner. It now moves that input to the caret, and so does a
composition's start, which asks the focused input where its caret is
(`PlatformInputHandler::ime_candidate_bounds`): an input need not report every caret
move, which would draw a frame after each. Over the canvas there, the hidden input
takes no clicks and has no padding or border.

IME keys and edits (`src/events.rs`): upstream dispatched a key to GPUI before checking for a
composition, so the Enter committing a CJK candidate pressed a single-line field's Enter. A key
during a composition, or reported as keyCode 229, as Safari reports that Enter after
`compositionend`, now stays the IME's. An edit's UTF-16 prefix and suffix no longer split a
surrogate pair, which turned a replaced emoji into U+FFFD. Both rules are in the added `src/ime_edit.rs`,
which `crates/material/tests/ime_edit.rs` runs natively.

The package-level `Cargo.lock` is not retained; the workspace lockfile owns
resolution. `gpui-fast-web` carries the identical `window.rs`, `ime_mirror.rs`, `events.rs` and
feature. Canvas sizing is upstream's: `scripts/check-gallery-dpr.py` validates it.
