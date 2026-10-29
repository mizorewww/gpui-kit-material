//! M3 carousel: multi-browse, uncontained, hero, center-aligned hero and
//! full-screen layouts whose items change size as they scroll along
//! keylines, snapping to items, with keyboard scrolling.
//!
//! Visuals follow the Figma "Material 3 Design Kit" `Carousel`: items are
//! surface-container-high with 28dp corners, 8dp apart inside 8/16dp
//! padding, with a title-medium label and label-small supporting text over
//! a scrim gradient at the bottom of large items. Small items are 56dp
//! (M3: 40–56dp).
//!
//! The layout follows M3's keyline model (as Material Components' carousel
//! implements it): a strategy turns the viewport width into a row of
//! keylines (each a position and an item width — large, medium, small).
//! Every item has the same extent in scroll space; its on-screen position
//! and width interpolate between the keylines it is passing, so an item
//! grows from small through medium to large as it scrolls in and shrinks as
//! it leaves. Content is laid out at the large width and masked by the item,
//! so it stays centered while the mask morphs. Near the end, the keylines
//! shift to the mirrored arrangement so the last items become large.
//!
//! Scrolling is driven by the wheel or trackpad and, while the carousel is
//! focused, Left/Right (one item), Home/End. When scrolling rests, the
//! carousel snaps to the nearest item on the default spatial curve;
//! reduced motion snaps at once.
use crate::{
    color::ColorRole,
    motion::{self, MotionToken},
    theme::color,
    tokens::typography::{TypeRole, rem},
};
use gpui::{prelude::*, *};
use std::time::Duration;

#[cfg(not(target_family = "wasm"))]
use std::time::Instant;
#[cfg(target_family = "wasm")]
use web_time::Instant;

/// Item corners.
pub const CAROUSEL_ITEM_RADIUS: Pixels = px(28.);
/// The smallest item a keyline strategy shows.
pub const CAROUSEL_SMALL_ITEM: f32 = 56.;
const GAP: f32 = 8.;
const PAD_X: f32 = 16.;
const PAD_Y: f32 = 8.;
/// How long scrolling must rest before the carousel snaps.
const SNAP_IDLE: Duration = Duration::from_millis(160);

/// A carousel's keyline strategy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarouselLayout {
    /// Large items, then one medium and one small item.
    #[default]
    MultiBrowse,
    /// Items keep one size and run off the trailing edge.
    Uncontained,
    /// One large item and a small one at the trailing edge.
    Hero,
    /// One large item between two small ones.
    CenteredHero,
    /// One item fills the carousel; no padding or gaps.
    FullScreen,
}

/// One keyline: an item's left edge and width on screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keyline {
    pub x: f32,
    pub width: f32,
}

/// The keylines of one arrangement plus the number of focal (large) items.
#[derive(Clone, Debug, PartialEq)]
pub struct Keylines {
    pub lines: Vec<Keyline>,
    pub focal: usize,
    /// Scroll-space extent of one item (large width plus the gap).
    pub step: f32,
}

fn lay_out(widths: &[f32], start: f32, gap: f32) -> Vec<Keyline> {
    let mut x = start;
    widths
        .iter()
        .map(|&width| {
            let line = Keyline { x, width };
            x += width + gap;
            line
        })
        .collect()
}

/// The start-aligned keylines of `layout` in a carousel `width` wide whose
/// large items are `large` wide.
pub(crate) fn keylines(layout: CarouselLayout, width: f32, large: f32) -> Keylines {
    let (pad, gap) = if layout == CarouselLayout::FullScreen {
        (0., 0.)
    } else {
        (PAD_X, GAP)
    };
    let available = (width - 2. * pad).max(CAROUSEL_SMALL_ITEM);
    let small = CAROUSEL_SMALL_ITEM;
    let (widths, focal, large) = match layout {
        CarouselLayout::FullScreen => (vec![available], 1, available),
        CarouselLayout::Hero => {
            let large = (available - small - gap).max(small);
            (vec![large, small], 1, large)
        }
        CarouselLayout::CenteredHero => {
            let large = (available - 2. * (small + gap)).max(small);
            (vec![small, large, small], 1, large)
        }
        CarouselLayout::Uncontained => {
            let large = large.min(available);
            let count = ((available + gap) / (large + gap)).ceil().max(1.) as usize;
            (vec![large; count], count, large)
        }
        CarouselLayout::MultiBrowse => {
            let large = large.min(available).max(small);
            // As many large items as leave room for a medium (at least
            // small) and a small item.
            let mut count = ((available - 2. * (small + gap) + gap) / (large + gap))
                .floor()
                .max(1.) as usize;
            let medium = |count: usize| available - count as f32 * (large + gap) - small - gap;
            while count > 1 && medium(count) < small {
                count -= 1;
            }
            let medium = medium(count).clamp(small, large);
            let mut widths = vec![large; count];
            widths.extend([medium, small]);
            (widths, count, large)
        }
    };
    Keylines {
        lines: lay_out(&widths, pad, gap),
        focal,
        step: large + gap,
    }
}

