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

struct DragCard;
impl Render for DragCard {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div().p(px(40.)).child(
            super::MaterialCard::new("action-card", CardVariant::Elevated)
                .draggable(true)
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
fn a_dragged_card_stays_under_the_pointer(cx: &mut TestAppContext) {
    use gpui::{Modifiers, MouseButton, point};
    let (_, cx) = cx.add_window_view(|_, _| DragCard);
    cx.run_until_parked();
    super::tests::redraw(cx);
    let home = cx.debug_bounds("action-card-body").unwrap();
    let start = home.center();
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    super::tests::redraw(cx);
    let to = start + point(px(30.), px(20.));
    cx.simulate_mouse_move(to, Some(MouseButton::Left), Modifiers::none());
    super::tests::redraw(cx);
    let carried = cx.debug_bounds("action-card-body").unwrap();
    assert_eq!(carried.origin - home.origin, point(px(30.), px(20.)));
}

struct SelectedCard(bool);
impl Render for SelectedCard {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div().p(px(20.)).child(
            super::MaterialCard::new("selected-card", CardVariant::Outlined)
                .selected(self.0)
                .accessibility_label("Option")
                .on_click(|_, _, _| {})
                .w(px(100.))
                .child(
                    div()
                        .h(px(60.))
                        .debug_selector(|| "selected-card-body".into()),
                ),
        )
    }
}
#[gpui::test]
fn a_selected_outlined_card_draws_a_secondary_edge_and_a_check(cx: &mut TestAppContext) {
    use crate::color::ColorRole;
    let (view, cx) = cx.add_window_view(|_, _| SelectedCard(true));
    cx.run_until_parked();
    let edge = |cx: &mut gpui::VisualTestContext| {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            let quads = window.painted_quads();
            let edge = quads
                .iter()
                .find(|q| q.border_widths.top.0 > 0. && q.bounds.size.width.0 == 100. * scale)
                .expect("the card's edge");
            (edge.border_widths.top.0 / scale, edge.border_color)
        })
    };
    let secondary: gpui::Hsla = crate::theme::color(ColorRole::SECONDARY).into();
    assert_eq!(edge(cx), (1., secondary), "selected: 1dp secondary");
    // MDC Android's check: 24dp, 8dp in from the top-end corner.
    let card = cx.debug_bounds("selected-card-body").unwrap();
    let check = cx
        .debug_bounds("material-card-check")
        .expect("a selected card shows its check");
    assert_eq!(check.size, gpui::size(px(24.), px(24.)));
    assert_eq!(check.top(), card.top() + px(8.));
    assert_eq!(check.right(), card.right() - px(8.));

    view.update(cx, |view, cx| {
        view.0 = false;
        cx.notify();
    });
    super::tests::redraw(cx);
    let outline: gpui::Hsla = crate::theme::color(ColorRole::OUTLINE_VARIANT).into();
    assert_eq!(edge(cx), (1., outline), "unselected: 1dp outline-variant");
    assert!(cx.debug_bounds("material-card-check").is_none(), "no check");
}

/// The fill of the 100px-wide card in the last frame.
fn card_fill(cx: &mut gpui::VisualTestContext) -> gpui::Background {
    cx.update(|window, _| {
        let scale = window.scale_factor();
        window
            .painted_quads()
            .iter()
            .find(|q| q.bounds.size.width.0 == 100. * scale && !q.background.is_transparent())
            .map(|q| q.background)
            .expect("the card's fill")
    })
}

struct Composed(bool);
impl Render for Composed {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let card = CardSurface::new(CardVariant::Filled).w(px(100.)).h(px(60.));
        div().child(if self.0 {
            card.selected(true).state(super::CardState::Hovered)
        } else {
            card.state(super::CardState::Hovered).selected(true)
        })
    }
}
#[gpui::test]
fn selection_and_state_compose_in_either_order(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| Composed(true));
    cx.run_until_parked();
    let first = card_fill(cx);
    view.update(cx, |view, cx| {
        view.0 = false;
        cx.notify();
    });
    super::tests::redraw(cx);
    assert_eq!(card_fill(cx), first);
}

