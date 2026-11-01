//! A progress stepper: numbered steps joined by hairlines, for a flow a person walks through
//! once. Material 3 specifies no stepper; this keeps to M3 roles and type, and to the form
//! Material 2's horizontal stepper gave it. Strings come from the caller.
use crate::{
    color::ColorRole,
    icons::IconName,
    theme::color,
    tokens::typography::{TypeRole, rem},
};
use gpui::{prelude::*, *};

/// The numbered circle's diameter in dp.
pub const STEPPER_DOT_SIZE: f32 = 24.;
const GAP: f32 = 8.;

/// Where a step stands against the current one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StepState {
    Finished,
    Current,
    Upcoming,
}

fn step_state(index: usize, current: usize) -> StepState {
    match index.cmp(&current) {
        std::cmp::Ordering::Less => StepState::Finished,
        std::cmp::Ordering::Equal => StepState::Current,
        std::cmp::Ordering::Greater => StepState::Upcoming,
    }
}

/// The steps of a flow, with `current` (0-based) on the primary color, finished steps on the
/// secondary container with a check and later ones as an outlined number. The labels keep
/// their width and the hairlines take what is left.
#[derive(gpui::IntoElement)]
pub struct Stepper {
    id: ElementId,
    label: SharedString,
    steps: Vec<SharedString>,
    current: usize,
}

impl Stepper {
    /// `label` names the whole progress for assistive technology.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            steps: Vec::new(),
            current: 0,
        }
    }
    pub fn steps<S: Into<SharedString>>(mut self, steps: impl IntoIterator<Item = S>) -> Self {
        self.steps = steps.into_iter().map(Into::into).collect();
        self
    }
    /// The current step, 0-based.
    pub fn current(mut self, current: usize) -> Self {
        self.current = current;
        self
    }
}

impl RenderOnce for Stepper {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        use ColorRole as C;
        let count = self.steps.len();
        let current = self.current;
        let number_style = TypeRole::LabelMedium.scale();
        let label_style = TypeRole::LabelLarge.scale();
        div()
            .id(self.id)
            .role(gpui::Role::Group)
            .aria_label(self.label)
            .flex()
            .w_full()
            .min_w_0()
            .items_center()
            .gap(rem(GAP))
            .children(self.steps.into_iter().enumerate().map(|(index, step)| {
                let state = step_state(index, current);
                let last = index + 1 == count;
                div()
                    .flex()
                    .min_w_0()
                    .items_center()
                    .gap(rem(GAP))
                    .when(!last, |item| item.flex_1())
                    .child(
                        div()
                            .flex()
                            .flex_shrink_0()
                            .items_center()
                            .justify_center()
                            .size(rem(STEPPER_DOT_SIZE))
                            .rounded_full()
                            .font_family(number_style.font_family)
                            .text_size(number_style.font_size)
                            .font_weight(FontWeight(number_style.font_weight as f32))
                            .map(|dot| match state {
                                StepState::Current => {
                                    dot.bg(color(C::PRIMARY)).text_color(color(C::ON_PRIMARY))
                                }
                                StepState::Finished => dot
                                    .bg(color(C::SECONDARY_CONTAINER))
                                    .text_color(color(C::ON_SECONDARY_CONTAINER)),
                                StepState::Upcoming => dot
                                    .border(px(1.))
                                    .border_color(color(C::OUTLINE))
                                    .text_color(color(C::ON_SURFACE_VARIANT)),
                            })
                            .map(|dot| match state {
                                StepState::Finished => {
                                    dot.child(IconName::Check.icon().size(rem(16.)))
                                }
                                _ => dot.child(SharedString::from((index + 1).to_string())),
                            }),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .font_family(label_style.font_family)
                            .text_size(label_style.font_size)
                            .line_height(label_style.line_height)
                            .font_weight(FontWeight(label_style.font_weight as f32))
                            .text_color(color(if state == StepState::Current {
                                C::ON_SURFACE
                            } else {
                                C::ON_SURFACE_VARIANT
                            }))
                            .child(step),
                    )
                    .when(!last, |item| {
                        item.child(
                            div()
                                .flex_1()
                                .min_w(rem(GAP))
                                .h(px(1.))
                                .bg(color(C::OUTLINE_VARIANT)),
                        )
                    })
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn steps_are_finished_current_or_upcoming_against_the_current_one() {
        assert_eq!(step_state(0, 1), StepState::Finished);
        assert_eq!(step_state(1, 1), StepState::Current);
        assert_eq!(step_state(2, 1), StepState::Upcoming);
    }
}
