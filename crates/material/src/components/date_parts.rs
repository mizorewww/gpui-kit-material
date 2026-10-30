//! Building blocks of the M3 date pickers: the calendar day cell, the year
//! chip, the month/year menu button, the docked month/year list item and the
//! calendar math they share.
//!
//! Reference: the Material 3 Design Kit "Date & time pickers" page
//! (`.Building Blocks/Local M3 calendar cell`, `.Building Blocks/Year`,
//! `.Building Blocks/Menu button`) and
//! <https://m3.material.io/components/date-pickers/specs>. Inspected tokens:
//! a 48dp cell around a 40dp circle with body-large text; selected days on
//! primary / on-primary, today outlined (1dp) in primary, range days on a
//! 40dp secondary-container band with on-secondary-container text, adjacent
//! month days at 38% on-surface; a 72×36 year pill (selected: primary with
//! title-medium on-primary, otherwise body-large on-surface-variant); a 40dp
//! label-large menu button with an 18dp drop arrow. State layers use the
//! shared [`state`] opacities and the 3dp secondary focus ring.
//!
//! The date arithmetic mirrors GPUI Base's calendar (`gpui_base::Calendar`,
//! whose month grid helper is private) with a fixed six-week grid so a
//! picker keeps its height while the month changes.
use super::InteractionPreview;
use super::button::ButtonFocusRing;
use super::ripple::Ripple;
use super::text_field::type_style;
use crate::{
    color::ColorRole,
    icons::{Icon, Symbol},
    motion::{self, MotionToken},
    theme::{blend, color},
    tokens::{
        state,
        typography::{TypeRole, rem},
    },
};
use chrono::{Datelike, Days, Months, NaiveDate, Weekday};
use gpui::{prelude::*, *};
use std::rc::Rc;

/// The day cell's square and its circular container, in dp.
pub const DAY_CELL: f32 = 48.;
const DAY_CONTAINER: f32 = 40.;
/// The year chip: a 72×36 pill in an 88×52 cell.
pub const YEAR_CELL_HEIGHT: f32 = 52.;
const YEAR_PILL: (f32, f32) = (72., 36.);
/// The docked menu list item height.
pub const LIST_ITEM_HEIGHT: f32 = 48.;
/// Weeks in every month grid: six keeps the picker height stable.
pub const GRID_WEEKS: usize = 6;

pub(crate) type Activate = Rc<dyn Fn(&mut Window, &mut App)>;

/// The weeks shown for `year`/`month` starting on `first_day`: always
/// `GRID_WEEKS` (six) rows of seven dates, padded with adjacent-month days.
pub fn month_weeks(year: i32, month: u32, first_day: Weekday) -> Vec<[NaiveDate; 7]> {
    let first = first_of_month(year, month);
    let offset =
        (first.weekday().num_days_from_sunday() + 7 - first_day.num_days_from_sunday()) % 7;
    let start = first - Days::new(offset as u64);
    (0..GRID_WEEKS)
        .map(|week| std::array::from_fn(|day| start + Days::new((week * 7 + day) as u64)))
        .collect()
}

/// The first day of `month` (1-based) in `year`; months outside 1..=12
/// carry into adjacent years.
pub fn first_of_month(year: i32, month: u32) -> NaiveDate {
    let total = year as i64 * 12 + month as i64 - 1;
    NaiveDate::from_ymd_opt(
        total.div_euclid(12) as i32,
        total.rem_euclid(12) as u32 + 1,
        1,
    )
    .unwrap_or(NaiveDate::MIN)
}

/// `date` moved by `months`, clamped to the end of shorter months.
pub fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
    let moved = if months >= 0 {
        date.checked_add_months(Months::new(months as u32))
    } else {
        date.checked_sub_months(Months::new(months.unsigned_abs()))
    };
    moved.unwrap_or(date)
}

/// The weekdays of a grid row starting at `first_day`.
pub fn week_days(first_day: Weekday) -> [Weekday; 7] {
    let mut day = first_day;
    std::array::from_fn(|_| {
        let current = day;
        day = day.succ();
        current
    })
}

/// Where a day sits in a selected range, for its band.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DayRange {
    /// Not in a range.
    #[default]
    None,
    /// The first day of a range: the band runs to its trailing edge.
    Start,
    /// A day between the ends: the band spans the cell.
    Middle,
    /// The last day: the band runs from its leading edge.
    End,
}

