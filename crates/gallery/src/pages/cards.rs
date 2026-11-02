//! Cards: the outlined, elevated and filled cards composed like the kit's
//! specimens (header, media, text and actions), their horizontal form, and
//! the kit's card states (hover, focus, press, drag), and a drop target:
//! at rest, dragged over, and a live one that takes files dragged from the
//! desktop or chosen with its button.
use super::prelude::*;
use gpui_kit_material::icons::{Icon, Symbol};

pub struct Page {
    /// The names of the files last dropped on or chosen for the live drop
    /// target.
    dropped: Option<SharedString>,
}

/// The kit's column order.
const VARIANTS: [(CardVariant, &str); 3] = [
    (CardVariant::Outlined, "Outlined"),
    (CardVariant::Elevated, "Elevated"),
    (CardVariant::Filled, "Filled"),
];
const CARD_WIDTH: f32 = 248.;

fn avatar() -> impl IntoElement {
    div()
        .size(rem(40.))
        .flex_shrink_0()
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(color(ColorRole::PRIMARY_CONTAINER))
        .text_color(color(ColorRole::ON_PRIMARY_CONTAINER))
        .map(|avatar| spec::type_role(avatar, TypeRole::TitleMedium))
        .child("A")
}

fn header(index: usize, more: bool) -> Div {
    div()
        .flex()
        .items_center()
        .gap(rem(16.))
        .pl(rem(16.))
        .pr(rem(if more { 4. } else { 16. }))
        .py(rem(if more { 12. } else { 16. }))
        .child(avatar())
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .child(spec::type_role(
                    accessible_text(("card-header", index), "Header"),
                    TypeRole::TitleMedium,
                ))
                .child(spec::type_role(
                    accessible_text(("card-subhead", index), "Subhead"),
                    TypeRole::BodyMedium,
                )),
        )
        .when(more, |header| {
            header.child(
                IconButton::new(
                    ("card-more", index),
                    Symbol::named("more_vert").expect("more_vert is a Material Symbol"),
                    "More options",
                )
                .preview_state(InteractionPreview::Enabled),
            )
        })
}

fn media() -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
        .child(
            spec::star_icon()
                .size(rem(48.))
                .color(color(ColorRole::OUTLINE_VARIANT)),
        )
}

fn vertical(index: usize, variant: CardVariant) -> impl IntoElement {
    CardSurface::new(variant)
        .interactive()
        .w(rem(CARD_WIDTH))
        .child(header(index, true))
        .child(media().h(rem(128.)))
        .child(
            div()
                .flex()
                .flex_col()
                .p(rem(16.))
                .gap(rem(4.))
                .child(spec::type_role(
                    accessible_text(("card-title", index), "Title"),
                    TypeRole::BodyLarge,
                ))
                .child(
                    spec::type_role(
                        accessible_text(("card-subtitle", index), "Subtitle"),
                        TypeRole::BodyMedium,
                    )
                    .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                )
                .child(
                    spec::type_role(
                        accessible_text(
                            ("card-text", index),
                            "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor",
                        ),
                        TypeRole::BodyMedium,
                    )
                    .pt(rem(16.))
                    .whitespace_normal()
                    .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                )
                .child(
                    div()
                        .pt(rem(24.))
                        .flex()
                        .justify_end()
                        .gap(rem(8.))
                        .child(
                            material_button(("card-secondary", index), ButtonVariant::Outlined)
                                .preview_state(InteractionPreview::Enabled)
                                .child("Secondary"),
                        )
                        .child(
                            material_button(("card-primary", index), ButtonVariant::Filled)
                                .preview_state(InteractionPreview::Enabled)
                                .child("Primary"),
                        ),
                ),
        )
}

/// A drop target's content as an application composes it: an upload icon,
/// a label and a button to choose a file instead.
/// The icon turns primary with the target, by the drag itself or forced.
fn drop_target_specimen(
    id: &'static str,
    active: bool,
    label: SharedString,
    choose: MaterialButton,
) -> Stateful<Div> {
    let upload: Icon = Symbol::named("upload")
        .expect("upload is a Material Symbol")
        .into();
    let primary = color(ColorRole::PRIMARY);
    drop_target(id, "Upload files", active)
        .w(rem(CARD_WIDTH))
        .h(rem(176.))
        .p(rem(16.))
        .gap(rem(12.))
        .child(
            div()
                .text_color(color(if active {
                    ColorRole::PRIMARY
                } else {
                    ColorRole::ON_SURFACE_VARIANT
                }))
                .group_drag_over::<ExternalPaths>(DROP_TARGET_GROUP, move |style| {
                    style.text_color(primary)
                })
                .child(upload.size(rem(32.))),
        )
        .child(
            spec::type_role(accessible_text((id, 0usize), label), TypeRole::BodyMedium)
                .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
        )
        .child(choose.child("Choose file"))
}

