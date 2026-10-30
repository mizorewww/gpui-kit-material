//! M3 Expressive linear progress: flat or wavy, 4dp or thick 8dp, measured
//! or indeterminate. Reference: <https://m3e.kyralabs.dev/components/progress/>
//! and the Figma kit's linear progress sets.
use super::paint::{dot, rounded_bar, stroke_polyline};
use crate::{color::ColorRole, theme::color};
use gpui::{prelude::*, *};
use gpui_base::{Easing, Progress};
use std::time::Duration;

/// The default and thick track/indicator thickness.
pub const PROGRESS_THICKNESS: Pixels = px(4.);
pub const PROGRESS_THICK_THICKNESS: Pixels = px(8.);
/// Space between the active indicator's end and the track (track-active-space).
const TRACK_GAP: f32 = 4.;
/// The stop indicator at the track's end.
const STOP_SIZE: f32 = 4.;
/// The wave's peak offset from the centerline: a 10dp wave height for the
/// 4dp indicator and 14dp for the thick one.
pub(super) const WAVE_AMPLITUDE: f32 = 3.;
/// Determinate and indeterminate wavelengths.
const DETERMINATE_WAVELENGTH: f32 = 40.;
const INDETERMINATE_WAVELENGTH: f32 = 20.;
/// The indeterminate loop.
pub const LINEAR_INDETERMINATE_MS: f32 = 1750.;
/// A wave travels one wavelength per second.
const WAVE_PERIOD: Duration = Duration::from_secs(1);

/// The shape of a progress indicator's active part.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProgressShape {
    #[default]
    Flat,
    /// M3 Expressive: the active indicator is a travelling wave.
    Wavy,
}

/// The two indeterminate lines `elapsed_ms` into the loop, each as
/// (tail, head) fractions of the track. A line is visible while its head
/// leads its tail. Compose's keyframes: the first line's head runs over
/// 0-1000ms and tail over 250-1250ms; the second's head over 650-1500ms and
/// tail over 900-1750ms, each on its own cubic easing.
pub fn linear_indeterminate_lines(elapsed_ms: f32) -> [(f32, f32); 2] {
    let t = if elapsed_ms.is_finite() {
        elapsed_ms.rem_euclid(LINEAR_INDETERMINATE_MS)
    } else {
        0.
    };
    let key = |delay: f32, duration: f32, (x1, y1, x2, y2): (f32, f32, f32, f32)| {
        let progress = ((t - delay) / duration).clamp(0., 1.);
        Easing::cubic_bezier(x1, y1, x2, y2)
            .map(|easing| easing.sample(progress))
            .unwrap_or(progress)
    };
    [
        (
            key(250., 1000., (0.4, 0., 1., 1.)),
            key(0., 1000., (0.2, 0., 0.8, 1.)),
        ),
        (
            key(900., 850., (0.1, 0., 0.45, 1.)),
            key(650., 850., (0., 0., 0.65, 1.)),
        ),
    ]
}

/// M3 Expressive linear progress indicator.
///
/// `value` makes it determinate: the active indicator, a 4dp gap, the track
/// and the stop dot at its end; `None` plays the two-line indeterminate
/// motion. A wavy indicator flattens near empty and full (at or below 10%,
/// at or above 95%). Reduced motion holds the wave still and shows one
/// fixed indeterminate frame. The caller supplies the accessible label.
#[derive(gpui::IntoElement)]
pub struct LinearProgress {
    id: ElementId,
    label: Option<SharedString>,
    value: Option<f32>,
    thickness: Pixels,
    shape: ProgressShape,
    muted: bool,
    playback: Option<RepeatAnimationPlayback>,
}

impl LinearProgress {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::unlabelled(id).label(label)
    }

    pub(super) fn unlabelled(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            value: None,
            thickness: PROGRESS_THICKNESS,
            shape: ProgressShape::Flat,
            muted: false,
            playback: None,
        }
    }

    pub(super) fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// A measured fraction (0-1), or `None` for indeterminate.
    pub fn value(mut self, value: Option<f32>) -> Self {
        self.value = value;
        self
    }

    /// The track and indicator thickness (4dp by default).
    pub fn thickness(mut self, thickness: Pixels) -> Self {
        self.thickness = thickness;
        self
    }

    /// The 8dp thick indicator.
    pub fn thick(self) -> Self {
        self.thickness(PROGRESS_THICK_THICKNESS)
    }

    pub fn shape(mut self, shape: ProgressShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn wavy(self) -> Self {
        self.shape(ProgressShape::Wavy)
    }

    /// A paused or stopping process keeps its fraction in the outline color.
    pub fn muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }

    /// A caller-owned clock that survives virtual unmounting.
    pub fn playback(mut self, playback: RepeatAnimationPlayback) -> Self {
        self.playback = Some(playback);
        self
    }
}

