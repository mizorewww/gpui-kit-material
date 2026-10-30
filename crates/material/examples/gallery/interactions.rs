//! Opt-in scripted interaction profile (`MATERIAL_PROFILE_SCRIPT=interactions`
//! with `MATERIAL_PROFILE_OUTPUT=<frames.json>`): page switches between
//! Button, Text field, Date picker and Examples, hover and press in the
//! Button preview, and continuous wheel scrolling of Overview and a Specs
//! tab. Every CPU frame scope is kept raw and labelled with the step that
//! was active when it started; `page_switch_ns` holds the first drawing
//! scope after each switch. Never enabled by the ordinary gallery and never
//! decides acceptance (`scripts/profile-gallery.py --summarize` judges it).
use super::{Gallery, Tab, catalog};
use gpui::{
    profiler::{self, CpuFrameSource, FrameEvent, FrameTimingCollector},
    *,
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

/// What a step does when it starts.
#[derive(Clone, Copy)]
enum Action {
    Navigate(&'static str),
    Tab(Tab),
    Move(f32, f32),
    Press(f32, f32),
    Release(f32, f32),
    /// Wheel 8 px per frame for the step, down then up.
    Scroll,
}

type Step = (&'static str, Action, u64);

/// The script: an action, then this much time before the next step.
const SCRIPT: &[Step] = &[
    ("switch:button", Action::Navigate("button"), 900),
    ("hover:enter", Action::Move(582., 492.), 700),
    ("press:down", Action::Press(582., 492.), 500),
    ("press:up", Action::Release(582., 492.), 700),
    ("hover:leave", Action::Move(700., 700.), 700),
    ("switch:text-field", Action::Navigate("text-field"), 900),
    ("switch:date-picker", Action::Navigate("date-picker"), 900),
    ("switch:examples", Action::Navigate("examples"), 900),
    ("switch:button-again", Action::Navigate("button"), 700),
    ("switch:overview", Action::Navigate("overview"), 700),
    ("scroll:overview", Action::Scroll, 3000),
    ("switch:button-specs", Action::Navigate("button"), 500),
    ("switch:tab-specs", Action::Tab(Tab::Specs), 900),
    ("scroll:specs", Action::Scroll, 3000),
    ("idle", Action::Move(700., 700.), 1000),
];

/// Profiling aids for a sampler: one interaction repeated for ~10 s.
const HOVER: &[Step] = &[
    ("hover:enter", Action::Move(582., 492.), 500),
    ("press:down", Action::Press(582., 492.), 400),
    ("press:up", Action::Release(582., 492.), 500),
    ("hover:leave", Action::Move(700., 700.), 500),
];
const SWITCH: &[Step] = &[
    ("switch:text-field", Action::Navigate("text-field"), 400),
    ("switch:date-picker", Action::Navigate("date-picker"), 400),
    ("switch:examples", Action::Navigate("examples"), 400),
    ("switch:button", Action::Navigate("button"), 400),
];

/// `interactions` runs [`SCRIPT`] once; `hover`, `switch`, `scroll-specs`
/// and `scroll-overview` repeat one interaction.
fn script(name: &str) -> Option<Vec<Step>> {
    let repeat = |lead: &[Step], body: &[Step], times: usize| {
        let mut steps = lead.to_vec();
        for _ in 0..times {
            steps.extend_from_slice(body);
        }
        steps
    };
    Some(match name {
        "interactions" => SCRIPT.to_vec(),
        "hover" => repeat(
            &[("switch:button", Action::Navigate("button"), 600)],
            HOVER,
            6,
        ),
        "switch" => repeat(&[], SWITCH, 6),
        "scroll-specs" => repeat(
            &[
                ("switch:button", Action::Navigate("button"), 300),
                ("switch:tab-specs", Action::Tab(Tab::Specs), 600),
            ],
            &[("scroll:specs", Action::Scroll, 3000)],
            4,
        ),
        "scroll-overview" => repeat(&[], &[("scroll:overview", Action::Scroll, 3000)], 4),
        _ => return None,
    })
}

pub struct Session {
    steps: Vec<Step>,
    output: PathBuf,
    collector: FrameTimingCollector,
}

/// Must run before the window opens so the initial draw is collected.
pub fn prepare() -> Option<Session> {
    let steps = script(&std::env::var("MATERIAL_PROFILE_SCRIPT").ok()?)?;
    let output = std::env::var_os("MATERIAL_PROFILE_OUTPUT")?;
    profiler::set_trace_enabled(true);
    Some(Session {
        steps,
        output: output.into(),
        collector: FrameTimingCollector::new(),
    })
}

fn dispatch(window: &mut Window, cx: &mut App, input: PlatformInput) {
    window.dispatch_event(input, cx);
}

fn scroll_frames(
    window: &mut Window,
    cx: &mut App,
    list: ListState,
    down: Rc<RefCell<bool>>,
    end: Instant,
    moved: Rc<RefCell<Vec<f32>>>,
) {
    if Instant::now() >= end {
        return;
    }
    let half = end - Duration::from_millis(1500);
    let going_down = Instant::now() < half && *down.borrow();
    let position = point(px(800.), px(500.));
    let before = list.scroll_px_offset_for_scrollbar().y;
    dispatch(
        window,
        cx,
        PlatformInput::Scroll(ScrollEvent {
            position,
            delta: ScrollDelta::Pixels(point(px(0.), px(if going_down { -8. } else { 8. }))),
            ..Default::default()
        }),
    );
    let after = list.scroll_px_offset_for_scrollbar().y;
    moved.borrow_mut().push(f32::from(after - before));
    window.on_next_frame(move |window, cx| scroll_frames(window, cx, list, down, end, moved));
}

impl Session {
    pub fn install(mut self, gallery: Entity<Gallery>, window: &mut Window, cx: &mut App) {
        let handle = window.window_handle();
        let window_id = handle.window_id();
        let origin = Instant::now();
        let marks: Rc<RefCell<Vec<(&'static str, Instant)>>> = Default::default();
        let moved: Rc<RefCell<Vec<f32>>> = Default::default();
        cx.spawn(async move |cx| {
            let executor = cx.background_executor().clone();
            let mut events: Vec<FrameEvent> = Vec::new();
            executor.timer(Duration::from_millis(1500)).await;
            let steps = std::mem::take(&mut self.steps);
            for (label, action, wait) in steps {
                let gallery = gallery.clone();
                let marks = marks.clone();
                let moved = moved.clone();
                let _ = handle.update(cx, move |_, window, cx| {
                    marks.borrow_mut().push((label, Instant::now()));
                    match action {
                        Action::Navigate(id) => {
                            let entry = catalog::position(id).expect("catalog entry");
                            gallery.update(cx, |gallery, cx| gallery.navigate(entry, cx));
                        }
                        Action::Tab(tab) => {
                            gallery.update(cx, |gallery, cx| gallery.set_tab(tab, cx))
                        }
                        Action::Move(x, y) => dispatch(
                            window,
                            cx,
                            PlatformInput::MouseMove(MouseMoveEvent {
                                position: point(px(x), px(y)),
                                ..Default::default()
                            }),
                        ),
                        Action::Press(x, y) => dispatch(
                            window,
                            cx,
                            PlatformInput::MouseDown(MouseDownEvent {
                                button: MouseButton::Left,
                                position: point(px(x), px(y)),
                                click_count: 1,
                                ..Default::default()
                            }),
                        ),
                        Action::Release(x, y) => dispatch(
                            window,
                            cx,
                            PlatformInput::MouseUp(MouseUpEvent {
                                button: MouseButton::Left,
                                position: point(px(x), px(y)),
                                click_count: 1,
                                ..Default::default()
                            }),
                        ),
                        Action::Scroll => {
                            let list = gallery.read(cx).list.clone();
                            let end = Instant::now() + Duration::from_millis(wait);
                            scroll_frames(
                                window,
                                cx,
                                list,
                                Rc::new(RefCell::new(true)),
                                end,
                                moved,
                            );
                        }
                    }
                });
                executor.timer(Duration::from_millis(wait)).await;
                events.extend(self.collector.collect_unseen());
            }
            let (gpu, _) = profiler::take_gpu_executions();
            profiler::set_trace_enabled(false);
            let report = report(
                events,
                gpu,
                window_id,
                origin,
                &marks.borrow(),
                &moved.borrow(),
            );
            if let Some(parent) = self.output.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            std::fs::write(&self.output, serde_json::to_vec_pretty(&report).unwrap())
                .expect("write interaction profile");
            eprintln!("Interaction profile: {}", self.output.display());
            std::process::exit(0);
        })
        .detach();
    }
}

fn report(
    events: Vec<FrameEvent>,
    gpu: Vec<profiler::GpuExecution>,
    window_id: WindowId,
    origin: Instant,
    marks: &[(&'static str, Instant)],
    moved: &[f32],
) -> Value {
    let ns = |at: Instant| -> i128 {
        if at >= origin {
            (at - origin).as_nanos() as i128
        } else {
            -((origin - at).as_nanos() as i128)
        }
    };
    let step_at = |at: Instant| -> &'static str {
        marks
            .iter()
            .rev()
            .find(|(_, mark)| *mark <= at)
            .map_or("startup", |(label, _)| label)
    };
    let mut draws = Vec::new();
    let mut scopes = Vec::new();
    for event in &events {
        match event {
            FrameEvent::Draw(draw) if draw.window_id == window_id => draws.push(*draw),
            FrameEvent::Complete(scope) if scope.window_id == window_id => scopes.push(*scope),
            _ => {}
        }
    }
    let rows: Vec<Value> = scopes
        .iter()
        .map(|scope| {
            let phases = scope
                .draw_start
                .and_then(|start| draws.iter().find(|draw| draw.draw_start == start))
                .map(|draw| {
                    json!({"request_layout_ns": draw.phases.request_layout.as_nanos(),
                        "prepaint_ns": draw.phases.prepaint.as_nanos(),
                        "paint_ns": draw.phases.paint.as_nanos(),
                        "draw_ns": (draw.draw_end - draw.draw_start).as_nanos()})
                });
            let gpu_ns: u128 = gpu
                .iter()
                .filter(|g| g.submitted_at >= scope.start && g.submitted_at <= scope.end)
                .map(|g| ((g.gpu_end_s - g.gpu_start_s).max(0.) * 1e9) as u128)
                .sum();
            json!({
                "source": format!("{:?}", scope.source),
                "step": step_at(scope.start),
                "start_ns": ns(scope.start),
                "cpu_elapsed_ns": scope.duration().as_nanos(),
                "presentation_wait_ns": scope.presentation_wait.as_nanos(),
                "draw_count": scope.draw_count,
                "present_count": scope.present_count,
                "gpu_time_ns": if scope.present_count > 0 { json!(gpu_ns) } else { Value::Null },
                "phases": phases,
            })
        })
        .collect();
    // The first drawing scope after each switch is that switch's cost.
    let mut switches = Vec::new();
    for (label, at) in marks {
        if !label.starts_with("switch:") {
            continue;
        }
        if let Some(scope) = scopes
            .iter()
            .find(|scope| scope.start >= *at && scope.draw_count > 0)
        {
            switches.push(json!({"step": label, "cpu_elapsed_ns": scope.duration().as_nanos()}));
        }
    }
    let initial = scopes
        .iter()
        .position(|scope| scope.source == CpuFrameSource::InitialDraw);
    json!({
        "schema": "gallery-interactions-v1",
        "metadata": {
            "release": !cfg!(debug_assertions),
            "backend": if gpui_kit::GPUI_FAST { "gpui-fast" } else { "gpui-pre" },
            "load_average_at_end": std::process::Command::new("sysctl").args(["-n", "vm.loadavg"]).output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned()),
            "script": std::env::var("MATERIAL_PROFILE_SCRIPT").ok(), "interactions_script": SCRIPT.iter().map(|(label, _, wait)| json!([label, wait])).collect::<Vec<_>>(),
            "cpu_elapsed_definition": "wall time of each complete main-thread CPU frame scope (render, layout, prepaint, paint, submission, cleanup)",
            "steady_definition": "cpu_scopes holds the initial draw first, then every drawing scope that is not the first one after a page switch",
        },
        "marks_ns": marks.iter().map(|(label, at)| json!([label, ns(*at)])).collect::<Vec<_>>(),
        "scroll_moved_px": moved,
        "page_switch_ns": switches.iter().map(|s| s["cpu_elapsed_ns"].clone()).collect::<Vec<_>>(),
        "page_switches": switches,
        "initial_scope_index": initial,
        "all_scopes": rows,
    })
}
