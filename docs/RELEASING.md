# Releasing

How the crates reach crates.io. What a release must meet is in
[PRODUCTION.md](PRODUCTION.md#release-criteria); this is the order of work.

## Order

1. `gpui-kit-material-color` 0.1.0: published, tag `color-v0.1.0`.
2. `gpui-material-symbols` 0.1.0 and its four data crates: published from
   [gpui-material-symbols](https://github.com/mizorewww/gpui-material-symbols).
3. A `gpui-kit` release containing every Base API the library calls (below).
4. `gpui-kit-material` 0.1.0, on that release and nothing patched.

## What the library still needs from Kit

`scripts/check-registry-consumer.sh` builds the packaged crate as crates.io would
receive it. On 2026-11-08 it failed as follows.

Against crates.io `gpui-kit`/`gpui-base` 0.9.0, 18 errors:

| Missing in 0.9.0 | Upstream |
| --- | --- |
| `ReducedMotion`, `Transition::reduced_motion` | gpui-kit#3839, merged, unreleased |
| `SliderState::accessibility_step`, `step_thumb` | gpui-kit#3840, merged, unreleased |
| `Slider::track_focus` | #3824, merged after 0.9.0 |
| `Toolbar::axis` | #3821, merged after 0.9.0 |
| `TimeField::separator` | #3829, merged after 0.9.0 |
| `Dialog::accessibility_label` | #3826, merged after 0.9.0 |
| `TransitionPlayback` | gpui-kit#3841, open |

Against Kit `main` (`8dfdd49d`), unpatched, only `TransitionPlayback` is missing
(4 errors). If #3841 is declined, Material keeps its own playback over Kit's public
`Easing`, `Interpolate` and `MotionValue`, as on branch
`release/transition-playback-internal`.

Nothing else in `vendor/` is needed to build the library: the unpatched build above
used crates.io `gpui-fast` 0.1.13. The engine patches change behavior only or are
dev-only (ARCHITECTURE.md, Consumer patch set).

## Engine patches after the release

Every gpui-fast PR of ours is merged on gpui-fast `main`; the latest release is
0.1.13, without them. Once gpui-fast releases them, re-vendor `vendor/gpui-fast*`
at that release and drop these local changes (diffing each against the merged
code, whose layout follows gpui-fast's CONTRIBUTING rather than our branches):

| gpui-fast PR | Vendored change |
| --- | --- |
| #104 wheel at limit | `gpui-fast`: `src/fast/layers/wheel.rs`, wheel part of `src/elements/div.rs` |
| #105 hover freeze holds listeners | `gpui-fast`: `src/fast/layers/hover_freeze.rs`, hover part of `div.rs`, `window.rs`'s `release_unhovered` |
| #106 splice forgets released entities | `gpui-fast`: `src/fast/splice.rs`, `EntityMap::is_alive` in `dependencies.rs` and `entity_map.rs` |
| #108, #109, #110 web IME position, keys, surrogate pairs | `gpui-fast-web`: `src/ime_mirror.rs`, `src/events.rs`, `src/ime_edit.rs`, IME part of `src/window.rs` |
| #111, #112 macOS fonts without `m`, app fonts in fallback cascades | `gpui-fast-apple`: `src/text_system.rs`, `src/open_type.rs`, `tests/font_fallback.rs` |
| #113 lazy tab stops | `gpui-fast`: `src/tab_stop.rs` |
| #114 tracked entities | `gpui-fast`: `App::record_entities_accessed` in `src/app.rs` |
| #115 em without `m` | `gpui-fast`: `src/text_system.rs` |

Not offered upstream, behavior only: hanging overflow spaces
(`src/text_system/line_layout.rs`, `line_wrapper.rs`), a wrapping change kept for the
Gallery; crates.io users get upstream wrapping. Dev-only, kept local for the Gallery
and profiler: the `frame-observation` feature and everything behind it (CPU frame
scopes, submission statistics, vsync control, presentation and GPU timing, the render
lease `Entity::update_for_render`), the `-macos`, `-linux`, `-windows`, `-wgpu` and
`wgpu` overlays, and the test-harness repairs and fixtures. The `gpui-pre*` patches
mirror these for the upstream backend, which has no PR channel of its own.

## Before the release

```sh
scripts/check-registry-consumer.sh --dir "$(mktemp -d)" --kit-git <Kit main commit>
```

CI's `release-readiness` job runs this against Kit's `main` on every push. It may
fail without failing CI until Kit releases.

## Release day

1. Kit has released. Set the root `Cargo.toml`'s `gpui-kit` to that release's registry
   version, without `git`/`rev`, and `gpui-base` to the same version. Drop the
   `[patch."https://github.com/longbridge/gpui-kit"]` table and the vendored Base
   patches now upstream; re-vendor what remains for the Gallery at the released
   source (`SOURCE.json`).
2. Update `crates/material/README.md`'s and the root README's `gpui-kit = "…"` line.
3. `scripts/check-registry-consumer.sh --dir "$(mktemp -d)"` passes, natively and
   with `--target wasm32-unknown-unknown`.
4. `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, `scripts/check-package.sh`, `scripts/check-features.sh`;
   CI green.
5. In CHANGELOG.md, turn the 0.1.0 summary under Unreleased into
   `## gpui-kit-material 0.1.0 (date)`. Update the README's crates table and quick start.
6. `cargo publish --dry-run -p gpui-kit-material`, then `cargo publish -p gpui-kit-material`.
7. Tag the commit `material-v0.1.0` and push the tag.

The Gallery, gallery-web, benches and consumer-check stay `publish = false`.
