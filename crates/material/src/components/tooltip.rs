//! M3 tooltips: plain (a short label) and rich (subhead, supporting text and
//! optional actions).
//!
//! Visuals follow the Figma "Material 3 Design Kit" `Plain Tooltip` and
//! `Rich Tooltip`: a plain tooltip is inverse-surface with 4dp corners, 4/8dp
//! padding and body-small inverse-on-surface text, at most 200dp wide; a rich
//! tooltip is surface-container with 12dp corners at level 2, 12/16dp
//! padding, a title-small subhead and body-medium supporting text (both
//! on-surface-variant) and text actions 8dp apart.
//!
//! Behavior reuses GPUI Base: the tooltip popup is Base's [`gpui_base::Tooltip`]
//! (the tooltip role), placed by Base's [`gpui_base::Positioner`] (side
//! placement, flipping and viewport clamping), with Base's application-wide
//! [`gpui_base::TooltipDefaults`] show delay and grace period. Base's
//! `TooltipOverlay` provider is not used because it removes its content at
//! once on hide (no exit motion) and cannot keep a rich tooltip open while the
//! pointer moves onto its actions; this wrapper adds exactly that.
//!
//! The tooltip shows after the show delay on hover, at once on keyboard
//! focus, and hides after the grace period once neither the trigger nor (for
//! a rich tooltip) the tooltip is hovered or focused; Escape hides it. It
//! fades on the fast effects spring and moves 4dp toward its trigger on the
//! fast spatial spring (GPUI has no element scale transform, so the M3 scale
//! is carried by that movement); reduced motion keeps only the fade. The
//! trigger's accessibility node is described by the tooltip text.
use crate::{
    color::ColorRole,
    motion::{self, MotionToken},
    theme::color,
    tokens::{
        elevation,
        typography::{TypeRole, rem},
    },
};
use gpui::{prelude::*, *};
use gpui_base::{Align, Placement};
use std::time::Duration;

/// The plain tooltip's maximum width before it wraps.
pub const PLAIN_TOOLTIP_MAX_WIDTH: f32 = 200.;
/// The rich tooltip's width in the design kit (M3 allows up to 320dp).
pub const RICH_TOOLTIP_WIDTH: f32 = 312.;
/// Gap between a tooltip and its trigger.
const TOOLTIP_OFFSET: f32 = 4.;
/// How far a tooltip travels toward its trigger while it appears.
const TOOLTIP_TRAVEL: f32 = 4.;

fn typed<E: Styled>(element: E, role: TypeRole) -> E {
    let scale = role.scale();
    element
        .font_family(scale.font_family)
        .text_size(scale.font_size)
        .line_height(scale.line_height)
        .font_weight(FontWeight(scale.font_weight as f32))
}

enum Content {
    Plain(SharedString),
    Rich {
        subhead: Option<SharedString>,
        supporting: SharedString,
        actions: Vec<AnyElement>,
    },
}

/// An M3 tooltip. With [`MaterialTooltip::trigger`] it wraps that element
/// and shows on hover and keyboard focus; without one it renders the tooltip
/// container in place (a specimen).
#[derive(gpui::IntoElement)]
pub struct MaterialTooltip {
    id: ElementId,
    content: Content,
    trigger: Option<AnyElement>,
    placement: Placement,
    align: Align,
    timing: TipTiming,
    forced: Option<bool>,
}

/// How a tooltip responds to input: which inputs show it and its delays
/// (`None` uses Base's application-wide [`gpui_base::TooltipDefaults`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TipTiming {
    on_hover: bool,
    on_focus: bool,
    show_delay: Option<Duration>,
    hide_delay: Option<Duration>,
}

impl Default for TipTiming {
    fn default() -> Self {
        Self {
            on_hover: true,
            on_focus: true,
            show_delay: None,
            hide_delay: None,
        }
    }
}

