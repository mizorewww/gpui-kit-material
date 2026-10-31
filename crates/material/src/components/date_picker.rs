//! M3 date pickers: docked, modal, modal input and full-screen, for a single
//! date or a date range.
//!
//! Reference: <https://m3.material.io/components/date-pickers/specs> and the
//! Material 3 Design Kit "Date & time pickers" page (inspected tokens:
//! surface-container-high containers with 28dp corners for the modal and
//! 16dp for the docked menu; a 120dp modal header with label-large
//! supporting text and a headline-large date over an outline-variant
//! divider; a 56dp month row with the month/year menu button and
//! previous/next icon buttons; a 48dp weekday row of body-large letters; a
//! 60dp action row of text buttons; the 128dp input header with the 52dp
//! wide "today" icon button and outlined text fields; the docked menu's 64dp
//! month/year selection row and 48dp month and year lists).
//!
//! Behavior: the value types and disabled-date matcher are GPUI Base's
//! calendar types ([`DateSelection`] = `gpui_base::Date`,
//! [`DateMatcher`] = `gpui_base::Matcher`) and range selection follows
//! `gpui_base::CalendarState::select_date`. Base's calendar has no keyboard
//! grid and fixes "today" to the system clock, so the picker keeps its own
//! state entity: a single tab stop on the day grid (or year grid/list) with
//! an active-descendant cursor that the arrow keys move by a day or week,
//! PageUp/PageDown by a month, Shift+PageUp/PageDown by a year and
//! Home/End to the week's ends; Enter or Space selects. Typed dates use
//! Kit's input through [`TextField`]. Every string — month and weekday
//! names, headlines, button labels and the input format — comes from the
//! caller's [`DatePickerLabels`].
//!
//! Motion: a month change slides the outgoing grid out and the incoming
//! grid in by the grid's width on the default spatial spring (both fade on
//! the way); selection fills crossfade on the fast effects spring; the menu
//! arrow turns on the fast spatial spring. Reduced motion shows the new
//! month at once.
use super::date_parts::{
    Activate, CalendarDay, DAY_CELL, DayRange, GRID_WEEKS, LIST_ITEM_HEIGHT, PickerListItem,
    PickerMenuButton, YearChip, add_months, first_of_month, month_weeks, week_days,
};
use super::text_field::type_style;
use super::{
    ButtonVariant, IconButton, IconButtonWidth, InteractionPreview, TextField, material_button,
};
use crate::{
    color::ColorRole,
    icons::{Icon, Symbol},
    motion::{self, MotionToken},
    theme::color,
    tokens::{
        elevation,
        typography::{TypeRole, rem},
    },
};
use chrono::{Datelike, Days, NaiveDate, Weekday};
use gpui::{prelude::*, *};
use gpui_base::input::{InputEvent, InputState};
use std::{ops::RangeInclusive, rc::Rc, time::Duration};

pub use gpui_base::{Date as DateSelection, Matcher as DateMatcher};

actions!(
    material_date_picker,
    [
        CursorLeft,
        CursorRight,
        CursorUp,
        CursorDown,
        CursorPreviousMonth,
        CursorNextMonth,
        CursorPreviousYear,
        CursorNextYear,
        CursorWeekStart,
        CursorWeekEnd,
        SelectCursor
    ]
);

const CONTEXT: &str = "MaterialDatePicker";

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("left", CursorLeft, Some(CONTEXT)),
        KeyBinding::new("right", CursorRight, Some(CONTEXT)),
        KeyBinding::new("up", CursorUp, Some(CONTEXT)),
        KeyBinding::new("down", CursorDown, Some(CONTEXT)),
        KeyBinding::new("pageup", CursorPreviousMonth, Some(CONTEXT)),
        KeyBinding::new("pagedown", CursorNextMonth, Some(CONTEXT)),
        KeyBinding::new("shift-pageup", CursorPreviousYear, Some(CONTEXT)),
        KeyBinding::new("shift-pagedown", CursorNextYear, Some(CONTEXT)),
        KeyBinding::new("home", CursorWeekStart, Some(CONTEXT)),
        KeyBinding::new("end", CursorWeekEnd, Some(CONTEXT)),
        KeyBinding::new("enter", SelectCursor, Some(CONTEXT)),
        KeyBinding::new("space", SelectCursor, Some(CONTEXT)),
    ]);
}

type Format<T> = Rc<dyn Fn(T) -> SharedString>;
type Parse = Rc<dyn Fn(&str) -> Option<NaiveDate>>;

/// The numeric date orders md3.io offers for text entry. The order is data; the
/// placeholder naming it stays a caller-supplied label.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DateFormat {
    /// `MM/DD/YYYY`.
    #[default]
    MonthDayYear,
    /// `DD/MM/YYYY`.
    DayMonthYear,
    /// `YYYY MM DD`.
    YearMonthDay,
}

impl DateFormat {
    pub const ALL: [Self; 3] = [Self::MonthDayYear, Self::DayMonthYear, Self::YearMonthDay];
    /// `date` as this format writes it.
    pub fn format(self, date: NaiveDate) -> SharedString {
        let (y, m, d) = (date.year(), date.month(), date.day());
        match self {
            Self::MonthDayYear => format!("{m:02}/{d:02}/{y}"),
            Self::DayMonthYear => format!("{d:02}/{m:02}/{y}"),
            Self::YearMonthDay => format!("{y} {m:02} {d:02}"),
        }
        .into()
    }
    /// The date `text` writes in this format, if it names a real one.
    pub fn parse(self, text: &str) -> Option<NaiveDate> {
        let separator = if self == Self::YearMonthDay { ' ' } else { '/' };
        let parts: Vec<&str> = text.trim().split(separator).map(str::trim).collect();
        let [a, b, c] = parts[..] else {
            return None;
        };
        let (y, m, d) = match self {
            Self::MonthDayYear => (c, a, b),
            Self::DayMonthYear => (c, b, a),
            Self::YearMonthDay => (a, b, c),
        };
        // Fewer digits are a year still being typed, not the year 2 or 25.
        if y.len() < 4 {
            return None;
        }
        NaiveDate::from_ymd_opt(y.parse().ok()?, m.parse().ok()?, d.parse().ok()?)
    }
}

/// Every caller-supplied string and formatter of a date picker. Nothing
/// user-facing is hard-coded in the component; galleries and apps build
/// this from their locale.
#[derive(Clone)]
pub struct DatePickerLabels {
    /// The modal header's supporting text, e.g. "Select date".
    pub title: SharedString,
    /// The headline while nothing is selected in calendar mode.
    pub no_selection: SharedString,
    /// The input-mode headline, e.g. "Enter date" (or "Enter dates").
    pub input_headline: SharedString,
    pub confirm: SharedString,
    pub cancel: SharedString,
    pub clear: SharedString,
    /// The full-screen picker's confirm action, e.g. "Save".
    pub save: SharedString,
    /// Accessible names of icon-only controls.
    pub close: SharedString,
    pub previous_month: SharedString,
    pub next_month: SharedString,
    pub previous_year: SharedString,
    pub next_year: SharedString,
    pub switch_to_input: SharedString,
    pub switch_to_calendar: SharedString,
    /// Accessible name of the year grid / month and year lists.
    pub choose_year: SharedString,
    pub choose_month: SharedString,
    /// Input-mode field labels and the format shown as placeholder.
    pub date_field: SharedString,
    pub start_field: SharedString,
    pub end_field: SharedString,
    pub input_placeholder: SharedString,
    /// Shown under a field whose text does not parse.
    pub invalid_input: SharedString,
    /// A narrow weekday letter for the weekday row ("S", "M", …).
    pub weekday: Format<Weekday>,
    /// A full month name for the docked month list (1-based month).
    pub month: Format<u32>,
    /// A short month name for the docked month menu button.
    pub month_short: Format<u32>,
    /// "August 2025": the modal month button and full-screen month headers.
    pub month_year: Rc<dyn Fn(i32, u32) -> SharedString>,
    /// The single-date headline, e.g. "Mon, Aug 17".
    pub headline: Format<NaiveDate>,
    /// The range headline, e.g. "Aug 17 – Aug 23" (either end may be open).
    pub range_headline: Rc<dyn Fn(Option<NaiveDate>, Option<NaiveDate>) -> SharedString>,
    /// The full date announced for a day cell.
    pub accessible_date: Format<NaiveDate>,
    /// A day-of-month or year number.
    pub number: Format<i32>,
    /// The text-input representation of a date, and its parser.
    pub format_input: Format<NaiveDate>,
    pub parse_input: Parse,
}

