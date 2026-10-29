//! Motion: the Expressive and Standard schemes' spatial and effects springs.
use super::prelude::*;
use gpui_kit_material::motion::{self, MotionToken};

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
/// The first preview row.
pub(crate) const PREVIEW_ROW: usize = 2;

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "motion",
        title: "Motion",
        description: "Spatial and effects springs of the Expressive and Standard motion schemes.",
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
            r if r > PREVIEW_ROW => 12.,
            _ => spec::ROW_GAP,
        }
    }

    fn interactive(&self, row: usize) -> bool {
        row < PREVIEW_ROW
    }

    // Keep Base's keyed retargeting and pinned curves alive independently of
    // virtual rows. An offscreen target change still runs its finite
    // transition (and requests frames until settled); remounting never
    // restarts it.
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
        match row {
            0 => {
                let shared = self.page.shared.clone();
                spec::demo(
                    "motion-scheme-demo",
                    "Motion scheme",
                    ConnectedButtonGroup::new("motion-scheme", "Motion scheme")
                        .options([("expressive", "Expressive"), ("standard", "Standard")])
                        .selected(match self.page.shared.read(cx).motion_scheme {
                            motion::MotionScheme::Expressive => "expressive",
                            motion::MotionScheme::Standard => "standard",
                        })
                        .on_change(move |value: &SharedString, _, cx| {
                            // A scheme is not a theme: the Gallery re-renders
                            // without a theme transition.
                            shared.update(cx, |shared, cx| {
                                shared.motion_scheme = if value == "standard" {
                                    motion::MotionScheme::Standard
                                } else {
                                    motion::MotionScheme::Expressive
                                };
                                cx.notify();
                            })
                        })
                        .reveal("reveal-motion-scheme", &list),
                )
                .into_any_element()
            }
            1 => material_button("play-motion", ButtonVariant::Tonal)
                .child("Toggle motion preview")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.target = !this.target;
                    cx.notify();
                }))
                .reveal("reveal-play-motion", &list)
                .into_any_element(),
            _ => {
                let index = row - PREVIEW_ROW;
                let (label, _, effects) = TOKENS[index];
                let progress = self.values[index];
                div()
                    .flex()
                    .items_center()
                    .gap(rem(12.))
                    .child(
                        div()
                            .w(rem(160.))
                            .child(spec::small_label(("motion-label", index), label)),
                    )
                    .child(
                        div().w(rem(200.)).h(rem(24.)).child(
                            div()
                                .size(rem(24.))
                                .rounded_full()
                                .bg(color(ColorRole::PRIMARY))
                                .when(effects, |view| view.opacity(progress))
                                .when(!effects, |view| view.ml(rem(progress * 160.))),
                        ),
                    )
                    .into_any_element()
            }
        }
    }
}
