//! Neutral shadows. Tone carries most layering; shadows stay faint.
use gpui::{BoxShadow, Hsla, hsla, px};

fn shadow(alpha: f32) -> Hsla {
    hsla(0., 0., 0., alpha)
}

/// M3 level 1 (cards): 0/1/2 at 6%.
pub fn level1() -> Vec<BoxShadow> {
    vec![BoxShadow::new(px(0.), px(1.), shadow(0.06)).blur_radius(px(2.))]
}

/// Menus, popovers and dialogs: 0/12/32 at 16%.
pub fn popover() -> Vec<BoxShadow> {
    vec![BoxShadow::new(px(0.), px(12.), shadow(0.16)).blur_radius(px(32.))]
}
