//! The scheduling contract the engine patch this replaces was held to.
use super::*;
use core::prelude::v1::test;
use gpui::{Context, Render, ScrollHandle, TestAppContext, VisualTestContext, div, point};
use std::cell::RefCell;

type Phases = Rc<RefCell<Vec<f32>>>;

/// Frame callbacks delivered by simulating the next frame.
fn next_frame(cx: &mut VisualTestContext) -> usize {
    let count = cx.update(|window, cx| window.simulate_next_frame(cx));
    cx.run_until_parked();
    count
}

fn last(phases: &Phases) -> f32 {
    *phases.borrow().last().expect("sampled")
}

/// A 20px square, mounted or not, sampling a caller-owned clock.
struct Mounted {
    mounted: bool,
    playback: RepeatAnimationPlayback,
    phases: Phases,
}

impl Render for Mounted {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let phases = self.phases.clone();
        div().size(px(100.)).children(self.mounted.then(|| {
            div()
                .size(px(20.))
                .repeat_when_visible("repeat", Duration::from_secs(1), move |this, t| {
                    phases.borrow_mut().push(t);
                    this
                })
                .playback(Some(self.playback.clone()))
        }))
    }
}

fn mounted(
    cx: &mut TestAppContext,
    playback: RepeatAnimationPlayback,
    mounted: bool,
) -> (Entity<Mounted>, &mut VisualTestContext) {
    let (view, cx) = cx.add_window_view(move |_, _| Mounted {
        mounted,
        playback,
        phases: Phases::default(),
    });
    cx.run_until_parked();
    (view, cx)
}

fn set_mounted(view: &Entity<Mounted>, mounted: bool, cx: &mut VisualTestContext) {
    view.update(cx, |view, cx| {
        view.mounted = mounted;
        cx.notify();
    });
    cx.run_until_parked();
}

#[gpui::test]
fn a_playback_starts_on_first_mount_and_a_new_handle_restarts(cx: &mut TestAppContext) {
    let playback = RepeatAnimationPlayback::default();
    let (view, cx) = mounted(cx, playback.clone(), false);
    cx.executor().advance_clock(Duration::from_secs(3));
    assert!(!playback.is_started(), "nothing mounted, no clock");
    assert_eq!(next_frame(cx), 0);
    set_mounted(&view, true, cx);
    let phases = view.read_with(cx, |view, _| view.phases.clone());
    assert_eq!(*phases.borrow(), vec![0.]);
    cx.executor().advance_clock(Duration::from_millis(250));
    next_frame(cx);
    assert!((last(&phases) - 0.25).abs() < 0.01, "{}", last(&phases));
    view.update(cx, |view, cx| {
        view.playback = RepeatAnimationPlayback::default();
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(last(&phases), 0.);
}

#[gpui::test]
fn a_playback_keeps_its_phase_while_unmounted_without_drawing(cx: &mut TestAppContext) {
    let (view, cx) = mounted(cx, RepeatAnimationPlayback::default(), true);
    let phases = view.read_with(cx, |view, _| view.phases.clone());
    set_mounted(&view, false, cx);
    // The mounted frame queued one callback; it draws, and queues nothing more.
    next_frame(cx);
    assert_eq!(next_frame(cx), 0);
    cx.executor().advance_clock(Duration::from_millis(2350));
    assert_eq!(next_frame(cx), 0);
    let sampled = phases.borrow().len();
    set_mounted(&view, true, cx);
    assert_eq!(phases.borrow().len(), sampled + 1);
    assert!((last(&phases) - 0.35).abs() < 0.01, "{}", last(&phases));
    assert!(next_frame(cx) > 0, "mounted and visible, it runs again");
}

#[gpui::test]
fn another_view_mounting_a_playback_samples_the_same_phase(cx: &mut TestAppContext) {
    let playback = RepeatAnimationPlayback::default();
    let (first, cx) = mounted(cx, playback.clone(), true);
    set_mounted(&first, false, cx);
    cx.executor().advance_clock(Duration::from_millis(60));
    let (second, cx) = mounted(cx, playback, true);
    let phases = second.read_with(cx, |view, _| view.phases.clone());
    assert!((last(&phases) - 0.06).abs() < 0.01, "{}", last(&phases));
}

/// A square 150px down a 100px scrolling viewport: clipped until scrolled.
struct Clipped {
    scroll: ScrollHandle,
    opacity: f32,
    phases: Phases,
}

impl Render for Clipped {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let phases = self.phases.clone();
        let square = div()
            .mt(px(150.))
            .size(px(20.))
            .opacity(self.opacity)
            .repeat_when_visible("clipped", Duration::from_secs(1), move |this, t| {
                phases.borrow_mut().push(t);
                this
            });
        div()
            .id("viewport")
            .size(px(100.))
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .child(div().h(px(400.)).w(px(100.)).child(square))
    }
}

fn clipped(cx: &mut TestAppContext) -> (Entity<Clipped>, &mut VisualTestContext) {
    let (view, cx) = cx.add_window_view(|_, _| Clipped {
        scroll: ScrollHandle::new(),
        opacity: 1.,
        phases: Phases::default(),
    });
    cx.run_until_parked();
    (view, cx)
}

fn scroll_to(view: &Entity<Clipped>, y: f32, cx: &mut VisualTestContext) {
    view.update(cx, |view, cx| {
        view.scroll.set_offset(point(px(0.), px(-y)));
        cx.notify();
    });
    cx.run_until_parked();
}

#[gpui::test]
fn a_clipped_repeat_requests_no_frames(cx: &mut TestAppContext) {
    let (view, cx) = clipped(cx);
    assert_eq!(next_frame(cx), 0, "an idle window draws nothing");
    assert_eq!(view.read_with(cx, |view, _| view.phases.borrow().len()), 1);
}

#[gpui::test]
fn scrolling_in_resumes_the_original_phase_even_transparent(cx: &mut TestAppContext) {
    let (view, cx) = clipped(cx);
    let phases = view.read_with(cx, |view, _| view.phases.clone());
    cx.executor().advance_clock(Duration::from_millis(2350));
    view.update(cx, |view, _| view.opacity = 0.);
    scroll_to(&view, 150., cx);
    assert!(
        (last(&phases) - 0.35).abs() < 0.01,
        "restarted: {}",
        last(&phases)
    );
    assert!(next_frame(cx) > 0, "visible, it keeps drawing");
    scroll_to(&view, 0., cx);
    // A callback queued while visible may still fire once.
    next_frame(cx);
    assert_eq!(next_frame(cx), 0, "clipped again, it stops");
}

#[gpui::test]
fn reduced_motion_holds_the_first_frame_and_requests_none(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let (view, cx) = clipped(cx);
    scroll_to(&view, 150., cx);
    assert_eq!(next_frame(cx), 0);
    let phases = view.read_with(cx, |view, _| view.phases.clone());
    assert!(phases.borrow().iter().all(|t| *t == 0.));
}

#[test]
fn the_phase_wraps_as_gpuis_repeat_does() {
    let period = Duration::from_secs(2);
    assert_eq!(phase(Duration::ZERO, period), 0.);
    assert_eq!(phase(Duration::from_secs(1), period), 0.5);
    assert_eq!(phase(Duration::from_secs(2), period), 1.);
    assert!((phase(Duration::from_millis(4500), period) - 0.25).abs() < 1e-5);
}

#[test]
#[should_panic(expected = "a repeating animation needs a period")]
fn a_zero_period_is_rejected() {
    let _ = div().repeat_when_visible("zero", Duration::ZERO, |this, _| this);
}
