# Opt-in paragraph layout with cluster metadata

Source: the published Apache-2.0 `gpui-pre` 0.3.10 package (`SOURCE.json`).
Files: `src/text_system.rs`, `src/text_system/line_layout.rs`, added
`src/text_system/line_layout_options.rs`, `src/platform.rs`, added
`tests/line_layout_options.rs`, and `Cargo.toml` (dependency pin and `[[test]]`).

Upstream `layout_line` returns glyph positions only: callers cannot recover the
paragraph base direction, shaping clusters or caret edges needed for bidi text,
grapheme-safe selection and visual caret movement. This adds an opt-in path;
the legacy API and its results are unchanged.

- `PlatformTextSystem::layout_line_with_options` is a defaulted trait method
  that returns an error, so a backend without cluster metadata never silently
  falls back to glyph-only layout. The Apple backends implement it
  (`gpui-pre-apple`/`gpui-fast-apple` `VENDORED.md`/`MATERIAL-MIGRATION.json`).
- `TextSystem::layout_line_with_options` / `shape_line_with_options` validate a
  single line, a finite positive font size and run boundaries that exactly
  cover the text, then return `DetailedLineLayout` / `DetailedShapedLine`
  sharing the painted `LineLayout`.
- `LineLayoutCache` keeps a separate detailed cache (`DetailedCacheKey`
  includes the options), so legacy and metadata layouts never alias; it follows
  frame swap, reuse, truncation and font-generation clearing like the legacy
  caches.
- `line_layout_options.rs` defines `ParagraphDirection`, `LineLayoutOptions`,
  `ShapedCluster`, `GlyphAddress`, `DetailedLineLayout` and the validator
  `merge_grapheme_clusters`, which coalesces native clusters overlapping one
  extended grapheme and rejects interleaved or conflicting-bidi input.

`Cargo.toml` adds `unicode-segmentation = "=1.13.3"` for that grapheme
validation. The adding commit did not record why the pin is exact; the effect
is that extended-grapheme boundaries come from one fixed table set (1.13.3
implements Unicode 17.0.0, `UNICODE_VERSION`), so validation results and the
tests in `tests/line_layout_options.rs` cannot shift with a lockfile update.
1.13.3 was already the single version in the workspace `Cargo.lock` (used by
`gpui-base`, `gpui-component`, `cosmic-text`, the web/wgpu backends and
others); the commit's lockfile change only adds the dependency edge, so the pin
adds no second copy. `gpui-fast` carries the same pin; the web backends keep
their published `1.10` requirement, which that lockfile entry satisfies. Revisit the pin together with the tests when
upgrading Unicode data.

`em_width`/`em_advance` (used for `ch`/`em` metrics) measure `m` through
platform fallback shaping when the font has no `m` glyph, instead of failing
for valid non-Latin fonts; Latin-font metrics are unchanged. This pairs with
`gpui-pre-apple` no longer refusing to load fonts without an `m` glyph.

Regression command:

```sh
cargo test -p gpui-pre --test line_layout_options
```
