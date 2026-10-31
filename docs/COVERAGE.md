# Coverage audit

Every component and variant in the three references, against the
[production standard](PRODUCTION.md). Audited at commit `45de656` by reading the
component source, playgrounds and tests, the Figma kit page outlines, the
md3.io configuration panels, and the m3e documentation pages. Refreshed at `0f56b40`
by reading the source, playgrounds and tests of every row again: the Tests, A11y,
Playground and Rust columns and the gaps checkable in code are current to it. The F, M
and E columns and gaps against the references keep the `45de656` audit.

**Result: 0 components are production-ready.** Of the 46 rows below, 44 are
implemented, 43 with a playground, all with tests. None has independent approval, committed
captures against the kit, or per-component performance results. RTL is inherited by
every component with tests for the direction-sensitive ones, but has no committed
captures or review with a real RTL locale.

Limits of this audit:

- Figma coverage comes from page outlines, capped at 1500 lines (Buttons, Lists
  and Sliders reach it) and supplemented by page exports, so a variant nested below their depth
  limit may be missed. Where the audit found no gap, that means none was found,
  not that coverage is proven.
- md3.io options were compared control by control. m3e pages were read at the
  level of component and section, not every option, so m3e gaps may be missing.
- Test counts are test functions at `0f56b40`, not a measure of quality. Tests in
  `scope/tests.rs`, `a11y_tests.rs`, `choice.rs`, `tile.rs` and `roving.rs` are in no row.
- A search of the test modules found no role, name or state assertion for these rows
  marked `attr`, and they were not downgraded without a closer look: Button group,
  Segmented, Checkbox, Switch, Radio, Text field, Search, Date picker, Rail, Bar,
  Drawer, Tabs, Top and Bottom app bar, Docked and Floating toolbar.
- Some rows marked `inherit` read the direction without an RTL test: Radio, Carousel,
  Menu, Rail, Bar, Date picker, Text field and Button group.

## Legend

| Column | Meaning |
| --- | --- |
| F / M / E | Listed in the Figma kit / md3.io / m3e. `-` means not listed. |
| Tests | Test functions for the component + for its playground |
| A11y | `tree`: asserted in the tree GPUI builds. `attr`: a unit test asserts role, name or state. `code`: set in code, no test. Add `RM` when a test covers reduced motion. |
| RTL | `inherit`: follows the inherited direction through engine mirroring, no component code or test of its own. `tested`: a test drives this component under RTL. `exempt`: stays left to right on purpose. |
| Evidence | Committed under `conformance/`. `none` means nothing beyond unit tests. |

## Actions

| Component | F | M | E | Rust | Playground | Tests | A11y | RTL | Evidence | Missing |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Button | Y | Y | Y | `MaterialButton` | Yes | 75 + 0 | attr, RM | option + inherit | Figma tokens, md3.io timing captures | none found |
| Toggle button | Y | Y | Y | `MaterialButton::toggle` | Yes (button) | in Button | attr, RM | inherit | as Button | none found |
| Icon button | Y | Y | Y | `IconButton` | Yes | 2 + 3 | code | inherit | md3.io capture | none found |
| Button group | Y | Y | Y | `ButtonGroup`, `ConnectedButtonGroup` | Yes | 25 + 4 | attr, RM | inherit | none | none found |
| Segmented button | Y | - | Y | `ConnectedButtonGroup::segmented` | Specs only | in Button group | attr | inherit | none | No playground; Figma density -3 |
| Split button | Y | Y | Y | `SplitButton` | Yes | 5 + 3 | code | inherit | none | none found |
| FAB | Y | Y | Y | `Fab` | Yes | 5 + 2 | code | inherit | none | none found |
| Extended FAB | Y | Y | - | `ExtendedFab` | Yes | in FAB + 3 | code | inherit | none | none found |
| FAB menu | Y | Y | Y | `FabMenu` | Yes | 8 + 3 | code | inherit | none | none found |

## Selection and input

