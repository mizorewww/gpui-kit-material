//! Retired instructions per Gallery frame, with what each frame rebuilt.
//! Compiled only with the `instruction-bench` feature; independent of the frame
//! profiler. Frames are bracketed with `on_next_frame`, as `component_bench` does.
//!
//! ```sh
//! MATERIAL_INSTR_OUTPUT=/tmp/g.json MATERIAL_INSTR_ENTRY=button MATERIAL_INSTR_TAB=specs \
//! MATERIAL_INSTR_MODE=scroll cargo run --release -p gpui-kit-material-gallery \
//!     --features instruction-bench
//! ```
//!
//! Modes: `scroll` (one 8 px wheel per frame, reversing every 60 frames), `redraw`
//! (notifies the Gallery each frame), `idle` (nothing), `switch` (navigates
//! between `MATERIAL_INSTR_ENTRY` and `MATERIAL_INSTR_ENTRY2` every 12 frames), and
//! `tour` (see [`Tour`]). Every report has `startup`: the counters at each step from
//! `main` to the first presented frame, see [`mark`].
use super::{Gallery, Tab, catalog};
use gpui::*;
use serde_json::{Value, json};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    time::Duration,
};

#[path = "../../bench/src/perf_counters.rs"]
mod perf_counters;
use perf_counters::{Counters, CountingAllocator};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// What a frame may rebuild.
#[derive(Clone, Copy)]
pub(super) enum Count {
    Gallery,
    Lead,
    Navigation,
    Row,
    Prepare,
    AfterLayout,
}
const COUNTS: usize = 6;

thread_local! {
    static TALLY: Cell<[u64; COUNTS]> = const { Cell::new([0; COUNTS]) };
    static STARTUP: RefCell<Vec<(&'static str, Counters)>> = const { RefCell::new(Vec::new()) };
    static STARTUP_FRAMES: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) };
}

/// Records the main thread's counters as a step of startup ends.
pub(super) fn mark(step: &'static str) {
    STARTUP.with(|marks| marks.borrow_mut().push((step, Counters::now())));
}

/// The frames after the window opened that startup's report ends with.
fn note_startup_frame(seen: usize, window: &Window) {
    let step = match seen {
        0 => "frame_1",
        1 => "frame_2",
        2 => "frame_3",
        3 => "frame_4",
        _ => return,
    };
    mark(step);
    let stats = phase_stats(window);
    let tally = TALLY.with(Cell::get);
    STARTUP_FRAMES.with(|frames| {
        frames.borrow_mut().push(json!({
            "step": step,
            "views_built": stats.views_built,
            "views_reused": stats.views_reused,
            "nodes_created": stats.nodes_created,
            "lines_shaped": stats.lines_shaped,
            "frames_drawn": window.layout_stats().frames,
            "gallery_renders": tally[Count::Gallery as usize],
            "lead_renders": tally[Count::Lead as usize],
            "row_builds": tally[Count::Row as usize],
        }))
    });
}

/// Each step's own counters: instructions, cycles, CPU ns, allocations, bytes.
fn startup_report() -> Value {
    STARTUP.with(|marks| {
        let marks = marks.borrow();
        let mut last = Counters::default();
        let mut steps = Vec::new();
        for (step, counters) in marks.iter() {
            steps.push(json!({"step": step, "counters": counters.since(last).row()}));
            last = *counters;
        }
        json!({
            "columns":["instructions","cycles","cpu_ns","allocations","allocated_bytes"],
            "steps":steps,
            "frames":STARTUP_FRAMES.with(|frames| frames.borrow().clone()),
        })
    })
}

/// Counts one rebuild. Costs a few instructions, in this feature's builds only.
pub(super) fn note(what: Count) {
    TALLY.with(|tally| {
        let mut counts = tally.get();
        counts[what as usize] += 1;
        tally.set(counts);
    });
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Scroll,
    Redraw,
    Idle,
    Switch,
    Tour,
}

/// What `tour` mode does: visit each stop of `MATERIAL_INSTR_TOUR`, `<entry>` or
/// `<entry>:specs`, for `MATERIAL_INSTR_TOUR_ROUNDS` rounds, one every
/// `MATERIAL_INSTR_TOUR_GAP` frames (more than a switch's own frames take to settle).
/// The stop's cost is the frames from its navigation until the next stop.
struct Tour {
    stops: Vec<(String, usize, bool)>,
    rounds: usize,
    gap: usize,
    /// The next stop, counted across rounds.
    next: usize,
    /// When the frame after the last navigation is due.
    since: usize,
    visits: Vec<Value>,
    frames: Vec<Value>,
    before: Option<PhaseStats>,
}