impl MaterialTooltip {
    /// A plain tooltip with `text`.
    pub fn plain(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            content: Content::Plain(text.into()),
            trigger: None,
            placement: Placement::Top,
            align: Align::Center,
            timing: TipTiming::default(),
            forced: None,
        }
    }
    /// A rich tooltip with `supporting` text.
    pub fn rich(id: impl Into<ElementId>, supporting: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            content: Content::Rich {
                subhead: None,
                supporting: supporting.into(),
                actions: Vec::new(),
            },
            trigger: None,
            placement: Placement::Bottom,
            align: Align::Center,
            timing: TipTiming::default(),
            forced: None,
        }
    }
    /// A rich tooltip's title-small subhead.
    pub fn subhead(mut self, subhead: impl Into<SharedString>) -> Self {
        if let Content::Rich { subhead: slot, .. } = &mut self.content {
            *slot = Some(subhead.into());
        }
        self
    }
    /// An action of a rich tooltip; the tooltip then stays open while the
    /// pointer is over it.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        if let Content::Rich { actions, .. } = &mut self.content {
            actions.push(action.into_any_element());
        }
        self
    }
    /// The element the tooltip describes.
    pub fn trigger(mut self, trigger: impl IntoElement) -> Self {
        self.trigger = Some(trigger.into_any_element());
        self
    }
    /// The preferred side; it flips when the tooltip does not fit.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }
    /// How the tooltip lines up with its trigger along the placement side:
    /// centered (default), or flush with the trigger's start or end edge.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
    /// Whether hovering the trigger shows the tooltip (default true).
    pub fn show_on_hover(mut self, show: bool) -> Self {
        self.timing.on_hover = show;
        self
    }
    /// Whether keyboard focus in the trigger shows the tooltip (default
    /// true).
    pub fn show_on_focus(mut self, show: bool) -> Self {
        self.timing.on_focus = show;
        self
    }
    /// The hover delay before showing (default: Base's show delay).
    pub fn show_delay(mut self, delay: Duration) -> Self {
        self.timing.show_delay = Some(delay);
        self
    }
    /// The grace period before hiding once input leaves (default: Base's
    /// grace period).
    pub fn hide_delay(mut self, delay: Duration) -> Self {
        self.timing.hide_delay = Some(delay);
        self
    }
    /// Keep the tooltip shown (or hidden) regardless of input, e.g. for a
    /// captured demo.
    pub fn force_visible(mut self, visible: bool) -> Self {
        self.forced = Some(visible);
        self
    }

    fn description(&self) -> SharedString {
        match &self.content {
            Content::Plain(text) => text.clone(),
            Content::Rich {
                subhead: Some(subhead),
                supporting,
                ..
            } => format!("{subhead}. {supporting}").into(),
            Content::Rich { supporting, .. } => supporting.clone(),
        }
    }

    fn interactive(&self) -> bool {
        matches!(&self.content, Content::Rich { actions, .. } if !actions.is_empty())
    }
}

fn container(id: ElementId, content: Content) -> gpui_base::Tooltip {
    match content {
        Content::Plain(text) => typed(gpui_base::Tooltip::new(id), TypeRole::BodySmall)
            .max_w(rem(PLAIN_TOOLTIP_MAX_WIDTH))
            .px(rem(8.))
            .py(rem(4.))
            .rounded(px(4.))
            .bg(color(ColorRole::INVERSE_SURFACE))
            .text_color(color(ColorRole::INVERSE_ON_SURFACE))
            .child(text),
        Content::Rich {
            subhead,
            supporting,
            actions,
        } => {
            let has_actions = !actions.is_empty();
            gpui_base::Tooltip::new(id)
                .w(rem(RICH_TOOLTIP_WIDTH))
                .flex()
                .flex_col()
                .gap(rem(8.))
                .pb(rem(if has_actions { 8. } else { 0. }))
                .rounded(px(12.))
                .bg(color(ColorRole::SURFACE_CONTAINER))
                .shadow(elevation::level2())
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(rem(4.))
                        .pt(rem(12.))
                        .px(rem(16.))
                        .pb(rem(if has_actions { 4. } else { 12. }))
                        .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                        .when_some(subhead, |content, subhead| {
                            content.child(typed(div(), TypeRole::TitleSmall).child(subhead))
                        })
                        .child(typed(div(), TypeRole::BodyMedium).child(supporting)),
                )
                .when(has_actions, |tip| {
                    tip.child(
                        div()
                            .flex()
                            .items_center()
                            .gap(rem(8.))
                            .px(rem(4.))
                            .children(actions),
                    )
                })
        }
    }
}

/// Input that keeps a tooltip open.
#[derive(Default)]
struct TipState {
    trigger_hovered: bool,
    tip_hovered: bool,
    keyboard_focus: bool,
    visible: bool,
    epoch: usize,
    timing: TipTiming,
    /// Whether the pointer is over the trigger, shown or not.
    pointer_over: bool,
    bounds: Option<Bounds<Pixels>>,
    focus: Option<FocusHandle>,
    seen_focus: Option<FocusHandle>,
    /// The side the positioner last placed the tooltip on, which differs
    /// from the requested placement when it flipped to fit.
    placed: std::rc::Rc<std::cell::Cell<Option<Placement>>>,
}

