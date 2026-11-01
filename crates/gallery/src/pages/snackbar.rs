//! Snackbar: the kit's configurations with and without the close
//! affordance, the action and close states, the live snackbar, and inline
//! messages.
use super::prelude::*;

pub struct Page {
    page: PageCx,
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
        Self { page }
    }

    fn row_count(&self, _: &Window) -> usize {
        4
    }

    fn interactive(&self, row: usize) -> bool {
        row == 2
    }

    fn render_row(&mut self, row: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
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
                let shared = self.page.shared.clone();
                spec::demo(
                    "snackbar-demo",
                    "Live snackbar",
                    spec::demo_row().child(
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
