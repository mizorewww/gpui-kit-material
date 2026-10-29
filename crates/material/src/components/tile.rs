//! Connected tiles: M3 lists whose items are separate surfaces with
//! connected corners, as settings and option panels use them.
use crate::color::ColorRole;
use crate::icons::{Icon, IconName};
use crate::theme::color;
use crate::tokens::{layout::*, shape::*, state, typography::*};
use gpui::{prelude::*, *};

use super::focus::{RingPlacement, RingedButton};
use super::text::accessible_text;

/// One tile of a connected group: the surface with [`tile_corners`].
pub fn tile(outer: OuterCorners) -> Div {
    tile_corners(
        div().w_full().min_w_0().bg(color(ColorRole::SURFACE)),
        outer,
    )
}

/// The corner radii [`tile_corners`] gives a tile with `outer` corners.
pub(crate) fn outer_corner_radii(outer: OuterCorners) -> Corners<Pixels> {
    let radius = |outside: bool| {
        if outside {
            TILE_RADIUS
        } else {
            TILE_INNER_RADIUS
        }
    };
    Corners {
        top_left: radius(outer[0]),
        top_right: radius(outer[1]),
        bottom_right: radius(outer[2]),
        bottom_left: radius(outer[3]),
    }
}

/// The tile at `index` of a vertical connected list of `count` tiles.
pub fn list_tile(index: usize, count: usize) -> Div {
    tile(list_corners(index, count))
}

/// The text scale of a tile's content. Both scales share one composition;
/// settings read one step larger than denser option panels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileScale {
    /// Settings pages: body-large (16px) labels over 14px supporting text,
    /// the M3 list item headline and supporting roles.
    Regular,
    /// Denser panels: 16px labels over 13px supporting text.
    Compact,
}

impl TileScale {
    pub fn label_size(self) -> Rems {
        match self {
            TileScale::Regular => TypeRole::BodyLarge.scale().font_size,
            TileScale::Compact => rem(16.),
        }
    }

    pub fn support_size(self) -> Rems {
        match self {
            TileScale::Regular => TEXT_SETTINGS_AUX,
            TileScale::Compact => rem(13.),
        }
    }

    /// The label's line box. Tiles that act are Buttons, whose text box is
    /// exactly the font size; an explicit line height keeps wrapped and
    /// clamped lines (and their descenders) inside it at every text scale.
    pub fn label_line_height(self) -> Rems {
        match self {
            TileScale::Regular => rem(24.),
            TileScale::Compact => rem(22.),
        }
    }

    pub fn support_line_height(self) -> Rems {
        match self {
            TileScale::Regular => rem(20.),
            TileScale::Compact => rem(18.),
        }
    }

    /// A tile's main label, medium weight.
    pub fn label(
        self,
        id: impl Into<gpui::ElementId>,
        value: impl Into<gpui::SharedString>,
    ) -> gpui::Stateful<gpui::Div> {
        use gpui::prelude::*;
        accessible_text(id, value)
            .text_size(self.label_size())
            .line_height(self.label_line_height())
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(color(ColorRole::ON_SURFACE))
    }

    /// Supporting text under a tile's label, regular gray.
    pub fn support(
        self,
        id: impl Into<gpui::ElementId>,
        value: impl Into<gpui::SharedString>,
    ) -> gpui::Stateful<gpui::Div> {
        use gpui::prelude::*;
        accessible_text(id, value)
            .min_w_0()
            .whitespace_normal()
            .text_size(self.support_size())
            .line_height(self.support_line_height())
            .font_weight(gpui::FontWeight::NORMAL)
            .text_color(color(ColorRole::ON_SURFACE_VARIANT))
    }

    /// One content tile of a connected group at this scale: the surface,
    /// its corners, and the reading size settings content inherits.
    pub fn surface(self, outer: OuterCorners) -> gpui::Div {
        use gpui::prelude::*;
        tile(outer)
            .flex()
            .flex_col()
            .flex_shrink_0()
            .when(self == TileScale::Regular, |tile| {
                tile.text_size(TEXT_SETTINGS_BODY)
            })
    }

