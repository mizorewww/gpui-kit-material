//! Single choice among tiles: the M3 radio indicator, a choice tile with
//! tonal selection, and category tints.
use crate::color::ColorRole;
use crate::icons::Icon;
use crate::theme::{blend, color};
use crate::tokens::{layout::*, shape::*, state, typography::*};
use gpui::{prelude::*, *};

/// The M3 radio of a choice: an outlined ring, filled with a dot when
/// selected; 38% ink when the choice is unavailable.
pub fn choice_radio(selected: bool, enabled: bool) -> Div {
    let tone = if !enabled {
        Rgba {
            a: 0.38,
            ..color(ColorRole::INK)
        }
    } else if selected {
        color(ColorRole::ACCENT)
    } else {
        color(ColorRole::CONTROL)
    };
    div()
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .size(rem(20.))
        .rounded_full()
        .border_2()
        .border_color(tone)
        .when(selected, |ring| {
            ring.child(div().size(rem(10.)).rounded_full().bg(tone))
        })
}

pub use choice_tile::{ChoiceGroup, ChoiceTile};
mod choice_tile {
    use super::*;
    use crate::components::button_group::{FirstChoice, LastChoice, NextChoice, PreviousChoice};
    use crate::components::focus::{RingPlacement, RingedButton};
    use crate::components::tile::outer_corner_radii;
    use gpui_base::v_flex;
    use std::rc::Rc;

