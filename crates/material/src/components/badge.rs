//! M3 badge as a configurable component: the small dot or the large label
//! badge, in any of md3.io's colors, on any corner of the element it marks.
//! Reference: the Figma kit's Badges page (6dp dot at the icon's corner; a
//! 16dp label-small pill 12dp across and 14dp down from it) and
//! <https://m3e.kyralabs.dev/components/badge/>.
//!
//! Showing and hiding fades on the fast effects spring; the dot also grows
//! from its corner on the fast spatial spring (GPUI has no element scale
//! transform, so a large badge, whose text cannot be scaled, only fades).
//! Reduced motion keeps the fade and settles the growth at once.
use super::feedback::{LARGE_BADGE_PADDING, LARGE_BADGE_SIZE, SMALL_BADGE_SIZE};
use super::text::accessible_text;
use crate::color::ColorRole;
use crate::motion::{self, MotionToken};
use crate::theme::color;
use crate::tokens::{shape::RADIUS_FULL, typography::*};
use gpui::{prelude::*, *};

/// A badge's container and label roles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeColor {
    /// The M3 default: error on on-error.
    #[default]
    Error,
    Primary,
    Secondary,
    Tertiary,
    Success,
    Warning,
    Info,
}

impl BadgeColor {
    pub const ALL: [Self; 7] = [
        Self::Error,
        Self::Primary,
        Self::Secondary,
        Self::Tertiary,
        Self::Success,
        Self::Warning,
        Self::Info,
    ];

    /// The (container, label) roles.
    pub const fn roles(self) -> (ColorRole, ColorRole) {
        match self {
            Self::Error => (ColorRole::ERROR, ColorRole::ON_ERROR),
            Self::Primary => (ColorRole::PRIMARY, ColorRole::ON_PRIMARY),
            Self::Secondary => (ColorRole::SECONDARY, ColorRole::ON_SECONDARY),
            Self::Tertiary => (ColorRole::TERTIARY, ColorRole::ON_TERTIARY),
            Self::Success => (ColorRole::SUCCESS, ColorRole::ON_SUCCESS),
            Self::Warning => (ColorRole::WARNING, ColorRole::ON_WARNING),
            Self::Info => (ColorRole::INFO, ColorRole::ON_INFO),
        }
    }
}

/// The corner of the anchored element a badge sits on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgePosition {
    #[default]
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

impl BadgePosition {
    pub const ALL: [Self; 4] = [
        Self::TopRight,
        Self::TopLeft,
        Self::BottomRight,
        Self::BottomLeft,
    ];
    const fn top(self) -> bool {
        matches!(self, Self::TopRight | Self::TopLeft)
    }
    const fn right(self) -> bool {
        matches!(self, Self::TopRight | Self::BottomRight)
    }
}

/// The large badge sits 12dp across from the icon's edge and overhangs it
/// by 2dp (its far edge 14dp in).
const LARGE_ACROSS: f32 = 12.;
const LARGE_OVERHANG: f32 = 2.;

/// The text a large badge shows for `label` with at most `max` as a count:
/// a number above `max` reads "max+"; other labels pass through.
pub fn badge_label(label: &str, max: Option<u32>) -> SharedString {
    match (label.trim().parse::<u64>(), max) {
        (Ok(count), Some(max)) if count > u64::from(max) => format!("{max}+").into(),
        _ => SharedString::from(label.to_owned()),
    }
}

/// An M3 badge.
///
/// [`Badge::small`] is the 6dp dot; [`Badge::large`] a label-small pill
/// around a count or short label ([`Self::max_count`] caps counts as
/// "99+"). With [`Self::anchor`] it wraps the element it marks and sits on
/// [`Self::position`]'s corner of that element's [`Self::inset`] box (the
/// icon inside it); without one it renders alone. [`Self::visible`]
/// animates it in and out. The caller supplies the label.
#[derive(gpui::IntoElement)]
pub struct Badge {
    id: ElementId,
    label: Option<SharedString>,
    max_count: Option<u32>,
    color: BadgeColor,
    position: BadgePosition,
    visible: bool,
    inset: f32,
    anchor: Option<AnyElement>,
}

