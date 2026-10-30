//! M3 Expressive circular progress: flat or wavy, 4dp or thick 8dp,
//! measured or indeterminate, as a standalone indicator. The Figma kit's
//! boxes: 40dp flat, 44dp thick flat, 48dp wavy, 52dp thick wavy.
//! Reference: <https://m3e.kyralabs.dev/components/progress/>.
use super::{
    circular::{CircularProgressFrame, circular_indeterminate_frame},
    linear::{PROGRESS_THICK_THICKNESS, PROGRESS_THICKNESS, ProgressShape, wave_amplitude},
    paint::{dot, stroke_polyline},
};
use crate::{color::ColorRole, theme::color};
use gpui::{prelude::*, *};
use gpui_base::Progress;
use std::{f32::consts::TAU, time::Duration};

/// The wavy circle's peak offset from its mean radius.
const CIRCULAR_WAVE_AMPLITUDE: f32 = 2.;
/// The wave's length along the circle.
const CIRCULAR_WAVELENGTH: f32 = 15.;
/// Clear space between the active arc's caps and the track.
const TRACK_GAP: f32 = 4.;
const INDETERMINATE_MS: f32 = 6000.;

/// The indicator box for a shape and thickness, in dp.
pub fn circular_progress_size(shape: ProgressShape, thickness: Pixels) -> f32 {
    let thick = f32::from(thickness) - 4.;
    match shape {
        ProgressShape::Flat => 40. + thick,
        ProgressShape::Wavy => 48. + thick,
    }
}

/// M3 Expressive circular progress indicator.
///
/// `value` makes it determinate (the arc starts at twelve o'clock with a
/// track and gap after it); `None` plays the reference's six-second growing
/// arc, whose track is hidden unless [`Self::show_track`] asks for it.
/// Reduced motion holds still. The caller supplies the accessible label.
#[derive(gpui::IntoElement)]
pub struct CircularProgress {
    id: ElementId,
    label: SharedString,
    value: Option<f32>,
    thickness: Pixels,
    shape: ProgressShape,
    show_track: Option<bool>,
    playback: Option<RepeatAnimationPlayback>,
}

impl CircularProgress {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: None,
            thickness: PROGRESS_THICKNESS,
            shape: ProgressShape::Flat,
            show_track: None,
            playback: None,
        }
    }

    pub fn value(mut self, value: Option<f32>) -> Self {
        self.value = value;
        self
    }

    pub fn thick(mut self) -> Self {
        self.thickness = PROGRESS_THICK_THICKNESS;
        self
    }

    pub fn shape(mut self, shape: ProgressShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn wavy(self) -> Self {
        self.shape(ProgressShape::Wavy)
    }

    /// Whether the track shows (default: only while determinate).
    pub fn show_track(mut self, show: bool) -> Self {
        self.show_track = Some(show);
        self
    }

    /// A caller-owned clock that survives virtual unmounting.
    pub fn playback(mut self, playback: RepeatAnimationPlayback) -> Self {
        self.playback = Some(playback);
        self
    }
}

#[derive(Clone, Copy, Debug)]
struct RingPaint {
    thickness: f32,
    amplitude: f32,
    /// The wave's phase in fractions of a wavelength.
    phase: f32,
    show_track: bool,
    ink: Rgba,
    track: Rgba,
}

/// The mean radius of the ring in a `size` box.
fn ring_radius(size: f32, thickness: f32, wavy: bool) -> f32 {
    size / 2. - thickness / 2. - if wavy { CIRCULAR_WAVE_AMPLITUDE } else { 0. }
}

fn paint_ring(
    bounds: Bounds<Pixels>,
    frame: CircularProgressFrame,
    paint: RingPaint,
    wavy: bool,
    window: &mut Window,
) {
    let size = f32::from(bounds.size.width.min(bounds.size.height));
    let center = bounds.center();
    let t = paint.thickness;
    let radius = ring_radius(size, t, wavy).max(1.);
    let start = frame.rotation_degrees.to_radians();
    let sweep = frame.sweep.clamp(0., 1.) * TAU;
    // The angle between cap centers that leaves TRACK_GAP clear.
    let gap = if sweep > 0. {
        ((TRACK_GAP + t) / radius).min(sweep)
    } else {
        0.
    };
    let at = |angle: f32, r: f32| {
        point(
            center.x + px(r * angle.cos()),
            center.y + px(r * angle.sin()),
        )
    };
    let arc = |from: f32, length: f32, wave: f32| -> Vec<Point<Pixels>> {
        let steps = ((length * radius / 2.).ceil() as usize).max(2);
        let waves = (TAU * radius / CIRCULAR_WAVELENGTH).round().max(1.);
        (0..=steps)
            .map(|i| {
                let angle = from + length * i as f32 / steps as f32;
                let r = radius + wave * (waves * angle - TAU * paint.phase).sin();
                at(angle, r)
            })
            .collect()
    };
    if paint.show_track {
        let length = TAU - sweep - 2. * gap;
        if sweep <= 0. {
            stroke_polyline(window, &arc(start, TAU, 0.), t, paint.track);
        } else if length > 0. {
            let points = arc(start + sweep + gap, length, 0.);
            stroke_polyline(window, &points, t, paint.track);
            dot(window, points[0], t, paint.track);
            dot(window, *points.last().unwrap(), t, paint.track);
        }
    }
    if sweep > 0. {
        let points = arc(start, sweep, paint.amplitude);
        stroke_polyline(window, &points, t, paint.ink);
        dot(window, points[0], t, paint.ink);
        dot(window, *points.last().unwrap(), t, paint.ink);
    }
}

