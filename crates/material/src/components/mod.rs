//! Material components. Each module documents the M3 component it
//! implements and what remains for M3 Expressive (`docs/ROADMAP.md`).
mod button;
pub use button::{
    ButtonDirection, ButtonIconPosition, ButtonProgressPlayback, ButtonShape, ButtonSize,
    ButtonTransitionInputs, ButtonTransitionPlayback, ButtonVariant, FabSize, IconButtonWidth,
    MaterialButton, material_button,
};
mod fab;
pub use fab::{ExtendedFab, Fab, FabColor};
mod fab_menu;
pub use fab_menu::{FabMenu, FabMenuColor, FabMenuItem};
mod icon_button;
mod split_button;
mod standard_button_group;
pub use icon_button::{IconButton, IconButtonVariant};
pub use split_button::{SplitButton, SplitButtonPart};
pub use standard_button_group::{ButtonGroup, EXPANDED_RATIO, standard_group_gap};
mod button_group;
mod controls;
mod divider;
mod switch;
pub use controls::{MaterialSideSheet, MaterialSlider, SearchField, SliderAppearance};
pub use divider::{
    DividerContext, DividerDirection, DividerOrientation, DividerVariant, MaterialDivider,
};
pub use switch::{MaterialSwitch, SwitchIcons};
mod caption;
mod checkbox;
mod chips;
mod choice;
mod feedback;
mod focus;
mod navigation;
mod preview;
mod progress;
mod radio;
mod roving;
mod roving_list;
mod tabs;
mod text;
mod tile;
mod title_bar;

pub use button_group::ConnectedButtonGroup;
pub use caption::{Caption, CaptionButton, CaptionButtons, CaptionLabels, title_bar_control};
pub use checkbox::{CheckboxState, MaterialCheckbox};
pub use chips::{AssistChip, CHIP_HEIGHT, ChipStyle, FilterChip, InputChip, SuggestionChip, chip};
pub use choice::{ChoiceGroup, ChoiceTile, Tint, choice_radio, tinted_icon};
pub use feedback::{
    BADGE_CONTAINER, BADGE_LABEL, BadgeKind, CARD_PADDING, CardSurface, CardVariant,
    LARGE_BADGE_PADDING, LARGE_BADGE_SIZE, NoticeKind, SMALL_BADGE_SIZE, Snackbar, StatusKind,
    badge, card, card_surface, large_badge, notice_banner, outlined_badge, outlined_status_chip,
    small_badge, status_chip,
};
pub use focus::{FocusRing, RevealFocus, RingedButton, contain_scroll};
pub use navigation::{
    NavBadge, NavIcon, NavItem, NavItemLayout, NavigationBar, NavigationItem, NavigationRail,
    RAIL_COLLAPSED_WIDTH, RAIL_EXPANDED_WIDTH, RailAlignment, navigation_scrim,
};
pub use preview::InteractionPreview;
pub use progress::{
    CircularProgressFrame, CircularProgressPlayback, circular_indeterminate_frame,
    circular_progress, icon_spinner, icon_spinner_with_playback, indeterminate_circular_progress,
    indeterminate_circular_progress_with_playback, indeterminate_linear_progress_with_playback,
    indeterminate_progress, indeterminate_progress_with_playback, linear_progress, progress,
    sample_circular_progress, spinner, spinner_with_playback,
};
pub use radio::{MaterialRadio, MaterialRadioGroup};
pub use tabs::{MaterialTabs, TabItem, TabsVariant};
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
    roving_list::init(cx);
    choice::init(cx);
    radio::init(cx);
}
