# Extraction from course2md

Where each shared course2md desktop primitive now lives. course2md has not
switched to the kit yet; this table is the guide for that change.

| course2md (`desktop/src`) | gpui-kit-material |
| --- | --- |
| `palettes.rs` | `color::palette` (serde behind the `serde` feature) |
| `theme.rs` tokens `ColorToken` / `CANVAS`… | `color::ColorRole::CANVAS`… (`ColorRole` avoids GPUI's accessibility `Role`) |
| `theme::palette_colors` | `color::Scheme::from_palette` |
| `theme::color`, `blend`, `apply_preference`, `system_dark` | `theme::color`, `theme::blend`, `theme::apply` (returns whether paint changed), `theme::system_dark` |
| `theme::apply_scale` | `theme::apply_scale` (gpui-component sync stays in the app) |
| radii, `TILE_*`, `tile_corners`, `list_corners` | `tokens::shape` (`OuterCorners`) |
| `TEXT_*` | `tokens::typography` (+ `rem(px)`) |
| `CONTROL_HEIGHT`, `ICON_SLOT`, `SETTING_ITEM_INSET`, `COMPACT_WINDOW`, `title_bar_height` | `tokens::layout` (`ITEM_INSET`, `app_bar_height`) |
| `shadow_card`, `shadow_popover` | `tokens::elevation::{level1, popover}` |
| `motion.rs` springs, `enter`, `state_enter`, `page_enter`, `value`, `selection_value`, `disclosure` | `motion` |
| `motion::{spinner, icon_spinner, progress, linear_progress, indeterminate_progress}` | `components` (progress) |
| `choice_group::SingleChoiceGroup` | `components::ConnectedButtonGroup` |
| `TileScale`, `TileGroup`, `tile`, `list_tile`, `tile_info`, `tile_chevron`, `disclosure_chevron` | `components` (tile); `TileScale::button` is now a `gpui_base::Button` |
| `ChoiceTile`, `choice_radio`, `Tint`, `tinted_icon` | `components` (choice) |
| `accessible_text`, `error_text`, `semantic_label`, `supporting_info`, `info_callout`, `page_heading`, `section_title`, `section_heading` | `components` (text) |
| `badge`, `outlined_badge`, `chip`, `notice_banner`, `card_surface` | `components` (feedback) |
| `focus_scroll::{FocusRing, RevealFocus, contain_scroll}` | `components` (focus) |
| `window_controls` | `components::{CaptionButtons, Caption, CaptionLabels, title_bar_control}` |
| `icons.rs` (subset used by components) | `icons::{IconName, Icon, Assets}` |

## Stayed in course2md

- **Wrappers around gpui-component controls**: `control`, `primary_pill`,
  `tonal_pill`, `outline_pill`, `text_button`, `icon_button`, `danger_pill`,
  `text_input`, `search_field`, `confirm_dialog`, `preference_switch`,
  `SelectField`, `menu_style`, scrollbars. They rely on course2md's reviewed
  gpui-component patches (`MenuStyle`, `label_text_size`, switch geometry). The
  roadmap replaces them with kit components on GPUI Base.
- **Product-specific layout**: `hero_top`, `task_dialog_top`, image viewer
  metrics, `cover_frame`, `nav_destination*`, `detail_row`, `transfer_status`.
- **Localization**: `tr!` strings now arrive as parameters.
