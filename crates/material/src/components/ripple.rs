//! The Material ripple as a component: press feedback for anything interactive.
//!
//! This is the one implementation. Buttons, navigation, tabs, cards and toolbars
//! use the element underneath directly; every other control wraps its surface
//! here. It observes pointer and touch input without consuming it, so activation
//! stays with Base and scrolling stays with the scroll container. A touch's wave
//! ends with the tap, with the scroll a pan becomes, or after a hold limit where the
//! system cancels a still finger without saying so.
//!
//! Timing comes from the reference's captured ripple (`button/ripple.rs`), and
//! reduced motion follows it there.
use super::button::ButtonRipple;
use crate::{color::ColorRole, theme::color};
use gpui::{prelude::*, *};

/// A wave over `child`, clipped to `child`'s bounds and these corners.
///
/// Wrap the surface the wave should fill. The ripple adds no layout of its own.
#[derive(gpui::IntoElement)]
pub struct Ripple {
    id: ElementId,
    child: AnyElement,
    ink: Option<Rgba>,
    corners: Corners<Pixels>,
    disabled: bool,
    specimen: bool,
    centered: bool,
    extension: Edges<Rems>,
    touch_pressed: Option<Entity<bool>>,
}

impl Ripple {
    pub fn new(id: impl Into<ElementId>, child: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            child: child.into_any_element(),
            ink: None,
            corners: Corners::default(),
            disabled: false,
            specimen: false,
            centered: false,
            extension: Edges::all(rems(0.)),
            touch_pressed: None,
        }
    }
    /// The wave's color: the content color the surface's state layers use
    /// (default on-surface). It paints at the reference's 10%.
    pub fn ink(mut self, ink: Rgba) -> Self {
        self.ink = Some(ink);
        self
    }
    /// The clip's corner radii.
    pub fn corners(mut self, corners: Corners<Pixels>) -> Self {
        self.corners = corners;
        self
    }
    /// A round surface, such as a selection control's state layer.
    pub fn round(self) -> Self {
        self.corners(Corners::all(px(9999.)))
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// A static specimen starts no waves and drops any it had.
    pub fn specimen(mut self, specimen: bool) -> Self {
        self.specimen = specimen;
        self
    }
    /// Start every wave at the center, as selection controls do.
    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }
    /// Answer presses this far outside the surface too, for a control whose
    /// target is larger than its state layer.
    pub fn hit_extension(mut self, extension: Edges<Rems>) -> Self {
        self.extension = extension;
        self
    }
    /// Mirror a held touch into `pressed`, for feedback beyond the wave.
    pub fn touch_pressed(mut self, pressed: &Entity<bool>) -> Self {
        self.touch_pressed = Some(pressed.clone());
        self
    }
}

impl RenderOnce for Ripple {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let corners = Corners {
            top_left: self.corners.top_left.into(),
            top_right: self.corners.top_right.into(),
            bottom_right: self.corners.bottom_right.into(),
            bottom_left: self.corners.bottom_left.into(),
        };
        let mut ripple = ButtonRipple::new(
            self.child,
            self.id,
            !self.specimen,
            !self.disabled,
            corners,
            Edges::all(px(0.).into()),
            self.ink.unwrap_or_else(|| color(ColorRole::ON_SURFACE)),
            window,
            cx,
        )
        .hit_extension(self.extension);
        if self.centered {
            ripple = ripple.centered();
        }
        if let Some(pressed) = self.touch_pressed {
            ripple = ripple.touch_pressed(pressed, cx);
        }
        ripple
    }
}
