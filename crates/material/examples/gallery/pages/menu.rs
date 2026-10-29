//! Menu: the kit's M3 Expressive variants (standard and vibrant, one to
//! three groups) and item building blocks, the baseline menu at each density
//! with its building blocks, and live menus: exposed dropdowns, a button
//! menu with shortcuts and a sub-menu indicator, and a vibrant icon-button
//! menu with groups and selection.
use super::prelude::*;
use super::text_fields::symbol;

const STATES: [InteractionPreview; 5] = InteractionPreview::ALL;
const FRUITS: [&str; 5] = ["Apple", "Banana", "Cherry", "Grape", "Mango"];

pub struct Page {
    edit_open: bool,
    view_open: bool,
    view_mode: usize,
    dropdown_open: [bool; 2],
    dropdown_selected: [Option<usize>; 2],
    last_action: Option<&'static str>,
}

/// The kit's `stars` placeholder at the menu icon size, filled when selected.
fn star(size: f32, filled: bool) -> impl IntoElement {
    symbol("stars", filled).size(rem(size))
}

/// A menu at its natural size inside a board cell (the popup reveals it).
fn variant_menu(id: impl Into<ElementId>, style: MenuStyle, groups: usize) -> MaterialMenu {
    let item = |_: usize| {
        MaterialMenuItem::new("Label")
            .leading(star(20., false))
            .submenu(true)
    };
    let mut menu = MaterialMenu::new(id).style(style);
    match groups {
        1 => menu = menu.items((0..6).map(item)),
        n => {
            for _ in 0..n {
                menu = menu.group(MenuGroup::new().items((0..4).map(item)));
            }
        }
    }
    menu
}

fn scrolling(id: &'static str, width: f32, content: impl IntoElement) -> AnyElement {
    div()
        .id(id)
        .w_full()
        .overflow_x_scroll()
        .child(div().w(rem(width)).child(content))
        .into_any_element()
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "menu",
        title: "Menu",
        description: "Menus display a list of choices on a temporary surface.",
        guideline: Some("https://m3.material.io/components/menus/overview"),
    };

    fn new(_: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            edit_open: false,
            view_open: false,
            view_mode: 1,
            dropdown_open: [false; 2],
            dropdown_selected: [Some(2), None],
            last_action: None,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        7
    }

    fn interactive(&self, row: usize) -> bool {
        row >= 5
    }

    /// `--active` opens the live menus so captures show the popups.
    fn apply_capture(&mut self, state: CaptureState, cx: &mut Context<Self>) {
        if state.active {
            self.edit_open = true;
            self.dropdown_open[0] = true;
            cx.notify();
        }
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match row {
            0 => scrolling(
                "menu-variants-scroll",
                6. * 208. + 5. * 16. + 2. * 20. + 2.,
                board("menu-variants")
                    .caption("Basic variants")
                    .columns(6)
                    .column_width(208.)
                    .column_labels([
                        "Standard",
                        "Standard · 2 groups",
                        "Standard · 3 groups",
                        "Vibrant",
                        "Vibrant · 2 groups",
                        "Vibrant · 3 groups",
                    ])
                    .cells(
                        [MenuStyle::Standard, MenuStyle::Vibrant]
                            .into_iter()
                            .flat_map(|style| {
                                (1..=3).map(move |groups| {
                                    div().h(rem(3. * 196. + 8.)).child(variant_menu(
                                        ("variant-menu", groups + 3 * (style as usize)),
                                        style,
                                        groups,
                                    ))
                                })
                            }),
                    ),
            ),
            1 => expressive_blocks(),
            2 => item_content_board(),
            3 => scrolling(
                "menu-baseline-scroll",
                3. * 200. + 2. * 16. + 2. * 20. + 2.,
                board("menu-baseline")
                    .caption("Baseline menu")
                    .columns(3)
                    .column_width(200.)
                    .column_labels(["Density 0", "Density −2", "Density −4"])
                    .cells(
                        MenuDensity::ALL
                            .into_iter()
                            .enumerate()
                            .map(|(ix, density)| {
                                div().h(rem(700.)).child(
                                    MaterialMenu::new(("baseline-menu", ix))
                                        .style(MenuStyle::Baseline)
                                        .density(density)
                                        .items((0..12).map(|item| {
                                            MaterialMenuItem::new("Menu item").selected(item == 9)
                                        })),
                                )
                            }),
                    ),
            ),
            4 => baseline_blocks(),
            5 => self.live_menus(cx),
            _ => self.live_dropdowns(cx),
        }
    }
}

