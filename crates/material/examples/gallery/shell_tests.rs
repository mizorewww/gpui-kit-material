//! Gallery shell behavior: navigation, lazily built pages, the Live preview
//! and Specs tabs, and playground configuration driving the preview.
use super::catalog::{self, CATALOG, Kind};
use super::pages::PAGES;
use super::playground::{Config, Value};
use super::shell::ShellLayout;
use super::{Gallery, Row, Tab};
use gpui::{
    AppContext, Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext, WindowHandle, point,
    px, size,
};

fn init(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit_material::init(cx);
    });
}

fn open(cx: &mut TestAppContext) -> (WindowHandle<Gallery>, VisualTestContext) {
    init(cx);
    let window = cx.open_window(size(px(1280.), px(900.)), Gallery::new);
    let mut visual = VisualTestContext::from_window(window.into(), cx);
    design_rem(&window, &mut visual);
    (window, visual)
}

/// The 100% text scale, as the theme sets it: the layout's breakpoints are in
/// design px, which window px only match at this rem.
fn design_rem(window: &WindowHandle<Gallery>, cx: &mut VisualTestContext) {
    window
        .update(cx, |_, window, _| {
            window.set_rem_size(px(gpui_kit_material::tokens::typography::BASE_PX))
        })
        .unwrap();
}

fn draw(window: &WindowHandle<Gallery>, cx: &mut VisualTestContext) {
    cx.update_window((*window).into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.run_until_parked();
}

fn settle(window: &WindowHandle<Gallery>, cx: &mut VisualTestContext) {
    for _ in 0..3 {
        draw(window, cx);
    }
}

fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("{selector} is rendered"))
}

fn click(window: &WindowHandle<Gallery>, cx: &mut VisualTestContext, at: gpui::Point<Pixels>) {
    cx.simulate_click(at, Modifiers::none());
    settle(window, cx);
}

fn go(window: &WindowHandle<Gallery>, cx: &mut VisualTestContext, id: &str) {
    let entry = catalog::position(id).unwrap();
    window
        .update(cx, |gallery, _, cx| gallery.navigate(entry, cx))
        .unwrap();
    settle(window, cx);
}

fn built_pages(window: &WindowHandle<Gallery>, cx: &mut VisualTestContext) -> Vec<&'static str> {
    window
        .update(cx, |gallery, _, _| {
            gallery
                .pages
                .iter()
                .enumerate()
                .filter(|(_, page)| page.is_some())
                .map(|(index, _)| PAGES[index].info.id)
                .collect()
        })
        .unwrap()
}

fn config(window: &WindowHandle<Gallery>, cx: &mut VisualTestContext) -> Config {
    window
        .update(cx, |gallery, _, cx| {
            let state = gallery.entries[gallery.current].playground.clone().unwrap();
            state.read(cx).config.clone()
        })
        .unwrap()
}

#[gpui::test]
fn navigation_switches_the_page_and_builds_only_pages_shown(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    settle(&window, &mut cx);
    // The overview lists cards and needs no design-kit page.
    assert!(built_pages(&window, &mut cx).is_empty());
    let overview = window.update(&mut cx, |g, _, _| g.rows.clone()).unwrap();
    assert_eq!(overview[0], Row::Lead);
    assert!(overview[1..].iter().all(|row| matches!(row, Row::Group(_))));

    let nav = bounds(&mut cx, "nav-button");
    click(&window, &mut cx, nav.center());
    window
        .update(&mut cx, |g, _, _| {
            assert_eq!(g.current, catalog::position("button").unwrap());
            // The preview tab is one row: header, preview and configuration.
            assert_eq!(g.rows, [Row::Lead]);
            assert!(g.entries[g.current].playground.is_some());
            assert!(
                g.entries
                    .iter()
                    .enumerate()
                    .all(|(i, e)| (i == g.current) == e.playground.is_some()),
                "only the shown page's playground is built"
            );
        })
        .unwrap();
    assert_eq!(built_pages(&window, &mut cx), ["buttons"]);
    assert!(cx.debug_bounds("page-button").is_some());
    assert!(cx.debug_bounds("playground-button").is_some());

    // An overview card navigates too.
    go(&window, &mut cx, "overview");
    let card = bounds(&mut cx, "overview-icon-button");
    click(&window, &mut cx, card.center());
    window
        .update(&mut cx, |g, _, _| {
            assert_eq!(g.current, catalog::position("icon-button").unwrap())
        })
        .unwrap();
    assert!(cx.debug_bounds("playground-button").is_none());
    assert!(cx.debug_bounds("preview-canvas").is_some());
    // Icon button shares the Buttons kit page: nothing new is built.
    assert_eq!(built_pages(&window, &mut cx), ["buttons"]);
    go(&window, &mut cx, "switch");
    assert!(cx.debug_bounds("playground-switch").is_some());
    assert_eq!(built_pages(&window, &mut cx), ["buttons", "switch"]);
}

