//! Dialogs: the kit's basic, list and scrollable list dialogs (each without
//! and with a hero icon), and live modal basic and full-screen dialogs.
use super::prelude::*;
use gpui_kit_material::icons::{Icon, Style, Symbol};
use std::sync::OnceLock;

/// The kit's hero icon (mobile_check), or a check when the catalog lacks it.
fn hero_icon() -> Icon {
    static PATH: OnceLock<String> = OnceLock::new();
    Icon::new(SharedString::new_static(
        PATH.get_or_init(|| {
            Symbol::named("mobile_check")
                .or_else(|| Symbol::named("check_circle"))
                .expect("a Material Symbol")
                .icon()
                .style(Style::Rounded)
                .asset_path()
        })
        .as_str(),
    ))
}

const SUPPORTING: &str = "A dialog is a type of modal window that appears in front of app content to provide critical information, or prompt for a decision to be made.";

/// The kit's two text actions, as specimens (no tab stops).
fn specimen_actions(dialog: MaterialDialog, id: usize) -> MaterialDialog {
    dialog
        .action(
            material_button(("dialog-action-2", id), ButtonVariant::Text)
                .preview_state(InteractionPreview::Enabled)
                .child("Action 2"),
        )
        .action(
            material_button(("dialog-action-1", id), ButtonVariant::Text)
                .preview_state(InteractionPreview::Enabled)
                .child("Action 1"),
        )
}

/// Three list items: monogram, label, "100+", checked checkbox.
fn list_items(id: usize, count: usize) -> Div {
    div().flex().flex_col().children((0..count).map(move |row| {
        ListItem::new(("dialog-list", id * 16 + row), "List item")
            .density(ListDensity::Compact)
            .container(ColorRole::SURFACE_CONTAINER_HIGH)
            .leading(ListLeading::Monogram("A".into()))
            .trailing_text("100+")
            .trailing(ListTrailing::Checkbox(true))
            .divider(row + 1 < count)
    }))
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Basic,
    List,
    Scrollable,
}

const KINDS: [(Kind, &str); 3] = [
    (Kind::Basic, "Basic dialog"),
    (Kind::List, "List dialog"),
    (Kind::Scrollable, "Scrollable list dialog"),
];

fn specimen(kind: Kind, icon: bool, id: usize) -> MaterialDialog {
    let title = if kind == Kind::Basic {
        "Basic dialog title"
    } else {
        "Dialog title"
    };
    let mut dialog = MaterialDialog::new(("dialog-specimen", id))
        .headline(title)
        .supporting_text(SUPPORTING);
    if icon {
        dialog = dialog.icon(hero_icon());
    }
    dialog = match kind {
        Kind::Basic => dialog,
        Kind::List => dialog.content(list_items(id, 3)),
        Kind::Scrollable => dialog.content(
            div()
                .id(("dialog-scroll", id))
                .h(rem(170.))
                .overflow_y_scroll()
                .border_t_1()
                .border_b_1()
                .border_color(color(ColorRole::OUTLINE_VARIANT))
                .child(list_items(id, 10)),
        ),
    };
    specimen_actions(dialog, id)
}

pub struct Page {
    page: PageCx,
    basic: bool,
    full_screen: bool,
}

const LIVE: usize = KINDS.len();

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "dialogs",
        title: "Dialogs",
        description: "Dialogs provide important prompts in a user flow.",
        guideline: Some("https://m3.material.io/components/dialogs/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            page,
            basic: false,
            full_screen: false,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        LIVE + 1
    }

    fn interactive(&self, row: usize) -> bool {
        row == LIVE
    }

    fn apply_capture(&mut self, state: CaptureState, cx: &mut Context<Self>) {
        // `--active` opens the basic dialog for a capture.
        self.basic = state.active;
        cx.notify();
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        if row < LIVE {
            let (kind, caption) = KINDS[row];
            return board(("dialogs", row))
                .caption(caption)
                .columns(2)
                .column_width(DIALOG_WIDTH)
                .column_labels(["Icon = False", "Icon = True"])
                .cells([false, true].map(|icon| specimen(kind, icon, row * 2 + usize::from(icon))))
                .into_any_element();
        }
        let list = self.page.rows.list().clone();
        let close_basic = cx.listener(|this, _, _, cx| {
            this.basic = false;
            cx.notify();
        });
        let confirm_basic = cx.listener(|this, _, _, cx| {
            this.basic = false;
            cx.notify();
        });
        let close_full = cx.listener(|this, _, _, cx| {
            this.full_screen = false;
            cx.notify();
        });
        let entity = cx.entity().downgrade();
        let entity_full = entity.clone();
        spec::demo(
            "dialog-demo",
            "Dialogs in use",
            spec::demo_row()
                .child(
                    material_button("open-dialog", ButtonVariant::Tonal)
                        .child("Open dialog")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.basic = true;
                            cx.notify();
                        }))
                        .reveal("reveal-open-dialog", &list),
                )
                .child(
                    material_button("open-full-dialog", ButtonVariant::Outlined)
                        .child("Open full-screen dialog")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.full_screen = true;
                            cx.notify();
                        }))
                        .reveal("reveal-open-full-dialog", &list),
                )
                .child(
                    MaterialDialog::new("live-dialog")
                        .icon(hero_icon())
                        .headline("Reset settings?")
                        .supporting_text(
                            "This will reset your app preferences back to their default settings.",
                        )
                        .action(
                            material_button("live-cancel", ButtonVariant::Text)
                                .child("Cancel")
                                .on_click(close_basic),
                        )
                        .action(
                            material_button("live-accept", ButtonVariant::Text)
                                .child("Accept")
                                .on_click(confirm_basic),
                        )
                        .modal(self.basic)
                        .on_dismiss(move |_, cx| {
                            let _ = entity.update(cx, |this, cx| {
                                this.basic = false;
                                cx.notify();
                            });
                        }),
                )
                .child(
                    MaterialDialog::new("live-full-dialog")
                        .headline("New event")
                        .supporting_text("Full-screen dialogs fill the window on compact screens.")
                        .content(div().flex().flex_col().children((0..4).map(|row| {
                            ListItem::new(
                                ("full-dialog-row", row),
                                ["Title", "Date", "Time", "Location"][row],
                            )
                            .supporting("Add details")
                            .divider(true)
                        })))
                        .full_screen(
                            "Close",
                            Some(
                                material_button("live-full-save", ButtonVariant::Text)
                                    .child("Save")
                                    .on_click(close_full)
                                    .into_any_element(),
                            ),
                        )
                        .modal(self.full_screen)
                        .on_dismiss(move |_, cx| {
                            let _ = entity_full.update(cx, |this, cx| {
                                this.full_screen = false;
                                cx.notify();
                            });
                        }),
                ),
        )
        .into_any_element()
    }
}
