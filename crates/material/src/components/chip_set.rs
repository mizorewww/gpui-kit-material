//! A set of chips: the Figma kit's chip groups and md3.io's chip set options
//! (vertical, group label, multiple, required, draggable).
//!
//! The set arranges chips of any kind, names the group for assistive
//! technology and reports reordering. Selection stays with the caller, as for
//! every control here; [`ChipSelection`] holds the rules so each caller
//! does not write them again.
use crate::{
    scope::{CapturedScope, horizontal_arrow},
    tokens::typography::rem,
};
use gpui::{prelude::*, *};
use std::rc::Rc;

/// M3 chips in a set sit 8dp apart.
const CHIP_GAP: f32 = 8.;

/// The selection rules of a chip set.
///
/// # Example
///
/// ```
/// use gpui_kit_material::prelude::*;
///
/// let rules = ChipSelection { multiple: false, required: true };
/// let mut selected = [true, false, false];
/// assert!(rules.apply(&mut selected, 1, true));
/// assert_eq!(selected, [false, true, false]);
/// // The last selected chip stays selected.
/// assert!(!rules.apply(&mut selected, 1, false));
/// ```
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
///
/// # Example
///
/// ```
/// use gpui_kit::prelude::*;
/// use gpui_kit_material::prelude::*;
///
/// let item = ChipSetItem::new("tomato", InputChip::new("tomato", "Tomato"))
///     .drag_preview(|_window, _cx| {
///         InputChip::new("tomato-dragged", "Tomato").dragged(true).into_any_element()
///     });
/// # let _ = item;
/// ```
pub struct ChipSetItem {
    key: ElementId,
    chip: AnyElement,
    drag_preview: Option<DragPreview>,
}

