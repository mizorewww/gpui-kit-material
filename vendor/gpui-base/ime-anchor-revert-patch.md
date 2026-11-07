# IME caret report taken out

Source: Git `b881cadb` (`crates/base`). Files: `src/input/base/element.rs`,
`src/input/base/state.rs`.

Upstream merged a report of the focused input's caret to the platform (#3827): each
paint compared the caret with the last one reported and, when it had moved, called
`Window::invalidate_character_coordinates`, which asks for another frame to tell the
platform. Every caret move therefore drew a follow-up frame, which broke upstream's
`ordinary_input_schedules_no_followup_frame`, and upstream is taking it out again.

This patch takes out the same lines ahead of that: the `ime_anchor` field, its
comparison in paint, and the test `a_focused_input_reports_a_moved_caret_to_the_platform`.
Nothing else changes. Native platforms still ask for the caret themselves when an
IME needs it. The web backends (`vendor/gpui-pre-web`, `vendor/gpui-fast-web`) ask
the focused input for it when a composition starts (`compositionstart`).

Drop this patch when Base is moved to an upstream revision without #3827.