    /// A whole tile that acts: the surface with M3 state layers, the tile's
    /// corners and this scale's padding. Callers add one [`TileScale::row`]
    /// holding the leading slot, text column and trailing items.
    pub fn button(self, id: impl Into<ElementId>, outer: OuterCorners) -> RingedButton {
        let (bg, ink) = (color(ColorRole::SURFACE), color(ColorRole::ON_SURFACE));
        let scale = self;
        // Connected tiles sit 2dp apart, so the focus ring uses the M3 inner
        // offset: an outer ring would be covered by the next tile.
        RingedButton::new(id, RingPlacement::Inner(outer_corner_radii(outer)))
            .map(move |button| {
                let button = button
                    .w_full()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .justify_start()
                    .text_left()
                    .whitespace_normal()
                    .bg(bg)
                    .text_color(ink)
                    .cursor_pointer();
                let button = match scale {
                    TileScale::Regular => button.min_h(rem(64.)).px(ITEM_INSET).py(rem(16.)),
                    TileScale::Compact => button
                        .min_h(rem(68.))
                        .pl(ITEM_INSET)
                        .pr(rem(16.))
                        .py(rem(12.)),
                };
                tile_corners(button, outer)
            })
            .state_layers(state::hovered(bg, ink), state::pressed(bg, ink))
    }

    /// The gap between a tile's leading slot, text column and trailing items.
    pub fn gap(self) -> Rems {
        match self {
            TileScale::Regular => rem(16.),
            TileScale::Compact => rem(12.),
        }
    }

    /// The content of a `button` tile: its leading slot, text column and
    /// trailing items in one row at this scale's gap. The component's own
    /// content row has a fixed 8px gap and does not wrap, so a tile's
    /// children go in here to keep text on the rows' ruler and let long
    /// labels wrap.
    pub fn row(self) -> gpui::Div {
        use gpui::prelude::*;
        gpui_base::h_flex()
            .flex_1()
            .w_full()
            .min_w_0()
            .items_center()
            .whitespace_normal()
            .gap(self.gap())
    }

    /// A tile's text column: its label and, when present, the value or
    /// explanation under it.
    pub fn text(
        self,
        id: impl Into<gpui::ElementId>,
        label: impl Into<gpui::SharedString>,
        support: Option<gpui::SharedString>,
    ) -> gpui::Div {
        use gpui::prelude::*;
        let id = id.into();
        gpui_base::v_flex()
            .flex_1()
            .min_w_0()
            .gap(rem(2.))
            .child(self.label(id.clone(), label))
            .when_some(support, |column, support| {
                column.child(
                    self.support(gpui::SharedString::from(format!("{id:?}-support")), support),
                )
            })
    }
}

pub use tile_group::TileGroup;
mod tile_group {
    use super::TileScale;
    use crate::tokens::{layout::*, shape::*};
    use gpui::{
        AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    };
    use gpui_base::v_flex;

    /// Builds a tile's content once the group knows which corners are outside.
    type TileBuilder = Box<dyn FnOnce(OuterCorners) -> AnyElement>;

    enum TileMain {
        /// Content on the plain surface; the group draws the tile.
        Content(AnyElement),
        /// A tile that draws itself (a button), given its corners.
        Built(TileBuilder),
        /// Two tiles side by side in one row of the group.
        Pair(TileBuilder, TileBuilder),
    }

    struct GroupTile {
        main: TileMain,
        details: Vec<AnyElement>,
    }

    /// One section: a quiet title above a connected group of tiles, each
    /// item on its own surface with 24px outer and 6px inner corners and 2px
    /// between tiles, at the group's [`TileScale`].
    #[derive(gpui::IntoElement)]
    pub struct TileGroup {
        id: SharedString,
        title: SharedString,
        scale: TileScale,
        tiles: Vec<GroupTile>,
    }

