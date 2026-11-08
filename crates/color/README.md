# gpui-kit-material-color

Material 3 color for [gpui-kit-material](https://github.com/mizorewww/gpui-kit-material),
with no GPUI dependency.

- `ColorRole` and `Scheme`: the semantic roles Material components paint with,
  and a scheme derived from a built-in palette.
- `themes`: the 20 md3.io reference presets in light and dark, at standard,
  medium and high contrast.
- `dynamic` (default feature): Material dynamic color ported from
  [material-color-utilities](https://github.com/material-foundation/material-color-utilities):
  HCT, tonal palettes and seed schemes.
- `serde` (default feature): serializable `ThemePreferences`.

```rust
use gpui_kit_material_color::{Color, ColorRole, dynamic::{SeedScheme, Variant}};

let scheme = SeedScheme::new(0x6750a4).variant(Variant::TonalSpot).scheme();
assert_eq!(scheme.get(ColorRole::PRIMARY), Color::hex(0x65558f));
```

Palette sources: [PALETTES.md](PALETTES.md).

## License

MIT, except the material-color-utilities port in `src/dynamic` (Apache-2.0).
See `licenses/`.
