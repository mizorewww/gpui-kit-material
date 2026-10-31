//! Elevation: the six levels on live surfaces. A configuration row picks a
//! level for the preview surface, whose shadow and tonal container follow
//! on the default spatial spring, and toggles the shadow and the tonal
//! color; the level cards below rise one level under the pointer.
use super::prelude::*;
use gpui_base::slider::{SliderEvent, SliderState};
use gpui_kit_material::motion::{self, MotionToken};
use gpui_kit_material::tokens::elevation;

/// Each level's height in dp and its tonal surface.
pub(crate) const LEVELS: [(f32, ColorRole); 6] = [
    (0., ColorRole::SURFACE),
    (1., ColorRole::SURFACE_CONTAINER_LOW),
    (3., ColorRole::SURFACE_CONTAINER),
    (6., ColorRole::SURFACE_CONTAINER_HIGH),
    (8., ColorRole::SURFACE_CONTAINER_HIGHEST),
    (12., ColorRole::SURFACE_CONTAINER_HIGHEST),
];
const ROLE_NAMES: [&str; 6] = [
    "surface",
    "surface-container-low",
    "surface-container",
    "surface-container-high",
    "surface-container-highest",
    "surface-container-highest",
];
const CARD_WIDTH: f32 = 152.;

pub struct Page {
    page: PageCx,
    pub(crate) level: u8,
    pub(crate) shadow: bool,
    pub(crate) tonal: bool,
    slider: Entity<SliderState>,
    _subscription: Subscription,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// The shadow `level` (fractional while animating) casts: the layers of the
/// levels around it, blended.
pub(crate) fn shadow_at(level: f32) -> Vec<BoxShadow> {
    let level = level.clamp(0., elevation::MAX_LEVEL as f32);
    let (low, high) = (level.floor() as u8, level.ceil() as u8);
    let t = level - level.floor();
    elevation::layers(low)
        .into_iter()
        .zip(elevation::layers(high))
        .map(|(a, b)| {
            let mut shadow = a.clone();
            shadow.offset = point(
                px(lerp(f32::from(a.offset.x), f32::from(b.offset.x), t)),
                px(lerp(f32::from(a.offset.y), f32::from(b.offset.y), t)),
            );
            shadow.blur_radius = px(lerp(f32::from(a.blur_radius), f32::from(b.blur_radius), t));
            shadow.spread_radius = px(lerp(
                f32::from(a.spread_radius),
                f32::from(b.spread_radius),
                t,
            ));
            // Level 0's layers are transparent: take the other's color.
            let color = if a.color.a == 0. { b.color } else { a.color };
            shadow.color = Hsla {
                a: lerp(a.color.a, b.color.a, t),
                ..color
            };
            shadow
        })
        .collect()
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "elevation",
        title: "Elevation",
        description: "Shadow levels and tonal surfaces, from flat to the highest level, on live surfaces.",
        guideline: Some("https://m3.material.io/styles/elevation/overview"),
    };