#[gpui::test]
fn top_sections_and_search_drive_the_navigation(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    settle(&window, &mut cx);
    window
        .update(&mut cx, |g, _, cx| {
            g.navigate(catalog::first_of(catalog::TopSection::Styles), cx)
        })
        .unwrap();
    settle(&window, &mut cx);
    assert!(cx.debug_bounds("nav-color").is_some());
    assert!(
        cx.debug_bounds("nav-button").is_none(),
        "Styles lists foundations"
    );
    assert!(
        cx.debug_bounds("specs-color").is_some(),
        "documents show their body"
    );
    window
        .update(&mut cx, |g, _, cx| g.set_query("sli".into(), cx))
        .unwrap();
    settle(&window, &mut cx);
    assert!(
        cx.debug_bounds("nav-slider").is_some(),
        "search spans sections"
    );
    assert!(cx.debug_bounds("nav-color").is_none());
    let matches: Vec<_> = window
        .update(&mut cx, |g, _, _| g.matches().collect())
        .unwrap();
    assert_eq!(matches, [catalog::position("slider").unwrap()]);
}

#[gpui::test]
fn specs_tab_lists_the_entry_slice_and_keeps_tab_and_config(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    go(&window, &mut cx, "button");
    let tabs = bounds(&mut cx, "card-tabs");
    click(
        &window,
        &mut cx,
        point(tabs.right() - tabs.size.width / 4., tabs.center().y),
    );
    let slice: Vec<usize> = (CATALOG[catalog::position("button").unwrap()]
        .spec
        .unwrap()
        .rows
        .unwrap())();
    window
        .update(&mut cx, |g, _, _| {
            assert_eq!(g.entries[g.current].tab, Tab::Specs);
            assert_eq!(g.rows.len(), 1 + slice.len());
            assert_eq!(g.rows[1], Row::Spec(slice[0]));
        })
        .unwrap();
    assert!(cx.debug_bounds("specs-button").is_some());
    assert!(
        cx.debug_bounds("playground-button").is_none(),
        "the preview is not built"
    );

    // Navigation keeps each page's tab and configuration.
    go(&window, &mut cx, "switch");
    window
        .update(&mut cx, |g, window, cx| {
            let state = g.entries[g.current].playground.clone().unwrap();
            state.update(cx, |state, cx| {
                state.set_value("checked", Value::Bool(false), window, cx)
            });
        })
        .unwrap();
    go(&window, &mut cx, "button");
    window
        .update(&mut cx, |g, _, _| {
            assert_eq!(g.entries[g.current].tab, Tab::Specs)
        })
        .unwrap();
    go(&window, &mut cx, "switch");
    assert!(!config(&window, &mut cx).bool("checked"));
    window
        .update(&mut cx, |g, _, _| assert_eq!(g.rows, [Row::Lead]))
        .unwrap();
}

