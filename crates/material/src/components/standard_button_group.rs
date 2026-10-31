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
    // The neighbours pay evenly as far as each can, and the group's extent stays
    // put: the press gains only what they can give up.
    let wanted = natural[pressed] * EXPANDED_RATIO;
    let mut paying: Vec<usize> = neighbours;
    let mut left = wanted;
    while !paying.is_empty() && left > 0. {
        let share = left / paying.len() as f32;
        let (spent, rest): (Vec<usize>, Vec<usize>) = paying
            .iter()
            .partition(|index| natural[**index] + deltas[**index] <= share);
        if spent.is_empty() {
            for index in &paying {
                deltas[*index] -= share;
            }
            left = 0.;
        } else {
            for index in &spent {
                left -= natural[*index] + deltas[*index];
                deltas[*index] = -natural[*index];
            }
            paying = rest;
        }
    }
    deltas[pressed] = wanted - left;
    deltas
}

/// Group density: each step below default takes 4dp off the buttons'
/// height (Figma kit "Segmented button" Density 0, −1, −2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonDensity {
    #[default]
    Default,
    Comfortable,
    Compact,
}

impl ButtonDensity {
    pub const ALL: [Self; 3] = [Self::Default, Self::Comfortable, Self::Compact];
    /// The height change in design px.
    pub const fn offset(self) -> f32 {
        match self {
            Self::Default => 0.,
            Self::Comfortable => -4.,
            Self::Compact => -8.,
        }
    }
}

/// Per-item state that survives renders: ids, press probes and the
/// natural main-axis extents measured while each item is at rest.
#[derive(Default)]
struct GroupState {
    items: Vec<ItemState>,
    /// Whether an equal group is sharing a parent too narrow for its widest
    /// width, so its buttons are narrower than they measured.
    shrunk: Rc<Cell<bool>>,
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
    vertical: bool,
    density: ButtonDensity,
    equal_widths: bool,
    items: Vec<MaterialButton>,
}