| Component | F | M | E | Rust | Playground | Tests | A11y | RTL | Evidence | Missing |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Checkbox | Y | Y | Y | `MaterialCheckbox` | Yes | 5 + 2 | attr | inherit | none | none found; `required` is a mark in the name, as GPUI has no required state |
| Switch | Y | Y | Y | `MaterialSwitch` | Yes | 8 + 1 | attr | inherit | none | none found; `required` is a mark in the name, as GPUI has no required state |
| Radio | Y | Y | Y | `MaterialRadio`, `MaterialRadioGroup` | Yes | 5 + 1 | attr | inherit | none | none found |
| Chips | Y | Y | Y | `AssistChip`, `FilterChip`, `InputChip`, `SuggestionChip`, `ChipSet` | Yes | 8 + 2 | code | inherit | none | none found |
| Slider | Y | Y | Y | `MaterialSlider` | Yes | 17 + 1 | attr | tested | none | No ripple by design: neither the Figma kit nor md3.io draws one on the Expressive handle |
| Text field | Y | Y | Y | `TextField` | Yes | 9 + 1 | attr | inherit | none | Input types only mask and filter; the web build's on-screen keyboard always gets `inputmode=text`. `required` is a mark in the name |
| Select | - | Y | - | `ExposedDropdown` | Yes | in Menu + 1 | code | inherit | none | `required` is a mark in the name |
| Search | Y | Y | Y | `SearchField`, `SearchView` | Yes | 8 + 2 | attr | inherit | none | none found |
| Date picker | Y | Y | Y | `DatePicker` | Yes | 10 + 2 | attr | inherit | none | none found |
| Time picker | Y | Y | Y | `TimePicker` | Yes | 9 + 2 | code | exempt (dial) | none | The dial has no ripple by design: the Figma kit draws only its selector |

## Navigation

| Component | F | M | E | Rust | Playground | Tests | A11y | RTL | Evidence | Missing |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Navigation rail | Y | Y | Y | `NavigationRail` | Yes | 16 (shared) + 2 | attr, RM | inherit | none | XR rail |
| Navigation bar | Y | Y | Y | `NavigationBar` | Yes | shared + 2 | attr, RM | inherit | none | XR bar |
| Navigation drawer | Y | Y | Y | `NavigationDrawer` | Yes | 5 + 2 | attr | inherit | none | none found |
| Navigation suite | - | - | Y | `NavigationSuite` | Yes | 7 + 4 | code | tested | none | Focus across a change of form is tested only from bar to rail. Defaults to the drawer when expanded (m3e: expanded rail, available as an option). Specs tab shows the kit's Navigation page: no reference has its own boards |
| Tabs | Y | Y | Y | `MaterialTabs` | Yes | 10 + 2 | attr, RM | tested | none | none found |
| Menu | Y | Y | Y | `MaterialMenu` | Yes | 16 + 3 | code | inherit | none | none found |
| Top app bar | Y | Y | Y | `TopAppBar` | Yes | 13 (shared) + 3 | attr, RM | inherit | none | XR app bar; no Small-image option, only a title slot |
| Bottom app bar | Y | Y | - | `BottomAppBar` | Yes | shared + 2 | attr, RM | inherit | none | none found |
| Docked toolbar | Y | Y | Y | `Toolbar` (docked) | Yes | 14 (shared) + 3 | attr, RM | inherit | none | none found |
| Floating toolbar | Y | Y | Y | `Toolbar` (floating) | Yes | shared | attr, RM | inherit | none | XR toolbar and its building blocks |

## Containment

| Component | F | M | E | Rust | Playground | Tests | A11y | RTL | Evidence | Missing |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Card | Y | Y | Y | `MaterialCard`, `CardSurface` | Yes | 7 + 2 | tree | inherit | Source audit only | Figma stacked and horizontal layouts. Image, title, body and actions are playground composition over slots |
| List | Y | Y | Y | `ListItem`, `segmented_list` | Yes | 6 + 2 | tree | inherit | none | none found |
| Carousel | Y | Y | Y | `MaterialCarousel` | Yes | 15 + 2 | tree | inherit | none | Figma multi-aspect ratio. No hover or pressed state, click handler or `preview_state` |
| Divider | Y | Y | Y | `MaterialDivider` | Yes | 5 + 1 | attr | option + inherit | md3.io capture | none found |
| Dialog | Y | Y | Y | `MaterialDialog` | Yes | 6 + 3 | tree | inherit | none | Figma list and scrollable-list forms (a `content` slot only); XR dialog; no `preview_state` |
| Bottom sheet | Y | Y | Y | `MaterialBottomSheet` | Yes | 10 (shared) + 2 | tree, RM | inherit | none | Drag handle has no state layer; no `preview_state` |
| Side sheet | Y | Y | Y | `MaterialSideSheet` | Yes | shared + 2 | tree, RM | inherit | none | No `preview_state` |

## Communication

| Component | F | M | E | Rust | Playground | Tests | A11y | RTL | Evidence | Missing |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Badge | Y | Y | Y | `Badge` | Yes | 5 + 2 | code | inherit | none | none found |
| Avatar | Y | - | - | `Avatar` | Yes | 3 + 0 | code | inherit | none | none found |
| Linear and circular progress | Y | Y | Y | `LinearProgress`, `CircularProgress` | Yes | 16 + 2 | tree, RM | inherit | none | Wave height and percentage label are not options. No reference defines a contained form |
| Loading indicator | Y | Y | Y | `LoadingIndicator` | Yes | 8 + 1 | attr | inherit | none | `.playback` still needs the vendored repeat patch; determinate turns opposite to Compose |
| Snackbar | Y | Y | Y | `Snackbar` | Yes | 7 + 3 | code | inherit | none | none found |
| Tooltip | Y | Y | Y | `MaterialTooltip` | Yes | 4 + 4 | code | inherit | none | No `preview_state` |
| Pull to refresh | - | - | Y | `PullToRefresh` | Yes | 14 + 3 | code, RM | inherit | m3e JVM fixture (`conformance/fixtures/m3e-pull-to-refresh.json`) | Refreshing spinner paints primary (reference: on-surface-variant) with the shared circular progress's stroke (reference: 2.5dp). No real touch-device run. Specs tab shows the kit's Loading & progress page |

