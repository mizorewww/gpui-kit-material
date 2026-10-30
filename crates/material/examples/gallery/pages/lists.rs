//! Lists: the kit's baseline lists and list-item matrix (each leading
//! element × trailing element × line count), the Expressive segmented
//! lists, list-item states, a live list, and the connected settings tiles.
use super::prelude::*;
use gpui_kit_material::icons::{Icon, Style, Symbol};
use gpui_kit_material::motion;
use std::sync::OnceLock;

/// A Material Symbol by name (rounded), resolved once.
fn symbol(index: usize) -> Icon {
    const NAMES: [&str; 3] = ["person", "arrow_right", "content_copy"];
    static PATHS: OnceLock<[String; 3]> = OnceLock::new();
    let paths = PATHS.get_or_init(|| {
        NAMES.map(|name| {
            Symbol::named(name)
                .expect("a Material Symbol")
                .icon()
                .style(Style::Rounded)
                .asset_path()
        })
    });
    Icon::new(SharedString::new_static(paths[index].as_str()))
}

const SUPPORTING: &str = "Supporting line text lorem ipsum dolor sit amet, consectetur.";

/// The kit's leading-element groups, in its order.
#[derive(Clone, Copy)]
enum Lead {
    None,
    Monogram,
    Icon,
    Image,
    Video,
    Checkbox,
    Radio,
    Switch,
}

const LEADS: [(Lead, &str); 8] = [
    (Lead::None, "Text only"),
    (Lead::Monogram, "Monogram"),
    (Lead::Icon, "Icon"),
    (Lead::Image, "Image"),
    (Lead::Video, "Video"),
    (Lead::Checkbox, "Checkbox"),
    (Lead::Radio, "Radio button"),
    (Lead::Switch, "Switch"),
];

/// Trailing columns: none, a checkbox, and an arrow (selection-control
/// leads have no trailing checkbox column, as in the kit).
#[derive(Clone, Copy, PartialEq)]
enum Trail {
    None,
    Checkbox,
    Arrow,
}

fn trails(lead: Lead) -> &'static [Trail] {
    match lead {
        Lead::Checkbox | Lead::Radio | Lead::Switch => &[Trail::None, Trail::Arrow],
        _ => &[Trail::None, Trail::Checkbox, Trail::Arrow],
    }
}

fn leading(lead: Lead) -> ListLeading {
    match lead {
        Lead::None => ListLeading::None,
        Lead::Monogram => ListLeading::Monogram("A".into()),
        Lead::Icon => ListLeading::Icon(symbol(0)),
        Lead::Image => ListLeading::Image(None),
        Lead::Video => ListLeading::Video(None),
        Lead::Checkbox => ListLeading::Checkbox(true),
        Lead::Radio => ListLeading::Radio(true),
        Lead::Switch => ListLeading::Switch(false),
    }
}

fn trailing(trail: Trail) -> ListTrailing {
    match trail {
        Trail::None => ListTrailing::None,
        Trail::Checkbox => ListTrailing::Checkbox(true),
        Trail::Arrow => ListTrailing::Icon(symbol(1)),
    }
}

/// One kit item: `lines` 3, 2 or 1.
fn kit_item(id: impl Into<ElementId>, lead: Lead, trail: Trail, lines: usize) -> ListItem {
    let item = ListItem::new(id, "List item")
        .density(ListDensity::Compact)
        .leading(leading(lead))
        .trailing(trailing(trail));
    match lines {
        3 => item.supporting(SUPPORTING).three_line(true),
        2 => item.supporting(SUPPORTING),
        _ => item,
    }
}

const ITEM_WIDTH: f32 = 232.;

pub struct Page {
    page: PageCx,
    pub(crate) details: bool,
    checked: [bool; 4],
    selected: usize,
}

const LISTS: usize = 0;
const MATRIX: usize = 1;
const SEGMENTED: usize = MATRIX + LEADS.len();
const STATES: usize = SEGMENTED + 1;
const LIVE: usize = STATES + 1;
const DETAILS: usize = LIVE + 1;
const TILE_STATES: usize = DETAILS + 1;

