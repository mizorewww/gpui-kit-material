# Question and exclamation marks do not start a line

Source: the published Apache-2.0 `gpui-pre` 0.3.10 package (`SOURCE.json`), the
same in `vendor/gpui-fast`. Files: `src/text_system/line_wrapper.rs` (read by
`LineWrapper::wrap_line` and `LineLayout::compute_wrap_boundaries`); in Fast also a
comment in `src/fast/line_break.rs`.

`?` could start a wrapped line, so a Dialog title left its question mark alone on
the last row. Unicode's EX class forbids a break before it (UAX #14, LB13).
`is_word_char` now holds `?`, and `！` and `？` for the full-width forms. `!` was
already there, and Fast's `line_break` already kept the full-width forms from
starting a line; plain GPUI had nothing that did.

What this gives up: upstream kept `?` breakable so that a long URL could wrap there
(`foo?b=2`). Such a URL still wraps at `/` and `&`, and where it overflows when it
has neither. Other full-width closing punctuation (`，`, `。`) can still start a line
in the upstream backend, which does not have Fast's `line_break`.
