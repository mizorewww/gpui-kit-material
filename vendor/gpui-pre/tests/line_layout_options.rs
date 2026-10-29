use gpui::{
    DetailedLineLayout, FontId, GlyphAddress, GlyphId, LineLayout, LineLayoutOptions,
    ParagraphDirection, ShapedCluster, ShapedGlyph, ShapedRun, merge_grapheme_clusters, point, px,
};
use std::{ops::Range, sync::Arc};

fn cluster(source: Range<usize>, index: usize, level: u8) -> ShapedCluster {
    ShapedCluster {
        source,
        glyphs: vec![GlyphAddress {
            run_index: 0,
            glyph_index: index,
        }],
        advance: px(index as f32 * 10.)..px(index as f32 * 10. + 10.),
        bidi_level: level,
    }
}

#[test]
fn graphemes_merge_marks_without_reordering_rtl_clusters() {
    let clusters = vec![
        cluster(3..4, 0, 1),
        cluster(0..1, 1, 1),
        cluster(1..3, 2, 1),
    ];
    let merged = merge_grapheme_clusters("a\u{301}b", clusters).unwrap();
    assert_eq!(merged.len(), 2);
    assert_eq!(merged[0].source, 3..4);
    assert_eq!(merged[1].source, 0..3);
    assert_eq!(merged[1].glyphs.len(), 2);
    assert_eq!(merged[1].advance, px(10.)..px(30.));
}

#[test]
fn zero_width_and_emoji_zwj_members_remain_indivisible() {
    let text = "👩\u{200d}💻x";
    let mut mark = cluster(4..7, 1, 0);
    mark.advance = px(10.)..px(10.);
    let merged = merge_grapheme_clusters(
        text,
        vec![
            cluster(0..4, 0, 0),
            mark,
            cluster(7..11, 2, 0),
            cluster(11..12, 3, 0),
        ],
    )
    .unwrap();
    assert_eq!(merged.len(), 2);
    assert_eq!(merged[0].source, 0..11);
    assert_eq!(merged[0].glyphs.len(), 3);
}

#[test]
fn native_ligature_spanning_graphemes_is_not_split() {
    let merged =
        merge_grapheme_clusters("ffi!", vec![cluster(0..3, 0, 0), cluster(3..4, 1, 0)]).unwrap();
    assert_eq!(merged[0].source, 0..3);
    assert_eq!(merged.len(), 2);
}

#[test]
fn inconsistent_native_visual_or_bidi_membership_is_rejected() {
    assert!(
        merge_grapheme_clusters(
            "a\u{301}b",
            vec![
                cluster(0..1, 0, 0),
                cluster(3..4, 1, 0),
                cluster(1..3, 2, 0)
            ]
        )
        .is_err()
    );
    assert!(
        merge_grapheme_clusters("a\u{301}", vec![cluster(0..1, 0, 0), cluster(1..3, 1, 1)])
            .is_err()
    );
}

fn detailed(index: usize) -> DetailedLineLayout {
    DetailedLineLayout {
        layout: Arc::new(LineLayout {
            len: 2,
            width: px(10.),
            runs: vec![ShapedRun {
                font_id: FontId(0),
                glyphs: vec![ShapedGlyph {
                    id: GlyphId(0),
                    position: point(px(0.), px(0.)),
                    index,
                    is_emoji: false,
                }],
            }],
            ..Default::default()
        }),
        clusters: vec![cluster(0..2, 0, 0)],
        paragraph_direction: ParagraphDirection::Ltr,
    }
}

#[test]
fn validator_rejects_mid_utf8_indices_and_ignored_direction() {
    assert!(
        detailed(0)
            .validate("é", LineLayoutOptions::default())
            .is_ok()
    );
    assert!(
        detailed(1)
            .validate("é", LineLayoutOptions::default())
            .is_err()
    );
    assert!(
        detailed(0)
            .validate(
                "é",
                LineLayoutOptions {
                    paragraph_direction: ParagraphDirection::Rtl
                }
            )
            .is_err()
    );
    let mut invalid = detailed(0);
    invalid.clusters[0].glyphs.clear();
    assert!(invalid.validate("é", LineLayoutOptions::default()).is_err());
}

#[test]
fn grapheme_merge_preserves_fallback_run_addresses() {
    let base = cluster(0..1, 0, 0);
    let mut mark = cluster(1..3, 0, 0);
    mark.glyphs[0].run_index = 1;
    mark.advance = px(10.)..px(10.);
    let merged = merge_grapheme_clusters("a\u{301}", vec![base, mark]).unwrap();
    assert_eq!(merged.len(), 1);
    assert_eq!(
        merged[0].glyphs,
        vec![
            GlyphAddress {
                run_index: 0,
                glyph_index: 0
            },
            GlyphAddress {
                run_index: 1,
                glyph_index: 0
            }
        ]
    );
}

#[test]
fn merging_does_not_hide_invalid_native_advance() {
    let mut mark = cluster(1..3, 1, 0);
    mark.advance = px(f32::NAN)..px(1.);
    assert!(merge_grapheme_clusters("a\u{301}", vec![cluster(0..1, 0, 0), mark]).is_err());
}
