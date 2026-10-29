//! Loading & progress: linear and circular indicators, determinate and not.
use super::prelude::*;

pub struct Page {
    spinner: RepeatAnimationPlayback,
    linear: RepeatAnimationPlayback,
    pub(crate) circular: CircularProgressPlayback,
}

/// The circular indicators' row.
pub(crate) const CIRCULAR_ROW: usize = 1;

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "loading-progress",
        title: "Loading & progress",
        description: "Loading and progress indicators show the status of a process, such as loading an app or submitting a form.",
        guideline: Some("https://m3.material.io/components/progress-indicators/overview"),
    };

    fn new(_: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            spinner: RepeatAnimationPlayback::default(),
            linear: RepeatAnimationPlayback::default(),
            circular: CircularProgressPlayback::default(),
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        2
    }

    fn prepare(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.circular.observe_reduced_motion(cx);
    }

    fn render_row(
        &mut self,
        row: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if row != CIRCULAR_ROW {
            return board("linear-progress")
                .caption("Linear progress")
                .row_labels(["Label", "Determinate", "Indeterminate"])
                .cell(semantic_label(
                    "progress-label",
                    "Loading content",
                    icon_spinner_with_playback(
                        "label-spinner",
                        ICON_SLOT,
                        self.spinner.clone(),
                        cx,
                    ),
                ))
                .cell(div().w_full().child(linear_progress(
                    "measured",
                    "Upload progress",
                    Some(0.62),
                    px(4.),
                    false,
                    window,
                    cx,
                )))
                .cell(
                    div()
                        .w_full()
                        .child(indeterminate_linear_progress_with_playback(
                            "indeterminate",
                            "Loading content",
                            px(4.),
                            false,
                            self.linear.clone(),
                            cx,
                        )),
                )
                .into_any_element();
        }
        board("circular-progress")
            .caption("Circular progress")
            .columns(4)
            .column_width(120.)
            .column_labels(["Indeterminate", "0%", "62%", "100%"])
            .cell(indeterminate_circular_progress_with_playback(
                "circular-animated",
                "Loading",
                size(rem(24.), rem(24.)),
                1.,
                &self.circular,
                cx,
            ))
            .cells(
                [0., 0.62, 1.]
                    .into_iter()
                    .enumerate()
                    .map(|(index, fraction)| {
                        circular_progress(
                            ("circular-measured", index),
                            "Progress",
                            CircularProgressFrame::determinate(fraction),
                            Some(fraction),
                            size(rem(24.), rem(24.)),
                            1.,
                        )
                    }),
            )
            .into_any_element()
    }
}
