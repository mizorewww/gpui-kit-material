//! A set of chips: the Figma kit's chip groups and md3.io's chip set options
//! (vertical, group label, multiple, required, draggable).
//!
//! The set arranges chips of any kind, names the group for assistive
//! technology and reports reordering. Selection stays with the caller, as for
//! every control here; [`ChipSelection`] holds the rules so each caller
//! does not write them again.
use crate::tokens::typography::rem;
use gpui::{prelude::*, *};
use std::rc::Rc;

/// M3 chips in a set sit 8dp apart.
const CHIP_GAP: f32 = 8.;

/// The selection rules of a chip set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ChipSelection {
    /// Several chips may be selected; otherwise choosing one clears the rest.
    pub multiple: bool,
    /// The last selected chip cannot be cleared.
    pub required: bool,
}

impl ChipSelection {
    /// Apply a request to set chip `index` to `on`. An out-of-range index is
    /// ignored. Returns whether the selection changed.
    pub fn apply(self, selected: &mut [bool], index: usize, on: bool) -> bool {
        if index >= selected.len() {
            return false;
        }
        let before: Vec<bool> = selected.to_vec();
        if on {
            if !self.multiple {
                selected.iter_mut().for_each(|chip| *chip = false);
            }
            selected[index] = true;
        } else {
            let others = selected.iter().enumerate().any(|(i, s)| *s && i != index);
            if !self.required || others {
                selected[index] = false;
            }
        }
        before != selected
    }
}

type DragPreview = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// One chip in a [`ChipSet`].
pub struct ChipSetItem {
    chip: AnyElement,
    drag_preview: Option<DragPreview>,
}

impl ChipSetItem {
    pub fn new(chip: impl IntoElement) -> Self {
        Self {
            chip: chip.into_any_element(),
            drag_preview: None,
        }
    }
    /// What follows the pointer while this chip is dragged, usually the same
    /// chip with [`dragged`](crate::components::AssistChip::dragged) set.
    /// A set item without one cannot be dragged.
    pub fn drag_preview(
        mut self,
        preview: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.drag_preview = Some(Rc::new(preview));
        self
    }
}

/// The chip being dragged: its position in the set and how it looks.
#[derive(Clone)]
struct DraggedChip {
    set: ElementId,
    from: usize,
    preview: DragPreview,
}

impl Render for DraggedChip {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        (self.preview)(window, cx)
    }
}

type Reorder = Rc<dyn Fn(usize, usize, &mut Window, &mut App)>;

/// Chips arranged in a wrapping row or a column, named as one group.
#[derive(gpui::IntoElement)]
pub struct ChipSet {
    id: ElementId,
    label: SharedString,
    vertical: bool,
    items: Vec<ChipSetItem>,
    on_reorder: Option<Reorder>,
    style: StyleRefinement,
}

impl ChipSet {
    /// `label` names the group for assistive technology.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            vertical: false,
            items: Vec::new(),
            on_reorder: None,
            style: StyleRefinement::default(),
        }
    }
    /// Stack the chips instead of wrapping them in a row.
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }
    pub fn item(mut self, item: ChipSetItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = ChipSetItem>) -> Self {
        self.items.extend(items);
        self
    }
    /// Make chips with a [`ChipSetItem::drag_preview`] draggable within the
    /// set. Called with the moved chip's position and where it was dropped.
    pub fn on_reorder(mut self, f: impl Fn(usize, usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_reorder = Some(Rc::new(f));
        self
    }
}

impl Styled for ChipSet {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for ChipSet {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let set = self.id.clone();
        let on_reorder = self.on_reorder;
        let slots = self.items.into_iter().enumerate().map(|(position, item)| {
            let preview = item.drag_preview.filter(|_| on_reorder.is_some());
            let on_reorder = on_reorder.clone();
            let set = set.clone();
            div()
                .id(("chip-slot", position))
                .when_some(preview, |slot, preview| {
                    let dragged = DraggedChip {
                        set: set.clone(),
                        from: position,
                        preview,
                    };
                    slot.cursor_grab()
                        .on_drag(dragged, |dragged, _, _, cx| cx.new(|_| dragged.clone()))
                        .drag_over::<DraggedChip>(|style, _, _, _| style.opacity(0.6))
                        .on_drop(move |dragged: &DraggedChip, window, cx| {
                            if dragged.set == set
                                && let Some(on_reorder) = &on_reorder
                            {
                                on_reorder(dragged.from, position, window, cx);
                            }
                        })
                })
                .child(item.chip)
        });
        let mut root = div()
            .id(self.id)
            .role(Role::Group)
            .aria_label(self.label)
            .flex()
            .gap(rem(CHIP_GAP))
            .map(|root| {
                if self.vertical {
                    root.flex_col().items_start()
                } else {
                    root.flex_wrap()
                }
            })
            .children(slots);
        root.style().refine(&self.style);
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn selection_follows_the_multiple_and_required_rules() {
        let single = ChipSelection::default();
        let mut selected = [false; 3];
        assert!(single.apply(&mut selected, 0, true));
        assert!(single.apply(&mut selected, 1, true));
        assert_eq!(selected, [false, true, false], "one at a time");
        assert!(
            single.apply(&mut selected, 1, false),
            "clearable by default"
        );

        let required = ChipSelection {
            multiple: false,
            required: true,
        };
        let mut selected = [false, true, false];
        assert!(!required.apply(&mut selected, 1, false), "the last stays");
        assert_eq!(selected, [false, true, false]);

        let both = ChipSelection {
            multiple: true,
            required: true,
        };
        assert!(both.apply(&mut selected, 2, true));
        assert_eq!(selected, [false, true, true], "several at once");
        assert!(both.apply(&mut selected, 1, false));
        assert_eq!(selected, [false, false, true], "another remains");
        assert!(!both.apply(&mut selected, 9, true), "out of range");
    }

    struct Harness {
        reorders: Vec<(usize, usize)>,
    }
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            ChipSet::new("set", "Filters")
                .vertical(true)
                .items((0..3).map(|i| {
                    ChipSetItem::new(
                        div()
                            .size(px(40.))
                            .debug_selector(move || format!("chip-{i}")),
                    )
                    .drag_preview(|_, _| div().size(px(40.)).into_any_element())
                }))
                .on_reorder({
                    let view = cx.entity();
                    move |from, to, _, cx| {
                        view.update(cx, |this, _| this.reorders.push((from, to)));
                    }
                })
        }
    }

    #[gpui::test]
    fn dragging_a_chip_onto_another_reports_a_reorder(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| Harness {
            reorders: Vec::new(),
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let first = cx.debug_bounds("chip-0").unwrap();
        let last = cx.debug_bounds("chip-2").unwrap();
        assert!(last.top() > first.bottom(), "a vertical set stacks");
        cx.simulate_mouse_down(first.center(), MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(last.center(), MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_up(last.center(), MouseButton::Left, Modifiers::default());
        view.read_with(cx, |view, _| assert_eq!(view.reorders, [(0, 2)]));
    }
}
