//! M3 Expressive sliders over GPUI Base's slider behavior (pointer, touch,
//! range thumbs and accessibility actions). Material owns the presentation:
//! the five track sizes, standard/centered/range tracks split around a 4dp
//! handle, discrete stop indicators, an icon inside the track, the value
//! indicator bubble shown while pressed, vertical orientation and keyboard
//! stepping. References: <https://m3e.kyralabs.dev/components/slider/>, the
//! Figma kit's Standard/Centered/Range slider sets, and
//! <https://md3.io/components/slider/> for the classic slider.
use super::InteractionPreview;
use super::button::ButtonFocusRing;
use crate::{
    color::ColorRole,
    icons::Icon,
    motion::{self, MotionToken},
    theme::color,
    tokens::{
        state,
        typography::{TypeRole, rem},
    },
};
use gpui::{prelude::*, *};
use gpui_base::slider::{
    Slider, SliderEvent, SliderIndicator, SliderState, SliderThumb, SliderTrack,
};

/// The M3 Expressive slider sizes (track thickness 16/24/40/56/96dp).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SliderSize {
    #[default]
    XSmall,
    Small,
    Medium,
    Large,
    XLarge,
}

impl SliderSize {
    pub const ALL: [Self; 5] = [
        Self::XSmall,
        Self::Small,
        Self::Medium,
        Self::Large,
        Self::XLarge,
    ];

    /// Track thickness, handle length, outer track corner and icon size (dp).
    const fn metrics(self) -> SizeMetrics {
        let (track, handle, corner, icon) = match self {
            Self::XSmall => (16., 44., 8., 0.),
            Self::Small => (24., 44., 8., 0.),
            Self::Medium => (40., 52., 12., 24.),
            Self::Large => (56., 68., 16., 24.),
            Self::XLarge => (96., 108., 28., 32.),
        };
        SizeMetrics {
            track,
            handle,
            corner,
            icon,
        }
    }

    /// The kit's name for the size; galleries may use it as a label.
    pub const fn name(self) -> &'static str {
        match self {
            Self::XSmall => "XSmall",
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::XLarge => "XLarge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SizeMetrics {
    track: f32,
    handle: f32,
    corner: f32,
    icon: f32,
}

/// Slider geometry. Classic matches the Google Fonts axis controls; Expressive
/// follows the M3 Expressive thick track and vertical handle presentation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SliderAppearance {
    Classic,
    #[default]
    Expressive,
}

/// The slider's accent (md3.io's Color option). Primary is the M3 default:
/// a primary active track and handle over a secondary-container inactive
/// track. The others use their own family throughout: the role for the
/// active track and handle, its container for the inactive track, and the
/// matching on-roles for stops and icons.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SliderColor {
    #[default]
    Primary,
    Secondary,
    Tertiary,
    Error,
}

impl SliderColor {
    pub const ALL: [Self; 4] = [Self::Primary, Self::Secondary, Self::Tertiary, Self::Error];
    /// (active, on-active, inactive, on-inactive) roles.
    const fn roles(self) -> (ColorRole, ColorRole, ColorRole, ColorRole) {
        use ColorRole as C;
        match self {
            Self::Primary => (
                C::PRIMARY,
                C::ON_PRIMARY,
                C::SECONDARY_CONTAINER,
                C::ON_SECONDARY_CONTAINER,
            ),
            Self::Secondary => (
                C::SECONDARY,
                C::ON_SECONDARY,
                C::SECONDARY_CONTAINER,
                C::ON_SECONDARY_CONTAINER,
            ),
            Self::Tertiary => (
                C::TERTIARY,
                C::ON_TERTIARY,
                C::TERTIARY_CONTAINER,
                C::ON_TERTIARY_CONTAINER,
            ),
            Self::Error => (
                C::ERROR,
                C::ON_ERROR,
                C::ERROR_CONTAINER,
                C::ON_ERROR_CONTAINER,
            ),
        }
    }
}

/// M3 Expressive slider details, in dp: the gap between the handle and the
/// track on either side, the corners where the track meets the gap, the
/// handle's width (narrowing while pressed) and the stop indicators.
const HANDLE_GAP: f32 = 6.;
const TRACK_INNER_CORNER: f32 = 2.;
const HANDLE_WIDTH: f32 = 4.;
const HANDLE_PRESSED_WIDTH: f32 = 2.;
const STOP_INDICATOR: f32 = 4.;
/// A stop's center never sits closer than this to a track end.
const STOP_INSET: f32 = 6.;
/// The value indicator: a 48x44dp minimum inverse-surface pill, 4dp from
/// the handle.
const BUBBLE_HEIGHT: f32 = 44.;
const BUBBLE_MIN_WIDTH: f32 = 48.;
const BUBBLE_PADDING: f32 = 16.;
const BUBBLE_GAP: f32 = 4.;
/// Below this fraction the icon moves from the active to the inactive track.
const ICON_MIN_FRACTION: f32 = 0.25;

/// A value-indicator formatter.
type ValueFormat = std::rc::Rc<dyn Fn(f32) -> SharedString>;

#[derive(gpui::IntoElement)]
pub struct MaterialSlider {
    id: ElementId,
    state: Entity<SliderState>,
    label: SharedString,
    appearance: SliderAppearance,
    size: SliderSize,
    axis: Axis,
    centered: bool,
    stops: bool,
    color: SliderColor,
    icon: Option<Icon>,
    value_indicator: bool,
    format: Option<ValueFormat>,
    disabled: bool,
    preview: Option<InteractionPreview>,
}

impl MaterialSlider {
    pub fn new(state: &Entity<SliderState>, label: impl Into<SharedString>) -> Self {
        Self {
            id: ElementId::Integer(state.entity_id().as_u64()),
            state: state.clone(),
            label: label.into(),
            appearance: SliderAppearance::default(),
            size: SliderSize::default(),
            axis: Axis::Horizontal,
            centered: false,
            stops: false,
            color: SliderColor::Primary,
            icon: None,
            value_indicator: true,
            format: None,
            disabled: false,
            preview: None,
        }
    }

