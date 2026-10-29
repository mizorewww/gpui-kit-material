//! Typography: the text size the Gallery follows and the fifteen type roles.
use super::prelude::*;

pub struct Page {
    page: PageCx,
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "typography",
        title: "Typography",
        description: "The type scale's fifteen roles, and the text size the whole Gallery follows.",
        guideline: Some("https://m3.material.io/styles/typography/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self { page }
    }

    fn row_count(&self, _: &Window) -> usize {
        1 + TypeRole::ALL.len()
    }

    fn row_gap(&self, row: usize) -> f32 {
        match row {
            0 => spec::HEADER_TO_BODY,
            1 => 40.,
            _ => 16.,
        }
    }

    fn interactive(&self, row: usize) -> bool {
        row == 0
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        if row == 0 {
            let shared = self.page.shared.clone();
            return spec::demo(
                "scale-demo",
                "Text size",
                ConnectedButtonGroup::new("scale", "Text size")
                    .options([
                        ("1", "100%"),
                        ("1.25", "125%"),
                        ("1.5", "150%"),
                        ("2", "200%"),
                    ])
                    .selected(self.page.shared.read(cx).scale.clone())
                    .on_change(move |value: &SharedString, _, cx| {
                        shared.update(cx, |shared, cx| {
                            shared.scale = value.clone();
                            cx.notify();
                        })
                    })
                    .reveal("reveal-scale", self.page.rows.list()),
            )
            .into_any_element();
        }
        let index = row - 1;
        let role = TypeRole::ALL[index];
        div()
            .flex()
            .flex_col()
            .gap(rem(4.))
            .child(
                spec::small_label(("type-label", index), role.name())
                    .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
            )
            .child(
                spec::type_role(
                    accessible_text(
                        ("type-sample", index),
                        "The quick brown fox jumps over the lazy dog",
                    ),
                    role,
                )
                .whitespace_normal(),
            )
            .into_any_element()
    }
}
