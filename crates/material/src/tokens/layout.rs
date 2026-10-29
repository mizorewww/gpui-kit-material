//! Control metrics and window breakpoints.
use super::typography::rem;
use gpui::{Pixels, Rems, Window, px};

/// Ordinary controls share this height, padding included, at every text scale.
pub const CONTROL_HEIGHT: Rems = rem(40.);
/// Horizontal inset of an item's content inside a tile or card.
pub const ITEM_INSET: Rems = rem(20.);
/// The leading icon slot of an icon + label cluster, and the gap after it.
pub const ICON_SLOT: Rems = rem(20.);
pub const ICON_GAP: Rems = rem(8.);
/// Supporting lines of an icon-labelled item start on its label text.
pub const LABEL_INSET: Rems = rem(20. + 8.);
/// Between a section title and its group's first tile. The gap follows the
/// title's text, so it is in rems; prefer this over [`SECTION_TITLE_GAP`].
pub const SECTION_TITLE_SPACE: Rems = rem(10.);
/// Legacy px form of [`SECTION_TITLE_SPACE`] that does not follow the text
/// scale; kept until its call sites move to the rem token.
pub const SECTION_TITLE_GAP: Pixels = px(10.);
/// Above and below the control row of an app bar.
pub const APP_BAR_BLOCK_PADDING: Rems = rem(8.);
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

/// An app bar of one control row with [`APP_BAR_BLOCK_PADDING`] above and
/// below, all following the text scale.
pub fn app_bar_height(window: &Window) -> Pixels {
    app_bar_height_at(window.rem_size())
}

fn app_bar_height_at(rem_size: Pixels) -> Pixels {
    (CONTROL_HEIGHT + APP_BAR_BLOCK_PADDING * 2.).to_pixels(rem_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_bar_height_follows_the_text_scale() {
        assert_eq!(app_bar_height_at(px(14.)), px(56.));
        assert_eq!(app_bar_height_at(px(21.)), px(84.));
    }

    #[test]
    fn section_title_space_matches_the_legacy_gap_at_100_percent() {
        assert_eq!(SECTION_TITLE_SPACE.to_pixels(px(14.)), SECTION_TITLE_GAP);
    }
}
