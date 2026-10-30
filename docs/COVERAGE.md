# Coverage audit

Every component and variant in the three references, against the
[production standard](PRODUCTION.md). Audited at commit `45de656` by reading the
component source, playgrounds and tests, the Figma kit page outlines, the
md3.io configuration panels, and the m3e documentation pages.

**Result: 0 components are production-ready.** Of the 46 rows below, 41 are
implemented, 40 with a playground, all with tests. None has independent approval, committed
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
- Test counts are test functions at the audited commit, not a measure of quality.

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
| Button | Y | Y | Y | `MaterialButton` | Yes | 66 + 0 | attr, RM | option + inherit | Figma tokens, md3.io timing captures | none found |
| Toggle button | Y | Y | Y | `MaterialButton::toggle` | Yes (button) | in Button | attr | inherit | as Button | none found |
| Icon button | Y | Y | Y | `IconButton` | Yes | 1 + 3 | attr | inherit | md3.io capture | none found |
| Button group | Y | Y | Y | `ButtonGroup`, `ConnectedButtonGroup` | Yes | 21 + 4 | attr | inherit | none | none found |
| Segmented button | Y | - | Y | `ConnectedButtonGroup::segmented` | Specs only | in Button group | attr | inherit | none | No playground; Figma density -3 |
| Split button | Y | Y | Y | `SplitButton` | Yes | 3 + 3 | code | inherit | none | none found |
| FAB | Y | Y | Y | `Fab` | Yes | 2 + 2 | attr | inherit | none | none found |
| Extended FAB | Y | Y | - | `ExtendedFab` | Yes | in FAB + 3 | attr | inherit | none | none found |
| FAB menu | Y | Y | Y | `FabMenu` | Yes | 5 + 3 | attr | inherit | none | none found |

## Selection and input

| Component | F | M | E | Rust | Playground | Tests | A11y | RTL | Evidence | Missing |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Checkbox | Y | Y | Y | `MaterialCheckbox` | Yes | 4 + 2 | attr | inherit | none | none found; `required` is a mark in the name, as GPUI has no required state |
| Switch | Y | Y | Y | `MaterialSwitch` | Yes | 8 + 1 | attr | inherit | none | none found; `required` is a mark in the name, as GPUI has no required state |
| Radio | Y | Y | Y | `MaterialRadio`, `MaterialRadioGroup` | Yes | 5 + 1 | attr | inherit | none | none found |
| Chips | Y | Y | Y | `AssistChip`, `FilterChip`, `InputChip`, `SuggestionChip`, `ChipSet` | Yes | 8 + 2 | code | inherit | none | none found |
| Slider | Y | Y | Y | `MaterialSlider` | Yes | 17 + 1 | attr | tested | none | No ripple by design: neither the Figma kit nor md3.io draws one on the Expressive handle |
| Text field | Y | Y | Y | `TextField` | Yes | 8 + 1 | attr | inherit | none | Input types mask and filter; platforms here have no soft keyboard to select. `required` is a mark in the name |
| Select | - | Y | - | `ExposedDropdown` | Yes | in Menu + 1 | code | inherit | none | `required` is a mark in the name |
| Search | Y | Y | Y | `SearchField`, `SearchView` | Yes | 8 + 2 | attr | inherit | none | none found |
| Date picker | Y | Y | Y | `DatePicker` | Yes | 9 + 2 | attr | inherit | none | none found |
| Time picker | Y | Y | Y | `TimePicker` | Yes | 9 + 2 | code | exempt (dial) | none | The dial has no ripple by design: the Figma kit draws only its selector |

## Navigation

| Component | F | M | E | Rust | Playground | Tests | A11y | RTL | Evidence | Missing |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Navigation rail | Y | Y | Y | `NavigationRail` | Yes | 16 (shared) + 2 | attr, RM | inherit | none | XR rail |
| Navigation bar | Y | Y | Y | `NavigationBar` | Yes | shared + 2 | attr, RM | inherit | none | XR bar |
| Navigation drawer | Y | Y | Y | `NavigationDrawer` | Yes | 5 + 2 | attr | inherit | none | none found |
| Navigation suite | - | - | Y | none | No | 0 | - | - | none | **Not implemented**; only `NavigationBar::adaptive` |
| Tabs | Y | Y | Y | `MaterialTabs` | Yes | 9 + 2 | attr, RM | tested | none | none found |
| Menu | Y | Y | Y | `MaterialMenu` | Yes | 16 + 3 | code | inherit | none | none found |
| Top app bar | Y | Y | Y | `TopAppBar` | Yes | 13 (shared) + 3 | attr, RM | inherit | none | XR app bar; no Small-image option, only a title slot |
| Bottom app bar | Y | Y | - | `BottomAppBar` | Yes | shared + 2 | attr, RM | inherit | none | none found |
| Docked toolbar | Y | Y | Y | `Toolbar` (docked) | Yes | 14 (shared) + 3 | attr, RM | inherit | none | none found |
| Floating toolbar | Y | Y | Y | `Toolbar` (floating) | Yes | shared | attr, RM | inherit | none | XR toolbar and its building blocks |

## Containment

