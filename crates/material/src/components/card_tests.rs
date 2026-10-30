use super::{CardSurface, CardVariant, card};
use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement,
    Styled, TestAppContext, Window, div, px,
};

struct Cards;
impl Render for Cards {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div().flex().children(
            [
                (CardVariant::Filled, "filled-child"),
                (CardVariant::Outlined, "outlined-child"),
                (CardVariant::Elevated, "elevated-child"),
            ]
            .into_iter()
            .map(|(variant, name)| {
                card(variant)
                    .w(px(100.))
                    .h(px(80.))
                    .flex_shrink_0()
                    .child(div().w_full().h_full().debug_selector(move || name.into()))
            }),
        )
    }
}
#[gpui::test]
fn variants_have_identical_real_content_rects(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| Cards);
    cx.run_until_parked();
    let filled = cx.debug_bounds("filled-child").unwrap();
    let outlined = cx.debug_bounds("outlined-child").unwrap();
    let elevated = cx.debug_bounds("elevated-child").unwrap();
    assert_eq!(filled.size, gpui::size(px(68.), px(48.)));
    assert_eq!(outlined.size, filled.size);
    assert_eq!(elevated.size, filled.size);
    assert_eq!(outlined.origin.x - filled.origin.x, px(100.));
    assert_eq!(outlined.origin.y, filled.origin.y);
}
struct EdgeChild;
impl Render for EdgeChild {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        CardSurface::new(CardVariant::Outlined)
            .w(px(80.))
            .h(px(60.))
            .rounded(px(6.))
            .child(
                div()
                    .absolute()
                    .left(px(-10.))
                    .top(px(-10.))
                    .w(px(100.))
                    .h(px(80.))
                    .bg(gpui::rgb(0xff0000))
                    .debug_selector(|| "edge-child".into()),
            )
    }
}
#[gpui::test]
fn edge_child_is_clipped_and_refined_outline_paints_above_it(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| EdgeChild);
    cx.run_until_parked();
    let child_bounds = cx
        .debug_bounds("edge-child")
        .expect("child participates in layout");
    assert_eq!(child_bounds.origin, gpui::point(px(-10.), px(-10.)));
    cx.update(|window, _| {
        let scale = window.scale_factor();
        let quads = window.painted_quads();
        let outline = quads
            .iter()
            .find(|q| q.border_widths.top.0 == scale)
            .expect("outline");
        assert_eq!(outline.bounds.size.width.0, 80. * scale);
        assert_eq!(outline.bounds.size.height.0, 60. * scale);
        assert_eq!(outline.corner_radii.top_left.0, 6. * scale);
        assert_eq!(outline.corner_radii.bottom_right.0, 6. * scale);
        #[cfg(feature = "gpui-fast")]
        {
            // Fast transports rounded clipping as public LayerFrame scenes
            // referenced by a sprite, not as a flat root quad or a Path.
            let scene = window.painted_scene();
            let child = scene
                .layers
                .frames
                .iter()
                .filter_map(|frame| frame.content.scene())
                .flat_map(|scene| scene.quads.iter())
                .find(|q| q.background == gpui::solid_background(gpui::rgb(0xff0000)))
                .expect("actual red child must remain in a rounded clip layer");
            assert_eq!(child.bounds.origin.x.0, -10. * scale);
            assert_eq!(child.content_mask.bounds.origin.x.0, 0.);
            assert_eq!(child.content_mask.bounds.size.width.0, 80. * scale);
            assert_eq!(child.content_mask.bounds.size.height.0, 60. * scale);
            let group = scene
                .polychrome_sprites
                .iter()
                .find(|group| group.corner_radii.top_left.0 == 6. * scale)
                .expect("rounded child clip group");
            assert_eq!(group.bounds.size.width.0, 80. * scale);
            assert_eq!(group.bounds.size.height.0, 60. * scale);
            assert!(
                outline.order > group.order,
                "outline must paint above child clip group"
            );
        }
        #[cfg(not(feature = "gpui-fast"))]
        {
            // Upstream has rectangular overflow only; do not claim rounded
            // clipping. Still verify its actual child mask and outline order.
            let child = quads
                .iter()
                .find(|q| q.bounds.origin.x.0 == -10. * scale)
                .expect("child quad");
            assert_eq!(child.content_mask.bounds.size.width.0, 80. * scale);
            assert_eq!(child.content_mask.bounds.size.height.0, 60. * scale);
            assert!(outline.order > child.order);
        }
    });
}
struct InteractiveCard {
    hidden: bool,
}
impl Render for InteractiveCard {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let hidden = self.hidden;
        CardSurface::new(CardVariant::Outlined)
            .id("accessible-card")
            .role(gpui::Role::Article)
            .aria_label("Reference card")
            .w(px(80.))
            .h(px(60.))
            .hover(move |style| {
                let style = style.opacity(0.25).rounded(px(3.));
                if hidden { style.invisible() } else { style }
            })
    }
}
#[gpui::test]
fn hover_refinement_and_accessibility_survive_surface_adapter(cx: &mut TestAppContext) {
    #[cfg(feature = "gpui-fast")]
    if std::env::var("GPUI_A11Y_ACTIVE").as_deref() != Ok("1") {
        // Existing Fast test-support switch is OnceLock-backed: use a child
        // process rather than racing other tests by mutating global env.
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "components::feedback::card_tests::hover_refinement_and_accessibility_survive_surface_adapter", "--nocapture"])
            .env("GPUI_A11Y_ACTIVE", "1").output().unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("material-card-ax-verified"),
            "the child process must execute the assertions, not match zero tests"
        );
        return;
    }
    let (view, cx) = cx.add_window_view(|_, _| InteractiveCard { hidden: false });
    cx.run_until_parked();
    cx.simulate_mouse_move(
        gpui::point(px(30.), px(30.)),
        None,
        gpui::Modifiers::default(),
    );
    super::tests::redraw(cx);
    cx.update(|window, _| {
        let scale = window.scale_factor();
        let quads = window.painted_quads();
        let outline = quads
            .iter()
            .find(|q| q.border_widths.top.0 == scale)
            .unwrap();
        assert_eq!(outline.corner_radii.top_left.0, 3. * scale);
        let expected: gpui::Hsla =
            crate::theme::color(crate::color::ColorRole::OUTLINE_VARIANT).into();
        assert!((outline.border_color.a - expected.a * 0.25).abs() < 0.00001);
        #[cfg(feature = "gpui-fast")]
        {
            let tree = window
                .debug_a11y_tree_json()
                .expect("accessibility enabled");
            assert!(tree.contains("Reference card"), "{tree}");
            assert!(tree.to_lowercase().contains("article"), "{tree}");
        }
    });
    view.update(cx, |view, cx| {
        view.hidden = true;
        cx.notify();
    });
    super::tests::redraw(cx);
    cx.update(|window, _| {
        assert!(
            window
                .painted_quads()
                .iter()
                .all(|q| q.border_widths.top.0 == 0.),
            "hidden hover must suppress outline"
        )
    });
    #[cfg(feature = "gpui-fast")]
    println!("material-card-ax-verified");
}
struct TextInheritance(std::rc::Rc<std::cell::RefCell<Vec<gpui::Hsla>>>);
impl Render for TextInheritance {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let probe = |seen: std::rc::Rc<std::cell::RefCell<Vec<gpui::Hsla>>>| {
            gpui::canvas(
                |_, _, _| (),
                move |_, _, window, _| {
                    seen.borrow_mut().push(window.text_style().color);
                },
            )
            .size(px(10.))
        };
        div()
            .text_color(gpui::rgb(0xff0000))
            .child(CardSurface::new(CardVariant::Filled).child(probe(self.0.clone())))
            .child(
                CardSurface::new(CardVariant::Filled)
                    .text_color(gpui::rgb(0x00ff00))
                    .child(probe(self.0.clone())),
            )
    }
}
#[gpui::test]
fn content_uses_on_surface_and_preserves_explicit_text_override(cx: &mut TestAppContext) {
    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let (_, cx) = cx.add_window_view(|_, _| TextInheritance(seen.clone()));
    cx.run_until_parked();
    let seen = seen.borrow();
    assert!(seen.len() >= 2);
    let pair = &seen[seen.len() - 2..];
    let default: gpui::Hsla = crate::theme::color(crate::color::ColorRole::ON_SURFACE).into();
    assert_eq!(pair[0], default);
    assert_eq!(pair[1], gpui::Hsla::from(gpui::rgb(0x00ff00)));
}

