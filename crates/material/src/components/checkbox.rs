//! M3 checkbox over GPUI Base's tri-state checkbox behavior.
//!
//! An 18dp container with 2dp corners and a 2dp outline inside a 40dp state
//! layer and a 48dp target. Selecting fills the container on the fast
//! effects spring while the check mark draws along its path on the fast
//! spatial spring; switching between checked and indeterminate morphs the
//! mark between the check and the dash. Reduced motion lands the mark at
//! once and keeps the short fill fade. Base owns activation (pointer, Space,
//! Enter), focus and the checkbox role with its mixed state.
use super::InteractionPreview;
use super::button::ButtonFocusRing;
use super::radio::selection_paint;
use crate::{
    color::ColorRole,
    motion::{self, MotionToken},
    theme::color,
    tokens::{state, typography::rem},
};
use gpui::{prelude::*, *};
pub use gpui_base::CheckboxState;
use std::rc::Rc;

/// Checkbox geometry, in dp.
const CONTAINER: f32 = 18.;
const CORNER: f32 = 2.;
const OUTLINE: f32 = 2.;
const MARK_STROKE: f32 = 2.;
const STATE_LAYER: f32 = 40.;
const TARGET: f32 = 48.;

/// The mark's three points in the 18dp container: the `check_small` and
/// `check_indeterminate_small` symbols' center lines (their 24dp boxes hold
/// the container 3dp in).
const CHECK: [(f32, f32); 3] = [(4., 9.), (7., 12.), (14.3, 4.7)];
const DASH: [(f32, f32); 3] = [(4., 9.), (9., 9.), (14., 9.)];

