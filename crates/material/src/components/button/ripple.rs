//! Passive pointer waves. Base owns every activation; this element never consumes input.
use gpui::*;
use gpui_base::{Easing, Keyframe, Keyframes, Timing, Transition, animate_keyframes, transition};
use std::time::Duration;

const MOTION: Duration = Duration::from_millis(400);
const LIFETIME: Duration = Duration::from_millis(450);
/// How long a held touch can go without an end before its wave is released. GPUI
/// reports a tap's end and a pan's scroll publicly, but not a system cancel of a
/// still finger; this keeps that wave from staying on screen.
pub(super) const TOUCH_HOLD_LIMIT: Duration = Duration::from_secs(5);
#[derive(Clone)]
struct Wave {
    id: u64,
    /// This wave's keyed-state ids, built once when it is pressed (or its
    /// expansion restarts), so painting allocates no per-frame ids.
    key: ElementId,
    expansion: ElementId,
    center: Point<Pixels>,
    radius: Pixels,
    released: bool,
    reduced_motion: bool,
    expansion_generation: u64,
    fade_duration: Option<Duration>,
}
#[derive(Default)]
struct RippleState {
    next: u64,
    /// The owning button's ripple id, the parent of every wave id.
    parent: Option<std::sync::Arc<ElementId>>,
    waves: Vec<Wave>,
    /// Whether a touch holds the current wave, and which touch, for its timeout.
    touch_held: bool,
    touch_generation: u64,
    /// Mirrors a held touch for press feedback beyond the wave.
    touch_pressed: Option<WeakEntity<bool>>,
}
/// The expansion clock id of `key` for one expansion generation.
fn expansion_key(key: &ElementId, generation: u64) -> ElementId {
    ElementId::NamedChild(
        std::sync::Arc::new(key.clone()),
        if generation == 0 {
            SharedString::new_static("expand")
        } else {
            format!("expand-{generation}").into()
        },
    )
}
impl RippleState {
    fn press(&mut self, bounds: Bounds<Pixels>, position: Point<Pixels>, cx: &mut Context<Self>) {
        self.next += 1;
        let parent = self
            .parent
            .clone()
            .unwrap_or_else(|| std::sync::Arc::new(ElementId::Name("ripple".into())));
        let key = ElementId::NamedChild(parent, self.next.to_string().into());
        self.waves.push(Wave {
            id: self.next,
            expansion: expansion_key(&key, 0),
            key,
            center: position - bounds.origin,
            radius: bounds.size.width.max(bounds.size.height),
            released: false,
            reduced_motion: cx.reduce_motion(),
            expansion_generation: 0,
            fade_duration: None,
        });
        cx.notify();
    }
    /// A touch came down: start its wave, and bound how long it can hold.
    fn touch_press(
        &mut self,
        bounds: Bounds<Pixels>,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.press(bounds, position, cx);
        self.touch_held = true;
        self.touch_generation += 1;
        let generation = self.touch_generation;
        self.mirror_touch(true, cx);
        let timer = cx.background_executor().timer(TOUCH_HOLD_LIMIT);
        cx.spawn(async move |this, cx| {
            timer.await;
            let _ = this.update(cx, |state, cx| {
                if state.touch_held && state.touch_generation == generation {
                    state.release(cx);
                }
            });
        })
        .detach();
    }
    fn mirror_touch(&self, on: bool, cx: &mut Context<Self>) {
        if let Some(pressed) = &self.touch_pressed {
            let _ = pressed.update(cx, |pressed, cx| {
                if *pressed != on {
                    *pressed = on;
                    cx.notify();
                }
            });
        }
    }
    /// Whether [`Self::release`] has anything to end.
    fn holds_press(&self) -> bool {
        self.touch_held || self.waves.iter().any(|wave| !wave.released)
    }
    fn release(&mut self, cx: &mut Context<Self>) {
        #[cfg(test)]
        RIPPLE_WRITES.with(|writes| writes.set(writes.get() + 1));
        if self.touch_held {
            self.touch_held = false;
            self.mirror_touch(false, cx);
        }
        let ids: Vec<_> = self
            .waves
            .iter_mut()
            .filter_map(|wave| {
                if wave.released {
                    return None;
                }
                wave.released = true;
                Some(wave.id)
            })
            .collect();
        if ids.is_empty() {
            return;
        }
        cx.notify();
        let timer = cx.background_executor().timer(LIFETIME);
        cx.spawn(async move |this, cx| {
            timer.await;
            let _ = this.update(cx, |state, cx| {
                state.waves.retain(|wave| !ids.contains(&wave.id));
                cx.notify();
            });
        })
        .detach();
    }
}
/// Ends the press `state` holds, if any. An idle ripple is only read: updating
/// an entity marks what read it as changed, so a global event that updated every
/// ripple would render every row of a scrolling list again each wheel event.
fn release_held(state: &WeakEntity<RippleState>, cx: &mut App) {
    let Some(state) = state.upgrade() else {
        return;
    };
    if state.read(cx).holds_press() {
        state.update(cx, |state, cx| state.release(cx));
    }
}
#[cfg(test)]
thread_local! {
    static WAVES_PAINTED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
thread_local! {
    static RIPPLE_WRITES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// How many times this thread's ripples wrote their state outside a press.
#[cfg(test)]
pub(crate) fn ripple_writes() -> usize {
    RIPPLE_WRITES.with(|writes| writes.get())
}

/// How many wave paints this thread has made, for tests across components.
#[cfg(test)]
pub(crate) fn waves_painted() -> usize {
    WAVES_PAINTED.with(|painted| painted.get())
}

/// Where a press at `position` starts its wave.
fn wave_origin(bounds: Bounds<Pixels>, position: Point<Pixels>, centered: bool) -> Point<Pixels> {
    if centered { bounds.center() } else { position }
}

struct PaintedWave {
    center: Point<Pixels>,
    radius: Pixels,
    opacity: f32,
}

pub(crate) struct ButtonRipple {
    child: AnyElement,
    state: Entity<RippleState>,
    enabled: bool,
    radius: Corners<AbsoluteLength>,
    border: Edges<AbsoluteLength>,
    color: Rgba,
    waves: Vec<PaintedWave>,
    centered: bool,
    extension: Edges<Rems>,
}
impl ButtonRipple {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        child: impl IntoElement,
        id: ElementId,
        mounted: bool,
        enabled: bool,
        radius: Corners<AbsoluteLength>,
        border: Edges<AbsoluteLength>,
        color: Rgba,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let state =
            window.use_keyed_state((id.clone(), "ripple"), cx, |_, _| RippleState::default());
        let reduced_motion = cx.reduce_motion();
        // One update reconciles mounting and media; an idle, initialized ripple
        // is only read, so drawing it writes nothing and allocates nothing.
        let settled = {
            let state = state.read(cx);
            state.parent.is_some() && state.waves.is_empty()
        };
        let count = if settled {
            0
        } else {
            state.update(cx, |state, _| {
                #[cfg(test)]
                RIPPLE_WRITES.with(|writes| writes.set(writes.get() + 1));
                if !mounted {
                    state.waves.clear();
                }
                if state.parent.is_none() {
                    state.parent = Some(std::sync::Arc::new((id.clone(), "ripple-wave").into()));
                }
                for wave in &mut state.waves {
                    if wave.reduced_motion && !reduced_motion {
                        wave.expansion_generation += 1;
                        wave.expansion = expansion_key(&wave.key, wave.expansion_generation);
                    }
                    wave.reduced_motion = reduced_motion;
                    if wave.released && wave.fade_duration.is_none() {
                        wave.fade_duration = Some(if reduced_motion {
                            Duration::ZERO
                        } else {
                            MOTION
                        });
                    }
                }
                state.waves.len()
            })
        };
        let mut waves = Vec::new();
        if count > 0 {
            let easing = Easing::cubic_bezier(0.4, 0., 0.2, 1.).expect("valid ripple easing");
            let frames = Keyframes::try_new([Keyframe::new(0., 0.), Keyframe::new(1., 1.)])
                .expect("valid ripple keyframes");
            let timing = Timing::new(MOTION).ease(easing.clone());
            waves.reserve_exact(count);
            for index in 0..count {
                // Read one wave at a time; its ids are shared, not rebuilt.
                let wave = state.read(cx).waves[index].clone();
                let (key, expansion) = (wave.key.clone(), wave.expansion.clone());
                let scale = animate_keyframes(expansion, &frames, timing.clone(), window, cx).value;
                // The complete reference stylesheet retains opacity in transition-property
                // under reduced motion. A running fade therefore keeps its original
                // duration; only a newly released wave adopts the current media duration.
                // An unpainted down/up pair adopts its first target (zero) directly.
                let opacity = transition(
                    (key, "opacity"),
                    if wave.released { 0. } else { 0.1 },
                    Transition::new(wave.fade_duration.unwrap_or(MOTION))
                        .reduced_motion(gpui_base::ReducedMotion::Continue)
                        .easing(easing.clone()),
                    window,
                    cx,
                );
                waves.push(PaintedWave {
                    center: wave.center,
                    radius: wave.radius * scale,
                    opacity,
                });
            }
        }
        Self {
            child: child.into_any_element(),
            state,
            enabled: enabled && mounted,
            radius,
            border,
            color,
            waves,
            centered: false,
            extension: Edges::all(rems(0.)),
        }
    }

    /// Start every wave at the center, as selection controls do.
    pub(crate) fn centered(mut self) -> Self {
        self.centered = true;
        self
    }

    /// Also start waves for presses this far outside the ripple's own
    /// bounds, for a control whose target is larger than its state layer.
    pub(crate) fn hit_extension(mut self, extension: Edges<Rems>) -> Self {
        self.extension = extension;
        self
    }

    /// Mirror a held touch into `pressed`, for press feedback beyond the
    /// wave. Mouse presses stay with the caller's own handlers.
    pub(crate) fn touch_pressed(self, pressed: Entity<bool>, cx: &mut App) -> Self {
        let current = self.state.read(cx).touch_pressed.as_ref();
        if current.map(|current| current.entity_id()) != Some(pressed.entity_id()) {
            let pressed = pressed.downgrade();
            self.state
                .update(cx, |state, _| state.touch_pressed = Some(pressed));
        }
        self
    }
}
impl IntoElement for ButtonRipple {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for ButtonRipple {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Hitbox {
        // Base occludes earlier pointer targets. Register this passive observer
        // above its own child, while later siblings still occlude both together.
        self.child.prepaint(window, cx);
        let rem_size = window.rem_size();
        let extended = (self.extension != Edges::all(rems(0.))).then(|| {
            let edges = self.extension.map(|edge| edge.to_pixels(rem_size));
            Bounds::new(
                bounds.origin - point(edges.left, edges.top),
                size(
                    bounds.size.width + edges.left + edges.right,
                    bounds.size.height + edges.top + edges.bottom,
                ),
            )
        });
        window.insert_hitbox(extended.unwrap_or(bounds), HitboxBehavior::Normal)
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        hitbox: &mut Hitbox,
        window: &mut Window,
        cx: &mut App,
    ) {
        // Reference ripple z-index:0 covers the static label/icon as well as
        // the background; the separate focus outline is painted after us.
        self.child.paint(window, cx);
        let rtl = window.layout_direction().is_rtl();
        let border = self.border.to_pixels(window.rem_size());
        let border = if rtl { border.flip_x() } else { border };
        // CSS absolute inset:0 uses the padding box, while wave coordinates and
        // diameter come from the outer DOM bounds. Preserve its border-width
        // offset, including the inherited (not border-subtracted) corner radius.
        let clip = Bounds::new(
            bounds.origin + point(border.left, border.top),
            size(
                bounds.size.width - border.left - border.right,
                bounds.size.height - border.top - border.bottom,
            ),
        );
        let radius = self.radius.to_pixels(window.rem_size());
        let radius = if rtl { radius.flip_x() } else { radius };
        for wave in &self.waves {
            if wave.opacity > 0.
                && let Some(path) = super::ripple_clip::paint_path(
                    clip,
                    radius,
                    clip.origin + wave.center,
                    wave.radius,
                )
            {
                let mut color = self.color;
                color.a *= wave.opacity;
                #[cfg(test)]
                {
                    tests::record_paint(color, radius, border);
                    WAVES_PAINTED.with(|painted| painted.set(painted.get() + 1));
                }
                window.paint_path(path, color);
            }
        }
        let state = self.state.downgrade();
        let enabled = self.enabled;
        let touch_hitbox = hitbox.clone();
        let touch_state = self.state.downgrade();
        let centered = self.centered;
        // GPUI offers each new touch as an unclaimed drag. Observing the offer
        // without claiming it leaves tap and scroll recognition untouched. A tap
        // ends in the mouse-up below, a pan in the scroll below, and a claimed drag
        // here.
        window.on_mouse_event(move |event: &TouchDragEvent, phase, window, cx| {
            if phase != DispatchPhase::Bubble {
                return;
            }
            match event.phase {
                TouchPhase::Started => {
                    if enabled && !window.default_prevented() && touch_hitbox.is_hovered(window) {
                        let position = wave_origin(bounds, event.start_position, centered);
                        let _ = touch_state
                            .update(cx, |state, cx| state.touch_press(bounds, position, cx));
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => release_held(&touch_state, cx),
                _ => {}
            }
        });
        // What only ends a press is installed while one is held, as a press draws
        // the ripple again. A list's held rows carry every listener on every frame,
        // and every wheel event calls every wheel listener. A press these outrun
        // still ends: a pan keeps scrolling and a mouse button comes up.
        if self.state.read(cx).holds_press() {
            let release_on_long_press = self.state.downgrade();
            window.on_mouse_event(move |event: &LongPressEvent, phase, _, cx| {
                if phase == DispatchPhase::Capture
                    && matches!(event.phase, TouchPhase::Ended | TouchPhase::Cancelled)
                {
                    release_held(&release_on_long_press, cx);
                }
            });
            let release_on_scroll = self.state.downgrade();
            window.on_mouse_event(move |_: &ScrollEvent, phase, _, cx| {
                if phase == DispatchPhase::Capture
                    && let Some(state) = release_on_scroll.upgrade()
                    && state.read(cx).touch_held
                {
                    state.update(cx, |state, cx| state.release(cx));
                }
            });
            let release_on_exit = self.state.downgrade();
            window.on_mouse_event(move |_: &MouseExitEvent, phase, _, cx| {
                if phase == DispatchPhase::Capture {
                    release_held(&release_on_exit, cx);
                }
            });
        }
        let hitbox = hitbox.clone();
        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
            // Every pointer button starts a wave, matching the reference's
            // captured right pointerdown; a handled (prevented) press does not.
            if enabled
                && phase == DispatchPhase::Bubble
                && !window.default_prevented()
                && hitbox.is_hovered(window)
                && !window.last_input_was_touch()
            {
                let position = wave_origin(bounds, event.position, centered);
                let _ = state.update(cx, |state, cx| state.press(bounds, position, cx));
            }
        });
        let state = self.state.downgrade();
        window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
            if phase == DispatchPhase::Capture {
                release_held(&state, cx);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::ButtonRipple;
    use crate::{components::button::MaterialButton, tokens::typography::rem};
    use gpui::prelude::*;
    use gpui::{
        Context, IntoElement, Modifiers, MouseButton, MouseExitEvent, Pixels, Point, Render,
        TestAppContext, VisualTestContext, Window, div, point, px, rgba,
    };
    use gpui_base::Easing;
    use std::time::Duration;
    use std::{cell::RefCell, rc::Rc};
    type PaintSample = (gpui::Rgba, gpui::Corners<Pixels>, gpui::Edges<Pixels>);
    thread_local! {
        static PAINT_SAMPLES: RefCell<Vec<PaintSample>> = const { RefCell::new(Vec::new()) };
    }
    pub(super) fn record_paint(
        color: gpui::Rgba,
        corners: gpui::Corners<Pixels>,
        border: gpui::Edges<Pixels>,
    ) {
        PAINT_SAMPLES.with(|samples| samples.borrow_mut().push((color, corners, border)));
    }

    struct StyledHarness {
        opacity: f32,
        hidden: bool,
    }
    impl Render for StyledHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let mut button = MaterialButton::new("styled-ripple")
                .child("Styled")
                .text_color(rgba(0x33669980))
                .opacity(self.opacity)
                .debug_selector(|| "styled-ripple".into());
            let style = button.style();
            style.visibility = Some(if self.hidden {
                gpui::Visibility::Hidden
            } else {
                gpui::Visibility::Visible
            });
            style.corner_radii.top_left = Some(px(3.).into());
            style.corner_radii.top_right = Some(px(4.).into());
            style.corner_radii.bottom_right = Some(px(5.).into());
            style.corner_radii.bottom_left = Some(px(6.).into());
            style.border_widths.top = Some(px(1.).into());
            style.border_widths.right = Some(px(2.).into());
            style.border_widths.bottom = Some(px(3.).into());
            style.border_widths.left = Some(px(4.).into());
            div().p(px(20.)).child(button)
        }
    }

    #[gpui::test]
    fn ripple_paint_uses_final_caller_style_and_hides_existing_waves(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| StyledHarness {
            opacity: 0.5,
            hidden: false,
        });
        draw(cx);
        let center = cx.debug_bounds("styled-ripple").unwrap().center();
        cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
        PAINT_SAMPLES.with(|samples| samples.borrow_mut().clear());
        draw(cx);
        let (color, corners, border) = PAINT_SAMPLES.with(|samples| {
            *samples
                .borrow()
                .last()
                .expect("live wave reaches paint_path")
        });
        let expected = rgba(0x33669980);
        for (actual, expected) in [color.r, color.g, color.b]
            .into_iter()
            .zip([expected.r, expected.g, expected.b])
        {
            assert!((actual - expected).abs() < 0.000001);
        }
        assert!((color.a - expected.a * 0.5 * 0.1).abs() < 0.000001);
        assert_eq!(
            (
                corners.top_left,
                corners.top_right,
                corners.bottom_right,
                corners.bottom_left
            ),
            (px(3.), px(4.), px(5.), px(6.))
        );
        assert_eq!(
            (border.top, border.right, border.bottom, border.left),
            (px(1.), px(2.), px(3.), px(4.))
        );
        for (opacity, hidden) in [(0., false), (1., true)] {
            view.update(cx, |view, cx| {
                view.opacity = opacity;
                view.hidden = hidden;
                cx.notify();
            });
            PAINT_SAMPLES.with(|samples| samples.borrow_mut().clear());
            draw(cx);
            PAINT_SAMPLES.with(|samples| {
                let samples = samples.borrow();
                assert!(
                    samples.iter().all(|(color, _, _)| color.a == 0.),
                    "hidden or transparent button must not paint a visible ripple"
                );
            });
        }
    }
    #[derive(Clone, Debug)]
    struct Sample {
        center: Point<Pixels>,
        radius: Pixels,
        opacity: f32,
    }
    struct Harness {
        samples: Rc<RefCell<Vec<Sample>>>,
        clicks: usize,
        disabled: bool,
        mounted: bool,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            // Medium has no target beyond what it paints, so this ripple around the
            // whole button sees the bounds a button's own ripple does.
            let button = MaterialButton::new("ripple-button")
                .button_size(crate::components::button::ButtonSize::Medium)
                .ripple(false)
                .disabled(self.disabled)
                .child("Wave")
                .debug_selector(|| "ripple-button".into())
                .on_click(cx.listener(|this, _, _, _| this.clicks += 1));
            let ripple = ButtonRipple::new(
                button,
                "observed-ripple".into(),
                self.mounted,
                !self.disabled,
                gpui::Corners {
                    top_left: rem(20.).into(),
                    top_right: rem(20.).into(),
                    bottom_right: rem(20.).into(),
                    bottom_left: rem(20.).into(),
                },
                gpui::Edges::all(rem(0.).into()),
                rgba(0xffffffff),
                window,
                cx,
            );
            *self.samples.borrow_mut() = ripple
                .waves
                .iter()
                .map(|wave| Sample {
                    center: wave.center,
                    radius: wave.radius,
                    opacity: wave.opacity,
                })
                .collect();
            div().p(px(20.)).child(ripple)
        }
    }
    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }
    #[gpui::test]
    fn pointer_waves_expand_fade_overlap_and_leave_activation_to_base(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let samples = Rc::new(RefCell::new(vec![]));
        let (view, cx) = cx.add_window_view({
            let samples = samples.clone();
            move |_, _| Harness {
                samples,
                clicks: 0,
                disabled: false,
                mounted: true,
            }
        });
        draw(cx);
        let bounds = cx.debug_bounds("ripple-button").unwrap();
        let position = bounds.origin + point(px(9.), px(12.));
        cx.simulate_mouse_down(position, MouseButton::Left, Modifiers::default());
        draw(cx);
        assert_eq!(samples.borrow().len(), 1);
        assert_eq!(samples.borrow()[0].center, point(px(9.), px(12.)));
        assert_eq!(samples.borrow()[0].radius, px(0.));
        assert_eq!(samples.borrow()[0].opacity, 0.1);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        let eased = Easing::cubic_bezier(0.4, 0., 0.2, 1.).unwrap().sample(0.25);
        assert!(
            (f32::from(samples.borrow()[0].radius)
                - f32::from(bounds.size.width.max(bounds.size.height)) * eased)
                .abs()
                < 0.001
        );
        cx.simulate_mouse_up(position, MouseButton::Left, Modifiers::default());
        draw(cx);
        cx.update(|_, cx| assert_eq!(view.read(cx).clicks, 1));
        cx.simulate_mouse_down(position, MouseButton::Right, Modifiers::default());
        draw(cx);
        assert_eq!(samples.borrow().len(), 2);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        assert!((samples.borrow()[0].opacity - 0.1 * (1. - eased)).abs() < 0.0001);
        assert_eq!(samples.borrow()[1].opacity, 0.1);
        cx.simulate_event(MouseExitEvent {
            position: point(px(900.), px(900.)),
            pressed_button: Some(MouseButton::Right),
            modifiers: Modifiers::default(),
        });
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(450));
        cx.run_until_parked();
        draw(cx);
        assert!(samples.borrow().is_empty());
        cx.update(|_, cx| assert_eq!(view.read(cx).clicks, 1));
        for name in ["space", "enter"] {
            let keystroke = cx.update(|_, cx| {
                gpui::Keystroke::parse_with_os(name, cx.platform_info().os).unwrap()
            });
            cx.simulate_event(gpui::KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            draw(cx);
            assert!(samples.borrow().is_empty(), "keyboard never creates a wave");
            cx.simulate_event(gpui::KeyUpEvent { keystroke });
            draw(cx);
        }
        cx.update(|_, cx| assert_eq!(view.read(cx).clicks, 3));
    }
    struct PreventingHarness {
        inner: gpui::Entity<Harness>,
    }
    impl Render for PreventingHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            // An ancestor that handles the press during capture prevents the
            // default, as GPUI focus transfer and nested controls do.
            div().child(self.inner.clone()).child(
                gpui::canvas(
                    |_, _, _| {},
                    |_, _, window, _| {
                        window.on_mouse_event(|_: &gpui::MouseDownEvent, phase, window, _| {
                            if phase == gpui::DispatchPhase::Capture {
                                window.prevent_default();
                            }
                        });
                    },
                )
                .size_0(),
            )
        }
    }
    struct IdleHarness {
        capacity: Rc<RefCell<Option<usize>>>,
    }
    impl Render for IdleHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let ripple = ButtonRipple::new(
                div().size(px(40.)),
                "idle-ripple".into(),
                true,
                true,
                gpui::Corners::default(),
                gpui::Edges::default(),
                rgba(0xffffffff),
                window,
                cx,
            );
            *self.capacity.borrow_mut() = Some(ripple.waves.capacity());
            div().child(ripple)
        }
    }
    #[gpui::test]
    fn idle_ripple_allocates_no_painted_waves(cx: &mut TestAppContext) {
        let capacity = Rc::new(RefCell::new(None));
        let (_, cx) = cx.add_window_view({
            let capacity = capacity.clone();
            move |_, _| IdleHarness { capacity }
        });
        draw(cx);
        draw(cx);
        assert_eq!(*capacity.borrow(), Some(0), "idle render must not allocate");
    }
    #[gpui::test]
    fn default_prevented_press_starts_no_wave(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let samples = Rc::new(RefCell::new(vec![]));
        let (_, cx) = cx.add_window_view({
            let samples = samples.clone();
            move |_, cx| PreventingHarness {
                inner: cx.new(|_| Harness {
                    samples,
                    clicks: 0,
                    disabled: false,
                    mounted: true,
                }),
            }
        });
        draw(cx);
        let center = cx.debug_bounds("ripple-button").unwrap().center();
        for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
            cx.simulate_mouse_down(center, button, Modifiers::default());
            draw(cx);
            assert!(
                samples.borrow().is_empty(),
                "{button:?}: a prevented press must not start a wave"
            );
            cx.simulate_mouse_up(center, button, Modifiers::default());
            draw(cx);
        }
    }
    #[gpui::test]
    fn reduced_motion_disable_and_unpainted_release_preserve_wave_lifecycle(
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let samples = Rc::new(RefCell::new(vec![]));
        let (view, cx) = cx.add_window_view({
            let samples = samples.clone();
            move |_, _| Harness {
                samples,
                clicks: 0,
                disabled: false,
                mounted: true,
            }
        });
        draw(cx);
        let bounds = cx.debug_bounds("ripple-button").unwrap();
        cx.simulate_mouse_down(bounds.center(), MouseButton::Left, Modifiers::default());
        draw(cx);
        assert_eq!(
            samples.borrow()[0].radius,
            bounds.size.width.max(bounds.size.height)
        );
        view.update(cx, |this, cx| {
            this.disabled = true;
            cx.notify();
        });
        draw(cx);
        assert_eq!(
            samples.borrow()[0].opacity,
            0.1,
            "disabling does not discard an existing wave"
        );
        cx.simulate_mouse_up(bounds.center(), MouseButton::Left, Modifiers::default());
        draw(cx);
        assert_eq!(samples.borrow()[0].opacity, 0.);
        cx.simulate_mouse_down(bounds.center(), MouseButton::Right, Modifiers::default());
        draw(cx);
        assert_eq!(
            samples.borrow().len(),
            1,
            "disabled cannot create another wave"
        );
        view.update(cx, |this, cx| {
            this.mounted = false;
            this.disabled = false;
            cx.notify();
        });
        draw(cx);
        assert!(samples.borrow().is_empty());
        view.update(cx, |this, cx| {
            this.mounted = true;
            cx.notify();
        });
        draw(cx);
        cx.simulate_mouse_down(bounds.center(), MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_up(bounds.center(), MouseButton::Left, Modifiers::default());
        draw(cx);
        assert_eq!(samples.borrow()[0].opacity, 0.);
    }
    #[gpui::test]
    fn raw_touch_starts_immediately_and_never_duplicates_base_tap_or_scroll(
        cx: &mut TestAppContext,
    ) {
        use gpui::{TouchEvent, TouchId, TouchPhase};
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let samples = Rc::new(RefCell::new(vec![]));
        let (view, cx) = cx.add_window_view({
            let samples = samples.clone();
            move |_, _| Harness {
                samples,
                clicks: 0,
                disabled: false,
                mounted: true,
            }
        });
        draw(cx);
        let center = cx.debug_bounds("ripple-button").unwrap().center();
        let touch = |cx: &mut VisualTestContext, id, phase, position| {
            cx.simulate_event(TouchEvent {
                id: TouchId(id),
                phase,
                position,
                ..Default::default()
            });
            draw(cx);
        };
        touch(cx, 1, TouchPhase::Started, center);
        assert_eq!(samples.borrow().len(), 1);
        assert_eq!(samples.borrow()[0].opacity, 0.1);
        cx.update(|_, cx| assert_eq!(view.read(cx).clicks, 0));
        touch(cx, 1, TouchPhase::Ended, center);
        assert_eq!(
            samples.borrow().len(),
            1,
            "synthetic mouse down must not create a second wave"
        );
        assert_eq!(samples.borrow()[0].opacity, 0.);
        cx.update(|_, cx| assert_eq!(view.read(cx).clicks, 1));
        touch(cx, 2, TouchPhase::Started, center);
        // A system cancel of a still finger has no public signal: the hold limit ends it.
        touch(cx, 2, TouchPhase::Cancelled, center);
        cx.executor().advance_clock(super::TOUCH_HOLD_LIMIT);
        cx.run_until_parked();
        draw(cx);
        assert_eq!(samples.borrow().last().unwrap().opacity, 0.);
        cx.update(|_, cx| assert_eq!(view.read(cx).clicks, 1));
        touch(cx, 3, TouchPhase::Started, center);
        touch(cx, 3, TouchPhase::Moved, center + point(px(0.), px(80.)));
        assert_eq!(
            samples.borrow().last().unwrap().opacity,
            0.,
            "a touch that moves becomes a scroll, which releases the wave"
        );
        touch(cx, 3, TouchPhase::Ended, center + point(px(0.), px(80.)));
        assert_eq!(samples.borrow().last().unwrap().opacity, 0.);
        cx.update(|_, cx| assert_eq!(view.read(cx).clicks, 1));
    }
    #[gpui::test]
    fn changing_reduced_motion_preserves_started_fade_and_restarts_expansion(
        cx: &mut TestAppContext,
    ) {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../conformance/reference/button-ripple.json"
        )))
        .unwrap();
        let dynamic = &fixture["dynamic_media"];
        let opacity = |name: &str| dynamic[name]["opacity"].as_f64().unwrap() as f32;
        cx.update(gpui_base::init);
        let samples = Rc::new(RefCell::new(vec![]));
        let (_, cx) = cx.add_window_view({
            let samples = samples.clone();
            move |_, _| Harness {
                samples,
                clicks: 0,
                disabled: false,
                mounted: true,
            }
        });
        draw(cx);
        let bounds = cx.debug_bounds("ripple-button").unwrap();
        cx.simulate_mouse_down(bounds.center(), MouseButton::Left, Modifiers::default());
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        assert_eq!(samples.borrow()[0].opacity, opacity("held_before"));
        cx.update(|_, cx| cx.set_reduce_motion(true));
        draw(cx);
        assert_eq!(
            samples.borrow()[0].radius,
            bounds.size.width.max(bounds.size.height)
        );
        assert_eq!(samples.borrow()[0].opacity, opacity("held_reduced"));
        cx.update(|_, cx| cx.set_reduce_motion(false));
        draw(cx);
        assert_eq!(
            samples.borrow()[0].radius,
            px(0.),
            "restoring CSS animation restarts expansion"
        );
        assert_eq!(samples.borrow()[0].opacity, opacity("held_restored"));
        cx.simulate_mouse_up(bounds.center(), MouseButton::Left, Modifiers::default());
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        assert!((samples.borrow()[0].opacity - opacity("fade_before")).abs() < 0.00001);
        cx.update(|_, cx| cx.set_reduce_motion(true));
        draw(cx);
        assert!((samples.borrow()[0].opacity - opacity("fade_reduced")).abs() < 0.00001);
        cx.update(|_, cx| cx.set_reduce_motion(false));
        draw(cx);
        assert!((samples.borrow()[0].opacity - opacity("fade_restored")).abs() < 0.00001);
        // The existing transition also advances while the media query is reduced.
        cx.update(|_, cx| cx.set_reduce_motion(true));
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        assert!((samples.borrow()[0].opacity - 0.0224439).abs() < 0.00001);
        cx.update(|_, cx| cx.set_reduce_motion(false));
        draw(cx);
        assert!((samples.borrow()[0].opacity - 0.0224439).abs() < 0.00001);
        cx.executor().advance_clock(Duration::from_millis(200));
        draw(cx);
        assert_eq!(
            samples.borrow()[0].opacity,
            0.,
            "media changes never restart the fade clock"
        );
        cx.executor().advance_clock(Duration::from_millis(50));
        cx.run_until_parked();
        draw(cx);
        assert!(samples.borrow().is_empty());
        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.simulate_mouse_down(bounds.center(), MouseButton::Left, Modifiers::default());
        draw(cx);
        cx.simulate_mouse_up(bounds.center(), MouseButton::Left, Modifiers::default());
        draw(cx);
        assert_eq!(
            samples.borrow()[0].opacity,
            opacity("release_while_reduced")
        );
        cx.update(|_, cx| cx.set_reduce_motion(false));
        draw(cx);
        assert_eq!(
            samples.borrow()[0].opacity,
            opacity("reduced_release_restored")
        );
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        assert_eq!(
            samples.borrow()[0].opacity,
            opacity("reduced_release_restored_100ms")
        );
        // The CSS duration is selected when the release is first styled, even
        // if the preference changes after the input event in the same task.
        cx.simulate_mouse_down(bounds.center(), MouseButton::Left, Modifiers::default());
        draw(cx);
        assert_eq!(samples.borrow().last().unwrap().opacity, 0.1);
        cx.update(|window, cx| {
            window.dispatch_event(
                gpui::PlatformInput::MouseUp(gpui::MouseUpEvent {
                    position: bounds.center(),
                    button: MouseButton::Left,
                    modifiers: Modifiers::default(),
                    click_count: 1,
                }),
                cx,
            );
            cx.set_reduce_motion(true);
        });
        draw(cx);
        assert_eq!(samples.borrow().last().unwrap().opacity, 0.);
        cx.update(|_, cx| cx.set_reduce_motion(false));
        draw(cx);
        assert_eq!(samples.borrow().last().unwrap().opacity, 0.);
    }

    #[gpui::test]
    fn ordinary_short_press_matches_browser_and_cleanup_uses_release_clock(
        cx: &mut TestAppContext,
    ) {
        cx.update(gpui_base::init);
        let samples = Rc::new(RefCell::new(vec![]));
        let (_, cx) = cx.add_window_view({
            let samples = samples.clone();
            move |_, _| Harness {
                samples,
                clicks: 0,
                disabled: false,
                mounted: true,
            }
        });
        draw(cx);
        let bounds = cx.debug_bounds("ripple-button").unwrap();
        let radius = f32::from(bounds.size.width.max(bounds.size.height));
        cx.simulate_mouse_down(bounds.center(), MouseButton::Left, Modifiers::default());
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(40));
        draw(cx);
        // Pinned Chromium fixture: ordinary.short_press, held for 40ms.
        assert!((f32::from(samples.borrow()[0].radius) / radius - 0.025863).abs() < 0.00001);
        cx.simulate_mouse_up(bounds.center(), MouseButton::Left, Modifiers::default());
        draw(cx);
        assert_eq!(samples.borrow()[0].opacity, 0.1);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        assert!((f32::from(samples.borrow()[0].radius) / radius - 0.5).abs() < 0.00001);
        assert!((samples.borrow()[0].opacity - 0.0763413).abs() < 0.00001);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        assert!((f32::from(samples.borrow()[0].radius) / radius - 0.875968).abs() < 0.00001);
        assert!((samples.borrow()[0].opacity - 0.0224439).abs() < 0.00001);
        cx.executor().advance_clock(Duration::from_millis(200));
        draw(cx);
        assert_eq!(samples.borrow()[0].radius, px(radius));
        assert_eq!(samples.borrow()[0].opacity, 0.);
        cx.executor().advance_clock(Duration::from_millis(49));
        cx.run_until_parked();
        draw(cx);
        assert_eq!(
            samples.borrow().len(),
            1,
            "cleanup waits 450ms after release"
        );
        cx.executor().advance_clock(Duration::from_millis(1));
        cx.run_until_parked();
        draw(cx);
        assert!(samples.borrow().is_empty());

        // With no intervening paint, CSS never has the active opacity as its
        // before-change style. Ordinary motion must also start fully faded.
        // simulate_mouse_* runs the executor between events, so dispatch both
        // within one App update to exercise an actually unpainted press.
        cx.update(|window, cx| {
            window.dispatch_event(
                gpui::PlatformInput::MouseDown(gpui::MouseDownEvent {
                    position: bounds.center(),
                    button: MouseButton::Left,
                    modifiers: Modifiers::default(),
                    click_count: 1,
                    first_mouse: false,
                }),
                cx,
            );
            window.dispatch_event(
                gpui::PlatformInput::MouseUp(gpui::MouseUpEvent {
                    position: bounds.center(),
                    button: MouseButton::Left,
                    modifiers: Modifiers::default(),
                    click_count: 1,
                }),
                cx,
            );
        });
        draw(cx);
        assert_eq!(samples.borrow().len(), 1);
        assert_eq!(samples.borrow()[0].opacity, 0.);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(cx);
        assert_eq!(
            samples.borrow()[0].opacity,
            0.,
            "an unpainted press must not flash later"
        );
    }
}

