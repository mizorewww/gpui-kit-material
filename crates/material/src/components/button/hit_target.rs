//! The 48dp target of a compact button (the reference's XS/S pseudo-element): the
//! Base root is the target, its negative margins keep it out of layout, and what the
//! button paints is a child. Hit testing, clipping and occlusion are then GPUI's own.
use gpui::{
    AbsoluteLength, DefiniteLength, Edges, Length, Overflow, Pixels, Rems, StyleRefinement,
};

use crate::tokens::typography::rem;

/// The root's height and margins for a painted height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Target {
    pub height: Pixels,
    pub margin_top: Pixels,
    pub margin_bottom: Pixels,
    overhang: Pixels,
    rem_size: Pixels,
}

fn definite(length: Length, rem_size: Pixels) -> Option<Pixels> {
    match length {
        Length::Definite(DefiniteLength::Absolute(length)) => Some(absolute(length, rem_size)),
        _ => None,
    }
}

fn absolute(length: AbsoluteLength, rem_size: Pixels) -> Pixels {
    length.to_pixels(rem_size)
}

impl Target {
    /// `None` when there is nothing to extend: the button is 48dp already, clips
    /// itself, or has a height or margin only layout can resolve.
    ///
    /// Layout snaps each metric to the device pixel grid on its own, so these are snapped
    /// here, and the height is what the snapped parts add up to: at a fractional rem the
    /// margin box would otherwise differ from the painted height by a device pixel.
    pub fn resolve(
        design_height: f32,
        style: &StyleRefinement,
        rem_size: Pixels,
        snap: impl Fn(Pixels) -> Pixels,
    ) -> Option<Self> {
        let clips = |overflow: Option<Overflow>| overflow.is_some_and(|o| o != Overflow::Visible);
        if clips(style.overflow.x) || clips(style.overflow.y) {
            return None;
        }
        let painted = match style.size.height {
            Some(height) => definite(height, rem_size)?,
            None => rem(design_height).to_pixels(rem_size),
        };
        let painted = snap(painted);
        let overhang = snap((rem(48.).to_pixels(rem_size) - painted) / 2.);
        if overhang <= Pixels::ZERO {
            return None;
        }
        let height = painted + overhang * 2.;
        let margin = |edge: Option<Length>| match edge {
            Some(length) => definite(length, rem_size),
            None => Some(Pixels::ZERO),
        };
        Some(Self {
            height,
            margin_top: snap(margin(style.margin.top)?) - overhang,
            margin_bottom: snap(margin(style.margin.bottom)?) - overhang,
            overhang,
            rem_size,
        })
    }

    /// How far the target reaches past the painted surface.
    pub fn extension(&self) -> Edges<Rems> {
        let reach = Rems(self.overhang / self.rem_size);
        Edges {
            top: reach,
            bottom: reach,
            left: Rems(0.),
            right: Rems(0.),
        }
    }
}

/// Moves what places the button from its painted style to its root's. The height stays
/// behind in the result for [`height_of`]; `Target` already accounts for it.
pub(super) fn take_layout(style: &mut StyleRefinement) -> StyleRefinement {
    let mut layout = StyleRefinement::default();
    layout.display = style.display.take();
    layout.visibility = style.visibility.take();
    layout.position = style.position.take();
    layout.inset = std::mem::take(&mut style.inset);
    layout.margin = std::mem::take(&mut style.margin);
    layout.margin.top = None;
    layout.margin.bottom = None;
    layout.flex_basis = style.flex_basis.take();
    layout.flex_grow = style.flex_grow.take();
    layout.flex_shrink = style.flex_shrink.take();
    layout.align_self = style.align_self.take();
    layout.size.width = style.size.width.take();
    layout.min_size.width = style.min_size.width.take();
    layout.max_size.width = style.max_size.width.take();
    layout.size.height = style.size.height.take();
    layout.min_size.height = style.min_size.height.take();
    layout.max_size.height = style.max_size.height.take();
    layout
}

