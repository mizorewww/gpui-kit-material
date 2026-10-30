use super::*;
use crate::components::date_parts::{day_paint, month_weeks};
use core::prelude::v1::test;

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

pub(crate) fn labels() -> DatePickerLabels {
    let s = SharedString::new_static;
    DatePickerLabels {
        title: s("Select date"),
        no_selection: s("Selected date"),
        input_headline: s("Enter date"),
        confirm: s("OK"),
        cancel: s("Cancel"),
        clear: s("Clear"),
        save: s("Save"),
        close: s("Close"),
        previous_month: s("Previous month"),
        next_month: s("Next month"),
        previous_year: s("Previous year"),
        next_year: s("Next year"),
        switch_to_input: s("Switch to input"),
        switch_to_calendar: s("Switch to calendar"),
        choose_year: s("Choose year"),
        choose_month: s("Choose month"),
        date_field: s("Date"),
        start_field: s("Start date"),
        end_field: s("End date"),
        input_placeholder: s("mm/dd/yyyy"),
        invalid_input: s("Invalid"),
        weekday: Rc::new(|w: Weekday| w.to_string()[..1].to_string().into()),
        month: Rc::new(|m| m.to_string().into()),
        month_short: Rc::new(|m| m.to_string().into()),
        month_year: Rc::new(|y, m| format!("{m}/{y}").into()),
        headline: Rc::new(|d: NaiveDate| d.to_string().into()),
        range_headline: Rc::new(|a, b| format!("{a:?}-{b:?}").into()),
        accessible_date: Rc::new(|d: NaiveDate| d.to_string().into()),
        number: Rc::new(|n| n.to_string().into()),
        format_input: Rc::new(|d: NaiveDate| d.format("%m/%d/%Y").to_string().into()),
        parse_input: Rc::new(|t| NaiveDate::parse_from_str(t, "%m/%d/%Y").ok()),
    }
}

#[test]
fn month_grid_has_six_weeks_starting_on_the_first_weekday() {
    let weeks = month_weeks(2025, 8, Weekday::Sun);
    assert_eq!(weeks.len(), 6);
    // August 2025 starts on a Friday: the grid opens on Sunday, July 27.
    assert_eq!(weeks[0][0], d(2025, 7, 27));
    assert_eq!(weeks[0][5], d(2025, 8, 1));
    let monday = month_weeks(2025, 8, Weekday::Mon);
    assert_eq!(monday[0][0], d(2025, 7, 28));
    assert_eq!(days_in_month(2024, 2), 29);
    assert_eq!(days_in_month(2025, 12), 31);
    assert_eq!(first_of_month(2025, 13), d(2026, 1, 1));
}

#[test]
fn range_selection_follows_base_calendar_rules() {
    let empty = DateSelection::Range(None, None);
    let start = next_selection(empty, d(2025, 8, 17));
    assert_eq!(start, DateSelection::Range(Some(d(2025, 8, 17)), None));
    let done = next_selection(start, d(2025, 8, 23));
    assert_eq!(
        done,
        DateSelection::Range(Some(d(2025, 8, 17)), Some(d(2025, 8, 23)))
    );
    // Before the start restarts; after completion restarts.
    assert_eq!(
        next_selection(start, d(2025, 8, 1)),
        DateSelection::Range(Some(d(2025, 8, 1)), None)
    );
    assert_eq!(
        next_selection(done, d(2025, 9, 2)),
        DateSelection::Range(Some(d(2025, 9, 2)), None)
    );
    assert_eq!(
        next_selection(DateSelection::Single(None), d(2025, 1, 1)),
        DateSelection::Single(Some(d(2025, 1, 1)))
    );
}

#[test]
fn keyboard_steps_move_by_day_week_month_and_year() {
    use CursorStep::*;
    let c = d(2025, 8, 17); // a Sunday
    let day = |s| cursor_target(DatePickerView::Day, c, s, Weekday::Sun);
    assert_eq!(day(Left), d(2025, 8, 16));
    assert_eq!(day(Right), d(2025, 8, 18));
    assert_eq!(day(Up), d(2025, 8, 10));
    assert_eq!(day(Down), d(2025, 8, 24));
    assert_eq!(day(PreviousMonth), d(2025, 7, 17));
    assert_eq!(day(NextYear), d(2026, 8, 17));
    assert_eq!(day(WeekStart), c);
    assert_eq!(day(WeekEnd), d(2025, 8, 23));
    assert_eq!(
        cursor_target(DatePickerView::Day, d(2025, 1, 31), NextMonth, Weekday::Sun),
        d(2025, 2, 28),
        "month steps clamp to shorter months"
    );
    let year = |s| cursor_target(DatePickerView::Year, c, s, Weekday::Sun);
    assert_eq!(year(Right).year(), 2026);
    assert_eq!(year(Down).year(), 2028);
    let month = |s| cursor_target(DatePickerView::Month, c, s, Weekday::Sun);
    assert_eq!(month(Down).month(), 9);
    assert_eq!(month(WeekEnd).month(), 12);
    assert_eq!(
        cursor_target(DatePickerView::Month, d(2025, 12, 1), Down, Weekday::Sun),
        d(2025, 12, 1),
        "the month list does not wrap into another year"
    );
}