    /// The resolved colors of a choice tile for one frame.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(super) struct TilePaint {
        pub(super) background: Rgba,
        /// The content color, which also tints the state layers.
        pub(super) ink: Rgba,
        /// The label color: the ink, at disabled emphasis when unavailable.
        pub(super) label: Rgba,
        pub(super) hover: Rgba,
        pub(super) press: Rgba,
    }

    /// `amount` is the animated selection (0 unselected, 1 selected).
    pub(super) fn tile_paint(amount: f32, selected: bool, enabled: bool) -> TilePaint {
        let background = blend(
            color(ColorRole::SURFACE),
            color(ColorRole::SECONDARY_CONTAINER),
            amount,
        );
        let ink = color(if selected {
            ColorRole::ON_SECONDARY_CONTAINER
        } else {
            ColorRole::ON_SURFACE
        });
        TilePaint {
            background,
            ink,
            label: if enabled { ink } else { state::disabled(ink) },
            hover: state::hovered(background, ink),
            press: state::pressed(background, ink),
        }
    }

    /// One choice in a connected group of tiles: tonal
    /// selection on the secondary container, a leading icon, the label with
    /// optional supporting text, a trailing status chip and a radio. An
    /// unavailable choice keeps its status chip at full strength.
    ///
    /// It is exposed as a radio button. Put the choices of one question in a
    /// [`ChoiceGroup`], which adds the radio group, roving focus and arrow
    /// keys.
    pub struct ChoiceTile {
        id: ElementId,
        index: usize,
        count: usize,
        outer: OuterCorners,
        icon: Option<AnyElement>,
        tinted: Option<(Icon, Tint, Pixels)>,
        title: SharedString,
        description: Option<SharedString>,
        status: Option<AnyElement>,
        selected: bool,
        enabled: bool,
    }

    impl ChoiceTile {
        /// The choice at `index` of a vertical group of `count`.
        pub fn new(
            id: impl Into<ElementId>,
            index: usize,
            count: usize,
            title: impl Into<SharedString>,
            selected: bool,
        ) -> Self {
            Self {
                id: id.into(),
                index,
                count,
                outer: list_corners(index, count),
                icon: None,
                tinted: None,
                title: title.into(),
                description: None,
                status: None,
                selected,
                enabled: true,
            }
        }

        /// Which corners are outside the group, when the tile is not simply
        /// one of a vertical list (e.g. choices hanging below another tile).
        pub fn corners(mut self, outer: OuterCorners) -> Self {
            self.outer = outer;
            self
        }

        pub fn icon(mut self, icon: impl IntoElement) -> Self {
            self.icon = Some(icon.into_any_element());
            self
        }

        /// A category icon in its tinted circle. While the tile is selected
        /// the circle turns to the surface, so it stays distinct from the
        /// secondary container behind it.
        pub fn tinted_icon(mut self, icon: Icon, tint: Tint, size: Pixels) -> Self {
            self.tinted = Some((icon, tint, size));
            self
        }

        pub fn description(mut self, description: impl Into<SharedString>) -> Self {
            let description = description.into();
            self.description = (!description.is_empty()).then_some(description);
            self
        }

        pub fn status(mut self, status: Option<impl IntoElement>) -> Self {
            self.status = status.map(IntoElement::into_any_element);
            self
        }

        pub fn enabled(mut self, enabled: bool) -> Self {
            self.enabled = enabled;
            self
        }

        /// Resolve the tile for this frame. A plain method rather than
        /// `RenderOnce`, so callers can still chain `on_click` and styles on
        /// the returned button. Outside a [`ChoiceGroup`] the tile has no
        /// arrow-key navigation.
        pub fn render(self, window: &mut Window, cx: &mut App) -> RingedButton {
            let id = self.id.clone();
            let selected = self.selected;
            let enabled = self.enabled;
            let amount = crate::motion::selection_value(
                (id.clone(), "selection"),
                if selected { 1. } else { 0. },
                window,
                cx,
            );
            let paint = tile_paint(amount, selected, enabled);
            let icon = self.icon.or_else(|| {
                self.tinted.map(|(icon, tint, size)| {
                    tinted_icon(icon, tint, size)
                        .bg(blend(tint.container(), color(ColorRole::SURFACE), amount))
                        .into_any_element()
                })
            });
            let title = self.title;
            let (outer, index, count) = (self.outer, self.index, self.count);
            let description = self.description;
            let status = self.status;
            // Connected tiles sit 2dp apart, so the ring uses the M3 inner
            // offset.
            RingedButton::new(id, RingPlacement::Inner(outer_corner_radii(outer)))
                .map(move |button| {
                    tile_corners(
                        button
                            .selected(selected)
                            .role(Role::RadioButton)
                            .aria_toggled(if selected {
                                gpui::accesskit::Toggled::True
                            } else {
                                gpui::accesskit::Toggled::False
                            })
                            .aria_selected(selected)
                            .aria_position_in_set(index + 1)
                            .aria_size_of_set(count)
                            .accessibility_label(title.clone())
                            .w_full()
                            .min_w_0()
                            .h_auto()
                            .min_h(rem(64.))
                            .justify_start()
                            .gap(rem(16.))
                            .pl(ITEM_INSET)
                            .pr(rem(20.))
                            .py(rem(12.)),
                        outer,
                    )
                    .bg(paint.background)
                    .text_color(paint.label)
                    .when(enabled, |button| {
                        button
                            .cursor_pointer()
                            .hover(move |style| style.bg(paint.hover))
                            .active(move |style| style.bg(paint.press))
                    })
                    .children(icon)
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(rem(2.))
                            .child(
                                div()
                                    .min_w_0()
                                    .whitespace_normal()
                                    .text_size(TEXT_SETTINGS_BODY)
                                    .line_height(rem(22.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(title),
                            )
                            .when_some(description, |column, description| {
                                column.child(
                                    div()
                                        .min_w_0()
                                        .whitespace_normal()
                                        .text_size(TEXT_SETTINGS_AUX)
                                        .line_height(rem(20.))
                                        .text_color(if selected {
                                            Rgba {
                                                a: paint.label.a * 0.8,
                                                ..paint.label
                                            }
                                        } else if enabled {
                                            color(ColorRole::ON_SURFACE_VARIANT)
                                        } else {
                                            state::disabled(color(ColorRole::ON_SURFACE_VARIANT))
                                        })
                                        .child(description),
                                )
                            }),
                    )
                    .children(status)
                    .child(choice_radio(selected, enabled))
                })
                .disabled(!enabled)
        }
    }

    type GroupChange = Rc<dyn Fn(&usize, &mut Window, &mut App)>;

    /// One single-choice question as connected [`ChoiceTile`]s under a section
    /// title: a radio group with roving focus. Tab enters at the selected
    /// choice; arrow keys, Home and End move focus among the available
    /// choices and select them, as a radio group does.
    #[derive(gpui::IntoElement)]
    pub struct ChoiceGroup {
        id: ElementId,
        title: SharedString,
        tiles: Vec<ChoiceTile>,
        on_change: Option<GroupChange>,
    }

    impl ChoiceGroup {
        /// `title` is both the section title and the radio group's name.
        pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
            Self {
                id: id.into(),
                title: title.into(),
                tiles: Vec::new(),
                on_change: None,
            }
        }

        /// Append a choice. Its index and count come from the group.
        pub fn tile(mut self, tile: ChoiceTile) -> Self {
            self.tiles.push(tile);
            self
        }

        /// Called with the index of a newly chosen tile.
        pub fn on_change(mut self, f: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
            self.on_change = Some(Rc::new(f));
            self
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub(super) enum Step {
        Next,
        Previous,
        First,
        Last,
    }

    /// The choice `step` lands on from `current`, among `enabled` choices;
    /// arrows wrap.
    pub(super) fn step_to(enabled: &[bool], current: Option<usize>, step: Step) -> Option<usize> {
        let available: Vec<usize> = (0..enabled.len()).filter(|&i| enabled[i]).collect();
        let (first, last) = (*available.first()?, *available.last()?);
        Some(match (step, current) {
            (Step::First, _) => first,
            (Step::Last, _) => last,
            (Step::Next, None) => first,
            (Step::Previous, None) => last,
            (Step::Next, Some(current)) => available
                .iter()
                .copied()
                .find(|&i| i > current)
                .unwrap_or(first),
            (Step::Previous, Some(current)) => available
                .iter()
                .rev()
                .copied()
                .find(|&i| i < current)
                .unwrap_or(last),
        })
    }

    /// The single tab stop of the group: the selected choice when it is
    /// available, otherwise the first available one.
    pub(super) fn entry(enabled: &[bool], selected: Option<usize>) -> Option<usize> {
        selected
            .filter(|&i| enabled[i])
            .or_else(|| enabled.iter().position(|&e| e))
    }

    #[derive(Clone)]
    struct Navigation {
        focus: Rc<[FocusHandle]>,
        enabled: Rc<[bool]>,
        selected: Option<usize>,
        on_change: Option<GroupChange>,
    }

    impl Navigation {
        fn go(&self, step: Step, window: &mut Window, cx: &mut App) {
            let current = self
                .focus
                .iter()
                .position(|focus| focus.is_focused(window))
                .or(self.selected);
            let Some(index) = step_to(&self.enabled, current, step) else {
                return;
            };
            self.focus[index].focus(window, cx);
            if self.selected != Some(index)
                && let Some(on_change) = &self.on_change
            {
                on_change(&index, window, cx);
            }
        }
    }

    impl RenderOnce for ChoiceGroup {
        fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
            let count = self.tiles.len();
            let handles = window.use_keyed_state((self.id.clone(), "choice-focus"), cx, |_, _| {
                Vec::<FocusHandle>::new()
            });
            let focus: Rc<[FocusHandle]> = handles.update(cx, |handles, cx| {
                // One handle per position: extra ones go with removed tiles.
                handles.truncate(count);
                while handles.len() < count {
                    handles.push(cx.focus_handle());
                }
                handles.as_slice().into()
            });
            let enabled: Rc<[bool]> = self.tiles.iter().map(|tile| tile.enabled).collect();
            let selected = self.tiles.iter().position(|tile| tile.selected);
            let entry = entry(&enabled, selected);
            let navigation = Navigation {
                focus: focus.clone(),
                enabled,
                selected,
                on_change: self.on_change.clone(),
            };
            let mut group = super::super::TileGroup::new(
                SharedString::from(format!("{:?}-title", self.id)),
                self.title.clone(),
            );
            for (index, tile) in self.tiles.into_iter().enumerate() {
                let chosen = tile.selected;
                let tile = ChoiceTile {
                    index,
                    count,
                    outer: list_corners(index, count),
                    ..tile
                };
                let handle = focus[index].clone();
                let on_change = self.on_change.clone();
                let button = tile
                    .render(window, cx)
                    .track_focus(&handle)
                    .tab_stop(entry == Some(index))
                    .on_click(move |_, window, cx| {
                        handle.focus(window, cx);
                        if !chosen && let Some(on_change) = &on_change {
                            on_change(&index, window, cx);
                        }
                    });
                group = group.tile(move |_| button.into_any_element());
            }
            let (next, previous, first, last) = (
                navigation.clone(),
                navigation.clone(),
                navigation.clone(),
                navigation,
            );
            div()
                .id(self.id)
                .role(Role::RadioGroup)
                .aria_label(self.title)
                .key_context("ChoiceGroup")
                .w_full()
                .min_w_0()
                .on_action(move |_: &NextChoice, window, cx| next.go(Step::Next, window, cx))
                .on_action(move |_: &PreviousChoice, window, cx| {
                    previous.go(Step::Previous, window, cx)
                })
                .on_action(move |_: &FirstChoice, window, cx| first.go(Step::First, window, cx))
                .on_action(move |_: &LastChoice, window, cx| last.go(Step::Last, window, cx))
                .child(group)
        }
    }
}

