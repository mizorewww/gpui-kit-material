# Changelog

Changes a consumer of these crates would notice. Development history is in the
commit messages.

## Unreleased

Distribution is by Git revision for now; see the [quick start](README.md#quick-start).

### Added

- A published `[patch]` block for downstream workspaces, built from Git by CI.
- `rust-version` 1.99.1, the toolchain this workspace is verified with.
- `[package.metadata.docs.rs]`, `deny.toml`, and CI checks of what each package ships.

- `TextFieldVariant::Hero`: a page's one main input, a 72dp pill with a ring while focused
  or populated, whose trailing slot takes a labeled button.
- `TextFieldShape`, on `TextField` and `ExposedDropdown`: medium corners for a form that
  rounds its fields with its surfaces.

### Changed

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
- `gpui-kit-material-gallery-web` declares OFL-1.1 and Apache-2.0. `gpui-kit-material`
  excludes `examples/`, which holds the Gallery's fonts, from its package.