/// The paint of one day cell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DayPaint {
    pub fill: Rgba,
    pub ink: Rgba,
    pub layer: Rgba,
    pub outline: Option<Rgba>,
}

/// The Figma kit's per-type paint. `selected` is the eased selection
/// amount, so the fill and ink crossfade on the effects spring.
pub(crate) fn day_paint(selected: f32, today: bool, range: bool, adjacent: bool) -> DayPaint {
    let t = selected.clamp(0., 1.);
    let rest_ink = if range {
        color(ColorRole::ON_SECONDARY_CONTAINER)
    } else if today {
        color(ColorRole::PRIMARY)
    } else if adjacent {
        state::disabled(color(ColorRole::ON_SURFACE))
    } else {
        color(ColorRole::ON_SURFACE)
    };
    let rest_layer = if today && !range {
        color(ColorRole::PRIMARY)
    } else {
        color(ColorRole::ON_SURFACE)
    };
    let primary = color(ColorRole::PRIMARY);
    DayPaint {
        fill: Rgba {
            a: primary.a * t,
            ..primary
        },
        ink: blend(rest_ink, color(ColorRole::ON_PRIMARY), t),
        layer: blend(rest_layer, color(ColorRole::ON_PRIMARY), t),
        outline: (today && t < 1.).then_some(Rgba {
            a: primary.a * (1. - t),
            ..primary
        }),
    }
}

/// The M3 disabled paint of a day: a selected day keeps an on-surface 12%
/// container, its label and today's outline turn on-surface 38%. Fading
/// the enabled paint instead leaves a dark on-primary label on a dim
/// primary container, unreadable in dark themes.
pub(crate) fn disabled_day_paint(paint: DayPaint, selected: f32) -> DayPaint {
    let on_surface = color(ColorRole::ON_SURFACE);
    let content = state::disabled(on_surface);
    DayPaint {
        fill: on_surface.opacity(state::DISABLED_CONTAINER * selected.clamp(0., 1.)),
        ink: content,
        layer: paint.layer,
        outline: paint.outline.map(|_| content),
    }
}

/// The M3 disabled colors of a year: (container, label).
pub(crate) fn disabled_year_paint(selected: bool) -> (Rgba, Rgba) {
    let on_surface = color(ColorRole::ON_SURFACE);
    (
        on_surface.opacity(if selected {
            state::DISABLED_CONTAINER
        } else {
            0.
        }),
        state::disabled(on_surface),
    )
}

/// One day of a calendar grid (`.Building Blocks/Local M3 calendar cell`).
#[derive(gpui::IntoElement)]
pub struct CalendarDay {
    id: ElementId,
    label: SharedString,
    accessible: Option<SharedString>,
    selected: bool,
    today: bool,
    adjacent: bool,
    range: DayRange,
    disabled: bool,
    cursor: bool,
    animate: bool,
    preview: Option<InteractionPreview>,
    on_click: Option<Activate>,
}

