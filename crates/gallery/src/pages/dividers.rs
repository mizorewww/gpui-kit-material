//! Dividers: horizontal and vertical variants and a custom divider.
use super::prelude::*;

pub struct Page;

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "dividers",
        title: "Dividers",
        description: "Dividers are thin lines that group content in lists or other containers.",
        guideline: Some("https://m3.material.io/components/divider/overview"),
    };

    fn new(_: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Page
    }

    fn row_count(&self, _: &Window) -> usize {
        3
    }

    fn render_row(&mut self, row: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
        match row {
            0 => board("horizontal-dividers")
                .caption("Horizontal")
                .row_labels([
                    "Full width",
                    "Inset",
                    "Middle inset",
                    "Inset · RTL",
                    "With subhead",
                ])
                .cells(
                    [
                        (DividerVariant::FullWidth, DividerDirection::Ltr),
                        (DividerVariant::Inset, DividerDirection::Ltr),
                        (DividerVariant::MiddleInset, DividerDirection::Ltr),
                        (DividerVariant::Inset, DividerDirection::Rtl),
                    ]
                    .into_iter()
                    .enumerate()
                    .map(|(index, (variant, direction))| {
                        div()
                            .w(rem(320.))
                            .h(rem(24.))
                            .flex()
                            .flex_col()
                            .justify_center()
                            .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
                            .child(
                                MaterialDivider::new(("horizontal-divider", index))
                                    .variant(variant)
                                    .direction(direction),
                            )
                            .into_any_element()
                    })
                    // The kit's divider with subhead: a middle-inset divider
                    // over a title-small subheader, 4dp apart.
                    .chain([div()
                        .w(rem(320.))
                        .flex()
                        .flex_col()
                        .gap(rem(4.))
                        .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
                        .child(
                            MaterialDivider::new("subhead-divider")
                                .variant(DividerVariant::MiddleInset),
                        )
                        .child(
                            spec::type_role(
                                accessible_text("subheader", "Subheader"),
                                TypeRole::TitleSmall,
                            )
                            .px(rem(16.))
                            .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                        )
                        .into_any_element()]),
                )
                .into_any_element(),
            1 => board("vertical-dividers")
                .caption("Vertical")
                .columns(3)
                .column_labels(["Full height", "Inset", "Middle inset"])
                .cells(
                    [
                        DividerVariant::FullWidth,
                        DividerVariant::Inset,
                        DividerVariant::MiddleInset,
                    ]
                    .into_iter()
                    .enumerate()
                    .map(|(index, variant)| {
                        div()
                            .flex()
                            .items_stretch()
                            .h(rem(72.))
                            .w(rem(48.))
                            .justify_center()
                            .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
                            .child(
                                MaterialDivider::new(("vertical-divider", index))
                                    .orientation(DividerOrientation::Vertical)
                                    .variant(variant),
                            )
                    }),
                )
                .into_any_element(),
            _ => board("custom-divider-board")
                .caption("Custom: primary color, 2px thickness, 24px / 8px insets")
                .cell(
                    div().w(rem(320.)).child(
                        MaterialDivider::new("custom-divider")
                            .variant(DividerVariant::MiddleInset)
                            .inset_start(px(24.))
                            .inset_end(px(8.))
                            .thickness(px(2.))
                            .color_role(ColorRole::PRIMARY),
                    ),
                )
                .into_any_element(),
        }
    }
}
