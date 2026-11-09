# gpui-fast-apple source

gpui-fast-apple 0.1.14 as published on crates.io, with the local changes below, recorded by
`scripts/vendor-gpui-fast.py --published 0.1.14`. `SOURCE.json` records the archive's
SHA-256, every upstream file hash and the original and patched hash of each changed
file. The package-level `Cargo.lock` is not vendored. Vendored for its regression
test only; it changes no runtime behavior.

Released in 0.1.14 and no longer patched here: fonts without an `m` (#111),
application fonts in macOS fallback cascades, rebuilt when fonts are added (#112), and
fonts with an `sbix` table drawn as color bitmap fonts (#119, upstream's
`src/fast/color_bitmap_fonts.rs`), which the Gallery's conformance emoji font
(`NotoColorEmoji-Probe-sbix.ttf`) needs.

Local, test and warning only, no runtime change:

- `tests/font_fallback.rs` (added, `[[test]]` in `Cargo.toml`): application
  fallback, cascade rebuild and sbix glyphs with the Gallery's fonts.
- `src/fast/rounded_clip_shader.rs`: `allow(dead_code)` on `stage` under
  `runtime_shaders`, a warning fix.