impl CalendarDay {
    /// A day showing `label` (its number, formatted by the caller).
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            accessible: None,
            selected: false,
            today: false,
            adjacent: false,
            range: DayRange::None,
            disabled: false,
            cursor: false,
            animate: false,
            preview: None,
            on_click: None,
        }
    }
    /// The full date announced by assistive technology.
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible = Some(label.into());
        self
    }
    /// A selected day, or a range end: the filled primary circle.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    /// Today's date: outlined in primary.
    pub fn today(mut self, today: bool) -> Self {
        self.today = today;
        self
    }
    /// A day of the previous or next month shown to fill the grid.
    pub fn adjacent(mut self, adjacent: bool) -> Self {
        self.adjacent = adjacent;
        self
    }
    /// The day's place in a selected range.
    pub fn range(mut self, range: DayRange) -> Self {
        self.range = range;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// The grid's keyboard cursor: the focus ring shows here while the grid
    /// has keyboard focus (`grid_focused`).
    pub(crate) fn cursor(mut self, cursor: bool) -> Self {
        self.cursor = cursor;
        self
    }
    /// Crossfade selection changes on the fast effects spring.
    pub(crate) fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }
    /// Render a static specimen in one forced interaction state.
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for CalendarDay {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let preview = self.preview;
        let disabled = self.disabled || preview.is_some_and(InteractionPreview::is_disabled);
        let live = !disabled && preview.is_none() && self.on_click.is_some();
        let target = if self.selected { 1. } else { 0. };
        let selected = if self.animate && preview.is_none() {
            motion::token_value(
                (self.id.clone(), "selected"),
                target,
                MotionToken::FastEffects,
                window,
                cx,
            )
        } else {
            target
        };
        let range = self.range != DayRange::None;
        let paint = day_paint(selected, self.today, range && !self.selected, self.adjacent);
        let paint = if disabled {
            disabled_day_paint(paint, selected)
        } else {
            paint
        };
        let forced = |state: InteractionPreview| !disabled && preview == Some(state);
        let hover = state::hovered(paint.fill, paint.layer);
        let press = state::pressed(paint.fill, paint.layer);
        let focused_layer = state::layer(paint.fill, paint.layer, state::FOCUS);
        let adjacent_hover = self.adjacent && !self.selected;
        let hover_ink = color(ColorRole::ON_SURFACE_VARIANT);
        let band = color(ColorRole::SECONDARY_CONTAINER);
        let half = DAY_CELL / 2.;
        let keyboard = self.cursor && live;
        let container = div()
            .id("day")
            .size(rem(DAY_CONTAINER))
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .bg(paint.fill)
            .when_some(paint.outline, |c, outline| {
                c.border(rem(1.)).border_color(outline)
            })
            .text_color(paint.ink)
            .map(|c| type_style(c, TypeRole::BodyLarge))
            .when(forced(InteractionPreview::Hovered), |c| {
                c.bg(hover)
                    .when(adjacent_hover, |c| c.text_color(hover_ink))
            })
            .when(forced(InteractionPreview::Focused), |c| {
                c.bg(focused_layer)
                    .when(adjacent_hover, |c| c.text_color(hover_ink))
            })
            .when(forced(InteractionPreview::Pressed), |c| c.bg(press))
            .when(live, |c| {
                c.cursor_pointer()
                    .hover(move |s| {
                        let s = s.bg(hover);
                        if adjacent_hover {
                            s.text_color(hover_ink)
                        } else {
                            s
                        }
                    })
                    .active(move |s| s.bg(press))
            })
            .child(self.label.clone());
        let container = Ripple::new((self.id.clone(), "ripple"), container)
            .round()
            .ink(paint.layer)
            .disabled(!live);
        let ring = ButtonFocusRing::resolved(container, keyboard, rem(DAY_CONTAINER / 2.))
            .forced(forced(InteractionPreview::Focused));
        let on_click = self.on_click.filter(|_| live);
        div()
            .id(self.id)
            .relative()
            .flex_shrink_0()
            .size(rem(DAY_CELL))
            .flex()
            .items_center()
            .justify_center()
            .role(Role::GridCell)
            .aria_label(self.accessible.unwrap_or(self.label))
            .aria_selected(self.selected || range)
            .when(self.cursor, |cell| cell.aria_active_descendant())
            .when(range, |cell| {
                let (left, width) = match self.range {
                    DayRange::Start => (half, half),
                    DayRange::End => (0., half),
                    _ => (0., DAY_CELL),
                };
                cell.child(
                    div()
                        .absolute()
                        .top(rem((DAY_CELL - DAY_CONTAINER) / 2.))
                        .left(rem(left))
                        .w(rem(width))
                        .h(rem(DAY_CONTAINER))
                        .bg(band)
                        .when(disabled, |band| band.opacity(state::DISABLED_CONTENT)),
                )
            })
            .child(ring)
            .when_some(on_click, |cell, f| {
                cell.on_click(move |_, window, cx| f(window, cx))
            })
    }
}

/// A year in the modal picker's year grid (`.Building Blocks/Year`).
#[derive(gpui::IntoElement)]
pub struct YearChip {
    id: ElementId,
    label: SharedString,
    selected: bool,
    cursor: bool,
    disabled: bool,
    preview: Option<InteractionPreview>,
    on_click: Option<Activate>,
}

