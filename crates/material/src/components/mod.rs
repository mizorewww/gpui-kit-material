//! Material components. Each module documents the M3 component it
//! implements and what remains for M3 Expressive (`docs/ROADMAP.md`).
mod button;
pub use button::{
    ButtonDirection, ButtonIconPosition, ButtonProgressPlayback, ButtonShape, ButtonSize,
    ButtonTransitionInputs, ButtonTransitionPlayback, ButtonVariant, MaterialButton,
    material_button,
};
mod button_group;
mod controls;
mod divider;
pub use controls::{
    MaterialSideSheet, MaterialSlider, MaterialSwitch, SearchField, SliderAppearance,
};
pub use divider::{
    DividerContext, DividerDirection, DividerOrientation, DividerVariant, MaterialDivider,
};
mod caption;
mod choice;
mod feedback;
mod focus;
mod progress;
mod text;
mod tile;
mod title_bar;

pub use button_group::ConnectedButtonGroup;
pub use caption::{Caption, CaptionButton, CaptionButtons, CaptionLabels, title_bar_control};
pub use choice::{ChoiceTile, Tint, choice_radio, tinted_icon};
pub use feedback::{
    BadgeKind, CARD_PADDING, CardVariant, NoticeKind, Snackbar, badge, card, card_surface, chip,
    notice_banner, outlined_badge,
};
pub use focus::{FocusRing, RevealFocus, contain_scroll};
pub use progress::{
    CircularProgressFrame, CircularProgressPlayback, circular_indeterminate_frame,
    circular_progress, icon_spinner, icon_spinner_with_playback, indeterminate_circular_progress,
    indeterminate_circular_progress_with_playback, indeterminate_linear_progress_with_playback,
    indeterminate_progress, indeterminate_progress_with_playback, linear_progress, progress,
    sample_circular_progress, spinner, spinner_with_playback,
};
pub use text::{
    accessible_text, error_text, info_callout, page_heading, section_heading, section_title,
    semantic_label, supporting_info,
};
pub use tile::{
    TileGroup, TileScale, disclosure_chevron, list_tile, tile, tile_chevron, tile_info,
};
pub use title_bar::{
    TitleBar, title_bar_leading, title_bar_window_options, traffic_light_position,
};

pub(crate) fn init(cx: &mut gpui::App) {
    button_group::init(cx);
}
