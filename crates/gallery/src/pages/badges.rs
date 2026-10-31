//! Badges: the kit's large and small badges, and both placed on an icon.
use super::prelude::*;

pub struct Page;

/// M3 badges sit on an icon's top-trailing corner: the small dot at the
/// corner, the large badge 12dp across and 14dp down.
fn badged(id: &'static str, badge: AnyElement) -> impl IntoElement {
    div()
        .id(id)
        .relative()
        .flex_shrink_0()
        .size(rem(24.))
        .child(
            IconName::Info
                .icon()
                .size(rem(24.))
                .color(color(ColorRole::ON_SURFACE_VARIANT)),
        )
        .child(badge)
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "badges",
        title: "Badges",
        description: "Badges are used to convey dynamic information, such as a count or status. A badge can include text, labels, or numbers.",
        guideline: Some("https://m3.material.io/components/badges/overview"),
    };

    fn new(_: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Page
    }

    fn row_count(&self, _: &Window) -> usize {
        2
    }

    fn render_row(&mut self, row: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
        if row == 0 {
            return board("badges")
                .caption("Badges")
                .columns(2)
                .column_labels(["Large", "Small"])
                .cell(large_badge("large-badge-kit", "3"))
                .cell(small_badge())
                .into_any_element();
        }
        board("badges-on-icons")
            .caption("On an icon")
            .columns(3)
            .column_width(120.)
            .column_labels(["Small", "Large", "Large · overflow"])
            .cell(badged(
                "small-badge-sample",
                small_badge()
                    .absolute()
                    .top_0()
                    .right_0()
                    .into_any_element(),
            ))
            .cell(badged(
                "large-badge-sample",
                large_badge("large-badge", "3")
                    .absolute()
                    .top(rem(-2.))
                    .left(rem(12.))
                    .into_any_element(),
            ))
            .cell(badged(
                "overflow-badge-sample",
                large_badge("overflow-badge", "999+")
                    .absolute()
                    .top(rem(-2.))
                    .left(rem(12.))
                    .into_any_element(),
            ))
            .into_any_element()
    }
}
