//! A progress stepper: numbered steps joined by hairlines, for a flow a person walks through
//! once. Material 3 specifies no stepper; this keeps to M3 roles and type, and to the form
//! Material 2's horizontal stepper gave it. Strings come from the caller.
//!
//! Assistive technology hears a list named by the caller, whose items are the steps: each with
//! its name, position in the list, whether it is the current one and, given
//! [`Stepper::state_labels`], the caller's words for its state.
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
///
/// ```
/// use gpui_kit_material::prelude::*;
///
/// let setup = Stepper::new("setup", "Setup progress")
///     .steps(["Source", "Service", "Account"])
///     .current(1)
///     .state_labels("Finished", "Current", "Upcoming");
/// # let _ = setup;
/// ```
#[derive(gpui::IntoElement)]
pub struct Stepper {
    id: ElementId,
    label: SharedString,
    steps: Vec<SharedString>,
    current: usize,
    state_labels: Option<[SharedString; 3]>,
}

impl Stepper {
    /// `label` names the whole progress for assistive technology.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            steps: Vec::new(),
            current: 0,
            state_labels: None,
        }
    }
    /// The steps' names, in order. Each is shown and is the step's accessible name; a label
    /// too long for its share of the width wraps.
    pub fn steps<S: Into<SharedString>>(mut self, steps: impl IntoIterator<Item = S>) -> Self {
        self.steps = steps.into_iter().map(Into::into).collect();
        self
    }
    /// The current step, 0-based. Past the last step, every step is finished.
    pub fn current(mut self, current: usize) -> Self {
        self.current = current;
        self
    }
    /// What assistive technology announces for a finished, the current and an upcoming step.
    /// Without them it still hears the current step as selected and each step's position.
    pub fn state_labels(
        mut self,
        finished: impl Into<SharedString>,
        current: impl Into<SharedString>,
        upcoming: impl Into<SharedString>,
    ) -> Self {
        self.state_labels = Some([finished.into(), current.into(), upcoming.into()]);
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
            let description = self
                .state_labels
                .as_ref()
                .map(|[finished, current, upcoming]| match state {
                    StepState::Finished => finished.clone(),
                    StepState::Current => current.clone(),
                    StepState::Upcoming => upcoming.clone(),
                });
            children.push(
                div()
                    .id(("stepper-step", index))
                    .role(gpui::Role::ListItem)
                    .aria_label(step.clone())
                    .aria_selected(state == StepState::Current)
                    .aria_position_in_set(index + 1)
                    .aria_size_of_set(count)
                    .when_some(description, |item, state| item.aria_description(state))
                    .debug_selector(|| format!("stepper-step-{index}"))
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
            .role(gpui::Role::List)
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

    struct Steps {
        current: usize,
        steps: Vec<&'static str>,
    }

    impl Render for Steps {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(16.));
            div().debug_selector(|| "root".into()).w(px(600.)).child(
                Stepper::new("steps", "Progress")
                    .steps(self.steps.iter().copied())
                    .current(self.current),
            )
        }
    }

    fn drawn(cx: &mut TestAppContext, view: Steps) -> &mut gpui::VisualTestContext {
        cx.update(|cx| {
            gpui_kit::init(cx);
            crate::init(cx);
        });
        let (_, cx) = cx.add_window_view(move |_, _| view);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx
    }

    #[gpui::test]
    fn the_last_step_ends_at_the_end_and_hairlines_take_what_is_left(cx: &mut TestAppContext) {
        let cx = drawn(
            cx,
            Steps {
                current: 1,
                steps: vec!["One", "Two"],
            },
        );
        let root = cx.debug_bounds("root").unwrap();
        let (first, second) = (
            cx.debug_bounds("stepper-step-0").unwrap(),
            cx.debug_bounds("stepper-step-1").unwrap(),
        );
        assert_eq!(first.left(), root.left());
        assert_eq!(second.right(), root.right());
        assert!(
            second.left() - first.right() > px(16.),
            "a hairline between"
        );
    }

    /// An empty stepper, a current step past the end and a lone step, none of which panic.
    #[gpui::test]
    fn no_steps_and_a_current_step_past_the_end_draw(cx: &mut TestAppContext) {
        let cx = drawn(
            cx,
            Steps {
                current: 9,
                steps: Vec::new(),
            },
        );
        assert_eq!(cx.debug_bounds("stepper-step-0"), None);
        assert_eq!(step_state(3, 9), StepState::Finished, "all done");
    }

    #[gpui::test]
    fn a_lone_step_has_no_hairline(cx: &mut TestAppContext) {
        let cx = drawn(
            cx,
            Steps {
                current: 0,
                steps: vec!["Only"],
            },
        );
        let root = cx.debug_bounds("root").unwrap();
        let only = cx.debug_bounds("stepper-step-0").unwrap();
        assert_eq!(only.left(), root.left());
        assert!(only.right() < root.right(), "no hairline stretches it");
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