/// Arrow keys, Home and End move within a [`ChoiceGroup`].
pub(crate) fn init(cx: &mut App) {
    use super::button_group::{FirstChoice, LastChoice, NextChoice, PreviousChoice};
    cx.bind_keys([
        KeyBinding::new("right", NextChoice, Some("ChoiceGroup")),
        KeyBinding::new("down", NextChoice, Some("ChoiceGroup")),
        KeyBinding::new("left", PreviousChoice, Some("ChoiceGroup")),
        KeyBinding::new("up", PreviousChoice, Some("ChoiceGroup")),
        KeyBinding::new("home", FirstChoice, Some("ChoiceGroup")),
        KeyBinding::new("end", LastChoice, Some("ChoiceGroup")),
    ]);
}

/// A distinct color family for a category icon: a tonal container role and
/// the content role that pairs with it. The names are the families' usual
/// hues; the exact colors follow the current scheme, so every palette keeps
/// its own ground and contrast.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tint {
    /// Error container.
    Orange,
    /// Primary container.
    Blue,
    /// Secondary container.
    Teal,
    /// Warning container.
    Amber,
    /// Success container.
    Green,
    /// Tertiary fixed.
    Pink,
    /// Tertiary container.
    Purple,
}

impl Tint {
    pub const ALL: [Tint; 7] = [
        Tint::Orange,
        Tint::Blue,
        Tint::Teal,
        Tint::Amber,
        Tint::Green,
        Tint::Pink,
        Tint::Purple,
    ];

