//! Shape, like md3.io's Shape style page: the corner radius scale, with a
//! configuration panel (radius slider, the corners it rounds) driving a
//! live preview surface. Choosing a token sets the slider; the preview's
//! corners move on the default spatial spring.
//!
//! Below it, the M3 Expressive shape library, which md3.io has no page for:
//! click any of the 35 shapes to morph to it, or scrub between the last two
//! with the slider.
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
/// The library's tiles: small enough for three across at the narrowest
/// window, wide enough for "12-sided cookie" on two lines.
const SHAPE_TILE_WIDTH: f32 = 96.;
const SHAPE_TILE_SIZE: f32 = 56.;
const MORPH_PREVIEW_SIZE: f32 = 180.;

/// The library in the order and with the names of the M3 shape chart
/// (<https://developer.android.com/images/reference/androidx/compose/material3/shapes.png>).
pub(crate) const LIBRARY: [(ShapeKind, &str); 35] = [
    (ShapeKind::Circle, "Circle"),
    (ShapeKind::Square, "Square"),
    (ShapeKind::Slanted, "Slanted"),
    (ShapeKind::Arch, "Arch"),
    (ShapeKind::SemiCircle, "Semicircle"),
    (ShapeKind::Oval, "Oval"),
    (ShapeKind::Pill, "Pill"),
    (ShapeKind::Triangle, "Triangle"),
    (ShapeKind::Arrow, "Arrow"),
    (ShapeKind::Fan, "Fan"),
    (ShapeKind::Diamond, "Diamond"),
    (ShapeKind::ClamShell, "Clamshell"),
    (ShapeKind::Pentagon, "Pentagon"),
    (ShapeKind::Gem, "Gem"),
    (ShapeKind::Sunny, "Sunny"),
    (ShapeKind::VerySunny, "Very sunny"),
    (ShapeKind::Cookie4Sided, "4-sided cookie"),
    (ShapeKind::Cookie6Sided, "6-sided cookie"),
    (ShapeKind::Cookie7Sided, "7-sided cookie"),
    (ShapeKind::Cookie9Sided, "9-sided cookie"),
    (ShapeKind::Cookie12Sided, "12-sided cookie"),
    (ShapeKind::Clover4Leaf, "4-leaf clover"),
    (ShapeKind::Clover8Leaf, "8-leaf clover"),
    (ShapeKind::Burst, "Burst"),
    (ShapeKind::SoftBurst, "Soft burst"),
    (ShapeKind::Boom, "Boom"),
    (ShapeKind::SoftBoom, "Soft boom"),
    (ShapeKind::Flower, "Flower"),
    (ShapeKind::Puffy, "Puffy"),
    (ShapeKind::PuffyDiamond, "Puffy diamond"),
    (ShapeKind::Ghostish, "Ghost-ish"),
    (ShapeKind::PixelCircle, "Pixel circle"),
    (ShapeKind::PixelTriangle, "Pixel triangle"),
    (ShapeKind::Bun, "Bun"),
    (ShapeKind::Heart, "Heart"),
];

fn library_name(kind: ShapeKind) -> &'static str {
    LIBRARY
        .iter()
        .find(|(candidate, _)| *candidate == kind)
        .map_or("", |(_, name)| name)
}

pub struct Page {
    page: PageCx,
    pub(crate) radius: f32,
    /// Which corners the radius rounds, in `CORNERS` order.
    pub(crate) corners: [bool; 4],
    pub(crate) slider: Entity<SliderState>,
    /// The morph demo's ends: the slider scrubs from `morph_from` to
    /// `morph_to`, and a click on the library makes the clicked shape the
    /// next `morph_to`.
    pub(crate) morph_from: ShapeKind,
    pub(crate) morph_to: ShapeKind,
    /// `Some` while the slider, not a spring, says how far the morph is.
    pub(crate) scrub: Option<f32>,
    pub(crate) morph_slider: Entity<SliderState>,
    _subscriptions: [Subscription; 2],
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

    /// Morph on the spring to `kind`, from the shape last shown.
    pub(crate) fn morph_to(&mut self, kind: ShapeKind, cx: &mut Context<Self>) {
        if kind != self.morph_to {
            self.morph_from = self.morph_to;
            self.morph_to = kind;
        }
        self.scrub = None;
        cx.notify();
    }

