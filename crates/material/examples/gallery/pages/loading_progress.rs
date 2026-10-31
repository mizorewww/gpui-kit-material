//! Loading & progress: the kit's progress indicators (linear and circular,
//! flat and wavy, 4dp and 8dp, determinate and indeterminate) and the M3
//! Expressive loading indicator's seven shapes, plus live indicators.
use super::prelude::*;

pub struct Page {
    linear: [RepeatAnimationPlayback; 4],
    circular_live: [RepeatAnimationPlayback; 4],
    loading: [RepeatAnimationPlayback; 2],
    spinner: RepeatAnimationPlayback,
    /// The button-loading arc's clock (the reference's 24dp canvas).
    pub(crate) circular: CircularProgressPlayback,
}

/// The circular indeterminate row, which holds the button-loading arc.
pub(crate) const CIRCULAR_ROW: usize = 3;

const LINEAR_WIDTH: f32 = 360.;
const LINEAR_STEPS: [f32; 6] = [0., 0.1, 0.2, 0.5, 0.8, 1.];
const CIRCULAR_STEPS: [f32; 6] = [0., 0.1, 0.3, 0.5, 0.8, 1.];
/// The kit's four columns: flat/wavy x 4dp/8dp.
const VARIANTS: [(ProgressShape, bool, &str); 4] = [
    (ProgressShape::Flat, false, "Flat · 4dp"),
    (ProgressShape::Flat, true, "Flat · 8dp"),
    (ProgressShape::Wavy, false, "Wave · 4dp"),
    (ProgressShape::Wavy, true, "Wave · 8dp"),
];

fn percent(value: f32) -> String {
    format!("{}%", (value * 100.).round())
}

fn linear(id: impl Into<ElementId>, shape: ProgressShape, thick: bool) -> LinearProgress {
    let bar = LinearProgress::new(id, "Progress").shape(shape);
    if thick { bar.thick() } else { bar }
}

fn circular(id: impl Into<ElementId>, shape: ProgressShape, thick: bool) -> CircularProgress {
    let ring = CircularProgress::new(id, "Progress").shape(shape);
    if thick { ring.thick() } else { ring }
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "loading-progress",
        title: "Loading & progress",
        description: "Progress indicators inform users about the status of ongoing processes, such as loading an app, submitting a form, or saving updates. Loading indicators use animation to mitigate perceived latency when progress isn't detectable.",
        guideline: Some("https://m3.material.io/components/progress-indicators/overview"),
    };

    fn new(_: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            linear: Default::default(),
            circular_live: Default::default(),
            loading: Default::default(),
            spinner: RepeatAnimationPlayback::default(),
            circular: CircularProgressPlayback::default(),
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        5
    }

    const PREPARES: bool = true;

    fn prepare(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.circular.observe_reduced_motion(cx);
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match row {
            0 => board("linear-determinate")
                .caption("Linear · Determinate")
                .columns(2)
                .column_labels(["4dp", "8dp"])
                .row_labels(
                    [ProgressShape::Flat, ProgressShape::Wavy]
                        .into_iter()
                        .flat_map(|shape| {
                            let name = if shape == ProgressShape::Flat {
                                "Flat"
                            } else {
                                "Wave"
                            };
                            LINEAR_STEPS.map(move |step| format!("{name} · {}", percent(step)))
                        }),
                )
                .cells(
                    [ProgressShape::Flat, ProgressShape::Wavy]
                        .into_iter()
                        .enumerate()
                        .flat_map(|(s, shape)| {
                            LINEAR_STEPS
                                .into_iter()
                                .enumerate()
                                .flat_map(move |(i, step)| {
                                    [false, true].map(move |thick| {
                                        div().w(rem(LINEAR_WIDTH)).child(
                                            linear(
                                                ("linear", s * 100 + i * 2 + usize::from(thick)),
                                                shape,
                                                thick,
                                            )
                                            .value(Some(step)),
                                        )
                                    })
                                })
                        }),
                )
                .into_any_element(),
            1 => board("linear-indeterminate")
                .caption("Linear · Indeterminate")
                .columns(2)
                .column_labels(["4dp", "8dp"])
                .row_labels(["Flat", "Wave"])
                .cells(
                    VARIANTS
                        .into_iter()
                        .enumerate()
                        .map(|(i, (shape, thick, _))| {
                            div().w(rem(LINEAR_WIDTH)).child(
                                linear(("linear-indeterminate", i), shape, thick)
                                    .playback(self.linear[i].clone()),
                            )
                        }),
                )
                .into_any_element(),
            2 => board("circular-determinate")
                .caption("Circular · Determinate")
                .columns(4)
                .column_labels(VARIANTS.map(|(_, _, name)| name))
                .row_labels(CIRCULAR_STEPS.map(percent))
                .cells(
                    CIRCULAR_STEPS
                        .into_iter()
                        .enumerate()
                        .flat_map(|(i, step)| {
                            VARIANTS
                                .into_iter()
                                .enumerate()
                                .map(move |(v, (shape, thick, _))| {
                                    circular(("circular", i * 4 + v), shape, thick)
                                        .value(Some(step))
                                })
                        }),
                )
                .into_any_element(),
            CIRCULAR_ROW => board("circular-indeterminate")
                .caption("Circular · Indeterminate")
                .columns(5)
                .column_labels(
                    VARIANTS
                        .map(|(_, _, name)| name)
                        .into_iter()
                        .chain(["Button loading"]),
                )
                .cells(
                    VARIANTS
                        .into_iter()
                        .enumerate()
                        .map(|(i, (shape, thick, _))| {
                            circular(("circular-indeterminate", i), shape, thick)
                                .show_track(true)
                                .playback(self.circular_live[i].clone())
                                .into_any_element()
                        }),
                )
                .cell(indeterminate_circular_progress_with_playback(
                    "circular-animated",
                    "Loading",
                    size(rem(24.), rem(24.)),
                    1.,
                    &self.circular,
                    cx,
                ))
                .into_any_element(),
            _ => board("loading-indicator")
                .caption("Loading indicator · Indeterminate")
                .columns(2)
                .column_labels(["Uncontained", "Contained"])
                .row_labels(
                    (1..=LoadingShape::SEQUENCE.len())
                        .map(|step| format!("Step {step}"))
                        .chain(["Live".into(), "Spinner".into()]),
                )
                .cells(
                    LoadingShape::SEQUENCE
                        .into_iter()
                        .enumerate()
                        .flat_map(|(i, shape)| {
                            [false, true].map(move |contained| {
                                LoadingIndicator::new(
                                    ("loading-step", i * 2 + usize::from(contained)),
                                    "Loading",
                                )
                                .contained(contained)
                                .shape(shape)
                                .into_any_element()
                            })
                        }),
                )
                .cells([false, true].map(|contained| {
                    LoadingIndicator::new(("loading-live", usize::from(contained)), "Loading")
                        .contained(contained)
                        .playback(self.loading[usize::from(contained)].clone())
                        .into_any_element()
                }))
                .cell(icon_spinner_with_playback(
                    "label-spinner",
                    "Loading content",
                    ICON_SLOT,
                    self.spinner.clone(),
                    cx,
                ))
                .into_any_element(),
        }
    }
}
