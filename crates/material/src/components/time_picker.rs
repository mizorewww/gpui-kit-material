//! M3 time pickers: the dial picker (vertical and horizontal, 12- and
//! 24-hour) and the keyboard input picker.
//!
//! Reference: <https://m3.material.io/components/time-pickers/specs> and the
//! Material 3 Design Kit "Date & time pickers" page (inspected tokens: a
//! surface-container-high container with 28dp corners; a label-medium
//! title; 96×80 display-large time selectors on primary-container
//! (selected) or surface-container-highest; a 1dp outline period selector,
//! 52×80 vertical or 216×38 horizontal, selected half on tertiary-container
//! with title-medium labels; a 256dp surface-container-highest clock face
//! with 48dp body-large numbers, the outer ring's selector 104dp and the
//! 24-hour inner ring's 76dp from the center, a 2dp primary hand, an 8dp
//! center dot and a 48dp primary selector; 96×72 display-medium input
//! fields with a 2dp primary outline while editing and a body-small label).
//!
//! Behavior: keyboard text entry is GPUI Base's segmented
//! [`gpui_base::TimeField`] (digits, arrows, Tab between segments, A/P for
//! the period), decorated as Material input fields. The dial is a slider:
//! pointer press and drag pick the nearest value (the inner ring for
//! 13–23 and 00 on a 24-hour face), releasing on an hour moves on to the
//! minutes, and the arrow keys step the value. Strings and number
//! formatting come from the caller's [`TimePickerLabels`].
//!
//! md3.io's options beyond the kit: [`TimePickerState::seconds`] adds a
//! seconds selector, dial unit and input segment; a minute step snaps the
//! dial and its arrow keys to 5- or 15-minute marks; earliest/latest
//! limits clamp every edit (dial, keys, period and typing) and fade the
//! dial numbers that fall outside them.
//!
//! Motion: the hand turns along the shorter arc and changes rings on the
//! default spatial spring, retargeting from wherever it is; reduced motion
//! moves it at once.
use super::InteractionPreview;
use super::button::ButtonFocusRing;
use super::date_parts::Activate;
use super::ripple::Ripple;
use super::text_field::type_style;
use super::{ButtonVariant, IconButton, material_button};
use crate::{
    color::ColorRole,
    icons::{Icon, Symbol},
    motion::{self, MotionToken},
    theme::color,
    tokens::{
        state,
        typography::{TypeRole, rem},
    },
};
use chrono::{NaiveTime, Timelike};
use gpui::{prelude::*, *};
use gpui_base::{HourCycle, TimeField, TimeFieldEvent, TimeFieldState, TimePrecision, TimeSegment};
use std::{cell::Cell, f32::consts::PI, rc::Rc};

type Format = Rc<dyn Fn(u32) -> SharedString>;
type PeriodChange = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type TimeConfirm = Rc<dyn Fn(NaiveTime, &mut Window, &mut App)>;

/// Every caller-supplied string of a time picker.
#[derive(Clone)]
pub struct TimePickerLabels {
    /// The dial picker's title, e.g. "Select time".
    pub select_title: SharedString,
    /// The input picker's title, e.g. "Enter time".
    pub enter_title: SharedString,
    pub hour: SharedString,
    pub minute: SharedString,
    pub am: SharedString,
    pub pm: SharedString,
    pub confirm: SharedString,
    pub cancel: SharedString,
    /// Accessible names of the mode switch.
    pub switch_to_input: SharedString,
    pub switch_to_dial: SharedString,
    /// A dial number ("7", "13").
    pub number: Format,
    /// A two-digit field value ("07", "05").
    pub two_digits: Format,
}

/// 12-hour (with a period selector) or 24-hour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeFormat {
    #[default]
    TwelveHour,
    TwentyFourHour,
}

/// The dial picker's layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeOrientation {
    #[default]
    Vertical,
    Horizontal,
}

/// Dial or keyboard input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeInputMode {
    #[default]
    Dial,
    Input,
}

/// Which value the dial edits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeUnit {
    #[default]
    Hour,
    Minute,
    /// With [`TimePickerState::seconds`].
    Second,
}

/// A time picker's state. Create it with `cx.new` and hand it to
/// [`TimePicker`].
pub struct TimePickerState {
    labels: Rc<TimePickerLabels>,
    time: NaiveTime,
    format: TimeFormat,
    mode: TimeInputMode,
    unit: TimeUnit,
    /// The hand's unwrapped target angle in turns, so a change turns along
    /// the shorter arc.
    hand: f32,
    /// The minute grid the dial and its keys snap to (1 = every minute).
    minute_step: u32,
    /// The seconds selector's label, when seconds are shown.
    second_label: Option<SharedString>,
    /// Earliest and latest selectable times.
    limits: Option<(NaiveTime, NaiveTime)>,
    dial_focus: FocusHandle,
    field: Entity<TimeFieldState>,
    _subscription: Subscription,
}

/// Events a picker emits as the person edits it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimePickerEvent {
    Change(NaiveTime),
}

impl EventEmitter<TimePickerEvent> for TimePickerState {}

