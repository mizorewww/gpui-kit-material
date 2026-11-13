# Releasing

How the crates reach crates.io. What a release must meet is in
[PRODUCTION.md](PRODUCTION.md#release-criteria); this is the order of work.

## Order

1. `gpui-kit-material-color` 0.1.0: published, tag `color-v0.1.0`.
2. `gpui-material-symbols` 0.1.0 and its four data crates: published from
   [gpui-material-symbols](https://github.com/mizorewww/gpui-material-symbols).
3. `gpui-kit` 0.9.1 (2026-11-09, tag `v0.9.1`), with every Base API the library calls:
   released, with `gpui-fast` 0.1.14 and `gpui-pre` 0.3.12.
4. `gpui-kit-material` 0.1.0, on that release and nothing patched.

## What the library needs from Kit

Nothing unreleased. The workspace depends on crates.io `gpui-kit` and `gpui-base` 0.9.1,
with no Git source, and `scripts/check-registry-consumer.sh` builds the packaged crate as
crates.io would receive it, with no `[patch]`, natively and for
`wasm32-unknown-unknown` (2026-11-09). The browser build needs a nightly toolchain, as
Kit's own WebAssembly guide says: published Kit enables its Web platforms' default
`multithreaded` feature, whose `wasm_thread` uses `#![feature]`, so on stable 1.98 it
fails in `wasm_thread`, not in this library. The workspace's `vendor/gpui-kit` manifest
keeps the Gallery single-threaded on stable. The Base APIs 0.9.0 lacked (`ReducedMotion`,
`Transition::reduced_motion`, `TransitionState`, `SliderState::accessibility_step` and
`step_thumb`, `Slider::track_focus`, `Toolbar::axis`, `TimeField::separator`,
`Dialog::accessibility_label`) are all in 0.9.1.

## What stays vendored, and why

None of it is needed to build the library; it is the Gallery's engine, and the optional
consumer patch set ([ARCHITECTURE.md](ARCHITECTURE.md#consumer-patch-set)).

| Vendored | On | Why it is still vendored |
| --- | --- | --- |
| `gpui-kit` | crates.io 0.9.1 source, unchanged | Its manifest keeps WASM single-threaded by default; published Kit enables the Web platforms' `multithreaded` default. |
| `gpui-fast` | crates.io 0.1.15 | Hanging overflow spaces (`src/text_system/line_layout.rs`, `line_wrapper.rs`), waiting on zed-industries/zed#66644; a wheel over nested scroll containers scrolls only the innermost one it moves, for the gesture (`src/fast/layers/wheel_latch.rs`); test-harness repairs and `try_update` for the deprecated `fetch_update`. |
| `gpui-fast-apple` | crates.io 0.1.15 | Its `font_fallback` regression test with the Gallery's fonts, and a warning fix; no runtime change. |
| `gpui-pre` | crates.io 0.3.12 | Hanging overflow spaces and a `measure_all` list's measurement after a width change (`vendor/gpui-pre/list-width-remeasure-patch.md`, released in gpui-fast 0.1.15 as #129), and the lazy tab-stop index and `em` metrics that gpui-fast has upstream, for the upstream backend, which has no PR channel of its own; test-harness repairs; `try_update`. |
| `gpui-pre-apple`, `gpui-pre-web` | crates.io 0.3.12 | Fallback and `sbix` fonts, and the browser IME, for the upstream backend; gpui-fast 0.1.14 has them upstream. |

Every Material gpui-fast PR (#104–#106, #108–#115, #119, #129) is in gpui-fast 0.1.15, so
`gpui-fast-macos`, `-linux`, `-windows`, `-wgpu` and `-web` are no longer vendored: they
come from crates.io 0.1.15 (`-wgpu`'s test-harness repairs only mattered to its own tests,
which this workspace does not run). `scripts/vendor-gpui-fast.py --published <version>`
records the vendored gpui-fast crates against a release's archives; `--rev` still
packages an unreleased commit with upstream's release step.

crates.io users get upstream wrapping, and lay out every item of a `measure_all` list in
each frame its width changes, until those two patches are released upstream.

## Before a release

```sh
scripts/check-registry-consumer.sh --dir "$(mktemp -d)"
RUSTUP_TOOLCHAIN=nightly-2026-10-31 \
    scripts/check-registry-consumer.sh --dir "$(mktemp -d)" --target wasm32-unknown-unknown
```

The browser check uses the nightly Kit 0.9.1's WebAssembly build names (`crates/story-web/rust-toolchain.toml`).

CI's `release-readiness` job runs both on every push, and its `package` job runs
`cargo publish --dry-run` of each library crate. To preview a Kit that is not released,
`--kit-git <Kit commit>` takes `gpui-kit` and `gpui-base` from Kit's Git at that commit.

## Release day

1. Kit has released what the library calls. Set the root `Cargo.toml`'s `gpui-kit` and
   `gpui-base` to that release's registry version, without `git`/`rev`, and drop any Kit
   Git `[patch]`. Re-vendor `vendor/gpui-kit` at the released source
   (`vendor/gpui-kit/normalize-manifest.py`, `SOURCE.json`), and move the vendored engine
   crates to the release Kit names (`scripts/vendor-gpui-fast.py --published`), dropping
   every crate that no longer carries a local change.
2. Update `crates/material/README.md`'s and the root README's `gpui-kit = "…"` line.
3. `scripts/check-registry-consumer.sh --dir "$(mktemp -d)"` passes, natively and
   with `--target wasm32-unknown-unknown` (on Kit's nightly while published Kit builds
   its Web platform multithreaded).
4. `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, `scripts/check-package.sh`, `scripts/check-features.sh`;
   CI green.
5. In CHANGELOG.md, turn the summary under Unreleased into
   `## gpui-kit-material <version> (date)`. Update the README's installation section and the consumer patch set's revision in ARCHITECTURE.md.
6. `cargo publish --dry-run -p gpui-kit-material`, then `cargo publish -p gpui-kit-material`.
7. Tag the commit `material-v<version>` and push the tag.

0.1.0 through 0.1.2 are published (tags `material-v0.1.0`, `color-v0.1.0`,
`material-v0.1.1`, `color-v0.1.1`, `material-v0.1.2`). For `gpui-kit-material` 0.1.3,
steps 1–6's dry runs are done (2026-11-13, Kit 0.9.1); what remains is `cargo publish -p
gpui-kit-material` and the `material-v0.1.3` tag. `gpui-kit-material-color` is unchanged since 0.1.1, so its manifest
keeps `version = "0.1.1"` rather than the workspace's, and it is not released again.

The Gallery, gallery-web, benches and consumer-check stay `publish = false`.