    /// The container role and the content role on it.
    pub const fn roles(self) -> (ColorRole, ColorRole) {
        match self {
            Tint::Orange => (ColorRole::ERROR_CONTAINER, ColorRole::ON_ERROR_CONTAINER),
            Tint::Blue => (
                ColorRole::PRIMARY_CONTAINER,
                ColorRole::ON_PRIMARY_CONTAINER,
            ),
            Tint::Teal => (
                ColorRole::SECONDARY_CONTAINER,
                ColorRole::ON_SECONDARY_CONTAINER,
            ),
            Tint::Amber => (ColorRole::WARNING_BG, ColorRole::WARNING),
            Tint::Green => (ColorRole::SUCCESS_BG, ColorRole::SUCCESS),
            Tint::Pink => (ColorRole::TERTIARY_FIXED, ColorRole::ON_TERTIARY_FIXED),
            Tint::Purple => (
                ColorRole::TERTIARY_CONTAINER,
                ColorRole::ON_TERTIARY_CONTAINER,
            ),
        }
    }

    pub fn container(self) -> Rgba {
        color(self.roles().0)
    }

    pub fn content(self) -> Rgba {
        color(self.roles().1)
    }
}

/// A category or resource icon in a round tonal container. Give the size in
/// rems where the circle shares a ruler with text, so both follow the text
/// scale.
pub fn tinted_icon(icon: impl Into<Icon>, tint: Tint, size: impl Into<AbsoluteLength>) -> Div {
    let size = size.into();
    let glyph = match size {
        AbsoluteLength::Pixels(size) => AbsoluteLength::Pixels(size * 0.5),
        AbsoluteLength::Rems(size) => AbsoluteLength::Rems(rems(size.0 * 0.5)),
    };
    div()
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .size(size)
        .rounded_full()
        .bg(tint.container())
        .text_color(tint.content())
        .child(icon.into().size(glyph))
}

#[cfg(test)]
mod tests {
    use super::choice_tile::{Step, entry, step_to, tile_paint};
    use super::*;
    use crate::components::focus::test_rings;
    use core::prelude::v1::test;
    use gpui::{Context, Modifiers, TestAppContext, VisualTestContext};

    #[test]
    fn selection_paints_the_secondary_container_and_keeps_its_state_layers() {
        let rest = tile_paint(0., false, true);
        assert_eq!(rest.background, color(ColorRole::SURFACE));
        assert_eq!(rest.ink, color(ColorRole::ON_SURFACE));
        let chosen = tile_paint(1., true, true);
        assert_eq!(chosen.background, color(ColorRole::SECONDARY_CONTAINER));
        assert_eq!(chosen.ink, color(ColorRole::ON_SECONDARY_CONTAINER));
        for paint in [rest, chosen] {
            assert_eq!(paint.hover, state::hovered(paint.background, paint.ink));
            assert_eq!(paint.press, state::pressed(paint.background, paint.ink));
            assert_eq!(paint.label, paint.ink);
        }
        let unavailable = tile_paint(0., false, false);
        assert_eq!(unavailable.label, state::disabled(unavailable.ink));
    }

