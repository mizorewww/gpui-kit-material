//! M3 Expressive standard (non-connected) button group.
//!
//! Buttons or icon buttons sit apart by the size's gap (XS 18, S 12, M–XL 8)
//! and interact: while one is pressed it widens by 15% of its own width and
//! its adjacent neighbours give up that width between them, on the fast
//! spatial spring, so the group's extent stays put. Releasing springs every
//! item back to its natural width. Reduced motion jumps.
//!
//! Each item stays a complete [`MaterialButton`] (activation, focus, state
//! layer, ripple and shape morph); the group only reads its press state and
//! sets its width while it moves.
use super::{ButtonSize, MaterialButton};
use crate::{
    motion::{self, MotionToken},
    tokens::typography::rem,
};
use gpui::{prelude::*, *};
use std::{cell::Cell, rc::Rc};

/// The pressed item's growth, as a share of its own width.
pub const EXPANDED_RATIO: f32 = 0.15;

/// The gap between items of a standard group of `size` buttons.
pub const fn standard_group_gap(size: ButtonSize) -> f32 {
    match size {
        ButtonSize::ExtraSmall => 18.,
        ButtonSize::Small => 12.,
        _ => 8.,
    }
}

/// Width changes for every item while `pressed` is held, given each item's
/// natural width: the pressed item gains `EXPANDED_RATIO` of its width and
/// its neighbours share that loss. Unmeasured widths (0) change nothing.
pub(crate) fn width_deltas(natural: &[f32], pressed: Option<usize>) -> Vec<f32> {
    let mut deltas = vec![0.; natural.len()];
    let Some(pressed) = pressed.filter(|&index| natural.get(index).is_some_and(|&w| w > 0.)) else {
        return deltas;
    };
    let neighbours: Vec<usize> = [pressed.checked_sub(1), Some(pressed + 1)]
        .into_iter()
        .flatten()
        .filter(|&index| index < natural.len())
        .collect();
    if neighbours.is_empty() {
        return deltas;
    }
    let growth = natural[pressed] * EXPANDED_RATIO;
    deltas[pressed] = growth;
    for index in &neighbours {
        deltas[*index] = -growth / neighbours.len() as f32;
    }
    deltas
}

/// Per-item state that survives renders: ids, press probes and the
/// natural widths measured while each item is at rest.
#[derive(Default)]
struct GroupState {
    items: Vec<ItemState>,
}

struct ItemState {
    id: ElementId,
    pressed: Rc<Cell<bool>>,
    natural: Rc<Cell<f32>>,
}

#[derive(gpui::IntoElement)]
pub struct ButtonGroup {
    id: ElementId,
    label: SharedString,
    size: ButtonSize,
    items: Vec<MaterialButton>,
}

impl ButtonGroup {
    /// `label` names the group for assistive technology.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            size: ButtonSize::Small,
            items: Vec::new(),
        }
    }
    /// The size of the group's buttons, which sets the gap between them.
    pub fn button_size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }
    /// Add a button or an icon button.
    pub fn item(mut self, item: impl Into<MaterialButton>) -> Self {
        self.items.push(item.into());
        self
    }
    pub fn items<I: Into<MaterialButton>>(mut self, items: impl IntoIterator<Item = I>) -> Self {
        self.items.extend(items.into_iter().map(Into::into));
        self
    }
}

impl RenderOnce for ButtonGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state((self.id.clone(), "button-group"), cx, |_, _| {
            GroupState::default()
        });
        let count = self.items.len();
        let (probes, naturals, ids) = state.update(cx, |state, _| {
            while state.items.len() < count {
                let index = state.items.len();
                state.items.push(ItemState {
                    id: ElementId::NamedChild(
                        std::sync::Arc::new((self.id.clone(), "button-group-width").into()),
                        index.to_string().into(),
                    ),
                    pressed: Rc::default(),
                    natural: Rc::default(),
                });
            }
            state.items.truncate(count);
            (
                state
                    .items
                    .iter()
                    .map(|item| item.pressed.clone())
                    .collect::<Vec<_>>(),
                state
                    .items
                    .iter()
                    .map(|item| item.natural.clone())
                    .collect::<Vec<_>>(),
                state
                    .items
                    .iter()
                    .map(|item| item.id.clone())
                    .collect::<Vec<_>>(),
            )
        });
        let pressed = probes.iter().position(|probe| probe.get());
        let natural: Vec<f32> = naturals.iter().map(|width| width.get()).collect();
        let targets = width_deltas(&natural, pressed);
        let mut row = div()
            .id(self.id)
            .role(Role::Group)
            .aria_label(self.label)
            .flex()
            .flex_row()
            .flex_shrink_0()
            .items_center()
            .gap(rem(standard_group_gap(self.size)));
        for (index, button) in self.items.into_iter().enumerate() {
            let delta = motion::token_value(
                ids[index].clone(),
                targets[index],
                MotionToken::FastSpatial,
                window,
                cx,
            );
            let moving = delta != 0.;
            let width = natural[index] + delta;
            let button = button
                .press_probe(probes[index].clone())
                .when(moving && natural[index] > 0., |button| {
                    button.w(px(width.max(0.))).min_w(px(0.))
                });
            let measured = naturals[index].clone();
            row = row.child(
                div().relative().flex().flex_shrink_0().child(button).child(
                    canvas(
                        move |bounds, _, _| {
                            // Only an item at rest shows its natural width.
                            if !moving {
                                measured.set(f32::from(bounds.size.width));
                            }
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                ),
            );
        }
        row
    }
}

