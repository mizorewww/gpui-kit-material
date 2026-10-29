//! Kit default icon paths at bb85882e1faab845d9e7be33b69a152879a90f69.
//!
//! Source: `crates/assets/default-icons.txt`, which generates Component's
//! compatibility IconName catalog. This is a semantic translation, not a claim
//! of visual equivalence to Lucide. Symbols use the primary reference Rounded style and default axes unless
//! the source explicitly requests a filled star. Construct once during init.
use gpui_material_symbols::{Icon as SymbolIcon, Style, Symbol};

/// Brand artwork and crossed-out favorites have no semantic equivalent in this
/// pinned Symbols catalog. Leave them unresolved for caller-owned assets.
#[cfg(test)]
pub(super) const UNSUPPORTED_PATHS: &[&str] = &[
    "icons/github.svg",
    "icons/heart-off.svg",
    "icons/star-off.svg",
];

/// Approximate translations requiring visual review; these preserve the action
/// but do not claim identical silhouettes or motion to the Kit artwork.
#[cfg(test)]
pub(super) const APPROXIMATE_PATHS: &[&str] = &[
    "icons/frame.svg",
    "icons/gallery-vertical-end.svg",
    "icons/loader.svg",
    "icons/resize-corner.svg",
    "icons/sort-ascending.svg",
    "icons/sort-descending.svg",
];