impl ChipSetItem {
    /// `key` names what this chip stands for and stays with it as the set is
    /// reordered: the chip's state, its keyboard focus above all, lives under
    /// it. Keys must be unique within the set.
    pub fn new(key: impl Into<ElementId>, chip: impl IntoElement) -> Self {
        Self {
            key: key.into(),
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
    /// A drag draws from the window's root, outside the set's scope.
    scope: CapturedScope,
}

impl Render for DraggedChip {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.scope.draw((self.preview)(window, cx))
    }
}

type Reorder = Rc<dyn Fn(usize, usize, &mut Window, &mut App)>;

/// Chips arranged in a wrapping row or a column, named as one group.
///
/// # Example
///
/// ```
/// use gpui_kit_material::prelude::*;
///
/// let categories = ["Shoes", "Bags", "Hats"];
/// let set = ChipSet::new("categories", "Categories")
///     .items(categories.into_iter().enumerate().map(|(index, label)| {
///         let chip = FilterChip::new(("category", index), label)
///             .on_toggle(move |on, _window, _cx| println!("{label}: {on}"));
///         ChipSetItem::new(label, chip)
///     }))
///     .on_reorder(|from, to, _window, _cx| println!("Move {from} to {to}"));
/// # let _ = set;
/// ```
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
    ///
    /// The keyboard reorders them too: Alt with an arrow along the set moves
    /// the chip holding focus one place, which calls this the same way. Focus
    /// moves with the chip, by the [`key`](ChipSetItem::new) of its item.
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

/// Which way a keystroke moves a chip along the set, if it does.
fn reorder_step(keystroke: &Keystroke, vertical: bool) -> Option<i32> {
    let modifiers = keystroke.modifiers;
    if !modifiers.alt || modifiers.control || modifiers.platform || modifiers.shift {
        return None;
    }
    if vertical {
        match keystroke.key.as_str() {
            "up" => Some(-1),
            "down" => Some(1),
            _ => None,
        }
    } else {
        horizontal_arrow(&keystroke.key)
    }
}

impl RenderOnce for ChipSet {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let set = self.id.clone();
        let on_reorder = self.on_reorder;
        let vertical = self.vertical;
        let scope = CapturedScope::now();
        // Only a chip that can be dragged can be dropped on or moved past.
        let movable: Rc<[bool]> = self
            .items
            .iter()
            .map(|item| item.drag_preview.is_some())
            .collect();
        let slots = self.items.into_iter().enumerate().map(|(position, item)| {
            let preview = item.drag_preview.filter(|_| on_reorder.is_some());
            let on_reorder = on_reorder.clone();
            let set = set.clone();
            let scope = scope.clone();
            let movable = movable.clone();
            div()
                .id(item.key)
                .when_some(preview, |slot, preview| {
                    let dragged = DraggedChip {
                        set: set.clone(),
                        from: position,
                        preview,
                        scope,
                    };
                    let keyed = on_reorder.clone();
                    slot.on_key_down(move |event, window, cx| {
                        let Some(step) = reorder_step(&event.keystroke, vertical) else {
                            return;
                        };
                        let target = position.checked_add_signed(step as isize);
                        let Some(target) =
                            target.filter(|target| movable.get(*target).copied().unwrap_or(false))
                        else {
                            return;
                        };
                        cx.stop_propagation();
                        if let Some(on_reorder) = &keyed {
                            on_reorder(position, target, window, cx);
                        }
                    })
                    .cursor_grab()
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
                        i,
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

    fn ink() -> crate::color::Scheme {
        crate::color::Scheme::from_palette(crate::color::PaletteId::Ink)
    }

    /// What a preview sees of where it is drawn.
    #[derive(gpui::IntoElement)]
    struct Probe(Rc<std::cell::Cell<Option<bool>>>);
    impl RenderOnce for Probe {
        fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
            self.0.set(Some(crate::theme::scheme() == ink()));
            div().size(px(40.))
        }
    }

    struct Scoped {
        scoped_in_preview: Rc<std::cell::Cell<Option<bool>>>,
    }
    impl Render for Scoped {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let seen = self.scoped_in_preview.clone();
            crate::scope::Scope::new("ink-chips").scheme(ink()).child(
                ChipSet::new("set", "Filters")
                    .item(
                        ChipSetItem::new(
                            "chip",
                            div().size(px(40.)).debug_selector(|| "chip".into()),
                        )
                        .drag_preview(move |_, _| Probe(seen.clone()).into_any_element()),
                    )
                    .on_reorder(|_, _, _, _| {}),
            )
        }
    }

    #[gpui::test]
    fn a_drag_preview_draws_in_the_sets_scope(cx: &mut TestAppContext) {
        let seen = Rc::new(std::cell::Cell::new(None));
        let (_, cx) = cx.add_window_view({
            let seen = seen.clone();
            move |_, _| Scoped {
                scoped_in_preview: seen,
            }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let chip = cx.debug_bounds("chip").unwrap().center();
        cx.simulate_mouse_down(chip, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(
            chip + point(px(30.), px(30.)),
            MouseButton::Left,
            Modifiers::default(),
        );
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(seen.get(), Some(true), "the preview is in the scope");
    }

    struct Keyed {
        vertical: bool,
        draggable: [bool; 3],
        focus: Vec<FocusHandle>,
        reorders: Vec<(usize, usize)>,
    }
    impl Render for Keyed {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let view = cx.entity();
            ChipSet::new("set", "Filters")
                .vertical(self.vertical)
                .items((0..3).map(|i| {
                    let item = ChipSetItem::new(i, div().size(px(40.)).track_focus(&self.focus[i]));
                    if self.draggable[i] {
                        item.drag_preview(|_, _| div().size(px(40.)).into_any_element())
                    } else {
                        item
                    }
                }))
                .on_reorder(move |from, to, _, cx| {
                    view.update(cx, |this, _| this.reorders.push((from, to)));
                })
        }
    }

    fn keyed(
        cx: &mut TestAppContext,
        vertical: bool,
        draggable: [bool; 3],
    ) -> (Entity<Keyed>, &mut VisualTestContext) {
        let (view, cx) = cx.add_window_view(move |_, cx| Keyed {
            vertical,
            draggable,
            focus: (0..3).map(|_| cx.focus_handle()).collect(),
            reorders: Vec::new(),
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        (view, cx)
    }

    fn press_on(
        view: &Entity<Keyed>,
        cx: &mut VisualTestContext,
        chip: usize,
        keys: &str,
    ) -> Vec<(usize, usize)> {
        cx.update(|window, cx| {
            let focus = view.read(cx).focus[chip].clone();
            window.focus(&focus, cx);
            window.draw(cx).clear(cx);
        });
        cx.simulate_keystrokes(keys);
        view.update(cx, |view, _| std::mem::take(&mut view.reorders))
    }

    #[gpui::test]
    fn alt_and_an_arrow_move_the_focused_chip(cx: &mut TestAppContext) {
        let (view, cx) = keyed(cx, false, [true; 3]);
        assert_eq!(press_on(&view, cx, 1, "alt-right"), [(1, 2)]);
        assert_eq!(press_on(&view, cx, 1, "alt-left"), [(1, 0)]);
        assert_eq!(press_on(&view, cx, 2, "alt-right"), [], "the end stays put");
        assert_eq!(press_on(&view, cx, 0, "alt-left"), [], "so does the start");
        assert_eq!(press_on(&view, cx, 1, "right"), [], "alt is what moves it");
        assert_eq!(press_on(&view, cx, 1, "alt-down"), [], "not across the set");
    }

    #[gpui::test]
    fn the_keys_follow_the_sets_axis(cx: &mut TestAppContext) {
        let (view, cx) = keyed(cx, true, [true; 3]);
        assert_eq!(press_on(&view, cx, 1, "alt-down"), [(1, 2)], "a column");
        assert_eq!(press_on(&view, cx, 1, "alt-right"), []);
    }

    #[gpui::test]
    fn a_chip_that_cannot_be_dragged_cannot_be_moved_past(cx: &mut TestAppContext) {
        let (view, cx) = keyed(cx, false, [true, false, true]);
        assert_eq!(press_on(&view, cx, 0, "alt-right"), []);
        assert_eq!(press_on(&view, cx, 1, "alt-right"), []);
    }
    /// Chips in the caller's order, each owning its focus the way a real chip
    /// does: in keyed state under the set, not in this view.
    struct Reordering {
        order: Vec<&'static str>,
    }
    impl Render for Reordering {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let view = cx.entity();
            ChipSet::new("set", "Filters")
                .items(self.order.iter().map(|&name| {
                    ChipSetItem::new(name, crate::components::AssistChip::new(name, name))
                        .drag_preview(|_, _| div().into_any_element())
                }))
                .on_reorder(move |from, to, _, cx| {
                    view.update(cx, |this, cx| {
                        let chip = this.order.remove(from);
                        this.order.insert(to, chip);
                        cx.notify();
                    });
                })
        }
    }

    #[gpui::test]
    fn focus_stays_on_a_chip_moved_again_and_again(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| Reordering {
            order: vec!["a", "b", "c", "d"],
        });
        let moved = cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            window.focus_next(cx);
            window.draw(cx).clear(cx);
            window.focused(cx).expect("the first chip takes focus")
        });
        for expected in [
            ["b", "a", "c", "d"],
            ["b", "c", "a", "d"],
            ["b", "c", "d", "a"],
        ] {
            cx.simulate_keystrokes("alt-right");
            cx.update(|window, cx| window.draw(cx).clear(cx));
            view.read_with(cx, |view, _| assert_eq!(view.order, expected));
            cx.update(|window, cx| {
                assert_eq!(window.focused(cx), Some(moved.clone()), "focus moved along");
            });
        }
    }
}
