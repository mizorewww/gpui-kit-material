//! Time picker playground: md3.io's Time picker configuration (dial or
//! input, 12- or 24-hour clock, orientation, presets, title, time, seconds,
//! minute step, earliest and latest times) on the real `TimePicker`.
//!
//! The picker edits a draft: OK writes it to the Time control (24-hour
//! `HH:MM` or `HH:MM:SS`), Cancel returns it to the control's time. A
//! configuration change rebuilds the picker state.
use super::super::shell::{fit_preview, preview_area};
use super::prelude::*;
use std::rc::Rc;

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Event time",
            values: &[
                ("preset", Set::Option("Event time")),
                ("title", Set::Text("Event starts")),
                ("time", Set::Text("10:30")),
                ("step", Set::Option("15")),
                ("limit", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Evening event",
            values: &[
                ("preset", Set::Option("Evening event")),
                ("title", Set::Text("Dinner reservation")),
                ("time", Set::Text("19:00")),
                ("format", Set::Option("24h")),
                ("orientation", Set::Option("horizontal")),
                ("step", Set::Option("5")),
                ("limit", Set::Bool(true)),
                ("earliest", Set::Text("17:00")),
                ("latest", Set::Text("23:00")),
            ],
        },
        Scenario {
            name: "Typed time",
            values: &[
                ("preset", Set::Option("Typed time")),
                ("mode", Set::Option("input")),
                ("title", Set::Text("Enter time")),
                ("time", Set::Text("07:45:30")),
                ("seconds", Set::Bool(true)),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Segmented {
                    id: "mode",
                    label: "Input mode",
                    options: &["dial", "input"],
                    default: 0,
                },
                Control::Segmented {
                    id: "format",
                    label: "Clock format",
                    options: &["12h", "24h"],
                    default: 0,
                },
                Control::Segmented {
                    id: "orientation",
                    label: "Orientation",
                    options: &["vertical", "horizontal"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "preset",
                    label: "Time picker",
                    options: &["Default", "Event time", "Evening event", "Typed time"],
                    default: 0,
                },
                Control::Text {
                    id: "title",
                    label: "Title",
                    default: "Select time",
                },
                Control::Text {
                    id: "time",
                    label: "Time",
                    default: "09:41",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "seconds",
                    label: "Show seconds",
                    default: false,
                },
                Control::Segmented {
                    id: "step",
                    label: "Minute step",
                    options: &["1", "5", "15"],
                    default: 0,
                },
                Control::Switch {
                    id: "limit",
                    label: "Limit times",
                    default: false,
                },
                Control::Text {
                    id: "earliest",
                    label: "Earliest time",
                    default: "09:00",
                },
                Control::Text {
                    id: "latest",
                    label: "Latest time",
                    default: "17:00",
                },
            ],
        },
    ],
    preview,
};

/// A 24-hour `HH:MM` or `HH:MM:SS` time.
fn parse_time(text: &str) -> Option<NaiveTime> {
    let text = text.trim();
    NaiveTime::parse_from_str(text, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(text, "%H:%M"))
        .ok()
}

fn format_time(time: NaiveTime, seconds: bool) -> SharedString {
    time.format(if seconds { "%H:%M:%S" } else { "%H:%M" })
        .to_string()
        .into()
}

/// English time picker strings with `title` for both modes.
fn labels(title: SharedString) -> TimePickerLabels {
    let s = SharedString::new_static;
    TimePickerLabels {
        select_title: title.clone(),
        enter_title: title,
        hour: s("Hour"),
        minute: s("Minute"),
        am: s("AM"),
        pm: s("PM"),
        confirm: s("OK"),
        cancel: s("Cancel"),
        switch_to_input: s("Switch to text input"),
        switch_to_dial: s("Switch to clock"),
        number: Rc::new(|n| n.to_string().into()),
        two_digits: Rc::new(|n| format!("{n:02}").into()),
        separator: s(":"),
    }
}

/// What a picker state is built from.
#[derive(Clone, Hash, PartialEq)]
struct Build {
    title: SharedString,
    twenty_four: bool,
    input: bool,
    seconds: bool,
    step: u32,
    limits: Option<(NaiveTime, NaiveTime)>,
}

/// The preview's picker state, rebuilt with its configuration.
struct Picker(Option<Entity<TimePickerState>>);

