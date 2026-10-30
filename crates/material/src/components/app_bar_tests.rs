use super::*;
use core::prelude::v1::test;
use gpui::{TestAppContext, VisualTestContext};
use std::time::Duration;

#[test]
fn heights_follow_the_expressive_bar_metrics() {
    use TopAppBarVariant::*;
    for variant in [Small, CenterAligned, Search] {
        assert_eq!(variant.expanded_height(false), 64.);
        assert_eq!(variant.expanded_height(true), 64.);
        assert_eq!(
            variant.collapse_fraction(false, 500.),
            0.,
            "never collapses"
        );
    }
    assert_eq!(MediumFlexible.expanded_height(false), 112.);
    assert_eq!(MediumFlexible.expanded_height(true), 136.);
    assert_eq!(LargeFlexible.expanded_height(false), 120.);
    assert_eq!(LargeFlexible.expanded_height(true), 152.);
}

#[test]
fn flexible_bars_collapse_with_the_scroll_offset() {
    let medium = TopAppBarVariant::MediumFlexible;
    assert_eq!(medium.collapse_fraction(false, 0.), 0.);
    assert_eq!(medium.collapse_fraction(false, 24.), 0.5);
    assert_eq!(medium.collapse_fraction(false, 48.), 1.);
    assert_eq!(medium.collapse_fraction(false, 400.), 1.);
    assert_eq!(medium.height(false, 24.), 88.);
    assert_eq!(medium.height(false, 400.), TOP_APP_BAR_HEIGHT);
    let large = TopAppBarVariant::LargeFlexible;
    assert_eq!(large.height(true, 44.), 108.);
}

#[test]
fn the_collapsed_title_arrives_as_the_large_title_leaves() {
    assert_eq!(collapsed_title_alpha(0.), 0.);
    assert_eq!(collapsed_title_alpha(0.5), 0.);
    assert_eq!(collapsed_title_alpha(1.), 1.);
    let mid = collapsed_title_alpha(0.75);
    assert!(mid > 0. && mid < 1.);
    // Monotonic.
    let samples: Vec<f32> = (0..=20)
        .map(|i| collapsed_title_alpha(i as f32 / 20.))
        .collect();
    assert!(samples.windows(2).all(|pair| pair[0] <= pair[1]));
}

#[test]
fn title_roles_match_the_kit() {
    assert_eq!(
        TopAppBarVariant::Small.title_roles(),
        (TypeRole::TitleLarge, TypeRole::LabelMedium)
    );
    assert_eq!(
        TopAppBarVariant::MediumFlexible.title_roles(),
        (TypeRole::HeadlineMedium, TypeRole::LabelLarge)
    );
    assert_eq!(
        TopAppBarVariant::LargeFlexible.title_roles(),
        (TypeRole::DisplaySmall, TypeRole::TitleMedium)
    );
}

#[test]
fn scrolled_under_paints_surface_container() {
    let flat = bar_paint(0.);
    assert_eq!(flat.container, color(ColorRole::SURFACE));
    assert_eq!(flat.search, color(ColorRole::SURFACE_CONTAINER));
    let scrolled = bar_paint(1.);
    assert_eq!(scrolled.container, color(ColorRole::SURFACE_CONTAINER));
    assert_eq!(scrolled.search, color(ColorRole::SURFACE_CONTAINER_HIGHEST));
}

#[test]
fn a_custom_flat_container_still_fades_to_surface_container() {
    let flat = bar_paint_on(ColorRole::SURFACE_CONTAINER_LOWEST, 0.);
    assert_eq!(flat.container, color(ColorRole::SURFACE_CONTAINER_LOWEST));
    let scrolled = bar_paint_on(ColorRole::SURFACE_CONTAINER_LOWEST, 1.);
    assert_eq!(scrolled.container, color(ColorRole::SURFACE_CONTAINER));
    assert_eq!(scrolled.search, color(ColorRole::SURFACE_CONTAINER_HIGHEST));
}

#[gpui::test]
fn the_container_role_paints_the_flat_bar(cx: &mut TestAppContext) {
    struct OnPane;
    impl Render for OnPane {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().w(px(412.)).child(
                TopAppBar::new("pane-bar", TopAppBarVariant::Small)
                    .title("Title")
                    .container(ColorRole::SURFACE_CONTAINER_LOWEST),
            )
        }
    }
    cx.update(gpui_base::init);
    let (_, cx) = cx.add_window_view(|_, _| OnPane);
    settle(cx);
    let paint = PAINTED.with(|painted| {
        painted
            .borrow()
            .iter()
            .find(|(key, _)| key.contains("pane-bar"))
            .expect("the bar painted")
            .1
            .0
    });
    assert_eq!(paint.container, color(ColorRole::SURFACE_CONTAINER_LOWEST));
}

struct Harness {
    variant: TopAppBarVariant,
    offset: f32,
    subtitle: bool,
    bottom_visible: bool,
}

impl Render for Harness {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let mut bar = TopAppBar::new("bar", self.variant)
            .title("Title")
            .navigation_icon(div().size(px(48.)))
            .action(div().size(px(48.)))
            .scroll_offset(self.offset);
        if self.subtitle {
            bar = bar.subtitle("Subtitle");
        }
        div().w(px(412.)).flex().flex_col().child(bar).child(
            BottomAppBar::new("bottom", "Actions")
                .action(div().size(px(48.)))
                .visible(self.bottom_visible),
        )
    }
}

