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
/// secondary container with a check and later ones as an outlined number. Each step takes
/// its label's width and the hairlines take what is left; where the labels do not fit, they
/// wrap rather than overlap.
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
        let dot_style = |dot: Div, state: StepState| match state {
            StepState::Current => dot.bg(color(C::PRIMARY)).text_color(color(C::ON_PRIMARY)),
            StepState::Finished => dot
                .bg(color(C::SECONDARY_CONTAINER))
                .text_color(color(C::ON_SECONDARY_CONTAINER)),
            StepState::Upcoming => dot
                .border(px(1.))
                .border_color(color(C::OUTLINE))
                .text_color(color(C::ON_SURFACE_VARIANT)),
        };
        let mut children = Vec::with_capacity(count * 2);
        for (index, step) in self.steps.into_iter().enumerate() {
            let state = step_state(index, current);
            children.push(
                div()
                    .flex()
                    .min_w_0()
                    .items_center()
                    .gap(rem(GAP))
                    .child(
                        dot_style(
                            div()
                                .flex()
                                .flex_shrink_0()
                                .items_center()
                                .justify_center()
                                .size(rem(STEPPER_DOT_SIZE))
                                .rounded_full()
                                .font_family(number_style.font_family)
                                .text_size(number_style.font_size)
                                .font_weight(FontWeight(number_style.font_weight as f32)),
                            state,
                        )
                        .map(|dot| match state {
                            StepState::Finished => dot.child(IconName::Check.icon().size(rem(16.))),
                            _ => dot.child(SharedString::from((index + 1).to_string())),
                        }),
                    )
                    .child(
                        div()
                            .debug_selector(|| format!("stepper-label-{index}"))
                            .min_w_0()
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
                    .into_any_element(),
            );
            if index + 1 < count {
                children.push(
                    div()
                        .flex_1()
                        .min_w(rem(GAP))
                        .h(px(1.))
                        .bg(color(C::OUTLINE_VARIANT))
                        .into_any_element(),
                );
            }
        }
        div()
            .id(self.id)
            .role(gpui::Role::Group)
            .aria_label(self.label)
            .flex()
            .w_full()
            .min_w_0()
            .items_center()
            .gap(rem(GAP))
            .children(children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::TestAppContext;

    #[test]
    fn steps_are_finished_current_or_upcoming_against_the_current_one() {
        assert_eq!(step_state(0, 1), StepState::Finished);
        assert_eq!(step_state(1, 1), StepState::Current);
        assert_eq!(step_state(2, 1), StepState::Upcoming);
    }

    struct Narrow {
        width: f32,
    }

    impl Render for Narrow {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(16.));
            div().w(px(self.width)).child(
                Stepper::new("steps", "Progress")
                    .steps([
                        "Choose the source of your documents",
                        "Connect the service to convert with",
                        "Sign in to your account",
                    ])
                    .current(1),
            )
        }
    }

    /// The labels of the three steps, drawn `width` wide.
    fn labels(cx: &mut TestAppContext, width: f32) -> Vec<Bounds<Pixels>> {
        cx.update(|cx| {
            gpui_kit::init(cx);
            crate::init(cx);
        });
        let (_, cx) = cx.add_window_view(|_, _| Narrow { width });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        ["stepper-label-0", "stepper-label-1", "stepper-label-2"]
            .into_iter()
            .map(|label| cx.debug_bounds(label).unwrap())
            .collect()
    }

    #[gpui::test]
    fn labels_never_overlap_at_a_narrow_width(cx: &mut TestAppContext) {
        let labels = labels(cx, 320.);
        for pair in labels.windows(2) {
            assert!(
                pair[0].right() <= pair[1].left(),
                "{:?} overlaps {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[gpui::test]
    fn labels_stay_inside_a_narrow_stepper_by_wrapping(cx: &mut TestAppContext) {
        let labels = labels(cx, 320.);
        assert!(labels[2].right() <= px(320.), "{:?}", labels[2]);
        let wide = self::labels(cx, 2000.);
        assert!(
            labels
                .iter()
                .zip(&wide)
                .any(|(narrow, wide)| narrow.size.height > wide.size.height),
            "a label that does not fit wraps: {labels:?}"
        );
    }
}
