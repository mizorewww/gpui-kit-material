# Source

This directory vendors `gpui-pre-apple` 0.3.12 from crates.io:
<https://crates.io/crates/gpui-pre-apple/0.3.12>.

Published archive SHA-256:
`a460605ec4e8cbcc9b213462a84b1d6cc3efe4ed8f0e18669659f07b2d5c6aec`.
The upstream Apache 2.0 license is retained in `LICENSE-APACHE`. The nested
`vendor/gpui` source is part of the published crate and supplies Metal shader
build definitions.

`SOURCE.json` records every original file hash and the original and patched
hash of each changed or added file. The archive ships no `.cargo_vcs_info.json`.
The package-level `Cargo.lock` is not retained; the workspace lockfile owns
resolution.

Text patches (`src/text_system.rs`, `src/open_type.rs`, added
`tests/font_fallback.rs`; `Cargo.toml` adds the `font_fallback` `[[test]]`):

- Fonts without an `m` glyph are no longer skipped at load (valid non-Latin
  fonts); GPUI measures `em`/`ch` through fallback instead.
- App-private fallback fonts are placed in the CoreText cascade from their
  concrete descriptors, in declared order, and previously selected fonts get
  their cascades rebuilt after a later font registration without dropping font
  IDs still used by shaped frames.
- Color-bitmap glyph support is detected once per font ID from the presence of
  an `sbix` table instead of matching the Apple Color Emoji PostScript names.