#[gpui::test]
fn configuration_controls_drive_the_preview_and_reset_restores_it(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    go(&window, &mut cx, "button");
    let initial = bounds(&mut cx, "playground-button");
    let defaults = config(&window, &mut cx);

    // Size: the segmented control's last option (XL) grows the button.
    let sizes = bounds(&mut cx, "playground-control-size");
    click(
        &window,
        &mut cx,
        point(sizes.right() - px(12.), sizes.bottom() - px(16.)),
    );
    assert_eq!(config(&window, &mut cx).option("size"), "XL");
    let large = bounds(&mut cx, "playground-button");
    assert!(
        large.size.height > initial.size.height,
        "{large:?} vs {initial:?}"
    );
    assert_eq!(
        window
            .update(&mut cx, |g, _, cx| g.entries[g.current]
                .playground
                .clone()
                .unwrap()
                .read(cx)
                .scenario)
            .unwrap(),
        None,
        "a change makes the configuration custom"
    );

    // Square shape: the switch at the end of its row.
    let square = bounds(&mut cx, "playground-control-square");
    click(
        &window,
        &mut cx,
        point(square.right() - px(20.), square.center().y),
    );
    assert!(config(&window, &mut cx).bool("square"));

    let reset = bounds(&mut cx, "configuration-reset");
    click(&window, &mut cx, reset.center());
    assert_eq!(config(&window, &mut cx), defaults);
    assert_eq!(bounds(&mut cx, "playground-button").size, initial.size);
}

#[gpui::test]
fn the_preview_writes_back_to_its_configuration(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    go(&window, &mut cx, "switch");
    assert!(config(&window, &mut cx).bool("checked"));
    let switch = bounds(&mut cx, "playground-switch");
    click(&window, &mut cx, switch.center());
    assert!(!config(&window, &mut cx).bool("checked"));

    // The text field's editor and its Value control share one value.
    go(&window, &mut cx, "text-field");
    let field = bounds(&mut cx, "playground-text-field");
    click(&window, &mut cx, field.center());
    cx.simulate_input("Hello");
    settle(&window, &mut cx);
    assert_eq!(config(&window, &mut cx).text("value"), "Hello");
    window
        .update(&mut cx, |g, window, cx| {
            let state = g.entries[g.current].playground.clone().unwrap();
            state.update(cx, |state, cx| {
                state.set_value("value", Value::Text("Edited".into()), window, cx)
            });
        })
        .unwrap();
    settle(&window, &mut cx);
    assert_eq!(config(&window, &mut cx).text("value"), "Edited");
}

#[gpui::test]
fn every_page_tab_and_playground_scenario_renders(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    for (index, entry) in CATALOG.iter().enumerate() {
        go(&window, &mut cx, entry.id);
        assert!(
            cx.debug_bounds(Box::leak(format!("page-{}", entry.id).into_boxed_str()))
                .is_some(),
            "{} renders its header",
            entry.id
        );
        let Kind::Component { playground } = entry.kind else {
            continue;
        };
        if let Some(playground) = playground {
            for scenario in 0..=playground.scenarios.len() {
                window
                    .update(&mut cx, |g, window, cx| {
                        let state = g.entries[index].playground.clone().unwrap();
                        state.update(cx, |state, cx| state.select_scenario(scenario, window, cx));
                    })
                    .unwrap();
                settle(&window, &mut cx);
            }
        } else {
            assert!(cx.debug_bounds("preview-canvas").is_some(), "{}", entry.id);
        }
        window
            .update(&mut cx, |g, _, cx| g.set_tab(Tab::Specs, cx))
            .unwrap();
        settle(&window, &mut cx);
        assert!(
            cx.debug_bounds(Box::leak(format!("specs-{}", entry.id).into_boxed_str()))
                .is_some(),
            "{} renders its specs",
            entry.id
        );
    }
}

#[gpui::test]
fn shared_theme_changes_start_a_transition_and_other_changes_rerender(cx: &mut TestAppContext) {
    use gpui_kit_material::color::themes::ThemeMode;
    let (window, mut cx) = open(cx);
    go(&window, &mut cx, "button");
    // The card's Light/Dark control changes the Gallery theme.
    let shared = window.update(&mut cx, |g, _, _| g.shared.clone()).unwrap();
    cx.update(|_, cx| {
        shared.update(cx, |s, cx| {
            s.reference_mode = ThemeMode::Dark;
            cx.notify();
        })
    });
    settle(&window, &mut cx);
    window
        .update(&mut cx, |g, _, cx| {
            assert_eq!(g.applied_theme.2, ThemeMode::Dark);
            assert_eq!(g.applied_theme, super::theme_key(g.shared.read(cx)));
        })
        .unwrap();
    let before = window.update(&mut cx, |g, _, _| g.renders.get()).unwrap();
    cx.update(|_, cx| {
        shared.update(cx, |s, cx| {
            s.snackbar = true;
            cx.notify();
        })
    });
    settle(&window, &mut cx);
    window
        .update(&mut cx, |g, _, _| {
            assert!(
                g.renders.get() > before,
                "an overlay change renders the Gallery"
            );
        })
        .unwrap();
}

