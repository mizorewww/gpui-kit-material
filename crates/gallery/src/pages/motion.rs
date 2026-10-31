//! Motion: the Expressive and Standard schemes' spatial and effects
//! springs. A configuration row picks the scheme, toggles reduced motion
//! and plays every token; each token shows its spring parameters, its
//! curve beside the other scheme's, and a live lane: spatial tokens move a
//! ball across its track, effects tokens fade a square.
use super::prelude::*;
use gpui_kit_material::motion::{self, MotionScheme, MotionToken};

pub struct Page {
    page: PageCx,
    pub(crate) target: bool,
    pub(crate) values: [f32; 6],
}

pub(crate) const TOKENS: [(&str, MotionToken, bool); 6] = [
    ("Fast spatial", MotionToken::FastSpatial, false),
    ("Default spatial", MotionToken::DefaultSpatial, false),
    ("Slow spatial", MotionToken::SlowSpatial, false),
    ("Fast effects", MotionToken::FastEffects, true),
    ("Default effects", MotionToken::DefaultEffects, true),
    ("Slow effects", MotionToken::SlowEffects, true),
];
/// The first token row.
pub(crate) const PREVIEW_ROW: usize = 1;
const BALL: f32 = 24.;
const PLOT_WIDTH: f32 = 160.;
const PLOT_HEIGHT: f32 = 64.;

/// "Damping 0.9 · stiffness 1400 · 229ms" for a Standard spring, or
/// "Sampled curve · 425ms" for an Expressive md3.io curve.
pub(crate) fn describe(scheme: MotionScheme, token: MotionToken) -> String {
    let curve = scheme.resolve(token);
    let millis = curve.duration().as_millis();
    match curve.spring() {
        Some(spring) => format!(
            "Damping {} · stiffness {} · {millis}ms",
            spring.damping, spring.stiffness
        ),
        None => format!("Sampled curve · {millis}ms"),
    }
}

