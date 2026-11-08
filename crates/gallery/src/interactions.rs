//! Opt-in scripted interaction profile (`MATERIAL_PROFILE_SCRIPT=interactions`
//! with `MATERIAL_PROFILE_OUTPUT=<frames.json>`): page switches between
//! Button, Text field, Date picker and Examples, hover and press in the
//! Button preview, and continuous wheel scrolling of Overview and a Specs
//! tab. It reads upstream's own profiler (`gpui-kit/profiler`): every frame's
//! `FrameEvent::Draw` and the `FrameEvent::Present` that submitted it are kept
//! raw and labelled with the step that was active when the draw started;
//! `page_switch_ns` holds the first frame after each switch. A frame's time is
//! its draw plus its submission: the platform callback around them (input
//! dispatch, cleanup) and the GPU are not timed. Never enabled by the ordinary
//! gallery (`scripts/profile-gallery.py` summarizes it).
use super::{Gallery, Tab, catalog};
use gpui::{
    profiler::{self, FrameEvent, FrameTimingCollector},
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
            profiler::set_trace_enabled(false);
            let report = report(events, window_id, origin, &marks.borrow(), &moved.borrow());
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
    let frames = frames(&events, window_id);
    let rows: Vec<Value> = frames
        .iter()
        .enumerate()
        .map(|(index, frame)| {
            let draw = &frame.draw;
            json!({
                "initial": index == 0,
                "step": step_at(draw.draw_start),
                "start_ns": ns(draw.draw_start),
                "frame_ns": frame.cpu().as_nanos(),
                "draw_ns": draw.draw_duration().as_nanos(),
                "present_ns": frame.present.map(|present| present.present_duration().as_nanos()),
                "phases": {
                    "request_layout_ns": draw.phases.request_layout.as_nanos(),
                    "prepaint_ns": draw.phases.prepaint.as_nanos(),
                    "paint_ns": draw.phases.paint.as_nanos(),
                },
            })
        })
        .collect();
    // The first frame after each switch is that switch's cost.
    let mut switches = Vec::new();
    for (label, at) in marks {
        if !label.starts_with("switch:") {
            continue;
        }
        if let Some(frame) = frames.iter().find(|frame| frame.draw.draw_start >= *at) {
            switches.push(json!({"step": label, "frame_ns": frame.cpu().as_nanos()}));
        }
    }
    json!({
        "schema": "gallery-interactions-v2",
        "metadata": {
            "release": !cfg!(debug_assertions),
            "backend": if gpui_kit::GPUI_FAST { "gpui-fast" } else { "gpui-pre" },
            "load_average_at_end": std::process::Command::new("sysctl").args(["-n", "vm.loadavg"]).output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned()),
            "script": std::env::var("MATERIAL_PROFILE_SCRIPT").ok(), "interactions_script": SCRIPT.iter().map(|(label, _, wait)| json!([label, wait])).collect::<Vec<_>>(),
            "frame_definition": "upstream profiler: Window::draw (render, layout, prepaint, paint) plus the platform submission of that draw, when there was one; the frame callback around them, drawable waits outside the submission and GPU time are not measured",
        },
        "marks_ns": marks.iter().map(|(label, at)| json!([label, ns(*at)])).collect::<Vec<_>>(),
        "scroll_moved_px": moved,
        "page_switch_ns": switches.iter().map(|s| s["frame_ns"].clone()).collect::<Vec<_>>(),
        "page_switches": switches,
        "frames": rows,
    })
}

/// A drawn frame and the submission of what it drew, if it was submitted.
struct Frame {
    draw: profiler::FrameTiming,
    present: Option<profiler::PresentTiming>,
}

impl Frame {
    fn cpu(&self) -> Duration {
        self.draw.draw_duration()
            + self
                .present
                .map_or(Duration::ZERO, |present| present.present_duration())
    }
}

/// `window`'s draws in order, each with the first submission that began after it
/// ended and before the next draw.
fn frames(events: &[FrameEvent], window: WindowId) -> Vec<Frame> {
    let mut frames: Vec<Frame> = Vec::new();
    for event in events {
        match event {
            FrameEvent::Draw(draw) if draw.window_id == window => frames.push(Frame {
                draw: *draw,
                present: None,
            }),
            FrameEvent::Present(present) if present.window_id == window => {
                if let Some(frame) = frames.last_mut()
                    && frame.present.is_none()
                    && present.present_start >= frame.draw.draw_end
                {
                    frame.present = Some(*present);
                }
            }
            _ => {}
        }
    }
    frames
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::profiler::{DrawPhases, FrameTiming, PresentTiming};

    #[test]
    fn a_frame_is_its_draw_and_the_submission_after_it() {
        let window = WindowId::from(1);
        let at = Instant::now();
        let ms = |n: u64| at + Duration::from_millis(n);
        let draw = |start, end| {
            FrameEvent::Draw(FrameTiming {
                window_id: window,
                dirty_at: None,
                invalidations: 1,
                draw_start: ms(start),
                draw_end: ms(end),
                refresh_interval: None,
                signal_at: None,
                phases: DrawPhases::default(),
            })
        };
        let present = |start, end| {
            FrameEvent::Present(PresentTiming {
                window_id: window,
                present_start: ms(start),
                present_end: ms(end),
                animation_interval: None,
            })
        };
        // A draw that was not submitted, then one that was, then a second
        // submission of the same draw, which is not a frame's.
        let frames = frames(
            &[draw(0, 2), draw(10, 13), present(13, 14), present(20, 21)],
            window,
        );
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].cpu(), Duration::from_millis(2));
        assert_eq!(frames[1].cpu(), Duration::from_millis(4));
    }
}
