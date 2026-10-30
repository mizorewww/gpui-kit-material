//! Like-for-like component costs in retired instructions per frame: a Kit and
//! a Material list with equivalent rows, and virtual lists whose rows hold six
//! instances of one component. One binary, runtime flags, one number per frame.
//!
//! ```sh
//! cargo run --release -p gpui-kit-material --example component_bench -- \
//!     --content material-list --mode scroll --frames 300 --out /tmp/m.json
//! ```
//!
//! Modes:
//! - `scroll`: every frame dispatches one 8 px wheel event over the list.
//! - `redraw`: every frame notifies the root view, nothing else changes.
//! - `idle`: nothing happens; counts main-thread work and frames drawn.
//!
//! A frame is everything the main thread did between two `on_next_frame`
//! callbacks: wheel dispatch, draw, submission and whatever ran between them.
//! Frames are bracketed here, so the program needs no engine patch.
extern crate gpui_kit as gpui;

#[cfg(not(target_family = "wasm"))]
#[path = "support/perf_counters.rs"]
mod perf_counters;

#[cfg(not(target_family = "wasm"))]
#[global_allocator]
static ALLOCATOR: perf_counters::CountingAllocator = perf_counters::CountingAllocator;

#[cfg(not(target_family = "wasm"))]
mod bench {
    use super::perf_counters::Counters;
    use gpui::{prelude::*, *};
    use gpui_kit::component::{
        Sizable as _,
        button::{Button, ButtonVariants as _},
        checkbox::Checkbox,
        input::{Input, InputState},
        list::ListItem as KitListItem,
        radio::Radio,
        switch::Switch,
        tab::{Tab, TabBar},
        tag::Tag,
    };
    use gpui_kit_material::icons::Assets;
    use gpui_kit_material::prelude::*;
    use gpui_kit_material::theme;
    use serde_json::{Value, json};
    use std::{cell::Cell, cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

    const STEP_PX: f32 = 8.;
    const PER_ROW: usize = 6;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Mode {
        Scroll,
        Redraw,
        Idle,
    }

    /// What a list row holds.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Content {
        /// The Material row of `engine_bench`.
        MaterialList,
        /// The same row, built from Kit's themed components.
        KitList,
        /// Plain quads and text: the engine without components.
        Simple,
        /// A row of six instances of one component.
        Cells(Component),
    }

