//! A block surface draws blocks inside a container and changes nothing else about
//! them: a selection, a find and a reveal go across and inside it as in a document
//! without one. `vendor/gpui-base/src/text`, `MarkdownExtensions::block_surface`.
extern crate gpui_kit as gpui;

use core::prelude::v1::test;
use gpui::{prelude::*, *};
use gpui_kit::base::{
    TextSelection, TextSelectionLayer,
    text::{MarkdownExtensions, RangeHighlight, SelectionFormat, TextView, TextViewState},
};

const HEADER: f32 = 30.;
const PADDING: f32 = 20.;
const WIDTH: f32 = 500.;

fn extensions() -> MarkdownExtensions {
    MarkdownExtensions::default()
        .parser_revision(1)
        .block_surface("summary", |_, _| {
            div()
                .bg(hsla(0.6, 0.5, 0.9, 1.))
                .rounded(px(20.))
                .p(px(PADDING))
                .child(div().h(px(HEADER)).child("Summary"))
        })
}

fn fenced(body: &str) -> String {
    format!("<!-- surface:summary -->\n\n{body}\n\n<!-- /surface -->")
}

fn words(count: usize) -> String {
    (0..count)
        .map(|ix| format!("w{ix}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn paragraphs(count: usize) -> String {
    (0..count)
        .map(|ix| format!("paragraph {ix}"))
        .collect::<Vec<_>>()
        .join("\n\n")
}

struct Root {
    view: Entity<TextViewState>,
    width: f32,
    height: f32,
    scrollable: bool,
    format: SelectionFormat,
    /// Without one, the same document is a plain one with comments in it.
    surfaces: bool,
}

impl Render for Root {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(self.width))
            .h(px(self.height))
            .text_size(px(16.))
            .line_height(px(24.))
            .child(TextSelectionLayer)
            .child(
                TextView::new(&self.view)
                    .scrollable(self.scrollable)
                    .selection_format(self.format)
                    .markdown_extensions(if self.surfaces {
                        extensions()
                    } else {
                        MarkdownExtensions::default().parser_revision(1)
                    }),
            )
    }
}

struct Doc<'a> {
    root: Entity<Root>,
    view: Entity<TextViewState>,
    cx: &'a mut VisualTestContext,
}

fn settle(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }
}

fn open<'a>(
    source: &str,
    (height, scrollable, format, surfaces): (f32, bool, SelectionFormat, bool),
    cx: &'a mut TestAppContext,
) -> Doc<'a> {
    cx.update(gpui_kit::init);
    let (root, cx) = cx.add_window_view(|_, cx| Root {
        view: cx.new(|cx| TextViewState::markdown(source, cx).selectable(true)),
        width: WIDTH,
        height,
        scrollable,
        format,
        surfaces,
    });
    let view = root.read_with(cx, |root, _| root.view.clone());
    settle(cx);
    Doc { root, view, cx }
}

impl Doc<'_> {
    /// A window position, from the view's top left.
    fn at(&mut self, x: f32, y: f32) -> Point<Pixels> {
        let bounds = self.view.read_with(self.cx, |view, _| view.bounds());
        bounds.origin + point(px(x), px(y))
    }

    fn drag(&mut self, from: Point<Pixels>, to: Point<Pixels>) {
        self.cx
            .simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
        settle(self.cx);
        self.cx
            .simulate_mouse_move(to, MouseButton::Left, Modifiers::default());
        settle(self.cx);
        self.cx
            .simulate_mouse_up(to, MouseButton::Left, Modifiers::default());
        settle(self.cx);
    }

    /// What the app's copy reads, without writing to the clipboard.
    fn copied(&mut self) -> String {
        self.cx
            .update(|window, cx| TextSelection::selected_text(window, cx))
    }

    fn select_all_plain_lines(&mut self) -> Vec<String> {
        self.copied().lines().map(str::to_string).collect()
    }

    fn scroll_top(&mut self) -> ListOffset {
        self.view
            .read_with(self.cx, |view, _| view.list_state().logical_scroll_top())
    }

    fn reveal(&mut self, needle: &str) {
        self.view.update(self.cx, |view, cx| {
            let text = view.rendered_text();
            let start = text.as_str().find(needle).unwrap();
            view.reveal_range(start..start + needle.len(), cx).unwrap();
        });
        settle(self.cx);
    }

    fn item_height(&mut self, item: usize) -> Pixels {
        self.view.read_with(self.cx, |view, _| {
            view.list_state().bounds_for_item(item).unwrap().size.height
        })
    }

    fn extent(&mut self) -> Pixels {
        self.view.read_with(self.cx, |view, _| {
            let list = view.list_state();
            list.max_offset_for_scrollbar().y + list.viewport_bounds().size.height
        })
    }
}