    impl TileGroup {
        pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
            Self {
                id: id.into(),
                title: title.into(),
                scale: TileScale::Regular,
                tiles: Vec::new(),
            }
        }

        /// The text scale of the group's content tiles.
        pub fn scale(mut self, scale: TileScale) -> Self {
            self.scale = scale;
            self
        }

        /// A tile that draws its own surface (a button row), given its corners
        /// as top-left, top-right, bottom-right, bottom-left (true = outside).
        pub fn tile(mut self, build: impl FnOnce(OuterCorners) -> AnyElement + 'static) -> Self {
            self.tiles.push(GroupTile {
                main: TileMain::Built(Box::new(build)),
                details: Vec::new(),
            });
            self
        }

        /// Two tiles sharing one row of the group.
        pub fn pair(
            mut self,
            leading: impl FnOnce(OuterCorners) -> AnyElement + 'static,
            trailing: impl FnOnce(OuterCorners) -> AnyElement + 'static,
        ) -> Self {
            self.tiles.push(GroupTile {
                main: TileMain::Pair(Box::new(leading), Box::new(trailing)),
                details: Vec::new(),
            });
            self
        }

        /// Content that belongs to the previous item (a disclosure, a summary
        /// line): it stays inside that item's tile.
        pub fn detail(mut self, element: impl IntoElement) -> Self {
            match self.tiles.last_mut() {
                Some(tile) => tile.details.push(element.into_any_element()),
                None => self.tiles.push(GroupTile {
                    main: TileMain::Content(element.into_any_element()),
                    details: Vec::new(),
                }),
            }
            self
        }
    }

    impl ParentElement for TileGroup {
        fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
            self.tiles
                .extend(elements.into_iter().map(|element| GroupTile {
                    main: TileMain::Content(element),
                    details: Vec::new(),
                }));
        }
    }

    impl RenderOnce for TileGroup {
        fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
            let count = self.tiles.len();
            let scale = self.scale;
            let title = super::super::section_title(self.id, self.title).px(ITEM_INSET);
            let mut group = v_flex().w_full().min_w_0().gap(TILE_GAP);
            for (index, tile) in self.tiles.into_iter().enumerate() {
                let first = index == 0;
                let last = index + 1 == count;
                let outer = list_corners(index, count);
                let element = match tile.main {
                    TileMain::Content(content) => scale
                        .surface(outer)
                        .child(content)
                        .children(tile.details)
                        .into_any_element(),
                    TileMain::Built(build) if tile.details.is_empty() => build(outer),
                    // Details continue below the built tile inside the same
                    // surface, so its own bottom corners stay inner.
                    TileMain::Built(build) => scale
                        .surface(outer)
                        .child(build([outer[0], outer[1], false, false]))
                        .children(tile.details)
                        .into_any_element(),
                    TileMain::Pair(leading, trailing) => v_flex()
                        .w_full()
                        .min_w_0()
                        .child(
                            div()
                                .grid()
                                .grid_cols(2)
                                .w_full()
                                .min_w_0()
                                .gap(TILE_GAP)
                                .child(leading([first, false, false, last]))
                                .child(trailing([false, first, last, false])),
                        )
                        .children(tile.details)
                        .into_any_element(),
                };
                group = group.child(element);
            }
            // The title's gap follows its text, so it is in rems; the tiles
            // keep their dp ground between them.
            v_flex()
                .w_full()
                .min_w_0()
                .flex_shrink_0()
                .gap(SECTION_TITLE_SPACE)
                .child(title)
                .child(group)
        }
    }
}

