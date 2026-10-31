//! Date & time pickers: the kit's modal, input and docked date pickers with
//! their building blocks, the dial and keyboard time pickers with theirs,
//! and live pickers (modal dialogs opened from buttons, an inline docked
//! picker and an inline dial).
//!
//! The gallery supplies the English strings and formatters; the components
//! hold none.
use super::prelude::*;
use super::text_fields::symbol;
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

fn short_month(month: u32) -> &'static str {
    &MONTHS[(month as usize + 11) % 12][..3]
}

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("a valid gallery date")
}

/// English date picker strings.
fn date_labels(range: bool) -> DatePickerLabels {
    let s = SharedString::new_static;
    let headline = |d: NaiveDate| -> SharedString {
        format!(
            "{}, {} {}",
            WEEKDAYS[d.weekday().num_days_from_monday() as usize],
            short_month(d.month()),
            d.day()
        )
        .into()
    };
    let short = |d: NaiveDate| format!("{} {}", short_month(d.month()), d.day());
    DatePickerLabels {
        title: if range {
            s("Depart - Return dates")
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
        input_placeholder: s("mm/dd/yyyy"),
        invalid_input: s("Use mm/dd/yyyy"),
        weekday: Rc::new(|w: Weekday| {
            ["S", "M", "T", "W", "T", "F", "S"][w.num_days_from_sunday() as usize].into()
        }),
        month: Rc::new(|m| MONTHS[(m as usize + 11) % 12].into()),
        month_short: Rc::new(|m| short_month(m).into()),
        month_year: Rc::new(|y, m| format!("{} {y}", MONTHS[(m as usize + 11) % 12]).into()),
        headline: Rc::new(headline),
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
        format_input: Rc::new(|d: NaiveDate| {
            format!("{:02}/{:02}/{}", d.month(), d.day(), d.year()).into()
        }),
        parse_input: Rc::new(|text| {
            let mut parts = text.split('/').map(|p| p.trim().parse::<u32>().ok());
            let (m, d, y) = (parts.next()??, parts.next()??, parts.next()??);
            if parts.next().is_some() || y < 1000 {
                return None;
            }
            NaiveDate::from_ymd_opt(y as i32, m, d)
        }),
    }
}

/// English time picker strings.
fn time_labels() -> TimePickerLabels {
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
        switch_to_input: s("Switch to text input"),
        switch_to_dial: s("Switch to clock"),
        number: Rc::new(|n| n.to_string().into()),
        two_digits: Rc::new(|n| format!("{n:02}").into()),
        separator: s(":"),
    }
}

/// The kit's fixed "today" (August 5, 2025) for specimens.
fn kit_today() -> NaiveDate {
    date(2025, 8, 5)
}

/// The row of the live pickers, which open from either component's page.
const LIVE: usize = 6;

/// The date picker's rows on its Specs tab.
pub fn date_picker_rows() -> Vec<usize> {
    vec![0, 1, 2, 3, LIVE]
}
/// The time picker's rows on its Specs tab.
pub fn time_picker_rows() -> Vec<usize> {
    vec![4, 5, LIVE]
}

