# Roadmap

**Target:** every component in Google's Figma Material 3 Design Kit plus the
union of <https://md3.io/> and <https://m3e.kyralabs.dev/>. Where references
disagree, priority is Figma kit > md3.io > m3e, per token family and per value
([Reference sources](ARCHITECTURE.md#reference-sources)).

A component is *implemented* when the real component ships with gallery page
and tests. It is *stable* only when it meets [PRODUCTION.md](PRODUCTION.md);
none does yet. [COVERAGE.md](COVERAGE.md) has the per-component audit and
[conformance/md3.json](../conformance/md3.json) the review record (every entry
is still `unreviewed`).

## Foundations

| Area | State | Remaining |
| --- | --- | --- |
| Color roles and reference themes | Implemented | Full M3 role set; Figma baseline scheme, md3.io for the other 19 themes. |
| Dynamic color | Implemented | HCT, tonal palettes and seed schemes with contrast levels, ported from material-color-utilities. Image-derived seeds and multiple source colors are missing. |
| Scoped theme | Implemented, with limits | `scope::Scope` gives a subtree (or a window's content) its own animated scheme through render, paint, Material's deferred draws and Kit's theme. Kit's `Root` overlays, Kit tooltips, Kit's scrollbar styles and what Kit paints in its own deferred draws stay on the application theme (`crates/material/src/scope.rs`). |
| Motion | Implemented | Expressive and Standard schemes, selectable per app. |
| Shape | Implemented | Corner scale, press morph, the Expressive shape library (35 shapes) and general morphing. Clipping content to a shape is missing. |
| Typography | Implemented, with limits | 15 roles and emphasized styles, Roboto 400/500/600. Letter spacing paints on text elements through a fork patch (`vendor/gpui-pre/letter-spacing-patch.md`); Kit's editors shape their own lines and do not apply it, and ligatures stay on. |
| Icons | Implemented | Material Symbols at the pinned revision; six approximate Kit translations await review. |
| RTL | Implemented, with limits | Direction is inherited (application, window, `Scope`) and layout mirrors in the engine (`vendor/gpui-pre/layout-direction-patch.md`), with icons, painted bars, pointer math and arrow keys handled per component. Needs a review pass with a real RTL locale and committed captures. |

## Components

| Group | Implemented | Missing |
| --- | --- | --- |
| Actions | button, toggle button, icon button, button group, connected group (`segmented()`), split button, FAB, extended FAB, FAB menu | Figma segmented density -3 |
| Selection | checkbox, switch, radio, assist/filter/input/suggestion chips, chip set, slider | |
| Text input | text field, select, search bar and view, date picker, time picker | |
| Navigation | navigation rail, bar, drawer and suite, tabs, menu, top app bar, bottom app bar, docked and floating toolbar | XR app bar, rail, bar, toolbar |
| Containment | card, list, carousel, divider, dialog, bottom sheet, side sheet | Figma stacked and horizontal card layouts, list and scrollable-list dialogs, multi-aspect carousel; XR dialog |
| Communication | badge, avatar, linear and circular progress (flat, wavy), loading indicator (with contained form), snackbar, plain and rich tooltip, pull to refresh | |
| Layout and primitives | m3e canonical layouts: list-detail, supporting pane, feed | m3e primitives: no public generic surface |

No reference defines a contained linear or circular progress; only the loading
indicator has a contained form, and it is implemented.

## Known gaps in implemented components

Details and evidence are in [COVERAGE.md](COVERAGE.md). The cross-cutting ones:

- **Touch.** Press feedback comes from the shared ripple. Controls that take a held
  state beyond the wave have it only where listed in COVERAGE.md (the switch).
- **Evidence.** No component has committed light/dark captures against the Figma
  kit, independent approval, or per-component performance results.

## Adoption

Switch [course2md](https://github.com/mizorewww/course2md) to these adapters
(mapping in [EXTRACTION.md](EXTRACTION.md)), removing its redundant local
implementations while keeping shared Kit behavior.

## Principles

- Cite official Material guidance per component; label project choices as such.
- Material owns looks and motion; Kit Component and Base own behavior. Prefer
  upstream fixes; keep narrow compatibility patches only until replacements
  pass the same regressions.
- Every component works in every theme, light and dark, at 100–200% text.
