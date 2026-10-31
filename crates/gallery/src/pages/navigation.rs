//! Navigation: the kit's navigation bar and navigation rail sections — bars
//! with vertical and horizontal items, collapsed and expanded rails (standard
//! and modal, top and middle aligned), every item building block, and live
//! bar and rail demos. The kit's deprecated drawer and XR sections are not
//! mirrored; the library's drawer and navigation suite have boards of their
//! own at the end, for those components' Specs tabs.
use super::prelude::*;
use gpui_kit_material::icons::Symbol;
use std::sync::OnceLock;

const RAIL: PageInfo = PageInfo {
    id: "navigation-rail",
    title: "Navigation rail",
    description: "Navigation rails provide access to primary destinations in your app, particularly in tablet and desktop screens.",
    guideline: Some("https://m3.material.io/components/navigation-rail/overview"),
};

/// The kit's star placeholder as a destination icon, outlined and filled.
fn star() -> NavIcon {
    static STAR: OnceLock<NavIcon> = OnceLock::new();
    STAR.get_or_init(|| {
        NavIcon::symbol(Symbol::named("stars").expect("stars is a Material Symbol"))
    })
    .clone()
}

fn symbol_icon(name: &'static str) -> NavIcon {
    NavIcon::symbol(Symbol::named(name).expect("a Material Symbol"))
}

/// The live demos' destinations.
fn demo_items() -> &'static [NavItem] {
    static ITEMS: OnceLock<Vec<NavItem>> = OnceLock::new();
    ITEMS.get_or_init(|| {
        vec![
            NavItem::new("home", "Home", symbol_icon("home")),
            NavItem::new("inbox", "Inbox", symbol_icon("inbox")).badge(NavBadge::Large("3".into())),
            NavItem::new("explore", "Explore", symbol_icon("explore")),
            NavItem::new("saved", "Saved", symbol_icon("bookmark")).badge(NavBadge::Small),
            NavItem::new("settings", "Settings", symbol_icon("settings")),
        ]
    })
}

/// `count` kit specimen destinations ("Label" with the star), the first
/// selected, as static specimens outside the tab order.
fn specimen_items(count: usize) -> impl Iterator<Item = NavItem> {
    (0..count).map(|index| {
        NavItem::new(SharedString::from(format!("item-{index}")), "Label", star())
            .preview_state(InteractionPreview::Enabled)
    })
}

