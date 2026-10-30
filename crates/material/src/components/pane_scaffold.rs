//! Canonical layouts: list-detail and supporting pane, the panes adapting to
//! the scaffold's own width and moving on Compose's pane motion, and the feed.
//!
//! Source: only m3e lists these (the Figma kit and md3.io do not).
//! `packages/m3e-css/src/layout.css`, `m3e-react/src/layout/PaneScaffold.tsx`
//! and `FeedLayout.tsx`; rules in [`super::pane_logic`]. Timing is this crate's
//! default spatial token, which is what Compose's pane spring resolves to
//! under each motion scheme. The feed's values are m3.material.io's through
//! m3e: 16/24dp margins, 8dp between items. Its 240dp column minimum is m3e's
//! own default, since no reference defines one.
use super::adaptive::{WindowSizeClass, WindowWidthClass, sized_content};
use super::pane_logic::*;
use crate::{
    color::ColorRole,
    motion::{self, MotionToken},
    theme::color,
    tokens::typography::{BASE_PX, rem},
};
use gpui::{prelude::*, *};
use std::rc::Rc;

#[cfg(not(target_family = "wasm"))]
use std::time::Instant;
#[cfg(target_family = "wasm")]
use web_time::Instant;

/// Compose `PaneMotionDefaults.DelayedAnimationSpec`'s delayed ratio.
const DELAYED_RATIO: f32 = 0.1;
/// The drag handle's touch target (Compose
/// `minimumInteractiveComponentSize`), in dp.
const HANDLE_TARGET: f32 = 48.;
/// The drag handle's bar at rest (`DragHandleTokens`, through m3e), in dp.
const HANDLE_BAR: (f32, f32) = (4., 48.);
/// The bar while pressed or dragged, in dp.
const HANDLE_BAR_ACTIVE: (f32, f32) = (12., 52.);
/// Travel below which a press and release on the handle is a click, in px.
const CLICK_SLOP: f32 = 3.;
/// The active bar's corner (medium), in dp.
const HANDLE_BAR_ACTIVE_RADIUS: f32 = 12.;

#[derive(Clone, Copy)]
struct PaneRun {
    from: PaneRect,
    to: PaneRect,
    delayed: bool,
    started: Instant,
}

/// Transient motion and split state. The history stays in the navigator.
struct ScaffoldState {
    value: Option<ThreePaneValue>,
    size: (f32, f32),
    targets: [Option<PaneRect>; 3],
    runs: [Option<PaneRun>; 3],
    first_pane_width: Option<f32>,
    dragging: Option<f32>,
    /// Where the handle was pressed: a release there is a click.
    pressed_at: f32,

    history_len: Option<usize>,
    focus_stack: Vec<FocusHandle>,
    pane_focus: [FocusHandle; 3],
    handle_focus: FocusHandle,
}

impl ScaffoldState {
    fn sample(&self, role: usize, now: Instant) -> (Option<PaneRect>, bool) {
        let Some(run) = self.runs[role] else {
            return (self.targets[role], false);
        };
        let curve = motion::curve(MotionToken::DefaultSpatial);
        let total = curve.duration().as_secs_f32().max(f32::EPSILON);
        let mut elapsed = now.saturating_duration_since(run.started).as_secs_f32() / total;
        if run.delayed {
            elapsed = (elapsed - DELAYED_RATIO) / (1. - DELAYED_RATIO);
        }
        if elapsed >= 1. {
            return (Some(run.to), false);
        }
        let t = (curve.ease())(elapsed.max(0.));
        (Some(run.from.lerp(&run.to, t)), true)
    }
}

/// A list-detail or supporting pane layout around caller-owned panes.
///
/// Panes adapt to the scaffold's own width, so a layout inside a navigation
/// suite adapts to the room it has. Give it a definite height. Each pane scrolls
/// on its own, and focus follows the destination and returns on back.
///
/// The navigator is controlled: call its `navigate_to` and `navigate_back` in
/// your handlers, and `notify` its entity.
#[derive(gpui::IntoElement)]
pub struct PaneScaffold<K: Clone + PartialEq + 'static> {
    id: ElementId,
    navigator: Entity<PaneNavigator<K>>,
    order: [PaneRole; 3],
    rule: PaneRule,
    size_class: Option<WindowSizeClass>,
    panes: [Option<(SharedString, AnyElement)>; 3],
    preferred_widths: [Option<f32>; 3],
    handle: Option<SharedString>,
    anchors: Rc<[PaneAnchor]>,
}

