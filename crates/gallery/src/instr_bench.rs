//! Retired instructions per Gallery frame, with what each frame rebuilt.
//! Compiled only with the `instruction-bench` feature; independent of the frame
//! profiler. Frames are bracketed with `on_next_frame`, as `component_bench` does.
//!
//! ```sh
//! MATERIAL_INSTR_OUTPUT=/tmp/g.json MATERIAL_INSTR_ENTRY=button MATERIAL_INSTR_TAB=specs \
//! MATERIAL_INSTR_MODE=scroll cargo run --release -p gpui-kit-material --example gallery \
//!     --features instruction-bench
//! ```
//!
//! Modes: `scroll` (one 8 px wheel per frame, reversing every 60 frames), `redraw`
//! (notifies the Gallery each frame), `idle` (nothing), and `switch` (navigates
//! between `MATERIAL_INSTR_ENTRY` and `MATERIAL_INSTR_ENTRY2` every 12 frames).
use super::{Gallery, Tab, catalog};
use gpui::*;
use serde_json::{Value, json};
use std::{cell::Cell, path::PathBuf, time::Duration};

#[path = "../support/perf_counters.rs"]
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
        other => panic!("unknown MATERIAL_INSTR_MODE {other}"),
    };
    let number = |name: &str, default: f32| {
        var(name).map_or(default, |value| {
            value.parse().unwrap_or_else(|_| panic!("{name}: {value}"))
        })
    };
    Some(Options {
        out: out.into(),
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

fn step(mut run: Run, window: &mut Window, cx: &mut App) {
    let now = Counters::now();
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
            Mode::Switch | Mode::Idle => {}
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
    window.on_next_frame(move |window, cx| step(run, window, cx));
}