    /// Morph back to the other end.
    pub(crate) fn swap_morph(&mut self, cx: &mut Context<Self>) {
        std::mem::swap(&mut self.morph_from, &mut self.morph_to);
        self.scrub = None;
        cx.notify();
    }

    fn render_morph(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let list = self.page.rows.list().clone();
        let shape = match self.scrub {
            Some(progress) => MaterialShape::new("shape-morph-preview", self.morph_to)
                .frame(ShapeFrame::morph(self.morph_from, self.morph_to, progress)),
            None => MaterialShape::new("shape-morph-preview", self.morph_to).animated(true),
        };
        let progress = self.scrub.map_or_else(
            || "Spring".to_owned(),
            |progress| format!("{}%", (progress * 100.).round()),
        );
        let configuration = gpui_base::v_flex()
            .w(rem(300.))
            .flex_shrink(1.)
            .gap(rem(16.))
            .child(spec::small_label(
                "shape-morph-ends",
                format!(
                    "{} to {}",
                    library_name(self.morph_from),
                    library_name(self.morph_to)
                ),
            ))
            .child(
                gpui_base::v_flex()
                    .gap(rem(4.))
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .child(spec::small_label("shape-morph-progress-label", "Progress"))
                            .child(spec::small_label("shape-morph-progress-value", progress)),
                    )
                    .child(
                        div()
                            .debug_selector(|| "shape-morph-slider".into())
                            .child(
                                MaterialSlider::new(&self.morph_slider, "Morph progress")
                                    .id("shape-morph-progress"),
                            )
                            .reveal("reveal-shape-morph-progress", &list),
                    ),
            )
            .child(
                div()
                    .debug_selector(|| "shape-morph-swap".into())
                    .child(
                        MaterialButton::new("shape-morph-swap")
                            .child("Morph back")
                            .on_click(cx.listener(|this, _, _, cx| this.swap_morph(cx))),
                    )
                    .reveal("reveal-shape-morph-swap", &list),
            );
        div()
            .flex()
            .flex_wrap()
            .gap(rem(32.))
            .child(configuration)
            .child(
                div()
                    .flex_1()
                    .min_w(rem(MORPH_PREVIEW_SIZE + 64.))
                    .h(rem(MORPH_PREVIEW_SIZE + 64.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(rem(16.))
                    .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
                    .child(
                        div().debug_selector(|| "shape-morph-preview".into()).child(
                            shape
                                .size(MORPH_PREVIEW_SIZE)
                                .label(library_name(self.morph_to)),
                        ),
                    ),
            )
            .into_any_element()
    }

    fn render_library(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let list = self.page.rows.list().clone();
        let current = self.morph_to;
        spec::demo(
            "shape-library",
            "Shape library",
            div()
                .flex()
                .flex_wrap()
                .gap(rem(8.))
                .children(
                    LIBRARY
                        .into_iter()
                        .enumerate()
                        .map(|(index, (kind, name))| {
                            let selected = kind == current;
                            gpui_base::Button::new(("shape-library-tile", index))
                                .accessibility_label(name)
                                .selected(selected)
                                .debug_selector(move || format!("shape-library-{index}"))
                                .w(rem(SHAPE_TILE_WIDTH))
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(rem(8.))
                                .p(rem(8.))
                                .rounded(rem(12.))
                                .cursor_pointer()
                                .when(selected, |tile| {
                                    tile.bg(color(ColorRole::SECONDARY_CONTAINER))
                                })
                                .child(
                                    MaterialShape::new(("shape-library", index), kind)
                                        .size(SHAPE_TILE_SIZE),
                                )
                                .child(
                                    spec::small_label(("shape-library-label", index), name)
                                        .whitespace_normal()
                                        .text_center(),
                                )
                                .on_click(
                                    cx.listener(move |this, _, _, cx| this.morph_to(kind, cx)),
                                )
                                .reveal(("reveal-shape-library", index), &list)
                        }),
                ),
        )
        .into_any_element()
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
        let morph_slider = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(100.)
                .step(1.)
                .default_value(100.)
        });
        let morph_subscription = cx.subscribe_in(
            &morph_slider,
            window,
            |this: &mut Self, _, event: &SliderEvent, _, cx| {
                if let SliderEvent::Change(value) = event {
                    this.scrub = Some(value.start() / 100.);
                    cx.notify();
                }
            },
        );
        Self {
            page,
            radius: 16.,
            corners: [true; 4],
            slider,
            morph_from: ShapeKind::Circle,
            morph_to: ShapeKind::Cookie9Sided,
            scrub: None,
            morph_slider,
            _subscriptions: [subscription, morph_subscription],
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        4
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
        match row {
            2 => return self.render_morph(cx),
            3 => return self.render_library(cx),
            _ => {}
        }
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

    #[test]
    fn the_library_lists_every_shape_once_with_a_name() {
        for kind in ShapeKind::ALL {
            let entries = LIBRARY.iter().filter(|(k, _)| *k == kind).count();
            assert_eq!(entries, 1, "{kind:?}");
            assert!(!library_name(kind).is_empty(), "{kind:?}");
        }
    }

    fn open(
        cx: &mut gpui::TestAppContext,
        width: f32,
    ) -> (
        WindowHandle<super::super::super::Gallery>,
        gpui::VisualTestContext,
        Entity<Page>,
    ) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let window = cx.open_window(
            size(px(width), px(3200.)),
            super::super::super::Gallery::new,
        );
        let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
        let page = window
            .update(&mut visual, |g, window, cx| {
                g.show_page::<Page>(window, cx).0
            })
            .expect("window is open");
        settle(&window, &mut visual);
        (window, visual, page)
    }

