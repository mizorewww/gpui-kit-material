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

#[test]
fn a_closing_popup_paints_until_both_channels_settle() {
    assert!(popup_visible(true, 0., 0.));
    assert!(popup_visible(false, 0.4, 0.));
    assert!(popup_visible(false, 0., 0.2));
    assert!(!popup_visible(false, 0., 0.));
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

/// The kit draws menus anchored to a text field as baseline menus: the select's
/// selected option is on the secondary container, not the expressive tertiary.
#[test]
fn an_exposed_dropdown_opens_a_baseline_menu() {
    assert_eq!(
        ExposedDropdown::new("select", "Fruit").style,
        MenuStyle::Baseline
    );
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
