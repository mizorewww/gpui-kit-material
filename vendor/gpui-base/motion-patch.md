# Workspace compatibility patches

This directory contains `crates/base` from Longbridge GPUI Kit Git revision
`bb85882e1faab845d9e7be33b69a152879a90f69`, licensed under Apache-2.0.
The upstream package declares version `0.8.1`, but this checkout is **not** the
published crates.io `0.8.1` source. `SOURCE.json` records every upstream file's
SHA-256, the normalized manifests, and the original and patched SHA-256 of
each local patch file. `.cargo_vcs_info.json` is a locally written
marker identifying the Git checkout actually vendored here.

The root and example manifests expand workspace-inherited values from that
same upstream revision. They do not inherit dependencies or lint settings from
the Material workspace. GPUI snapshot dependencies remain pinned to `=0.3.10`;
optional GPUI Fast support and all upstream dependencies/features are retained.
Dependency resolution belongs to the consuming workspace's `Cargo.lock`; there
is no stale package-level lockfile from the older published archive. Upstream
development examples retain references to sibling story-web fonts and repository
fixtures not included in this Base-only snapshot. They are not claimed to run
standalone; the upstream package also excludes these examples from publication.

The local compatibility surface is restricted to:

- `src/a11y_disabled.rs`: transparent accessibility state presentation for
  disabled and busy controls.
- `src/button.rs`: caller-controlled `busy(bool)`, independent of disabled.
- `src/toggle.rs`: busy state and the nonfocusable activation/focus corrections.
- `src/motion.rs`: CSS reversing-adjusted start values, reversal duration and
  negative-delay scaling, plus explicit per-transition reduced-motion policy.
  The default finishes transitions immediately under reduced motion; callers
  may continue non-spatial effects. Keyframe playback is unchanged.
  `TransitionPlayback<T>` is a caller-owned handle that keeps one transition's
  clock and interruption/reversal history across unmounts (for virtualized
  rows) using the same sampler as `transition_with_status`; it retains no
  view, window or scheduled callback.
- `src/lib.rs`: public exports `ReducedMotion` and `TransitionPlayback`, and
  `RootTab`/`RootTabPrev` aliases for the `root::Tab`/`TabPrev` actions (the
  `root` module is private upstream), so an app can handle Root's focus-cycling
  keys itself, as the virtualized Gallery does.
- `src/text_selection.rs`: `Hitbox` literals add `additional_bounds: None`,
  required by the GPUI `Hitbox` field patch
  (`vendor/gpui-pre/hitbox-extension-patch.md`). No behavior change.
- `src/input/base/state.rs`: `InputBaseState::set_caret_color(Option<Hsla>)`,
  a caller caret colour that wins over the projected editor style. Kit's
  `Input`/`Textarea` re-project the whole style from the Component theme on
  every render, so a field whose caret follows its own state (Material's
  error caret) had no way to paint it. `None` restores the projected caret;
  nothing else in the style changes. Test:
  `a_caller_caret_color_wins_over_the_projected_style`.

These patches require source unification for both the Kit Git dependency and
any registry Base dependency in the consuming workspace. Cargo patches are not
inherited by downstream consumers of a published Material crate.

Validation must include the retained compatibility tests and upstream behavior
before this source change is accepted. This source/provenance update itself
does not assert that compilation or runtime checks have passed.