    /// Distinguish sliders that share one state (e.g. specimens).
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
    pub fn appearance(mut self, appearance: SliderAppearance) -> Self {
        self.appearance = appearance;
        self
    }
    pub fn size(mut self, size: SliderSize) -> Self {
        self.size = size;
        self
    }
    pub fn vertical(mut self) -> Self {
        self.axis = Axis::Vertical;
        self
    }
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }
    /// The active track grows from the middle of the range toward the value.
    pub fn centered(mut self, centered: bool) -> Self {
        self.centered = centered;
        self
    }
    /// Show a stop indicator at every step (a discrete slider).
    pub fn stops(mut self, stops: bool) -> Self {
        self.stops = stops;
        self
    }
    /// The accent color family (default primary). See [`SliderColor`].
    pub fn color(mut self, color: SliderColor) -> Self {
        self.color = color;
        self
    }
    /// An icon inside the track's leading end (medium and larger sizes).
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    /// Whether pressing shows the value indicator (default: shown).
    pub fn value_indicator(mut self, show: bool) -> Self {
        self.value_indicator = show;
        self
    }
    /// The value indicator's text; by default the value rounded.
    pub fn format(mut self, format: impl Fn(f32) -> SharedString + 'static) -> Self {
        self.format = Some(std::rc::Rc::new(format));
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Render a fixed interaction state as a specimen.
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
}

fn slider_geometry(appearance: SliderAppearance) -> (f32, f32, f32) {
    match appearance {
        SliderAppearance::Classic => (4., 20., 20.),
        SliderAppearance::Expressive => (16., 4., 44.),
    }
}

/// The colors of a slider in one state. Disabled colors follow the kit's
/// tokens through [`state`], composited rather than pre-mixed (the policy
/// documented on the switch): disabled content is its role at
/// [`state::DISABLED_CONTENT`] and the disabled inactive track at
/// [`state::DISABLED_CONTAINER`].
#[derive(Clone, Copy, Debug, PartialEq)]
struct SliderPaint {
    active: Rgba,
    inactive: Rgba,
    handle: Rgba,
    /// Stops and the icon over the active track.
    on_active: Rgba,
    /// Stops and the icon over the inactive track.
    on_inactive: Rgba,
}

fn slider_paint(disabled: bool, accent: SliderColor) -> SliderPaint {
    if disabled {
        let on_surface = color(ColorRole::ON_SURFACE);
        SliderPaint {
            active: state::disabled(on_surface),
            inactive: on_surface.opacity(state::DISABLED_CONTAINER),
            handle: state::disabled(on_surface),
            on_active: color(ColorRole::INVERSE_ON_SURFACE),
            on_inactive: state::disabled(on_surface),
        }
    } else {
        let (active, on_active, inactive, on_inactive) = accent.roles();
        SliderPaint {
            active: color(active),
            inactive: color(inactive),
            handle: color(active),
            on_active: color(on_active),
            on_inactive: color(on_inactive),
        }
    }
}

/// One track piece along the main axis (px from the start: left, or the
/// bottom of a vertical slider).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Segment {
    start: f32,
    end: f32,
    active: bool,
    /// Whether each end is a track end (outer corner) rather than a gap.
    outer_start: bool,
    outer_end: bool,
}

/// A handle's center along a track `length` long, keeping the handle
/// inside the track.
fn handle_center(length: f32, handle: f32, fraction: f32) -> f32 {
    handle / 2. + fraction.clamp(0., 1.) * (length - handle).max(0.)
}

/// Split the track around the handles. `handles` are fractions (one, or a
/// range's two); `centered` grows the active track from the middle.
fn track_segments(length: f32, handle: f32, handles: &[f32], centered: bool) -> Vec<Segment> {
    let clear = handle / 2. + HANDLE_GAP;
    let centers: Vec<f32> = handles
        .iter()
        .map(|&f| handle_center(length, handle, f))
        .collect();
    let mut segments = Vec::new();
    let mut push = |start: f32, end: f32, active, outer_start, outer_end| {
        if end - start > 0.5 {
            segments.push(Segment {
                start,
                end,
                active,
                outer_start,
                outer_end,
            });
        }
    };
    match (centers.as_slice(), centered) {
        ([low, high], _) => {
            push(0., low - clear, false, true, false);
            push(low + clear, high - clear, true, false, false);
            push(high + clear, length, false, false, true);
        }
        ([at], true) => {
            let middle = length / 2.;
            if *at >= middle {
                push(0., middle, false, true, false);
                push(middle, at - clear, true, false, false);
                push(at + clear, length, false, false, true);
            } else {
                push(0., at - clear, false, true, false);
                push(at + clear, middle, true, false, false);
                push(middle, length, false, false, true);
            }
        }
        ([at], false) => {
            push(0., at - clear, true, true, false);
            push(at + clear, length, false, false, true);
        }
        _ => push(0., length, false, true, true),
    }
    segments
}

/// Stop indicator centers along the track and whether each lies on the
/// active track; stops under a handle (or its gap) are hidden. Without
/// `steps`, only the inactive ends carry a stop.
fn stop_positions(
    length: f32,
    handle: f32,
    segments: &[Segment],
    steps: Option<usize>,
    handles: &[f32],
) -> Vec<(f32, bool)> {
    let clear = handle / 2. + HANDLE_GAP;
    let inset = |x: f32| x.clamp(STOP_INSET, (length - STOP_INSET).max(STOP_INSET));
    let candidates: Vec<f32> = match steps {
        Some(count) if count >= 1 => (0..=count)
            .map(|i| inset(handle_center(length, handle, i as f32 / count as f32)))
            .collect(),
        _ => segments
            .iter()
            .filter(|s| !s.active)
            .flat_map(|s| {
                [
                    s.outer_start.then_some(STOP_INSET),
                    s.outer_end.then_some(length - STOP_INSET),
                ]
            })
            .flatten()
            .collect(),
    };
    candidates
        .into_iter()
        .filter(|&x| {
            handles
                .iter()
                .all(|&f| (x - handle_center(length, handle, f)).abs() > clear - 1.)
        })
        .map(|x| {
            let active = segments
                .iter()
                .any(|s| s.active && x >= s.start && x <= s.end);
            (x, active)
        })
        .collect()
}

/// Everything the track canvas paints, in design px.
#[derive(Clone, Debug)]
struct TrackPaint {
    axis: Axis,
    metrics: SizeMetrics,
    handle: f32,
    handles: Vec<f32>,
    centered: bool,
    steps: Option<usize>,
    paint: SliderPaint,
    /// Pixels per design px.
    scale: f32,
}