/// Before, two paragraphs in a surface, after: one 24px line each, with a 12px
/// paragraph gap (0.75rem at a 16px rem). The `_Y`s are the middles of their lines.
const DOCUMENT: &str = "before text\n\n<!-- surface:summary -->\n\ninside one\n\ninside two\n\n<!-- /surface -->\n\nafter text";
const BEFORE_Y: f32 = 12.;
const ONE_Y: f32 = 24. + 12. + PADDING + HEADER + 12.;
const TWO_Y: f32 = ONE_Y + 24. + 12.;
const AFTER_Y: f32 = TWO_Y + 12. + PADDING + 12. + 12.;

fn copies(format: SelectionFormat, scrollable: bool, cx: &mut TestAppContext) -> [String; 4] {
    let mut doc = open(DOCUMENT, (600., scrollable, format, true), cx);
    let inner = PADDING + 1.;
    let far = WIDTH - 60.;
    let (before, one, two, after) = (
        doc.at(1., BEFORE_Y),
        doc.at(inner, ONE_Y),
        doc.at(far, TWO_Y),
        doc.at(far, AFTER_Y),
    );
    let mut copy = |from, to| {
        doc.drag(from, to);
        let forward = doc.copied();
        doc.cx.update(|window, cx| TextSelection::clear(window, cx));
        settle(doc.cx);
        doc.drag(to, from);
        assert_eq!(forward, doc.copied(), "a drag copies the same both ways");
        doc.cx.update(|window, cx| TextSelection::clear(window, cx));
        settle(doc.cx);
        forward.trim_end().to_string()
    };
    [
        // Inside, across the blocks of the surface.
        copy(one, two),
        // In from before it, out past it, and all the way over it.
        copy(before, two),
        copy(one, after),
        copy(before, after),
    ]
}

#[gpui::test]
fn a_selection_copies_as_text_across_and_inside(cx: &mut TestAppContext) {
    for scrollable in [false, true] {
        assert_eq!(
            copies(SelectionFormat::Plain, scrollable, cx),
            [
                "inside one\ninside two",
                "before text\ninside one\ninside two",
                "inside one\ninside two\nafter text",
                "before text\ninside one\ninside two\nafter text",
            ],
            "scrollable: {scrollable}"
        );
    }
}

#[gpui::test]
fn a_selection_copies_as_source_without_fences(cx: &mut TestAppContext) {
    for scrollable in [false, true] {
        assert_eq!(
            copies(SelectionFormat::Source, scrollable, cx),
            [
                "inside one\n\ninside two",
                "before text\n\ninside one\n\ninside two",
                "inside one\n\ninside two\n\nafter text",
                "before text\n\ninside one\n\ninside two\n\nafter text",
            ],
            "scrollable: {scrollable}"
        );
    }
}

#[gpui::test]
fn a_select_all_copies_no_fences_and_no_title(cx: &mut TestAppContext) {
    let mut doc = open(DOCUMENT, (600., true, SelectionFormat::Plain, true), cx);
    doc.view.update(doc.cx, |view, cx| view.select_all(cx));
    settle(doc.cx);
    assert_eq!(
        doc.select_all_plain_lines(),
        ["before text", "inside one", "inside two", "after text"]
    );
}

#[gpui::test]
fn the_title_is_not_selectable_text(cx: &mut TestAppContext) {
    let mut doc = open(DOCUMENT, (600., false, SelectionFormat::Plain, true), cx);
    let title = doc.at(PADDING + 1., 24. + 12. + PADDING + 15.);
    let in_the_line = doc.at(PADDING + 40., ONE_Y);
    doc.drag(title, in_the_line);
    let copied = doc.copied();
    assert!(!copied.is_empty());
    assert!("inside one".starts_with(copied.trim_end()), "{copied}");
}

