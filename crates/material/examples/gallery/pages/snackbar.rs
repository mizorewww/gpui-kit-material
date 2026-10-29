//! Snackbar: a snackbar specimen, the live snackbar, and inline messages.
use super::prelude::*;

pub struct Page {
    page: PageCx,
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "snackbar",
        title: "Snackbar",
        description: "Snackbars show short updates about app processes at the bottom of the screen.",
        guideline: Some("https://m3.material.io/components/snackbar/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self { page }
    }

    fn row_count(&self, _: &Window) -> usize {
        2
    }

    fn interactive(&self, row: usize) -> bool {
        row == 0
    }

    fn render_row(&mut self, row: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
        if row == 0 {
            let shared = self.page.shared.clone();
            return spec::demo(
                "snackbar-demo",
                "Snackbar",
                spec::demo_row()
                    .child(Snackbar::new("snackbar-specimen", "Message sent"))
                    .child(
                        material_button("show-snackbar", ButtonVariant::Tonal)
                            .child("Show snackbar")
                            .on_click(move |_, _, cx| {
                                shared.update(cx, |shared, cx| {
                                    shared.snackbar = true;
                                    cx.notify();
                                })
                            })
                            .reveal("reveal-show-snackbar", self.page.rows.list()),
                    ),
            )
            .into_any_element();
        }
        spec::demo(
            "inline-messages",
            "Inline messages",
            div()
                .flex()
                .flex_col()
                .gap(rem(12.))
                .child(notice_banner(
                    "notice-info",
                    NoticeKind::Info,
                    "An informational notice wraps its text when narrow.",
                ))
                .child(notice_banner(
                    "notice-danger",
                    NoticeKind::Danger,
                    "A failure stays visible with its recovery action.",
                ))
                .child(info_callout(
                    "callout",
                    "Standalone guidance sits in its own low-emphasis panel.",
                ))
                .child(error_text("error", "Error text is announced as an alert.")),
        )
        .into_any_element()
    }
}