impl<K: Clone + PartialEq + 'static> PaneScaffold<K> {
    fn new(
        id: impl Into<ElementId>,
        navigator: &Entity<PaneNavigator<K>>,
        order: [PaneRole; 3],
    ) -> Self {
        Self {
            id: id.into(),
            navigator: navigator.clone(),
            order,
            rule: PaneRule::default(),
            size_class: None,
            panes: [None, None, None],
            preferred_widths: [None; 3],
            handle: None,
            anchors: Rc::from(DEFAULT_PANE_ANCHORS),
        }
    }

    /// A list-detail layout: list, detail, then the optional extra pane.
    pub fn list_detail(id: impl Into<ElementId>, navigator: &Entity<PaneNavigator<K>>) -> Self {
        Self::new(id, navigator, LIST_DETAIL_ORDER)
    }

    /// A supporting pane layout: main, supporting, then the optional extra pane.
    pub fn supporting_pane(id: impl Into<ElementId>, navigator: &Entity<PaneNavigator<K>>) -> Self {
        Self::new(id, navigator, SUPPORTING_PANE_ORDER)
    }

    /// Supply a pane; `label` names its region for assistive technology.
    pub fn pane(
        mut self,
        role: PaneRole,
        label: impl Into<SharedString>,
        content: impl IntoElement,
    ) -> Self {
        self.panes[role.index()] = Some((label.into(), content.into_any_element()));
        self
    }

    /// How many panes may sit side by side.
    pub fn rule(mut self, rule: PaneRule) -> Self {
        self.rule = rule;
        self
    }

    /// Classify this size instead of the scaffold's own, for a layout shown
    /// smaller than the window it stands for, as previews are. `None` reads
    /// the scaffold's width.
    pub fn size_class(mut self, size_class: Option<WindowSizeClass>) -> Self {
        self.size_class = size_class;
        self
    }

    /// A pane's preferred width in dp, overriding the directive's.
    pub fn preferred_width(mut self, role: PaneRole, width: f32) -> Self {
        self.preferred_widths[role.index()] = (width.is_finite() && width >= 0.).then_some(width);
        self
    }

    /// Show a drag handle between two panes; `label` is its accessible name.
    /// It takes a drag, the arrows, Home, End, and Enter for the next anchor.
    pub fn drag_handle(mut self, label: impl Into<SharedString>) -> Self {
        self.handle = Some(label.into());
        self
    }

    /// The positions the handle settles at.
    pub fn anchors(mut self, anchors: impl IntoIterator<Item = PaneAnchor>) -> Self {
        self.anchors = anchors.into_iter().collect();
        self
    }
}

const PANE_SELECTORS: [&str; 3] = ["pane-primary", "pane-secondary", "pane-tertiary"];

/// Move to a settled handle position: the first pane's width from its center.
fn settle_handle(
    state: &Entity<ScaffoldState>,
    position: f32,
    spacer: f32,
    window_notify: &mut App,
) {
    state.update(window_notify, |state, cx| {
        state.first_pane_width = Some((position - (spacer / 2.).trunc()).max(0.));
        state.dragging = None;
        cx.notify();
    });
}