fn ring_canvas(frame: CircularProgressFrame, paint: RingPaint, wavy: bool) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| paint_ring(bounds, frame, paint, wavy, window),
    )
    .size_full()
}

impl RenderOnce for CircularProgress {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let wavy = self.shape == ProgressShape::Wavy;
        let box_size = circular_progress_size(self.shape, self.thickness);
        let target = self
            .value
            .map(|v| if v.is_finite() { v.clamp(0., 1.) } else { 0. });
        let mut paint = RingPaint {
            thickness: f32::from(self.thickness),
            amplitude: 0.,
            phase: 0.,
            show_track: self.show_track.unwrap_or(target.is_some()),
            ink: color(ColorRole::PRIMARY),
            track: color(ColorRole::SECONDARY_CONTAINER),
        };
        let root = Progress::new(self.id.clone())
            .accessibility_label(self.label)
            .indeterminate(target.is_none())
            .value(target.unwrap_or(0.) * 100.)
            .flex_shrink_0()
            .size(px(box_size));
        let (id, playback) = (self.id, self.playback);
        match target {
            Some(target) => {
                let amount = crate::motion::value(id.clone(), target, window, cx).clamp(0., 1.);
                let frame = CircularProgressFrame::determinate(amount);
                if !wavy {
                    return root.child(ring_canvas(frame, paint, false));
                }
                paint.amplitude = crate::motion::value(
                    (id.clone(), "wave-amplitude"),
                    wave_amplitude(target) / super::linear::WAVE_AMPLITUDE
                        * CIRCULAR_WAVE_AMPLITUDE,
                    window,
                    cx,
                )
                .max(0.);
                root.child(
                    div()
                        .size_full()
                        .with_animation(
                            id,
                            Animation::new(Duration::from_secs(1)).repeat(),
                            move |slot, t| {
                                slot.child(ring_canvas(
                                    frame,
                                    RingPaint { phase: t, ..paint },
                                    true,
                                ))
                            },
                        )
                        .when_some(playback, |a, p| a.with_repeat_playback(p))
                        .repeat_when_visible(),
                )
            }
            None => {
                if wavy {
                    paint.amplitude = CIRCULAR_WAVE_AMPLITUDE;
                }
                if cx.reduce_motion() {
                    return root.child(ring_canvas(circular_indeterminate_frame(0.), paint, wavy));
                }
                root.child(
                    div()
                        .size_full()
                        .with_animation(
                            id,
                            Animation::new(Duration::from_secs_f32(INDETERMINATE_MS / 1000.))
                                .repeat(),
                            move |slot, t| {
                                let elapsed = t * INDETERMINATE_MS;
                                slot.child(ring_canvas(
                                    circular_indeterminate_frame(elapsed),
                                    // Six wavelengths per loop keep the wave seamless.
                                    RingPaint {
                                        phase: t * 6.,
                                        ..paint
                                    },
                                    wavy,
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
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn boxes_follow_the_kit_sizes() {
        let sizes = [
            (ProgressShape::Flat, PROGRESS_THICKNESS, 40.),
            (ProgressShape::Flat, PROGRESS_THICK_THICKNESS, 44.),
            (ProgressShape::Wavy, PROGRESS_THICKNESS, 48.),
            (ProgressShape::Wavy, PROGRESS_THICK_THICKNESS, 52.),
        ];
        for (shape, thickness, size) in sizes {
            assert_eq!(circular_progress_size(shape, thickness), size);
        }
        // The flat ring's stroke touches its box; the wave's peaks do too.
        assert_eq!(ring_radius(40., 4., false), 18.);
        assert_eq!(
            ring_radius(48., 4., true) + CIRCULAR_WAVE_AMPLITUDE + 2.,
            24.
        );
    }
}
