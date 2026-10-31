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

- `src/motion.rs`: CSS reversing-adjusted start values, reversal duration and
  negative-delay scaling, plus explicit per-transition reduced-motion policy.
  The default finishes transitions immediately under reduced motion; callers
  may continue non-spatial effects. Keyframe playback is unchanged.
  `TransitionPlayback<T>` is a caller-owned handle that keeps one transition's
  clock and interruption/reversal history across unmounts (for virtualized
  rows) using the same sampler as `transition_with_status`; it retains no
  view, window or scheduled callback.
- `src/lib.rs`: public exports `ReducedMotion` and `TransitionPlayback`.
- `src/text_selection.rs`: `Hitbox` literals add `additional_bounds: None`,
  required by the GPUI `Hitbox` field patch
  (`vendor/gpui-pre/hitbox-extension-patch.md`). No behavior change.
- `src/toolbar.rs`: `Toolbar::axis(Axis)`. Upstream handles only Left/Right
  and always reports a horizontal orientation, so a vertical toolbar needed
  a private copy of the subtree-constrained focus walk and still announced
  "horizontal". A vertical axis moves focus with Up/Down (Left/Right no
  longer move it) and reports `Orientation::Vertical`; the default stays
  horizontal and unchanged. Test: `a_vertical_toolbar_roves_with_up_and_down`.

- `src/slider.rs`, right to left: `SliderState` records whether it last rendered
  under GPUI's right-to-left layout direction, and pointer math then measures a
  horizontal slider from its right edge. Layout mirrors the thumb and indicator.
- `src/slider.rs`: three narrow corrections. Dragging snapped with
  `(value / step).round() * step`, so steps were counted from 0 rather than
  `min` (a 3..100 slider with step 5 landed on 10, not 8 or 13); values now
  snap to `min + k * step` inside the range.
  The accessibility Increment/Decrement actions set a single value, which
  collapsed a range; they now move only the range's end (never past its
  start). `Slider::track_focus(&FocusHandle)` makes the `Role::Slider` node
  itself focusable (observed by `test_support`), so keyboard focus and
  assistive technology land on the slider instead of an unnamed wrapper.
  Tests: `dragging_snaps_from_min`,
  `accessibility_steps_move_only_a_ranges_end`.

- `src/dialog.rs`: `Dialog::accessibility_label`. The dialog host is the
  `Role::Dialog` node but upstream gives it no name and no way to set one,
  so modal dialogs were announced as an unnamed "dialog". The label (a
  caller's headline, typically) is set on the host; without it nothing
  changes. Test: `a_dialog_takes_an_accessible_name`.

These patches require source unification for both the Kit Git dependency and
any registry Base dependency in the consuming workspace. Cargo patches are not
inherited by downstream consumers of a published Material crate.

Validation must include the retained compatibility tests and upstream behavior
before this source change is accepted. This source/provenance update itself
does not assert that compilation or runtime checks have passed.