/// The engine's cumulative counters, as far as a stop reports them.
#[derive(Clone, Copy, Default)]
struct PhaseStats {
    build_ns: u128,
    prepaint_ns: u128,
    paint_ns: u128,
    compute_layout_ns: u128,
    shape_ns: u128,
    lines_shaped: u64,
    views_built: u64,
    views_reused: u64,
    nodes_created: u64,
    measure_calls: u64,
}

fn phase_stats(window: &Window) -> PhaseStats {
    let stats = window.layout_stats();
    PhaseStats {
        build_ns: stats.build_time.as_nanos(),
        prepaint_ns: stats.prepaint_time.as_nanos(),
        paint_ns: stats.paint_time.as_nanos(),
        compute_layout_ns: stats.compute_layout_time.as_nanos(),
        shape_ns: stats.shape_time.as_nanos(),
        lines_shaped: stats.lines_shaped,
        views_built: stats.views_built,
        views_reused: stats.views_reused,
        nodes_created: stats.nodes_created,
        measure_calls: stats.measure_calls,
    }
}

impl PhaseStats {
    fn since(&self, earlier: &Self) -> Value {
        json!({
            "build_ns": self.build_ns - earlier.build_ns,
            "prepaint_ns": self.prepaint_ns - earlier.prepaint_ns,
            "paint_ns": self.paint_ns - earlier.paint_ns,
            "compute_layout_ns": self.compute_layout_ns - earlier.compute_layout_ns,
            "shape_ns": self.shape_ns - earlier.shape_ns,
            "lines_shaped": self.lines_shaped - earlier.lines_shaped,
            "views_built": self.views_built - earlier.views_built,
            "views_reused": self.views_reused - earlier.views_reused,
            "nodes_created": self.nodes_created - earlier.nodes_created,
            "measure_calls": self.measure_calls - earlier.measure_calls,
        })
    }
}

pub(super) struct Options {
    out: PathBuf,
    mode: Mode,
    mode_name: String,
    entry: String,
    other: Option<String>,
    specs: bool,
    quiet: usize,
    settle: usize,
    frames: usize,
    at: Point<Pixels>,
    tour: Option<Tour>,
}

/// The run the environment asks for, if any.
pub(super) fn options() -> Option<Options> {
    let var = |name: &str| std::env::var(name).ok();
    let out = var("MATERIAL_INSTR_OUTPUT")?;
    let mode_name = var("MATERIAL_INSTR_MODE").unwrap_or_else(|| "scroll".into());
    let mode = match mode_name.as_str() {
        "scroll" => Mode::Scroll,
        "redraw" => Mode::Redraw,
        "idle" => Mode::Idle,
        "switch" => Mode::Switch,
        "tour" => Mode::Tour,
        other => panic!("unknown MATERIAL_INSTR_MODE {other}"),
    };
    let number = |name: &str, default: f32| {
        var(name).map_or(default, |value| {
            value.parse().unwrap_or_else(|_| panic!("{name}: {value}"))
        })
    };
    let tour = (mode == Mode::Tour).then(|| {
        let stops = var("MATERIAL_INSTR_TOUR")
            .expect("MATERIAL_INSTR_TOUR")
            .split(',')
            .map(|stop| {
                let (id, specs) = match stop.split_once(':') {
                    Some((id, "specs")) => (id, true),
                    _ => (stop, false),
                };
                let entry =
                    catalog::position(id).unwrap_or_else(|| panic!("no catalog entry {id}"));
                (stop.to_string(), entry, specs)
            })
            .collect();
        Tour {
            stops,
            rounds: number("MATERIAL_INSTR_TOUR_ROUNDS", 3.) as usize,
            gap: number("MATERIAL_INSTR_TOUR_GAP", 20.) as usize,
            next: 0,
            since: 0,
            visits: Vec::new(),
            frames: Vec::new(),
            before: None,
        }
    });
    Some(Options {
        out: out.into(),
        tour,
        mode,
        mode_name,
        entry: var("MATERIAL_INSTR_ENTRY").unwrap_or_else(|| "button".into()),
        other: var("MATERIAL_INSTR_ENTRY2"),
        specs: var("MATERIAL_INSTR_TAB").as_deref() == Some("specs"),
        quiet: number("MATERIAL_INSTR_QUIET", 45.) as usize,
        settle: number("MATERIAL_INSTR_SETTLE", 120.) as usize,
        frames: number("MATERIAL_INSTR_FRAMES", 300.) as usize,
        at: point(
            px(number("MATERIAL_INSTR_X", 800.)),
            px(number("MATERIAL_INSTR_Y", 500.)),
        ),
    })
}

