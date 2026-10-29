#![cfg(target_os = "macos")]
use gpui::{FontRun, LineLayoutOptions, ParagraphDirection, PlatformTextSystem, font, px};

#[test]
fn explicit_direction_preserves_visual_clusters_and_graphemes() {
    let fonts = backend();
    let font_id = fonts.font_id(&font("Helvetica")).unwrap();
    for text in [
        "abc אבג 123 xyz",
        "אבג",
        "abc",
        "e\u{301}",
        "السَّلَام",
        "بِسْمِ",
        "لا",
        "कि",
        "👩‍👩‍👧‍👦",
        "☀️",
        "👍🏽",
        "  אבג  ",
        "a\u{2067}אבג\u{2069}z",
        "a\u{202b}אבג\u{202c}z",
        "\u{202e}abc\u{202c}",
        " 123 () ",
        "\u{feff}ab",
    ] {
        for direction in [
            ParagraphDirection::Auto,
            ParagraphDirection::Ltr,
            ParagraphDirection::Rtl,
        ] {
            let options = LineLayoutOptions {
                paragraph_direction: direction,
            };
            let detailed = fonts
                .layout_line_with_options(
                    text,
                    px(20.),
                    &[FontRun {
                        font_id,
                        len: text.len(),
                    }],
                    options,
                )
                .unwrap_or_else(|error| panic!("{text:?} {direction:?}: {error:#}"));
            detailed.validate(text, options).unwrap();
            assert!(
                !detailed.clusters.is_empty(),
                "visible source disappeared: {text:?} {direction:?}"
            );
            if text == "אבג" || text == "abc" {
                let visual = detailed
                    .clusters
                    .iter()
                    .map(|cluster| &text[cluster.source.clone()])
                    .collect::<String>();
                assert_eq!(
                    visual,
                    if text == "אבג" { "גבא" } else { "abc" },
                    "wrong visual traversal {direction:?}"
                );
            }
            for pair in detailed.clusters.windows(2) {
                assert!(
                    pair[0].advance.start <= pair[1].advance.start,
                    "visual order {text:?} {direction:?}: {:?}",
                    detailed.clusters
                );
            }
            if ["e\u{301}", "कि", "👩‍👩‍👧‍👦", "☀️", "👍🏽"].contains(&text)
            {
                assert_eq!(
                    detailed.clusters.len(),
                    1,
                    "EGC split {text:?}: {:?}",
                    detailed.clusters
                );
                assert_eq!(detailed.clusters[0].source, 0..text.len());
            }
        }
    }
}

#[test]
fn paragraph_direction_and_invalid_input() {
    let fonts = backend();
    let font_id = fonts.font_id(&font("Helvetica")).unwrap();
    for (text, expected) in [
        ("abc אבג", ParagraphDirection::Ltr),
        ("אבג abc", ParagraphDirection::Rtl),
    ] {
        let actual = fonts
            .layout_line_with_options(
                text,
                px(20.),
                &[FontRun {
                    font_id,
                    len: text.len(),
                }],
                LineLayoutOptions::default(),
            )
            .unwrap();
        assert_eq!(actual.paragraph_direction, expected);
    }
    let rtl = LineLayoutOptions {
        paragraph_direction: ParagraphDirection::Rtl,
    };
    let empty = fonts
        .layout_line_with_options("", px(20.), &[], rtl)
        .unwrap();
    assert_eq!(empty.paragraph_direction, ParagraphDirection::Rtl);
    assert!(empty.clusters.is_empty());
    assert!(
        fonts
            .layout_line_with_options("x", px(20.), &[], rtl)
            .is_err()
    );
    assert!(
        fonts
            .layout_line_with_options("x", px(f32::NAN), &[FontRun { font_id, len: 1 }], rtl)
            .is_err()
    );
    assert!(
        fonts
            .layout_line_with_options("a\nb", px(20.), &[FontRun { font_id, len: 3 }], rtl)
            .is_err()
    );
}

fn backend() -> impl PlatformTextSystem {
    gpui_apple::AppleTextSystem::new()
}

#[test]
fn attached_arabic_marks_do_not_expand_pen_cells() {
    let fonts = backend();
    let font_id = fonts.font_id(&font("Helvetica")).unwrap();
    let text = "بِسْمِ";
    for direction in [
        ParagraphDirection::Auto,
        ParagraphDirection::Ltr,
        ParagraphDirection::Rtl,
    ] {
        let detailed = fonts
            .layout_line_with_options(
                text,
                px(24.),
                &[FontRun {
                    font_id,
                    len: text.len(),
                }],
                LineLayoutOptions {
                    paragraph_direction: direction,
                },
            )
            .unwrap();
        assert_eq!(detailed.clusters.len(), 3);
        assert_eq!(
            detailed
                .clusters
                .iter()
                .map(|cluster| cluster.source.clone())
                .collect::<Vec<_>>(),
            vec![8..12, 4..8, 0..4]
        );
        assert!(f32::from(detailed.clusters[0].advance.start).abs() < 0.001);
        for pair in detailed.clusters.windows(2) {
            assert!(
                f32::from(pair[0].advance.end - pair[1].advance.start).abs() < 0.001,
                "marks changed cell boundary: {:?}",
                detailed.clusters
            );
        }
        assert!(
            f32::from(detailed.clusters.last().unwrap().advance.end - detailed.layout.width).abs()
                < 0.001
        );
    }
}

