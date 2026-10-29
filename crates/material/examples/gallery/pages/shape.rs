//! Shape: the corner radius scale.
use super::prelude::*;

pub struct Page;

const TOKENS: [(&str, Pixels); 10] = [
    ("None · 0", RADIUS_NONE),
    ("Extra small · 4", RADIUS_EXTRA_SMALL),
    ("Small · 8", RADIUS_SMALL),
    ("Medium · 12", RADIUS_MEDIUM),
    ("Large · 16", RADIUS_LARGE),
    ("Large increased · 20", RADIUS_LARGE_INCREASED),
    ("Extra large · 28", RADIUS_EXTRA_LARGE),
    ("Extra large increased · 32", RADIUS_EXTRA_LARGE_INCREASED),
    ("Extra extra large · 48", RADIUS_EXTRA_EXTRA_LARGE),
    ("Full", RADIUS_FULL),
];

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "shape",
        title: "Shape",
        description: "The corner radius scale, from none to full, including the Expressive increased steps.",
        guideline: Some("https://m3.material.io/styles/shape/overview"),
    };

    fn new(_: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Page
    }

    fn row_count(&self, _: &Window) -> usize {
        1
    }

    fn render_row(&mut self, _: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
        board("shape-scale")
            .caption("Corner radius")
            .columns(5)
            .cells(
                TOKENS
                    .into_iter()
                    .enumerate()
                    .map(|(index, (name, radius))| {
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(rem(8.))
                            .py(rem(8.))
                            .child(
                                div()
                                    .size(rem(96.))
                                    .rounded(radius)
                                    .bg(color(ColorRole::PRIMARY_CONTAINER)),
                            )
                            .child(spec::small_label(("shape-token", index), name).text_center())
                    }),
            )
            .into_any_element()
    }
}
