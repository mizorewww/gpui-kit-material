//! Shape, like md3.io's Shape style page: the corner radius scale, with a
//! configuration panel (radius slider, the corners it rounds) driving a
//! live preview surface. Choosing a token sets the slider; the preview's
//! corners move on the default spatial spring.
use super::prelude::*;
use gpui_base::slider::{SliderEvent, SliderState};
use gpui_kit_material::motion::{self, MotionToken};

/// The scale, from none to full.
pub(crate) const TOKENS: [(&str, f32); 10] = [
    ("None", 0.),
    ("Extra small", 4.),
    ("Small", 8.),
    ("Medium", 12.),
    ("Large", 16.),
    ("Large increased", 20.),
    ("Extra large", 28.),
    ("Extra large increased", 32.),
    ("Extra extra large", 48.),
    ("Full", FULL),
];
/// The slider's top stop, which reads as full.
pub(crate) const FULL: f32 = 64.;
const CORNERS: [(&str, &str); 4] = [
    ("top-left", "Top left"),
    ("top-right", "Top right"),
    ("bottom-right", "Bottom right"),
    ("bottom-left", "Bottom left"),
];
const PREVIEW_WIDTH: f32 = 280.;
const PREVIEW_HEIGHT: f32 = 180.;
const TOKEN_WIDTH: f32 = 128.;

pub struct Page {
    page: PageCx,
    pub(crate) radius: f32,
    /// Which corners the radius rounds, in `CORNERS` order.
    pub(crate) corners: [bool; 4],
    pub(crate) slider: Entity<SliderState>,
    _subscription: Subscription,
}

/// The token a radius is, if any.
pub(crate) fn token_name(radius: f32) -> Option<&'static str> {
    TOKENS
        .iter()
        .find(|(_, value)| (*value - radius).abs() < 0.5)
        .map(|(name, _)| *name)
}

/// The painted radius: full rounds to half the preview's short side.
fn painted(radius: f32) -> f32 {
    if radius >= FULL {
        PREVIEW_HEIGHT / 2.
    } else {
        radius
    }
}