impl DatePickerLabels {
    /// Format and parse text entry in `format`, replacing
    /// [`Self::format_input`] and [`Self::parse_input`].
    pub fn date_format(mut self, format: DateFormat) -> Self {
        self.format_input = Rc::new(move |date| format.format(date));
        self.parse_input = Rc::new(move |text| format.parse(text));
        self
    }
}

/// Calendar or text entry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DateInputMode {
    #[default]
    Calendar,
    Input,
}

/// What the picker body shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DatePickerView {
    /// The day grid.
    #[default]
    Day,
    /// The modal year grid, or the docked year list.
    Year,
    /// The docked month list.
    Month,
}

/// The M3 date picker presentations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DatePickerVariant {
    /// A dialog with a header, used on mobile and for text input.
    #[default]
    Modal,
    /// A menu below a text field, used on desktop.
    Docked,
    /// A full-screen dialog with a scrolling month list, for ranges.
    FullScreen,
}

/// A month transition in flight: the month it leaves and its direction.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Slide {
    from: (i32, u32),
    direction: f32,
    generation: u64,
}

/// A date picker's state: selection, displayed month, view, mode and the
/// keyboard cursor. Create it once per picker with `cx.new` and hand it to
/// [`DatePicker`].
pub struct DatePickerState {
    focus: FocusHandle,
    labels: Rc<DatePickerLabels>,
    selection: DateSelection,
    today: NaiveDate,
    year: i32,
    month: u32,
    cursor: NaiveDate,
    view: DatePickerView,
    mode: DateInputMode,
    disabled: Option<Rc<DateMatcher>>,
    years: RangeInclusive<i32>,
    first_day: Weekday,
    slide: Option<Slide>,
    generation: u64,
    year_scroll: UniformListScrollHandle,
    list_scroll: UniformListScrollHandle,
    month_scroll: UniformListScrollHandle,
    inputs: Option<(Entity<InputState>, Option<Entity<InputState>>)>,
    invalid: (bool, bool),
    /// The inputs have yet to show the selection; a draw has the window to.
    inputs_stale: bool,
    close_on_selection: bool,
    _subscriptions: Vec<Subscription>,
}

/// Events a picker emits as the person edits it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatePickerEvent {
    /// The selection changed (possibly to an incomplete range).
    Change(DateSelection),
    /// With [`DatePickerState::close_on_selection`], the choice is complete
    /// and the picker asks to close. Emitted after the matching `Change`.
    Dismiss(DateSelection),
}

impl EventEmitter<DatePickerEvent> for DatePickerState {}