#[cfg(test)]
mod tests {
    use super::{ButtonGroup, EXPANDED_RATIO, standard_group_gap, width_deltas};
    use crate::components::{ButtonSize, InteractionPreview, MaterialButton};
    use crate::motion;
    use gpui::{
        Context, InteractiveElement as _, IntoElement, ParentElement as _, Render, Styled as _,
        TestAppContext, VisualTestContext, Window, div, prelude::FluentBuilder as _, px,
    };
    use std::time::Duration;

    const ITEMS: [&str; 3] = ["item-0", "item-1", "item-2"];

    #[core::prelude::v1::test]
    fn the_pressed_item_grows_and_its_neighbours_pay() {
        assert_eq!(width_deltas(&[100., 100., 100.], None), [0., 0., 0.]);
        let deltas = width_deltas(&[100., 80., 100., 60.], Some(1));
        let growth = 80. * EXPANDED_RATIO;
        assert_eq!(deltas, [-growth / 2., growth, -growth / 2., 0.]);
        assert!(deltas.iter().sum::<f32>().abs() < 1e-4, "the extent stays");
        let end = width_deltas(&[100., 100.], Some(1));
        assert_eq!(end, [-100. * EXPANDED_RATIO, 100. * EXPANDED_RATIO]);
        assert_eq!(width_deltas(&[0., 100.], Some(0)), [0., 0.], "unmeasured");
        assert_eq!(
            width_deltas(&[100.], Some(0)),
            [0.],
            "nothing to borrow from"
        );
        assert_eq!(standard_group_gap(ButtonSize::ExtraSmall), 18.);
        assert_eq!(standard_group_gap(ButtonSize::Large), 8.);
    }

    struct Harness {
        pressed: bool,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div()
                .flex()
                .items_start()
                .p(px(40.))
                .child(
                    ButtonGroup::new("group", "Group").items((0..3usize).map(|index| {
                        MaterialButton::new(("item", index))
                            .debug_selector(move || ITEMS[index].into())
                            .child("Label")
                            .when(index == 1 && self.pressed, |button| {
                                button.preview_state(InteractionPreview::Pressed)
                            })
                    })),
                )
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }

    #[gpui::test]
    fn pressing_widens_an_item_and_release_restores_it(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(false);
            motion::set_motion_scheme(motion::MotionScheme::Expressive);
        });
        let (view, cx) = cx.add_window_view(|_, _| Harness { pressed: false });
        draw(cx);
        draw(cx);
        let widths = |cx: &mut VisualTestContext| {
            ITEMS.map(|item| cx.debug_bounds(item).unwrap().size.width)
        };
        let rest = widths(cx);
        let extent = |cx: &mut VisualTestContext| {
            cx.debug_bounds("item-2").unwrap().right() - cx.debug_bounds("item-0").unwrap().left()
        };
        let rest_extent = extent(cx);
        view.update(cx, |view, cx| {
            view.pressed = true;
            cx.notify();
        });
        for _ in 0..3 {
            draw(cx);
        }
        cx.executor().advance_clock(Duration::from_secs(2));
        draw(cx);
        draw(cx);
        let pressed = widths(cx);
        assert!(
            (pressed[1] - rest[1] * (1. + EXPANDED_RATIO)).abs() < px(0.5),
            "{pressed:?} vs {rest:?}"
        );
        assert!(pressed[0] < rest[0] && pressed[2] < rest[2]);
        assert!(
            (extent(cx) - rest_extent).abs() < px(0.5),
            "the group keeps its extent"
        );
        view.update(cx, |view, cx| {
            view.pressed = false;
            cx.notify();
        });
        for _ in 0..3 {
            draw(cx);
        }
        cx.executor().advance_clock(Duration::from_secs(2));
        draw(cx);
        draw(cx);
        assert_eq!(widths(cx), rest);
    }
}
