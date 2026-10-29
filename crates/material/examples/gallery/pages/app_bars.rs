//! App bars: no M3 top or bottom app bar yet. The Gallery's window title bar
//! and caption buttons, the app-drawn bar this kit has, are shown instead.
use super::prelude::*;

pub struct Page {
    page: PageCx,
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "app-bars",
        title: "App bars",
        description: "App bars display navigation, actions, and text at the top or bottom of a screen.",
        guideline: Some("https://m3.material.io/components/app-bars/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self { page }
    }

    fn row_count(&self, _: &Window) -> usize {
        1
    }

    fn interactive(&self, _: usize) -> bool {
        true
    }

    fn render_row(&mut self, _: usize, window: &mut Window, _: &mut Context<Self>) -> AnyElement {
        let caption = match Caption::detect(window) {
            Caption::System => "Caption buttons: drawn by the system",
            Caption::Windows => "Caption buttons: Windows style, drawn by the app",
            Caption::Linux => "Caption buttons: Linux style, drawn by the app",
        };
        spec::demo(
            "window-controls",
            "Window title bar",
            div()
                .flex()
                .flex_col()
                .gap(rem(12.))
                .child(accessible_text("caption-style", caption))
                .child(
                    // Inline previews of the app-drawn styles: inert and in the
                    // page's flow, not over the real title bar.
                    spec::demo_row().children(
                        [
                            (Caption::Windows, "caption-preview-windows", "Windows"),
                            (Caption::Linux, "caption-preview-linux", "Linux"),
                        ]
                        .into_iter()
                        .enumerate()
                        .map(|(index, (caption, id, name))| {
                            div()
                                .flex()
                                .flex_col()
                                .gap(rem(8.))
                                .child(spec::small_label(("caption-preview-name", index), name))
                                .child(
                                    div()
                                        .debug_selector(move || id.to_string())
                                        .flex()
                                        .rounded(RADIUS_SMALL)
                                        .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
                                        .child(
                                            CaptionButtons::new(
                                                px(48.),
                                                super::super::caption_labels(),
                                            )
                                            .caption(caption)
                                            .preview(id),
                                        ),
                                )
                        }),
                    ),
                )
                .child(
                    // A sample bar: its control takes the press itself instead
                    // of starting a window drag.
                    div()
                        .flex()
                        .h(rem(56.))
                        .px(rem(16.))
                        .gap(rem(12.))
                        .items_center()
                        .rounded(RADIUS_SMALL)
                        .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
                        .child(accessible_text("sample-title", "Sample title bar"))
                        .child(div().flex_1())
                        .child(
                            title_bar_control(
                                material_button("sample-bar-settings", ButtonVariant::Text)
                                    .button_size(ButtonSize::ExtraSmall)
                                    .child("Settings")
                                    .on_click(|_, window, cx| {
                                        super::super::settings::open(window, cx)
                                    }),
                            )
                            .reveal("reveal-sample-bar-settings", self.page.rows.list()),
                        ),
                ),
        )
        .into_any_element()
    }
}
