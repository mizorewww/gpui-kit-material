# Production standard

What a component must meet before it is called **stable**, and what a crate
release must meet before it is published. Project rules that apply to all work
are in [AGENTS.md](../AGENTS.md); this document adds the bar for "done".

Nothing is stable today. [COVERAGE.md](COVERAGE.md) audits every component
against this standard.

A criterion is met only with evidence someone else can check: a test, a
committed capture, a raw timing file or a linked review. A successful build, a
source reading or a reference fixture alone proves nothing. Anything unmeasured
is **unverified**, never assumed.

## Component criteria

### API

- Builders follow the crate's conventions: `Type::new(id, ...)`, chainable
  `self` setters, `on_*` callbacks taking `(&mut Window, &mut App)`.
- State is controlled by the caller. A component reports changes and never
  stores the value the caller owns. Internal state is limited to transient
  interaction and motion.
- No hard-coded user-facing strings. Labels and accessible names come from the
  caller, and a label required for accessibility is a required argument.
- Options are real API. An option implemented only in a gallery playground does
  not count.
- Every public item has rustdoc. Each component has at least one runnable doc
  example that compiles under `cargo test --doc`.
- Semver: after 1.0, breaking changes need a major release. Replaced API stays
  for at least one minor release, marked `#[deprecated(since, note)]` with the
  replacement named. Before 1.0, breaking changes bump the minor version and
  are listed in the changelog.

### Material 3 completeness

- Every variant, size, state and option in the Figma kit and in md3.io's
  configuration panel for the component is implemented, or excluded with a
  written reason. m3e options count where neither defines the value.
- Values follow the priority **Figma kit > md3.io > m3e.kyralabs.dev**, per token
  family and per value. Each value records its source in rustdoc and a fixture,
  and a test holds the implementation to the fixture.
- Colors are semantic `ColorRole`s through `theme::color`; text-aligned geometry
  uses `typography::rem`.
- Every state change the specification animates is animated, including
  interruption and retargeting, with the source of its timing recorded.

### Interaction

- Pointer: hover, press, release outside, cancel, and drag where the component
  has it.
- Touch: touch-down feedback, cancellation on scroll, and a 48 dp minimum target
  where the specification requires one.
- Keyboard: Tab order, activation keys and arrow-key behavior as the WAI-ARIA
  pattern for the role describes.
- Focus: the focus ring appears for keyboard focus and not for pointer focus.
  Focus is trapped and restored by modal surfaces.
- Disabled and loading states block activation and are exposed to assistive
  technology.

### Accessibility

- Correct role, name, state and value in the accessibility tree, asserted by a
  test against the tree GPUI builds.
- A manual pass with VoiceOver on macOS. NVDA or Narrator on Windows and Orca on
  Linux once those backends publish a tree. Unavailable platforms are recorded,
  not skipped silently.
- Reduced motion: a test shows spatial motion is removed and effects follow the
  specification.
- Text scale: layout holds at 100, 125, 150 and 200%, with no clipped or
  overlapping content.
- RTL: mirrored layout, icons and keyboard direction, with a test. Directional
  icons flip and others do not.
- Contrast: correct in every reference theme at standard, medium and high
  contrast, light and dark.

### Visual evidence

- Committed captures in light and dark of each variant and state, beside the
  matching Figma kit export, with the capture's JSON sidecar.
- Differences from the kit are listed and explained. An unexplained difference
  fails.
- Motion is compared with timed samples or video, because a still capture cannot
  establish it.

### Tests

- Rule tests for tokens and pure logic, against fixtures where a reference
  defines the value.
- Interaction tests that drive events through `TestAppContext` and assert the
  result.
- A regression test for every fixed bug.

### Performance

Budgets are the targets in [AGENTS.md](../AGENTS.md#performance-and-delivery)
and [DEVELOPMENT.md](DEVELOPMENT.md#targets): steady-state main-thread CPU p95 ≤
1 ms and max ≤ 2 ms (WASM ×2), idle windows draw nothing, ≥ 500 new frames/s
while scrolling with vsync off. A component meets them when its Gallery page does
in a release build, with raw per-frame timings and hardware context preserved.
Averages never establish a target.

### Platforms

- Desktop: macOS, Linux and Windows. Each needs a run of the real gallery, not
  only a build.
- Web: the WASM gallery matches desktop in behavior and appearance. A difference
  needs a written reason.

### Robustness

- No panics on reachable paths, including empty collections, out-of-range values,
  zero or extreme sizes and missing assets. `unwrap` and `expect` in non-test code
  need an adjacent reason why they cannot fail.
- Long text, large collections and rapid repeated input degrade gracefully.
  Long collections are virtualized.

### Review

An independent reviewer (a subagent or person who did not write the component)
approves it against this list, and the verdict is recorded with the commit it
covers in [conformance/md3.json](../conformance/md3.json). A later change to the
component invalidates the verdict for what it touches.

## Release criteria

A crate release also requires:

- **CI** green on macOS, Linux and Windows, both GPUI backends, and the WASM
  gallery build.
- **Published dependency graph.** `cargo publish --dry-run` succeeds and a new
  project can depend on the crates from a registry with no workspace-only
  `[patch]` tables. Every vendored engine patch has landed upstream, been
  published under its own package, or been removed.
- **docs.rs** builds the crates, with its feature and target configuration in
  `[package.metadata.docs.rs]`, on a machine without the gallery's assets.
- **MSRV** declared as `rust-version`, verified in CI and treated as a compatible
  change only within a minor release.
- **CHANGELOG.md** in Keep a Changelog form, with every breaking change and
  deprecation listed.
- **Licensing and provenance.** Every vendored source, font, icon and palette has
  its license and pinned hash recorded. `cargo deny check` passes, or its
  exceptions are explained.
- **Performance evidence** for the release commit, under
  `conformance/performance/`, passing every target in `targets.json`.
- **Coverage** regenerated: [COVERAGE.md](COVERAGE.md) shows which components are
  stable, and anything unstable is marked so in its rustdoc.
