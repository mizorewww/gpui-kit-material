//! Gallery shell behavior across pages and virtual rows.
use super::Gallery;
use super::page::GalleryPage;
use super::pages::{self, PAGES};
use gpui::{AppContext, Entity, TestAppContext, WindowHandle, px, size};

fn init(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit_material::init(cx);
    });
}
fn draw(window: &WindowHandle<Gallery>, cx: &mut TestAppContext) {
    cx.update_window((*window).into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.run_until_parked();
}
fn frame(window: &WindowHandle<Gallery>, cx: &mut TestAppContext) {
    cx.update_window((*window).into(), |_, window, cx| {
        window.simulate_next_frame(cx);
    })
    .unwrap();
    cx.run_until_parked();
    draw(window, cx);
}
fn page_index(id: &str) -> usize {
    PAGES.iter().position(|page| page.info.id == id).unwrap()
}
fn page<P: GalleryPage>(gallery: &Gallery) -> Entity<P> {
    let index = PAGES
        .iter()
        .position(|page| page.info.id == P::INFO.id)
        .unwrap();
    gallery.pages[index].any_entity().downcast::<P>().unwrap()
}
/// The list row of body row `row` of page `id`.
fn body_row(gallery: &Gallery, id: &str, row: usize) -> usize {
    gallery.headers[page_index(id)] + 1 + row
}
fn scroll_to(window: &WindowHandle<Gallery>, cx: &mut TestAppContext, row: usize) {
    window
        .update(cx, |gallery, _, cx| {
            gallery.list.scroll_to(gpui::ListOffset {
                item_ix: row,
                offset_in_item: px(0.),
            });
            cx.notify();
        })
        .unwrap();
    draw(window, cx);
}

#[gpui::test]
fn every_page_row_renders_inside_its_page(cx: &mut TestAppContext) {
    init(cx);
    let window = cx.open_window(size(px(1280.), px(900.)), Gallery::new);
    draw(&window, cx);
    let total = window.update(cx, |g, _, _| g.row_count()).unwrap();
    assert_eq!(
        total,
        window
            .update(cx, |g, _, _| g.counts.iter().map(|c| c + 1).sum::<usize>())
            .unwrap()
    );
    for row in 0..total {
        scroll_to(&window, cx, row);
        let bounds = window
            .update(cx, |g, _, _| g.list.bounds_for_item(row))
            .unwrap();
        assert!(
            bounds.is_some_and(|b| b.size.height > px(0.)),
            "row {row} renders"
        );
    }
    for (index, entry) in PAGES.iter().enumerate() {
        let header = window.update(cx, |g, _, _| g.headers[index]).unwrap();
        scroll_to(&window, cx, header);
        let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
        let selector: &'static str = format!("page-{}", entry.info.id).leak();
        assert!(
            visual.debug_bounds(selector).is_some(),
            "{} renders its header first",
            entry.info.id
        );
    }
}

#[gpui::test]
fn anchored_rows_render_the_content_their_state_drives(cx: &mut TestAppContext) {
    use pages::buttons::{Row, row_of};
    init(cx);
    let window = cx.open_window(size(px(960.), px(600.)), Gallery::new);
    draw(&window, cx);
    for (page, row, selector) in [
        (
            "buttons",
            row_of(Row::EffectSamples),
            "button-effect-samples",
        ),
        (
            "buttons",
            row_of(Row::ToggleShape(0)),
            "button-toggle-shapes",
        ),
        (
            "buttons",
            row_of(Row::LoadingSize(0)),
            "button-loading-sizes",
        ),
        ("buttons", row_of(Row::LoadingCount), "loading-click-count"),
        ("color", 4, "color-swatches"),
    ] {
        let global = window.update(cx, |g, _, _| body_row(g, page, row)).unwrap();
        scroll_to(&window, cx, global);
        let row_bounds = window
            .update(cx, |g, _, _| g.list.bounds_for_item(global))
            .unwrap()
            .expect("anchored row is mounted");
        let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
        let content = visual
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is not rendered"));
        assert!(
            row_bounds.contains(&content.origin),
            "{selector} must render in row {global}"
        );
    }
}

#[gpui::test]
fn button_sync_runs_after_list_and_does_not_treat_remeasurement_as_unmount(
    cx: &mut TestAppContext,
) {
    use pages::buttons::{Page, Row, row_of};
    init(cx);
    let window = cx.open_window(size(px(1100.), px(500.)), Gallery::new);
    let buttons = window.update(cx, |g, _, _| page::<Page>(g)).unwrap();
    cx.update(|cx| buttons.update(cx, |page, _| page.sync_rows.insert(999)));
    draw(&window, cx);
    let effect = row_of(Row::EffectSamples);
    cx.read(|cx| {
        let rows = &buttons.read(cx).sync_rows;
        assert!(!rows.contains(&999), "zero-size canvas prepaint must run");
        assert!(rows.contains(&effect));
        assert!(rows.contains(&row_of(Row::LoadingSize(0))));
    });
    let global = window
        .update(cx, |g, _, _| body_row(g, "buttons", effect))
        .unwrap();
    scroll_to(&window, cx, global);
    assert!(cx.read(|cx| !buttons.read(cx).sync_rows.contains(&effect)));
    cx.update(|cx| {
        buttons.update(cx, |page, cx| {
            page.active = true;
            cx.notify();
        })
    });
    window.update(cx, |g, _, _| g.list.remeasure()).unwrap();
    draw(&window, cx);
    assert!(
        cx.read(|cx| !buttons.read(cx).sync_rows.contains(&effect)),
        "remeasure must not inject rest targets into the visible row"
    );
}

#[gpui::test]
fn choice_tiles_are_a_radio_group_driven_by_arrow_keys(cx: &mut TestAppContext) {
    use pages::radio_button::{CHOICE_ROW, Page};
    init(cx);
    let window = cx.open_window(size(px(960.), px(600.)), Gallery::new);
    draw(&window, cx);
    let row = window
        .update(cx, |g, _, _| body_row(g, "radio-button", CHOICE_ROW))
        .unwrap();
    scroll_to(&window, cx, row);
    window
        .update(cx, |gallery, window, cx| {
            gallery.focus_row(row, false, window, cx);
            assert!(gallery.row_focus[row].contains_focused(window, cx));
        })
        .unwrap();
    let radio = window.update(cx, |g, _, _| page::<Page>(g)).unwrap();
    let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
    // Down selects the next choice; the unavailable third is skipped and the
    // arrows wrap; End and Home land on the available ends.
    for (key, engine) in [("down", 1), ("down", 0), ("up", 1), ("home", 0), ("end", 1)] {
        visual.simulate_keystrokes(key);
        draw(&window, cx);
        assert_eq!(cx.read(|cx| radio.read(cx).engine), engine, "after {key}");
    }
}

#[gpui::test]
fn keyboard_mounts_offscreen_row_and_returns_without_extra_tab_stop(cx: &mut TestAppContext) {
    init(cx);
    let window = cx.open_window(size(px(960.), px(350.)), Gallery::new);
    cx.run_until_parked();
    // A roving group contributes exactly one tab stop. The next row must
    // mount even after the page is scrolled far away.
    let (first, second) = window
        .update(cx, |g, _, _| {
            (body_row(g, "color", 0), body_row(g, "color", 1))
        })
        .unwrap();
    scroll_to(&window, cx, first);
    let far = window.update(cx, |_, _, _| page_index("sliders")).unwrap();
    window
        .update(cx, |gallery, window, cx| {
            gallery.focus_row(first, false, window, cx);
            assert!(gallery.row_focus[first].contains_focused(window, cx));
            gallery.scroll_to_section(far);
            cx.notify();
        })
        .unwrap();
    draw(&window, cx);
    window
        .update(cx, |gallery, window, cx| {
            assert!(
                gallery.list.bounds_for_item(second).is_none(),
                "next logical row must be unmounted"
            );
            gallery.advance_gallery_focus(false, window, cx);
            assert_eq!(gallery.pending_focus, Some((second, false)));
        })
        .unwrap();
    draw(&window, cx);
    frame(&window, cx);
    window
        .update(cx, |gallery, window, cx| {
            assert!(gallery.row_focus[second].contains_focused(window, cx));
            assert!(
                !gallery.row_focus[second].is_focused(window),
                "no container-only tab stop"
            );
            gallery.advance_gallery_focus(true, window, cx);
        })
        .unwrap();
    draw(&window, cx);
    frame(&window, cx);
    window
        .update(cx, |gallery, window, cx| {
            assert!(gallery.row_focus[first].contains_focused(window, cx));
        })
        .unwrap();
}

#[gpui::test]
fn virtual_navigation_preserves_page_and_input_entities(cx: &mut TestAppContext) {
    use pages::{buttons, lists, search, sliders};
    init(cx);
    let window = cx.open_window(size(px(700.), px(250.)), Gallery::new);
    cx.run_until_parked();
    let (b, l, s, sl) = window
        .update(cx, |g, _, _| {
            (
                page::<buttons::Page>(g),
                page::<lists::Page>(g),
                page::<search::Page>(g),
                page::<sliders::Page>(g),
            )
        })
        .unwrap();
    let original = cx.update(|cx| {
        b.update(cx, |page, _| {
            page.selections[0] = true;
            page.loading_clicks = 9;
        });
        l.update(cx, |page, _| page.details = true);
        (
            s.read(cx).search.entity_id(),
            sl.read(cx).slider.entity_id(),
        )
    });
    for id in ["sliders", "color", "buttons", "lists", "search"] {
        window
            .update(cx, |g, _, cx| {
                g.scroll_to_section(page_index(id));
                cx.notify();
            })
            .unwrap();
        draw(&window, cx);
    }
    cx.read(|cx| {
        assert_eq!(s.read(cx).search.entity_id(), original.0);
        assert_eq!(sl.read(cx).slider.entity_id(), original.1);
        assert!(b.read(cx).selections[0]);
        assert_eq!(b.read(cx).loading_clicks, 9);
        assert!(l.read(cx).details);
    });
}

#[gpui::test]
fn page_panels_center_at_the_page_maximum_width(cx: &mut TestAppContext) {
    use super::spec::PAGE_MAX_WIDTH;
    init(cx);
    for width in [700., 960., 1600.] {
        let window = cx.open_window(size(px(width), px(600.)), Gallery::new);
        window
            .update(cx, |_, window, _| window.set_rem_size(px(14.)))
            .unwrap();
        draw(&window, cx);
        let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
        let bounds = visual.debug_bounds("gallery-first-row").unwrap();
        let sidebar = if width < 960. { 0. } else { 176. };
        let content = width - sidebar;
        let row = content.min(PAGE_MAX_WIDTH);
        assert!((f32::from(bounds.size.width) - row).abs() < 0.01, "{width}");
        assert!(
            (f32::from(bounds.origin.x) - sidebar - (content - row) / 2.).abs() < 0.01,
            "width {width}: {bounds:?}"
        );
    }
}

#[gpui::test]
fn width_dependent_rows_splice_and_shift_later_pages(cx: &mut TestAppContext) {
    init(cx);
    let window = cx.open_window(size(px(1600.), px(600.)), Gallery::new);
    draw(&window, cx);
    let color = page_index("color");
    let (wide, wide_next) = window
        .update(cx, |g, _, _| (g.counts[color], g.headers[color + 1]))
        .unwrap();
    window
        .update(cx, |_, window, _| window.set_rem_size(px(28.)))
        .unwrap();
    draw(&window, cx);
    let (narrow, narrow_next, focus, total) = window
        .update(cx, |g, _, _| {
            (
                g.counts[color],
                g.headers[color + 1],
                g.row_focus.len(),
                g.list.item_count(),
            )
        })
        .unwrap();
    assert!(narrow > wide, "fewer swatch columns need more rows");
    assert_eq!(narrow_next - wide_next, narrow - wide);
    assert_eq!(focus, total);
    let rows = window
        .update(cx, |g, _, _| g.page_rows[page_index("buttons")].global(0))
        .unwrap();
    assert_eq!(
        rows,
        window
            .update(cx, |g, _, _| g.headers[page_index("buttons")] + 1)
            .unwrap()
    );
}

#[gpui::test]
fn circular_feedback_keeps_phase_and_observes_reduced_motion_offscreen(cx: &mut TestAppContext) {
    use gpui_kit_material::components::circular_indeterminate_frame;
    use pages::loading_progress::{CIRCULAR_ROW, Page};
    use std::time::Duration;
    init(cx);
    let window = cx.open_window(size(px(700.), px(250.)), Gallery::new);
    draw(&window, cx);
    let loading = window.update(cx, |g, _, _| page::<Page>(g)).unwrap();
    let row = window
        .update(cx, |g, _, _| body_row(g, "loading-progress", CIRCULAR_ROW))
        .unwrap();
    // An unused playback must start on the circular row's first render.
    cx.executor().advance_clock(Duration::from_secs(2));
    scroll_to(&window, cx, row);
    cx.executor().advance_clock(Duration::from_millis(150));
    let far = page_index("sliders");
    window
        .update(cx, |gallery, _, cx| {
            assert!(gallery.list.bounds_for_item(row).is_some());
            assert_eq!(
                loading.read(cx).circular.sample(cx),
                circular_indeterminate_frame(150.),
                "the mounted row must use the page-owned playback"
            );
            gallery.scroll_to_section(far);
            cx.notify();
        })
        .unwrap();
    draw(&window, cx);
    cx.executor().advance_clock(Duration::from_millis(350));
    window
        .update(cx, |gallery, _, cx| {
            assert!(gallery.list.bounds_for_item(row).is_none());
            cx.set_reduce_motion(true);
        })
        .unwrap();
    draw(&window, cx);
    window
        .update(cx, |_, _, cx| cx.set_reduce_motion(false))
        .unwrap();
    draw(&window, cx);
    cx.executor().advance_clock(Duration::from_secs(1));
    scroll_to(&window, cx, row);
    window
        .update(cx, |gallery, _, cx| {
            assert!(gallery.list.bounds_for_item(row).is_some());
            assert_eq!(
                loading.read(cx).circular.sample(cx),
                circular_indeterminate_frame(500.),
                "remounting must preserve the reduced-motion stop"
            );
        })
        .unwrap();
}

#[gpui::test]
fn motion_previews_keep_phase_across_virtual_unmount_and_offscreen_retarget(
    cx: &mut TestAppContext,
) {
    use gpui_kit_material::motion::MotionScheme;
    use pages::motion::{PREVIEW_ROW, Page, TOKENS};
    use std::time::Duration;
    init(cx);
    let window = cx.open_window(size(px(700.), px(250.)), Gallery::new);
    draw(&window, cx);
    let motion = window.update(cx, |g, _, _| page::<Page>(g)).unwrap();
    let first = window
        .update(cx, |g, _, _| body_row(g, "motion", PREVIEW_ROW))
        .unwrap();
    let far = page_index("sliders");
    window
        .update(cx, |g, _, cx| {
            g.scroll_to_section(far);
            cx.notify();
        })
        .unwrap();
    draw(&window, cx);
    window
        .update(cx, |g, _, cx| {
            for row in first..first + TOKENS.len() {
                assert!(g.list.bounds_for_item(row).is_none());
            }
            assert_eq!(motion.read(cx).values, [0.; 6]);
            motion.update(cx, |page, cx| {
                page.target = true;
                cx.notify();
            });
        })
        .unwrap();
    draw(&window, cx);
    cx.executor().advance_clock(Duration::from_millis(100));
    draw(&window, cx);
    let ongoing = cx.read(|cx| {
        let values = motion.read(cx).values;
        for (index, (_, token, _)) in TOKENS.into_iter().enumerate() {
            let curve = MotionScheme::Expressive.resolve(token);
            let expected = curve.ease()(0.1 / curve.duration().as_secs_f32());
            assert!(
                (values[index] - expected).abs() < 0.002,
                "channel {index}: {} != {expected}",
                values[index]
            );
        }
        values
    });
    scroll_to(&window, cx, first);
    window
        .update(cx, |g, _, cx| {
            assert!(g.list.bounds_for_item(first).is_some(), "preview remounts");
            assert_eq!(motion.read(cx).values, ongoing, "remount must not restart");
            g.scroll_to_section(far);
            motion.update(cx, |page, cx| {
                page.target = false;
                cx.notify();
            });
        })
        .unwrap();
    draw(&window, cx);
    assert_eq!(
        cx.read(|cx| motion.read(cx).values),
        ongoing,
        "reversal starts at the sampled value"
    );
    cx.executor().advance_clock(Duration::from_secs(5));
    draw(&window, cx);
    assert_eq!(cx.read(|cx| motion.read(cx).values), [0.; 6]);
}

#[gpui::test]
fn shared_theme_changes_start_a_transition_and_other_changes_rerender(cx: &mut TestAppContext) {
    use gpui_kit_material::color::themes::ThemeMode;
    init(cx);
    let window = cx.open_window(size(px(960.), px(600.)), Gallery::new);
    draw(&window, cx);
    let shared = window.update(cx, |g, _, _| g.shared.clone()).unwrap();
    cx.update(|cx| {
        shared.update(cx, |s, cx| {
            s.reference_mode = ThemeMode::Dark;
            cx.notify();
        })
    });
    draw(&window, cx);
    window
        .update(cx, |g, _, cx| {
            assert_eq!(g.applied_theme.2, ThemeMode::Dark);
            assert_eq!(g.applied_theme, super::theme_key(g.shared.read(cx)));
        })
        .unwrap();
    let before = window.update(cx, |g, _, _| g.renders.get()).unwrap();
    cx.update(|cx| {
        shared.update(cx, |s, cx| {
            s.snackbar = true;
            cx.notify();
        })
    });
    draw(&window, cx);
    window
        .update(cx, |g, _, cx| {
            assert!(
                g.renders.get() > before,
                "an overlay change renders the Gallery"
            );
            assert_eq!(g.applied_theme, super::theme_key(g.shared.read(cx)));
        })
        .unwrap();
}
