use super::*;
use core::prelude::v1::test;
use gpui::{Modifiers, TestAppContext, VisualTestContext};
use std::time::Duration;

#[test]
fn standard_buttons_use_secondary_container_when_selected() {
    let rest = button_paint(ToolbarColor::Standard, false, 0., false);
    assert_eq!(
        rest.container.a, 0.,
        "icon buttons have no resting container"
    );
    assert_eq!(rest.ink, color(ColorRole::ON_SURFACE_VARIANT));
    let selected = button_paint(ToolbarColor::Standard, false, 1., false);
    assert_eq!(selected.container, color(ColorRole::SECONDARY_CONTAINER));
    assert_eq!(selected.ink, color(ColorRole::ON_SECONDARY_CONTAINER));
    let label = button_paint(ToolbarColor::Standard, true, 0., false);
    assert_eq!(label.container, color(ColorRole::SURFACE_CONTAINER));
}

#[test]
fn vibrant_buttons_invert_to_surface_container_when_selected() {
    let rest = button_paint(ToolbarColor::Vibrant, false, 0., false);
    assert_eq!(rest.ink, color(ColorRole::ON_PRIMARY_CONTAINER));
    let label = button_paint(ToolbarColor::Vibrant, true, 0., false);
    assert_eq!(label.container, color(ColorRole::PRIMARY_CONTAINER));
    let selected = button_paint(ToolbarColor::Vibrant, true, 1., false);
    assert_eq!(selected.container, color(ColorRole::SURFACE_CONTAINER));
    assert_eq!(selected.ink, color(ColorRole::ON_SURFACE));
}

#[test]
fn disabled_buttons_use_on_surface_emphasis() {
    for scheme in ToolbarColor::ALL {
        let disabled = button_paint(scheme, false, 1., true);
        assert_eq!(disabled.ink, state::disabled(color(ColorRole::ON_SURFACE)));
        assert_eq!(
            disabled.container,
            color(ColorRole::ON_SURFACE).opacity(0.10)
        );
        assert_eq!(disabled.layer.a, 0.);
    }
}

#[test]
fn containers_follow_the_color_scheme() {
    assert_eq!(
        ToolbarColor::Standard.container(),
        ColorRole::SURFACE_CONTAINER
    );
    assert_eq!(
        ToolbarColor::Vibrant.container(),
        ColorRole::PRIMARY_CONTAINER
    );
}

struct Harness {
    configuration: ToolbarConfiguration,
    orientation: ToolbarOrientation,
    visible: bool,
    selected: bool,
    clicks: usize,
}

fn icon() -> NavIcon {
    NavIcon::new("outlined.svg", "filled.svg")
}

impl Render for Harness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let toggle = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.selected = !this.selected;
            this.clicks += 1;
            cx.notify();
        });
        div().w(px(412.)).h(px(400.)).child(
            Toolbar::new("toolbar", "Actions")
                .configuration(self.configuration)
                .orientation(self.orientation)
                .visible(self.visible)
                .item(
                    ToolbarButton::icon("first", icon(), "First")
                        .toggle(true)
                        .selected(self.selected)
                        .on_click(toggle),
                )
                .item(ToolbarButton::icon("second", icon(), "Second"))
                .item(ToolbarButton::label("third", "Third")),
        )
    }
}