| Component | F | M | E | Rust | Playground | Tests | A11y | RTL | Evidence | Missing |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Card | Y | Y | Y | `MaterialCard`, `CardSurface` | Yes | 5 + 2 | attr | inherit | Source audit only | Figma stacked and horizontal layouts. Image, title, body and actions are playground composition over slots |
| List | Y | Y | Y | `ListItem`, `segmented_list` | Yes | 6 + 2 | tree | inherit | none | none found |
| Carousel | Y | Y | Y | `MaterialCarousel` | Yes | 15 + 2 | tree | inherit | none | Figma multi-aspect ratio. No hover or pressed state, click handler or `preview_state` |
| Divider | Y | Y | Y | `MaterialDivider` | Yes | 5 + 1 | attr | option + inherit | md3.io capture | none found |
| Dialog | Y | Y | Y | `MaterialDialog` | Yes | 6 + 3 | tree | inherit | none | Figma list and scrollable-list forms (a `content` slot only); XR dialog; no `preview_state` |
| Bottom sheet | Y | Y | Y | `MaterialBottomSheet` | Yes | 10 (shared) + 2 | tree, RM | inherit | none | Drag handle has no state layer; no `preview_state` |
| Side sheet | Y | Y | Y | `MaterialSideSheet` | Yes | shared + 2 | tree, RM | inherit | none | No `preview_state` |

## Communication

| Component | F | M | E | Rust | Playground | Tests | A11y | RTL | Evidence | Missing |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Badge | Y | Y | Y | `Badge` | Yes | 3 + 2 | attr | inherit | none | none found |
| Avatar | Y | - | - | `Avatar` | Yes | 3 + 0 | attr | inherit | none | none found |
| Linear and circular progress | Y | Y | Y | `LinearProgress`, `CircularProgress` | Yes | 16 + 2 | tree, RM | inherit | none | Wave height and percentage label are not options. No reference defines a contained form |
| Loading indicator | Y | Y | Y | `LoadingIndicator` | Yes | 5 + 1 | attr | inherit | none | none found |
| Snackbar | Y | Y | Y | `Snackbar` | Yes | 7 + 3 | code | inherit | none | none found |
| Tooltip | Y | Y | Y | `MaterialTooltip` | Yes | 4 + 4 | code | inherit | none | No `preview_state` |
| Pull to refresh | - | - | Y | none | No | 0 | - | - | none | **Not implemented**; only a loading-indicator scenario with that name |

## Layout and primitives

| Component | F | M | E | Rust | Playground | Tests | A11y | RTL | Evidence | Missing |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Canonical layouts | - | - | Y | none | No | 0 | - | - | none | **Not implemented**: list-detail, supporting pane, feed |
| Primitives | - | - | Y | Partial: `CardSurface`, `FocusRing`, `Ripple` | No | - | - | - | none | No public generic surface |
| XR variants | Y | - | - | none | No | 0 | - | - | none | **Not implemented**: app bar, dialog, navigation rail and bar, toolbar |

## Foundations

| Area | State | Source | Evidence | Missing |
| --- | --- | --- | --- | --- |
| Color | Implemented | Figma baseline, md3.io other themes | Fixtures, tests | Dynamic color (HCT, seed schemes) |
| Typography | Implemented, with limits | Figma | Fixtures, tests | Kit's editors do not apply letter spacing; ligatures stay on under it |
| Shape | Partial | Figma | Fixtures, tests | Expressive shape library, general morphing |
| Elevation and state layers | Implemented | Figma | Fixtures, tests | none found |
| Motion | Implemented | md3.io (Expressive), m3e (Standard) | Fixtures, tests | No per-component timed comparison outside buttons |
| Icons | Implemented | Material Symbols | Provenance in Symbols repository | Six approximate Kit translations unreviewed |
| Scoped theme | Implemented, with limits | n/a | Tests in `src/scope/tests.rs` | Kit's `Root` overlays, Kit tooltips, Kit's scrollbar styles, drag previews outside `current_scope` and what Kit reads while painting its own deferred draws keep the application theme. Under GPUI Fast a scheme scope on screen rebuilds subtrees that read Kit's theme on each draw |

## Cross-cutting status

Status of each [production criterion](PRODUCTION.md) across all components.

| Criterion | Status |
| --- | --- |
| Controlled state, caller-supplied strings | Followed in source; no automated check |
| Rustdoc with runnable examples | **Not met**: almost no doc examples |
| Pointer and keyboard | Implemented and tested per component to varying depth |
| Touch | Press feedback and cancellation on scroll through the shared ripple, tested on public touch events; a system cancel of a still finger releases only at a 5 s hold limit; only the first finger starts a wave; no real touch device run; 48 dp targets not audited |
| Focus ring | Implemented; carousel has focus but no other states |
| Assistive technology pass | **Not done**: tree assertions only, under GPUI Fast |
| Reduced motion | Supported by the `motion` module; tested only where the tables say `RM` |
| Text scale 100–200% | Runtime support; **no per-component layout tests** |
| RTL | Inherited direction with mirrored layout, tests for mirroring, tabs and slider; **no captures or review with a real RTL locale** |
| Contrast themes | Tokens tested; no per-component capture |
| Visual evidence | **Not met**: no committed light/dark captures against the kit |
| Performance budgets | **Unverified** per component and as a whole on this commit |
| Linux and Windows | Built and unit-tested in CI; **no real gallery run recorded** |
| WASM parity | Same gallery code builds; **no recorded parity comparison** |
| Reachable panics | **Not audited**: about 15 non-test `expect`/`unwrap` sites remain, on constants such as symbol names and easings |
| Independent review | **None**: every entry in `conformance/md3.json` is `unreviewed` |

Release criteria not met: the crates need workspace `[patch]` tables and Git
dependencies, so they cannot publish; no `rust-version`, `CHANGELOG.md`,
`[package.metadata.docs.rs]` or `cargo deny` configuration exists. CI on the three
desktop systems and WASM is in place.
