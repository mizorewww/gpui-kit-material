# gpui-fast-wgpu source

gpui-fast `main` at `bbdbc42d`, packaged by upstream's own release step, as
`scripts/vendor-gpui-fast.py` reproduces it (version kept at 0.1.13). `SOURCE.json`
records every upstream file hash and the original and patched hash of each changed
file. The package-level `Cargo.lock` is not vendored.

Vendored because `gpui-fast` here is gpui-fast `main`, whose backends differ from the
published 0.1.13. Test harness only, no runtime change:

- The tests in `src/cosmic_text_system.rs` and the shader-validation tests in
  `src/wgpu_renderer.rs` and `src/fast/rounded_clip.rs` are compiled out
  (`#[cfg(any())]`). They read fonts from Zed's repository and use a `naga`
  dev-dependency, neither of which the published package has.
- `src/fast/layers/tests.rs` and `src/fast/layers/rounded_clip_texture_tests.rs` find
  `rounded_clip_fixtures.rs` in `vendor/gpui-fast` instead of Zed's `crates/gpui`, for
  a build with `test-support`.
