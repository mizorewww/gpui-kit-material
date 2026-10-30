//! Sheets: the kit's standard and modal side sheets (with and without back
//! and actions), standard and modal bottom sheets, and live sheets.
use super::prelude::*;

/// Specimen side sheets are smaller than the kit's 320×700dp so the four
/// variants share one row.
const SIDE_HEIGHT: f32 = 400.;
const SIDE_WIDTH: f32 = 184.;
const BOTTOM_WIDTH: f32 = 412.;
const BOTTOM_HEIGHT: f32 = 300.;

/// The kit's four side-sheet variants: (back, actions).
const VARIANTS: [(bool, bool); 4] = [(false, false), (true, false), (false, true), (true, true)];

fn specimen_button(
    id: (&'static str, usize),
    variant: ButtonVariant,
    label: &'static str,
) -> MaterialButton {
    material_button(id, variant)
        .preview_state(InteractionPreview::Enabled)
        .child(label)
}

fn side_specimen(modal: bool, index: usize) -> impl IntoElement {
    let (back, actions) = VARIANTS[index];
    let id = usize::from(modal) * 4 + index;
    let mut sheet = MaterialSideSheet::new(("side-specimen", id))
        .specimen(true)
        .modal(modal)
        .width(rem(SIDE_WIDTH))
        .headline("Title")
        .close_button("Close");
    if back {
        sheet = sheet.back("Back", |_, _| {});
    }
    if actions {
        sheet = sheet
            .action(specimen_button(
                ("sheet-save", id),
                ButtonVariant::Filled,
                "Save",
            ))
            .action(specimen_button(
                ("sheet-cancel", id),
                ButtonVariant::Outlined,
                "Cancel",
            ));
    }
    div().w(rem(SIDE_WIDTH)).h(rem(SIDE_HEIGHT)).child(sheet)
}

/// A frame behind a modal specimen: content under the 32% scrim.
fn scrim_frame(content: impl IntoElement) -> Div {
    div()
        .p(rem(16.))
        .rounded(rem(16.))
        .bg(color(ColorRole::SCRIM).opacity(SCRIM_OPACITY))
        .child(content)
}

pub struct Page {
    page: PageCx,
    side: bool,
    modal_side: bool,
    bottom: bool,
    modal_bottom: bool,
}

const STANDARD: usize = 0;
const MODAL: usize = 1;
const BOTTOM: usize = 2;
const LIVE: usize = 3;

fn close<T: 'static>(
    entity: WeakEntity<T>,
    f: impl Fn(&mut T) + 'static,
) -> impl Fn(&mut Window, &mut App) + 'static {
    move |_, cx| {
        let _ = entity.update(cx, |this, cx| {
            f(this);
            cx.notify();
        });
    }
}

