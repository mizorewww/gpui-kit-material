# Workspace compatibility patches

This directory contains `crates/base` from Longbridge GPUI Kit Git revision
`46bf6d498fa101a3b11cc64e6d8c20d28d17b720`, licensed under Apache-2.0.
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

The local compatibility surface is restricted to two unmerged fork branches,
cherry-picked onto 46bf6d49 without conflicts:

- `src/motion.rs`: explicit per-transition reduced-motion policy
  (`ReducedMotion`). The default finishes transitions immediately under
  reduced motion; callers may continue non-spatial effects. Keyframe playback
  is unchanged. `TransitionPlayback<T>` is a caller-owned handle that keeps one
  transition's clock and interruption/reversal history across unmounts (for
  virtualized rows) using the same sampler as `transition_with_status`; it
  retains no view, window or scheduled callback. Fork branch
  `material/motion-transition-playback`, commits 59bd3b1e and d59cae42, written
  on the pre-squash CSS reversal that upstream merged as #3820.
- `src/lib.rs`: public exports `ReducedMotion` and `TransitionPlayback`.
- `src/slider.rs`: fork branch `material/slider-accessibility-steps`, commits
  43757c62 and 56d21f7d, on top of step snapping from `min` (merged upstream as
  #3822):
  - The accessibility Increment/Decrement actions set a single value, which
    collapsed a range, and went through `set_value`, which emits nothing, so a
    value owned by events missed a screen reader's edit. `SliderState::step_thumb`
    now moves one thumb (a range's end never past its start, or either thumb
    for a caller whose thumbs are nodes of their own) and emits `Change` when
    it moved. Tests: `accessibility_steps_move_only_the_stepped_thumb`, and in
    `crates/material/tests/slider_state.rs` (this crate's own tests do not run
    in the workspace) `an_accessibility_step_reports_a_change`,
    `each_range_thumb_steps_by_itself`.
  - Increment and Decrement moved by `step_value()`, so a continuous slider
    took 0 and reported a 0 step; the new `SliderState::accessibility_step` is
    the step, or a hundredth of the range when the step is not positive and
    finite, as Material's keyboard does, and Base's slider node reports it and
    moves by it. `step_thumb` lands on the grid counted from `min`, as the
    keyboard does. Tests: `accessibility_steps_land_on_the_step_grid`, and in
    Material `an_accessibility_step_lands_on_the_step_grid`,
    `a_continuous_slider_steps_by_a_hundredth_of_its_range`.
  - `step_thumb` stepped any value: a NaN value became `max` after one
    Increment, and a `min` above `max` or a reversed range was stepped through
    bounds that cannot hold. As Material's keyboard (`stepped_value`) does,
    such a value now comes back unchanged and `step_thumb` emits nothing.
    Tests: `stepping_leaves_an_invalid_value_or_range_alone` (Base and
    Material), `a_range_thumb_snaps_to_the_grid_and_stops_at_the_other`.

  Right-to-left pointer math and a range's `Role::Group` node are not patched
  here: Material does both (`crates/material/src/components/slider/mirrored.rs`
  and `range_group.rs`).

Merged upstream and no longer patched here: CSS transition reversal (#3820),
slider step snapping from `min` with a dropped handle standing at its value and
continuous drags for a step that is not positive and finite (#3822),
`Toolbar::axis` (#3821), `TextView` block surfaces (#3830, the `> [!name]`
marker on a top-level blockquote), and before those `min` above the default
`max` (#3823), `Slider::track_focus` (#3824), a disabled `SliderThumb` (#3825),
`Dialog::accessibility_label` (#3826), the input IME caret (#3827), notifying
an input moved by its parent (#3828), `TimeField::separator` (#3829) and
publishing the input layout at the end of prepaint (#3831). Those files equal
upstream.

These patches require source unification for both the Kit Git dependency and
any registry Base dependency in the consuming workspace. Cargo patches are not
inherited by downstream consumers of a published Material crate.

Validation must include the retained compatibility tests and upstream behavior
before this source change is accepted. This source/provenance update itself
does not assert that compilation or runtime checks have passed.
