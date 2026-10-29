# Vendored test harness repairs

Source: the published Apache-2.0 `gpui-pre` 0.3.10 package (`SOURCE.json`).
Test-only; no runtime behavior changes.

- `src/svg_renderer.rs`: the SVG tests `include_bytes!` fonts from
  `../../../assets/fonts/...`, a Zed monorepo path absent from the published
  archive. They now read `tests/fonts/...`; the fonts are restored from the
  Zed revision recorded in the manifest (`zed-rev`), with licenses and
  hashes in `tests/fonts/README.md`.
- `src/style.rs`: three tests used `#[perf]` from Zed's unpublished
  `util_macros`; they run as ordinary `#[test]`s with unchanged assertions.
- `Cargo.toml.orig` is not retained; `Cargo.toml` is the normalized published
  manifest plus the additions in `line-layout-options-patch.md`.