    fn settle(
        window: &WindowHandle<super::super::super::Gallery>,
        cx: &mut gpui::VisualTestContext,
    ) {
        for _ in 0..3 {
            cx.update_window((*window).into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .expect("window is open");
            cx.run_until_parked();
        }
    }

    #[gpui::test]
    fn clicking_a_shape_morphs_to_it_and_the_button_morphs_back(cx: &mut gpui::TestAppContext) {
        let (window, mut cx, page) = open(cx, 1280.);
        let heart = LIBRARY
            .iter()
            .position(|(kind, _)| *kind == ShapeKind::Heart)
            .expect("listed");
        let tile = cx
            .debug_bounds(Box::leak(format!("shape-library-{heart}").into_boxed_str()))
            .expect("the heart is rendered");
        cx.simulate_click(tile.center(), Modifiers::none());
        settle(&window, &mut cx);
        let ends = cx.read(|cx| {
            let page = page.read(cx);
            (page.morph_from, page.morph_to, page.scrub)
        });
        assert_eq!(ends, (ShapeKind::Cookie9Sided, ShapeKind::Heart, None));

        let swap = cx.debug_bounds("shape-morph-swap").expect("rendered");
        cx.simulate_click(swap.center(), Modifiers::none());
        settle(&window, &mut cx);
        let ends = cx.read(|cx| (page.read(cx).morph_from, page.read(cx).morph_to));
        assert_eq!(ends, (ShapeKind::Heart, ShapeKind::Cookie9Sided));
    }

    #[gpui::test]
    fn the_slider_scrubs_until_a_shape_is_clicked(cx: &mut gpui::TestAppContext) {
        let (window, mut cx, page) = open(cx, 1280.);
        let slider = cx.debug_bounds("shape-morph-slider").expect("rendered");
        cx.simulate_click(slider.center(), Modifiers::none());
        settle(&window, &mut cx);
        let scrub = cx.read(|cx| page.read(cx).scrub);
        let scrub = scrub.expect("the slider took over");
        assert!((0.3..=0.7).contains(&scrub), "{scrub}");

        page.update(&mut cx, |page, cx| page.morph_to(ShapeKind::Bun, cx));
        settle(&window, &mut cx);
        assert_eq!(cx.read(|cx| page.read(cx).scrub), None);
    }

    #[gpui::test]
    fn the_library_reflows_at_a_narrow_window(cx: &mut gpui::TestAppContext) {
        let (_, mut cx, _) = open(cx, 420.);
        let first = cx.debug_bounds("shape-library-0").expect("rendered");
        let last = cx.debug_bounds("shape-library-34").expect("rendered");
        assert!(last.top() > first.top(), "the tiles wrap");
        assert!(last.right() <= px(420.), "{last:?}");
        let preview = cx.debug_bounds("shape-morph-preview").expect("rendered");
        assert!(preview.right() <= px(420.), "{preview:?}");
    }
}