/// No spec board is drawn wider than its row at the Gallery's reference
/// widths: wide boards wrap their columns instead (no horizontal
/// scrolling, nothing clipped).
#[gpui::test]
fn every_spec_board_fits_its_row(cx: &mut TestAppContext) {
    use super::spec::OVERFLOWS;
    use gpui::{ListOffset, VisualContext as _};
    init(cx);
    for width in [480., 760., 1000., 1280., 1600.] {
        let window = cx.open_window(size(px(width), px(900.)), Gallery::new);
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        design_rem(&window, &mut cx);
        cx.simulate_resize(size(px(width), px(900.)));
        for (index, entry) in CATALOG.iter().enumerate() {
            if entry.spec.is_none() {
                continue;
            }
            window
                .update(&mut cx, |g, _, cx| {
                    g.navigate(index, cx);
                    if matches!(entry.kind, Kind::Component { .. }) {
                        g.set_tab(Tab::Specs, cx);
                    }
                })
                .unwrap();
            settle(&window, &mut cx);
            let rows = window.update(&mut cx, |g, _, _| g.row_count()).unwrap();
            for row in 0..rows {
                window
                    .update(&mut cx, |g, _, cx| {
                        g.list.scroll_to(ListOffset {
                            item_ix: row,
                            offset_in_item: px(0.),
                        });
                        cx.notify();
                    })
                    .unwrap();
                // The first frame measures a new board; the next regroups.
                draw(&window, &mut cx);
                OVERFLOWS.with(|overflows| overflows.borrow_mut().clear());
                draw(&window, &mut cx);
                let overflows = OVERFLOWS.with(|overflows| overflows.take());
                assert!(
                    overflows.is_empty(),
                    "{} at {width}px, row {row}: {overflows:?}",
                    entry.id
                );
            }
        }
        let _ = cx.window_handle();
    }
}

/// The reference widths and the layout each must get.
const WIDTHS: [(f32, ShellLayout); 5] = [
    (1600., ShellLayout::Wide),
    (1280., ShellLayout::Wide),
    (1000., ShellLayout::Medium),
    (760., ShellLayout::Compact),
    (480., ShellLayout::Compact),
];

fn resize(window: &WindowHandle<Gallery>, cx: &mut VisualTestContext, width: f32) {
    cx.simulate_resize(size(px(width), px(900.)));
    settle(window, cx);
}

#[gpui::test]
fn the_shell_arranges_itself_for_the_window_width(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    go(&window, &mut cx, "button");
    for (width, layout) in WIDTHS {
        resize(&window, &mut cx, width);
        assert_eq!(ShellLayout::at(width), layout);
        let canvas = bounds(&mut cx, "preview-canvas");
        let config = bounds(&mut cx, "configuration-card");
        if layout == ShellLayout::Wide {
            assert!(config.left() > canvas.right(), "{width}: side by side");
            assert!(config.top() < canvas.top(), "{width}: tops in one row");
        } else {
            assert!(config.top() >= canvas.bottom(), "{width}: stacked");
            assert!(
                (config.left() - canvas.left()).abs() <= px(1.)
                    && (config.right() - canvas.right()).abs() <= px(1.),
                "{width}: full width"
            );
        }
        assert_eq!(
            cx.debug_bounds("nav-button").is_some(),
            layout != ShellLayout::Compact,
            "{width}: the navigation is in the layout unless it is a drawer"
        );
        assert!(canvas.left() >= px(0.), "{width}");
        assert!(
            canvas.right() <= px(width),
            "{width}: nothing past the edge"
        );
        assert!(
            config.right() <= px(width),
            "{width}: nothing past the edge"
        );
    }
}

