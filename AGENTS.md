# Project instructions

- Material 3 (and M3 Expressive, per [docs/ROADMAP.md](docs/ROADMAP.md)) for GPUI.
  Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) before adding a layer or dependency.
- Components paint with `ColorRole`s via `theme::color`, size text-aligned
  geometry in rems (`typography::rem`), and take behavior from `gpui-base`.
  Do not add a `gpui-component` dependency.
- No hard-coded user-facing strings in components; accept labels from the caller.
- Every new component appears in `examples/gallery.rs` and has tests for its rules.
- Before merging: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- Keep commits atomic by concern; record history in commit messages, not documents.