/// An explanation inside a tile: the information icon and supporting text.
pub fn tile_info(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Div {
    gpui_base::h_flex()
        .w_full()
        .min_w_0()
        .items_start()
        .gap(rem(10.))
        .child(
            div()
                .flex()
                .flex_shrink_0()
                .h(rem(20.))
                .items_center()
                .child(
                    IconName::Info
                        .icon()
                        .size(rem(18.))
                        .color(color(ColorRole::ON_SURFACE_VARIANT)),
                ),
        )
        .child(TileScale::Regular.support(id, text).flex_1())
}

/// The trailing chevron of a tile that opens something.
pub fn tile_chevron() -> Icon {
    IconName::ChevronRight
        .icon()
        .size(rem(20.))
        .color(color(ColorRole::ON_SURFACE_VARIANT))
}

/// The chevron of an in-place disclosure; it turns over when open.
pub fn disclosure_chevron(open: bool) -> Icon {
    if open {
        IconName::ChevronUp
    } else {
        IconName::ChevronDown
    }
    .icon()
    .size(rem(20.))
    .color(color(ColorRole::ON_SURFACE_VARIANT))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::focus::test_rings;
    use core::prelude::v1::test;
    use gpui::{Context, Modifiers, TestAppContext, VisualTestContext};

    struct Harness;
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().p(px(20.)).w(px(300.)).child(
                TileScale::Regular
                    .button("ring-tile", [true, true, false, false])
                    .debug_selector(|| "ring-tile".into())
                    .on_click(|_, _, _| {})
                    .child(TileScale::Regular.row().child("Tile")),
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
    fn tile_buttons_show_the_inner_ring_only_for_keyboard_focus(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let (_, cx) = cx.add_window_view(|_, _| Harness);
        draw(cx);
        let tile = cx.debug_bounds("ring-tile").unwrap();
        cx.simulate_click(tile.center(), Modifiers::default());
        draw(cx);
        assert!(test_rings::painted("ring-tile").is_none());
        let keystroke =
            cx.update(|_, cx| gpui::Keystroke::parse_with_os("a", cx.platform_info().os).unwrap());
        cx.simulate_event(gpui::KeyDownEvent {
            keystroke,
            is_held: false,
            prefer_character_input: false,
        });
        draw(cx);
        assert_eq!(test_rings::painted("ring-tile"), Some(tile.inset(px(3.))));
    }

    struct GroupHarness;
    impl Render for GroupHarness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(21.));
            div().w(px(300.)).debug_selector(|| "group".into()).child(
                TileGroup::new("group-title", "Title")
                    .child(div().h(px(30.)).debug_selector(|| "first".into()))
                    .child(div().h(px(30.)).debug_selector(|| "second".into())),
            )
        }
    }

    #[gpui::test]
    fn the_title_gap_follows_the_text_scale_and_tiles_keep_their_ground(cx: &mut TestAppContext) {
        let (_, cx) = cx.add_window_view(|_, _| GroupHarness);
        draw(cx);
        let group = cx.debug_bounds("group").unwrap();
        let first = cx.debug_bounds("first").unwrap();
        let second = cx.debug_bounds("second").unwrap();
        // 150% text scale: a 30px title line, then the 15px title gap.
        assert_eq!(first.top() - group.top(), px(30. + 15.));
        assert_eq!(second.top() - first.bottom(), TILE_GAP);
    }

    #[test]
    fn regular_tile_labels_use_body_large() {
        assert_eq!(
            TileScale::Regular.label_size(),
            TypeRole::BodyLarge.scale().font_size
        );
        assert_eq!(TileScale::Regular.label_line_height(), rem(24.));
        assert_eq!(TileScale::Regular.support_size(), TEXT_SETTINGS_AUX);
    }

    #[test]
    fn ring_corners_match_the_tile_corners() {
        let corners = outer_corner_radii([true, true, false, false]);
        assert_eq!(corners.top_left, TILE_RADIUS);
        assert_eq!(corners.top_right, TILE_RADIUS);
        assert_eq!(corners.bottom_right, TILE_INNER_RADIUS);
        assert_eq!(corners.bottom_left, TILE_INNER_RADIUS);
    }
}
