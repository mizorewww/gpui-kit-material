//! Where a line of Chinese or Japanese text may not be wrapped.
//!
//! [`crate::LineWrapper`] may break before any character that is not a word
//! character (CJK is not space separated), so a full-width comma or a closing
//! bracket can open a line of its own: `非常好` / `，我们一起`. CJK
//! typesetting forbids that (W3C clreq / jlreq, UAX #14 classes CL, CP, EX,
//! IS and NS), and equally forbids an opening bracket or quote ending a line.
//! The wrappers ([`crate::LineWrapper`] for fragments and truncation,
//! `LineLayout::compute_wrap_boundaries` for shaped text) ask
//! [`allows_break_between`] before they take a break in front of such a
//! character, so the line breaks one character earlier and the punctuation
//! moves down together with the character it belongs to. Where no allowed
//! break fits, they still break where the line overflows.

/// Whether a line may wrap between `prev` and `c`, two characters with no
/// space between them, in text that otherwise breaks between any two
/// characters.
pub(crate) fn allows_break_between(prev: char, c: char) -> bool {
    !is_glue(prev)
        && !is_glue(c)
        && !never_starts_line(c)
        && !never_ends_line(prev)
        && !(prev == '—' && c == '—')
        && !(is_cjk(prev) && closes_cjk_text(c))
}

/// No-break spaces and word joiners (UAX #14 classes GL and WJ) hold the
/// characters on either side together: `11\u{a0}分\u{a0}39\u{a0}秒` never breaks
/// inside, while the text around it still may.
pub(crate) fn is_glue(c: char) -> bool {
    matches!(
        c,
        '\u{00A0}' | '\u{2007}' | '\u{202F}' | '\u{2060}' | '\u{FEFF}'
    )
}

/// Whether a line that overflows at `c` with no allowed break left should take
/// `prev` down too. The forced break would otherwise leave `c` opening a line
/// (`llama.cpp` / `，`). It stays where it is when moving `prev` would not help,
/// because `prev` may not start a line either, or when glue is involved.
pub(crate) fn forced_break_carries_previous(prev_prev: char, prev: char, c: char) -> bool {
    !is_glue(prev)
        && !is_glue(c)
        && !allows_break_between(prev, c)
        && allows_break_between(prev_prev, prev)
}

/// ASCII punctuation that, written right after CJK text, closes it like its
/// full-width form would (UAX #14 classes CL, CP, EX, IS and PO): `你好,世界`.
/// After Latin text a break before it stays, since that is what lets a long
/// path or URL wrap.
fn closes_cjk_text(c: char) -> bool {
    matches!(c, ',' | '.' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '%')
}

/// Han, kana, Hangul and full-width forms, the scripts that wrap between
/// characters rather than at spaces.
fn is_cjk(c: char) -> bool {
    matches!(
        c,
        '\u{3000}'..='\u{30FF}'
            | '\u{3400}'..='\u{4DBF}'
            | '\u{4E00}'..='\u{9FFF}'
            | '\u{AC00}'..='\u{D7AF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{FF00}'..='\u{FFEF}'
            | '\u{20000}'..='\u{3FFFF}'
    )
}

/// Closing and full-width punctuation that must stay at the end of the line
/// before it (clreq's 行首禁则).
fn never_starts_line(c: char) -> bool {
    matches!(
        c,
        // Full-width stops, commas and marks.
        '、' | '。' | '，' | '．' | '：' | '；' | '？' | '！' | '｡' | '､'
            // Closing brackets and quotes.
            | '）' | '］' | '｝' | '」' | '』' | '】' | '〉' | '》' | '〕' | '〗' | '〙'
            | '〛' | '〞' | '〟' | '｠' | '｣' | '’' | '”'
            // Ellipses, middle dots and the like, iteration marks.
            | '…' | '‥' | '・' | '･' | '〜' | '‼' | '⁇' | '⁈' | '⁉' | '々' | '〻' | 'ゝ'
            | 'ゞ' | 'ヽ' | 'ヾ'
            // Postfix full-width signs.
            | '％'
    )
}

