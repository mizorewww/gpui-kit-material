use super::*;
use core::prelude::v1::test;

#[test]
fn filled_and_outlined_paint_m3_roles_in_every_state() {
    use ColorRole as C;
    let rest = FieldState::default();
    let filled = field_paint(TextFieldVariant::Filled, rest);
    assert_eq!(filled.container, color(C::SURFACE_CONTAINER_HIGHEST));
    assert_eq!(filled.indicator, color(C::ON_SURFACE_VARIANT));
    assert_eq!(filled.indicator_width, 1.);
    assert_eq!(filled.label, color(C::ON_SURFACE_VARIANT));
    assert_eq!(filled.input, color(C::ON_SURFACE));
    assert_eq!(filled.state_layer.a, 0.);

    let hovered = field_paint(
        TextFieldVariant::Filled,
        FieldState {
            hovered: true,
            ..rest
        },
    );
    assert_eq!(hovered.indicator, color(C::ON_SURFACE));
    assert_eq!(hovered.state_layer.a, color(C::ON_SURFACE).a * state::HOVER);

    let focused = field_paint(
        TextFieldVariant::Outlined,
        FieldState {
            focused: true,
            ..rest
        },
    );
    assert_eq!(focused.indicator, color(C::PRIMARY));
    assert_eq!(focused.indicator_width, 3.);
    assert_eq!(focused.label, color(C::PRIMARY));
    assert_eq!(focused.caret, color(C::PRIMARY), "the caret is primary");
    assert_eq!(focused.container.a, 0., "outlined fields have no container");

    let outlined = field_paint(TextFieldVariant::Outlined, rest);
    assert_eq!(outlined.indicator, color(C::OUTLINE));

    let error = field_paint(
        TextFieldVariant::Filled,
        FieldState {
            error: true,
            ..rest
        },
    );
    assert_eq!(error.indicator, color(C::ERROR));
    assert_eq!(error.label, color(C::ERROR));
    assert_eq!(error.trailing, color(C::ERROR));
    assert_eq!(error.supporting, color(C::ERROR));
    assert_eq!(
        error.caret,
        color(C::ERROR),
        "an error field's caret is error"
    );
    let error_hover = field_paint(
        TextFieldVariant::Filled,
        FieldState {
            error: true,
            hovered: true,
            ..rest
        },
    );
    assert_eq!(error_hover.indicator, color(C::ON_ERROR_CONTAINER));

    let disabled = field_paint(
        TextFieldVariant::Outlined,
        FieldState {
            disabled: true,
            error: true,
            ..rest
        },
    );
    let on_surface = color(C::ON_SURFACE);
    assert_eq!(disabled.label, state::disabled(on_surface));
    assert_eq!(disabled.supporting, state::disabled(on_surface));
    assert_eq!(disabled.indicator.a, on_surface.a * 0.12);
    let disabled_filled = field_paint(
        TextFieldVariant::Filled,
        FieldState {
            disabled: true,
            ..rest
        },
    );
    assert_eq!(disabled_filled.container.a, on_surface.a * 0.04);
}

#[test]
fn the_label_floats_from_body_large_to_body_small() {
    let (left, top, size, line) = label_geometry(TextFieldVariant::Filled, false, 0.);
    assert_eq!((left, top, size, line), (16., 16., 16., 24.));
    let (left, top, size, line) = label_geometry(TextFieldVariant::Filled, false, 1.);
    assert_eq!((left, top, size, line), (16., 8., 12., 16.));
    // A leading icon keeps a filled label after it, but an outlined label
    // floats over the outline at the content edge.
    assert_eq!(label_geometry(TextFieldVariant::Filled, true, 1.).0, 52.);
    assert_eq!(label_geometry(TextFieldVariant::Outlined, true, 0.).0, 52.);
    let (left, top, ..) = label_geometry(TextFieldVariant::Outlined, true, 1.);
    assert_eq!((left, top), (16., -8.));
}

