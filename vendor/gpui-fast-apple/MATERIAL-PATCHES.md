# gpui-fast-apple source

gpui-fast `main` at `bbdbc42d`, packaged by upstream's own release step, as
`scripts/vendor-gpui-fast.py` reproduces it (version kept at 0.1.13). `SOURCE.json`
records every upstream file hash and the original and patched hash of each changed
file. The package-level `Cargo.lock` is not vendored.

Merged upstream and no longer patched here: fonts without an `m` (#111) and
application fonts in macOS fallback cascades, rebuilt when fonts are added (#112).

- `src/text_system.rs`: a font with an `sbix` table is drawn as a color bitmap
  font, as Apple Color Emoji is; upstream recognizes only Apple's emoji by name. The
  Gallery's conformance emoji font (`NotoColorEmoji-Probe-sbix.ttf`) needs it. Not
  yet offered upstream.
- `tests/font_fallback.rs` (added, `[[test]]` in `Cargo.toml`): application
  fallback, cascade rebuild and sbix glyphs with the Gallery's fonts.
- Dev-only, behind `frame-observation` (`Cargo.toml`): `src/metal_renderer.rs`
  `draw_with_status` with a per-draw autorelease pool, `nextDrawable` wait and GPU
  timestamps; `src/fast/layers/raster.rs` GPU timestamps for raster tiles.
- `src/fast/rounded_clip_shader.rs`: `allow(dead_code)` on `stage` under
  `runtime_shaders`, a warning fix.
