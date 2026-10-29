//! Opt-in paragraph layout with backend-provided cluster geometry.
//!
//! The legacy line layout API deliberately remains unchanged. Backends that
//! cannot preserve this information must report an error instead of inventing
//! clusters from glyph positions or guessing paragraph direction from scripts.
use crate::{LineLayout, Pixels, Result, ShapedLine};
use anyhow::ensure;
use std::{ops::Range, sync::Arc};

/// The paragraph's base direction, independent of alignment or flex direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ParagraphDirection {
    /// Let the platform's Unicode bidi implementation resolve the base level.
    #[default]
    Auto,
    /// Explicit left-to-right paragraph base level.
    Ltr,
    /// Explicit right-to-left paragraph base level.
    Rtl,
}

/// Options for the opt-in, cluster-preserving single-line layout API.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LineLayoutOptions {
    /// The requested paragraph base direction.
    pub paragraph_direction: ParagraphDirection,
}

/// Identifies a glyph in the unchanged platform-shaped line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GlyphAddress {
    /// Index into `LineLayout::runs`.
    pub run_index: usize,
    /// Index into the selected run's glyphs.
    pub glyph_index: usize,
}

/// An indivisible shaping cluster, with its logical source and visual geometry.
///
/// Each extended grapheme must stay indivisible, including across font fallback.
/// Combining marks must remain with their base. A ligature may span multiple
/// graphemes. Font fallback can put members of one cluster in different runs.
/// Glyph addresses do not imply logical byte order.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapedCluster {
    /// Nonempty UTF-8 byte range in the original, unmodified text.
    pub source: Range<usize>,
    /// All glyphs belonging to the cluster. An invisible control may have none.
    pub glyphs: Vec<GlyphAddress>,
    /// Visual horizontal advance interval, ordered low to high, in line space.
    /// This is not an ink bounding box; zero-advance clusters are permitted.
    pub advance: Range<Pixels>,
    /// Unicode bidi embedding level for this cluster (odd levels run RTL).
    pub bidi_level: u8,
}

/// A platform layout with sufficient metadata for visual cluster operations.
#[derive(Debug)]
pub struct DetailedLineLayout {
    /// The original shaped geometry, shared with the corresponding painted line.
    pub layout: Arc<LineLayout>,
    /// Clusters in visual left-to-right order, including ties/overlaps in the
    /// order supplied by the shaper. Do not reconstruct this order by sorting
    /// coordinates. Logical source ranges do not overlap; every glyph occurs once.
    /// Source gaps are allowed for controls omitted by the platform shaper.
    pub clusters: Vec<ShapedCluster>,
    /// Resolved paragraph direction. A backend must never return `Auto` here.
    pub paragraph_direction: ParagraphDirection,
}

impl DetailedLineLayout {
    /// Check the metadata contract before admitting a platform result to cache.
    /// This checks structural consistency; the backend remains responsible for
    /// Unicode shaping and for not dividing a grapheme or required ligature.
    pub fn validate(&self, text: &str, options: LineLayoutOptions) -> Result<()> {
        ensure!(
            self.layout.len == text.len(),
            "layout source length mismatch"
        );
        ensure!(
            self.paragraph_direction != ParagraphDirection::Auto,
            "unresolved paragraph direction"
        );
        ensure!(
            options.paragraph_direction == ParagraphDirection::Auto
                || options.paragraph_direction == self.paragraph_direction,
            "platform ignored explicit paragraph direction"
        );
        ensure!(
            f32::from(self.layout.width).is_finite() && self.layout.width >= crate::px(0.),
            "invalid line advance"
        );
        let mut seen: Vec<Vec<bool>> = self
            .layout
            .runs
            .iter()
            .map(|run| vec![false; run.glyphs.len()])
            .collect();
        let mut ranges = Vec::with_capacity(self.clusters.len());
        for cluster in &self.clusters {
            ensure!(
                cluster.source.start < cluster.source.end
                    && cluster.source.end <= text.len()
                    && text.is_char_boundary(cluster.source.start)
                    && text.is_char_boundary(cluster.source.end),
                "invalid cluster source range"
            );
            ensure!(
                f32::from(cluster.advance.start).is_finite()
                    && f32::from(cluster.advance.end).is_finite()
                    && cluster.advance.start <= cluster.advance.end,
                "invalid cluster advance interval"
            );
            ranges.push(cluster.source.clone());
            for address in &cluster.glyphs {
                let glyph = self
                    .layout
                    .runs
                    .get(address.run_index)
                    .and_then(|run| run.glyphs.get(address.glyph_index))
                    .ok_or_else(|| anyhow::anyhow!("cluster references a nonexistent glyph"))?;
                ensure!(
                    !seen[address.run_index][address.glyph_index],
                    "glyph belongs to multiple clusters"
                );
                ensure!(
                    cluster.source.contains(&glyph.index) && text.is_char_boundary(glyph.index),
                    "glyph source is outside its cluster"
                );
                ensure!(
                    f32::from(glyph.position.x).is_finite()
                        && f32::from(glyph.position.y).is_finite(),
                    "invalid glyph position"
                );
                seen[address.run_index][address.glyph_index] = true;
            }
        }
        ranges.sort_unstable_by_key(|range| range.start);
        ensure!(
            ranges.windows(2).all(|pair| pair[0].end <= pair[1].start),
            "overlapping logical clusters"
        );
        ensure!(
            seen.iter().flatten().all(|seen| *seen),
            "glyph missing cluster metadata"
        );
        Ok(())
    }
}

