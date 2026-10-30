//! Standalone engine benchmark: a long list of Material components, scrolled,
//! idled or force-redrawn under the frame profiler, independent of the Gallery.
//!
//! ```sh
//! cargo run --release -p gpui-kit-material --example engine_bench \
//!     --features frame-profiler -- --scenario scroll --vsync off --seconds 5 \
//!     --out /tmp/bench/scroll-off.json
//! ```
//!
//! Scenarios:
//! - `scroll`: every frame dispatches one 8 px wheel event over the list,
//!   reversing at either end, and asks for the next frame.
//! - `idle`: nothing happens after startup; counts frames drawn or presented.
//! - `redraw`: every frame notifies the root view without changing anything.
//! - `startup`: records only the window's first frames, then quits.
//!
//! The JSON holds the raw per-frame CPU scopes, draw phases, submissions and
//! Metal GPU executions; `scripts/profile-gallery.py summarize` turns it into
//! the AGENTS.md target verdicts. This program never decides acceptance.
extern crate gpui_kit as gpui;

#[cfg(feature = "frame-profiler")]
mod bench {
    use gpui::{prelude::*, *};
    use gpui_kit_material::icons::Assets;
    use gpui_kit_material::prelude::*;
    use gpui_kit_material::theme;
    use serde_json::{Value, json};
    use std::{
        cell::Cell,
        path::PathBuf,
        rc::Rc,
        time::{Duration, Instant},
    };

    const ROWS: usize = 4_000;
    const STEP_PX: f32 = 8.;
    /// Startup frames are excluded from steady-state statistics but kept raw.
    const SETTLE: Duration = Duration::from_millis(750);
    /// When a frame finished, with the main thread's CPU time if available.
    type FrameMark = (Instant, Option<u64>);

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Scenario {
        Scroll,
        Idle,
        Redraw,
        Startup,
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Content {
        /// Buttons, switch, checkbox, chip and text per row.
        Material,
        /// Plain quads and text only: isolates the engine from components.
        Simple,
        /// Gallery-like sections: a title and a tall board of component
        /// cells that scrolls horizontally inside its row.
        Boards,
    }

    struct Options {
        scenario: Scenario,
        content: Content,
        vsync: bool,
        seconds: f64,
        out: PathBuf,
        width: f32,
        height: f32,
        /// Profiling aid: open a small plain window first, then the measured
        /// one after this delay, so a sampler can attach in between.
        open_delay_ms: u64,
    }

    fn parse() -> Options {
        let mut options = Options {
            scenario: Scenario::Scroll,
            content: Content::Material,
            vsync: false,
            seconds: 5.,
            out: PathBuf::from("engine-bench.json"),
            width: 1280.,
            height: 800.,
            open_delay_ms: 0,
        };
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            let mut value = || args.next().unwrap_or_else(|| usage(&arg));
            match arg.as_str() {
                "--scenario" => {
                    options.scenario = match value().as_str() {
                        "scroll" => Scenario::Scroll,
                        "idle" => Scenario::Idle,
                        "redraw" => Scenario::Redraw,
                        "startup" => Scenario::Startup,
                        _ => usage("--scenario"),
                    }
                }
                "--content" => {
                    options.content = match value().as_str() {
                        "material" => Content::Material,
                        "simple" => Content::Simple,
                        "boards" => Content::Boards,
                        _ => usage("--content"),
                    }
                }
                "--vsync" => {
                    options.vsync = match value().as_str() {
                        "on" => true,
                        "off" => false,
                        _ => usage("--vsync"),
                    }
                }
                "--seconds" => {
                    options.seconds = value().parse().unwrap_or_else(|_| usage("--seconds"));
                    if !(0.5..=120.).contains(&options.seconds) {
                        usage("--seconds");
                    }
                }
                "--out" => options.out = value().into(),
                "--width" => options.width = value().parse().unwrap_or_else(|_| usage("--width")),
                "--open-delay-ms" => {
                    options.open_delay_ms =
                        value().parse().unwrap_or_else(|_| usage("--open-delay-ms"))
                }
                "--height" => {
                    options.height = value().parse().unwrap_or_else(|_| usage("--height"))
                }
                _ => usage(&arg),
            }
        }
        options
    }