/// The heights [`take_layout`] moved, for the surface they size. The root keeps the
/// target's own height, which callers of this set after refining with the layout.
pub(super) fn height_of(layout: &StyleRefinement) -> StyleRefinement {
    let mut heights = StyleRefinement::default();
    heights.size.height = layout.size.height;
    heights.min_size.height = layout.min_size.height;
    heights.max_size.height = layout.max_size.height;
    heights
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Styled as _, div, px};

    fn refinement(build: impl FnOnce(gpui::Div) -> gpui::Div) -> StyleRefinement {
        let mut element = build(div());
        element.style().clone()
    }

    #[test]
    fn the_target_is_48dp_around_the_painted_height_at_any_text_scale() {
        let plain = StyleRefinement::default();
        for (rem_size, scale) in [(px(14.), 1.), (px(28.), 2.)] {
            for (design, overhang) in [(32., 8.), (40., 4.)] {
                let target = Target::resolve(design, &plain, rem_size, |p| p).unwrap();
                assert_eq!(target.height, px(48. * scale));
                assert_eq!(target.margin_top, px(-overhang * scale));
                assert_eq!(target.margin_bottom, px(-overhang * scale));
            }
            assert_eq!(Target::resolve(48., &plain, rem_size, |p| p), None);
            assert_eq!(Target::resolve(56., &plain, rem_size, |p| p), None);
        }
    }

    #[test]
    fn a_caller_height_margin_or_clip_is_respected() {
        let rem_size = px(14.);
        let dense = refinement(|b| b.h(px(30.)));
        assert_eq!(
            Target::resolve(40., &dense, rem_size, |p| p)
                .unwrap()
                .margin_top,
            px(-9.)
        );
        let margin = refinement(|b| b.mt(px(6.)).mb(px(2.)));
        let target = Target::resolve(32., &margin, rem_size, |p| p).unwrap();
        assert_eq!(target.margin_top, px(-2.));
        assert_eq!(target.margin_bottom, px(-6.));
        assert_eq!(
            Target::resolve(32., &refinement(|b| b.overflow_hidden()), rem_size, |p| p),
            None
        );
        assert_eq!(
            Target::resolve(32., &refinement(|b| b.h_full()), rem_size, |p| p),
            None
        );
    }
}

#[cfg(test)]
mod layout_tests {
    use super::super::{ButtonSize, ButtonVariant, MaterialButton};
    use gpui::{
        Context, IntoElement, Modifiers, Pixels, Render, ScrollHandle, TestAppContext,
        VisualTestContext, Window, div, point, prelude::*, px,
    };
    use std::{cell::Cell, rc::Rc};

