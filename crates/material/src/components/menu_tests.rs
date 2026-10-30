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
    assert_eq!(menu_height(MenuStyle::Baseline, d, &baseline), 688.);
    // Three Expressive groups of four, the first labelled: 2dp gaps.
    let groups = vec![
        (true, vec![(false, false); 4]),
        (false, vec![(false, false); 4]),
        (false, vec![(false, false); 4]),
    ];
    assert_eq!(
        menu_height(MenuStyle::Vibrant, d, &groups),
        3. * (4. + 4. * 48.) + 32. + 2. * 2.
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
}

impl Render for DropdownHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div().p(px(20.)).child(
            ExposedDropdown::new("dropdown", "Fruit")
                .options(["Apple", "Banana", "Cherry"])
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