fn paint_track(bounds: Bounds<Pixels>, track: &TrackPaint, window: &mut Window) {
    let s = track.scale;
    let horizontal = track.axis == Axis::Horizontal;
    // Painted, so neither its segments nor their corners mirror with layout.
    let rtl = crate::scope::is_rtl(window);
    let length = f32::from(if horizontal {
        bounds.size.width
    } else {
        bounds.size.height
    }) / s;
    let thickness = track.metrics.track;
    let segments = track_segments(length, track.handle, &track.handles, track.centered);
    // A main-axis span and a cross-axis centered band, in window px.
    let rect = |start: f32, end: f32, cross: f32| {
        let (a, b) = (start * s, end * s);
        if horizontal {
            Bounds::new(
                point(
                    if rtl {
                        bounds.right() - px(b)
                    } else {
                        bounds.origin.x + px(a)
                    },
                    bounds.center().y - px(cross * s / 2.),
                ),
                size(px(b - a), px(cross * s)),
            )
        } else {
            Bounds::new(
                point(
                    bounds.center().x - px(cross * s / 2.),
                    bounds.bottom() - px(b),
                ),
                size(px(cross * s), px(b - a)),
            )
        }
    };
    for segment in &segments {
        let half = (segment.end - segment.start).min(thickness) / 2.;
        let corner = |outer: bool| {
            px(if outer {
                track.metrics.corner
            } else {
                TRACK_INNER_CORNER
            }
            .min(half)
                * s)
        };
        let (start, end) = (corner(segment.outer_start), corner(segment.outer_end));
        let (start, end) = if horizontal && rtl {
            (end, start)
        } else {
            (start, end)
        };
        let radii = if horizontal {
            Corners {
                top_left: start,
                bottom_left: start,
                top_right: end,
                bottom_right: end,
            }
        } else {
            Corners {
                bottom_left: start,
                bottom_right: start,
                top_left: end,
                top_right: end,
            }
        };
        let ink = if segment.active {
            track.paint.active
        } else {
            track.paint.inactive
        };
        window
            .paint_quad(fill(rect(segment.start, segment.end, thickness), ink).corner_radii(radii));
    }
    for (at, active) in stop_positions(length, track.handle, &segments, track.steps, &track.handles)
    {
        let ink = if active {
            track.paint.on_active
        } else {
            track.paint.on_inactive
        };
        let dot = rect(
            at - STOP_INDICATOR / 2.,
            at + STOP_INDICATOR / 2.,
            STOP_INDICATOR,
        );
        window.paint_quad(fill(dot, ink).corner_radii(px(STOP_INDICATOR * s / 2.)));
    }
}

/// The value as a rounded number, the default indicator text.
fn default_format(value: f32) -> SharedString {
    let rounded = value.round();
    if (value - rounded).abs() < 0.005 {
        format!("{}", rounded as i64).into()
    } else {
        format!("{value:.1}").into()
    }
}

/// The classic (M3 baseline) slider: a 4dp track and a 20dp round handle
/// with a 40dp state layer, in a 48dp tall target. It shares the
/// Expressive slider's focus, keyboard, press, preview, axis and range
/// handling; only the geometry differs.
const CLASSIC_TARGET: f32 = 48.;
const CLASSIC_STATE_LAYER: f32 = 40.;

/// Keyboard stepping: arrows by one step, Page keys by a tenth of the range
/// rounded to whole steps, Home/End to the ends. `bounds` limits the moving
/// thumb (a range thumb stops at the other one); `range` is the slider's
/// min and max, the origin of its step grid. `None` for other keys or an
/// invalid range (min above max, or NaN), which would otherwise panic.
fn stepped_value(
    key: &str,
    value: f32,
    bounds: (f32, f32),
    range: (f32, f32),
    step: f32,
) -> Option<f32> {
    let ((lo, hi), (min, max)) = (bounds, range);
    if !(min <= max && lo <= hi) || value.is_nan() {
        return None;
    }
    let discrete = step > 0. && step.is_finite();
    let step = if discrete { step } else { (max - min) / 100. };
    let page = if step > 0. {
        ((max - min) / 10. / step).round().max(1.) * step
    } else {
        0.
    };
    let next = match key {
        "right" | "up" => value + step,
        "left" | "down" => value - step,
        "pageup" => value + page,
        "pagedown" => value - page,
        "home" => lo,
        "end" => hi,
        _ => return None,
    };
    let next = if discrete {
        min + ((next - min) / step).round() * step
    } else {
        next
    };
    Some(next.min(hi).max(lo))
}

