use super::*;
use core::prelude::v1::test;
use std::cell::{Cell, RefCell};

#[test]
fn items_paint_the_kit_roles_for_each_style() {
    use ColorRole as C;
    let standard = item_paint(MenuStyle::Standard, false);
    assert_eq!(standard.container, None);
    assert_eq!(standard.label, color(C::ON_SURFACE));
    assert_eq!(standard.icon, color(C::ON_SURFACE_VARIANT));
    let selected = item_paint(MenuStyle::Standard, true);
    assert_eq!(selected.container, Some(color(C::TERTIARY_CONTAINER)));
    assert_eq!(selected.label, color(C::ON_TERTIARY_CONTAINER));
    assert_eq!(selected.supporting, color(C::TERTIARY));
    let vibrant = item_paint(MenuStyle::Vibrant, true);
    assert_eq!(vibrant.container, Some(color(C::TERTIARY)));
    assert_eq!(vibrant.label, color(C::ON_TERTIARY));
    let baseline = item_paint(MenuStyle::Baseline, true);
    assert_eq!(baseline.container, Some(color(C::SECONDARY_CONTAINER)));
    assert_eq!(baseline.label, color(C::ON_SECONDARY_CONTAINER));
    assert_eq!(container_role(MenuStyle::Baseline), C::SURFACE_CONTAINER);
    assert_eq!(
        container_role(MenuStyle::Standard),
        C::SURFACE_CONTAINER_LOW
    );
    assert_eq!(container_role(MenuStyle::Vibrant), C::TERTIARY_CONTAINER);
}

#[test]
fn item_and_menu_heights_follow_the_kit() {
    let d = MenuDensity::Default;
    assert_eq!(item_height(MenuStyle::Standard, d, false, false), 48.);
    assert_eq!(item_height(MenuStyle::Standard, d, true, false), 64.);
    assert_eq!(item_height(MenuStyle::Baseline, d, false, false), 56.);
    assert_eq!(
        item_height(MenuStyle::Baseline, MenuDensity::Compact, false, false),
        48.
    );
    assert_eq!(
        item_height(MenuStyle::Baseline, MenuDensity::Dense, false, false),
        40.
    );
    assert_eq!(item_height(MenuStyle::Baseline, d, false, true), 56. + 17.);
    // The kit's baseline menu: 12 items of 56 inside 8dp padding.
    let baseline = vec![(false, vec![(false, false); 12])];
    assert_eq!(menu_height(MenuStyle::Baseline, d, baseline), 688.);
    // Three Expressive groups of four, the first labelled: 2dp gaps.
    let groups = vec![
        (true, vec![(false, false); 4]),
        (false, vec![(false, false); 4]),
        (false, vec![(false, false); 4]),
    ];
    assert_eq!(
        menu_height(MenuStyle::Vibrant, d, groups),
        3. * (4. + 4. * 48.) + 32. + 2. * 2.
    );
}

#[test]
fn a_menu_measures_its_groups_without_collecting_them() {
    let menu = MaterialMenu::new("menu")
        .style(MenuStyle::Vibrant)
        .group(
            MenuGroup::new()
                .label("Group")
                .items((0..4).map(|_| MaterialMenuItem::new("Item"))),
        )
        .group(MenuGroup::new().items((0..4).map(|_| MaterialMenuItem::new("Item"))));
    assert_eq!(
        menu.natural_height(),
        2. * (4. + 4. * 48.) + 32. + 2.,
        "two groups, the first labelled, 2dp apart"
    );
}

#[test]
fn keyboard_navigation_skips_disabled_items_and_wraps() {
    use super::super::roving::{Direction, target};
    // Menus rove with the shared roving helper; `skip` marks disabled items.
    let skip = [false, true, false, false];
    assert_eq!(target(&skip, None, Direction::Next), Some(0));
    assert_eq!(target(&skip, Some(0), Direction::Next), Some(2));
    assert_eq!(target(&skip, Some(3), Direction::Next), Some(0));
    assert_eq!(target(&skip, Some(0), Direction::Previous), Some(3));
    assert_eq!(target(&skip, Some(2), Direction::Previous), Some(0));
    assert_eq!(target(&skip, None, Direction::First), Some(0));
    assert_eq!(target(&skip, None, Direction::Last), Some(3));
    assert_eq!(target(&[true, true], None, Direction::Next), None);
}

struct ContextHarness {
    picked: Rc<Cell<usize>>,
    built: Rc<Cell<usize>>,
    focus: FocusHandle,
}

impl Render for ContextHarness {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let (picked, built) = (self.picked.clone(), self.built.clone());
        let region = move |id: &'static str, label: &'static str| {
            let (picked, built) = (picked.clone(), built.clone());
            MaterialContextMenu::new(id, move |_, _| {
                built.set(built.get() + 1);
                let picked = picked.clone();
                MaterialMenu::new((id, 0usize)).item(
                    MaterialMenuItem::new(label)
                        .leading(div().debug_selector(move || label.into()).size_full())
                        .on_click(move |_, _, _| picked.set(picked.get() + 1)),
                )
            })
        };
        div()
            .size_full()
            .track_focus(&self.focus)
            .child(
                div()
                    .debug_selector(|| "row".into())
                    .relative()
                    .w(px(400.))
                    .h(px(300.))
                    .child(region("outer", "outer-item"))
                    .child(
                        div()
                            .debug_selector(|| "inner".into())
                            .relative()
                            .size(px(100.))
                            .child(region("inner", "inner-item")),
                    ),
            )
            .child(div().debug_selector(|| "elsewhere".into()).size(px(40.)))
    }
}

fn context_menu(cx: &mut TestAppContext) -> (Entity<ContextHarness>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(move |_, cx| ContextHarness {
        picked: Rc::default(),
        built: Rc::default(),
        focus: cx.focus_handle(),
    });
    settle(cx);
    (view, cx)
}

fn right_press(cx: &mut VisualTestContext, at: Point<Pixels>) {
    cx.simulate_mouse_down(at, MouseButton::Right, Modifiers::default());
    cx.simulate_mouse_up(at, MouseButton::Right, Modifiers::default());
    settle(cx);
}

#[gpui::test]
fn a_right_press_opens_the_menu_at_the_pointer(cx: &mut TestAppContext) {
    let (view, cx) = context_menu(cx);
    let built = view.update(cx, |v, _| v.built.clone());
    assert_eq!(built.get(), 0, "nothing is built before the press");
    let row = cx.debug_bounds("row").unwrap();
    let at = row.origin + point(px(200.), px(150.));
    cx.simulate_click(at, Modifiers::default());
    settle(cx);
    assert!(
        cx.debug_bounds("material-menu").is_none(),
        "a left press does not"
    );

    right_press(cx, at);
    let menu = cx.debug_bounds("material-menu").expect("the menu opened");
    assert_eq!(menu.origin.x, at.x, "its start is at the pointer");
    assert!(menu.origin.y >= at.y && menu.origin.y - at.y <= px(4.));

    let moved = row.origin + point(px(150.), px(280.));
    right_press(cx, moved);
    let menu = cx.debug_bounds("material-menu").expect("the menu stayed");
    assert_eq!(menu.origin.x, moved.x, "a second press moves it");
}