impl TimePickerState {
    pub fn new(
        labels: TimePickerLabels,
        time: NaiveTime,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let field = cx.new(|cx| {
            let mut field = TimeFieldState::new(window, cx).hour_cycle(HourCycle::H12);
            field.set_time(time, window, cx);
            field
        });
        let subscription = cx.subscribe_in(
            &field,
            window,
            |this, field, event: &TimeFieldEvent, window, cx| {
                let TimeFieldEvent::Change(typed) = *event;
                // A typed time outside the limits lands on the nearest one.
                let time = this.clamp(typed);
                if time != typed {
                    field.update(cx, |field, cx| field.set_time(time, window, cx));
                }
                this.apply(time);
                cx.emit(TimePickerEvent::Change(time));
                cx.notify();
            },
        );
        let mut state = Self {
            labels: Rc::new(labels),
            time,
            format: TimeFormat::TwelveHour,
            mode: TimeInputMode::Dial,
            unit: TimeUnit::Hour,
            hand: 0.,
            minute_step: 1,
            second_label: None,
            limits: None,
            dial_focus: cx.focus_handle().tab_stop(true),
            field,
            _subscription: subscription,
        };
        state.hand = hand_turns(time, state.unit, None);
        state
    }
    /// Use the 24-hour clock.
    pub fn format(
        mut self,
        format: TimeFormat,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        self.set_format(format, window, cx);
        self
    }
    pub fn input_mode(mut self) -> Self {
        self.mode = TimeInputMode::Input;
        self
    }
    /// Show seconds: a third selector (named `label`), a seconds dial unit
    /// and a seconds input segment.
    pub fn seconds(
        mut self,
        label: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        self.second_label = Some(label.into());
        let time = self.time;
        self.field.update(cx, |field, cx| {
            field.set_precision(TimePrecision::Second, window, cx);
            field.set_time(time, window, cx);
        });
        self
    }
    /// Snap the dial and its arrow keys to every `step` minutes (e.g. 5 or
    /// 15; 0 and 1 mean every minute).
    pub fn minute_step(mut self, step: u32) -> Self {
        self.minute_step = step.clamp(1, 30);
        self
    }
    /// Only times from `earliest` to `latest` can be chosen; the current
    /// time moves inside them.
    pub fn limits(
        mut self,
        earliest: NaiveTime,
        latest: NaiveTime,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        self.limits = Some((earliest.min(latest), earliest.max(latest)));
        let time = self.clamp(self.time);
        if time != self.time {
            self.apply(time);
            self.field
                .update(cx, |field, cx| field.set_time(time, window, cx));
        }
        self
    }
    pub fn unit(mut self, unit: TimeUnit) -> Self {
        self.unit = unit;
        self.hand = hand_turns(self.time, unit, None);
        self
    }

    pub fn labels(&self) -> &TimePickerLabels {
        &self.labels
    }
    pub fn time(&self) -> NaiveTime {
        self.time
    }
    pub fn current_format(&self) -> TimeFormat {
        self.format
    }
    pub fn mode(&self) -> TimeInputMode {
        self.mode
    }
    pub fn current_unit(&self) -> TimeUnit {
        self.unit
    }
    pub fn shows_seconds(&self) -> bool {
        self.second_label.is_some()
    }
    pub fn current_minute_step(&self) -> u32 {
        self.minute_step
    }
    pub fn current_limits(&self) -> Option<(NaiveTime, NaiveTime)> {
        self.limits
    }
    /// `time` moved inside the limits.
    fn clamp(&self, time: NaiveTime) -> NaiveTime {
        clamp_time(time, self.limits)
    }
    pub fn dial_focus(&self) -> &FocusHandle {
        &self.dial_focus
    }
    /// Base's time field, for focusing input mode.
    pub fn field(&self) -> &Entity<TimeFieldState> {
        &self.field
    }

    fn apply(&mut self, time: NaiveTime) {
        self.time = time;
        self.hand = hand_turns(time, self.unit, Some(self.hand));
    }

    pub fn set_format(&mut self, format: TimeFormat, window: &mut Window, cx: &mut Context<Self>) {
        self.format = format;
        let cycle = match format {
            TimeFormat::TwelveHour => HourCycle::H12,
            TimeFormat::TwentyFourHour => HourCycle::H23,
        };
        self.field
            .update(cx, |field, cx| field.set_hour_cycle(cycle, window, cx));
        cx.notify();
    }

    /// Set the time as a person would, emitting [`TimePickerEvent::Change`].
    pub fn set_time(&mut self, time: NaiveTime, window: &mut Window, cx: &mut Context<Self>) {
        let time = self.clamp(time);
        if time == self.time {
            return;
        }
        self.apply(time);
        self.field
            .update(cx, |field, cx| field.set_time(time, window, cx));
        cx.emit(TimePickerEvent::Change(time));
        cx.notify();
    }

    /// Edit `unit` on the dial.
    pub fn set_unit(&mut self, unit: TimeUnit, cx: &mut Context<Self>) {
        if self.unit != unit {
            self.unit = unit;
            self.hand = hand_turns(self.time, unit, Some(self.hand));
            cx.notify();
        }
    }

    /// AM (`false`) or PM (`true`).
    pub fn set_pm(&mut self, pm: bool, window: &mut Window, cx: &mut Context<Self>) {
        let hour = self.time.hour() % 12 + if pm { 12 } else { 0 };
        let time = self.time.with_hour(hour).unwrap_or(self.time);
        self.set_time(time, window, cx);
    }

    pub fn toggle_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.mode = match self.mode {
            TimeInputMode::Dial => {
                let field = self.field.clone();
                window.defer(cx, move |window, cx| {
                    let handle = field.read(cx).focus_handle(cx);
                    handle.focus(window, cx);
                });
                TimeInputMode::Input
            }
            TimeInputMode::Input => TimeInputMode::Dial,
        };
        cx.notify();
    }

    /// Step the dial's value by `delta`, wrapping within the hour or day.
    pub fn step(&mut self, delta: i32, window: &mut Window, cx: &mut Context<Self>) {
        let time = match self.unit {
            TimeUnit::Minute if self.minute_step > 1 => {
                let minute = step_minutes(self.time.minute(), self.minute_step, delta);
                self.time.with_minute(minute).unwrap_or(self.time)
            }
            unit => step_time(self.time, unit, self.format, delta),
        };
        self.set_time(time, window, cx);
    }

    /// Choose the dial value nearest a pointer at `offset` from the face's
    /// center (in dp).
    fn pick(&mut self, offset: Point<f32>, window: &mut Window, cx: &mut Context<Self>) {
        let mut time = dial_pick(self.time, self.unit, self.format, offset);
        if self.unit == TimeUnit::Minute && self.minute_step > 1 {
            let step = self.minute_step as f32;
            let minute = ((time.minute() as f32 / step).round() * step) as u32 % 60;
            time = time.with_minute(minute).unwrap_or(time);
        }
        self.set_time(time, window, cx);
    }
}