impl RenderOnce for MaterialSlider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Key handling runs after the draw, so it keeps this frame's direction.
        let mirrored_keys = crate::scope::is_rtl(window);
        let id = self.id.clone();
        let classic = self.appearance == SliderAppearance::Classic;
        // A single slider focuses its Role::Slider node; a range focuses each
        // thumb, so Tab moves between them and each takes the arrow keys.
        let focus = window
            .use_keyed_state((id.clone(), "focus"), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            })
            .read(cx)
            .clone();
        let start_focus = window
            .use_keyed_state((id.clone(), "focus-start"), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            })
            .read(cx)
            .clone();
        let press = window.use_keyed_state((id.clone(), "pressed"), cx, |_, _| false);
        let preview = self.preview;
        let disabled = self.disabled || preview.is_some_and(|p| p.is_disabled());
        let live = !disabled && preview.is_none();
        let forced = |s| !disabled && preview == Some(s);
        let hovered = forced(InteractionPreview::Hovered);
        // Nothing would hear the release of a press that outlives being live.
        if !live && *press.read(cx) {
            press.update(cx, |pressed, _| *pressed = false);
        }
        let pressed = forced(InteractionPreview::Pressed) || (live && *press.read(cx));
        #[cfg(test)]
        SLIDER_PRESSED.with(|cell| cell.set(pressed));

        let state = self.state.read(cx);
        let value = state.value();
        let is_range = value.is_range();
        let (min, max, step) = (state.min_value(), state.max_value(), state.step_value());
        let percentage = state.percentage();
        let handles: Vec<f32> = if is_range {
            vec![percentage.start, percentage.end]
        } else {
            vec![percentage.end]
        };
        let values: Vec<f32> = if is_range {
            vec![value.start(), value.end()]
        } else {
            vec![value.end()]
        };
        // Which thumb holds keyboard focus.
        let keyboard = live && window.last_input_was_keyboard();
        let focused_thumb = if is_range {
            if start_focus.is_focused(window) {
                Some(0)
            } else if focus.is_focused(window) {
                Some(1)
            } else {
                None
            }
        } else {
            focus.is_focused(window).then_some(0)
        };
        let thumb_focused = move |i: usize| {
            forced(InteractionPreview::Focused) || (keyboard && focused_thumb == Some(i))
        };
        let steps = (self.stops && step > 0. && max > min)
            .then(|| ((max - min) / step).round() as usize)
            .filter(|n| (1..=200).contains(n));

        let metrics = self.size.metrics();
        let target_width = if pressed {
            HANDLE_PRESSED_WIDTH
        } else {
            HANDLE_WIDTH
        };
        let handle = if classic {
            slider_geometry(SliderAppearance::Classic).1
        } else if preview.is_some() {
            target_width
        } else {
            motion::token_value(
                (id.clone(), "handle-width"),
                target_width,
                MotionToken::FastSpatial,
                window,
                cx,
            )
            .clamp(HANDLE_PRESSED_WIDTH, HANDLE_WIDTH)
        };
        let bubble = if !self.value_indicator || classic {
            0.
        } else if preview.is_some() {
            if pressed { 1. } else { 0. }
        } else {
            motion::token_value(
                (id.clone(), "value-indicator"),
                if pressed { 1. } else { 0. },
                MotionToken::FastEffects,
                window,
                cx,
            )
            .clamp(0., 1.)
        };
        let paint = slider_paint(disabled, self.color);
        // A handle's state layer: on-primary over the Expressive handle,
        // primary around the classic one.
        let layer = |i: usize| {
            if thumb_focused(i) {
                state::FOCUS
            } else if pressed {
                state::PRESS
            } else if hovered {
                state::HOVER
            } else {
                0.
            }
        };
        let live_hover = state::layer(paint.handle, paint.on_active, state::HOVER);
        let horizontal = self.axis == Axis::Horizontal;
        let cross = if classic {
            CLASSIC_TARGET
        } else {
            metrics.handle
        };
        let format = self
            .format
            .unwrap_or_else(|| std::rc::Rc::new(default_format));
        let ring_radius = rem(handle / 2.);
        let label = self.label;
        let orientation = if horizontal {
            Orientation::Horizontal
        } else {
            Orientation::Vertical
        };

        let thumbs = handles
            .iter()
            .zip(values.iter())
            .enumerate()
            .map(|(i, (&p, &v))| {
                let start = is_range && i == 0;
                let focused = thumb_focused(i);
                let thumb = SliderThumb::new(&self.state)
                    .axis(self.axis)
                    .start(start)
                    .disabled(!live)
                    .absolute()
                    .when(is_range, |thumb| {
                        // Each range thumb is a focusable slider of its own.
                        let handle = if start { &start_focus } else { &focus };
                        thumb
                            .role(Role::Slider)
                            .aria_label(label.clone())
                            .aria_numeric_value(v as f64)
                            .aria_min_numeric_value(min as f64)
                            .aria_max_numeric_value(max as f64)
                            .aria_orientation(orientation)
                            .when(live, |thumb| thumb.track_focus(handle).tab_stop(true))
                    });
                if classic {
                    let size = handle;
                    let thumb = thumb.rounded_full().bg(paint.handle);
                    let thumb = if horizontal {
                        thumb
                            .left(relative(p))
                            .ml(rem(-size / 2.))
                            .top(rem((CLASSIC_TARGET - size) / 2.))
                    } else {
                        thumb
                            .bottom(relative(p))
                            .mb(rem(-size / 2.))
                            .left(rem((CLASSIC_TARGET - size) / 2.))
                    }
                    .w(rem(size))
                    .h(rem(size));
                    let opacity = layer(i);
                    let halo = (CLASSIC_STATE_LAYER - size) / 2.;
                    let thumb = thumb.when(opacity > 0. && !disabled, |thumb| {
                        thumb.child(
                            div()
                                .absolute()
                                .top(rem(-halo))
                                .left(rem(-halo))
                                .size(rem(CLASSIC_STATE_LAYER))
                                .rounded_full()
                                .bg(paint.handle.opacity(opacity)),
                        )
                    });
                    return ButtonFocusRing::resolved(thumb, focused, ring_radius)
                        .into_any_element();
                }
                let handle_ink = if disabled {
                    paint.handle
                } else {
                    state::layer(paint.handle, paint.on_active, layer(i))
                };
                let thumb = thumb
                    .rounded(rem(2.))
                    .bg(handle_ink)
                    .when(live && !pressed && !focused, |t| {
                        t.hover(move |s| s.bg(live_hover))
                    });
                let thumb = if horizontal {
                    thumb
                        .left(relative(p))
                        .ml(rem(-handle / 2.))
                        .top_0()
                        .h_full()
                        .w(rem(handle))
                } else {
                    thumb
                        .bottom(relative(p))
                        .mb(rem(-handle / 2.))
                        .left_0()
                        .w_full()
                        .h(rem(handle))
                };
                let text = format(v);
                // A specimen labels only the end thumb, as the kit does.
                let labelled = preview.is_none() || i + 1 == handles.len();
                let thumb = thumb.when(bubble > 0.01 && labelled, |thumb| {
                    let pill = div()
                        .absolute()
                        .flex()
                        .items_center()
                        .justify_center()
                        .h(rem(BUBBLE_HEIGHT))
                        .min_w(rem(BUBBLE_MIN_WIDTH))
                        .px(rem(BUBBLE_PADDING))
                        .rounded_full()
                        .bg(color(ColorRole::INVERSE_SURFACE))
                        .text_color(color(ColorRole::INVERSE_ON_SURFACE))
                        .map(|pill| {
                            let scale = TypeRole::LabelLarge.scale();
                            pill.text_size(scale.font_size)
                                .letter_spacing(scale.letter_spacing)
                                .line_height(scale.line_height)
                                .font_weight(FontWeight(scale.font_weight as f32))
                        })
                        .whitespace_nowrap()
                        .opacity(bubble)
                        .child(text);
                    // Centered on the handle, outside its far edge: a fixed-size
                    // anchor box keeps the pill centered without measuring it.
                    let anchor = div().absolute().flex().items_center().justify_center();
                    let anchor = if horizontal {
                        anchor
                            .bottom(relative(1.))
                            .mb(rem(BUBBLE_GAP - (1. - bubble) * 8.))
                            .left(rem(handle / 2. - 60.))
                            .w(rem(120.))
                            .h(rem(BUBBLE_HEIGHT))
                    } else {
                        anchor
                            .right(relative(1.))
                            .mr(rem(BUBBLE_GAP + metrics.handle / 2. - (1. - bubble) * 8.))
                            .top(rem(handle / 2. - BUBBLE_HEIGHT / 2.))
                            .w(rem(120.))
                            .h(rem(BUBBLE_HEIGHT))
                            .justify_end()
                    };
                    thumb.child(anchor.child(pill.relative()))
                });
                ButtonFocusRing::resolved(thumb, focused, ring_radius).into_any_element()
            });

        let track_view = if classic {
            classic_track(&self.state, self.axis, disabled, &handles, &paint, thumbs)
        } else {
            let scale = f32::from(rem(1.).to_pixels(window.rem_size()));
            let track = TrackPaint {
                axis: self.axis,
                metrics,
                handle,
                handles: handles.clone(),
                centered: self.centered,
                steps,
                paint,
                scale,
            };
            // The icon sits inside the active track's leading end, or past the
            // handle on the inactive track when the active part is too short.
            let icon = self
                .icon
                .filter(|_| metrics.icon > 0. && !self.centered && !is_range)
                .map(|icon| {
                    let inset = ((metrics.track - metrics.icon) / 2.).min(16.);
                    (icon, inset, handles[0])
                });
            let indicator = SliderIndicator::new(&self.state)
                .absolute()
                .children(thumbs);
            let indicator = if horizontal {
                indicator
                    .left(rem(handle / 2.))
                    .right(rem(handle / 2.))
                    .top_0()
                    .bottom_0()
            } else {
                indicator
                    .top(rem(handle / 2.))
                    .bottom(rem(handle / 2.))
                    .left_0()
                    .right_0()
            };
            SliderTrack::new(&self.state)
                .axis(self.axis)
                .disabled(!live)
                .relative()
                .size_full()
                .child(
                    canvas(
                        |_, _, _| {},
                        move |bounds, _, window, _| paint_track(bounds, &track, window),
                    )
                    .absolute()
                    .size_full(),
                )
                .when_some(icon, |view, (icon, inset, p)| {
                    let size = metrics.icon;
                    let cross = (metrics.handle - size) / 2.;
                    // Too short an active track moves the icon past the handle.
                    let on_active = p >= ICON_MIN_FRACTION;
                    let ink = if on_active {
                        paint.on_active
                    } else {
                        paint.on_inactive
                    };
                    let slot = div()
                        .absolute()
                        .size(rem(size))
                        .child(icon.size(rem(size)).color(ink));
                    view.child(match (horizontal, on_active) {
                        (true, true) => slot.top(rem(cross)).left(rem(inset)),
                        (true, false) => slot
                            .top(rem(cross))
                            .left(relative(p))
                            .ml(rem(handle + HANDLE_GAP + inset)),
                        (false, true) => slot.left(rem(cross)).bottom(rem(inset)),
                        (false, false) => slot
                            .left(rem(cross))
                            .bottom(relative(p))
                            .mb(rem(handle + HANDLE_GAP + inset)),
                    })
                })
                .child(indicator)
                .into_any_element()
        };

        let slider = Slider::new(&self.state)
            .axis(self.axis)
            .accessibility_label(label)
            // A specimen is not operated, whichever state it shows.
            .disabled(!live)
            .when(live && !is_range, |slider| slider.track_focus(&focus))
            .size_full()
            .child(track_view);
        let state_entity = self.state.clone();
        let (release, release_out) = (press.clone(), press.clone());
        // A release can go where no listener hears it: a move with no button
        // says it happened. Only a press in progress listens to the window.
        let lost_release = (live && *press.read(cx)).then(|| {
            let press = press.clone();
            canvas(
                |_, _, _| {},
                move |_, _, window, _| {
                    window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                        if phase == DispatchPhase::Capture && !event.dragging() && *press.read(cx) {
                            press.update(cx, |pressed, _| *pressed = false);
                            window.refresh();
                        }
                    });
                },
            )
            .absolute()
            .size_0()
        });
        div()
            .id(id)
            .flex_shrink_0()
            .map(|root| {
                if horizontal {
                    root.w_full().h(rem(cross))
                } else {
                    root.h_full().w(rem(cross))
                }
            })
            .when(live, |root| {
                root.capture_any_mouse_down(move |event, window, cx| {
                    // Only the primary button presses (and releases) the slider.
                    if event.button != MouseButton::Left {
                        return;
                    }
                    press.update(cx, |pressed, _| *pressed = true);
                    window.refresh();
                })
                .on_mouse_up(MouseButton::Left, move |_, window, cx| {
                    release.update(cx, |pressed, _| *pressed = false);
                    window.refresh();
                })
                .on_mouse_up_out(MouseButton::Left, move |_, window, cx| {
                    release_out.update(cx, |pressed, _| *pressed = false);
                    window.refresh();
                })
                .on_key_down(move |event, window, cx| {
                    let key = match event.keystroke.key.as_str() {
                        "left" if mirrored_keys => "right",
                        "right" if mirrored_keys => "left",
                        key => key,
                    };
                    let moves_start = is_range && start_focus.is_focused(window);
                    state_entity.update(cx, |state, cx| {
                        let value = state.value();
                        let range = (state.min_value(), state.max_value());
                        let (current, bounds) = if moves_start {
                            (value.start(), (range.0, value.end()))
                        } else if value.is_range() {
                            (value.end(), (value.start(), range.1))
                        } else {
                            (value.end(), range)
                        };
                        let Some(next) =
                            stepped_value(key, current, bounds, range, state.step_value())
                        else {
                            return;
                        };
                        cx.stop_propagation();
                        // At an end the key changes nothing and reports nothing.
                        if next == current {
                            return;
                        }
                        if moves_start {
                            state.set_value((next, value.end()), window, cx);
                        } else if value.is_range() {
                            state.set_value((value.start(), next), window, cx);
                        } else {
                            state.set_value(next, window, cx);
                        }
                        cx.emit(SliderEvent::Change(state.value()));
                    });
                })
            })
            .child(slider)
            .children(lost_release)
            .into_any_element()
    }
}

