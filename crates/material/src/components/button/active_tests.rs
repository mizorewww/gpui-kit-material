use super::{ButtonShape, ButtonSize, ButtonVariant, MaterialButton, shadow::ShadowPair};
use gpui::{
    Context, FocusHandle, Modifiers, Render, TestAppContext, VisualTestContext, Window, div,
    prelude::*, px,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

struct Probe {
    variant: ButtonVariant,
    size: ButtonSize,
    shape: ButtonShape,
    ripple: bool,
    selected: bool,
    toggle: bool,
    active: bool,
    disabled: bool,
    clicks: usize,
    changes: usize,
    focus: FocusHandle,
    radius: Rc<Cell<f32>>,
    effects: Rc<RefCell<(f32, ShadowPair)>>,
}
impl Render for Probe {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let changes = cx.entity().downgrade();
        let mut button = MaterialButton::new("active-probe")
            .variant(self.variant)
            .button_size(self.size)
            .shape(self.shape)
            .ripple(self.ripple)
            .toggle(self.toggle)
            .selected(self.selected)
            .active_state(self.active)
            .disabled(self.disabled)
            .track_focus(&self.focus)
            .debug_selector(|| "active-probe".into())
            .child("Active action")
            .on_click(cx.listener(|this, _, _, _| this.clicks += 1))
            .on_change(move |_, _, _, cx| {
                let _ = changes.update(cx, |this, _| this.changes += 1);
            });
        button.radius_observer = Some(self.radius.clone());
        button.effects_observer = Some(self.effects.clone());
        div().p(px(30.)).child(button)
    }
}
fn probe(cx: &mut Context<Probe>) -> Probe {
    Probe {
        variant: ButtonVariant::Filled,
        size: ButtonSize::Small,
        shape: ButtonShape::Round,
        ripple: false,
        selected: false,
        toggle: false,
        active: false,
        disabled: false,
        clicks: 0,
        changes: 0,
        focus: cx.focus_handle(),
        radius: Rc::new(Cell::new(0.)),
        effects: Rc::new(RefCell::new((0., ShadowPair::level(0)))),
    }
}
fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}
fn settle(cx: &mut VisualTestContext) {
    draw(cx);
    cx.executor().advance_clock(Duration::from_secs(1));
    draw(cx);
}
#[gpui::test]
fn controlled_active_matches_reference_even_while_disabled(cx: &mut TestAppContext) {
    cx.update(gpui_base::init);
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../conformance/reference/button-active.json"
    ))
    .unwrap();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 300);
    let (view, cx) = cx.add_window_view(|_, cx| probe(cx));
    for row in fixture["rows"].as_array().unwrap() {
        view.update(cx, |this, cx| {
            this.variant = ButtonVariant::ALL
                .into_iter()
                .find(|v| v.name() == row["variant"].as_str().unwrap())
                .unwrap();
            this.size = ButtonSize::ALL
                .into_iter()
                .find(|v| v.name() == row["size"].as_str().unwrap())
                .unwrap();
            this.shape = ButtonShape::ALL
                .into_iter()
                .find(|v| v.name() == row["shape"].as_str().unwrap())
                .unwrap();
            this.ripple = row["ripple"].as_bool().unwrap();
            this.selected = row["selected"].as_bool().unwrap();
            this.toggle = row["toggle"].as_bool().unwrap();
            cx.notify();
        });
        for (key, active, disabled) in [
            ("before", false, false),
            ("active", true, false),
            ("disabled_active", true, true),
            ("disabled_inactive", false, true),
            ("restored", false, false),
        ] {
            view.update(cx, |this, cx| {
                this.active = active;
                this.disabled = disabled;
                cx.notify();
            });
            settle(cx);
            cx.update(|window, cx| {
                let state = view.read(cx);
                let expected = &row[key];
                let radius: f32 = expected["radius"]
                    .as_str()
                    .unwrap()
                    .trim_end_matches("px")
                    .parse()
                    .unwrap();
                let opacity: f32 = expected["opacity"].as_str().unwrap().parse().unwrap();
                assert!((state.radius.get() - radius).abs() < 0.001, "{row:?} {key}");
                assert!(
                    (state.effects.borrow().0 - opacity).abs() < 0.001,
                    "{row:?} {key}"
                );
                let level = if expected["shadow"] == "none" { 0 } else { 1 };
                assert_eq!(state.effects.borrow().1, ShadowPair::level(level));
                assert_eq!((state.clicks, state.changes), (0, 0));
                assert!(
                    !state.focus.is_focused(window),
                    "presentation must not move focus"
                );
            });
        }
    }
}
#[gpui::test]
fn controlled_active_leaves_base_activation_and_caller_state_ownership_intact(
    cx: &mut TestAppContext,
) {
    cx.update(gpui_base::init);
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut state = probe(cx);
        state.active = true;
        state
    });
    settle(cx);
    let center = cx.debug_bounds("active-probe").unwrap().center();
    cx.simulate_click(center, Modifiers::default());
    settle(cx);
    cx.update(|_, cx| {
        let state = view.read(cx);
        assert_eq!(state.clicks, 1);
        assert!(state.active);
    });
    view.update(cx, |this, cx| {
        this.disabled = true;
        cx.notify();
    });
    settle(cx);
    cx.simulate_click(center, Modifiers::default());
    cx.update(|_, cx| {
        let state = view.read(cx);
        assert_eq!(state.clicks, 1);
        assert!(state.active);
    });
    view.update(cx, |this, cx| {
        this.active = false;
        cx.notify();
    });
    settle(cx);
    cx.update(|_, cx| {
        let state = view.read(cx);
        assert!(state.disabled);
        assert!(!state.active);
    });
}

#[gpui::test]
fn controlled_active_retargets_spatial_motion_without_a_pointer_event(cx: &mut TestAppContext) {
    cx.update(gpui_base::init);
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut state = probe(cx);
        state.size = ButtonSize::Large;
        state
    });
    settle(cx);
    let radius = view.read_with(cx, |this, _| this.radius.clone());
    assert_eq!(radius.get(), 48.);
    view.update(cx, |this, cx| {
        this.active = true;
        cx.notify();
    });
    draw(cx);
    assert_eq!(radius.get(), 48.);
    cx.executor().advance_clock(Duration::from_millis(100));
    draw(cx);
    let expected = 48.
        + (16. - 48.)
            * crate::motion::MotionScheme::Expressive
                .resolve(crate::motion::MotionToken::FastSpatial)
                .ease()(0.1 / 0.425);
    assert!((radius.get() - expected).abs() < 0.001);
    let interrupted = radius.get();
    view.update(cx, |this, cx| {
        this.active = false;
        cx.notify();
    });
    draw(cx);
    assert!((radius.get() - interrupted).abs() < 0.001);
    settle(cx);
    assert_eq!(radius.get(), 48.);
    cx.update(|_, cx| cx.set_reduce_motion(true));
    view.update(cx, |this, cx| {
        this.active = true;
        cx.notify();
    });
    draw(cx);
    assert_eq!(radius.get(), 16.);
    view.update(cx, |this, cx| {
        this.disabled = true;
        cx.notify();
    });
    draw(cx);
    assert_eq!(radius.get(), 16.);
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).clicks, 0);
        assert_eq!(view.read(cx).changes, 0);
    });
}