impl<K: Clone + PartialEq + 'static> RenderOnce for PaneScaffold<K> {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let Self {
            id,
            navigator,
            order,
            rule,
            size_class,
            panes,
            preferred_widths,
            handle,
            anchors,
        } = self;
        let builder_id = id.clone();
        div()
            .id(id)
            .size_full()
            .overflow_hidden()
            .child(sized_content(
                (builder_id.clone(), "panes"),
                move |bounds, window, cx| {
                    let size = bounds.size;
                    let (width, height) = (f32::from(size.width), f32::from(size.height));
                    let directive = pane_directive(
                        size_class.unwrap_or_else(|| WindowSizeClass::of_size(size)),
                        rule,
                    );
                    navigator.update(cx, |navigator, cx| {
                        if navigator.set_directive(directive) {
                            cx.notify();
                        }
                    });
                    let (value, strategies, history_len, destination) = {
                        let navigator = navigator.read(cx);
                        (
                            navigator.value(),
                            *navigator.strategies(),
                            navigator.history().len(),
                            navigator.current().map(|current| current.pane),
                        )
                    };
                    let state =
                        window.use_keyed_state((builder_id.clone(), "pane-state"), cx, |_, cx| {
                            ScaffoldState {
                                value: None,
                                size: (0., 0.),
                                targets: [None; 3],
                                runs: [None; 3],
                                first_pane_width: None,
                                dragging: None,
                                pressed_at: 0.,

                                history_len: None,
                                focus_stack: Vec::new(),
                                pane_focus: [
                                    cx.focus_handle(),
                                    cx.focus_handle(),
                                    cx.focus_handle(),
                                ],
                                handle_focus: cx.focus_handle().tab_stop(true),
                            }
                        });
                    let now = cx.background_executor().now();
                    let reduced = cx.reduce_motion();
                    let (dragging, first_pane_width) = {
                        let state = state.read(cx);
                        (state.dragging, state.first_pane_width)
                    };
                    let layout = three_pane_layout(&PaneLayoutInput {
                        width,
                        height,
                        directive,
                        value,
                        order: &order,
                        strategies: &strategies,
                        present: [panes[0].is_some(), panes[1].is_some(), panes[2].is_some()],
                        preferred_widths,
                        dragging_offset: dragging,
                        first_pane_width,
                    });
                    state.update(cx, |state, _| {
                        let origin: [Option<PaneRect>; 3] =
                            std::array::from_fn(|role| state.sample(role, now).0);
                        let resized = state.size != (width, height);
                        match state.value {
                            // The first frame, or a resize: the panes are simply there.
                            Some(previous) if !resized && !reduced => {
                                if previous != value {
                                    let motions = pane_motions(previous, value, &order);
                                    let transitions = pane_transitions(
                                        &motions,
                                        &order,
                                        &origin,
                                        &layout.panes,
                                        width,
                                    );
                                    for (role, transition) in transitions.iter().enumerate() {
                                        state.runs[role] = transition.map(|transition| PaneRun {
                                            from: transition.from,
                                            to: transition.to,
                                            delayed: transition.motion.delayed(),
                                            started: now,
                                        });
                                    }
                                } else if dragging.is_some() {
                                    state.runs = [None; 3];
                                } else {
                                    for (role, from) in origin.iter().enumerate() {
                                        if let (Some(from), Some(to)) = (*from, layout.panes[role])
                                            && state.targets[role] != Some(to)
                                        {
                                            state.runs[role] = Some(PaneRun {
                                                from,
                                                to,
                                                delayed: false,
                                                started: now,
                                            });
                                        }
                                    }
                                }
                            }
                            _ => state.runs = [None; 3],
                        }
                        state.value = Some(value);
                        state.size = (width, height);
                        state.targets = layout.panes;
                    });

                    // Focus follows the destination, and back returns it.
                    let previous_len = state.read(cx).history_len;
                    if previous_len != Some(history_len) {
                        let target =
                            destination.map(|role| state.read(cx).pane_focus[role.index()].clone());
                        let grew = previous_len.is_some_and(|len| history_len > len);
                        let shrank = previous_len.is_some_and(|len| history_len < len);
                        let focused = window.focused(cx);
                        let restore = state.update(cx, |state, _| {
                            state.history_len = Some(history_len);
                            if grew {
                                state.focus_stack.extend(focused);
                                None
                            } else if shrank {
                                state.focus_stack.pop()
                            } else {
                                None
                            }
                        });
                        let next = if grew { target } else { restore.or(target) };
                        if let Some(next) = next {
                            window.defer(cx, move |window, cx| next.focus(window, cx));
                        }
                    }

                    let mut running = false;
                    let mut shown = [None; 3];
                    {
                        let current = state.read(cx);
                        for (slot, role) in order.into_iter().enumerate() {
                            let index = role.index();
                            let (rect, moving) = current.sample(index, now);
                            running |= moving;
                            // An exiting pane stays only while it moves.
                            let settled_hidden = layout.panes[index].is_none() && !moving;
                            shown[slot] = rect.filter(|_| !settled_hidden).map(|rect| (role, rect));
                        }
                    }

                    if !running {
                        let idle = state.read(cx).runs.iter().any(Option::is_some);
                        if idle {
                            state.update(cx, |state, _| state.runs = [None; 3]);
                        }
                    } else {
                        window.request_animation_frame();
                    }

                    let mut panes = panes;
                    let mut root = div().relative().size_full();
                    for (role, rect) in shown.into_iter().flatten() {
                        let Some((label, content)) = panes[role.index()].take() else {
                            continue;
                        };
                        let focus = state.read(cx).pane_focus[role.index()].clone();

                        if rect.width <= 0. {
                            continue;
                        }
                        let selector = PANE_SELECTORS[role.index()];
                        root = root.child(
                            div()
                                .id((builder_id.clone(), selector))
                                .debug_selector(move || selector.into())
                                .role(Role::Region)
                                .aria_label(label)
                                .track_focus(&focus)
                                .absolute()
                                .left(px(rect.x))
                                .top(px(rect.y))
                                .w(px(rect.width))
                                .h(px(rect.height))
                                .overflow_y_scroll()
                                .child(content),
                        );
                    }
                    if let (Some(label), Some(offset)) = (handle, layout.handle_offset) {
                        root = root.child(drag_handle(
                            (builder_id.clone(), "pane-handle"),
                            label,
                            offset,
                            bounds,
                            directive.horizontal_spacer,
                            anchors,
                            &state,
                            window,
                            cx,
                        ));
                    }
                    root.into_any_element()
                },
            ))
    }
}