impl Page {
    fn lists(&self) -> AnyElement {
        let list = |index: usize, lead: Lead, trail: Trail| {
            div()
                .w(rem(240.))
                .py(rem(8.))
                .bg(color(ColorRole::SURFACE))
                .children((0..6).map(move |row| {
                    let item = kit_item(("kit-list", index * 6 + row), lead, trail, 1);
                    if index == 0 {
                        item.trailing_text("100+")
                    } else {
                        item
                    }
                }))
        };
        board("lists")
            .caption("Lists")
            .columns(3)
            .column_width(240.)
            .cells([
                list(0, Lead::Monogram, Trail::Checkbox),
                list(1, Lead::Monogram, Trail::Checkbox),
                list(2, Lead::None, Trail::Checkbox),
            ])
            .into_any_element()
    }

    fn matrix(&self, group: usize) -> AnyElement {
        let (lead, name) = LEADS[group];
        let trails = trails(lead);
        board(("list-items", group))
            .band(Band::of(group, LEADS.len()))
            .caption(if group == 0 { "List Items" } else { "" })
            .side_label(name)
            .columns(1)
            .column_width(3. * ITEM_WIDTH + 2. * 16.)
            .row_labels(["3 line", "2 line", "1 line"])
            // One cell per line count holding the trailing columns, so the
            // kit's three columns never wrap.
            .cells([3usize, 2, 1].into_iter().map(move |lines| {
                div()
                    .flex()
                    .gap(rem(16.))
                    .children((0..3).map(move |column| {
                        let id = ("list-item", (group * 3 + lines) * 3 + column);
                        div().w(rem(ITEM_WIDTH)).flex_shrink_0().children(
                            trails
                                .get(column)
                                .map(|&trail| kit_item(id, lead, trail, lines)),
                        )
                    }))
            }))
            .into_any_element()
    }

    fn segmented(&self) -> AnyElement {
        let list = |index: usize, supporting: bool| {
            div()
                .w(rem(280.))
                .child(segmented_list((0..6).map(move |row| {
                    let item = ListItem::new(("segmented", index * 6 + row), "Label text")
                        .leading(ListLeading::Icon(symbol(2)))
                        .trailing_text("⌘C")
                        .trailing(ListTrailing::Icon(symbol(1)))
                        .selected(index == 1 && row == 2);
                    if supporting {
                        item.supporting("Supporting line text, lorem ipsum")
                    } else {
                        item
                    }
                })))
        };
        board("segmented")
            .caption("Segmented lists")
            .columns(2)
            .column_width(280.)
            .column_labels(["One line", "Two line, selected"])
            .cells([list(0, false), list(1, true)])
            .into_any_element()
    }

    fn states(&self) -> AnyElement {
        const WIDTH: f32 = 280.;
        board("list-states")
            .caption("List item states")
            .columns(1)
            .column_width(2. * WIDTH + 16.)
            .row_labels(
                std::iter::once("").chain(InteractionPreview::ALL.map(InteractionPreview::name)),
            )
            .cell(
                div().flex().gap(rem(16.)).children(
                    ["Unselected", "Selected"]
                        .into_iter()
                        .enumerate()
                        .map(|(index, label)| {
                            div()
                                .w(rem(WIDTH))
                                .flex()
                                .justify_center()
                                .child(spec::small_label(("state-column", index), label))
                        }),
                ),
            )
            .cells(
                InteractionPreview::ALL
                    .into_iter()
                    .enumerate()
                    .map(|(row, state)| {
                        div()
                            .flex()
                            .gap(rem(16.))
                            .children([false, true].into_iter().map(move |selected| {
                                div().w(rem(WIDTH)).child(
                                    ListItem::new(
                                        ("list-state", row * 2 + usize::from(selected)),
                                        "Label text",
                                    )
                                    .overline("Overline")
                                    .supporting("Supporting line text")
                                    .leading(ListLeading::Icon(symbol(0)))
                                    .trailing(ListTrailing::Icon(symbol(1)))
                                    .selected(selected)
                                    .preview_state(state)
                                    .on_click(|_, _, _| {}),
                                )
                            }))
                    }),
            )
            .into_any_element()
    }