#[gpui::test]
fn the_rendered_text_and_a_find_ignore_the_surface(cx: &mut TestAppContext) {
    let mut doc = open(DOCUMENT, (600., true, SelectionFormat::Plain, true), cx);
    let rendered = doc
        .view
        .read_with(doc.cx, |view, _| view.rendered_text().as_str().to_string());
    assert_eq!(
        rendered,
        "before text\ninside one\ninside two\nafter text\n"
    );

    // A match inside, and one over each edge, draw on a real frame.
    doc.view.update(doc.cx, |view, cx| {
        let color = gpui::red();
        view.set_range_highlights(
            [
                RangeHighlight::new(
                    rendered.find("one").unwrap()..rendered.find("two").unwrap(),
                    color,
                ),
                RangeHighlight::new(
                    rendered.find("text").unwrap()..rendered.find(" one").unwrap(),
                    color,
                ),
                RangeHighlight::new(
                    rendered.find(" two").unwrap()..rendered.find("after").unwrap() + 5,
                    color,
                ),
            ],
            cx,
        )
        .unwrap();
    });
    settle(doc.cx);

    // Source positions still map through the fences.
    doc.view.read_with(doc.cx, |view, _| {
        let start = DOCUMENT.find("inside two").unwrap();
        let range = view.rendered_text().range_for_source(start..start + 10);
        assert_eq!(
            range,
            rendered.find("inside two").map(|start| start..start + 10)
        );
    });
}

#[gpui::test]
fn a_reveal_reaches_a_line_inside_a_surface(cx: &mut TestAppContext) {
    // A long surface, which is one row: the line has to be scrolled to within it.
    let source = format!("{}\n\nafter", fenced(&words(800)));
    let mut doc = open(&source, (100., true, SelectionFormat::Plain, true), cx);
    doc.reveal("w790");
    let top = doc.scroll_top();
    assert_eq!(top.item_ix, 0, "{top:?}");
    assert!(top.offset_in_item > px(100.), "{top:?}");
}

#[gpui::test]
fn a_reveal_reaches_a_surface_off_screen(cx: &mut TestAppContext) {
    let source = format!(
        "{}\n\n{}\n\n{}",
        paragraphs(200),
        fenced("target line"),
        paragraphs(50)
    );
    let mut doc = open(&source, (100., true, SelectionFormat::Plain, true), cx);
    doc.reveal("target line");
    let top = doc.scroll_top();
    assert!((190..=200).contains(&top.item_ix), "{top:?}");
    doc.reveal("paragraph 3");
    assert!(doc.scroll_top().item_ix <= 3);
}

#[gpui::test]
fn a_surface_measures_as_its_blocks_and_its_container(cx: &mut TestAppContext) {
    let plain = {
        let mut doc = open(DOCUMENT, (600., true, SelectionFormat::Plain, false), cx);
        // The comments are blocks of their own here: before, comment, one, two, comment, after.
        assert_eq!(
            doc.view
                .read_with(doc.cx, |view, _| view.list_state().item_count()),
            6
        );
        [doc.item_height(2), doc.item_height(3)]
    };
    let mut doc = open(DOCUMENT, (600., true, SelectionFormat::Plain, true), cx);
    assert_eq!(
        doc.view
            .read_with(doc.cx, |view, _| view.list_state().item_count()),
        3
    );
    // Each paragraph's gap is in its row. In the surface the last one's moves below
    // the container, so it is still one row's worth of each.
    assert_eq!(
        doc.item_height(1),
        plain[0] + plain[1] + px(HEADER + 2. * PADDING),
    );
}

#[gpui::test]
fn a_surface_wraps_inside_its_padding_and_remeasures_at_a_new_width(cx: &mut TestAppContext) {
    let source = format!("{}\n\nafter", fenced(&words(200)));
    // Comment, paragraph, comment, after; the paragraph wraps at the full width.
    let full_width = {
        let mut doc = open(&source, (600., true, SelectionFormat::Plain, false), cx);
        doc.item_height(1)
    };
    let mut doc = open(&source, (600., true, SelectionFormat::Plain, true), cx);
    let (wide, wide_extent) = (doc.item_height(0), doc.extent());
    // The padding takes width from the text, so it wraps into more lines.
    assert!(
        wide > full_width + px(HEADER + 2. * PADDING),
        "{wide:?} {full_width:?}"
    );
    doc.root.update(doc.cx, |root, cx| {
        root.width = WIDTH / 2.;
        cx.notify();
    });
    settle(doc.cx);
    let narrow = doc.item_height(0);
    assert!(narrow > wide * 1.5, "{wide:?} to {narrow:?}");
    assert!(doc.extent() > wide_extent);
}