impl DatePickerState {
    /// A single-date picker showing `today`'s month, with nothing selected.
    pub fn new(labels: DatePickerLabels, today: NaiveDate, cx: &mut Context<Self>) -> Self {
        let state = Self {
            focus: cx.focus_handle().tab_stop(true),
            labels: Rc::new(labels),
            selection: DateSelection::Single(None),
            today,
            year: today.year(),
            month: today.month(),
            cursor: today,
            view: DatePickerView::Day,
            mode: DateInputMode::Calendar,
            disabled: None,
            years: today.year() - 100..=today.year() + 100,
            first_day: Weekday::Sun,
            slide: None,
            generation: 0,
            year_scroll: UniformListScrollHandle::new(),
            list_scroll: UniformListScrollHandle::new(),
            month_scroll: UniformListScrollHandle::new(),
            inputs: None,
            invalid: (false, false),
            inputs_stale: false,
            close_on_selection: false,
            _subscriptions: Vec::new(),
        };
        state.reveal_cursor();
        state
    }
    /// Select a range instead of a single date.
    pub fn range(mut self) -> Self {
        self.selection = DateSelection::Range(None, None);
        self
    }
    /// Start with `selection`, showing its (first) month.
    pub fn selection(mut self, selection: impl Into<DateSelection>) -> Self {
        self.apply(selection.into());
        self.reveal_cursor();
        self
    }
    /// Dates matching `matcher` cannot be selected.
    pub fn disabled_dates(mut self, matcher: impl Into<DateMatcher>) -> Self {
        self.disabled = Some(Rc::new(matcher.into()));
        self
    }
    /// The years the year grid and lists offer.
    pub fn years(mut self, years: RangeInclusive<i32>) -> Self {
        self.years = years;
        let shown = clamp_month(&self.years, self.year, self.month);
        if shown != (self.year, self.month) {
            (self.year, self.month) = shown;
            self.cursor = first_of_month(shown.0, shown.1);
        }
        self.reveal_cursor();
        self
    }
    /// The weekday each grid row starts on (default Sunday).
    pub fn first_day_of_week(mut self, day: Weekday) -> Self {
        self.first_day = day;
        self
    }
    /// Start in `view` (e.g. a specimen of the year grid).
    pub fn view(mut self, view: DatePickerView) -> Self {
        self.view = view;
        self.reveal_cursor();
        self
    }
    /// Start in text-input mode.
    pub fn input_mode(mut self) -> Self {
        self.mode = DateInputMode::Input;
        self
    }
    /// Edit typed dates in these Kit inputs (the second for a range's end).
    /// The picker parses them with [`DatePickerLabels::parse_input`].
    pub fn with_inputs(
        mut self,
        start: Entity<InputState>,
        end: Option<Entity<InputState>>,
        cx: &mut Context<Self>,
    ) -> Self {
        self._subscriptions
            .push(cx.subscribe(&start, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.parse_inputs(cx);
                }
            }));
        if let Some(end) = &end {
            self._subscriptions
                .push(cx.subscribe(end, |this, _, event: &InputEvent, cx| {
                    if matches!(event, InputEvent::Change) {
                        this.parse_inputs(cx);
                    }
                }));
        }
        self.inputs = Some((start, end));
        self.inputs_stale = true;
        self
    }

    /// Emit [`DatePickerEvent::Dismiss`] once a choice is complete.
    pub fn close_on_selection(mut self, close: bool) -> Self {
        self.close_on_selection = close;
        self
    }
    /// Change [`Self::close_on_selection`] on a live picker.
    pub fn set_close_on_selection(&mut self, close: bool) {
        self.close_on_selection = close;
    }
    pub fn labels(&self) -> &DatePickerLabels {
        &self.labels
    }
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus
    }
    pub fn value(&self) -> DateSelection {
        self.selection
    }
    pub fn today(&self) -> NaiveDate {
        self.today
    }
    /// The displayed `(year, month)`.
    pub fn displayed_month(&self) -> (i32, u32) {
        (self.year, self.month)
    }
    pub fn current_view(&self) -> DatePickerView {
        self.view
    }
    pub fn mode(&self) -> DateInputMode {
        self.mode
    }
    /// The keyboard cursor's date.
    pub fn cursor(&self) -> NaiveDate {
        self.cursor
    }
    pub fn is_range(&self) -> bool {
        !self.selection.is_single()
    }
    pub fn is_disabled(&self, date: NaiveDate) -> bool {
        self.disabled.as_ref().is_some_and(|m| m.matched(&date))
    }

    fn apply(&mut self, selection: DateSelection) {
        self.selection = selection;
        if let Some(start) = selection.start() {
            // A date outside `years` stays selected, but the grid shows the
            // nearest month the picker offers.
            (self.year, self.month) = clamp_month(&self.years, start.year(), start.month());
            self.cursor = if (start.year(), start.month()) == (self.year, self.month) {
                start
            } else {
                first_of_month(self.year, self.month)
            };
        }
    }

    /// Replace the selection without emitting.
    pub fn set_value(&mut self, selection: DateSelection, cx: &mut Context<Self>) {
        self.apply(selection);
        self.inputs_stale = self.inputs.is_some();
        cx.notify();
    }

    /// Select `date` as a person would: a single picker replaces its date; a
    /// range starts, completes (when `date` is not before the start) or
    /// restarts, as in `gpui_base::CalendarState::select_date`. Disabled
    /// dates are ignored. Returns whether the selection changed; choosing the
    /// selected date again changes nothing, but still closes a closing picker.
    pub fn select(&mut self, date: NaiveDate, cx: &mut Context<Self>) -> bool {
        // An adjacent month's day can lie past the offered years, where the
        // grid cannot show it.
        if self.is_disabled(date) || !self.years.contains(&date.year()) {
            return false;
        }
        let previous = self.selection;
        self.selection = next_selection(previous, date);
        let changed = self.selection != previous;
        self.cursor = date;
        if (date.year(), date.month()) != (self.year, self.month)
            && self.view == DatePickerView::Day
        {
            self.show_month(date.year(), date.month(), cx);
        }
        if changed {
            self.report_change(cx);
        } else if self.close_on_selection && self.selection.is_complete() {
            cx.emit(DatePickerEvent::Dismiss(self.selection));
        }
        cx.notify();
        changed
    }

    /// Clear the selection (keeping single or range).
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.selection = if self.is_range() {
            DateSelection::Range(None, None)
        } else {
            DateSelection::Single(None)
        };
        self.invalid = (false, false);
        cx.emit(DatePickerEvent::Change(self.selection));
        cx.notify();
    }

    /// Show `year`/`month`, sliding from the current month.
    /// Months outside `years` show the nearest month inside them.
    pub fn show_month(&mut self, year: i32, month: u32, cx: &mut Context<Self>) {
        let (year, month) = clamp_month(&self.years, year, month);
        let target = first_of_month(year, month);
        let current = (self.year, self.month);
        let to = (target.year(), target.month());
        if (self.cursor.year(), self.cursor.month()) != to {
            let day = self.cursor.day().min(days_in_month(to.0, to.1));
            self.cursor = NaiveDate::from_ymd_opt(to.0, to.1, day).unwrap_or(target);
        }
        if to == current {
            return;
        }
        self.generation += 1;
        let direction = if to > current { 1. } else { -1. };
        self.slide = Some(Slide {
            from: current,
            direction,
            generation: self.generation,
        });
        (self.year, self.month) = to;
        // Drop the outgoing grid once its slide has finished.
        let generation = self.generation;
        let settle = motion::curve(MotionToken::DefaultSpatial).duration();
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(settle + Duration::from_millis(16))
                .await;
            this.update(cx, |this, cx| {
                if this.slide.is_some_and(|s| s.generation == generation) {
                    this.slide = None;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    pub fn next_month(&mut self, cx: &mut Context<Self>) {
        self.show_month(self.year, self.month + 1, cx);
    }
    pub fn previous_month(&mut self, cx: &mut Context<Self>) {
        let (year, month) = previous(self.year, self.month);
        self.show_month(year, month, cx);
    }
    pub fn next_year(&mut self, cx: &mut Context<Self>) {
        self.show_month(self.year + 1, self.month, cx);
    }
    pub fn previous_year(&mut self, cx: &mut Context<Self>) {
        self.show_month(self.year - 1, self.month, cx);
    }

    /// Switch the body between the day grid, the year grid/list and the
    /// docked month list.
    pub fn set_view(&mut self, view: DatePickerView, cx: &mut Context<Self>) {
        if self.view != view {
            self.view = view;
            self.cursor = NaiveDate::from_ymd_opt(
                self.year,
                self.month,
                self.cursor.day().min(days_in_month(self.year, self.month)),
            )
            .unwrap_or(self.cursor);
            self.reveal_cursor();
            cx.notify();
        }
    }
    /// Toggle `view` against the day grid.
    pub fn toggle_view(&mut self, view: DatePickerView, cx: &mut Context<Self>) {
        let next = if self.view == view {
            DatePickerView::Day
        } else {
            view
        };
        self.set_view(next, cx);
    }

    /// Choose a year from the year grid or list and return to the days.
    pub fn choose_year(&mut self, year: i32, cx: &mut Context<Self>) {
        let month = self.month;
        self.view = DatePickerView::Day;
        self.show_month(year, month, cx);
        cx.notify();
    }
    /// Choose a month from the docked list and return to the days.
    pub fn choose_month(&mut self, month: u32, cx: &mut Context<Self>) {
        let year = self.year;
        self.view = DatePickerView::Day;
        self.show_month(year, month, cx);
        cx.notify();
    }

    /// Switch between calendar and text input. Entering input mode writes
    /// the selection into the inputs; leaving it parses them.
    pub fn toggle_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.mode {
            DateInputMode::Calendar => {
                self.mode = DateInputMode::Input;
                self.inputs_stale = false;
                self.write_inputs(window, cx);
            }
            DateInputMode::Input => {
                self.parse_inputs(cx);
                self.mode = DateInputMode::Calendar;
            }
        }
        cx.notify();
    }

    fn write_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((start, end)) = self.inputs.clone() else {
            return;
        };
        let format = self.labels.format_input.clone();
        let text = |date: Option<NaiveDate>| -> SharedString {
            date.map(|d| format(d)).unwrap_or_default()
        };
        let (first, last) = (self.selection.start(), self.selection.end());
        start.update(cx, |input, cx| input.set_value(text(first), window, cx));
        if let Some(end) = end {
            end.update(cx, |input, cx| input.set_value(text(last), window, cx));
        }
        self.invalid = (false, false);
    }

    /// Parse the text inputs into the selection. Invalid or disabled text
    /// marks its field invalid and leaves the selection unchanged.
    pub fn parse_inputs(&mut self, cx: &mut Context<Self>) {
        let Some((start, end)) = self.inputs.clone() else {
            return;
        };
        let parse = self.labels.parse_input.clone();
        let read = |input: &Entity<InputState>, cx: &App| -> Result<Option<NaiveDate>, ()> {
            let text = input.read(cx).value();
            let text = text.trim();
            if text.is_empty() {
                return Ok(None);
            }
            parse(text).map(Some).ok_or(())
        };
        let first = read(&start, cx);
        let last = end.as_ref().map(|end| read(end, cx));
        let disabled = |d: &Result<Option<NaiveDate>, ()>| matches!(d, Ok(Some(date)) if self.is_disabled(*date));
        self.invalid = (
            first.is_err() || disabled(&first),
            last.as_ref().is_some_and(|l| l.is_err() || disabled(l)),
        );
        if self.invalid != (false, false) {
            cx.notify();
            return;
        }
        let next = match (self.selection, first, last) {
            (DateSelection::Single(_), Ok(date), _) => DateSelection::Single(date),
            (DateSelection::Range(..), Ok(a), Some(Ok(b))) => match (a, b) {
                (Some(a), Some(b)) if b < a => {
                    self.invalid.1 = true;
                    cx.notify();
                    return;
                }
                _ => DateSelection::Range(a, b),
            },
            _ => return,
        };
        if next != self.selection {
            self.apply(next);
            // Text that parses is not a choice made: only a pick closes the
            // picker, or a date would close it under the person still typing.
            cx.emit(DatePickerEvent::Change(self.selection));
        }
        cx.notify();
    }

    fn report_change(&mut self, cx: &mut Context<Self>) {
        cx.emit(DatePickerEvent::Change(self.selection));
        if self.close_on_selection && self.selection.is_complete() {
            cx.emit(DatePickerEvent::Dismiss(self.selection));
        }
    }

    /// Move the keyboard cursor and keep it visible.
    fn move_cursor(&mut self, step: CursorStep, cx: &mut Context<Self>) {
        let next = cursor_target(self.view, self.cursor, step, self.first_day);
        let years = self.years.clone();
        let next = if years.contains(&next.year()) {
            next
        } else {
            self.cursor
        };
        self.cursor = next;
        if self.view == DatePickerView::Day
            && (next.year(), next.month()) != (self.year, self.month)
        {
            self.show_month(next.year(), next.month(), cx);
        }
        self.reveal_cursor();
        cx.notify();
    }

    fn reveal_cursor(&self) {
        let years_from = *self.years.start();
        // Never past the lists' ends: a year outside `years` reveals the
        // nearest one.
        let years = (self.years.end() - years_from + 1).max(1) as usize;
        match self.view {
            DatePickerView::Year => {
                let index = ((self.cursor.year() - years_from).max(0) as usize).min(years - 1);
                self.year_scroll
                    .scroll_to_item(index / YEAR_COLUMNS, ScrollStrategy::Center);
                self.list_scroll
                    .scroll_to_item(index, ScrollStrategy::Center);
            }
            DatePickerView::Month => {
                self.list_scroll
                    .scroll_to_item(self.cursor.month0() as usize, ScrollStrategy::Center);
            }
            DatePickerView::Day => {
                let index = month_list_index(&self.years, self.year, self.month0());
                self.month_scroll.scroll_to_item(index, ScrollStrategy::Top);
            }
        }
    }

    fn month0(&self) -> usize {
        self.month.saturating_sub(1) as usize
    }

    fn activate_cursor(&mut self, cx: &mut Context<Self>) {
        match self.view {
            DatePickerView::Day => {
                self.select(self.cursor, cx);
            }
            DatePickerView::Year => self.choose_year(self.cursor.year(), cx),
            DatePickerView::Month => self.choose_month(self.cursor.month(), cx),
        }
    }
}

impl Render for DatePickerState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

/// `year`/`month` (normalized, so month 13 is January next year) kept
/// inside the January of the first and the December of the last of `years`.
fn clamp_month(years: &RangeInclusive<i32>, year: i32, month: u32) -> (i32, u32) {
    let date = first_of_month(year, month);
    let to = (date.year(), date.month());
    if years.start() > years.end() {
        return to;
    }
    to.clamp((*years.start(), 1), (*years.end(), 12))
}

/// The full-screen month list's row for `year`/`month0`, never past its
/// last row (`years` holds twelve rows per year).
fn month_list_index(years: &RangeInclusive<i32>, year: i32, month0: usize) -> usize {
    let count = (years.end() - years.start() + 1).max(1) as usize * 12;
    ((year - years.start()).max(0) as usize * 12 + month0).min(count - 1)
}

/// Whether `year`/`month` lies inside `years`.
fn month_in_range(years: &RangeInclusive<i32>, year: i32, month: u32) -> bool {
    let date = first_of_month(year, month);
    clamp_month(years, year, month) == (date.year(), date.month())
}

fn previous(year: i32, month: u32) -> (i32, u32) {
    if month <= 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    }
}

