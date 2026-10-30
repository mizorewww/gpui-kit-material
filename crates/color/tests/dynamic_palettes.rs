//! Vectors from material-color-utilities: typescript/contrast/contrast_test.ts,
//! blend/blend_test.ts, palettes/palettes_test.ts, dislike and temperature.
use gpui_kit_material_color::dynamic::{Hct, TemperatureCache, TonalPalette, blend, contrast};
use gpui_kit_material_color::dynamic::{fix_if_disliked, is_disliked, raw_temperature};

const RED: u32 = 0xffff0000;
const BLUE: u32 = 0xff0000ff;
const GREEN: u32 = 0xff00ff00;
const YELLOW: u32 = 0xffffff00;
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
fn contrast_vectors() {
    close(contrast::ratio_of_tones(-10., 110.), 21., 3);
    assert_eq!(contrast::lighter(90., 10.), None);
    assert_eq!(contrast::lighter(110., 2.), None);
    assert_eq!(contrast::lighter(-10., 2.), None);
    close(contrast::lighter_unsafe(100., 2.), 100., 3);
    assert_eq!(contrast::darker(10., 20.), None);
    assert_eq!(contrast::darker(110., 2.), None);
    assert_eq!(contrast::darker(-10., 2.), None);
    close(contrast::darker_unsafe(0., 2.), 0., 3);
}

#[test]
fn harmonize_vectors() {
    for (design, source, expected) in [
        (RED, BLUE, 0xfffb0057),
        (RED, GREEN, 0xffd85600),
        (RED, YELLOW, 0xffd85600),
        (BLUE, GREEN, 0xff0047a3),
        (BLUE, RED, 0xff5700dc),
        (BLUE, YELLOW, 0xff0047a3),
        (GREEN, BLUE, 0xff00fc94),
        (GREEN, RED, 0xffb1f000),
        (GREEN, YELLOW, 0xffb1f000),
        (YELLOW, BLUE, 0xffebffba),
        (YELLOW, GREEN, 0xffebffba),
        (YELLOW, RED, 0xfffff6e3),
    ] {
        assert_eq!(
            blend::harmonize(design, source),
            expected,
            "{design:x} toward {source:x}"
        );
    }
}

const TONES: [f64; 12] = [100., 95., 90., 80., 70., 60., 50., 40., 30., 20., 10., 0.];

#[track_caller]
fn assert_tones(palette: TonalPalette, expected: [u32; 12]) {
    for (tone, argb) in TONES.into_iter().zip(expected) {
        assert_eq!(palette.tone(tone), argb, "tone {tone}");
    }
}

const BLUE_A1: [u32; 12] = [
    0xffffffff, 0xfff1efff, 0xffe0e0ff, 0xffbec2ff, 0xff9da3ff, 0xff7c84ff, 0xff5a64ff, 0xff343dff,
    0xff0000ef, 0xff0001ac, 0xff00006e, 0xff000000,
];

#[test]
fn tonal_palette_of_blue() {
    assert_tones(TonalPalette::from_argb(BLUE), BLUE_A1);
}

// Upstream's CorePalette is deprecated in favor of DynamicScheme and is not
// ported; its vectors still pin `from_hue_and_chroma` at the chromas it uses.
#[test]
fn core_palette_vectors() {
    let hct = Hct::from_argb(BLUE);
    assert_tones(
        TonalPalette::from_hue_and_chroma(hct.hue(), hct.chroma().max(48.)),
        BLUE_A1,
    );
    assert_tones(
        TonalPalette::from_hue_and_chroma(hct.hue(), 16.),
        [
            0xffffffff, 0xfff1efff, 0xffe1e0f9, 0xffc5c4dd, 0xffa9a9c1, 0xff8f8fa6, 0xff75758b,
            0xff5c5d72, 0xff444559, 0xff2e2f42, 0xff191a2c, 0xff000000,
        ],
    );
    assert_tones(
        TonalPalette::from_hue_and_chroma(hct.hue(), hct.chroma()),
        BLUE_A1,
    );
    assert_tones(
        TonalPalette::from_hue_and_chroma(hct.hue(), hct.chroma() / 3.),
        [
            0xffffffff, 0xfff1efff, 0xffe0e0ff, 0xffc1c3f4, 0xffa5a7d7, 0xff8b8dbb, 0xff7173a0,
            0xff585b86, 0xff40436d, 0xff2a2d55, 0xff14173f, 0xff000000,
        ],
    );
}

