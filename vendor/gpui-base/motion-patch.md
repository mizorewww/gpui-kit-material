# Workspace compatibility patch

This directory contains `crates/base` from Longbridge GPUI Kit Git revision
`8dfdd49de27c0726dd052fb2cfd2b0807d17980b` (Kit `main`), licensed under
Apache-2.0. The upstream package declares version `0.9.0`, but this checkout is
**not** the published crates.io `0.9.0` source. `SOURCE.json` records every
upstream file's SHA-256, the normalized manifests, and the original and patched
SHA-256 of each local patch file. `.cargo_vcs_info.json` is a locally written
marker identifying the Git checkout actually vendored here.

The root and example manifests expand workspace-inherited values from that
same upstream revision. They do not inherit dependencies or lint settings from
the Material workspace. GPUI snapshot dependencies remain pinned to `=0.3.11`;
optional GPUI Fast support and all upstream dependencies/features are retained.
Dependency resolution belongs to the consuming workspace's `Cargo.lock`.
Upstream development examples retain references to sibling story-web fonts and
repository fixtures not included in this Base-only snapshot. They are not
claimed to run standalone; the upstream package also excludes these examples
from publication.

The only local change is one open upstream PR, longbridge/gpui-kit#3841
(branch `material/motion-transition-playback-v2` on mizorewww/gpui-kit, head
`a4cf38985e8b856905ba504f5b73968dc003dcea`). That head merges Kit `main`
`2fffa1ad`, whose `crates/base` equals `8dfdd49d`. `src/lib.rs` and
`src/motion.rs` equal that PR head:

- `src/motion.rs`: `TransitionState<T>`, caller-owned state that keeps one
  transition's clock and interruption/reversal history across unmounts (for
  virtualized rows), using the same sampler as `transition_with_status` and
  honoring the transition's reduced-motion policy. It retains no view or
  window; `sample(target, policy, window, cx)` requests the next animation
  frame while the status is `Delayed` or `Running`.
- `src/lib.rs`: the public export of `TransitionState`.

Every other file equals Kit `8dfdd49d`. Merged upstream and no longer patched
here: per-transition reduced motion (`Transition::reduced_motion`,
`ReducedMotion`; #3839), assistive-technology Increment/Decrement that move one
thumb onto the step grid and report the change, with
`SliderState::accessibility_step` (#3840), and before those CSS transition
reversal (#3820), `Toolbar::axis` (#3821), slider step snapping from `min`
(#3822), `min` above the default `max` (#3823), `Slider::track_focus` (#3824),
a disabled `SliderThumb` (#3825), `Dialog::accessibility_label` (#3826), the
input IME caret (#3827), notifying an input moved by its parent (#3828),
`TimeField::separator` (#3829), `TextView` block surfaces (#3830) and
publishing the input layout at the end of prepaint (#3831).

This patch requires source unification for both the Kit Git dependency and
any registry Base dependency in the consuming workspace. Cargo patches are not
inherited by downstream consumers of a published Material crate.