#[cfg(test)]
mod target_tests {
    use super::RippleState;
    use crate::components::button::{ButtonSize, ButtonVariant, MaterialButton, PressState};
    use gpui::prelude::*;
    use gpui::{
        Context, ElementId, Entity, InputEvent, IntoElement, Modifiers, MouseButton,
        MouseMoveEvent, Render, TestAppContext, TouchEvent, TouchId, TouchPhase, VisualTestContext,
        Window, div, point, px,
    };

    struct Harness {
        size: ButtonSize,
        outlined: bool,
        own_clip: bool,
        ancestor_clip: bool,
        overlap: bool,
        focus: gpui::FocusHandle,
        occlude: bool,
        scrolls: usize,
        clicks: [usize; 2],
        ripple_states: [Option<Entity<RippleState>>; 2],
        press_state: Option<Entity<PressState>>,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            let button = MaterialButton::new("target")
                .button_size(self.size)
                .track_focus(&self.focus)
                .when(self.occlude, |button| button.occlude())
                .variant(if self.outlined {
                    ButtonVariant::Outlined
                } else {
                    ButtonVariant::Filled
                })
                .w(px(80.))
                .child("Target")
                .debug_selector(|| "target".into())
                .when(self.own_clip, |button| button.overflow_hidden())
                .on_click(cx.listener(|this, _, _, _| this.clicks[0] += 1));
            let button = gpui::RenderOnce::render(button, window, cx).into_any_element();
            self.ripple_states[0] = Some(window.use_keyed_state(
                (ElementId::from("target"), "ripple"),
                cx,
                |_, _| panic!("existing ripple"),
            ));
            self.press_state = Some(window.use_keyed_state(
                (ElementId::from("target"), "material-press"),
                cx,
                |_, _| panic!("existing press"),
            ));
            let neighbor = if self.overlap {
                let neighbor = MaterialButton::new("neighbor")
                    .button_size(ButtonSize::ExtraSmall)
                    .absolute()
                    .top(px(40.))
                    .w(px(80.))
                    .child("Next")
                    .on_click(cx.listener(|this, _, _, _| this.clicks[1] += 1));
                let neighbor = gpui::RenderOnce::render(neighbor, window, cx).into_any_element();
                self.ripple_states[1] = Some(window.use_keyed_state(
                    (ElementId::from("neighbor"), "ripple"),
                    cx,
                    |_, _| panic!("existing neighbor ripple"),
                ));
                Some(neighbor)
            } else {
                None
            };
            div()
                .size_full()
                .p(px(20.))
                .on_scroll(cx.listener(|this, _, _, _| this.scrolls += 1))
                .child(
                    div()
                        .relative()
                        .w(px(80.))
                        .h(px(self.size.geometry().height))
                        .when(self.ancestor_clip, |parent| parent.overflow_hidden())
                        .child(button)
                        .children(neighbor),
                )
        }
    }
    fn setup(cx: &mut TestAppContext) -> (Entity<Harness>, &mut VisualTestContext) {
        cx.update(gpui_base::init);
        let (view, cx) = cx.add_window_view(|_, cx| Harness {
            size: ButtonSize::ExtraSmall,
            outlined: false,
            own_clip: false,
            ancestor_clip: false,
            overlap: false,
            focus: cx.focus_handle(),
            occlude: false,
            scrolls: 0,
            clicks: [0; 2],
            ripple_states: [None, None],
            press_state: None,
        });
        draw(cx);
        (view, cx)
    }
    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }
    fn wave_count(view: &Entity<Harness>, cx: &mut VisualTestContext, id: &'static str) -> u64 {
        view.read_with(cx, |view, cx| {
            view.ripple_states[usize::from(id == "neighbor")]
                .as_ref()
                .unwrap()
                .read(cx)
                .next
        })
    }
    fn press(view: &Entity<Harness>, cx: &mut VisualTestContext) -> (bool, bool) {
        view.read_with(cx, |view, cx| {
            let state = view.press_state.as_ref().unwrap().read(cx);
            (state.pointer, state.hovered)
        })
    }

    #[gpui::test]
    fn compact_edges_share_activation_hover_press_and_ripple_without_layout_growth(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        for size in [ButtonSize::ExtraSmall, ButtonSize::Small] {
            view.update(cx, |view, cx| {
                view.size = size;
                cx.notify();
            });
            draw(cx);
            let bounds = cx.debug_bounds("target").unwrap();
            assert_eq!(bounds.size.height, px(size.geometry().height));
            let edge = point(bounds.center().x, bounds.origin.y - px(2.));
            cx.simulate_event(MouseMoveEvent {
                position: edge,
                pressed_button: None,
                modifiers: Modifiers::default(),
            });
            cx.run_until_parked();
            assert!(press(&view, cx).1, "outer target must hover");
            let before = wave_count(&view, cx, "target");
            let clicks = view.read_with(cx, |view, _| view.clicks[0]);
            cx.simulate_mouse_down(edge, MouseButton::Left, Modifiers::default());
            assert!(press(&view, cx).0);
            assert_eq!(wave_count(&view, cx, "target"), before + 1);
            cx.simulate_mouse_move(edge, MouseButton::Left, Modifiers::default());
            assert!(press(&view, cx).1, "held movement uses expanded target");
            cx.simulate_mouse_up(edge, MouseButton::Left, Modifiers::default());
            assert!(!press(&view, cx).0);
            assert_eq!(view.read_with(cx, |view, _| view.clicks[0]), clicks + 1);
            assert_eq!(
                cx.debug_bounds("target").unwrap().size.height,
                bounds.size.height
            );
        }
        view.update(cx, |view, cx| {
            view.size = ButtonSize::Medium;
            cx.notify();
        });
        draw(cx);
        let bounds = cx.debug_bounds("target").unwrap();
        let before = wave_count(&view, cx, "target");
        cx.simulate_click(
            point(bounds.center().x, bounds.origin.y - px(2.)),
            Modifiers::default(),
        );
        assert_eq!(
            wave_count(&view, cx, "target"),
            before,
            "medium has no expanded target"
        );
    }

    #[gpui::test]
    fn target_and_ripple_obey_border_width_clipping_and_neighbor_occlusion(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        view.update(cx, |view, cx| {
            view.outlined = true;
            cx.notify();
        });
        draw(cx);
        let bounds = cx.debug_bounds("target").unwrap();
        for own in [true, false] {
            view.update(cx, |view, cx| {
                view.own_clip = own;
                view.ancestor_clip = !own;
                cx.notify();
            });
            draw(cx);
            cx.simulate_click(
                point(bounds.center().x, bounds.origin.y - px(2.)),
                Modifiers::default(),
            );
            assert_eq!(
                wave_count(&view, cx, "target"),
                0,
                "passive ripple obeys the same clip as activation"
            );
            assert_eq!(view.read_with(cx, |view, _| view.clicks[0]), 0);
        }
        view.update(cx, |view, cx| {
            view.own_clip = false;
            view.ancestor_clip = false;
            view.overlap = true;
            cx.notify();
        });
        draw(cx);
        cx.simulate_click(
            point(bounds.center().x, bounds.origin.y + px(35.)),
            Modifiers::default(),
        );
        assert_eq!(
            wave_count(&view, cx, "target"),
            0,
            "later neighbor blocks earlier passive ripple"
        );
        assert_eq!(wave_count(&view, cx, "neighbor"), 1);
        assert_eq!(view.read_with(cx, |view, _| view.clicks), [0, 1]);
    }

    #[gpui::test]
    fn raw_touch_at_extended_edge_starts_one_wave_and_activates_once(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        let bounds = cx.debug_bounds("target").unwrap();
        let edge = point(bounds.center().x, bounds.origin.y - px(2.));
        for phase in [TouchPhase::Started, TouchPhase::Ended] {
            cx.update(|window, cx| {
                window.dispatch_event(
                    TouchEvent {
                        id: TouchId(1),
                        phase,
                        position: edge,
                        ..Default::default()
                    }
                    .to_platform_input(),
                    cx,
                );
            });
            assert_eq!(press(&view, cx).0, phase == TouchPhase::Started);
            assert_eq!(
                wave_count(&view, cx, "target"),
                1,
                "synthesized mouse must not duplicate raw wave"
            );
        }
        assert_eq!(view.read_with(cx, |view, _| view.clicks[0]), 1);
    }
    #[gpui::test]
    fn caller_occlusion_can_block_scroll_beyond_material_default(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        let position = cx.debug_bounds("target").unwrap().center();
        let scroll = |cx: &mut VisualTestContext| {
            cx.simulate_event(gpui::ScrollEvent {
                position,
                delta: gpui::ScrollDelta::Pixels(point(px(0.), px(10.))),
                touch_phase: TouchPhase::Moved,
                ..Default::default()
            });
        };
        scroll(cx);
        assert_eq!(
            view.read_with(cx, |view, _| view.scrolls),
            1,
            "default allows ancestor scrolling"
        );
        view.update(cx, |view, cx| {
            view.occlude = true;
            cx.notify();
        });
        draw(cx);
        scroll(cx);
        assert_eq!(
            view.read_with(cx, |view, _| view.scrolls),
            1,
            "caller occlude must not be overwritten during render"
        );
    }
}