#[gpui::test]
fn an_activated_context_item_runs_and_the_menu_leaves_the_tree(cx: &mut TestAppContext) {
    let (view, cx) = context_menu(cx);
    let (picked, built) = view.update(cx, |v, _| (v.picked.clone(), v.built.clone()));
    let row = cx.debug_bounds("row").unwrap();
    right_press(cx, row.origin + point(px(200.), px(150.)));
    let item = cx.debug_bounds("outer-item").unwrap().center();
    cx.simulate_click(item, Modifiers::default());
    settle(cx);
    assert_eq!(picked.get(), 1);
    assert!(cx.debug_bounds("material-menu").is_none());
    let settled = built.get();
    draw(cx);
    assert_eq!(built.get(), settled, "a closed menu builds nothing");
}

#[gpui::test]
fn escape_closes_a_context_menu_and_returns_focus(cx: &mut TestAppContext) {
    let (view, cx) = context_menu(cx);
    let focus = view.update(cx, |v, _| v.focus.clone());
    cx.update(|window, cx| focus.focus(window, cx));
    settle(cx);
    let row = cx.debug_bounds("row").unwrap();
    right_press(cx, row.origin + point(px(200.), px(150.)));
    cx.update(|window, _| assert!(!focus.is_focused(window), "the menu takes focus"));
    cx.simulate_keystrokes("down");
    settle(cx);
    cx.simulate_keystrokes("escape");
    settle(cx);
    assert!(cx.debug_bounds("material-menu").is_none());
    cx.update(|window, _| assert!(focus.is_focused(window), "focus comes back"));
}

#[gpui::test]
fn a_press_outside_closes_a_context_menu(cx: &mut TestAppContext) {
    let (_, cx) = context_menu(cx);
    let row = cx.debug_bounds("row").unwrap();
    right_press(cx, row.origin + point(px(200.), px(150.)));
    assert!(cx.debug_bounds("material-menu").is_some());
    let outside = cx.debug_bounds("elsewhere").unwrap().center();
    cx.simulate_click(outside, Modifiers::default());
    settle(cx);
    assert!(cx.debug_bounds("material-menu").is_none());
}

#[gpui::test]
fn the_innermost_context_menu_under_the_pointer_opens(cx: &mut TestAppContext) {
    let (_, cx) = context_menu(cx);
    let inner = cx.debug_bounds("inner").unwrap().center();
    right_press(cx, inner);
    assert!(cx.debug_bounds("inner-item").is_some());
    assert!(cx.debug_bounds("outer-item").is_none());
}

#[test]
fn a_closing_popup_paints_until_both_channels_settle() {
    assert!(popup_visible(true, 0., 0.));
    assert!(popup_visible(false, 0.4, 0.));
    assert!(popup_visible(false, 0., 0.2));
    assert!(!popup_visible(false, 0., 0.));
}

#[test]
fn item_text_too_long_for_the_menu_ends_in_an_ellipsis() {
    let mut line = item_text(
        TypeRole::LabelLarge,
        color(ColorRole::ON_SURFACE),
        "A label".into(),
    );
    let style = line.style();
    assert!(matches!(
        style.text.text_overflow,
        Some(TextOverflow::Truncate(_))
    ));
    assert_eq!(style.text.white_space, Some(WhiteSpace::Nowrap));
}

struct LongLabel;

impl Render for LongLabel {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        MaterialMenu::new("menu").width(160.).item(
            MaterialMenuItem::new("A label far too long for this menu to hold on one line")
                .supporting_text("Supporting text that is also far too long for this menu")
                .trailing(div().debug_selector(|| "trailing".into()).size_full()),
        )
    }
}

#[gpui::test]
fn a_long_label_leaves_the_trailing_slot_inside_the_menu(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
    });
    let (_, cx) = cx.add_window_view(|_, _| LongLabel);
    draw(cx);
    let menu = cx.debug_bounds("material-menu").expect("the menu drew");
    let trailing = cx.debug_bounds("trailing").expect("the slot drew");
    assert!(trailing.right() <= menu.right());
    assert!(trailing.size.width > px(0.));
}

struct PopupHarness {
    open: bool,
    picked: Rc<RefCell<Vec<usize>>>,
    changes: Rc<Cell<usize>>,
}

impl Render for PopupHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let menu = MaterialMenu::new("menu").items((0..3).map(|ix| {
            let picked = self.picked.clone();
            MaterialMenuItem::new("Item")
                .disabled(ix == 1)
                .on_click(move |_, _, _| picked.borrow_mut().push(ix))
        }));
        let changes = self.changes.clone();
        div().p(px(20.)).child(
            MenuPopup::new(
                "popup",
                div()
                    .debug_selector(|| "trigger".into())
                    .size(px(40.))
                    .bg(gpui::red()),
                menu,
            )
            .open(self.open)
            .on_open_change(cx.listener(move |this, open: &bool, _, cx| {
                this.open = *open;
                changes.set(changes.get() + 1);
                cx.notify();
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

fn settle(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(600));
        draw(cx);
    }
}

fn setup(cx: &mut TestAppContext) -> (Entity<PopupHarness>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    cx.add_window_view(|_, _| PopupHarness {
        open: false,
        picked: Rc::default(),
        changes: Rc::default(),
    })
}

#[gpui::test]
fn the_popup_opens_below_its_trigger_and_navigates_by_keyboard(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx);
    draw(cx);
    assert!(cx.debug_bounds("material-menu").is_none());
    let trigger = cx.debug_bounds("trigger").unwrap();
    cx.simulate_mouse_down(trigger.center(), MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_up(trigger.center(), MouseButton::Left, Modifiers::default());
    settle(cx);
    cx.update(|_, cx| assert!(view.read(cx).open, "a pointer press opens the menu"));
    let surface = cx.debug_bounds("menu-popup-surface").unwrap();
    assert_eq!(surface.top(), trigger.bottom() + px(4.));
    assert_eq!(surface.size.height, px(3. * 48. + 4.), "fully revealed");
    // The menu root has focus: Down focuses the first item, the next Down
    // skips the disabled one, and Enter activates.
    cx.simulate_keystrokes("down down enter");
    settle(cx);
    cx.update(|_, cx| {
        assert_eq!(*view.read(cx).picked.borrow(), vec![2]);
        assert!(!view.read(cx).open, "activating an item closes the menu");
    });
    assert!(
        cx.debug_bounds("material-menu").is_none(),
        "a settled close removes the surface"
    );
}

struct CoveredTrigger {
    open: bool,
    changes: Rc<Cell<usize>>,
    covers: Rc<Cell<usize>>,
}

impl Render for CoveredTrigger {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let menu = MaterialMenu::new("menu").items([MaterialMenuItem::new("Item")]);
        let changes = self.changes.clone();
        let covers = self.covers.clone();
        div()
            .relative()
            .size_full()
            .child(
                div().p(px(20.)).child(
                    MenuPopup::new(
                        "popup",
                        crate::components::MaterialButton::new("menu-trigger")
                            .debug_selector(|| "trigger".into())
                            .child("Open"),
                        menu,
                    )
                    .open(self.open)
                    .on_open_change(cx.listener(
                        move |this, open: &bool, _, cx| {
                            this.open = *open;
                            changes.set(changes.get() + 1);
                            cx.notify();
                        },
                    )),
                ),
            )
            .child(
                // What an app bar or a dialog's scrim does to the trigger.
                div()
                    .id("cover")
                    .absolute()
                    .inset_0()
                    .occlude()
                    .on_click(move |_, _, _| covers.set(covers.get() + 1)),
            )
    }
}

#[gpui::test]
fn a_press_on_what_covers_the_trigger_does_not_open_the_menu(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|_, _| CoveredTrigger {
        open: false,
        changes: Rc::default(),
        covers: Rc::default(),
    });
    draw(cx);
    let trigger = cx.debug_bounds("trigger").unwrap();
    cx.simulate_click(trigger.center(), Modifiers::default());
    settle(cx);
    cx.update(|_, cx| {
        let view = view.read(cx);
        assert_eq!(view.changes.get(), 0, "the covered trigger heard the press");
        assert!(!view.open);
        assert_eq!(view.covers.get(), 1, "the cover took it");
    });
}