pub struct Page {
    page: PageCx,
    modal_day: Entity<DatePickerState>,
    modal_year: Entity<DatePickerState>,
    full_screen: Entity<DatePickerState>,
    input_single: Entity<DatePickerState>,
    input_range: Entity<DatePickerState>,
    docked: [Entity<DatePickerState>; 3],
    dials: Vec<Entity<TimePickerState>>,
    live_date: Entity<DatePickerState>,
    live_range: Entity<DatePickerState>,
    live_docked: Entity<DatePickerState>,
    live_time: Entity<TimePickerState>,
    live_inline_time: Entity<TimePickerState>,
    open: Option<LiveDialog>,
    result: Option<SharedString>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LiveDialog {
    Date,
    Range,
    Time,
}

const STATES: [InteractionPreview; 5] = InteractionPreview::ALL;

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "date-time-pickers",
        title: "Date & time pickers",
        description: "Date pickers let users select a date, or a range of dates. Time pickers help users select and set a specific time.",
        guideline: Some("https://m3.material.io/components/date-pickers/overview"),
    };

    fn new(page: PageCx, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let today = kit_today();
        let aug17 = date(2025, 8, 17);
        let mut date_state = |range: bool, f: &dyn Fn(DatePickerState) -> DatePickerState| {
            cx.new(|cx| {
                let state = DatePickerState::new(date_labels(range), today, cx);
                f(if range { state.range() } else { state })
            })
        };
        let modal_day = date_state(false, &|s| s.selection(aug17));
        let modal_year = date_state(false, &|s| s.selection(aug17).view(DatePickerView::Year));
        let full_screen = date_state(true, &|s| s.selection((aug17, date(2025, 8, 23))));
        let input_single = date_state(false, &|s| s.input_mode());
        let input_range = date_state(true, &|s| s.input_mode());
        let docked = [
            date_state(false, &|s| s.selection(aug17)),
            date_state(false, &|s| s.selection(aug17).view(DatePickerView::Month)),
            date_state(false, &|s| s.selection(aug17).view(DatePickerView::Year)),
        ];
        let seven = NaiveTime::from_hms_opt(7, 0, 0).expect("valid time");
        let twenty = NaiveTime::from_hms_opt(20, 0, 0).expect("valid time");
        let mut dials = Vec::new();
        for (time, format, input) in [
            (seven, TimeFormat::TwelveHour, false),
            (seven, TimeFormat::TwelveHour, false),
            (twenty, TimeFormat::TwelveHour, true),
            (twenty, TimeFormat::TwentyFourHour, false),
            (twenty, TimeFormat::TwentyFourHour, false),
            (twenty, TimeFormat::TwentyFourHour, true),
        ] {
            dials.push(cx.new(|cx| {
                let state = TimePickerState::new(time_labels(), time, window, cx)
                    .format(format, window, cx);
                if input { state.input_mode() } else { state }
            }));
        }
        let real_today = chrono_today();
        let start = cx.new(|cx| InputState::new(window, cx));
        let end = cx.new(|cx| InputState::new(window, cx));
        let single = cx.new(|cx| InputState::new(window, cx));
        let live_date = cx.new(|cx| {
            DatePickerState::new(date_labels(false), real_today, cx)
                .disabled_dates(DateMatcher::custom(|d: &NaiveDate| {
                    d.weekday() == Weekday::Sun
                }))
                .with_inputs(single, None, cx)
        });
        let live_range = cx.new(|cx| {
            DatePickerState::new(date_labels(true), real_today, cx)
                .range()
                .with_inputs(start, Some(end), cx)
        });
        let live_docked = cx.new(|cx| DatePickerState::new(date_labels(false), real_today, cx));
        let now = NaiveTime::from_hms_opt(9, 41, 0).expect("valid time");
        let live_time = cx.new(|cx| TimePickerState::new(time_labels(), now, window, cx));
        let live_inline_time = cx.new(|cx| {
            TimePickerState::new(time_labels(), now, window, cx).format(
                TimeFormat::TwentyFourHour,
                window,
                cx,
            )
        });
        for state in [&live_date, &live_range, &live_docked] {
            cx.observe(state, |_, _, cx| cx.notify()).detach();
        }
        for state in [&live_time, &live_inline_time] {
            cx.observe(state, |_, _, cx| cx.notify()).detach();
        }
        Self {
            page,
            modal_day,
            modal_year,
            full_screen,
            input_single,
            input_range,
            docked,
            dials,
            live_date,
            live_range,
            live_docked,
            live_time,
            live_inline_time,
            open: None,
            result: None,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        LIVE + 1
    }

    fn interactive(&self, row: usize) -> bool {
        row == LIVE
    }

    fn render_row(
        &mut self,
        row: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match row {
            0 => board("modal-date")
                .caption("Modal Date Picker")
                .columns(3)
                .column_labels(["Day", "Year", "Full-screen (range)"])
                .cells([
                    DatePicker::new("modal-day", &self.modal_day)
                        .specimen()
                        .into_any_element(),
                    DatePicker::new("modal-year", &self.modal_year)
                        .specimen()
                        .into_any_element(),
                    DatePicker::new("full-screen", &self.full_screen)
                        .full_screen()
                        .specimen()
                        .into_any_element(),
                ])
                .into_any_element(),
            1 => board("input-date")
                .caption("Input Date Picker")
                .columns(2)
                .column_labels(["Single input", "Range"])
                .cells([
                    DatePicker::new("input-single", &self.input_single)
                        .specimen()
                        .show_clear(false)
                        .into_any_element(),
                    DatePicker::new("input-range", &self.input_range)
                        .specimen()
                        .show_clear(false)
                        .into_any_element(),
                ])
                .into_any_element(),
            2 => board("docked-date")
                .caption("Docked Input Date Picker [desktop]")
                .columns(3)
                .column_labels(["Day", "Month", "Year"])
                .cells(
                    self.docked
                        .iter()
                        .enumerate()
                        .map(|(i, state)| docked_specimen(i, state).into_any_element()),
                )
                .into_any_element(),
            3 => calendar_blocks(),
            4 => board("dial-time")
                .caption("Time picker")
                .columns(3)
                .column_labels(["Vertical", "Horizontal", "Keyboard input"])
                .row_labels(["12 hour", "24 hour"])
                .cells(self.dials.iter().enumerate().map(|(i, state)| {
                    let orientation = if i % 3 == 1 {
                        TimeOrientation::Horizontal
                    } else {
                        TimeOrientation::Vertical
                    };
                    TimePicker::new(("time-specimen", i), state)
                        .orientation(orientation)
                        .specimen()
                        .into_any_element()
                }))
                .into_any_element(),
            5 => time_blocks(),
            _ => self.live(window, cx),
        }
    }

    fn apply_capture(&mut self, state: CaptureState, cx: &mut Context<Self>) {
        if state.active {
            self.open = Some(LiveDialog::Date);
            cx.notify();
        }
    }
}

