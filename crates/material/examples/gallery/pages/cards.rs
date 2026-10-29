//! Cards: the elevated, filled and outlined cards composed like the kit's
//! specimens (header, media, text and actions), and their horizontal form.
use super::prelude::*;

pub struct Page;

const VARIANTS: [(CardVariant, &str); 3] = [
    (CardVariant::Elevated, "Elevated"),
    (CardVariant::Filled, "Filled"),
    (CardVariant::Outlined, "Outlined"),
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

fn header(index: usize) -> Div {
    div()
        .flex()
        .items_center()
        .gap(rem(16.))
        .pl(rem(16.))
        .pr(rem(4.))
        .py(rem(12.))
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
}

fn media() -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .bg(color(ColorRole::SURFACE_CONTAINER_HIGHEST))
        .child(
            spec::star_icon()
                .size(rem(48.))
                .color(color(ColorRole::OUTLINE_VARIANT)),
        )
}

fn vertical(index: usize, variant: CardVariant) -> impl IntoElement {
    CardSurface::new(variant)
        .w(rem(CARD_WIDTH))
        .child(header(index))
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

fn horizontal(index: usize, variant: CardVariant) -> impl IntoElement {
    CardSurface::new(variant)
        .w(rem(CARD_WIDTH))
        .flex_row()
        .child(header(index + 10).flex_1().min_w_0())
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
        Page
    }

    fn row_count(&self, _: &Window) -> usize {
        2
    }

    fn render_row(&mut self, row: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
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
