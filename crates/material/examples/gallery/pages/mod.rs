//! The page registry: every design-kit page in Figma order, foundations
//! first. This is the only file pages share; each page module owns its
//! state and rows (see `super::page`).
use super::page::{PageEntry, entry};

pub mod app_bars;
pub mod avatars;
pub mod badges;
pub mod buttons;
pub mod cards;
pub mod carousel;
pub mod checkboxes;
pub mod chips;
pub mod color;
pub mod date_time_pickers;
pub mod dialogs;
pub mod dividers;
pub mod elevation;
pub mod examples;
pub mod lists;
pub mod loading_progress;
pub mod menu;
pub mod motion;
pub mod navigation;
pub mod radio_button;
pub mod search;
pub mod shape;
pub mod sheets;
pub mod sliders;
pub mod snackbar;
pub mod switch;
pub mod tabs;
pub mod text_fields;
pub mod toolbars;
pub mod tooltips;
pub mod typography;

/// The common imports of page modules.
pub mod prelude {
    pub(crate) use super::super::GalleryReveal;
    pub(crate) use super::super::page::{
        CaptureState, GalleryPage, GalleryShared, PageCx, PageInfo,
    };
    pub(crate) use super::super::spec::{self, Band, board};
    pub use gpui::{prelude::*, *};
    pub use gpui_kit_material::icons::IconName;
    pub use gpui_kit_material::prelude::*;
}

pub static PAGES: &[PageEntry] = &[
    entry::<color::Page>(),
    entry::<typography::Page>(),
    entry::<shape::Page>(),
    entry::<motion::Page>(),
    entry::<elevation::Page>(),
    entry::<examples::Page>(),
    entry::<app_bars::Page>(),
    entry::<avatars::Page>(),
    entry::<badges::Page>(),
    entry::<buttons::Page>(),
    entry::<cards::Page>(),
    entry::<carousel::Page>(),
    entry::<checkboxes::Page>(),
    entry::<chips::Page>(),
    entry::<date_time_pickers::Page>(),
    entry::<dialogs::Page>(),
    entry::<dividers::Page>(),
    entry::<lists::Page>(),
    entry::<loading_progress::Page>(),
    entry::<menu::Page>(),
    entry::<navigation::Page>(),
    entry::<radio_button::Page>(),
    entry::<search::Page>(),
    entry::<sheets::Page>(),
    entry::<sliders::Page>(),
    entry::<snackbar::Page>(),
    entry::<switch::Page>(),
    entry::<tabs::Page>(),
    entry::<text_fields::Page>(),
    entry::<toolbars::Page>(),
    entry::<tooltips::Page>(),
];

#[cfg(test)]
mod tests {
    use super::PAGES;
    use core::prelude::v1::test;

    #[test]
    fn registry_lists_every_figma_page_in_order_with_foundations_first() {
        let titles: Vec<_> = PAGES.iter().map(|page| page.info.title).collect();
        assert_eq!(
            titles,
            [
                "Color",
                "Typography",
                "Shape",
                "Motion",
                "Elevation",
                "Examples",
                "App bars",
                "Avatars",
                "Badges",
                "Buttons",
                "Cards",
                "Carousel",
                "Checkboxes",
                "Chips",
                "Date & time pickers",
                "Dialogs",
                "Dividers",
                "Lists",
                "Loading & progress",
                "Menu",
                "Navigation",
                "Radio button",
                "Search",
                "Sheets",
                "Sliders",
                "Snackbar",
                "Switch",
                "Tabs",
                "Text fields",
                "Toolbars",
                "Tooltips",
            ]
        );
        let mut ids: Vec<_> = PAGES.iter().map(|page| page.info.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), PAGES.len(), "page ids are unique");
        for page in PAGES {
            assert!(!page.info.description.is_empty(), "{}", page.info.id);
            assert!(
                page.info
                    .guideline
                    .is_some_and(|url| url.starts_with("https://m3.material.io/")),
                "{}",
                page.info.id
            );
        }
    }
}
