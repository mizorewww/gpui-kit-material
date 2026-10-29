//! Opt-in whole-window CPU frame evidence. Never enabled by the ordinary gallery.
use gpui::{
    App, Window, WindowId,
    profiler::{self, CpuFrameSource, FrameEvent, FrameTiming, FrameTimingCollector},
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

const BUDGET_NS: u128 = 2_000_000;
// Pinned gpui-pre profiler.rs uses a 16 MiB ring. A full returned batch may
// have lost entries: conservatively reject it rather than claiming completeness.
const RING_CAPACITY: usize = (16 * 1024 * 1024) / std::mem::size_of::<FrameEvent>();

pub struct Session {
    output: PathBuf,
    duration: Duration,
    collector: FrameTimingCollector,
    metadata: Value,
}
fn command(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
/// Must run before opening the first window, so the collector includes its first draw.
pub fn prepare() -> Option<Session> {
    let output = std::env::var_os("MATERIAL_PROFILE_OUTPUT")?;
    if std::env::args_os().any(|arg| arg == "--capture") {
        eprintln!("Frame profiling and screenshot capture must run separately");
        std::process::exit(2);
    }
    let seconds = std::env::var("MATERIAL_PROFILE_SECONDS").unwrap_or_else(|_| "10".into());
    let seconds: f64 = seconds.parse().unwrap_or_else(|_| {
        eprintln!("MATERIAL_PROFILE_SECONDS must be a finite positive number");
        std::process::exit(2)
    });
    if !seconds.is_finite() || !(0.1..=3600.).contains(&seconds) {
        eprintln!("MATERIAL_PROFILE_SECONDS must be in 0.1..=3600");
        std::process::exit(2);
    }
    // These subprocesses finish before tracing. No hardware/git probe runs in a frame.
    let status = command(
        "git",
        &["-C", env!("CARGO_MANIFEST_DIR"), "status", "--porcelain"],
    );
    let metadata = json!({
        "runtime_source_commit": command("git", &["-C", env!("CARGO_MANIFEST_DIR"), "rev-parse", "HEAD"]),
        "runtime_source_dirty": status.as_ref().map(|s| !s.is_empty()), "runtime_source_status":status,
        "binary_build_identity":Value::Null,"build_identity_note":"runtime Git state is not binary provenance; build commit and binary hash must be recorded by the external build runner",
        "release": !cfg!(debug_assertions), "debug_assertions":cfg!(debug_assertions),
        "executable":std::env::current_exe().ok(),
        "os":std::env::consts::OS, "arch":std::env::consts::ARCH,
        "hardware": {"uname":command("uname", &["-a"]),
            "model":command("sysctl", &["-n", "hw.model"]),
            "cpu":command("sysctl", &["-n", "machdep.cpu.brand_string"]),
            "memory_bytes":command("sysctl", &["-n", "hw.memsize"]),
            "available_parallelism":std::thread::available_parallelism().ok().map(usize::from)},
        "requested_seconds":seconds,
        "backend": if gpui_kit::GPUI_FAST { "native GPUI Fast" } else { "native GPUI upstream" },
        "renderer": "concrete renderer not exposed by FrameTimingCollector",
        "build_flags": {"rustflags_at_compile":option_env!("RUSTFLAGS"),"encoded_rustflags_at_compile":option_env!("CARGO_ENCODED_RUSTFLAGS"),"feature":"frame-profiler","unavailable_flags_are_null":true},
        "initial_gallery_state":{"theme":"baseline","mode":"light","contrast":"standard","text_scale":1,"density":"auto"},
        "scenario_coverage":"startup and naturally scheduled frames only; interaction matrix not automated",
        "workload":"gallery startup and naturally requested frames; no synthetic refresh, no warm-up exclusion",
        "gpu_time_ns":Value::Null,"gpu_measurement":"unavailable: Present measures CPU platform submission, not GPU execution",
        "ring_capacity_events":RING_CAPACITY,"ring_overflow_policy":"full batch is incomplete, even if no loss can be proven"
    });
    profiler::set_trace_enabled(true);
    Some(Session {
        output: output.into(),
        duration: Duration::from_secs_f64(seconds),
        collector: FrameTimingCollector::new(),
        metadata,
    })
}

struct Recording {
    events: Vec<FrameEvent>,
    errors: Vec<String>,
    batch_sizes: Vec<usize>,
}
impl Recording {
    fn collect(&mut self, collector: &mut FrameTimingCollector) {
        let events = collector.collect_unseen();
        self.batch_sizes.push(events.len());
        if events.len() >= RING_CAPACITY {
            self.errors
                .push("frame ring full: event loss cannot be excluded".into());
        }
        self.events.extend(events);
    }
}
impl Session {
    pub fn install(mut self, window: &Window, cx: &mut App) {
        let window_id = window.window_handle().window_id();
        let viewport = window.viewport_size();
        self.metadata["window"] = json!({"id":format!("{window_id:?}"),"viewport_logical_px":[f32::from(viewport.width),f32::from(viewport.height)],"dpr":window.scale_factor(),"rem_size_px":f32::from(window.rem_size()),"snapshot":"initial window; viewport changes are not tracked"});
        let quit_output = self.output.clone();
        cx.on_app_quit(move |_| {
            let output = quit_output.clone();
            async move {
                let evidence = json!({"schema":"material-gallery-cpu-frames-v2","complete":false,"acceptance_eligible":false,"errors":["application quit before timed capture completed"]});
                let _ = std::fs::write(output, serde_json::to_vec_pretty(&evidence).unwrap());
                eprintln!("Frame profile incomplete: application quit before timed capture completed");
                std::process::exit(2);
            }
        }).detach();
        let executor = cx.background_executor().clone();
        cx.spawn(async move |_| {
            let started = Instant::now();
            let mut recording = Recording {
                events: Vec::new(),
                errors: Vec::new(),
                batch_sizes: Vec::new(),
            };
            loop {
                executor
                    .timer(
                        self.duration
                            .saturating_sub(started.elapsed())
                            .min(Duration::from_millis(100)),
                    )
                    .await;
                recording.collect(&mut self.collector);
                if started.elapsed() >= self.duration {
                    break;
                }
            }
            profiler::set_trace_enabled(false);
            let report = report(recording, window_id, self.metadata, started.elapsed());
            let code = exit_code(&report);
            let write = (|| -> anyhow::Result<()> {
                if let Some(parent) = self.output.parent().filter(|p| !p.as_os_str().is_empty()) {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&self.output, serde_json::to_vec_pretty(&report)?)?;
                Ok(())
            })();
            if let Err(error) = write {
                eprintln!("Could not write frame profile: {error:#}");
                std::process::exit(2);
            }
            eprintln!("Frame profile: {} (exit {code})", self.output.display());
            std::process::exit(code);
        })
        .detach();
    }
}
fn exit_code(report: &Value) -> i32 {
    if report["summary"]["over_budget_frames"].as_u64().unwrap() > 0
        || report["diagnostic_stage_summary"]["over_budget_frames"]
            .as_u64()
            .unwrap()
            > 0
    {
        1
    } else if !report["acceptance_eligible"].as_bool().unwrap() {
        2
    } else {
        0
    }
}
fn summary(values: &[u128]) -> Value {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let percentile = |percent: usize| {
        sorted
            .get((sorted.len() * percent).div_ceil(100).saturating_sub(1))
            .copied()
    };
    json!({"frames":values.len(),"max_ns":sorted.last(),"median_ns":percentile(50),"p95_ns":percentile(95),"p99_ns":percentile(99),"over_budget_frames":values.iter().filter(|v| **v>BUDGET_NS).count()})
}
fn report(mut recording: Recording, target: WindowId, metadata: Value, elapsed: Duration) -> Value {
    let mut pending: HashMap<WindowId, FrameTiming> = HashMap::new();
    let mut last_submitted: HashMap<WindowId, FrameTiming> = HashMap::new();
    let mut draws = HashMap::new();
    let mut presentations = HashMap::new();
    let mut draw_rows: Vec<Value> = Vec::new();
    let mut draw_indices: HashMap<Instant, usize> = HashMap::new();
    let mut submission_rows = Vec::new();
    let mut raw = Vec::new();
    let mut frames = Vec::new();
    let mut costs = Vec::new();
    let mut scopes = Vec::new();
    let mut pairs = Vec::new();
    let origin = recording
        .events
        .iter()
        .map(|e| match e {
            FrameEvent::Draw(d) => d.draw_start,
            FrameEvent::Present(p) => p.present_start,
            FrameEvent::Complete(c) => c.start,
        })
        .min();
    for event in &recording.events {
        let Some(origin) = origin else {
            recording
                .errors
                .push("present without any recorded draw".into());
            break;
        };
        let timestamp = |at: Instant| {
            if at >= origin {
                (at - origin).as_nanos() as i128
            } else {
                -((origin - at).as_nanos() as i128)
            }
        };
        match *event {
            FrameEvent::Complete(scope) => {
                raw.push(json!({"event":"complete","window":format!("{:?}",scope.window_id),"source":format!("{:?}",scope.source),"start_ns":timestamp(scope.start),"end_ns":timestamp(scope.end),"duration_ns":scope.duration().as_nanos(),"draw_start_ns":scope.draw_start.map(timestamp),"present_start_ns":scope.present_start.map(timestamp),"draw_count":scope.draw_count,"present_count":scope.present_count,"next_frame_callbacks":scope.next_frame_callbacks,"power_interrupted":scope.power_interrupted}));
                if scope.window_id == target {
                    scopes.push(scope);
                }
            }
            FrameEvent::Draw(d) => {
                let phases = json!({"request_layout_ns":d.phases.request_layout.as_nanos(),"prepaint_ns":d.phases.prepaint.as_nanos(),"paint_ns":d.phases.paint.as_nanos()});
                raw.push(json!({"event":"draw","window":format!("{:?}",d.window_id),"start_ns":timestamp(d.draw_start),"end_ns":timestamp(d.draw_end),"dirty_ns":d.dirty_at.map(timestamp),"invalidations":d.invalidations,"refresh_interval_ns":d.refresh_interval.map(|v|v.as_nanos()),"signal_ns":d.signal_at.map(timestamp),"phases":phases}));
                if d.window_id != target {
                    continue;
                }
                if draws.insert(d.draw_start, d).is_some() {
                    recording.errors.push("duplicate raw draw timestamp".into());
                }
                if d.draw_end < d.draw_start {
                    recording
                        .errors
                        .push("invalid draw timestamp ordering".into());
                }
                if let Some(previous) = pending.insert(d.window_id, d)
                    && let Some(&index) = draw_indices.get(&previous.draw_start)
                {
                    draw_rows[index]["disposition"] = json!("superseded_before_submit");
                    draw_rows[index]["superseded_by_draw_start_ns"] =
                        json!(timestamp(d.draw_start));
                }
                draw_indices.insert(d.draw_start, draw_rows.len());
                draw_rows.push(json!({"draw_start_ns":timestamp(d.draw_start),"draw_end_ns":timestamp(d.draw_end),"disposition":"pending_at_capture_end"}));
            }
            FrameEvent::Present(p) => {
                raw.push(json!({"event":"present","window":format!("{:?}",p.window_id),"start_ns":timestamp(p.present_start),"end_ns":timestamp(p.present_end),"animation_interval_ns":p.animation_interval.map(|v|v.as_nanos())}));
                if p.window_id != target {
                    continue;
                }
                if presentations.insert(p.present_start, p).is_some() {
                    recording
                        .errors
                        .push("duplicate raw present timestamp".into());
                }
                if p.present_end < p.present_start {
                    recording
                        .errors
                        .push("invalid present timestamp ordering".into());
                }
                let fresh = pending.remove(&p.window_id);
                let d = fresh.or_else(|| last_submitted.get(&p.window_id).copied());
                submission_rows.push(json!({"present_start_ns":timestamp(p.present_start),"present_end_ns":timestamp(p.present_end),"disposition":if fresh.is_some() { "new_draw" } else if d.is_some() { "re_present" } else { "unknown_scene" },"draw_start_ns":d.map(|draw| timestamp(draw.draw_start))}));
                let Some(d) = d else {
                    recording.errors.push(format!(
                        "present references an unobserved scene: {:?}",
                        p.window_id
                    ));
                    continue;
                };
                if fresh.is_none() {
                    if p.present_start < d.draw_end {
                        recording.errors.push("re-present precedes its draw".into());
                    }
                    continue;
                }
                last_submitted.insert(p.window_id, d);
                if let Some(&index) = draw_indices.get(&d.draw_start) {
                    draw_rows[index]["disposition"] = json!("submitted");
                }
                if d.draw_end < d.draw_start
                    || p.present_end < p.present_start
                    || p.present_start < d.draw_end
                {
                    recording
                        .errors
                        .push("invalid draw/present timestamp ordering".into());
                    continue;
                }
                if p.window_id != target {
                    continue;
                }
                let draw = (d.draw_end - d.draw_start).as_nanos();
                let submit = (p.present_end - p.present_start).as_nanos();
                let total = draw + submit;
                costs.push(total);
                pairs.push((d, p));
                frames.push(json!({"index":frames.len(),"first_frame":frames.is_empty(),"draw_start_ns":timestamp(d.draw_start),"present_end_ns":timestamp(p.present_end),"draw_ns":draw,"submit_ns":submit,"measured_cpu_stage_ns":total,"wall_span_ns":(p.present_end-d.draw_start).as_nanos(),"request_layout_ns":d.phases.request_layout.as_nanos(),"prepaint_ns":d.phases.prepaint.as_nanos(),"paint_ns":d.phases.paint.as_nanos(),"measured_stages_over_budget":total>BUDGET_NS,"gpu_time_ns":Value::Null}));
            }
        }
    }
    for id in pending.keys() {
        recording
            .errors
            .push(format!("draw missing present at end of capture: {id:?}"));
    }
    if frames.is_empty() {
        recording
            .errors
            .push("no complete gallery frame captured".into());
    }
    let mut reasons = Vec::new();
    let mut draw_scopes = HashMap::new();
    let mut present_scopes = HashMap::new();
    let mut scope_costs = Vec::new();
    let mut scope_rows = Vec::new();
    let mut saw_initial = false;
    for (index, scope) in scopes.iter().enumerate() {
        if scope.end < scope.start
            || scope.power_interrupted
            || scope.draw_count > 1
            || scope.present_count > 1
        {
            recording.errors.push(format!(
                "invalid/interrupted/ambiguous complete scope {index}"
            ));
        }
        if (scope.draw_count == 0) != scope.draw_start.is_none()
            || (scope.present_count == 0) != scope.present_start.is_none()
        {
            recording.errors.push(format!(
                "complete scope {index} has inconsistent counts/timestamps"
            ));
        }
        if scope.source == CpuFrameSource::InitialDraw {
            saw_initial = true;
        }
        if scope.source == CpuFrameSource::TestDraw {
            reasons.push("test-driven frame is not native runtime acceptance".to_owned());
        }
        for at in [scope.draw_start, scope.present_start]
            .into_iter()
            .flatten()
        {
            if at < scope.start || at > scope.end {
                recording
                    .errors
                    .push(format!("nested timestamp outside complete scope {index}"));
            }
        }
        if let Some(at) = scope.draw_start {
            if draw_scopes.insert(at, index).is_some() {
                recording
                    .errors
                    .push("duplicate complete scope for draw".into());
            }
            match draws.get(&at) {
                Some(draw) if draw.draw_end <= scope.end => {}
                _ => recording.errors.push(format!(
                    "complete scope {index} draw has no contained raw Draw event"
                )),
            }
        }
        if let Some(at) = scope.present_start {
            if present_scopes.insert(at, index).is_some() {
                recording
                    .errors
                    .push("duplicate complete scope for present".into());
            }
            match presentations.get(&at) {
                Some(present) if present.present_end <= scope.end => {}
                _ => recording.errors.push(format!(
                    "complete scope {index} present has no contained raw Present event"
                )),
            }
        }
        let cost = scope.duration().as_nanos();
        scope_costs.push(cost);
        scope_rows.push(json!({"index":index,"source":format!("{:?}",scope.source),"cpu_elapsed_ns":cost,"draw_count":scope.draw_count,"present_count":scope.present_count,"next_frame_callbacks":scope.next_frame_callbacks,"over_budget":cost>BUDGET_NS,"gpu_time_ns":Value::Null}));
    }
    for at in draws.keys() {
        if !draw_scopes.contains_key(at) {
            recording
                .errors
                .push("raw draw has no owning complete scope".into());
        }
    }
    for at in presentations.keys() {
        if !present_scopes.contains_key(at) {
            recording
                .errors
                .push("raw present has no owning complete scope".into());
        }
    }
    if scopes.is_empty() {
        recording
            .errors
            .push("no complete CPU scope events recorded".into());
    }
    if !saw_initial {
        reasons.push("first synchronous window draw scope was not observed".into());
    }
    // Superseding an initial draw is normal event flow, but it does not
    // establish a single measured startup frame through the first submission.
    // Preserve the distinction between complete events and acceptance evidence.
    if let Some(first_present) = presentations.keys().min()
        && let Some(&submit_scope) = present_scopes.get(first_present)
        && scopes.iter().enumerate().any(|(index, scope)| {
            scope.source == CpuFrameSource::InitialDraw && index != submit_scope
        })
    {
        reasons.push("initial window drawing and first submission span separate CPU scopes; startup work is diagnostic, not a complete first-frame measurement".into());
    }
    for (index, (draw, present)) in pairs.iter().enumerate() {
        match (
            draw_scopes.get(&draw.draw_start),
            present_scopes.get(&present.present_start),
        ) {
            (Some(&d), Some(&p)) => {
                frames[index]["complete_scope_indices"] =
                    json!(if d == p { vec![d] } else { vec![d, p] });
                if scopes[d].end < draw.draw_end || scopes[p].end < present.present_end {
                    recording
                        .errors
                        .push(format!("frame {index} exceeds its owning scope"));
                }
                if d == p {
                    frames[index]["cpu_frame_ns"] = json!(scopes[d].duration().as_nanos());
                    frames[index]["over_budget"] =
                        json!(scopes[d].duration().as_nanos() > BUDGET_NS);
                } else {
                    frames[index]["cpu_frame_ns"] = Value::Null;
                    frames[index]["over_budget"] = Value::Null;
                    frames[index]["diagnostic_split_scope_cpu_sum_ns"] =
                        json!(scopes[d].duration().as_nanos() + scopes[p].duration().as_nanos());
                    reasons.push(format!("frame {index} draw and submit belong to separate scopes; split CPU sum is diagnostic, not a complete-frame measurement"));
                }
            }
            _ => {
                recording.errors.push(format!(
                    "frame {index} missing an exact complete scope association"
                ));
            }
        }
    }
    json!({"schema":"material-gallery-cpu-frames-v2","statistics_definition":"nearest-rank quantiles; median is p50 (lower middle for even sample counts)","budget_ns":BUDGET_NS,"budget_comparison":"every complete CPU scope must be at or below budget; known over-budget work fails even when acceptance is incomplete","acceptance_eligible":recording.errors.is_empty() && reasons.is_empty(),"eligibility_reasons":reasons,"scope":"whole synchronous CPU frame scopes plus exact draw/present associations","cpu_work_definition":"Complete event end-start includes callbacks, draw, submission and cleanup; split initial draw and submission remain separately measured, idle gaps are not CPU work","complete":recording.errors.is_empty(),"errors":recording.errors,"metadata":metadata,"capture_elapsed_ns":elapsed.as_nanos(),"summary":summary(&scope_costs),"first_scope_summary":summary(&scope_costs[..scope_costs.len().min(1)]),"subsequent_scopes_summary":summary(&scope_costs[scope_costs.len().min(1)..]),"diagnostic_stage_summary":summary(&costs),"first_frame_stage_summary":summary(&costs[..costs.len().min(1)]),"subsequent_frames_stage_summary":summary(&costs[costs.len().min(1)..]),"collector_batch_sizes":recording.batch_sizes,"raw_events":raw,"cpu_scopes":scope_rows,"draw_lifecycle":draw_rows,"submissions":submission_rows,"frames":frames})
}

#[cfg(test)]
mod tests {
    use super::{Recording, exit_code, report, summary};
    use gpui::{
        WindowId,
        profiler::{
            CpuFrameSource, CpuFrameTiming, DrawPhases, FrameEvent, FrameTiming, PresentTiming,
        },
    };
    use serde_json::json;
    use std::time::{Duration, Instant};

    fn draw() -> FrameTiming {
        let start = Instant::now();
        FrameTiming {
            window_id: WindowId::from(1),
            dirty_at: Some(start),
            invalidations: 1,
            draw_start: start,
            draw_end: start + Duration::from_millis(1),
            refresh_interval: None,
            signal_at: None,
            phases: DrawPhases::default(),
        }
    }
    #[test]
    fn partial_events_never_claim_completeness() {
        let draw = draw();
        let evidence = report(
            Recording {
                events: vec![FrameEvent::Draw(draw)],
                errors: vec![],
                batch_sizes: vec![1],
            },
            draw.window_id,
            json!({}),
            Duration::ZERO,
        );
        assert_eq!(evidence["complete"], false);
        assert_eq!(evidence["acceptance_eligible"], false);
    }
    #[test]
    fn first_frame_includes_submit_but_remains_diagnostic() {
        let draw = draw();
        let present = PresentTiming {
            window_id: draw.window_id,
            present_start: draw.draw_end + Duration::from_millis(3),
            present_end: draw.draw_end + Duration::from_millis(4),
            animation_interval: None,
        };
        let evidence = report(
            Recording {
                events: vec![FrameEvent::Draw(draw), FrameEvent::Present(present)],
                errors: vec![],
                batch_sizes: vec![2],
            },
            draw.window_id,
            json!({}),
            Duration::ZERO,
        );
        assert_eq!(evidence["complete"], false);
        assert_eq!(evidence["acceptance_eligible"], false);
        assert_eq!(evidence["frames"][0]["measured_cpu_stage_ns"], 2_000_000);
        assert_eq!(evidence["frames"][0]["wall_span_ns"], 5_000_000);
        assert_eq!(
            evidence["first_frame_stage_summary"]["over_budget_frames"],
            0
        );
    }
    #[test]
    fn inclusive_budget_accepts_boundary_but_counts_over_budget_first_frame() {
        let stats = summary(&[2_000_001, 2_000_000, 100, 200]);
        assert_eq!(stats["over_budget_frames"], 1);
        assert_eq!(stats["max_ns"], 2_000_001);
        assert_eq!(stats["p99_ns"], 2_000_001);
    }
    #[test]
    fn split_initial_frame_keeps_idle_out_of_cpu_but_is_not_eligible() {
        let d = draw();
        let p = PresentTiming {
            window_id: d.window_id,
            present_start: d.draw_end + Duration::from_millis(30),
            present_end: d.draw_end + Duration::from_millis(31),
            animation_interval: None,
        };
        let first = CpuFrameTiming {
            window_id: d.window_id,
            source: CpuFrameSource::InitialDraw,
            start: d.draw_start,
            end: d.draw_end + Duration::from_micros(100),
            draw_start: Some(d.draw_start),
            present_start: None,
            draw_count: 1,
            present_count: 0,
            next_frame_callbacks: 0,
            power_interrupted: false,
        };
        let submit = CpuFrameTiming {
            window_id: d.window_id,
            source: CpuFrameSource::Platform,
            start: p.present_start,
            end: p.present_end,
            draw_start: None,
            present_start: Some(p.present_start),
            draw_count: 0,
            present_count: 1,
            next_frame_callbacks: 0,
            power_interrupted: false,
        };
        let evidence = report(
            Recording {
                events: vec![
                    FrameEvent::Draw(d),
                    FrameEvent::Complete(first),
                    FrameEvent::Present(p),
                    FrameEvent::Complete(submit),
                ],
                errors: vec![],
                batch_sizes: vec![4],
            },
            d.window_id,
            json!({}),
            Duration::ZERO,
        );
        assert_eq!(evidence["complete"], true);
        assert_eq!(evidence["acceptance_eligible"], false);
        assert!(evidence["frames"][0]["cpu_frame_ns"].is_null());
        assert_eq!(
            evidence["frames"][0]["diagnostic_split_scope_cpu_sum_ns"],
            2_100_000
        );
        assert_eq!(evidence["summary"]["max_ns"], 1_100_000);
        assert_eq!(exit_code(&evidence), 2);
    }
    #[test]
    fn callback_only_over_budget_scope_fails_even_without_complete_frame_pairs() {
        let start = Instant::now();
        let id = WindowId::from(1);
        let scope = CpuFrameTiming {
            window_id: id,
            source: CpuFrameSource::Platform,
            start,
            end: start + Duration::from_nanos(2_000_001),
            draw_start: None,
            present_start: None,
            draw_count: 0,
            present_count: 0,
            next_frame_callbacks: 1,
            power_interrupted: false,
        };
        let evidence = report(
            Recording {
                events: vec![FrameEvent::Complete(scope)],
                errors: vec![],
                batch_sizes: vec![1],
            },
            id,
            json!({}),
            Duration::ZERO,
        );
        assert_eq!(evidence["acceptance_eligible"], false);
        assert_eq!(evidence["summary"]["over_budget_frames"], 1);
        assert_eq!(exit_code(&evidence), 1);
    }

    // Startup draw is replaced, then a platform frame is submitted and its
    // scene is submitted again without new drawing work.
    fn coalesced_startup(first_cost: Duration) -> Vec<FrameEvent> {
        let first = draw();
        let mut next = first;
        next.draw_start = first.draw_start + Duration::from_millis(10);
        next.draw_end = next.draw_start + Duration::from_micros(100);
        let present = PresentTiming {
            window_id: first.window_id,
            present_start: next.draw_end,
            present_end: next.draw_end + Duration::from_micros(100),
            animation_interval: None,
        };
        let repeated = PresentTiming {
            present_start: present.present_end + Duration::from_millis(20),
            present_end: present.present_end
                + Duration::from_millis(20)
                + Duration::from_micros(100),
            ..present
        };
        let initial = CpuFrameTiming {
            window_id: first.window_id,
            source: CpuFrameSource::InitialDraw,
            start: first.draw_start,
            end: first.draw_start + first_cost,
            draw_start: Some(first.draw_start),
            present_start: None,
            draw_count: 1,
            present_count: 0,
            next_frame_callbacks: 0,
            power_interrupted: false,
        };
        let submitted = CpuFrameTiming {
            source: CpuFrameSource::Platform,
            start: next.draw_start,
            end: present.present_end,
            draw_start: Some(next.draw_start),
            present_start: Some(present.present_start),
            present_count: 1,
            ..initial
        };
        let resubmitted = CpuFrameTiming {
            start: repeated.present_start,
            end: repeated.present_end,
            draw_start: None,
            present_start: Some(repeated.present_start),
            draw_count: 0,
            ..submitted
        };
        vec![
            FrameEvent::Draw(first),
            FrameEvent::Complete(initial),
            FrameEvent::Draw(next),
            FrameEvent::Present(present),
            FrameEvent::Complete(submitted),
            FrameEvent::Present(repeated),
            FrameEvent::Complete(resubmitted),
        ]
    }

    fn report_events(events: Vec<FrameEvent>) -> serde_json::Value {
        report(
            Recording {
                batch_sizes: vec![events.len()],
                events,
                errors: vec![],
            },
            WindowId::from(1),
            json!({}),
            Duration::ZERO,
        )
    }

    #[test]
    fn superseded_startup_and_represent_keep_all_cpu_scopes_without_idle() {
        let evidence = report_events(coalesced_startup(Duration::from_millis(1)));
        assert_eq!(evidence["complete"], true);
        assert_eq!(evidence["acceptance_eligible"], false);
        assert!(
            evidence["eligibility_reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason
                    .as_str()
                    .unwrap()
                    .contains("initial window drawing and first submission"))
        );
        assert_eq!(
            evidence["draw_lifecycle"][0]["disposition"],
            "superseded_before_submit"
        );
        assert_eq!(evidence["draw_lifecycle"][1]["disposition"], "submitted");
        assert_eq!(evidence["submissions"][1]["disposition"], "re_present");
        assert_eq!(evidence["summary"]["frames"], 3);
        assert_eq!(evidence["summary"]["max_ns"], 1_000_000);
        assert_eq!(evidence["cpu_scopes"][2]["cpu_elapsed_ns"], 100_000);
        assert_eq!(exit_code(&evidence), 2);
    }

    #[test]
    fn scope_declared_present_cannot_hide_a_filtered_raw_present() {
        let mut events = coalesced_startup(Duration::from_millis(1));
        events.remove(3); // Keep the complete scope declaring this submission.
        let evidence = report_events(events);
        assert_eq!(evidence["complete"], false);
        assert_eq!(evidence["acceptance_eligible"], false);
        assert!(evidence["errors"].as_array().unwrap().iter().any(|error| {
            error
                .as_str()
                .unwrap()
                .contains("present has no contained raw Present")
        }));
        assert_eq!(evidence["summary"]["frames"], 3);
        assert_eq!(exit_code(&evidence), 2);
    }

    #[test]
    fn pending_draw_at_capture_boundary_remains_incomplete_and_budgeted() {
        let events = coalesced_startup(Duration::from_millis(1));
        let evidence = report_events(events[..2].to_vec());
        assert_eq!(
            evidence["draw_lifecycle"][0]["disposition"],
            "pending_at_capture_end"
        );
        assert_eq!(evidence["summary"]["frames"], 1);
        assert_eq!(evidence["complete"], false);
        assert_eq!(exit_code(&evidence), 2);
    }

    #[test]
    fn superseded_over_budget_work_fails_even_when_submission_evidence_is_missing() {
        let mut events = coalesced_startup(Duration::from_nanos(2_000_001));
        events.remove(3);
        let evidence = report_events(events);
        assert_eq!(evidence["summary"]["over_budget_frames"], 1);
        assert_eq!(evidence["complete"], false);
        assert_eq!(exit_code(&evidence), 1);
        let boundary = report_events(coalesced_startup(Duration::from_millis(2)));
        assert_eq!(boundary["summary"]["over_budget_frames"], 0);
        assert_eq!(boundary["complete"], true);
        assert_eq!(boundary["acceptance_eligible"], false);
        assert_eq!(exit_code(&boundary), 2);
    }
}
