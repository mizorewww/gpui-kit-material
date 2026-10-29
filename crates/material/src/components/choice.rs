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

pub use choice_tile::ChoiceTile;
mod choice_tile {
    use super::*;
    use gpui_base::v_flex;

    /// One choice in a connected group of tiles: tonal
    /// selection on the secondary container, a leading icon, the label with
    /// optional supporting text, a trailing status chip and a radio. An
    /// unavailable choice keeps its status chip at full strength.
    pub struct ChoiceTile {
        id: ElementId,
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
        /// the returned button.
        pub fn render(self, window: &mut Window, cx: &mut App) -> gpui_base::Button {
            let id = self.id;
            let selected = self.selected;
            let enabled = self.enabled;
            let amount = crate::motion::selection_value(
                SharedString::from(format!("choice-tile-{id:?}")),
                if selected { 1. } else { 0. },
                window,
                cx,
            );
            let bg = blend(
                color(ColorRole::SURFACE),
                color(ColorRole::SECONDARY_CONTAINER),
                amount,
            );
            let ink = if selected {
                color(ColorRole::ON_SECONDARY_CONTAINER)
            } else {
                color(ColorRole::INK)
            };
            let fg = if enabled {
                ink
            } else {
                Rgba { a: 0.38, ..ink }
            };
            let icon = self.icon.or_else(|| {
                self.tinted.map(|(icon, tint, size)| {
                    tinted_icon(icon, tint, size)
                        .bg(blend(tint.container(), color(ColorRole::SURFACE), amount))
                        .into_any_element()
                })
            });
            let title = self.title;
            tile_corners(
                gpui_base::Button::new(id.clone())
                    .selected(selected)
                    .disabled(!enabled)
                    .aria_toggled(if selected {
                        gpui::accesskit::Toggled::True
                    } else {
                        gpui::accesskit::Toggled::False
                    })
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
                self.outer,
            )
            .bg(bg)
            // The outline is reserved for keyboard focus, so focus never
            // moves the content.
            .border_2()
            .border_color(Rgba { a: 0., ..bg })
            .text_color(fg)
            .when(enabled, |button| {
                button
                    .cursor_pointer()
                    .hover(move |style| style.bg(state::hovered(bg, ink)))
                    .active(move |style| style.bg(state::pressed(bg, ink)))
            })
            .focus(|style| style.border_color(color(ColorRole::ACCENT)))
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
                    .when_some(self.description, |column, description| {
                        column.child(
                            div()
                                .min_w_0()
                                .whitespace_normal()
                                .text_size(TEXT_SETTINGS_AUX)
                                .line_height(rem(20.))
                                .text_color(if selected {
                                    Rgba { a: 0.8, ..fg }
                                } else {
                                    color(ColorRole::GRAY)
                                })
                                .child(description),
                        )
                    }),
            )
            .children(self.status)
            .child(choice_radio(selected, enabled))
        }
    }
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
    use super::*;
    use core::prelude::v1::test;

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
