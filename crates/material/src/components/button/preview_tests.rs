//! Forced preview states paint what the live states paint.
use super::{ButtonSize, ButtonVariant, InteractionPreview, MaterialButton, shadow::ShadowPair};
use gpui::{
    Context, FocusHandle, Modifiers, MouseMoveEvent, Render, TestAppContext, VisualTestContext,
    Window, div, prelude::*, px,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

struct Probe {
    variant: ButtonVariant,
    preview: Option<InteractionPreview>,
    active: bool,
    ripple: bool,
    disabled: bool,
    focus: FocusHandle,
    radius: Rc<Cell<f32>>,
    effects: Rc<RefCell<(f32, ShadowPair)>>,
}
impl Render for Probe {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let mut button = MaterialButton::new("preview-probe")
            .variant(self.variant)
            .button_size(ButtonSize::Medium)
            .ripple(self.ripple)
            .active_state(self.active)
            .disabled(self.disabled)
            .track_focus(&self.focus)
            .debug_selector(|| "preview-probe".into())
            .child("Label");
        if let Some(preview) = self.preview {
            button = button.preview_state(preview);
        }
        button.radius_observer = Some(self.radius.clone());
        button.effects_observer = Some(self.effects.clone());
        div().p(px(30.)).child(button)
    }
}
fn settle(cx: &mut VisualTestContext) {
    for _ in 0..2 {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.executor().advance_clock(Duration::from_secs(1));
    }
}
fn sample(view: &gpui::Entity<Probe>, cx: &mut VisualTestContext) -> (f32, ShadowPair, f32) {
    settle(cx);
    view.read_with(cx, |probe, _| {
        let (opacity, shadow) = probe.effects.borrow().clone();
        (opacity, shadow, probe.radius.get())
    })
}

#[gpui::test]
fn forced_states_match_live_hover_focus_press_and_disabled(cx: &mut TestAppContext) {
    cx.update(gpui_base::init);
    let (view, cx) = cx.add_window_view(|_, cx| Probe {
        variant: ButtonVariant::Elevated,
        preview: None,
        active: false,
        ripple: true,
        disabled: false,
        focus: cx.focus_handle(),
        radius: Rc::new(Cell::new(0.)),
        effects: Rc::new(RefCell::new((0., ShadowPair::level(0)))),
    });
    let set = |cx: &mut VisualTestContext, f: &dyn Fn(&mut Probe)| {
        view.update(cx, |probe, cx| {
            f(probe);
            cx.notify();
        })
    };
    let rest = sample(&view, cx);

    // Live hover.
    let center = cx.debug_bounds("preview-probe").unwrap().center();
    cx.simulate_event(MouseMoveEvent {
        position: center,
        pressed_button: None,
        modifiers: Modifiers::default(),
    });
    let live_hover = sample(&view, cx);
    cx.simulate_event(MouseMoveEvent {
        position: gpui::point(px(1.), px(1.)),
        pressed_button: None,
        modifiers: Modifiers::default(),
    });
    assert_eq!(sample(&view, cx), rest, "hover released");
    set(cx, &|p| p.preview = Some(InteractionPreview::Hovered));
    assert_eq!(sample(&view, cx), live_hover);
    assert_eq!(live_hover.0, crate::tokens::state::HOVER);

    // Live keyboard focus.
    set(cx, &|p| p.preview = None);
    cx.update(|window, cx| {
        let focus = view.read(cx).focus.clone();
        focus.focus(window, cx);
    });
    cx.simulate_keystrokes("shift");
    let live_focus = sample(&view, cx);
    cx.update(|window, cx| window.blur(cx));
    set(cx, &|p| p.preview = Some(InteractionPreview::Focused));
    assert_eq!(sample(&view, cx), live_focus);
    assert_eq!(live_focus.0, crate::tokens::state::FOCUS);

    // A press with the ripple at rest: the controlled pressed shape and the
    // whole-container press layer.
    set(cx, &|p| {
        p.preview = None;
        p.active = true;
        p.ripple = false;
    });
    let live_press = sample(&view, cx);
    set(cx, &|p| {
        p.preview = Some(InteractionPreview::Pressed);
        p.active = false;
        p.ripple = true;
    });
    assert_eq!(sample(&view, cx), live_press);
    assert_eq!(live_press.0, crate::tokens::state::PRESS);
    assert_ne!(live_press.2, rest.2, "pressed shape morphs");

    set(cx, &|p| {
        p.preview = None;
        p.disabled = true;
    });
    let live_disabled = sample(&view, cx);
    set(cx, &|p| {
        p.preview = Some(InteractionPreview::Disabled);
        p.disabled = false;
    });
    assert_eq!(sample(&view, cx), live_disabled);

    set(cx, &|p| p.preview = Some(InteractionPreview::Enabled));
    assert_eq!(sample(&view, cx), rest);
}

#[gpui::test]
fn previews_leave_the_tab_order_and_ignore_the_pointer(cx: &mut TestAppContext) {
    cx.update(gpui_base::init);
    let (view, cx) = cx.add_window_view(|_, cx| Probe {
        variant: ButtonVariant::Filled,
        preview: Some(InteractionPreview::Enabled),
        active: false,
        ripple: true,
        disabled: false,
        focus: cx.focus_handle(),
        radius: Rc::new(Cell::new(0.)),
        effects: Rc::new(RefCell::new((0., ShadowPair::level(0)))),
    });
    let rest = sample(&view, cx);
    let center = cx.debug_bounds("preview-probe").unwrap().center();
    cx.simulate_event(MouseMoveEvent {
        position: center,
        pressed_button: None,
        modifiers: Modifiers::default(),
    });
    assert_eq!(sample(&view, cx), rest, "pointer cannot change a specimen");
    cx.update(|window, cx| window.focus_next(cx));
    let focused = cx.update(|window, cx| view.read(cx).focus.is_focused(window));
    assert!(!focused, "a specimen is not a tab stop");
}
