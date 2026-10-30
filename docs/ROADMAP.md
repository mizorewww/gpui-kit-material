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
| Dynamic color | Missing | HCT, tonal palettes and seed schemes with contrast levels. |
| Scoped theme | Missing | `theme::color` reads one thread-local scheme, so a theme cannot apply to a subtree (a Gallery preview, or per window). |
| Motion | Implemented | Expressive and Standard schemes, selectable per app. |
| Shape | Partial | Corner scale, press morph and the 8 loading-indicator shapes. The Expressive shape library (35 shapes) and general morphing are missing. |
| Typography | Partial | 15 roles and emphasized styles, Roboto 400/500/600. Letter spacing is recorded but not painted: GPUI 0.3.10 exposes no tracking. |
| Icons | Implemented | Material Symbols at the pinned revision; six approximate Kit translations await review. |
| RTL | Missing | Only `MaterialButton` and `MaterialDivider` take a direction. No mirrored layout elsewhere, and no inherited direction. |

## Components

| Group | Implemented | Missing |
| --- | --- | --- |
| Actions | button, toggle button, icon button, button group, connected group (`segmented()`), split button, FAB, extended FAB, FAB menu | Figma segmented density -3 |
| Selection | checkbox, switch, radio, assist/filter/input/suggestion chips, slider | chip group type |
| Text input | text field, select, search bar and view, date picker, time picker | |
| Navigation | navigation rail, bar and drawer, tabs, menu, top app bar, bottom app bar, docked and floating toolbar | navigation suite; XR app bar, rail, bar, toolbar |
| Containment | card, list, carousel, divider, dialog, bottom sheet, side sheet | Figma stacked and horizontal card layouts, list and scrollable-list dialogs, multi-aspect carousel; XR dialog |
| Communication | badge, avatar, linear and circular progress (flat, wavy), loading indicator (with contained form), snackbar, plain and rich tooltip | pull to refresh |
| Layout | | m3e canonical layouts |

No reference defines a contained linear or circular progress; only the loading
indicator has a contained form, and it is implemented.

## Known gaps in implemented components

Details and evidence are in [COVERAGE.md](COVERAGE.md). The cross-cutting ones:

- **Gallery-only options.** Some md3.io options exist only in the playground,
  not as component API: visible labels and `required` on checkbox and switch,
  read-only, required and input types on text field, snackbar duration and
  position, FAB position, search expand/collapse behavior, date format, chip
  set behavior.
- **Ripple.** Only buttons, navigation, tabs, cards and toolbars ripple.
  Selection controls, chips, lists, menus and pickers have state layers only.
- **Touch.** Raw touch handling exists in the button only.
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
