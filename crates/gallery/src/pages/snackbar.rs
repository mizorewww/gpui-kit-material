//! Snackbar: the kit's configurations with and without the close
//! affordance, the action and close states, the live snackbar (also shown
//! from a dialog, above it), and inline messages.
use super::prelude::*;

pub struct Page {
    page: PageCx,
    /// The live demo's dialog, whose action shows the snackbar over it.
    dialog: bool,
}

/// The kit's configurations: message, action, longer action.
const CONFIGURATIONS: [(&str, Option<&str>, bool); 5] = [
    ("Single-line snackbar", None, false),
    ("Single-line snackbar with action", Some("Action"), false),
    (
        "Two-line snackbar without action, wrapping onto a second line",
        None,
        false,
    ),
    (
        "Two-line snackbar with action, wrapping onto a second line",
        Some("Action"),
        false,
    ),
    (
        "Two-line snackbar with longer action",
        Some("Longer action"),
        true,
    ),
];
const STATES: [InteractionPreview; 4] = [
    InteractionPreview::Enabled,
    InteractionPreview::Hovered,
    InteractionPreview::Focused,
    InteractionPreview::Pressed,
];
const WIDTH: f32 = 344.;

fn specimen(
    id: impl Into<ElementId>,
    message: &'static str,
    action: Option<&'static str>,
    longer: bool,
    close: bool,
) -> Snackbar {
    let bar = Snackbar::new(id, message).longer_action(longer);
    let bar = match action {
        Some(label) => bar.action(label, |_, _, _| {}),
        None => bar,
    };
    if close {
        bar.close("Close", |_, _, _| {})
    } else {
        bar
    }
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "snackbar",
        title: "Snackbar",
        description: "Snackbars provide brief messages about app processes at the bottom of the screen.",
        guideline: Some("https://m3.material.io/components/snackbar/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            page,
            dialog: false,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        4
    }

    fn interactive(&self, row: usize) -> bool {
        row == 2
    }

    fn apply_capture(&mut self, state: CaptureState, cx: &mut Context<Self>) {
        // `--active` opens the dialog and shows the snackbar above it.
        self.dialog = state.active;
        self.page.shared.update(cx, |shared, cx| {
            shared.snackbar = state.active;
            cx.notify();
        });
        cx.notify();
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match row {
            0 => board("snackbars")
                .caption("Snackbar")
                .columns(2)
                .column_labels(["Without close affordance", "With close affordance"])
                .cells(CONFIGURATIONS.into_iter().enumerate().flat_map(
                    |(i, (message, action, longer))| {
                        [false, true].map(move |close| {
                            div().w(rem(WIDTH)).child(
                                specimen(
                                    ("snackbar", i * 2 + usize::from(close)),
                                    message,
                                    action,
                                    longer,
                                    close,
                                )
                                .preview_state(InteractionPreview::Enabled),
                            )
                        })
                    },
                ))
                .into_any_element(),
            1 => board("snackbar-states")
                .caption("Building blocks: action and close affordance")
                .columns(STATES.len())
                .column_labels(STATES.map(|s| s.name()))
                .cells(STATES.into_iter().enumerate().map(|(i, state)| {
                    div().w(rem(240.)).child(
                        specimen(("snackbar-state", i), "Message", Some("Label"), false, true)
                            .preview_state(state),
                    )
                }))
                .into_any_element(),
            2 => {
                let show = |shared: Entity<GalleryShared>| {
                    move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
                        shared.update(cx, |shared, cx| {
                            shared.snackbar = true;
                            cx.notify();
                        })
                    }
                };
                let close = cx.listener(|this, _, _, cx| {
                    this.dialog = false;
                    cx.notify();
                });
                let entity = cx.entity().downgrade();
                spec::demo(
                    "snackbar-demo",
                    "Live snackbar",
                    spec::demo_row()
                        .child(
                            material_button("show-snackbar", ButtonVariant::Tonal)
                                .child("Show snackbar")
                                .on_click(show(self.page.shared.clone()))
                                .reveal("reveal-show-snackbar", self.page.rows.list()),
                        )
                        .child(
                            material_button("snackbar-open-dialog", ButtonVariant::Outlined)
                                .child("Show from a dialog")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.dialog = true;
                                    cx.notify();
                                }))
                                .reveal("reveal-snackbar-open-dialog", self.page.rows.list()),
                        )
                        // A snackbar shown from a dialog appears above it and
                        // its scrim; the dialog stays open and keeps focus.
                        .child(
                            MaterialDialog::new("snackbar-dialog")
                                .headline("Share link")
                                .supporting_text(
                                    "Copying the link shows a snackbar above this dialog.",
                                )
                                .action(
                                    material_button("snackbar-dialog-done", ButtonVariant::Text)
                                        .child("Done")
                                        .on_click(close),
                                )
                                .action(
                                    material_button("snackbar-dialog-copy", ButtonVariant::Text)
                                        .child("Copy link")
                                        .on_click(show(self.page.shared.clone())),
                                )
                                .modal(self.dialog)
                                .on_dismiss(move |_, cx| {
                                    let _ = entity.update(cx, |this, cx| {
                                        this.dialog = false;
                                        cx.notify();
                                    });
                                }),
                        ),
                )
                .into_any_element()
            }
            _ => spec::demo(
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
                        "notice-warning",
                        NoticeKind::Warning,
                        "Some files could not be checked yet.",
                    ))
                    .child(notice_banner(
                        "notice-danger",
                        NoticeKind::Danger,
                        "A failure stays visible with its recovery action.",
                    ))
                    .child(
                        notice_banner(
                            "notice-long",
                            NoticeKind::Danger,
                            "A failure explained at length wraps onto several lines: what \
                             happened, why it stands in the way, and where to change it, \
                             then what to do once it is changed. Its icon stays beside the \
                             first line, and the action follows the text.",
                        )
                        .child(
                            material_button("notice-long-action", ButtonVariant::Text)
                                .child("Open settings"),
                        ),
                    )
                    .child(info_callout(
                        "callout",
                        "Standalone guidance sits in its own low-emphasis panel.",
                    ))
                    .child(error_text("error", "Error text is announced as an alert.")),
            )
            .into_any_element(),
        }
    }
}