impl Page {
    fn live_menus(&self, cx: &mut Context<Self>) -> AnyElement {
        let status = self
            .last_action
            .map(|action| format!("Last action: {action}"))
            .unwrap_or_else(|| "Choose a command".to_owned());
        let edit_menu = MaterialMenu::new("edit-menu")
            .group(
                MenuGroup::new()
                    .item(
                        MaterialMenuItem::new("Cut")
                            .leading(symbol("content_cut", false).size(rem(20.)))
                            .trailing_text("⌘X")
                            .on_click(act("Cut", cx)),
                    )
                    .item(
                        MaterialMenuItem::new("Copy")
                            .leading(symbol("content_copy", false).size(rem(20.)))
                            .trailing_text("⌘C")
                            .on_click(act("Copy", cx)),
                    )
                    .item(
                        MaterialMenuItem::new("Paste")
                            .leading(symbol("content_paste", false).size(rem(20.)))
                            .trailing_text("⌘V")
                            .disabled(true),
                    ),
            )
            .group(
                MenuGroup::new()
                    .item(
                        MaterialMenuItem::new("Share")
                            .supporting_text("Send a copy")
                            .leading(symbol("share", false).size(rem(20.)))
                            .submenu(true)
                            .on_click(act("Share", cx)),
                    )
                    .item(
                        MaterialMenuItem::new("Delete")
                            .leading(symbol("delete", false).size(rem(20.)))
                            .on_click(act("Delete", cx)),
                    ),
            )
            .width(240.);
        let edit_open = self.edit_open;
        let edit = MenuPopup::new(
            "edit-popup",
            material_button("edit-trigger", ButtonVariant::Tonal)
                .child("Edit")
                .on_click(cx.listener(move |this, event: &ClickEvent, _, cx| {
                    // Pointer presses reach the popover; keys arrive here.
                    if matches!(event, ClickEvent::Keyboard(_)) {
                        this.edit_open = !edit_open;
                        cx.notify();
                    }
                })),
            edit_menu,
        )
        .open(self.edit_open)
        .on_open_change(cx.listener(|this, open: &bool, _, cx| {
            this.edit_open = *open;
            cx.notify();
        }));

        let modes = ["List", "Grid", "Gallery"];
        let view_menu = MaterialMenu::new("view-menu")
            .style(MenuStyle::Vibrant)
            .group(
                MenuGroup::new()
                    .label("Layout")
                    .items(modes.into_iter().enumerate().map(|(ix, mode)| {
                        MaterialMenuItem::new(mode)
                            .selected(self.view_mode == ix)
                            .trailing(symbol("check", false).size(rem(20.)))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.view_mode = ix;
                                cx.notify();
                            }))
                    })),
            )
            .group(
                MenuGroup::new()
                    .item(
                        MaterialMenuItem::new("Sort")
                            .submenu(true)
                            .on_click(act("Sort", cx)),
                    )
                    .item(
                        MaterialMenuItem::new("Filter")
                            .submenu(true)
                            .on_click(act("Filter", cx)),
                    ),
            );
        let view_open = self.view_open;
        let view = MenuPopup::new(
            "view-popup",
            material_button("view-trigger", ButtonVariant::Filled)
                .icon(symbol("more_vert", false))
                .accessibility_label("View options")
                .on_click(cx.listener(move |this, event: &ClickEvent, _, cx| {
                    if matches!(event, ClickEvent::Keyboard(_)) {
                        this.view_open = !view_open;
                        cx.notify();
                    }
                })),
            view_menu,
        )
        .open(self.view_open)
        .on_open_change(cx.listener(|this, open: &bool, _, cx| {
            this.view_open = *open;
            cx.notify();
        }));
        spec::demo(
            "live-menus",
            "Menus in use",
            div()
                .flex()
                .flex_col()
                .gap(rem(12.))
                .h(rem(380.))
                .child(accessible_text("menu-status", status))
                .child(spec::demo_row().items_start().child(edit).child(view)),
        )
        .into_any_element()
    }

    fn live_dropdowns(&self, cx: &mut Context<Self>) -> AnyElement {
        let dropdown = |ix: usize, variant: TextFieldVariant, cx: &mut Context<Self>| {
            ExposedDropdown::new(("fruit-dropdown", ix), "Fruit")
                .variant(variant)
                .options(FRUITS)
                .selected(self.dropdown_selected[ix])
                .open(self.dropdown_open[ix])
                .supporting_text("Exposed dropdown menu")
                .on_open_change(cx.listener(move |this, open: &bool, _, cx| {
                    this.dropdown_open[ix] = *open;
                    cx.notify();
                }))
                .on_select(cx.listener(move |this, option: &usize, _, cx| {
                    this.dropdown_selected[ix] = Some(*option);
                    cx.notify();
                }))
        };
        spec::demo(
            "live-dropdowns",
            "Exposed dropdown menus",
            div().h(rem(360.)).child(
                spec::demo_row()
                    .items_start()
                    .child(dropdown(0, TextFieldVariant::Filled, cx))
                    .child(
                        dropdown(1, TextFieldVariant::Outlined, cx)
                            .leading_icon(symbol("nutrition", false)),
                    )
                    .child(
                        ExposedDropdown::new("fruit-disabled", "Fruit")
                            .options(FRUITS)
                            .selected(Some(0))
                            .disabled(true)
                            .supporting_text("Disabled"),
                    ),
            ),
        )
        .into_any_element()
    }
}