    #[test]
    fn arrows_skip_unavailable_choices_and_wrap() {
        let enabled = [true, false, true, true];
        assert_eq!(step_to(&enabled, Some(0), Step::Next), Some(2));
        assert_eq!(step_to(&enabled, Some(3), Step::Next), Some(0));
        assert_eq!(step_to(&enabled, Some(2), Step::Previous), Some(0));
        assert_eq!(step_to(&enabled, Some(0), Step::Previous), Some(3));
        assert_eq!(step_to(&enabled, None, Step::Next), Some(0));
        assert_eq!(step_to(&enabled, None, Step::Last), Some(3));
        assert_eq!(step_to(&[false, false], Some(0), Step::Next), None);
    }

    #[test]
    fn tab_enters_at_the_selected_available_choice() {
        assert_eq!(entry(&[true, true, true], Some(2)), Some(2));
        assert_eq!(entry(&[true, true, false], Some(2)), Some(0));
        assert_eq!(entry(&[false, true], None), Some(1));
    }

    struct GroupHarness {
        selected: usize,
        changes: Vec<usize>,
    }

    impl Render for GroupHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            let mut group = ChoiceGroup::new("engines", "Engine").on_change(cx.listener(
                |this, index: &usize, _, cx| {
                    this.selected = *index;
                    this.changes.push(*index);
                    cx.notify();
                },
            ));
            for (index, title) in ["Local", "Cloud", "Remote"].into_iter().enumerate() {
                group = group.tile(
                    ChoiceTile::new(("engine", index), 0, 0, title, self.selected == index)
                        .enabled(index != 1),
                );
            }
            div().w(px(400.)).child(group)
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }

    #[gpui::test]
    fn a_choice_group_is_a_radio_group_with_arrow_keys(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            crate::components::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| GroupHarness {
            selected: 0,
            changes: Vec::new(),
        });
        draw(cx);
        // The selected choice is the group's only tab stop.
        cx.update(|window, cx| window.focus_next(cx));
        let first = cx.update(|window, cx| window.focused(cx));
        assert!(first.is_some());
        cx.update(|window, cx| window.focus_next(cx));
        assert_eq!(cx.update(|window, cx| window.focused(cx)), first);
        cx.simulate_keystrokes("a");
        draw(cx);
        assert!(
            test_rings::painted("engine").is_some(),
            "keyboard focus shows the ring"
        );
        // Down skips the unavailable choice and selects the next one.
        cx.simulate_keystrokes("down");
        draw(cx);
        view.read_with(cx, |h, _| {
            assert_eq!(h.selected, 2);
            assert_eq!(h.changes, [2]);
        });
        // And wraps back to the first.
        cx.simulate_keystrokes("down");
        draw(cx);
        view.read_with(cx, |h, _| assert_eq!(h.changes, [2, 0]));
    }

    struct TileHarness;
    impl Render for TileHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().w(px(400.)).child(
                ChoiceTile::new("pointer-tile", 0, 1, "Only", false)
                    .render(window, cx)
                    .debug_selector(|| "pointer-tile".into())
                    .on_click(|_, _, _| {}),
            )
        }
    }

    #[gpui::test]
    fn pointer_focus_shows_no_ring(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let (_, cx) = cx.add_window_view(|_, _| TileHarness);
        draw(cx);
        let tile = cx.debug_bounds("pointer-tile").unwrap();
        cx.simulate_click(tile.center(), Modifiers::default());
        draw(cx);
        assert!(test_rings::painted("pointer-tile").is_none());
    }

    #[test]
    fn tints_are_distinct_semantic_role_pairs() {
        let mut containers = Tint::ALL.map(|tint| tint.roles().0 as usize).to_vec();
        containers.sort_unstable();
        containers.dedup();
        assert_eq!(containers.len(), Tint::ALL.len(), "one container per tint");
        for tint in Tint::ALL {
            let (container, content) = tint.roles();
            assert_ne!(container, content);
            assert_eq!(tint.container(), color(container));
            assert_eq!(tint.content(), color(content));
        }
    }
}
