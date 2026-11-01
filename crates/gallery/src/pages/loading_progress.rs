//! Loading & progress: the kit's progress indicators (linear and circular,
//! flat and wavy, 4dp and 8dp, determinate and indeterminate) and the M3
//! Expressive loading indicator's seven shapes, plus live indicators.
use super::prelude::*;

const LOADING_INDICATOR: usize = 4;
const PULL_TO_REFRESH: usize = 5;
const STEPPER: usize = 6;

/// The progress indicators' rows on the Progress page's Specs tab.
pub fn progress_rows() -> Vec<usize> {
    (0..LOADING_INDICATOR).chain([STEPPER]).collect()
}
/// The loading indicator's rows on its Specs tab.
pub fn loading_indicator_rows() -> Vec<usize> {
    vec![LOADING_INDICATOR]
}
/// Pull to refresh's rows on its Specs tab.
pub fn pull_to_refresh_rows() -> Vec<usize> {
    vec![PULL_TO_REFRESH]
}

/// Pull to refresh at rest and refreshing, for each indicator. The kit has no
/// pull-to-refresh page: these specimens are the real component over a list.
fn pull_to_refresh_board() -> AnyElement {
    const INDICATORS: [PullToRefreshIndicator; 2] = [
        PullToRefreshIndicator::Default,
        PullToRefreshIndicator::LoadingIndicator,
    ];
    board("pull-to-refresh")
        .caption("Pull to refresh")
        .columns(2)
        .column_width(240.)
        .column_labels(["Default indicator", "Loading indicator"])
        .row_labels(["At rest", "Refreshing"])
        .cells([false, true].into_iter().flat_map(|refreshing| {
            INDICATORS
                .into_iter()
                .enumerate()
                .map(move |(ix, indicator)| {
                    let id = ix * 2 + usize::from(refreshing);
                    let mut list = PullToRefresh::new(("pull-specimen", id), "Refreshing items")
                        .indicator(indicator)
                        .refreshing(refreshing);
                    for item in 1..=3 {
                        list = list.child(
                            ListItem::new(
                                SharedString::from(format!("pull-specimen-{id}-{item}")),
                                SharedString::from(format!("Item {item}")),
                            )
                            .container(ColorRole::SURFACE),
                        );
                    }
                    div()
                        .w(rem(240.))
                        .h(rem(200.))
                        .overflow_hidden()
                        .rounded(rem(12.))
                        .border_1()
                        .border_color(color(ColorRole::OUTLINE_VARIANT))
                        .bg(color(ColorRole::SURFACE))
                        .child(list)
                })
        }))
        .into_any_element()
}

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
        STEPPER + 1
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
            PULL_TO_REFRESH => pull_to_refresh_board(),
            STEPPER => spec::demo(
                "stepper",
                "Stepper",
                gpui_base::v_flex()
                    .w_full()
                    .gap(rem(24.))
                    .children((0..3).map(|current| {
                        Stepper::new(("stepper", current), "Setup progress")
                            .steps(["Source", "Service", "Account"])
                            .current(current)
                    })),
            )
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
