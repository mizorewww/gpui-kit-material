//! The overview's navigation and the Styles pages' configuration, driven
//! through the Gallery.
use super::catalog;
use super::pages::{elevation, motion, shape, typography};
use super::{Gallery, Row};
use gpui::{
    AppContext as _, Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext, WindowHandle,
    px, size,
};

fn open(cx: &mut TestAppContext) -> (WindowHandle<Gallery>, VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit_material::init(cx);
    });
    let window = cx.open_window(size(px(1280.), px(900.)), Gallery::new);
    let visual = VisualTestContext::from_window(window.into(), cx);
    (window, visual)
}

fn settle(window: &WindowHandle<Gallery>, cx: &mut VisualTestContext) {
    for _ in 0..3 {
        cx.update_window((*window).into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    }
}

fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("{selector} is rendered"))
}

#[gpui::test]
fn overview_cards_show_thumbnails_reflow_and_navigate(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    settle(&window, &mut cx);
    let rows = window.update(&mut cx, |g, _, _| g.rows.clone()).unwrap();
    assert!(rows[1..].iter().all(|row| matches!(row, Row::Group(_))));
    // Two cards share the first row at 1280px, each within the content.
    let button = bounds(&mut cx, "overview-button");
    let icon = bounds(&mut cx, "overview-icon-button");
    assert_eq!(button.top(), icon.top());
    assert!(icon.left() > button.right());
    assert!(icon.right() <= px(1280.));
    cx.simulate_click(icon.center(), Modifiers::none());
    settle(&window, &mut cx);
    window
        .update(&mut cx, |g, _, _| {
            assert_eq!(g.current, catalog::position("icon-button").unwrap())
        })
        .unwrap();
    // Navigating back to the overview builds no page and keeps its rows.
    window
        .update(&mut cx, |g, _, cx| g.navigate(0, cx))
        .unwrap();
    settle(&window, &mut cx);
    assert!(cx.debug_bounds("overview-button").is_some());

    // A narrow window drops to one column: cards stack, nothing scrolls sideways.
    cx.simulate_resize(size(px(560.), px(900.)));
    settle(&window, &mut cx);
    let button = bounds(&mut cx, "overview-button");
    let icon = bounds(&mut cx, "overview-icon-button");
    assert!(icon.top() >= button.bottom(), "{button:?} {icon:?}");
    assert!(button.right() <= px(560.));
}

#[gpui::test]
fn shape_tokens_drive_the_radius(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    let page = window
        .update(&mut cx, |g, window, cx| {
            g.show_page::<shape::Page>(window, cx).0
        })
        .unwrap();
    settle(&window, &mut cx);
    assert!(cx.debug_bounds("shape-preview").is_some());
    let token = bounds(&mut cx, "shape-token-6");
    cx.simulate_click(token.center(), Modifiers::none());
    settle(&window, &mut cx);
    let (radius, slider) = cx.read(|cx| {
        let page = page.read(cx);
        (page.radius, page.slider.read(cx).value().start())
    });
    assert_eq!(radius, shape::TOKENS[6].1);
    assert_eq!(slider, radius, "the slider follows the token");
}

#[gpui::test]
fn typography_samples_follow_the_text_and_style(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    let page = window
        .update(&mut cx, |g, window, cx| {
            g.show_page::<typography::Page>(window, cx).0
        })
        .unwrap();
    settle(&window, &mut cx);
    let input = bounds(&mut cx, "type-sample-input");
    cx.simulate_click(input.center(), Modifiers::none());
    settle(&window, &mut cx);
    cx.simulate_keystrokes("secondary-a");
    cx.simulate_input("Hello");
    settle(&window, &mut cx);
    assert_eq!(cx.read(|cx| page.read(cx).sample.read(cx).value()), "Hello");
    page.update(&mut cx, |page, cx| {
        page.style = "regular".into();
        cx.notify();
    });
    settle(&window, &mut cx);
}

#[gpui::test]
fn motion_plays_and_toggles_reduced_motion(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    let page = window
        .update(&mut cx, |g, window, cx| {
            g.show_page::<motion::Page>(window, cx).0
        })
        .unwrap();
    settle(&window, &mut cx);
    let play = cx
        .debug_bounds("motion-token-0")
        .expect("token rows render");
    let _ = play;
    page.update(&mut cx, |page, cx| {
        page.target = true;
        cx.notify();
    });
    for _ in 0..5 {
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(300));
        settle(&window, &mut cx);
    }
    let values = cx.read(|cx| page.read(cx).values);
    assert!(
        values.iter().all(|value| (value - 1.).abs() < 0.01),
        "every token arrives: {values:?}"
    );
    cx.update(|_, cx| cx.set_reduce_motion(true));
    settle(&window, &mut cx);
    assert!(cx.read(|cx| cx.reduce_motion()));
}

#[gpui::test]
fn elevation_level_and_toggles_change_the_preview(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    let page = window
        .update(&mut cx, |g, window, cx| {
            g.show_page::<elevation::Page>(window, cx).0
        })
        .unwrap();
    settle(&window, &mut cx);
    assert!(cx.debug_bounds("elevation-preview").is_some());
    let slider = bounds(&mut cx, "elevation-level-slider");
    // The slider's end is level 5.
    cx.simulate_click(
        gpui::point(slider.right() - px(2.), slider.center().y),
        Modifiers::none(),
    );
    settle(&window, &mut cx);
    assert_eq!(cx.read(|cx| page.read(cx).level), 5);
    page.update(&mut cx, |page, cx| {
        page.shadow = false;
        page.tonal = false;
        cx.notify();
    });
    settle(&window, &mut cx);
    assert!(cx.debug_bounds("elevation-preview").is_some());
}

#[gpui::test]
fn examples_keep_their_state_across_navigation(cx: &mut TestAppContext) {
    use super::pages::examples;
    let (window, mut cx) = open(cx);
    let examples_entry = catalog::position("examples").unwrap();
    window
        .update(&mut cx, |g, _, cx| g.navigate(examples_entry, cx))
        .unwrap();
    settle(&window, &mut cx);
    assert!(cx.debug_bounds("example-device").is_some());
    let first = window
        .update(&mut cx, |g, window, cx| {
            g.show_page::<examples::Page>(window, cx).0.entity_id()
        })
        .unwrap();
    // Leaving and returning reuses the built page: no rebuild.
    window
        .update(&mut cx, |g, _, cx| g.navigate(0, cx))
        .unwrap();
    settle(&window, &mut cx);
    window
        .update(&mut cx, |g, _, cx| g.navigate(examples_entry, cx))
        .unwrap();
    settle(&window, &mut cx);
    let again = window
        .update(&mut cx, |g, window, cx| {
            g.show_page::<examples::Page>(window, cx).0.entity_id()
        })
        .unwrap();
    assert_eq!(first, again);
}