// Keep paths literal: no formatting or catalog-name lookup during rendering.
const MAPPINGS: &[(&str, Symbol)] = &[
    ("icons/a-large-small.svg", Symbol::FORMAT_SIZE),
    ("icons/arrow-down.svg", Symbol::ARROW_DOWNWARD),
    ("icons/arrow-left.svg", Symbol::ARROW_BACK),
    ("icons/arrow-right.svg", Symbol::ARROW_FORWARD),
    ("icons/arrow-up.svg", Symbol::ARROW_UPWARD),
    ("icons/asterisk.svg", Symbol::ASTERISK),
    ("icons/ban.svg", Symbol::BLOCK),
    ("icons/battery-charging.svg", Symbol::BATTERY_CHARGING_FULL),
    ("icons/battery-full.svg", Symbol::BATTERY_FULL),
    ("icons/battery-low.svg", Symbol::BATTERY_1_BAR),
    ("icons/battery-medium.svg", Symbol::BATTERY_4_BAR),
    ("icons/battery-warning.svg", Symbol::BATTERY_ALERT),
    ("icons/battery.svg", Symbol::BATTERY_0_BAR),
    ("icons/bell.svg", Symbol::NOTIFICATIONS),
    ("icons/book-open.svg", Symbol::MENU_BOOK),
    ("icons/bot.svg", Symbol::SMART_TOY),
    ("icons/building-2.svg", Symbol::APARTMENT),
    ("icons/calendar.svg", Symbol::CALENDAR_MONTH),
    ("icons/case-sensitive.svg", Symbol::MATCH_CASE),
    ("icons/chart-pie.svg", Symbol::PIE_CHART),
    ("icons/check.svg", Symbol::CHECK),
    ("icons/chevron-down.svg", Symbol::KEYBOARD_ARROW_DOWN),
    ("icons/chevron-left.svg", Symbol::CHEVRON_LEFT),
    ("icons/chevron-right.svg", Symbol::CHEVRON_RIGHT),
    ("icons/chevron-up.svg", Symbol::KEYBOARD_ARROW_UP),
    ("icons/chevrons-up-down.svg", Symbol::UNFOLD_MORE),
    ("icons/circle-alert.svg", Symbol::ERROR),
    ("icons/circle-check.svg", Symbol::CHECK_CIRCLE),
    ("icons/circle-user.svg", Symbol::ACCOUNT_CIRCLE),
    ("icons/circle-x.svg", Symbol::CANCEL),
    ("icons/close.svg", Symbol::CLOSE),
    ("icons/copy.svg", Symbol::CONTENT_COPY),
    ("icons/cpu.svg", Symbol::DEVELOPER_BOARD),
    ("icons/dash.svg", Symbol::REMOVE),
    ("icons/delete.svg", Symbol::BACKSPACE),
    ("icons/ellipsis-vertical.svg", Symbol::MORE_VERT),
    ("icons/ellipsis.svg", Symbol::MORE_HORIZ),
    ("icons/external-link.svg", Symbol::OPEN_IN_NEW),
    ("icons/eye-off.svg", Symbol::VISIBILITY_OFF),
    ("icons/eye.svg", Symbol::VISIBILITY),
    ("icons/file-text.svg", Symbol::DESCRIPTION),
    ("icons/file.svg", Symbol::DRAFT),
    ("icons/folder-closed.svg", Symbol::FOLDER),
    ("icons/folder-open.svg", Symbol::FOLDER_OPEN),
    ("icons/folder.svg", Symbol::FOLDER),
    ("icons/frame.svg", Symbol::GRID_3X3),
    ("icons/gallery-vertical-end.svg", Symbol::VIEW_CAROUSEL),
    ("icons/globe.svg", Symbol::LANGUAGE),
    ("icons/hard-drive.svg", Symbol::HARD_DRIVE),
    ("icons/heart.svg", Symbol::FAVORITE),
    ("icons/inbox.svg", Symbol::INBOX),
    ("icons/info.svg", Symbol::INFO),
    ("icons/inspector.svg", Symbol::SELECT_WINDOW),
    ("icons/layout-dashboard.svg", Symbol::DASHBOARD),
    ("icons/loader-circle.svg", Symbol::PROGRESS_ACTIVITY),
    ("icons/loader.svg", Symbol::PROGRESS_ACTIVITY),
    ("icons/map.svg", Symbol::MAP),
    ("icons/maximize.svg", Symbol::OPEN_IN_FULL),
    ("icons/memory-stick.svg", Symbol::MEMORY_ALT),
    ("icons/menu.svg", Symbol::MENU),
    ("icons/mic.svg", Symbol::MIC),
    ("icons/minimize.svg", Symbol::CLOSE_FULLSCREEN),
    ("icons/minus.svg", Symbol::REMOVE),
    ("icons/moon.svg", Symbol::DARK_MODE),
    ("icons/network.svg", Symbol::LAN),
    ("icons/palette.svg", Symbol::PALETTE),
    ("icons/panel-bottom-open.svg", Symbol::BOTTOM_PANEL_OPEN),
    ("icons/panel-bottom.svg", Symbol::DOCK_TO_BOTTOM),
    ("icons/panel-left-close.svg", Symbol::LEFT_PANEL_CLOSE),
    ("icons/panel-left-open.svg", Symbol::LEFT_PANEL_OPEN),
    ("icons/panel-left.svg", Symbol::DOCK_TO_LEFT),
    ("icons/panel-right-close.svg", Symbol::RIGHT_PANEL_CLOSE),
    ("icons/panel-right-open.svg", Symbol::RIGHT_PANEL_OPEN),
    ("icons/panel-right.svg", Symbol::DOCK_TO_RIGHT),
    ("icons/pause.svg", Symbol::PAUSE),
    ("icons/play.svg", Symbol::PLAY_ARROW),
    ("icons/plus.svg", Symbol::ADD),
    ("icons/redo-2.svg", Symbol::REDO),
    ("icons/redo.svg", Symbol::REDO),
    ("icons/refresh-cw.svg", Symbol::REFRESH),
    ("icons/replace.svg", Symbol::FIND_REPLACE),
    ("icons/resize-corner.svg", Symbol::RESIZE),
    ("icons/rotate-cw.svg", Symbol::ROTATE_RIGHT),
    ("icons/search.svg", Symbol::SEARCH),
    ("icons/settings-2.svg", Symbol::TUNE),
    ("icons/settings.svg", Symbol::SETTINGS),
    ("icons/sort-ascending.svg", Symbol::KEYBOARD_ARROW_UP),
    ("icons/sort-descending.svg", Symbol::KEYBOARD_ARROW_DOWN),
    ("icons/square-terminal.svg", Symbol::TERMINAL),
    ("icons/square.svg", Symbol::CROP_SQUARE),
    ("icons/star-fill.svg", Symbol::STAR),
    ("icons/star.svg", Symbol::STAR),
    ("icons/sun.svg", Symbol::LIGHT_MODE),
    ("icons/thumbs-down.svg", Symbol::THUMB_DOWN),
    ("icons/thumbs-up.svg", Symbol::THUMB_UP),
    ("icons/triangle-alert.svg", Symbol::WARNING),
    ("icons/undo-2.svg", Symbol::UNDO),
    ("icons/undo.svg", Symbol::UNDO),
    ("icons/user.svg", Symbol::PERSON),
    ("icons/window-close.svg", Symbol::CLOSE),
    ("icons/window-maximize.svg", Symbol::CROP_SQUARE),
    ("icons/window-minimize.svg", Symbol::REMOVE),
    ("icons/window-restore.svg", Symbol::FILTER_NONE),
];

pub(super) fn mappings() -> impl Iterator<Item = (&'static str, SymbolIcon)> {
    MAPPINGS.iter().map(|&(path, symbol)| {
        (
            path,
            symbol
                .icon()
                .style(Style::Rounded)
                .filled(path == "icons/star-fill.svg"),
        )
    })
}