/// The reference picker's widths, in design px.
const VERTICAL_WIDTH: f32 = 328.;
const HORIZONTAL_WIDTH: f32 = 584.;

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let seconds = config.bool("seconds");
    let limits = config
        .bool("limit")
        .then(|| {
            Some((
                parse_time(&config.text("earliest"))?,
                parse_time(&config.text("latest"))?,
            ))
        })
        .flatten();
    let spec = Build {
        title: config.text("title"),
        twenty_four: config.option("format") == "24h",
        input: config.option("mode") == "input",
        seconds,
        step: config.option("step").parse().unwrap_or(1),
        limits,
    };
    let time = parse_time(&config.text("time"))
        .unwrap_or_else(|| NaiveTime::from_hms_opt(9, 41, 0).expect("a valid time"));
    let holder = preview.local("picker", window, cx, |_, _| Picker(None));
    let rebuild = preview.changed("time-build", spec.clone()) || holder.read(cx).0.is_none();
    let state = if rebuild {
        let state = cx.new(|cx| {
            let mut state = TimePickerState::new(labels(spec.title.clone()), time, window, cx)
                .minute_step(spec.step);
            if spec.twenty_four {
                state = state.format(TimeFormat::TwentyFourHour, window, cx);
            }
            if spec.seconds {
                state = state.seconds("Second", window, cx);
            }
            if let Some((earliest, latest)) = spec.limits {
                state = state.limits(earliest, latest, window, cx);
            }
            if spec.input {
                state = state.input_mode();
            }
            state
        });
        // Its edits re-render the preview.
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        holder.update(cx, |holder, _| holder.0 = Some(state.clone()));
        state
    } else {
        let state = holder.read(cx).0.clone().expect("built above");
        if preview.changed("time-value", config.text("time")) && state.read(cx).time() != time {
            state.update(cx, |state, cx| state.set_time(time, window, cx));
        }
        state
    };
    let write = preview.setter("time", cx);
    let revert = state.clone();
    // A narrow window shows the vertical picker, whatever the configuration asks.
    let horizontal = config.option("orientation") == "horizontal"
        && preview_area(window, true).0 >= HORIZONTAL_WIDTH;
    let picker = TimePicker::new("playground-time-picker", &state)
        .orientation(if horizontal {
            TimeOrientation::Horizontal
        } else {
            TimeOrientation::Vertical
        })
        .on_confirm(move |time, window, cx| {
            write(Value::Text(format_time(time, seconds)), window, cx)
        })
        .on_cancel(move |window, cx| {
            revert.update(cx, |state, cx| state.set_time(time, window, cx))
        });
    let picker = div()
        .debug_selector(|| "playground-time-picker".into())
        .child(picker)
        .into_any_element();
    if horizontal {
        picker
    } else {
        fit_preview(VERTICAL_WIDTH, picker, window)
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn times_parse_with_or_without_seconds() {
        let t = |h, m, s| NaiveTime::from_hms_opt(h, m, s).unwrap();
        assert_eq!(parse_time("09:41"), Some(t(9, 41, 0)));
        assert_eq!(parse_time("07:45:30"), Some(t(7, 45, 30)));
        assert_eq!(parse_time("25:00"), None);
        assert_eq!(format_time(t(7, 45, 30), true), "07:45:30");
        assert_eq!(format_time(t(7, 45, 30), false), "07:45");
    }

    #[gpui::test]
    fn every_scenario_renders_and_ok_writes_the_dial_choice(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, &PLAYGROUND);
        every_scenario_renders(&state, cx, "playground-time-picker");
        cx.update(|window, cx| state.update(cx, |state, cx| state.reset(window, cx)));
        settle(cx);
        // The window's rem is 16px: design px scale by 16/14.
        let s = 16. / 14.;
        let picker = rect(cx, "playground-time-picker");
        // The 3 o'clock number of the 256dp dial (36dp in, 176dp down).
        let three = point(
            picker.left() + px((36. + 128. + 104.) * s),
            picker.top() + px((176. + 128.) * s),
        );
        cx.simulate_click(three, Modifiers::none());
        settle(cx);
        assert_eq!(config(&state, cx).text("time"), "09:41", "a draft until OK");
        // OK: the last action, 24dp in from the right and 20dp up.
        let ok = point(
            picker.left() + px((328. - 24. - 24.) * s),
            picker.top() + picker.size.height - px((20. + 20.) * s),
        );
        cx.simulate_click(ok, Modifiers::none());
        settle(cx);
        assert_eq!(config(&state, cx).text("time"), "03:41");
    }
}