impl Render for TimePickerState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

/// Radii of the dial's rings, from the face's center to a number's center.
pub const DIAL_SIZE: f32 = 256.;
const OUTER_RADIUS: f32 = 104.;
const INNER_RADIUS: f32 = 76.;
const NUMBER_CELL: f32 = 48.;

/// The angle in turns (clockwise from 12 o'clock) of `time`'s `unit`.
fn angle_of(time: NaiveTime, unit: TimeUnit) -> f32 {
    match unit {
        TimeUnit::Hour => (time.hour() % 12) as f32 / 12.,
        TimeUnit::Minute => time.minute() as f32 / 60.,
        TimeUnit::Second => time.second() as f32 / 60.,
    }
}

/// `time` moved inside `limits` (earliest, latest).
pub(crate) fn clamp_time(time: NaiveTime, limits: Option<(NaiveTime, NaiveTime)>) -> NaiveTime {
    match limits {
        Some((earliest, latest)) => time.max(earliest).min(latest),
        None => time,
    }
}

/// Step `minute` by `delta` marks of a `step`-minute grid: an off-grid
/// minute first moves to the neighboring mark in the step's direction.
pub(crate) fn step_minutes(minute: u32, step: u32, delta: i32) -> u32 {
    let (minute, step) = (minute as i32, step.max(1) as i32);
    let floor = minute.div_euclid(step) * step;
    let next = if floor == minute || delta > 0 {
        floor + step * delta
    } else {
        floor + step * (delta + 1)
    };
    next.rem_euclid(60) as u32
}

/// Whether some time in hour `hour` (or, for minutes and seconds, `time`
/// with that value) lies inside `limits`.
pub(crate) fn within_limits(
    time: NaiveTime,
    unit: TimeUnit,
    value: u32,
    limits: Option<(NaiveTime, NaiveTime)>,
) -> bool {
    let Some((earliest, latest)) = limits else {
        return true;
    };
    match unit {
        TimeUnit::Hour => {
            let start = NaiveTime::from_hms_opt(value, 0, 0);
            let end = NaiveTime::from_hms_opt(value, 59, 59);
            matches!((start, end), (Some(start), Some(end)) if start <= latest && end >= earliest)
        }
        TimeUnit::Minute => match time.with_minute(value) {
            Some(t) => {
                let (start, end) = (
                    t.with_second(0).unwrap_or(t),
                    t.with_second(59).unwrap_or(t),
                );
                start <= latest && end >= earliest
            }
            None => false,
        },
        TimeUnit::Second => time
            .with_second(value)
            .is_some_and(|t| t >= earliest && t <= latest),
    }
}

/// The unwrapped target turn nearest `previous`.
pub(crate) fn hand_turns(time: NaiveTime, unit: TimeUnit, previous: Option<f32>) -> f32 {
    let angle = angle_of(time, unit);
    match previous {
        None => angle,
        Some(previous) => {
            let delta = (angle - previous).rem_euclid(1.);
            let delta = if delta > 0.5 { delta - 1. } else { delta };
            previous + delta
        }
    }
}

/// Whether the hand sits on the 24-hour face's inner ring (00 and 13–23).
pub(crate) fn inner_ring(time: NaiveTime, unit: TimeUnit, format: TimeFormat) -> bool {
    format == TimeFormat::TwentyFourHour
        && unit == TimeUnit::Hour
        && (time.hour() == 0 || time.hour() > 12)
}

pub(crate) fn step_time(
    time: NaiveTime,
    unit: TimeUnit,
    format: TimeFormat,
    delta: i32,
) -> NaiveTime {
    match unit {
        TimeUnit::Minute => {
            let minute = (time.minute() as i32 + delta).rem_euclid(60) as u32;
            time.with_minute(minute).unwrap_or(time)
        }
        TimeUnit::Second => {
            let second = (time.second() as i32 + delta).rem_euclid(60) as u32;
            time.with_second(second).unwrap_or(time)
        }
        TimeUnit::Hour => {
            let hour = match format {
                TimeFormat::TwentyFourHour => (time.hour() as i32 + delta).rem_euclid(24) as u32,
                // A 12-hour dial steps within the period.
                TimeFormat::TwelveHour => {
                    time.hour() / 12 * 12 + (time.hour() as i32 % 12 + delta).rem_euclid(12) as u32
                }
            };
            time.with_hour(hour).unwrap_or(time)
        }
    }
}

/// The value nearest `offset` (dp from the center, y down).
pub(crate) fn dial_pick(
    time: NaiveTime,
    unit: TimeUnit,
    format: TimeFormat,
    offset: Point<f32>,
) -> NaiveTime {
    let turn = (offset.x.atan2(-offset.y) / (2. * PI)).rem_euclid(1.);
    match unit {
        TimeUnit::Minute => {
            let minute = (turn * 60.).round() as u32 % 60;
            time.with_minute(minute).unwrap_or(time)
        }
        TimeUnit::Second => {
            let second = (turn * 60.).round() as u32 % 60;
            time.with_second(second).unwrap_or(time)
        }
        TimeUnit::Hour => {
            let position = (turn * 12.).round() as u32 % 12;
            let hour = match format {
                TimeFormat::TwelveHour => time.hour() / 12 * 12 + position,
                TimeFormat::TwentyFourHour => {
                    let distance = (offset.x * offset.x + offset.y * offset.y).sqrt();
                    let inner = distance < (OUTER_RADIUS + INNER_RADIUS) / 2.;
                    match (inner, position) {
                        (true, 0) => 0,
                        (true, p) => p + 12,
                        (false, 0) => 12,
                        (false, p) => p,
                    }
                }
            };
            time.with_hour(hour).unwrap_or(time)
        }
    }
}