pub(crate) fn days_in_month(year: i32, month: u32) -> u32 {
    let first = first_of_month(year, month);
    (first_of_month(year, month + 1) - first).num_days() as u32
}

/// The range rule of `gpui_base::CalendarState::select_date`.
pub(crate) fn next_selection(current: DateSelection, value: NaiveDate) -> DateSelection {
    match current {
        DateSelection::Single(_) => DateSelection::Single(Some(value)),
        DateSelection::Range(None, _) => DateSelection::Range(Some(value), None),
        DateSelection::Range(Some(start), None) if value >= start => {
            DateSelection::Range(Some(start), Some(value))
        }
        DateSelection::Range(Some(_), _) => DateSelection::Range(Some(value), None),
    }
}

/// One keyboard movement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CursorStep {
    Left,
    Right,
    Up,
    Down,
    PreviousMonth,
    NextMonth,
    PreviousYear,
    NextYear,
    WeekStart,
    WeekEnd,
}

const YEAR_COLUMNS: usize = 3;

/// Where `step` moves the cursor in `view`: days and weeks in the day grid,
/// years in rows of three in the year grid, and one row in the docked lists.
pub(crate) fn cursor_target(
    view: DatePickerView,
    cursor: NaiveDate,
    step: CursorStep,
    first_day: Weekday,
) -> NaiveDate {
    use CursorStep::*;
    let days = |n: i64| {
        if n >= 0 {
            cursor + Days::new(n as u64)
        } else {
            cursor - Days::new(n.unsigned_abs())
        }
    };
    match (view, step) {
        (_, PreviousMonth) => add_months(cursor, -1),
        (_, NextMonth) => add_months(cursor, 1),
        (_, PreviousYear) => add_months(cursor, -12),
        (_, NextYear) => add_months(cursor, 12),
        (DatePickerView::Day, Left) => days(-1),
        (DatePickerView::Day, Right) => days(1),
        (DatePickerView::Day, Up) => days(-7),
        (DatePickerView::Day, Down) => days(7),
        (DatePickerView::Day, WeekStart | WeekEnd) => {
            let offset = (cursor.weekday().num_days_from_sunday() + 7
                - first_day.num_days_from_sunday())
                % 7;
            if step == WeekStart {
                days(-(offset as i64))
            } else {
                days(6 - offset as i64)
            }
        }
        (DatePickerView::Year, Left) => add_months(cursor, -12),
        (DatePickerView::Year, Right) => add_months(cursor, 12),
        (DatePickerView::Year, Up) => add_months(cursor, -12 * YEAR_COLUMNS as i32),
        (DatePickerView::Year, Down) => add_months(cursor, 12 * YEAR_COLUMNS as i32),
        (DatePickerView::Month, Up | Left) if cursor.month() > 1 => add_months(cursor, -1),
        (DatePickerView::Month, Down | Right) if cursor.month() < 12 => add_months(cursor, 1),
        (DatePickerView::Month, WeekStart) => add_months(cursor, 1 - cursor.month() as i32),
        (DatePickerView::Month, WeekEnd) => add_months(cursor, 12 - cursor.month() as i32),
        (DatePickerView::Year, WeekStart | WeekEnd) | (DatePickerView::Month, _) => cursor,
    }
}

type Confirm = Rc<dyn Fn(DateSelection, &mut Window, &mut App)>;

/// An M3 date picker over a [`DatePickerState`].
#[derive(gpui::IntoElement)]
pub struct DatePicker {
    id: ElementId,
    state: Entity<DatePickerState>,
    variant: DatePickerVariant,
    specimen: bool,
    show_clear: bool,
    show_actions: bool,
    height: Option<f32>,
    on_confirm: Option<Confirm>,
    on_cancel: Option<Activate>,
}

/// Widths of the presentations, in dp.
pub const DATE_PICKER_WIDTH: f32 = 360.;
pub const DATE_INPUT_PICKER_WIDTH: f32 = 328.;
const GRID_PADDING: f32 = 12.;
const SLIDE_DISTANCE: f32 = DAY_CELL * 7.;

impl DatePicker {
    pub fn new(id: impl Into<ElementId>, state: &Entity<DatePickerState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            variant: DatePickerVariant::Modal,
            specimen: false,
            show_clear: true,
            show_actions: true,
            height: None,
            on_confirm: None,
            on_cancel: None,
        }
    }
    pub fn variant(mut self, variant: DatePickerVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn docked(self) -> Self {
        self.variant(DatePickerVariant::Docked)
    }
    pub fn full_screen(self) -> Self {
        self.variant(DatePickerVariant::FullScreen)
    }
    /// A static specimen: no tab stops, no motion, no pointer input.
    pub fn specimen(mut self) -> Self {
        self.specimen = true;
        self
    }
    /// Show the Clear action (default on).
    pub fn show_clear(mut self, show: bool) -> Self {
        self.show_clear = show;
        self
    }
    /// Show the action row (default on; a docked picker that commits on
    /// selection may hide it).
    pub fn show_actions(mut self, show: bool) -> Self {
        self.show_actions = show;
        self
    }
    /// The full-screen picker's height in dp (default 696).
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
    /// OK / Save: receives the selection.
    pub fn on_confirm(
        mut self,
        f: impl Fn(DateSelection, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_confirm = Some(Rc::new(f));
        self
    }
    /// Cancel / Close.
    pub fn on_cancel(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Rc::new(f));
        self
    }
}

/// A snapshot of the state a render needs.
struct Snapshot {
    /// The direction the grid rendered in, for the arrow keys.
    rtl: bool,
    labels: Rc<DatePickerLabels>,
    selection: DateSelection,
    today: NaiveDate,
    year: i32,
    month: u32,
    cursor: NaiveDate,
    view: DatePickerView,
    mode: DateInputMode,
    disabled: Option<Rc<DateMatcher>>,
    years: RangeInclusive<i32>,
    first_day: Weekday,
    slide: Option<Slide>,
    focused: bool,
    focus: FocusHandle,
    year_scroll: UniformListScrollHandle,
    list_scroll: UniformListScrollHandle,
    month_scroll: UniformListScrollHandle,
    inputs: Option<(Entity<InputState>, Option<Entity<InputState>>)>,
    invalid: (bool, bool),
}

