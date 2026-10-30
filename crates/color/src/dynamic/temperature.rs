// Copyright 2023 Google LLC
// SPDX-License-Identifier: Apache-2.0
// Ported to Rust from material-color-utilities (typescript/temperature), modified
// for this crate: temperatures are stored beside each color instead of in a map
// keyed by object identity. See licenses/material-color-utilities.txt.

//! Color temperature: analogous and complementary colors.
use super::hct::Hct;
use super::utils::{lab_from_argb, sanitize_degrees_double, sanitize_degrees_int};
use std::f64::consts::PI;

/// The input color with its hue circle, sorted by temperature, computed once.
#[derive(Clone, Debug)]
pub struct TemperatureCache {
    input: Hct,
    /// One color per degree, 0 to 360 inclusive, at the input's chroma and tone.
    by_hue: Vec<Hct>,
    coldest: (Hct, f64),
    warmest: (Hct, f64),
}

impl TemperatureCache {
    /// Build the hue circle of `input`: 361 solves.
    pub fn new(input: Hct) -> Self {
        let by_hue: Vec<Hct> = (0..=360)
            .map(|hue| Hct::from_hct(f64::from(hue), input.chroma(), input.tone()))
            .collect();
        // Upstream sorts stably by temperature with the input appended last:
        // on ties the coldest is the first in that order and the warmest the
        // last.
        let mut coldest = (input, f64::INFINITY);
        let mut warmest = (input, f64::NEG_INFINITY);
        for hct in by_hue.iter().copied().chain([input]) {
            let temperature = raw_temperature(hct);
            if temperature < coldest.1 {
                coldest = (hct, temperature);
            }
            if temperature >= warmest.1 {
                warmest = (hct, temperature);
            }
        }
        Self {
            input,
            by_hue,
            coldest,
            warmest,
        }
    }

    /// The warmest color at the input's chroma and tone.
    pub fn warmest(&self) -> Hct {
        self.warmest.0
    }

    /// The coldest color at the input's chroma and tone.
    pub fn coldest(&self) -> Hct {
        self.coldest.0
    }

    /// `count` colors of differing hue, equidistant in temperature over a
    /// wheel of `divisions` steps, with the input in the middle. Art usually
    /// describes this as 5 colors on a wheel of 12. `count` and `divisions`
    /// of 0 return only the input.
    pub fn analogous(&self, count: usize, divisions: usize) -> Vec<Hct> {
        if count == 0 || divisions == 0 {
            return vec![self.input];
        }
        // JavaScript's Math.round on a non-negative hue; at most 360.
        let start_hue = self.input.hue().round() as i32;
        let start = self.by_hue[start_hue as usize];
        let mut last_temp = self.relative_temperature(start);
        let mut all = vec![start];

        let mut total_delta = 0.;
        for i in 0..360 {
            let hct = self.by_hue[sanitize_degrees_int(start_hue + i) as usize];
            let temp = self.relative_temperature(hct);
            total_delta += (temp - last_temp).abs();
            last_temp = temp;
        }
        let mut hue_addend = 1;
        let step = total_delta / divisions as f64;
        let mut delta_so_far = 0.;
        last_temp = self.relative_temperature(start);
        while all.len() < divisions {
            let hct = self.by_hue[sanitize_degrees_int(start_hue + hue_addend) as usize];
            let temp = self.relative_temperature(hct);
            delta_so_far += (temp - last_temp).abs();
            let mut satisfied = delta_so_far >= all.len() as f64 * step;
            let mut index_addend = 1;
            // Keep adding this hue until its temperature is insufficient, so
            // colors without enough discrete steps (white, black) still fill
            // the answer.
            while satisfied && all.len() < divisions {
                all.push(hct);
                satisfied = delta_so_far >= (all.len() + index_addend) as f64 * step;
                index_addend += 1;
            }
            last_temp = temp;
            hue_addend += 1;
            if hue_addend > 360 {
                while all.len() < divisions {
                    all.push(hct);
                }
                break;
            }
        }

        let mut answers = vec![self.input];
        let increase = (count - 1) / 2;
        for i in 1..=increase {
            answers.insert(0, all[(all.len() - i % all.len()) % all.len()]);
        }
        for i in 1..=count - increase - 1 {
            answers.push(all[i % all.len()]);
        }
        answers
    }

    /// The color as far from the input in temperature as the input is from
    /// the other end: art's color across the wheel.
    pub fn complement(&self) -> Hct {
        let (coldest_hue, coldest_temp) = (self.coldest.0.hue(), self.coldest.1);
        let (warmest_hue, warmest_temp) = (self.warmest.0.hue(), self.warmest.1);
        let range = warmest_temp - coldest_temp;
        let cold_to_warm = is_between(self.input.hue(), coldest_hue, warmest_hue);
        let (start_hue, end_hue) = if cold_to_warm {
            (warmest_hue, coldest_hue)
        } else {
            (coldest_hue, warmest_hue)
        };
        let mut smallest_error = 1000.;
        let mut answer = self.by_hue[self.input.hue().round() as usize];
        let complement_relative = 1. - self.input_relative_temperature();
        // The color in the other section closest to the inverse percentile.
        for addend in 0..=360 {
            let hue = sanitize_degrees_double(start_hue + f64::from(addend));
            if !is_between(hue, start_hue, end_hue) {
                continue;
            }
            let possible = self.by_hue[hue.round() as usize];
            let relative = (raw_temperature(possible) - coldest_temp) / range;
            let error = (complement_relative - relative).abs();
            if error < smallest_error {
                smallest_error = error;
                answer = possible;
            }
        }
        answer
    }

    /// Temperature relative to all colors of the same chroma and tone, 0 to 1.
    /// 0.5 when they do not differ, as at T100 where only white exists.
    pub fn relative_temperature(&self, hct: Hct) -> f64 {
        let range = self.warmest.1 - self.coldest.1;
        if range == 0. {
            return 0.5;
        }
        (raw_temperature(hct) - self.coldest.1) / range
    }

    /// [`Self::relative_temperature`] of the input.
    pub fn input_relative_temperature(&self) -> f64 {
        self.relative_temperature(self.input)
    }
}

/// Whether `angle` lies between two others, rotating clockwise.
fn is_between(angle: f64, a: f64, b: f64) -> bool {
    if a < b {
        a <= angle && angle <= b
    } else {
        a <= angle || angle <= b
    }
}

/// The cool-warm factor of a color (Ou, Woodcock and Wright, in L*a*b*):
/// below 0 is cool, above 0 warm.
pub fn raw_temperature(color: Hct) -> f64 {
    let [_, a, b] = lab_from_argb(color.argb());
    let hue = sanitize_degrees_double(b.atan2(a) * 180. / PI);
    let chroma = (a * a + b * b).sqrt();
    -0.5 + 0.02 * chroma.powf(1.07) * (sanitize_degrees_double(hue - 50.) * PI / 180.).cos()
}
