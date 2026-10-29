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

#[test]
fn registered_non_latin_font_is_used_in_explicit_fallback_cascade() {
    use gpui::{FontFallbacks, TextSystem};
    use std::{borrow::Cow, sync::Arc};
    let fonts = gpui_apple::AppleTextSystem::new();
    fonts.add_fonts(vec![
        Cow::Borrowed(include_bytes!("../../../crates/material/examples/gallery/fonts/roboto/Roboto-Regular.ttf")),
        Cow::Borrowed(include_bytes!("../../../crates/material/examples/gallery/fonts/conformance/notosanshebrew/NotoSansHebrew-Regular.ttf")),
    ]).unwrap();
    let hebrew = fonts
        .font_id(&font("Noto Sans Hebrew"))
        .expect("a non-Latin font does not require m");
    fonts.add_fonts(vec![Cow::Borrowed(include_bytes!("../../../crates/material/examples/gallery/fonts/conformance/notoemoji/NotoEmoji-Regular.ttf"))]).unwrap();
    let emoji = fonts
        .font_id(&font("Noto Emoji"))
        .expect("a font does not require Latin m");
    assert!(fonts.glyph_for_char(emoji, 'm').is_none());
    let alef = fonts
        .glyph_for_char(hebrew, 'א')
        .expect("actual requested font covers Hebrew");
    let mut primary = font("Roboto");
    primary.fallbacks = Some(FontFallbacks::from_fonts(vec!["Noto Sans Hebrew".into()]));
    let primary_id = fonts.font_id(&primary).unwrap();
    let text = "א";
    let line = fonts
        .layout_line_with_options(
            text,
            px(24.),
            &[FontRun {
                len: text.len(),
                font_id: primary_id,
            }],
            LineLayoutOptions::default(),
        )
        .unwrap();
    assert_eq!(line.layout.runs.len(), 1);
    assert_eq!(
        line.layout.runs[0].font_id, hebrew,
        "CoreText must use the app-private cascade font rather than a system substitute"
    );
    assert_eq!(line.layout.runs[0].glyphs[0].id, alef);
    let arial = fonts.font_id(&font("Arial")).unwrap();
    assert!(fonts.glyph_for_char(arial, 'א').is_some());
    for (fallbacks, expected) in [
        (vec!["Arial".into(), "Noto Sans Hebrew".into()], arial),
        (vec!["Noto Sans Hebrew".into(), "Arial".into()], hebrew),
    ] {
        let mut request = font("Roboto");
        request.fallbacks = Some(FontFallbacks::from_fonts(fallbacks));
        let id = fonts.font_id(&request).unwrap();
        let ordered = fonts
            .layout_line_with_options(
                text,
                px(24.),
                &[FontRun {
                    len: text.len(),
                    font_id: id,
                }],
                LineLayoutOptions::default(),
            )
            .unwrap();
        assert_eq!(
            ordered.layout.runs[0].font_id, expected,
            "explicit mixed system/app fallback order must be honored"
        );
    }
    let system = TextSystem::new(Arc::new(fonts));
    assert!(system.em_width(emoji, px(24.)).unwrap() > px(0.));
    assert!(system.em_advance(emoji, px(24.)).unwrap() > px(0.));
}

#[test]
fn adding_a_registered_fallback_rebuilds_cached_cascades() {
    use gpui::FontFallbacks;
    use std::borrow::Cow;
    let fonts = gpui_apple::AppleTextSystem::new();
    fonts
        .add_fonts(vec![Cow::Borrowed(include_bytes!(
            "../../../crates/material/examples/gallery/fonts/roboto/Roboto-Regular.ttf"
        ))])
        .unwrap();
    let mut primary = font("Roboto");
    primary.fallbacks = Some(FontFallbacks::from_fonts(vec!["Noto Sans Hebrew".into()]));
    let before = fonts.font_id(&primary).unwrap();
    fonts.add_fonts(vec![Cow::Borrowed(include_bytes!("../../../crates/material/examples/gallery/fonts/conformance/notosanshebrew/NotoSansHebrew-Regular.ttf"))]).unwrap();
    let expected = fonts.font_id(&font("Noto Sans Hebrew")).unwrap();
    let after = fonts.font_id(&primary).unwrap();
    assert_ne!(before, after);
    assert!(
        fonts.font_metrics(before).units_per_em > 0,
        "existing frames retain valid font IDs"
    );
    let text = "א";
    let line = fonts
        .layout_line_with_options(
            text,
            px(24.),
            &[FontRun {
                len: text.len(),
                font_id: after,
            }],
            LineLayoutOptions::default(),
        )
        .unwrap();
    assert_eq!(line.layout.runs[0].font_id, expected);
}

#[test]
fn registered_sbix_color_font_retains_its_actual_glyphs() {
    use gpui::FontFallbacks;
    use std::borrow::Cow;
    let fonts = gpui_apple::AppleTextSystem::new();
    fonts.add_fonts(vec![
        Cow::Borrowed(include_bytes!("../../../crates/material/examples/gallery/fonts/roboto/Roboto-Regular.ttf")),
        Cow::Borrowed(include_bytes!("../../../crates/material/examples/gallery/fonts/conformance/notocoloremoji/NotoColorEmoji-Probe-sbix.ttf")),
    ]).unwrap();
    let emoji = fonts.font_id(&font("Noto Color Emoji")).unwrap();
    let mut primary = font("Roboto");
    primary.fallbacks = Some(FontFallbacks::from_fonts(vec!["Noto Color Emoji".into()]));
    let id = fonts.font_id(&primary).unwrap();
    let text = "👩‍👩‍👧‍👦";
    let detailed = fonts
        .layout_line_with_options(
            text,
            px(24.),
            &[FontRun {
                font_id: id,
                len: text.len(),
            }],
            LineLayoutOptions::default(),
        )
        .unwrap();
    assert_eq!(detailed.clusters.len(), 1);
    assert_eq!(detailed.layout.runs.len(), 1);
    assert_eq!(detailed.layout.runs[0].font_id, emoji);
    assert!(
        detailed.layout.runs[0]
            .glyphs
            .iter()
            .all(|g| g.id.0 != 0 && g.is_emoji)
    );
    let params = gpui::RenderGlyphParams {
        font_id: emoji,
        glyph_id: detailed.layout.runs[0].glyphs[0].id,
        font_size: px(24.),
        subpixel_variant: gpui::point(0, 0),
        scale_factor: 1.,
        is_emoji: true,
        subpixel_rendering: false,
        dilation: 0,
    };
    let bounds = fonts.glyph_raster_bounds(&params).unwrap();
    assert!(
        bounds.size.width.0 > 20 && bounds.size.height.0 > 20,
        "bitmap glyph needs real raster bounds: {bounds:?}"
    );
    let (_, bitmap) = fonts.rasterize_glyph(&params, bounds).unwrap();
    assert!(
        bitmap.chunks_exact(4).any(|pixel| pixel[3] != 0),
        "registered color glyph must actually paint pixels"
    );
}
