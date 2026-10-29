//! Sliders: the expressive and classic sliders over one value.
use super::prelude::*;

pub struct Page {
    page: PageCx,
    pub(crate) slider: Entity<gpui_base::slider::SliderState>,
    _drag: Subscription,
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "sliders",
        title: "Sliders",
        description: "Sliders let people make selections from a range of values.",
        guideline: Some("https://m3.material.io/components/sliders/overview"),
    };

    fn new(page: PageCx, _: &mut Window, cx: &mut Context<Self>) -> Self {
        let slider = cx.new(|_| {
            gpui_base::slider::SliderState::new()
                .min(0.)
                .max(100.)
                .default_value(40.)
        });
        // The slider state is not a view; notify this page (the narrowest
        // owner the cached Gallery observes) so the sliders repaint.
        let drag = cx.observe(&slider, |_, _, cx| cx.notify());
        Self {
            page,
            slider,
            _drag: drag,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        2
    }

    fn interactive(&self, _: usize) -> bool {
        true
    }

    fn render_row(&mut self, row: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
        let list = self.page.rows.list();
        if row == 0 {
            spec::demo(
                "expressive-slider",
                "Expressive slider",
                MaterialSlider::new(&self.slider, "Expressive slider")
                    .reveal("reveal-slider", list),
            )
            .into_any_element()
        } else {
            spec::demo(
                "classic-slider",
                "Classic slider",
                MaterialSlider::new(&self.slider, "Classic slider")
                    .appearance(SliderAppearance::Classic)
                    .reveal("reveal-slider-classic", list),
            )
            .into_any_element()
        }
    }
}