/// A release of the handle at `x` from the scaffold's left edge: a click moves
/// to the next anchor, as Compose's does, and a drag settles on the closest.
#[allow(clippy::too_many_arguments)]
fn release_handle(
    state: &Entity<ScaffoldState>,
    anchors: &[PaneAnchor],
    width: f32,
    offset: f32,
    spacer: f32,
    x: f32,
    cx: &mut App,
) {
    let current = state.read(cx);
    if current.dragging.is_none() {
        return;
    }
    let clicked = (x - current.pressed_at).abs() < CLICK_SLOP;
    let position = if clicked {
        next_pane_anchor(anchors, width, offset)
    } else {
        closest_pane_anchor(anchors, width, x)
    };
    settle_handle(state, position.unwrap_or(x), spacer, cx);
}

/// The handle between two expanded panes.
#[allow(clippy::too_many_arguments)]
fn drag_handle(
    id: impl Into<ElementId>,
    label: SharedString,
    offset: f32,
    bounds: Bounds<Pixels>,
    spacer: f32,
    anchors: Rc<[PaneAnchor]>,
    state: &Entity<ScaffoldState>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let width = f32::from(bounds.size.width);
    let origin = bounds.origin.x;
    // The panes are laid out in reading order and the layout mirrors them, so a
    // pointer and the arrows are the only things that have to be turned around.
    let rtl = crate::scope::is_rtl(window);
    let local = move |x: Pixels| {
        let x = f32::from(x - origin).clamp(0., width.max(0.));
        if rtl { width.max(0.) - x } else { x }
    };
    let (dragged, focus) = {
        let state = state.read(cx);
        (state.dragging.is_some(), state.handle_focus.clone())
    };
    let target = f32::from(rem(HANDLE_TARGET).to_pixels(window.rem_size()));
    let (bar_w, bar_h) = if dragged {
        HANDLE_BAR_ACTIVE
    } else {
        HANDLE_BAR
    };
    let keys = {
        let state = state.clone();
        let anchors = anchors.clone();
        move |event: &KeyDownEvent, _: &mut Window, cx: &mut App| {
            let key = event.keystroke.key.as_str();
            let position = match (key, crate::scope::horizontal_arrow(key, rtl)) {
                (_, Some(-1)) => previous_pane_anchor(&anchors, width, offset),
                (_, Some(_)) | ("enter", _) => next_pane_anchor(&anchors, width, offset),
                ("home", _) => Some(0.),
                ("end", _) => Some(width),
                _ => None,
            };
            if let Some(position) = position {
                settle_handle(&state, position, spacer, cx);
                cx.stop_propagation();
            }
        }
    };
    let press = {
        let state = state.clone();
        move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
            let focus = state.read(cx).handle_focus.clone();
            focus.focus(window, cx);
            state.update(cx, |state, cx| {
                state.dragging = Some(offset);
                state.pressed_at = local(event.position.x);
                cx.notify();
            });
            cx.stop_propagation();
        }
    };
    // A press and release can arrive before the frame that would install the
    // window listeners, so the handle takes a release itself as well.
    let up = {
        let state = state.clone();
        let anchors = anchors.clone();
        move |event: &MouseUpEvent, _: &mut Window, cx: &mut App| {
            release_handle(
                &state,
                &anchors,
                width,
                offset,
                spacer,
                local(event.position.x),
                cx,
            );
        }
    };
    // Only a drag in progress listens to the whole window.
    let tracking = dragged.then(|| {
        let state = state.clone();
        canvas(
            |_, _, _| {},
            move |_, _, window, _| {
                let moving = state.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                    if phase == DispatchPhase::Capture
                        && event.dragging()
                        && moving.read(cx).dragging.is_some()
                    {
                        let x = local(event.position.x);
                        moving.update(cx, |state, cx| {
                            state.dragging = Some(x);
                            cx.notify();
                        });
                    }
                });
                window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                    if phase == DispatchPhase::Capture {
                        release_handle(
                            &state,
                            &anchors,
                            width,
                            offset,
                            spacer,
                            local(event.position.x),
                            cx,
                        );
                    }
                });
            },
        )
        .absolute()
        .inset_0()
    });
    div()
        .absolute()
        .inset_0()
        .children(tracking)
        .child(
            div()
                .id(id.into())
                .debug_selector(|| "pane-drag-handle".into())
                .role(Role::Splitter)
                .aria_label(label)
                .track_focus(&focus)
                .absolute()
                .left(px(offset - target / 2.))
                .top(px((f32::from(bounds.size.height) - target) / 2.))
                .size(px(target))
                .flex()
                .items_center()
                .justify_center()
                .cursor_col_resize()
                .rounded_full()
                .focus_visible(|ring| ring.border_2().border_color(color(ColorRole::SECONDARY)))
                .on_key_down(keys)
                .on_mouse_down(MouseButton::Left, press)
                .on_mouse_up(MouseButton::Left, up)
                .child(
                    div()
                        .w(rem(bar_w))
                        .h(rem(bar_h))
                        .when(!dragged, |bar| bar.rounded_full())
                        .when(dragged, |bar| bar.rounded(rem(HANDLE_BAR_ACTIVE_RADIUS)))
                        .bg(color(if dragged {
                            ColorRole::ON_SURFACE
                        } else {
                            ColorRole::OUTLINE
                        })),
                ),
        )
        .into_any_element()
}