/// Opening brackets and quotes that must start the line after them
/// (clreq's 行尾禁则).
///
/// The ASCII brackets are here too, as in UAX #14 (class OP). This only takes
/// breaks away, and only after the bracket: the wrappers break before a word
/// character only after a space, so in Latin text, code and URLs, where `(`
/// is followed by a letter, nothing changes, while `(` no longer ends a line
/// before the CJK text it opens. ASCII closing punctuation is left out, since
/// a break before `)` or `?` is what lets a long path or URL wrap at all.
fn never_ends_line(c: char) -> bool {
    matches!(
        c,
        // ASCII brackets.
        '(' | '[' | '{'
            // Full-width and CJK brackets and quotes.
            | '（'
            | '［'
            | '｛'
            | '「'
            | '『'
            | '【'
            | '〈'
            | '《'
            | '〔'
            | '〖'
            | '〘'
            | '〚'
            | '〝'
            | '｟'
            | '｢'
            | '‘'
            | '“'
    )
}

#[cfg(test)]
mod tests {
    use super::{allows_break_between, is_glue, never_ends_line, never_starts_line};
    use crate::{
        Boundary, FontId, GlyphId, IndentAdjustment, LineFragment, LineLayout, LineWrapper, Pixels,
        ShapedGlyph, ShapedRun, TestAppContext, TestDispatcher, TextRun, TextSystem,
        WindowTextSystem, font, point, px,
    };
    use std::sync::Arc;

    const SENTENCE: &str = "今天的天气非常好，我们一起去公园散步吧。";
    const MIXED: &str = "我用（Rust）重写了程序：速度提升了８％，超出预期……「渲染」部分表现亮眼——尤其是《文档》提到的部分。";

    fn wrapper() -> (LineWrapper, Arc<TextSystem>) {
        let cx = TestAppContext::build(TestDispatcher::new(0), None);
        let text_system = cx.text_system().clone();
        let id = text_system.resolve_font(&font(".ZedMono"));
        (
            LineWrapper::new(id, px(16.), text_system.clone()),
            text_system,
        )
    }

    fn width_of(text_system: &TextSystem, wrapper: &LineWrapper, text: &str) -> Pixels {
        text.chars()
            .map(|c| {
                text_system
                    .advance(wrapper.font_id, wrapper.font_size, c)
                    .unwrap()
                    .width
            })
            .fold(px(0.), |sum, width| sum + width)
    }

    fn wrap(wrapper: &mut LineWrapper, text: &str, width: Pixels) -> Vec<usize> {
        wrapper
            .wrap_line(
                &[LineFragment::text(text)],
                width,
                IndentAdjustment::NoIndent,
            )
            .map(|Boundary { ix, .. }| ix)
            .collect()
    }

    #[test]
    fn a_comma_moves_down_with_the_character_before_it() {
        let (mut wrapper, text_system) = wrapper();
        let fits = "今天的天气非常好";
        let comma = width_of(&text_system, &wrapper, "，");
        assert!(comma > px(0.));
        // The line holds everything before the comma, but not the comma.
        let width = width_of(&text_system, &wrapper, fits) + comma / 2.;

        let boundaries = wrap(&mut wrapper, SENTENCE, width);
        assert_eq!(boundaries[0], fits.len() - "好".len());
        assert_eq!(&SENTENCE[boundaries[0]..], "好，我们一起去公园散步吧。");
    }

    #[test]
    fn an_ascii_comma_after_cjk_text_moves_down_with_it() {
        let (mut wrapper, text_system) = wrapper();
        let sentence = "今天的天气非常好,我们一起去公园散步吧.";
        let fits = "今天的天气非常好";
        let comma = width_of(&text_system, &wrapper, ",");
        assert!(comma > px(0.));
        let width = width_of(&text_system, &wrapper, fits) + comma / 2.;

        let boundaries = wrap(&mut wrapper, sentence, width);
        assert_eq!(&sentence[boundaries[0]..], "好,我们一起去公园散步吧.");
    }

    #[test]
    fn no_width_starts_a_line_with_closing_or_ends_one_with_opening_punctuation() {
        let text = MIXED;
        let (mut wrapper, text_system) = wrapper();
        let one = width_of(&text_system, &wrapper, "中");
        // From the narrowest line `（Rust）` fits on, so no break is forced.
        for columns in 6..text.chars().count() {
            let width = one * columns as f32 + one / 2.;
            for ix in wrap(&mut wrapper, text, width) {
                let next = text[ix..].chars().next().unwrap();
                let prev = text[..ix].chars().next_back().unwrap();
                assert!(
                    !never_starts_line(next) && !never_ends_line(prev),
                    "{columns} columns: `{}` / `{}`",
                    &text[..ix],
                    &text[ix..],
                );
            }
        }
    }

