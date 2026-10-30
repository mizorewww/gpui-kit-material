// Copyright 2021 Google LLC
// SPDX-License-Identifier: Apache-2.0
// Ported to Rust from material-color-utilities (typescript/hct), modified for
// this crate. See licenses/material-color-utilities.txt.

//! CAM16, its viewing conditions and the HCT color space.
use super::solver;
use super::utils::{
    WHITE_POINT_D65, argb_from_xyz, blue_from_argb, clamp_double, green_from_argb, lerp,
    linearized, lstar_from_argb, lstar_from_y, red_from_argb, sanitize_degrees_double, signum,
    y_from_lstar,
};
use std::f64::consts::PI;
use std::sync::LazyLock;

/// The environment a color is seen in, with the CAM16 intermediates that
/// depend only on it. Field names are CAM16's own terms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewingConditions {
    /// Background induction factor.
    pub n: f64,
    /// Achromatic response of the white point.
    pub aw: f64,
    /// Brightness induction factor.
    pub nbb: f64,
    /// Chromatic induction factor.
    pub ncb: f64,
    /// Impact of surround.
    pub c: f64,
    /// Chromatic induction from surround.
    pub nc: f64,
    /// Per-channel degree of adaptation.
    pub rgb_d: [f64; 3],
    /// Luminance-level adaptation factor.
    pub fl: f64,
    /// Fourth root of [`Self::fl`].
    pub fl_root: f64,
    /// Base exponential nonlinearity.
    pub z: f64,
}

static DEFAULT_CONDITIONS: LazyLock<ViewingConditions> = LazyLock::new(|| {
    ViewingConditions::make(
        WHITE_POINT_D65,
        (200. / PI) * y_from_lstar(50.) / 100.,
        50.,
        2.,
        false,
    )
});

impl ViewingConditions {
    /// sRGB-like viewing conditions: the ones HCT is defined in.
    pub fn standard() -> &'static Self {
        &DEFAULT_CONDITIONS
    }

    /// Conditions from physical parameters: the white point in XYZ, the
    /// adapting luminance, the L* of the surrounding area, the surround (0
    /// dark, 1 dim, 2 average) and whether the eye discounts the illuminant.
    pub fn make(
        white_point: [f64; 3],
        adapting_luminance: f64,
        background_lstar: f64,
        surround: f64,
        discounting_illuminant: bool,
    ) -> Self {
        let [x, y, z] = white_point;
        let r_w = x * 0.401288 + y * 0.650173 + z * -0.051461;
        let g_w = x * -0.250268 + y * 1.204414 + z * 0.045854;
        let b_w = x * -0.002079 + y * 0.048952 + z * 0.953127;
        let f = 0.8 + surround / 10.;
        let c = if f >= 0.9 {
            lerp(0.59, 0.69, (f - 0.9) * 10.)
        } else {
            lerp(0.525, 0.59, (f - 0.8) * 10.)
        };
        let d = if discounting_illuminant {
            1.
        } else {
            f * (1. - (1. / 3.6) * ((-adapting_luminance - 42.) / 92.).exp())
        };
        let d = clamp_double(0., 1., d);
        let rgb_d = [
            d * (100. / r_w) + 1. - d,
            d * (100. / g_w) + 1. - d,
            d * (100. / b_w) + 1. - d,
        ];
        let k = 1. / (5. * adapting_luminance + 1.);
        let k4 = k * k * k * k;
        let k4f = 1. - k4;
        let fl = k4 * adapting_luminance + 0.1 * k4f * k4f * (5. * adapting_luminance).cbrt();
        let n = y_from_lstar(background_lstar) / y;
        let z_base = 1.48 + n.sqrt();
        let nbb = 0.725 / n.powf(0.2);
        let factors = [
            ((fl * rgb_d[0] * r_w) / 100.).powf(0.42),
            ((fl * rgb_d[1] * g_w) / 100.).powf(0.42),
            ((fl * rgb_d[2] * b_w) / 100.).powf(0.42),
        ];
        let rgb_a = factors.map(|factor| (400. * factor) / (factor + 27.13));
        let aw = (2. * rgb_a[0] + rgb_a[1] + 0.05 * rgb_a[2]) * nbb;
        Self {
            n,
            aw,
            nbb,
            ncb: nbb,
            c,
            nc: f,
            rgb_d,
            fl,
            fl_root: fl.powf(0.25),
            z: z_base,
        }
    }
}

/// A color in CAM16, the appearance model behind HCT's hue and chroma.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cam16 {
    /// Hue in degrees.
    pub hue: f64,
    /// Chroma: colorfulness relative to white.
    pub chroma: f64,
    /// Lightness.
    pub j: f64,
    /// Brightness.
    pub q: f64,
    /// Colorfulness.
    pub m: f64,
    /// Saturation.
    pub s: f64,
    /// CAM16-UCS J coordinate.
    pub jstar: f64,
    /// CAM16-UCS a coordinate.
    pub astar: f64,
    /// CAM16-UCS b coordinate.
    pub bstar: f64,
}

