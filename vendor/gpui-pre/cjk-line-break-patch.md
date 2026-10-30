# CJK and no-break line wrapping

Files: `src/text_system/line_wrapper.rs`, `src/text_system/line_layout.rs`,
`src/text_system.rs`, added `src/text_system/line_break.rs`. `gpui-fast` carries
the same rules in `src/fast/line_break.rs`.

Upstream may break before any character that is not a word character, so a
full-width comma or closing bracket can open a line, and a no-break space does
not hold anything together. Both wrapping paths (`LineWrapper` for fragments and
truncation, `LineLayout::compute_wrap_boundaries` for shaped text used by
`StyledText`) now ask `line_break::allows_break_between`.

- The punctuation rules (UAX #14 classes CL, CP, EX, IS, NS and OP; W3C clreq and
  jlreq) come from `gpui-fast` 0.1.12's Apache-2.0 `src/fast/line_break.rs`, so
  both backends wrap alike.
- Local to this workspace, in both backends: no break beside no-break spaces or
  word joiners (`is_glue`; UAX #14 GL and WJ). CoreText shapes U+2060 to no glyph,
  so shaped wrapping also checks the text just before each glyph.
- Local to this workspace, in both backends: where no allowed break fits and the
  overflow would leave punctuation opening a line, the previous character moves
  down with it (`forced_break_carries_previous`), unless it may not start a line
  either. Lines still break where they overflow when nothing else is possible.

Taken from the course2md desktop's `zed-cjk-line-break` patch, adapted to these
wrappers. Latin text, code and URLs still wrap as before.

Regression command:

```sh
cargo test -p gpui-pre --lib text_system
cargo test -p gpui-fast --lib text_system
```
