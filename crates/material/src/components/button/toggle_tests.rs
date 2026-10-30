use super::{
    ButtonShape, ButtonSize, ButtonTransitionPlayback, ButtonVariant, MaterialButton, MotionToken,
    alpha, colors, resting_radius, toggle_color_targets,
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
            // m3e button.css toggles: on-surface at 10%, an outline-variant
            // wash for outlined, and no container for text.
            let (bg, fg, border) = resolved(true);
            assert_eq!(
                bg,
                match variant {
                    ButtonVariant::Text => colors(variant, true).0,
                    ButtonVariant::Outlined => {
                        alpha(crate::theme::color(C::OUTLINE_VARIANT), 0.1)
                    }
                    _ => alpha(crate::theme::color(C::ON_SURFACE), 0.1),
                },
                "{variant:?}"
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
        crate::motion::set_motion_scheme(crate::motion::MotionScheme::Expressive);
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
                .position(0.1);
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

// Two real buttons share inputs and a fake clock; only the second unmounts.
type UnmountColors = (gpui::Rgba, gpui::Rgba, gpui::Rgba);

struct ToggleUnmountHarness {
    playback: ButtonTransitionPlayback,
    scheme: crate::motion::MotionScheme,
    selected: bool,
    mount_second: bool,
    radius: [Rc<Cell<f32>>; 2],
    colors: [Rc<Cell<UnmountColors>>; 2],
}
impl Render for ToggleUnmountHarness {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        crate::motion::set_motion_scheme(self.scheme);
        let mut root = div().flex().items_start();
        for (index, id) in ["always-mounted-toggle", "remounted-toggle"]
            .into_iter()
            .enumerate()
        {
            if index == 1 && !self.mount_second {
                continue;
            }
            let mut button = MaterialButton::new(id)
                .toggle(true)
                .selected(self.selected)
                .variant(ButtonVariant::Filled)
                .button_size(ButtonSize::Large)
                .shape(ButtonShape::Round)
                .child("Toggle");
            if index == 1 {
                button = button.transition_playback(self.playback.clone());
            }
            button.radius_observer = Some(self.radius[index].clone());
            button.colors_observer = Some(self.colors[index].clone());
            root = root.child(button);
        }
        root
    }
}

#[gpui::test]
fn controlled_toggle_transition_survives_virtual_unmount(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        cx.set_reduce_motion(false);
    });
    let radius = std::array::from_fn(|_| Rc::new(Cell::new(f32::NAN)));
    let colors = std::array::from_fn(|_| {
        Rc::new(Cell::new((
            gpui::Rgba::default(),
            gpui::Rgba::default(),
            gpui::Rgba::default(),
        )))
    });
    let (view, cx) = cx.add_window_view(|_, _| ToggleUnmountHarness {
        playback: ButtonTransitionPlayback::default(),
        scheme: crate::motion::MotionScheme::Expressive,
        selected: false,
        mount_second: true,
        radius: radius.clone(),
        colors: colors.clone(),
    });
    draw(cx);
    let initial_radius = radius[0].get();
    let initial_colors = colors[0].get();
    assert_eq!(radius[1].get(), initial_radius);
    assert_eq!(colors[1].get(), initial_colors);

    // Start both actual rendered transitions at exactly the same instant.
    view.update(cx, |this, cx| {
        this.selected = true;
        cx.notify();
    });
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(10));
    draw(cx);
    assert_eq!(radius[0].get(), radius[1].get());
    assert_eq!(colors[0].get(), colors[1].get());
    assert_ne!(radius[0].get(), initial_radius);
    assert_ne!(colors[0].get(), initial_colors);

    view.update(cx, |this, cx| {
        this.mount_second = false;
        cx.notify();
    });
    let absent_radius = radius[1].get();
    let absent_colors = colors[1].get();
    // Two completed draws discard frame-local keyed state; neither frame
    // constructs the second button. Keep the control running throughout.
    for _ in 0..2 {
        cx.executor().advance_clock(Duration::from_millis(10));
        draw(cx);
        assert_eq!(radius[1].get(), absent_radius);
        assert_eq!(colors[1].get(), absent_colors);
    }
    cx.executor().advance_clock(Duration::from_millis(10));
    view.update(cx, |this, cx| {
        this.mount_second = true;
        // A running transition must retain its original curve across a scheme
        // switch, just like the continuously mounted control.
        this.scheme = crate::motion::MotionScheme::Standard;
        cx.notify();
    });
    draw(cx);

    // At 40ms both expressive transitions are still in flight. Compare the
    // rendered samples to an always-mounted control, not a parameter mirror.
    let expected_radius = radius[0].get();
    let expected_colors = colors[0].get();
    let target_radius = resting_radius(ButtonSize::Large, ButtonShape::Round, true);
    let (bg, fg, border) = toggle_color_targets(ButtonVariant::Filled, false, true, true);
    let target_colors = (bg.resolve(), fg.resolve(), border.resolve());
    assert!((expected_radius - target_radius).abs() > 0.001);
    assert_ne!(expected_colors, target_colors);
    let radius_matches = (radius[1].get() - expected_radius).abs() < 0.001;
    let actual_colors = colors[1].get();
    let colors_match = [
        (actual_colors.0, expected_colors.0),
        (actual_colors.1, expected_colors.1),
        (actual_colors.2, expected_colors.2),
    ]
    .into_iter()
    .all(|(actual, expected)| {
        [
            (actual.r, expected.r),
            (actual.g, expected.g),
            (actual.b, expected.b),
            (actual.a, expected.a),
        ]
        .into_iter()
        .all(|(actual, expected)| (actual - expected).abs() < 0.00001)
    });
    assert!(
        radius_matches && colors_match,
        "remount must preserve both transitions: radius actual={} control={expected_radius}; colors actual={actual_colors:?} control={expected_colors:?}",
        radius[1].get(),
    );
}

