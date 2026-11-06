# Workspace compatibility patches

This directory contains `crates/base` from Longbridge GPUI Kit Git revision
`b2aaa3e5f9a387dea74981586bfca33e7e08420d`, licensed under Apache-2.0.
The upstream package declares version `0.9.0`, but this checkout is **not** the
published crates.io `0.9.0` source. `SOURCE.json` records every upstream file's
SHA-256, the normalized manifests, and the original and patched SHA-256 of
each local patch file. `.cargo_vcs_info.json` is a locally written
marker identifying the Git checkout actually vendored here.

The root and example manifests expand workspace-inherited values from that
same upstream revision. They do not inherit dependencies or lint settings from
the Material workspace. GPUI snapshot dependencies remain pinned to `=0.3.11`;
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
- `src/toolbar.rs`: `Toolbar::axis(Axis)`. Upstream handles only Left/Right
  and always reports a horizontal orientation, so a vertical toolbar needed
  a private copy of the subtree-constrained focus walk and still announced
  "horizontal". A vertical axis moves focus with Up/Down (Left/Right no
  longer move it) and reports `Orientation::Vertical`; the default stays
  horizontal and unchanged. Test: `a_vertical_toolbar_roves_with_up_and_down`.

- `src/slider.rs`: the bullets below, offered upstream as the branches
  `material/slider-accessibility-steps` (stacked on `material/slider-step-from-min`),
  `material/slider-min-above-default-max`, `material/slider-thumb-disabled` and
  `material/slider-track-focus`. Right-to-left pointer math and a range's
  `Role::Group` node are not patched here: Material does both
  (`crates/material/src/components/slider/mirrored.rs` and `range_group.rs`).
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
  `accessibility_steps_move_only_the_stepped_thumb`.
- `src/slider.rs`, four more. A drag kept the pointer's percentage, so a step-25 slider
  dropped at 30% had value 25 with its handle at 30%; the handle now stands at the value.
  Increment and Decrement went through `set_value`, which emits nothing, so a value owned by
  events missed a screen reader's edit; `SliderState::step_thumb` now emits `Change` when the
  thumb moved, and steps either thumb of a range for a caller whose thumbs are nodes of their
  own. `SliderState::new().min(150.).max(200.)` panicked in `min`, whose `clamp` saw the
  default max of 100; it no longer clamps by a pair that can be inverted. `SliderThumb`
  reports its `disabled` flag on its accessibility node. Tests, in `crates/material/tests/slider_state.rs`
  since this crate's own do not run in the workspace:
  `a_dropped_handle_stands_at_its_value`, `an_accessibility_step_reports_a_change`,
  `each_range_thumb_steps_by_itself`, `min_can_be_set_above_the_default_max_before_max`.
- `src/slider.rs`, steps. A step of 0, a negative step or a non-finite one divided by
  zero in `snap_to_step`, and the NaN clamped to `max`, so a drag threw the handle to the
  end; such a step now leaves the value continuous. Increment and Decrement moved by
  `step_value()`, so a continuous slider took 0 and reported a 0 step; the new
  `SliderState::accessibility_step` is the step, or a hundredth of the range when the step
  is not positive and finite, as Material's keyboard does, and Base's slider node reports
  it and moves by it. `step_thumb` added its delta without snapping (step 10 at 33 went
  to 43 where the arrow keys give 40); it now lands on the grid counted from `min`, as the
  keyboard does. Tests, in `crates/material/tests/slider_state.rs`:
  `a_zero_step_drags_continuously`, `an_accessibility_step_lands_on_the_step_grid`,
  `a_continuous_slider_steps_by_a_hundredth_of_its_range`; and in Base,
  `dragging_snaps_from_min`, `accessibility_steps_land_on_the_step_grid`.
- `src/slider.rs`, step guards. `step_thumb` stepped any value: a NaN value became NaN
  plus the delta, which `min`/`max` turned into `max`, so one Increment threw the thumb
  to the end; a `min` above `max` and a range whose start is past its end were stepped
  through bounds that cannot hold. As Material's keyboard (`stepped_value`) does, such a
  value now comes back unchanged and `SliderState::step_thumb` emits nothing. Tests, in
  `crates/material/tests/slider_state.rs`:
  `stepping_leaves_an_invalid_value_or_range_alone`,
  `a_range_thumb_snaps_to_the_grid_and_stops_at_the_other`; and in Base,
  `stepping_leaves_an_invalid_value_or_range_alone`.

- `src/dialog.rs`: `Dialog::accessibility_label`. The dialog host is the
  `Role::Dialog` node but upstream gives it no name and no way to set one,
  so modal dialogs were announced as an unnamed "dialog". The label (a
  caller's headline, typically) is set on the host; without it nothing
  changes. Test: `a_dialog_takes_an_accessible_name`.

- `src/time_field.rs`: `TimeField::separator`. The field writes `:` between its
  segments, a string its caller cannot change, and some locales write `7.05`. The
  default stays `:`. Test: `the_time_separator_comes_from_the_labels` in Material.

These patches require source unification for both the Kit Git dependency and
any registry Base dependency in the consuming workspace. Cargo patches are not
inherited by downstream consumers of a published Material crate.

Validation must include the retained compatibility tests and upstream behavior
before this source change is accepted. This source/provenance update itself
does not assert that compilation or runtime checks have passed.