    fn usage(arg: &str) -> ! {
        eprintln!(
            "invalid argument {arg}\nusage: engine_bench [--scenario scroll|idle|redraw|startup] \
             [--content material|simple|boards] [--vsync on|off] [--seconds 0.5..=120] [--out file.json] [--width px] [--height px] [--open-delay-ms ms]"
        );
        std::process::exit(2)
    }

    struct Bench {
        list: ListState,
        content: Content,
        down: bool,
        redraws: Rc<Cell<u64>>,
    }

    impl Bench {
        fn simple_row(index: usize) -> AnyElement {
            div()
                .flex()
                .flex_row()
                .id(("row", index))
                .w_full()
                .h(rem(64.))
                .px(rem(16.))
                .gap(rem(12.))
                .items_center()
                .border_b_1()
                .border_color(color(ColorRole::OUTLINE_VARIANT))
                .child(SharedString::from(format!("{index}")))
                .child(
                    div()
                        .size(rem(40.))
                        .rounded(rem(20.))
                        .bg(color(ColorRole::PRIMARY_CONTAINER)),
                )
                .child(
                    div()
                        .px(rem(24.))
                        .py(rem(10.))
                        .rounded(rem(20.))
                        .bg(color(ColorRole::PRIMARY))
                        .text_color(color(ColorRole::ON_PRIMARY))
                        .child("Button"),
                )
                .child(
                    div()
                        .flex_1()
                        .child("Supporting text that wraps nothing and stays on one line"),
                )
                .into_any_element()
        }

        fn board_row(index: usize) -> AnyElement {
            let rows = 3 + index % 4;
            let cell = |row: usize, column: usize| {
                let id = index * 1000 + row * 100 + column;
                let element: AnyElement = match column % 4 {
                    0 => MaterialButton::new(("board-button", id))
                        .variant(ButtonVariant::Tonal)
                        .child("Tonal")
                        .into_any_element(),
                    1 => MaterialSwitch::new(("board-switch", id), "Switch")
                        .checked(row.is_multiple_of(2))
                        .into_any_element(),
                    2 => MaterialCheckbox::new(("board-check", id), "Check")
                        .checked(row % 2 == 1)
                        .into_any_element(),
                    _ => FilterChip::new(("board-chip", id), "Chip")
                        .selected(row.is_multiple_of(2))
                        .into_any_element(),
                };
                div()
                    .w(rem(140.))
                    .h(rem(72.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(element)
            };
            div()
                .id(("section", index))
                .w_full()
                .flex()
                .flex_col()
                .gap(rem(12.))
                .p(rem(24.))
                .child(SharedString::from(format!("Section {index}")))
                .child(
                    div()
                        .id(("board", index))
                        .w_full()
                        .overflow_x_scroll()
                        .rounded(rem(16.))
                        .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
                        .child(div().flex().flex_col().children((0..rows).map(|row| {
                            div()
                                .flex()
                                .flex_row()
                                .children((0..10).map(move |c| cell(row, c)))
                        }))),
                )
                .into_any_element()
        }

        fn row(index: usize) -> AnyElement {
            let even = index.is_multiple_of(2);
            div()
                .flex()
                .flex_row()
                .id(("row", index))
                .w_full()
                .h(rem(64.))
                .px(rem(16.))
                .gap(rem(12.))
                .items_center()
                .border_b_1()
                .border_color(color(ColorRole::OUTLINE_VARIANT))
                .child(
                    div()
                        .w(rem(64.))
                        .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                        .child(SharedString::from(format!("{index}"))),
                )
                .child(
                    MaterialButton::new(("filled", index))
                        .variant(ButtonVariant::Filled)
                        .child("Filled"),
                )
                .child(
                    MaterialButton::new(("outlined", index))
                        .variant(ButtonVariant::Outlined)
                        .child("Outlined"),
                )
                .child(MaterialSwitch::new(("switch", index), "Switch").checked(even))
                .child(MaterialCheckbox::new(("check", index), "Check").checked(!even))
                .child(FilterChip::new(("chip", index), "Filter").selected(even))
                .child(
                    div()
                        .flex_1()
                        .text_color(color(ColorRole::ON_SURFACE))
                        .child("Supporting text that wraps nothing and stays on one line"),
                )
                .into_any_element()
        }
    }

    impl Render for Bench {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            self.redraws.set(self.redraws.get() + 1);
            let content = self.content;
            div()
                .size_full()
                .bg(color(ColorRole::SURFACE))
                .text_color(color(ColorRole::ON_SURFACE))
                .child(
                    list(self.list.clone(), move |index, _, _| match content {
                        Content::Material => Self::row(index),
                        Content::Simple => Self::simple_row(index),
                        Content::Boards => Self::board_row(index),
                    })
                    .size_full()
                    .with_sizing_behavior(ListSizingBehavior::Auto),
                )
        }
    }