// typescript/scheme/scheme_test.ts: SchemeAndroid reads these core palette tones.
#[test]
fn android_scheme_vectors() {
    let hct = Hct::from_argb(0xff6750a4);
    let palette = |hue: f64, chroma: f64| TonalPalette::from_hue_and_chroma(hue, chroma);
    assert_eq!(
        palette(hct.hue(), hct.chroma().max(48.)).tone(90.),
        0xffe9ddff
    );
    assert_eq!(palette(hct.hue(), 16.).tone(90.), 0xffe8def8);
    assert_eq!(palette(hct.hue() + 60., 24.).tone(90.), 0xffffd9e3);
    assert_eq!(palette(hct.hue(), 4.).tone(98.), 0xfffdf8fd);
    assert_eq!(palette(hct.hue(), 4.).tone(20.), 0xff313033);
    assert_eq!(palette(hct.hue(), 4.).tone(10.), 0xff1c1b1e);
    assert_eq!(palette(hct.hue(), 4.).tone(95.), 0xfff4eff4);
    let blue = Hct::from_argb(BLUE);
    assert_eq!(
        palette(blue.hue(), blue.chroma().max(48.)).tone(90.),
        0xffe0e0ff
    );
}

#[test]
fn key_color_vectors() {
    let exact = TonalPalette::from_hue_and_chroma(50., 60.).key_color();
    assert!((exact.hue() - 50.).abs() < 10.);
    assert!((exact.chroma() - 60.).abs() < 0.5);
    assert!(exact.tone() > 0. && exact.tone() < 100.);

    let high = TonalPalette::from_hue_and_chroma(149., 200.).key_color();
    assert!((high.hue() - 149.).abs() < 10.);
    assert!(high.chroma() > 89.);
    assert!(high.tone() > 0. && high.tone() < 100.);

    let low = TonalPalette::from_hue_and_chroma(50., 3.).key_color();
    assert!((low.hue() - 50.).abs() < 10.);
    assert!((low.chroma() - 3.).abs() < 0.5);
    assert!((low.tone() - 50.).abs() < 0.5);
}

#[test]
fn dislike_vectors() {
    for color in [
        0xfff6ede4, 0xfff3e7db, 0xfff7ead0, 0xffeadaba, 0xffd7bd96, 0xffa07e56, 0xff825c43,
        0xff604134, 0xff3a312a, 0xff292420,
    ] {
        assert!(!is_disliked(Hct::from_argb(color)), "{color:x}");
    }
    for color in [0xff95884b, 0xff716b40, 0xffb08e00, 0xff4c4308, 0xff464521] {
        let hct = Hct::from_argb(color);
        assert!(is_disliked(hct), "{color:x}");
        assert!(!is_disliked(fix_if_disliked(hct)), "{color:x}");
    }
    let color = Hct::from_hct(100., 50., 67.);
    assert!(!is_disliked(color));
    assert_eq!(fix_if_disliked(color).argb(), color.argb());
}

#[test]
fn raw_temperature_vectors() {
    close(raw_temperature(Hct::from_argb(BLUE)), -1.393, 3);
    close(raw_temperature(Hct::from_argb(RED)), 2.351, 3);
    close(raw_temperature(Hct::from_argb(GREEN)), -0.267, 3);
    close(raw_temperature(Hct::from_argb(WHITE)), -0.5, 3);
    close(raw_temperature(Hct::from_argb(BLACK)), -0.5, 3);
}

#[test]
fn relative_temperature_vectors() {
    for (argb, expected) in [
        (BLUE, 0.),
        (RED, 1.),
        (GREEN, 0.467),
        (WHITE, 0.5),
        (BLACK, 0.5),
    ] {
        close(
            TemperatureCache::new(Hct::from_argb(argb)).input_relative_temperature(),
            expected,
            3,
        );
    }
}

#[test]
fn complement_vectors() {
    for (argb, expected) in [
        (BLUE, 0xff9d0002),
        (RED, 0xff007bfc),
        (GREEN, 0xffffd2c9),
        (WHITE, 0xffffffff),
        (BLACK, 0xff000000),
    ] {
        assert_eq!(
            TemperatureCache::new(Hct::from_argb(argb))
                .complement()
                .argb(),
            expected,
            "{argb:x}"
        );
    }
}

#[test]
fn analogous_vectors() {
    for (argb, expected) in [
        (
            BLUE,
            [0xff00590c, 0xff00564e, 0xff0000ff, 0xff6700cc, 0xff81009f],
        ),
        (
            RED,
            [0xfff60082, 0xfffc004c, 0xffff0000, 0xffd95500, 0xffaf7200],
        ),
        (
            GREEN,
            [0xffcee900, 0xff92f500, 0xff00ff00, 0xff00fd6f, 0xff00fab3],
        ),
        (BLACK, [BLACK; 5]),
        (WHITE, [WHITE; 5]),
    ] {
        let colors: Vec<u32> = TemperatureCache::new(Hct::from_argb(argb))
            .analogous(5, 12)
            .iter()
            .map(Hct::argb)
            .collect();
        assert_eq!(colors, expected, "{argb:x}");
    }
}