fn harness(
    cx: &mut TestAppContext,
    configuration: ToolbarConfiguration,
    orientation: ToolbarOrientation,
) -> (Entity<Harness>, &mut VisualTestContext) {
    cx.update(gpui_base::init);
    cx.add_window_view(move |_, _| Harness {
        configuration,
        orientation,
        visible: true,
        selected: false,
        clicks: 0,
    })
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

fn settle(cx: &mut VisualTestContext) {
    draw(cx);
    cx.executor().advance_clock(Duration::from_secs(2));
    draw(cx);
}

fn painted(part: &str) -> (ButtonPaint, f32) {
    PAINTED.with(|painted| {
        *painted
            .borrow()
            .iter()
            .find(|(key, _)| key.contains(part))
            .expect("the button painted")
            .1
    })
}

fn shown() -> f32 {
    SHOWN.with(|shown| *shown.borrow().values().next().expect("toolbar drawn"))
}

#[gpui::test]
fn floating_toolbars_are_64dp_pills(cx: &mut TestAppContext) {
    let (_, cx) = harness(
        cx,
        ToolbarConfiguration::Floating,
        ToolbarOrientation::Horizontal,
    );
    settle(cx);
    let bounds = cx.debug_bounds("toolbar").unwrap();
    assert_eq!(bounds.size.height, px(64.));
    assert!(bounds.size.width < px(412.), "hugs its items");
}

#[gpui::test]
fn docked_toolbars_span_the_width(cx: &mut TestAppContext) {
    let (_, cx) = harness(
        cx,
        ToolbarConfiguration::Docked,
        ToolbarOrientation::Horizontal,
    );
    settle(cx);
    let bounds = cx.debug_bounds("toolbar").unwrap();
    assert_eq!(bounds.size, size(px(412.), px(64.)));
}

#[gpui::test]
fn vertical_toolbars_are_64dp_wide(cx: &mut TestAppContext) {
    let (_, cx) = harness(
        cx,
        ToolbarConfiguration::Floating,
        ToolbarOrientation::Vertical,
    );
    settle(cx);
    let bounds = cx.debug_bounds("toolbar").unwrap();
    assert_eq!(bounds.size.width, px(64.));
    assert!(bounds.size.height > bounds.size.width);
}

#[gpui::test]
fn clicking_a_toggle_selects_it_with_a_fade(cx: &mut TestAppContext) {
    let (view, cx) = harness(
        cx,
        ToolbarConfiguration::Floating,
        ToolbarOrientation::Horizontal,
    );
    settle(cx);
    assert_eq!(painted("first").0.container.a, 0.);
    let toolbar = cx.debug_bounds("toolbar").unwrap();
    // The first item's target starts at the 8dp padding.
    let at = point(toolbar.left() + px(8. + 24.), toolbar.center().y);
    cx.simulate_click(at, Modifiers::default());
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).clicks, 1));
    cx.executor().advance_clock(Duration::from_millis(20));
    draw(cx);
    let mid = painted("first").0.container.a;
    assert!(mid > 0. && mid < 1., "{mid}");
    settle(cx);
    assert_eq!(
        painted("first").0.container,
        color(ColorRole::SECONDARY_CONTAINER)
    );
}

#[gpui::test]
fn pressing_morphs_the_corners(cx: &mut TestAppContext) {
    let (_, cx) = harness(
        cx,
        ToolbarConfiguration::Floating,
        ToolbarOrientation::Horizontal,
    );
    settle(cx);
    assert_eq!(painted("second").1, 20.);
    let toolbar = cx.debug_bounds("toolbar").unwrap();
    let at = point(toolbar.left() + px(8. + 48. + 4. + 24.), toolbar.center().y);
    cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::default());
    settle(cx);
    assert!((painted("second").1 - PRESSED_RADIUS).abs() < 0.01);
    cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::default());
    settle(cx);
    assert!((painted("second").1 - 20.).abs() < 0.01);
}

#[gpui::test]
fn hiding_slides_on_the_spatial_spring_and_reduced_motion_jumps(cx: &mut TestAppContext) {
    let (view, cx) = harness(
        cx,
        ToolbarConfiguration::Floating,
        ToolbarOrientation::Horizontal,
    );
    settle(cx);
    assert_eq!(shown(), 1.);
    view.update(cx, |view, cx| {
        view.visible = false;
        cx.notify();
    });
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(60));
    draw(cx);
    let mid = shown();
    assert!(mid > 0. && mid < 1., "{mid}");
    settle(cx);
    assert!(shown().abs() < 1e-3);

    cx.update(|_, cx| cx.set_reduce_motion(true));
    view.update(cx, |view, cx| {
        view.visible = true;
        cx.notify();
    });
    draw(cx);
    draw(cx);
    assert_eq!(shown(), 1., "spatial motion completes at once");
}

#[gpui::test]
fn arrows_rove_focus_along_the_orientation(cx: &mut TestAppContext) {
    for (orientation, next, previous) in [
        (ToolbarOrientation::Horizontal, "right", "left"),
        (ToolbarOrientation::Vertical, "down", "up"),
    ] {
        let (_, cx) = harness(cx, ToolbarConfiguration::Floating, orientation);
        settle(cx);
        cx.update(|window, cx| window.focus_next(cx));
        draw(cx);
        let first = cx.update(|window, cx| window.focused(cx)).unwrap();
        cx.simulate_keystrokes(next);
        draw(cx);
        let second = cx.update(|window, cx| window.focused(cx)).unwrap();
        assert_ne!(first, second, "{orientation:?}");
        cx.simulate_keystrokes(previous);
        draw(cx);
        assert_eq!(cx.update(|window, cx| window.focused(cx)).unwrap(), first);
    }
}

#[gpui::test]
fn previewed_buttons_leave_the_tab_order(cx: &mut TestAppContext) {
    struct Specimen;
    impl Render for Specimen {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().child(
                ToolbarButton::icon("specimen", icon(), "Specimen")
                    .preview_state(InteractionPreview::Pressed),
            )
        }
    }
    cx.update(gpui_base::init);
    let (_, cx) = cx.add_window_view(|_, _| Specimen);
    settle(cx);
    cx.update(|window, cx| window.focus_next(cx));
    assert!(cx.update(|window, cx| window.focused(cx)).is_none());
    assert!((painted("specimen").1 - PRESSED_RADIUS).abs() < 0.01);
}