/// Every preview at every reference width and at a short window stays inside
/// its canvas: the page never scrolls sideways, the canvas grows with a tall
/// preview, and nothing covers the card's header or is clipped by the card.
#[gpui::test]
fn every_preview_fits_its_canvas_at_every_width(cx: &mut TestAppContext) {
    use super::shell::PREVIEW_DRAWN;
    let (window, mut cx) = open(cx);
    for (width, _) in WIDTHS {
        for height in [780., 1200.] {
            cx.simulate_resize(size(px(width), px(height)));
            settle(&window, &mut cx);
            for entry in CATALOG {
                if !matches!(entry.kind, Kind::Component { .. }) {
                    continue;
                }
                go(&window, &mut cx, entry.id);
                let canvas = bounds(&mut cx, "preview-canvas");
                let drawn = PREVIEW_DRAWN
                    .with(|drawn| drawn.get())
                    .expect("the preview was drawn");
                let label = format!("{} at {width}x{height}", entry.id);
                assert!(
                    drawn.left() >= canvas.left() - px(0.5)
                        && drawn.right() <= canvas.right() + px(0.5),
                    "{label}: {drawn:?} in {canvas:?}"
                );
                assert!(
                    drawn.top() >= canvas.top() - px(0.5)
                        && drawn.bottom() <= canvas.bottom() + px(0.5),
                    "{label}: {drawn:?} in {canvas:?}"
                );
            }
        }
    }
}

/// A contained overlay's screen keeps its height in the growing canvas.
#[gpui::test]
fn overlay_screens_keep_their_height(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    for id in ["dialog", "bottom-sheet", "side-sheet"] {
        go(&window, &mut cx, id);
        let drawn = super::shell::PREVIEW_DRAWN
            .with(|drawn| drawn.get())
            .expect("the preview was drawn");
        assert!(drawn.size.height >= px(320.), "{id}: {drawn:?}");
    }
}

/// The default time picker is taller than a 780px window's pane: its card
/// grows and the configuration card beside it keeps the row's height.
#[gpui::test]
fn a_tall_preview_grows_its_card_at_a_short_window(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    go(&window, &mut cx, "time-picker");
    cx.simulate_resize(size(px(1280.), px(780.)));
    settle(&window, &mut cx);
    let canvas = bounds(&mut cx, "preview-canvas");
    let config = bounds(&mut cx, "configuration-card");
    let tabs = bounds(&mut cx, "card-tabs");
    let drawn = super::shell::PREVIEW_DRAWN
        .with(|drawn| drawn.get())
        .expect("the preview was drawn");
    assert!(drawn.top() >= tabs.bottom(), "{drawn:?} under {tabs:?}");
    assert!(drawn.bottom() <= canvas.bottom(), "{drawn:?} in {canvas:?}");
    assert_eq!(config.bottom(), canvas.bottom() + px(1.));
}

#[gpui::test]
fn the_compact_drawer_opens_navigates_and_dismisses(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    cx.update(|_, cx| cx.set_reduce_motion(true));
    go(&window, &mut cx, "button");
    resize(&window, &mut cx, 480.);
    let open = |window: &WindowHandle<Gallery>, cx: &mut VisualTestContext, value: bool| {
        window
            .update(cx, |g, _, cx| g.set_drawer_open(value, cx))
            .unwrap();
        settle(window, cx);
    };
    let is_open = |window: &WindowHandle<Gallery>, cx: &mut VisualTestContext| {
        window.update(cx, |g, _, _| g.drawer_open).unwrap()
    };
    // The scrim dismisses, and takes the press from the page beneath it.
    open(&window, &mut cx, true);
    click(&window, &mut cx, point(px(470.), px(450.)));
    assert!(!is_open(&window, &mut cx));
    window
        .update(&mut cx, |g, _, _| {
            assert_eq!(g.current, catalog::position("button").unwrap())
        })
        .unwrap();
    // A destination navigates and closes: the first group's heading, then
    // the Overview row (12dp padding, 48dp dense rows).
    open(&window, &mut cx, true);
    click(&window, &mut cx, point(px(100.), px(12. + 48. + 24.)));
    assert!(!is_open(&window, &mut cx));
    window
        .update(&mut cx, |g, _, _| {
            assert_eq!(g.current, catalog::position("overview").unwrap())
        })
        .unwrap();
    // Widening the window leaves no modal drawer behind.
    open(&window, &mut cx, true);
    resize(&window, &mut cx, 1280.);
    assert!(!is_open(&window, &mut cx));
}

