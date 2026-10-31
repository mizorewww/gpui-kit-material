#![cfg(target_os = "macos")]
// Text patches kept in the Apple backends: fonts without an `m` glyph,
// app-private fallback cascades and sbix color bitmaps.
use gpui::{FontRun, PlatformTextSystem, font, px};

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
        .layout_line(
            text,
            px(24.),
            &[FontRun {
                len: text.len(),
                font_id: primary_id,
            }],
        );
    assert_eq!(line.runs.len(), 1);
    assert_eq!(
        line.runs[0].font_id, hebrew,
        "CoreText must use the app-private cascade font rather than a system substitute"
    );
    assert_eq!(line.runs[0].glyphs[0].id, alef);
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
            .layout_line(
                text,
                px(24.),
                &[FontRun {
                    len: text.len(),
                    font_id: id,
                }],
        );
        assert_eq!(
            ordered.runs[0].font_id, expected,
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
        .layout_line(
            text,
            px(24.),
            &[FontRun {
                len: text.len(),
                font_id: after,
            }],
        );
    assert_eq!(line.runs[0].font_id, expected);
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
        .layout_line(
            text,
            px(24.),
            &[FontRun {
                font_id: id,
                len: text.len(),
            }],
        );
    assert_eq!(detailed.runs.len(), 1);
    assert_eq!(detailed.runs[0].font_id, emoji);
    assert!(
        detailed.runs[0]
            .glyphs
            .iter()
            .all(|g| g.id.0 != 0 && g.is_emoji)
    );
    let params = gpui::RenderGlyphParams {
        font_id: emoji,
        glyph_id: detailed.runs[0].glyphs[0].id,
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