    fn live(&mut self, cx: &mut Context<Self>) -> AnyElement {
        const ROWS: [(&str, &str); 4] = [
            ("Wi-Fi", "Connected"),
            ("Bluetooth", "Off"),
            ("Airplane mode", "Disables wireless connections"),
            ("Location", "Used by 3 apps"),
        ];
        let list = self.page.rows.list().clone();
        spec::demo(
            "list-demo",
            "Lists in use",
            div()
                .w(rem(360.))
                .flex()
                .flex_col()
                .py(rem(8.))
                .rounded(rem(16.))
                .overflow_hidden()
                .bg(color(ColorRole::SURFACE))
                .children(
                    ROWS.into_iter()
                        .enumerate()
                        .map(|(index, (title, support))| {
                            ListItem::new(("live-item", index), title)
                                .supporting(support)
                                .leading(ListLeading::Monogram(title[..1].to_string().into()))
                                .trailing(ListTrailing::Switch(self.checked[index]))
                                .selected(self.selected == index)
                                .divider(index + 1 < ROWS.len())
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.checked[index] = !this.checked[index];
                                    this.selected = index;
                                    cx.notify();
                                }))
                        }),
                )
                .reveal("reveal-list-demo", &list),
        )
        .into_any_element()
    }

    fn details(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let details = self.details;
        let rows = self.page.rows.clone();
        let toggle = cx.listener(move |this, _, _, cx| {
            this.details = !this.details;
            rows.remeasure(DETAILS..=DETAILS);
            cx.notify();
        });
        let settings = TileGroup::new("tiles", "Connected tiles")
            .tile(move |outer| {
                TileScale::Regular
                    .button("details", outer)
                    .on_click(toggle)
                    .child(
                        TileScale::Regular
                            .row()
                            .child(TileScale::Regular.text(
                                "details-text",
                                "Show details",
                                Some("A tile that discloses content in place".into()),
                            ))
                            .child(disclosure_chevron(details)),
                    )
                    .into_any_element()
            })
            .detail(motion::disclosure(
                "details-content",
                details,
                div()
                    .px(ITEM_INSET)
                    .pb(rem(16.))
                    .child(tile_info("details-info", "Details stay inside their tile.")),
            ))
            .child(
                div()
                    .px(ITEM_INSET)
                    .py(rem(16.))
                    .child(TileScale::Regular.text(
                        "plain",
                        "A content tile",
                        Some("Its corners follow its place in the group".into()),
                    )),
            );
        spec::demo(
            "connected-tiles",
            "Connected tiles",
            settings.reveal("reveal-settings", self.page.rows.list()),
        )
        .into_any_element()
    }
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "lists",
        title: "Lists",
        description: "Lists are continuous, vertical indexes of text and images.",
        guideline: Some("https://m3.material.io/components/lists/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            page,
            details: false,
            checked: [true, false, false, true],
            selected: 0,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        TILE_STATES + 1
    }

    fn row_gap(&self, row: usize) -> f32 {
        match row {
            0 => spec::HEADER_TO_BODY,
            r if (MATRIX..SEGMENTED).contains(&r) => Band::of(r - MATRIX, LEADS.len()).row_gap(),
            _ => spec::ROW_GAP,
        }
    }

    fn interactive(&self, row: usize) -> bool {
        row == LIVE || row == DETAILS
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match row {
            LISTS => self.lists(),
            r if (MATRIX..SEGMENTED).contains(&r) => self.matrix(r - MATRIX),
            SEGMENTED => self.segmented(),
            STATES => self.states(),
            LIVE => self.live(cx),
            DETAILS => self.details(cx),
            _ => board("tile-states")
                .caption("Tile button states")
                .columns(5)
                .column_labels(InteractionPreview::ALL.map(InteractionPreview::name))
                .cells(
                    InteractionPreview::ALL
                        .into_iter()
                        .enumerate()
                        .map(|(index, state)| {
                            TileScale::Regular
                                .button(("tile-state", index), [true; 4])
                                .preview_state(state)
                                .child(TileScale::Regular.text(
                                    ("tile-state-text", index),
                                    "Label",
                                    None,
                                ))
                        }),
                )
                .into_any_element(),
        }
    }
}
