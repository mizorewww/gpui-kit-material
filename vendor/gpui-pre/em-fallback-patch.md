# em metrics for fonts without `m`

Source: the published Apache-2.0 `gpui-pre` 0.3.10 package (`SOURCE.json`).
File: `src/text_system.rs`. `gpui-fast` carries the same change.

`TextSystem::em_width` and `em_advance` (used for `ch` and `em`) measure `m`
through platform fallback shaping when the font has no `m` glyph, instead of
failing for valid non-Latin fonts; Latin-font metrics are unchanged. This pairs
with `gpui-pre-apple` no longer refusing to load fonts without an `m` glyph
(`tests/font_fallback.rs` there).