/// Everything one painted frame needs.
#[derive(Clone, Copy, Debug)]
struct BarPaint {
    thickness: f32,
    amplitude: f32,
    wavelength: f32,
    /// The wave's phase, in fractions of a wavelength.
    phase: f32,
    ink: Rgba,
    track: Rgba,
}

/// The active intervals of one frame, in fractions of the track.
#[derive(Clone, Copy, Debug)]
enum Active {
    Measured(f32),
    Lines([(f32, f32); 2]),
}

/// The track pieces left between the active intervals (px), each kept
/// `TRACK_GAP` clear of an adjacent active interval.
pub(super) fn track_pieces(width: f32, active: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let mut active: Vec<(f32, f32)> = active.iter().copied().filter(|(a, b)| b > a).collect();
    active.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut pieces = Vec::new();
    let mut start = 0.;
    let mut start_gap = 0.;
    for (a, b) in active {
        let end = a - TRACK_GAP.min(b - a);
        if end - (start + start_gap) > 0.5 {
            pieces.push((start + start_gap, end));
        }
        start = start.max(b);
        start_gap = TRACK_GAP.min(b - a);
    }
    if width - (start + start_gap) > 0.5 {
        pieces.push((start + start_gap, width));
    }
    pieces
}

fn paint_bar(bounds: Bounds<Pixels>, active: Active, paint: BarPaint, window: &mut Window) {
    let width = f32::from(bounds.size.width);
    let x0 = f32::from(bounds.origin.x);
    let cy = f32::from(bounds.center().y);
    let t = paint.thickness;
    let intervals: Vec<(f32, f32)> = match active {
        Active::Measured(amount) => vec![(0., width * amount)],
        Active::Lines(lines) => lines
            .iter()
            .map(|&(tail, head)| (width * tail, width * head))
            .collect(),
    };
    for (a, b) in track_pieces(width, &intervals) {
        rounded_bar(window, x0 + a, x0 + b, cy, t, paint.track);
    }
    if let Active::Measured(amount) = active
        && amount < 0.995
    {
        // The stop dot sits centered in the track's rounded end.
        let size = STOP_SIZE.min(t);
        dot(
            window,
            point(px(x0 + width - t / 2.), px(cy)),
            size,
            paint.ink,
        );
    }
    for (a, b) in intervals {
        if b - a <= 0.5 {
            continue;
        }
        if paint.amplitude <= 0.01 {
            rounded_bar(window, x0 + a, x0 + b, cy, t, paint.ink);
            continue;
        }
        // The wave's centerline, inset by the round caps.
        let (from, to) = (a + t / 2., b - t / 2.);
        if to <= from {
            dot(
                window,
                point(px(x0 + (a + b) / 2.), px(cy)),
                t.min(b - a),
                paint.ink,
            );
            continue;
        }
        let k = std::f32::consts::TAU / paint.wavelength;
        let y = |x: f32| cy + paint.amplitude * (k * x - std::f32::consts::TAU * paint.phase).sin();
        let steps = ((to - from) / 2.).ceil().max(1.) as usize;
        let points: Vec<Point<Pixels>> = (0..=steps)
            .map(|i| {
                let x = from + (to - from) * i as f32 / steps as f32;
                point(px(x0 + x), px(y(x)))
            })
            .collect();
        stroke_polyline(window, &points, t, paint.ink);
        dot(window, points[0], t, paint.ink);
        dot(window, points[steps], t, paint.ink);
    }
}

fn bar_canvas(active: Active, paint: BarPaint) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| paint_bar(bounds, active, paint, window),
    )
    .size_full()
}

/// Determinate amplitude: flat at or below 10% and at or above 95%.
pub(super) fn wave_amplitude(amount: f32) -> f32 {
    if amount > 0.1 && amount < 0.95 {
        WAVE_AMPLITUDE
    } else {
        0.
    }
}