    struct Column {
        size: ButtonSize,
        clip: bool,
        cover: bool,
        disabled: bool,
        clicks: Rc<Cell<usize>>,
        covers: Rc<Cell<usize>>,
    }
    impl Render for Column {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            let clicks = self.clicks.clone();
            let covers = self.covers.clone();
            div()
                .relative()
                .size_full()
                .p(px(40.))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_start()
                        .debug_selector(|| "column".into())
                        .when(self.clip, |column| column.overflow_hidden())
                        .child(
                            div()
                                .h(px(10.))
                                .w(px(100.))
                                .debug_selector(|| "above".into()),
                        )
                        .child(
                            MaterialButton::new("compact")
                                .button_size(self.size)
                                .variant(ButtonVariant::Outlined)
                                .disabled(self.disabled)
                                .debug_selector(|| "compact".into())
                                .child("Label")
                                .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
                        )
                        .child(
                            div()
                                .h(px(10.))
                                .w(px(100.))
                                .debug_selector(|| "below".into()),
                        ),
                )
                .when(self.cover, |root| {
                    root.child(
                        div()
                            .id("cover")
                            .absolute()
                            .left_0()
                            .top(px(0.))
                            .w(px(300.))
                            .h(px(48.))
                            .block_mouse_except_scroll()
                            .on_click(move |_, _, _| covers.set(covers.get() + 1)),
                    )
                })
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        size: ButtonSize,
    ) -> (gpui::Entity<Column>, &mut VisualTestContext) {
        cx.update(gpui_base::init);
        let (view, cx) = cx.add_window_view(|_, _| Column {
            size,
            clip: false,
            cover: false,
            disabled: false,
            clicks: Default::default(),
            covers: Default::default(),
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        (view, cx)
    }

    fn redraw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }

    #[gpui::test]
    fn compact_buttons_keep_their_layout_and_a_48dp_target(cx: &mut TestAppContext) {
        for (size, painted) in [(ButtonSize::ExtraSmall, 32.), (ButtonSize::Small, 40.)] {
            let (view, cx) = setup(cx, size);
            let button = cx.debug_bounds("compact").unwrap();
            let above = cx.debug_bounds("above").unwrap();
            let below = cx.debug_bounds("below").unwrap();
            let column = cx.debug_bounds("column").unwrap();
            assert_eq!(button.size.height, px(painted));
            assert_eq!(button.origin.y, above.origin.y + px(10.), "no space above");
            assert_eq!(
                below.origin.y,
                button.origin.y + px(painted),
                "no space below"
            );
            assert_eq!(
                column.size.height,
                px(painted + 20.),
                "the parent does not grow"
            );
            let reach = (48. - painted) / 2.;
            let x = button.center().x;
            let clicks = view.read_with(cx, |view, _| view.clicks.clone());
            for (y, expected, why) in [
                (
                    button.origin.y - px(reach - 1.),
                    1,
                    "inside the upper strip",
                ),
                (
                    button.bottom() + px(reach - 1.),
                    2,
                    "inside the lower strip",
                ),
                (button.center().y, 3, "on the surface"),
                (button.origin.y - px(reach + 1.), 3, "past the upper strip"),
                (button.bottom() + px(reach + 1.), 3, "past the lower strip"),
            ] {
                cx.simulate_click(point(x, y), Modifiers::default());
                assert_eq!(clicks.get(), expected, "{why}");
            }
        }
    }

    #[gpui::test]
    fn the_target_obeys_clipping_occlusion_and_disabled(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, ButtonSize::ExtraSmall);
        let button = cx.debug_bounds("compact").unwrap();
        let edge = point(button.center().x, button.origin.y - px(4.));
        let (clicks, covers) =
            view.read_with(cx, |view, _| (view.clicks.clone(), view.covers.clone()));
        view.update(cx, |view, _| view.clip = true);
        redraw(cx);
        // The strip above the first child is outside a clipping column.
        let above_column = point(
            button.center().x,
            cx.debug_bounds("column").unwrap().origin.y - px(1.),
        );
        cx.simulate_click(above_column, Modifiers::default());
        assert_eq!(clicks.get(), 0, "an ancestor's clip cuts the target");
        view.update(cx, |view, _| {
            view.clip = false;
            view.cover = true;
        });
        redraw(cx);
        let covered = point(button.center().x, px(44.));
        cx.simulate_click(covered, Modifiers::default());
        assert_eq!(
            (clicks.get(), covers.get()),
            (0, 1),
            "a later element occludes it"
        );
        view.update(cx, |view, _| {
            view.cover = false;
            view.disabled = true;
        });
        redraw(cx);
        cx.simulate_click(edge, Modifiers::default());
        assert_eq!(clicks.get(), 0, "a disabled button has no target");
    }

    struct Scroller {
        scroll: ScrollHandle,
        size: ButtonSize,
    }
    impl Render for Scroller {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div()
                .id("scroller")
                .track_scroll(&self.scroll)
                .overflow_y_scroll()
                .w(px(200.))
                .h(px(100.))
                .child(div().h(px(100.) - px(32.)))
                .child(
                    MaterialButton::new("last")
                        .button_size(self.size)
                        .child("Label"),
                )
        }
    }

    /// The target's lower strip overhangs a scroller's last row, and GPUI's scroll range
    /// follows children's bounds, so the range grows by the overhang at most.
    #[gpui::test]
    fn a_scrollers_range_grows_at_most_by_the_overhang(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let scroll = ScrollHandle::new();
        let (_, cx) = cx.add_window_view({
            let scroll = scroll.clone();
            move |_, _| Scroller {
                scroll,
                size: ButtonSize::ExtraSmall,
            }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let range: Pixels = scroll.max_offset().y;
        assert!(range <= px(8.), "{range:?}");
    }
}
