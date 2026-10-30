//! The kit's "Layout breakpoints" section (`Examples/Layout grid`): one
//! window per M3 window size class, without and with a navigation region,
//! drawn to scale with its layout columns. The windows wrap to the page
//! width instead of scrolling sideways.
use super::super::prelude::*;

/// One window size class: the kit's frame size and the M3 layout columns.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SizeClass {
    pub name: &'static str,
    /// The kit's frame, in dp.
    pub size: (f32, f32),
    pub columns: usize,
    pub margin: f32,
    pub gutter: f32,
    /// The navigation region: a bottom bar for compact windows, a rail
    /// beside the content otherwise.
    pub navigation: Navigation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Navigation {
    Bar,
    Rail,
}

/// The kit's frames (`Window size class=…`); compact's columns are the
/// kit's (4 × 70dp in 328dp: 16dp margins and gutters). Larger classes
/// follow the M3 layout guidance: 24dp margins and gutters with 8 columns
/// for medium and 12 beyond.
pub const SIZE_CLASSES: [SizeClass; 5] = [
    SizeClass {
        name: "Compact (0-599dp)",
        size: (360., 640.),
        columns: 4,
        margin: 16.,
        gutter: 16.,
        navigation: Navigation::Bar,
    },
    SizeClass {
        name: "Medium (600-839dp)",
        size: (600., 800.),
        columns: 8,
        margin: 24.,
        gutter: 24.,
        navigation: Navigation::Rail,
    },
    SizeClass {
        name: "Expanded (840-1199dp)",
        size: (905., 680.),
        columns: 12,
        margin: 24.,
        gutter: 24.,
        navigation: Navigation::Rail,
    },
    SizeClass {
        name: "Large (1200-1599dp)",
        size: (1280., 720.),
        columns: 12,
        margin: 24.,
        gutter: 24.,
        navigation: Navigation::Rail,
    },
    SizeClass {
        name: "Extra-large (1600+dp)",
        size: (1600., 900.),
        columns: 12,
        margin: 24.,
        gutter: 24.,
        navigation: Navigation::Rail,
    },
];

/// The navigation bar's and the collapsed rail's thickness in dp.
const NAVIGATION: f32 = 80.;
/// The windows are drawn at a quarter of their size.
const SCALE: f32 = 0.25;

impl SizeClass {
    /// The width of one layout column in dp, inside `width` dp of content.
    pub fn column_width(&self, width: f32) -> f32 {
        let columns = self.columns as f32;
        ((width - 2. * self.margin - (columns - 1.) * self.gutter) / columns).max(0.)
    }
}

fn window_frame(id: (&'static str, usize), class: SizeClass, navigation: bool) -> AnyElement {
    let (width, height) = class.size;
    let scaled = |dp: f32| rem(dp * SCALE);
    let content_width = if navigation && class.navigation == Navigation::Rail {
        width - NAVIGATION
    } else {
        width
    };
    let column = class.column_width(content_width);
    let columns = div()
        .flex_1()
        .h_full()
        .flex()
        .px(scaled(class.margin))
        .gap(scaled(class.gutter))
        .children((0..class.columns).map(move |_| {
            div()
                .w(scaled(column))
                .h_full()
                .bg(color(ColorRole::TERTIARY).opacity(0.12))
        }));
    let region = || {
        div()
            .flex_shrink_0()
            .bg(color(ColorRole::SURFACE_CONTAINER))
    };
    div()
        .id(id)
        .w(scaled(width))
        .h(scaled(height))
        .flex_shrink_0()
        .flex()
        .when(class.navigation == Navigation::Bar, |frame| {
            frame.flex_col()
        })
        .overflow_hidden()
        .rounded(rem(4.))
        .border_1()
        .border_color(color(ColorRole::OUTLINE_VARIANT))
        .bg(color(ColorRole::SURFACE))
        .when(
            navigation && class.navigation == Navigation::Rail,
            |frame| frame.child(region().w(scaled(NAVIGATION)).h_full()),
        )
        .child(columns)
        .when(navigation && class.navigation == Navigation::Bar, |frame| {
            frame.child(region().h(scaled(NAVIGATION)).w_full())
        })
        .into_any_element()
}

/// The section: each size class captioned, its plain window over its
/// window with a navigation region; the classes wrap to the row width.
pub fn layout_grid() -> AnyElement {
    spec::demo(
        "layout-grid",
        "Layout breakpoints · window size classes",
        spec::demo_row().items_start().gap(rem(32.)).children(
            SIZE_CLASSES.into_iter().enumerate().map(|(index, class)| {
                div()
                    .flex()
                    .flex_col()
                    .gap(rem(8.))
                    .child(
                        accessible_text(("layout-class", index), class.name)
                            .map(|text| spec::type_role(text, TypeRole::LabelMedium))
                            .text_color(color(ColorRole::ON_SURFACE)),
                    )
                    .child(
                        accessible_text(
                            ("layout-columns", index),
                            format!(
                                "{} columns · {}dp margins",
                                class.columns, class.margin as u32
                            ),
                        )
                        .map(|text| spec::type_role(text, TypeRole::BodySmall))
                        .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                    )
                    .child(window_frame(("layout-window", index), class, false))
                    .child(window_frame(("layout-window-nav", index), class, true))
            }),
        ),
    )
    .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn compact_columns_match_the_kit() {
        // The kit's compact grid: four 70dp columns.
        assert_eq!(SIZE_CLASSES[0].column_width(360.), 70.);
    }

    #[test]
    fn size_classes_cover_the_m3_breakpoints_in_order() {
        let widths: Vec<f32> = SIZE_CLASSES.iter().map(|class| class.size.0).collect();
        assert!(widths.windows(2).all(|pair| pair[0] < pair[1]));
        for class in SIZE_CLASSES {
            assert!(class.column_width(class.size.0 - NAVIGATION) > 0.);
        }
    }
}
