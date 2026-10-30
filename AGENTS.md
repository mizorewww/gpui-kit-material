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

- Implement every component in Google's Figma **Material 3 Design Kit**, plus
  the union of <https://md3.io/> and <https://m3e.kyralabs.dev/>. When the
  references disagree, priority is **Figma kit > md3.io > m3e.kyralabs.dev**,
  applied per token family (color, shape, type, motion, state, geometry): use
  the first reference that defines the value, and record the source with the
  value. Follow [docs/ROADMAP.md](docs/ROADMAP.md) and
  [conformance/md3.json](conformance/md3.json).
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
- A component is stable only when it meets
  [docs/PRODUCTION.md](docs/PRODUCTION.md). Keep
  [docs/COVERAGE.md](docs/COVERAGE.md) honest when a component's status changes.
- The Gallery is an interactive application, not a static board. Its UI follows
  md3.io's component pages (navigation by component group, live preview with
  theme and light/dark switch, and a configuration panel whose switches,
  segmented buttons and selects expose every variant, size, state and content
  option), built from this library's own components. Every preview is the real,
  working component: clicks, keyboard, menus, dialogs and animations work. The
  Figma kit's spec boards may appear as a secondary view of a component page.
- No horizontal scrolling for content in the Gallery. Spec boards, previews and
  configuration panels reflow to the available width (wrap columns into
  labelled rows or groups, stack sections) so every specimen is fully visible
  at any supported window width; nothing may be clipped or hidden off-screen.

## Performance and delivery

- Performance must clearly beat web implementations, measured on release builds
  on an idle machine with raw per-frame timings, maxima, hardware/build context
  and independent review. A profile taken during builds is not evidence.
  - Steady-state frames (scrolling, hover, running animations, typing): main
    thread CPU work (render, layout, prepaint, paint, scene submission) p95
    <= 1 ms and max <= 2 ms.
  - A Material list costs at most 2x the instructions per frame of an equivalent
    Kit list. Material components and Gallery composition are the first place to
    optimize; engine patches need evidence that the engine is the cause.
  - Idle windows draw nothing; no missed refreshes at 120 Hz with vertical sync
    on. Report GPU time separately.
  - First frame at most 20% above Kit's stories baseline on the same machine, and
    Gallery page switches <= 8 ms.
  - WASM: the same steady-state budgets x2; keep main-thread frame time >= 5x
    below md3.io in Chrome for interactions with real main-thread work.
  Averages alone never establish a target; incomplete timing stays unverified.
- Notify the narrowest owning view, avoid render-time notification loops and
  virtualize long collections. Measure before introducing caches and identify
  each cache's invalidation owner. Do not simplify required animations to pass.
- Use bounded captures and stop owned test processes when sampling completes.
- Before merging: `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
  plus relevant WASM, profiler and compatibility checks for the change.
- Work on `main`. Keep commits atomic by concern and push completed milestones
  to GitHub frequently. Preserve remote/user changes without force-pushing.
  Record implementation history in commit messages, not documents.