#[gpui::test]
fn controlled_toggle_offscreen_reversal_matches_mounted_control(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        cx.set_reduce_motion(false);
    });
    let radius = std::array::from_fn(|_| Rc::new(Cell::new(f32::NAN)));
    let colors = std::array::from_fn(|_| {
        Rc::new(Cell::new((
            gpui::Rgba::default(),
            gpui::Rgba::default(),
            gpui::Rgba::default(),
        )))
    });
    let (view, cx) = cx.add_window_view(|_, _| ToggleUnmountHarness {
        playback: ButtonTransitionPlayback::default(),
        scheme: crate::motion::MotionScheme::Expressive,
        selected: false,
        mount_second: true,
        radius: radius.clone(),
        colors: colors.clone(),
    });
    draw(cx);
    let initial = radius[0].get();
    view.update(cx, |this, cx| {
        this.selected = true;
        this.mount_second = false;
        this.playback.sync(
            super::ButtonTransitionInputs {
                variant: ButtonVariant::Filled,
                size: ButtonSize::Large,
                shape: ButtonShape::Round,
                toggle: true,
                selected: true,
                ..Default::default()
            },
            cx,
        );
        cx.notify();
    });
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(20));
    draw(cx);
    assert_ne!(radius[0].get(), initial);
    view.update(cx, |this, cx| {
        this.selected = false;
        this.playback.sync(
            super::ButtonTransitionInputs {
                variant: ButtonVariant::Filled,
                size: ButtonSize::Large,
                shape: ButtonShape::Round,
                toggle: true,
                selected: false,
                ..Default::default()
            },
            cx,
        );
        cx.notify();
    });
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(5));
    draw(cx);
    view.update(cx, |this, cx| {
        this.mount_second = true;
        cx.notify();
    });
    draw(cx);
    assert_ne!(
        radius[0].get(),
        initial,
        "reverse transition must still be running"
    );
    assert!((radius[0].get() - radius[1].get()).abs() < 0.00001);
    assert_eq!(
        colors[0].get(),
        colors[1].get(),
        "offscreen retarget must retain Base reversal history"
    );
}