#[gpui::test]
fn escape_closes_the_menu(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx);
    view.update(cx, |v, cx| {
        v.open = true;
        cx.notify();
    });
    settle(cx);
    assert!(cx.debug_bounds("material-menu").is_some());
    cx.simulate_keystrokes("escape");
    settle(cx);
    cx.update(|_, cx| assert!(!view.read(cx).open));
    assert!(cx.debug_bounds("material-menu").is_none());
}

struct DropdownHarness {
    open: bool,
    selected: Option<usize>,
    disabled_options: Vec<usize>,
    density: TextFieldDensity,
}

impl Render for DropdownHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div().p(px(20.)).child(
            ExposedDropdown::new("dropdown", "Fruit")
                .options(["Apple", "Banana", "Cherry"])
                .disabled_options(self.disabled_options.clone())
                .density(self.density)
                .selected(self.selected)
                .open(self.open)
                .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                    this.open = *open;
                    cx.notify();
                }))
                .on_select(cx.listener(|this, ix: &usize, _, cx| {
                    this.selected = Some(*ix);
                    cx.notify();
                })),
        )
    }
}

#[gpui::test]
fn the_exposed_dropdown_selects_an_option(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|_, _| DropdownHarness {
        open: false,
        selected: None,
        disabled_options: Vec::new(),
        density: TextFieldDensity::Default,
    });
    draw(cx);
    let trigger = cx.debug_bounds("dropdown-trigger").unwrap();
    cx.simulate_click(trigger.center(), Modifiers::default());
    settle(cx);
    cx.update(|_, cx| assert!(view.read(cx).open));
    let surface = cx.debug_bounds("menu-popup-surface").unwrap();
    // Click the second option (rows are 48dp under a 2dp inset).
    cx.simulate_click(
        point(surface.center().x, surface.top() + px(2. + 48. + 24.)),
        Modifiers::default(),
    );
    settle(cx);
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).selected, Some(1));
        assert!(!view.read(cx).open);
    });
}

#[gpui::test]
fn a_disabled_option_cannot_be_chosen_and_compact_is_48dp(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|_, _| DropdownHarness {
        open: false,
        selected: None,
        disabled_options: vec![1],
        density: TextFieldDensity::Compact,
    });
    draw(cx);
    let trigger = cx.debug_bounds("dropdown-trigger").unwrap();
    assert_eq!(trigger.size.height, px(48.), "a compact field");
    cx.simulate_click(trigger.center(), Modifiers::default());
    settle(cx);
    let surface = cx.debug_bounds("menu-popup-surface").unwrap();
    cx.simulate_click(
        point(surface.center().x, surface.top() + px(2. + 48. + 24.)),
        Modifiers::default(),
    );
    settle(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).selected, None, "banana is disabled"));
}

#[test]
fn expressive_density_takes_half_the_baseline_steps() {
    let s = MenuStyle::Standard;
    assert_eq!(item_height(s, MenuDensity::Compact, false, false), 44.);
    assert_eq!(item_height(s, MenuDensity::Dense, false, false), 40.);
    assert_eq!(item_height(s, MenuDensity::Dense, true, false), 56.);
}

#[test]
fn item_tops_follow_groups_labels_and_dividers() {
    let item = || MaterialMenuItem::new("Item");
    let expressive = MaterialMenu::new("menu")
        .group(MenuGroup::new().label("First").items([item(), item()]))
        .group(MenuGroup::new().items([item()]));
    assert_eq!(expressive.item_top(0), Some(2. + 32.));
    assert_eq!(expressive.item_top(1), Some(2. + 32. + 48.));
    // Past the first group's bottom inset, the 2dp gap and the next inset.
    assert_eq!(expressive.item_top(2), Some(2. + 32. + 96. + 2. + 2. + 2.));
    assert_eq!(expressive.item_top(3), None);
    let baseline = MaterialMenu::new("menu")
        .style(MenuStyle::Baseline)
        .items([item().divider(true), item()]);
    assert_eq!(baseline.item_top(1), Some(8. + 56. + 17.));
}

#[test]
fn placements_choose_the_reveal_edge_and_cascade_side() {
    assert!(!opens_upward(Anchor::TopLeft));
    assert!(opens_upward(Anchor::BottomRight));
    assert!(!cascades_left(Anchor::TopLeft));
    assert!(!cascades_left(Anchor::LeftCenter));
    assert!(cascades_left(Anchor::TopRight));
    assert!(cascades_left(Anchor::RightCenter));
}

struct CascadeHarness {
    open: bool,
    anchor: Anchor,
    close_on_activate: bool,
    submenu: bool,
    picked: Rc<RefCell<Vec<&'static str>>>,
}

impl Render for CascadeHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let pick = |name: &'static str| {
            let picked = self.picked.clone();
            MaterialMenuItem::new(name).on_click(move |_, _, _| picked.borrow_mut().push(name))
        };
        let open_submenu = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.submenu = true;
            cx.notify();
        });
        let menu = MaterialMenu::new("menu").items([
            pick("Cut"),
            MaterialMenuItem::new("Share")
                .submenu(true)
                .on_click(open_submenu),
        ]);
        let submenu = MaterialMenu::new("submenu").items([pick("Email"), pick("Link")]);
        // Room on every side for the menu and its reserved cascade column.
        div().pl(px(500.)).pt(px(200.)).flex().items_start().child(
            MenuPopup::new(
                "popup",
                div()
                    .debug_selector(|| "trigger".into())
                    .w(px(100.))
                    .h(px(40.)),
                menu,
            )
            .open(self.open)
            .anchor(self.anchor)
            .close_on_activate(self.close_on_activate)
            .submenu(1, submenu, self.submenu)
            .on_submenu_change(cx.listener(|this, open: &bool, _, cx| {
                this.submenu = *open;
                cx.notify();
            }))
            .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                this.open = *open;
                if !*open {
                    this.submenu = false;
                }
                cx.notify();
            })),
        )
    }
}

fn cascade(
    cx: &mut TestAppContext,
    anchor: Anchor,
) -> (Entity<CascadeHarness>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    cx.add_window_view(move |_, _| CascadeHarness {
        open: true,
        anchor,
        close_on_activate: true,
        submenu: false,
        picked: Rc::default(),
    })
}