impl Cam16 {
    /// Perceptual distance in CAM16-UCS.
    pub fn distance(&self, other: &Self) -> f64 {
        let d_j = self.jstar - other.jstar;
        let d_a = self.astar - other.astar;
        let d_b = self.bstar - other.bstar;
        1.41 * (d_j * d_j + d_a * d_a + d_b * d_b).sqrt().powf(0.63)
    }

    /// An ARGB color in the standard viewing conditions.
    pub fn from_argb(argb: u32) -> Self {
        let red = linearized(red_from_argb(argb));
        let green = linearized(green_from_argb(argb));
        let blue = linearized(blue_from_argb(argb));
        let x = 0.41233895 * red + 0.35762064 * green + 0.18051042 * blue;
        let y = 0.2126 * red + 0.7152 * green + 0.0722 * blue;
        let z = 0.01932141 * red + 0.11916382 * green + 0.95034478 * blue;
        Self::from_xyz_in(x, y, z, ViewingConditions::standard())
    }

    /// An XYZ color seen in `vc`.
    pub fn from_xyz_in(x: f64, y: f64, z: f64, vc: &ViewingConditions) -> Self {
        let r_c = 0.401288 * x + 0.650173 * y - 0.051461 * z;
        let g_c = -0.250268 * x + 1.204414 * y + 0.045854 * z;
        let b_c = -0.002079 * x + 0.048952 * y + 0.953127 * z;
        let r_d = vc.rgb_d[0] * r_c;
        let g_d = vc.rgb_d[1] * g_c;
        let b_d = vc.rgb_d[2] * b_c;
        let adapt = |d: f64| {
            let af = (vc.fl * d.abs() / 100.).powf(0.42);
            signum(d) * 400. * af / (af + 27.13)
        };
        let (r_a, g_a, b_a) = (adapt(r_d), adapt(g_d), adapt(b_d));
        let a = (11. * r_a + -12. * g_a + b_a) / 11.;
        let b = (r_a + g_a - 2. * b_a) / 9.;
        let u = (20. * r_a + 20. * g_a + 21. * b_a) / 20.;
        let p2 = (40. * r_a + 20. * g_a + b_a) / 20.;
        let hue = sanitize_degrees_double(b.atan2(a) * 180. / PI);
        let hue_radians = hue * PI / 180.;
        let ac = p2 * vc.nbb;
        let j = 100. * (ac / vc.aw).powf(vc.c * vc.z);
        let q = (4. / vc.c) * (j / 100.).sqrt() * (vc.aw + 4.) * vc.fl_root;
        let hue_prime = if hue < 20.14 { hue + 360. } else { hue };
        let e_hue = 0.25 * ((hue_prime * PI / 180. + 2.).cos() + 3.8);
        let p1 = (50000. / 13.) * e_hue * vc.nc * vc.ncb;
        let t = (p1 * (a * a + b * b).sqrt()) / (u + 0.305);
        let alpha = t.powf(0.9) * (1.64 - 0.29_f64.powf(vc.n)).powf(0.73);
        let c = alpha * (j / 100.).sqrt();
        let m = c * vc.fl_root;
        let s = 50. * ((alpha * vc.c) / (vc.aw + 4.)).sqrt();
        let jstar = ((1. + 100. * 0.007) * j) / (1. + 0.007 * j);
        let mstar = (1. / 0.0228) * (1. + 0.0228 * m).ln();
        Self {
            hue,
            chroma: c,
            j,
            q,
            m,
            s,
            jstar,
            astar: mstar * hue_radians.cos(),
            bstar: mstar * hue_radians.sin(),
        }
    }

    /// From lightness, chroma and hue in the standard viewing conditions.
    pub fn from_jch(j: f64, c: f64, h: f64) -> Self {
        Self::from_jch_in(j, c, h, ViewingConditions::standard())
    }

    /// From lightness, chroma and hue in `vc`.
    pub fn from_jch_in(j: f64, c: f64, h: f64, vc: &ViewingConditions) -> Self {
        let q = (4. / vc.c) * (j / 100.).sqrt() * (vc.aw + 4.) * vc.fl_root;
        let m = c * vc.fl_root;
        let alpha = c / (j / 100.).sqrt();
        let s = 50. * ((alpha * vc.c) / (vc.aw + 4.)).sqrt();
        let hue_radians = h * PI / 180.;
        let jstar = ((1. + 100. * 0.007) * j) / (1. + 0.007 * j);
        let mstar = (1. / 0.0228) * (1. + 0.0228 * m).ln();
        Self {
            hue: h,
            chroma: c,
            j,
            q,
            m,
            s,
            jstar,
            astar: mstar * hue_radians.cos(),
            bstar: mstar * hue_radians.sin(),
        }
    }

    /// From CAM16-UCS coordinates in the standard viewing conditions.
    pub fn from_ucs(jstar: f64, astar: f64, bstar: f64) -> Self {
        let vc = ViewingConditions::standard();
        let m = (astar * astar + bstar * bstar).sqrt();
        let big_m = ((m * 0.0228).exp() - 1.) / 0.0228;
        let c = big_m / vc.fl_root;
        let mut h = bstar.atan2(astar) * (180. / PI);
        if h < 0. {
            h += 360.;
        }
        let j = jstar / (1. - (jstar - 100.) * 0.007);
        Self::from_jch_in(j, c, h, vc)
    }

