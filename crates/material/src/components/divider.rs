//! Noninteractive separators from the pinned md3.io divider factory.
use crate::{color::ColorRole, theme::color};
use gpui::{prelude::*, *};
use gpui_base::StyledExt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DividerOrientation {
    #[default]
    Horizontal,
    Vertical,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DividerVariant {
    #[default]
    FullWidth,
    Inset,
    MiddleInset,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DividerDirection {
    #[default]
    Ltr,
    Rtl,
}
/// Explicit equivalent of the reference's ancestor list/card margin rule.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DividerContext {
    #[default]
    Plain,
    List,
    Card,
}

/// A separator without focus, activation, or intrinsic animation.
///
/// Insets use logical start/end. Inset variants leave their main size automatic:
/// a vertical divider needs a stretching row or an explicit height from its
/// caller. Insets are retained when switching back from `FullWidth`.
/// Uses GPUI block/flex layout; browser inline formatting with text siblings
/// is not implemented.
/// Default thickness and spacing are fixed CSS pixel geometry, independent of
/// typography. Callers can supply rem-valued insets when aligning with text.
#[derive(IntoElement)]
pub struct MaterialDivider {
    id: ElementId,
    orientation: DividerOrientation,
    variant: DividerVariant,
    direction: DividerDirection,
    context: DividerContext,
    thickness: Pixels,
    inset_start: Option<Length>,
    inset_end: Option<Length>,
    color_role: ColorRole,
    style: StyleRefinement,
}
impl MaterialDivider {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            orientation: DividerOrientation::Horizontal,
            variant: DividerVariant::FullWidth,
            direction: DividerDirection::Ltr,
            context: DividerContext::Plain,
            thickness: px(1.),
            inset_start: None,
            inset_end: None,
            color_role: ColorRole::OUTLINE_VARIANT,
            style: StyleRefinement::default(),
        }
    }
    pub fn orientation(mut self, orientation: DividerOrientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn variant(mut self, variant: DividerVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Resolves logical horizontal insets; the parent owns its own flow direction.
    pub fn direction(mut self, direction: DividerDirection) -> Self {
        self.direction = direction;
        self
    }
    pub fn context(mut self, context: DividerContext) -> Self {
        self.context = context;
        self
    }
    /// Zero requests the reference default. Nonfinite/negative values also use 1px.
    pub fn thickness(mut self, thickness: Pixels) -> Self {
        self.thickness = if f32::from(thickness).is_finite() && thickness > px(0.) {
            thickness
        } else {
            px(1.)
        };
        self
    }
    pub fn inset_start(mut self, inset: impl Into<Length>) -> Self {
        self.inset_start = Some(inset.into());
        self
    }
    pub fn inset_end(mut self, inset: impl Into<Length>) -> Self {
        self.inset_end = Some(inset.into());
        self
    }
    pub fn color_role(mut self, role: ColorRole) -> Self {
        self.color_role = role;
        self
    }

    fn styled_line(&self) -> Div {
        let horizontal = self.orientation == DividerOrientation::Horizontal;
        let mut line = div()
            .block()
            .m_0()
            .p_0()
            .border_0()
            .bg(color(self.color_role));
        if self.context != DividerContext::Plain {
            line = line.my(px(8.));
        }
        line = if horizontal {
            line.w_full().h(self.thickness)
        } else {
            line.h_full().w(self.thickness)
        };
        if self.variant != DividerVariant::FullWidth {
            let start = self.inset_start.unwrap_or(px(16.).into());
            let end = self.inset_end.unwrap_or(
                px(if self.variant == DividerVariant::MiddleInset {
                    16.
                } else {
                    0.
                })
                .into(),
            );
            line = if horizontal {
                let (left, right) = if self.direction == DividerDirection::Ltr {
                    (start, end)
                } else {
                    (end, start)
                };
                line.w_auto().ml(left).mr(right)
            } else {
                line.h_auto().mt(start).mb(end)
            };
        }
        line.refine_style(&self.style)
    }
}
impl MaterialDivider {
    fn into_line(self) -> Stateful<Div> {
        self.styled_line()
            .id(self.id)
            .role(Role::Splitter)
            .when(self.orientation == DividerOrientation::Vertical, |line| {
                line.aria_orientation(accesskit::Orientation::Vertical)
            })
    }
}
impl Styled for MaterialDivider {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for MaterialDivider {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.into_line()
    }
}

#[cfg(test)]
#[path = "divider_tests.rs"]
mod tests;
