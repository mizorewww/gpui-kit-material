//! Cards: the outlined, elevated and filled cards composed like the kit's
//! specimens (header, media, text and actions), their horizontal form, and
//! the kit's card states (hover, focus, press, drag), selectable cards
//! (toggles) and a card group (one choice, a radio group).
use super::prelude::*;
use gpui_kit_material::icons::Symbol;

pub struct Page {
    /// Which of the selectable cards are selected.
    selected: [bool; 3],
    /// The card group's choice.
    choice: usize,
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
        Page {
            selected: [true, false, false],
            choice: 0,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        5
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        if row == 4 {
            let mut group = CardGroup::new("card-group", "Card group")
                .flex()
                .flex_wrap()
                .gap(rem(8.))
                .on_change(cx.listener(|page, index: &usize, _, cx| {
                    page.choice = *index;
                    cx.notify();
                }));
            for (i, name) in ["First", "Second", "Third"].into_iter().enumerate() {
                group = group.card(
                    MaterialCard::new(("card-choice", i), CardVariant::Outlined)
                        .selected(self.choice == i)
                        .accessibility_label(name)
                        .w(rem(160.))
                        .child(media().h(rem(72.)))
                        .child(div().p(rem(16.)).child(spec::type_role(
                            accessible_text(("card-choice-title", i), name),
                            TypeRole::TitleMedium,
                        ))),
                );
            }
            return board("card-choice")
                .caption("Card group: one choice")
                .columns(1)
                .cells([group])
                .into_any_element();
        }
        if row == 3 {
            return board("card-selectable")
                .caption("Selectable cards")
                .columns(3)
                .column_labels(VARIANTS.map(|(_, name)| name))
                .cells(
                    VARIANTS
                        .into_iter()
                        .enumerate()
                        .map(|(i, (variant, name))| {
                            MaterialCard::new(("card-select", i), variant)
                                .selected(self.selected[i])
                                .accessibility_label(name)
                                .on_click(cx.listener(move |page, _, _, cx| {
                                    page.selected[i] = !page.selected[i];
                                    cx.notify();
                                }))
                                .w(rem(160.))
                                .child(media().h(rem(72.)))
                                .child(
                                    div()
                                        .p(rem(16.))
                                        .debug_selector(move || format!("card-select-{i}"))
                                        .child(spec::type_role(
                                            accessible_text(("card-select-title", i), name),
                                            TypeRole::TitleMedium,
                                        )),
                                )
                        }),
                )
                .into_any_element();
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
