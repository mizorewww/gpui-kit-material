# Source and manifest policy

Source repository: <https://github.com/longbridge/gpui-kit>

Source release: `gpui-kit` 0.9.1, tag `v0.9.1`,
revision `39ab849a93b5f15507b969164d49891db8d5b470`

The complete `crates/kit` directory is copied without implementation changes.
`Cargo.toml.orig` preserves the original manifest. `SOURCE.json` records SHA-256
hashes for every original file, the source workspace manifest, both retained
licenses, and the generated local manifest. `LICENSE-APACHE` and
`LICENSE-DOCS.md` come from the repository root.

The local `Cargo.toml` expands the source workspace's edition, Clippy lints and
dependency declarations. Source workspace siblings (`gpui-base`,
`gpui-component`, `gpui-kit-assets`) are the same 0.9.1 release on crates.io, as in
the published `gpui-kit` 0.9.1. Moving to another release means refreshing the
facade copy and this tag together.

The behavioral manifest changes are limited to platform feature selection:

- Native non-mobile platform dependencies retain upstream defaults and features.
- That target predicate explicitly excludes WASM, preventing its native default
  features from also applying to a browser build.
- The WASM target explicitly declares both the platform and Web dependencies,
  with `default-features = false`. The same applies to optional GPUI Fast
  platform/Web dependencies. All other inherited feature requests are retained.
- The new, non-default `multithreaded-web` feature forwards `multithreaded` to
  the ordinary platform and Web crates, and weakly forwards it to Fast crates
  only when Fast is selected. It preserves an explicit route to upstream's
  worker/atomics functionality and its existing toolchain/runtime requirements.
  No toolchain override or bootstrap environment variable is supplied here.

`MANIFEST.diff` shows the complete difference from the original, including
workspace normalization. Reproduce with Python 3.11 or newer and an unmodified
checkout at the release tag:

```sh
python3.13 vendor/gpui-kit/normalize-manifest.py /path/to/gpui-kit-checkout
```

The script performs no Cargo invocation, network access, or compilation.
