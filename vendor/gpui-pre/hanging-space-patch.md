# Hanging overflow space

Source: the published Apache-2.0 `gpui-pre` 0.3.10 package (`SOURCE.json`), the
same in `vendor/gpui-fast`. Files: `src/text_system/line_wrapper.rs`,
`src/text_system/line_layout.rs`.

**A real upstream bug.** A space that overflowed a row started the next one, so
wrapped text could begin with a stray space (18 of 5679 rows of ordinary sentences
in the patch audit), and a word that fit could be pushed down by the space after
it. After content, an overflowing space now hangs past the edge and the next word
is the break. Leading indentation still wraps. Four upstream expectations encoded
the old rows and now assert the corrected ones.

Editors may rely on preserving those spaces, so report it upstream rather than
assuming the fix is universally wanted.