/// Painted text and cluster metadata sharing the exact same shaped geometry.
#[derive(Clone, Debug)]
pub struct DetailedShapedLine {
    /// Original text and decoration runs. This retains the legacy painter:
    /// its mixed-bidi decoration handling is not upgraded by this metadata API.
    pub line: ShapedLine,
    /// The cluster metadata and shared geometry.
    pub layout: Arc<DetailedLineLayout>,
}

/// Coalesce native shaping clusters that overlap one extended grapheme.
///
/// Input must already be in visual order. This preserves that order rather than
/// sorting by coordinates, which cannot disambiguate zero-width/overlapping
/// advances. A native ligature spanning several graphemes remains indivisible.
/// Missing controls are not fabricated: only the backend knows their pen position.
/// If one merged cluster would straddle unrelated visual clusters, or its native
/// bidi levels disagree, the metadata is rejected rather than guessed.
pub fn merge_grapheme_clusters(
    text: &str,
    clusters: Vec<ShapedCluster>,
) -> Result<Vec<ShapedCluster>> {
    use unicode_segmentation::UnicodeSegmentation;
    if clusters.is_empty() {
        return Ok(clusters);
    }
    let graphemes: Vec<Range<usize>> = text
        .grapheme_indices(true)
        .map(|(start, grapheme)| start..start + grapheme.len())
        .collect();
    let mut expanded = Vec::with_capacity(clusters.len());
    for cluster in &clusters {
        ensure!(
            cluster.source.start < cluster.source.end
                && cluster.source.end <= text.len()
                && text.is_char_boundary(cluster.source.start)
                && text.is_char_boundary(cluster.source.end),
            "invalid native cluster source range"
        );
        ensure!(
            f32::from(cluster.advance.start).is_finite()
                && f32::from(cluster.advance.end).is_finite()
                && cluster.advance.start <= cluster.advance.end,
            "invalid native cluster advance"
        );
        let first = graphemes.partition_point(|range| range.end <= cluster.source.start);
        let after = graphemes.partition_point(|range| range.start < cluster.source.end);
        expanded.push(graphemes[first].start..graphemes[after - 1].end);
    }
    let mut source_order: Vec<usize> = (0..clusters.len()).collect();
    source_order.sort_unstable_by_key(|index| expanded[*index].start);
    let mut groups: Vec<(Range<usize>, Vec<usize>)> = Vec::new();
    for index in source_order {
        let range = &expanded[index];
        if let Some((source, members)) = groups.last_mut()
            && range.start < source.end
        {
            source.end = source.end.max(range.end);
            members.push(index);
        } else {
            groups.push((range.clone(), vec![index]));
        }
    }
    for (_, members) in &mut groups {
        members.sort_unstable();
    }
    groups.sort_unstable_by_key(|(_, members)| members[0]);
    let mut original: Vec<Option<ShapedCluster>> = clusters.into_iter().map(Some).collect();
    let mut result = Vec::with_capacity(groups.len());
    for (source, members) in groups {
        ensure!(
            members.windows(2).all(|pair| pair[1] == pair[0] + 1),
            "grapheme members are interleaved with unrelated visual clusters"
        );
        let mut merged = original[members[0]]
            .take()
            .expect("unique cluster membership");
        merged.source = source;
        for index in members.into_iter().skip(1) {
            let cluster = original[index].take().expect("unique cluster membership");
            ensure!(
                cluster.bidi_level == merged.bidi_level,
                "grapheme has conflicting native bidi levels"
            );
            merged.advance.start = merged.advance.start.min(cluster.advance.start);
            merged.advance.end = merged.advance.end.max(cluster.advance.end);
            merged.glyphs.extend(cluster.glyphs);
        }
        result.push(merged);
    }
    Ok(result)
}