    fn new(page: PageCx, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let slider = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(elevation::MAX_LEVEL as f32)
                .step(1.)
                .default_value(3.)
        });
        let subscription = cx.subscribe_in(
            &slider,
            window,
            |this: &mut Self, _, event: &SliderEvent, _, cx| {
                if let SliderEvent::Change(value) = event {
                    this.level = value.start().round() as u8;
                    cx.notify();
                }
            },
        );
        Self {
            page,
            level: 3,
            shadow: true,
            tonal: true,
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

    fn interactive(&self, row: usize) -> bool {
        row == 0
    }

    fn render_row(
        &mut self,
        row: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let list = self.page.rows.list().clone();
        let (shadows, tonal) = (self.shadow, self.tonal);
        let fill = move |level: usize| {
            color(if tonal {
                LEVELS[level].1
            } else {
                ColorRole::SURFACE_CONTAINER_LOW
            })
        };
        if row == 1 {
            return spec::demo(
                "elevation-levels",
                "Levels",
                div()
                    .flex()
                    .flex_wrap()
                    .gap(rem(24.))
                    .p(rem(16.))
                    .rounded(rem(16.))
                    .bg(color(ColorRole::SURFACE))
                    .children(LEVELS.iter().enumerate().map(|(level, (dp, _))| {
                        let raised = (level + 1).min(elevation::MAX_LEVEL as usize);
                        div()
                            .id(("elevation-level", level))
                            .w(rem(CARD_WIDTH))
                            .h(rem(112.))
                            .p(rem(12.))
                            .flex()
                            .flex_col()
                            .justify_end()
                            .rounded(RADIUS_MEDIUM)
                            .bg(fill(level))
                            // Level 0 is the surface itself: outline it.
                            .when(level == 0, |card| {
                                card.border_1()
                                    .border_color(color(ColorRole::OUTLINE_VARIANT))
                            })
                            .when(shadows, |card| {
                                card.shadow(elevation::level(level as u8))
                                    .hover(move |style| {
                                        style.shadow(elevation::level(raised as u8))
                                    })
                            })
                            .child(
                                spec::small_label(
                                    ("elevation-name", level),
                                    format!("Level {level} · {dp}dp"),
                                )
                                .text_color(color(ColorRole::ON_SURFACE)),
                            )
                            .child(
                                spec::small_label(("elevation-role", level), ROLE_NAMES[level])
                                    .whitespace_normal()
                                    .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                            )
                    })),
            )
            .into_any_element();
        }
        let shown = motion::token_value(
            "elevation-preview-level",
            self.level as f32,
            MotionToken::DefaultSpatial,
            window,
            cx,
        );
        let switch =
            |id: &'static str, label: &'static str, checked: bool, set: fn(&mut Self, bool)| {
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(rem(40.))
                    .child(spec::small_label(id, label))
                    .child(
                        MaterialSwitch::new(id, label)
                            .checked(checked)
                            .on_change(cx.listener(move |this, on: &bool, _, cx| {
                                set(this, *on);
                                cx.notify();
                            }))
                            .reveal(id, &list),
                    )
            };
        let level = self.level as usize;
        let configuration = gpui_base::v_flex()
            .w(rem(300.))
            .gap(rem(8.))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(spec::small_label("elevation-level-label", "Level"))
                    .child(spec::small_label(
                        "elevation-level-value",
                        format!("{level} · {}dp", LEVELS[level].0),
                    )),
            )
            .child(
                div()
                    .debug_selector(|| "elevation-level-slider".into())
                    .child(MaterialSlider::new(&self.slider, "Level").id("elevation-level"))
                    .reveal("reveal-elevation-level", &list),
            )
            .child(switch(
                "elevation-shadow",
                "Shadow",
                self.shadow,
                |this, on| this.shadow = on,
            ))
            .child(switch(
                "elevation-tonal",
                "Tonal surface",
                self.tonal,
                |this, on| this.tonal = on,
            ));
        let preview = div()
            .flex_1()
            .min_w(rem(280.))
            .h(rem(240.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(rem(16.))
            .bg(color(ColorRole::SURFACE))
            .child(
                div()
                    .debug_selector(|| "elevation-preview".into())
                    .w(rem(240.))
                    .h(rem(150.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(RADIUS_LARGE)
                    .bg(fill(level))
                    .when(shadows, |surface| surface.shadow(shadow_at(shown)))
                    .map(|text| spec::type_role(text, TypeRole::TitleMedium))
                    .text_color(color(ColorRole::ON_SURFACE))
                    .child(format!("Level {level}")),
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
    fn a_fractional_level_blends_its_neighbours() {
        let whole = shadow_at(2.);
        let exact = elevation::layers(2);
        assert_eq!(whole[1].blur_radius, exact[1].blur_radius);
        let half = shadow_at(2.5);
        let (a, b) = (elevation::layers(2), elevation::layers(3));
        let mid = (f32::from(a[1].blur_radius) + f32::from(b[1].blur_radius)) / 2.;
        assert!((f32::from(half[1].blur_radius) - mid).abs() < 1e-4);
        // Rising from level 0 fades the shadow in rather than snapping.
        let low = shadow_at(0.25);
        assert!(low[0].color.a > 0. && low[0].color.a < elevation::layers(1)[0].color.a);
    }
}