impl YearChip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected: false,
            cursor: false,
            disabled: false,
            preview: None,
            on_click: None,
        }
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub(crate) fn cursor(mut self, cursor: bool) -> Self {
        self.cursor = cursor;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

impl RenderOnce for YearChip {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let preview = self.preview;
        let disabled = self.disabled || preview.is_some_and(InteractionPreview::is_disabled);
        let live = !disabled && preview.is_none() && self.on_click.is_some();
        let forced = |state: InteractionPreview| !disabled && preview == Some(state);
        let (fill, ink, layer, role) = if self.selected {
            (
                color(ColorRole::PRIMARY),
                color(ColorRole::ON_PRIMARY),
                color(ColorRole::ON_PRIMARY),
                TypeRole::TitleMedium,
            )
        } else {
            let clear = Rgba {
                a: 0.,
                ..color(ColorRole::ON_SURFACE)
            };
            (
                clear,
                color(ColorRole::ON_SURFACE_VARIANT),
                color(ColorRole::ON_SURFACE),
                TypeRole::BodyLarge,
            )
        };
        let (fill, ink) = if disabled {
            disabled_year_paint(self.selected)
        } else {
            (fill, ink)
        };
        let hover = state::hovered(fill, layer);
        let press = state::pressed(fill, layer);
        let focus = state::layer(fill, layer, state::FOCUS);
        let pill = div()
            .id("year")
            .w(rem(YEAR_PILL.0))
            .h(rem(YEAR_PILL.1))
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .bg(fill)
            .text_color(ink)
            .map(|c| type_style(c, role))
            .when(forced(InteractionPreview::Hovered), |c| c.bg(hover))
            .when(forced(InteractionPreview::Focused), |c| c.bg(focus))
            .when(forced(InteractionPreview::Pressed), |c| c.bg(press))
            .when(live, |c| {
                c.cursor_pointer()
                    .hover(move |s| s.bg(hover))
                    .active(move |s| s.bg(press))
            })
            .child(self.label.clone());
        let pill = Ripple::new((self.id.clone(), "ripple"), pill)
            .round()
            .ink(layer)
            .disabled(!live);
        let ring = ButtonFocusRing::resolved(pill, self.cursor && live, rem(YEAR_PILL.1 / 2.))
            .forced(forced(InteractionPreview::Focused));
        let on_click = self.on_click.filter(|_| live);
        div()
            .id(self.id)
            .flex_1()
            .h(rem(YEAR_CELL_HEIGHT))
            .flex()
            .items_center()
            .justify_center()
            .role(Role::GridCell)
            .aria_label(self.label)
            .aria_selected(self.selected)
            .when(self.cursor, |cell| cell.aria_active_descendant())
            .child(ring)
            .when_some(on_click, |cell, f| {
                cell.on_click(move |_, window, cx| f(window, cx))
            })
    }
}

/// The month/year dropdown trigger (`.Building Blocks/Menu button`): a
/// label-large label and an 18dp arrow that points up while its list is
/// open, on a 40dp pill. The arrow turns on the fast spatial spring.
#[derive(gpui::IntoElement)]
pub struct PickerMenuButton {
    id: ElementId,
    label: SharedString,
    accessible: Option<SharedString>,
    open: bool,
    disabled: bool,
    arrow: bool,
    preview: Option<InteractionPreview>,
    on_click: Option<Activate>,
}

