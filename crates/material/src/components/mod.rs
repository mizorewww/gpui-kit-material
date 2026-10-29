//! Material components. Each module documents the M3 component it
//! implements and what remains for M3 Expressive (`docs/ROADMAP.md`).
mod button;
pub use button::{
    ButtonDirection, ButtonIconPosition, ButtonShape, ButtonSize, ButtonVariant, MaterialButton,
    material_button,
};
mod button_group;
mod controls;
pub use controls::{
    MaterialSideSheet, MaterialSlider, MaterialSwitch, SearchField, SliderAppearance,
};
mod caption;
mod choice;
mod feedback;
mod focus;
mod progress;
mod text;
mod tile;

pub use button_group::ConnectedButtonGroup;
pub use caption::{Caption, CaptionButton, CaptionButtons, CaptionLabels, title_bar_control};
pub use choice::{ChoiceTile, Tint, choice_radio, tinted_icon};
pub use feedback::{
    BadgeKind, NoticeKind, badge, card_surface, chip, notice_banner, outlined_badge,
};
pub use focus::{FocusRing, RevealFocus, contain_scroll};
pub use progress::{
    CircularProgressFrame, circular_indeterminate_frame, circular_progress, icon_spinner,
    indeterminate_progress, linear_progress, progress, sample_circular_progress, spinner,
};
pub use text::{
    accessible_text, error_text, info_callout, page_heading, section_heading, section_title,
    semantic_label, supporting_info,
};
pub use tile::{
    TileGroup, TileScale, disclosure_chevron, list_tile, tile, tile_chevron, tile_info,
};

pub(crate) fn init(cx: &mut gpui::App) {
    button_group::init(cx);
}
