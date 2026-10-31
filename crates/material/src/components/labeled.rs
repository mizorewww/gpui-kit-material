//! The visible label shared by selection controls.
//!
//! md3.io's checkbox and switch take a visible label, optional supporting
//! text, a label position and a required mark. The control keeps its own
//! accessible name; this row adds the text beside it and makes it a second
//! activation target, as an HTML `<label>` is.
use super::text_field::type_style;
use crate::{
    color::ColorRole,
    theme::color,
    tokens::{
        state,
        typography::{TypeRole, rem},
    },
};
use gpui::{prelude::*, *};
use std::rc::Rc;

/// Which side of a selection control its visible label sits on. Both follow
/// the layout direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LabelPosition {
    Start,
    #[default]
    End,
}

/// `label` with the required mark md3.io and the Figma kit append.
pub fn required_label(label: &SharedString, required: bool) -> SharedString {
    if required && !label.is_empty() {
        format!("{label}*").into()
    } else {
        label.clone()
    }
}

/// What pressing a visible label does.
pub(crate) type Activate = Rc<dyn Fn(&mut Window, &mut App)>;

/// The text beside a selection control.
pub(crate) struct ControlLabel {
    pub id: ElementId,
    pub label: SharedString,
    pub supporting: Option<SharedString>,
    pub position: LabelPosition,
    pub error: bool,
    pub disabled: bool,
    /// Stretch across the available width with the text filling the rest,
    /// as a switch row does; otherwise the row hugs its content.
    pub fill: bool,
    /// Space between the control and the text, in design px.
    pub gap: f32,
    pub activate: Option<Activate>,
}

impl ControlLabel {
    pub fn row(self, control: impl IntoElement) -> Stateful<Div> {
        let ink = |role: ColorRole| {
            let ink = color(role);
            if self.disabled {
                state::disabled(ink)
            } else {
                ink
            }
        };
        let (text_role, supporting_role) = if self.error {
            (ColorRole::ERROR, ColorRole::ERROR)
        } else {
            (ColorRole::ON_SURFACE, ColorRole::ON_SURFACE_VARIANT)
        };
        let activate = self.activate.filter(|_| !self.disabled);
        let mut text = div()
            .id((self.id.clone(), "label-text"))
            .when(cfg!(test), |text| {
                text.debug_selector(|| "control-label".into())
            })
            .flex()
            .flex_col()
            .min_w_0()
            .when(self.fill, |text| text.flex_1())
            .child(
                type_style(div(), TypeRole::BodyLarge)
                    .text_color(ink(text_role))
                    .child(self.label),
            )
            .children(self.supporting.map(|supporting| {
                type_style(
                    super::text::accessible_text((self.id.clone(), "supporting"), supporting),
                    TypeRole::BodyMedium,
                )
                .text_color(ink(supporting_role))
            }));
        if let Some(activate) = activate {
            text = text
                .cursor_pointer()
                .on_click(move |_, window, cx| activate(window, cx));
        }
        div()
            .id((self.id, "labeled"))
            .flex()
            .items_center()
            .gap(rem(self.gap))
            .when(self.fill, |row| row.w_full())
            .when(self.position == LabelPosition::Start, |row| {
                row.flex_row_reverse().justify_between()
            })
            .child(div().flex().flex_shrink_0().child(control))
            .child(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_required_mark_follows_a_non_empty_label() {
        let label = SharedString::new_static("Name");
        assert_eq!(required_label(&label, false), "Name");
        assert_eq!(required_label(&label, true), "Name*");
        assert_eq!(required_label(&SharedString::default(), true), "");
    }
}