impl Badge {
    /// The small badge: a dot without text.
    pub fn small(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            max_count: None,
            color: BadgeColor::Error,
            position: BadgePosition::TopRight,
            visible: true,
            inset: 0.,
            anchor: None,
        }
    }

    /// The large badge around `label`, e.g. "3".
    pub fn large(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            label: Some(label.into()),
            ..Self::small(id)
        }
    }

    /// Show counts above `max` as "max+".
    pub fn max_count(mut self, max: u32) -> Self {
        self.max_count = Some(max);
        self
    }

    pub fn color(mut self, color: BadgeColor) -> Self {
        self.color = color;
        self
    }

    pub fn position(mut self, position: BadgePosition) -> Self {
        self.position = position;
        self
    }

    /// Controlled visibility; changes animate.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// How far the marked icon sits inside the anchor, in design px (e.g.
    /// 8 for a 24dp icon in a 40dp icon button).
    pub fn inset(mut self, inset: f32) -> Self {
        self.inset = inset;
        self
    }

    /// The element the badge marks.
    pub fn anchor(mut self, anchor: impl IntoElement) -> Self {
        self.anchor = Some(anchor.into_any_element());
        self
    }

    /// The label as shown.
    pub fn text(&self) -> Option<SharedString> {
        self.label
            .as_deref()
            .map(|label| badge_label(label, self.max_count))
    }
}