    /// The ARGB color in the standard viewing conditions.
    pub fn to_argb(&self) -> u32 {
        let [x, y, z] = self.xyz_in(ViewingConditions::standard());
        argb_from_xyz(x, y, z)
    }

    /// The XYZ coordinates of this color seen in `vc`.
    pub fn xyz_in(&self, vc: &ViewingConditions) -> [f64; 3] {
        let alpha = if self.chroma == 0. || self.j == 0. {
            0.
        } else {
            self.chroma / (self.j / 100.).sqrt()
        };
        let t = (alpha / (1.64 - 0.29_f64.powf(vc.n)).powf(0.73)).powf(1. / 0.9);
        let h_rad = self.hue * PI / 180.;
        let e_hue = 0.25 * ((h_rad + 2.).cos() + 3.8);
        let ac = vc.aw * (self.j / 100.).powf(1. / vc.c / vc.z);
        let p1 = e_hue * (50000. / 13.) * vc.nc * vc.ncb;
        let p2 = ac / vc.nbb;
        let (h_sin, h_cos) = h_rad.sin_cos();
        let gamma = (23. * (p2 + 0.305) * t) / (23. * p1 + 11. * t * h_cos + 108. * t * h_sin);
        let a = gamma * h_cos;
        let b = gamma * h_sin;
        let r_a = (460. * p2 + 451. * a + 288. * b) / 1403.;
        let g_a = (460. * p2 - 891. * a - 261. * b) / 1403.;
        let b_a = (460. * p2 - 220. * a - 6300. * b) / 1403.;
        let unadapt = |adapted: f64| {
            let base = ((27.13 * adapted.abs()) / (400. - adapted.abs())).max(0.);
            signum(adapted) * (100. / vc.fl) * base.powf(1. / 0.42)
        };
        let r_f = unadapt(r_a) / vc.rgb_d[0];
        let g_f = unadapt(g_a) / vc.rgb_d[1];
        let b_f = unadapt(b_a) / vc.rgb_d[2];
        [
            1.86206786 * r_f - 1.01125463 * g_f + 0.14918677 * b_f,
            0.38752654 * r_f + 0.62144744 * g_f - 0.00897398 * b_f,
            -0.01584150 * r_f - 0.03412294 * g_f + 1.04996444 * b_f,
        ]
    }
}

/// Hue, chroma and tone: CAM16 hue and chroma with L* as tone, so a tone
/// difference predicts contrast (40 gives at least 3:1, 50 at least 4.5:1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hct {
    hue: f64,
    chroma: f64,
    tone: f64,
    argb: u32,
}

impl Hct {
    /// The sRGB color closest to the request. Hue and tone are met when
    /// possible and chroma is reduced to what they allow. Non-finite inputs
    /// count as 0.
    pub fn from_hct(hue: f64, chroma: f64, tone: f64) -> Self {
        let finite = |value: f64| if value.is_finite() { value } else { 0. };
        Self::from_argb(solver::solve_to_argb(
            finite(hue),
            finite(chroma),
            finite(tone),
        ))
    }

    /// An ARGB color. The alpha channel is ignored for measurement and kept
    /// in [`Self::argb`].
    pub fn from_argb(argb: u32) -> Self {
        let cam = Cam16::from_argb(argb);
        Self {
            hue: cam.hue,
            chroma: cam.chroma,
            tone: lstar_from_argb(argb),
            argb,
        }
    }

    /// Hue in `0.0..360.0`.
    pub fn hue(&self) -> f64 {
        self.hue
    }

    /// Chroma, whose maximum depends on hue and tone.
    pub fn chroma(&self) -> f64 {
        self.chroma
    }

    /// Tone (L*) in `0.0..=100.0`.
    pub fn tone(&self) -> f64 {
        self.tone
    }

    /// The color as ARGB.
    pub fn argb(&self) -> u32 {
        self.argb
    }

    /// Whether `hue` is in upstream's blue range, `250..270`.
    pub fn is_blue(hue: f64) -> bool {
        (250. ..270.).contains(&hue)
    }

    /// Whether `hue` is in upstream's yellow range, `105..125`.
    pub fn is_yellow(hue: f64) -> bool {
        (105. ..125.).contains(&hue)
    }

    /// Whether `hue` is in upstream's cyan range, `170..207`.
    pub fn is_cyan(hue: f64) -> bool {
        (170. ..207.).contains(&hue)
    }

    /// This color as it would appear in `vc`, expressed in the standard
    /// viewing conditions.
    pub fn in_viewing_conditions(&self, vc: &ViewingConditions) -> Self {
        let [x, y, z] = Cam16::from_argb(self.argb).xyz_in(vc);
        let recast = Cam16::from_xyz_in(x, y, z, ViewingConditions::standard());
        Self::from_hct(recast.hue, recast.chroma, lstar_from_y(y))
    }
}