impl Snapshot {
    /// Whether the month `delta` months from the shown one is offered.
    fn can_step_months(&self, delta: i32) -> bool {
        let total = self.year as i64 * 12 + self.month as i64 - 1 + delta as i64;
        month_in_range(
            &self.years,
            total.div_euclid(12) as i32,
            total.rem_euclid(12) as u32 + 1,
        )
    }
    fn is_disabled(&self, date: NaiveDate) -> bool {
        self.disabled.as_ref().is_some_and(|m| m.matched(&date))
    }
    fn headline(&self) -> SharedString {
        match self.selection {
            DateSelection::Single(Some(date)) => (self.labels.headline)(date),
            DateSelection::Single(None) | DateSelection::Range(None, None) => {
                self.labels.no_selection.clone()
            }
            DateSelection::Range(start, end) => (self.labels.range_headline)(start, end),
        }
    }
    fn range_of(&self, date: NaiveDate) -> (bool, DayRange) {
        match self.selection {
            DateSelection::Single(selected) => (selected == Some(date), DayRange::None),
            DateSelection::Range(Some(start), Some(end)) if start != end => {
                if date == start {
                    (true, DayRange::Start)
                } else if date == end {
                    (true, DayRange::End)
                } else if date > start && date < end {
                    (false, DayRange::Middle)
                } else {
                    (false, DayRange::None)
                }
            }
            DateSelection::Range(start, end) => {
                (start == Some(date) || end == Some(date), DayRange::None)
            }
        }
    }
}

fn icon(name: &str) -> Icon {
    Icon::from(Symbol::named(name).expect("a Material Symbol in the pinned catalog"))
}

impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // OK reads the inputs, so they have to say what the picker holds.
        let state = self.state.read(cx);
        if state.inputs_stale && state.mode == DateInputMode::Input {
            self.state.update(cx, |state, cx| {
                state.inputs_stale = false;
                state.write_inputs(window, cx);
            });
        }
        let s = self.state.read(cx);
        let snap = Rc::new(Snapshot {
            rtl: crate::scope::is_rtl(window),
            labels: s.labels.clone(),
            selection: s.selection,
            today: s.today,
            year: s.year,
            month: s.month,
            cursor: s.cursor,
            view: s.view,
            mode: s.mode,
            disabled: s.disabled.clone(),
            years: s.years.clone(),
            first_day: s.first_day,
            slide: s.slide,
            focused: s.focus.is_focused(window),
            focus: s.focus.clone(),
            year_scroll: s.year_scroll.clone(),
            list_scroll: s.list_scroll.clone(),
            month_scroll: s.month_scroll.clone(),
            inputs: s.inputs.clone(),
            invalid: s.invalid,
        });
        let ctx = PickerCx {
            id: self.id.clone(),
            state: self.state.clone(),
            snap,
            specimen: self.specimen,
            reduce_motion: cx.reduce_motion(),
        };
        let actions = ActionRow {
            show_clear: self.show_clear,
            on_confirm: self.on_confirm.clone(),
            on_cancel: self.on_cancel.clone(),
        };
        match (self.variant, ctx.snap.mode) {
            (DatePickerVariant::Docked, _) => ctx
                .docked(self.show_actions.then_some(actions))
                .into_any_element(),
            (DatePickerVariant::FullScreen, DateInputMode::Calendar) => ctx
                .full_screen(actions, self.height.unwrap_or(696.))
                .into_any_element(),
            (DatePickerVariant::Modal | DatePickerVariant::FullScreen, DateInputMode::Input) => ctx
                .modal_input(actions)
                .when(cfg!(test), |input| {
                    input.debug_selector(|| "date-picker-input".into())
                })
                .into_any_element(),
            (DatePickerVariant::Modal, DateInputMode::Calendar) => {
                ctx.modal(actions).into_any_element()
            }
        }
    }
}

struct ActionRow {
    show_clear: bool,
    on_confirm: Option<Confirm>,
    on_cancel: Option<Activate>,
}

struct PickerCx {
    id: ElementId,
    state: Entity<DatePickerState>,
    snap: Rc<Snapshot>,
    specimen: bool,
    reduce_motion: bool,
}

fn label_text(
    id: impl Into<ElementId>,
    text: SharedString,
    role: TypeRole,
    ink: Rgba,
) -> Stateful<Div> {
    div()
        .id(id)
        .text_color(ink)
        .map(|d| type_style(d, role))
        .whitespace_nowrap()
        .overflow_hidden()
        .text_ellipsis()
        .child(text)
}

impl PickerCx {
    fn child_id(&self, name: &'static str) -> ElementId {
        ElementId::NamedChild(self.id.clone().into(), name.into())
    }

