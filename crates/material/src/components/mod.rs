//! Material components. Each module documents the M3 component it
//! implements and what remains for M3 Expressive (`docs/ROADMAP.md`).
mod button;
pub use button::{
    ButtonDirection, ButtonIconPosition, ButtonProgressPlayback, ButtonShape, ButtonSize,
    ButtonTone, ButtonTransitionInputs, ButtonTransitionPlayback, ButtonVariant, FabSize,
    IconButtonWidth, MaterialButton, material_button,
};
mod fab;
pub use fab::{ExtendedFab, Fab, FabColor, FabPosition};
mod fab_menu;
pub use fab_menu::{FabMenu, FabMenuAlign, FabMenuColor, FabMenuItem, FabMenuPresentation};
mod icon_button;
mod split_button;
mod standard_button_group;
pub use icon_button::{IconButton, IconButtonVariant};
pub use split_button::{SplitButton, SplitButtonPart};
pub use standard_button_group::{ButtonDensity, ButtonGroup, EXPANDED_RATIO, standard_group_gap};
// Forced accessibility trees in tests (GPUI_A11Y_ACTIVE) exist only in GPUI Fast.
#[cfg(all(test, feature = "gpui-fast"))]
mod a11y_tests;
mod app_bar;
mod avatar;
mod badge;
pub use badge::{Badge, BadgeColor, BadgePosition, badge_label};
mod button_group;
#[cfg(test)]
mod color_guard_tests;
#[cfg(test)]
mod controls;
mod divider;
mod slider;
mod snackbar;
mod switch;
mod toolbar;
pub use divider::{
    DividerContext, DividerDirection, DividerOrientation, DividerVariant, MaterialDivider,
};
pub use search::{
    SEARCH_BAR_HEIGHT, SEARCH_SUGGESTION_HEIGHT, SearchField, SearchSuggestion, SearchView,
    SearchViewLayout, SearchViewStyle,
};
pub use slider::{MaterialSlider, SliderAppearance, SliderColor, SliderSize};
pub use snackbar::{SNACKBAR_CLOSE_ICON, Snackbar, SnackbarDuration, SnackbarPosition};
pub use switch::{MaterialSwitch, SwitchIcons};
mod caption;
mod carousel;
mod date_parts;
mod date_picker;
mod time_picker;
pub use chrono::{Datelike, NaiveDate, NaiveTime, Timelike, Weekday};
pub use date_parts::{
    CalendarDay, DAY_CELL, DayRange, PickerListItem, PickerMenuButton, YearChip, add_months,
    first_of_month, month_weeks, week_days,
};
pub use date_picker::{
    DATE_INPUT_PICKER_WIDTH, DATE_PICKER_WIDTH, DateFormat, DateInputMode, DateMatcher, DatePicker,
    DatePickerEvent, DatePickerLabels, DatePickerState, DatePickerVariant, DatePickerView,
    DateSelection, PickerDialog, today,
};
pub use time_picker::{
    ClockDial, DIAL_SIZE, PeriodSelector, TimeFormat, TimeInputMode, TimeOrientation, TimePicker,
    TimePickerEvent, TimePickerLabels, TimePickerState, TimeSelector, TimeUnit,
};
mod checkbox;
mod chip_set;
mod chips;
mod choice;
mod dialog;
mod feedback;
mod focus;
mod prepaint_bounds;
pub use prepaint_bounds::{PrepaintBounds, prepaint_bounds};
mod labeled;
mod list;
mod loading_indicator;
mod menu;
mod navigation;
mod navigation_drawer;
mod preview;
mod progress;
mod radio;
mod ripple;
#[cfg(test)]
mod ripple_tests;
mod roving;
mod search;
mod sheets;
mod tabs;
mod text;
mod text_field;
mod tile;
mod title_bar;
mod tooltip;

