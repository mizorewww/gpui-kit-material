//! Date picker playground: md3.io's Date picker configuration (docked,
//! modal, modal input or full-screen; initial view; presets; label; date
//! and range end; date format; range; close on selection; disabled;
//! earliest and latest dates) on the real `DatePicker`.
//!
//! The preview is a field showing the chosen date in the chosen format;
//! clicking it (or its calendar button) opens the picker below it (docked)
//! or in a dialog. Choosing writes Date (and Range end) back as ISO dates;
//! with "Close on selection" a complete choice closes the picker, otherwise
//! OK / Save does. A configuration change rebuilds the picker state.
use super::prelude::*;
use gpui_base::input::InputState;
use std::rc::Rc;

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const FORMATS: [&str; 3] = ["MM/DD/YYYY", "DD/MM/YYYY", "YYYY MM DD"];
const DATE_FORMATS: [DateFormat; 3] = DateFormat::ALL;

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Profile form",
            values: &[
                ("preset", Set::Option("Profile form")),
                ("label", Set::Text("Start date")),
                ("date", Set::Text("2025-09-01")),
                ("close", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Trip dates",
            values: &[
                ("preset", Set::Option("Trip dates")),
                ("variant", Set::Option("fullscreen")),
                ("label", Set::Text("Trip dates")),
                ("range", Set::Bool(true)),
                ("date", Set::Text("2025-08-17")),
                ("range_end", Set::Text("2025-08-23")),
            ],
        },
        Scenario {
            name: "Date of birth",
            values: &[
                ("preset", Set::Option("Date of birth")),
                ("variant", Set::Option("modal input")),
                ("label", Set::Text("Date of birth")),
                ("format", Set::Option("DD/MM/YYYY")),
                ("limit", Set::Bool(true)),
                ("earliest", Set::Text("1900-01-01")),
                ("latest", Set::Text("2025-08-05")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Select {
                    id: "variant",
                    label: "Variant",
                    options: &["docked", "modal", "modal input", "fullscreen"],
                    default: 0,
                },
                Control::Segmented {
                    id: "view",
                    label: "Initial view",
                    options: &["day", "month", "year"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "preset",
                    label: "Date picker",
                    options: &["Default", "Profile form", "Trip dates", "Date of birth"],
                    default: 0,
                },
                Control::Text {
                    id: "label",
                    label: "Label",
                    default: "Date",
                },
                Control::Text {
                    id: "date",
                    label: "Date",
                    default: "2025-08-17",
                },
                Control::Text {
                    id: "range_end",
                    label: "Range end",
                    default: "",
                },
                Control::Segmented {
                    id: "format",
                    label: "Date format",
                    options: &FORMATS,
                    default: 0,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "range",
                    label: "Date range",
                    default: false,
                },
                Control::Switch {
                    id: "close",
                    label: "Close on selection",
                    default: false,
                },
                Control::Switch {
                    id: "disabled",
                    label: "Disabled",
                    default: false,
                },
                Control::Switch {
                    id: "limit",
                    label: "Limit dates",
                    default: false,
                },
                Control::Text {
                    id: "earliest",
                    label: "Earliest date",
                    default: "2025-01-01",
                },
                Control::Text {
                    id: "latest",
                    label: "Latest date",
                    default: "2025-12-31",
                },
            ],
        },
    ],
    preview,
};

fn short_month(month: u32) -> &'static str {
    &MONTHS[(month as usize + 11) % 12][..3]
}

/// An ISO `YYYY-MM-DD` date from a Text control.
fn iso(text: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d").ok()
}

fn to_iso(date: Option<NaiveDate>) -> SharedString {
    date.map(|d| d.format("%Y-%m-%d").to_string().into())
        .unwrap_or_default()
}

