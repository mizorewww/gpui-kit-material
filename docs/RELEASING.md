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
| `TransitionState` | gpui-kit#3841, merged, unreleased |

Against Kit `main` (`8da4e005`, which merges #3841), unpatched, the packaged library
builds (`scripts/check-registry-consumer.sh --kit-git 8da4e005258ac0aac9a6a332cce746eeaafa08a8`,
2026-11-08). Everything the library calls is in Kit `main`; the only remaining blocker
is a Kit release. Plan B, Material's own playback on branch
`release/transition-playback-internal`, is no longer needed.

Nothing else in `vendor/` is needed to build the library: the unpatched build above
used crates.io `gpui-fast` 0.1.13. The engine patches change behavior only or are
dev-only (ARCHITECTURE.md, Consumer patch set).

## Engine patches after the release

Every gpui-fast PR of ours is merged on gpui-fast `main`; the latest release is
0.1.13, without them. `vendor/gpui-fast*` already track `main` (`97439ecf`, produced by
`scripts/vendor-gpui-fast.py`, which runs upstream's release renaming), so these changes
now come from upstream and are no longer local patches. Once gpui-fast releases them,
replace the vendored copies with that release and drop what is left of the patches below:

| gpui-fast PR (now upstream) | Where it lives upstream |
| --- | --- |
| #104 wheel at limit | `gpui-fast`: `src/fast/layers/wheel.rs`, wheel part of `src/elements/div.rs` |
| #105 hover freeze holds listeners | `gpui-fast`: `src/fast/layers/hover_freeze.rs`, hover part of `div.rs`, `window.rs`'s `release_unhovered` |
| #106 splice forgets released entities | `gpui-fast`: `src/fast/splice.rs`, `EntityMap::is_alive` in `dependencies.rs` and `entity_map.rs` |
| #108, #109, #110 web IME position, keys, surrogate pairs | `gpui-fast-web`: `src/ime_mirror.rs`, `src/events.rs`, `src/ime_edit.rs`, IME part of `src/window.rs` |
| #111, #112 macOS fonts without `m`, app fonts in fallback cascades | `gpui-fast-apple`: `src/text_system.rs`, `src/open_type.rs`, `tests/font_fallback.rs` |
| #119 `sbix` color fonts | `gpui-fast-apple`: `src/fast/color_bitmap_fonts.rs`, its part of `src/text_system.rs` |
| #113 lazy tab stops | `gpui-fast`: `src/tab_stop.rs` |
| #114 tracked entities | `gpui-fast`: `App::record_entities_accessed` in `src/app.rs` |
| #115 em without `m` | `gpui-fast`: `src/text_system.rs` |

Not offered upstream, behavior only: hanging overflow spaces
(`src/text_system/line_layout.rs`, `line_wrapper.rs`), a wrapping change kept for the
Gallery (waiting on zed-industries/zed#66644); crates.io users get upstream wrapping.
A `measure_all` list's measurement after a width change (`src/elements/list.rs`,
`vendor/gpui-pre/list-width-remeasure-patch.md`) is to be offered upstream; until then
crates.io users lay out every item of such a list in each frame its width changes.
Kept local, no API: the test-harness repairs and fixtures (`gpui-fast-apple`'s
`tests/font_fallback.rs` among them), a warning fix, and `try_update` for the
deprecated `fetch_update`. The `-web`, `-macos`, `-linux` and `-windows` crates are
unmodified copies of `main` (`-wgpu` differs only in test-harness repairs) and go with
the release. Nothing vendored adds or changes
engine API: the dev-only frame observation (CPU frame scopes, submission statistics,
vsync control, presentation and GPU timing, the render lease) was dropped, since
gpui-fast takes no API additions. The `gpui-pre*` patches mirror the remaining ones for
the upstream backend, which has no PR channel of its own.

## Before the release

```sh
scripts/check-registry-consumer.sh --dir "$(mktemp -d)" --kit-git <Kit main commit>
```

CI's `release-readiness` job runs this against Kit's `main` on every push. It may
fail without failing CI until Kit releases.

## Release day

1. Kit has released. Set the root `Cargo.toml`'s `gpui-kit` to that release's registry
   version, without `git`/`rev`, and `gpui-base` to the same version. Drop the
   `[patch."https://github.com/longbridge/gpui-kit"]` table and the `gpui-base` Git
   patch; re-vendor `vendor/gpui-kit` for the Gallery at the released source
   (`SOURCE.json`).
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
