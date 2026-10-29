# Workspace transition patch

Source: published `gpui-base` 0.8.1, Apache-2.0; original source revision is
recorded in `.cargo_vcs_info.json`. The normalized Cargo manifest preserves
published dependencies, including GPUI `=0.3.10`.

The local patch provides CSS reversing-adjusted start values, reversal duration
and negative-delay scaling, and explicit per-transition reduced-motion policy.
The default still finishes transitions immediately when reduced motion is on;
callers may opt in for non-spatial effects. Keyframe playback is unaffected.

The root `[patch.crates-io]` applies only to this workspace. It is not inherited
by applications depending on a published Material crate. Those applications
need this patch or a future Base release with the corresponding functionality.

Validate this excluded crate separately with
`cargo test --manifest-path vendor/gpui-base/Cargo.toml --locked --features test-support --lib motion::`.
The separate lockfile pins the upstream dev dependencies used by these tests.