impl RenderOnce for Badge {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let text = self.text();
        let (container, ink) = self.color.roles();
        let id = self.id;
        let child = |name: &'static str| ElementId::NamedChild(id.clone().into(), name.into());
        let target = if self.visible { 1. } else { 0. };
        let fade = motion::token_value(child("fade"), target, MotionToken::FastEffects, window, cx)
            .clamp(0., 1.);
        let grow = motion::token_value(child("grow"), target, MotionToken::FastSpatial, window, cx)
            .max(0.);
        let hidden = !self.visible && fade <= 0.;
        let position = self.position;
        let inset = self.inset;
        let badge = match text {
            Some(text) => {
                let scale = TypeRole::LabelSmall.scale();
                accessible_text(child("label"), text)
                    .flex()
                    .flex_shrink_0()
                    .items_center()
                    .justify_center()
                    .h(LARGE_BADGE_SIZE)
                    .min_w(LARGE_BADGE_SIZE)
                    .px(LARGE_BADGE_PADDING)
                    .rounded(RADIUS_FULL)
                    .bg(color(container))
                    .text_color(color(ink))
                    .text_size(scale.font_size)
                    .letter_spacing(scale.letter_spacing)
                    .line_height(scale.line_height)
                    .font_weight(FontWeight(scale.font_weight as f32))
                    .whitespace_nowrap()
                    .when(cfg!(test), |b| b.debug_selector(|| "badge-label".into()))
                    .opacity(fade)
                    .when(self.anchor.is_some(), |badge| {
                        let across = rem(inset + LARGE_ACROSS);
                        let overhang = rem(inset - LARGE_OVERHANG);
                        badge
                            .absolute()
                            .map(|b| {
                                if position.top() {
                                    b.top(overhang)
                                } else {
                                    b.bottom(overhang)
                                }
                            })
                            .map(|b| {
                                if position.right() {
                                    b.left(across)
                                } else {
                                    b.right(across)
                                }
                            })
                    })
                    .into_any_element()
            }
            None => {
                // The dot grows from its corner.
                let size = SMALL_BADGE_SIZE * grow;
                div()
                    .id(child("dot"))
                    .flex_shrink_0()
                    .size(size)
                    .rounded(RADIUS_FULL)
                    .bg(color(container))
                    .when(cfg!(test), |b| b.debug_selector(|| "badge-dot".into()))
                    .opacity(fade)
                    .when(self.anchor.is_some(), |dot| {
                        let edge = rem(inset);
                        dot.absolute()
                            .map(|d| {
                                if position.top() {
                                    d.top(edge)
                                } else {
                                    d.bottom(edge)
                                }
                            })
                            .map(|d| {
                                if position.right() {
                                    d.right(edge)
                                } else {
                                    d.left(edge)
                                }
                            })
                    })
                    .into_any_element()
            }
        };
        let badge = (!hidden).then_some(badge);
        match self.anchor {
            Some(anchor) => div()
                .id(id)
                .relative()
                .flex_shrink_0()
                .child(anchor)
                .children(badge)
                .into_any_element(),
            None => div()
                .id(id)
                .flex_shrink_0()
                .children(badge)
                .into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::{Context, TestAppContext};

    #[test]
    fn counts_above_the_maximum_read_max_plus() {
        assert_eq!(badge_label("8", Some(99)), "8");
        assert_eq!(badge_label("100", Some(99)), "99+");
        assert_eq!(badge_label("10", Some(9)), "9+");
        assert_eq!(badge_label("999", Some(999)), "999");
        assert_eq!(badge_label("1000", Some(999)), "999+");
        // Labels that are not counts pass through.
        assert_eq!(badge_label("New", Some(9)), "New");
        assert_eq!(badge_label("100", None), "100");
    }

    #[test]
    fn every_color_pairs_a_container_with_its_on_role() {
        assert_eq!(
            BadgeColor::default().roles(),
            (ColorRole::ERROR, ColorRole::ON_ERROR)
        );
        let mut containers: Vec<_> = BadgeColor::ALL.iter().map(|c| c.roles().0).collect();
        containers.dedup();
        assert_eq!(containers.len(), BadgeColor::ALL.len());
        assert_eq!(
            BadgeColor::Success.roles(),
            (ColorRole::SUCCESS, ColorRole::ON_SUCCESS)
        );
    }

    struct Harness {
        position: BadgePosition,
        large: bool,
        visible: bool,
    }

    impl Render for Harness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(BASE_PX));
            let badge = if self.large {
                Badge::large("badge", "1234").max_count(999)
            } else {
                Badge::small("badge")
            };
            div().flex().items_start().p(px(100.)).child(
                badge
                    .position(self.position)
                    .visible(self.visible)
                    .anchor(div().debug_selector(|| "icon".into()).size(px(24.))),
            )
        }
    }

    #[gpui::test]
    fn the_badge_sits_on_the_requested_corner_and_hides(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| Harness {
            position: BadgePosition::TopRight,
            large: true,
            visible: true,
        });
        let draw = |cx: &mut gpui::VisualTestContext| {
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
            cx.run_until_parked();
        };
        draw(cx);
        let icon = cx.debug_bounds("icon").unwrap();
        let label = cx
            .debug_bounds("badge-label")
            .expect("a large badge's label");
        assert_eq!(label.left(), icon.left() + px(LARGE_ACROSS));
        assert_eq!(label.top(), icon.top() - px(LARGE_OVERHANG));
        view.update(cx, |h, cx| {
            h.position = BadgePosition::BottomLeft;
            cx.notify();
        });
        draw(cx);
        let label = cx.debug_bounds("badge-label").unwrap();
        assert_eq!(label.right(), icon.right() - px(LARGE_ACROSS));
        assert_eq!(label.bottom(), icon.bottom() + px(LARGE_OVERHANG));
        view.update(cx, |h, cx| {
            h.visible = false;
            cx.notify();
        });
        for _ in 0..3 {
            cx.executor()
                .advance_clock(std::time::Duration::from_secs(1));
            draw(cx);
        }
        assert!(cx.debug_bounds("badge-label").is_none(), "hidden");
        // A dot sits in the corner itself.
        view.update(cx, |h, cx| {
            h.large = false;
            h.visible = true;
            h.position = BadgePosition::TopRight;
            cx.notify();
        });
        for _ in 0..3 {
            cx.executor()
                .advance_clock(std::time::Duration::from_secs(1));
            draw(cx);
        }
        let dot = cx.debug_bounds("badge-dot").expect("a shown dot");
        assert_eq!(dot.size, size(SMALL_BADGE_SIZE, SMALL_BADGE_SIZE));
        assert_eq!((dot.right(), dot.top()), (icon.right(), icon.top()));
    }
}
