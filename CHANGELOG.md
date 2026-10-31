# Changelog

Changes a consumer of these crates would notice. Development history is in the
commit messages.

## Unreleased

Distribution is by Git revision for now; see the [quick start](README.md#quick-start).

### Added

- Cargo feature groups on `gpui-kit-material`, with `full` naming every group; see the
  [README](README.md#crates). `default` still enables everything.
- A `dynamic` feature on `gpui-kit-material-color`, on by default.
- A published `[patch]` block for downstream workspaces, built from Git by CI.
- `rust-version` 1.99.1, the toolchain this workspace is verified with.
- `[package.metadata.docs.rs]`, `deny.toml`, and CI checks of what each package ships.

### Changed

- **Breaking:** with `default-features = false`, `gpui-kit-material` now has no
  components until a group or `full` is enabled.
- **Breaking:** the `frame-profiler` and `instruction-bench` features moved to
  `gpui-kit-material-gallery`. A consumer wanting Kit's profiler enables `gpui-kit/profiler`.
- **Breaking:** the Gallery and benches moved out of `gpui-kit-material`'s examples:
  `cargo run -p gpui-kit-material-gallery`, and `-p gpui-kit-material-bench --bin engine_bench`.
- `gpui-kit-material`'s `serde` feature now controls the color crate's, which was always on.
- Kit is pinned by revision (`bb85882`) instead of `branch = "main"`, so a consumer
  without this workspace's lockfile gets Component at the same revision as Base.
- `Entity::update_for_render` is dev-only, behind `frame-observation`; a consumer's
  engine keeps upstream's API here.
- `LineLayoutCache::layout_wrapped_line` is back, as upstream's zero-spacing form.
- Performance tooling judges the targets in `AGENTS.md`: the 500 FPS, 0.3 ms redraw and
  16 ms first-frame limits are gone, and the first frame and Material list are judged
  against Kit.

### Fixed

- Each crate ships the MIT text its `license` names, and the Apache notices.
- The site publishes IBM Plex's OFL and the Material Symbols license and notice.
- `gpui-kit-material-gallery` and `-gallery-web` declare OFL-1.1 and Apache-2.0. The
  Gallery's fonts are in their own unpublished crate, not in the library's package.
