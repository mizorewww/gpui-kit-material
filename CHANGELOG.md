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
- `MaterialMenu::max_height` and `ExposedDropdown::menu_max_height`: past it a menu
  scrolls, with the keyboard's item kept in view and the selected item shown on open.
  A `MenuPopup` also scrolls rather than outgrow its window.
- `ExposedDropdown::full_width`: the field fills its parent and the menu matches it.
- `NoticeKind::Warning`, for a notice that needs attention when nothing has failed.
- `MaterialContextMenu`: a Material menu that opens at the pointer on a right press.
- `TintFamily`, and `ColorRole`s for its seven container and content pairs: hue families
  harmonized toward each scheme's primary, as Material's custom colors are.
- `ListLeading::Tinted`: a `tinted_icon` as a list item's leading element.
- `TextFieldVariant::Hero`: a page's one main input, a 72dp pill with a ring while focused
  or populated, whose trailing slot takes a labeled button.
- `TextFieldShape`, on `TextField` and `ExposedDropdown`: medium corners for a form that
  rounds its fields with its surfaces.
- `MarkdownExtensions::block_surface`, in `gpui_kit::base::text`: the top-level blocks between
  `<!-- surface:name -->` and `<!-- /surface -->` draw inside a container the caller builds
  from theme roles. They stay the document's for selection, find, reveal and copy.
- `ButtonTone`, on `MaterialButton`, `IconButton` and `ConnectedButtonGroup`: neutral and
  accent roles for a variant, for screens that need more than one level of action.
- `SearchFieldStyle`, `CardVariant::Surface` and `MaterialDialog::container`: a search bar,
  card and dialog that sit on the surface, for a page whose ground is a step below it.
- `ListItem::switch_icons`: the handle icons of a list item's switch.
- `Stepper`: numbered steps joined by hairlines, for a flow a person walks through once. It
  is a list to assistive technology, with `state_labels` for what each step's state is called.
- `status_fact_chip`: one of several facts in a row, with an id and accessible name. Facts
  that need attention and work under way are filled. `status_chip_outlined`, its earlier
  name without them, stays as a deprecated alias.
- `Tint::Danger`, so `Tint::ALL` has eight entries. Code sizing an array by it needs the new length.
- `PullToRefresh::nested_scroll`: a scroller inside the refreshing area takes the wheel first.
- `ListItem::expressive`: the kit's expressive row, tighter, whose corners round from 4dp to
  16dp when it is selected, pressed or focused.
- `MaterialDivider::subhead`: the kit's divider with subhead.

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
- `LineLayoutCache::layout_wrapped_line` is gone from both engines: `layout_wrapped_line_spaced`
  replaces it, and nothing outside the crate could reach it.
- Performance tooling judges the targets in `AGENTS.md`: the 500 FPS, 0.3 ms redraw and
  16 ms first-frame limits are gone, and the first frame and Material list are judged
  against Kit.

- A `MaterialSlider` handle dropped between steps stands at its value, and a screen reader's
  Increment and Decrement emit `SliderEvent::Change`. A range's thumbs each take those actions and
  report their step and when they are disabled. Keys with Cmd, Ctrl or Alt pass through a slider.
- A browser IME's Enter on a candidate no longer presses a field's Enter, and replacing an emoji
  no longer inserts U+FFFD. The hidden input over the caret no longer takes clicks.
- `icons::set_style_pending`: icons wait only for the font of their style, and a caller's own
  asset never waits.
- A `TextFieldVariant::Hero` field set through `variant` without a label is named by its
  placeholder, and with neither logs a warning once.
- **Breaking:** `TextField::hero` takes the label, which is the field's only accessible name.
- **Breaking:** the enums that gained a variant here (`Tint`, `NoticeKind`, `TextFieldVariant`,
  `CardVariant`) are now `#[non_exhaustive]`: a `match` on one outside this crate needs a `_`
  arm, and adding to them later is no longer breaking. Before this, the added variants
  themselves broke any exhaustive match.
- A tile's label has the list headline's weight.
- A `TextField::hero` that is `required` shows its mark on the placeholder, or on the label
  in its place, and an `ExposedDropdown` in the hero variant shows its label as a placeholder.
- `IconButton::tone` sets a standard button's icon color at rest, where it did nothing.

- Breaking: `ChipSetItem::new` takes a key first. It stays with the chip as the set is
  reordered, and the chip's focus is kept under it.
- Breaking: `TimePickerLabels` has a required `separator` field, for what stands between
  a time's hours and minutes.
- Text wrapping: `?` and `!` stay off the start of a line, as other closing marks do. A
  long URL no longer wraps before `?`; it still wraps at `/` and `&`.

### Fixed

- A sub-menu fits the room past its owner item, so a tall one no longer leaves the window.
- The first arrow key in a menu moves from its selected item, not from the top or the end.
- A keyboard-focused menu item paints the focus layer as well as the ring.
- An accent, filled toggle shows whether it is selected, on `MaterialButton`, `IconButton`
  and `ConnectedButtonGroup`.
- A `Stepper`'s labels wrap at narrow widths instead of overlapping.
- A multi-line hero field keeps its text off the ring; its ring fades as focus changes, and
  it has a hover layer.
- `MaterialDialog::container` applies to a full-screen dialog.
- A side sheet's back button is the kit's narrow icon button, and an `Avatar`'s placeholder
  glyph has the kit's ink height.
- Engine: a view drawn around a nested view that writes as it paints no longer grows with
  every frame.

- A menu item's label or supporting text too long for the menu ends in an ellipsis
  instead of stopping mid-letter.
- Each crate ships the MIT text its `license` names, and the Apache notices.
- The site publishes IBM Plex's OFL and the Material Symbols license and notice.
- `gpui-kit-material-gallery` and `-gallery-web` declare OFL-1.1 and Apache-2.0. The
  Gallery's fonts are in their own unpublished crate, not in the library's package.