impl ButtonGroup {
    /// `label` names the group for assistive technology.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            size: ButtonSize::Small,
            vertical: false,
            density: ButtonDensity::Default,
            equal_widths: false,
            items: Vec::new(),
        }
    }
    /// The size of the group's buttons, which sets the gap between them.
    pub fn button_size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }
    /// Stack the buttons vertically; a pressed button then grows in height.
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }
    /// Shorten every button by the density's step.
    pub fn density(mut self, density: ButtonDensity) -> Self {
        self.density = density;
        self
    }
    /// Give every button the widest one's width.
    pub fn equal_widths(mut self, equal: bool) -> Self {
        self.equal_widths = equal;
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
        let shrunk = state.read(cx).shrunk.clone();
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
            let items = &state.items;
            (
                items
                    .iter()
                    .map(|item| item.pressed.clone())
                    .collect::<Vec<_>>(),
                items
                    .iter()
                    .map(|item| item.natural.clone())
                    .collect::<Vec<_>>(),
                items.iter().map(|item| item.id.clone()).collect::<Vec<_>>(),
            )
        });
        let pressed = probes.iter().position(|probe| probe.get());
        let natural: Vec<f32> = naturals.iter().map(|extent| extent.get()).collect();
        let vertical = self.vertical;
        let equal = self.equal_widths;
        // Horizontal equal widths share the widest measured width (which
        // then holds while the group is equal), or take the same off it in a
        // parent too narrow for that; vertical ones stretch to the column.
        let widest = natural.iter().copied().fold(0., f32::max);
        let shares = equal && !vertical && widest > 0.;
        let is_shrunk = shares && shrunk.get();
        let targets = if is_shrunk {
            width_deltas(&vec![widest; natural.len()], pressed)
        } else {
            width_deltas(&natural, pressed)
        };
        let height = (self.size.height() + self.density.offset()).max(0.);
        let dense = self.density != ButtonDensity::Default;
        let mut row = div()
            .id(self.id)
            .role(Role::Group)
            .aria_label(self.label)
            .flex()
            .map(|row| {
                if vertical {
                    row.flex_col()
                } else {
                    row.flex_row()
                }
            })
            .map(|row| {
                if is_shrunk {
                    // Its buttons no longer tell how wide it would like to be.
                    let gap = rem(standard_group_gap(self.size)).to_pixels(window.rem_size());
                    let gaps = natural.len().saturating_sub(1) as f32;
                    row.w_full()
                        .min_w_0()
                        .max_w(px(widest * natural.len() as f32 + f32::from(gap) * gaps))
                } else if shares {
                    row.min_w_0()
                } else {
                    row.flex_shrink_0()
                }
            })
            .map(|row| match (vertical, equal) {
                (true, true) => row.items_stretch(),
                _ => row.items_center(),
            })
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
            let extent = if is_shrunk { widest } else { natural[index] } + delta;
            let button = button
                .press_probe(probes[index].clone())
                .when(dense, |button| button.h(rem(height)))
                .when(shares && !is_shrunk, |button| button.min_w(px(widest)))
                .when(is_shrunk, |button| button.w_full().min_w_0())
                .when(equal && vertical, |button| button.w_full())
                .when(moving && natural[index] > 0. && !is_shrunk, |button| {
                    if vertical {
                        button.h(px(extent.max(0.))).min_h(px(0.))
                    } else {
                        button.w(px(extent.max(0.))).min_w(px(0.))
                    }
                });
            let measured = naturals[index].clone();
            let shrunk = shrunk.clone();
            let slot = div().relative().flex().map(|slot| {
                if is_shrunk {
                    // Shrinking takes the same from equal bases.
                    let basis = if moving { extent.max(0.) } else { widest };
                    slot.flex_basis(px(basis)).min_w_0()
                } else if shares {
                    slot.min_w_0()
                } else {
                    slot.flex_shrink_0()
                }
            });
            row = row.child(
                slot.child(button).child(
                    canvas(
                        move |bounds, window, _| {
                            // Only an item at rest shows its natural extent.
                            let extent = f32::from(if vertical {
                                bounds.size.height
                            } else {
                                bounds.size.width
                            });
                            let narrower = shares && extent + 0.01 < widest;
                            if shares && !moving && shrunk.get() != narrower {
                                shrunk.set(narrower);
                                window.request_animation_frame();
                            }
                            if !moving && !narrower && (measured.get() - extent).abs() > 0.01 {
                                measured.set(extent);
                                if equal && !vertical {
                                    // The others take the new widest width next frame.
                                    window.request_animation_frame();
                                }
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

#[cfg(test)]
mod layout_tests {
    use super::{ButtonDensity, ButtonGroup};
    use crate::components::MaterialButton;
    use gpui::{
        Context, InteractiveElement as _, IntoElement, ParentElement as _, Render, Styled as _,
        TestAppContext, VisualTestContext, Window, div, px,
    };

    const ITEMS: [&str; 3] = ["item-0", "item-1", "item-2"];
    const LABELS: [&str; 3] = ["S", "Medium", "Extra large"];

    struct Harness {
        vertical: bool,
        density: ButtonDensity,
        equal: bool,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().flex().items_start().p(px(40.)).child(
                ButtonGroup::new("group", "Sizes")
                    .vertical(self.vertical)
                    .density(self.density)
                    .equal_widths(self.equal)
                    .items((0..3usize).map(|index| {
                        MaterialButton::new(("item", index))
                            .debug_selector(move || ITEMS[index].into())
                            .child(LABELS[index])
                    })),
            )
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        for _ in 0..3 {
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
        }
    }

    fn bounds(harness: Harness, cx: &mut TestAppContext) -> [gpui::Bounds<gpui::Pixels>; 3] {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (_, cx) = cx.add_window_view(move |_, _| harness);
        draw(cx);
        ITEMS.map(|item| cx.debug_bounds(item).unwrap())
    }

    #[gpui::test]
    fn a_vertical_compact_group_stacks_shorter_buttons(cx: &mut TestAppContext) {
        let [first, second, _] = bounds(
            Harness {
                vertical: true,
                density: ButtonDensity::Compact,
                equal: false,
            },
            cx,
        );
        assert_eq!(first.size.height, px(40. - 8.));
        assert_eq!(second.top(), first.bottom() + px(12.), "the S gap");
    }

    #[gpui::test]
    fn equal_widths_share_the_widest_button(cx: &mut TestAppContext) {
        let [a, b, c] = bounds(
            Harness {
                vertical: false,
                density: ButtonDensity::Default,
                equal: true,
            },
            cx,
        );
        assert!(c.size.width > px(80.));
        assert_eq!(a.size.width, c.size.width);
        assert_eq!(b.size.width, c.size.width);
    }

    struct Narrow {
        width: f32,
    }
    impl Render for Narrow {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().flex().w(px(self.width)).child(
                ButtonGroup::new("group", "Sizes")
                    .equal_widths(true)
                    .items((0..3usize).map(|index| {
                        MaterialButton::new(("item", index))
                            .debug_selector(move || ITEMS[index].into())
                            .child(LABELS[index])
                    })),
            )
        }
    }

    /// Equal widths share a parent too narrow for the widest button three times,
    /// and take the widest again when there is room.
    #[gpui::test]
    fn equal_widths_shrink_together_in_a_narrow_parent(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| Narrow { width: 600. });
        draw(cx);
        let [_, _, wide] = ITEMS.map(|item| cx.debug_bounds(item).unwrap());
        assert!(wide.size.width > px(80.));
        let natural = wide.size.width;

        cx.update(|_, cx| view.update(cx, |view, _| view.width = 180.));
        draw(cx);
        let [a, b, c] = ITEMS.map(|item| cx.debug_bounds(item).unwrap());
        assert!(c.right() <= a.left() + px(180.), "inside: {a:?} .. {c:?}");
        assert_eq!(a.size.width, c.size.width);
        assert_eq!(b.size.width, c.size.width);
        assert!(c.size.width < natural);

        cx.update(|_, cx| view.update(cx, |view, _| view.width = 600.));
        draw(cx);
        assert_eq!(cx.debug_bounds(ITEMS[0]).unwrap().size.width, natural);
    }

    #[gpui::test]
    fn equal_vertical_buttons_stretch_to_the_column(cx: &mut TestAppContext) {
        let [a, _, c] = bounds(
            Harness {
                vertical: true,
                density: ButtonDensity::Comfortable,
                equal: true,
            },
            cx,
        );
        assert_eq!(a.size.width, c.size.width);
        assert_eq!(a.size.height, px(36.));
    }

    #[core::prelude::v1::test]
    fn a_neighbour_is_not_pushed_below_nothing() {
        let deltas = super::width_deltas(&[48., 400.], Some(1));
        assert!(deltas[0] >= -48., "{deltas:?}");
        let total: f32 = deltas.iter().sum();
        assert!(
            total.abs() < 0.001,
            "the group's extent stays put: {deltas:?}"
        );
        // A wide neighbour beside a narrow one takes up what it cannot.
        let deltas = super::width_deltas(&[10., 400., 300.], Some(1));
        assert!(deltas[0] >= -10., "{deltas:?}");
        let total: f32 = deltas.iter().sum();
        assert!(total.abs() < 0.001, "{deltas:?}");
    }
}
