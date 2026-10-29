//! Bounded scrolling evidence for native, profiler-enabled Gallery runs.
//!
//! Start the existing CPU profiler before opening the window (including its first
//! frame), then transfer its collector through `Session::from_prepared`. Do not also install the ordinary timed profiler, which exits the app.
//! Inject its existing CPU report builder through `finish_cpu`; this module does
//! not redefine the CPU acceptance boundary. FPS overlay remains enabled.
use gpui::{
    App, ListState, ScrollHandle, VsyncStatus, Window, WindowId,
    profiler::{self, FrameEvent, FrameTimingCollector},
    px,
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

#[derive(Clone)]
pub(super) enum ScrollTarget {
    /// Existing nonvirtual Gallery, using its main content handle, not navigation.
    Pixels(ScrollHandle),
    /// Virtual Gallery. Small steps operate on the already measured viewport.
    List(ListState),
}
impl From<ScrollHandle> for ScrollTarget {
    fn from(handle: ScrollHandle) -> Self {
        Self::Pixels(handle)
    }
}
impl From<ListState> for ScrollTarget {
    fn from(list: ListState) -> Self {
        Self::List(list)
    }
}
#[derive(Clone, Copy, PartialEq)]
struct Position {
    item: Option<usize>,
    offset: f32,
}
impl Position {
    fn json(self) -> Value {
        json!({"item":self.item,"offset":self.offset})
    }
}
impl ScrollTarget {
    fn position(&self) -> Position {
        match self {
            Self::Pixels(handle) => Position {
                item: None,
                offset: -f32::from(handle.offset().y),
            },
            Self::List(list) => {
                let top = list.logical_scroll_top();
                Position {
                    item: Some(top.item_ix),
                    offset: f32::from(top.offset_in_item),
                }
            }
        }
    }
    fn step(&self, down: bool) {
        let distance = if down { 8. } else { -8. };
        match self {
            Self::Pixels(handle) => {
                let mut offset = handle.offset();
                // ScrollHandle::set_offset itself does not clamp.
                offset.y = px((f32::from(offset.y) - distance)
                    .clamp(-f32::from(handle.max_offset().y).max(0.), 0.));
                handle.set_offset(offset);
            }
            Self::List(list) => list.scroll_by(px(distance)),
        }
    }
}

pub(super) struct Session {
    output: PathBuf,
    duration: Duration,
    collector: FrameTimingCollector,
}
impl Session {
    pub(super) fn from_prepared(
        output: PathBuf,
        duration: Duration,
        collector: FrameTimingCollector,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            (Duration::from_secs(2)..=Duration::from_secs(120)).contains(&duration),
            "scroll duration must be 2..=120 seconds"
        );
        anyhow::ensure!(
            !std::env::args_os().any(|arg| arg == "--capture"),
            "scroll measurement and capture cannot run together"
        );
        Ok(Self {
            output,
            duration,
            collector,
        })
    }

    /// `finish_cpu` receives every captured event (including startup), target ID,
    /// elapsed scenario time and ring-integrity errors. It must use the existing
    /// whole-frame report, not sum Draw/Present stages or filter slow frames.
    pub(super) fn install(
        mut self,
        window: &mut Window,
        cx: &mut App,
        target: ScrollTarget,
        finish_cpu: impl FnOnce(Vec<FrameEvent>, WindowId, Duration, Vec<String>) -> Value + 'static,
    ) {
        let quit_output = self.output.clone();
        cx.on_app_quit(move |_| {
            let output = quit_output.clone();
            async move {
                let report = json!({"schema":"gallery-scroll-observations-v1", "acceptance_eligible":false,
                    "exit_code":2,"errors":["application quit before scroll capture completed"]});
                let _ = std::fs::write(output, serde_json::to_vec_pretty(&report).unwrap());
                std::process::exit(2);
            }
        }).detach();
        let window_id = window.window_handle().window_id();
        let handle = window.window_handle();
        let started = Instant::now();
        let state = Rc::new(RefCell::new(Drive {
            started,
            duration: self.duration,
            target,
            raw: Vec::new(),
            errors: Vec::new(),
            previous: None,
            down: true,
            leg_started: started,
            moved_up: false,
            moved_down: false,
            stopped: false,
        }));
        if window.set_vsync(false).is_err() || window.vsync_status() != VsyncStatus::Disabled {
            state
                .borrow_mut()
                .errors
                .push("VSync could not be disabled and confirmed".into());
            state.borrow_mut().stopped = true;
        }
        if window.submission_stats().successful_new_frames.is_none() {
            state
                .borrow_mut()
                .errors
                .push("successful new-frame counter unsupported".into());
            state.borrow_mut().stopped = true;
        }
        // Record the initial count/position; no startup sample is discarded.
        state.borrow_mut().observe(window);
        schedule(state.clone(), window);
        let executor = cx.background_executor().clone();
        cx.spawn(async move |cx| {
            let mut events = Vec::new();
            let mut cpu_errors = Vec::new();
            loop {
                executor.timer(Duration::from_millis(100)).await;
                let batch = self.collector.collect_unseen();
                if batch.len() >= (16 * 1024 * 1024) / std::mem::size_of::<FrameEvent>() {
                    cpu_errors.push("profiler ring full; event loss cannot be excluded".into());
                }
                events.extend(batch);
                if started.elapsed() >= self.duration || state.borrow().stopped {
                    break;
                }
            }
            if handle
                .update(cx, |_, window, _| {
                    let mut state = state.borrow_mut();
                    state.observe(window);
                    state.stopped = true;
                })
                .is_err()
            {
                state
                    .borrow_mut()
                    .errors
                    .push("window closed before final sample".into());
            }
            let final_batch = self.collector.collect_unseen();
            if final_batch.len() >= (16 * 1024 * 1024) / std::mem::size_of::<FrameEvent>() {
                cpu_errors.push("final profiler batch full; event loss cannot be excluded".into());
            }
            events.extend(final_batch);
            profiler::set_trace_enabled(false);
            let cpu = finish_cpu(events, window_id, started.elapsed(), cpu_errors);
            let report = state.borrow().report(cpu);
            let code = report["exit_code"].as_i64().unwrap_or(2) as i32;
            if let Some(parent) = self.output.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(error) =
                std::fs::write(&self.output, serde_json::to_vec_pretty(&report).unwrap())
            {
                eprintln!("Could not write scroll evidence: {error}");
                std::process::exit(2);
            }
            std::process::exit(code);
        })
        .detach();
    }
}

