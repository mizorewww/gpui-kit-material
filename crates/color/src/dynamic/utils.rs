// Copyright 2021 Google LLC
// SPDX-License-Identifier: Apache-2.0
// Ported to Rust from material-color-utilities (typescript/utils), modified for
// this crate. See licenses/material-color-utilities.txt.

//! Math and color-space conversions shared by the dynamic color port.

const SRGB_TO_XYZ: [[f64; 3]; 3] = [
    [0.41233895, 0.35762064, 0.18051042],
    [0.2126, 0.7152, 0.0722],
    [0.01932141, 0.11916382, 0.95034478],
];

const XYZ_TO_SRGB: [[f64; 3]; 3] = [
    [
        3.2413774792388685,
        -1.5376652402851851,
        -0.49885366846268053,
    ],
    [-0.9691452513005321, 1.8758853451067872, 0.04156585616912061],
    [
        0.05562093689691305,
        -0.20395524564742123,
        1.0571799111220335,
    ],
];

/// The D65 white point in XYZ: white on a sunny day.
pub const WHITE_POINT_D65: [f64; 3] = [95.047, 100.0, 108.883];

/// -1 below zero, 0 at zero, 1 above zero.
pub fn signum(num: f64) -> f64 {
    if num < 0. {
        -1.
    } else if num == 0. {
        0.
    } else {
        1.
    }
}

/// `start` at `amount` 0 and `stop` at `amount` 1.
pub fn lerp(start: f64, stop: f64, amount: f64) -> f64 {
    (1. - amount) * start + amount * stop
}

/// `input` limited to `min..=max`. Unlike [`f64::clamp`] this accepts
/// `min > max` (returning `min` for anything below it), which the 2025 color
/// spec relies on and which must not panic.
pub fn clamp_double(min: f64, max: f64, input: f64) -> f64 {
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}

/// An integer angle in `0..360`.
pub fn sanitize_degrees_int(degrees: i32) -> i32 {
    degrees.rem_euclid(360)
}

/// An angle in `0.0..360.0`.
pub fn sanitize_degrees_double(degrees: f64) -> f64 {
    let degrees = degrees % 360.;
    if degrees < 0. {
        degrees + 360.
    } else {
        degrees
    }
}

/// The sign of the shortest rotation from one angle to another; 1 when they
/// are exactly opposite.
pub fn rotation_direction(from: f64, to: f64) -> f64 {
    if sanitize_degrees_double(to - from) <= 180. {
        1.
    } else {
        -1.
    }
}

/// The distance of two angles on a circle, in degrees.
pub fn difference_degrees(a: f64, b: f64) -> f64 {
    180. - ((a - b).abs() - 180.).abs()
}

/// A 1x3 row vector times a 3x3 matrix.
pub(crate) fn matrix_multiply(row: [f64; 3], matrix: &[[f64; 3]; 3]) -> [f64; 3] {
    [
        row[0] * matrix[0][0] + row[1] * matrix[0][1] + row[2] * matrix[0][2],
        row[0] * matrix[1][0] + row[1] * matrix[1][1] + row[2] * matrix[1][2],
        row[0] * matrix[2][0] + row[1] * matrix[2][1] + row[2] * matrix[2][2],
    ]
}

/// Opaque ARGB from 8-bit channels.
pub const fn argb_from_rgb(red: u8, green: u8, blue: u8) -> u32 {
    0xff00_0000 | (red as u32) << 16 | (green as u32) << 8 | blue as u32
}

/// Opaque ARGB from linear RGB channels in `0.0..=100.0`.
pub fn argb_from_linrgb(linrgb: [f64; 3]) -> u32 {
    argb_from_rgb(
        delinearized(linrgb[0]),
        delinearized(linrgb[1]),
        delinearized(linrgb[2]),
    )
}

/// The red channel of an ARGB color.
pub const fn red_from_argb(argb: u32) -> u8 {
    (argb >> 16) as u8
}

/// The green channel of an ARGB color.
pub const fn green_from_argb(argb: u32) -> u8 {
    (argb >> 8) as u8
}

/// The blue channel of an ARGB color.
pub const fn blue_from_argb(argb: u32) -> u8 {
    argb as u8
}

/// Opaque ARGB from XYZ.
pub fn argb_from_xyz(x: f64, y: f64, z: f64) -> u32 {
    argb_from_linrgb(matrix_multiply([x, y, z], &XYZ_TO_SRGB))
}

/// XYZ of an ARGB color.
pub fn xyz_from_argb(argb: u32) -> [f64; 3] {
    matrix_multiply(
        [
            linearized(red_from_argb(argb)),
            linearized(green_from_argb(argb)),
            linearized(blue_from_argb(argb)),
        ],
        &SRGB_TO_XYZ,
    )
}

/// L*a*b* of an ARGB color.
pub fn lab_from_argb(argb: u32) -> [f64; 3] {
    let [x, y, z] = xyz_from_argb(argb);
    let fx = lab_f(x / WHITE_POINT_D65[0]);
    let fy = lab_f(y / WHITE_POINT_D65[1]);
    let fz = lab_f(z / WHITE_POINT_D65[2]);
    [116. * fy - 16., 500. * (fx - fy), 200. * (fy - fz)]
}

/// The gray with lightness `lstar`.
pub fn argb_from_lstar(lstar: f64) -> u32 {
    let component = delinearized(y_from_lstar(lstar));
    argb_from_rgb(component, component, component)
}

/// L* of an ARGB color.
pub fn lstar_from_argb(argb: u32) -> f64 {
    116. * lab_f(xyz_from_argb(argb)[1] / 100.) - 16.
}

/// Y in XYZ (relative luminance) of an L* value.
pub fn y_from_lstar(lstar: f64) -> f64 {
    100. * lab_invf((lstar + 16.) / 116.)
}

/// L* of a Y value.
pub fn lstar_from_y(y: f64) -> f64 {
    lab_f(y / 100.) * 116. - 16.
}

/// An 8-bit sRGB channel as linear RGB in `0.0..=100.0`.
pub fn linearized(component: u8) -> f64 {
    let normalized = f64::from(component) / 255.;
    if normalized <= 0.040449936 {
        normalized / 12.92 * 100.
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4) * 100.
    }
}

/// A linear RGB channel in `0.0..=100.0` as 8-bit sRGB, clamped.
pub fn delinearized(component: f64) -> u8 {
    let normalized = component / 100.;
    let delinearized = if normalized <= 0.0031308 {
        normalized * 12.92
    } else {
        1.055 * normalized.powf(1. / 2.4) - 0.055
    };
    // A saturating cast: out-of-range values clamp and NaN becomes 0.
    (delinearized * 255.).round() as u8
}

fn lab_f(t: f64) -> f64 {
    const E: f64 = 216. / 24389.;
    const KAPPA: f64 = 24389. / 27.;
    if t > E {
        t.powf(1. / 3.)
    } else {
        (KAPPA * t + 16.) / 116.
    }
}

fn lab_invf(ft: f64) -> f64 {
    const E: f64 = 216. / 24389.;
    const KAPPA: f64 = 24389. / 27.;
    let ft3 = ft * ft * ft;
    if ft3 > E {
        ft3
    } else {
        (116. * ft - 16.) / KAPPA
    }
}