    /// One component, or the baseline of its row.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Component {
        Empty,
        ButtonM,
        ButtonK,
        IconButtonM,
        SwitchM,
        SwitchK,
        CheckboxM,
        CheckboxK,
        RadioM,
        RadioK,
        ChipM,
        TagK,
        ListItemM,
        ListItemK,
        TextFieldM,
        InputK,
        TabsM,
        TabsK,
        /// A surface with an idle Material ripple over it.
        RippleIdle,
        /// The same surface without one.
        RippleNone,
        TextPlain,
        TextTracked,
    }

    const CELLS: &[(&str, Component)] = &[
        ("empty", Component::Empty),
        ("button-m", Component::ButtonM),
        ("button-k", Component::ButtonK),
        ("icon-button-m", Component::IconButtonM),
        ("switch-m", Component::SwitchM),
        ("switch-k", Component::SwitchK),
        ("checkbox-m", Component::CheckboxM),
        ("checkbox-k", Component::CheckboxK),
        ("radio-m", Component::RadioM),
        ("radio-k", Component::RadioK),
        ("chip-m", Component::ChipM),
        ("tag-k", Component::TagK),
        ("list-item-m", Component::ListItemM),
        ("list-item-k", Component::ListItemK),
        ("text-field-m", Component::TextFieldM),
        ("input-k", Component::InputK),
        ("tabs-m", Component::TabsM),
        ("tabs-k", Component::TabsK),
        ("ripple-idle", Component::RippleIdle),
        ("ripple-none", Component::RippleNone),
        ("text-plain", Component::TextPlain),
        ("text-tracked", Component::TextTracked),
    ];

    struct Options {
        mode: Mode,
        content: Content,
        content_name: String,
        frames: usize,
        /// Frames with no input first, as a window's first frames settle.
        quiet_frames: usize,
        settle_frames: usize,
        rows: usize,
        idle_seconds: f64,
        width: f32,
        height: f32,
        out: PathBuf,
    }

    fn usage(arg: &str) -> ! {
        let cells: Vec<&str> = CELLS.iter().map(|(name, _)| *name).collect();
        eprintln!(
            "invalid argument {arg}\nusage: component_bench [--mode scroll|redraw|idle] \
             [--content material-list|kit-list|simple|{}] [--frames n] [--quiet-frames n] [--settle-frames n] \
             [--rows n] [--idle-seconds s] [--width px] [--height px] [--out file.json]",
            cells.join("|")
        );
        std::process::exit(2)
    }

    fn parse() -> Options {
        let mut options = Options {
            mode: Mode::Scroll,
            content: Content::MaterialList,
            content_name: "material-list".into(),
            frames: 300,
            quiet_frames: 45,
            settle_frames: 90,
            rows: 4_000,
            idle_seconds: 3.,
            width: 1280.,
            height: 800.,
            out: "component-bench.json".into(),
        };
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            let mut value = || args.next().unwrap_or_else(|| usage(&arg));
            match arg.as_str() {
                "--mode" => {
                    options.mode = match value().as_str() {
                        "scroll" => Mode::Scroll,
                        "redraw" => Mode::Redraw,
                        "idle" => Mode::Idle,
                        _ => usage("--mode"),
                    }
                }
                "--content" => {
                    let name = value();
                    options.content = match name.as_str() {
                        "material-list" => Content::MaterialList,
                        "kit-list" => Content::KitList,
                        "simple" => Content::Simple,
                        other => Content::Cells(
                            CELLS
                                .iter()
                                .find(|(cell, _)| *cell == other)
                                .map(|(_, cell)| *cell)
                                .unwrap_or_else(|| usage("--content")),
                        ),
                    };
                    options.content_name = name;
                }
                "--frames" => {
                    options.frames = value().parse().unwrap_or_else(|_| usage("--frames"))
                }
                "--quiet-frames" => {
                    options.quiet_frames =
                        value().parse().unwrap_or_else(|_| usage("--quiet-frames"))
                }
                "--settle-frames" => {
                    options.settle_frames =
                        value().parse().unwrap_or_else(|_| usage("--settle-frames"))
                }
                "--rows" => options.rows = value().parse().unwrap_or_else(|_| usage("--rows")),
                "--idle-seconds" => {
                    options.idle_seconds =
                        value().parse().unwrap_or_else(|_| usage("--idle-seconds"))
                }
                "--width" => options.width = value().parse().unwrap_or_else(|_| usage("--width")),
                "--height" => {
                    options.height = value().parse().unwrap_or_else(|_| usage("--height"))
                }
                "--out" => options.out = value().into(),
                _ => usage(&arg),
            }
        }
        options
    }

    type Inputs = Rc<RefCell<HashMap<usize, Entity<InputState>>>>;

    struct Bench {
        list: ListState,
        content: Content,
        renders: Rc<Cell<u64>>,
        row_builds: Rc<Cell<u64>>,
        inputs: Inputs,
    }

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

    /// The row frame both component lists share.
    fn list_row(index: usize) -> Stateful<Div> {
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
    }

    fn supporting() -> Div {
        div()
            .flex_1()
            .text_color(color(ColorRole::ON_SURFACE))
            .child("Supporting text that wraps nothing and stays on one line")
    }

    fn material_row(index: usize) -> AnyElement {
        let even = index.is_multiple_of(2);
        list_row(index)
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
            .child(supporting())
            .into_any_element()
    }

    /// Kit has no filter chip; its nearest same-sized control is a ghost
    /// button, which also keeps a hitbox and focus handle like the chip does.
    fn kit_row(index: usize) -> AnyElement {
        let even = index.is_multiple_of(2);
        list_row(index)
            .child(Button::new(("filled", index)).primary().label("Filled"))
            .child(Button::new(("outlined", index)).outline().label("Outlined"))
            .child(Switch::new(("switch", index)).checked(even))
            .child(Checkbox::new(("check", index)).checked(!even))
            .child(Button::new(("chip", index)).ghost().label("Filter"))
            .child(supporting())
            .into_any_element()
    }

    fn cell(
        kind: Component,
        id: usize,
        inputs: &Inputs,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let on = id.is_multiple_of(2);
        match kind {
            Component::Empty => div().into_any_element(),
            Component::ButtonM => MaterialButton::new(("c", id))
                .variant(ButtonVariant::Filled)
                .child("Button")
                .into_any_element(),
            Component::ButtonK => Button::new(("c", id))
                .primary()
                .label("Button")
                .into_any_element(),
            Component::IconButtonM => IconButton::new(
                ("c", id),
                gpui_kit_material::icons::IconName::Check,
                "Check",
            )
            .into_any_element(),
            Component::SwitchM => MaterialSwitch::new(("c", id), "Switch")
                .checked(on)
                .into_any_element(),
            Component::SwitchK => Switch::new(("c", id)).checked(on).into_any_element(),
            Component::CheckboxM => MaterialCheckbox::new(("c", id), "Check")
                .checked(on)
                .into_any_element(),
            Component::CheckboxK => Checkbox::new(("c", id)).checked(on).into_any_element(),
            Component::RadioM => MaterialRadio::new(("c", id), "Radio")
                .selected(on)
                .into_any_element(),
            Component::RadioK => Radio::new(("c", id)).checked(on).into_any_element(),
            Component::ChipM => FilterChip::new(("c", id), "Filter")
                .selected(on)
                .into_any_element(),
            Component::TagK => Tag::secondary().small().child("Filter").into_any_element(),
            Component::ListItemM => ListItem::new(("c", id), "List item").into_any_element(),
            Component::ListItemK => KitListItem::new(("c", id))
                .child("List item")
                .into_any_element(),
            Component::TextFieldM => {
                let state = input_state(id, inputs, window, cx);
                TextField::new(("c", id), &state)
                    .label("Label")
                    .into_any_element()
            }
            Component::InputK => {
                let state = input_state(id, inputs, window, cx);
                Input::new(&state).into_any_element()
            }
            Component::TabsM => MaterialTabs::new(("c", id), "Sections")
                .items((0..3).map(|index| TabItem::new(format!("t{index}"), "Tab")))
                .selected(SharedString::new_static("t0"))
                .into_any_element(),
            Component::TabsK => TabBar::new(("c", id))
                .children((0..3).map(|_| Tab::new().label("Tab")))
                .selected_index(0)
                .into_any_element(),
            Component::RippleIdle => Ripple::new(
                ("c", id),
                div()
                    .size(rem(40.))
                    .rounded_full()
                    .bg(color(ColorRole::PRIMARY_CONTAINER)),
            )
            .round()
            .into_any_element(),
            Component::RippleNone => div()
                .size(rem(40.))
                .rounded_full()
                .bg(color(ColorRole::PRIMARY_CONTAINER))
                .into_any_element(),
            Component::TextPlain => div().child("Label text").into_any_element(),
            Component::TextTracked => div()
                .letter_spacing(px(0.5))
                .child("Label text")
                .into_any_element(),
        }
    }

    fn input_state(
        id: usize,
        inputs: &Inputs,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<InputState> {
        inputs
            .borrow_mut()
            .entry(id)
            .or_insert_with(|| cx.new(|cx| InputState::new(window, cx)))
            .clone()
    }

    fn cell_row(
        kind: Component,
        index: usize,
        inputs: &Inputs,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let mut row = div()
            .id(("row", index))
            .flex()
            .flex_row()
            .w_full()
            .h(rem(64.))
            .px(rem(8.))
            .gap(rem(8.))
            .items_center()
            .overflow_hidden();
        for column in 0..PER_ROW {
            row = row.child(div().w(rem(180.)).flex_shrink_0().child(cell(
                kind,
                index * PER_ROW + column,
                inputs,
                window,
                cx,
            )));
        }
        row.into_any_element()
    }

    impl Render for Bench {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            self.renders.set(self.renders.get() + 1);
            let (content, inputs, builds) =
                (self.content, self.inputs.clone(), self.row_builds.clone());
            div()
                .size_full()
                .bg(color(ColorRole::SURFACE))
                .text_color(color(ColorRole::ON_SURFACE))
                .child(
                    list(self.list.clone(), move |index, window, cx| {
                        builds.set(builds.get() + 1);
                        match content {
                            Content::MaterialList => material_row(index),
                            Content::KitList => kit_row(index),
                            Content::Simple => simple_row(index),
                            Content::Cells(kind) => cell_row(kind, index, &inputs, window, cx),
                        }
                    })
                    .size_full()
                    .with_sizing_behavior(ListSizingBehavior::Auto),
                )
        }
    }

    /// One recorded frame: the counters, then root renders, row builds, frames
    /// drawn and views built during it.
    type Frame = ([u64; 5], u64, u64, u64, u64);

    struct Run {
        mode: Mode,
        quiet: usize,
        settle: usize,
        frames: usize,
        view: Entity<Bench>,
        engine_at_start: Value,
        recorded: Vec<Frame>,
        seen: usize,
        last: Counters,
        last_stats: (u64, u64),
        last_renders: u64,
        last_builds: u64,
        positions: Vec<f32>,
    }

    fn stats(window: &Window) -> (u64, u64) {
        #[cfg(feature = "gpui-fast")]
        {
            let stats = window.layout_stats();
            (stats.frames, stats.views_built)
        }
        #[cfg(not(feature = "gpui-fast"))]
        {
            let _ = window;
            (0, 0)
        }
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

    type Done = Rc<dyn Fn(Run, &mut Window, &mut App)>;

    fn step(mut run: Run, done: Done, window: &mut Window, cx: &mut App) {
        let now = Counters::now();
        let (renders, builds) = {
            let bench = run.view.read(cx);
            (bench.renders.get(), bench.row_builds.get())
        };
        let frame_stats = stats(window);
        if run.seen == run.quiet + run.settle {
            run.engine_at_start = engine_counters(window);
        }
        if run.seen > run.quiet + run.settle {
            run.recorded.push((
                now.since(run.last).row(),
                renders - run.last_renders,
                builds - run.last_builds,
                frame_stats.0.saturating_sub(run.last_stats.0),
                frame_stats.1.saturating_sub(run.last_stats.1),
            ));
        }
        run.seen += 1;
        if run.recorded.len() >= run.frames {
            done(run, window, cx);
            return;
        }
        let quiet = run.seen <= run.quiet;
        match run.mode {
            _ if quiet => {}
            Mode::Scroll => {
                let list = run.view.read(cx).list.clone();
                let top = list.logical_scroll_top();
                run.positions
                    .push(top.item_ix as f32 * 1000. + f32::from(top.offset_in_item));
                let bounds = list.viewport_bounds();
                window.dispatch_event(
                    PlatformInput::Scroll(ScrollEvent {
                        position: bounds.center(),
                        delta: ScrollDelta::Pixels(point(px(0.), px(-STEP_PX))),
                        ..Default::default()
                    }),
                    cx,
                );
            }
            Mode::Redraw => run.view.update(cx, |_, cx| cx.notify()),
            Mode::Idle => {}
        }
        run.last = now;
        run.last_stats = frame_stats;
        run.last_renders = renders;
        run.last_builds = builds;
        window.on_next_frame(move |window, cx| step(run, done, window, cx));
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

    fn write_report(out: &std::path::Path, report: &Value) {
        if let Some(parent) = out.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(out, serde_json::to_vec(report).unwrap()).expect("write report");
    }

    pub fn main() {
        let options = parse();
        let metadata = json!({
            "commit": command("git", &["-C", env!("CARGO_MANIFEST_DIR"), "rev-parse", "HEAD"]),
            "dirty": command("git", &["-C", env!("CARGO_MANIFEST_DIR"), "status", "--porcelain"])
                .map(|status| !status.is_empty()),
            "cpu": command("sysctl", &["-n", "machdep.cpu.brand_string"]),
            "load_at_start": command("sysctl", &["-n", "vm.loadavg"]),
            "release": !cfg!(debug_assertions),
            "backend": if gpui_kit::GPUI_FAST { "gpui-fast" } else { "gpui-pre" },
        });
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
                let renders = Rc::new(Cell::new(0));
                let row_builds = Rc::new(Cell::new(0));
                let (content, rows) = (options.content, options.rows);
                let (window, view) = gpui_kit::open_window(
                    WindowOptions {
                        // The bench runs unfocused on a shared desktop; it must
                        // not be throttled like an inactive window.
                        inactive_frame_interval: None,
                        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                            None,
                            size(px(options.width), px(options.height)),
                            cx,
                        ))),
                        ..Default::default()
                    },
                    cx,
                    {
                        let (renders, row_builds) = (renders.clone(), row_builds.clone());
                        move |_, cx| {
                            cx.new(|_| Bench {
                                list: ListState::new(rows, ListAlignment::Top, px(256.)),
                                content,
                                renders,
                                row_builds,
                                inputs: Default::default(),
                            })
                        }
                    },
                )
                .expect("open window");
                cx.activate(true);
                let name = options.content_name.clone();
                let out = options.out.clone();
                let mode = options.mode;
                if mode == Mode::Idle {
                    let seconds = options.idle_seconds;
                    cx.spawn(async move |cx| {
                        let executor = cx.background_executor().clone();
                        let timer = |s: f64| executor.timer(std::time::Duration::from_secs_f64(s));
                        timer(2.).await;
                        let before = window
                            .update(cx, |_, window, _| (Counters::now(), stats(window)))
                            .ok();
                        let start_renders = cx.update(|cx| view.read(cx).renders.get());
                        timer(seconds).await;
                        let after = window
                            .update(cx, |_, window, _| (Counters::now(), stats(window)))
                            .ok();
                        let end_renders = cx.update(|cx| view.read(cx).renders.get());
                        if let (Some(before), Some(after)) = (before, after) {
                            write_report(
                                &out,
                                &json!({
                                    "schema":"component-bench-v1","content":name,"mode":"idle",
                                    "seconds":seconds,"metadata":metadata,
                                    "columns":["instructions","cycles","cpu_ns","allocations","allocated_bytes"],
                                    "total":after.0.since(before.0).row(),
                                    "frames_drawn":after.1.0.saturating_sub(before.1.0),
                                    "views_built":after.1.1.saturating_sub(before.1.1),
                                    "root_renders":end_renders-start_renders,
                                }),
                            );
                        }
                        cx.update(|cx| cx.quit());
                    })
                    .detach();
                    return;
                }
                let done: Done = Rc::new(move |run: Run, window, cx| {
                    let (viewport, scale) = (window.viewport_size(), window.scale_factor());
                    let moved = run.positions.windows(2).filter(|w| w[0] != w[1]).count();
                    write_report(
                        &out,
                        &json!({
                            "schema":"component-bench-v1","content":name,
                            "mode":format!("{:?}",run.mode).to_lowercase(),
                            "metadata":metadata,
                            "viewport":[f32::from(viewport.width),f32::from(viewport.height)],
                            "scale":scale,"step_px":STEP_PX,"per_row":PER_ROW,
                            "scroll_steps_that_moved":moved,
                            "engine_at_start":run.engine_at_start,
                            "engine_at_end":engine_counters(window),
                            "columns":["instructions","cycles","cpu_ns","allocations","allocated_bytes","root_renders","row_builds","frames_drawn","views_built"],
                            "frames":run.recorded.iter().map(|(c,r,b,f,v)|{
                                json!([c[0],c[1],c[2],c[3],c[4],r,b,f,v])
                            }).collect::<Vec<_>>(),
                        }),
                    );
                    cx.quit();
                });
                let (frames, quiet, settle) =
                    (options.frames, options.quiet_frames, options.settle_frames);
                window
                    .update(cx, move |_, window, cx| {
                        let run = Run {
                            mode,
                            quiet,
                            settle,
                            frames,
                            view,
                            engine_at_start: Value::Null,
                            recorded: Vec::with_capacity(frames),
                            seen: 0,
                            last: Counters::now(),
                            last_stats: (0, 0),
                            last_renders: 0,
                            last_builds: 0,
                            positions: Vec::with_capacity(frames + settle),
                        };
                        let _ = cx;
                        window.on_next_frame(move |window, cx| step(run, done, window, cx));
                    })
                    .ok();
            });
    }
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    bench::main();
}

#[cfg(target_family = "wasm")]
fn main() {}