fn point_on(turn: f32, radius: f32) -> (f32, f32) {
    let a = turn * 2. * PI;
    (
        DIAL_SIZE / 2. + radius * a.sin(),
        DIAL_SIZE / 2. - radius * a.cos(),
    )
}

/// The 256dp clock face (`.Building Blocks/Clock face`): numbers, the
/// hand and the selector. Interactive over a [`TimePickerState`], or a
/// static specimen of `time`.
#[derive(gpui::IntoElement)]
pub struct ClockDial {
    id: ElementId,
    state: Option<Entity<TimePickerState>>,
    time: NaiveTime,
    unit: TimeUnit,
    format: TimeFormat,
    labels: Rc<TimePickerLabels>,
    second_label: Option<SharedString>,
    limits: Option<(NaiveTime, NaiveTime)>,
}

impl ClockDial {
    /// A live dial editing `state`.
    pub fn new(id: impl Into<ElementId>, state: &Entity<TimePickerState>, cx: &App) -> Self {
        let s = state.read(cx);
        Self {
            id: id.into(),
            state: Some(state.clone()),
            time: s.time,
            unit: s.unit,
            format: s.format,
            labels: s.labels.clone(),
            second_label: s.second_label.clone(),
            limits: s.limits,
        }
    }
    /// A static specimen of `time`.
    pub fn specimen(
        id: impl Into<ElementId>,
        labels: Rc<TimePickerLabels>,
        time: NaiveTime,
        unit: TimeUnit,
        format: TimeFormat,
    ) -> Self {
        Self {
            id: id.into(),
            state: None,
            time,
            unit,
            format,
            labels,
            second_label: None,
            limits: None,
        }
    }
}