struct Run {
    options: Options,
    gallery: Entity<Gallery>,
    entries: (usize, Option<usize>),
    on_first: bool,
    seen: usize,
    frames: Vec<[u64; 11]>,
    last: Counters,
    last_tally: [u64; COUNTS],
    last_stats: (u64, u64),
    moved: usize,
    last_offset: f32,
    engine_at_start: Value,
}

fn stats(window: &Window) -> (u64, u64) {
    let stats = window.layout_stats();
    (stats.frames, stats.views_built)
}

fn engine_counters(window: &Window) -> Value {
    let stats = window.layout_stats();
    json!({
        "frames": stats.frames,
        "views_built": stats.views_built,
        "views_reused": stats.views_reused,
        "nodes_created": stats.nodes_created,
        "nodes_reused": stats.nodes_reused,
        "lines_shaped": stats.lines_shaped,
        "layer_frames_composited": stats.layer_frames_composited,
        "layer_frames_repainted": stats.layer_frames_repainted,
        "layer_rebuilds_for_input": stats.layer_rebuilds_for_input,
        "layers_demoted": stats.layers_demoted,
    })
}

fn offset(gallery: &Gallery) -> f32 {
    if gallery.shows_lead_only() {
        f32::from(gallery.lead_scroll.offset().y)
    } else {
        let top = gallery.list.logical_scroll_top();
        top.item_ix as f32 * 100_000. + f32::from(top.offset_in_item)
    }
}

/// One frame of a tour: closes the current stop when due, and starts the next.
fn tour_step(mut run: Run, window: &mut Window, cx: &mut App) {
    let now = Counters::now();
    note_startup_frame(run.seen, window);
    if run.seen == 0 {
        window.reset_layout_stats();
    }
    run.seen += 1;
    let mut tour = run.options.tour.take().expect("tour mode");
    let spent = now.since(run.last);
    let stats = phase_stats(window);
    if run.seen > run.options.quiet {
        if tour.before.is_some() {
            tour.frames.push(json!(spent.row()));
        }
        let due = run.seen >= tour.since;
        if due {
            if let Some(before) = tour.before.take() {
                let at = tour.next - 1;
                let (stop, ..) = &tour.stops[at % tour.stops.len()];
                tour.visits.push(json!({
                    "stop": stop,
                    "visit": at / tour.stops.len(),
                    "frames": std::mem::take(&mut tour.frames),
                    "engine": stats.since(&before),
                    "scroll_top": offset(run.gallery.read(cx)),
                }));
            }
            if tour.next == tour.stops.len() * tour.rounds {
                run.options.tour = Some(tour);
                finish(run, window, cx);
                return;
            }
            let (_, entry, specs) = tour.stops[tour.next % tour.stops.len()];
            tour.next += 1;
            tour.since = run.seen + tour.gap;
            tour.before = Some(stats);
            run.gallery.update(cx, |gallery, cx| {
                gallery.navigate(entry, cx);
                if specs {
                    gallery.set_tab(Tab::Specs, cx);
                }
            });
        }
    }
    run.options.tour = Some(tour);
    // From before the navigation, so that a stop's first frame has it in.
    run.last = now;
    window.on_next_frame(move |window, cx| tour_step(run, window, cx));
}

fn step(mut run: Run, window: &mut Window, cx: &mut App) {
    let now = Counters::now();
    note_startup_frame(run.seen, window);
    let tally = TALLY.with(Cell::get);
    let frame_stats = stats(window);
    let warm = run.options.quiet + run.options.settle;
    if run.seen == warm {
        run.engine_at_start = engine_counters(window);
    }
    if run.seen > warm {
        let delta = |what: Count| tally[what as usize] - run.last_tally[what as usize];
        let spent = now.since(run.last).row();
        run.frames.push([
            spent[0],
            spent[1],
            spent[2],
            spent[3],
            spent[4],
            delta(Count::Gallery),
            delta(Count::Row),
            frame_stats.0.saturating_sub(run.last_stats.0),
            frame_stats.1.saturating_sub(run.last_stats.1),
            delta(Count::Lead),
            delta(Count::Navigation),
        ]);
    }
    run.seen += 1;
    if run.frames.len() >= run.options.frames {
        finish(run, window, cx);
        return;
    }
    let position = offset(run.gallery.read(cx));
    if position != run.last_offset {
        run.moved += 1;
    }
    run.last_offset = position;
    if run.seen > run.options.quiet {
        match run.options.mode {
            Mode::Scroll => {
                let down = (run.seen / 60).is_multiple_of(2);
                window.dispatch_event(
                    PlatformInput::Scroll(ScrollEvent {
                        position: run.options.at,
                        delta: ScrollDelta::Pixels(point(px(0.), px(if down { -8. } else { 8. }))),
                        ..Default::default()
                    }),
                    cx,
                );
            }
            Mode::Redraw => run.gallery.update(cx, |_, cx| cx.notify()),
            Mode::Switch if run.seen.is_multiple_of(12) => {
                if let Some(other) = run.entries.1 {
                    run.on_first = !run.on_first;
                    let entry = if run.on_first { run.entries.0 } else { other };
                    run.gallery
                        .update(cx, |gallery, cx| gallery.navigate(entry, cx));
                }
            }
            Mode::Switch | Mode::Idle | Mode::Tour => {}
        }
    }
    run.last = now;
    run.last_tally = tally;
    run.last_stats = frame_stats;
    window.on_next_frame(move |window, cx| step(run, window, cx));
}