#[gpui::test]
fn compact_search_shows_its_results_in_the_drawer(cx: &mut TestAppContext) {
    let (window, mut cx) = open(cx);
    resize(&window, &mut cx, 480.);
    window
        .update(&mut cx, |g, _, cx| g.set_query("sli".into(), cx))
        .unwrap();
    settle(&window, &mut cx);
    window
        .update(&mut cx, |g, _, _| {
            assert!(g.drawer_open);
            assert_eq!(
                g.matches().collect::<Vec<_>>(),
                [catalog::position("slider").unwrap()]
            );
        })
        .unwrap();
}

mod examples {
    use super::super::pages::examples::app::{Body, Device, ExamplesApp};
    use super::super::playground::Value;
    use super::*;
    use gpui::Entity;

    fn set(
        window: &WindowHandle<Gallery>,
        cx: &mut VisualTestContext,
        id: &'static str,
        value: usize,
    ) {
        window
            .update(cx, |g, window, cx| {
                let state = g.entries[g.current].playground.clone().unwrap();
                state.update(cx, |state, cx| {
                    state.set_value(id, Value::Choice(value), window, cx)
                });
            })
            .unwrap();
        settle(window, cx);
    }

    fn app(window: &WindowHandle<Gallery>, cx: &mut VisualTestContext) -> Entity<ExamplesApp> {
        window
            .update(cx, |g, _, cx| {
                g.entries[g.current]
                    .playground
                    .clone()
                    .unwrap()
                    .read(cx)
                    .local_entity::<ExamplesApp>("example-app")
                    .expect("the preview built the app")
            })
            .unwrap()
    }