impl Page {
    fn set_radius(&mut self, radius: f32, window: &mut Window, cx: &mut Context<Self>) {
        self.radius = radius;
        self.slider
            .update(cx, |slider, cx| slider.set_value(radius, window, cx));
        cx.notify();
    }
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "shape",
        title: "Shape",
        description: "The corner scale: make your components squarer or rounder. Drag the radius or pick a token to preview it.",
        guideline: Some("https://m3.material.io/styles/shape/overview"),
    };

    fn new(page: PageCx, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let slider = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(FULL)
                .step(1.)
                .default_value(16.)
        });
        let subscription = cx.subscribe_in(
            &slider,
            window,
            |this: &mut Self, _, event: &SliderEvent, _, cx| {
                if let SliderEvent::Change(value) = event {
                    this.radius = value.start();
                    cx.notify();
                }
            },
        );
        Self {
            page,
            radius: 16.,
            corners: [true; 4],
            slider,
            _subscription: subscription,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        2
    }

    fn row_gap(&self, row: usize) -> f32 {
        if row == 0 { spec::HEADER_TO_BODY } else { 40. }
    }

    fn interactive(&self, _: usize) -> bool {
        true
    }

    fn render_row(
        &mut self,
        row: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let list = self.page.rows.list().clone();
        if row == 1 {
            return spec::demo(
                "shape-scale",
                "Corner radius scale",
                div().flex().flex_wrap().gap(rem(12.)).children(
                    TOKENS
                        .into_iter()
                        .enumerate()
                        .map(|(index, (name, value))| {
                            let selected = (self.radius - value).abs() < 0.5;
                            let label = if value >= FULL {
                                name.to_owned()
                            } else {
                                format!("{name} · {value}")
                            };
                            gpui_base::Button::new(("shape-token", index))
                                .accessibility_label(name)
                                .selected(selected)
                                .debug_selector(move || format!("shape-token-{index}"))
                                .w(rem(TOKEN_WIDTH))
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(rem(8.))
                                .p(rem(8.))
                                .rounded(rem(12.))
                                .cursor_pointer()
                                .when(selected, |token| {
                                    token.bg(color(ColorRole::SECONDARY_CONTAINER))
                                })
                                .child(
                                    div()
                                        .size(rem(72.))
                                        .rounded(rem(if value >= FULL { 36. } else { value }))
                                        .bg(color(ColorRole::PRIMARY_CONTAINER)),
                                )
                                .child(
                                    spec::small_label(("shape-token-label", index), label)
                                        .whitespace_normal()
                                        .text_center(),
                                )
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.set_radius(value, window, cx)
                                }))
                                .reveal(("reveal-shape-token", index), &list)
                        }),
                ),
            )
            .into_any_element();
        }
        let shown = motion::token_value(
            "shape-preview-radius",
            painted(self.radius),
            MotionToken::DefaultSpatial,
            window,
            cx,
        )
        .max(0.);
        let corner = |index: usize| rem(if self.corners[index] { shown } else { 0. });
        let value = token_name(self.radius).map_or_else(
            || format!("{}", self.radius.round()),
            |name| {
                if self.radius >= FULL {
                    name.to_owned()
                } else {
                    format!("{name} · {}", self.radius.round())
                }
            },
        );
        let configuration = gpui_base::v_flex()
            .w(rem(300.))
            .gap(rem(16.))
            .child(
                gpui_base::v_flex()
                    .gap(rem(4.))
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .child(spec::small_label("shape-radius-label", "Radius"))
                            .child(spec::small_label("shape-radius-value", value)),
                    )
                    .child(
                        div()
                            .debug_selector(|| "shape-radius-slider".into())
                            .child(MaterialSlider::new(&self.slider, "Radius").id("shape-radius"))
                            .reveal("reveal-shape-radius", &list),
                    ),
            )
            .child(spec::small_label("shape-corners-label", "Corners"))
            .child(div().flex().flex_wrap().gap(rem(8.)).children(
                CORNERS.into_iter().enumerate().map(|(index, (id, label))| {
                    FilterChip::new(id, label)
                        .selected(self.corners[index])
                        .on_toggle(cx.listener(move |this, selected: &bool, _, cx| {
                            this.corners[index] = *selected;
                            cx.notify();
                        }))
                        .reveal(("reveal-shape-corner", index), &list)
                }),
            ));
        let preview = div()
            .flex_1()
            .min_w(rem(PREVIEW_WIDTH))
            .h(rem(PREVIEW_HEIGHT + 64.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(rem(16.))
            .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
            .child(
                div()
                    .debug_selector(|| "shape-preview".into())
                    .w(rem(PREVIEW_WIDTH))
                    .h(rem(PREVIEW_HEIGHT))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(color(ColorRole::PRIMARY_CONTAINER))
                    .text_color(color(ColorRole::ON_PRIMARY_CONTAINER))
                    .rounded_tl(corner(0))
                    .rounded_tr(corner(1))
                    .rounded_br(corner(2))
                    .rounded_bl(corner(3))
                    .map(|text| spec::type_role(text, TypeRole::TitleMedium))
                    .child("Aa"),
            );
        div()
            .flex()
            .flex_wrap()
            .gap(rem(32.))
            .child(configuration)
            .child(preview)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn radii_name_their_tokens_and_full_rounds_the_short_side() {
        assert_eq!(token_name(12.), Some("Medium"));
        assert_eq!(token_name(13.), None);
        assert_eq!(token_name(FULL), Some("Full"));
        assert_eq!(painted(FULL), PREVIEW_HEIGHT / 2.);
        assert_eq!(painted(28.), 28.);
        // The token values are the shape tokens.
        assert_eq!(TOKENS[6].1, f32::from(RADIUS_EXTRA_LARGE));
        assert_eq!(TOKENS[8].1, f32::from(RADIUS_EXTRA_EXTRA_LARGE));
    }
}