#[test]
fn day_cells_use_the_kit_roles() {
    let selected = day_paint(1., false, false, false);
    assert_eq!(selected.fill, color(ColorRole::PRIMARY));
    assert_eq!(selected.ink, color(ColorRole::ON_PRIMARY));
    assert!(selected.outline.is_none());
    let today = day_paint(0., true, false, false);
    assert_eq!(today.ink, color(ColorRole::PRIMARY));
    assert_eq!(today.outline, Some(color(ColorRole::PRIMARY)));
    assert_eq!(today.fill.a, 0.);
    let range = day_paint(0., false, true, false);
    assert_eq!(range.ink, color(ColorRole::ON_SECONDARY_CONTAINER));
    let adjacent = day_paint(0., false, false, true);
    assert_eq!(
        adjacent.ink,
        crate::tokens::state::disabled(color(ColorRole::ON_SURFACE))
    );
}

struct Harness(Entity<DatePickerState>);
impl Render for Harness {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div().size_full().child(DatePicker::new("picker", &self.0))
    }
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

#[gpui::test]
fn the_grid_is_keyboard_operable(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|_, cx| {
        let state = cx.new(|cx| {
            DatePickerState::new(labels(), d(2025, 8, 5), cx)
                .disabled_dates(DateMatcher::custom(|date: &NaiveDate| date.day() == 7))
        });
        Harness(state)
    });
    let state = view.read_with(cx, |h, _| h.0.clone());
    draw(cx);
    cx.update(|window, cx| state.read(cx).focus_handle().clone().focus(window, cx));
    draw(cx);
    cx.simulate_keystrokes("right enter");
    assert_eq!(
        state.read_with(cx, |s, _| s.value()),
        DateSelection::Single(Some(d(2025, 8, 6)))
    );
    cx.simulate_keystrokes("right enter");
    assert_eq!(
        state.read_with(cx, |s, _| s.value()),
        DateSelection::Single(Some(d(2025, 8, 6))),
        "disabled dates cannot be selected"
    );
    cx.simulate_keystrokes("pagedown");
    assert_eq!(state.read_with(cx, |s, _| s.displayed_month()), (2025, 9));
    cx.simulate_keystrokes("shift-pageup");
    assert_eq!(state.read_with(cx, |s, _| s.displayed_month()), (2024, 9));
    assert_eq!(state.read_with(cx, |s, _| s.cursor()), d(2024, 9, 7));
}

#[gpui::test]
fn typed_dates_parse_into_the_selection(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
    });
    let (view, cx) = cx.add_window_view(|window, cx| {
        let input = cx.new(|cx| InputState::new(window, cx));
        let state = cx.new(|cx| {
            DatePickerState::new(labels(), d(2025, 8, 5), cx)
                .input_mode()
                .with_inputs(input, None, cx)
        });
        Harness(state)
    });
    let state = view.read_with(cx, |h, _| h.0.clone());
    let input = state.read_with(cx, |s, _| s.inputs.clone().unwrap().0);
    cx.update(|window, cx| {
        input.update(cx, |i, cx| i.set_value("08/17/2025", window, cx));
        state.update(cx, |s, cx| s.parse_inputs(cx));
    });
    assert_eq!(
        state.read_with(cx, |s, _| s.value()),
        DateSelection::Single(Some(d(2025, 8, 17)))
    );
    cx.update(|window, cx| {
        input.update(cx, |i, cx| i.set_value("17/08/2025", window, cx));
        state.update(cx, |s, cx| s.parse_inputs(cx));
    });
    assert_eq!(state.read_with(cx, |s, _| s.invalid), (true, false));
    assert_eq!(
        state.read_with(cx, |s, _| s.value()),
        DateSelection::Single(Some(d(2025, 8, 17))),
        "invalid text keeps the last valid date"
    );
}

#[test]
fn months_stay_inside_the_offered_years() {
    let years = 2020..=2022;
    assert_eq!(clamp_month(&years, 2019, 5), (2020, 1));
    assert_eq!(clamp_month(&years, 2023, 1), (2022, 12));
    assert_eq!(clamp_month(&years, 2021, 13), (2022, 1), "normalized");
    assert!(month_in_range(&years, 2022, 12));
    assert!(!month_in_range(&years, 2023, 1));
    // The full-screen list has 36 rows; a later year reveals the last.
    assert_eq!(month_list_index(&years, 2021, 2), 14);
    assert_eq!(month_list_index(&years, 2030, 6), 35);
}

#[gpui::test]
fn chevrons_and_set_value_respect_the_years(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_base::init(cx);
        crate::init(cx);
        cx.set_reduce_motion(true);
    });
    let state =
        cx.new(|cx| DatePickerState::new(labels(), d(2025, 1, 5), cx).years(2025..=2025));
    state.update(cx, |s, cx| s.previous_month(cx));
    assert_eq!(state.read_with(cx, |s, _| s.displayed_month()), (2025, 1));
    state.update(cx, |s, cx| s.previous_year(cx));
    assert_eq!(state.read_with(cx, |s, _| s.displayed_month()), (2025, 1));
    state.update(cx, |s, cx| s.show_month(2025, 12, cx));
    state.update(cx, |s, cx| s.next_month(cx));
    assert_eq!(state.read_with(cx, |s, _| s.displayed_month()), (2025, 12));
    // A selection outside the years stays, but the grid shows their end.
    state.update(cx, |s, cx| {
        s.set_value(DateSelection::Single(Some(d(2030, 6, 1))), cx)
    });
    assert_eq!(
        state.read_with(cx, |s, _| s.value()),
        DateSelection::Single(Some(d(2030, 6, 1)))
    );
    assert_eq!(state.read_with(cx, |s, _| s.displayed_month()), (2025, 12));
    // The grid renders there without indexing past its list.
    let (_, cx) = cx.add_window_view(|_, _| Harness(state.clone()));
    draw(cx);
}