#[gpui::test]
fn placements_put_the_menu_above_below_or_beside_the_trigger(cx: &mut TestAppContext) {
    let (view, cx) = cascade(cx, Anchor::TopLeft);
    settle(cx);
    let trigger = cx.debug_bounds("trigger").unwrap();
    for anchor in [
        Anchor::BottomLeft,
        Anchor::TopRight,
        Anchor::LeftCenter,
        Anchor::RightCenter,
    ] {
        view.update(cx, |view, cx| {
            view.anchor = anchor;
            cx.notify();
        });
        settle(cx);
        let surface = cx.debug_bounds("menu-popup-surface").unwrap();
        match anchor {
            Anchor::BottomLeft => {
                assert_eq!(surface.bottom(), trigger.top() - px(4.), "above");
                assert_eq!(surface.left(), trigger.left(), "start-aligned");
            }
            Anchor::TopRight => {
                assert_eq!(surface.top(), trigger.bottom() + px(4.), "below");
                assert_eq!(surface.right(), trigger.right(), "end-aligned");
            }
            Anchor::LeftCenter => {
                assert_eq!(surface.top(), trigger.top(), "right start");
                assert_eq!(surface.left(), trigger.right() + px(4.), "right start");
            }
            _ => {
                assert_eq!(surface.top(), trigger.top(), "left start");
                assert_eq!(surface.right(), trigger.left() - px(4.), "left start");
            }
        }
    }
}

#[gpui::test]
fn a_submenu_cascades_beside_its_item_by_keyboard(cx: &mut TestAppContext) {
    let (view, cx) = cascade(cx, Anchor::TopLeft);
    settle(cx);
    // Down twice reaches "Share"; Right opens its sub-menu beside it.
    cx.simulate_keystrokes("down down right");
    settle(cx);
    cx.update(|_, cx| assert!(view.read(cx).submenu));
    let surface = cx.debug_bounds("menu-popup-surface").unwrap();
    let submenu = cx.debug_bounds("menu-popup-submenu").unwrap();
    assert_eq!(submenu.left(), surface.right() + px(4.));
    // Its first item lines up with "Share", the second 48dp row.
    assert_eq!(submenu.top(), surface.top() + px(48.));
    // Left closes it; Right, Down twice and Enter pick "Link" and close all.
    cx.simulate_keystrokes("left");
    settle(cx);
    cx.update(|_, cx| assert!(!view.read(cx).submenu));
    cx.simulate_keystrokes("right down down enter");
    settle(cx);
    cx.update(|_, cx| {
        assert_eq!(*view.read(cx).picked.borrow(), vec!["Link"]);
        assert!(
            !view.read(cx).open,
            "picking from a sub-menu closes the menu"
        );
    });
}

#[gpui::test]
fn the_menu_can_stay_open_after_activation(cx: &mut TestAppContext) {
    let (view, cx) = cascade(cx, Anchor::TopLeft);
    view.update(cx, |view, cx| {
        view.close_on_activate = false;
        cx.notify();
    });
    settle(cx);
    cx.simulate_keystrokes("down enter");
    settle(cx);
    cx.update(|_, cx| {
        assert_eq!(*view.read(cx).picked.borrow(), vec!["Cut"]);
        assert!(view.read(cx).open, "the menu stays open");
    });
}

struct ButtonTrigger {
    open: bool,
}

impl Render for ButtonTrigger {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let menu = MaterialMenu::new("menu").items([MaterialMenuItem::new("Item")]);
        div().p(px(40.)).flex().items_start().child(
            MenuPopup::new(
                "popup",
                // A Material button blocks the mouse for elements under it.
                crate::components::MaterialButton::new("trigger")
                    .debug_selector(|| "trigger".into())
                    .child("Open"),
                menu,
            )
            .open(self.open)
            .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                this.open = *open;
                cx.notify();
            })),
        )
    }
}

#[gpui::test]
fn a_material_button_trigger_toggles_the_menu_by_pointer(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|_, _| ButtonTrigger { open: false });
    settle(cx);
    let trigger = cx.debug_bounds("trigger").unwrap().center();
    cx.simulate_click(trigger, Modifiers::default());
    settle(cx);
    cx.update(|_, cx| assert!(view.read(cx).open, "a press opens the menu"));
    cx.simulate_click(trigger, Modifiers::default());
    settle(cx);
    cx.update(|_, cx| assert!(!view.read(cx).open, "a second press closes it"));
}

struct FullWidthDropdown {
    width: Pixels,
    open: bool,
}

impl Render for FullWidthDropdown {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div().flex().w(self.width).child(
            div().flex_1().min_w_0().child(
                ExposedDropdown::new("dropdown", "Folder")
                    .options(["Inbox", "Archive"])
                    .full_width()
                    .open(self.open),
            ),
        )
    }
}

#[gpui::test]
fn a_full_width_dropdown_fills_its_parent_and_sizes_its_menu_to_it(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|_, _| FullWidthDropdown {
        width: px(500.),
        open: false,
    });
    settle(cx);
    let trigger = cx.debug_bounds("dropdown-trigger").unwrap();
    assert_eq!(trigger.size.width, px(500.));
    view.update(cx, |v, cx| {
        v.open = true;
        cx.notify();
    });
    settle(cx);
    let menu = cx.debug_bounds("material-menu").unwrap();
    assert_eq!(menu.size.width, px(500.));

    view.update(cx, |v, cx| {
        v.width = px(300.);
        v.open = false;
        cx.notify();
    });
    settle(cx);
    assert_eq!(
        cx.debug_bounds("dropdown-trigger").unwrap().size.width,
        px(300.)
    );
    view.update(cx, |v, cx| {
        v.open = true;
        cx.notify();
    });
    settle(cx);
    assert_eq!(
        cx.debug_bounds("material-menu").unwrap().size.width,
        px(300.)
    );
}

/// The kit draws menus anchored to a text field as baseline menus: the select's
/// selected option is on the secondary container, not the expressive tertiary.
#[test]
fn an_exposed_dropdown_opens_a_baseline_menu() {
    assert_eq!(
        ExposedDropdown::new("select", "Fruit").style,
        MenuStyle::Baseline
    );
}

/// With motion on, a menu stays on screen as it closes: a second click or Enter
/// finds its items still there.
#[gpui::test]
fn a_closing_menu_does_not_activate_again(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
    });
    let (view, cx) = cx.add_window_view(|_, _| PopupHarness {
        open: true,
        picked: Rc::default(),
        changes: Rc::default(),
    });
    draw(cx);
    settle(cx);
    let surface = cx.debug_bounds("menu-popup-surface").expect("open");
    let spot = surface.origin + gpui::point(gpui::px(20.), gpui::px(20.));
    cx.simulate_click(spot, gpui::Modifiers::none());
    draw(cx);
    let picked = view.read_with(cx, |view, _| view.picked.borrow().len());
    assert_eq!(picked, 1);
    assert!(
        cx.debug_bounds("menu-popup-surface").is_some(),
        "still closing"
    );
    cx.simulate_click(spot, gpui::Modifiers::none());
    draw(cx);
    assert_eq!(view.read_with(cx, |view, _| view.picked.borrow().len()), 1);
}

/// A caller that only remembers what it was last told still ends up with the
/// sub-menu closed when the parent closes.
#[gpui::test]
fn closing_the_parent_reports_the_submenu_closed(cx: &mut TestAppContext) {
    let (view, cx) = cascade(cx, Anchor::BottomLeft);
    view.update(cx, |view, cx| {
        view.submenu = true;
        cx.notify();
    });
    draw(cx);
    settle(cx);
    view.update(cx, |view, cx| {
        // Closed from outside, without the harness resetting it.
        view.open = false;
        cx.notify();
    });
    draw(cx);
    cx.run_until_parked();
    assert!(
        !view.read_with(cx, |view, _| view.submenu),
        "told it closed"
    );
}