impl RenderOnce for ClockDial {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (time, unit, format) = (self.time, self.unit, self.format);
        let inner = inner_ring(time, unit, format);
        let target_radius = if inner { INNER_RADIUS } else { OUTER_RADIUS };
        let (turn, radius) = match &self.state {
            Some(state) => {
                let target = state.read(cx).hand;
                (
                    motion::token_value(
                        (self.id.clone(), "hand"),
                        target,
                        MotionToken::DefaultSpatial,
                        window,
                        cx,
                    ),
                    motion::token_value(
                        (self.id.clone(), "radius"),
                        target_radius,
                        MotionToken::DefaultSpatial,
                        window,
                        cx,
                    ),
                )
            }
            None => (angle_of(time, unit), target_radius),
        };
        let primary = color(ColorRole::PRIMARY);
        let on_primary = color(ColorRole::ON_PRIMARY);
        let between = match unit {
            TimeUnit::Minute => time.minute() % 5 != 0,
            TimeUnit::Second => time.second() % 5 != 0,
            TimeUnit::Hour => false,
        };
        let bounds = Rc::new(Cell::new(Bounds::<Pixels>::default()));
        let observed = bounds.clone();
        let hand = canvas(
            move |b, _, _| observed.set(b),
            move |b, _, window, _| {
                let scale = b.size.width / DIAL_SIZE;
                let at = |(x, y): (f32, f32)| b.origin + point(scale * x, scale * y);
                let center = at((DIAL_SIZE / 2., DIAL_SIZE / 2.));
                let tip = at(point_on(turn, radius));
                let mut line = PathBuilder::stroke(scale * 2.);
                line.move_to(center);
                line.line_to(tip);
                if let Ok(path) = line.build() {
                    window.paint_path(path, primary);
                }
                let circle = |c: Point<Pixels>, r: Pixels, paint: Rgba, window: &mut Window| {
                    window.paint_quad(
                        fill(Bounds::new(c - point(r, r), size(r * 2., r * 2.)), paint)
                            .corner_radii(r),
                    );
                };
                circle(tip, scale * (NUMBER_CELL / 2.), primary, window);
                circle(center, scale * 4., primary, window);
                if between {
                    circle(tip, scale * 2., on_primary, window);
                }
            },
        )
        .absolute()
        .inset_0();
        let numbers: Vec<(u32, f32, f32, SharedString)> = match (unit, format) {
            (TimeUnit::Minute | TimeUnit::Second, _) => (0..12)
                .map(|i| {
                    (
                        i * 5,
                        i as f32 / 12.,
                        OUTER_RADIUS,
                        (self.labels.two_digits)(i * 5),
                    )
                })
                .collect(),
            (TimeUnit::Hour, TimeFormat::TwelveHour) => (0..12)
                .map(|i| {
                    let label = if i == 0 { 12 } else { i };
                    (
                        label,
                        i as f32 / 12.,
                        OUTER_RADIUS,
                        (self.labels.number)(label),
                    )
                })
                .collect(),
            (TimeUnit::Hour, TimeFormat::TwentyFourHour) => (0..12)
                .map(|i| {
                    let label = if i == 0 { 12 } else { i };
                    (
                        label,
                        i as f32 / 12.,
                        OUTER_RADIUS,
                        (self.labels.number)(label),
                    )
                })
                .chain((0..12).map(|i| {
                    let hour = if i == 0 { 0 } else { i + 12 };
                    (
                        hour,
                        i as f32 / 12.,
                        INNER_RADIUS,
                        (self.labels.two_digits)(hour),
                    )
                }))
                .collect(),
        };
        let selected = match (unit, format) {
            (TimeUnit::Minute, _) => time.minute(),
            (TimeUnit::Second, _) => time.second(),
            (TimeUnit::Hour, TimeFormat::TwelveHour) => (time.hour() + 11) % 12 + 1,
            (TimeUnit::Hour, TimeFormat::TwentyFourHour) => time.hour(),
        };
        let ink = color(ColorRole::ON_SURFACE);
        let faded = state::disabled(ink);
        let limits = self.limits;
        // A 12-hour face's number names an hour in the current period.
        let hour_of = move |label: u32| match format {
            TimeFormat::TwelveHour => label % 12 + time.hour() / 12 * 12,
            TimeFormat::TwentyFourHour => label,
        };
        let mut face = div()
            .id(self.id.clone())
            .relative()
            .flex_shrink_0()
            .size(rem(DIAL_SIZE))
            .rounded_full()
            .bg(color(ColorRole::SURFACE_CONTAINER_HIGHEST))
            .role(Role::Slider)
            .aria_label(match unit {
                TimeUnit::Hour => self.labels.hour.clone(),
                TimeUnit::Minute => self.labels.minute.clone(),
                TimeUnit::Second => self
                    .second_label
                    .clone()
                    .unwrap_or_else(|| self.labels.minute.clone()),
            })
            .aria_numeric_value(selected as f64)
            .child(hand)
            .children(numbers.into_iter().map(|(value, turn, r, label)| {
                let (x, y) = point_on(turn, r);
                div()
                    .absolute()
                    .left(rem(x - NUMBER_CELL / 2.))
                    .top(rem(y - NUMBER_CELL / 2.))
                    .size(rem(NUMBER_CELL))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(if value == selected {
                        on_primary
                    } else {
                        let value = if unit == TimeUnit::Hour {
                            hour_of(value)
                        } else {
                            value
                        };
                        if within_limits(time, unit, value, limits) {
                            ink
                        } else {
                            faded
                        }
                    })
                    .map(|d| type_style(d, TypeRole::BodyLarge))
                    .child(label)
            }));
        let Some(state) = self.state else {
            return face.into_any_element();
        };
        let focus = state.read(cx).dial_focus.clone();
        let pressed = window.use_keyed_state((self.id.clone(), "dragging"), cx, |_, _| false);
        let offset = move |position: Point<Pixels>, b: Bounds<Pixels>| {
            let scale = f32::from(b.size.width) / DIAL_SIZE;
            let c = b.center();
            point(
                f32::from(position.x - c.x) / scale,
                f32::from(position.y - c.y) / scale,
            )
        };
        face = face
            .track_focus(&focus)
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, {
                let (state, bounds, pressed) = (state.clone(), bounds.clone(), pressed.clone());
                let focus = focus.clone();
                move |e, window, cx| {
                    focus.focus(window, cx);
                    pressed.update(cx, |p, _| *p = true);
                    let o = offset(e.position, bounds.get());
                    state.update(cx, |s, cx| s.pick(o, window, cx));
                }
            })
            .on_mouse_move({
                let (state, bounds, pressed) = (state.clone(), bounds.clone(), pressed.clone());
                move |e, window, cx| {
                    if *pressed.read(cx) && e.pressed_button == Some(MouseButton::Left) {
                        let o = offset(e.position, bounds.get());
                        state.update(cx, |s, cx| s.pick(o, window, cx));
                    }
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let (state, pressed) = (state.clone(), pressed.clone());
                move |_, _, cx| {
                    if *pressed.read(cx) {
                        pressed.update(cx, |p, _| *p = false);
                        state.update(cx, |s, cx| {
                            // Releasing moves on to the next unit.
                            let next = match s.unit {
                                TimeUnit::Hour => TimeUnit::Minute,
                                TimeUnit::Minute if s.shows_seconds() => TimeUnit::Second,
                                unit => unit,
                            };
                            s.set_unit(next, cx)
                        });
                    }
                }
            })
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                pressed.update(cx, |p, _| *p = false);
            })
            .on_key_down(move |e, window, cx| {
                let delta = match e.keystroke.key.as_str() {
                    "up" | "right" => 1,
                    "down" | "left" => -1,
                    _ => return,
                };
                cx.stop_propagation();
                state.update(cx, |s, cx| s.step(delta, window, cx));
            });
        ButtonFocusRing::new(face, focus, true, rem(DIAL_SIZE / 2.)).into_any_element()
    }
}

/// A dial picker's hour or minute selector (`.Building Blocks/Input`): a
/// 96×80 display-large box, primary-container while its unit is on the dial.
#[derive(gpui::IntoElement)]
pub struct TimeSelector {
    id: ElementId,
    value: SharedString,
    label: SharedString,
    selected: bool,
    compact: bool,
    preview: Option<InteractionPreview>,
    on_click: Option<Activate>,
}

/// A compact selector's width: three of them (with seconds) fit where the
/// kit's two 96dp selectors and a period selector do.
const COMPACT_SELECTOR: f32 = 72.;