/// The mirrored arrangement used at the end of the scroll range.
fn end_keylines(start: &Keylines, layout: CarouselLayout) -> Keylines {
    if matches!(
        layout,
        CarouselLayout::Uncontained | CarouselLayout::FullScreen | CarouselLayout::CenteredHero
    ) {
        return start.clone();
    }
    let widths: Vec<f32> = start.lines.iter().rev().map(|line| line.width).collect();
    let first = start.lines.first().map_or(0., |line| line.x);
    let gap = if layout == CarouselLayout::FullScreen {
        0.
    } else {
        GAP
    };
    Keylines {
        lines: lay_out(&widths, first, gap),
        focal: start.focal,
        step: start.step,
    }
}

/// The furthest scroll offset for `count` items.
pub(crate) fn max_scroll(keylines: &Keylines, count: usize, layout: CarouselLayout) -> f32 {
    let on_screen = match layout {
        // Hero layouts let every item reach the focal keyline.
        CarouselLayout::CenteredHero => return (count.saturating_sub(1)) as f32 * keylines.step,
        _ => keylines.lines.len(),
    };
    count.saturating_sub(on_screen) as f32 * keylines.step
}

/// The on-screen keyline of an item at scroll-space position `p` (its index
/// times the step, minus the scroll), interpolating between keylines and
/// continuing past the edges at the edge items' size.
pub(crate) fn place(
    lines: &[Keyline],
    step: f32,
    p: f32,
    leading_index: usize,
    gap: f32,
) -> Keyline {
    // Scroll-space keyline j sits at (j - leading_index) * step.
    let j = p / step + leading_index as f32;
    let last = lines.len() - 1;
    if j <= 0. {
        let edge = lines[0];
        let small = CAROUSEL_SMALL_ITEM.min(edge.width);
        // Leaving the leading edge: shrink to small while moving off.
        let t = (-j).min(1.);
        let width = edge.width + (small - edge.width) * t;
        // One step fully clears the leading padding.
        let x = edge.x + j * (small + gap + edge.x);
        return Keyline { x, width };
    }
    if j >= last as f32 {
        let edge = lines[last];
        let x = edge.x + (j - last as f32) * (edge.width + gap);
        return Keyline {
            x,
            width: edge.width,
        };
    }
    let i = j.floor() as usize;
    let t = j - i as f32;
    let (a, b) = (lines[i], lines[i + 1]);
    Keyline {
        x: a.x + (b.x - a.x) * t,
        width: a.width + (b.width - a.width) * t,
    }
}

fn blend(a: &[Keyline], b: &[Keyline], t: f32) -> Vec<Keyline> {
    a.iter()
        .zip(b)
        .map(|(a, b)| Keyline {
            x: a.x + (b.x - a.x) * t,
            width: a.width + (b.width - a.width) * t,
        })
        .collect()
}

/// One carousel item: caller content (e.g. an image) masked by the item,
/// with an optional label and supporting text on large items.
pub struct CarouselItem {
    content: Option<AnyElement>,
    label: Option<SharedString>,
    supporting: Option<SharedString>,
}

impl CarouselItem {
    pub fn new() -> Self {
        Self {
            content: None,
            label: None,
            supporting: None,
        }
    }
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn supporting(mut self, supporting: impl Into<SharedString>) -> Self {
        self.supporting = Some(supporting.into());
        self
    }
}

