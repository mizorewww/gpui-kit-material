//! Carousel: the kit's hero, center-aligned hero, multi-browse and
//! uncontained carousels at mobile (412dp) and tablet (600dp) widths, and a
//! full-screen carousel. Every carousel scrolls (wheel, trackpad, or
//! Left/Right/Home/End while focused) and snaps to its items.
use super::prelude::*;
use gpui_kit_material::icons::{Icon, Style, Symbol};
use std::sync::OnceLock;

/// The kit's placeholder artwork: the shapes symbol.
fn shapes() -> Icon {
    static PATH: OnceLock<String> = OnceLock::new();
    Icon::new(SharedString::new_static(
        PATH.get_or_init(|| {
            Symbol::named("interests")
                .or_else(|| Symbol::named("category"))
                .expect("a Material Symbol")
                .icon()
                .style(Style::Rounded)
                .asset_path()
        })
        .as_str(),
    ))
}

fn items(count: usize, labels: bool) -> impl Iterator<Item = CarouselItem> {
    (0..count).map(move |index| {
        let item = CarouselItem::new().content(
            shapes()
                .size(rem(96.))
                .color(color(ColorRole::OUTLINE_VARIANT)),
        );
        if labels {
            item.label(SharedString::from(format!("Item {}", index + 1)))
                .supporting("Supporting text")
        } else {
            item
        }
    })
}

const LAYOUTS: [(CarouselLayout, &str); 4] = [
    (CarouselLayout::Hero, "Hero"),
    (CarouselLayout::CenteredHero, "Center-aligned hero"),
    (CarouselLayout::MultiBrowse, "Multi-browse"),
    (CarouselLayout::Uncontained, "Uncontained"),
];

/// (width, large item) per context.
const CONTEXTS: [(f32, f32); 2] = [(412., 188.), (600., 184.)];

pub struct Page;

const FULL_SCREEN: usize = LAYOUTS.len();

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "carousel",
        title: "Carousel",
        description: "Carousels show a collection of items that can be scrolled on and off the screen.",
        guideline: Some("https://m3.material.io/components/carousel/overview"),
    };

    fn new(_: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Page
    }

    fn row_count(&self, _: &Window) -> usize {
        FULL_SCREEN + 1
    }

    fn row_gap(&self, row: usize) -> f32 {
        match row {
            0 => spec::HEADER_TO_BODY,
            r if r < FULL_SCREEN => Band::of(r, LAYOUTS.len()).row_gap(),
            _ => spec::ROW_GAP,
        }
    }

    // Every carousel is a live, focusable scroller.
    fn interactive(&self, _: usize) -> bool {
        true
    }

    fn render_row(&mut self, row: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
        if row == FULL_SCREEN {
            return board("carousel-full-screen")
                .caption("Full screen")
                .columns(1)
                .column_width(412.)
                .cell(
                    MaterialCarousel::new("carousel-full", "Full-screen carousel")
                        .layout(CarouselLayout::FullScreen)
                        .width(412.)
                        .item_height(640.)
                        .items(items(6, true)),
                )
                .into_any_element();
        }
        let (layout, name) = LAYOUTS[row];
        board(("carousel", row))
            .band(Band::of(row, LAYOUTS.len()))
            .side_label(name)
            .columns(1)
            .column_width(600.)
            .when(row == 0, |board| board.caption("Carousel"))
            .row_labels(["Mobile", "Tablet"])
            .cells(
                CONTEXTS
                    .into_iter()
                    .enumerate()
                    .map(|(context, (width, large))| {
                        div().w(rem(600.)).child(
                            MaterialCarousel::new(("carousel-specimen", row * 2 + context), name)
                                .layout(layout)
                                .width(width)
                                .large_item_width(if layout == CarouselLayout::Uncontained {
                                    // The kit's uncontained items: 154dp and 163dp.
                                    [154., 163.][context]
                                } else {
                                    large
                                })
                                .item_height(205.)
                                .items(items(8, row == 2 && context == 0)),
                        )
                    }),
            )
            .into_any_element()
    }
}
