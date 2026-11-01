# Changelog

Changes a consumer of these crates would notice. Development history is in the
commit messages.

## Unreleased

Distribution is by Git revision for now; see the [quick start](README.md#quick-start).

### Added

- Cargo feature groups on `gpui-kit-material`, with `full` naming every group; see the
  [README](README.md#crates). `default` still enables everything.
- A `dynamic` feature on `gpui-kit-material-color`, on by default.
- `MaterialMenu::max_height` and `ExposedDropdown::menu_max_height`: past it a menu
  scrolls, with the keyboard's item kept in view and the selected item shown on open.
  A `MenuPopup` also scrolls rather than outgrow its window.
- `MaterialContextMenu`: a Material menu that opens at the pointer on a right press.
- A published `[patch]` block for downstream workspaces, built from Git by CI.
- `NoticeKind::Warning`, for a notice that needs attention when nothing has failed.
- `rust-version` 1.99.1, the toolchain this workspace is verified with.
- `[package.metadata.docs.rs]`, `deny.toml`, and CI checks of what each package ships.
- `ExposedDropdown::full_width`: the field fills its parent and the menu matches it.

- `TintFamily`, and `ColorRole`s for its seven container and content pairs: hue families
  harmonized toward each scheme's primary, as Material's custom colors are.
- `ListLeading::Tinted`: a `tinted_icon` as a list item's leading element.

- `TextFieldVariant::Hero`: a page's one main input, a 72dp pill with a ring while focused
  or populated, whose trailing slot takes a labeled button.
- `TextFieldShape`, on `TextField` and `ExposedDropdown`: medium corners for a form that
  rounds its fields with its surfaces.

- `ButtonTone`, on `MaterialButton`, `IconButton` and `ConnectedButtonGroup`: neutral and
  accent roles for a variant, for screens that need more than one level of action.

- `SearchFieldStyle`, `CardVariant::Surface` and `MaterialDialog::container`: a search bar,
  card and dialog that sit on the surface, for a page whose ground is a step below it.

- `ListItem::switch_icons`: the handle icons of a list item's switch.

- `Stepper`: numbered steps joined by hairlines, for a flow a person walks through once.

### Changed

- **Breaking:** with `default-features = false`, `gpui-kit-material` now has no
  components until a group or `full` is enabled.
- **Breaking:** the `frame-profiler` and `instruction-bench` features moved to
  `gpui-kit-material-gallery`. A consumer wanting Kit's profiler enables `gpui-kit/profiler`.
- **Breaking:** the Gallery and benches moved out of `gpui-kit-material`'s examples:
  `cargo run -p gpui-kit-material-gallery`, and `-p gpui-kit-material-bench --bin engine_bench`.
- `gpui-kit-material`'s `serde` feature now controls the color crate's, which was always on.
- `advance_component_theme` also projects Base's scrollbar colors and `TextView`'s
  link and table-head colors, so they follow the palette instead of Kit's default theme.
- `Tint` paints its own `TintFamily`. Its old mapping onto the container roles gave one
  color for blue, teal, pink and purple in every source palette.
- Kit is pinned by revision (`bb85882`) instead of `branch = "main"`, so a consumer
  without this workspace's lockfile gets Component at the same revision as Base.
- `Entity::update_for_render` is dev-only, behind `frame-observation`; a consumer's
  engine keeps upstream's API here.
- `LineLayoutCache::layout_wrapped_line` is back, as upstream's zero-spacing form.
- Performance tooling judges the targets in `AGENTS.md`: the 500 FPS, 0.3 ms redraw and
  16 ms first-frame limits are gone, and the first frame and Material list are judged
  against Kit.

- Breaking: `ChipSetItem::new` takes a key first. It stays with the chip as the set is
  reordered, and the chip's focus is kept under it.
- Breaking: `TimePickerLabels` has a required `separator` field, for what stands between
  a time's hours and minutes.
- Text wrapping: `?` and `!` stay off the start of a line, as other closing marks do. A
  long URL no longer wraps before `?`; it still wraps at `/` and `&`.

### Fixed

- A menu item's label or supporting text too long for the menu ends in an ellipsis
  instead of stopping mid-letter.
- Each crate ships the MIT text its `license` names, and the Apache notices.
- The site publishes IBM Plex's OFL and the Material Symbols license and notice.
- `gpui-kit-material-gallery` and `-gallery-web` declare OFL-1.1 and Apache-2.0. The
  Gallery's fonts are in their own unpublished crate, not in the library's package.