/// What a menu's height is worked out from is what its divider takes up.
#[gpui::test]
fn an_expressive_dividers_height_is_what_it_renders(cx: &mut TestAppContext) {
    let height = |divider| item_height(MenuStyle::Standard, MenuDensity::default(), false, divider);
    assert_eq!(height(true) - height(false), 5.);
    assert_eq!(EXPRESSIVE_DIVIDER, 2. + 1. + 2.);
    struct Divider;
    impl Render for Divider {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().flex().items_start().child(
                div()
                    .w(px(100.))
                    .debug_selector(|| "line".into())
                    .child(divider_line(true).mt_0()),
            )
        }
    }
    let (_, cx) = cx.add_window_view(|_, _| Divider);
    draw(cx);
    assert_eq!(cx.debug_bounds("line").unwrap().size.height, px(5.));
}

#[test]
fn a_scrolling_menu_moves_only_as_far_as_a_row_needs() {
    assert_eq!(reveal_offset(0., (48., 96.), 200.), 0., "already in view");
    assert_eq!(reveal_offset(0., (192., 240.), 200.), 40., "below the view");
    assert_eq!(reveal_offset(100., (48., 96.), 200.), 48., "above the view");
    assert_eq!(centered_offset((480., 528.), 200., 960.), 404.);
    assert_eq!(
        centered_offset((0., 48.), 200., 960.),
        0.,
        "clamped at the top"
    );
    assert_eq!(
        centered_offset((912., 960.), 200., 960.),
        760.,
        "and at the end"
    );
}

struct ScrollingHarness {
    open: bool,
    items: usize,
    max_height: Option<f32>,
    selected: Option<usize>,
    submenu: Option<usize>,
    submenu_open: bool,
    picked: Rc<RefCell<Vec<usize>>>,
}

impl Render for ScrollingHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let selected = self.selected;
        let submenu = self.submenu;
        let view = cx.entity();
        let mut menu = MaterialMenu::new("menu").items((0..self.items).map(|ix| {
            let picked = self.picked.clone();
            let view = view.clone();
            MaterialMenuItem::new("Item")
                .leading(
                    div()
                        .debug_selector(move || format!("item-{ix}"))
                        .size_full(),
                )
                .selected(selected == Some(ix))
                .submenu(submenu == Some(ix))
                .on_click(move |_, _, cx| {
                    picked.borrow_mut().push(ix);
                    if submenu == Some(ix) {
                        view.update(cx, |this, cx| {
                            this.submenu_open = true;
                            cx.notify();
                        });
                    }
                })
        }));
        if let Some(height) = self.max_height {
            menu = menu.max_height(height);
        }
        let mut popup = MenuPopup::new("popup", div().size(px(40.)), menu)
            .open(self.open)
            .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                this.open = *open;
                cx.notify();
            }));
        if let Some(owner) = self.submenu {
            popup = popup.submenu(
                owner,
                MaterialMenu::new("submenu").item(
                    MaterialMenuItem::new("Sub")
                        .leading(div().debug_selector(|| "sub-item".into()).size_full()),
                ),
                self.submenu_open,
            );
        }
        div().p(px(20.)).child(popup)
    }
}

fn scrolling(
    cx: &mut TestAppContext,
    harness: ScrollingHarness,
) -> (Entity<ScrollingHarness>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(move |_, _| harness);
    settle(cx);
    (view, cx)
}

fn tall_menu(items: usize, max_height: Option<f32>) -> ScrollingHarness {
    ScrollingHarness {
        open: true,
        items,
        max_height,
        selected: None,
        submenu: None,
        submenu_open: false,
        picked: Rc::default(),
    }
}

fn assert_shown(cx: &mut VisualTestContext, ix: usize) {
    let surface = cx.debug_bounds("menu-popup-surface").unwrap();
    let item = cx
        .debug_bounds(Box::leak(format!("item-{ix}").into_boxed_str()))
        .unwrap();
    assert!(
        item.top() >= surface.top() && item.bottom() <= surface.bottom(),
        "item {ix} at {item:?} is outside {surface:?}"
    );
}

#[gpui::test]
fn a_menu_past_its_max_height_scrolls_with_the_keyboard(cx: &mut TestAppContext) {
    let harness = tall_menu(20, Some(200.));
    let picked = harness.picked.clone();
    let (_, cx) = scrolling(cx, harness);
    let surface = cx.debug_bounds("menu-popup-surface").unwrap();
    assert_eq!(surface.size.height, px(200.));
    let below = cx.debug_bounds("item-12").unwrap();
    assert!(below.top() > surface.bottom(), "item 12 starts out of view");
    for _ in 0..13 {
        cx.simulate_keystrokes("down");
    }
    draw(cx);
    assert_shown(cx, 12);
    cx.simulate_keystrokes("end");
    draw(cx);
    assert_shown(cx, 19);
    cx.simulate_keystrokes("home");
    draw(cx);
    assert_shown(cx, 0);
    for _ in 0..13 {
        cx.simulate_keystrokes("down");
    }
    draw(cx);
    let item = cx.debug_bounds("item-12").unwrap().center();
    cx.simulate_click(item, Modifiers::default());
    settle(cx);
    assert_eq!(
        *picked.borrow(),
        vec![12],
        "a scrolled item takes its press"
    );
}

#[gpui::test]
fn a_popup_taller_than_its_window_scrolls_inside_it(cx: &mut TestAppContext) {
    let (_, cx) = scrolling(cx, tall_menu(200, None));
    let surface = cx.debug_bounds("menu-popup-surface").unwrap();
    let viewport = cx.update(|window, _| window.viewport_size());
    assert!(surface.top() >= px(0.));
    assert!(surface.bottom() <= viewport.height);
}

#[gpui::test]
fn a_scrolling_menu_opens_at_its_selected_item(cx: &mut TestAppContext) {
    let mut harness = tall_menu(20, Some(200.));
    harness.selected = Some(15);
    let (_, cx) = scrolling(cx, harness);
    assert_shown(cx, 15);
}

#[gpui::test]
fn a_menu_within_its_max_height_keeps_its_natural_height(cx: &mut TestAppContext) {
    let (_, cx) = scrolling(cx, tall_menu(3, Some(400.)));
    let surface = cx.debug_bounds("menu-popup-surface").unwrap();
    assert_eq!(surface.size.height, px(3. * 48. + 4.));
}

#[gpui::test]
fn a_sub_menu_opens_beside_its_owner_in_a_scrolled_menu(cx: &mut TestAppContext) {
    let mut harness = tall_menu(20, Some(200.));
    harness.submenu = Some(12);
    let (_, cx) = scrolling(cx, harness);
    for _ in 0..13 {
        cx.simulate_keystrokes("down");
    }
    draw(cx);
    cx.simulate_keystrokes("right");
    settle(cx);
    let owner = cx.debug_bounds("item-12").unwrap().center().y;
    let sub = cx.debug_bounds("sub-item").unwrap().center().y;
    assert!(
        (owner - sub).abs() < px(1.),
        "owner {owner:?}, sub-menu {sub:?}"
    );
}

struct LongDropdown;

impl Render for LongDropdown {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        ExposedDropdown::new("dropdown", "Folder")
            .options((0..40).map(|ix| format!("Folder {ix}")))
            .menu_max_height(240.)
            .open(true)
    }
}

#[gpui::test]
fn an_exposed_dropdown_limits_its_menu(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(|_, _| LongDropdown);
    settle(cx);
    let surface = cx.debug_bounds("menu-popup-surface").unwrap();
    assert_eq!(surface.size.height, px(240.));
}

/// A tall sub-menu owned by an item well down its menu, near whichever window edge it opens to.
struct TallCascade {
    anchor: Anchor,
}