    #[test]
    fn ideographs_and_latin_text_still_break_as_before() {
        assert!(allows_break_between('好', '我'));
        assert!(allows_break_between('，', '我'));
        assert!(allows_break_between('用', '（'));
        assert!(allows_break_between('）', '重'));
        assert!(!allows_break_between('—', '—'));
        assert!(!allows_break_between('８', '％'));
        assert!(allows_break_between('去', '('));
        assert!(!allows_break_between('(', '公'));
        assert!(!allows_break_between('园', ')'));
        // ASCII closing punctuation stays with CJK text, not with Latin text.
        assert!(!allows_break_between('好', ','));
        assert!(!allows_break_between('な', '.'));
        assert!(!allows_break_between('８', '%'));
        assert!(allows_break_between('m', '.'));
        assert!(allows_break_between('b', ')'));
        assert!(allows_break_between(',', '你'));

        let (mut wrapper, text_system) = wrapper();
        let one = width_of(&text_system, &wrapper, "中");
        assert_eq!(wrap(&mut wrapper, "中文中文中文", one * 2.5), [6, 12]);

        // Latin words still break at spaces only, and CJK after them anywhere.
        let width = width_of(&text_system, &wrapper, "world, 你好") + one / 2.;
        assert_eq!(wrap(&mut wrapper, "Hello world, 你好世界", width), [6, 19]);
    }

    #[test]
    fn text_after_an_element_breaks_as_if_it_started_the_line() {
        let (mut wrapper, text_system) = wrapper();
        let one = width_of(&text_system, &wrapper, "中");
        let fragments = [
            LineFragment::text("看（"),
            LineFragment::element(one, 1),
            LineFragment::text("我们一起散步"),
        ];
        // `（` holds on to the element, not to the `我` after it.
        let boundaries: Vec<_> = wrapper
            .wrap_line(&fragments, one * 3.5, IndentAdjustment::NoIndent)
            .map(|Boundary { ix, .. }| ix)
            .collect();
        assert_eq!(boundaries[0], "看（".len() + 1);
    }

    #[test]
    fn shaped_text_keeps_a_comma_with_the_character_before_it() {
        let cx = TestAppContext::build(TestDispatcher::new(0), None);
        let text_system = WindowTextSystem::new(cx.text_system().clone());
        let runs = [TextRun {
            len: SENTENCE.len(),
            font: font(".ZedMono"),
            ..Default::default()
        }];
        let shape = |wrap_width| {
            text_system
                .shape_text(SENTENCE.into(), px(16.), &runs, wrap_width, None)
                .unwrap()
                .remove(0)
        };
        let glyph_x = |line: &crate::WrappedLine, index: usize| {
            line.layout
                .unwrapped_layout
                .runs
                .iter()
                .flat_map(|run| &run.glyphs)
                .find(|glyph| glyph.index == index)
                .unwrap()
                .position
                .x
        };

        // The line ends halfway through the comma.
        let comma = SENTENCE.find('，').unwrap();
        let line = shape(None);
        let width = (glyph_x(&line, comma) + glyph_x(&line, comma + '，'.len_utf8())) / 2.;

        let line = shape(Some(width));
        let boundary = line.layout.wrap_boundaries()[0];
        let layout = &line.layout.unwrapped_layout;
        let ix = layout.runs[boundary.run_ix].glyphs[boundary.glyph_ix].index;
        assert_eq!(&SENTENCE[ix..], "好，我们一起去公园散步吧。");
    }

    /// A shaped line with one glyph per character, each `advance` wide, as
    /// text elements wrap it in `LineLayout::compute_wrap_boundaries`.
    fn layout_of(text: &str, advance: f32) -> LineLayout {
        let glyphs: Vec<_> = text
            .char_indices()
            .enumerate()
            .map(|(column, (index, _))| ShapedGlyph {
                id: GlyphId(0),
                position: point(px(advance * column as f32), px(0.)),
                index,
                is_emoji: false,
            })
            .collect();
        LineLayout {
            font_size: px(16.),
            width: px(advance * glyphs.len() as f32),
            ascent: px(12.),
            descent: px(4.),
            runs: vec![ShapedRun {
                font_id: FontId(0),
                glyphs,
            }],
            len: text.len(),
        }
    }

