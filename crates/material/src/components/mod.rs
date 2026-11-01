//! Material components. Each module documents the M3 component it
//! implements and what remains for M3 Expressive (`docs/ROADMAP.md`).
//!
//! Each group of components is a Cargo feature, and `Cargo.toml` declares what a group
//! needs from another. The modules with no gate are the parts they all share.
#[cfg(feature = "buttons")]
mod button;
#[cfg(feature = "buttons")]
pub use button::{
    ButtonDirection, ButtonIconPosition, ButtonProgressPlayback, ButtonShape, ButtonSize,
    ButtonTransitionInputs, ButtonTransitionPlayback, ButtonVariant, FabSize, IconButtonWidth,
    MaterialButton, material_button,
};
#[cfg(feature = "buttons")]
mod fab;
#[cfg(feature = "buttons")]
pub use fab::{ExtendedFab, Fab, FabColor, FabPosition};
#[cfg(feature = "buttons")]
mod fab_menu;
#[cfg(feature = "buttons")]
pub use fab_menu::{FabMenu, FabMenuAlign, FabMenuColor, FabMenuItem, FabMenuPresentation};
#[cfg(feature = "buttons")]
mod icon_button;
#[cfg(feature = "buttons")]
mod split_button;
#[cfg(feature = "buttons")]
mod standard_button_group;
#[cfg(feature = "buttons")]
pub use icon_button::{IconButton, IconButtonVariant};
#[cfg(feature = "buttons")]
pub use split_button::{SplitButton, SplitButtonPart};
#[cfg(feature = "buttons")]
pub use standard_button_group::{ButtonDensity, ButtonGroup, EXPANDED_RATIO, standard_group_gap};
// Forced accessibility trees in tests (GPUI_A11Y_ACTIVE) exist only in GPUI Fast.
#[cfg(all(test, feature = "full", feature = "gpui-fast"))]
mod a11y_tests;
#[cfg(feature = "navigation")]
mod app_bar;
#[cfg(feature = "containment")]
mod avatar;
#[cfg(feature = "feedback")]
mod badge;
#[cfg(feature = "feedback")]
pub use badge::{Badge, BadgeColor, BadgePosition, badge_label};
#[cfg(feature = "buttons")]
mod button_group;
#[cfg(all(test, feature = "full"))]
mod color_guard_tests;
#[cfg(all(test, feature = "full"))]
mod controls;
#[cfg(feature = "containment")]
mod divider;
#[cfg(feature = "selection")]
mod slider;
#[cfg(feature = "feedback")]
mod snackbar;
#[cfg(feature = "selection")]
mod switch;
#[cfg(feature = "navigation")]
mod toolbar;
#[cfg(feature = "containment")]
pub use divider::{
    DividerContext, DividerDirection, DividerOrientation, DividerVariant, MaterialDivider,
};
#[cfg(feature = "inputs")]
pub use search::{
    SEARCH_BAR_HEIGHT, SEARCH_SUGGESTION_HEIGHT, SearchField, SearchSuggestion, SearchView,
    SearchViewLayout, SearchViewStyle,
};
#[cfg(feature = "selection")]
pub use slider::{MaterialSlider, SliderAppearance, SliderColor, SliderSize};
#[cfg(feature = "feedback")]
pub use snackbar::{SNACKBAR_CLOSE_ICON, Snackbar, SnackbarDuration, SnackbarPosition};
#[cfg(feature = "selection")]
pub use switch::{MaterialSwitch, SwitchIcons};
#[cfg(feature = "window-chrome")]
mod caption;
#[cfg(feature = "containment")]
mod carousel;
#[cfg(feature = "pickers")]
mod date_parts;
#[cfg(feature = "pickers")]
mod date_picker;
#[cfg(feature = "pickers")]
mod time_picker;
#[cfg(feature = "pickers")]
pub use chrono::{Datelike, NaiveDate, NaiveTime, Timelike, Weekday};
#[cfg(feature = "pickers")]
pub use date_parts::{
    CalendarDay, DAY_CELL, DayRange, PickerListItem, PickerMenuButton, YearChip, add_months,
    first_of_month, month_weeks, week_days,
};
#[cfg(feature = "pickers")]
pub use date_picker::{
    DATE_INPUT_PICKER_WIDTH, DATE_PICKER_WIDTH, DateFormat, DateInputMode, DateMatcher, DatePicker,
    DatePickerEvent, DatePickerLabels, DatePickerState, DatePickerVariant, DatePickerView,
    DateSelection, PickerDialog, today,
};
#[cfg(feature = "pickers")]
pub use time_picker::{
    ClockDial, DIAL_SIZE, PeriodSelector, TimeFormat, TimeInputMode, TimeOrientation, TimePicker,
    TimePickerEvent, TimePickerLabels, TimePickerState, TimeSelector, TimeUnit,
};
#[cfg(feature = "selection")]
mod checkbox;
#[cfg(feature = "selection")]
mod chip_set;
#[cfg(feature = "selection")]
mod chips;
#[cfg(feature = "selection")]
mod choice;
#[cfg(feature = "dialogs")]
mod dialog;
#[cfg(feature = "feedback")]
mod feedback;
mod focus;
mod interaction;
mod prepaint_bounds;
pub use prepaint_bounds::{PrepaintBounds, prepaint_bounds};
mod labeled;
#[cfg(feature = "containment")]
mod list;
#[cfg(feature = "loading")]
mod loading_indicator;
#[cfg(feature = "menus")]
mod menu;
#[cfg(feature = "navigation")]
mod navigation;
#[cfg(feature = "navigation")]
mod navigation_drawer;
mod preview;
#[cfg(feature = "progress")]
mod progress;
#[cfg(feature = "selection")]
mod radio;
mod ripple;
#[cfg(all(test, feature = "full"))]
mod ripple_tests;
mod roving;
#[cfg(feature = "inputs")]
mod search;
#[cfg(feature = "dialogs")]
mod sheets;
#[cfg(feature = "navigation")]
mod tabs;
mod text;
#[cfg(feature = "inputs")]
mod text_field;
#[cfg(feature = "selection")]
mod tile;
#[cfg(feature = "window-chrome")]
mod title_bar;
#[cfg(feature = "feedback")]
mod tooltip;