    /// Run `f` on the state from a listener.
    fn update(
        &self,
        f: impl Fn(&mut DatePickerState, &mut Window, &mut Context<DatePickerState>) + 'static,
    ) -> impl Fn(&mut Window, &mut App) + 'static {
        let state = self.state.clone();
        move |window, cx| state.update(cx, |s, cx| f(s, window, cx))
    }

    fn preview(&self) -> Option<InteractionPreview> {
        self.specimen.then_some(InteractionPreview::Enabled)
    }

    fn icon_button(
        &self,
        name: &'static str,
        symbol: &str,
        label: SharedString,
        f: impl Fn(&mut DatePickerState, &mut Window, &mut Context<DatePickerState>) + 'static,
    ) -> IconButton {
        let click = self.update(f);
        let mut button = IconButton::new(self.child_id(name), icon(symbol), label)
            .on_click(move |_, window, cx| click(window, cx));
        if let Some(preview) = self.preview() {
            button = button.preview_state(preview);
        }
        button
    }

    fn text_button(
        &self,
        name: &'static str,
        label: SharedString,
        f: impl Fn(&mut Window, &mut App) + 'static,
    ) -> AnyElement {
        let mut button = material_button(self.child_id(name), ButtonVariant::Text)
            .child(label)
            .on_click(move |_, window, cx| f(window, cx));
        if let Some(preview) = self.preview() {
            button = button.preview_state(preview);
        }
        button.into_any_element()
    }

    fn action_row(&self, actions: ActionRow, clear: bool) -> Div {
        let l = &self.snap.labels;
        let cancel = actions.on_cancel.clone();
        let confirm = actions.on_confirm.clone();
        let state = self.state.clone();
        div()
            .w_full()
            .flex()
            .items_start()
            .justify_between()
            .pt(rem(4.))
            .pr(rem(12.))
            .pb(rem(8.))
            .pl(rem(12.))
            .gap(rem(8.))
            .child(if clear && actions.show_clear {
                self.text_button("clear", l.clear.clone(), {
                    let state = self.state.clone();
                    move |_, cx| state.update(cx, |s, cx| s.clear(cx))
                })
            } else {
                div().into_any_element()
            })
            .child(
                div()
                    .flex()
                    .gap(rem(8.))
                    .child(
                        self.text_button("cancel", l.cancel.clone(), move |window, cx| {
                            if let Some(f) = &cancel {
                                f(window, cx);
                            }
                        }),
                    )
                    .child(
                        self.text_button("confirm", l.confirm.clone(), move |window, cx| {
                            if state.read(cx).mode == DateInputMode::Input {
                                state.update(cx, |s, cx| s.parse_inputs(cx));
                                if state.read(cx).invalid != (false, false) {
                                    return;
                                }
                            }
                            let value = state.read(cx).selection;
                            if let Some(f) = &confirm {
                                f(value, window, cx);
                            }
                        }),
                    ),
            )
    }

    fn surface(&self, width: f32, radius: f32) -> Stateful<Div> {
        div()
            .id(self.id.clone())
            .w(rem(width))
            .flex()
            .flex_col()
            .flex_shrink_0()
            .overflow_hidden()
            .rounded(rem(radius))
            .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
            .role(Role::Dialog)
    }

    fn modal_header(&self) -> Div {
        let l = &self.snap.labels;
        div()
            .w_full()
            .h(rem(120.))
            .flex()
            .items_end()
            .gap(rem(8.))
            .pt(rem(16.))
            .pr(rem(12.))
            .pb(rem(12.))
            .pl(rem(24.))
            .border_b(rem(1.))
            .border_color(color(ColorRole::OUTLINE_VARIANT))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(rem(36.))
                    .child(label_text(
                        "title",
                        l.title.clone(),
                        TypeRole::LabelLarge,
                        color(ColorRole::ON_SURFACE_VARIANT),
                    ))
                    .child(
                        label_text(
                            "headline",
                            self.snap.headline(),
                            TypeRole::HeadlineLarge,
                            color(ColorRole::ON_SURFACE),
                        )
                        .role(Role::Heading),
                    ),
            )
            .child(self.icon_button(
                "mode",
                "edit",
                l.switch_to_input.clone(),
                |s, window, cx| s.toggle_mode(window, cx),
            ))
    }

    fn month_row(&self) -> Div {
        let snap = &self.snap;
        let l = &snap.labels;
        let year_view = snap.view == DatePickerView::Year;
        let mut menu = PickerMenuButton::new(
            self.child_id("month-menu"),
            (l.month_year)(snap.year, snap.month),
        )
        .accessible_label(l.choose_year.clone())
        .open(year_view)
        .on_click(self.update(|s, _, cx| s.toggle_view(DatePickerView::Year, cx)));
        if let Some(preview) = self.preview() {
            menu = menu.preview_state(preview);
        }
        div()
            .w_full()
            .h(rem(56.))
            .flex()
            .items_center()
            .justify_between()
            .pt(rem(4.))
            .pr(rem(12.))
            .pb(rem(4.))
            .pl(rem(16.))
            .child(menu)
            .when(!year_view, |row| {
                row.child(
                    div()
                        .flex()
                        .child(
                            self.icon_button(
                                "previous",
                                "chevron_backward",
                                l.previous_month.clone(),
                                |s, _, cx| s.previous_month(cx),
                            )
                            .disabled(!snap.can_step_months(-1)),
                        )
                        .child(
                            self.icon_button(
                                "next",
                                "chevron_forward",
                                l.next_month.clone(),
                                |s, _, cx| s.next_month(cx),
                            )
                            .disabled(!snap.can_step_months(1)),
                        ),
                )
            })
    }

    fn weekday_row(&self) -> Div {
        let l = &self.snap.labels;
        div()
            .flex()
            .flex_shrink_0()
            .children(week_days(self.snap.first_day).into_iter().map(|day| {
                div()
                    .size(rem(DAY_CELL))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(color(ColorRole::ON_SURFACE))
                    .map(|d| type_style(d, TypeRole::BodyLarge))
                    .child((l.weekday)(day))
            }))
    }

    /// The keyboard host around a grid or list: one tab stop, the picker's
    /// key context and actions.
    fn key_host(&self, element: Stateful<Div>) -> Stateful<Div> {
        if self.specimen {
            return element;
        }
        fn on_step<A: Action>(
            element: Stateful<Div>,
            state: &Entity<DatePickerState>,
            step: CursorStep,
        ) -> Stateful<Div> {
            let state = state.clone();
            element.on_action(move |_: &A, _, cx| state.update(cx, |s, cx| s.move_cursor(step, cx)))
        }
        let state = &self.state;
        let select = self.update(|s, _, cx| s.activate_cursor(cx));
        let element = element
            .track_focus(&self.snap.focus)
            .key_context(CONTEXT)
            .on_action(move |_: &SelectCursor, window, cx| select(window, cx));
        let (left, right) = if self.snap.rtl {
            (CursorStep::Right, CursorStep::Left)
        } else {
            (CursorStep::Left, CursorStep::Right)
        };
        let element = on_step::<CursorLeft>(element, state, left);
        let element = on_step::<CursorRight>(element, state, right);
        let element = on_step::<CursorUp>(element, state, CursorStep::Up);
        let element = on_step::<CursorDown>(element, state, CursorStep::Down);
        let element = on_step::<CursorPreviousMonth>(element, state, CursorStep::PreviousMonth);
        let element = on_step::<CursorNextMonth>(element, state, CursorStep::NextMonth);
        let element = on_step::<CursorPreviousYear>(element, state, CursorStep::PreviousYear);
        let element = on_step::<CursorNextYear>(element, state, CursorStep::NextYear);
        let element = on_step::<CursorWeekStart>(element, state, CursorStep::WeekStart);
        on_step::<CursorWeekEnd>(element, state, CursorStep::WeekEnd)
    }

    /// The weeks of one month: `adjacent` shows the neighbouring months'
    /// days (docked) instead of empty cells (modal).
    fn weeks(&self, year: i32, month: u32, adjacent: bool, live: bool, key: &str) -> Div {
        let snap = &self.snap;
        let weeks = month_weeks(year, month, snap.first_day);
        div()
            .flex()
            .flex_col()
            .children(weeks.into_iter().enumerate().map(|(row, week)| {
                div()
                    .id(SharedString::from(format!("{key}-week-{row}")))
                    .flex()
                    .role(Role::Row)
                    .children(week.into_iter().map(|date| {
                        let outside = date.month() != month;
                        if outside && !adjacent {
                            return div().size(rem(DAY_CELL)).flex_shrink_0().into_any_element();
                        }
                        self.day(date, outside, live, key).into_any_element()
                    }))
            }))
    }

    fn day(&self, date: NaiveDate, outside: bool, live: bool, key: &str) -> CalendarDay {
        let snap = &self.snap;
        let (selected, range) = if outside {
            (false, DayRange::None)
        } else {
            snap.range_of(date)
        };
        let id = SharedString::from(format!("{key}-{}", date.num_days_from_ce()));
        let mut day = CalendarDay::new(id, (snap.labels.number)(date.day() as i32))
            .accessible_label((snap.labels.accessible_date)(date))
            .selected(selected)
            .today(date == snap.today && !outside)
            .adjacent(outside)
            .range(range)
            .disabled(snap.is_disabled(date) || !snap.years.contains(&date.year()));
        if self.specimen || !live {
            day = day.preview_state(InteractionPreview::Enabled);
        } else {
            let state = self.state.clone();
            let focus = snap.focus.clone();
            day = day
                .animate(!self.reduce_motion)
                .cursor(snap.focused && date == snap.cursor)
                .on_click(move |window, cx| {
                    focus.focus(window, cx);
                    state.update(cx, |s, cx| {
                        s.select(date, cx);
                    });
                });
        }
        day
    }

    /// The day grid with its month slide.
    fn day_grid(&self, adjacent: bool) -> Stateful<Div> {
        let snap = &self.snap;
        let incoming = self
            .weeks(snap.year, snap.month, adjacent, true, "in")
            .w(rem(SLIDE_DISTANCE));
        let body = div().relative().overflow_hidden().w(rem(SLIDE_DISTANCE));
        let body = match snap.slide.filter(|_| !self.specimen && !self.reduce_motion) {
            Some(slide) => {
                let spatial = motion::curve(MotionToken::DefaultSpatial);
                let ease = spatial.ease();
                let animation = Animation::new(spatial.duration());
                let dir = slide.direction;
                let outgoing = self
                    .weeks(slide.from.0, slide.from.1, adjacent, false, "out")
                    .absolute()
                    .top_0()
                    .left_0()
                    .w(rem(SLIDE_DISTANCE));
                body.child(incoming.with_animation(
                    ("month-in", slide.generation),
                    animation.clone(),
                    move |el, t| {
                        let p = ease(t);
                        el.relative()
                            .left(rem(SLIDE_DISTANCE * dir * (1. - p)))
                            .opacity((t * 1.5).clamp(0., 1.))
                    },
                ))
                .child(outgoing.with_animation(
                    ("month-out", slide.generation),
                    animation,
                    move |el, t| {
                        let p = ease(t);
                        el.left(rem(-SLIDE_DISTANCE * dir * p))
                            .opacity((1. - t * 2.).clamp(0., 1.))
                    },
                ))
            }
            None => body.child(incoming),
        };
        let l = &snap.labels;
        self.key_host(
            div()
                .id(self.child_id("grid"))
                .flex()
                .flex_col()
                .px(rem(GRID_PADDING))
                .role(Role::Grid)
                .aria_label((l.month_year)(snap.year, snap.month))
                .aria_row_count(GRID_WEEKS)
                .aria_column_count(7)
                .child(self.weekday_row())
                .child(body),
        )
    }

    /// The modal year grid: three years per 52dp row, virtualized.
    fn year_grid(&self, height: f32) -> Stateful<Div> {
        let snap = self.snap.clone();
        let first = *snap.years.start();
        let count = (snap.years.end() - first + 1).max(0) as usize;
        let rows = count.div_ceil(YEAR_COLUMNS);
        let state = self.state.clone();
        let specimen = self.specimen;
        let list = uniform_list(self.child_id("years"), rows, move |range, _, _| {
            range
                .map(|row| {
                    div()
                        .id(row)
                        .w_full()
                        .flex()
                        .gap(rem(8.))
                        .px(rem(24.))
                        .role(Role::Row)
                        .children((0..YEAR_COLUMNS).filter_map(|col| {
                            let year = first + (row * YEAR_COLUMNS + col) as i32;
                            (year <= *snap.years.end()).then(|| {
                                let mut chip =
                                    YearChip::new(year as usize, (snap.labels.number)(year))
                                        .selected(year == snap.year);
                                if specimen {
                                    chip = chip.preview_state(InteractionPreview::Enabled);
                                } else {
                                    let state = state.clone();
                                    chip = chip
                                        .cursor(snap.focused && year == snap.cursor.year())
                                        .on_click(move |_, cx| {
                                            state.update(cx, |s, cx| s.choose_year(year, cx))
                                        });
                                }
                                chip
                            })
                        }))
                })
                .collect()
        })
        .track_scroll(&self.snap.year_scroll)
        .w_full()
        .h(rem(height));
        self.key_host(
            div()
                .id(self.child_id("year-grid"))
                .w_full()
                .role(Role::Grid)
                .aria_label(self.snap.labels.choose_year.clone())
                .child(list),
        )
    }

    fn modal(self, actions: ActionRow) -> Stateful<Div> {
        let body_height = DAY_CELL * (GRID_WEEKS as f32 + 1.);
        let body = match self.snap.view {
            DatePickerView::Day | DatePickerView::Month => self.day_grid(false).into_any_element(),
            DatePickerView::Year => self.year_grid(body_height).into_any_element(),
        };
        self.surface(DATE_PICKER_WIDTH, 28.)
            .child(self.modal_header())
            .child(self.month_row())
            .child(div().h(rem(body_height)).child(body))
            .child(self.action_row(actions, true))
    }

    fn field(
        &self,
        name: &'static str,
        input: Option<&Entity<InputState>>,
        label: SharedString,
        invalid: bool,
        first: bool,
    ) -> AnyElement {
        let l = &self.snap.labels;
        let field = match input {
            Some(input) if !self.specimen => TextField::new(self.child_id(name), input),
            _ => TextField::specimen(self.child_id(name), ""),
        }
        .outlined()
        .label(label)
        .placeholder(l.input_placeholder.clone())
        .error(invalid)
        .w_full();
        let field = if invalid {
            field.supporting_text(l.invalid_input.clone())
        } else {
            field
        };
        let field = if self.specimen && first {
            field.preview_state(InteractionPreview::Focused)
        } else if self.specimen {
            field.preview_state(InteractionPreview::Enabled)
        } else {
            field
        };
        div().flex_1().min_w_0().child(field).into_any_element()
    }

    fn modal_input(self, actions: ActionRow) -> Stateful<Div> {
        let l = self.snap.labels.clone();
        let range = !self.snap.selection.is_single();
        let inputs = self.snap.inputs.clone();
        let (start, end) = match &inputs {
            Some((start, end)) => (Some(start), end.as_ref()),
            None => (None, None),
        };
        let invalid = self.snap.invalid;
        let mut button = IconButton::new(
            self.child_id("mode"),
            icon("today"),
            l.switch_to_calendar.clone(),
        )
        .width(IconButtonWidth::Wide)
        .on_click({
            let f = self.update(|s, window, cx| s.toggle_mode(window, cx));
            move |_, window, cx| f(window, cx)
        });
        if let Some(preview) = self.preview() {
            button = button.preview_state(preview);
        }
        let header = div()
            .w_full()
            .h(rem(128.))
            .flex()
            .flex_col()
            .gap(rem(36.))
            .pt(rem(16.))
            .pr(rem(12.))
            .pb(rem(8.))
            .pl(rem(24.))
            .border_b(rem(1.))
            .border_color(color(ColorRole::OUTLINE_VARIANT))
            .child(label_text(
                "title",
                l.title.clone(),
                TypeRole::LabelLarge,
                color(ColorRole::ON_SURFACE_VARIANT),
            ))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(rem(8.))
                    .child(
                        label_text(
                            "headline",
                            l.input_headline.clone(),
                            TypeRole::HeadlineLarge,
                            color(ColorRole::ON_SURFACE),
                        )
                        .flex_1()
                        .min_w_0()
                        .role(Role::Heading),
                    )
                    .child(button),
            );
        let fields = div()
            .w_full()
            .flex()
            .gap(rem(12.))
            .pt(rem(18.))
            .pr(rem(24.))
            .pb(rem(16.))
            .pl(rem(24.))
            .min_h(rem(90.))
            .child(self.field(
                "start-field",
                start,
                if range {
                    l.start_field.clone()
                } else {
                    l.date_field.clone()
                },
                invalid.0,
                true,
            ))
            .when(range, |row| {
                row.child(self.field("end-field", end, l.end_field.clone(), invalid.1, false))
            });
        self.surface(DATE_INPUT_PICKER_WIDTH, 28.)
            .child(header)
            .child(fields)
            .child(self.action_row(actions, false))
    }

    /// The docked menu's selection row: month and year steppers with their
    /// menu buttons; the inactive one dims while the other's list is open.
    fn docked_row(&self) -> Div {
        let snap = &self.snap;
        let l = &snap.labels;
        let view = snap.view;
        let stepper = |name: &'static str,
                       label: SharedString,
                       list: DatePickerView,
                       previous: (
            SharedString,
            fn(&mut DatePickerState, &mut Context<DatePickerState>),
        ),
                       next: (
            SharedString,
            fn(&mut DatePickerState, &mut Context<DatePickerState>),
        ),
                       accessible: SharedString| {
            let open = view == list;
            let dimmed = view != DatePickerView::Day && !open;
            let mut menu = PickerMenuButton::new(self.child_id(name), label)
                .accessible_label(accessible)
                .open(open)
                .arrow(!dimmed)
                .disabled(dimmed)
                .on_click(self.update(move |s, _, cx| s.toggle_view(list, cx)));
            if let Some(preview) = self.preview() {
                menu = menu.preview_state(if dimmed {
                    InteractionPreview::Disabled
                } else {
                    preview
                });
            }
            let (prev_label, prev) = previous;
            let (next_label, next) = next;
            let show_steppers = view == DatePickerView::Day;
            // Steppers stop at the ends of `years`.
            let months = if list == DatePickerView::Month { 1 } else { 12 };
            let (has_prev, has_next) =
                (snap.can_step_months(-months), snap.can_step_months(months));
            div()
                .flex()
                .items_center()
                .when(show_steppers, |row| {
                    row.child(
                        self.icon_button(
                            if list == DatePickerView::Month {
                                "prev-month"
                            } else {
                                "prev-year"
                            },
                            "chevron_left",
                            prev_label,
                            move |s, _, cx| prev(s, cx),
                        )
                        .disabled(!has_prev),
                    )
                })
                .child(menu)
                .when(show_steppers, |row| {
                    row.child(
                        self.icon_button(
                            if list == DatePickerView::Month {
                                "next-month"
                            } else {
                                "next-year"
                            },
                            "chevron_right",
                            next_label,
                            move |s, _, cx| next(s, cx),
                        )
                        .disabled(!has_next),
                    )
                })
        };
        div()
            .w_full()
            .h(rem(64.))
            .flex()
            .items_center()
            .justify_between()
            .when(view != DatePickerView::Day, |row| {
                row.px(rem(40.))
                    .border_b(rem(1.))
                    .border_color(color(ColorRole::OUTLINE_VARIANT))
            })
            .child(stepper(
                "docked-month",
                (l.month_short)(snap.month),
                DatePickerView::Month,
                (l.previous_month.clone(), DatePickerState::previous_month),
                (l.next_month.clone(), DatePickerState::next_month),
                l.choose_month.clone(),
            ))
            .child(stepper(
                "docked-year",
                (l.number)(snap.year),
                DatePickerView::Year,
                (l.previous_year.clone(), DatePickerState::previous_year),
                (l.next_year.clone(), DatePickerState::next_year),
                l.choose_year.clone(),
            ))
    }

    /// The docked month or year list.
    fn docked_list(&self, months: bool) -> Stateful<Div> {
        let snap = self.snap.clone();
        let first = *snap.years.start();
        let count = if months {
            12
        } else {
            (snap.years.end() - first + 1).max(0) as usize
        };
        let state = self.state.clone();
        let specimen = self.specimen;
        let list = uniform_list(self.child_id("docked-list"), count, move |range, _, _| {
            range
                .map(|index| {
                    let (label, selected, cursor) = if months {
                        let month = index as u32 + 1;
                        (
                            (snap.labels.month)(month),
                            month == snap.month,
                            month == snap.cursor.month(),
                        )
                    } else {
                        let year = first + index as i32;
                        (
                            (snap.labels.number)(year),
                            year == snap.year,
                            year == snap.cursor.year(),
                        )
                    };
                    let mut item = PickerListItem::new(index, label).selected(selected);
                    if specimen {
                        item = item.preview_state(InteractionPreview::Enabled);
                    } else {
                        let state = state.clone();
                        item = item.cursor(snap.focused && cursor).on_click(move |_, cx| {
                            state.update(cx, |s, cx| {
                                if months {
                                    s.choose_month(index as u32 + 1, cx)
                                } else {
                                    s.choose_year(first + index as i32, cx)
                                }
                            })
                        });
                    }
                    item
                })
                .collect()
        })
        .track_scroll(&self.snap.list_scroll)
        .h(rem(LIST_ITEM_HEIGHT * 7.));
        let label = if months {
            self.snap.labels.choose_month.clone()
        } else {
            self.snap.labels.choose_year.clone()
        };
        self.key_host(
            div()
                .id(self.child_id("docked-list-host"))
                .w_full()
                .pr(rem(16.))
                .pt(rem(8.))
                .pb(rem(20.))
                .role(Role::ListBox)
                .aria_label(label)
                .child(list),
        )
    }

    fn docked(self, actions: Option<ActionRow>) -> Stateful<Div> {
        let view = self.snap.view;
        let surface = self
            .surface(DATE_PICKER_WIDTH, 16.)
            .shadow(elevation::level(3))
            .child(self.docked_row());
        match view {
            DatePickerView::Day => surface
                .child(self.day_grid(true).pb(rem(4.)))
                .when_some(actions, |s, actions| {
                    s.child(self.action_row(actions, false))
                }),
            DatePickerView::Month => surface.child(self.docked_list(true)),
            DatePickerView::Year => surface.child(self.docked_list(false)),
        }
    }

    /// The full-screen range picker: close and save, the header, a fixed
    /// weekday row and a vertically scrolling, virtualized month list.
    fn full_screen(self, actions: ActionRow, height: f32) -> Stateful<Div> {
        let l = self.snap.labels.clone();
        let cancel = actions.on_cancel.clone();
        let confirm = actions.on_confirm.clone();
        let state = self.state.clone();
        let mut close = IconButton::new(self.child_id("close"), icon("close"), l.close.clone())
            .on_click(move |_, window, cx| {
                if let Some(f) = &cancel {
                    f(window, cx);
                }
            });
        if let Some(preview) = self.preview() {
            close = close.preview_state(preview);
        }
        let save = self.text_button("save", l.save.clone(), move |window, cx| {
            let value = state.read(cx).selection;
            if let Some(f) = &confirm {
                f(value, window, cx);
            }
        });
        let header = div()
            .w_full()
            .flex()
            .flex_col()
            .pt(rem(12.))
            .border_b(rem(1.))
            .border_color(color(ColorRole::OUTLINE_VARIANT))
            .child(
                div()
                    .h(rem(48.))
                    .flex()
                    .items_center()
                    .justify_between()
                    .pr(rem(12.))
                    .child(close)
                    .child(save),
            )
            .child(
                div()
                    .h(rem(72.))
                    .flex()
                    .items_end()
                    .gap(rem(8.))
                    .pl(rem(64.))
                    .pr(rem(12.))
                    .pb(rem(12.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(rem(8.))
                            .pb(rem(4.))
                            .child(label_text(
                                "title",
                                l.title.clone(),
                                TypeRole::LabelLarge,
                                color(ColorRole::ON_SURFACE_VARIANT),
                            ))
                            .child(
                                label_text(
                                    "headline",
                                    self.snap.headline(),
                                    TypeRole::TitleLarge,
                                    color(ColorRole::ON_SURFACE),
                                )
                                .role(Role::Heading),
                            ),
                    )
                    .child(self.icon_button(
                        "mode",
                        "edit",
                        l.switch_to_input.clone(),
                        |s, window, cx| s.toggle_mode(window, cx),
                    )),
            );
        let snap = self.snap.clone();
        let first = *snap.years.start();
        let count = ((snap.years.end() - first + 1).max(0) as usize) * 12;
        let me = Rc::new(PickerCx {
            id: self.id.clone(),
            state: self.state.clone(),
            snap: snap.clone(),
            specimen: self.specimen,
            reduce_motion: self.reduce_motion,
        });
        let months = uniform_list(self.child_id("months"), count, move |range, _, _| {
            range
                .map(|index| {
                    let year = first + (index / 12) as i32;
                    let month = (index % 12) as u32 + 1;
                    div()
                        .id(index)
                        .flex()
                        .flex_col()
                        .px(rem(GRID_PADDING))
                        .child(
                            div()
                                .h(rem(48.))
                                .pt(rem(16.))
                                .px(rem(24.))
                                .pb(rem(12.))
                                .child(label_text(
                                    "month",
                                    (me.snap.labels.month_year)(year, month),
                                    TypeRole::TitleSmall,
                                    color(ColorRole::ON_SURFACE_VARIANT),
                                )),
                        )
                        .child(me.weeks(year, month, false, true, &format!("m{index}")))
                })
                .collect()
        })
        .track_scroll(&self.snap.month_scroll)
        .flex_1();
        let grid = self.key_host(
            div()
                .id(self.child_id("month-list"))
                .flex_1()
                .min_h_0()
                .flex()
                .flex_col()
                .role(Role::Grid)
                .aria_label(l.title.clone())
                .child(months),
        );
        div()
            .id(self.id.clone())
            .w(rem(DATE_PICKER_WIDTH))
            .h(rem(height))
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
            .role(Role::Dialog)
            .child(header)
            .child(div().px(rem(GRID_PADDING)).child(self.weekday_row()))
            .child(grid)
            .child(self.action_row(actions, true).border_t(rem(0.)))
    }
}

