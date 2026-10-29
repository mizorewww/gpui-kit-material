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
