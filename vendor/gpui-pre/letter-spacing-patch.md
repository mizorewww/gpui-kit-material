# Letter spacing

Source: the published Apache-2.0 `gpui-pre` 0.3.10 package (`SOURCE.json`), the
same in `vendor/gpui-fast`. Files: `src/style.rs`, `src/styled.rs`,
`src/text_system.rs`, `src/text_system/line_layout.rs`,
`src/text_system/line_wrapper.rs`, and the text element (`src/elements/text.rs`,
in Fast `src/fast/text.rs`).

**A needed feature, not a bug fix.** Upstream GPUI has no tracking, so nothing here
corrects upstream behavior. It can be reverted alone or proposed upstream alone: it
depends on no other local patch.

GPUI's text style has no tracking, so Material's type scale could record its
letter spacing but not paint it. No platform text system is involved in this patch:
spacing is applied to the shaped line, so every backend (CoreText, DirectWrite,
cosmic-text, the web) gets the same result from one implementation.

- `TextStyle::letter_spacing` (absolute length, cascading) and
  `Styled::letter_spacing`.
- `apply_letter_spacing` moves each glyph by `spacing` per preceding visual advance
  and grows the line by one `spacing` per advance, as CSS does, including after the
  last character. Glyphs at the same origin (combining marks) stay together.
  Carets and hit tests read the moved positions.
- Zero spacing leaves every path as it was: the cache sees a zero in its key and
  `apply_letter_spacing` returns at once.
- `LineLayoutCache::layout_line_spaced` and `layout_wrapped_line_spaced` add the
  spacing to the cache key, so a spaced line is shaped once and reused like any
  other. `layout_line` keeps its signature and passes zero, as does `shape_text`, and zero
  leaves the layout untouched. `layout_wrapped_line`, whose one caller is
  `shape_text_spaced`, became the spaced form.
- `WindowTextSystem::shape_text_spaced`, used by the text element.
- `LineWrapper::letter_spacing`, added to each character's width by truncation, so
  a truncated line is measured as it paints. A pooled wrapper's handle starts at
  zero.
- Fast's text measurement compares the spacing before reusing a kept measurement.

Limits:

- Fast's kept measurements compare the spacing as written, so a pixel-sized line with
  a spacing in rems is not measured again when the rem changes alone.
- Ligatures are not disabled. CSS turns them off under non-zero spacing, but GPUI
  fonts opt in through `FontFeatures` chosen by the caller.
- `shape_line` callers, which include Kit's editors, pass no spacing. An editor
  shapes its own lines, so the value of a text field is not tracked.

Regression command:

```sh
cargo test -p gpui-fast --lib letter_spacing
cargo test -p gpui-pre --lib letter_spacing
```