/// Today's date on the local clock, for a picker's today marker.
pub fn today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

/// A modal host for pickers: a scrim and a centered dialog over the
/// window, built on GPUI Base's dialog (focus trap, Escape and outside
/// press dismissal). The surface enters on the default spatial spring.
#[derive(gpui::IntoElement)]
pub struct PickerDialog {
    id: ElementId,
    open: bool,
    content: AnyElement,
    on_dismiss: Option<Activate>,
}

impl PickerDialog {
    pub fn new(id: impl Into<ElementId>, open: bool, content: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            open,
            content: content.into_any_element(),
            on_dismiss: None,
        }
    }
    /// Escape or a press on the scrim.
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for PickerDialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.open {
            return div().into_any_element();
        }
        let focus = window
            .use_keyed_state((self.id.clone(), "dialog-focus"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        let dismiss = self.on_dismiss.clone();
        let mut scrim = color(ColorRole::SCRIM);
        scrim.a *= 0.32;
        let surface = div()
            .rounded(rem(28.))
            .shadow(elevation::level(3))
            .child(self.content);
        gpui_base::Dialog::new(cx)
            .focus_handle(focus)
            .open(true)
            .on_open_change(move |open, _, window, cx| {
                if !open && let Some(f) = &dismiss {
                    f(window, cx);
                }
            })
            .backdrop(div().size_full().bg(scrim).occlude())
            .popup(motion::state_enter((self.id, "enter"), surface))
            .into_any_element()
    }
}

#[cfg(test)]
#[path = "date_picker_tests.rs"]
mod tests;