impl Page {
    fn live(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let list = self.page.rows.list().clone();
        let entity = cx.entity().downgrade();
        let toggle = |id: &'static str, label: &'static str, f: fn(&mut Self)| {
            material_button(id, ButtonVariant::Tonal)
                .child(label)
                .on_click(cx.listener(move |this, _, _, cx| {
                    f(this);
                    cx.notify();
                }))
                .reveal(
                    ElementId::NamedChild(std::sync::Arc::new(id.into()), "reveal".into()),
                    &list,
                )
        };
        let body = |text: &'static str| {
            div()
                .px(rem(24.))
                .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                .child(text)
        };
        spec::demo(
            "sheets-demo",
            "Sheets in use",
            div()
                .flex()
                .flex_col()
                .gap(rem(16.))
                .child(
                    spec::demo_row()
                        .child(toggle("toggle-side", "Standard side sheet", |p| {
                            p.side = !p.side
                        }))
                        .child(toggle("open-modal-side", "Modal side sheet", |p| {
                            p.modal_side = true
                        }))
                        .child(toggle("toggle-bottom", "Standard bottom sheet", |p| {
                            p.bottom = !p.bottom
                        }))
                        .child(toggle("open-modal-bottom", "Modal bottom sheet", |p| {
                            p.modal_bottom = true
                        })),
                )
                .child(
                    div()
                        .relative()
                        .h(rem(360.))
                        .overflow_hidden()
                        .rounded(RADIUS_MEDIUM)
                        .border_1()
                        .border_color(color(ColorRole::OUTLINE_VARIANT))
                        .bg(color(ColorRole::SURFACE_CONTAINER_LOWEST))
                        // The screen the sheets open over: a hint and some
                        // content that standard sheets leave interactive.
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .py(rem(8.))
                                .child(
                                    div()
                                        .py(rem(8.))
                                        .child(body("Open a sheet with the buttons above.")),
                                )
                                .children(
                                    [
                                        ("Photos", "12 new items"),
                                        ("Documents", "Shared with you"),
                                        ("Music", "3 playlists"),
                                        ("Downloads", "Updated today"),
                                    ]
                                    .into_iter()
                                    .enumerate()
                                    .map(
                                        |(ix, (label, supporting))| {
                                            ListItem::new(("sheet-content", ix), label)
                                                .supporting(supporting)
                                                .leading(ListLeading::Monogram(
                                                    label[..1].to_owned().into(),
                                                ))
                                        },
                                    ),
                                ),
                        )
                        .child(
                            MaterialSideSheet::new("live-side")
                                .open(self.side)
                                .width(rem(280.))
                                .headline("Filters")
                                .close_button("Close")
                                .on_dismiss(close(entity.clone(), |p: &mut Self| p.side = false))
                                .child(body("Standard side sheets leave the content interactive.")),
                        )
                        .child(
                            MaterialBottomSheet::new("live-bottom")
                                .open(self.bottom)
                                .drag_handle("Drag handle")
                                .partial_height(rem(120.))
                                .on_dismiss(close(entity.clone(), |p: &mut Self| p.bottom = false))
                                .child(div().h(rem(240.)).child(body(
                                    "Drag the sheet up to expand it or down to dismiss it.",
                                ))),
                        ),
                )
                .child(
                    MaterialSideSheet::new("live-modal-side")
                        .modal(true)
                        .open(self.modal_side)
                        .headline("Title")
                        .back(
                            "Back",
                            close(entity.clone(), |p: &mut Self| p.modal_side = false),
                        )
                        .close_button("Close")
                        .on_dismiss(close(entity.clone(), |p: &mut Self| p.modal_side = false))
                        .action(
                            material_button("modal-side-save", ButtonVariant::Filled)
                                .child("Save")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.modal_side = false;
                                    cx.notify();
                                })),
                        )
                        .action(
                            material_button("modal-side-cancel", ButtonVariant::Outlined)
                                .child("Cancel")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.modal_side = false;
                                    cx.notify();
                                })),
                        )
                        .child(body("Modal side sheets block the content behind a scrim.")),
                )
                .child(
                    MaterialBottomSheet::new("live-modal-bottom")
                        .modal(true)
                        .open(self.modal_bottom)
                        .drag_handle("Drag handle")
                        .on_dismiss(close(entity, |p: &mut Self| p.modal_bottom = false))
                        .child(
                            div().h(rem(280.)).child(body(
                                "Drag down, press Escape or tap the scrim to dismiss.",
                            )),
                        ),
                ),
        )
        .into_any_element()
    }
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "sheets",
        title: "Sheets",
        description: "Sheets show secondary content anchored to the edge of the screen.",
        guideline: Some("https://m3.material.io/components/side-sheets/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            page,
            side: false,
            modal_side: false,
            bottom: false,
            modal_bottom: false,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        LIVE + 1
    }

    fn interactive(&self, row: usize) -> bool {
        row == LIVE
    }

    fn apply_capture(&mut self, state: CaptureState, cx: &mut Context<Self>) {
        // `--active` opens the standard sheets, `--selected` the modal side
        // sheet, for captures.
        self.side = state.active;
        self.bottom = state.active;
        self.modal_side = state.selected;
        cx.notify();
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match row {
            STANDARD => board("side-standard")
                .caption("Side sheets · Standard")
                .columns(4)
                .column_width(SIDE_WIDTH)
                .cells((0..4).map(|index| side_specimen(false, index)))
                .into_any_element(),
            MODAL => board("side-modal")
                .caption("Side sheets · Modal")
                .columns(1)
                .cell(scrim_frame(
                    div()
                        .flex()
                        .gap(rem(16.))
                        .children((0..4).map(|index| side_specimen(true, index))),
                ))
                .into_any_element(),
            BOTTOM => board("bottom-sheets")
                .caption("Bottom sheets")
                .columns(2)
                .column_width(BOTTOM_WIDTH + 32.)
                .column_labels(["Standard", "Modal"])
                .cells([false, true].map(|modal| {
                    let sheet = div().w(rem(BOTTOM_WIDTH)).h(rem(BOTTOM_HEIGHT)).child(
                        MaterialBottomSheet::new(("bottom-specimen", usize::from(modal)))
                            .modal(modal)
                            .drag_handle("Drag handle"),
                    );
                    if modal {
                        scrim_frame(sheet).pb_0().rounded_b_none()
                    } else {
                        div().p(rem(16.)).pb_0().child(sheet)
                    }
                }))
                .into_any_element(),
            _ => self.live(cx),
        }
    }
}
