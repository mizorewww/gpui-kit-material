use super::{
    ButtonShape, ButtonSize, ButtonVariant, MaterialButton, MotionToken, alpha, colors,
    resting_radius, toggle_color_targets,
};
use crate::color::ColorRole;
use gpui::{
    Context, IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, MouseButton, Render,
    TestAppContext, VisualTestContext, Window, div, prelude::*, px,
};
use std::{cell::Cell, rc::Rc, time::Duration};

#[test]
fn toggle_roles_and_shape_swap_match_reference_for_every_variant_and_size() {
    use ColorRole as C;
    for variant in ButtonVariant::ALL {
        for selected in [false, true] {
            let resolved = |disabled| {
                let (bg, fg, border) = toggle_color_targets(variant, disabled, true, selected);
                (bg.resolve(), fg.resolve(), border.resolve())
            };
            let (bg, fg, border) = resolved(false);
            let expected = match (variant, selected) {
                (ButtonVariant::Filled, false) => {
                    Some((C::SURFACE_CONTAINER, C::ON_SURFACE_VARIANT))
                }
                (ButtonVariant::Filled | ButtonVariant::Elevated, true) => {
                    Some((C::PRIMARY, C::ON_PRIMARY))
                }
                (ButtonVariant::Tonal, true) => Some((C::SECONDARY, C::ON_SECONDARY)),
                (ButtonVariant::Outlined, true) => {
                    Some((C::INVERSE_SURFACE, C::INVERSE_ON_SURFACE))
                }
                _ => None,
            };
            if let Some((expected_bg, expected_fg)) = expected {
                assert_eq!(bg, crate::theme::color(expected_bg));
                assert_eq!(fg, crate::theme::color(expected_fg));
                assert_eq!(border.a, 0.);
            } else {
                assert_eq!((bg, fg, border), colors(variant, false));
            }
            let (bg, fg, border) = resolved(true);
            assert_eq!(
                bg.a,
                if variant == ButtonVariant::Outlined {
                    0.
                } else {
                    0.1
                }
            );
            assert_eq!(fg, alpha(crate::theme::color(C::ON_SURFACE_VARIANT), 0.38));
            assert_eq!(
                border.a,
                if variant == ButtonVariant::Outlined {
                    0.1
                } else {
                    0.
                }
            );
        }
    }
    for size in ButtonSize::ALL {
        let g = size.geometry();
        for shape in ButtonShape::ALL {
            assert_eq!(
                resting_radius(size, shape, true),
                if shape == ButtonShape::Round {
                    g.square
                } else {
                    g.height / 2.
                }
            );
            assert_eq!(
                resting_radius(size, shape, false),
                if shape == ButtonShape::Round {
                    g.height / 2.
                } else {
                    g.square
                }
            );
        }
    }
}