/// The classic track: a 4dp inactive bar with the active part from the
/// start (or between a range's thumbs), along either axis.
fn classic_track(
    state: &Entity<SliderState>,
    axis: Axis,
    disabled: bool,
    handles: &[f32],
    paint: &SliderPaint,
    thumbs: impl IntoIterator<Item = AnyElement>,
) -> AnyElement {
    let track = slider_geometry(SliderAppearance::Classic).0;
    let inset = rem((CLASSIC_TARGET - track) / 2.);
    let (from, to) = match handles {
        [start, end] => (*start, *end),
        [end] => (0., *end),
        _ => (0., 0.),
    };
    let horizontal = axis == Axis::Horizontal;
    let active = div().absolute().rounded_full().bg(paint.active);
    let active = if horizontal {
        active
            .top_0()
            .h_full()
            .left(relative(from))
            .w(relative((to - from).max(0.)))
    } else {
        active
            .left_0()
            .w_full()
            .bottom(relative(from))
            .h(relative((to - from).max(0.)))
    };
    // The value-mapping bar spans the whole track; thumbs overhang its ends.
    let bar = SliderIndicator::new(state)
        .absolute()
        .rounded_full()
        .bg(paint.inactive)
        .child(active);
    let bar = if horizontal {
        bar.left_0().top(inset).w_full().h(rem(track))
    } else {
        bar.bottom_0().left(inset).h_full().w(rem(track))
    };
    // Thumbs are placed in the 48dp target, relative to the track view.
    let thumbs = div().absolute().inset_0().children(thumbs);
    SliderTrack::new(state)
        .axis(axis)
        .disabled(disabled)
        .relative()
        .size_full()
        .child(bar)
        .child(thumbs)
        .into_any_element()
}

