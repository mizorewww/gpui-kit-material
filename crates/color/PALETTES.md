# Built-in palette sources

Retrieved 2026-09-08 for course2md, where these palettes originated.
`src/palette.rs` holds the stored colors; `src/scheme.rs` derives the
semantic roles (containers, status tints, elevated surfaces) from them.

| Family | Upstream | Mapping |
| --- | --- | --- |
| Paper / Ink | course2md (original) | Original neutral light/dark pair with blue actions. |
| Nord / Nord Snow | [Nord palette](https://www.nordtheme.com/docs/colors-and-palettes), [source](https://github.com/nordtheme/nord/blob/develop/src/nord.scss) | Nord uses Polar Night backgrounds, Snow Storm text and Frost accent. Nord Snow is this application's light adaptation, with light surfaces and darker status foregrounds. |
| Tokyo Night / Day | [Tokyo Night](https://github.com/folke/tokyonight.nvim), [Night](https://github.com/folke/tokyonight.nvim/blob/main/extras/wezterm/tokyonight_night.toml), [Day](https://github.com/folke/tokyonight.nvim/blob/main/extras/wezterm/tokyonight_day.toml) | Retains the upstream backgrounds, blue accents and selection. Night retains its upstream text color; Day uses the application text-role adaptation below. Raised surfaces and some supporting/status text are also adapted for a document interface. |
| Catppuccin | [Palette](https://github.com/catppuccin/palette/blob/main/palette.json) | Latte, Frappé, Macchiato and Mocha use their named text, base, mantle, crust, surface and blue colors. Dark canvas uses mantle and cards use base; Latte's raised surface and green/yellow foregrounds are adapted. |

For Tokyo Day the upstream terminal
foreground is `#3760bf`; this kit uses `#343b58` for body text, `#596078` for
supporting text and `#737b96` for subtle text. These are application choices to
separate reading hierarchy from blue actions, not claims about upstream text
roles. The upstream Day background `#e1e2e7`, blue `#2e7de9` and selection
`#b7c1e3` remain the palette's basis.

Every palette's window ground is painted as Material 3's surface container:
its canvas color mixed with 4.5% (light) or 3% (dark) of its accent, so the
surface-colored tiles of grouped settings and lists read as a layer above it.
Dark palettes also paint their surfaces with 4.5% of their text color, as
Material 3 lifts dark surface containers. The stored colors above are
unchanged.

Original licenses are in `licenses/`. They cover the palette source material;
upstream projects do not endorse this kit or course2md.

## md3.io reference themes

`src/themes/` contains the 20 presets from
[floor/material at c910072](https://github.com/floor/material/tree/c910072f3fc5b660276731d8a935d7186f288364/src/styles/themes),
including light/dark modes and standard/medium/high contrast. Values are resolved
by Chromium after compiling the pinned source with Dart Sass 1.85.1. The source
is MIT licensed; its notice is bundled as `src/themes/LICENSE-material`.

`tests/fixtures/md3-colors.json` records the captured values and environment.
These tables preserve the reference implementation, including its highcontrast
preset's different fixed roles in light and dark. They do not generate colors
from arbitrary seeds. The original palettes above remain separate legacy
adapters, and application extensions (selection, find highlights, and status
containers) are derived by the kit rather than copied reference roles.