impl Render for TallCascade {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let menu = MaterialMenu::new("menu")
            .items((0..6).map(|_| MaterialMenuItem::new("Item").submenu(true)));
        let submenu =
            MaterialMenu::new("submenu").items((0..60).map(|_| MaterialMenuItem::new("Sub")));
        let upward = opens_upward(self.anchor);
        div()
            .size_full()
            .flex()
            .flex_col()
            .when(upward, |root| root.justify_end())
            .p(px(20.))
            .child(
                MenuPopup::new(
                    "popup",
                    div()
                        .debug_selector(|| "trigger".into())
                        .w(px(100.))
                        .h(px(40.)),
                    menu,
                )
                .open(true)
                .anchor(self.anchor)
                // Below the trigger the last item leaves the least room under it, above the
                // trigger the first item leaves the least room over it.
                .submenu(if upward { 0 } else { 5 }, submenu, true),
            )
    }
}

fn tall_cascade_stays_in_the_window(anchor: Anchor, cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(move |_, _| TallCascade { anchor });
    settle(cx);
    let window = cx.update(|window, _| window.viewport_size());
    let submenu = cx.debug_bounds("menu-popup-submenu").unwrap();
    assert!(
        submenu.top() >= px(0.) && submenu.bottom() <= window.height,
        "{anchor:?}: sub-menu at {:?}..{:?} in a window {:?} tall",
        submenu.top(),
        submenu.bottom(),
        window.height
    );
    let surface = cx.debug_bounds("menu-popup-surface").unwrap();
    assert!(
        surface.top() >= px(0.) && surface.bottom() <= window.height,
        "{anchor:?}: menu at {:?}..{:?}",
        surface.top(),
        surface.bottom()
    );
}

#[gpui::test]
fn a_tall_submenu_below_its_trigger_stays_in_the_window(cx: &mut TestAppContext) {
    tall_cascade_stays_in_the_window(Anchor::TopLeft, cx);
}

#[gpui::test]
fn a_tall_submenu_above_its_trigger_stays_in_the_window(cx: &mut TestAppContext) {
    tall_cascade_stays_in_the_window(Anchor::BottomLeft, cx);
}

/// A menu centered on its selected item, as a dropdown opens.
struct SelectedMenu {
    picked: Rc<RefCell<Vec<usize>>>,
}

impl Render for SelectedMenu {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let menu = MaterialMenu::new("menu")
            .max_height(240.)
            .items((0..100).map(|ix| {
                let picked = self.picked.clone();
                MaterialMenuItem::new("Option")
                    .selected(ix == 50)
                    .on_click(move |_, _, _| picked.borrow_mut().push(ix))
            }));
        div().child(
            MenuPopup::new(
                "popup",
                div().debug_selector(|| "trigger".into()).size(px(40.)),
                menu,
            )
            .open(true),
        )
    }
}

fn pick_after(keys: &str, cx: &mut TestAppContext) -> Vec<usize> {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let picked: Rc<RefCell<Vec<usize>>> = Rc::default();
    let (_, cx) = cx.add_window_view({
        let picked = picked.clone();
        move |_, _| SelectedMenu { picked }
    });
    settle(cx);
    cx.simulate_keystrokes(keys);
    settle(cx);
    picked.borrow().clone()
}

#[gpui::test]
fn the_first_arrow_key_moves_from_the_selected_item(cx: &mut TestAppContext) {
    assert_eq!(pick_after("down enter", cx), vec![51]);
}

#[gpui::test]
fn the_first_arrow_key_up_moves_from_the_selected_item(cx: &mut TestAppContext) {
    assert_eq!(pick_after("up enter", cx), vec![49]);
}

struct HeroDropdown;

impl Render for HeroDropdown {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        ExposedDropdown::new("dropdown", "Language")
            .variant(TextFieldVariant::Hero)
            .options(["English", "Chinese"])
    }
}

#[gpui::test]
fn a_hero_dropdown_shows_its_label_where_a_field_puts_its_placeholder(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(|_, _| HeroDropdown);
    draw(cx);
    assert!(cx.debug_bounds("text-field-placeholder").is_some());
}

struct RowDropdown(bool);
impl Render for RowDropdown {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        ExposedDropdown::new("dropdown", "Language")
            .variant(TextFieldVariant::Outlined)
            .show_label(self.0)
            .options(["English", "Chinese"])
            .selected(Some(0))
    }
}

#[gpui::test]
fn a_dropdown_named_by_its_row_draws_no_label(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|_, _| RowDropdown(true));
    draw(cx);
    assert!(cx.debug_bounds("text-field-label").is_some());
    view.update(cx, |view, cx| {
        view.0 = false;
        cx.notify();
    });
    draw(cx);
    assert!(cx.debug_bounds("text-field-label").is_none());
    assert!(cx.debug_bounds("dropdown-trigger").is_some());
}

#[gpui::test]
fn a_keyboard_focused_item_paints_the_focus_layer(cx: &mut TestAppContext) {
    let (_, cx) = cascade(cx, Anchor::TopLeft);
    settle(cx);
    let paint = item_paint(MenuStyle::Standard, false);
    let focused = gpui::Background::from(state::layer(
        alpha(paint.layer, 0.),
        paint.layer,
        state::FOCUS,
    ));
    let painted = |cx: &mut VisualTestContext| {
        cx.update(|window, _| {
            // A popup's rounded clip keeps its quads in a layer of their own under Fast.
            #[cfg(feature = "gpui-fast")]
            let in_layers = window
                .painted_scene()
                .layers
                .frames
                .iter()
                .filter_map(|frame| frame.content.scene())
                .flat_map(|scene| scene.quads.iter())
                .any(|quad| quad.background == focused);
            #[cfg(not(feature = "gpui-fast"))]
            let in_layers = false;
            in_layers
                || window
                    .painted_quads()
                    .iter()
                    .any(|quad| quad.background == focused)
        })
    };
    assert!(!painted(cx), "no item has focus yet");
    cx.simulate_keystrokes("down");
    settle(cx);
    assert!(painted(cx), "as a focused specimen paints it");
}

/// A dropdown whose menu is the caller's: a labelled group of values with
/// supporting text and leading icons, then an action that is not a value.
struct RichDropdown {
    open: bool,
    picked: Rc<RefCell<Vec<&'static str>>>,
    full_width: bool,
}

fn rich_menu(picked: &Rc<RefCell<Vec<&'static str>>>) -> MaterialMenu {
    let item = |label: &'static str| {
        let picked = picked.clone();
        MaterialMenuItem::new(label)
            .leading(div().debug_selector(move || label.into()).size_full())
            .on_click(move |_, _, _| picked.borrow_mut().push(label))
    };
    MaterialMenu::new("rich-menu")
        .style(MenuStyle::Baseline)
        .group(
            MenuGroup::new()
                .label("Models")
                .item(item("Fast").supporting_text("Quick answers").divider(true))
                .item(
                    item("Deep")
                        .supporting_text("Long reasoning")
                        .selected(true),
                ),
        )
        .group(MenuGroup::new().item(item("Manage")))
}

impl Render for RichDropdown {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div().w(px(400.)).p(px(20.)).child(
            ExposedDropdown::new("dropdown", "Model")
                .menu(rich_menu(&self.picked))
                .value("Deep")
                .when(self.full_width, |dropdown| dropdown.full_width())
                .open(self.open)
                .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                    this.open = *open;
                    cx.notify();
                })),
        )
    }
}