struct ToggleHarness {
    selected: bool,
    disabled: bool,
    toggle_on_click: bool,
    apply_change: bool,
    focusable: bool,
    requests: Vec<bool>,
    clicks: usize,
    radius: Rc<Cell<f32>>,
}
impl Render for ToggleHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        crate::motion::apply_scheme(crate::motion::MotionScheme::Expressive);
        let entity = cx.entity().downgrade();
        let mut button = MaterialButton::new("toggle-test")
            .toggle(true)
            .selected(self.selected)
            .disabled(self.disabled)
            .toggle_on_click(self.toggle_on_click)
            .focusable(self.focusable)
            .button_size(ButtonSize::Large)
            .child("Toggle")
            .debug_selector(|| "toggle-test".into())
            .on_change(move |next, _, _, cx| {
                entity
                    .update(cx, |this, cx| {
                        this.requests.push(next);
                        if this.apply_change {
                            this.selected = next;
                        }
                        cx.notify();
                    })
                    .unwrap();
            })
            .on_click(cx.listener(|this, _, _, _| this.clicks += 1));
        button.radius_observer = Some(self.radius.clone());
        div().flex().items_start().child(button)
    }
}
fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}
fn key(cx: &mut VisualTestContext, name: &str) {
    let keystroke =
        cx.update(|_, cx| Keystroke::parse_with_os(name, cx.platform_info().os).unwrap());
    cx.simulate_event(KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent { keystroke });
}
#[gpui::test]
fn controlled_toggle_uses_one_base_activation_for_pointer_keyboard_and_disabled(
    cx: &mut TestAppContext,
) {
    cx.update(|cx| {
        gpui_base::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|_, _| ToggleHarness {
        selected: false,
        disabled: false,
        toggle_on_click: true,
        apply_change: false,
        focusable: true,
        requests: vec![],
        clicks: 0,
        radius: Rc::new(Cell::new(0.)),
    });
    draw(cx);
    let point = cx.debug_bounds("toggle-test").unwrap().center();
    cx.simulate_click(point, Modifiers::default());
    draw(cx);
    key(cx, "enter");
    draw(cx);
    key(cx, "space");
    draw(cx);
    cx.update(|_, cx| {
        let state = view.read(cx);
        assert_eq!(state.requests, [true, true, true]);
        assert_eq!(state.clicks, 3);
        assert!(
            !state.selected,
            "requests must not mutate controlled selection"
        );
    });
    view.update(cx, |this, cx| {
        this.selected = true;
        this.apply_change = true;
        cx.notify();
    });
    draw(cx);
    cx.simulate_click(point, Modifiers::default());
    draw(cx);
    cx.update(|_, cx| {
        assert!(!view.read(cx).selected);
        assert_eq!(view.read(cx).requests, [true, true, true, false]);
    });
    view.update(cx, |this, cx| {
        this.toggle_on_click = false;
        cx.notify();
    });
    draw(cx);
    cx.simulate_click(point, Modifiers::default());
    draw(cx);
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).requests.len(), 4);
        assert_eq!(view.read(cx).clicks, 5);
    });
    view.update(cx, |this, cx| {
        this.disabled = true;
        cx.notify();
    });
    draw(cx);
    cx.simulate_click(point, Modifiers::default());
    key(cx, "enter");
    key(cx, "space");
    draw(cx);
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).requests.len(), 4);
        assert_eq!(view.read(cx).clicks, 5);
    });
}
#[gpui::test]
fn external_selection_retargets_fast_spatial_without_snapping(cx: &mut TestAppContext) {
    cx.update(gpui_base::init);
    let radius = Rc::new(Cell::new(0.));
    let (view, cx) = cx.add_window_view({
        let radius = radius.clone();
        move |_, _| ToggleHarness {
            selected: false,
            disabled: false,
            toggle_on_click: true,
            apply_change: true,
            focusable: true,
            requests: vec![],
            clicks: 0,
            radius,
        }
    });
    draw(cx);
    assert_eq!(radius.get(), 48.);
    view.update(cx, |this, cx| {
        this.selected = true;
        cx.notify();
    });
    draw(cx);
    assert_eq!(radius.get(), 48.);
    cx.executor().advance_clock(Duration::from_millis(100));
    draw(cx);
    let expected = 48.
        + (28. - 48.)
            * crate::motion::MotionScheme::Expressive
                .resolve(MotionToken::FastSpatial)
                .ease()(0.1 / 0.425);
    assert!((radius.get() - expected).abs() < 0.001);
    let before = radius.get();
    view.update(cx, |this, cx| {
        this.selected = false;
        cx.notify();
    });
    draw(cx);
    assert!((radius.get() - before).abs() < 0.001);
    cx.executor().advance_clock(Duration::from_secs(1));
    draw(cx);
    assert_eq!(radius.get(), 48.);
    cx.update(|_, cx| cx.set_reduce_motion(true));
    view.update(cx, |this, cx| {
        this.selected = true;
        cx.notify();
    });
    draw(cx);
    assert_eq!(radius.get(), 28.);
    let point = cx.debug_bounds("toggle-test").unwrap().center();
    cx.simulate_mouse_down(point, MouseButton::Left, Modifiers::default());
    draw(cx);
    assert_eq!(radius.get(), 16.);
    cx.simulate_mouse_up(point, MouseButton::Left, Modifiers::default());
    draw(cx);
    assert_eq!(radius.get(), 48.);
}