/// English date picker strings in date format `format`.
fn labels(range: bool, format: DateFormat) -> DatePickerLabels {
    let s = SharedString::new_static;
    let short = |d: NaiveDate| format!("{} {}", short_month(d.month()), d.day());
    let hint = SharedString::new_static(
        FORMATS[DATE_FORMATS
            .iter()
            .position(|f| *f == format)
            .unwrap_or_default()],
    );
    DatePickerLabels {
        title: if range {
            s("Select dates")
        } else {
            s("Select date")
        },
        no_selection: if range {
            s("Start date – End date")
        } else {
            s("Selected date")
        },
        input_headline: if range {
            s("Enter dates")
        } else {
            s("Enter date")
        },
        confirm: s("OK"),
        cancel: s("Cancel"),
        clear: s("Clear"),
        save: s("Save"),
        close: s("Close"),
        previous_month: s("Previous month"),
        next_month: s("Next month"),
        previous_year: s("Previous year"),
        next_year: s("Next year"),
        switch_to_input: s("Switch to text input"),
        switch_to_calendar: s("Switch to calendar"),
        choose_year: s("Choose year"),
        choose_month: s("Choose month"),
        date_field: s("Date"),
        start_field: s("Start date"),
        end_field: s("End date"),
        input_placeholder: hint.clone(),
        invalid_input: format!("Use {hint}").into(),
        weekday: Rc::new(|w: Weekday| {
            ["S", "M", "T", "W", "T", "F", "S"][w.num_days_from_sunday() as usize].into()
        }),
        month: Rc::new(|m| MONTHS[(m as usize + 11) % 12].into()),
        month_short: Rc::new(|m| short_month(m).into()),
        month_year: Rc::new(|y, m| format!("{} {y}", MONTHS[(m as usize + 11) % 12]).into()),
        headline: Rc::new(|d: NaiveDate| {
            format!(
                "{}, {} {}",
                WEEKDAYS[d.weekday().num_days_from_monday() as usize],
                short_month(d.month()),
                d.day()
            )
            .into()
        }),
        range_headline: Rc::new(move |a, b| {
            let part = |d: Option<NaiveDate>, empty: &str| d.map(short).unwrap_or(empty.into());
            format!("{} – {}", part(a, "Start date"), part(b, "End date")).into()
        }),
        accessible_date: Rc::new(|d: NaiveDate| {
            format!(
                "{} {} {}, {}",
                WEEKDAYS[d.weekday().num_days_from_monday() as usize],
                MONTHS[d.month0() as usize],
                d.day(),
                d.year()
            )
            .into()
        }),
        number: Rc::new(|n| n.to_string().into()),
        format_input: Rc::new(move |d| format.format(d)),
        parse_input: Rc::new(move |text| format.parse(text)),
    }
}

/// The preview's picker: its state (rebuilt with the configuration) and
/// whether it is open.
struct Picker {
    state: Option<Entity<DatePickerState>>,
    open: bool,
}

/// The configuration a picker state is built from.
#[derive(Clone, Copy, Hash, PartialEq)]
struct Build {
    range: bool,
    format: DateFormat,
    view: usize,
    input: bool,
    docked: bool,
    limits: Option<(NaiveDate, NaiveDate)>,
}

/// The selection the configuration describes.
fn selection(config: &Config, range: bool) -> DateSelection {
    let start = iso(&config.text("date"));
    if range {
        DateSelection::Range(start, iso(&config.text("range_end")))
    } else {
        DateSelection::Single(start)
    }
}

/// Write `value` back to Date and Range end.
fn write_back(
    state: &mut PlaygroundState,
    value: DateSelection,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) {
    let (start, end) = match value {
        DateSelection::Single(d) => (d, None),
        DateSelection::Range(a, b) => (a, b),
    };
    state.set_value("date", Value::Text(to_iso(start)), window, cx);
    if !value.is_single() {
        state.set_value("range_end", Value::Text(to_iso(end)), window, cx);
    }
}

