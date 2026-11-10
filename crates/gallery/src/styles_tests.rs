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
fn the_home_repaints_the_gallery_and_opens_pages(cx: &mut TestAppContext) {
    use gpui_kit_material::prelude::ThemeId;
    let (window, mut cx) = open(cx);
    settle(&window, &mut cx);
    let (shared, lead) = window
        .update(&mut cx, |g, _, _| (g.shared.clone(), g.lead.clone()))
        .unwrap();
    // A hero shape morphs to the next one when pressed, without leaving the page.
    let shapes = |cx: &mut VisualTestContext| {
        cx.update(|_, cx| lead.read(cx).home.clone().unwrap().read(cx).shapes)
    };
    let before = shapes(&mut cx);
    let shape = bounds(&mut cx, "home-shape-0");
    cx.simulate_click(shape.center(), Modifiers::none());
    settle(&window, &mut cx);
    assert_eq!(shapes(&mut cx)[0], before[0] + 1);
    assert_eq!(shapes(&mut cx)[1..], before[1..]);
    window
        .update(&mut cx, |g, _, _| assert_eq!(g.current, 0))
        .unwrap();
    // A swatch, scrolled into view under the hero, paints its theme across
    // the Gallery.
    let scroll = |window: &WindowHandle<Gallery>, cx: &mut VisualTestContext, offset: f32| {
        window
            .update(cx, |g, _, cx| {
                g.list.scroll_to(gpui::ListOffset {
                    item_ix: 0,
                    offset_in_item: px(offset),
                });
                cx.notify();
            })
            .unwrap();
        settle(window, cx);
    };
    scroll(&window, &mut cx, 400.);
    let ocean = bounds(&mut cx, "home-swatch-3");
    cx.simulate_click(ocean.center(), Modifiers::none());
    settle(&window, &mut cx);
    cx.update(|_, cx| assert_eq!(shared.read(cx).reference_theme, Some(ThemeId::Ocean)));
    scroll(&window, &mut cx, 0.);
    // The hero's actions open the Styles and Examples pages.
    let styles = bounds(&mut cx, "home-styles");
    cx.simulate_click(styles.center(), Modifiers::none());
    settle(&window, &mut cx);
    window
        .update(&mut cx, |g, _, _| {
            assert_eq!(g.current, catalog::position("color").unwrap())
        })
        .unwrap();
    window
        .update(&mut cx, |g, _, cx| g.navigate(0, cx))
        .unwrap();
    settle(&window, &mut cx);
    let examples = bounds(&mut cx, "home-examples");
    cx.simulate_click(examples.center(), Modifiers::none());
    settle(&window, &mut cx);
    window
        .update(&mut cx, |g, _, _| {
            assert_eq!(g.current, catalog::position("examples").unwrap())
        })
        .unwrap();
}

/// Scroll the overview's first row (the home) by `offset`.
fn scroll_home(window: &WindowHandle<Gallery>, cx: &mut VisualTestContext, offset: f32) {
    window
        .update(cx, |g, _, cx| {
            g.list.scroll_to(gpui::ListOffset {
                item_ix: 0,
                offset_in_item: px(offset),
            });
            cx.notify();
        })
        .unwrap();
    settle(window, cx);
}

#[gpui::test]
fn the_copy_button_says_copied_for_a_moment(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    settle(&window, &mut cx);
    let lead = window.update(&mut cx, |g, _, _| g.lead.clone()).unwrap();
    let copied = |cx: &mut VisualTestContext| {
        cx.update(|_, cx| lead.read(cx).home.clone().unwrap().read(cx).copied)
    };
    scroll_home(&window, &mut cx, 600.);
    let copy = bounds(&mut cx, "home-install-copy");
    cx.simulate_click(copy.center(), Modifiers::none());
    settle(&window, &mut cx);
    assert!(copied(&mut cx));
    assert_eq!(
        cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text())),
        Some("cargo add gpui-kit-material".to_string())
    );
    // A second copy restarts the moment rather than ending it early.
    cx.executor()
        .advance_clock(super::home::COPIED_FOR - std::time::Duration::from_millis(500));
    cx.simulate_click(copy.center(), Modifiers::none());
    settle(&window, &mut cx);
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(1000));
    settle(&window, &mut cx);
    assert!(copied(&mut cx), "the second copy's moment is still running");
    cx.executor().advance_clock(super::home::COPIED_FOR);
    settle(&window, &mut cx);
    assert!(!copied(&mut cx), "the button offers to copy again");
}

