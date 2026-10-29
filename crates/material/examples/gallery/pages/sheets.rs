//! Sheets: the standard side sheet.
use super::prelude::*;

pub struct Page {
    page: PageCx,
    sheet: bool,
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "sheets",
        title: "Sheets",
        description: "Sheets show secondary content anchored to the edge of the screen.",
        guideline: Some("https://m3.material.io/components/side-sheets/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self { page, sheet: false }
    }

    fn row_count(&self, _: &Window) -> usize {
        1
    }

    fn interactive(&self, _: usize) -> bool {
        true
    }

    fn render_row(&mut self, _: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let list = self.page.rows.list().clone();
        spec::demo(
            "side-sheet-demo",
            "Standard side sheet",
            div()
                .relative()
                .h(rem(220.))
                .p(px(8.))
                .overflow_hidden()
                .rounded(RADIUS_MEDIUM)
                .bg(color(ColorRole::SURFACE))
                .child(
                    material_button("open-sheet", ButtonVariant::Tonal)
                        .child("Toggle side sheet")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.sheet = !this.sheet;
                            cx.notify();
                        }))
                        .reveal("reveal-open-sheet", &list),
                )
                .child(
                    MaterialSideSheet::new("gallery-sheet")
                        .open(self.sheet)
                        .width(rem(280.))
                        .child(
                            div().p(rem(20.)).child("Standard side sheet").child(
                                material_button("close-sheet", ButtonVariant::Text)
                                    .child("Close")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.sheet = false;
                                        cx.notify();
                                    }))
                                    .reveal("reveal-close-sheet", &list),
                            ),
                        ),
                ),
        )
        .into_any_element()
    }
}
