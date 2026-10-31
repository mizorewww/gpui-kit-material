//! Examples: one running example app — Home, Upcoming, Detailed view,
//! Reviews, Library, Gallery and Messaging from the kit's example layouts —
//! on one device at a time. The configuration panel (see
//! `playgrounds::examples`) picks the device and the screen; [`app`] holds
//! the app and [`screens`] its screens, whose every control is a crate
//! component and works. Photos are the kit's tonal placeholders. The Specs
//! tab is this page: the kit's layout breakpoints.
use super::prelude::*;

pub mod app;
mod layout_grid;
mod parts;
mod screens;

const CANONICAL: usize = 1;

/// The Examples page's rows on its Specs tab.
pub fn layout_grid_rows() -> Vec<usize> {
    vec![0]
}
/// The canonical layouts' rows on their Specs tab.
pub fn canonical_rows() -> Vec<usize> {
    vec![CANONICAL]
}

/// One pane of a canonical layout's diagram.
fn pane(role: ColorRole, label: &'static str, share: f32) -> impl IntoElement {
    div()
        .flex_grow(share)
        .flex_basis(px(0.))
        .min_w_0()
        .h_full()
        .p(rem(8.))
        .rounded(rem(12.))
        .bg(color(role))
        .overflow_hidden()
        .map(|text| spec::type_role(text, TypeRole::LabelMedium))
        .text_color(color(ColorRole::ON_SURFACE))
        .child(label)
}

/// How each canonical layout arranges its panes in each window size class,
/// after M3's canonical layouts: the panes that show, in proportion.
fn canonical_board() -> AnyElement {
    type Panes = &'static [(ColorRole, &'static str, f32)];
    const LIST: (ColorRole, &str, f32) = (ColorRole::SECONDARY_CONTAINER, "List", 1.);
    const DETAIL: (ColorRole, &str, f32) = (ColorRole::SURFACE_CONTAINER_HIGHEST, "Detail", 1.);
    const MAIN: (ColorRole, &str, f32) = (ColorRole::SURFACE_CONTAINER_HIGHEST, "Main", 2.);
    const SUPPORT: (ColorRole, &str, f32) = (ColorRole::TERTIARY_CONTAINER, "Side", 1.);
    const FEED: (ColorRole, &str, f32) = (ColorRole::SURFACE_CONTAINER_HIGHEST, "Feed", 1.);
    const LAYOUTS: [[Panes; 3]; 3] = [
        [&[LIST], &[LIST, DETAIL], &[LIST, DETAIL]],
        [&[MAIN], &[MAIN, SUPPORT], &[MAIN, SUPPORT]],
        [&[FEED], &[FEED, FEED], &[FEED, FEED, FEED, FEED]],
    ];
    // The diagrams' widths, in proportion to the size classes.
    const WIDTHS: [f32; 3] = [72., 168., 240.];
    board("canonical-layouts")
        .caption("Canonical layouts")
        .columns(3)
        .column_width(WIDTHS[2])
        .column_labels(["Compact", "Medium", "Expanded"])
        .row_labels(["List-detail", "Supporting pane", "Feed"])
        .cells(LAYOUTS.into_iter().flat_map(|panes| {
            panes.into_iter().zip(WIDTHS).map(|(panes, width)| {
                div()
                    .w(rem(width))
                    .h(rem(120.))
                    .p(rem(4.))
                    .flex()
                    .gap(rem(4.))
                    .rounded(rem(16.))
                    .border_1()
                    .border_color(color(ColorRole::OUTLINE_VARIANT))
                    .children(
                        panes
                            .iter()
                            .map(|(role, label, share)| pane(*role, label, *share)),
                    )
            })
        }))
        .into_any_element()
}

pub struct Page {
    _page: PageCx,
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "examples",
        title: "Examples",
        description: "Example layouts are meant to be a starting point, not a final destination. Feel free to modify them, experiment, and add your own unique touch to create a design that is truly your own.",
        guideline: Some(
            "https://m3.material.io/foundations/layout/applying-layout/window-size-classes",
        ),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self { _page: page }
    }

    fn row_count(&self, _: &Window) -> usize {
        CANONICAL + 1
    }

    fn render_row(&mut self, row: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
        if row == CANONICAL {
            canonical_board()
        } else {
            layout_grid::layout_grid()
        }
    }
}