fn rich_dropdown(
    full_width: bool,
    cx: &mut TestAppContext,
) -> (Entity<RichDropdown>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    cx.add_window_view(move |_, _| RichDropdown {
        open: false,
        picked: Rc::default(),
        full_width,
    })
}

#[gpui::test]
fn a_dropdown_opens_the_callers_menu_and_closes_on_its_items(cx: &mut TestAppContext) {
    let (view, cx) = rich_dropdown(false, cx);
    draw(cx);
    let trigger = cx.debug_bounds("dropdown-trigger").unwrap();
    cx.simulate_click(trigger.center(), Modifiers::default());
    settle(cx);
    cx.update(|_, cx| assert!(view.read(cx).open));
    for label in ["Fast", "Deep", "Manage"] {
        assert!(cx.debug_bounds(label).is_some(), "{label} is in the menu");
    }
    // Without a width of its own, the menu takes the field's (210 by default).
    let menu = cx.debug_bounds("material-menu").unwrap();
    assert_eq!(menu.size.width, px(210.));
    // The action item runs its own handler and closes the menu.
    let manage = cx.debug_bounds("Manage").unwrap();
    cx.simulate_click(manage.center(), Modifiers::default());
    settle(cx);
    cx.update(|_, cx| {
        assert_eq!(*view.read(cx).picked.borrow(), vec!["Manage"]);
        assert!(!view.read(cx).open);
    });
}

#[gpui::test]
fn the_keyboard_opens_a_callers_menu_and_chooses_in_it(cx: &mut TestAppContext) {
    let (view, cx) = rich_dropdown(true, cx);
    draw(cx);
    let trigger = cx.debug_bounds("dropdown-trigger").unwrap();
    assert_eq!(trigger.size.width, px(360.), "a full-width field");
    cx.update(|window, cx| window.focus_next(cx));
    draw(cx);
    cx.simulate_keystrokes("down");
    settle(cx);
    cx.update(|_, cx| assert!(view.read(cx).open));
    assert_eq!(
        cx.debug_bounds("material-menu").unwrap().size.width,
        px(360.),
        "a full-width field's menu is as wide as the field"
    );
    // Down opens at the selected item; one more moves to the action.
    cx.simulate_keystrokes("down enter");
    settle(cx);
    cx.update(|_, cx| {
        assert_eq!(*view.read(cx).picked.borrow(), vec!["Manage"]);
        assert!(!view.read(cx).open);
    });
}

#[test]
fn a_dropdowns_value_text_overrides_its_selected_option() {
    let dropdown = ExposedDropdown::new("dropdown", "Fruit")
        .options(["Apple", "Banana"])
        .selected(Some(1));
    assert_eq!(dropdown.value_text(), "Banana");
    assert_eq!(dropdown.value("Pear").value_text(), "Pear");
    assert_eq!(
        ExposedDropdown::new("dropdown", "Model")
            .menu(MaterialMenu::new("menu"))
            .value_text(),
        ""
    );
}

#[test]
fn options_configuration_beside_a_callers_menu_is_detected_as_inert() {
    let menu = || ExposedDropdown::new("dropdown", "Model").menu(MaterialMenu::new("menu"));
    assert!(!menu().value("Deep").mixes_menu_and_options());
    assert!(
        !ExposedDropdown::new("dropdown", "Fruit")
            .options(["Apple"])
            .on_select(|_, _, _| {})
            .mixes_menu_and_options()
    );
    assert!(menu().options(["Apple"]).mixes_menu_and_options());
    assert!(menu().selected(Some(0)).mixes_menu_and_options());
    assert!(menu().disabled_options([0]).mixes_menu_and_options());
    assert!(menu().on_select(|_, _, _| {}).mixes_menu_and_options());
    assert!(
        menu()
            .menu_style(MenuStyle::Standard)
            .mixes_menu_and_options()
    );
}

#[test]
fn a_menu_opens_on_its_side_then_the_other_then_where_there_is_more_room() {
    let side = |asks_above, need: f32, room| popup_side(asks_above, (need, need), room, 60.);
    // Below by default: it fits below, so it stays there.
    assert_eq!(side(false, 300., (400., 300.)), Some(false));
    // It fits only above.
    assert_eq!(side(false, 300., (300., 299.)), Some(true));
    // It fits on neither side: the side with more room, where it scrolls.
    assert_eq!(side(false, 300., (250., 200.)), Some(true));
    assert_eq!(side(false, 300., (200., 250.)), Some(false));
    assert_eq!(
        side(false, 300., (200., 200.)),
        Some(false),
        "a tie keeps its side"
    );
    // Asked to open above, the same rules from the other side.
    assert_eq!(side(true, 300., (300., 400.)), Some(true));
    assert_eq!(side(true, 300., (299., 300.)), Some(false));
    assert_eq!(side(true, 300., (200., 200.)), Some(true));
    // Less than a row on both sides: no side, and the window's room instead.
    assert_eq!(side(false, 300., (40., 50.)), None);
    // A menu shorter than a row still opens where it fits.
    assert_eq!(side(false, 30., (0., 40.)), Some(false));
    // A sub-menu's column decides with the menu: here it fits only below.
    assert_eq!(
        popup_side(false, (500., 150.), (300., 200.), 60.),
        Some(false)
    );
    assert_eq!(
        popup_side(false, (150., 500.), (300., 200.), 60.),
        Some(true)
    );
}

/// A six-option dropdown in a short window, its field `top` from the window's top.
struct ShortWindowDropdown {
    top: f32,
}

impl Render for ShortWindowDropdown {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div().pt(px(self.top)).pl(px(20.)).child(
            ExposedDropdown::new("dropdown", "Fruit")
                .options(["A", "B", "C", "D", "E", "F"])
                .open(true),
        )
    }
}

fn short_window_dropdown(
    top: f32,
    cx: &mut TestAppContext,
) -> (Bounds<Pixels>, Bounds<Pixels>, Pixels) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(move |_, _| ShortWindowDropdown { top });
    cx.simulate_resize(size(px(800.), px(620.)));
    settle(cx);
    let viewport = cx.update(|window, _| window.viewport_size());
    (
        cx.debug_bounds("dropdown-trigger").unwrap(),
        cx.debug_bounds("menu-popup-surface").unwrap(),
        viewport.height,
    )
}

#[gpui::test]
fn a_dropdown_with_no_room_below_opens_above_its_field(cx: &mut TestAppContext) {
    // 6 × 56 + 2 × 8 = 352dp of menu: 172dp below the 56dp field, 368dp above it.
    let (field, surface, _) = short_window_dropdown(380., cx);
    assert_eq!(
        surface.size.height,
        px(6. * 56. + 16.),
        "whole, not scrolled"
    );
    assert_eq!(
        surface.bottom(),
        field.top() - px(4.),
        "the menu at {surface:?} opens above the field at {field:?}"
    );
}

#[gpui::test]
fn a_dropdown_that_fits_nowhere_scrolls_on_the_roomier_side_off_its_field(cx: &mut TestAppContext) {
    // A field near the middle of the window has less than the menu's 352dp on either side:
    // 258dp above it and 282dp below.
    let (field, surface, height) = short_window_dropdown(270., cx);
    assert!(
        surface.top() >= field.bottom() + px(4.) || surface.bottom() <= field.top() - px(4.),
        "the menu at {surface:?} covers the field at {field:?}"
    );
    assert!(
        surface.top() >= field.bottom(),
        "below, where there is more room"
    );
    assert!(surface.size.height < px(6. * 56. + 16.), "it scrolls");
    assert!(surface.top() >= px(8.) && surface.bottom() <= height - px(8.));
}