#[cfg(feature = "navigation")]
pub use app_bar::{
    BOTTOM_APP_BAR_HEIGHT, BottomAppBar, TOP_APP_BAR_HEIGHT, TopAppBar, TopAppBarVariant,
    collapsed_title_alpha,
};
#[cfg(feature = "containment")]
pub use avatar::{AVATAR_SIZE, Avatar, AvatarContent, monogram_initials};
#[cfg(feature = "buttons")]
pub use button_group::{ConnectedButtonGroup, GroupLabels, GroupSelection};
#[cfg(feature = "window-chrome")]
pub use caption::{Caption, CaptionButton, CaptionButtons, CaptionLabels, title_bar_control};
#[cfg(feature = "containment")]
pub use carousel::{
    CAROUSEL_ITEM_RADIUS, CAROUSEL_SMALL_ITEM, CarouselItem, CarouselLayout, Keyline, Keylines,
    MaterialCarousel,
};
#[cfg(feature = "selection")]
pub use checkbox::{CheckboxState, MaterialCheckbox};
#[cfg(feature = "selection")]
pub use chip_set::{ChipSelection, ChipSet, ChipSetItem};
#[cfg(feature = "selection")]
pub use chips::{AssistChip, CHIP_HEIGHT, ChipStyle, FilterChip, InputChip, SuggestionChip, chip};
#[cfg(feature = "selection")]
pub use choice::{ChoiceGroup, ChoiceTile, Tint, choice_radio, tinted_icon};
#[cfg(feature = "dialogs")]
pub use dialog::{
    DIALOG_WIDTH, DialogActionAlignment, DialogAnimation, DialogSize, MaterialDialog, SCRIM_OPACITY,
};
#[cfg(feature = "feedback")]
pub use feedback::{
    BADGE_CONTAINER, BADGE_LABEL, CARD_PADDING, CardState, CardSurface, CardVariant,
    LARGE_BADGE_PADDING, LARGE_BADGE_SIZE, MaterialCard, NoticeKind, SMALL_BADGE_SIZE, StatusKind,
    card, card_surface, large_badge, notice_banner, outlined_status_chip, small_badge, status_chip,
};
#[cfg(feature = "feedback")]
#[allow(deprecated)]
pub use feedback::{BadgeKind, badge, outlined_badge};
pub use focus::{FocusRing, RevealFocus, RingedButton, contain_scroll};
pub use labeled::{LabelPosition, required_label};
#[cfg(feature = "containment")]
pub use list::{
    LIST_SEGMENT_GAP, LIST_SEGMENT_INNER_RADIUS, LIST_SEGMENT_RADIUS, ListDensity, ListItem,
    ListLeading, ListLines, ListTrailing, list_subheader, segmented_list,
};
#[cfg(feature = "loading")]
pub use loading_indicator::{
    LOADING_INDICATOR_SIZE, LoadingFrame, LoadingIndicator, LoadingShape, loading_indicator_frame,
};
#[cfg(all(feature = "menus", feature = "inputs"))]
pub use menu::ExposedDropdown;
#[cfg(feature = "menus")]
pub use menu::{
    MaterialContextMenu, MaterialMenu, MaterialMenuItem, MenuDensity, MenuGroup, MenuPopup,
    MenuStyle,
};
#[cfg(feature = "navigation")]
pub use navigation::{
    MEDIUM_WIDTH, NavBadge, NavIcon, NavItem, NavItemLayout, NavigationBar, NavigationItem,
    NavigationRail, RAIL_COLLAPSED_WIDTH, RAIL_EXPANDED_WIDTH, RailAlignment, navigation_scrim,
};
#[cfg(feature = "navigation")]
pub use navigation_drawer::{DRAWER_WIDTH, DrawerPosition, NavigationDrawer};
#[cfg(feature = "adaptive")]
mod adaptive;
#[cfg(feature = "adaptive")]
pub use adaptive::{
    SizedContent, WindowHeightClass, WindowSizeClass, WindowWidthClass, sized_content,
};
#[cfg(feature = "adaptive")]
mod pane_logic;
#[cfg(feature = "adaptive")]
mod pane_scaffold;
#[cfg(feature = "adaptive")]
pub use pane_logic::{
    AdaptStrategies, AdaptStrategy, BackNavigationBehavior, DEFAULT_PANE_ANCHORS,
    LIST_DETAIL_ORDER, LIST_DETAIL_STRATEGIES, PaneAdaptedValue, PaneAnchor, PaneDestination,
    PaneDirective, PaneLayout, PaneLayoutInput, PaneMotion, PaneNavigator, PaneRect, PaneRole,
    PaneRule, PaneTransition, SUPPORTING_PANE_ORDER, SUPPORTING_PANE_STRATEGIES, ThreePaneValue,
    closest_pane_anchor, next_pane_anchor, pane_directive, pane_motions, pane_transitions,
    previous_pane_anchor, three_pane_layout, three_pane_value,
};
#[cfg(feature = "adaptive")]
pub use pane_scaffold::{
    FEED_COLUMN_MIN_WIDTH, FEED_GAP, FEED_MARGIN, FEED_MARGIN_COMPACT, FeedLayout, FeedMetrics,
    PaneScaffold, feed_metrics,
};
#[cfg(feature = "loading")]
mod pull_to_refresh;
#[cfg(feature = "loading")]
pub use pull_to_refresh::{
    PULL_TO_REFRESH_DRAG_MULTIPLIER, PULL_TO_REFRESH_INDICATOR_SIZE,
    PULL_TO_REFRESH_LOADING_INDICATOR_SIZE, PULL_TO_REFRESH_MAX_DISTANCE,
    PULL_TO_REFRESH_MIN_ALPHA, PULL_TO_REFRESH_SPINNER_SIZE, PULL_TO_REFRESH_THRESHOLD, PullArrow,
    PullModel, PullToRefresh, PullToRefreshIndicator, pull_arrow, pull_loading_rotation,
    pull_vertical_offset,
};
#[cfg(feature = "navigation")]
mod navigation_suite;
#[cfg(feature = "navigation")]
pub use navigation_suite::{
    NavigationSuite, NavigationSuiteType, SUITE_PRIMARY_ACTION_PADDING, navigation_suite_type,
};
pub use preview::InteractionPreview;
#[cfg(feature = "progress")]
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
#[cfg(feature = "selection")]
pub use radio::{MaterialRadio, MaterialRadioGroup};
pub use ripple::Ripple;
#[cfg(feature = "dialogs")]
pub use sheets::{BottomSheetDetent, MaterialBottomSheet, MaterialSideSheet, SideSheetEdge};
#[cfg(feature = "navigation")]
pub use tabs::{MaterialTabs, TabItem, TabsVariant};
pub use text::{
    accessible_text, error_text, info_callout, page_heading, section_heading, section_title,
    semantic_label, supporting_info,
};
#[cfg(feature = "inputs")]
pub use text_field::{
    TEXT_FIELD_HEIGHT, TEXT_FIELD_WIDTH, TextField, TextFieldDensity, TextFieldInputType,
    TextFieldVariant,
};
#[cfg(feature = "selection")]
pub use tile::{
    TileGroup, TileScale, disclosure_chevron, list_tile, tile, tile_chevron, tile_info,
};
#[cfg(feature = "window-chrome")]
pub use title_bar::{
    TitleBar, title_bar_leading, title_bar_room, title_bar_window_options, traffic_light_position,
};
#[cfg(feature = "navigation")]
pub use toolbar::{
    TOOLBAR_FAB_GAP, TOOLBAR_MARGIN, TOOLBAR_SIZE, Toolbar, ToolbarButton, ToolbarColor,
    ToolbarConfiguration, ToolbarOrientation,
};
#[cfg(feature = "feedback")]
pub use tooltip::{MaterialTooltip, PLAIN_TOOLTIP_MAX_WIDTH, RICH_TOOLTIP_WIDTH};

pub(crate) fn init(cx: &mut gpui::App) {
    roving::init(cx);
    #[cfg(feature = "menus")]
    menu::init(cx);
    #[cfg(feature = "pickers")]
    date_picker::init(cx);
}
