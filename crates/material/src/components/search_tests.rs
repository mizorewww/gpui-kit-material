use super::*;
use core::prelude::v1::test;
use std::cell::Cell;

struct BarHarness {
    state: Entity<InputState>,
    expanded: bool,
    layout: SearchViewLayout,
    style: SearchViewStyle,
    disabled: bool,
    picked: Rc<Cell<Option<usize>>>,
}

impl Render for BarHarness {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let picked = self.picked.clone();
        div().size(px(420.)).p(px(10.)).child(
            SearchView::new(
                "view",
                SearchField::new(&self.state)
                    .leading(div().size(px(24.)))
                    .disabled(self.disabled),
            )
            .layout(self.layout)
            .view_style(self.style)
            .expanded(self.expanded)
            .suggestions((0..3).map(|ix| {
                let picked = picked.clone();
                SearchSuggestion::new("Label")
                    .supporting_text("Supporting")
                    .on_click(move |_, _, _| picked.set(Some(ix)))
            })),
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
    layout: SearchViewLayout,
) -> (Entity<BarHarness>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_reduce_motion(true);
    });
    cx.add_window_view(move |window, cx| BarHarness {
        state: cx.new(|cx| InputState::new(window, cx).placeholder("Search")),
        expanded: false,
        layout,
        style: SearchViewStyle::Contained,
        disabled: false,
        picked: Rc::default(),
    })
}

#[gpui::test]
fn the_bar_paints_a_surface_container_high_pill(cx: &mut TestAppContext) {
    let (_, cx) = setup(cx, SearchViewLayout::Docked);
    draw(cx);
    let bar = cx.debug_bounds("search-bar").unwrap();
    assert_eq!(bar.size, size(px(360.), px(56.)));
    cx.update(|window, _| {
        let scale = window.scale_factor();
        let expected = gpui::solid_background(color(ColorRole::SURFACE_CONTAINER_HIGH));
        let quads = window.painted_quads();
        let pill = quads
            .iter()
            .find(|q| {
                q.bounds.size.width.0 == 360. * scale
                    && q.bounds.size.height.0 == 56. * scale
                    && q.background == expected
            })
            .expect("the bar paints its container");
        assert_eq!(pill.background, expected);
        assert_ne!(
            pill.background,
            gpui::solid_background(color(ColorRole::SURFACE_CONTAINER_HIGHEST))
        );
        assert_eq!(pill.corner_radii.top_left.0, 28. * scale, "a full pill");
    });
}

#[gpui::test]
fn the_bar_edits_through_kit_input(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, SearchViewLayout::Docked);
    draw(cx);
    let bar = cx.debug_bounds("search-bar").unwrap();
    cx.simulate_click(bar.center(), Modifiers::default());
    cx.simulate_input("home");
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).state.read(cx).value().as_ref(), "home"));
}

#[gpui::test]
fn the_docked_view_reveals_its_rows_and_activates_suggestions(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, SearchViewLayout::Docked);
    draw(cx);
    assert!(cx.debug_bounds("search-results").is_none());
    view.update(cx, |v, cx| {
        v.expanded = true;
        cx.notify();
    });
    draw(cx);
    let bar = cx.debug_bounds("search-bar").unwrap();
    let results = cx.debug_bounds("search-results").unwrap();
    assert_eq!(results.top(), bar.bottom() + px(2.));
    assert_eq!(results.size.height, px(3. * 64.));
    cx.simulate_click(
        point(results.center().x, results.top() + px(96.)),
        Modifiers::default(),
    );
    cx.update(|_, cx| assert_eq!(view.read(cx).picked.get(), Some(1)));
    view.update(cx, |v, cx| {
        v.expanded = false;
        cx.notify();
    });
    draw(cx);
    draw(cx);
    assert!(
        cx.debug_bounds("search-results").is_none(),
        "a settled collapse removes the rows"
    );
}

#[gpui::test]
fn a_collapsing_view_ignores_clicks_on_its_fading_rows(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, SearchViewLayout::Docked);
    cx.update(|_, cx| cx.set_reduce_motion(false));
    view.update(cx, |v, cx| {
        v.expanded = true;
        cx.notify();
    });
    draw(cx);
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(2));
    draw(cx);
    let results = cx.debug_bounds("search-results").unwrap();
    let row = point(results.center().x, results.top() + px(32.));
    view.update(cx, |v, cx| {
        v.expanded = false;
        cx.notify();
    });
    draw(cx);
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(20));
    draw(cx);
    assert!(cx.debug_bounds("search-results").is_some(), "still fading");
    cx.simulate_click(row, Modifiers::default());
    cx.update(|_, cx| assert_eq!(view.read(cx).picked.get(), None));
}

#[gpui::test]
fn the_full_screen_view_fills_its_area(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, SearchViewLayout::FullScreen);
    view.update(cx, |v, cx| {
        v.expanded = true;
        cx.notify();
    });
    draw(cx);
    let bar = cx.debug_bounds("search-bar").unwrap();
    let results = cx.debug_bounds("search-results").unwrap();
    assert_eq!(bar.size.width, px(400. - 24.), "the bar insets 12dp");
    assert_eq!(results.top(), bar.bottom());
    assert_eq!(results.bottom(), px(410.));
}

#[test]
fn results_grow_with_the_expansion_and_cap_at_max_visible() {
    assert_eq!(results_height(3, 5, 1.), 192.);
    assert_eq!(results_height(8, 5, 1.), 320.);
    assert_eq!(results_height(3, 5, 0.5), 96.);
    assert_eq!(results_height(3, 5, 1.1), 192., "overshoot stays inside");
    assert_eq!(results_height(3, 5, 0.), 0.);
}

#[gpui::test]
fn a_divided_view_joins_bar_and_results_with_a_divider(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, SearchViewLayout::Docked);
    view.update(cx, |v, cx| {
        v.style = SearchViewStyle::Divided;
        v.expanded = true;
        cx.notify();
    });
    draw(cx);
    let bar = cx.debug_bounds("search-bar").unwrap();
    let divider = cx.debug_bounds("search-divider").unwrap();
    let results = cx.debug_bounds("search-results").unwrap();
    assert_eq!(divider.top(), bar.bottom());
    assert_eq!(divider.size.height, px(1.));
    assert_eq!(results.top(), divider.bottom(), "no gap");
    // The contained view has no divider.
    view.update(cx, |v, cx| {
        v.style = SearchViewStyle::Contained;
        cx.notify();
    });
    draw(cx);
    assert!(cx.debug_bounds("search-divider").is_none());
}

#[gpui::test]
fn a_disabled_bar_takes_no_input(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, SearchViewLayout::Docked);
    view.update(cx, |v, cx| {
        v.disabled = true;
        cx.notify();
    });
    draw(cx);
    let bar = cx.debug_bounds("search-bar").unwrap();
    cx.simulate_click(bar.center(), Modifiers::default());
    cx.simulate_input("home");
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).state.read(cx).value().as_ref(), ""));
}
