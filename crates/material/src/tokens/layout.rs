//! Control metrics and window breakpoints.
use super::typography::rem;
use gpui::{Pixels, Rems, Window, px};

/// Ordinary controls share this height, padding included, at every text scale.
pub const CONTROL_HEIGHT: Rems = rem(40.);
/// M3 medium and large button heights.
pub const BUTTON_MEDIUM: Rems = rem(48.);
pub const BUTTON_LARGE: Rems = rem(56.);
/// Horizontal inset of an item's content inside a tile or card.
pub const ITEM_INSET: Rems = rem(20.);
/// The leading icon slot of an icon + label cluster, and the gap after it.
pub const ICON_SLOT: Rems = rem(20.);
pub const ICON_GAP: Rems = rem(8.);
/// Supporting lines of an icon-labelled item start on its label text.
pub const LABEL_INSET: Rems = rem(20. + 8.);
/// Between a section title and its group's first tile.
pub const SECTION_TITLE_GAP: Pixels = px(10.);
/// A select at the trailing end of a row keeps a readable width.
pub const INLINE_SELECT_WIDTH: Rems = rem(280.);

/// The window width, in 100%-scale px, below which pages use narrow
/// gutters, compact navigation and stacked controls.
pub const COMPACT_WINDOW: f32 = 960.;

/// The window width in 100%-scale px, so breakpoints follow the text scale.
pub fn design_width(window: &Window) -> f32 {
    f32::from(window.bounds().size.width) * super::typography::BASE_PX
        / f32::from(window.rem_size())
}

pub fn is_compact(window: &Window) -> bool {
    design_width(window) < COMPACT_WINDOW
}

/// An app bar of one control row with 8px above and below.
pub fn app_bar_height(window: &Window) -> Pixels {
    CONTROL_HEIGHT.to_pixels(window.rem_size()) + px(16.)
}