pub use app_bar::{
    BOTTOM_APP_BAR_HEIGHT, BottomAppBar, TOP_APP_BAR_HEIGHT, TopAppBar, TopAppBarVariant,
    collapsed_title_alpha,
};
pub use avatar::{AVATAR_SIZE, Avatar, AvatarContent, monogram_initials};
pub use button_group::{ConnectedButtonGroup, GroupLabels, GroupSelection};
pub use caption::{Caption, CaptionButton, CaptionButtons, CaptionLabels, title_bar_control};
pub use carousel::{
    CAROUSEL_ITEM_RADIUS, CAROUSEL_SMALL_ITEM, CarouselItem, CarouselLayout, Keyline, Keylines,
    MaterialCarousel,
};
pub use checkbox::{CheckboxState, MaterialCheckbox};
pub use chip_set::{ChipSelection, ChipSet, ChipSetItem};
pub use chips::{AssistChip, CHIP_HEIGHT, ChipStyle, FilterChip, InputChip, SuggestionChip, chip};
pub use choice::{ChoiceGroup, ChoiceTile, Tint, choice_radio, tinted_icon};
pub use dialog::{
    DIALOG_WIDTH, DialogActionAlignment, DialogAnimation, DialogSize, MaterialDialog, SCRIM_OPACITY,
};
pub use feedback::{
    BADGE_CONTAINER, BADGE_LABEL, CARD_PADDING, CardState, CardSurface, CardVariant,
    LARGE_BADGE_PADDING, LARGE_BADGE_SIZE, MaterialCard, NoticeKind, SMALL_BADGE_SIZE, StatusKind,
    card, card_surface, large_badge, notice_banner, outlined_status_chip, small_badge, status_chip,
};
#[allow(deprecated)]
pub use feedback::{BadgeKind, badge, outlined_badge};
pub use focus::{FocusRing, RevealFocus, RingedButton, contain_scroll};
pub use labeled::{LabelPosition, required_label};
pub use list::{
    LIST_SEGMENT_GAP, LIST_SEGMENT_INNER_RADIUS, LIST_SEGMENT_RADIUS, ListDensity, ListItem,
    ListLeading, ListLines, ListTrailing, list_subheader, segmented_list,
};
pub use loading_indicator::{
    LOADING_INDICATOR_SIZE, LoadingFrame, LoadingIndicator, LoadingShape, loading_indicator_frame,
};
pub use menu::{
    ExposedDropdown, MaterialMenu, MaterialMenuItem, MenuDensity, MenuGroup, MenuPopup, MenuStyle,
};
pub use navigation::{
    MEDIUM_WIDTH, NavBadge, NavIcon, NavItem, NavItemLayout, NavigationBar, NavigationItem,
    NavigationRail, RAIL_COLLAPSED_WIDTH, RAIL_EXPANDED_WIDTH, RailAlignment, navigation_scrim,
};
pub use navigation_drawer::{DRAWER_WIDTH, DrawerPosition, NavigationDrawer};
mod adaptive;
pub use adaptive::{
    SizedContent, WindowHeightClass, WindowSizeClass, WindowWidthClass, sized_content,
};
mod pane_logic;
mod pane_scaffold;
pub use pane_logic::{
    AdaptStrategies, AdaptStrategy, BackNavigationBehavior, DEFAULT_PANE_ANCHORS,
    LIST_DETAIL_ORDER, LIST_DETAIL_STRATEGIES, PaneAdaptedValue, PaneAnchor, PaneDestination,
    PaneDirective, PaneLayout, PaneLayoutInput, PaneMotion, PaneNavigator, PaneRect, PaneRole,
    PaneRule, PaneTransition, SUPPORTING_PANE_ORDER, SUPPORTING_PANE_STRATEGIES, ThreePaneValue,
    closest_pane_anchor, next_pane_anchor, pane_directive, pane_motions, pane_transitions,
    previous_pane_anchor, three_pane_layout, three_pane_value,
};
pub use pane_scaffold::{
    FEED_COLUMN_MIN_WIDTH, FEED_GAP, FEED_MARGIN, FEED_MARGIN_COMPACT, FeedLayout, FeedMetrics,
    PaneScaffold, feed_metrics,
};
mod pull_to_refresh;
pub use pull_to_refresh::{
    PULL_TO_REFRESH_DRAG_MULTIPLIER, PULL_TO_REFRESH_INDICATOR_SIZE,
    PULL_TO_REFRESH_LOADING_INDICATOR_SIZE, PULL_TO_REFRESH_MAX_DISTANCE,
    PULL_TO_REFRESH_MIN_ALPHA, PULL_TO_REFRESH_SPINNER_SIZE, PULL_TO_REFRESH_THRESHOLD, PullArrow,
    PullModel, PullToRefresh, PullToRefreshIndicator, pull_arrow, pull_loading_rotation,
    pull_vertical_offset,
};
mod navigation_suite;
pub use navigation_suite::{
    NavigationSuite, NavigationSuiteType, SUITE_PRIMARY_ACTION_PADDING, navigation_suite_type,
};
pub use preview::InteractionPreview;
pub use progress::{
    CircularProgress, CircularProgressFrame, CircularProgressPlayback, LINEAR_INDETERMINATE_MS,
    LinearProgress, PROGRESS_THICK_THICKNESS, PROGRESS_THICKNESS, ProgressShape,
    circular_indeterminate_frame, circular_progress, circular_progress_size, icon_spinner,
    icon_spinner_with_playback, indeterminate_circular_progress,
    indeterminate_circular_progress_with_playback, indeterminate_linear_progress_with_playback,
    indeterminate_progress, indeterminate_progress_with_label,
    indeterminate_progress_with_playback, linear_indeterminate_lines, linear_progress, progress,
    progress_with_label, sample_circular_progress, spinner, spinner_with_playback,
};
pub use radio::{MaterialRadio, MaterialRadioGroup};
pub use ripple::Ripple;
pub use sheets::{BottomSheetDetent, MaterialBottomSheet, MaterialSideSheet, SideSheetEdge};
pub use tabs::{MaterialTabs, TabItem, TabsVariant};
pub use text::{
    accessible_text, error_text, info_callout, page_heading, section_heading, section_title,
    semantic_label, supporting_info,
};
pub use text_field::{
    TEXT_FIELD_HEIGHT, TEXT_FIELD_WIDTH, TextField, TextFieldDensity, TextFieldInputType,
    TextFieldVariant,
};
pub use tile::{
    TileGroup, TileScale, disclosure_chevron, list_tile, tile, tile_chevron, tile_info,
};
pub use title_bar::{
    TitleBar, title_bar_leading, title_bar_room, title_bar_window_options, traffic_light_position,
};
pub use toolbar::{
    TOOLBAR_FAB_GAP, TOOLBAR_MARGIN, TOOLBAR_SIZE, Toolbar, ToolbarButton, ToolbarColor,
    ToolbarConfiguration, ToolbarOrientation,
};
pub use tooltip::{MaterialTooltip, PLAIN_TOOLTIP_MAX_WIDTH, RICH_TOOLTIP_WIDTH};

pub(crate) fn init(cx: &mut gpui::App) {
    roving::init(cx);
    menu::init(cx);
    date_picker::init(cx);
}