fn write(out: &std::path::Path, report: &Value) {
    if let Some(parent) = out.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(out, serde_json::to_vec(report).unwrap()).expect("write report");
}

fn finish(run: Run, window: &mut Window, cx: &mut App) {
    write(
        &run.options.out,
        &json!({
            "schema":"gallery-instr-v1","content":run.options.entry,"mode":run.options.mode_name,
            "specs":run.options.specs,"per_row":1,
            "viewport":[f32::from(window.viewport_size().width),f32::from(window.viewport_size().height)],
            "scale":window.scale_factor(),
            "scroll_steps_that_moved":run.moved,
            "engine_at_start":run.engine_at_start,
            "engine_at_end":engine_counters(window),
            "tally_final":TALLY.with(Cell::get),
            "startup":startup_report(),
            "tour":run.options.tour.as_ref().map(|tour| &tour.visits),
            "columns":["instructions","cycles","cpu_ns","allocations","allocated_bytes","gallery_renders","row_builds","frames_drawn","views_built","lead_renders","navigation_builds"],
            "frames":run.frames,
        }),
    );
    cx.quit();
}

/// Shows the asked-for page and starts recording.
pub(super) fn install(
    options: Options,
    gallery: Entity<Gallery>,
    window: &mut Window,
    cx: &mut App,
) {
    let position = |id: &str| catalog::position(id).unwrap_or_else(|| panic!("no entry {id}"));
    let first = position(&options.entry);
    let other = options.other.as_deref().map(position);
    let specs = options.specs;
    gallery.update(cx, |gallery, cx| {
        gallery.navigate(first, cx);
        if specs {
            gallery.set_tab(Tab::Specs, cx);
        }
    });
    if options.mode == Mode::Idle {
        let out = options.out.clone();
        let (entry, seconds) = (options.entry.clone(), 3.);
        let handle = window.window_handle();
        cx.spawn(async move |cx| {
            let executor = cx.background_executor().clone();
            executor.timer(Duration::from_secs(4)).await;
            let read = |cx: &mut AsyncApp| {
                handle
                    .update(cx, |_, window, _| {
                        (Counters::now(), stats(window), TALLY.with(Cell::get))
                    })
                    .ok()
            };
            let before = read(cx);
            executor.timer(Duration::from_secs_f64(seconds)).await;
            let after = read(cx);
            if let (Some(before), Some(after)) = (before, after) {
                write(
                    &out,
                    &json!({
                        "schema":"gallery-instr-v1","content":entry,"mode":"idle","seconds":seconds,
                        "columns":["instructions","cycles","cpu_ns","allocations","allocated_bytes"],
                        "total":after.0.since(before.0).row(),
                        "frames_drawn":after.1.0.saturating_sub(before.1.0),
                        "views_built":after.1.1.saturating_sub(before.1.1),
                        "root_renders":after.2[Count::Gallery as usize]-before.2[Count::Gallery as usize],
                    }),
                );
            }
            cx.update(|cx| cx.quit());
        })
        .detach();
        return;
    }
    let frames = options.frames;
    let run = Run {
        options,
        gallery,
        entries: (first, other),
        on_first: true,
        seen: 0,
        frames: Vec::with_capacity(frames),
        last: Counters::now(),
        last_tally: [0; COUNTS],
        last_stats: (0, 0),
        moved: 0,
        last_offset: 0.,
        engine_at_start: Value::Null,
    };
    if run.options.tour.is_some() {
        window.on_next_frame(move |window, cx| tour_step(run, window, cx));
    } else {
        window.on_next_frame(move |window, cx| step(run, window, cx));
    }
}