impl TipState {
    fn wanted(&self) -> bool {
        self.trigger_hovered || self.tip_hovered || self.keyboard_focus
    }
}

/// What a change of input does to a tooltip: show now, show after the show
/// delay, hide after the grace period, or nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TipChange {
    ShowNow,
    ShowAfterDelay,
    HideAfterGrace,
    Keep,
}

pub(crate) fn tip_change(visible: bool, wanted: bool, keyboard: bool) -> TipChange {
    match (visible, wanted) {
        (false, true) if keyboard => TipChange::ShowNow,
        (false, true) => TipChange::ShowAfterDelay,
        (true, false) => TipChange::HideAfterGrace,
        _ => TipChange::Keep,
    }
}

/// Apply an input change to the tooltip `state`, scheduling the delayed
/// show or hide on the window.
fn update_tip(
    state: &Entity<TipState>,
    window: &mut Window,
    cx: &mut App,
    change: impl FnOnce(&mut TipState),
) {
    let (visible, wanted, keyboard, epoch) = state.update(cx, |tip, _| {
        change(tip);
        tip.epoch += 1;
        (tip.visible, tip.wanted(), tip.keyboard_focus, tip.epoch)
    });
    let defaults = gpui_base::TooltipDefaults::global(cx);
    let timing = state.read(cx).timing;
    let (delay, show) = match tip_change(visible, wanted, keyboard) {
        TipChange::ShowNow => {
            state.update(cx, |tip, _| tip.visible = true);
            window.refresh();
            return;
        }
        TipChange::ShowAfterDelay => (
            timing.show_delay.unwrap_or_else(|| defaults.show_delay()),
            true,
        ),
        TipChange::HideAfterGrace => (
            timing.hide_delay.unwrap_or_else(|| defaults.grace_period()),
            false,
        ),
        TipChange::Keep => return,
    };
    let state = state.downgrade();
    window
        .spawn(cx, async move |cx| {
            cx.background_executor().timer(delay).await;
            let _ = cx.update(|window, cx| {
                let Some(state) = state.upgrade() else { return };
                let current = state.read(cx);
                if current.epoch == epoch && current.wanted() == show && current.visible != show {
                    state.update(cx, |tip, _| tip.visible = show);
                    window.refresh();
                }
            });
        })
        .detach();
}

impl RenderOnce for MaterialTooltip {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Some(trigger) = self.trigger.take() else {
            return container(self.id, self.content).into_any_element();
        };
        let description = self.description();
        let interactive = self.interactive();
        let placement = self.placement;
        let align = self.align;
        let timing = self.timing;
        let id = self.id;
        let state = window.use_keyed_state((id.clone(), "tooltip"), cx, |_, cx| TipState {
            focus: Some(cx.focus_handle()),
            ..TipState::default()
        });
        if state.read(cx).timing != timing {
            state.update(cx, |tip, _| tip.timing = timing);
            // Input that no longer shows the tooltip stops holding it.
            let changed = state.clone();
            window.defer(cx, move |window, cx| {
                update_tip(&changed, window, cx, |tip| {
                    tip.trigger_hovered = tip.pointer_over && timing.on_hover;
                    tip.keyboard_focus &= timing.on_focus;
                });
            });
        }
        let focus = state
            .read(cx)
            .focus
            .clone()
            .expect("a tooltip focus handle");
        // Keyboard focus inside the trigger shows the tooltip. Containment is
        // reliable after the frame commits, so resolve it then, and only when
        // window focus has moved.
        let focused = window.focused(cx);
        if state.read(cx).seen_focus != focused {
            let observed = state.clone();
            let container_focus = focus.clone();
            window.defer(cx, move |window, cx| {
                let keyboard = timing.on_focus
                    && container_focus.contains_focused(window, cx)
                    && window.last_input_was_keyboard();
                observed.update(cx, |tip, _| tip.seen_focus = focused);
                if keyboard != observed.read(cx).keyboard_focus {
                    update_tip(&observed, window, cx, |tip| tip.keyboard_focus = keyboard);
                }
            });
        }
        let visible = self.forced.unwrap_or(state.read(cx).visible);
        let target = if visible { 1. } else { 0. };
        let fade = motion::token_value(
            (id.clone(), "tooltip-fade"),
            target,
            MotionToken::FastEffects,
            window,
            cx,
        );
        let travel = motion::token_value(
            (id.clone(), "tooltip-travel"),
            target,
            MotionToken::FastSpatial,
            window,
            cx,
        );
        let shown = visible || fade > 0.;
        let bounds = state.read(cx).bounds;