#[gpui::test]
fn a_menu_turned_above_its_trigger_cascades_upward_too(cx: &mut TestAppContext) {
    let (view, cx) = cascade(cx, Anchor::TopLeft);
    // The trigger spans 200..240 of a 300 tall window: no room below it.
    cx.simulate_resize(size(px(1000.), px(300.)));
    settle(cx);
    cx.simulate_keystrokes("down down right");
    settle(cx);
    cx.update(|_, cx| assert!(view.read(cx).submenu));
    let trigger = cx.debug_bounds("trigger").unwrap();
    let surface = cx.debug_bounds("menu-popup-surface").unwrap();
    let submenu = cx.debug_bounds("menu-popup-submenu").unwrap();
    assert_eq!(surface.bottom(), trigger.top() - px(4.), "above");
    // Above the trigger a sub-menu's last item lines up with its owner's bottom,
    // and "Share" is the menu's last item.
    assert_eq!(submenu.bottom(), surface.bottom());
    assert_eq!(submenu.left(), surface.right() + px(4.));
}

/// A trigger as tall as the window: there is no room beside it.
struct WindowTallTrigger;

impl Render for WindowTallTrigger {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let menu = MaterialMenu::new("menu").items((0..3).map(|_| MaterialMenuItem::new("Item")));
        MenuPopup::new(
            "popup",
            div()
                .debug_selector(|| "trigger".into())
                .w(px(100.))
                .h(px(600.)),
            menu,
        )
        .open(true)
    }
}

#[gpui::test]
fn a_trigger_with_no_room_beside_it_keeps_its_menu_whole(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(|_, _| WindowTallTrigger);
    cx.simulate_resize(size(px(800.), px(600.)));
    settle(cx);
    let surface = cx.debug_bounds("menu-popup-surface").unwrap();
    assert_eq!(
        surface.size.height,
        px(3. * 48. + 4.),
        "whole, inside the window"
    );
    assert!(surface.top() >= px(8.) && surface.bottom() <= px(592.));
}

/// A three-item menu whose last item owns a sub-menu of `items`, `top` from the
/// window's top.
struct TallSubmenu {
    top: f32,
    items: usize,
}

impl Render for TallSubmenu {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let menu = MaterialMenu::new("menu")
            .items((0..3).map(|ix| MaterialMenuItem::new("Item").submenu(ix == 2)));
        let submenu = MaterialMenu::new("submenu")
            .items((0..self.items).map(|_| MaterialMenuItem::new("Sub")));
        div().pt(px(self.top)).pl(px(20.)).child(
            MenuPopup::new(
                "popup",
                div()
                    .debug_selector(|| "trigger".into())
                    .w(px(100.))
                    .h(px(40.)),
                menu,
            )
            .open(true)
            .submenu(2, submenu, true),
        )
    }
}

fn tall_submenu(
    top: f32,
    items: usize,
    cx: &mut TestAppContext,
) -> (Bounds<Pixels>, Bounds<Pixels>, Bounds<Pixels>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(move |_, _| TallSubmenu { top, items });
    cx.simulate_resize(size(px(800.), px(600.)));
    settle(cx);
    (
        cx.debug_bounds("trigger").unwrap(),
        cx.debug_bounds("menu-popup-surface").unwrap(),
        cx.debug_bounds("menu-popup-submenu").unwrap(),
    )
}

fn assert_off_the_trigger(trigger: Bounds<Pixels>, popup: Bounds<Pixels>, what: &str) {
    assert!(
        popup.top() >= trigger.bottom() + px(4.) || popup.bottom() <= trigger.top() - px(4.),
        "the {what} at {popup:?} covers the trigger at {trigger:?}"
    );
    assert!(
        popup.top() >= px(8.) && popup.bottom() <= px(592.),
        "{what} {popup:?}"
    );
}

#[gpui::test]
fn a_tall_submenu_near_the_bottom_turns_the_menu_above_its_trigger(cx: &mut TestAppContext) {
    let (trigger, surface, submenu) = tall_submenu(540., 12, cx);
    assert_eq!(surface.bottom(), trigger.top() - px(4.), "above");
    assert_off_the_trigger(trigger, surface, "menu");
    assert_off_the_trigger(trigger, submenu, "sub-menu");
}

#[gpui::test]
fn a_tall_submenu_that_fits_nowhere_scrolls_beside_its_menu(cx: &mut TestAppContext) {
    let (trigger, surface, submenu) = tall_submenu(200., 12, cx);
    assert_eq!(
        surface.top(),
        trigger.bottom() + px(4.),
        "below, where there is more room"
    );
    assert_off_the_trigger(trigger, submenu, "sub-menu");
}

/// An open menu whose trigger moves to `top`.
struct MovingTrigger {
    top: f32,
}

impl Render for MovingTrigger {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        let menu = MaterialMenu::new("menu").items((0..3).map(|_| MaterialMenuItem::new("Item")));
        div().pt(px(self.top)).child(
            MenuPopup::new(
                "popup",
                div().debug_selector(|| "trigger".into()).size(px(40.)),
                menu,
            )
            .open(true),
        )
    }
}

#[gpui::test]
fn a_moved_trigger_chooses_its_menus_side_again_on_the_next_frame(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|_, _| MovingTrigger { top: 20. });
    cx.simulate_resize(size(px(800.), px(600.)));
    settle(cx);
    // Nothing is left waiting for a frame.
    while cx.update(|window, cx| window.simulate_next_frame(cx)) > 0 {
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }
    view.update(cx, |view, cx| {
        view.top = 500.;
        cx.notify();
    });
    // One frame lays the trigger out where it now is, from the side chosen before.
    cx.update(|window, cx| window.draw(cx).clear(cx));
    // The frame it asks for chooses again; nothing else redraws it.
    cx.update(|window, cx| {
        window.simulate_next_frame(cx);
        window.draw(cx).clear(cx);
    });
    let trigger = cx.debug_bounds("trigger").unwrap();
    let surface = cx.debug_bounds("menu-popup-surface").unwrap();
    assert_eq!(
        surface.bottom(),
        trigger.top() - px(4.),
        "above the moved trigger"
    );
}

#[gpui::test]
fn a_submenu_that_fits_only_above_turns_its_menu_above(cx: &mut TestAppContext) {
    // The 148dp menu fits in the 228dp below its trigger, but the sub-menu's column, from
    // the last item's top (96dp) plus its 292dp, does not; above, from that item's bottom,
    // the 292dp fit in 308dp.
    let (trigger, surface, submenu) = tall_submenu(320., 6, cx);
    assert_eq!(surface.bottom(), trigger.top() - px(4.), "above");
    assert_eq!(
        submenu.size.height,
        px(6. * 48. + 4.),
        "whole, not scrolled"
    );
    assert_off_the_trigger(trigger, submenu, "sub-menu");
}

#[gpui::test]
fn a_trigger_below_the_window_keeps_its_submenu_inside_it(cx: &mut TestAppContext) {
    let (_, surface, submenu) = tall_submenu(640., 14, cx);
    assert!(
        surface.top() >= px(8.) && surface.bottom() <= px(592.),
        "menu {surface:?}"
    );
    assert!(
        submenu.top() >= px(8.) && submenu.bottom() <= px(592.),
        "sub-menu {submenu:?}"
    );
}