#[test]
fn the_notch_opens_around_the_floating_label() {
    assert_eq!(notch(40., 1.), (12., 48.));
    assert_eq!(notch(40., 0.).1, 0.);
    let (start, width) = notch(40., 0.5);
    assert_eq!(width, 24.);
    assert_eq!(start + width / 2., 36., "the notch opens from its center");
    assert_eq!(
        notch(40., 1.2),
        notch(40., 1.),
        "overshoot never over-opens"
    );
}

struct Harness {
    state: Entity<InputState>,
    variant: TextFieldVariant,
    disabled: bool,
}

impl Render for Harness {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div().p(px(20.)).child(
            div().debug_selector(|| "field".into()).w(px(210.)).child(
                TextField::new("field", &self.state)
                    .variant(self.variant)
                    .label("Label")
                    .supporting_text("Supporting")
                    .max_length(20)
                    .disabled(self.disabled),
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

fn setup(
    cx: &mut TestAppContext,
    variant: TextFieldVariant,
    disabled: bool,
) -> (Entity<Harness>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_reduce_motion(true);
    });
    cx.add_window_view(|window, cx| Harness {
        state: cx.new(|cx| InputState::new(window, cx)),
        variant,
        disabled,
    })
}

#[gpui::test]
fn typing_floats_the_label_and_counts_characters(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, TextFieldVariant::Filled, false);
    draw(cx);
    let field = cx.debug_bounds("field").unwrap();
    let label = cx.debug_bounds("text-field-label").unwrap();
    assert_eq!(
        label.top() - field.top(),
        px(16.),
        "resting label is centered"
    );
    let indicator = cx.debug_bounds("text-field-indicator").unwrap();
    assert_eq!(indicator.size.height, px(1.));
    assert_eq!(indicator.bottom(), field.top() + px(56.));

    cx.simulate_click(field.center(), Modifiers::default());
    cx.simulate_input("Hello");
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).state.read(cx).value().as_ref(), "Hello"));
    let label = cx.debug_bounds("text-field-label").unwrap();
    assert_eq!(label.top() - field.top(), px(8.), "the label floats");
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    draw(cx);
    let indicator = cx.debug_bounds("text-field-focus-indicator").unwrap();
    assert_eq!(
        indicator.size.height,
        px(3.),
        "focus fades in a 3dp indicator"
    );
    assert_eq!(indicator.bottom(), field.top() + px(56.));
}

#[gpui::test]
fn the_outlined_notch_opens_when_the_label_floats(cx: &mut TestAppContext) {
    let (_, cx) = setup(cx, TextFieldVariant::Outlined, false);
    draw(cx);
    assert!(
        cx.debug_bounds("text-field-outline-end").is_none(),
        "a resting label leaves the outline whole"
    );
    let field = cx.debug_bounds("field").unwrap();
    cx.simulate_click(field.center(), Modifiers::default());
    draw(cx);
    let start = cx.debug_bounds("text-field-outline-start").unwrap();
    let end = cx.debug_bounds("text-field-outline-end").unwrap();
    let label = cx.debug_bounds("text-field-label").unwrap();
    assert_eq!(start.right(), field.left() + px(12.));
    assert!(end.left() > label.left(), "the notch spans the label");
    assert_eq!(label.top(), field.top() - px(8.), "centered on the outline");
}

#[gpui::test]
fn a_disabled_field_takes_no_focus_or_text(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, TextFieldVariant::Filled, true);
    draw(cx);
    let field = cx.debug_bounds("field").unwrap();
    cx.simulate_click(field.center(), Modifiers::default());
    cx.simulate_input("x");
    draw(cx);
    cx.update(|window, cx| {
        let state = view.read(cx).state.read(cx);
        assert!(state.value().is_empty());
        assert!(!state.focus_handle(cx).is_focused(window));
    });
}
