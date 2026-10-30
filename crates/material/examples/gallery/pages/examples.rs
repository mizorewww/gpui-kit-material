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
        1
    }

    fn render_row(&mut self, _: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
        layout_grid::layout_grid()
    }
}