impl Default for CarouselItem {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
struct ScrollAnimation {
    from: f32,
    to: f32,
    started: Instant,
}

#[derive(Default)]
struct CarouselState {
    scroll: f32,
    width: Option<f32>,
    animation: Option<ScrollAnimation>,
    epoch: usize,
}

impl CarouselState {
    fn animate_to(&mut self, to: f32, reduced: bool) {
        if reduced || (to - self.scroll).abs() < 0.5 {
            self.scroll = to;
            self.animation = None;
        } else {
            self.animation = Some(ScrollAnimation {
                from: self.scroll,
                to,
                started: Instant::now(),
            });
        }
    }
}

/// The nearest item boundary to `scroll`.
pub(crate) fn snap(scroll: f32, step: f32, max: f32) -> f32 {
    if step <= 0. {
        return 0.;
    }
    ((scroll / step).round() * step).clamp(0., max.max(0.))
}

/// An M3 carousel. `label` names it for assistive technology.
#[derive(gpui::IntoElement)]
pub struct MaterialCarousel {
    id: ElementId,
    label: SharedString,
    layout: CarouselLayout,
    items: Vec<CarouselItem>,
    large: f32,
    height: f32,
    width: Option<f32>,
}

impl MaterialCarousel {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            layout: CarouselLayout::MultiBrowse,
            items: Vec::new(),
            large: 188.,
            height: 205.,
            width: None,
        }
    }
    pub fn layout(mut self, layout: CarouselLayout) -> Self {
        self.layout = layout;
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = CarouselItem>) -> Self {
        self.items.extend(items);
        self
    }
    /// The large item's width in dp (multi-browse and uncontained).
    pub fn large_item_width(mut self, width: f32) -> Self {
        self.large = width.max(CAROUSEL_SMALL_ITEM);
        self
    }
    /// The item height in dp.
    pub fn item_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
    /// A fixed carousel width in dp; otherwise it fills its parent and uses
    /// the width measured in the previous frame.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

fn typed<E: Styled>(element: E, role: TypeRole) -> E {
    let scale = role.scale();
    element
        .font_family(scale.font_family)
        .text_size(scale.font_size)
        .line_height(scale.line_height)
        .font_weight(FontWeight(scale.font_weight as f32))
}

/// Schedule a snap once scrolling has rested.
fn schedule_snap(
    state: &Entity<CarouselState>,
    step: f32,
    max: f32,
    window: &mut Window,
    cx: &mut App,
) {
    let epoch = state.update(cx, |state, _| {
        state.epoch += 1;
        state.epoch
    });
    let state = state.downgrade();
    window
        .spawn(cx, async move |cx| {
            cx.background_executor().timer(SNAP_IDLE).await;
            let _ = cx.update(|window, cx| {
                let Some(state) = state.upgrade() else { return };
                if state.read(cx).epoch != epoch {
                    return;
                }
                let reduced = cx.reduce_motion();
                state.update(cx, |state, _| {
                    let to = snap(state.scroll, step, max);
                    state.animate_to(to, reduced);
                });
                window.refresh();
            });
        })
        .detach();
}