struct GroupHarness {
    selected: usize,
    changes: Vec<usize>,
}
impl Render for GroupHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let mut group = super::CardGroup::new("choices", "Choices")
            .flex()
            .gap(px(8.))
            .on_change(cx.listener(|this, index: &usize, _, cx| {
                this.selected = *index;
                this.changes.push(*index);
                cx.notify();
            }));
        for index in 0..3 {
            group = group.card(
                super::MaterialCard::new(("choice", index), CardVariant::Outlined)
                    .selected(self.selected == index)
                    .w(px(80.))
                    .child(div().h(px(40.))),
            );
        }
        group
    }
}
#[gpui::test]
fn a_card_group_is_one_tab_stop_whose_arrows_choose(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        crate::components::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|_, _| GroupHarness {
        selected: 1,
        changes: Vec::new(),
    });
    super::tests::redraw(cx);
    // The selected card is the group's only tab stop.
    cx.update(|window, cx| window.focus_next(cx));
    let entry = cx.update(|window, cx| window.focused(cx));
    assert!(entry.is_some());
    cx.update(|window, cx| window.focus_next(cx));
    assert_eq!(cx.update(|window, cx| window.focused(cx)), entry);
    cx.simulate_keystrokes("right");
    super::tests::redraw(cx);
    view.read_with(cx, |h, _| {
        assert_eq!((h.selected, &h.changes[..]), (2, &[2][..]))
    });
    cx.simulate_keystrokes("left");
    super::tests::redraw(cx);
    view.read_with(cx, |h, _| assert_eq!(h.changes, [2, 1]));
}

/// An action card rounded by its caller, clipped by it or not.
struct RoundedCard {
    clip: bool,
}
impl Render for RoundedCard {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let card = super::MaterialCard::new("rounded-card", CardVariant::Filled)
            .accessibility_label("Open")
            .on_click(|_, _, _| {})
            .w(px(100.))
            .rounded(px(20.))
            .child(div().h(px(60.)));
        div().p(px(40.)).child(if self.clip {
            card.overflow_hidden()
        } else {
            card
        })
    }
}

/// The card's fill and its keyboard focus ring in the last frame, as
/// `(fill radius, ring bounds, ring radius, ring clipped)` in logical px.
fn rounded_card(cx: &mut TestAppContext, clip: bool) -> (f32, gpui::Bounds<f32>, f32, bool) {
    cx.update(gpui_base::init);
    let (_, cx) = cx.add_window_view(move |_, _| RoundedCard { clip });
    cx.run_until_parked();
    cx.update(|window, cx| window.focus_next(cx));
    // A key press puts the window in keyboard modality.
    let keystroke =
        cx.update(|_, cx| gpui::Keystroke::parse_with_os("a", cx.platform_info().os).unwrap());
    cx.simulate_event(gpui::KeyDownEvent {
        keystroke,
        is_held: false,
        prefer_character_input: false,
    });
    super::tests::redraw(cx);
    cx.update(|window, _| {
        let scale = window.scale_factor();
        let quads = window.painted_quads();
        let fill = quads
            .iter()
            .find(|q| q.bounds.size.width.0 == 100. * scale && !q.background.is_transparent())
            .expect("the card's fill");
        let ring = quads
            .iter()
            .find(|q| q.border_widths.top.0 == 3. * scale)
            .expect("the focus ring");
        // The ring may be painted in pieces, each under its own mask: it is
        // clipped when their masks together do not cover it.
        let covered = quads
            .iter()
            .filter(|q| q.border_widths.top.0 == 3. * scale && q.bounds == ring.bounds)
            .map(|q| q.content_mask.bounds)
            .reduce(|a, b| a.union(&b))
            .expect("the ring's masks");
        let clipped = covered.intersect(&ring.bounds) != ring.bounds;
        let bounds = gpui::Bounds {
            origin: gpui::point(
                ring.bounds.origin.x.0 / scale,
                ring.bounds.origin.y.0 / scale,
            ),
            size: gpui::size(
                ring.bounds.size.width.0 / scale,
                ring.bounds.size.height.0 / scale,
            ),
        };
        (
            fill.corner_radii.top_left.0 / scale,
            bounds,
            ring.corner_radii.top_left.0 / scale,
            clipped,
        )
    })
}

#[gpui::test]
fn a_cards_surface_and_ring_follow_its_rounding(cx: &mut TestAppContext) {
    let (fill, ring, ring_radius, clipped) = rounded_card(cx, false);
    assert_eq!(fill, 20., "the surface takes the caller's radius");
    assert_eq!(ring_radius, 25., "the ring follows it, 5px out");
    assert_eq!(
        ring.size.width, 110.,
        "the ring stays 2dp outside, 3dp wide"
    );
    assert!(!clipped, "nothing clips the ring");
}

#[gpui::test]
fn a_callers_clip_on_the_card_leaves_its_ring_whole(cx: &mut TestAppContext) {
    let (fill, _, ring_radius, clipped) = rounded_card(cx, true);
    assert_eq!(fill, 20.);
    assert_eq!(ring_radius, 25.);
    assert!(!clipped, "the card's own clip does not cut its ring");
}
