//! Corner radii and the connected-group corner rule.
use gpui::{Pixels, Styled, px};

pub const RADIUS_PILL: Pixels = px(999.);
/// Chips, badges, small covers and notices.
pub const RADIUS_SMALL: Pixels = px(8.);
/// Text fields and select fields.
pub const FIELD_RADIUS: Pixels = px(12.);
pub const RADIUS_CARD: Pixels = px(16.);
/// Menus and the fields that open them.
pub const MENU_RADIUS: Pixels = px(16.);
/// Large standalone cards and hero surfaces.
pub const RADIUS_HERO: Pixels = px(28.);

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
    use super::list_corners;

    #[test]
    fn vertical_groups_round_only_their_ends() {
        assert_eq!(list_corners(0, 1), [true; 4]);
        assert_eq!(list_corners(0, 3), [true, true, false, false]);
        assert_eq!(list_corners(1, 3), [false; 4]);
        assert_eq!(list_corners(2, 3), [false, false, true, true]);
    }
}
