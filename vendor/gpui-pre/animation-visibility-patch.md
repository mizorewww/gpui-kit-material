# Animation scheduling contract

Source: the published Apache-2.0 `gpui-pre` 0.3.10 crates.io archive. That
archive ships no `.cargo_vcs_info.json`, so no upstream Git revision is claimed;
`SOURCE.json` records the archive and per-file hashes. Files:
`src/elements/animation.rs`.

Animations sample `background_executor().now()`, including spring updates,
ordinary repeat origins and synced phases. Production retains the platform's
monotonic clock; tests advance that same clock through the executor. Animation
chain stage transitions retain their existing semantics.

`AnimationElement::repeat_when_visible` is an opt-in for a single nonzero repeat
whose drawing stays within its layout bounds and whose motion cannot move or
grow into the viewport from outside. Ordinary animations and springs retain
unconditional scheduling. Do not apply the opt-in to entrance animations,
overflowing content, or shadows extending beyond the declared bounds.

Sampling retains the original time origin. Prepaint computes the intersection
of layout bounds, ancestor content mask and viewport; paint submits the next
frame request only for a positive-area intersection. This keeps scheduling out
of retryable prepaint transactions. Requests notify the original owning view.
One callback or throttled timer queued before clipping can still fire; it must
not sustain an offscreen loop. Throttled timers share persistent pending state.
Opacity does not stop the clock, and reduced motion keeps its existing policy.

This does not cache the owning view or suppress frame requests made by nested
animations. Callers needing viewport-driven circular progress must use its
separate sampling/scheduling API, not wrap an unconditional sampler.

Virtualized callers may retain `RepeatAnimationPlayback` in their owning view
and pass it through `with_repeat_playback`. This opt-in supports a single nonzero
repeat and retains only its time origin, initialized on first layout. Unmounting
does not reset its phase. The handle stores no view owner, pending timer, layout
or paint cache; each mount retains its own scheduling state. A previously queued
timer may notify its old owner once. A new handle starts a new playback.
Use one handle per logical playback with unchanged duration and clock policy
within one app. Synced repeats still use the app epoch, and reduced motion keeps
the existing static sampling policy. Component-specific stop policies are not
provided by this handle.

Regression command:

```sh
cargo test -p gpui-pre --lib elements::animation::tests
```

The patch is workspace-local. Downstream consumers must use the same engine
source until the API is available upstream.
