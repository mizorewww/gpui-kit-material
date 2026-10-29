//! M3 state layers and disabled emphasis, resolved over the container so a
//! selected control keeps its color under hover and press.
use crate::theme::blend;
use gpui::Rgba;

pub const HOVER: f32 = 0.08;
pub const FOCUS: f32 = 0.10;
pub const PRESS: f32 = 0.10;
pub const DRAG: f32 = 0.16;
/// Disabled containers and content: on-surface at 12% and 38%.
pub const DISABLED_CONTAINER: f32 = 0.12;
pub const DISABLED_CONTENT: f32 = 0.38;

/// `container` under a hover layer of its `content` color.
pub fn hovered(container: Rgba, content: Rgba) -> Rgba {
    blend(container, content, HOVER)
}

/// `container` under a press layer of its `content` color.
pub fn pressed(container: Rgba, content: Rgba) -> Rgba {
    blend(container, content, PRESS)
}

/// `color` at disabled content emphasis.
pub fn disabled(color: Rgba) -> Rgba {
    Rgba {
        a: DISABLED_CONTENT,
        ..color
    }
}