fn harness(
    cx: &mut TestAppContext,
    variant: TopAppBarVariant,
) -> (Entity<Harness>, &mut VisualTestContext) {
    cx.update(gpui_base::init);
    cx.add_window_view(move |_, _| Harness {
        variant,
        offset: 0.,
        subtitle: false,
        bottom_visible: true,
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

fn painted() -> (BarPaint, f32) {
    PAINTED.with(|painted| {
        *painted
            .borrow()
            .iter()
            .find(|(key, _)| key.contains("bar"))
            .expect("the bar painted")
            .1
    })
}

fn bar_height(cx: &mut VisualTestContext) -> Pixels {
    cx.debug_bounds("top-app-bar")
        .expect("bar drawn")
        .size
        .height
}

#[gpui::test]
fn scrolling_under_fades_the_container(cx: &mut TestAppContext) {
    let (view, cx) = harness(cx, TopAppBarVariant::Small);
    settle(cx);
    assert_eq!(bar_height(cx), px(64.));
    assert_eq!(painted().0.container, color(ColorRole::SURFACE));
    view.update(cx, |view, cx| {
        view.offset = 10.;
        cx.notify();
    });
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(30));
    draw(cx);
    let mid = painted().0.container;
    assert_ne!(mid, color(ColorRole::SURFACE), "the fade has begun");
    assert_ne!(mid, color(ColorRole::SURFACE_CONTAINER), "mid-fade");
    settle(cx);
    assert_eq!(painted().0.container, color(ColorRole::SURFACE_CONTAINER));
}

#[gpui::test]
fn reduced_motion_keeps_the_scrolled_fade(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let (view, cx) = harness(cx, TopAppBarVariant::Small);
    settle(cx);
    view.update(cx, |view, cx| {
        view.offset = 10.;
        cx.notify();
    });
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(10));
    draw(cx);
    // Effects tokens keep their fade under reduced motion; it still settles.
    settle(cx);
    assert_eq!(painted().0.container, color(ColorRole::SURFACE_CONTAINER));
}

#[gpui::test]
fn the_large_bar_collapses_to_the_small_height(cx: &mut TestAppContext) {
    let (view, cx) = harness(cx, TopAppBarVariant::LargeFlexible);
    settle(cx);
    assert_eq!(bar_height(cx), px(120.));
    view.update(cx, |view, cx| {
        view.offset = 28.;
        cx.notify();
    });
    draw(cx);
    assert_eq!(
        bar_height(cx),
        px(92.),
        "follows the offset without a clock"
    );
    assert_eq!(painted().1, 0.5);
    view.update(cx, |view, cx| {
        view.offset = 1000.;
        view.subtitle = true;
        cx.notify();
    });
    draw(cx);
    assert_eq!(bar_height(cx), px(64.));
    view.update(cx, |view, cx| {
        view.offset = 0.;
        cx.notify();
    });
    draw(cx);
    assert_eq!(bar_height(cx), px(152.), "subtitle adds its line");
}

#[gpui::test]
fn the_bottom_bar_hides_on_the_spatial_spring(cx: &mut TestAppContext) {
    let (view, cx) = harness(cx, TopAppBarVariant::Small);
    settle(cx);
    let height =
        |cx: &mut VisualTestContext| cx.debug_bounds("bottom-app-bar").unwrap().size.height;
    assert_eq!(height(cx), px(80.));
    view.update(cx, |view, cx| {
        view.bottom_visible = false;
        cx.notify();
    });
    draw(cx);
    cx.executor().advance_clock(Duration::from_millis(60));
    draw(cx);
    let mid = height(cx);
    assert!(mid > px(0.) && mid < px(80.), "{mid:?}");
    settle(cx);
    assert_eq!(height(cx), px(0.));
}

#[test]
fn a_centered_fab_takes_the_first_half_of_the_actions_before_it() {
    assert_eq!(actions_before_center(0), 0);
    assert_eq!(actions_before_center(1), 1);
    assert_eq!(actions_before_center(2), 1);
    assert_eq!(actions_before_center(3), 2);
    assert_eq!(actions_before_center(4), 2);
}

#[gpui::test]
fn the_bottom_bar_fab_trails_or_centers(cx: &mut TestAppContext) {
    struct Bar {
        centered: bool,
    }
    impl Render for Bar {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            let action =
                |name: &'static str| div().debug_selector(move || name.into()).size(px(48.));
            div().w(px(412.)).child(
                BottomAppBar::new("bottom", "Actions")
                    .action(action("first-action"))
                    .action(action("second-action"))
                    .fab(action("fab").size(px(56.)))
                    .fab_centered(self.centered),
            )
        }
    }
    cx.update(gpui_base::init);
    let (view, cx) = cx.add_window_view(|_, _| Bar { centered: false });
    settle(cx);
    let fab = cx.debug_bounds("fab").unwrap();
    assert_eq!(fab.right(), px(412. - TITLE_INSET), "trailing FAB");
    let second = cx.debug_bounds("second-action").unwrap();
    assert!(second.right() < fab.left());
    view.update(cx, |view, cx| {
        view.centered = true;
        cx.notify();
    });
    settle(cx);
    let fab = cx.debug_bounds("fab").unwrap();
    // The two flexible rows are equal, so the FAB sits in their middle.
    let middle = (EDGE + (412. - TITLE_INSET)) / 2.;
    assert!((f32::from(fab.center().x) - middle).abs() < 0.5, "{fab:?}");
    assert!(cx.debug_bounds("first-action").unwrap().right() < fab.left());
    let second = cx.debug_bounds("second-action").unwrap();
    assert!(second.left() > fab.right(), "the second half follows");
    assert_eq!(second.right(), px(412. - TITLE_INSET));
}