#[gpui::test]
fn offscreen_settled_toggle_uses_new_scheme_for_next_target(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        cx.set_reduce_motion(false);
    });
    let radius = std::array::from_fn(|_| Rc::new(Cell::new(f32::NAN)));
    let colors = std::array::from_fn(|_| {
        Rc::new(Cell::new((
            gpui::Rgba::default(),
            gpui::Rgba::default(),
            gpui::Rgba::default(),
        )))
    });
    let (view, cx) = cx.add_window_view(|_, _| ToggleUnmountHarness {
        playback: ButtonTransitionPlayback::default(),
        scheme: crate::motion::MotionScheme::Expressive,
        selected: false,
        mount_second: true,
        radius: radius.clone(),
        colors: colors.clone(),
    });
    draw(cx);
    view.update(cx, |this, cx| {
        this.selected = true;
        cx.notify();
    });
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(10));
    draw(cx);
    view.update(cx, |this, cx| {
        this.mount_second = false;
        cx.notify();
    });
    draw(cx);
    cx.executor().advance_clock(Duration::from_secs(2));
    draw(cx);
    // Only the mounted control has observed completion. The retained handle
    // must settle its previous target before selecting the new Standard curve.
    view.update(cx, |this, cx| {
        this.scheme = crate::motion::MotionScheme::Standard;
        this.selected = false;
        this.mount_second = true;
        cx.notify();
    });
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(20));
    draw(cx);
    assert!(
        (radius[0].get() - radius[1].get()).abs() < 0.00001,
        "new target must use the current scheme after an offscreen completion"
    );
    assert_eq!(colors[0].get(), colors[1].get());
}

#[gpui::test]
fn visible_target_updates_sync_cache_before_offscreen_reversal(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        cx.set_reduce_motion(false);
    });
    let radius = std::array::from_fn(|_| Rc::new(Cell::new(f32::NAN)));
    let colors = std::array::from_fn(|_| {
        Rc::new(Cell::new((
            gpui::Rgba::default(),
            gpui::Rgba::default(),
            gpui::Rgba::default(),
        )))
    });
    let (view, cx) = cx.add_window_view(|_, _| ToggleUnmountHarness {
        playback: ButtonTransitionPlayback::default(),
        scheme: crate::motion::MotionScheme::Expressive,
        selected: false,
        mount_second: true,
        radius: radius.clone(),
        colors: colors.clone(),
    });
    view.update(cx, |this, cx| {
        this.playback.sync(
            super::ButtonTransitionInputs {
                size: ButtonSize::Large,
                toggle: true,
                ..Default::default()
            },
            cx,
        );
    });
    draw(cx);
    let initial = radius[0].get();
    view.update(cx, |this, cx| {
        this.selected = true;
        cx.notify();
    });
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(20));
    draw(cx);
    assert_ne!(radius[0].get(), initial);
    view.update(cx, |this, cx| {
        this.selected = false;
        this.mount_second = false;
        this.playback.sync(
            super::ButtonTransitionInputs {
                variant: ButtonVariant::Filled,
                size: ButtonSize::Large,
                shape: ButtonShape::Round,
                toggle: true,
                selected: false,
                ..Default::default()
            },
            cx,
        );
        cx.notify();
    });
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(5));
    draw(cx);
    view.update(cx, |this, cx| {
        this.mount_second = true;
        cx.notify();
    });
    draw(cx);
    assert_ne!(
        radius[0].get(),
        initial,
        "reverse transition must still be running"
    );
    assert!((radius[0].get() - radius[1].get()).abs() < 0.00001);
    assert_eq!(
        colors[0].get(),
        colors[1].get(),
        "offscreen retarget must retain Base reversal history"
    );
}