#[cfg(test)]
thread_local! {
    /// Whether the slider last rendered on this thread shows a press.
    static SLIDER_PRESSED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use std::{cell::Cell, rc::Rc};

    #[test]
    fn sizes_follow_the_m3_expressive_tokens() {
        let tracks = SliderSize::ALL.map(|s| s.metrics().track);
        assert_eq!(tracks, [16., 24., 40., 56., 96.]);
        let handles = SliderSize::ALL.map(|s| s.metrics().handle);
        assert_eq!(handles, [44., 44., 52., 68., 108.]);
        let corners = SliderSize::ALL.map(|s| s.metrics().corner);
        assert_eq!(corners, [8., 8., 12., 16., 28.]);
        let icons = SliderSize::ALL.map(|s| s.metrics().icon);
        assert_eq!(icons, [0., 0., 24., 24., 32.]);
        assert_eq!(slider_geometry(SliderAppearance::Classic), (4., 20., 20.));
    }

    #[test]
    fn the_track_splits_around_the_handle_with_6dp_gaps() {
        // A 4dp handle at 50% of 204dp is centered at 102dp.
        let s = track_segments(204., 4., &[0.5], false);
        assert_eq!(s.len(), 2);
        assert_eq!(
            (s[0].start, s[0].end, s[0].active),
            (0., 102. - 2. - 6., true)
        );
        assert_eq!(
            (s[1].start, s[1].end, s[1].active),
            (102. + 2. + 6., 204., false)
        );
        assert!(s[0].outer_start && !s[0].outer_end && s[1].outer_end);
        // At 0 the active track disappears; at 1 the inactive one does.
        assert!(
            track_segments(204., 4., &[0.], false)
                .iter()
                .all(|s| !s.active)
        );
        assert!(
            track_segments(204., 4., &[1.], false)
                .iter()
                .all(|s| s.active)
        );
    }

    #[test]
    fn centered_and_range_tracks() {
        let right = track_segments(200., 4., &[0.75], true);
        let active: Vec<_> = right.iter().filter(|s| s.active).collect();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].start, 100.);
        let left = track_segments(200., 4., &[0.25], true);
        assert_eq!(left.iter().find(|s| s.active).unwrap().end, 100.);
        let range = track_segments(204., 4., &[0.25, 0.75], false);
        assert_eq!(
            range.iter().map(|s| s.active).collect::<Vec<_>>(),
            [false, true, false]
        );
        assert!(range[0].outer_start && range[2].outer_end);
    }

    #[test]
    fn stops_mark_steps_and_hide_under_the_handle() {
        let segments = track_segments(204., 4., &[0.5], false);
        // Ten steps: eleven stops, the middle one hidden by the handle.
        let stops = stop_positions(204., 4., &segments, Some(10), &[0.5]);
        assert_eq!(stops.len(), 10);
        assert_eq!(stops.iter().filter(|(_, active)| *active).count(), 5);
        assert_eq!(stops[0].0, STOP_INSET, "end stops sit 6dp in");
        // Continuous: one stop at the inactive end only.
        let end = stop_positions(204., 4., &segments, None, &[0.5]);
        assert_eq!(end, vec![(204. - STOP_INSET, false)]);
    }

    #[test]
    fn slider_uses_kit_roles_and_unified_disabled_tokens() {
        let enabled = slider_paint(false, SliderColor::Primary);
        assert_eq!(enabled.active, color(ColorRole::PRIMARY));
        assert_eq!(enabled.inactive, color(ColorRole::SECONDARY_CONTAINER));
        assert_eq!(enabled.on_active, color(ColorRole::ON_PRIMARY));
        assert_eq!(
            enabled.on_inactive,
            color(ColorRole::ON_SECONDARY_CONTAINER)
        );
        let disabled = slider_paint(true, SliderColor::Tertiary);
        let on_surface = color(ColorRole::ON_SURFACE);
        assert_eq!(disabled.active, state::disabled(on_surface));
        assert_eq!(disabled.handle, state::disabled(on_surface));
        assert_eq!(
            disabled.inactive,
            on_surface.opacity(state::DISABLED_CONTAINER)
        );
    }

    #[test]
    fn slider_colors_use_their_own_role_family() {
        let tertiary = slider_paint(false, SliderColor::Tertiary);
        assert_eq!(tertiary.active, color(ColorRole::TERTIARY));
        assert_eq!(tertiary.handle, color(ColorRole::TERTIARY));
        assert_eq!(tertiary.inactive, color(ColorRole::TERTIARY_CONTAINER));
        assert_eq!(tertiary.on_active, color(ColorRole::ON_TERTIARY));
        let error = slider_paint(false, SliderColor::Error);
        assert_eq!(error.active, color(ColorRole::ERROR));
        assert_eq!(error.inactive, color(ColorRole::ERROR_CONTAINER));
        assert_eq!(error.on_inactive, color(ColorRole::ON_ERROR_CONTAINER));
        let secondary = slider_paint(false, SliderColor::Secondary);
        assert_eq!(secondary.active, color(ColorRole::SECONDARY));
        assert_eq!(
            slider_paint(true, SliderColor::Error),
            slider_paint(true, SliderColor::Primary),
            "disabled ignores the accent"
        );
    }

    #[test]
    fn keys_step_page_and_jump() {
        let full = (0., 100.);
        assert_eq!(stepped_value("right", 50., full, full, 1.), Some(51.));
        assert_eq!(stepped_value("down", 0., full, full, 1.), Some(0.));
        assert_eq!(stepped_value("pageup", 50., full, full, 1.), Some(60.));
        assert_eq!(stepped_value("end", 5., full, full, 1.), Some(100.));
        assert_eq!(stepped_value("a", 5., full, full, 1.), None);
        assert_eq!(default_format(42.0001), "42");
        assert_eq!(default_format(0.25), "0.2");
    }

    #[test]
    fn page_keys_stay_on_the_step_grid() {
        let full = (0., 100.);
        // A tenth of 0..100 is 10, off a step-3 grid: page by 9 instead.
        assert_eq!(stepped_value("pageup", 0., full, full, 3.), Some(9.));
        assert_eq!(stepped_value("pagedown", 99., full, full, 3.), Some(90.));
        // The grid counts from min.
        assert_eq!(
            stepped_value("right", 7., (2., 52.), (2., 52.), 5.),
            Some(12.)
        );
        // A range thumb stops at the other thumb.
        assert_eq!(stepped_value("end", 10., (0., 40.), full, 1.), Some(40.));
    }

    #[test]
    fn an_invalid_range_does_not_panic() {
        assert_eq!(stepped_value("right", 5., (10., 0.), (10., 0.), 1.), None);
        assert_eq!(
            stepped_value("right", f32::NAN, (0., 1.), (0., 1.), 1.),
            None
        );
        assert_eq!(
            stepped_value("right", 0.5, (0., f32::NAN), (0., f32::NAN), 1.),
            None
        );
    }

    /// A slider configured by `configure`, over `state`.
    struct Configured(Entity<SliderState>, fn(MaterialSlider) -> MaterialSlider);
    impl Render for Configured {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(crate::tokens::typography::BASE_PX));
            div().w(px(240.)).h(px(240.)).p(px(60.)).child(
                div()
                    .size_full()
                    .debug_selector(|| "slider".into())
                    .child((self.1)(MaterialSlider::new(&self.0, "Level"))),
            )
        }
    }

    fn configured(
        cx: &mut TestAppContext,
        state: SliderState,
        configure: fn(MaterialSlider) -> MaterialSlider,
    ) -> (Entity<SliderState>, &mut VisualTestContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, cx| Configured(cx.new(|_| state), configure));
        draw(cx);
        let state = view.read_with(cx, |v, _| v.0.clone());
        (state, cx)
    }

    fn count_changes(state: &Entity<SliderState>, cx: &mut VisualTestContext) -> Rc<Cell<usize>> {
        let changes = Rc::new(Cell::new(0));
        cx.update(|_, cx| {
            let changes = changes.clone();
            cx.subscribe(state, move |_, event: &SliderEvent, _| {
                if matches!(event, SliderEvent::Change(_)) {
                    changes.set(changes.get() + 1)
                }
            })
            .detach();
        });
        changes
    }

    #[gpui::test]
    fn a_classic_slider_is_tabbable_and_steps_with_arrows(cx: &mut TestAppContext) {
        let (state, cx) = configured(cx, SliderState::new().max(100.).default_value(30.), |s| {
            s.appearance(SliderAppearance::Classic)
        });
        cx.update(|window, cx| window.focus_next(cx));
        cx.simulate_keystrokes("right right");
        assert_eq!(state.read_with(cx, |s, _| s.value().end()), 32.);
    }

    #[gpui::test]
    fn a_vertical_classic_slider_follows_its_axis(cx: &mut TestAppContext) {
        let (_, cx) = configured(cx, SliderState::new().max(100.).default_value(30.), |s| {
            s.appearance(SliderAppearance::Classic).vertical()
        });
        let bounds = cx.debug_bounds("slider").unwrap();
        let slider = cx.update(|window, _| {
            gpui_base::test_support::snapshots(window)
                .into_iter()
                .find(|s| s.role() == Some(Role::Slider))
                .unwrap()
        });
        assert_eq!(slider.bounds().size.width, px(48.), "a 48dp wide target");
        assert_eq!(slider.bounds().size.height, bounds.size.height);
    }

    #[gpui::test]
    fn keyboard_focus_lands_on_the_slider_node(cx: &mut TestAppContext) {
        let (_, cx) = configured(cx, SliderState::new().max(100.).default_value(30.), |s| s);
        cx.update(|window, cx| window.focus_next(cx));
        draw(cx);
        let slider = cx.update(|window, _| {
            gpui_base::test_support::snapshots(window)
                .into_iter()
                .find(|s| s.role() == Some(Role::Slider))
                .unwrap()
        });
        assert_eq!(slider.label(), Some("Level"));
        assert_eq!(slider.focused(), Some(true), "{slider:?}");
    }

    #[gpui::test]
    fn tab_moves_between_range_thumbs_and_each_takes_arrows(cx: &mut TestAppContext) {
        let (state, cx) = configured(
            cx,
            SliderState::new().max(100.).default_value((20., 60.)),
            |s| s,
        );
        cx.update(|window, cx| window.focus_next(cx));
        cx.simulate_keystrokes("right");
        assert_eq!(state.read_with(cx, |s, _| s.value()), (21., 60.).into());
        cx.update(|window, cx| window.focus_next(cx));
        cx.simulate_keystrokes("left left");
        assert_eq!(state.read_with(cx, |s, _| s.value()), (21., 58.).into());
    }

    #[gpui::test]
    fn keys_at_an_end_report_no_change(cx: &mut TestAppContext) {
        let (state, cx) = configured(cx, SliderState::new().max(100.).default_value(100.), |s| s);
        let changes = count_changes(&state, cx);
        cx.update(|window, cx| window.focus_next(cx));
        cx.simulate_keystrokes("right end pageup");
        assert_eq!(changes.get(), 0);
        cx.simulate_keystrokes("left");
        assert_eq!(changes.get(), 1);
    }

    #[gpui::test]
    fn only_the_primary_button_presses_the_slider(cx: &mut TestAppContext) {
        let (_, cx) = configured(cx, SliderState::new().max(100.).default_value(30.), |s| s);
        let thumb_width = |cx: &mut VisualTestContext| {
            cx.update(|window, _| {
                gpui_base::test_support::find(window, &[], &ElementId::from(("slider-thumb", 0u32)))
                    .unwrap()
                    .bounds()
                    .size
                    .width
            })
        };
        let resting = thumb_width(cx);
        assert_eq!(resting, px(HANDLE_WIDTH));
        let at = cx.debug_bounds("slider").unwrap().center();
        cx.simulate_mouse_down(at, MouseButton::Right, Modifiers::default());
        cx.simulate_mouse_up(at, MouseButton::Right, Modifiers::default());
        draw(cx);
        assert_eq!(thumb_width(cx), resting, "a right click left it pressed");
    }

    struct SliderHarness(Entity<SliderState>, Option<InteractionPreview>);
    impl Render for SliderHarness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            let slider = MaterialSlider::new(&self.0, "Level");
            let slider = match self.1 {
                Some(state) => slider.preview_state(state),
                None => slider,
            };
            div()
                .w(px(200.))
                .p(px(60.))
                .child(div().debug_selector(|| "slider".into()).child(slider))
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|w, cx| {
            w.refresh();
            w.draw(cx).clear(cx);
        });
    }

    #[gpui::test]
    fn keyboard_steps_the_value_and_emits_changes(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let (view, cx) = cx.add_window_view(|_, cx| {
            SliderHarness(
                cx.new(|_| SliderState::new().max(100.).default_value(30.)),
                None,
            )
        });
        draw(cx);
        let state = view.read_with(cx, |v, _| v.0.clone());
        let changes = std::rc::Rc::new(std::cell::Cell::new(0));
        cx.update(|_, cx| {
            let changes = changes.clone();
            cx.subscribe(&state, move |_, _: &SliderEvent, _| {
                changes.set(changes.get() + 1)
            })
            .detach();
        });
        let bounds = cx.debug_bounds("slider").unwrap();
        cx.simulate_click(bounds.center(), Modifiers::default());
        draw(cx);
        let before = state.read_with(cx, |s, _| s.value().end());
        cx.simulate_keystrokes("right right");
        draw(cx);
        let after = state.read_with(cx, |s, _| s.value().end());
        assert_eq!(after, before + 2.);
        cx.simulate_keystrokes("end");
        assert_eq!(state.read_with(cx, |s, _| s.value().end()), 100.);
        assert!(changes.get() >= 3);
    }

    #[gpui::test]
    fn a_pressed_specimen_shows_the_narrow_handle(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let (_, cx) = cx.add_window_view(|_, cx| {
            SliderHarness(
                cx.new(|_| SliderState::new().max(100.).default_value(30.)),
                Some(InteractionPreview::Pressed),
            )
        });
        draw(cx);
        let slider = cx.debug_bounds("slider").unwrap();
        assert_eq!(slider.size.height, px(44.), "the XSmall handle height");
    }

    #[gpui::test]
    fn a_lost_release_lets_go_of_the_handle(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let (_, cx) = cx.add_window_view(|_, cx| {
            SliderHarness(
                cx.new(|_| SliderState::new().max(100.).default_value(30.)),
                None,
            )
        });
        draw(cx);
        let center = cx.debug_bounds("slider").unwrap().center();
        cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        assert!(SLIDER_PRESSED.with(|p| p.get()), "pressed");
        cx.simulate_mouse_move(
            point(px(5.), px(5.)),
            None::<MouseButton>,
            Modifiers::default(),
        );
        draw(cx);
        assert!(!SLIDER_PRESSED.with(|p| p.get()), "it let go");
    }

    #[gpui::test]
    fn disabling_during_a_press_ends_it(cx: &mut TestAppContext) {
        struct Toggled(Entity<SliderState>, bool);
        impl Render for Toggled {
            fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                window.set_rem_size(px(14.));
                div().w(px(200.)).p(px(60.)).child(
                    div()
                        .debug_selector(|| "slider".into())
                        .child(MaterialSlider::new(&self.0, "Level").disabled(self.1)),
                )
            }
        }
        cx.update(gpui_base::init);
        let (view, cx) = cx.add_window_view(|_, cx| {
            Toggled(
                cx.new(|_| SliderState::new().max(100.).default_value(50.)),
                false,
            )
        });
        draw(cx);
        let center = cx.debug_bounds("slider").unwrap().center();
        cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        view.update(cx, |view, cx| {
            view.1 = true;
            cx.notify();
        });
        draw(cx);
        cx.simulate_mouse_up(center, MouseButton::Left, Modifiers::default());
        draw(cx);
        view.update(cx, |view, cx| {
            view.1 = false;
            cx.notify();
        });
        draw(cx);
        assert!(
            !SLIDER_PRESSED.with(|p| p.get()),
            "enabled again, it is not held"
        );
    }

    #[gpui::test]
    fn a_specimen_slider_does_not_move(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let (view, cx) = cx.add_window_view(|_, cx| {
            SliderHarness(
                cx.new(|_| SliderState::new().max(100.).default_value(30.)),
                Some(InteractionPreview::Hovered),
            )
        });
        draw(cx);
        let before = view.read_with(cx, |view, cx| view.0.read(cx).value());
        let bounds = cx.debug_bounds("slider").unwrap();
        let spot = point(bounds.left() + px(5.), bounds.center().y);
        cx.simulate_click(spot, Modifiers::default());
        draw(cx);
        assert_eq!(
            view.read_with(cx, |view, cx| view.0.read(cx).value()),
            before
        );
    }
}