type BadgeSpec = (&'static str, fn() -> NavBadge);
const BADGES: [BadgeSpec; 3] = [
    ("No badge", || NavBadge::None),
    ("Small badge", || NavBadge::Small),
    ("Large badge", || NavBadge::Large("3".into())),
];
const ITEM_STATES: [InteractionPreview; 4] = [
    InteractionPreview::Enabled,
    InteractionPreview::Hovered,
    InteractionPreview::Focused,
    InteractionPreview::Pressed,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    BarVertical,
    BarHorizontal,
    /// One badge kind of the vertical item board: unselected and selected.
    BarVerticalItems(usize),
    BarHorizontalItems(usize),
    BarDemo,
    RailSection,
    RailCollapsed,
    /// Top-aligned (0) or middle-aligned (1) expanded rails.
    RailExpanded(usize),
    /// One badge kind of the rail's vertical item board.
    RailVerticalItems(usize),
    RailExpandedItems(usize),
    RailDemo,
    Drawer,
    Suite,
}

const ROWS: [Row; 17] = [
    Row::BarVertical,
    Row::BarHorizontal,
    Row::BarVerticalItems(0),
    Row::BarVerticalItems(1),
    Row::BarVerticalItems(2),
    Row::BarHorizontalItems(0),
    Row::BarHorizontalItems(1),
    Row::BarHorizontalItems(2),
    Row::BarDemo,
    Row::RailSection,
    Row::RailCollapsed,
    Row::RailExpanded(0),
    Row::RailExpanded(1),
    Row::RailVerticalItems(0),
    Row::RailVerticalItems(1),
    Row::RailVerticalItems(2),
    Row::RailExpandedItems(0),
];
const LAST_ROWS: [Row; 5] = [
    Row::RailExpandedItems(1),
    Row::RailExpandedItems(2),
    Row::RailDemo,
    Row::Drawer,
    Row::Suite,
];

fn rows_where(keep: fn(Row) -> bool) -> Vec<usize> {
    (0..ROWS.len() + LAST_ROWS.len())
        .filter(|index| keep(row(*index)))
        .collect()
}
/// The navigation bar's rows on its Specs tab.
pub fn bar_rows() -> Vec<usize> {
    rows_where(|row| {
        matches!(
            row,
            Row::BarVertical
                | Row::BarHorizontal
                | Row::BarVerticalItems(_)
                | Row::BarHorizontalItems(_)
                | Row::BarDemo
        )
    })
}
/// The navigation rail's rows on its Specs tab, without the page's rail heading.
pub fn rail_rows() -> Vec<usize> {
    rows_where(|row| {
        matches!(
            row,
            Row::RailCollapsed
                | Row::RailExpanded(_)
                | Row::RailVerticalItems(_)
                | Row::RailExpandedItems(_)
                | Row::RailDemo
        )
    })
}
/// The navigation drawer's rows on its Specs tab.
pub fn drawer_rows() -> Vec<usize> {
    rows_where(|row| matches!(row, Row::Drawer))
}
/// The navigation suite's rows on its Specs tab.
pub fn suite_rows() -> Vec<usize> {
    rows_where(|row| matches!(row, Row::Suite))
}

/// The drawer open beside (standard) and over (modal) its content.
fn drawer_board() -> AnyElement {
    const WIDTH: f32 = 280.;
    let cells = [false, true].map(|modal| {
        let drawer = NavigationDrawer::new(("drawer-specimen", usize::from(modal)), "Mail")
            .headline("Mail")
            .modal(modal)
            .open(true)
            .focus_on_open(false)
            .width(WIDTH)
            .selected("home")
            .section(3, "Library")
            .items(demo_items().iter().cloned());
        div()
            .relative()
            .flex()
            .w(rem(WIDTH + 40.))
            .h(rem(420.))
            .overflow_hidden()
            .rounded(rem(16.))
            .border_1()
            .border_color(color(ColorRole::OUTLINE_VARIANT))
            .bg(color(ColorRole::SURFACE))
            .child(drawer)
    });
    board("navigation-drawer")
        .caption("Navigation drawer")
        .columns(2)
        .column_width(WIDTH + 40.)
        .column_labels(["Standard", "Modal"])
        .cells(cells)
        .into_any_element()
}

/// The suite's four forms, each fixed: the form a window's width selects.
fn suite_board() -> AnyElement {
    const FORMS: [(NavigationSuiteType, &str, f32); 4] = [
        (NavigationSuiteType::BarCompact, "Bar", 280.),
        (NavigationSuiteType::RailCollapsed, "Rail", 280.),
        (NavigationSuiteType::RailExpanded, "Expanded rail", 400.),
        (NavigationSuiteType::Drawer, "Drawer", 400.),
    ];
    board("navigation-suite")
        .caption("Navigation suite")
        .columns(2)
        .column_width(400.)
        .column_labels(["Compact and medium", "Expanded"])
        .cells(
            FORMS
                .into_iter()
                .enumerate()
                .map(|(ix, (form, name, width))| {
                    div()
                        .w(rem(width))
                        .h(rem(320.))
                        .overflow_hidden()
                        .rounded(rem(16.))
                        .border_1()
                        .border_color(color(ColorRole::OUTLINE_VARIANT))
                        .child(
                            NavigationSuite::new(("suite-specimen", ix), "Main")
                                .items(demo_items().iter().cloned())
                                .selected("home")
                                .layout_type(Some(form))
                                .child(
                                    div()
                                        .size_full()
                                        .p(rem(16.))
                                        .bg(color(ColorRole::SURFACE))
                                        .map(|text| spec::type_role(text, TypeRole::TitleMedium))
                                        .text_color(color(ColorRole::ON_SURFACE))
                                        .child(name),
                                ),
                        )
                }),
        )
        .into_any_element()
}

fn row(index: usize) -> Row {
    ROWS.iter()
        .chain(LAST_ROWS.iter())
        .nth(index)
        .copied()
        .expect("a Navigation page row")
}

pub struct Page {
    page: PageCx,
    bar_selected: SharedString,
    rail_selected: SharedString,
    rail_expanded: bool,
    rail_modal: bool,
}

/// A board of single items: rows unselected/selected (and, for the rail,
/// labelled/unlabelled) of one badge kind, columns the interaction states.
fn item_board(
    id: (&'static str, usize),
    caption: &'static str,
    badge: usize,
    layout: NavItemLayout,
    labels: &'static [bool],
) -> AnyElement {
    let (badge_name, make_badge) = BADGES[badge];
    let rows: Vec<(bool, bool)> = labels
        .iter()
        .flat_map(|label| [(*label, false), (*label, true)])
        .collect();
    let row_labels: Vec<SharedString> = rows
        .iter()
        .map(|(label, selected)| {
            SharedString::from(format!(
                "{}{}",
                if *selected { "Selected" } else { "Unselected" },
                if *label { "" } else { "\nno label" }
            ))
        })
        .collect();
    board(id)
        .band(Band::of(badge, BADGES.len()))
        .caption(caption)
        .side_label(badge_name)
        .columns(ITEM_STATES.len())
        .column_width(if layout == NavItemLayout::Expanded {
            150.
        } else {
            104.
        })
        .when(badge == 0, |board| {
            board.column_labels(ITEM_STATES.map(InteractionPreview::name))
        })
        .row_labels(row_labels)
        .cells(
            rows.into_iter()
                .enumerate()
                .flat_map(move |(row, (label, selected))| {
                    ITEM_STATES
                        .into_iter()
                        .enumerate()
                        .map(move |(column, state)| {
                            NavigationItem::new(
                                (id.0, (badge * 8 + row) * 4 + column),
                                NavItem::new("item", "Label", star())
                                    .badge(make_badge())
                                    .preview_state(state),
                            )
                            .layout(layout)
                            .selected(selected)
                            .show_label(label)
                        })
                }),
        )
        .into_any_element()
}

fn specimen_menu(expanded: bool) -> impl IntoElement {
    div()
        .size(rem(48.))
        .flex()
        .items_center()
        .justify_center()
        .child(
            menu_icon(expanded)
                .size(rem(24.))
                .color(color(ColorRole::ON_SURFACE_VARIANT)),
        )
}

/// The rail's menu button: a standard 40dp icon button in a 48dp target.
fn menu_button(
    id: &'static str,
    expanded: bool,
    label: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let ink = color(ColorRole::ON_SURFACE_VARIANT);
    div()
        .size(rem(48.))
        .flex()
        .items_center()
        .justify_center()
        .child(FocusRing::new(
            (id, 0usize),
            gpui_base::Button::new(id)
                .accessibility_label(label)
                .size(rem(40.))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(move |style| style.bg(ink.opacity(0.08)))
                .active(move |style| style.bg(ink.opacity(0.10)))
                .child(menu_icon(expanded).size(rem(24.)).color(ink))
                .on_click(on_click),
        ))
}

/// The rail's FAB (collapsed) or extended FAB (expanded): primary container,
/// 56dp, large corners.
fn rail_fab(id: &'static str, expanded: bool, label: &'static str) -> impl IntoElement {
    let ink = color(ColorRole::ON_PRIMARY_CONTAINER);
    let container = color(ColorRole::PRIMARY_CONTAINER);
    FocusRing::new(
        (id, 0usize),
        gpui_base::Button::new(id)
            .accessibility_label(label)
            .h(rem(56.))
            .min_w(rem(56.))
            .when(expanded, |fab| fab.px(rem(16.)).gap(rem(12.)))
            .rounded(rem(16.))
            .bg(container)
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(move |style| style.bg(state_layer(container, ink, 0.08)))
            .active(move |style| style.bg(state_layer(container, ink, 0.10)))
            .text_color(ink)
            .child(edit_icon().size(rem(24.)).color(ink))
            .when(expanded, |fab| {
                fab.child(
                    div()
                        .text_size(TypeRole::TitleMedium.scale().font_size)
                        .font_weight(FontWeight::MEDIUM)
                        .child(label),
                )
            }),
    )
}

fn state_layer(container: Rgba, ink: Rgba, opacity: f32) -> Rgba {
    gpui_kit_material::tokens::state::layer(container, ink, opacity)
}

fn edit_icon() -> gpui_kit_material::icons::Icon {
    static PATH: OnceLock<String> = OnceLock::new();
    gpui_kit_material::icons::Icon::new(SharedString::new_static(
        PATH.get_or_init(|| {
            Symbol::named("edit")
                .expect("edit is a Material Symbol")
                .icon()
                .style(gpui_kit_material::icons::Style::Rounded)
                .filled(true)
                .asset_path()
        })
        .as_str(),
    ))
}

/// The menu (collapsed) or menu-open (expanded) icon of the rail's menu.
fn menu_icon(expanded: bool) -> gpui_kit_material::icons::Icon {
    static PATHS: OnceLock<[String; 2]> = OnceLock::new();
    let paths = PATHS.get_or_init(|| {
        ["menu", "menu_open"].map(|name| {
            Symbol::named(name)
                .expect("a Material Symbol")
                .icon()
                .style(gpui_kit_material::icons::Style::Rounded)
                .asset_path()
        })
    });
    gpui_kit_material::icons::Icon::new(SharedString::new_static(
        paths[usize::from(expanded)].as_str(),
    ))
}

const COLLAPSED_HEIGHT: f32 = 620.;
const EXPANDED_HEIGHT: f32 = 800.;

/// A static rail specimen from the kit's boards.
fn specimen_rail(
    id: (&'static str, usize),
    count: usize,
    expanded: bool,
    modal: bool,
    alignment: RailAlignment,
    sections: bool,
) -> impl IntoElement {
    let rail = NavigationRail::new(id, "Specimen navigation rail")
        .items(specimen_items(count))
        .selected("item-0")
        .expanded(expanded)
        .modal(modal)
        .alignment(alignment)
        .menu(specimen_menu(expanded))
        .fab(specimen_fab(expanded))
        .when(sections, |rail| {
            rail.section(5, "Section Header")
                .section(count, "Section Header")
        });
    div()
        .h(rem(if expanded {
            EXPANDED_HEIGHT
        } else {
            COLLAPSED_HEIGHT
        }))
        .when(modal, |frame| frame.rounded(rem(16.)).overflow_hidden())
        .child(rail)
}

fn specimen_fab(expanded: bool) -> impl IntoElement {
    let ink = color(ColorRole::ON_PRIMARY_CONTAINER);
    div()
        .h(rem(56.))
        .min_w(rem(56.))
        .when(expanded, |fab| fab.px(rem(16.)).gap(rem(12.)))
        .rounded(rem(16.))
        .bg(color(ColorRole::PRIMARY_CONTAINER))
        .flex()
        .items_center()
        .justify_center()
        .text_color(ink)
        .child(edit_icon().size(rem(24.)).color(ink))
        .when(expanded, |fab| {
            fab.child(
                div()
                    .text_size(TypeRole::TitleMedium.scale().font_size)
                    .font_weight(FontWeight::MEDIUM)
                    .child("Label"),
            )
        })
}

impl Page {
    fn bar_boards(&self, horizontal: bool) -> AnyElement {
        let counts: &[usize] = if horizontal {
            &[3, 4, 5, 6]
        } else {
            &[3, 4, 5]
        };
        let layout = if horizontal {
            NavItemLayout::Horizontal
        } else {
            NavItemLayout::Vertical
        };
        board(if horizontal {
            "bar-horizontal"
        } else {
            "bar-vertical"
        })
        .caption(if horizontal {
            "Navigation bar: Horizontal items"
        } else {
            "Navigation bar: Vertical items"
        })
        .columns(1)
        .cells(counts.iter().map(|count| {
            div()
                .w(rem(if horizontal {
                    560.
                } else {
                    104. * *count as f32
                }))
                .child(
                    NavigationBar::new(
                        (if horizontal { "h-bar" } else { "v-bar" }, *count),
                        "Specimen navigation bar",
                    )
                    .layout(layout)
                    .items(specimen_items(*count))
                    .selected("item-0"),
                )
        }))
        .into_any_element()
    }

    fn rail_collapsed(&self) -> AnyElement {
        board("rail-collapsed")
            .caption("Navigation Rail")
            .columns(8)
            .column_width(96.)
            .column_labels(["Top", "", "", "", "Middle", "", "", ""])
            .cells((0..8).map(|index| {
                specimen_rail(
                    ("collapsed-rail", index),
                    3 + index % 4,
                    false,
                    false,
                    if index < 4 {
                        RailAlignment::Top
                    } else {
                        RailAlignment::Middle
                    },
                    false,
                )
            }))
            .into_any_element()
    }

    fn rail_expanded(&self, band: usize) -> AnyElement {
        let alignment = if band == 0 {
            RailAlignment::Top
        } else {
            RailAlignment::Middle
        };
        board(("rail-expanded", band))
            .band(Band::of(band, 2))
            .when(band == 0, |board| {
                board.caption("Navigation Rail: Expanded")
            })
            .side_label(if band == 0 { "Top" } else { "Middle" })
            .columns(8)
            .column_width(RAIL_EXPANDED_WIDTH)
            .column_labels(["Standard", "", "", "", "Modal", "", "", ""])
            .cells((0..8).map(move |index| {
                let sections = index % 4 == 3;
                specimen_rail(
                    ("expanded-rail", band * 8 + index),
                    if sections { 9 } else { 3 + index % 4 },
                    true,
                    index >= 4,
                    alignment,
                    sections,
                )
            }))
            .into_any_element()
    }

    fn bar_demo(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let on_change = cx.listener(|this, value: &SharedString, _, cx| {
            this.bar_selected = value.clone();
            cx.notify();
        });
        let on_change = std::rc::Rc::new(on_change);
        let vertical = on_change.clone();
        spec::demo(
            "bar-demo",
            "Navigation bar in use",
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(rem(16.))
                .child(
                    div().w(rem(412.)).child(
                        NavigationBar::new("demo-bar-vertical", "Main")
                            .items(demo_items().iter().cloned())
                            .selected(self.bar_selected.clone())
                            .on_change(move |value, window, cx| vertical(value, window, cx)),
                    ),
                )
                .child(
                    div().w(rem(720.)).max_w_full().child(
                        NavigationBar::new("demo-bar-horizontal", "Main")
                            .layout(NavItemLayout::Horizontal)
                            .items(demo_items().iter().cloned())
                            .selected(self.bar_selected.clone())
                            .on_change(move |value, window, cx| on_change(value, window, cx)),
                    ),
                ),
        )
        .into_any_element()
    }

    fn rail_demo(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let expanded = self.rail_expanded && !self.rail_modal;
        let standard_select = cx.listener(|this, value: &SharedString, _, cx| {
            this.rail_selected = value.clone();
            cx.notify();
        });
        let modal_select = cx.listener(|this, value: &SharedString, _, cx| {
            this.rail_selected = value.clone();
            cx.notify();
        });
        let toggle = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.rail_expanded = !this.rail_expanded;
            this.rail_modal = false;
            cx.notify();
        });
        let open_modal = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.rail_modal = true;
            cx.notify();
        });
        let close = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.rail_modal = false;
            cx.notify();
        });
        let dismiss = cx.entity().downgrade();
        let scrim_dismiss = dismiss.clone();
        let close_modal = move |cx: &mut App, entity: &WeakEntity<Self>| {
            let _ = entity.update(cx, |this, cx| {
                this.rail_modal = false;
                cx.notify();
            });
        };
        let label = demo_items()
            .iter()
            .find(|item| item.value() == &self.rail_selected)
            .map(|item| item.label().clone())
            .unwrap_or_default();
        let standard = NavigationRail::new("demo-rail", "Main")
            .items(demo_items().iter().cloned())
            .section(3, "Library")
            .selected(self.rail_selected.clone())
            .expanded(expanded)
            .menu(menu_button(
                "demo-rail-menu",
                expanded,
                if expanded {
                    "Collapse navigation"
                } else {
                    "Expand navigation"
                },
                toggle,
            ))
            .fab(rail_fab("demo-rail-fab", expanded, "Compose"))
            .on_change(move |value, window, cx| standard_select(value, window, cx));
        let modal = self.rail_modal.then(|| {
            div()
                .absolute()
                .inset_0()
                .child(navigation_scrim("demo-rail-scrim", move |_, cx| {
                    close_modal(cx, &scrim_dismiss)
                }))
                .child(
                    div().absolute().top_0().bottom_0().left_0().child(
                        NavigationRail::new("demo-modal-rail", "Main")
                            .items(demo_items().iter().cloned())
                            .section(3, "Library")
                            .selected(self.rail_selected.clone())
                            .expanded(true)
                            .modal(true)
                            .menu(menu_button(
                                "demo-modal-menu",
                                true,
                                "Close navigation",
                                close,
                            ))
                            .fab(rail_fab("demo-modal-fab", true, "Compose"))
                            .on_change(move |value, window, cx| modal_select(value, window, cx))
                            .on_dismiss(move |_, cx| close_modal(cx, &dismiss)),
                    ),
                )
        });
        spec::demo(
            "rail-demo",
            "Navigation rail in use",
            div()
                .flex()
                .flex_col()
                .gap(rem(12.))
                .child(
                    spec::demo_row().child(
                        MaterialButton::new("open-modal-rail")
                            .variant(ButtonVariant::Tonal)
                            .child("Open modal rail")
                            .on_click(open_modal),
                    ),
                )
                .child(
                    div()
                        .relative()
                        .w(rem(720.))
                        .max_w_full()
                        .h(rem(560.))
                        .rounded(rem(16.))
                        .overflow_hidden()
                        .border_1()
                        .border_color(color(ColorRole::OUTLINE_VARIANT))
                        .flex()
                        .child(standard)
                        .child(
                            div()
                                .flex_1()
                                .h_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
                                .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                                .child(SharedString::from(format!("Destination: {label}"))),
                        )
                        .children(modal),
                ),
        )
        .into_any_element()
    }
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "navigation",
        title: "Navigation",
        description: "Navigation bars offer a persistent, convenient way to switch between primary destinations in an app. 3-5 destinations is the recommended range.",
        guideline: Some("https://m3.material.io/components/navigation-bar/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            page,
            bar_selected: "home".into(),
            rail_selected: "home".into(),
            rail_expanded: false,
            rail_modal: false,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        ROWS.len() + LAST_ROWS.len()
    }

    fn row_gap(&self, index: usize) -> f32 {
        match row(index) {
            _ if index == 0 => spec::HEADER_TO_BODY,
            Row::BarVerticalItems(band)
            | Row::BarHorizontalItems(band)
            | Row::RailVerticalItems(band)
            | Row::RailExpandedItems(band)
            | Row::RailExpanded(band) => Band::of(
                band,
                if matches!(row(index), Row::RailExpanded(_)) {
                    2
                } else {
                    3
                },
            )
            .row_gap(),
            Row::RailSection => 48.,
            _ => spec::ROW_GAP,
        }
    }

    fn interactive(&self, index: usize) -> bool {
        matches!(row(index), Row::BarDemo | Row::RailDemo | Row::RailSection)
    }

    fn render_row(&mut self, index: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        const BAR_VERTICAL: &[bool] = &[true];
        const RAIL_VERTICAL: &[bool] = &[true, false];
        match row(index) {
            Row::BarVertical => self.bar_boards(false),
            Row::BarHorizontal => self.bar_boards(true),
            Row::BarVerticalItems(badge) => item_board(
                ("bar-vertical-item", badge),
                "Building Blocks - Vertical Nav Item",
                badge,
                NavItemLayout::Vertical,
                BAR_VERTICAL,
            ),
            Row::BarHorizontalItems(badge) => item_board(
                ("bar-horizontal-item", badge),
                "Building Blocks - Horizontal Nav Item",
                badge,
                NavItemLayout::Horizontal,
                BAR_VERTICAL,
            ),
            Row::BarDemo => self.bar_demo(cx),
            Row::RailSection => spec::section_header(&RAIL).into_any_element(),
            Row::RailCollapsed => self.rail_collapsed(),
            Row::RailExpanded(band) => self.rail_expanded(band),
            Row::RailVerticalItems(badge) => item_board(
                ("rail-vertical-item", badge),
                "Building Blocks - Vertical Nav Item",
                badge,
                NavItemLayout::Vertical,
                RAIL_VERTICAL,
            ),
            Row::RailExpandedItems(badge) => item_board(
                ("rail-expanded-item", badge),
                "Building Blocks - Horizontal Nav Item",
                badge,
                NavItemLayout::Expanded,
                BAR_VERTICAL,
            ),
            Row::RailDemo => self.rail_demo(cx),
            Row::Drawer => drawer_board(),
            Row::Suite => suite_board(),
        }
    }

    fn apply_capture(&mut self, state: CaptureState, cx: &mut Context<Self>) {
        // `--selected` captures the live rail expanded.
        if state.selected {
            self.rail_expanded = true;
            let last = ROWS.len() + LAST_ROWS.len() - 1;
            self.page.rows.remeasure(last..=last);
            cx.notify();
        }
    }
}
