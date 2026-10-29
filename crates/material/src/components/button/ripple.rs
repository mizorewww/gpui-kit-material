//! Passive pointer waves. Base owns every activation; this element never consumes input.
use gpui::*;
use gpui_base::{Easing, Keyframe, Keyframes, Timing, Transition, animate_keyframes, transition};
use std::time::Duration;

const MOTION: Duration = Duration::from_millis(400);
const LIFETIME: Duration = Duration::from_millis(450);
#[derive(Clone)]
struct Wave {
    id: u64,
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
    waves: Vec<Wave>,
}
impl RippleState {
    fn press(&mut self, bounds: Bounds<Pixels>, position: Point<Pixels>, cx: &mut Context<Self>) {
        self.next += 1;
        self.waves.push(Wave {
            id: self.next,
            center: position - bounds.origin,
            radius: bounds.size.width.max(bounds.size.height),
            released: false,
            reduced_motion: cx.reduce_motion(),
            expansion_generation: 0,
            fade_duration: None,
        });
        cx.notify();
    }
    fn release(&mut self, cx: &mut Context<Self>) {
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
struct PaintedWave {
    center: Point<Pixels>,
    radius: Pixels,
    opacity: f32,
}

pub(super) struct ButtonRipple {
    child: AnyElement,
    state: Entity<RippleState>,
    enabled: bool,
    radius: Corners<AbsoluteLength>,
    border: Edges<AbsoluteLength>,
    color: Rgba,
    waves: Vec<PaintedWave>,
}
impl ButtonRipple {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
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
        if !mounted {
            state.update(cx, |state, _| state.waves.clear());
        }
        let reduced_motion = cx.reduce_motion();
        state.update(cx, |state, _| {
            for wave in &mut state.waves {
                if wave.reduced_motion && !reduced_motion {
                    wave.expansion_generation += 1;
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
        });
        let frames = Keyframes::try_new([Keyframe::new(0., 0.), Keyframe::new(1., 1.)]).unwrap();
        let timing = Timing::new(MOTION).ease(Easing::cubic_bezier(0.4, 0., 0.2, 1.).unwrap());
        let waves = state
            .read(cx)
            .waves
            .clone()
            .iter()
            .map(|wave| {
                let key = ElementId::NamedChild(
                    id.clone().into(),
                    format!("ripple-wave-{}", wave.id).into(),
                );
                let expansion = ElementId::NamedChild(
                    key.clone().into(),
                    format!("expand-{}", wave.expansion_generation).into(),
                );
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
                        .easing(Easing::cubic_bezier(0.4, 0., 0.2, 1.).unwrap()),
                    window,
                    cx,
                );
                PaintedWave {
                    center: wave.center,
                    radius: wave.radius * scale,
                    opacity,
                }
            })
            .collect();
        Self {
            child: child.into_any_element(),
            state,
            enabled: enabled && mounted,
            radius,
            border,
            color,
            waves,
        }
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
        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);
        self.child.prepaint(window, cx);
        hitbox
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
        let border = self.border.to_pixels(window.rem_size());
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
                tests::record_paint(color, radius, border);
                window.paint_path(path, color);
            }
        }
        let state = self.state.downgrade();
        let enabled = self.enabled;
        let touch_hitbox = hitbox.clone();
        let touch_state = self.state.downgrade();
        window.observe_touch_event(move |event, window, cx| match event.phase {
            TouchPhase::Started
                if enabled && touch_hitbox.is_hovered_at(event.position, window) =>
            {
                let _ = touch_state.update(cx, |state, cx| state.press(bounds, event.position, cx));
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                let _ = touch_state.update(cx, |state, cx| state.release(cx));
            }
            _ => {}
        });
        let hitbox = hitbox.clone();
        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
            if enabled
                && phase == DispatchPhase::Bubble
                && hitbox.is_hovered_at(event.position, window)
                && !window.last_input_was_touch()
            {
                let _ = state.update(cx, |state, cx| state.press(bounds, event.position, cx));
            }
        });
        let state = self.state.downgrade();
        window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
            if phase == DispatchPhase::Capture {
                let _ = state.update(cx, |state, cx| state.release(cx));
            }
        });
        let state = self.state.downgrade();
        window.on_mouse_event(move |_: &MouseExitEvent, phase, _, cx| {
            if phase == DispatchPhase::Capture {
                let _ = state.update(cx, |state, cx| state.release(cx));
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
            let button = MaterialButton::new("ripple-button")
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
        touch(cx, 2, TouchPhase::Cancelled, center);
        assert_eq!(samples.borrow()[1].opacity, 0.);
        cx.update(|_, cx| assert_eq!(view.read(cx).clicks, 1));
        touch(cx, 3, TouchPhase::Started, center);
        touch(cx, 3, TouchPhase::Moved, center + point(px(0.), px(80.)));
        assert_eq!(
            samples.borrow()[2].opacity,
            0.1,
            "moving out does not release the wave"
        );
        touch(cx, 3, TouchPhase::Ended, center + point(px(0.), px(80.)));
        assert_eq!(samples.borrow()[2].opacity, 0.);
        cx.update(|_, cx| assert_eq!(view.read(cx).clicks, 1));
        touch(cx, 4, TouchPhase::Started, center);
        touch(cx, 5, TouchPhase::Started, center);
        assert_eq!(
            samples
                .borrow()
                .iter()
                .filter(|wave| wave.opacity > 0.)
                .count(),
            2
        );
        touch(cx, 5, TouchPhase::Cancelled, center);
        assert!(
            samples.borrow().iter().all(|wave| wave.opacity == 0.),
            "any pointer cancellation releases all waves"
        );
        touch(cx, 4, TouchPhase::Cancelled, center);
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