#[gpui::test]
fn browsing_takes_focus_to_the_first_card(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    settle(&window, &mut cx);
    let browse = bounds(&mut cx, "home-browse");
    cx.simulate_click(browse.center(), Modifiers::none());
    settle(&window, &mut cx);
    // The Gallery moves focus on the frame after the one that drew the row.
    cx.update(|window, cx| window.simulate_next_frame(cx));
    settle(&window, &mut cx);
    let first = super::shell::overview_groups().next().unwrap();
    let card = catalog::CATALOG
        .iter()
        .position(|entry| entry.group == first && super::shell::overview_entry(entry))
        .unwrap();
    window
        .update(&mut cx, |g, window, cx| {
            assert!(
                g.row_focus[1].contains_focused(window, cx) && !g.row_focus[1].is_focused(window),
                "focus is in the first group"
            );
        })
        .unwrap();
    // The focused control is the first card: Enter opens its page.
    // GPUI's keyboard click is a press and a release.
    let enter = gpui::Keystroke::parse("enter").unwrap();
    cx.simulate_event(gpui::KeyDownEvent {
        keystroke: enter.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(gpui::KeyUpEvent { keystroke: enter });
    settle(&window, &mut cx);
    window
        .update(&mut cx, |g, _, _| assert_eq!(g.current, card))
        .unwrap();
}

/// The home's parts that must stay inside the window.
const HOME_PARTS: [&str; 13] = [
    "home-hero",
    "home-browse",
    "home-styles",
    "home-examples",
    "home-shape-3",
    "home-tile-theme",
    "home-swatch-7",
    "home-tile-live",
    "home-chip-2",
    "home-toggle-2",
    "home-tile-install",
    "home-install-copy",
    "home-library",
];

/// The home reflows at every width and text scale: nothing reaches past the
/// window's edge.
#[gpui::test]
fn the_home_fits_every_width(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    let sizes = [
        (1600., 1.),
        (1280., 1.),
        (1000., 1.),
        (840., 1.),
        (600., 1.),
        (412., 1.),
        (360., 1.),
        (412., 1.5),
        (360., 2.),
        (320., 2.),
    ];
    for (width, scale) in sizes {
        window
            .update(&mut cx, |_, window, _| {
                window.set_rem_size(px(gpui_kit_material::tokens::typography::BASE_PX * scale))
            })
            .unwrap();
        cx.simulate_resize(size(px(width), px(900.)));
        // Each part is checked where it is drawn, the page scrolled to it.
        let mut offset = 0.;
        let mut checked = std::collections::HashSet::new();
        while checked.len() < HOME_PARTS.len() && offset < 6000. {
            window
                .update(&mut cx, |g, _, cx| {
                    g.list.scroll_to(gpui::ListOffset {
                        item_ix: 0,
                        offset_in_item: px(offset),
                    });
                    cx.notify();
                })
                .unwrap();
            // A fit measures on one frame and draws to fit on the next.
            settle(&window, &mut cx);
            settle(&window, &mut cx);
            for selector in HOME_PARTS {
                let Some(element) = cx.debug_bounds(selector) else {
                    continue;
                };
                if element.bottom() < px(0.) || element.top() > px(900.) {
                    continue;
                }
                checked.insert(selector);
                assert!(
                    element.left() >= px(-0.5) && element.right() <= px(width) + px(0.5),
                    "{selector} at {width}px and {scale}x: {element:?}"
                );
            }
            offset += 300.;
        }
        assert_eq!(
            checked.len(),
            HOME_PARTS.len(),
            "at {width}px and {scale}x only {checked:?} were drawn"
        );
    }
}

#[gpui::test]
fn overview_cards_show_thumbnails_reflow_and_navigate(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    settle(&window, &mut cx);
    let rows = window.update(&mut cx, |g, _, _| g.rows.clone()).unwrap();
    assert!(rows[1..].iter().all(|row| matches!(row, Row::Group(_))));
    // The home's first action scrolls to the first group's cards.
    let browse = bounds(&mut cx, "home-browse");
    cx.simulate_click(browse.center(), Modifiers::none());
    settle(&window, &mut cx);
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
