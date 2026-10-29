//! Low-frequency observation of successful distinct window frame submissions.
use super::*;
use std::time::Duration;

pub(super) struct FpsDisplay {
    label: SharedString,
    // Dropping the view cancels its timer; the task only holds a weak view.
    _task: Task<()>,
}

impl FpsDisplay {
    pub(super) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut count = window.submission_stats().successful_new_frames;
        let mut sampled_at = cx.background_executor().now();
        // A label change redraws this display: one frame the counter itself
        // caused, which must not keep an idle window reporting 2 FPS.
        let mut own_frames = 0;
        let task = cx.spawn_in(window, async move |view, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;
                if view
                    .update_in(cx, |this, window, cx| {
                        let now = cx.background_executor().now();
                        let current = window.submission_stats().successful_new_frames;
                        let label: SharedString = match (count, current) {
                            (Some(previous), Some(current)) => {
                                let fps = sample_rate(
                                    previous + own_frames,
                                    current,
                                    now.saturating_duration_since(sampled_at),
                                );
                                format!("FPS: {fps:.1}").into()
                            }
                            (_, None) => "FPS: unavailable".into(),
                            (None, Some(_)) => "FPS: —".into(),
                        };
                        count = current;
                        sampled_at = now;
                        own_frames = 0;
                        if this.label != label {
                            this.label = label;
                            own_frames = 1;
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            label: if count.is_some() {
                "FPS: —".into()
            } else {
                "FPS: unavailable".into()
            },
            _task: task,
        }
    }
}

fn sample_rate(previous: u64, current: u64, elapsed: Duration) -> f64 {
    if elapsed.is_zero() {
        return 0.;
    }
    current.saturating_sub(previous) as f64 / elapsed.as_secs_f64()
}

impl Render for FpsDisplay {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        // No live region: updating this counter must not produce announcements.
        div()
            .id("gallery-fps")
            .text_size(TEXT_AUX)
            .aria_label("FPS: successful newly drawn frame submissions per second, excluding redraws of this counter; not display refresh rate")
            .child(self.label.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::sample_rate;
    use std::time::Duration;
    #[test]
    fn rate_uses_real_elapsed_and_becomes_zero_when_idle() {
        assert_eq!(sample_rate(10, 25, Duration::from_millis(750)), 20.);
        assert_eq!(sample_rate(25, 25, Duration::from_millis(500)), 0.);
        assert_eq!(sample_rate(25, 26, Duration::from_millis(500)), 2.);
        assert_eq!(sample_rate(25, 25, Duration::ZERO), 0.);
        // Excluding the display's own redraw leaves an idle window at zero.
        assert_eq!(sample_rate(25 + 1, 26, Duration::from_millis(500)), 0.);
    }

    #[gpui::test]
    fn dropping_display_releases_entity_and_pending_timer(cx: &mut gpui::TestAppContext) {
        use gpui::AppContext as _;
        struct EmptyView;
        impl gpui::Render for EmptyView {
            fn render(
                &mut self,
                _: &mut gpui::Window,
                _: &mut gpui::Context<Self>,
            ) -> impl gpui::IntoElement {
                gpui::Empty
            }
        }
        let window = cx.add_window(|_, _| EmptyView);
        let display = window
            .update(cx, |_, window, cx| {
                cx.new(|cx| super::FpsDisplay::new(window, cx))
            })
            .unwrap();
        let weak = display.downgrade();
        drop(display);
        cx.run_until_parked();
        assert!(
            weak.upgrade().is_none(),
            "sampling task must not retain its owner"
        );
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.run_until_parked();
        assert!(weak.upgrade().is_none());
    }
}