impl DetailedShapedLine {
    /// Retain a contiguous visual cluster range without reshaping its source.
    ///
    /// Positions and source indices remain in the original line coordinate space.
    /// This narrow primitive supports a single plain-color decoration only;
    /// mixed styles and decorations need a bidi-aware decoration painter.
    pub fn visual_slice(&self, range: Range<usize>) -> Result<ShapedLine> {
        self.layout
            .validate(&self.line.text, LineLayoutOptions::default())?;
        ensure!(
            Arc::ptr_eq(&self.line.layout, &self.layout.layout),
            "painted line and metadata no longer share geometry"
        );
        ensure!(
            range.start <= range.end && range.end <= self.layout.clusters.len(),
            "invalid visual cluster range"
        );
        ensure!(
            self.line.decoration_runs.len() <= 1
                && self
                    .line
                    .decoration_runs
                    .iter()
                    .all(|run| run.underline.is_none()
                        && run.strikethrough.is_none()
                        && run.background_color.is_none()),
            "visual slicing requires plain uniform text"
        );
        let mut keep: Vec<Vec<bool>> = self
            .layout
            .layout
            .runs
            .iter()
            .map(|run| vec![false; run.glyphs.len()])
            .collect();
        for cluster in &self.layout.clusters[range] {
            for address in &cluster.glyphs {
                keep[address.run_index][address.glyph_index] = true;
            }
        }
        let source = &self.layout.layout;
        let runs = source
            .runs
            .iter()
            .zip(keep)
            .map(|(run, keep)| crate::ShapedRun {
                font_id: run.font_id,
                glyphs: run
                    .glyphs
                    .iter()
                    .zip(keep)
                    .filter(|(_, keep)| *keep)
                    .map(|(glyph, _)| glyph.clone())
                    .collect(),
            })
            .collect();
        Ok(ShapedLine {
            layout: Arc::new(LineLayout {
                font_size: source.font_size,
                width: source.width,
                ascent: source.ascent,
                descent: source.descent,
                runs,
                len: source.len,
            }),
            text: self.line.text.clone(),
            decoration_runs: self.line.decoration_runs.clone(),
        })
    }
}

#[cfg(test)]
mod visual_slice_tests {
    use super::*;
    use crate::{FontId, GlyphId, ShapedGlyph, ShapedRun, point, px};
    #[test]
    fn visual_slice_preserves_source_and_positions_across_font_runs() {
        let glyph = |index, x| ShapedGlyph {
            id: GlyphId(index as u32),
            position: point(px(x), px(0.)),
            index,
            is_emoji: false,
        };
        let geometry = Arc::new(LineLayout {
            font_size: px(16.),
            width: px(30.),
            len: 5,
            runs: vec![
                ShapedRun {
                    font_id: FontId(0),
                    glyphs: vec![glyph(3, 0.), glyph(0, 10.)],
                },
                ShapedRun {
                    font_id: FontId(1),
                    glyphs: vec![glyph(1, 10.)],
                },
            ],
            ..Default::default()
        });
        let detailed = DetailedShapedLine {
            line: ShapedLine {
                layout: geometry.clone(),
                text: "a\u{301}א".into(),
                decoration_runs: Default::default(),
            },
            layout: Arc::new(DetailedLineLayout {
                layout: geometry,
                paragraph_direction: ParagraphDirection::Rtl,
                clusters: vec![
                    ShapedCluster {
                        source: 3..5,
                        advance: px(0.)..px(10.),
                        bidi_level: 1,
                        glyphs: vec![GlyphAddress {
                            run_index: 0,
                            glyph_index: 0,
                        }],
                    },
                    ShapedCluster {
                        source: 0..3,
                        advance: px(10.)..px(30.),
                        bidi_level: 0,
                        glyphs: vec![
                            GlyphAddress {
                                run_index: 0,
                                glyph_index: 1,
                            },
                            GlyphAddress {
                                run_index: 1,
                                glyph_index: 0,
                            },
                        ],
                    },
                ],
            }),
        };
        let mut detailed = detailed;
        let slice = detailed.visual_slice(1..2).unwrap();
        assert_eq!(slice.text, detailed.line.text);
        assert_eq!(slice.len(), 5);
        assert_eq!(slice.runs[0].glyphs.len(), 1);
        assert_eq!(slice.runs[0].glyphs[0].index, 0);
        assert_eq!(slice.runs[0].glyphs[0].position.x, px(10.));
        assert_eq!(slice.runs[1].glyphs[0].index, 1);
        assert!(detailed.visual_slice(0..3).is_err());
        assert!(
            detailed
                .visual_slice(2..2)
                .unwrap()
                .runs
                .iter()
                .all(|r| r.glyphs.is_empty())
        );
        detailed.line = detailed.line.clone().with_len(0);
        assert!(detailed.visual_slice(0..1).is_err());
    }
}