fn chrono_today() -> NaiveDate {
    gpui_kit_material::components::today()
}

/// A docked picker below its text field, as the kit lays it out.
fn docked_specimen(index: usize, state: &Entity<DatePickerState>) -> impl IntoElement {
    let field = TextField::specimen(("docked-field", index), "08/17/2025")
        .outlined()
        .label("Date")
        .supporting_text("MM/DD/YYYY")
        .trailing_icon(symbol("calendar_today", false).size(rem(24.)))
        .preview_state(InteractionPreview::Focused)
        .w(rem(312.));
    div().flex().flex_col().gap(rem(8.)).child(field).child(
        DatePicker::new(("docked", index), state)
            .docked()
            .specimen(),
    )
}

/// The calendar cell, year and menu button building blocks in every state.
fn calendar_blocks() -> AnyElement {
    let columns = [
        "Default (day)",
        "Today",
        "Selected",
        "Range (Middle)",
        "Prev/Next",
        "Year",
        "Year selected",
        "Menu button",
    ];
    let cells = STATES.into_iter().enumerate().flat_map(|(r, state)| {
        let day =
            move |c: usize| CalendarDay::new(("block-day", r * 10 + c), "00").preview_state(state);
        [
            day(0).into_any_element(),
            day(1).today(true).into_any_element(),
            day(2).selected(true).into_any_element(),
            day(3).range(DayRange::Middle).into_any_element(),
            day(4).adjacent(true).into_any_element(),
            YearChip::new(("block-year", r), "2025")
                .preview_state(state)
                .into_any_element(),
            YearChip::new(("block-year-selected", r), "2025")
                .selected(true)
                .preview_state(state)
                .into_any_element(),
            PickerMenuButton::new(("block-menu", r), "Label")
                .preview_state(state)
                .into_any_element(),
        ]
    });
    board("calendar-blocks")
        .caption("Building Blocks")
        .columns(columns.len())
        .column_labels(columns)
        .row_labels(STATES.map(InteractionPreview::name))
        .cells(cells)
        .into_any_element()
}

/// Time picker building blocks: selectors, period selectors and faces.
fn time_blocks() -> AnyElement {
    let labels = Rc::new(time_labels());
    let seven = NaiveTime::from_hms_opt(7, 0, 0).expect("valid time");
    let twenty = NaiveTime::from_hms_opt(20, 0, 0).expect("valid time");
    let period_states = [
        InteractionPreview::Enabled,
        InteractionPreview::Hovered,
        InteractionPreview::Focused,
        InteractionPreview::Pressed,
    ];
    let periods = |horizontal: bool| {
        div()
            .flex()
            .flex_col()
            .gap(rem(12.))
            .children([false, true].map(|pm| {
                div()
                    .flex()
                    .gap(rem(12.))
                    .children(period_states.map(|state| {
                        PeriodSelector::new(
                            SharedString::from(format!("period-{horizontal}-{pm}-{state:?}")),
                            "AM",
                            "PM",
                        )
                        .pm(pm)
                        .horizontal(horizontal)
                        .preview_state(pm, state)
                    }))
            }))
    };
    let selectors = div()
        .flex()
        .flex_col()
        .gap(rem(12.))
        .children([false, true].map(|selected| {
            div()
                .flex()
                .gap(rem(12.))
                .children(period_states.map(|state| {
                    TimeSelector::new(
                        SharedString::from(format!("selector-{selected}-{state:?}")),
                        "00",
                        "Hour",
                    )
                    .selected(selected)
                    .preview_state(state)
                }))
        }));
    board("time-blocks")
        .caption("Building Blocks")
        .columns(3)
        .column_labels(["Clock face", "Time selector", "Period selector"])
        .cells([
            ClockDial::specimen(
                "face-12",
                labels.clone(),
                seven,
                TimeUnit::Hour,
                TimeFormat::TwelveHour,
            )
            .into_any_element(),
            selectors.into_any_element(),
            periods(false).into_any_element(),
            ClockDial::specimen(
                "face-24",
                labels.clone(),
                twenty,
                TimeUnit::Hour,
                TimeFormat::TwentyFourHour,
            )
            .into_any_element(),
            ClockDial::specimen(
                "face-minutes",
                labels,
                seven.with_minute(7).unwrap_or(seven),
                TimeUnit::Minute,
                TimeFormat::TwelveHour,
            )
            .into_any_element(),
            periods(true).into_any_element(),
        ])
        .into_any_element()
}

