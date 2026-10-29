use super::{ButtonIconPosition, ButtonSize, ButtonVariant, MaterialButton};
use crate::icons::IconName;
use gpui::{
    Context, IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, ParentElement, Render,
    Styled, TestAppContext, Window, accesskit, div, prelude::*, px,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

struct Probe {
    size: ButtonSize,
    variant: ButtonVariant,
    icon: Option<ButtonIconPosition>,
    progress: bool,
    loading: bool,
    disabled: bool,
    toggle: bool,
    clicks: usize,
    a11y: Rc<RefCell<Vec<accesskit::Node>>>,
    opacity: Rc<Cell<f32>>,
}
impl Render for Probe {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let mut button = MaterialButton::new("loading-probe")
            .button_size(self.size)
            .variant(self.variant)
            .toggle(self.toggle)
            .disabled(self.disabled)
            .progress(self.progress, "Saving data")
            .loading(self.loading, "Saving data")
            .on_click(cx.listener(|this, _, _, _| this.clicks += 1))
            .debug_selector(|| "loading-probe".into())
            .child(
                div()
                    .w(px(80.))
                    .h(px(20.))
                    .debug_selector(|| "loading-label".into()),
            );
        if let Some(position) = self.icon {
            button = button.icon(IconName::Check.icon()).icon_position(position);
        }
        button.base.a11y_observer = Some(self.a11y.clone());
        button.progress_opacity_observer = Some(self.opacity.clone());
        div().flex().items_start().child(button)
    }
}

#[gpui::test]
fn loading_and_progress_have_distinct_activation_and_busy_semantics(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        cx.set_reduce_motion(true);
    });
    let a11y = Rc::new(RefCell::new(Vec::new()));
    let (view, cx) = cx.add_window_view({
        let a11y = a11y.clone();
        move |_, _| Probe {
            size: ButtonSize::Small,
            variant: ButtonVariant::Filled,
            icon: None,
            progress: false,
            loading: false,
            disabled: false,
            toggle: false,
            clicks: 0,
            a11y,
            opacity: Rc::new(Cell::new(0.)),
        }
    });
    for toggle in [false, true] {
        for (progress, loading, disabled) in [
            (true, false, false),
            (false, true, false),
            (true, true, false),
            (false, false, true),
            (false, false, false),
        ] {
            view.update(cx, |state, cx| {
                state.progress = progress;
                state.loading = loading;
                state.disabled = disabled;
                state.toggle = toggle;
                cx.notify();
            });
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
            let nodes = a11y.borrow();
            let node = nodes.last().unwrap();
            assert_eq!(node.is_busy(), loading);
            assert_eq!(node.is_disabled(), disabled || loading);
            assert_eq!(
                node.supports_action(accesskit::Action::Click),
                !disabled && !loading
            );
            drop(nodes);
            assert_eq!(
                cx.debug_bounds("material-button-progress-slot").is_some(),
                progress || loading
            );
            let before = cx.update(|_, cx| view.read(cx).clicks);
            let center = cx.debug_bounds("loading-probe").unwrap().center();
            cx.simulate_click(center, Modifiers::default());
            for name in ["enter", "space"] {
                let keystroke = cx
                    .update(|_, cx| Keystroke::parse_with_os(name, cx.platform_info().os).unwrap());
                cx.simulate_event(KeyDownEvent {
                    keystroke: keystroke.clone(),
                    is_held: false,
                    prefer_character_input: false,
                });
                cx.simulate_event(KeyUpEvent { keystroke });
            }
            assert_eq!(
                cx.update(|_, cx| view.read(cx).clicks) - before,
                3 * usize::from(!disabled && !loading)
            );
        }
    }
}

#[gpui::test]
fn loading_slot_preserves_original_icon_padding_and_moves_trailing_icon_to_start(
    cx: &mut TestAppContext,
) {
    cx.update(|cx| {
        gpui_base::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|_, _| Probe {
        size: ButtonSize::Small,
        variant: ButtonVariant::Filled,
        icon: None,
        progress: false,
        loading: false,
        disabled: false,
        toggle: false,
        clicks: 0,
        a11y: Rc::new(RefCell::new(Vec::new())),
        opacity: Rc::new(Cell::new(0.)),
    });
    for variant in ButtonVariant::ALL {
        for size in ButtonSize::ALL {
            for icon in [
                None,
                Some(ButtonIconPosition::Leading),
                Some(ButtonIconPosition::Trailing),
            ] {
                view.update(cx, |state, cx| {
                    state.variant = variant;
                    state.size = size;
                    state.icon = icon;
                    state.progress = false;
                    cx.notify();
                });
                cx.update(|window, cx| {
                    window.refresh();
                    window.draw(cx).clear(cx);
                });
                let before = cx.debug_bounds("loading-probe").unwrap();
                view.update(cx, |state, cx| {
                    state.progress = true;
                    cx.notify();
                });
                cx.update(|window, cx| {
                    window.refresh();
                    window.draw(cx).clear(cx);
                });
                let after = cx.debug_bounds("loading-probe").unwrap();
                let slot = cx.debug_bounds("material-button-progress-slot").unwrap();
                let label = cx.debug_bounds("loading-label").unwrap();
                let expected_delta = if icon.is_none() {
                    size.icon_size() + size.geometry().gap
                } else {
                    0.
                };
                assert!(
                    (f32::from(after.size.width - before.size.width) - expected_delta).abs() < 0.02,
                    "{variant:?} {size:?} {icon:?}"
                );
                assert_eq!(slot.size.width, px(size.icon_size()));
                assert!(slot.right() <= label.left());
            }
        }
    }
}

#[gpui::test]
fn visible_progress_fades_when_disabled_but_newly_shown_progress_adopts_opacity(
    cx: &mut TestAppContext,
) {
    cx.update(|cx| {
        gpui_base::init(cx);
        cx.set_reduce_motion(true);
    });
    let opacity = Rc::new(Cell::new(0.));
    let (view, cx) = cx.add_window_view({
        let opacity = opacity.clone();
        move |_, _| Probe {
            size: ButtonSize::Small,
            variant: ButtonVariant::Filled,
            icon: None,
            progress: true,
            loading: false,
            disabled: false,
            toggle: false,
            clicks: 0,
            a11y: Rc::new(RefCell::new(Vec::new())),
            opacity,
        }
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert_eq!(opacity.get(), 1.);
    view.update(cx, |state, cx| {
        state.loading = true;
        cx.notify();
    });
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert_eq!(opacity.get(), 1.);
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(50));
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    let expected = 1.
        - 0.3
            * gpui_base::Easing::cubic_bezier(0.2, 0., 0., 1.)
                .unwrap()
                .sample(0.5);
    assert!((opacity.get() - expected).abs() < 0.00001);
    view.update(cx, |state, cx| {
        state.loading = false;
        state.progress = false;
        cx.notify();
    });
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    view.update(cx, |state, cx| {
        state.loading = true;
        cx.notify();
    });
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert_eq!(opacity.get(), 0.7);
}