/// A spring's 0 → 1 position over its duration, as a plot. The y range
/// leaves room for overshoot.
fn plot(scheme: MotionScheme, token: MotionToken, ink: Rgba, path_width: f32) -> impl IntoElement {
    let ease = scheme.resolve(token).ease();
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
            let y = |value: f32| h - h * 0.15 - value * h * 0.7;
            let mut builder = PathBuilder::stroke(px(path_width));
            for step in 0..=48 {
                let t = step as f32 / 48.;
                let at = bounds.origin + point(px(t * w), px(y(ease(t))));
                if step == 0 {
                    builder.move_to(at);
                } else {
                    builder.line_to(at);
                }
            }
            if let Ok(path) = builder.build() {
                window.paint_path(path, ink);
            }
        },
    )
    .absolute()
    .inset_0()
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "motion",
        title: "Motion",
        description: "Spatial and effects springs of the Expressive and Standard motion schemes. Play them, compare their curves, and try reduced motion.",
        guideline: Some("https://m3.material.io/styles/motion/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            page,
            target: false,
            values: [0.; 6],
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        PREVIEW_ROW + TOKENS.len()
    }

    fn row_gap(&self, row: usize) -> f32 {
        match row {
            0 => spec::HEADER_TO_BODY,
            PREVIEW_ROW => 32.,
            _ => 16.,
        }
    }

    fn interactive(&self, row: usize) -> bool {
        row < PREVIEW_ROW
    }

    // Keep Base's keyed retargeting and pinned curves alive independently of
    // virtual rows. An offscreen target change still runs its finite
    // transition (and requests frames until settled); remounting never
    // restarts it.
    const PREPARES: bool = true;

    fn prepare(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let target = if self.target { 1. } else { 0. };
        self.values = std::array::from_fn(|index| {
            motion::token_value(
                ("motion-preview", index),
                target,
                TOKENS[index].1,
                window,
                cx,
            )
        });
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let list = self.page.rows.list().clone();
        let scheme = self.page.shared.read(cx).motion_scheme;
        if row == 0 {
            let shared = self.page.shared.clone();
            let reduced = cx.reduce_motion();
            return div()
                .flex()
                .flex_wrap()
                .items_end()
                .gap(rem(24.))
                .child(spec::demo(
                    "motion-scheme-demo",
                    "Motion scheme",
                    ConnectedButtonGroup::new("motion-scheme", "Motion scheme")
                        .options([("expressive", "Expressive"), ("standard", "Standard")])
                        .selected(match scheme {
                            MotionScheme::Expressive => "expressive",
                            MotionScheme::Standard => "standard",
                        })
                        .on_change(move |value: &SharedString, _, cx| {
                            // A scheme is not a theme: the Gallery re-renders
                            // without a theme transition.
                            shared.update(cx, |shared, cx| {
                                shared.motion_scheme = if value == "standard" {
                                    MotionScheme::Standard
                                } else {
                                    MotionScheme::Expressive
                                };
                                cx.notify();
                            })
                        })
                        .reveal("reveal-motion-scheme", &list),
                ))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(rem(12.))
                        .h(rem(40.))
                        .child(spec::small_label("reduced-motion-label", "Reduced motion"))
                        .child(
                            MaterialSwitch::new("reduced-motion", "Reduced motion")
                                .checked(reduced)
                                .on_change(cx.listener(|_, on: &bool, _, cx| {
                                    cx.set_reduce_motion(*on);
                                    cx.notify();
                                }))
                                .reveal("reveal-reduced-motion", &list),
                        ),
                )
                .child(
                    material_button("play-motion", ButtonVariant::Filled)
                        .icon(super::super::playground::symbol("play_arrow", 20.))
                        .child(if self.target { "Play back" } else { "Play" })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.target = !this.target;
                            cx.notify();
                        }))
                        .reveal("reveal-play-motion", &list),
                )
                .into_any_element();
        }
        let index = row - PREVIEW_ROW;
        let (label, token, effects) = TOKENS[index];
        let progress = self.values[index];
        let other = match scheme {
            MotionScheme::Expressive => MotionScheme::Standard,
            MotionScheme::Standard => MotionScheme::Expressive,
        };
        let lane = div()
            .flex_1()
            .min_w(rem(160.))
            .h(rem(PLOT_HEIGHT))
            .px(rem(12.))
            .flex()
            .items_center()
            .rounded(rem(12.))
            .bg(color(ColorRole::SURFACE_CONTAINER))
            .child(if effects {
                div()
                    .size(rem(40.))
                    .rounded(rem(8.))
                    .bg(color(ColorRole::PRIMARY))
                    .opacity(progress.clamp(0., 1.))
                    .into_any_element()
            } else {
                // The ball's left edge travels the track less its own width.
                div()
                    .relative()
                    .flex_1()
                    .h(rem(BALL))
                    .mr(rem(BALL))
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left(relative(progress))
                            .size(rem(BALL))
                            .rounded_full()
                            .bg(color(ColorRole::PRIMARY)),
                    )
                    .into_any_element()
            });
        div()
            .debug_selector(move || format!("motion-token-{index}"))
            .flex()
            .flex_wrap()
            .items_center()
            .gap(rem(16.))
            .child(
                gpui_base::v_flex()
                    .w(rem(220.))
                    .gap(rem(2.))
                    .child(
                        spec::small_label(("motion-label", index), label)
                            .text_color(color(ColorRole::ON_SURFACE)),
                    )
                    .child(
                        spec::small_label(("motion-spring", index), describe(scheme, token))
                            .whitespace_normal()
                            .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                    ),
            )
            .child(
                div()
                    .relative()
                    .w(rem(PLOT_WIDTH))
                    .h(rem(PLOT_HEIGHT))
                    .rounded(rem(12.))
                    .border_1()
                    .border_color(color(ColorRole::OUTLINE_VARIANT))
                    .child(plot(other, token, color(ColorRole::OUTLINE_VARIANT), 1.5))
                    .child(plot(scheme, token, color(ColorRole::PRIMARY), 2.5)),
            )
            .child(lane)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn each_token_describes_its_scheme_curve() {
        let expressive = describe(MotionScheme::Expressive, MotionToken::FastSpatial);
        let standard = describe(MotionScheme::Standard, MotionToken::FastSpatial);
        assert_eq!(expressive, "Sampled curve · 425ms");
        assert!(standard.starts_with("Damping "), "{standard}");
        assert_ne!(expressive, standard, "the schemes' spatial springs differ");
        assert!(describe(MotionScheme::Expressive, MotionToken::SlowEffects).ends_with("ms"));
    }
}
