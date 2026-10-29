//! Corner radii and the connected-group corner rule.
use gpui::{Pixels, Styled, px};

// The M3 / M3 Expressive corner scale. Corners are dp-sized geometry and do
// not follow the text scale.

/// M3 corner none (0).
pub const RADIUS_NONE: Pixels = px(0.);
/// M3 corner extra-small (4): snackbars, text fields, small flat surfaces.
pub const RADIUS_EXTRA_SMALL: Pixels = px(4.);
/// M3 corner small (8): chips, badges, small covers and notices.
pub const RADIUS_SMALL: Pixels = px(8.);
/// M3 corner medium (12): cards.
pub const RADIUS_MEDIUM: Pixels = px(12.);
/// M3 corner large (16).
pub const RADIUS_LARGE: Pixels = px(16.);
/// M3 Expressive corner large-increased (20).
pub const RADIUS_LARGE_INCREASED: Pixels = px(20.);
/// M3 corner extra-large (28): dialogs, large hero surfaces.
pub const RADIUS_EXTRA_LARGE: Pixels = px(28.);
/// M3 Expressive corner extra-large-increased (32).
pub const RADIUS_EXTRA_LARGE_INCREASED: Pixels = px(32.);
/// M3 Expressive corner extra-extra-large (48).
pub const RADIUS_EXTRA_EXTRA_LARGE: Pixels = px(48.);
/// M3 corner full: large enough to round any control into a pill.
pub const RADIUS_FULL: Pixels = px(999.);

/// Alias of [`RADIUS_FULL`].
pub const RADIUS_PILL: Pixels = RADIUS_FULL;
/// Filled and outlined text fields: M3 extra-small (top corners for filled).
pub const FIELD_RADIUS: Pixels = RADIUS_EXTRA_SMALL;
/// Cards: M3 medium.
pub const RADIUS_CARD: Pixels = RADIUS_MEDIUM;
/// Menus and the fields that open them: M3 large.
pub const MENU_RADIUS: Pixels = RADIUS_LARGE;
/// Large standalone cards and hero surfaces: M3 extra-large.
pub const RADIUS_HERO: Pixels = RADIUS_EXTRA_LARGE;

/// Connected tiles keep 24px corners on their group's outside and 6px
/// where tiles meet, with 2px of ground between them.
pub const TILE_RADIUS: Pixels = px(24.);
pub const TILE_INNER_RADIUS: Pixels = px(6.);
pub const TILE_GAP: Pixels = px(2.);

/// Which corners of a connected item are on its group's outside, as
/// top-left, top-right, bottom-right, bottom-left.
pub type OuterCorners = [bool; 4];

/// Round `element` as a connected tile: outside corners at
/// [`TILE_RADIUS`], inner ones at [`TILE_INNER_RADIUS`].
pub fn tile_corners<E: Styled>(element: E, outer: OuterCorners) -> E {
    let radius = |outside: bool| {
        if outside {
            TILE_RADIUS
        } else {
            TILE_INNER_RADIUS
        }
    };
    element
        .rounded_tl(radius(outer[0]))
        .rounded_tr(radius(outer[1]))
        .rounded_br(radius(outer[2]))
        .rounded_bl(radius(outer[3]))
}

/// The outside corners of item `index` in a vertical group of `count`.
pub fn list_corners(index: usize, count: usize) -> OuterCorners {
    let first = index == 0;
    let last = index + 1 >= count;
    [first, first, last, last]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertical_groups_round_only_their_ends() {
        assert_eq!(list_corners(0, 1), [true; 4]);
        assert_eq!(list_corners(0, 3), [true, true, false, false]);
        assert_eq!(list_corners(1, 3), [false; 4]);
        assert_eq!(list_corners(2, 3), [false, false, true, true]);
    }

    #[test]
    fn corner_scale_matches_m3() {
        let scale = [
            (RADIUS_NONE, 0.),
            (RADIUS_EXTRA_SMALL, 4.),
            (RADIUS_SMALL, 8.),
            (RADIUS_MEDIUM, 12.),
            (RADIUS_LARGE, 16.),
            (RADIUS_LARGE_INCREASED, 20.),
            (RADIUS_EXTRA_LARGE, 28.),
            (RADIUS_EXTRA_LARGE_INCREASED, 32.),
            (RADIUS_EXTRA_EXTRA_LARGE, 48.),
        ];
        for (radius, expected) in scale {
            assert_eq!(radius, px(expected));
        }
        assert_eq!(FIELD_RADIUS, px(4.), "M3 text fields use extra-small");
        assert_eq!(RADIUS_CARD, px(12.), "M3 cards use medium");
    }
}