impl TimeSelector {
    /// `value` is the formatted number; `label` names it ("Hour").
    pub fn new(
        id: impl Into<ElementId>,
        value: impl Into<SharedString>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            label: label.into(),
            selected: false,
            compact: false,
            preview: None,
            on_click: None,
        }
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    /// 72dp wide with display-medium digits, for hours, minutes and seconds
    /// side by side.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for TimeSelector {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (fill, ink) = if self.selected {
            (
                color(ColorRole::PRIMARY_CONTAINER),
                color(ColorRole::ON_PRIMARY_CONTAINER),
            )
        } else {
            (
                color(ColorRole::SURFACE_CONTAINER_HIGHEST),
                color(ColorRole::ON_SURFACE),
            )
        };
        let preview = self.preview;
        let disabled = preview.is_some_and(InteractionPreview::is_disabled);
        let hover = state::hovered(fill, ink);
        let press = state::pressed(fill, ink);
        let focus = window
            .use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let ripple_id = ElementId::NamedChild(self.id.clone().into(), "ripple".into());
        let mut button = gpui_base::Button::new(self.id)
            .accessibility_label(self.label)
            .disabled(disabled)
            .w(rem(if self.compact { COMPACT_SELECTOR } else { 96. }))
            .h(rem(80.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(rem(8.))
            .bg(fill)
            .text_color(ink)
            .map(|b| {
                type_style(
                    b,
                    if self.compact {
                        TypeRole::DisplayMedium
                    } else {
                        TypeRole::DisplayLarge
                    },
                )
            })
            .when(disabled, |b| b.opacity(state::DISABLED_CONTENT))
            .child(self.value);
        button = match preview {
            Some(InteractionPreview::Hovered) => button.bg(hover),
            Some(InteractionPreview::Focused) => button.bg(state::layer(fill, ink, state::FOCUS)),
            Some(InteractionPreview::Pressed) => button.bg(press),
            Some(_) => button,
            None => button
                .cursor_pointer()
                .hover(move |s| s.bg(hover))
                .active(move |s| s.bg(press)),
        };
        if let Some(f) = self.on_click.filter(|_| preview.is_none()) {
            button = button.on_click(move |_, window, cx| f(window, cx));
        }
        let button = button.track_focus(&focus).tab_stop(preview.is_none());
        let button = Ripple::new(ripple_id, button)
            .corners(Corners::all(rem(8.).to_pixels(window.rem_size())))
            .ink(ink)
            .disabled(disabled)
            .specimen(preview.is_some());
        ButtonFocusRing::new(button, focus, !disabled, rem(8.))
            .forced(!disabled && preview == Some(InteractionPreview::Focused))
    }
}

/// The AM/PM selector (`.Building Blocks/Period Selector`), stacked
/// (52 wide) or side by side (216 wide).
#[derive(gpui::IntoElement)]
pub struct PeriodSelector {
    id: ElementId,
    am: SharedString,
    pm: SharedString,
    is_pm: bool,
    horizontal: bool,
    height: f32,
    preview: Option<(bool, InteractionPreview)>,
    on_change: Option<PeriodChange>,
}

impl PeriodSelector {
    pub fn new(
        id: impl Into<ElementId>,
        am: impl Into<SharedString>,
        pm: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            am: am.into(),
            pm: pm.into(),
            is_pm: false,
            horizontal: false,
            height: 80.,
            preview: None,
            on_change: None,
        }
    }
    pub fn pm(mut self, pm: bool) -> Self {
        self.is_pm = pm;
        self
    }
    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }
    /// The stacked selector's height (80 beside dial selectors, 72 beside
    /// input fields).
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
    /// Force `state` on the AM (`false`) or PM (`true`) half.
    pub fn preview_state(mut self, pm: bool, state: InteractionPreview) -> Self {
        self.preview = Some((pm, state));
        self
    }
    pub fn on_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for PeriodSelector {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let outline = color(ColorRole::OUTLINE);
        let specimen = self.preview.is_some();
        let half = |pm: bool, label: SharedString, window: &mut Window, cx: &mut App| {
            let selected = pm == self.is_pm;
            let (fill, ink) = if selected {
                (
                    color(ColorRole::TERTIARY_CONTAINER),
                    color(ColorRole::ON_TERTIARY_CONTAINER),
                )
            } else {
                let ink = color(ColorRole::ON_SURFACE_VARIANT);
                (Rgba { a: 0., ..ink }, ink)
            };
            let forced = self.preview.filter(|(p, _)| *p == pm).map(|(_, s)| s);
            let hover = state::hovered(fill, ink);
            let press = state::pressed(fill, ink);
            let focus = window
                .use_keyed_state(
                    (self.id.clone(), if pm { "pm" } else { "am" }),
                    cx,
                    |_, cx| cx.focus_handle(),
                )
                .read(cx)
                .clone();
            let mut button = gpui_base::Button::new(if pm { "pm" } else { "am" })
                .accessibility_label(label.clone())
                .role(Role::RadioButton)
                .aria_toggled(if selected {
                    accesskit::Toggled::True
                } else {
                    accesskit::Toggled::False
                })
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .bg(fill)
                .text_color(ink)
                .map(|b| type_style(b, TypeRole::TitleMedium))
                .child(label);
            button = match forced {
                Some(InteractionPreview::Hovered) => button.bg(hover),
                Some(InteractionPreview::Focused) => {
                    button.bg(state::layer(fill, ink, state::FOCUS))
                }
                Some(InteractionPreview::Pressed) => button.bg(press),
                Some(_) => button,
                None if !specimen => button
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover))
                    .active(move |s| s.bg(press)),
                None => button,
            };
            if let Some(f) = self.on_change.clone().filter(|_| !specimen) {
                button = button.on_click(move |_, window, cx| f(pm, window, cx));
            }
            let button = button.track_focus(&focus).tab_stop(!specimen);
            let button = Ripple::new(if pm { "pm-ripple" } else { "am-ripple" }, button)
                .ink(ink)
                .specimen(specimen);
            ButtonFocusRing::new(button, focus, true, rem(0.))
                .inward()
                .forced(forced == Some(InteractionPreview::Focused))
        };
        let am = half(false, self.am.clone(), window, cx);
        let pm = half(true, self.pm.clone(), window, cx);
        let divider = div().flex_shrink_0().bg(outline);
        div()
            .id(self.id.clone())
            .flex()
            .flex_shrink_0()
            .overflow_hidden()
            .rounded(rem(8.))
            .border(rem(1.))
            .border_color(outline)
            .role(Role::RadioGroup)
            .map(|d| {
                if self.horizontal {
                    d.flex_row()
                        .w(rem(216.))
                        .h(rem(38.))
                        .child(am)
                        .child(divider.w(rem(1.)).h_full())
                        .child(pm)
                } else {
                    d.flex_col()
                        .w(rem(52.))
                        .h(rem(self.height))
                        .child(am)
                        .child(divider.h(rem(1.)).w_full())
                        .child(pm)
                }
            })
    }
}