        let hover_state = state.clone();
        let escape_state = state.clone();
        let measure = state.clone();
        let wrapper = div()
            .id(id.clone())
            .relative()
            .track_focus(&focus)
            .tab_stop(false)
            .aria_description(description)
            .on_hover(move |hovered, window, cx| {
                let hovered = *hovered;
                update_tip(&hover_state, window, cx, |tip| {
                    tip.pointer_over = hovered;
                    tip.trigger_hovered = hovered && tip.timing.on_hover;
                });
            })
            .on_key_down(move |event, window, cx| {
                if event.keystroke.key == "escape" && escape_state.read(cx).visible {
                    escape_state.update(cx, |tip, _| {
                        tip.visible = false;
                        tip.keyboard_focus = false;
                        tip.trigger_hovered = false;
                        tip.epoch += 1;
                    });
                    window.refresh();
                }
            })
            .child(trigger)
            .child(
                canvas(
                    move |bounds, window, cx| {
                        if measure.read(cx).bounds != Some(bounds) {
                            measure.update(cx, |tip, _| tip.bounds = Some(bounds));
                            // A shown tooltip follows a moved trigger in the
                            // next frame; settled bounds stop the refreshes.
                            if shown {
                                window.refresh();
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

        let Some(bounds) = bounds.filter(|_| shown) else {
            return div()
                .relative()
                .flex_shrink_0()
                .child(wrapper)
                .into_any_element();
        };
        let tip_state = state.clone();
        // The tooltip starts `TOOLTIP_TRAVEL` farther from its trigger, on
        // the side it is actually shown: a flipped tooltip travels away from
        // the trigger, not into it.
        let placed = state.read(cx).placed.clone();
        let side = placed.get().unwrap_or(placement);
        let away = rem(TOOLTIP_TRAVEL * (1. - travel));
        let tip = div()
            .id("tooltip-motion")
            .when(cfg!(test), |tip| {
                tip.debug_selector(|| "tooltip-popup".into())
            })
            .relative()
            .opacity(fade.clamp(0., 1.))
            .map(|tip| match side {
                Placement::Top => tip.top(-away),
                Placement::Bottom => tip.top(away),
                Placement::Left => tip.left(-away),
                Placement::Right => tip.left(away),
            })
            .when(interactive, |tip| {
                tip.on_hover(move |hovered, window, cx| {
                    let hovered = *hovered;
                    update_tip(&tip_state, window, cx, |tip| tip.tip_hovered = hovered);
                })
            })
            .child(container(
                ElementId::NamedChild(std::sync::Arc::new(id), "popup".into()),
                self.content,
            ))
            .child({
                // The side is known only once positioned (before this
                // prepaints): draw again when it changed.
                let placed = placed.clone();
                canvas(
                    move |_, window, _| {
                        if placed.get().is_some_and(|placed| placed != side) {
                            window.request_animation_frame();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_0()
            });
        let positioner = gpui_base::Positioner::side(bounds)
            .placement(placement)
            .align(align)
            .offset(rem(TOOLTIP_OFFSET).to_pixels(window.rem_size()))
            .on_position(move |resolved| placed.set(resolved.placement))
            .child(tip);
        let positioner = if interactive {
            positioner.occlude()
        } else {
            positioner
        };
        div()
            .relative()
            .flex_shrink_0()
            .child(wrapper)
            .child(deferred(crate::scope::current_scope(positioner)).with_priority(200))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::{Context, TestAppContext, VisualTestContext};
    use std::time::Duration;

    #[test]
    fn keyboard_focus_shows_at_once_and_hover_waits() {
        assert_eq!(tip_change(false, true, true), TipChange::ShowNow);
        assert_eq!(tip_change(false, true, false), TipChange::ShowAfterDelay);
        assert_eq!(tip_change(true, false, false), TipChange::HideAfterGrace);
        assert_eq!(tip_change(true, true, false), TipChange::Keep);
        assert_eq!(tip_change(false, false, false), TipChange::Keep);
    }

    struct Harness;
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(16.));
            div().p(px(100.)).child(
                MaterialTooltip::plain("tip", "Save")
                    .trigger(div().debug_selector(|| "trigger".into()).size(px(40.))),
            )
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
    }

    fn tooltip_shown(cx: &mut VisualTestContext) -> bool {
        cx.debug_bounds("tooltip-popup").is_some()
    }

    #[gpui::test]
    fn hover_shows_after_the_delay_and_hides_after_the_grace(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
            gpui_base::TooltipDefaults::new()
                .with_show_delay(Duration::from_millis(500))
                .with_grace_period(Duration::from_millis(300))
                .install(cx);
        });
        let (_, cx) = cx.add_window_view(|_, _| Harness);
        draw(cx);
        let trigger = cx.debug_bounds("trigger").unwrap();
        cx.simulate_mouse_move(trigger.center(), None, Modifiers::default());
        draw(cx);
        assert!(!tooltip_shown(cx));
        cx.executor().advance_clock(Duration::from_millis(500));
        draw(cx);
        draw(cx);
        assert!(tooltip_shown(cx), "shown after the show delay");
        let popup = cx.debug_bounds("tooltip-popup").unwrap();
        assert!(
            popup.bottom() <= trigger.top(),
            "a plain tooltip sits above"
        );
        cx.simulate_mouse_move(point(px(5.), px(5.)), None, Modifiers::default());
        draw(cx);
        assert!(tooltip_shown(cx), "kept during the grace period");
        cx.executor().advance_clock(Duration::from_millis(300));
        draw(cx);
        cx.executor().advance_clock(Duration::from_secs(1));
        draw(cx);
        draw(cx);
        assert!(!tooltip_shown(cx), "hidden after the grace period");
    }

    struct OptionsHarness {
        hover: bool,
        align: Align,
    }
    impl Render for OptionsHarness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(16.));
            div().p(px(100.)).child(
                MaterialTooltip::plain("tip", "Save to favorites")
                    .align(self.align)
                    .show_on_hover(self.hover)
                    .show_delay(Duration::from_millis(50))
                    .hide_delay(Duration::from_millis(20))
                    .trigger(div().debug_selector(|| "trigger".into()).size(px(40.))),
            )
        }
    }

    #[gpui::test]
    fn options_set_the_delays_alignment_and_hover_response(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
            gpui_base::TooltipDefaults::new()
                .with_show_delay(Duration::from_secs(5))
                .install(cx);
        });
        let (view, cx) = cx.add_window_view(|_, _| OptionsHarness {
            hover: false,
            align: Align::Start,
        });
        draw(cx);
        let trigger = cx.debug_bounds("trigger").unwrap();
        cx.simulate_mouse_move(trigger.center(), None, Modifiers::default());
        cx.executor().advance_clock(Duration::from_millis(200));
        draw(cx);
        assert!(!tooltip_shown(cx), "hover does not show it");
        view.update(cx, |h, cx| {
            h.hover = true;
            cx.notify();
        });
        draw(cx);
        cx.simulate_mouse_move(
            trigger.center() + point(px(1.), px(1.)),
            None,
            Modifiers::default(),
        );
        draw(cx);
        // The tooltip's own show delay replaces the 5s default.
        cx.executor().advance_clock(Duration::from_millis(50));
        draw(cx);
        draw(cx);
        assert!(tooltip_shown(cx), "shown after its own delay");
        let popup = cx.debug_bounds("tooltip-popup").unwrap();
        assert_eq!(popup.left(), trigger.left(), "start-aligned");
        cx.simulate_mouse_move(point(px(5.), px(5.)), None, Modifiers::default());
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(20));
        draw(cx);
        cx.executor().advance_clock(Duration::from_secs(1));
        draw(cx);
        draw(cx);
        assert!(!tooltip_shown(cx), "hidden after its own hide delay");
    }

    /// A trigger against the window's top edge: a top tooltip flips below.
    struct EdgeHarness;
    impl Render for EdgeHarness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(crate::tokens::typography::BASE_PX));
            div().pl(px(100.)).child(
                MaterialTooltip::plain("tip", "Save")
                    .trigger(div().debug_selector(|| "trigger".into()).size(px(40.))),
            )
        }
    }

    #[gpui::test]
    fn a_flipped_tooltip_travels_away_from_its_trigger(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            gpui_base::TooltipDefaults::new()
                .with_show_delay(Duration::ZERO)
                .install(cx);
        });
        let (_, cx) = cx.add_window_view(|_, _| EdgeHarness);
        draw(cx);
        let trigger = cx.debug_bounds("trigger").unwrap();
        cx.simulate_mouse_move(trigger.center(), None, Modifiers::default());
        // Mid-entrance (the motion clock does not advance between draws).
        for _ in 0..3 {
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
        }
        let popup = cx
            .debug_bounds("tooltip-popup")
            .expect("the tooltip is shown");
        assert!(
            popup.top() >= trigger.bottom() + px(TOOLTIP_OFFSET),
            "a tooltip flipped below its trigger moved up into it: {popup:?} vs {trigger:?}"
        );
    }
}
