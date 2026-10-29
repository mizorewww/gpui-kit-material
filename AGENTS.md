# Project instructions

## Foundation and reuse

- This is a Material 3 / M3 Expressive layer on **Longbridge GPUI Kit**:
  <https://github.com/longbridge/gpui-kit>. `gpui-component` is that repository's
  styled component crate, exposed as `gpui_kit::component`; it is a preferred
  foundation, not a prohibited dependency or an unrelated toolkit.
- Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and the checked-out upstream
  architecture, coding guides, component implementation and tests before adding
  a layer, dependency, generic control or platform implementation.
- Prefer the `gpui-kit` public facade and its `component` APIs. Reuse existing
  controls, state, focus, overlays, window setup, icons, scrolling and motion
  infrastructure before writing local equivalents. Application startup should
  use Kit's `application`, `init` and `open_window` entry points.
- Keep local components as Material styling/composition adapters where possible.
  Use `gpui_kit::base` for a specific presentation capability missing from
  Component, with a source-backed explanation of that gap. Do not rebuild
  generic behavior merely to avoid `gpui-component`.
- Before deleting a local implementation, identify its replacement and verify
  controlled state, keyboard/touch behavior, focus, accessibility, animation,
  layout, desktop/WASM support and the performance budget. Equal names or package
  versions do not prove equivalent implementations. Retain only necessary,
  independently reviewed compatibility patches until their replacement works.
- Keep one compatible GPUI and Base implementation across Material, Kit and
  Symbols. Align Git/path/registry sources and backend features; commit
  `Cargo.lock`. Do not introduce a reverse dependency from reusable icons into
  the Material library.

## Material conformance

- Implement the union of all components at <https://m3e.kyralabs.dev/> and
  <https://md3.io/>. Overlaps use **m3e.kyralabs.dev** as the authority for the
  entire component's visuals, typography, states and animations. Follow
  [docs/ROADMAP.md](docs/ROADMAP.md) and [conformance/md3.json](conformance/md3.json).
- Reusing Kit behavior does not authorize replacing Material visuals or motion
  with Kit defaults. Preserve exact reference parity, including interruptions,
  reduced motion and theme/text-scale changes.
- Paint with semantic `ColorRole`s via `theme::color` and size text-aligned
  geometry in rems (`typography::rem`). Bridge these roles to Kit themes instead
  of duplicating generic theme infrastructure.
- Material icons come from the sibling `../gpui-material-symbols` repository.
  Reuse its Kit integration and improve that repository when needed; preserve
  its full Google catalog, variation support and provenance. Do not add another
  local generic icon renderer or substitute Lucide for Material icons.
- No hard-coded user-facing strings in components; accept labels from callers.
- Every component must appear in the shared desktop/WASM gallery, have tests
  for its meaningful rules, and receive independent subagent approval. A source
  review, successful build or reference fixture alone is not conformance PASS.

## Performance and delivery

- The **entire Gallery CPU frame must be <= 2 ms** on desktop and WASM, including
  rendering, layout, prepaint, paint, submission and cleanup. Include the first
  frame. Any measured frame > 2 ms fails; averages cannot establish acceptance.
  Preserve raw frame timings, maxima, build/hardware context and independent
  review. Report GPU time separately; incomplete timing remains unverified.
- Notify the narrowest owning view, avoid render-time notification loops and
  virtualize long collections. Measure before introducing caches and identify
  each cache's invalidation owner. Do not simplify required animations to pass.
- Coordinate one build or benchmark at a time; default to `CARGO_BUILD_JOBS=2`.
  Use bounded captures and stop owned test processes when sampling completes.
- Before merging: `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
  plus relevant WASM, profiler and compatibility checks for the change.
- Work on `main`. Keep commits atomic by concern and push completed milestones
  to GitHub frequently. Preserve remote/user changes without force-pushing.
  Record implementation history in commit messages, not documents.
