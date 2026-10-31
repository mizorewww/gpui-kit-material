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
    tokens::typography::{BASE_PX, TypeRole, rem},
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
/// Travel before a press on an item is a swipe rather than a click, in dp.
const SWIPE_SLOP: f32 = 8.;

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

/// A caller's current-item change handler.
type ChangeHandler = std::rc::Rc<dyn Fn(&usize, &mut Window, &mut App)>;

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
    /// The space between items, in dp.
    pub gap: f32,
}

/// A carousel's spacing in dp: between items and at the leading and
/// trailing edges. Full-screen carousels use neither.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarouselSpacing {
    pub gap: f32,
    pub padding: f32,
}

impl Default for CarouselSpacing {
    /// The kit's 8dp gap and 16dp padding.
    fn default() -> Self {
        Self {
            gap: GAP,
            padding: PAD_X,
        }
    }
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
#[cfg(test)]
pub(crate) fn keylines(layout: CarouselLayout, width: f32, large: f32) -> Keylines {
    spaced_keylines(layout, width, large, CarouselSpacing::default())
}

/// [`keylines`] with caller spacing.
pub(crate) fn spaced_keylines(
    layout: CarouselLayout,
    width: f32,
    large: f32,
    spacing: CarouselSpacing,
) -> Keylines {
    let (pad, gap) = if layout == CarouselLayout::FullScreen {
        (0., 0.)
    } else {
        (spacing.padding.max(0.), spacing.gap.max(0.))
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
            // Items run off the trailing edge: keylines cover the viewport
            // past the trailing padding.
            let count = ((width - pad + gap) / (large + gap)).ceil().max(1.) as usize;
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
        gap,
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
    Keylines {
        lines: lay_out(&widths, first, start.gap),
        focal: start.focal,
        step: start.step,
        gap: start.gap,
    }
}

/// The furthest scroll offset for `count` items in a carousel `width` wide.
pub(crate) fn max_scroll(
    keylines: &Keylines,
    count: usize,
    layout: CarouselLayout,
    width: f32,
) -> f32 {
    let on_screen = match layout {
        // Hero layouts let every item reach the focal keyline.
        CarouselLayout::CenteredHero => return (count.saturating_sub(1)) as f32 * keylines.step,
        // Uncontained items keep one size, so the end is where the last
        // item's trailing edge meets the trailing padding. Its keylines run
        // past the viewport, so their count says nothing about the end.
        CarouselLayout::Uncontained => {
            let pad = keylines.lines.first().map_or(0., |line| line.x);
            let content = count as f32 * keylines.step - keylines.gap;
            return (content - (width - 2. * pad)).max(0.);
        }
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
        // Past the trailing keyline an item waits beyond the trailing
        // padding (the leading padding mirrors it), so none peeks in.
        let x = edge.x + (j - last as f32) * (edge.width + gap + lines[0].x);
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

/// Blend `a` toward `b` into `out`, reusing its allocation.
fn blend_into(out: &mut Vec<Keyline>, a: &[Keyline], b: &[Keyline], t: f32) {
    out.clear();
    out.extend(a.iter().zip(b).map(|(a, b)| Keyline {
        x: a.x + (b.x - a.x) * t,
        width: a.width + (b.width - a.width) * t,
    }));
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

/// The start and end keylines for one layout, width and large size.
struct KeylineCache {
    key: (CarouselLayout, u32, u32, u32, u32),
    start: Keylines,
    end: Keylines,
}

#[derive(Default)]
struct CarouselState {
    /// Whether the start position has been applied.
    started: bool,
    scroll: f32,
    width: Option<f32>,
    animation: Option<ScrollAnimation>,
    /// Keylines change only with the layout, width or large size; render
    /// rebuilds them when that key changes.
    keylines: Option<KeylineCache>,
    /// The blended keylines of the current frame, reused between frames.
    lines: Vec<Keyline>,
    /// When resting scroll should snap; each wheel event pushes it back.
    snap_due: Option<Instant>,
    /// The snap step and end of the latest wheel event.
    snap_to: (f32, f32),
    /// Whether the one snap task is waiting for `snap_due`.
    snap_pending: bool,
    /// The caller's current item last applied, and the item last reported.
    applied: Option<usize>,
    reported: Option<usize>,
    /// The caller's change handler, for the snap task.
    on_change: Option<ChangeHandler>,
    /// While the pointer drags: its x and the scroll when the drag began.
    drag: Option<(Pixels, f32)>,
    /// The pointer has travelled far enough to be a swipe, not a click.
    swiped: bool,
    /// The item step the scroll was last laid out with.
    laid_out_step: Option<f32>,
}

#[cfg(test)]
thread_local! {
    /// Snap tasks spawned on this thread, for tests.
    static SNAP_TASKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

impl CarouselState {
    /// The scroll the carousel is at or heading to.
    fn destination(&self) -> f32 {
        self.animation.map_or(self.scroll, |animation| animation.to)
    }

    /// The item a scroll position settles on.
    fn item_at(scroll: f32, step: f32) -> usize {
        if step <= 0. {
            0
        } else {
            (scroll / step).round().max(0.) as usize
        }
    }

    /// Where the items are now: part way along an animation, if one runs.
    /// Whatever interrupts one starts from here, not from its origin.
    fn position(&self) -> f32 {
        let Some(animation) = self.animation else {
            return self.scroll;
        };
        let curve = motion::curve(MotionToken::DefaultSpatial);
        let duration = curve.duration().as_secs_f32().max(f32::EPSILON);
        let t = animation.started.elapsed().as_secs_f32() / duration;
        if t >= 1. {
            animation.to
        } else {
            animation.from + (animation.to - animation.from) * curve.ease()(t)
        }
    }

    fn animate_to(&mut self, to: f32, reduced: bool) {
        let from = self.position();
        self.scroll = from;
        if reduced || (to - from).abs() < 0.5 {
            self.scroll = to;
            self.animation = None;
        } else {
            self.animation = Some(ScrollAnimation {
                from,
                to,
                started: Instant::now(),
            });
        }
    }
}

/// Tell the caller the item `to` settles on, once per change.
fn report(state: &Entity<CarouselState>, to: f32, step: f32, window: &mut Window, cx: &mut App) {
    let item = CarouselState::item_at(to, step);
    let handler = state.update(cx, |state, _| {
        if state.reported == Some(item) {
            return None;
        }
        state.reported = Some(item);
        state.applied = Some(item);
        state.on_change.clone()
    });
    if let Some(handler) = handler {
        handler(&item, window, cx);
    }
}

/// The nearest item boundary to `scroll`, or the end of the range when that
/// is nearer (an uncontained carousel's end need not fall on a boundary).
pub(crate) fn snap(scroll: f32, step: f32, max: f32) -> f32 {
    let max = max.max(0.);
    if step <= 0. {
        return 0.;
    }
    let boundary = ((scroll / step).round() * step).clamp(0., max);
    if (max - scroll).abs() < (boundary - scroll).abs() {
        max
    } else {
        boundary
    }
}

/// The next item boundary after `current` (or before it, backward), within
/// `0..=max`; the end of the range counts as a boundary.
pub(crate) fn neighbor(current: f32, step: f32, max: f32, forward: bool) -> f32 {
    let max = max.max(0.);
    if step <= 0. {
        return 0.;
    }
    let to = if forward {
        (((current + 0.5) / step).floor() + 1.) * step
    } else {
        (((current - 0.5) / step).ceil() - 1.) * step
    };
    to.clamp(0., max)
}

/// The kit paints carousel labels in untokenized white (`#FFFFFF`, Figma
/// "Material 3 Design Kit" › Carousel › Label) over the scrim gradient on
/// caller imagery, in both themes; no color role is white in both.
const LABEL_ON_IMAGERY: Rgba = Rgba {
    r: 1.,
    g: 1.,
    b: 1.,
    a: 1.,
};

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
    spacing: CarouselSpacing,
    radius: Pixels,
    snap: bool,
    current: Option<usize>,
    on_change: Option<ChangeHandler>,
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
            spacing: CarouselSpacing::default(),
            radius: CAROUSEL_ITEM_RADIUS,
            snap: true,
            current: None,
            on_change: None,
        }
    }
    /// The space between items, in dp (the kit's 8dp by default).
    pub fn gap(mut self, gap: f32) -> Self {
        self.spacing.gap = gap.max(0.);
        self
    }
    /// The leading and trailing padding, in dp (the kit's 16dp by default).
    pub fn padding(mut self, padding: f32) -> Self {
        self.spacing.padding = padding.max(0.);
        self
    }
    /// Item corners (the kit's 28dp extra-large shape by default).
    pub fn corner_radius(mut self, radius: Pixels) -> Self {
        self.radius = radius.max(px(0.));
        self
    }
    /// Whether resting scroll settles on an item boundary (on by default).
    pub fn snap(mut self, snap: bool) -> Self {
        self.snap = snap;
        self
    }
    /// The current item: the leading focal one (the hero, centered). A new
    /// value scrolls there on the default spatial curve.
    pub fn current(mut self, index: usize) -> Self {
        self.current = Some(index);
        self
    }
    /// Called with the new current item when scrolling, dragging or the
    /// keyboard moves the carousel to another one.
    pub fn on_change(mut self, handler: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(handler));
        self
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
        .letter_spacing(scale.letter_spacing)
        .line_height(scale.line_height)
        .font_weight(FontWeight(scale.font_weight as f32))
}

/// Let go of a swipe at the position it reached, snapping if the carousel does.
fn end_swipe(
    state: &Entity<CarouselState>,
    snaps: bool,
    step: f32,
    max: f32,
    window: &mut Window,
    cx: &mut App,
) {
    let reduced = cx.reduce_motion();
    let to = state.update(cx, |state, _| {
        state.drag = None;
        let to = if snaps {
            snap(state.scroll, step, max)
        } else {
            state.scroll
        };
        state.animate_to(to, reduced);
        to
    });
    report(state, to, step, window, cx);
    window.refresh();
}

/// Snap once scrolling has rested for [`SNAP_IDLE`]. Each wheel event only
/// pushes the deadline back; one task waits for it.
fn schedule_snap(
    state: &Entity<CarouselState>,
    step: f32,
    max: f32,
    window: &mut Window,
    cx: &mut App,
) {
    let due = cx.background_executor().now() + SNAP_IDLE;
    let spawn = state.update(cx, |state, _| {
        state.snap_due = Some(due);
        state.snap_to = (step, max);
        !std::mem::replace(&mut state.snap_pending, true)
    });
    if !spawn {
        return;
    }
    #[cfg(test)]
    SNAP_TASKS.with(|tasks| tasks.set(tasks.get() + 1));
    let state = state.downgrade();
    window
        .spawn(cx, async move |cx| {
            loop {
                let wait = cx.update(|_, cx| {
                    let due = state.upgrade()?.read(cx).snap_due?;
                    Some(due.saturating_duration_since(cx.background_executor().now()))
                });
                match wait {
                    Ok(Some(wait)) if !wait.is_zero() => cx.background_executor().timer(wait).await,
                    Ok(Some(_)) => break,
                    _ => {
                        // Something else took over and called the snap off.
                        let _ = cx.update(|_, cx| {
                            if let Some(state) = state.upgrade() {
                                state.update(cx, |state, _| state.snap_pending = false);
                            }
                        });
                        return;
                    }
                }
            }
            let _ = cx.update(|window, cx| {
                let Some(state) = state.upgrade() else { return };
                let reduced = cx.reduce_motion();
                let (to, step) = state.update(cx, |state, _| {
                    state.snap_pending = false;
                    state.snap_due = None;
                    let (step, max) = state.snap_to;
                    let to = snap(state.scroll, step, max);
                    state.animate_to(to, reduced);
                    (to, step)
                });
                report(&state, to, step, window, cx);
                window.refresh();
            });
        })
        .detach();
}

impl RenderOnce for MaterialCarousel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Input runs after the draw, so it keeps this frame's direction.
        let rtl = crate::scope::is_rtl(window);
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
        // Window px per design dp (`rem` sizes from a 14px base).
        let rem_size = f32::from(window.rem_size()) / BASE_PX;
        let width = self
            .width
            .or(state.read(cx).width.map(|w| w / rem_size))
            .unwrap_or(412.);
        let spacing = self.spacing;
        let key = (
            layout,
            width.to_bits(),
            self.large.to_bits(),
            spacing.gap.to_bits(),
            spacing.padding.to_bits(),
        );
        if state.read(cx).keylines.as_ref().map(|cache| cache.key) != Some(key) {
            let start = spaced_keylines(layout, width, self.large, spacing);
            let end = end_keylines(&start, layout);
            state.update(cx, |state, _| {
                state.keylines = Some(KeylineCache { key, start, end })
            });
        }
        let count = self.items.len();
        let (step, max, shift_len, large, gap) = {
            let start = &state
                .read(cx)
                .keylines
                .as_ref()
                .expect("keylines were just cached")
                .start;
            let step = start.step;
            let max = max_scroll(start, count, layout, width);
            let shift_len = ((start.lines.len() - start.focal) as f32 * step).min(max);
            let large = start.lines.iter().map(|line| line.width).fold(0., f32::max);
            (step, max, shift_len, large, start.gap)
        };
        let reduced = cx.reduce_motion();

        // A carousel measures its width a frame late and follows its parent,
        // and its step follows the width. What is showing keeps showing.
        if state.read(cx).laid_out_step != Some(step) {
            state.update(cx, |state, _| {
                if let Some(last) = state.laid_out_step.filter(|last| *last > 0.) {
                    let scale = step / last;
                    state.scroll = state.position() * scale;
                    state.animation = None;
                    if let Some((_, from)) = state.drag.as_mut() {
                        *from *= scale;
                    }
                }
                state.laid_out_step = Some(step);
            });
        }

        // Advance a snap or keyboard scroll animation.
        if !state.read(cx).started {
            // A center-aligned hero starts with its first item at the
            // leading small keyline and the second at the hero.
            let start = if layout == CarouselLayout::CenteredHero {
                step.min(max)
            } else {
                0.
            };
            state.update(cx, |state, _| {
                state.started = true;
                state.scroll = start;
            });
        }
        // Follow the caller's current item when it changes.
        let on_change = self.on_change.clone();
        state.update(cx, |state, _| state.on_change = on_change);
        if let Some(current) = self.current
            && state.read(cx).applied != Some(current)
        {
            let to = (current as f32 * step).clamp(0., max);
            let first = state.read(cx).applied.is_none();
            state.update(cx, |state, _| {
                state.applied = Some(current);
                state.reported = Some(current);
                if CarouselState::item_at(state.destination(), step) != current {
                    // The first frame starts there; later changes move.
                    state.animate_to(to, reduced || first);
                }
            });
            if state.read(cx).animation.is_some() {
                window.request_animation_frame();
            }
        }
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
        let shift = if shift_len > 0. {
            ((scroll - (max - shift_len)) / shift_len).clamp(0., 1.)
        } else {
            0.
        };
        // Blend into the state's buffer; it is handed back after the items
        // are built, so animating frames allocate no keylines.
        let lines = state.update(cx, |state, _| {
            let mut lines = std::mem::take(&mut state.lines);
            if let Some(cache) = &state.keylines {
                blend_into(&mut lines, &cache.start.lines, &cache.end.lines, shift);
            }
            lines
        });
        let leading = if layout == CarouselLayout::CenteredHero {
            1
        } else {
            0
        };
        let pad_y = if layout == CarouselLayout::FullScreen {
            0.
        } else {
            PAD_Y
        };

        let radius = self.radius;
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
                // Each item is a list item named by the caller's label.
                let name = item.label.clone();
                let description = item.supporting.clone();
                Some(
                    div()
                        .id(("carousel-item", index))
                        .role(Role::ListItem)
                        .when_some(name, |tile, name| tile.aria_label(name))
                        .when_some(description, |tile, text| tile.aria_description(text))
                        .absolute()
                        .top(rem(pad_y))
                        .left(rem(line.x))
                        .w(rem(line.width))
                        .h(rem(self.height))
                        .overflow_hidden()
                        .rounded(radius)
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
                            let ink = LABEL_ON_IMAGERY;
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
        let grab = state.clone();
        let snaps = self.snap;
        let dragging = state.read(cx).drag.is_some();
        let fixed = self.width.is_some();
        let keyboard_focus = focus.is_focused(window) && window.last_input_was_keyboard();
        let carousel = div()
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
                let horizontal = delta.x.abs() > delta.y.abs();
                let amount = if horizontal {
                    // Items laid out from the right follow the fingers the other way.
                    if rtl {
                        f32::from(delta.x)
                    } else {
                        -f32::from(delta.x)
                    }
                } else {
                    -f32::from(delta.y)
                } / rem_size;
                if amount == 0. || max <= 0. {
                    return;
                }
                let before = wheel.read(cx).position();
                let after = (before + amount).clamp(0., max);
                let moved = after != before;
                // A vertical wheel the carousel cannot follow (it is at that
                // end) scrolls the page instead.
                if horizontal || moved {
                    cx.stop_propagation();
                }
                if !moved {
                    return;
                }
                wheel.update(cx, |state, _| {
                    state.animation = None;
                    state.scroll = after;
                });
                window.refresh();
                if snaps {
                    schedule_snap(&wheel, step, max, window, cx);
                } else {
                    report(&wheel, after, step, window, cx);
                }
            })
            .on_key_down(move |event, window, cx| {
                let current = keys
                    .read(cx)
                    .animation
                    .map_or(keys.read(cx).scroll, |a| a.to);
                let to = match event.keystroke.key.as_str() {
                    "right" => neighbor(current, step, max, !rtl),
                    "left" => neighbor(current, step, max, rtl),
                    "home" => 0.,
                    "end" => max,
                    _ => return,
                };
                cx.stop_propagation();
                let reduced = cx.reduce_motion();
                keys.update(cx, |state, _| {
                    state.snap_due = None;
                    state.animate_to(to, reduced);
                });
                report(&keys, to, step, window, cx);
                window.refresh();
            })
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                // Swipe: the pointer drags the items.
                grab.update(cx, |state, _| {
                    state.scroll = state.position();
                    state.animation = None;
                    state.snap_due = None;
                    state.swiped = false;
                    state.drag = Some((event.position.x, state.scroll));
                });
                window.refresh();
            })
            .when(dragging, |carousel| {
                let follow = state.clone();
                carousel.cursor_grabbing().child(
                    canvas(
                        |_, _, _| (),
                        move |_, _, window, _| {
                            let moving = follow.clone();
                            window.on_mouse_event(
                                move |event: &MouseMoveEvent, phase, window, cx| {
                                    if phase != DispatchPhase::Capture {
                                        return;
                                    }
                                    if !event.dragging() {
                                        // The button came up where we could not see it.
                                        if moving.read(cx).drag.is_some() {
                                            end_swipe(&moving, snaps, step, max, window, cx);
                                        }
                                        return;
                                    }
                                    moving.update(cx, |state, _| {
                                        if let Some((x, from)) = state.drag {
                                            let delta = f32::from(event.position.x - x) / rem_size;
                                            state.swiped |= delta.abs() >= SWIPE_SLOP;
                                            let delta = if rtl { -delta } else { delta };
                                            state.scroll = (from - delta).clamp(0., max);
                                        }
                                    });
                                    window.refresh();
                                },
                            );
                            let dropping = follow.clone();
                            window.on_mouse_event(
                                move |event: &MouseUpEvent, phase, window, cx| {
                                    if phase != DispatchPhase::Capture
                                        || event.button != MouseButton::Left
                                    {
                                        return;
                                    }
                                    // A swipe's release clicks nothing under it.
                                    if dropping.read(cx).swiped {
                                        cx.stop_propagation();
                                    }
                                    end_swipe(&dropping, snaps, step, max, window, cx);
                                },
                            );
                        },
                    )
                    .absolute()
                    .size_0(),
                )
            })
            .children(items)
            .when(keyboard_focus, |carousel| {
                // The M3 focus indicator: a 3dp secondary ring.
                carousel.child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded(radius)
                        .border(px(3.))
                        .border_color(color(ColorRole::SECONDARY)),
                )
            })
            .child(
                canvas(
                    move |bounds, window, cx| {
                        let width = f32::from(bounds.size.width);
                        if measure.read(cx).width != Some(width) {
                            measure.update(cx, |state, _| state.width = Some(width));
                            // This frame laid items out for the old width; draw
                            // again with the new one (`refresh` is ignored while
                            // drawing).
                            if !fixed {
                                window.request_animation_frame();
                            }
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            );
        state.update(cx, |state, _| state.lines = lines);
        carousel
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
        assert_eq!(max_scroll(&start, 3, CarouselLayout::MultiBrowse, 412.), 0.);
        assert_eq!(
            max_scroll(&start, 5, CarouselLayout::MultiBrowse, 412.),
            2. * start.step
        );
    }

    #[test]
    fn an_uncontained_carousel_reaches_its_last_item() {
        let layout = CarouselLayout::Uncontained;
        let start = keylines(layout, 412., 188.);
        let max = max_scroll(&start, 6, layout, 412.);
        // Six 188dp items 8dp apart, in 380dp between the paddings.
        assert_eq!(max, 6. * 196. - 8. - (412. - 2. * PAD_X));
        // At the end the last item meets the trailing padding.
        let last = place(&start.lines, start.step, 5. * start.step - max, 0, GAP);
        assert!((last.x + last.width - (412. - PAD_X)).abs() < 0.01);
        // Snapping and stepping reach that end, though it is off the grid.
        assert_eq!(snap(max - 1., start.step, max), max);
        assert_eq!(neighbor(3. * start.step, start.step, max, true), 4. * 196.);
        assert_eq!(neighbor(4. * start.step, start.step, max, true), max);
        assert_eq!(neighbor(max, start.step, max, false), 4. * 196.);
        // Too few items to scroll.
        assert_eq!(max_scroll(&start, 1, layout, 412.), 0.);
    }

    #[test]
    fn blending_reuses_the_keyline_buffer() {
        let start = keylines(CarouselLayout::MultiBrowse, 412., 188.);
        let end = end_keylines(&start, CarouselLayout::MultiBrowse);
        let mut lines = Vec::new();
        blend_into(&mut lines, &start.lines, &end.lines, 0.25);
        let buffer = lines.as_ptr();
        blend_into(&mut lines, &start.lines, &end.lines, 0.75);
        assert_eq!(lines.as_ptr(), buffer, "the next frame reallocated");
        blend_into(&mut lines, &start.lines, &end.lines, 1.);
        assert_eq!(lines, end.lines);
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
            cx.debug_bounds("item-3").is_some(),
            "the next item scrolled in"
        );
        assert!(
            cx.debug_bounds("item-4").is_none(),
            "items beyond the keylines stay hidden"
        );
    }

    /// A carousel inside a page that counts the scroll events reaching it.
    struct Page {
        layout: CarouselLayout,
        fixed: Option<f32>,
        page_scrolls: std::rc::Rc<std::cell::Cell<usize>>,
    }
    impl gpui::Render for Page {
        fn render(&mut self, window: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(BASE_PX));
            let page_scrolls = self.page_scrolls.clone();
            let carousel = MaterialCarousel::new("carousel", "Photos")
                .layout(self.layout)
                .items((0..6).map(|i| {
                    CarouselItem::new().content(
                        div()
                            .debug_selector(move || format!("item-{i}"))
                            .size_full(),
                    )
                }));
            div()
                .id("page")
                .w(px(600.))
                .on_scroll(move |_, _, _| page_scrolls.set(page_scrolls.get() + 1))
                .child(match self.fixed {
                    Some(width) => carousel.width(width),
                    None => carousel,
                })
        }
    }

    fn page(
        cx: &mut gpui::TestAppContext,
        layout: CarouselLayout,
        fixed: Option<f32>,
    ) -> (gpui::Entity<Page>, &mut gpui::VisualTestContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        cx.add_window_view(move |_, _| Page {
            layout,
            fixed,
            page_scrolls: Default::default(),
        })
    }

    fn redraw(cx: &mut gpui::VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
    }

    #[gpui::test]
    fn the_end_key_brings_the_last_uncontained_item_in(cx: &mut gpui::TestAppContext) {
        let (_, cx) = page(cx, CarouselLayout::Uncontained, Some(412.));
        redraw(cx);
        cx.update(|window, cx| window.focus_next(cx));
        cx.simulate_keystrokes("end");
        redraw(cx);
        let last = cx.debug_bounds("item-5").expect("the last item is shown");
        assert!(
            (f32::from(last.right()) - (412. - PAD_X)).abs() < 0.5,
            "the last item ends at the trailing padding: {last:?}"
        );
    }

    #[gpui::test]
    fn a_vertical_wheel_at_an_end_scrolls_the_page(cx: &mut gpui::TestAppContext) {
        let (view, cx) = page(cx, CarouselLayout::MultiBrowse, Some(412.));
        redraw(cx);
        let position = cx.debug_bounds("item-0").unwrap().center();
        let scrolls = view.read_with(cx, |page, _| page.page_scrolls.clone());
        let wheel = |cx: &mut gpui::VisualTestContext, x: f32, y: f32| {
            cx.simulate_event(gpui::ScrollEvent {
                position,
                delta: gpui::ScrollDelta::Pixels(point(px(x), px(y))),
                touch_phase: TouchPhase::Moved,
                ..Default::default()
            });
        };
        // At the start, scrolling up cannot move the carousel.
        wheel(cx, 0., 40.);
        assert_eq!(scrolls.get(), 1, "the page did not receive the wheel");
        // Scrolling down moves it, so it keeps the wheel.
        wheel(cx, 0., -40.);
        assert_eq!(scrolls.get(), 1);
        // A horizontal gesture always belongs to the carousel.
        wheel(cx, 400., 0.);
        wheel(cx, 400., 0.);
        assert_eq!(scrolls.get(), 1);
    }

    #[gpui::test]
    fn wheel_events_share_one_snap_task(cx: &mut gpui::TestAppContext) {
        let (_, cx) = page(cx, CarouselLayout::MultiBrowse, Some(412.));
        redraw(cx);
        let position = cx.debug_bounds("item-0").unwrap().center();
        for _ in 0..5 {
            cx.simulate_event(gpui::ScrollEvent {
                position,
                delta: gpui::ScrollDelta::Pixels(point(px(-30.), px(0.))),
                touch_phase: TouchPhase::Moved,
                ..Default::default()
            });
        }
        assert_eq!(
            SNAP_TASKS.with(|tasks| tasks.get()),
            1,
            "every wheel event spawned a snap task"
        );
        // 150dp scrolled rests nearer the second item, which snaps to the
        // leading keyline.
        cx.executor().advance_clock(SNAP_IDLE * 2);
        cx.run_until_parked();
        redraw(cx);
        let second = cx.debug_bounds("item-1").unwrap();
        assert!((f32::from(second.left()) - PAD_X).abs() < 0.5, "{second:?}");
    }

    #[gpui::test]
    fn a_new_measured_width_draws_again(cx: &mut gpui::TestAppContext) {
        let (_, cx) = page(cx, CarouselLayout::MultiBrowse, None);
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let frames = cx.update(|window, cx| window.simulate_next_frame(cx));
        assert!(frames > 0, "the 600px width was stored without a new frame");
        redraw(cx);
        // The next frame lays out at 600dp: four large items (tablet).
        assert!(cx.debug_bounds("item-3").is_some());
        let frames = cx.update(|window, cx| window.simulate_next_frame(cx));
        assert_eq!(frames, 0, "an unchanged width requests nothing");
    }

    /// A carousel whose current item the test controls and observes.
    struct Controlled {
        current: usize,
        snap: bool,
        reported: std::rc::Rc<std::cell::Cell<Option<usize>>>,
    }
    impl gpui::Render for Controlled {
        fn render(
            &mut self,
            window: &mut Window,
            cx: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            window.set_rem_size(px(BASE_PX));
            let reported = self.reported.clone();
            MaterialCarousel::new("controlled", "Photos")
                .width(412.)
                .gap(12.)
                .padding(24.)
                .snap(self.snap)
                .current(self.current)
                .on_change(cx.listener(move |this, item: &usize, _, cx| {
                    reported.set(Some(*item));
                    this.current = *item;
                    cx.notify();
                }))
                .items((0..8).map(|i| {
                    CarouselItem::new().content(
                        div()
                            .debug_selector(move || format!("item-{i}"))
                            .size_full(),
                    )
                }))
        }
    }

    fn controlled(
        cx: &mut gpui::TestAppContext,
        current: usize,
        snap: bool,
    ) -> (gpui::Entity<Controlled>, &mut gpui::VisualTestContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        cx.add_window_view(move |_, _| Controlled {
            current,
            snap,
            reported: Default::default(),
        })
    }

    #[test]
    fn spacing_sets_the_padding_and_gap_of_the_keylines() {
        let spacing = CarouselSpacing {
            gap: 12.,
            padding: 24.,
        };
        let lines = spaced_keylines(CarouselLayout::MultiBrowse, 412., 188., spacing);
        assert_eq!(lines.lines[0].x, 24.);
        assert_eq!(lines.lines[1].x, 24. + lines.lines[0].width + 12.);
        assert_eq!(lines.step, 188. + 12.);
        // Full screen ignores both.
        let full = spaced_keylines(CarouselLayout::FullScreen, 412., 188., spacing);
        assert_eq!((full.lines[0].x, full.gap), (0., 0.));
    }

    #[gpui::test]
    fn the_current_item_starts_and_follows_the_caller(cx: &mut gpui::TestAppContext) {
        let (view, cx) = controlled(cx, 2, true);
        redraw(cx);
        // Item 2 starts at the leading keyline, inside the 24dp padding.
        let third = cx.debug_bounds("item-2").unwrap();
        assert!((f32::from(third.left()) - 24.).abs() < 0.5, "{third:?}");
        view.update(cx, |view, cx| {
            view.current = 3;
            cx.notify();
        });
        redraw(cx);
        // (Item 3 is the last before the end arrangement shifts in.)
        let fourth = cx.debug_bounds("item-3").unwrap();
        assert!((f32::from(fourth.left()) - 24.).abs() < 0.5, "{fourth:?}");
        // The keyboard moves on and reports the new item.
        cx.update(|window, cx| window.focus_next(cx));
        cx.simulate_keystrokes("left");
        redraw(cx);
        let reported = view.read_with(cx, |view, _| view.reported.get());
        assert_eq!(reported, Some(2));
    }

    #[gpui::test]
    fn dragging_swipes_and_snaps_to_the_nearest_item(cx: &mut gpui::TestAppContext) {
        use gpui::{Modifiers, MouseButton};
        let (view, cx) = controlled(cx, 0, true);
        redraw(cx);
        let start = cx.debug_bounds("item-0").unwrap().center();
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        redraw(cx);
        // 260dp left: past the middle of the second step (200dp).
        let to = start - point(px(260.), px(0.));
        cx.simulate_mouse_move(to, Some(MouseButton::Left), Modifiers::none());
        redraw(cx);
        cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::none());
        redraw(cx);
        let reported = view.read_with(cx, |view, _| view.reported.get());
        assert_eq!(reported, Some(1), "a 260dp drag settles on item 1");
        let second = cx.debug_bounds("item-1").unwrap();
        assert!((f32::from(second.left()) - 24.).abs() < 0.5, "{second:?}");
    }

    #[gpui::test]
    fn without_snapping_a_drag_rests_where_released(cx: &mut gpui::TestAppContext) {
        use gpui::{Modifiers, MouseButton};
        let (_, cx) = controlled(cx, 0, false);
        redraw(cx);
        let home = cx.debug_bounds("item-1").unwrap();
        let start = cx.debug_bounds("item-0").unwrap().center();
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        let to = start - point(px(60.), px(0.));
        cx.simulate_mouse_move(to, Some(MouseButton::Left), Modifiers::none());
        cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::none());
        redraw(cx);
        let moved = cx.debug_bounds("item-1").unwrap();
        assert!(moved.left() < home.left(), "the drag moved the items");
        assert!(
            (f32::from(moved.left()) - 24.).abs() > 1.,
            "it did not snap to an item"
        );
    }

    #[test]
    fn a_new_target_starts_from_where_the_animation_is() {
        let duration = motion::curve(MotionToken::DefaultSpatial).duration();
        let mut state = CarouselState {
            animation: Some(ScrollAnimation {
                from: 0.,
                to: 200.,
                started: Instant::now() - duration / 2,
            }),
            ..Default::default()
        };
        state.animate_to(400., false);
        let from = state.animation.expect("still moving").from;
        assert!(from > 50., "it went back to the old origin: {from}");
    }

    #[gpui::test]
    fn pressing_a_moving_carousel_holds_it_where_it_is(cx: &mut gpui::TestAppContext) {
        use gpui::{Modifiers, MouseButton};
        let (view, cx) = controlled(cx, 0, true);
        cx.update(|_, cx| cx.set_reduce_motion(false));
        redraw(cx);
        let start = cx.debug_bounds("item-0").unwrap().center();
        view.update(cx, |view, cx| {
            view.current = 1;
            cx.notify();
        });
        redraw(cx);
        std::thread::sleep(std::time::Duration::from_millis(100));
        redraw(cx);
        let moving = cx.debug_bounds("item-1").unwrap().left();
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        redraw(cx);
        let held = cx.debug_bounds("item-1").unwrap().left();
        assert!(
            held <= moving + px(0.5),
            "it jumped back: {moving:?} {held:?}"
        );
    }

    #[gpui::test]
    fn a_measured_width_rests_like_a_fixed_one(cx: &mut gpui::TestAppContext) {
        let left = |cx: &mut gpui::TestAppContext, fixed| {
            let (_, cx) = page(cx, CarouselLayout::CenteredHero, fixed);
            for _ in 0..3 {
                redraw(cx);
            }
            cx.debug_bounds("item-1").unwrap().left()
        };
        let measured = left(cx, None);
        assert_eq!(measured, left(cx, Some(600.)), "the first width stayed");
    }

    #[gpui::test]
    fn a_key_during_the_wheels_pause_is_not_undone_by_its_snap(cx: &mut gpui::TestAppContext) {
        let (view, cx) = controlled(cx, 0, true);
        // With motion: the wheel's position stays behind while the key's
        // animation runs, which is what an old snap would land on.
        cx.update(|_, cx| cx.set_reduce_motion(false));
        redraw(cx);
        let position = cx.debug_bounds("item-0").unwrap().center();
        let wheel = |cx: &mut gpui::VisualTestContext, x: f32| {
            cx.simulate_event(gpui::ScrollEvent {
                position,
                delta: gpui::ScrollDelta::Pixels(point(px(x), px(0.))),
                touch_phase: TouchPhase::Moved,
                ..Default::default()
            });
        };
        wheel(cx, -150.);
        cx.update(|window, cx| window.focus_next(cx));
        cx.simulate_keystrokes("home");
        redraw(cx);
        cx.executor().advance_clock(SNAP_IDLE * 2);
        cx.run_until_parked();
        redraw(cx);
        let reported = view.read_with(cx, |view, _| view.reported.get());
        assert_ne!(reported, Some(1), "the key's destination stands");
        // The wheel still snaps afterwards.
        wheel(cx, -650.);
        cx.executor().advance_clock(SNAP_IDLE * 2);
        cx.run_until_parked();
        redraw(cx);
        let reported = view.read_with(cx, |view, _| view.reported.get());
        assert!(reported.is_some_and(|item| item > 1), "{reported:?}");
    }

    #[gpui::test]
    fn a_lost_release_ends_the_swipe(cx: &mut gpui::TestAppContext) {
        use gpui::{Modifiers, MouseButton};
        let (_, cx) = controlled(cx, 0, false);
        redraw(cx);
        let start = cx.debug_bounds("item-0").unwrap().center();
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        redraw(cx);
        cx.simulate_mouse_move(
            start - point(px(60.), px(0.)),
            Some(MouseButton::Left),
            Modifiers::none(),
        );
        redraw(cx);
        let released = cx.debug_bounds("item-1").unwrap();
        // No button any more: this move is not a drag.
        cx.simulate_mouse_move(
            start - point(px(120.), px(0.)),
            None::<MouseButton>,
            Modifiers::none(),
        );
        redraw(cx);
        assert_eq!(cx.debug_bounds("item-1").unwrap(), released);
    }

    #[gpui::test]
    fn a_release_of_another_button_does_not_end_the_swipe(cx: &mut gpui::TestAppContext) {
        use gpui::{Modifiers, MouseButton};
        let (_, cx) = controlled(cx, 0, false);
        redraw(cx);
        let start = cx.debug_bounds("item-0").unwrap().center();
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        redraw(cx);
        cx.simulate_mouse_up(start, MouseButton::Right, Modifiers::none());
        redraw(cx);
        let to = start - point(px(60.), px(0.));
        let home = cx.debug_bounds("item-1").unwrap();
        cx.simulate_mouse_move(to, Some(MouseButton::Left), Modifiers::none());
        redraw(cx);
        assert!(cx.debug_bounds("item-1").unwrap().left() < home.left());
    }

    struct RtlCarousel;
    impl gpui::Render for RtlCarousel {
        fn render(&mut self, window: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(BASE_PX));
            crate::scope::Scope::new("rtl-carousel")
                .direction(gpui::LayoutDirection::Rtl)
                .child(
                    MaterialCarousel::new("carousel", "Photos")
                        .width(412.)
                        .snap(false)
                        .items((0..8).map(|i| {
                            CarouselItem::new().content(
                                div()
                                    .debug_selector(move || format!("item-{i}"))
                                    .size_full(),
                            )
                        })),
                )
        }
    }

    #[gpui::test]
    fn a_horizontal_swipe_moves_items_with_the_fingers_right_to_left(
        cx: &mut gpui::TestAppContext,
    ) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (_, cx) = cx.add_window_view(|_, _| RtlCarousel);
        redraw(cx);
        let home = cx.debug_bounds("item-1").unwrap();
        cx.simulate_event(gpui::ScrollEvent {
            position: home.center(),
            delta: gpui::ScrollDelta::Pixels(point(px(30.), px(0.))),
            touch_phase: TouchPhase::Moved,
            ..Default::default()
        });
        redraw(cx);
        assert!(
            cx.debug_bounds("item-1").unwrap().right() > home.right(),
            "fingers moving right carry the items right"
        );
    }

    struct Clickable(std::rc::Rc<std::cell::Cell<usize>>);
    impl gpui::Render for Clickable {
        fn render(&mut self, window: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(BASE_PX));
            let clicks = self.0.clone();
            MaterialCarousel::new("carousel", "Photos")
                .width(412.)
                .snap(false)
                .items((0..8).map(move |i| {
                    let clicks = clicks.clone();
                    CarouselItem::new().content(
                        div()
                            .id(("content", i as usize))
                            .debug_selector(move || format!("item-{i}"))
                            .size_full()
                            .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
                    )
                }))
        }
    }

    #[gpui::test]
    fn a_swipe_is_not_a_click(cx: &mut gpui::TestAppContext) {
        use gpui::{Modifiers, MouseButton};
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let clicks = std::rc::Rc::new(std::cell::Cell::new(0));
        let (_, cx) = cx.add_window_view({
            let clicks = clicks.clone();
            move |_, _| Clickable(clicks)
        });
        redraw(cx);
        let start = cx.debug_bounds("item-1").unwrap().center();
        cx.simulate_click(start, Modifiers::none());
        assert_eq!(clicks.get(), 1, "a click still clicks");
        redraw(cx);
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        redraw(cx);
        let to = start - point(px(40.), px(0.));
        cx.simulate_mouse_move(to, Some(MouseButton::Left), Modifiers::none());
        redraw(cx);
        cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::none());
        redraw(cx);
        assert_eq!(clicks.get(), 1, "the swipe's release did not click");
    }
}
