//! Extra pointer geometry for the reference XS/S pseudo-element.
use gpui::{Bounds, ContentMask, Hitbox, HitboxBehavior, Pixels, Style, Window, point, px, size};
use std::{cell::Cell, rc::Rc};

use crate::tokens::typography::rem;

/// Prepaint-resolved target shared by Base and its passive visual observers.
pub(super) type SharedTarget = Rc<Cell<Option<ResolvedTarget>>>;

#[derive(Clone, Copy)]
pub(super) struct ResolvedTarget {
    bounds: Bounds<Pixels>,
    additional: Option<Bounds<Pixels>>,
    mask: ContentMask<Pixels>,
}

impl ResolvedTarget {
    pub(super) fn new(
        bounds: Bounds<Pixels>,
        additional: Option<Bounds<Pixels>>,
        window: &Window,
    ) -> Self {
        Self {
            bounds,
            additional,
            mask: window.content_mask(),
        }
    }

    pub(super) fn insert(self, window: &mut Window) -> Hitbox {
        window.with_content_mask(Some(self.mask), |window| {
            window.insert_hitbox_with_extension(
                self.bounds,
                self.additional,
                HitboxBehavior::Normal,
            )
        })
    }
}

/// The caller enables this only for XS/S. Absolute pseudo-elements use the
/// padding box: asymmetric borders shift its center as well as reduce its width.
/// Returning a second rectangle preserves the original button border hit area.
pub(super) fn extra_bounds(
    bounds: Bounds<Pixels>,
    style: &Style,
    window: &Window,
) -> Option<Bounds<Pixels>> {
    let borders = style.border_widths.to_pixels(window.rem_size());
    let width = bounds.size.width - borders.left - borders.right;
    let padding_height = bounds.size.height - borders.top - borders.bottom;
    if width <= px(0.) || padding_height < px(0.) {
        return None;
    }
    let height = rem(48.).to_pixels(window.rem_size());
    Some(Bounds::new(
        point(
            bounds.origin.x + borders.left,
            bounds.origin.y + borders.top + (padding_height - height) / 2.,
        ),
        size(width, height),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Context, Edges, IntoElement, Render, TestAppContext, div};

    struct Empty;
    impl Render for Empty {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }

    #[gpui::test]
    fn target_uses_padding_box_center_and_tracks_text_scale(cx: &mut TestAppContext) {
        let (_, cx) = cx.add_window_view(|_, _| Empty);
        cx.update(|window, _| {
            window.set_rem_size(px(14.));
            let bounds = Bounds::new(point(px(100.), px(100.)), size(px(80.), px(32.)));
            let style = Style {
                border_widths: Edges {
                    top: px(1.).into(),
                    right: px(2.).into(),
                    bottom: px(3.).into(),
                    left: px(4.).into(),
                },
                ..Default::default()
            };
            assert_eq!(
                extra_bounds(bounds, &style, window),
                Some(Bounds::new(
                    point(px(104.), px(91.)),
                    size(px(74.), px(48.))
                )),
            );
            window.set_rem_size(px(28.));
            assert_eq!(
                extra_bounds(bounds, &Style::default(), window)
                    .unwrap()
                    .size
                    .height,
                px(96.),
            );
        });
    }
}
