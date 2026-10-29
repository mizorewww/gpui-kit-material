//! Type scale in rems. 14px is 1rem at the 100% text scale.
use gpui::{Rems, rems};

/// The px size of 1rem at the 100% text scale.
pub const BASE_PX: f32 = 14.;

/// Express a 100%-scale px size in rems.
pub const fn rem(px: f32) -> Rems {
    rems(px / BASE_PX)
}

/// Supporting and auxiliary text (12px).
pub const TEXT_AUX: Rems = rems(0.857);
/// Body and control labels (14px).
pub const TEXT_BODY: Rems = rems(1.);
/// Long-form reading (16px).
pub const TEXT_READER: Rems = rems(1.143);
/// Card and dialog titles (18px).
pub const TEXT_TITLE: Rems = rems(1.286);
/// Section headings on a page (22px).
pub const TEXT_SECTION: Rems = rem(22.);
/// Page titles: M3 headline small at 30px.
pub const TEXT_HEADLINE: Rems = rem(30.);
pub const HEADLINE_LINE_HEIGHT: Rems = rem(36.);
/// Settings content reads one step larger than navigation.
pub const TEXT_SETTINGS_BODY: Rems = rem(16.);
pub const TEXT_SETTINGS_AUX: Rems = TEXT_BODY;
/// The main label of a settings tile (17px).
pub const TEXT_TILE_LABEL: Rems = rem(17.);
