//! Sliders: the kit's standard, centered and range sliders in every size and
//! state, horizontal and vertical, and live sliders.
use super::prelude::*;
use gpui_base::slider::SliderState;
use gpui_kit_material::icons::Symbol;

/// The kit's state columns.
const STATES: [InteractionPreview; 4] = [
    InteractionPreview::Enabled,
    InteractionPreview::Hovered,
    InteractionPreview::Pressed,
    InteractionPreview::Disabled,
];
const LENGTH: f32 = 168.;
const VERTICAL_LENGTH: f32 = 180.;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Standard,
    Centered,
    Range,
}

pub struct Page {
    page: PageCx,
    /// Specimen values per kind: three each, as the kit's rows.
    specimens: [[Entity<SliderState>; 3]; 3],
    pub(crate) slider: Entity<gpui_base::slider::SliderState>,
    live: [Entity<SliderState>; 4],
    _observers: Vec<Subscription>,
}

fn state(cx: &mut App, value: impl Into<gpui_base::slider::SliderValue>) -> Entity<SliderState> {
    let value = value.into();
    cx.new(|_| SliderState::new().min(0.).max(100.).default_value(value))
}

impl Page {
    fn specimen(
        &self,
        kind: Kind,
        size: SliderSize,
        preview: InteractionPreview,
        index: usize,
        vertical: bool,
    ) -> MaterialSlider {
        let entity = &self.specimens[kind as usize][index];
        let slider = MaterialSlider::new(entity, "Specimen")
            .id(SharedString::from(format!(
                "slider-{}-{}-{}-{index}-{vertical}",
                kind as usize,
                size.name(),
                preview.name()
            )))
            .size(size)
            .centered(kind == Kind::Centered)
            .preview_state(preview);
        if vertical { slider.vertical() } else { slider }
    }

    fn board(&self, kind: Kind, vertical: bool, caption: &'static str) -> AnyElement {
        board((caption, usize::from(vertical)))
            .caption(caption)
            .columns(STATES.len())
            .column_labels(STATES.map(|s| s.name()))
            .row_labels(SliderSize::ALL.map(|s| s.name()))
            .cells(SliderSize::ALL.into_iter().flat_map(|size| {
                STATES.map(|preview| {
                    let sliders = (0..3).map(|i| {
                        let slider = self.specimen(kind, size, preview, i, vertical);
                        if vertical {
                            div().h(rem(VERTICAL_LENGTH)).child(slider)
                        } else {
                            div().w(rem(LENGTH)).child(slider)
                        }
                    });
                    if vertical {
                        div().flex().gap(rem(16.)).pl(rem(52.)).children(sliders)
                    } else {
                        div()
                            .flex()
                            .flex_col()
                            .gap(rem(44.))
                            .pt(rem(40.))
                            .children(sliders)
                    }
                })
            }))
            .into_any_element()
    }
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "sliders",
        title: "Sliders",
        description: "Sliders let users make selections from a range of values.",
        guideline: Some("https://m3.material.io/components/sliders/overview"),
    };

    fn new(page: PageCx, _: &mut Window, cx: &mut Context<Self>) -> Self {
        let specimens = [
            [state(cx, 50.), state(cx, 100.), state(cx, 0.)],
            [state(cx, 50.), state(cx, 25.), state(cx, 75.)],
            [
                state(cx, (40., 60.)),
                state(cx, (25., 50.)),
                state(cx, (50., 75.)),
            ],
        ];
        let slider = state(cx, 40.);
        let live = [
            cx.new(|_| {
                SliderState::new()
                    .min(0.)
                    .max(100.)
                    .step(10.)
                    .default_value(30.)
            }),
            state(cx, 70.),
            state(cx, (20., 60.)),
            state(cx, 60.),
        ];
        // Slider states are not views; notify this page (the narrowest
        // owner the cached Gallery observes) so the sliders repaint.
        let observers = std::iter::once(&slider)
            .chain(live.iter())
            .map(|entity| cx.observe(entity, |_, _, cx| cx.notify()))
            .collect();
        Self {
            page,
            specimens,
            slider,
            live,
            _observers: observers,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        6
    }

    fn interactive(&self, row: usize) -> bool {
        row == 5
    }

    fn render_row(&mut self, row: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
        match row {
            0 => self.board(Kind::Standard, false, "Standard · Horizontal"),
            1 => self.board(Kind::Standard, true, "Standard · Vertical"),
            2 => self.board(Kind::Centered, false, "Centered · Horizontal"),
            3 => self.board(Kind::Centered, true, "Centered · Vertical"),
            4 => self.board(Kind::Range, false, "Range · Horizontal"),
            _ => {
                let list = self.page.rows.list();
                let labelled = |label: &'static str, slider: AnyElement| {
                    div()
                        .flex()
                        .flex_col()
                        .gap(rem(8.))
                        .pt(rem(48.))
                        .w(rem(280.))
                        .child(spec::small_label(label, label))
                        .child(slider)
                };
                spec::demo(
                    "live-sliders",
                    "Live sliders",
                    spec::demo_row()
                        .items_start()
                        .child(labelled(
                            "Medium with icon",
                            MaterialSlider::new(&self.slider, "Volume")
                                .size(SliderSize::Medium)
                                .icon(
                                    Symbol::named("volume_up")
                                        .expect("volume_up is a Material Symbol"),
                                )
                                .reveal("reveal-slider", list)
                                .into_any_element(),
                        ))
                        .child(labelled(
                            "Discrete",
                            MaterialSlider::new(&self.live[0], "Discrete")
                                .size(SliderSize::Small)
                                .stops(true)
                                .reveal("reveal-discrete", list)
                                .into_any_element(),
                        ))
                        .child(labelled(
                            "Centered",
                            MaterialSlider::new(&self.live[1], "Centered")
                                .centered(true)
                                .reveal("reveal-centered", list)
                                .into_any_element(),
                        ))
                        .child(labelled(
                            "Range",
                            MaterialSlider::new(&self.live[2], "Range")
                                .reveal("reveal-range", list)
                                .into_any_element(),
                        ))
                        .child(labelled(
                            "Classic",
                            MaterialSlider::new(&self.slider, "Classic slider")
                                .appearance(SliderAppearance::Classic)
                                .reveal("reveal-slider-classic", list)
                                .into_any_element(),
                        ))
                        .child(
                            div().pl(rem(56.)).h(rem(220.)).child(
                                MaterialSlider::new(&self.live[3], "Vertical")
                                    .size(SliderSize::Large)
                                    .vertical()
                                    .reveal("reveal-vertical", list),
                            ),
                        ),
                )
                .into_any_element()
            }
        }
    }
}