/// The feed's margins below 600dp of its own width.
pub const FEED_MARGIN_COMPACT: f32 = 16.;
/// The feed's margins from 600dp.
pub const FEED_MARGIN: f32 = 24.;
/// The space between feed items.
pub const FEED_GAP: f32 = 8.;
/// The narrowest feed column.
pub const FEED_COLUMN_MIN_WIDTH: f32 = 240.;

/// How a feed lays out in a given width: use it to virtualize a long feed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeedMetrics {
    /// The number of columns.
    pub columns: u16,
    /// The side margin, in design px.
    pub margin: u16,
}

/// The feed's layout at a measured width in px, given the px of a design px at
/// the current text scale: one column below 600dp, then as many as fit.
pub fn feed_metrics(width: f32, design_px: f32) -> FeedMetrics {
    if WindowWidthClass::of(width) == WindowWidthClass::Compact {
        return FeedMetrics {
            columns: 1,
            margin: FEED_MARGIN_COMPACT as u16,
        };
    }
    let inner = width - 2. * FEED_MARGIN * design_px;
    let gap = FEED_GAP * design_px;
    let columns = ((inner + gap) / (FEED_COLUMN_MIN_WIDTH * design_px + gap)).floor();
    FeedMetrics {
        columns: if columns.is_finite() {
            columns.clamp(1., u16::MAX as f32) as u16
        } else {
            1
        },
        margin: FEED_MARGIN as u16,
    }
}