/// The mark's points at `morph` (0 check, 1 dash).
fn mark_points(morph: f32) -> [(f32, f32); 3] {
    let t = morph.clamp(0., 1.);
    std::array::from_fn(|i| {
        let (a, b) = (CHECK[i], DASH[i]);
        (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
    })
}

/// The polyline through `points` cut to the `progress` fraction of its
/// length (0 nothing, 1 all of it).
fn partial_polyline(points: [(f32, f32); 3], progress: f32) -> Vec<(f32, f32)> {
    let progress = if progress.is_finite() {
        progress.clamp(0., 1.)
    } else {
        0.
    };
    let segment = |a: (f32, f32), b: (f32, f32)| ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
    let lengths = [segment(points[0], points[1]), segment(points[1], points[2])];
    let mut remaining = (lengths[0] + lengths[1]) * progress;
    let mut out = vec![points[0]];
    for (i, length) in lengths.into_iter().enumerate() {
        if remaining <= 0. {
            break;
        }
        let (a, b) = (points[i], points[i + 1]);
        if remaining >= length {
            out.push(b);
            remaining -= length;
        } else {
            let f = if length > 0. { remaining / length } else { 0. };
            out.push((a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f));
            remaining = 0.;
        }
    }
    out
}

/// The colors of a checkbox in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
struct CheckboxPaint {
    /// The selected container.
    container: Rgba,
    mark: Rgba,
    /// The unselected outline at rest and while active.
    outline: Rgba,
    outline_active: Rgba,
    hover: Rgba,
    focus: Rgba,
    press: Rgba,
}

/// M3 checkbox colors: primary (error) container with an on-primary
/// (on-error) mark; an on-surface-variant (error) outline; disabled
/// containers and outlines at on-surface 38% with a surface mark. State
/// layers follow the shared selection-control policy.
fn checkbox_paint(selected: bool, enabled: bool, error: bool) -> CheckboxPaint {
    let accent = if error {
        ColorRole::ERROR
    } else {
        ColorRole::PRIMARY
    };
    let layers = selection_paint(selected, enabled, accent, error);
    let outline = selection_paint(false, enabled, accent, error);
    if !enabled {
        let disabled = state::disabled(color(ColorRole::ON_SURFACE));
        return CheckboxPaint {
            container: disabled,
            mark: color(ColorRole::SURFACE),
            outline: disabled,
            outline_active: disabled,
            hover: layers.hover,
            focus: layers.focus,
            press: layers.press,
        };
    }
    CheckboxPaint {
        container: color(accent),
        mark: color(if error {
            ColorRole::ON_ERROR
        } else {
            ColorRole::ON_PRIMARY
        }),
        outline: outline.ink,
        outline_active: outline.ink_active,
        hover: layers.hover,
        focus: layers.focus,
        press: layers.press,
    }
}

type Change = Rc<dyn Fn(&CheckboxState, &mut Window, &mut App)>;

/// Controlled tri-state M3 checkbox. Activation reports the next state:
/// unchecked and indeterminate become checked, checked becomes unchecked.
#[derive(gpui::IntoElement)]
pub struct MaterialCheckbox {
    id: ElementId,
    label: SharedString,
    state: CheckboxState,
    error: bool,
    disabled: bool,
    preview: Option<InteractionPreview>,
    focus: Option<FocusHandle>,
    tab_stop: bool,
    on_change: Option<Change>,
}

impl MaterialCheckbox {
    /// `label` is the checkbox's accessible name.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: CheckboxState::Unchecked,
            error: false,
            disabled: false,
            preview: None,
            focus: None,
            tab_stop: true,
            on_change: None,
        }
    }
    pub fn state(mut self, state: CheckboxState) -> Self {
        self.state = state;
        self
    }
    pub fn checked(self, checked: bool) -> Self {
        self.state(if checked {
            CheckboxState::Checked
        } else {
            CheckboxState::Unchecked
        })
    }
    /// Paint the error colors, e.g. for a required choice left unchecked.
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Render a static specimen in one forced interaction state. See
    /// [`InteractionPreview`].
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
    /// Use a caller-owned focus handle.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }
    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }
    /// Called with the next state when the checkbox is activated.
    pub fn on_change(
        mut self,
        f: impl Fn(&CheckboxState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

const CHECKBOX_GROUP: &str = "material-checkbox";

impl RenderOnce for MaterialCheckbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus = self.focus.clone().unwrap_or_else(|| {
            window
                .use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        let preview = self.preview;
        let disabled = self.disabled || preview.is_some_and(|p| p.is_disabled());
        let enabled = !disabled;
        let live = enabled && preview.is_none();
        let forced = |state| enabled && preview == Some(state);
        let keyboard_focus = forced(InteractionPreview::Focused)
            || (live && focus.is_focused(window) && window.last_input_was_keyboard());
        let selected = self.state != CheckboxState::Unchecked;
        // An unchecking mark keeps the shape it had, so it retracts rather
        // than morphing on its way out.
        let shape = window.use_keyed_state((self.id.clone(), "shape"), cx, |_, _| 0_f32);
        let shape_target = match self.state {
            CheckboxState::Checked => 0.,
            CheckboxState::Indeterminate => 1.,
            CheckboxState::Unchecked => *shape.read(cx),
        };
        if *shape.read(cx) != shape_target {
            shape.update(cx, |shape, _| *shape = shape_target);
        }
        let target = if selected { 1. } else { 0. };
        let (fill, draw, morph) = if preview.is_some() {
            (target, target, shape_target)
        } else {
            (
                motion::token_value(
                    (self.id.clone(), "fill"),
                    target,
                    MotionToken::FastEffects,
                    window,
                    cx,
                ),
                motion::selection_value((self.id.clone(), "mark"), target, window, cx),
                motion::selection_value((self.id.clone(), "morph"), shape_target, window, cx),
            )
        };
        let fill = fill.clamp(0., 1.);
        let paint = checkbox_paint(selected, enabled, self.error);
        let active = keyboard_focus
            || forced(InteractionPreview::Hovered)
            || forced(InteractionPreview::Pressed);
        let outline = if active {
            paint.outline_active
        } else {
            paint.outline
        };
        // The fill paints under the outline too, so the outline fades out as
        // the fill arrives instead of doubling a translucent container.
        let border = Rgba {
            a: outline.a * (1. - fill),
            ..outline
        };
        let background = Rgba {
            a: paint.container.a * fill,
            ..paint.container
        };
        let points = mark_points(morph);
        let mark_color = paint.mark;
        let mark = canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let line = partial_polyline(points, draw);
                if line.len() < 2 {
                    return;
                }
                let unit = bounds.size.width / CONTAINER;
                let at = |(x, y): (f32, f32)| bounds.origin + point(unit * x, unit * y);
                let mut path = PathBuilder::stroke(unit * MARK_STROKE);
                path.move_to(at(line[0]));
                for &p in &line[1..] {
                    path.line_to(at(p));
                }
                if let Ok(path) = path.build() {
                    window.paint_path(path, mark_color);
                }
            },
        )
        .absolute()
        .inset_0();
        let container = div()
            .when(cfg!(test), |c| {
                c.debug_selector(|| "checkbox-container".into())
            })
            .relative()
            .flex_shrink_0()
            .size(rem(CONTAINER))
            .rounded(rem(CORNER))
            .border(rem(OUTLINE))
            .border_color(border)
            .bg(background)
            .when(live && !selected && !self.error, |c| {
                let ink = paint.outline_active;
                c.group_hover(CHECKBOX_GROUP, move |style| style.border_color(ink))
            })
            .child(mark);
        let layer = div()
            .id("state-layer")
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .size(rem(STATE_LAYER))
            .rounded_full()
            .when(keyboard_focus, |layer| layer.bg(paint.focus))
            .when(forced(InteractionPreview::Hovered), |layer| {
                layer.bg(paint.hover)
            })
            .when(forced(InteractionPreview::Pressed), |layer| {
                layer.bg(paint.press)
            })
            .when(live && !keyboard_focus, |layer| {
                let (hover, press) = (paint.hover, paint.press);
                layer
                    .group_hover(CHECKBOX_GROUP, move |style| style.bg(hover))
                    .group_active(CHECKBOX_GROUP, move |style| style.bg(press))
            })
            .child(container);
        let ringed = ButtonFocusRing::new(layer, focus.clone(), enabled, rem(STATE_LAYER / 2.))
            .forced(forced(InteractionPreview::Focused));
        let on_change = self.on_change;
        gpui_base::Checkbox::new(self.id)
            .track_focus(&focus)
            .accessibility_label(self.label)
            .state(self.state)
            .disabled(disabled)
            .tab_stop(self.tab_stop && preview.is_none())
            .when_some(on_change, |checkbox, on_change| {
                checkbox.on_change(move |next, _, window, cx| on_change(&next, window, cx))
            })
            .group(CHECKBOX_GROUP)
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .size(rem(TARGET))
            .when(live, |checkbox| checkbox.cursor_pointer())
            .child(ringed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_mark_draws_along_its_path_and_morphs_to_a_dash() {
        assert_eq!(partial_polyline(CHECK, 0.), vec![CHECK[0]]);
        assert_eq!(partial_polyline(CHECK, 1.), CHECK.to_vec());
        let half = partial_polyline(CHECK, 0.2);
        assert_eq!(half.len(), 2, "the short leg draws first");
        assert!(half[1].0 < CHECK[1].0);
        assert_eq!(partial_polyline(CHECK, f32::NAN), vec![CHECK[0]]);
        assert_eq!(partial_polyline(CHECK, 1.2), CHECK.to_vec());
        assert_eq!(mark_points(0.), CHECK);
        assert_eq!(mark_points(1.), DASH);
        assert_eq!(mark_points(2.), DASH, "a morph overshoot stays a dash");
    }

    #[test]
    fn checkbox_uses_m3_roles() {
        let c = color;
        let on = checkbox_paint(true, true, false);
        assert_eq!(on.container, c(ColorRole::PRIMARY));
        assert_eq!(on.mark, c(ColorRole::ON_PRIMARY));
        let off = checkbox_paint(false, true, false);
        assert_eq!(off.outline, c(ColorRole::ON_SURFACE_VARIANT));
        assert_eq!(off.outline_active, c(ColorRole::ON_SURFACE));
        let error = checkbox_paint(true, true, true);
        assert_eq!(error.container, c(ColorRole::ERROR));
        assert_eq!(error.mark, c(ColorRole::ON_ERROR));
        assert_eq!(
            checkbox_paint(false, true, true).outline,
            c(ColorRole::ERROR)
        );
        let disabled = checkbox_paint(true, false, true);
        assert_eq!(
            disabled.container,
            state::disabled(c(ColorRole::ON_SURFACE))
        );
        assert_eq!(disabled.mark, c(ColorRole::SURFACE));
        assert_eq!(disabled.hover.a, 0.);
        assert_eq!(
            (CONTAINER, CORNER, OUTLINE, STATE_LAYER, TARGET),
            (18., 2., 2., 40., 48.)
        );
    }

    struct Harness {
        state: CheckboxState,
        changes: Vec<CheckboxState>,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().flex().items_start().child(
                div().debug_selector(|| "checkbox".into()).child(
                    MaterialCheckbox::new("box", "Accept")
                        .state(self.state)
                        .on_change(cx.listener(|this, next: &CheckboxState, _, cx| {
                            this.state = *next;
                            this.changes.push(*next);
                            cx.notify();
                        })),
                ),
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
    fn space_and_clicks_cycle_through_the_tri_state(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| Harness {
            state: CheckboxState::Indeterminate,
            changes: Vec::new(),
        });
        draw(cx);
        let target = cx.debug_bounds("checkbox").unwrap();
        assert_eq!(target.size.height, px(48.));
        assert_eq!(
            cx.debug_bounds("checkbox-container").unwrap().size,
            size(px(18.), px(18.))
        );
        cx.simulate_click(target.center(), Modifiers::default());
        draw(cx);
        view.read_with(cx, |h, _| {
            assert_eq!(h.changes, [CheckboxState::Checked], "mixed becomes checked")
        });
        // The click focused the checkbox; Space toggles it.
        let space = Keystroke::parse("space").unwrap();
        cx.simulate_event(KeyDownEvent {
            keystroke: space.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke: space });
        draw(cx);
        view.read_with(cx, |h, _| {
            assert_eq!(
                h.changes,
                [CheckboxState::Checked, CheckboxState::Unchecked]
            )
        });
    }
}