/// An M3 time picker over a [`TimePickerState`].
#[derive(gpui::IntoElement)]
pub struct TimePicker {
    id: ElementId,
    state: Entity<TimePickerState>,
    orientation: TimeOrientation,
    specimen: bool,
    on_confirm: Option<TimeConfirm>,
    on_cancel: Option<Activate>,
}

impl TimePicker {
    pub fn new(id: impl Into<ElementId>, state: &Entity<TimePickerState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            orientation: TimeOrientation::Vertical,
            specimen: false,
            on_confirm: None,
            on_cancel: None,
        }
    }
    pub fn orientation(mut self, orientation: TimeOrientation) -> Self {
        self.orientation = orientation;
        self
    }
    /// A static specimen: no tab stops, no motion, no pointer input.
    pub fn specimen(mut self) -> Self {
        self.specimen = true;
        self
    }
    pub fn on_confirm(mut self, f: impl Fn(NaiveTime, &mut Window, &mut App) + 'static) -> Self {
        self.on_confirm = Some(Rc::new(f));
        self
    }
    pub fn on_cancel(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Rc::new(f));
        self
    }
}

fn named(id: &ElementId, name: &'static str) -> ElementId {
    ElementId::NamedChild(id.clone().into(), name.into())
}

fn symbol(name: &str) -> Icon {
    Icon::from(Symbol::named(name).expect("a Material Symbol in the pinned catalog"))
}