/// A feed: a grid of items, as many columns as fit.
///
/// It takes its height from its items, so put it in a scrolling parent. Columns
/// come from the feed's width in the previous frame, the window's on the first.
/// It builds every item: for a long feed, drive a virtualized list from
/// [`feed_metrics`] instead.
///
/// ```
/// use gpui_kit_material::prelude::*;
///
/// let feed = FeedLayout::new("stories");
/// # let _ = feed;
/// assert_eq!(feed_metrics(360., 1.).columns, 1);
/// assert_eq!(feed_metrics(1000., 1.).columns, 3);
/// ```
#[derive(gpui::IntoElement)]
pub struct FeedLayout {
    id: ElementId,
    items: Vec<(AnyElement, bool)>,
}

impl FeedLayout {
    /// An empty feed.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
        }
    }

    /// Add an item one column wide.
    pub fn item(mut self, item: impl IntoElement) -> Self {
        self.items.push((item.into_any_element(), false));
        self
    }

    /// Add an item two columns wide, or one where only one fits.
    pub fn wide_item(mut self, item: impl IntoElement) -> Self {
        self.items.push((item.into_any_element(), true));
        self
    }
}

impl RenderOnce for FeedLayout {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let design_px = f32::from(window.rem_size()) / BASE_PX;
        let initial = feed_metrics(f32::from(window.viewport_size().width), design_px);
        let metrics = window.use_keyed_state((self.id.clone(), "feed-metrics"), cx, |_, _| initial);
        let current = *metrics.read(cx);
        let columns = current.columns.max(1);
        div()
            .id(self.id)
            .debug_selector(|| "feed-layout".into())
            .relative()
            .w_full()
            .grid()
            .grid_cols(columns)
            .gap(rem(FEED_GAP))
            .px(rem(f32::from(current.margin)))
            .child(
                canvas(
                    move |bounds, _, cx| {
                        let measured = feed_metrics(f32::from(bounds.size.width), design_px);
                        metrics.update(cx, |metrics, cx| {
                            if *metrics != measured {
                                *metrics = measured;
                                cx.notify();
                            }
                        });
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            )
            .children(self.items.into_iter().map(|(item, wide)| {
                div()
                    .min_w_0()
                    .when(wide, |cell| cell.col_span(2.min(columns)))
                    .child(item)
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_feed_has_one_column_when_compact_then_as_many_as_fit() {
        assert_eq!(
            feed_metrics(360., 1.),
            FeedMetrics {
                columns: 1,
                margin: 16
            }
        );
        assert_eq!(
            feed_metrics(599., 1.),
            FeedMetrics {
                columns: 1,
                margin: 16
            }
        );
        // 600 - 48 = 552 holds two 240s and a gap.
        assert_eq!(feed_metrics(600., 1.).columns, 2);
        assert_eq!(feed_metrics(600., 1.).margin, 24);
        // 1000 - 48 = 952 holds three (736) but not four (984).
        assert_eq!(feed_metrics(1000., 1.).columns, 3);
        // Text at 200% doubles the column minimum.
        assert_eq!(feed_metrics(1000., 2.).columns, 1);
        assert_eq!(feed_metrics(f32::NAN, 1.).columns, 1);
        assert_eq!(feed_metrics(0., 1.).columns, 1);
    }

    struct Screen {
        navigator: Entity<PaneNavigator<u32>>,
        width: f32,
        rtl: bool,
        handle: bool,
    }

    impl Render for Screen {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(BASE_PX));
            let scaffold = PaneScaffold::list_detail("layout", &self.navigator)
                .pane(PaneRole::Secondary, "Inbox", div().child("List"))
                .pane(PaneRole::Primary, "Message", div().child("Detail"))
                .pane(PaneRole::Tertiary, "Notes", div().child("Notes"));
            let scaffold = if self.handle {
                scaffold.drag_handle("Resize panes")
            } else {
                scaffold
            };
            let mut scope = crate::scope::Scope::new("direction");
            if self.rtl {
                scope = scope.direction(LayoutDirection::Rtl);
            }
            div()
                .w(px(self.width))
                .h(px(800.))
                .child(scope.child(scaffold))
        }
    }

    fn open(
        cx: &mut gpui::TestAppContext,
        width: f32,
        reduced: bool,
    ) -> (Entity<Screen>, &mut gpui::VisualTestContext) {
        cx.update(|cx| cx.set_reduce_motion(reduced));
        let (view, cx) = cx.add_window_view(move |_, cx| Screen {
            navigator: cx.new(|_| PaneNavigator::list_detail()),
            width,
            rtl: false,
            handle: false,
        });
        draw(cx);
        (view, cx)
    }

    fn draw(cx: &mut gpui::VisualTestContext) {
        for _ in 0..3 {
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
            cx.run_until_parked();
        }
    }

    fn navigate(
        view: &Entity<Screen>,
        cx: &mut gpui::VisualTestContext,
        change: impl FnOnce(&mut PaneNavigator<u32>),
    ) {
        cx.update(|_, cx| {
            let navigator = view.read(cx).navigator.clone();
            navigator.update(cx, |navigator, cx| {
                change(navigator);
                cx.notify();
            });
            view.update(cx, |_, cx| cx.notify());
        });
        draw(cx);
    }

    #[gpui::test]
    fn the_scaffold_adapts_to_its_own_width_not_the_windows(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx, 500., true);
        assert!(cx.debug_bounds("pane-secondary").is_some(), "the list");
        assert!(cx.debug_bounds("pane-primary").is_none());
        cx.update(|_, cx| view.update(cx, |view, _| view.width = 1000.));
        draw(cx);
        let list = cx.debug_bounds("pane-secondary").unwrap();
        let detail = cx.debug_bounds("pane-primary").unwrap();
        assert_eq!(list.size.width, px(360.));
        assert_eq!(detail.left() - list.right(), px(24.));
        assert_eq!(detail.size.width, px(616.));
        assert!(cx.debug_bounds("pane-tertiary").is_none());
        cx.update(|_, cx| view.update(cx, |view, _| view.width = 1700.));
        draw(cx);
        assert!(cx.debug_bounds("pane-tertiary").is_some());
    }

    #[gpui::test]
    fn one_pane_follows_the_destination_and_back_returns(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx, 500., true);
        navigate(&view, cx, |navigator| {
            navigator.navigate_to(PaneRole::Primary, Some(7))
        });
        assert!(cx.debug_bounds("pane-primary").is_some());
        assert!(cx.debug_bounds("pane-secondary").is_none());
        navigate(&view, cx, |navigator| {
            navigator.navigate_back(BackNavigationBehavior::default());
        });
        assert!(cx.debug_bounds("pane-secondary").is_some());
        assert!(cx.debug_bounds("pane-primary").is_none());
    }

    #[gpui::test]
    fn the_panes_slide_and_the_old_pane_stays_until_it_is_out(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx, 500., false);
        navigate(&view, cx, |navigator| {
            navigator.navigate_to(PaneRole::Primary, None)
        });
        // Mid-flight both panes exist, the old one moving left of the origin
        // and the new one arriving from the right.
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(60));
        draw(cx);
        let old = cx.debug_bounds("pane-secondary").expect("still leaving");
        let new = cx.debug_bounds("pane-primary").expect("arriving");
        assert!(old.left() < px(0.), "{:?}", old.left());
        assert!(new.left() > px(0.), "{:?}", new.left());
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(3));
        draw(cx);
        assert!(cx.debug_bounds("pane-secondary").is_none());
        assert_eq!(cx.debug_bounds("pane-primary").unwrap().left(), px(0.));
    }

    #[gpui::test]
    fn reduced_motion_moves_the_panes_at_once(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx, 500., true);
        navigate(&view, cx, |navigator| {
            navigator.navigate_to(PaneRole::Primary, None)
        });
        assert!(cx.debug_bounds("pane-secondary").is_none());
        assert_eq!(cx.debug_bounds("pane-primary").unwrap().left(), px(0.));
    }

    #[gpui::test]
    fn right_to_left_mirrors_the_panes(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx, 1000., true);
        cx.update(|_, cx| view.update(cx, |view, _| view.rtl = true));
        draw(cx);
        let list = cx.debug_bounds("pane-secondary").unwrap();
        let detail = cx.debug_bounds("pane-primary").unwrap();
        assert!(list.left() > detail.left(), "the list is on the right");
        assert_eq!(list.right(), px(1000.));
    }

    #[gpui::test]
    fn right_to_left_turns_the_handles_pointer_and_arrows_around(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx, 1000., true);
        cx.update(|_, cx| {
            view.update(cx, |view, _| {
                view.handle = true;
                view.rtl = true;
            })
        });
        draw(cx);
        let handle = cx.debug_bounds("pane-drag-handle").unwrap().center();
        assert_eq!(handle.x, px(1000. - 372.));
        // Dragging toward the end of the reading direction, here leftward,
        // widens the first pane; it settles on the nearest anchor.
        cx.simulate_mouse_down(handle, MouseButton::Left, Modifiers::none());
        draw(cx);
        let target = point(px(480.), handle.y);
        cx.simulate_mouse_move(target, MouseButton::Left, Modifiers::none());
        draw(cx);
        assert_eq!(
            cx.debug_bounds("pane-drag-handle").unwrap().center().x,
            px(480.),
            "the handle stays under the pointer"
        );
        cx.simulate_mouse_up(target, MouseButton::Left, Modifiers::none());
        draw(cx);
        assert_eq!(
            cx.debug_bounds("pane-drag-handle").unwrap().center().x,
            px(500.)
        );
        // Focus by press, then: left moves on in reading order, to the 75% anchor.
        let handle = cx.debug_bounds("pane-drag-handle").unwrap().center();
        cx.simulate_click(handle, Modifiers::none());
        draw(cx);
        assert_eq!(
            cx.debug_bounds("pane-drag-handle").unwrap().center().x,
            px(240.)
        );
        cx.simulate_keystrokes("right");
        draw(cx);
        assert_eq!(
            cx.debug_bounds("pane-drag-handle").unwrap().center().x,
            px(500.),
            "right goes back toward the start"
        );
    }

    #[gpui::test]
    fn focus_moves_to_the_new_pane_and_returns_on_back(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx, 500., true);
        navigate(&view, cx, |navigator| {
            navigator.navigate_to(PaneRole::Primary, None)
        });
        let focused = cx.update(|window, cx| window.focused(cx));
        assert!(focused.is_some(), "focus followed the destination");
        navigate(&view, cx, |navigator| {
            navigator.navigate_back(BackNavigationBehavior::default());
        });
        // Nothing was focused before, so focus follows the destination back.
        let back = cx.update(|window, cx| window.focused(cx));
        assert!(back.is_some());
        assert_ne!(back, focused);
    }

    #[gpui::test]
    fn the_handle_drags_settles_on_an_anchor_and_takes_keys(cx: &mut gpui::TestAppContext) {
        let (view, cx) = open(cx, 1000., true);
        cx.update(|_, cx| view.update(cx, |view, _| view.handle = true));
        draw(cx);
        let handle = cx.debug_bounds("pane-drag-handle").unwrap().center();
        assert_eq!(handle.x, px(372.));
        cx.simulate_mouse_down(handle, MouseButton::Left, Modifiers::none());
        draw(cx);
        let target = point(px(520.), handle.y);
        cx.simulate_mouse_move(target, MouseButton::Left, Modifiers::none());
        draw(cx);
        assert_eq!(
            cx.debug_bounds("pane-drag-handle").unwrap().center().x,
            px(520.)
        );
        cx.simulate_mouse_up(target, MouseButton::Left, Modifiers::none());
        draw(cx);
        // The 50% anchor: a center of 500, so the first pane is 488 wide.
        assert_eq!(
            cx.debug_bounds("pane-drag-handle").unwrap().center().x,
            px(500.)
        );
        assert_eq!(
            cx.debug_bounds("pane-secondary").unwrap().size.width,
            px(488.)
        );
        // A click moves to the next anchor and focuses the handle for the keys,
        // even when its press and release arrive within one frame.
        let handle = cx.debug_bounds("pane-drag-handle").unwrap().center();
        cx.simulate_click(handle, Modifiers::none());
        draw(cx);
        assert_eq!(
            cx.debug_bounds("pane-drag-handle").unwrap().center().x,
            px(760.)
        );
        cx.simulate_keystrokes("left");
        draw(cx);
        assert_eq!(
            cx.debug_bounds("pane-drag-handle").unwrap().center().x,
            px(500.)
        );
        cx.simulate_keystrokes("home");
        draw(cx);
        assert_eq!(
            cx.debug_bounds("pane-primary").unwrap().size.width,
            px(1000.)
        );
    }
}
