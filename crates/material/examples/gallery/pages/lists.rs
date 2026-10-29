//! Lists: list tiles, connected tiles with a disclosure, and tile states.
use super::prelude::*;
use gpui_kit_material::motion;

pub struct Page {
    page: PageCx,
    pub(crate) details: bool,
}

const DETAILS_ROW: usize = 1;

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
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        3
    }

    fn interactive(&self, row: usize) -> bool {
        row == DETAILS_ROW
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match row {
            0 => {
                let items = [
                    ("Display", "Theme, contrast and text size"),
                    ("Notifications", "Alerts and sounds"),
                    ("About", "Version and licenses"),
                ];
                spec::demo(
                    "list-tiles",
                    "List tiles",
                    div().flex().flex_col().gap(TILE_GAP).children(
                        items
                            .into_iter()
                            .enumerate()
                            .map(|(index, (title, support))| {
                                list_tile(index, items.len())
                                    .px(ITEM_INSET)
                                    .py(rem(12.))
                                    .child(
                                        TileScale::Regular
                                            .row()
                                            .child(TileScale::Regular.text(
                                                ("list-tile", index),
                                                title,
                                                Some(support.into()),
                                            ))
                                            .child(tile_chevron()),
                                    )
                            }),
                    ),
                )
                .into_any_element()
            }
            DETAILS_ROW => {
                let details = self.details;
                let rows = self.page.rows.clone();
                let toggle = cx.listener(move |this, _, _, cx| {
                    this.details = !this.details;
                    rows.remeasure(DETAILS_ROW..=DETAILS_ROW);
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
