//! Search: the search field.
use super::prelude::*;

pub struct Page {
    page: PageCx,
    pub(crate) search: Entity<gpui_base::input::InputState>,
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "search",
        title: "Search",
        description: "Search lets people enter a keyword or phrase to get relevant information.",
        guideline: Some("https://m3.material.io/components/search/overview"),
    };

    fn new(page: PageCx, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| {
            gpui_base::input::InputState::new(window, cx).placeholder("Search components")
        });
        Self { page, search }
    }

    fn row_count(&self, _: &Window) -> usize {
        1
    }

    fn interactive(&self, _: usize) -> bool {
        true
    }

    fn render_row(&mut self, _: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
        spec::demo(
            "search-demo",
            "Search field",
            SearchField::new(&self.search).reveal("reveal-search", self.page.rows.list()),
        )
        .into_any_element()
    }
}