impl RenderOnce for TimePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = self.state.read(cx);
        let labels = s.labels.clone();
        let (time, format, mode, unit) = (s.time, s.format, s.mode, s.unit);
        let second_label = s.second_label.clone();
        let seconds = second_label.is_some();
        let field = s.field.clone();
        let id = self.id.clone();
        let specimen = self.specimen;
        let preview = specimen.then_some(InteractionPreview::Enabled);
        let twelve = format == TimeFormat::TwelveHour;
        let horizontal =
            self.orientation == TimeOrientation::Horizontal && mode == TimeInputMode::Dial;
        let update = |f: fn(&mut TimePickerState, &mut Window, &mut Context<TimePickerState>)| {
            let state = self.state.clone();
            move |window: &mut Window, cx: &mut App| state.update(cx, |s, cx| f(s, window, cx))
        };
        let state = self.state.clone();
        let period = |height: f32, horizontal: bool| {
            let state = state.clone();
            let mut period =
                PeriodSelector::new(named(&id, "period"), labels.am.clone(), labels.pm.clone())
                    .pm(time.hour() >= 12)
                    .horizontal(horizontal)
                    .height(height)
                    .on_change(move |pm, window, cx| {
                        state.update(cx, |s, cx| s.set_pm(pm, window, cx))
                    });
            if specimen {
                period = period.preview_state(false, InteractionPreview::Enabled);
            }
            period
        };
        let display_hour = if twelve {
            (time.hour() + 11) % 12 + 1
        } else {
            time.hour()
        };
        let selector = |name: &'static str, which: TimeUnit, value: u32, label: SharedString| {
            let state = self.state.clone();
            let mut selector =
                TimeSelector::new(named(&id, name), (labels.two_digits)(value), label)
                    .selected(unit == which)
                    .compact(seconds)
                    .on_click(move |_, cx| state.update(cx, |s, cx| s.set_unit(which, cx)));
            if let Some(p) = preview {
                selector = selector.preview_state(p);
            }
            selector
        };
        let separator = |height: f32| {
            div()
                .w(rem(24.))
                .h(rem(height))
                .flex()
                .items_center()
                .justify_center()
                .text_color(color(ColorRole::ON_SURFACE))
                .map(|d| type_style(d, TypeRole::DisplayLarge))
                .child(":")
        };
        let time_row = div()
            .flex()
            .child(selector(
                "hour",
                TimeUnit::Hour,
                display_hour,
                labels.hour.clone(),
            ))
            .child(separator(80.))
            .child(selector(
                "minute",
                TimeUnit::Minute,
                time.minute(),
                labels.minute.clone(),
            ))
            .when_some(second_label.clone(), |row, label| {
                row.child(separator(80.)).child(selector(
                    "second",
                    TimeUnit::Second,
                    time.second(),
                    label,
                ))
            });
        let dial = if specimen {
            ClockDial::specimen(named(&id, "dial"), labels.clone(), time, unit, format)
        } else {
            ClockDial::new(named(&id, "dial"), &self.state, cx)
        };
        let title = if mode == TimeInputMode::Dial {
            labels.select_title.clone()
        } else {
            labels.enter_title.clone()
        };
        let header = div()
            .h(rem(40.))
            .pt(rem(24.))
            .px(rem(24.))
            .text_color(color(ColorRole::ON_SURFACE_VARIANT))
            .map(|d| type_style(d, TypeRole::LabelMedium))
            .child(title);
        let (mode_icon, mode_label) = match mode {
            TimeInputMode::Dial => ("keyboard", labels.switch_to_input.clone()),
            TimeInputMode::Input => ("schedule", labels.switch_to_dial.clone()),
        };
        let toggle = update(TimePickerState::toggle_mode);
        let mut mode_button = IconButton::new(named(&id, "mode"), symbol(mode_icon), mode_label)
            .on_click(move |_, window, cx| toggle(window, cx));
        if let Some(p) = preview {
            mode_button = mode_button.preview_state(p);
        }
        let text_button = |name: &'static str, label: SharedString, f: Activate| {
            let mut b = material_button(named(&id, name), ButtonVariant::Text)
                .child(label)
                .on_click(move |_, window, cx| f(window, cx));
            if let Some(p) = preview {
                b = b.preview_state(p);
            }
            b
        };
        let cancel = self.on_cancel.clone();
        let confirm = self.on_confirm.clone();
        let read = self.state.clone();
        let actions = div()
            .flex()
            .items_center()
            .justify_between()
            .pr(rem(24.))
            .pb(rem(20.))
            .pl(rem(12.))
            .when(horizontal, |a| a.pt(rem(20.)))
            .child(mode_button)
            .child(
                div()
                    .flex()
                    .gap(rem(8.))
                    .child(text_button(
                        "cancel",
                        labels.cancel.clone(),
                        Rc::new(move |window, cx| {
                            if let Some(f) = &cancel {
                                f(window, cx)
                            }
                        }),
                    ))
                    .child(text_button(
                        "confirm",
                        labels.confirm.clone(),
                        Rc::new(move |window, cx| {
                            let time = read.read(cx).time;
                            if let Some(f) = &confirm {
                                f(time, window, cx)
                            }
                        }),
                    )),
            );
        // Three compact selectors and two separators are 48dp wider than
        // the kit's two selectors and separator.
        let extra = if seconds { 48. } else { 0. };
        let width = extra
            + match (mode, horizontal, twelve) {
                (TimeInputMode::Input, _, false) => 264.,
                (_, true, _) => 572.,
                _ => 328.,
            };
        let surface = div()
            .id(self.id.clone())
            .w(rem(width))
            .flex()
            .flex_col()
            .flex_shrink_0()
            .rounded(rem(28.))
            .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
            .role(Role::Dialog)
            .aria_label(labels.select_title.clone());
        match mode {
            TimeInputMode::Input => {
                let field_focused = field.read(cx).focus_handle(cx).is_focused(window);
                let editing = field.read(cx).selected_segment();
                let segment_labels = labels.clone();
                let period_state = self.state.clone();
                let period_labels = labels.clone();
                let pm = time.hour() >= 12;
                let input = TimeField::new(named(&id, "field"), &field)
                    .disabled(specimen)
                    .flex()
                    .items_start()
                    .text_color(color(ColorRole::ON_SURFACE))
                    .map(|d| type_style(d, TypeRole::DisplayLarge))
                    .line_height(rem(72.))
                    .render_segment(move |segment, segment_state, _, _| {
                        let which = segment_state.segment();
                        if which == TimeSegment::Period {
                            let state = period_state.clone();
                            return segment
                                .clear_children()
                                .ml(rem(12.))
                                .child(
                                    PeriodSelector::new(
                                        "period",
                                        period_labels.am.clone(),
                                        period_labels.pm.clone(),
                                    )
                                    .pm(pm)
                                    .height(72.)
                                    .on_change(
                                        move |pm, window, cx| {
                                            state.update(cx, |s, cx| s.set_pm(pm, window, cx))
                                        },
                                    ),
                                )
                                .into_any_element();
                        }
                        let selected = which == editing;
                        let (fill, ink) = if selected {
                            (
                                color(ColorRole::PRIMARY_CONTAINER),
                                color(ColorRole::ON_PRIMARY_CONTAINER),
                            )
                        } else {
                            (
                                color(ColorRole::SURFACE_CONTAINER_HIGHEST),
                                color(ColorRole::ON_SURFACE),
                            )
                        };
                        let label = match which {
                            TimeSegment::Hour => segment_labels.hour.clone(),
                            TimeSegment::Second => second_label
                                .clone()
                                .unwrap_or_else(|| segment_labels.minute.clone()),
                            _ => segment_labels.minute.clone(),
                        };
                        segment
                            .clear_children()
                            .flex()
                            .flex_col()
                            .gap(rem(7.))
                            .w(rem(if seconds { COMPACT_SELECTOR } else { 96. }))
                            .child(
                                div()
                                    .h(rem(72.))
                                    .w_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .gap(rem(1.))
                                    .rounded(rem(8.))
                                    .bg(fill)
                                    .when(selected, |b| {
                                        b.border(rem(2.)).border_color(color(ColorRole::PRIMARY))
                                    })
                                    .text_color(ink)
                                    .map(|b| type_style(b, TypeRole::DisplayMedium))
                                    .child((segment_labels.two_digits)(segment_state.value()))
                                    .when(selected && field_focused, |b| {
                                        b.child(
                                            div()
                                                .w(rem(2.))
                                                .h(rem(42.))
                                                .bg(color(ColorRole::PRIMARY)),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                                    .map(|d| type_style(d, TypeRole::BodySmall))
                                    .child(label),
                            )
                            .into_any_element()
                    });
                surface
                    .gap(rem(20.))
                    .child(header)
                    .child(div().px(rem(24.)).child(input))
                    .child(actions)
            }
            TimeInputMode::Dial if horizontal => surface
                .child(header)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(rem(52.))
                        .px(rem(24.))
                        .child(
                            div()
                                .w(rem(216. + extra))
                                .flex()
                                .flex_col()
                                .gap(rem(16.))
                                .child(time_row)
                                .when(twelve, |c| c.child(period(38., true))),
                        )
                        .child(dial),
                )
                .child(actions),
            TimeInputMode::Dial => surface
                .gap(rem(20.))
                .child(header)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(rem(36.))
                        .px(rem(24.))
                        .child(
                            div()
                                .flex()
                                .gap(rem(12.))
                                .child(time_row)
                                .when(twelve, |r| r.child(period(80., false))),
                        )
                        .child(dial),
                )
                .child(actions),
        }
    }
}

#[cfg(test)]
#[path = "time_picker_tests.rs"]
mod tests;