/// Expressive item building blocks: style × selection columns, state rows.
fn expressive_blocks() -> AnyElement {
    let columns = [
        (MenuStyle::Standard, false),
        (MenuStyle::Standard, true),
        (MenuStyle::Vibrant, false),
        (MenuStyle::Vibrant, true),
    ];
    let cells = STATES
        .into_iter()
        .enumerate()
        .flat_map(move |(row, state)| {
            columns
                .into_iter()
                .enumerate()
                .map(move |(col, (style, selected))| {
                    MaterialMenu::new(("expressive-block", row * 4 + col))
                        .style(style)
                        .elevated(false)
                        .width(228.)
                        .item(
                            MaterialMenuItem::new("Label")
                                .leading(star(20., selected))
                                .submenu(true)
                                .selected(selected)
                                .preview_state(state),
                        )
                })
        });
    scrolling(
        "menu-blocks-scroll",
        88. + 4. * 228. + 3. * 16. + 2. * 20. + 2.,
        board("menu-expressive-blocks")
            .caption("Building blocks · menu item")
            .columns(4)
            .column_width(228.)
            .column_labels([
                "Standard",
                "Standard selected",
                "Vibrant",
                "Vibrant selected",
            ])
            .row_labels(STATES.map(InteractionPreview::name))
            .cells(cells),
    )
}

/// Leading and trailing elements, supporting text and dividers.
fn item_content_board() -> AnyElement {
    let menus = [
        MaterialMenu::new("content-shortcut")
            .elevated(false)
            .width(228.)
            .item(
                MaterialMenuItem::new("Copy")
                    .leading(symbol("content_copy", false).size(rem(20.)))
                    .trailing_text("⌘C"),
            ),
        MaterialMenu::new("content-supporting")
            .elevated(false)
            .width(228.)
            .item(
                MaterialMenuItem::new("Label")
                    .supporting_text("Supporting text")
                    .leading(star(20., false))
                    .trailing_text("⌘C"),
            ),
        MaterialMenu::new("content-divider")
            .elevated(false)
            .width(228.)
            .item(
                MaterialMenuItem::new("Label")
                    .leading(star(20., false))
                    .divider(true),
            )
            .item(MaterialMenuItem::new("Label").leading(star(20., false))),
    ];
    scrolling(
        "menu-content-scroll",
        3. * 228. + 2. * 16. + 2. * 20. + 2.,
        board("menu-item-content")
            .caption("Building blocks · elements")
            .columns(3)
            .column_width(228.)
            .column_labels(["Trailing text", "Supporting text", "Divider"])
            .cells(menus),
    )
}

/// Baseline item building blocks in each state.
fn baseline_blocks() -> AnyElement {
    let cells = STATES.into_iter().enumerate().flat_map(|(row, state)| {
        [false, true].into_iter().map(move |selected| {
            MaterialMenu::new(("baseline-block", row * 2 + selected as usize))
                .style(MenuStyle::Baseline)
                .elevated(false)
                .item(
                    MaterialMenuItem::new("Menu item")
                        .leading(symbol("content_cut", false))
                        .submenu(true)
                        .selected(selected)
                        .divider(true)
                        .preview_state(state),
                )
        })
    });
    scrolling(
        "menu-baseline-blocks-scroll",
        88. + 2. * 200. + 16. + 2. * 20. + 2.,
        board("menu-baseline-blocks")
            .caption("Baseline building blocks · menu list item")
            .columns(2)
            .column_width(200.)
            .column_labels(["Unselected", "Selected"])
            .row_labels(STATES.map(InteractionPreview::name))
            .cells(cells),
    )
}

/// Record a command in the page's status line.
fn act(
    label: &'static str,
    cx: &mut Context<Page>,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static + use<> {
    let page = cx.entity().downgrade();
    move |_, _, cx| {
        page.update(cx, |this, cx| {
            this.last_action = Some(label);
            cx.notify();
        })
        .ok();
    }
}
