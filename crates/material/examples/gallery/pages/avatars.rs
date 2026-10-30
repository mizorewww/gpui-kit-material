//! Avatars: the kit's generic avatar styles (placeholder, monogram, check),
//! the same styles disabled, and the sizes the component scales to.
//!
//! The kit's 3D avatar illustrations are raster artwork that is not part of
//! this repository; image avatars take any caller-supplied image
//! (`Avatar::image`).
use super::prelude::*;

pub struct Page;

/// One avatar per kit style, in the kit's order.
fn styles(prefix: &'static str, disabled: bool, size: f32) -> [Avatar; 3] {
    [
        Avatar::new((prefix, 0usize), "Avatar placeholder"),
        Avatar::new((prefix, 1usize), "Monogram A").monogram("A"),
        Avatar::new((prefix, 2usize), "Selected").selected(true),
    ]
    .map(|avatar| avatar.disabled(disabled).size(size))
}

const SIZES: [f32; 6] = [24., 32., 40., 56., 72., 96.];

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "avatars",
        title: "Avatars",
        description: "Avatars, user images and 3D avatars.",
        guideline: Some("https://m3.material.io/styles/icons/overview"),
    };

    fn new(_: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Page
    }

    fn row_count(&self, _: &Window) -> usize {
        3
    }

    fn render_row(&mut self, row: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
        match row {
            0 => board("generic-avatar")
                .caption("Generic avatar")
                .columns(3)
                .column_width(96.)
                .column_labels(["Avatar", "Monogram", "Check"])
                .row_labels(["Enabled", "Disabled"])
                .cells(styles("avatar-enabled", false, AVATAR_SIZE))
                .cells(styles("avatar-disabled", true, AVATAR_SIZE))
                .into_any_element(),
            1 => board("avatar-sizes")
                .caption("Sizes · scaled from the 40dp kit avatar")
                .columns(SIZES.len())
                .column_labels(SIZES.map(|size| format!("{size}dp")))
                .row_labels(["Avatar", "Monogram", "Icon"])
                .cells((0..3).flat_map(|style| {
                    SIZES.into_iter().enumerate().map(move |(index, size)| {
                        let id = ("avatar-size", style * SIZES.len() + index);
                        let avatar = Avatar::new(id, "Avatar").size(size);
                        match style {
                            0 => avatar,
                            1 => avatar.monogram("JD"),
                            _ => avatar.icon(spec::star_icon()),
                        }
                    })
                }))
                .into_any_element(),
            _ => spec::demo(
                "avatar-in-use",
                "Avatars in use",
                div()
                    .w(rem(360.))
                    .max_w_full()
                    .flex()
                    .flex_col()
                    .bg(color(ColorRole::SURFACE))
                    .rounded(rem(12.))
                    .py(rem(8.))
                    .children(
                        [
                            ("Ali Connors", "Brunch this weekend?", "AC", false),
                            ("Sandra Adams", "Summer BBQ", "SA", true),
                            ("Trevor Hansen", "Oui oui", "TH", false),
                        ]
                        .into_iter()
                        .enumerate()
                        .map(
                            |(index, (name, message, initials, selected))| {
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(rem(16.))
                                    .px(rem(16.))
                                    .py(rem(8.))
                                    .child(
                                        Avatar::new(("avatar-row", index), name)
                                            .monogram(initials)
                                            .selected(selected),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                accessible_text(("avatar-name", index), name)
                                                    .map(|text| {
                                                        spec::type_role(text, TypeRole::BodyLarge)
                                                    })
                                                    .text_color(color(ColorRole::ON_SURFACE)),
                                            )
                                            .child(
                                                accessible_text(("avatar-message", index), message)
                                                    .map(|text| {
                                                        spec::type_role(text, TypeRole::BodyMedium)
                                                    })
                                                    .text_color(color(
                                                        ColorRole::ON_SURFACE_VARIANT,
                                                    )),
                                            ),
                                    )
                            },
                        ),
                    ),
            )
            .into_any_element(),
        }
    }
}
