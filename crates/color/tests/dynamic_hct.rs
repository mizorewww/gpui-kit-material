//! Vectors from material-color-utilities: typescript/hct/hct_test.ts,
//! hct_round_trip_test.ts and utils/utils_test.ts, at the tolerances upstream
//! uses (Jasmine `toBeCloseTo(x, p)` means `|a - x| < 10^-p / 2`).
use gpui_kit_material_color::dynamic::utils::{
    argb_from_rgb, blue_from_argb, green_from_argb, lstar_from_y, red_from_argb,
    rotation_direction, y_from_lstar,
};
use gpui_kit_material_color::dynamic::{Cam16, Hct, ViewingConditions};

const RED: u32 = 0xffff0000;
const GREEN: u32 = 0xff00ff00;
const BLUE: u32 = 0xff0000ff;
const WHITE: u32 = 0xffffffff;
const BLACK: u32 = 0xff000000;

#[track_caller]
fn close(actual: f64, expected: f64, precision: i32) {
    let limit = 10f64.powi(-precision) / 2.;
    assert!(
        (actual - expected).abs() < limit,
        "{actual} is not within {limit} of {expected}"
    );
}

#[test]
fn cam_of_primaries() {
    for (argb, [hue, chroma, j, m, s, q]) in [
        (RED, [27.408, 113.358, 46.445, 89.494, 91.890, 105.989]),
        (GREEN, [142.140, 108.410, 79.332, 85.588, 78.605, 138.520]),
        (BLUE, [282.788, 87.231, 25.466, 68.867, 93.675, 78.481]),
        (WHITE, [209.492, 2.869, 100.0, 2.265, 12.068, 155.521]),
        (BLACK, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
    ] {
        let cam = Cam16::from_argb(argb);
        close(cam.hue, hue, 3);
        close(cam.chroma, chroma, 3);
        close(cam.j, j, 3);
        close(cam.m, m, 3);
        close(cam.s, s, 3);
        close(cam.q, q, 3);
    }
}

#[test]
fn cam_round_trips_primaries() {
    for argb in [RED, GREEN, BLUE] {
        assert_eq!(Cam16::from_argb(argb).to_argb(), argb);
    }
}

#[test]
fn argb_to_hct() {
    let green = Hct::from_argb(GREEN);
    close(green.hue(), 142.139, 2);
    close(green.chroma(), 108.410, 2);
    close(green.tone(), 87.737, 2);
    let blue = Hct::from_argb(BLUE);
    close(blue.hue(), 282.788, 2);
    close(blue.chroma(), 87.230, 2);
    close(blue.tone(), 32.302, 2);
    let tone_90 = Hct::from_hct(282.788, 87.230, 90.);
    close(tone_90.hue(), 282.239, 2);
    close(tone_90.chroma(), 19.144, 2);
    close(tone_90.tone(), 90.035, 2);
}

#[test]
fn default_viewing_conditions() {
    let vc = ViewingConditions::standard();
    close(vc.n, 0.184, 3);
    close(vc.aw, 29.981, 3);
    close(vc.nbb, 1.017, 3);
    close(vc.ncb, 1.017, 3);
    close(vc.c, 0.69, 3);
    close(vc.nc, 1.0, 3);
    close(vc.rgb_d[0], 1.021, 3);
    close(vc.rgb_d[1], 0.986, 3);
    close(vc.rgb_d[2], 0.934, 3);
    close(vc.fl, 0.388, 3);
    close(vc.fl_root, 0.789, 3);
    close(vc.z, 1.909, 3);
}

fn on_boundary(argb: u32) -> bool {
    [
        red_from_argb(argb),
        green_from_argb(argb),
        blue_from_argb(argb),
    ]
    .iter()
    .any(|channel| *channel == 0 || *channel == 255)
}

#[test]
fn solver_returns_a_sufficiently_close_color() {
    for hue in (15..360).step_by(30) {
        for chroma in (0..=100).step_by(10) {
            for tone in (20..=80).step_by(10) {
                let (h, c, t) = (f64::from(hue), f64::from(chroma), f64::from(tone));
                let color = Hct::from_hct(h, c, t);
                if chroma > 0 {
                    assert!((color.hue() - h).abs() <= 4., "{h} {c} {t}");
                }
                assert!(color.chroma() >= 0. && color.chroma() <= c + 2.5);
                if color.chroma() < c - 2.5 {
                    assert!(on_boundary(color.argb()), "{h} {c} {t}");
                }
                assert!((color.tone() - t).abs() <= 0.5, "{h} {c} {t}");
            }
        }
    }
}

#[test]
fn hct_round_trip_preserves_the_color() {
    for r in (0..296u32).step_by(37) {
        for g in (0..296u32).step_by(37) {
            for b in (0..296u32).step_by(37) {
                let argb = argb_from_rgb(r.min(255) as u8, g.min(255) as u8, b.min(255) as u8);
                let hct = Hct::from_argb(argb);
                assert_eq!(
                    Hct::from_hct(hct.hue(), hct.chroma(), hct.tone()).argb(),
                    argb
                );
            }
        }
    }
}

#[test]
fn argb_from_rgb_values() {
    assert_eq!(argb_from_rgb(255, 255, 255), 0xffffffff);
    assert_eq!(argb_from_rgb(0, 0, 0), 0xff000000);
    assert_eq!(argb_from_rgb(50, 150, 250), 0xff3296fa);
}

#[test]
fn rotation_direction_matches_the_original_implementation() {
    fn original(from: f64, to: f64) -> f64 {
        let (a, b, c) = (to - from, to - from + 360., to - from - 360.);
        let (a_abs, b_abs, c_abs) = (a.abs(), b.abs(), c.abs());
        let picked = if a_abs <= b_abs && a_abs <= c_abs {
            a
        } else if b_abs <= a_abs && b_abs <= c_abs {
            b
        } else {
            c
        };
        if picked >= 0. { 1. } else { -1. }
    }
    for from in 0..24 {
        for to in 0..24 {
            let (from, to) = (f64::from(from) * 15., 7.5 + f64::from(to) * 15.);
            assert_eq!(rotation_direction(from, to), original(from, to));
        }
    }
}

#[test]
fn y_from_lstar_values() {
    for (lstar, y) in [
        (0.0, 0.0),
        (0.1, 0.0110705),
        (0.2, 0.0221411),
        (0.3, 0.0332116),
        (0.4, 0.0442822),
        (0.5, 0.0553528),
        (1.0, 0.1107056),
        (2.0, 0.2214112),
        (3.0, 0.3321169),
        (4.0, 0.4428225),
        (5.0, 0.5535282),
        (8.0, 0.8856451),
        (10.0, 1.1260199),
        (15.0, 1.9085832),
        (20.0, 2.9890524),
        (25.0, 4.4154767),
        (30.0, 6.2359055),
        (40.0, 11.2509737),
        (50.0, 18.4186518),
        (60.0, 28.1233342),
        (70.0, 40.7494157),
        (80.0, 56.6812907),
        (90.0, 76.3033539),
        (95.0, 87.6183294),
        (99.0, 97.4360239),
        (100.0, 100.0),
    ] {
        close(y_from_lstar(lstar), y, 5);
    }
}

#[test]
fn lstar_from_y_values() {
    for (y, lstar) in [
        (0.0, 0.0),
        (0.1, 0.9032962),
        (0.2, 1.8065925),
        (0.3, 2.7098888),
        (0.4, 3.6131851),
        (0.5, 4.5164814),
        (0.8856451, 8.0),
        (1.0, 8.9914424),
        (2.0, 15.4872443),
        (3.0, 20.0438970),
        (4.0, 23.6714419),
        (5.0, 26.7347653),
        (10.0, 37.8424304),
        (15.0, 45.6341970),
        (20.0, 51.8372115),
        (25.0, 57.0754208),
        (30.0, 61.6542222),
        (40.0, 69.4695307),
        (50.0, 76.0692610),
        (60.0, 81.8381891),
        (70.0, 86.9968642),
        (80.0, 91.6848609),
        (90.0, 95.9967686),
        (95.0, 98.0335184),
        (99.0, 99.6120372),
        (100.0, 100.0),
    ] {
        close(lstar_from_y(y), lstar, 5);
    }
}

#[test]
fn lstar_and_y_are_inverses() {
    for step in 0..=1000 {
        let value = f64::from(step) * 0.1;
        close(y_from_lstar(lstar_from_y(value)), value, 8);
        close(lstar_from_y(y_from_lstar(value)), value, 8);
    }
}

#[test]
fn extreme_inputs_do_not_panic() {
    for value in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        -1e9,
        1e9,
        0.,
        -0.,
    ] {
        for (h, c, t) in [
            (value, 40., 50.),
            (120., value, 50.),
            (120., 40., value),
            (value, value, value),
        ] {
            let _ = Hct::from_hct(h, c, t);
        }
    }
}
