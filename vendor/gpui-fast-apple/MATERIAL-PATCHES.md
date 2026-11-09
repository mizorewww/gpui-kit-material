# gpui-fast-apple source

gpui-fast `main` at `97439ecf`, packaged by upstream's own release step, as
`scripts/vendor-gpui-fast.py` reproduces it (version kept at 0.1.13). `SOURCE.json`
records every upstream file hash and the original and patched hash of each changed
file. The package-level `Cargo.lock` is not vendored.

Merged upstream and no longer patched here: fonts without an `m` (#111),
application fonts in macOS fallback cascades, rebuilt when fonts are added (#112), and
fonts with an `sbix` table drawn as color bitmap fonts (#119, upstream's
`src/fast/color_bitmap_fonts.rs`), which the Gallery's conformance emoji font
(`NotoColorEmoji-Probe-sbix.ttf`) needs.

Local, test and warning only, no runtime change:

- `tests/font_fallback.rs` (added, `[[test]]` in `Cargo.toml`): application
  fallback, cascade rebuild and sbix glyphs with the Gallery's fonts.
- `src/fast/rounded_clip_shader.rs`: `allow(dead_code)` on `stage` under
  `runtime_shaders`, a warning fix.
