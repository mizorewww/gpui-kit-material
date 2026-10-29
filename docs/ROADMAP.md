# Roadmap

## 0.1 — Extraction (done)

course2md's shared primitives as a standalone, tested kit on GPUI Base. See
[EXTRACTION.md](EXTRACTION.md).

## Next — Material 3 Expressive

The target is the union of all components at https://m3e.kyralabs.dev/ and
https://md3.io/. For overlaps, https://m3e.kyralabs.dev/ is authoritative, including
visuals, typography, interaction states and animations. Both desktop and WASM
galleries must match the selected reference exactly, with independent subagent
approval for every component and whole-gallery CPU frame time <= 2 ms (including
the first frame). GPU timings are reported separately. Land atomic commits and
push completed milestones to GitHub frequently. The source catalog and acceptance
gates live in [conformance/md3.json](../conformance/md3.json).

Work in this order; each step lands with gallery coverage and tests.

1. **Color**
   - HCT color space and tonal palettes (port of material-color-utilities) in
     `gpui-kit-material-color`.
   - The full M3 role set: `primary`/`on_primary`/`primary_container`…,
     secondary, tertiary, error, `surface_dim`/`surface_bright`,
     `surface_container_lowest…highest`, `outline`, `outline_variant`,
     inverse and fixed roles.
   - Dynamic schemes (tonal spot, expressive, vibrant…) from a seed color, with
     contrast levels. The built-in palettes map onto the new roles; the legacy
     `ColorRole` names stay as deprecated aliases for one release.
2. **Motion**: the Expressive motion scheme (spatial and effects springs,
   fast/default/slow) beside Standard, selectable per app.
3. **Shape**: the full corner scale (none → full), the Expressive shape
   library (35 shapes) and shape morphing for buttons and indicators.
4. **Typography**: the baseline and emphasized type scale (display, headline,
   title, body, label × large/medium/small) as text roles.
5. **Components** (Expressive variants first), on GPUI Base behavior:
   - Buttons: filled, tonal, elevated, outlined, text; sizes XS–XL; toggle
     buttons; shape morph on press; icon buttons; split buttons; FAB, extended
     FAB, FAB menu.
   - Button groups: standard and connected (current `ConnectedButtonGroup`).
   - Selection: checkbox, radio, switch, chips (assist, filter, input,
     suggestion), segmented, slider (with Expressive sizes).
   - Text fields (filled, outlined), search bar and view, menus (Expressive
     vertical menus), select.
   - Containment: cards, lists, dialogs, bottom/side sheets, tooltips
     (plain, rich), dividers.
   - Navigation: app bars (small, medium/large flexible), navigation rail
     (collapsed/expanded), navigation bar, tabs, toolbars (docked, floating).
   - Feedback: progress (wavy and contained linear/circular), loading
     indicator (morphing shapes), badges, snackbar.
6. **Icons**: Material Symbols via
   [gpui-material-symbols](https://github.com/mizorewww/gpui-material-symbols),
   using the sibling `../gpui-material-symbols` repository and replacing the
   Lucide glyphs. Improve that repository as needed for parity and performance;
   keep its compatibility requirements and commit changes atomically there.
7. **course2md adoption**: switch course2md to the kit, removing its local
   copies and gpui-component wrappers as the replacements land.

## Principles

- Official Material guidance is cited per component; project choices are
  labelled as such.
- Components own looks and motion; behavior and accessibility stay in GPUI
  Base. Fix a missing behavior upstream rather than forking it here.
- Every component works in every palette, light and dark, at 100–200% text.