#[gpui::test]
fn adapter_preserves_base_toggle_accessibility_and_plain_button_semantics(cx: &mut TestAppContext) {
    use gpui::{ElementId, Role, Toggled, accesskit};
    use std::cell::RefCell;
    struct Probe(Rc<RefCell<Vec<accesskit::Node>>>);
    impl Render for Probe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().children(
                [
                    (true, true, false),
                    (true, false, true),
                    (false, true, false),
                ]
                .into_iter()
                .enumerate()
                .map(|(index, (toggle, selected, disabled))| {
                    let mut base = super::BaseControl::new(ElementId::Integer(index as u64))
                        .disabled(disabled)
                        .accessibility_label("Selection");
                    base.toggle = toggle;
                    base.selected = selected;
                    base.on_change = Some(Rc::new(|_, _, _, _| {}));
                    base.a11y_observer = Some(self.0.clone());
                    base
                }),
            )
        }
    }
    let captured = Rc::new(RefCell::new(vec![]));
    let (_, cx) = cx.add_window_view({
        let captured = captured.clone();
        move |_, _| Probe(captured)
    });
    draw(cx);
    let nodes = captured.borrow();
    assert_eq!(nodes[0].role(), Role::Button);
    assert_eq!(nodes[0].label(), Some("Selection"));
    assert_eq!(nodes[0].toggled(), Some(Toggled::True));
    assert!(nodes[0].supports_action(accesskit::Action::Click));
    assert_eq!(nodes[1].toggled(), Some(Toggled::False));
    assert!(!nodes[1].supports_action(accesskit::Action::Click));
    assert_eq!(nodes[2].toggled(), None);
}

#[gpui::test]
fn nonfocusable_toggle_remains_pointer_only(cx: &mut TestAppContext) {
    cx.update(gpui_base::init);
    let (view, cx) = cx.add_window_view(|_, _| ToggleHarness {
        selected: false,
        disabled: false,
        toggle_on_click: true,
        apply_change: true,
        focusable: false,
        requests: vec![],
        clicks: 0,
        radius: Rc::new(Cell::new(0.)),
    });
    draw(cx);
    let point = cx.debug_bounds("toggle-test").unwrap().center();
    cx.simulate_click(point, Modifiers::default());
    draw(cx);
    key(cx, "enter");
    key(cx, "space");
    draw(cx);
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).requests, [true]);
        assert_eq!(view.read(cx).clicks, 1);
    });
}

#[gpui::test]
fn toggle_hover_layers_keep_filled_and_tonal_flat_and_elevated_raised(cx: &mut TestAppContext) {
    use super::shadow::ShadowPair;
    use std::cell::RefCell;
    struct Probe {
        variant: ButtonVariant,
        selected: bool,
        sample: Rc<RefCell<(f32, ShadowPair)>>,
    }
    impl Render for Probe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let mut button = MaterialButton::new("toggle-effects")
                .toggle(true)
                .selected(self.selected)
                .variant(self.variant)
                .debug_selector(|| "toggle-effects".into())
                .child("Toggle");
            button.effects_observer = Some(self.sample.clone());
            div().flex().items_start().child(button)
        }
    }
    cx.update(gpui_base::init);
    let sample = Rc::new(RefCell::new((0., ShadowPair::level(0))));
    let (view, cx) = cx.add_window_view({
        let sample = sample.clone();
        move |_, _| Probe {
            variant: ButtonVariant::Filled,
            selected: false,
            sample,
        }
    });
    for variant in ButtonVariant::ALL {
        for selected in [false, true] {
            view.update(cx, |this, cx| {
                this.variant = variant;
                this.selected = selected;
                cx.notify();
            });
            draw(cx);
            let center = cx.debug_bounds("toggle-effects").unwrap().center();
            cx.simulate_event(gpui::MouseMoveEvent {
                position: center,
                pressed_button: None,
                modifiers: Modifiers::default(),
            });
            cx.run_until_parked();
            draw(cx);
            cx.executor().advance_clock(Duration::from_millis(500));
            draw(cx);
            assert_eq!(sample.borrow().0, 0.08);
            assert_eq!(
                sample.borrow().1,
                ShadowPair::level(if variant == ButtonVariant::Elevated {
                    2
                } else {
                    0
                })
            );
        }
    }
}