struct Observation {
    ns: u128,
    count: Option<u64>,
    position: Position,
    down: bool,
    moved: bool,
    vsync: &'static str,
}
struct Drive {
    started: Instant,
    duration: Duration,
    target: ScrollTarget,
    raw: Vec<Observation>,
    errors: Vec<String>,
    previous: Option<Position>,
    down: bool,
    leg_started: Instant,
    moved_up: bool,
    moved_down: bool,
    stopped: bool,
}
impl Drive {
    fn observe(&mut self, window: &Window) -> bool {
        let position = self.target.position();
        if let Some(previous) = self.previous
            && let (Some(a), Some(b)) = (previous.item, position.item)
            && a.abs_diff(b) > 1
        {
            self.errors.push("small list scroll skipped more than one logical row; measurement cannot prove continuous movement".into());
            self.stopped = true;
        }
        let moved = self.previous.is_some_and(|previous| previous != position);
        if moved {
            if self.down {
                self.moved_down = true;
            } else {
                self.moved_up = true;
            }
        }
        self.raw.push(Observation {
            ns: self.started.elapsed().as_nanos(),
            count: window.submission_stats().successful_new_frames,
            position,
            down: self.down,
            moved,
            vsync: match window.vsync_status() {
                VsyncStatus::Disabled => "disabled",
                VsyncStatus::Enabled => "enabled",
                VsyncStatus::Unsupported => "unsupported",
            },
        });
        self.previous = Some(position);
        moved
    }
    fn report(&self, cpu: Value) -> Value {
        let mut windows = Vec::new();
        let mut anchor = 0;
        for end in 1..self.raw.len() {
            let a = &self.raw[anchor];
            let b = &self.raw[end];
            if b.ns - a.ns < 250_000_000 && end + 1 != self.raw.len() {
                continue;
            }
            let fps = a.count.zip(b.count).and_then(|(a_count, b_count)| {
                (b.ns > a.ns)
                    .then(|| b_count.saturating_sub(a_count) as f64 * 1e9 / (b.ns - a.ns) as f64)
            });
            windows.push(json!({"start_ns":a.ns,"end_ns":b.ns,"start_count":a.count,
                "end_count":b.count,"fps":fps,"partial":b.ns-a.ns<250_000_000,"includes_driver_start_and_turns":true}));
            anchor = end;
        }
        let minimum = windows
            .iter()
            .filter(|w| w["partial"] == false)
            .filter_map(|w| w["fps"].as_f64())
            .reduce(f64::min);
        let raw: Vec<_> = self.raw.iter().enumerate().map(|(index,o)| {
            let previous = index.checked_sub(1).map(|i| &self.raw[i]);
            json!({"timestamp_ns":o.ns,"successful_new_frames":o.count,"position":o.position.json(),
                "direction":if o.down {"down"} else {"up"},"position_changed":o.moved,"actual_vsync":o.vsync,
                "observed_interval_ns":previous.map(|p|o.ns-p.ns),
                "counter_increment":previous.and_then(|p|p.count.zip(o.count)).map(|(a,b)|b.saturating_sub(a))})
        }).collect();
        let visited_items: std::collections::BTreeSet<_> =
            self.raw.iter().filter_map(|o| o.position.item).collect();
        let offset_min = self.raw.iter().map(|o| o.position.offset).reduce(f32::min);
        let offset_max = self.raw.iter().map(|o| o.position.offset).reduce(f32::max);
        let total_scrollable_pixels = match &self.target {
            ScrollTarget::Pixels(handle) => Some(f32::from(handle.max_offset().y)),
            _ => None,
        };
        let known_cpu_failure = cpu["summary"]["over_budget_frames"].as_u64().unwrap_or(0) > 0
            || cpu["diagnostic_stage_summary"]["over_budget_frames"]
                .as_u64()
                .unwrap_or(0)
                > 0;
        let partial_minimum = windows
            .iter()
            .filter(|w| w["partial"] == true)
            .filter_map(|w| w["fps"].as_f64())
            .reduce(f64::min);
        let known_fps_failure = minimum.is_some_and(|fps| fps < 500.);
        let mut errors = self.errors.clone();
        if !self.moved_up || !self.moved_down {
            errors.push("did not verify actual movement in both directions".into());
        }
        if minimum.is_none() {
            errors.push("no complete >=250ms observation window".into());
        }
        json!({"schema":"gallery-scroll-observations-v1","raw_observations":raw,"windows":windows,
            "minimum_observed_window_fps":minimum,"minimum_partial_window_fps":partial_minimum,"moved_up":self.moved_up,"moved_down":self.moved_down,
            "errors":errors,"cpu":cpu,"fps_overlay":true,
            "actual_vsync_initial":self.raw.first().map(|o|o.vsync),
            "actual_vsync_final":self.raw.last().map(|o|o.vsync),
            "timing_domains":"CPU collector includes synchronous InitialDraw before open returns; FPS observations start when driver installs after open, not at CPU startup",
            "coverage":{"visited_logical_items":visited_items,"observed_offset_min":offset_min,"observed_offset_max":offset_max,
                "total_scrollable_pixels":total_scrollable_pixels,"visited_sections":null,"complete_gallery_traversal":false,
                "limitation":"one-second alternating local legs; logical items listed only for ListState; section identity not instrumented, full Gallery coverage unverified"},
            "acceptance_eligible":false,"exit_code":if known_cpu_failure||known_fps_failure {1}else{2},
            "strict_fps_limit":"callback observation timestamps are not per-successful-submission timestamps; coalesced increments cannot prove individual frame intervals; the full-window counter also includes FPS overlay and animations, and cannot attribute every counted frame to scrolling",
            "scope":"programmatic bidirectional main-content scrolling; no synthetic refresh without offset change; startup and turns retained; GPU completion unmeasured"})
    }
}
fn schedule(state: Rc<RefCell<Drive>>, window: &mut Window) {
    if state.borrow().stopped {
        return;
    }
    window.on_next_frame(move |window, _| {
        let mut drive = state.borrow_mut();
        if drive.stopped || drive.started.elapsed() >= drive.duration {
            return;
        }
        let moved = drive.observe(window);
        if window.vsync_status() != VsyncStatus::Disabled {
            drive.errors.push("VSync changed during scrolling".into());
            drive.stopped = true;
            return;
        }
        if drive.leg_started.elapsed() >= Duration::from_secs(1) || (!moved && drive.raw.len() > 2)
        {
            drive.down = !drive.down;
            drive.leg_started = Instant::now();
        }
        let before = drive.target.position();
        drive.target.step(drive.down);
        if drive.target.position() != before {
            window.refresh();
        }
        drop(drive);
        schedule(state, window);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn drive(count: u64) -> Drive {
        let now = Instant::now();
        Drive {
            started: now,
            duration: Duration::from_secs(2),
            target: ScrollTarget::Pixels(ScrollHandle::new()),
            errors: Vec::new(),
            previous: None,
            down: false,
            leg_started: now,
            moved_up: true,
            moved_down: true,
            stopped: true,
            raw: vec![
                Observation {
                    ns: 0,
                    count: Some(0),
                    position: Position {
                        item: None,
                        offset: 0.,
                    },
                    down: true,
                    moved: false,
                    vsync: "disabled",
                },
                Observation {
                    ns: 250_000_000,
                    count: Some(count),
                    position: Position {
                        item: None,
                        offset: 8.,
                    },
                    down: false,
                    moved: true,
                    vsync: "disabled",
                },
            ],
        }
    }
    #[test]
    fn window_rate_never_claims_strict_fps_acceptance_and_preserves_coalescing() {
        let report = drive(125)
            .report(json!({"acceptance_eligible":true,"summary":{"over_budget_frames":0}}));
        assert_eq!(report["minimum_observed_window_fps"], 500.);
        assert_eq!(report["raw_observations"][1]["counter_increment"], 125);
        assert_eq!(report["acceptance_eligible"], false);
        assert_eq!(report["exit_code"], 2);
    }
    #[test]
    fn observed_failure_is_not_hidden_by_incomplete_evidence() {
        assert_eq!(drive(124).report(json!({}))["exit_code"], 1);
        assert_eq!(
            drive(126).report(json!({"summary":{"over_budget_frames":1}}))["exit_code"],
            1
        );
    }
    #[test]
    fn short_tail_is_preserved_without_polluting_full_window_minimum() {
        let mut drive = drive(125);
        drive.raw.push(Observation {
            ns: 250_001_000,
            count: Some(125),
            position: Position {
                item: None,
                offset: 0.,
            },
            down: false,
            moved: true,
            vsync: "disabled",
        });
        let report = drive.report(json!({}));
        assert_eq!(report["minimum_observed_window_fps"], 500.);
        assert_eq!(report["minimum_partial_window_fps"], 0.);
        assert_eq!(report["windows"].as_array().unwrap().len(), 2);
        assert_eq!(report["raw_observations"].as_array().unwrap().len(), 3);
        assert_eq!(report["windows"][1]["partial"], true);
        assert_eq!(report["exit_code"], 2);
    }
}