struct ActionCard {
    clicks: std::rc::Rc<std::cell::Cell<usize>>,
}
impl Render for ActionCard {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let clicks = self.clicks.clone();
        div().p(px(40.)).child(
            super::MaterialCard::new("action-card", CardVariant::Elevated)
                .accessibility_label("Open")
                .draggable(true)
                .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
                .w(px(100.))
                .child(
                    div()
                        .h(px(60.))
                        .debug_selector(|| "action-card-body".into()),
                ),
        )
    }
}

#[gpui::test]
fn a_material_card_clicks_drags_and_springs_home(cx: &mut TestAppContext) {
    use gpui::{Modifiers, MouseButton, point};
    let clicks = std::rc::Rc::new(std::cell::Cell::new(0));
    let probe = clicks.clone();
    let (_, cx) = cx.add_window_view(move |_, _| ActionCard { clicks: probe });
    cx.run_until_parked();
    super::tests::redraw(cx);
    let home = cx.debug_bounds("action-card-body").unwrap();
    // A click acts.
    cx.simulate_click(home.center(), Modifiers::none());
    super::tests::redraw(cx);
    cx.run_until_parked();
    assert_eq!(clicks.get(), 1);

    // A press that travels carries the card with the pointer and is no click.
    let start = home.center();
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    super::tests::redraw(cx);
    let to = start + point(px(30.), px(20.));
    cx.simulate_mouse_move(to, Some(MouseButton::Left), Modifiers::none());
    super::tests::redraw(cx);
    let carried = cx.debug_bounds("action-card-body").unwrap();
    assert_eq!(carried.origin - home.origin, point(px(30.), px(20.)));
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::none());
    super::tests::redraw(cx);
    cx.run_until_parked();
    assert_eq!(clicks.get(), 1, "a drag is not a click");

    // Released, it springs back to its place.
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(2));
    super::tests::redraw(cx);
    super::tests::redraw(cx);
    assert_eq!(
        cx.debug_bounds("action-card-body").unwrap().origin,
        home.origin
    );
    // And it clicks again afterwards.
    cx.simulate_click(home.center(), Modifiers::none());
    super::tests::redraw(cx);
    cx.run_until_parked();
    assert_eq!(clicks.get(), 2);
}
