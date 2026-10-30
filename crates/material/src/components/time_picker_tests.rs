use super::*;
use core::prelude::v1::test;

fn t(h: u32, m: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(h, m, 0).unwrap()
}

pub(crate) fn labels() -> TimePickerLabels {
    let s = SharedString::new_static;
    TimePickerLabels {
        select_title: s("Select time"),
        enter_title: s("Enter time"),
        hour: s("Hour"),
        minute: s("Minute"),
        am: s("AM"),
        pm: s("PM"),
        confirm: s("OK"),
        cancel: s("Cancel"),
        switch_to_input: s("Keyboard"),
        switch_to_dial: s("Dial"),
        number: Rc::new(|n| n.to_string().into()),
        two_digits: Rc::new(|n| format!("{n:02}").into()),
    }
}

#[test]
fn the_hand_turns_along_the_shorter_arc() {
    // From 11 o'clock to 1 o'clock goes forward through 12.
    let from = hand_turns(t(11, 0), TimeUnit::Hour, None);
    let to = hand_turns(t(1, 0), TimeUnit::Hour, Some(from));
    assert!((to - (from + 2. / 12.)).abs() < 1e-5, "{from} -> {to}");
    let back = hand_turns(t(10, 0), TimeUnit::Hour, Some(to));
    assert!((back - (to - 3. / 12.)).abs() < 1e-5);
    assert!((hand_turns(t(7, 30), TimeUnit::Minute, None) - 0.5).abs() < 1e-5);
}

#[test]
fn twenty_four_hour_hours_use_the_inner_ring() {
    let f = TimeFormat::TwentyFourHour;
    assert!(inner_ring(t(20, 0), TimeUnit::Hour, f));
    assert!(inner_ring(t(0, 0), TimeUnit::Hour, f));
    assert!(!inner_ring(t(12, 0), TimeUnit::Hour, f));
    assert!(!inner_ring(t(7, 0), TimeUnit::Hour, f));
    assert!(!inner_ring(t(20, 0), TimeUnit::Minute, f));
    assert!(!inner_ring(
        t(20, 0),
        TimeUnit::Hour,
        TimeFormat::TwelveHour
    ));
}

#[test]
fn pointer_positions_pick_the_nearest_value() {
    let base = t(9, 0);
    // Straight down on the outer ring of a 12-hour face is 6 (keeping AM).
    let six = dial_pick(
        base,
        TimeUnit::Hour,
        TimeFormat::TwelveHour,
        point(0., 100.),
    );
    assert_eq!(six, t(6, 0));
    let pm = dial_pick(
        t(21, 0),
        TimeUnit::Hour,
        TimeFormat::TwelveHour,
        point(0., -100.),
    );
    assert_eq!(pm, t(12, 0), "12 o'clock in the PM period is noon");
    let f = TimeFormat::TwentyFourHour;
    assert_eq!(dial_pick(base, TimeUnit::Hour, f, point(0., 76.)), t(18, 0));
    assert_eq!(dial_pick(base, TimeUnit::Hour, f, point(0., -76.)), t(0, 0));
    assert_eq!(
        dial_pick(base, TimeUnit::Hour, f, point(0., -104.)),
        t(12, 0)
    );
    assert_eq!(
        dial_pick(base, TimeUnit::Minute, f, point(100., 0.)),
        t(9, 15)
    );
}

#[test]
fn steps_wrap_within_the_period_or_day() {
    let h12 = TimeFormat::TwelveHour;
    assert_eq!(step_time(t(23, 0), TimeUnit::Hour, h12, 1), t(12, 0));
    assert_eq!(step_time(t(0, 0), TimeUnit::Hour, h12, -1), t(11, 0));
    let h24 = TimeFormat::TwentyFourHour;
    assert_eq!(step_time(t(23, 0), TimeUnit::Hour, h24, 1), t(0, 0));
    assert_eq!(step_time(t(5, 59), TimeUnit::Minute, h24, 1), t(5, 0));
}

struct Harness(Entity<TimePickerState>);
impl Render for Harness {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div().size_full().child(TimePicker::new("picker", &self.0))
    }
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

#[gpui::test]
fn the_dial_steps_and_the_period_toggles(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|window, cx| {
        Harness(cx.new(|cx| TimePickerState::new(labels(), t(7, 0), window, cx)))
    });
    let state = view.read_with(cx, |h, _| h.0.clone());
    draw(cx);
    cx.update(|window, cx| state.read(cx).dial_focus().clone().focus(window, cx));
    cx.simulate_keystrokes("up up");
    assert_eq!(state.read_with(cx, |s, _| s.time()), t(9, 0));
    cx.update(|window, cx| state.update(cx, |s, cx| s.set_pm(true, window, cx)));
    assert_eq!(state.read_with(cx, |s, _| s.time()), t(21, 0));
    // Base's field follows the dial.
    let field = state.read_with(cx, |s, _| s.field().clone());
    assert_eq!(field.read_with(cx, |f, _| f.time()), t(21, 0));
}

#[gpui::test]
fn typing_in_input_mode_updates_the_time(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|window, cx| {
        Harness(cx.new(|cx| {
            TimePickerState::new(labels(), t(7, 0), window, cx)
                .format(TimeFormat::TwentyFourHour, window, cx)
                .input_mode()
        }))
    });
    let state = view.read_with(cx, |h, _| h.0.clone());
    draw(cx);
    cx.update(|window, cx| {
        let handle = state.read(cx).field().read(cx).focus_handle(cx);
        handle.focus(window, cx);
    });
    draw(cx);
    cx.simulate_keystrokes("2 0 3 5");
    assert_eq!(state.read_with(cx, |s, _| s.time()), t(20, 35));
}