impl RenderOnce for LinearProgress {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let wavy = self.shape == ProgressShape::Wavy;
        let thickness = f32::from(self.thickness).max(1.);
        let height = if wavy {
            thickness + 2. * WAVE_AMPLITUDE
        } else {
            thickness
        };
        let target = self
            .value
            .map(|v| if v.is_finite() { v.clamp(0., 1.) } else { 0. });
        let mut paint = BarPaint {
            thickness,
            amplitude: 0.,
            wavelength: DETERMINATE_WAVELENGTH,
            phase: 0.,
            ink: color(if self.muted {
                ColorRole::OUTLINE
            } else {
                ColorRole::PRIMARY
            }),
            track: color(ColorRole::SECONDARY_CONTAINER),
        };
        let root = Progress::new(self.id.clone())
            .when_some(self.label, |p, label| p.accessibility_label(label))
            .indeterminate(target.is_none())
            .value(target.unwrap_or(0.) * 100.)
            .w_full()
            .h(px(height));
        let id = self.id;
        let playback = self.playback;
        match target {
            Some(target) => {
                // Spatial overshoot must not extend a fraction outside its track.
                let amount = crate::motion::value(id.clone(), target, window, cx).clamp(0., 1.);
                if !wavy {
                    return root.child(bar_canvas(Active::Measured(amount), paint));
                }
                paint.amplitude = crate::motion::value(
                    (id.clone(), "wave-amplitude"),
                    wave_amplitude(target),
                    window,
                    cx,
                )
                .max(0.);
                root.child(
                    div()
                        .size_full()
                        .with_animation(id, Animation::new(WAVE_PERIOD).repeat(), move |slot, t| {
                            slot.child(bar_canvas(
                                Active::Measured(amount),
                                BarPaint { phase: t, ..paint },
                            ))
                        })
                        .when_some(playback, |a, p| a.with_repeat_playback(p))
                        .repeat_when_visible(),
                )
            }
            None => {
                if wavy {
                    paint.amplitude = WAVE_AMPLITUDE;
                    paint.wavelength = INDETERMINATE_WAVELENGTH;
                }
                if cx.reduce_motion() {
                    // One representative frame: the first line mid-track.
                    let lines = linear_indeterminate_lines(600.);
                    return root.child(bar_canvas(Active::Lines(lines), paint));
                }
                let loop_ms = LINEAR_INDETERMINATE_MS;
                root.child(
                    div()
                        .size_full()
                        .with_animation(
                            id,
                            Animation::new(Duration::from_secs_f32(loop_ms / 1000.)).repeat(),
                            move |slot, t| {
                                #[cfg(test)]
                                super::tests::record_repeat_phase(t);
                                let elapsed = t * loop_ms;
                                slot.child(bar_canvas(
                                    Active::Lines(linear_indeterminate_lines(elapsed)),
                                    BarPaint {
                                        phase: elapsed / 1000.,
                                        ..paint
                                    },
                                ))
                            },
                        )
                        .when_some(playback, |a, p| a.with_repeat_playback(p))
                        .repeat_when_visible(),
                )
            }
        }
    }
}

#[cfg(test)]
mod linear_tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn two_lines_follow_the_reference_keyframes() {
        let at = linear_indeterminate_lines;
        assert_eq!(at(0.), [(0., 0.), (0., 0.)]);
        // The first line's head leads; its tail starts at 250ms.
        let [(tail, head), second] = at(250.);
        assert_eq!(tail, 0.);
        assert!(head > 0.1 && head < 0.5, "{head}");
        assert_eq!(second, (0., 0.));
        // The second line starts at 650ms and ends with the loop.
        assert!(at(800.)[1].1 > 0.);
        assert_eq!(at(1250.)[0], (1., 1.));
        let end = at(LINEAR_INDETERMINATE_MS - 0.01)[1];
        assert!(end.1 - end.0 < 0.001, "{end:?}");
        for ms in (0..1750).step_by(25) {
            for (tail, head) in at(ms as f32) {
                assert!((0. ..=1.).contains(&tail) && (0. ..=1.).contains(&head));
                assert!(head >= tail - 1e-6, "{ms}: {tail} {head}");
            }
        }
    }

    #[test]
    fn track_pieces_keep_a_4dp_gap_beside_active_lines() {
        assert_eq!(track_pieces(300., &[(0., 0.)]), vec![(0., 300.)]);
        assert_eq!(track_pieces(300., &[(0., 150.)]), vec![(154., 300.)]);
        assert_eq!(
            track_pieces(300., &[(200., 260.), (30., 90.)]),
            vec![(0., 26.), (94., 196.), (264., 300.)]
        );
        // A line shorter than the gap only clears its own length.
        assert_eq!(track_pieces(100., &[(0., 2.)]), vec![(4., 100.)]);
        assert!(track_pieces(100., &[(0., 100.)]).is_empty());
    }

    #[test]
    fn the_wave_flattens_near_empty_and_full() {
        assert_eq!(wave_amplitude(0.1), 0.);
        assert_eq!(wave_amplitude(0.5), WAVE_AMPLITUDE);
        assert_eq!(wave_amplitude(0.95), 0.);
        assert_eq!(WAVE_AMPLITUDE * 2. + 4., 10.);
        assert_eq!(WAVE_AMPLITUDE * 2. + 8., 14.);
    }
}
