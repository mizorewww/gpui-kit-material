//! Noninteractive separators from the pinned md3.io divider factory.
use crate::{
    color::ColorRole,
    theme::color,
    tokens::typography::{TypeRole, rem},
};
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
/// The side margin and the gap above the text of a divider with a subhead (the kit's
/// `Horizontal/Divider with subhead`).
const SUBHEAD_MARGIN: f32 = 16.;
const SUBHEAD_GAP: f32 = 4.;

/// Insets use logical start/end. Inset variants leave their main size automatic:
/// a vertical divider needs a stretching row or an explicit height from its
/// caller. Insets are retained when switching back from `FullWidth`.
/// Uses GPUI block/flex layout; browser inline formatting with text siblings
/// is not implemented.
/// Default thickness and spacing are fixed CSS pixel geometry, independent of
/// typography. Callers can supply rem-valued insets when aligning with text.
///
/// # Example
///
/// ```
/// use gpui_kit_material::prelude::*;
///
/// let between = MaterialDivider::new("between");
/// let inset = MaterialDivider::new("inset")
///     .variant(DividerVariant::Inset)
///     .inset_start(rem(72.));
/// let headed = MaterialDivider::new("recent").subhead("Recent");
/// let beside = MaterialDivider::new("beside").orientation(DividerOrientation::Vertical);
/// # let _ = (between, inset, headed, beside);
/// ```
#[derive(gpui::IntoElement)]
pub struct MaterialDivider {
    id: ElementId,
    orientation: DividerOrientation,
    variant: DividerVariant,
    context: DividerContext,
    thickness: Pixels,
    inset_start: Option<Length>,
    inset_end: Option<Length>,
    color_role: ColorRole,
    subhead: Option<SharedString>,
    style: StyleRefinement,
}
impl MaterialDivider {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            orientation: DividerOrientation::Horizontal,
            variant: DividerVariant::FullWidth,
            context: DividerContext::Plain,
            thickness: px(1.),
            inset_start: None,
            inset_end: None,
            color_role: ColorRole::OUTLINE_VARIANT,
            subhead: None,
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
    /// The kit's `Divider with subhead`: a horizontal line 16dp in from each side, and under
    /// it this title-small text, which heads what follows. The line's own variant and insets
    /// give way to that margin, and a vertical divider has none.
    pub fn subhead(mut self, subhead: impl Into<SharedString>) -> Self {
        self.subhead = Some(subhead.into());
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
                line.w_auto().ml(start).mr(end)
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
    fn render(mut self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let subhead = self
            .subhead
            .take()
            .filter(|_| self.orientation == DividerOrientation::Horizontal);
        let Some(subhead) = subhead else {
            return self.into_line().into_any_element();
        };
        self.variant = DividerVariant::FullWidth;
        self.inset_start = None;
        self.inset_end = None;
        let scale = TypeRole::TitleSmall.scale();
        let text_id = ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "subhead".into());
        div()
            .debug_selector(|| "divider-with-subhead".into())
            .flex()
            .flex_col()
            .w_full()
            .px(rem(SUBHEAD_MARGIN))
            .gap(rem(SUBHEAD_GAP))
            .child(self.into_line())
            .child(
                super::text::accessible_text(text_id, subhead)
                    .font_family(scale.font_family)
                    .text_size(scale.font_size)
                    .line_height(scale.line_height)
                    .font_weight(FontWeight(scale.font_weight as f32))
                    .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
            )
            .into_any_element()
    }
}

#[cfg(test)]
#[path = "divider_tests.rs"]
mod tests;