    fn wrapped_lines(text: &str, wrap_width: f32) -> Vec<&str> {
        let layout = layout_of(text, 10.);
        let mut start = 0;
        let mut lines = Vec::new();
        for boundary in layout.compute_wrap_boundaries(text, px(wrap_width), None) {
            let end = layout.runs[boundary.run_ix].glyphs[boundary.glyph_ix].index;
            lines.push(&text[start..end]);
            start = end;
        }
        lines.push(&text[start..]);
        lines
    }

    #[test]
    fn shaped_lines_keep_cjk_punctuation_with_the_character_before_it() {
        // Eight columns fit, so the comma would open the second line.
        assert_eq!(
            wrapped_lines(SENTENCE, 85.),
            ["今天的天气非常", "好，我们一起去公", "园散步吧。"]
        );
        // `(` ends no line, `)` may start one.
        assert_eq!(
            wrapped_lines("我们去(公园)散步", 45.),
            ["我们去", "(公园)", "散步"]
        );
        // Latin text still breaks after spaces only.
        assert_eq!(
            wrapped_lines("Hello world, 你好世界", 135.),
            ["Hello world, ", "你好世界"]
        );
    }

    #[test]
    fn no_width_breaks_a_shaped_line_inside_cjk_punctuation() {
        // From the narrowest line `（Rust）` fits on, so no break is forced.
        for columns in 6..MIXED.chars().count() {
            let lines = wrapped_lines(MIXED, 10. * columns as f32 + 5.);
            for pair in lines.windows(2) {
                let prev = pair[0].chars().next_back().unwrap();
                let next = pair[1].chars().next().unwrap();
                assert!(
                    allows_break_between(prev, next),
                    "{columns} columns: {lines:?}"
                );
            }
        }
    }

    #[test]
    fn a_shaped_line_breaks_where_it_overflows_when_no_break_is_allowed() {
        // Nothing may break between these, so lines break where they overflow.
        let text = "，。、！？：；，。";
        assert_eq!(
            wrapped_lines(text, 25.),
            ["，。", "、！", "？：", "；，", "。"]
        );
        assert_eq!(
            wrapped_lines(text, 5.),
            ["，", "。", "、", "！", "？", "：", "；", "，", "。"]
        );
        assert_eq!(
            wrapped_lines("好，，，，，，", 35.),
            ["好，，", "，，，", "，"]
        );
    }

    #[test]
    fn glue_forbids_a_break_on_either_side() {
        for glue in ['\u{a0}', '\u{2007}', '\u{202f}', '\u{2060}', '\u{feff}'] {
            assert!(is_glue(glue));
            assert!(!allows_break_between(glue, '分'));
            assert!(!allows_break_between('分', glue));
        }
        assert!(!is_glue(' '));
        assert!(allows_break_between('分', '秒'));
    }

    #[test]
    fn wrapping_never_breaks_inside_no_break_spaces() {
        let (mut wrapper, text_system) = wrapper();
        let one = width_of(&text_system, &wrapper, "中");
        let glued = "11\u{a0}分\u{a0}39\u{a0}秒";
        let second = "1\u{a0}分\u{a0}26\u{a0}秒";
        let text = format!("预计剩余 {glued}，数据分块到达，上一块于 {second}前到达");
        let spans = [
            (text.find(glued).unwrap(), glued.len()),
            (text.find(second).unwrap(), second.len()),
        ];
        // Wide enough for each glued span: lines break around it, never inside.
        for columns in 10..40 {
            let width = one * columns as f32 + one / 2.;
            for ix in wrap(&mut wrapper, &text, width) {
                assert!(
                    spans
                        .iter()
                        .all(|(from, len)| ix <= *from || ix >= from + len),
                    "{columns} columns: break inside a glued span at {ix}"
                );
            }
        }
    }