impl RenderOnce for MaterialCarousel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let state = window.use_keyed_state((id.clone(), "carousel"), cx, |_, _| {
            CarouselState::default()
        });
        let focus = window
            .use_keyed_state((id.clone(), "carousel-focus"), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            })
            .read(cx)
            .clone();
        let layout = self.layout;
        let rem_size = f32::from(window.rem_size()) / 16.;
        let width = self
            .width
            .or(state.read(cx).width.map(|w| w / rem_size))
            .unwrap_or(412.);
        let start = keylines(layout, width, self.large);
        let end = end_keylines(&start, layout);
        let count = self.items.len();
        let step = start.step;
        let max = max_scroll(&start, count, layout);
        let reduced = cx.reduce_motion();

        // Advance a snap or keyboard scroll animation.
        let mut scroll = state.read(cx).scroll.clamp(0., max);
        if let Some(animation) = state.read(cx).animation {
            let curve = motion::curve(MotionToken::DefaultSpatial);
            let duration = curve.duration().as_secs_f32().max(f32::EPSILON);
            let t = animation.started.elapsed().as_secs_f32() / duration;
            if t >= 1. || reduced {
                scroll = animation.to;
                state.update(cx, |state, _| {
                    state.scroll = animation.to;
                    state.animation = None;
                });
            } else {
                scroll = animation.from + (animation.to - animation.from) * curve.ease()(t);
                window.request_animation_frame();
            }
        }

        // Shift toward the mirrored keylines over the final steps.
        let shift_len = ((start.lines.len() - start.focal) as f32 * step).min(max);
        let shift = if shift_len > 0. {
            ((scroll - (max - shift_len)) / shift_len).clamp(0., 1.)
        } else {
            0.
        };
        let lines = blend(&start.lines, &end.lines, shift);
        let gap = if layout == CarouselLayout::FullScreen {
            0.
        } else {
            GAP
        };
        let leading = if layout == CarouselLayout::CenteredHero {
            1
        } else {
            0
        };
        let large = start.lines.iter().map(|line| line.width).fold(0., f32::max);
        let pad_y = if layout == CarouselLayout::FullScreen {
            0.
        } else {
            PAD_Y
        };

        let items = self
            .items
            .into_iter()
            .enumerate()
            .filter_map(|(index, item)| {
                let p = index as f32 * step - scroll;
                let line = place(&lines, step, p, leading, gap);
                if line.x + line.width < 0. || line.x > width {
                    return None;
                }
                // Labels fade in as an item approaches the large size.
                let text = ((line.width - CAROUSEL_SMALL_ITEM)
                    / (large - CAROUSEL_SMALL_ITEM).max(1.))
                .clamp(0., 1.);
                let text = (text - 0.5).max(0.) * 2.;
                let has_text = item.label.is_some() || item.supporting.is_some();
                Some(
                    div()
                        .absolute()
                        .top(rem(pad_y))
                        .left(rem(line.x))
                        .w(rem(line.width))
                        .h(rem(self.height))
                        .overflow_hidden()
                        .rounded(CAROUSEL_ITEM_RADIUS)
                        .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
                        .child(
                            // Content keeps the large width, centered under the
                            // morphing mask.
                            div()
                                .absolute()
                                .top_0()
                                .left(rem((line.width - large) / 2.))
                                .w(rem(large))
                                .h_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .children(item.content),
                        )
                        .when(has_text && text > 0., |tile| {
                            // The kit paints white text over a scrim gradient on
                            // imagery, independent of the theme.
                            let ink = gpui::white();
                            tile.child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .right_0()
                                    .bottom_0()
                                    .flex()
                                    .flex_col()
                                    .gap(rem(4.))
                                    .p(rem(16.))
                                    .opacity(text)
                                    .bg(linear_gradient(
                                        180.,
                                        linear_color_stop(color(ColorRole::SCRIM).opacity(0.), 0.),
                                        linear_color_stop(color(ColorRole::SCRIM).opacity(0.4), 1.),
                                    ))
                                    .text_color(ink)
                                    .when_some(item.label, |text, label| {
                                        text.child(
                                            typed(div(), TypeRole::TitleMedium)
                                                .truncate()
                                                .child(label),
                                        )
                                    })
                                    .when_some(item.supporting, |text, supporting| {
                                        text.child(
                                            typed(div(), TypeRole::LabelSmall)
                                                .truncate()
                                                .child(supporting),
                                        )
                                    }),
                            )
                        }),
                )
            });

        let wheel = state.clone();
        let keys = state.clone();
        let measure = state.clone();
        let fixed = self.width.is_some();
        let keyboard_focus = focus.is_focused(window) && window.last_input_was_keyboard();
        div()
            .id(id)
            .role(Role::List)
            .aria_label(self.label)
            .track_focus(&focus)
            .relative()
            .overflow_hidden()
            .map(|carousel| {
                if fixed {
                    carousel.w(rem(width))
                } else {
                    carousel.w_full()
                }
            })
            .h(rem(self.height + 2. * pad_y))
            .on_scroll(move |event, window, cx| {
                let delta = event.delta.pixel_delta(px(16.));
                // Horizontal gestures scroll; vertical wheels scroll too when
                // they dominate, as a horizontal list has no other use.
                let amount = if delta.x.abs() > delta.y.abs() {
                    -f32::from(delta.x)
                } else {
                    -f32::from(delta.y)
                } / rem_size;
                if amount == 0. || max <= 0. {
                    return;
                }
                cx.stop_propagation();
                wheel.update(cx, |state, _| {
                    state.animation = None;
                    state.scroll = (state.scroll + amount).clamp(0., max);
                });
                window.refresh();
                schedule_snap(&wheel, step, max, window, cx);
            })
            .on_key_down(move |event, window, cx| {
                let current = keys
                    .read(cx)
                    .animation
                    .map_or(keys.read(cx).scroll, |a| a.to);
                let to = match event.keystroke.key.as_str() {
                    "right" => snap(current, step, max) + step,
                    "left" => snap(current, step, max) - step,
                    "home" => 0.,
                    "end" => max,
                    _ => return,
                }
                .clamp(0., max);
                cx.stop_propagation();
                let reduced = cx.reduce_motion();
                keys.update(cx, |state, _| state.animate_to(to, reduced));
                window.refresh();
            })
            .children(items)
            .when(keyboard_focus, |carousel| {
                // The M3 focus indicator: a 3dp secondary ring.
                carousel.child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded(CAROUSEL_ITEM_RADIUS)
                        .border(px(3.))
                        .border_color(color(ColorRole::SECONDARY)),
                )
            })
            .child(
                canvas(
                    move |bounds, _, cx| {
                        let width = f32::from(bounds.size.width);
                        if measure.read(cx).width != Some(width) {
                            measure.update(cx, |state, _| state.width = Some(width));
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    fn widths(keylines: &Keylines) -> Vec<f32> {
        keylines
            .lines
            .iter()
            .map(|line| line.width.round())
            .collect()
    }

    #[test]
    fn strategies_match_the_kit_on_mobile_and_tablet() {
        // Mobile (412dp): 188, 120, 56 — the kit's multi-browse.
        assert_eq!(
            widths(&keylines(CarouselLayout::MultiBrowse, 412., 188.)),
            [188., 120., 56.]
        );
        // Tablet (600dp): 184, 184, 120, 56.
        assert_eq!(
            widths(&keylines(CarouselLayout::MultiBrowse, 600., 184.)),
            [184., 184., 120., 56.]
        );
        // Hero (412dp): 316 and 56; center-aligned hero 56, 252, 56.
        assert_eq!(
            widths(&keylines(CarouselLayout::Hero, 412., 188.)),
            [316., 56.]
        );
        assert_eq!(
            widths(&keylines(CarouselLayout::CenteredHero, 412., 188.)),
            [56., 252., 56.]
        );
        let full = keylines(CarouselLayout::FullScreen, 412., 188.);
        assert_eq!(widths(&full), [412.]);
        assert_eq!(full.lines[0].x, 0.);
    }

    #[test]
    fn items_morph_between_keylines() {
        let start = keylines(CarouselLayout::MultiBrowse, 412., 188.);
        let step = start.step;
        // On a keyline an item takes its size.
        assert_eq!(place(&start.lines, step, step, 0, GAP).width, 120.);
        // Halfway from the medium to the small keyline it is in between.
        let half = place(&start.lines, step, 1.5 * step, 0, GAP);
        assert!((half.width - 88.).abs() < 0.01);
        // Leaving the leading edge it shrinks.
        assert!(place(&start.lines, step, -0.5 * step, 0, GAP).width < 188.);
    }

    #[test]
    fn scrolling_snaps_to_items_within_range() {
        assert_eq!(snap(130., 196., 1000.), 196.);
        assert_eq!(snap(90., 196., 1000.), 0.);
        assert_eq!(snap(5000., 196., 392.), 392.);
        let start = keylines(CarouselLayout::MultiBrowse, 412., 188.);
        assert_eq!(max_scroll(&start, 3, CarouselLayout::MultiBrowse), 0.);
        assert_eq!(
            max_scroll(&start, 5, CarouselLayout::MultiBrowse),
            2. * start.step
        );
    }

    #[test]
    fn the_end_arrangement_mirrors_the_start() {
        let start = keylines(CarouselLayout::MultiBrowse, 412., 188.);
        let end = end_keylines(&start, CarouselLayout::MultiBrowse);
        assert_eq!(widths(&end), [56., 120., 188.]);
        let last = end.lines.last().unwrap();
        assert!((last.x + last.width - (412. - PAD_X)).abs() < 0.01);
    }

    struct Harness;
    impl gpui::Render for Harness {
        fn render(&mut self, window: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(16.));
            div().child(
                MaterialCarousel::new("carousel", "Photos")
                    .width(412.)
                    .items((0..6).map(|i| {
                        CarouselItem::new().content(
                            div()
                                .debug_selector(move || format!("item-{i}"))
                                .size(px(10.)),
                        )
                    })),
            )
        }
    }

    #[gpui::test]
    fn arrow_keys_scroll_one_item(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (_, cx) = cx.add_window_view(|_, _| Harness);
        let draw = |cx: &mut gpui::VisualTestContext| {
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
        };
        draw(cx);
        let before = cx.debug_bounds("item-1").unwrap();
        cx.update(|window, cx| window.focus_next(cx));
        let keystroke = cx.update(|_, cx| {
            gpui::Keystroke::parse_with_os("right", cx.platform_info().os).unwrap()
        });
        cx.simulate_event(gpui::KeyDownEvent {
            keystroke,
            is_held: false,
            prefer_character_input: false,
        });
        draw(cx);
        let after = cx.debug_bounds("item-1").unwrap();
        assert!(
            after.left() < before.left(),
            "the next item moved toward the start"
        );
        assert!(
            cx.debug_bounds("item-4").is_some(),
            "a further item scrolled in"
        );
    }
}
