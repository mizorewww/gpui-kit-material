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

- The performance target is to beat the best web implementation, md3.io, on the
  same machine: for each comparable interaction (scrolling, hover, running
  animations, typing, page switch, first load), the Gallery's main-thread frame
  time p95 and max are below md3.io's best run in Chrome, both natively and as
  WASM in the same browser. Idle windows draw nothing. Measure release builds
  on an idle machine with raw per-frame timings, maxima, and hardware/build
  context; a profile taken during builds is not evidence. Averages alone never
  establish a target; incomplete timing stays unverified.
  - The profiler's finer budgets (steady CPU p95 1 ms, Material list against
    Kit list, first frame against Kit's stories, page switch 8 ms) are
    diagnostics for finding regressions, not release gates.
  - Material components and Gallery composition are the first place to
    optimize; engine patches need evidence that the engine is the cause.
- Notify the narrowest owning view, avoid render-time notification loops and
  virtualize long collections. Measure before introducing caches and identify
  each cache's invalidation owner. Do not simplify required animations to pass.
- Use bounded captures and stop owned test processes when sampling completes.
  Stop processes only by the PIDs you started (`kill $pid`). Never use
  `pkill`, `killall` or `pkill -f` with a pattern: on macOS `pkill -f cat -x`
  treats `-x` as a second pattern and kills every process whose command line
  contains "cat" (Application, Notification...), which took down the user's
  desktop session. Never change power settings (`pmset`) or system services.
- This machine has 16 GB of memory and is shared with other sessions and CI
  runners. Run heavy cargo commands with `CARGO_BUILD_JOBS=3 nice -n 15`, keep
  at most two building agents at once, and check `uptime` before release or
  WASM builds; parallel builds have frozen it.
- Keep disk use bounded; Rust build output is the main risk. Check free space
  before heavy builds and stop below 40 GB. Avoid scratch copies of the repo; delete
  any you make, and every extra `target/`, as soon as you have the result. Remove
  agent worktrees and their build output when their branch merges. Keep raw
  evidence (JSON, images, logs), which is small.
- Before merging: `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
  plus relevant WASM, profiler and compatibility checks for the change.
- Work on `main`. Keep commits atomic by concern and push completed milestones
  to GitHub frequently. Preserve remote/user changes without force-pushing.
  Record implementation history in commit messages, not documents.