    /// Main-thread CPU time; the first frame target counts CPU, not the
    /// wall time the window server takes to show a window.
    fn thread_cpu_ns() -> Option<u64> {
        #[cfg(target_os = "macos")]
        {
            unsafe extern "C" {
                fn clock_gettime_nsec_np(clock_id: u32) -> u64;
            }
            const CLOCK_THREAD_CPUTIME_ID: u32 = 16;
            // SAFETY: a pure query of the calling thread's CPU clock.
            Some(unsafe { clock_gettime_nsec_np(CLOCK_THREAD_CPUTIME_ID) })
        }
        #[cfg(not(target_os = "macos"))]
        None
    }

    fn engine_counters(window: &Window) -> Value {
        #[cfg(feature = "gpui-fast")]
        {
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
                "tiles_dirtied": stats.tiles_dirtied,
                "layer_rebuilds_for_input": stats.layer_rebuilds_for_input,
                "layers_demoted": stats.layers_demoted,
            })
        }
        #[cfg(not(feature = "gpui-fast"))]
        {
            let _ = window;
            Value::Null
        }
    }

    fn quantile(sorted: &[u128], q: f64) -> Option<u128> {
        if sorted.is_empty() {
            return None;
        }
        let rank = ((q * sorted.len() as f64).ceil() as usize).clamp(1, sorted.len());
        Some(sorted[rank - 1])
    }

    fn summary(values: impl Iterator<Item = u128>) -> Value {
        let mut values: Vec<u128> = values.collect();
        values.sort_unstable();
        json!({
            "count": values.len(),
            "p50_ns": quantile(&values, 0.5),
            "p95_ns": quantile(&values, 0.95),
            "p99_ns": quantile(&values, 0.99),
            "max_ns": values.last(),
        })
    }

    fn command(program: &str, args: &[&str]) -> Option<String> {
        let output = std::process::Command::new(program)
            .args(args)
            .output()
            .ok()?;
        output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }

    pub fn main() {
        let options = parse();
        let metadata = json!({
            "uname": command("uname", &["-a"]),
            "model": command("sysctl", &["-n", "hw.model"]),
            "cpu": command("sysctl", &["-n", "machdep.cpu.brand_string"]),
            "load_average_at_start": command("sysctl", &["-n", "vm.loadavg"]),
            "commit": command("git", &["-C", env!("CARGO_MANIFEST_DIR"), "rev-parse", "HEAD"]),
            "dirty": command("git", &["-C", env!("CARGO_MANIFEST_DIR"), "status", "--porcelain"])
                .map(|status| !status.is_empty()),
            "release": !cfg!(debug_assertions),
            "backend": if gpui_kit::GPUI_FAST { "gpui-fast" } else { "gpui-pre" },
        });
        let process_start = Instant::now();
        gpui::profiler::set_trace_enabled(true);
        let mut collector = gpui::profiler::FrameTimingCollector::new();
        gpui_kit::application()
            .with_assets(Assets::new())
            .run(move |cx: &mut App| {
                cx.text_system()
                    .add_fonts(vec![
                        std::borrow::Cow::Borrowed(include_bytes!(
                            "gallery/fonts/roboto/Roboto-Regular.ttf"
                        )),
                        std::borrow::Cow::Borrowed(include_bytes!(
                            "gallery/fonts/roboto/Roboto-Medium.ttf"
                        )),
                    ])
                    .expect("Failed to load the Roboto fonts");
                gpui_kit::init(cx);
                gpui_kit_material::init(cx);
                theme::init_component_theme(theme::scheme(), 1., cx);
                let open_delay = Duration::from_millis(options.open_delay_ms);
                let open = move |cx: &mut App| {
                let redraws = Rc::new(Cell::new(0));
                let cpu_at_open = thread_cpu_ns();
                let open_started = Instant::now();
                let options_size = size(px(options.width), px(options.height));
                let (window, view) = gpui_kit::open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                            None,
                            options_size,
                            cx,
                        ))),
                        ..Default::default()
                    },
                    cx,
                    {
                        let redraws = redraws.clone();
                        move |_, cx| {
                            cx.new(|_| Bench {
                                list: ListState::new(ROWS, ListAlignment::Top, px(256.)),
                                content: options.content,
                                down: true,
                                redraws,
                            })
                        }
                    },
                )
                .expect("open window");
                let open_returned = Instant::now();
                let cpu_at_open_returned = thread_cpu_ns();
                // The first platform frame presents the window's first frame; a
                // callback scheduled from inside it runs after that submission.
                let after_first_frame: Rc<Cell<Option<FrameMark>>> = Rc::new(Cell::new(None));
                let settle_counters: Rc<std::cell::RefCell<Value>> =
                    Rc::new(std::cell::RefCell::new(Value::Null));
                window
                    .update(cx, {
                        let after_first_frame = after_first_frame.clone();
                        move |_, window, _| {
                            window.on_next_frame(move |window, _| {
                                window.on_next_frame(move |_, _| {
                                    after_first_frame.set(Some((Instant::now(), thread_cpu_ns())));
                                });
                            });
                        }
                    })
                    .ok();
                cx.activate(true);
                let scenario = options.scenario;
                window
                    .update(cx, |_, window, _| {
                        window.activate_window();
                        if let Err(error) = window.set_vsync(options.vsync) {
                            eprintln!("set_vsync({}) failed: {error:#}", options.vsync);
                        }
                    })
                    .ok();
                let duration = Duration::from_secs_f64(if scenario == Scenario::Startup {
                    0.5
                } else {
                    options.seconds
                });
                let settle_until = open_returned + SETTLE;
                let view_for_driver = view.clone();
                // Frame-driven driver: one wheel step or root notify per frame.
                fn drive(
                    window: &mut Window,
                    cx: &mut App,
                    view: Entity<Bench>,
                    scenario: Scenario,
                    settle_until: Instant,
                    end: Instant,
                ) {
                    let now = Instant::now();
                    if now >= end {
                        return;
                    }
                    if now >= settle_until {
                        match scenario {
                            Scenario::Scroll => {
                                let (list, down) = {
                                    let bench = view.read(cx);
                                    (bench.list.clone(), bench.down)
                                };
                                let at_end = list.is_scrolled_to_end() == Some(true);
                                let at_top = list.logical_scroll_top().item_ix == 0
                                    && list.logical_scroll_top().offset_in_item <= px(0.);
                                let down = if down && at_end {
                                    false
                                } else if !down && at_top {
                                    true
                                } else {
                                    down
                                };
                                view.update(cx, |bench, _| bench.down = down);
                                let bounds = list.viewport_bounds();
                                window.dispatch_event(
                                    PlatformInput::Scroll(ScrollEvent {
                                        position: bounds.center(),
                                        delta: ScrollDelta::Pixels(point(
                                            px(0.),
                                            px(if down { -STEP_PX } else { STEP_PX }),
                                        )),
                                        ..Default::default()
                                    }),
                                    cx,
                                );
                            }
                            Scenario::Redraw => view.update(cx, |_, cx| cx.notify()),
                            Scenario::Idle | Scenario::Startup => return,
                        }
                    }
                    window.on_next_frame(move |window, cx| {
                        drive(window, cx, view, scenario, settle_until, end)
                    });
                }
                let end = open_returned + duration;
                window
                    .update(cx, move |_, window, cx| {
                        if matches!(scenario, Scenario::Scroll | Scenario::Redraw) {
                            drive(window, cx, view_for_driver, scenario, settle_until, end);
                        }
                    })
                    .ok();
                {
                    let settle_counters = settle_counters.clone();
                    cx.spawn(async move |cx| {
                        cx.background_executor()
                            .timer(settle_until.saturating_duration_since(Instant::now()))
                            .await;
                        let _ = window.update(cx, |_, window, _| {
                            *settle_counters.borrow_mut() = engine_counters(window);
                        });
                    })
                    .detach();
                }
                let window_id = window.window_id();
                let view_for_report = view.clone();
                cx.spawn(async move |cx| {
                    let mut events = Vec::new();
                    loop {
                        cx.background_executor()
                            .timer(Duration::from_millis(100))
                            .await;
                        events.extend(collector.collect_unseen());
                        if Instant::now() >= end + Duration::from_millis(100) {
                            break;
                        }
                    }
                    let (gpu, gpu_dropped) = gpui::profiler::take_gpu_executions();
                    gpui::profiler::set_trace_enabled(false);
                    let (refresh, viewport, scale, vsync_status, final_counters) = window
                        .update(cx, |_, window, cx| {
                            (
                                window.refresh_interval(cx).map(|d| d.as_nanos()),
                                window.viewport_size(),
                                window.scale_factor(),
                                format!("{:?}", window.vsync_status()),
                                engine_counters(window),
                            )
                        })
                        .unwrap_or((None, Size::default(), 0., "unknown".into(), Value::Null));
                    let redraws = cx
                        .update(|cx| view_for_report.read(cx).redraws.get());
                    let report = report(ReportInput {
                        events,
                        gpu,
                        gpu_dropped,
                        window_id,
                        process_start,
                        open_started,
                        open_returned,
                        settle_until,
                        end,
                        scenario,
                        vsync: options.vsync,
                        metadata,
                        refresh,
                        viewport,
                        scale,
                        vsync_status,
                        redraws,
                        counters: json!({
                            "at_settle": settle_counters.borrow().clone(),
                            "final": final_counters,
                        }),
                        first_frame_cpu: json!({
                            "definition": "main-thread CPU time from the open_window call to a callback scheduled inside the first platform frame (after its submission); idle waits do not count",
                            "open_window_cpu_ns": cpu_at_open_returned.zip(cpu_at_open).map(|(b, a)| b - a),
                            "cpu_to_after_first_frame_ns": after_first_frame.get().and_then(|(_, cpu)| cpu.zip(cpu_at_open).map(|(b, a)| b - a)),
                            "wall_to_after_first_frame_ns": after_first_frame.get().map(|(at, _)| at.duration_since(open_started).as_nanos()),
                        }),
                    });
                    if let Some(parent) = options.out.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    std::fs::write(&options.out, serde_json::to_vec_pretty(&report).unwrap())
                        .expect("write report");
                    eprintln!(
                        "engine_bench: {}",
                        serde_json::to_string(&report["summary"]).unwrap()
                    );
                    cx.update(|cx| cx.quit());
                })
                .detach();
                };
                if open_delay.is_zero() {
                    open(cx);
                } else {
                    // A plain window first initializes AppKit's lazily loaded
                    // frameworks, which a sampler attached later must not see.
                    let _ = cx.open_window(
                        WindowOptions {
                            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                                None,
                                size(px(200.), px(120.)),
                                cx,
                            ))),
                            ..Default::default()
                        },
                        |_, cx| cx.new(|_| Warmup),
                    );
                    cx.spawn(async move |cx| {
                        cx.background_executor().timer(open_delay).await;
                        cx.update(open);
                    })
                    .detach();
                }
            });
    }

    struct Warmup;

    impl Render for Warmup {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child("warm-up")
        }
    }

    struct ReportInput {
        events: Vec<gpui::profiler::FrameEvent>,
        gpu: Vec<gpui::profiler::GpuExecution>,
        gpu_dropped: u64,
        window_id: WindowId,
        process_start: Instant,
        open_started: Instant,
        open_returned: Instant,
        settle_until: Instant,
        end: Instant,
        scenario: Scenario,
        vsync: bool,
        metadata: Value,
        refresh: Option<u128>,
        viewport: Size<Pixels>,
        scale: f32,
        vsync_status: String,
        redraws: u64,
        counters: Value,
        first_frame_cpu: Value,
    }

    fn report(input: ReportInput) -> Value {
        use gpui::profiler::{CpuFrameSource, FrameEvent};
        let origin = input.open_started;
        let ns = |at: Instant| -> i128 {
            if at >= origin {
                at.duration_since(origin).as_nanos() as i128
            } else {
                -(origin.duration_since(at).as_nanos() as i128)
            }
        };
        let mut scopes = Vec::new();
        let mut draws = Vec::new();
        let mut presents = Vec::new();
        for event in &input.events {
            match event {
                FrameEvent::Complete(frame) if frame.window_id == input.window_id => {
                    scopes.push(frame)
                }
                FrameEvent::Draw(draw) if draw.window_id == input.window_id => draws.push(draw),
                FrameEvent::Present(present) if present.window_id == input.window_id => {
                    presents.push(present)
                }
                _ => {}
            }
        }
        // GPU executions belong to the CPU scope during which they were committed.
        let mut gpu_by_scope = vec![0u128; scopes.len()];
        let mut gpu_unmatched = 0usize;
        for execution in &input.gpu {
            let gpu_ns = ((execution.gpu_end_s - execution.gpu_start_s).max(0.) * 1e9) as u128;
            match scopes.iter().position(|scope| {
                scope.start <= execution.submitted_at && execution.submitted_at <= scope.end
            }) {
                Some(index) => gpu_by_scope[index] += gpu_ns,
                None => gpu_unmatched += 1,
            }
        }
        let raw_scopes: Vec<Value> = scopes
            .iter()
            .enumerate()
            .map(|(index, scope)| {
                let phases = draws
                    .iter()
                    .find(|draw| scope.start <= draw.draw_start && draw.draw_end <= scope.end);
                let present = presents.iter().find(|present| {
                    scope.start <= present.present_start && present.present_end <= scope.end
                });
                let total = scope.duration().as_nanos();
                let wait = scope.presentation_wait.as_nanos();
                json!({
                    "source": format!("{:?}", scope.source),
                    "start_ns": ns(scope.start),
                    "end_ns": ns(scope.end),
                    "cpu_ns": total,
                    "drawable_wait_ns": wait,
                    "cpu_excluding_drawable_wait_ns": total.saturating_sub(wait),
                    "draw_count": scope.draw_count,
                    "present_count": scope.present_count,
                    "next_frame_callbacks": scope.next_frame_callbacks,
                    "steady": scope.start >= input.settle_until && scope.end <= input.end,
                    "request_layout_ns": phases.map(|d| d.phases.request_layout.as_nanos()),
                    "prepaint_ns": phases.map(|d| d.phases.prepaint.as_nanos()),
                    "paint_ns": phases.map(|d| d.phases.paint.as_nanos()),
                    "draw_ns": phases.map(|d| d.draw_duration().as_nanos()),
                    "present_ns": present.map(|p| p.present_duration().as_nanos()),
                    "successful_new_frame": present.and_then(|p| p.successful_new_frame),
                    "gpu_ns": (gpu_by_scope[index] > 0).then_some(gpu_by_scope[index]),
                })
            })
            .collect();
        let steady: Vec<&Value> = raw_scopes
            .iter()
            .filter(|s| s["steady"] == true && s["draw_count"].as_u64() > Some(0))
            .collect();
        let field = |name: &'static str| {
            let steady = &steady;
            move || {
                steady
                    .iter()
                    .filter_map(move |s| s[name].as_u64().map(u128::from))
            }
        };
        // New frames per 100 ms window inside the steady span.
        let new_frames: Vec<i128> = presents
            .iter()
            .filter(|p| p.successful_new_frame == Some(true))
            .map(|p| ns(p.present_end))
            .collect();
        let steady_start = ns(input.settle_until);
        let steady_end = ns(input.end);
        let window_ns = 100_000_000i128;
        let mut windows = Vec::new();
        let mut at = steady_start;
        while at + window_ns <= steady_end {
            let count = new_frames
                .iter()
                .filter(|&&t| t >= at && t < at + window_ns)
                .count();
            windows.push(count as f64 * 1e9 / window_ns as f64);
            at += window_ns;
        }
        let steady_new_frames = new_frames
            .iter()
            .filter(|&&t| t >= steady_start && t < steady_end)
            .count();
        let steady_seconds = (steady_end - steady_start) as f64 / 1e9;
        // First frame: open_window call to the end of the first successful submission.
        let first_present = presents
            .iter()
            .find(|p| p.successful_new_frame == Some(true))
            .map(|p| ns(p.present_end));
        let first_scopes_cpu: u128 = scopes
            .iter()
            .filter(|s| first_present.is_some_and(|first| ns(s.end) <= first))
            .map(|s| s.duration().as_nanos())
            .sum();
        let initial = scopes
            .iter()
            .find(|s| s.source == CpuFrameSource::InitialDraw)
            .map(|s| s.duration().as_nanos());
        let all_gpu: Vec<u128> = gpu_by_scope.iter().copied().filter(|&g| g > 0).collect();
        let idle_draws = raw_scopes
            .iter()
            .filter(|s| s["steady"] == true && s["draw_count"].as_u64() > Some(0))
            .count();
        let idle_presents = raw_scopes
            .iter()
            .filter(|s| s["steady"] == true && s["present_count"].as_u64() > Some(0))
            .count();
        json!({
            "schema": "material-engine-bench-v1",
            "scenario": format!("{:?}", input.scenario).to_lowercase(),
            "requested_vsync": input.vsync,
            "actual_vsync": input.vsync_status,
            "rows": ROWS,
            "step_px": STEP_PX,
            "viewport_logical_px": [f32::from(input.viewport.width), f32::from(input.viewport.height)],
            "scale_factor": input.scale,
            "display_refresh_interval_ns": input.refresh,
            "metadata": input.metadata,
            "time_origin": "open_window call",
            "process_start_ns": ns(input.process_start),
            "open_returned_ns": ns(input.open_returned),
            "steady_window_ns": [steady_start, steady_end],
            "root_renders": input.redraws,
            "engine_counters": input.counters,
            "first_frame_cpu": input.first_frame_cpu,
            "gpu_dropped": input.gpu_dropped,
            "gpu_unmatched_executions": gpu_unmatched,
            "summary": {
                "steady_frames": steady.len(),
                "cpu_excluding_drawable_wait": summary(field("cpu_excluding_drawable_wait_ns")()),
                "drawable_wait": summary(field("drawable_wait_ns")()),
                "cpu_including_wait": summary(field("cpu_ns")()),
                "layout": summary(field("request_layout_ns")()),
                "prepaint": summary(field("prepaint_ns")()),
                "paint": summary(field("paint_ns")()),
                "present": summary(field("present_ns")()),
                "gpu": summary(field("gpu_ns")()),
                "gpu_all_frames": summary(all_gpu.into_iter()),
                "new_frames_per_s": steady_new_frames as f64 / steady_seconds.max(1e-9),
                "min_100ms_window_new_frames_per_s": windows.iter().copied().fold(None, |m: Option<f64>, v| Some(m.map_or(v, |m| m.min(v)))),
                "initial_draw_ns": initial,
                "first_frame_main_thread_cpu_ns": input.first_frame_cpu["cpu_to_after_first_frame_ns"].clone(),
                "first_present_end_ns_from_open": first_present,
                "cpu_before_first_present_ns": first_scopes_cpu,
                "steady_frames_drawn": idle_draws,
                "steady_frames_presented": idle_presents,
            },
            "fps_windows_100ms": windows,
            "raw_scopes": raw_scopes,
            "raw_successful_new_frame_end_ns": new_frames,
            "raw_gpu": input.gpu.iter().map(|g| json!({
                "submitted_ns": ns(g.submitted_at),
                "gpu_ns": ((g.gpu_end_s - g.gpu_start_s).max(0.) * 1e9) as u128,
                "kind": g.kind,
                "completed": g.completed,
            })).collect::<Vec<_>>(),
        })
    }
}

#[cfg(feature = "frame-profiler")]
fn main() {
    bench::main();
}

#[cfg(not(feature = "frame-profiler"))]
fn main() {
    eprintln!("engine_bench requires --features frame-profiler");
    std::process::exit(2);
}