impl Page {
    fn live(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let list = self.page.rows.list().clone();
        let open = |which: LiveDialog| {
            cx.listener(move |this: &mut Self, _: &ClickEvent, window, cx| {
                this.open = Some(which);
                let focus = match which {
                    LiveDialog::Date => this.live_date.read(cx).focus_handle().clone(),
                    LiveDialog::Range => this.live_range.read(cx).focus_handle().clone(),
                    LiveDialog::Time => this.live_time.read(cx).dial_focus().clone(),
                };
                focus.focus(window, cx);
                cx.notify();
            })
        };
        let close = cx.listener(|this: &mut Self, _: &(), _, cx| {
            this.open = None;
            cx.notify();
        });
        let close = Rc::new(close);
        let dismiss = {
            let close = close.clone();
            move |window: &mut Window, cx: &mut App| close(&(), window, cx)
        };
        let page = cx.entity();
        let confirm_date = {
            let page = page.clone();
            move |value: DateSelection, _: &mut Window, cx: &mut App| {
                page.update(cx, |this, cx| {
                    let labels = date_labels(!value.is_single());
                    this.result = Some(match value {
                        DateSelection::Single(Some(d)) => (labels.headline)(d),
                        DateSelection::Single(None) => "No date".into(),
                        DateSelection::Range(a, b) => (labels.range_headline)(a, b),
                    });
                    this.open = None;
                    cx.notify();
                })
            }
        };
        let confirm_time = {
            let page = page.clone();
            move |time: NaiveTime, _: &mut Window, cx: &mut App| {
                page.update(cx, |this, cx| {
                    this.result = Some(time.format("%H:%M").to_string().into());
                    this.open = None;
                    cx.notify();
                })
            }
        };
        let dialog = match self.open {
            Some(LiveDialog::Date) => Some(
                DatePicker::new("live-modal", &self.live_date)
                    .on_confirm(confirm_date.clone())
                    .on_cancel(dismiss.clone())
                    .into_any_element(),
            ),
            Some(LiveDialog::Range) => Some(
                DatePicker::new("live-range", &self.live_range)
                    .full_screen()
                    .height(640.)
                    .on_confirm(confirm_date.clone())
                    .on_cancel(dismiss.clone())
                    .into_any_element(),
            ),
            Some(LiveDialog::Time) => Some(
                TimePicker::new("live-time", &self.live_time)
                    .on_confirm(confirm_time)
                    .on_cancel(dismiss.clone())
                    .into_any_element(),
            ),
            None => None,
        };
        let button = |id: &'static str, label: &'static str, which: LiveDialog| {
            material_button(id, ButtonVariant::Tonal)
                .child(label)
                .on_click(open(which))
                .reveal(SharedString::from(format!("reveal-{id}")), &list)
        };
        let result = self
            .result
            .clone()
            .unwrap_or_else(|| "Nothing chosen yet".into());
        spec::demo(
            "live-pickers",
            "Pickers in use",
            div()
                .flex()
                .flex_col()
                .gap(rem(24.))
                .child(
                    spec::demo_row()
                        .child(button("open-date", "Pick a date", LiveDialog::Date))
                        .child(button("open-range", "Pick a range", LiveDialog::Range))
                        .child(button("open-time", "Pick a time", LiveDialog::Time))
                        .child(
                            accessible_text("picker-result", result)
                                .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                        ),
                )
                .child(
                    spec::demo_row()
                        .items_start()
                        .child(DatePicker::new("live-docked", &self.live_docked).docked())
                        .child(TimePicker::new("live-inline-time", &self.live_inline_time)),
                )
                .when_some(dialog, |column, dialog| {
                    column.child(PickerDialog::new("live-dialog", true, dialog).on_dismiss(dismiss))
                }),
        )
        .into_any_element()
    }
}
