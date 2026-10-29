//! Elevation: surfaces at the six levels.
use super::prelude::*;
use gpui_kit_material::tokens::elevation;

pub struct Page;

const LEVELS: [&str; 6] = [
    "Level 0", "Level 1", "Level 2", "Level 3", "Level 4", "Level 5",
];

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "elevation",
        title: "Elevation",
        description: "Surfaces at the six elevation levels, from flat to the highest shadow.",
        guideline: Some("https://m3.material.io/styles/elevation/overview"),
    };

    fn new(_: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Page
    }

    fn row_count(&self, _: &Window) -> usize {
        1
    }

    fn render_row(&mut self, _: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
        board("elevation-levels")
            .caption("Elevation levels")
            .columns(6)
            .cells(LEVELS.into_iter().enumerate().map(|(level, name)| {
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(rem(12.))
                    .py(rem(12.))
                    .child(
                        div()
                            .w(rem(104.))
                            .h(rem(72.))
                            .rounded(RADIUS_MEDIUM)
                            .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
                            .shadow(elevation::level(level as u8)),
                    )
                    .child(spec::small_label(("elevation-level", level), name))
            }))
            .into_any_element()
    }
}
