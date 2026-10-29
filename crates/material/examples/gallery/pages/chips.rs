//! Chips: assist chips in every interaction state, live assist chips, and
//! the status chips.
use super::prelude::*;

pub struct Page {
    page: PageCx,
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "chips",
        title: "Chips",
        description: "Chips help people enter information, make selections, filter content, or trigger actions.",
        guideline: Some("https://m3.material.io/components/chips/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self { page }
    }

    fn row_count(&self, _: &Window) -> usize {
        3
    }

    fn interactive(&self, row: usize) -> bool {
        row == 1
    }

    fn render_row(&mut self, row: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
        let list = self.page.rows.list().clone();
        match row {
            0 => board("assist-chips")
                .caption("Assist chips")
                .columns(2)
                .column_width(160.)
                .column_labels(["Label", "Label & icon"])
                .row_labels(InteractionPreview::ALL.map(InteractionPreview::name))
                .cells(InteractionPreview::ALL.into_iter().enumerate().flat_map(
                    |(index, state)| {
                        [
                            AssistChip::new(("assist-state", index), "Label").preview_state(state),
                            AssistChip::new(("assist-state-icon", index), "Label")
                                .icon(IconName::Info)
                                .preview_state(state),
                        ]
                    },
                ))
                .into_any_element(),
            1 => {
                let snackbar = |shared: Entity<GalleryShared>| {
                    move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
                        shared.update(cx, |shared, cx| {
                            shared.snackbar = true;
                            cx.notify();
                        })
                    }
                };
                spec::demo(
                    "live-chips",
                    "Assist chips in use",
                    spec::demo_row()
                        .child(
                            AssistChip::new("assist-chip", "Assist chip")
                                .on_click(snackbar(self.page.shared.clone()))
                                .reveal("reveal-assist-chip", &list),
                        )
                        .child(
                            AssistChip::new("assist-chip-icon", "With leading icon")
                                .icon(IconName::Info)
                                .on_click(snackbar(self.page.shared.clone()))
                                .reveal("reveal-assist-chip-icon", &list),
                        )
                        .child(
                            AssistChip::new("assist-chip-disabled", "Disabled")
                                .icon(IconName::Info)
                                .disabled(true),
                        ),
                )
                .into_any_element()
            }
            _ => spec::demo(
                "status-chips",
                "Status chips",
                spec::demo_row()
                    .child(status_chip(StatusKind::Success).child("Ready"))
                    .child(status_chip(StatusKind::Warning).child("Needs attention"))
                    .child(status_chip(StatusKind::Danger).child("Failed"))
                    .child(status_chip(StatusKind::Progress).child("Working"))
                    .child(status_chip(StatusKind::Neutral).child("Queued"))
                    .child(status_chip(StatusKind::Tonal).child("Default"))
                    .child(outlined_status_chip("outlined", "Outlined")),
            )
            .into_any_element(),
        }
    }
}