## Layout and primitives

| Component | F | M | E | Rust | Playground | Tests | A11y | RTL | Evidence | Missing |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Canonical layouts | - | - | Y | `PaneScaffold`, `FeedLayout` | Yes | 25 + 5 | code, RM | tested | none | Levitated panes and hinges (not in m3e either). Handle fling velocity. `FeedLayout` is not virtualized; `feed_metrics` lets a caller do it. Feed takes one extra frame to settle its columns when its width changes class. Specs tab shows the kit's Examples page |
| Primitives | - | - | Y | Partial: `CardSurface`, `FocusRing`, `Ripple` | No | - | - | - | none | No public generic surface |
| XR variants | Y | - | - | none | No | 0 | - | - | none | **Not implemented**: app bar, dialog, navigation rail and bar, toolbar |

## Foundations

| Area | State | Source | Evidence | Missing |
| --- | --- | --- | --- | --- |
| Color | Implemented | Figma baseline, md3.io other themes, material-color-utilities (dynamic) | Fixtures, tests, upstream vectors | Image-derived seeds; multiple source colors; no committed seed captures |
| Typography | Implemented, with limits | Figma | Fixtures, tests | Kit's editors do not apply letter spacing; ligatures stay on under it |
| Shape | Implemented | Figma (corners), Compose `MaterialShapes` (library, morphing) | Fixtures, tests | Clipping content to a shape |
| Elevation and state layers | Implemented | Figma | Fixtures, tests | none found |
| Motion | Implemented | md3.io (Expressive), m3e (Standard) | Fixtures, tests | No per-component timed comparison outside buttons |
| Icons | Implemented | Material Symbols | Provenance in Symbols repository | Six approximate Kit translations unreviewed |
| Scoped theme | Implemented, with limits | n/a | Tests in `src/scope/tests.rs` | Kit's `Root` overlays, Kit tooltips, Kit's scrollbar styles, drag previews outside `current_scope` and what Kit reads while painting its own deferred draws keep the application theme. Under GPUI Fast the view holding a scheme scope rebuilds on each draw that reaches it; nothing outside it does |

## Cross-cutting status

Status of each [production criterion](PRODUCTION.md) across all components.

| Criterion | Status |
| --- | --- |
| Controlled state, caller-supplied strings | Followed in source; no automated check |
| Rustdoc with runnable examples | **Not met**: almost no doc examples |
| Pointer and keyboard | Implemented and tested per component to varying depth |
| Touch | Press feedback and cancellation on scroll through the shared ripple, tested on public touch events; a system cancel of a still finger releases only at a 5 s hold limit; only the first finger starts a wave; no real touch device run; the 48 dp target is implemented and tested for compact buttons, not audited elsewhere |
| Focus ring | Implemented; carousel has focus but no other states |
| Assistive technology pass | **Not done**: tree assertions only, under GPUI Fast |
| Reduced motion | Supported by the `motion` module; tested only where the tables say `RM` |
| Text scale 100–200% | Runtime support; **no per-component layout tests** |
| RTL | Inherited direction with mirrored layout, tests for mirroring, tabs, slider, navigation suite and canonical layouts; **no captures or review with a real RTL locale** |
| Contrast themes | Tokens tested; no per-component capture |
| Visual evidence | **Not met**: no committed light/dark captures against the kit |
| Performance budgets | **Unverified** per component and as a whole on this commit |
| Linux and Windows | Built and unit-tested in CI; **no real gallery run recorded** |
| WASM parity | Same gallery code builds; **no recorded parity comparison** |
| Reachable panics | **Not audited**: about 16 non-test `expect`/`unwrap` sites remain, on constants such as symbol names and easings |
| Independent review | **None**: every component entry in `conformance/md3.json` is `unreviewed`, and its performance entries `unverified` |

Release criteria not met: `gpui-kit-material` needs `[patch]` tables and Git
dependencies, so it is distributed by Git revision and cannot publish to crates.io.
`gpui-kit-material-color` has a publish dry run in CI. In place: `rust-version`,
`CHANGELOG.md`, `[package.metadata.docs.rs]`, `deny.toml`, and CI on the three desktop
systems and WASM, including a downstream build from Git.
