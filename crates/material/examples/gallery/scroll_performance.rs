//! Bounded scrolling evidence for native, profiler-enabled Gallery runs.
//!
//! Start the existing CPU profiler before opening the window (including its first
//! frame), then transfer its collector through `Session::from_prepared`. Do not also install the ordinary timed profiler, which exits the app.
//! Inject its existing CPU report builder through `finish_cpu`; this module does
//! not redefine the CPU acceptance boundary. FPS overlay remains enabled.
use gpui::{
    App, ListState, PlatformInput, ScrollDelta, ScrollEvent, ScrollHandle, VsyncStatus, Window,
    WindowId, point,
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

// Signed offsets preserve queued signals that predate driver installation.
fn journal_offset(at: Instant, origin: Instant) -> i128 {
    if let Some(elapsed) = at.checked_duration_since(origin) {
        elapsed.as_nanos() as i128
    } else {
        -(origin.duration_since(at).as_nanos() as i128)
    }
}

#[derive(Default)]
struct JournalEvidence {
    entries: Vec<Value>,
    batches: Vec<Value>,
    lost: u64,
}
impl JournalEvidence {
    fn drain(&mut self, batch: profiler::journal::DrainedEntries, id: WindowId, origin: Instant) {
        use profiler::journal::{
            ForegroundJournalEntry as Entry, FrameSkipReason, IntervalBoundary,
        };
        self.lost = self.lost.saturating_add(batch.lost);
        self.batches
            .push(json!({"entries":batch.entries.len(),"lost":batch.lost}));
        for entry in batch.entries {
            let value = match entry {
                Entry::PlatformSignal(signal) if signal.window_id == id => json!({
                    "kind":"platform_signal",
                    "signal_at_ns":signal.signal_at.map(|at|journal_offset(at, origin)),
                    "handled_at_ns":journal_offset(signal.handled_at, origin),
                    "source":match signal.source {
                        gpui::FrameRequestSource::NativeCallback => "native_callback",
                        gpui::FrameRequestSource::LocalSchedule => "local_schedule",
                    }
                }),
                Entry::Boundary(IntervalBoundary::FrameSkipped(skip)) if skip.window_id == id => {
                    json!({
                        "kind":"frame_skipped","at_ns":journal_offset(skip.at, origin),
                        "reason":match skip.reason {
                            FrameSkipReason::NoRenderNeeded => "no_render_needed",
                            FrameSkipReason::InactiveFrameRateLimit => "inactive_frame_rate_limit",
                            FrameSkipReason::ThermalFrameRateLimit => "thermal_frame_rate_limit",
                        }
                    })
                }
                Entry::ReentrantFrameSkipped { window_id, at } if window_id == id => json!({
                    "kind":"reentrant_frame_skipped","at_ns":journal_offset(at, origin)
                }),
                Entry::Discontinuity { lost } => json!({"kind":"discontinuity","lost":lost}),
                _ => continue,
            };
            self.entries.push(value);
        }
    }
    fn report(&self) -> Value {
        json!({"entries":self.entries,"batches":self.batches,"lost":self.lost,
            "scope":"existing foreground journal collected after driver installation; only target-window platform signals and published skips plus stream-wide discontinuities; skip reasons are not exhaustive; coalesced request sources are not wake counts; no new hot-path instrumentation",
            "time_basis":"signed nanoseconds relative to driver started, same origin as raw observations; negative offsets preserve signals requested before installation; null signal_at means unavailable",
            "loss_scope":"lost aggregates batch lost values once; discontinuity markers describe the same loss, not additional loss; shared foreground stream loss cannot be assigned to a window"})
    }
}

/// Read cumulative counters only: never reset stats or enable per-node timing.
fn engine_snapshot(window: &Window) -> Value {
    #[cfg(feature = "gpui-fast")]
    {
        let stats = window.layout_stats();
        json!({
            "view_retention": window.view_retention(),
            "frames": stats.frames,
            "views_built": stats.views_built,
            "views_reused": stats.views_reused,
            "nodes_created": stats.nodes_created,
            "nodes_reused": stats.nodes_reused,
            "lines_shaped": stats.lines_shaped,
            "layer_frames_composited": stats.layer_frames_composited,
            "layer_frames_repainted": stats.layer_frames_repainted,
            "tiles_dirtied": stats.tiles_dirtied,
            "layer_rebuilds_for_input": stats.layer_rebuilds_for_input,
            "layers_demoted": stats.layers_demoted
        })
    }
    #[cfg(not(feature = "gpui-fast"))]
    {
        let _ = window;
        Value::Null
    }
}

fn engine_environment() -> Value {
    // Deliberate whitelist; never serialize the process environment wholesale.
    let entry = |key| match std::env::var_os(key) {
        Some(value) => json!({"present":true,"value":value.to_string_lossy()}),
        None => json!({"present":false,"value":null}),
    };
    json!({"GPUI_VIEW_RETENTION":entry("GPUI_VIEW_RETENTION"),
        "GPUI_SCROLL_LAYERS":entry("GPUI_SCROLL_LAYERS")})
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(super) enum InputMode {
    #[default]
    Programmatic,
    Wheel,
}
impl InputMode {
    pub(super) fn from_environment() -> anyhow::Result<Self> {
        match std::env::var("MATERIAL_SCROLL_INPUT").as_deref() {
            Ok("wheel") => Ok(Self::Wheel),
            Ok("programmatic") | Err(std::env::VarError::NotPresent) => Ok(Self::Programmatic),
            _ => anyhow::bail!("MATERIAL_SCROLL_INPUT must be wheel or programmatic"),
        }
    }
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Programmatic => "programmatic",
            Self::Wheel => "wheel",
        }
    }
}

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
/// Plain record of one dispatched wheel event; serialized only after capture.
#[derive(Clone, Copy)]
struct WheelInput {
    ns: u128,
    position: [f32; 2],
    delta_y: f32,
}
/// Upper bound on per-second samples, so buffers are allocated once before
/// measurement and never grow inside a measured frame.
const MAX_SAMPLES_PER_SECOND: usize = 4_000;
fn sample_capacity(duration: Duration) -> usize {
    (duration.as_secs_f64().ceil() as usize + 1) * MAX_SAMPLES_PER_SECOND
}
#[derive(Clone, Copy, PartialEq)]
struct Position {
    item: Option<usize>,
    offset: f32,
    /// Absolute content offset from the top in pixels (list: sum of known row
    /// heights above the top row plus the in-row offset).
    px: f32,
}
impl Position {
    fn json(self) -> Value {
        json!({"item":self.item,"offset":self.offset,"content_px":self.px})
    }
}
impl ScrollTarget {
    fn position(&self) -> Position {
        match self {
            Self::Pixels(handle) => Position {
                item: None,
                offset: -f32::from(handle.offset().y),
                px: -f32::from(handle.offset().y),
            },
            Self::List(list) => {
                let top = list.logical_scroll_top();
                Position {
                    item: Some(top.item_ix),
                    offset: f32::from(top.offset_in_item),
                    px: -f32::from(list.scroll_px_offset_for_scrollbar().y),
                }
            }
        }
    }
    fn wheel(&self, down: bool, window: &mut Window, cx: &mut App) -> Option<WheelInput> {
        let bounds = match self {
            Self::Pixels(handle) => handle.bounds(),
            Self::List(list) => list.viewport_bounds(),
        }
        .intersect(&gpui::Bounds::new(
            point(px(0.), px(0.)),
            window.viewport_size(),
        ));
        if bounds.size.width <= px(0.) || bounds.size.height <= px(0.) {
            return None;
        }
        let position = bounds.center();
        let delta = if down { -8. } else { 8. };
        window.dispatch_event(
            PlatformInput::Scroll(ScrollEvent {
                position,
                delta: ScrollDelta::Pixels(point(px(0.), px(delta))),
                ..Default::default()
            }),
            cx,
        );
        Some(WheelInput {
            ns: 0,
            position: [f32::from(position.x), f32::from(position.y)],
            delta_y: delta,
        })
    }
    fn at_end(&self) -> bool {
        match self {
            Self::Pixels(handle) => {
                -f32::from(handle.offset().y) >= f32::from(handle.max_offset().y) - 0.5
            }
            // Unknown heights (None) are not the end: keep scrolling to measure.
            Self::List(list) => list.is_scrolled_to_end() == Some(true),
        }
    }
    fn at_top(&self) -> bool {
        let position = self.position();
        position.item.unwrap_or(0) == 0 && position.offset <= 0.
    }
    fn item_count(&self) -> Option<usize> {
        match self {
            Self::Pixels(_) => None,
            Self::List(list) => Some(list.item_count()),
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

/// Default input cadence. Input is driven by a timer, independent of frame
/// callbacks; actual timing still depends on foreground availability.
const DEFAULT_INPUT_HZ: u32 = 1_000;
fn input_interval_from_environment() -> anyhow::Result<Duration> {
    let hz = match std::env::var("MATERIAL_SCROLL_INPUT_HZ") {
        Ok(value) => value
            .parse::<u32>()
            .map_err(|_| anyhow::anyhow!("MATERIAL_SCROLL_INPUT_HZ must be an integer"))?,
        Err(std::env::VarError::NotPresent) => DEFAULT_INPUT_HZ,
        Err(_) => anyhow::bail!("MATERIAL_SCROLL_INPUT_HZ must be valid Unicode"),
    };
    anyhow::ensure!(
        (100..=MAX_SAMPLES_PER_SECOND as u32 / 2).contains(&hz),
        "MATERIAL_SCROLL_INPUT_HZ must be in 100..={}",
        MAX_SAMPLES_PER_SECOND / 2
    );
    Ok(Duration::from_secs(1) / hz)
}

pub(super) struct Session {
    output: PathBuf,
    duration: Duration,
    collector: FrameTimingCollector,
    input_mode: InputMode,
    input_interval: Duration,
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
            input_mode: InputMode::from_environment()?,
            input_interval: input_interval_from_environment()?,
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
        finish_cpu: impl FnOnce(Vec<FrameEvent>, WindowId, Duration, Vec<String>, Vec<usize>) -> Value
        + 'static,
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
        let mut journal_collector = cx.foreground_journal().collector();
        let state = Rc::new(RefCell::new(Drive {
            started,
            duration: self.duration,
            target,
            input_mode: self.input_mode,
            engine_initial: engine_snapshot(window),
            engine_final: Value::Null,
            engine_environment: engine_environment(),
            inputs: Vec::with_capacity(sample_capacity(self.duration)),
            raw: Vec::with_capacity(sample_capacity(self.duration)),
            errors: Vec::new(),
            previous: None,
            down: true,
            input_interval: self.input_interval,
            refresh_interval: window.refresh_interval(cx),
            traversal: Traversal::default(),
            new_draw_presents_ns: Vec::new(),
            successful_submission_diagnostic: Value::Null,
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
        state.borrow_mut().observe(window, cx);
        let executor = cx.background_executor().clone();
        // Timer-driven input: each tick is one input event dispatched outside
        // any frame callback. Frames are produced (or coalesced) by the engine
        // on its own schedule, so the driver cannot pace frames to one input.
        cx.spawn({
            let state = state.clone();
            let executor = executor.clone();
            async move |cx| {
                let interval = state.borrow().input_interval;
                let mut next = Instant::now();
                loop {
                    // Skip missed deadlines rather than burst catch-up inputs.
                    next = (next + interval).max(Instant::now());
                    executor
                        .timer(next.saturating_duration_since(Instant::now()))
                        .await;
                    if state.borrow().stopped {
                        break;
                    }
                    let ticked = handle.update(cx, |_, window, cx| {
                        tick(&mut state.borrow_mut(), window, cx);
                    });
                    if ticked.is_err() {
                        break;
                    }
                }
            }
        })
        .detach();
        cx.spawn(async move |cx| {
            let mut events = Vec::new();
            let mut cpu_errors = Vec::new();
            let mut batch_sizes = Vec::new();
            let mut journal = JournalEvidence::default();
            loop {
                executor.timer(Duration::from_millis(100)).await;
                journal.drain(journal_collector.collect_unseen(), window_id, started);
                let batch = self.collector.collect_unseen();
                batch_sizes.push(batch.len());
                if batch.len() >= (16 * 1024 * 1024) / std::mem::size_of::<FrameEvent>() {
                    cpu_errors.push("profiler ring full; event loss cannot be excluded".into());
                }
                events.extend(batch);
                if started.elapsed() >= self.duration || state.borrow().stopped {
                    break;
                }
            }
            if handle
                .update(cx, |_, window, cx| {
                    let mut state = state.borrow_mut();
                    state.observe(window, cx);
                    state.engine_final = engine_snapshot(window);
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
            batch_sizes.push(final_batch.len());
            if final_batch.len() >= (16 * 1024 * 1024) / std::mem::size_of::<FrameEvent>() {
                cpu_errors.push("final profiler batch full; event loss cannot be excluded".into());
            }
            events.extend(final_batch);
            journal.drain(journal_collector.collect_unseen(), window_id, started);
            profiler::set_trace_enabled(false);
            let presents = new_draw_presents(&events, window_id, started);
            state.borrow_mut().new_draw_presents_ns = presents;
            state.borrow_mut().successful_submission_diagnostic =
                successful_submissions(&events, window_id, started);
            let cpu = finish_cpu(
                events,
                window_id,
                started.elapsed(),
                cpu_errors,
                batch_sizes,
            );
            let mut report = state.borrow().report(cpu);
            report["foreground_journal"] = journal.report();
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
    thermal_state: &'static str,
    window_active: bool,
}
struct Drive {
    started: Instant,
    duration: Duration,
    target: ScrollTarget,
    input_mode: InputMode,
    inputs: Vec<WheelInput>,
    engine_initial: Value,
    engine_final: Value,
    engine_environment: Value,
    raw: Vec<Observation>,
    errors: Vec<String>,
    previous: Option<Position>,
    down: bool,
    input_interval: Duration,
    refresh_interval: Option<Duration>,
    traversal: Traversal,
    /// First Present-attempt end after Draw, ns since driver start. These
    /// profiler events do not establish backend success or retry identity.
    new_draw_presents_ns: Vec<i128>,
    successful_submission_diagnostic: Value,
    moved_up: bool,
    moved_down: bool,
    stopped: bool,
}
impl Drive {
    fn observe(&mut self, window: &Window, cx: &App) -> bool {
        let position = self.target.position();
        // Continuity in content pixels: one 8px input separates observations.
        // Row indices may legitimately jump across rows shorter than a step.
        if let Some(previous) = self.previous
            && (position.px - previous.px).abs() > CONTINUITY_PX
        {
            self.errors.push(format!(
                "content offset jumped {:.1}px between observations (limit {CONTINUITY_PX}px); measurement cannot prove continuous movement",
                position.px - previous.px
            ));
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
        if self.raw.len() == self.raw.capacity() {
            self.errors.push(
                "observation buffer full; stopped rather than allocate while measuring".into(),
            );
            self.stopped = true;
            return false;
        }
        self.raw.push(Observation {
            ns: self.started.elapsed().as_nanos(),
            count: window.submission_stats().successful_new_frames,
            position,
            down: self.down,
            moved,
            // App forwards this to the platform getter. On macOS this reads
            // NSProcessInfo.thermalState; no observer, timer, or extra render is
            // installed. This is one sampled state, not a transition history.
            thermal_state: match cx.thermal_state() {
                gpui::ThermalState::Nominal => "nominal",
                gpui::ThermalState::Fair => "fair",
                gpui::ThermalState::Serious => "serious",
                gpui::ThermalState::Critical => "critical",
            },
            window_active: window.is_window_active(),
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
                "direction":if o.down {"down"} else {"up"},"position_changed":o.moved,"actual_vsync":o.vsync,"thermal_state":o.thermal_state,"window_active":o.window_active,
                "observed_interval_ns":previous.map(|p|o.ns-p.ns),
                "counter_increment":previous.and_then(|p|p.count.zip(o.count)).map(|(a,b)|b.saturating_sub(a))})
        }).collect();
        let visited_items: std::collections::BTreeSet<_> =
            self.raw.iter().filter_map(|o| o.position.item).collect();
        // Require contiguous observed logical top-row positions. This does not
        // establish that those rows, or later viewport rows, were submitted.
        let contiguous_top_rows = visited_items
            .iter()
            .enumerate()
            .all(|(index, item)| index == *item);
        let complete_traversal = self.traversal.laps >= 1
            && (matches!(self.target, ScrollTarget::Pixels(_)) || contiguous_top_rows);
        // Diagnostic attempts only. Present events include failed submissions;
        // aggregate success counts cannot identify which attempt succeeded.
        let span_end = self.raw.last().map_or(0, |o| o.ns as i128);
        let in_span: Vec<i128> = self
            .new_draw_presents_ns
            .iter()
            .copied()
            .filter(|ns| (0..=span_end).contains(ns))
            .collect();
        let longest_gap_ns = in_span.windows(2).map(|w| w[1] - w[0]).max();
        let inverse_interior_attempt_gap_hz = longest_gap_ns.map(|gap| 1e9 / gap.max(1) as f64);
        let counted_new_frames = self
            .raw
            .first()
            .zip(self.raw.last())
            .and_then(|(a, b)| a.count.zip(b.count))
            .map(|(a, b)| b.saturating_sub(a));
        let aggregate_counter_covers_attempt_count =
            counted_new_frames.is_some_and(|count| count as usize >= in_span.len());
        // Descriptive correlation only: a rate near/below display refresh
        // cannot identify the limiting mechanism or invalidate known failure.
        let mut full_rates: Vec<f64> = windows
            .iter()
            .filter(|w| w["partial"] == false)
            .filter_map(|w| w["fps"].as_f64())
            .collect();
        full_rates.sort_by(f64::total_cmp);
        let median_rate = full_rates.get(full_rates.len() / 2).copied();
        let refresh_hz = self
            .refresh_interval
            .filter(|interval| !interval.is_zero())
            .map(|interval| 1e9 / interval.as_nanos() as f64);
        let observed_rate_near_or_below_refresh = median_rate
            .zip(refresh_hz)
            .is_some_and(|(rate, hz)| rate <= hz * 1.1)
            && self.raw.iter().all(|o| o.vsync == "disabled");
        let offset_min = self.raw.iter().map(|o| o.position.offset).reduce(f32::min);
        let offset_max = self.raw.iter().map(|o| o.position.offset).reduce(f32::max);
        let total_scrollable_pixels = match &self.target {
            ScrollTarget::Pixels(handle) => Some(f32::from(handle.max_offset().y)),
            _ => None,
        };
        let known_cpu_failure = super::performance::known_cpu_failure(&cpu);
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
        if !complete_traversal {
            errors.push("did not complete a full top-to-bottom-to-top Gallery traversal".into());
        }
        if inverse_interior_attempt_gap_hz.is_none() {
            errors.push(
                "fewer than two draw-associated submission attempts inside the driver span".into(),
            );
        }
        errors.push("successful-new-frame timestamps are diagnostic only; complete start/end intervals and per-frame scroll-position association remain unverified; aggregate counts cannot prove rendered traversal".into());
        json!({"schema":"gallery-scroll-observations-v1","input_mode":self.input_mode.name(),"raw_inputs":self.inputs.iter().map(|i| json!({"timestamp_ns":i.ns,"position_px":i.position,"delta_y_px":i.delta_y})).collect::<Vec<_>>(),
            "engine_counters":{"initial":self.engine_initial,"final":self.engine_final,
                "environment_overrides":self.engine_environment,
                "scope":"cumulative window counters at driver install (after initial draw) and final observation; not per-frame metrics; counters are never reset and extra timing is not enabled; unavailable backend or missing final window is null"},"raw_observations":raw,"windows":windows,
            "minimum_observed_window_fps":minimum,"minimum_partial_window_fps":partial_minimum,"moved_up":self.moved_up,"moved_down":self.moved_down,
            "errors":errors,"cpu":cpu,"fps_overlay":true,
            "actual_vsync_initial":self.raw.first().map(|o|o.vsync),
            "actual_vsync_final":self.raw.last().map(|o|o.vsync),
            "sampled_state_limit":"thermal_state and window_active are read during each observation; they do not prove state remained unchanged between observations; thermal getter calls the platform and its cost is included in driver work",
            "timing_domains":"CPU collector includes synchronous InitialDraw before open returns; FPS observations start when driver installs after open, not at CPU startup",
            "coverage":{"visited_logical_items":visited_items,"item_count":self.target.item_count(),
                "contiguous_top_rows_from_zero":contiguous_top_rows,"laps":self.traversal.laps,
                "observed_offset_min":offset_min,"observed_offset_max":offset_max,
                "total_scrollable_pixels":total_scrollable_pixels,"visited_sections":null,
                "visited_sections_reason":"section identity is not exposed to the driver; row coverage is recorded instead",
                "logical_offset_traversal_complete":complete_traversal,"complete_gallery_traversal":false,
                "definition":"logical offset lap reaches the reported bottom and returns to top; input observations do not prove every row was painted and successfully submitted, including rows below the deepest top row"},
            "input_driver":{"source":"foreground timer, independent of frame callbacks","interval_ns":self.input_interval.as_nanos(),"step_px":8,"pacing_limit":"foreground work can delay timers; overdue deadlines are caught up with immediate ticks, so requested cadence is not measured constant input frequency"},
            "display_refresh_interval_ns":self.refresh_interval.map(|interval| interval.as_nanos()),
            "median_full_window_fps":median_rate,"observed_rate_near_or_below_refresh":observed_rate_near_or_below_refresh,"refresh_comparison_limit":"rate comparison is diagnostic correlation, not proof of compositor pacing or an excuse to ignore measured FPS failure",
            "successful_submission_diagnostic":self.successful_submission_diagnostic,
            "submission_attempt_diagnostic":{"raw_attempt_end_ns":self.new_draw_presents_ns,"draw_associated_attempts_in_span":in_span.len(),"counted_successful_new_frames":counted_new_frames,
                "all_submissions_successful":Value::Null,"aggregate_counter_covers_attempt_count":aggregate_counter_covers_attempt_count,"longest_interior_attempt_gap_ns":longest_gap_ns,
                "inverse_longest_interior_attempt_gap_hz":inverse_interior_attempt_gap_hz,
                "definition":"attempt-only interior gaps; excludes start-to-first and last-to-end intervals; not a complete-run minimum or successful-frame rate; a failed first attempt and successful retry cannot be distinguished by aggregate counts"},
            "acceptance_eligible":false,
            "exit_code":if known_cpu_failure||known_fps_failure {1} else {2},
            "counter_limit":"the full-window counter also includes FPS overlay and animation frames; input timing does not prove each counted frame carries changed scroll content",
            "scope":"attempted logical top-bottom-top main-content traversal; programmatic mode refreshes on offset change, wheel mode dispatches through platform input without explicit refresh; startup and turns retained; GPU time is in cpu.gpu"})
    }
}
/// Consume the engine's tri-state success marker directly, including retries
/// with no intervening Draw. Never infer success from event order or totals.
fn successful_submissions(events: &[FrameEvent], window: WindowId, origin: Instant) -> Value {
    let mut successful = Vec::new();
    let mut markers = Vec::new();
    let mut false_count = 0usize;
    let mut unknown_count = 0usize;
    for event in events {
        let FrameEvent::Present(present) = event else {
            continue;
        };
        if present.window_id != window {
            continue;
        }
        let at = journal_offset(present.present_end, origin);
        markers
            .push(json!({"present_end_ns":at,"successful_new_frame":present.successful_new_frame}));
        match present.successful_new_frame {
            Some(true) => successful.push(at),
            Some(false) => false_count += 1,
            None => unknown_count += 1,
        }
    }
    json!({"raw_markers":markers,"successful_new_frame_end_ns":successful,
        "successful_new_frame_count":successful.len(),"false_count":false_count,"unknown_count":unknown_count,
        "scope":"all captured target-window Present markers, including negative offsets before driver installation; Some(true) comes from backend success and new-draw identity, not Draw/Present pairing; false includes failed attempts or already-counted scenes",
        "limit":"diagnostic only; start/end interval coverage and association with changed scroll content remain unverified; timestamps are CPU submission completion, not GPU completion"})
}

/// First Present-attempt end after each Draw, relative to origin. This does
/// not identify success: a failed attempt consumes pending and a successful
/// retry is omitted. Retained only as a diagnostic, never an acceptance metric.
fn new_draw_presents(events: &[FrameEvent], window: WindowId, origin: Instant) -> Vec<i128> {
    let mut pending = false;
    let mut presents = Vec::new();
    for event in events {
        match event {
            FrameEvent::Draw(draw) if draw.window_id == window => pending = true,
            FrameEvent::Present(present) if present.window_id == window && pending => {
                pending = false;
                presents.push(match present.present_end.checked_duration_since(origin) {
                    Some(after) => after.as_nanos() as i128,
                    None => -(origin.duration_since(present.present_end).as_nanos() as i128),
                });
            }
            _ => {}
        }
    }
    presents
}

/// One 8px input plus half a pixel of rounding between observations.
const CONTINUITY_PX: f32 = 8.5;

/// Full-Gallery traversal state: top -> bottom -> top is one lap.
#[derive(Default, Clone, Copy)]
struct Traversal {
    reached_end: bool,
    laps: u32,
    /// Last time the position changed, to detect a stalled traversal.
    last_progress_ns: u128,
}
/// A leg that makes no progress for this long cannot prove continuous scrolling.
const STALL_NS: u128 = 1_000_000_000;

fn tick(drive: &mut Drive, window: &mut Window, cx: &mut App) {
    if drive.stopped || drive.started.elapsed() >= drive.duration {
        drive.stopped = true;
        return;
    }
    let moved = drive.observe(window, cx);
    if drive.stopped {
        return;
    }
    if window.vsync_status() != VsyncStatus::Disabled {
        drive.errors.push("VSync changed during scrolling".into());
        drive.stopped = true;
        return;
    }
    let now = drive.started.elapsed().as_nanos();
    if moved {
        drive.traversal.last_progress_ns = now;
    }
    // Turn only at the real ends of the Gallery, never on a timer.
    if drive.down && drive.target.at_end() {
        drive.traversal.reached_end = true;
        drive.down = false;
        drive.traversal.last_progress_ns = now;
    } else if !drive.down && drive.target.at_top() {
        if drive.traversal.reached_end {
            drive.traversal.laps += 1;
            drive.traversal.reached_end = false;
        }
        drive.down = true;
        drive.traversal.last_progress_ns = now;
    } else if now.saturating_sub(drive.traversal.last_progress_ns) > STALL_NS {
        drive.errors.push(
            "scrolling made no progress for 1 s before reaching a Gallery end; traversal unverified"
                .into(),
        );
        drive.stopped = true;
        return;
    }
    match drive.input_mode {
        InputMode::Programmatic => {
            let before = drive.target.position();
            drive.target.step(drive.down);
            if drive.target.position() != before {
                window.refresh();
            }
        }
        InputMode::Wheel => {
            if let Some(mut input) = drive.target.wheel(drive.down, window, cx) {
                input.ns = drive.started.elapsed().as_nanos();
                if drive.inputs.len() == drive.inputs.capacity() {
                    drive.errors.push(
                        "input buffer full; stopped rather than allocate while measuring".into(),
                    );
                    drive.stopped = true;
                    return;
                }
                drive.inputs.push(input);
            } else {
                drive
                    .errors
                    .push("main scroll viewport has no visible area for wheel input".into());
                drive.stopped = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[gpui::test]
    fn short_rows_skip_indices_but_stay_pixel_continuous(cx: &mut gpui::TestAppContext) {
        use gpui::{Context, IntoElement, ListAlignment, ParentElement, Render, Styled, div, list};
        let list_state = ListState::new(200, ListAlignment::Top, px(128.));
        struct View(ListState);
        impl Render for View {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().w(px(300.)).h(px(200.)).child(
                    list(self.0.clone(), |_, _, _| {
                        div().h(px(3.)).w_full().into_any_element()
                    })
                    .size_full(),
                )
            }
        }
        let (_view, cx) = cx.add_window_view(|_, _| View(list_state.clone()));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let target = ScrollTarget::List(list_state);
        let before = target.position();
        cx.update(|window, cx| assert!(target.wheel(true, window, cx).is_some()));
        let after = target.position();
        // 8px over 3px rows crosses more than one row index...
        assert!(after.item.unwrap() > before.item.unwrap() + 1);
        // ...yet content moved exactly one step, within the continuity limit.
        assert!(((after.px - before.px) - 8.).abs() < 0.01);
        assert!((after.px - before.px).abs() <= CONTINUITY_PX);
    }
    #[gpui::test]
    fn wheel_dispatch_moves_list_through_scroll_handler(cx: &mut gpui::TestAppContext) {
        use gpui::{Context, IntoElement, ListAlignment, ParentElement, Render, Styled, div, list};
        // Match Gallery overdraw so measured content extends past the viewport.
        let list_state = ListState::new(100, ListAlignment::Top, px(128.));
        let events = Rc::new(std::cell::Cell::new(0));
        let observed = events.clone();
        list_state.set_scroll_handler(move |_, _, _| observed.set(observed.get() + 1));
        struct View(ListState);
        impl Render for View {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().w(px(300.)).h(px(200.)).child(
                    list(self.0.clone(), |_, _, _| {
                        div().h(px(40.)).w_full().into_any_element()
                    })
                    .size_full(),
                )
            }
        }
        let (_view, cx) = cx.add_window_view(|_, _| View(list_state.clone()));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let target = ScrollTarget::List(list_state);
        let before = target.position();
        cx.update(|window, cx| {
            assert!(target.wheel(true, window, cx).is_some());
        });
        assert!(
            target.position() != before,
            "dispatched input must move the list"
        );
        assert_eq!(
            events.get(),
            1,
            "must use List's event handler, not scroll_by"
        );
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert!(target.wheel(false, window, cx).is_some());
        });
        assert!(
            target.position() == before,
            "reverse input must return to the initial position"
        );
        assert_eq!(events.get(), 2);
    }

    #[test]
    fn journal_offsets_preserve_negative_times_and_stream_loss() {
        let origin = Instant::now();
        assert_eq!(journal_offset(origin - Duration::from_nanos(7), origin), -7);
        assert_eq!(journal_offset(origin + Duration::from_nanos(9), origin), 9);
        assert_eq!(journal_offset(origin, origin), 0);
        use profiler::journal::{
            DrainedEntries, ForegroundJournalEntry as Entry, FrameSkipReason, FrameSkipped,
            IntervalBoundary, PlatformSignal,
        };
        let id = WindowId::from(1);
        let mut evidence = JournalEvidence::default();
        evidence.drain(
            DrainedEntries {
                lost: 3,
                entries: vec![
                    Entry::PlatformSignal(PlatformSignal {
                        window_id: id,
                        signal_at: Some(origin - Duration::from_nanos(7)),
                        handled_at: origin,
                        source: gpui::FrameRequestSource::LocalSchedule,
                    }),
                    Entry::PlatformSignal(PlatformSignal {
                        window_id: WindowId::from(2),
                        signal_at: None,
                        handled_at: origin,
                        source: gpui::FrameRequestSource::NativeCallback,
                    }),
                    Entry::Boundary(IntervalBoundary::FrameSkipped(FrameSkipped {
                        window_id: id,
                        at: origin + Duration::from_nanos(9),
                        reason: FrameSkipReason::ThermalFrameRateLimit,
                    })),
                    Entry::Discontinuity { lost: 3 },
                ],
            },
            id,
            origin,
        );
        let report = evidence.report();
        assert_eq!(report["lost"], 3);
        assert_eq!(report["entries"].as_array().unwrap().len(), 3);
        assert_eq!(report["entries"][0]["signal_at_ns"], -7);
        assert_eq!(report["entries"][0]["source"], "local_schedule");
        assert_eq!(report["entries"][1]["at_ns"], 9);
        assert_eq!(report["entries"][1]["reason"], "thermal_frame_rate_limit");
        assert_eq!(report["entries"][2]["lost"], 3);
        assert_eq!(report["batches"][0]["entries"], 4);
    }

    fn drive(count: u64) -> Drive {
        let now = Instant::now();
        Drive {
            started: now,
            duration: Duration::from_secs(2),
            target: ScrollTarget::Pixels(ScrollHandle::new()),
            input_mode: InputMode::Programmatic,
            engine_initial: Value::Null,
            engine_final: Value::Null,
            engine_environment: json!({}),
            inputs: Vec::new(),
            errors: Vec::new(),
            previous: None,
            down: false,
            input_interval: Duration::from_millis(1),
            refresh_interval: None,
            traversal: Traversal {
                reached_end: false,
                laps: 1,
                last_progress_ns: 0,
            },
            new_draw_presents_ns: Vec::new(),
            successful_submission_diagnostic: Value::Null,
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
                        px: 0.,
                    },
                    down: true,
                    moved: false,
                    vsync: "disabled",
                    thermal_state: "nominal",
                    window_active: true,
                },
                Observation {
                    ns: 250_000_000,
                    count: Some(count),
                    position: Position {
                        item: None,
                        offset: 8.,
                        px: 8.,
                    },
                    down: false,
                    moved: true,
                    vsync: "disabled",
                    thermal_state: "nominal",
                    window_active: true,
                },
            ],
        }
    }
    #[test]
    fn rate_near_refresh_is_diagnostic_and_low_counter_rate_still_fails() {
        let mut paced = drive(15); // 60 FPS over the 250 ms window
        paced.refresh_interval = Some(Duration::from_nanos(16_666_667));
        let report = paced.report(json!({"acceptance_eligible":true}));
        assert_eq!(report["observed_rate_near_or_below_refresh"], true);
        assert_eq!(report["acceptance_eligible"], false);
        assert_eq!(report["exit_code"], 1);
        let mut fast = drive(250);
        fast.refresh_interval = Some(Duration::from_nanos(16_666_667));
        assert_eq!(
            fast.report(json!({}))["observed_rate_near_or_below_refresh"],
            false
        );
    }
    #[test]
    fn fast_attempts_covered_by_success_count_and_logical_lap_never_pass() {
        let mut candidate = drive(126);
        candidate.new_draw_presents_ns = (0..126).map(|i| i * 1_980_000).collect();
        let report = candidate.report(json!({"acceptance_eligible":true}));
        assert_eq!(
            report["coverage"]["logical_offset_traversal_complete"],
            true
        );
        assert_eq!(report["coverage"]["complete_gallery_traversal"], false);
        assert_eq!(
            report["submission_attempt_diagnostic"]["aggregate_counter_covers_attempt_count"],
            true
        );
        assert!(report["submission_attempt_diagnostic"]["all_submissions_successful"].is_null());
        assert_eq!(report["acceptance_eligible"], false);
        assert_eq!(report["exit_code"], 2);
        // Even fast attempt timestamps cannot mask actual successful counts <500 FPS.
        let mut slow_counter = drive(124);
        slow_counter.new_draw_presents_ns = (0..124).map(|i| i * 1_000_000).collect();
        let report = slow_counter.report(json!({"acceptance_eligible":true}));
        assert_eq!(report["exit_code"], 1);
        assert_eq!(report["acceptance_eligible"], false);
    }
    #[test]
    fn incomplete_traversal_is_not_eligible() {
        let mut partial = drive(126);
        partial.traversal.laps = 0;
        partial.new_draw_presents_ns = (0..126).map(|i| i * 1_980_000).collect();
        let report = partial.report(json!({"acceptance_eligible":true}));
        assert_eq!(report["coverage"]["complete_gallery_traversal"], false);
        assert_eq!(report["acceptance_eligible"], false);
        assert_eq!(report["exit_code"], 2);
    }
    #[test]
    fn successful_retry_without_new_draw_uses_marker_and_preserves_unknown() {
        let origin = Instant::now();
        let id = WindowId::from(1);
        let present = |at, marker| {
            FrameEvent::Present(gpui::profiler::PresentTiming {
                window_id: id,
                present_start: origin + Duration::from_nanos(at),
                present_end: origin + Duration::from_nanos(at + 1),
                animation_interval: None,
                successful_new_frame: marker,
            })
        };
        // A failed attempt, successful retry with no new Draw, re-present,
        // and unsupported success reporting must remain distinct.
        let events = [
            present(10, Some(false)),
            present(20, Some(true)),
            present(30, Some(false)),
            present(40, None),
        ];
        let report = successful_submissions(&events, id, origin);
        assert_eq!(report["successful_new_frame_end_ns"], json!([21]));
        assert_eq!(report["successful_new_frame_count"], 1);
        assert_eq!(report["false_count"], 2);
        assert_eq!(report["unknown_count"], 1);
        assert!(report["raw_markers"][3]["successful_new_frame"].is_null());
    }

    #[test]
    fn re_presents_are_not_new_draw_submissions() {
        use gpui::profiler::{DrawPhases, FrameTiming, PresentTiming};
        let origin = Instant::now();
        let id = WindowId::from(1);
        let draw = FrameTiming {
            window_id: id,
            dirty_at: None,
            invalidations: 1,
            draw_start: origin,
            draw_end: origin + Duration::from_micros(100),
            refresh_interval: None,
            signal_at: None,
            phases: DrawPhases::default(),
        };
        let present = |at: u64| PresentTiming {
            window_id: id,
            present_start: origin + Duration::from_micros(at),
            present_end: origin + Duration::from_micros(at + 10),
            animation_interval: None,
            successful_new_frame: None,
        };
        let events = [
            FrameEvent::Draw(draw),
            FrameEvent::Present(present(200)),
            FrameEvent::Present(present(400)),
        ];
        assert_eq!(new_draw_presents(&events, id, origin), vec![210_000]);
    }
    #[test]
    fn window_rate_never_claims_strict_fps_acceptance_and_preserves_coalescing() {
        let report = drive(125)
            .report(json!({"acceptance_eligible":true,"summary":{"over_budget_frames":0}}));
        assert_eq!(report["minimum_observed_window_fps"], 500.);
        assert_eq!(report["raw_observations"][1]["counter_increment"], 125);
        assert_eq!(report["raw_observations"][1]["thermal_state"], "nominal");
        assert_eq!(report["raw_observations"][1]["window_active"], true);
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
                px: 0.,
            },
            down: false,
            moved: true,
            vsync: "disabled",
            thermal_state: "nominal",
            window_active: true,
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
