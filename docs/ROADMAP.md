# Roadmap

**Target:** the union of all components at <https://m3e.kyralabs.dev/> and
<https://md3.io/>. Where they overlap, m3e.kyralabs.dev is authoritative for the
whole component — visuals, typography, states and animations. The component
catalog, evidence and acceptance gates live in
[conformance/md3.json](../conformance/md3.json); a component is done only when it
meets [AGENTS.md](../AGENTS.md) (desktop + WASM gallery, tests, independent
review, frame budget).

Legend: ✅ implemented (conformance review still pending) · 🟡 partial · ⬜ not started

## Foundations

| Area | State | Remaining |
| --- | --- | --- |
| Color roles & reference themes | ✅ | Full M3 role set and 20 md3.io themes × mode × contrast. |
| Dynamic color | ⬜ | HCT, tonal palettes and seed schemes (tonal spot, expressive, vibrant…) with contrast levels; port of material-color-utilities. |
| Motion | ✅ | Expressive and Standard schemes, selectable per app. |
| Shape | 🟡 | Corner scale and press morph exist; Expressive shape library (35 shapes) and general shape morphing pending. |
| Typography | 🟡 | 15 roles as tokens; letter-spacing rendering and emphasized scale pending. |
| Icons | ✅ | Material Symbols via the pinned gpui-material-symbols revision; six approximate Kit translations await review. |

## Components

Expressive variants first, each adapting Kit `gpui-component` controls (Base
only for verified gaps).

| Group | Implemented | Not started |
| --- | --- | --- |
| Buttons | ✅ button (5 variants, XS–XL, toggle, loading, morph) · ✅ connected button group | icon button, split button, FAB, extended FAB, FAB menu, segmented |
| Selection | 🟡 switch, slider · 🟡 chip | checkbox, radio, chip variants (assist/filter/input/suggestion) |
| Input | 🟡 search field | text fields (filled/outlined), search view, select, menus, date/time pickers |
| Containment | ✅ divider · 🟡 card, side sheet, tiles | lists, dialogs, bottom sheet, tooltips, carousel |
| Navigation | 🟡 custom title bar | app bars, navigation rail/bar/drawer, tabs, toolbars |
| Feedback | 🟡 linear/circular progress, badge, snackbar | wavy/contained progress, loading indicator (morphing shapes), pull to refresh |

## Adoption

Switch [course2md](https://github.com/mizorewww/course2md) to these adapters
(mapping in [EXTRACTION.md](EXTRACTION.md)), removing its redundant local
implementations while keeping shared Kit behavior.

## Principles

- Cite official Material guidance per component; label project choices as such.
- Material owns looks and motion; Kit Component and Base own behavior. Prefer
  upstream fixes; keep narrow compatibility patches only until replacements
  pass the same regressions.
- Every component works in every palette, light and dark, at 100–200% text.