/// The names of `paths`, joined for the live target's label.
#[cfg(not(target_arch = "wasm32"))]
fn file_names(paths: &[std::path::PathBuf]) -> SharedString {
    paths
        .iter()
        .filter_map(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(", ")
        .into()
}

fn horizontal(index: usize, variant: CardVariant) -> impl IntoElement {
    CardSurface::new(variant)
        .w(rem(CARD_WIDTH))
        .flex_row()
        .child(header(index + 10, false).flex_1().min_w_0())
        .child(media().w(rem(80.)))
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "cards",
        title: "Cards",
        description: "Cards are versatile containers, holding anything from images to headlines, supporting text, buttons, lists, and other components.",
        guideline: Some("https://m3.material.io/components/cards/overview"),
    };

    fn new(_: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Page { dropped: None }
    }

    fn row_count(&self, _: &Window) -> usize {
        4
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        if row == 3 {
            let live = self
                .dropped
                .clone()
                .unwrap_or_else(|| "Drop a file here".into());
            let specimen = |id: &'static str| {
                material_button((id, 1usize), ButtonVariant::Outlined)
                    .preview_state(InteractionPreview::Enabled)
            };
            // The pointer-free way in: the button opens the file picker.
            #[cfg(not(target_arch = "wasm32"))]
            {
                let choose = material_button(("drop-live", 1usize), ButtonVariant::Outlined)
                    .on_click(cx.listener(|_, _, _, cx| {
                        let picked = cx.prompt_for_paths(PathPromptOptions {
                            files: true,
                            directories: false,
                            multiple: true,
                            prompt: None,
                        });
                        cx.spawn(async move |page, cx| {
                            if let Ok(Ok(Some(paths))) = picked.await {
                                let _ = page.update(cx, |page, cx| {
                                    page.dropped = Some(file_names(&paths));
                                    cx.notify();
                                });
                            }
                        })
                        .detach();
                    }));
                return board("drop-target")
                    .caption("Drop target")
                    .columns(3)
                    .column_labels(["At rest", "Dragged over", "Live"])
                    .cells([
                        drop_target_specimen(
                            "drop-rest",
                            false,
                            "Drop a file here".into(),
                            specimen("drop-rest"),
                        ),
                        drop_target_specimen(
                            "drop-over",
                            true,
                            "Drop a file here".into(),
                            specimen("drop-over"),
                        ),
                        drop_target_specimen("drop-live", false, live, choose).on_drop(
                            cx.listener(|page, paths: &ExternalPaths, _, cx| {
                                page.dropped = Some(file_names(paths.paths()));
                                cx.notify();
                            }),
                        ),
                    ])
                    .into_any_element();
            }
            // A browser hands the page neither dropped files nor a file
            // picker's paths, so the web gallery shows the specimens only.
            #[cfg(target_arch = "wasm32")]
            {
                let _ = (live, cx);
                return board("drop-target")
                    .caption("Drop target (the live target is in the desktop gallery)")
                    .columns(2)
                    .column_labels(["At rest", "Dragged over"])
                    .cells([
                        drop_target_specimen(
                            "drop-rest",
                            false,
                            "Drop a file here".into(),
                            specimen("drop-rest"),
                        ),
                        drop_target_specimen(
                            "drop-over",
                            true,
                            "Drop a file here".into(),
                            specimen("drop-over"),
                        ),
                    ])
                    .into_any_element();
            }
        }
        if row == 2 {
            return board("card-states")
                .caption("Building blocks: card states")
                .columns(3)
                .column_labels(VARIANTS.map(|(_, name)| name))
                .row_labels(CardState::ALL.map(CardState::name))
                .cells(CardState::ALL.into_iter().flat_map(|state| {
                    VARIANTS.map(|(variant, _)| {
                        CardSurface::new(variant)
                            .state(state)
                            .w(rem(160.))
                            .h(rem(120.))
                    })
                }))
                .into_any_element();
        }
        let board = board(("cards", row))
            .columns(3)
            .column_labels(VARIANTS.map(|(_, name)| name));
        if row == 0 {
            board.caption("Stacked card").cells(
                VARIANTS
                    .into_iter()
                    .enumerate()
                    .map(|(i, (v, _))| vertical(i, v)),
            )
        } else {
            board.caption("Horizontal card").cells(
                VARIANTS
                    .into_iter()
                    .enumerate()
                    .map(|(i, (v, _))| horizontal(i, v)),
            )
        }
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_drop_target_icon_is_a_material_symbol() {
        assert!(Symbol::named("upload").is_some());
    }
}