impl PickerMenuButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            accessible: None,
            open: false,
            disabled: false,
            arrow: true,
            preview: None,
            on_click: None,
        }
    }
    /// What the button does, for assistive technology (defaults to the label).
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible = Some(label.into());
        self
    }
    /// Whether the list it controls is open: the arrow points up.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Show the drop arrow (the docked picker hides it on the inactive menu).
    pub fn arrow(mut self, arrow: bool) -> Self {
        self.arrow = arrow;
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

fn drop_arrow() -> Icon {
    Icon::from(Symbol::named("arrow_drop_down").expect("arrow_drop_down is a Material Symbol"))
}

impl RenderOnce for PickerMenuButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let preview = self.preview;
        let disabled = self.disabled || preview.is_some_and(InteractionPreview::is_disabled);
        let turn = if preview.is_some() {
            if self.open { 1. } else { 0. }
        } else {
            motion::token_value(
                (self.id.clone(), "arrow"),
                if self.open { 1. } else { 0. },
                MotionToken::FastSpatial,
                window,
                cx,
            )
        };
        let ink = color(ColorRole::ON_SURFACE_VARIANT);
        let clear = Rgba { a: 0., ..ink };
        let ripple_id = ElementId::NamedChild(self.id.clone().into(), "ripple".into());
        let mut button = gpui_base::Button::new(self.id.clone())
            .accessibility_label(self.accessible.unwrap_or_else(|| self.label.clone()))
            .disabled(disabled)
            .h(rem(40.))
            .pl(rem(8.))
            .pr(rem(4.))
            .flex()
            .items_center()
            .gap(rem(8.))
            .rounded_full()
            .text_color(ink)
            .map(|b| type_style(b, TypeRole::LabelLarge))
            .when(disabled, |b| b.opacity(state::DISABLED_CONTENT))
            .child(self.label)
            .when(self.arrow, |b| {
                b.child(
                    drop_arrow()
                        .size(rem(18.))
                        .color(ink)
                        .rotate(std::f32::consts::PI * turn),
                )
            });
        if let Some(f) = self.on_click.filter(|_| !disabled) {
            button = button.on_click(move |_, window, cx| f(window, cx));
        }
        let hover = state::hovered(clear, ink);
        let press = state::pressed(clear, ink);
        let focus = window
            .use_keyed_state((self.id, "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let enabled = !disabled;
        button = match preview {
            Some(InteractionPreview::Hovered) => button.bg(hover),
            Some(InteractionPreview::Focused) => button.bg(state::layer(clear, ink, state::FOCUS)),
            Some(InteractionPreview::Pressed) => button.bg(press),
            Some(_) => button,
            None if enabled => button
                .cursor_pointer()
                .hover(move |s| s.bg(hover))
                .active(move |s| s.bg(press)),
            None => button,
        };
        let button = button.track_focus(&focus).tab_stop(preview.is_none());
        let button = Ripple::new(ripple_id, button)
            .round()
            .ink(ink)
            .disabled(!enabled)
            .specimen(preview.is_some());
        ButtonFocusRing::new(button, focus, enabled, rem(20.))
            .forced(enabled && preview == Some(InteractionPreview::Focused))
    }
}

/// A docked month or year menu row: a 48dp list item with a leading check
/// on the selection, which rests on a hovered-strength layer.
#[derive(gpui::IntoElement)]
pub struct PickerListItem {
    id: ElementId,
    label: SharedString,
    selected: bool,
    cursor: bool,
    preview: Option<InteractionPreview>,
    on_click: Option<Activate>,
}

impl PickerListItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected: false,
            cursor: false,
            preview: None,
            on_click: None,
        }
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub(crate) fn cursor(mut self, cursor: bool) -> Self {
        self.cursor = cursor;
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

impl RenderOnce for PickerListItem {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let preview = self.preview;
        let disabled = preview.is_some_and(InteractionPreview::is_disabled);
        let live = !disabled && preview.is_none() && self.on_click.is_some();
        let ink = color(ColorRole::ON_SURFACE);
        let clear = Rgba { a: 0., ..ink };
        // The kit rests the selected row on the hover-strength layer.
        let rest = if self.selected {
            state::hovered(clear, ink)
        } else {
            clear
        };
        let hover = state::layer(rest, ink, state::HOVER);
        let press = state::layer(rest, ink, state::PRESS);
        let row = div()
            .id("item")
            .h(rem(LIST_ITEM_HEIGHT))
            .w_full()
            .flex()
            .items_center()
            .gap(rem(16.))
            .px(rem(16.))
            .bg(rest)
            .text_color(ink)
            .map(|r| type_style(r, TypeRole::BodyLarge))
            .when(preview == Some(InteractionPreview::Hovered), |r| {
                r.bg(hover)
            })
            .when(preview == Some(InteractionPreview::Pressed), |r| {
                r.bg(press)
            })
            .when(live, |r| {
                r.cursor_pointer()
                    .hover(move |s| s.bg(hover))
                    .active(move |s| s.bg(press))
            })
            .child(
                div()
                    .size(rem(24.))
                    .flex_shrink_0()
                    .when(self.selected, |slot| {
                        slot.child(
                            Icon::from(crate::icons::IconName::Check)
                                .size(rem(24.))
                                .color(color(ColorRole::ON_SURFACE_VARIANT)),
                        )
                    }),
            )
            .child(self.label.clone());
        let row = Ripple::new((self.id.clone(), "ripple"), row)
            .ink(ink)
            .disabled(!live);
        let ring = ButtonFocusRing::resolved(row, self.cursor && live, rem(0.))
            .inward()
            .forced(preview == Some(InteractionPreview::Focused));
        let on_click = self.on_click.filter(|_| live);
        div()
            .id(self.id)
            .w_full()
            .role(Role::ListBoxOption)
            .aria_label(self.label)
            .aria_selected(self.selected)
            .when(self.cursor, |item| item.aria_active_descendant())
            .child(ring)
            .when_some(on_click, |item, f| {
                item.on_click(move |_, window, cx| f(window, cx))
            })
    }
}