    fn screen(window: &WindowHandle<Gallery>, cx: &mut VisualTestContext) -> &'static str {
        config(window, cx).option("screen")
    }

    fn top(window: &WindowHandle<Gallery>, cx: &mut VisualTestContext) {
        window
            .update(cx, |g, _, cx| {
                g.scroll_to_top();
                cx.notify();
            })
            .unwrap();
        settle(window, cx);
    }

    /// One device at a time, on every screen, inside the canvas at every
    /// width, with the page holding a single row: nothing scrolls sideways
    /// and the page has no row per screen to scroll past.
    #[gpui::test]
    fn one_device_fits_the_canvas_at_every_width(cx: &mut TestAppContext) {
        let (window, mut cx) = open(cx);
        go(&window, &mut cx, "examples");
        window
            .update(&mut cx, |g, _, _| assert_eq!(g.rows, [Row::Lead]))
            .unwrap();
        for (width, _) in WIDTHS {
            resize(&window, &mut cx, width);
            for device in Device::ALL {
                set(&window, &mut cx, "device", device.index());
                for body in Body::ALL {
                    set(&window, &mut cx, "screen", body.index());
                    top(&window, &mut cx);
                    let canvas = bounds(&mut cx, "preview-canvas");
                    let frame = bounds(&mut cx, "example-device");
                    let label = format!("{device:?} {body:?} at {width}");
                    assert!(
                        frame.left() >= canvas.left() && frame.right() <= canvas.right(),
                        "{label}: {frame:?} in {canvas:?}"
                    );
                    assert!(
                        frame.top() >= canvas.top() && frame.bottom() <= canvas.bottom(),
                        "{label}: {frame:?} in {canvas:?}"
                    );
                    // The whole device scales: its aspect ratio is its own.
                    let (outer_width, outer_height) = device.outer();
                    let ratio = f32::from(frame.size.width) / f32::from(frame.size.height);
                    assert!(
                        (ratio - outer_width / outer_height).abs() < 0.01,
                        "{label}: {frame:?}"
                    );
                }
            }
        }
    }

    /// A preview asks for its arrangement while it renders, after the lead
    /// row, which must not read the playground state for it. The hint still
    /// arrives, both ways, without a refresh, and a settled page keeps a
    /// preview frame to itself.
    #[gpui::test]
    fn a_device_rearranges_the_panes_without_the_lead_row_reading_the_state(
        cx: &mut TestAppContext,
    ) {
        let (window, mut cx) = open(cx);
        go(&window, &mut cx, "examples");
        cx.simulate_resize(size(px(1600.), px(1200.)));
        settle(&window, &mut cx);
        // `draw` refreshes the window, building every view: not this.
        let frames = |cx: &mut VisualTestContext| {
            for _ in 0..3 {
                cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
                    .unwrap();
                cx.run_until_parked();
            }
        };
        let switch = |cx: &mut VisualTestContext, device: Device| {
            window
                .update(cx, |g, window, cx| {
                    let state = g.entries[g.current].playground.clone().unwrap();
                    state.update(cx, |state, cx| {
                        state.set_value("device", Value::Choice(device.index()), window, cx)
                    });
                })
                .unwrap();
            frames(cx);
        };
        let side_by_side = |cx: &mut VisualTestContext| {
            let canvas = bounds(cx, "preview-canvas");
            let config = bounds(cx, "configuration-card");
            config.left() > canvas.right()
        };
        let (preview, lead) = window
            .update(&mut cx, |g, _, _| {
                let views = g.entries[g.current].views.clone().unwrap();
                (views.preview, g.lead.clone())
            })
            .unwrap();
        assert!(side_by_side(&mut cx), "a phone leaves room for the panel");
        switch(&mut cx, Device::Desktop);
        assert!(!side_by_side(&mut cx), "a desktop needs the width");
        let rendered = cx.read(|cx| lead.read(cx).renders.get());
        for _ in 0..3 {
            cx.update(|_, cx| cx.notify(preview.entity_id()));
            frames(&mut cx);
        }
        assert_eq!(cx.read(|cx| lead.read(cx).renders.get()), rendered);
        switch(&mut cx, Device::Phone);
        assert!(side_by_side(&mut cx), "and the panel comes back");
    }

    #[gpui::test]
    fn a_device_keeps_its_real_size_where_there_is_room(cx: &mut TestAppContext) {
        let (window, mut cx) = open(cx);
        go(&window, &mut cx, "examples");
        cx.simulate_resize(size(px(1600.), px(1200.)));
        settle(&window, &mut cx);
        let frame = bounds(&mut cx, "example-device");
        assert_eq!(frame.size, size(px(428.), px(933.)));
    }

    /// A press lands on what is painted, at the scaled size: the phone's
    /// navigation bar switches the app, and the panel follows.
    #[gpui::test]
    fn a_scaled_navigation_bar_takes_real_presses(cx: &mut TestAppContext) {
        let (window, mut cx) = open(cx);
        go(&window, &mut cx, "examples");
        // Narrow enough to scale the phone, tall enough to show all of it.
        cx.simulate_resize(size(px(400.), px(1500.)));
        settle(&window, &mut cx);
        let frame = bounds(&mut cx, "example-device");
        let scale = f32::from(frame.size.width) / Device::Phone.outer().0;
        assert!(scale < 1., "a 400px window scales the phone");
        // The second of four destinations, in the bar above the gesture bar.
        let press = point(
            frame.left() + px((8. + 412. * 1.5 / 4.) * scale),
            frame.bottom() - px((8. + 24. + 40.) * scale),
        );
        click(&window, &mut cx, press);
        assert_eq!(screen(&window, &mut cx), "Library");
    }

    #[gpui::test]
    fn the_panel_and_the_app_move_each_other_and_keep_a_history(cx: &mut TestAppContext) {
        let (window, mut cx) = open(cx);
        go(&window, &mut cx, "examples");
        let app = app(&window, &mut cx);
        let run = |window: &WindowHandle<Gallery>,
                   cx: &mut VisualTestContext,
                   f: fn(&mut ExamplesApp, &mut gpui::Window, &mut gpui::Context<ExamplesApp>)| {
            window
                .update(cx, |_, window, cx| {
                    app.update(cx, |app, cx| f(app, window, cx))
                })
                .unwrap();
            settle(window, cx);
        };
        run(&window, &mut cx, |app, window, cx| {
            app.go(Body::Library, window, cx)
        });
        assert_eq!(screen(&window, &mut cx), "Library");
        // A panel change is a navigation as well.
        set(&window, &mut cx, "screen", Body::Messaging.index());
        run(&window, &mut cx, |app, window, cx| app.go_back(window, cx));
        assert_eq!(screen(&window, &mut cx), "Library");
        run(&window, &mut cx, |app, window, cx| app.go_back(window, cx));
        assert_eq!(screen(&window, &mut cx), "Home");
        run(&window, &mut cx, |app, window, cx| {
            app.go_forward(window, cx)
        });
        assert_eq!(screen(&window, &mut cx), "Library");
        run(&window, &mut cx, |app, window, cx| {
            app.go_forward(window, cx)
        });
        assert_eq!(screen(&window, &mut cx), "Messaging");
        // With nothing to go back to, Back lands on Home.
        run(&window, &mut cx, |app, window, cx| app.go_back(window, cx));
        run(&window, &mut cx, |app, window, cx| app.go_back(window, cx));
        run(&window, &mut cx, |app, window, cx| app.go_back(window, cx));
        assert_eq!(screen(&window, &mut cx), "Home");
    }

    #[gpui::test]
    fn messages_send_to_their_conversation_and_clear(cx: &mut TestAppContext) {
        let (window, mut cx) = open(cx);
        go(&window, &mut cx, "examples");
        let app = app(&window, &mut cx);
        for message in ["One", "  ", "Two"] {
            window
                .update(&mut cx, |_, window, cx| {
                    app.update(cx, |app, cx| {
                        app.message
                            .update(cx, |input, cx| input.set_value(message, window, cx));
                        app.send(window, cx);
                    })
                })
                .unwrap();
        }
        window
            .update(&mut cx, |_, _, cx| {
                app.update(cx, |app, cx| {
                    // Blank messages are not sent; a suggestion sends itself.
                    assert_eq!(app.sent[0], ["One", "Two"]);
                    assert_eq!(app.message.read(cx).value(), "");
                    app.conversation = 1;
                    app.send_text("Great!", cx);
                    assert_eq!(app.sent[1], ["Great!"]);
                    assert_eq!(app.sent[0].len(), 2);
                })
            })
            .unwrap();
    }

    #[gpui::test]
    fn the_player_plays_pauses_and_changes_track(cx: &mut TestAppContext) {
        let (window, mut cx) = open(cx);
        go(&window, &mut cx, "examples");
        let app = app(&window, &mut cx);
        window
            .update(&mut cx, |_, _, cx| {
                app.update(cx, |app, cx| {
                    assert!(!app.playing);
                    app.play(0, cx);
                    assert!(app.playing);
                    app.play(0, cx);
                    assert!(!app.playing);
                    // Another track starts from its beginning.
                    app.play(1, cx);
                    assert!(app.playing);
                    assert_eq!((app.track, app.position), (1, 0.));
                    app.next_track(cx);
                    app.next_track(cx);
                    assert_eq!(app.track, 0);
                    app.play(0, cx);
                })
            })
            .unwrap();
    }

    #[gpui::test]
    fn every_screen_draws_on_every_device_with_a_snackbar_up(cx: &mut TestAppContext) {
        let (window, mut cx) = open(cx);
        go(&window, &mut cx, "examples");
        let app = app(&window, &mut cx);
        window
            .update(&mut cx, |_, _, cx| {
                app.update(cx, |app, cx| app.say("Saved", cx))
            })
            .unwrap();
        for device in Device::ALL {
            set(&window, &mut cx, "device", device.index());
            for body in Body::ALL {
                set(&window, &mut cx, "screen", body.index());
                top(&window, &mut cx);
                assert!(cx.debug_bounds("example-snackbar").is_some(), "{body:?}");
            }
        }
    }
}