fn build(
    build: Build,
    value: DateSelection,
    picker: &Entity<Picker>,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> Entity<DatePickerState> {
    let inputs = build.input.then(|| {
        (
            cx.new(|cx| InputState::new(window, cx)),
            build
                .range
                .then(|| cx.new(|cx| InputState::new(window, cx))),
        )
    });
    let state =
        cx.new(|cx| {
            let mut state = DatePickerState::new(labels(build.range, build.format), today(), cx);
            if build.range {
                state = state.range();
            }
            if let Some((earliest, latest)) = build.limits {
                state = state.years(earliest.year()..=latest.year()).disabled_dates(
                    DateMatcher::custom(move |d: &NaiveDate| *d < earliest || *d > latest),
                );
            }
            state = state.selection(value);
            // Month is the docked picker's list; the modal shows the year grid.
            state = match (build.view, build.docked) {
                (1, true) => state.view(DatePickerView::Month),
                (1, false) | (2, _) => state.view(DatePickerView::Year),
                _ => state,
            };
            if let Some((start, end)) = inputs {
                if let Some(d) = value.start() {
                    start.update(cx, |input, cx| {
                        input.set_value(build.format.format(d), window, cx)
                    });
                }
                state = state.input_mode().with_inputs(start, end, cx);
            }
            state
        });
    let picker = picker.clone();
    cx.subscribe_in(
        &state,
        window,
        move |playground: &mut PlaygroundState, _, event: &DatePickerEvent, window, cx| {
            if let DatePickerEvent::Dismiss(value) = *event {
                write_back(playground, value, window, cx);
                picker.update(cx, |picker, cx| {
                    picker.open = false;
                    cx.notify();
                });
            }
        },
    )
    .detach();
    state
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let range = config.bool("range");
    let format = DATE_FORMATS[config.choice("format")];
    let variant = config.option("variant");
    let disabled = config.bool("disabled");
    let limits = config
        .bool("limit")
        .then(|| Some((iso(&config.text("earliest"))?, iso(&config.text("latest"))?)))
        .flatten()
        .filter(|(a, b)| a <= b);
    let spec = Build {
        range,
        format,
        view: config.choice("view"),
        input: variant == "modal input",
        docked: variant == "docked",
        limits,
    };
    let value = selection(config, range);
    let picker = preview.local("picker", window, cx, |_, _| Picker {
        state: None,
        open: false,
    });
    let rebuild = preview.changed("picker-build", spec) || picker.read(cx).state.is_none();
    let state = if rebuild {
        let state = build(spec, value, &picker, window, cx);
        picker.update(cx, |picker, _| picker.state = Some(state.clone()));
        state
    } else {
        let state = picker.read(cx).state.clone().expect("built above");
        // Control edits move the picker's selection.
        if preview.changed(
            "picker-value",
            (config.text("date"), config.text("range_end")),
        ) && state.read(cx).value() != value
        {
            state.update(cx, |state, cx| state.set_value(value, cx));
        }
        state
    };
    let open = picker.read(cx).open && !disabled;

    let shown: SharedString = match value {
        DateSelection::Single(Some(d)) => format.format(d),
        DateSelection::Range(Some(a), b) => match b {
            Some(b) => format!("{} – {}", format.format(a), format.format(b)).into(),
            None => format.format(a),
        },
        _ => SharedString::default(),
    };
    if preview.changed("close-on-selection", config.bool("close")) {
        let close = config.bool("close");
        state.update(cx, |state, _| state.set_close_on_selection(close));
    }
    // Opening focuses the picker's grid, so the keyboard takes over.
    let grid = state.read(cx).focus_handle().clone();
    let toggle = {
        let picker = picker.clone();
        move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
            if disabled {
                return;
            }
            let open = picker.update(cx, |picker, cx| {
                // Opening only: a press outside, Escape, Cancel or a choice
                // closes the picker.
                picker.open = true;
                cx.notify();
                picker.open
            });
            if open {
                let grid = grid.clone();
                window.defer(cx, move |window, cx| grid.focus(window, cx));
            }
        }
    };
    let field = div()
        .id("date-field")
        .debug_selector(|| "date-field".into())
        .when(!disabled, |field| field.cursor_pointer())
        .on_click(toggle.clone())
        .child(
            TextField::specimen("date-field-text", shown)
                .variant(TextFieldVariant::Outlined)
                .label(config.text("label"))
                .float_label(open)
                .supporting_text(FORMATS[config.choice("format")])
                .disabled(disabled)
                .trailing_icon(
                    IconButton::new(
                        "date-field-open",
                        symbol("calendar_today", 24.),
                        "Open date picker",
                    )
                    .disabled(disabled)
                    .on_click(toggle),
                )
                .when(open, |field| {
                    field.preview_state(InteractionPreview::Focused)
                })
                .w(rem(312.)),
        );
    let close = {
        let picker = picker.clone();
        move |_: &mut Window, cx: &mut App| {
            picker.update(cx, |picker, cx| {
                picker.open = false;
                cx.notify();
            })
        }
    };
    let confirm = {
        let close = close.clone();
        let playground = cx.weak_entity();
        move |value: DateSelection, window: &mut Window, cx: &mut App| {
            let _ = playground.update(cx, |state, cx| write_back(state, value, window, cx));
            close(window, cx);
        }
    };
    let surface = open.then(|| {
        let picker = DatePicker::new("playground-date-picker", &state)
            .on_confirm(confirm)
            .on_cancel(close.clone());
        match variant {
            "docked" => picker.docked().into_any_element(),
            "fullscreen" => picker.full_screen().height(560.).into_any_element(),
            _ => picker.into_any_element(),
        }
    });
    let docked = variant == "docked";
    div()
        .debug_selector(|| "playground-date-picker".into())
        .flex()
        .flex_col()
        .items_start()
        .gap(rem(8.))
        .child(field)
        .when_some(surface, |column, surface| {
            let surface = div()
                .debug_selector(|| "date-surface".into())
                .child(surface);
            if docked {
                // A docked picker drops over the page below its field, as a
                // menu does, closing on a press outside it.
                let outside = close.clone();
                column.child(
                    div().relative().h_0().child(
                        deferred(
                            anchored().snap_to_window_with_margin(px(8.)).child(
                                surface
                                    .occlude()
                                    .on_mouse_down_out(move |_, window, cx| outside(window, cx)),
                            ),
                        )
                        .with_priority(1),
                    ),
                )
            } else {
                column.child(
                    PickerDialog::new("playground-date-dialog", true, surface).on_dismiss(close),
                )
            }
        })
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn iso_controls_round_trip() {
        let d = NaiveDate::from_ymd_opt(2025, 8, 17).unwrap();
        assert_eq!(iso("2025-08-17"), Some(d));
        assert_eq!(to_iso(Some(d)), "2025-08-17");
    }

    #[gpui::test]
    fn every_scenario_renders_and_the_docked_picker_selects(cx: &mut TestAppContext) {
        let (state, cx) = open(cx, &PLAYGROUND);
        every_scenario_renders(&state, cx, "playground-date-picker");
        scenario(&state, cx, "Profile form");
        let field = rect(cx, "date-field");
        cx.simulate_click(
            point(field.left() + px(40.), field.top() + px(30.)),
            Modifiers::none(),
        );
        settle(cx);
        // The docked picker opens on September 2025 with the 1st selected
        // and its grid focused; choosing a day closes it (Close on
        // selection) and writes the date back.
        assert!(cx.debug_bounds("date-surface").is_some(), "open");
        cx.simulate_keystrokes("right enter");
        settle(cx);
        assert_eq!(config(&state, cx).text("date"), "2025-09-02");
        assert!(cx.debug_bounds("date-surface").is_none(), "closed");
    }
}