    #[test]
    fn a_shaped_line_keeps_text_joined_by_glyphless_word_joiners() {
        // CoreText shapes a word joiner (U+2060) to no glyph, so the glyph
        // before 计 is 预; the joiner sits only in the text between them.
        let text = "数据·预\u{2060}计\u{2060}剩\u{2060}余";
        let glyphs: Vec<_> = text
            .char_indices()
            .filter(|(_, character)| *character != '\u{2060}')
            .enumerate()
            .map(|(column, (index, _))| ShapedGlyph {
                id: GlyphId(0),
                position: point(px(16. * column as f32), px(0.)),
                index,
                is_emoji: false,
            })
            .collect();
        let width = px(16. * glyphs.len() as f32);
        let layout = LineLayout {
            font_size: px(16.),
            width,
            ascent: px(12.),
            descent: px(4.),
            runs: vec![ShapedRun {
                font_id: FontId(0),
                glyphs,
            }],
            len: text.len(),
        };
        let joined: Vec<_> = ["计", "剩", "余"]
            .iter()
            .map(|character| text.find(character).unwrap())
            .collect();
        for wrap_width in 64..112 {
            for boundary in layout.compute_wrap_boundaries(text, px(wrap_width as f32), None) {
                let index = layout.runs[boundary.run_ix].glyphs[boundary.glyph_ix].index;
                assert!(
                    !joined.contains(&index),
                    "break inside joined text at {index}, width {wrap_width}"
                );
            }
        }
    }

    #[test]
    fn a_forced_break_carries_the_character_before_closing_punctuation() {
        // No allowed break exists in `abc，`, so the comma takes `c` down with it.
        assert_eq!(wrapped_lines("abc，", 35.), ["ab", "c，"]);
        // Moving a character that may not start a line either would not help.
        assert_eq!(wrapped_lines("，。、", 25.), ["，。", "、"]);

        let (mut wrapper, text_system) = wrapper();
        let fits = "abc";
        let width = width_of(&text_system, &wrapper, fits) + px(1.);
        assert_eq!(wrap(&mut wrapper, "abc，", width), [2]);
    }

    #[cfg(target_os = "macos")]
    fn shaped_boundaries(
        text_system: &WindowTextSystem,
        text: &str,
        run: &TextRun,
        width: f32,
    ) -> Vec<usize> {
        let lines = text_system
            .shape_text(
                text.to_owned().into(),
                px(16.),
                std::slice::from_ref(run),
                Some(px(width)),
                None,
            )
            .unwrap();
        let layout = &lines[0].layout;
        layout
            .wrap_boundaries()
            .iter()
            .map(|boundary| {
                layout.unwrapped_layout.runs[boundary.run_ix].glyphs[boundary.glyph_ix].index
            })
            .collect()
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn core_text_never_opens_a_line_with_closing_punctuation() {
        let cx = TestAppContext::build(TestDispatcher::new(0), None);
        let text_system = WindowTextSystem::new(cx.text_system().clone());
        let text = "缺少识别组件 llama.cpp，安装后重新检查。你好（世界）、再见！好的，谢谢。";
        let run = TextRun {
            len: text.len(),
            font: font("PingFang SC"),
            ..Default::default()
        };
        // From two glyphs per line up: narrower lines must force breaks anywhere.
        for width in 40..240 {
            for index in shaped_boundaries(&text_system, text, &run, width as f32) {
                let next = text[index..].chars().next().unwrap();
                assert!(
                    !never_starts_line(next),
                    "line starts with {next:?} at width {width}"
                );
            }
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn core_text_keeps_glued_text_together() {
        let cx = TestAppContext::build(TestDispatcher::new(0), None);
        let text_system = WindowTextSystem::new(cx.text_system().clone());
        let glued = "11\u{a0}分\u{a0}39\u{a0}秒";
        // Word joiners (zero width, no glyph) hold the CJK label together.
        let text = format!(
            "360.0 MB / 1.56 GB · 速度 1.8 MB/s · 预\u{2060}计\u{2060}剩\u{2060}余\u{a0}{glued}\u{a0}· 数据分块到达，上一块于 1\u{a0}分\u{a0}26\u{a0}秒前到达"
        );
        let start = text.find("预").unwrap();
        let end = text.find(" 数据").unwrap();
        let run = TextRun {
            len: text.len(),
            font: font("PingFang SC"),
            ..Default::default()
        };
        // Wide enough for the glued span: lines break around it, never inside.
        for width in 200..480 {
            for index in shaped_boundaries(&text_system, &text, &run, width as f32) {
                assert!(
                    index <= start || index > end,
                    "break inside a glued span at {index}, width {width}"
                );
            }
        }
    }
}
