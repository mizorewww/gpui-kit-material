//! Block surfaces are presentation: everything that reads the document sees the
//! blocks between the fences as if the fences were not there. The public behavior,
//! gestures included, is tested from outside in
//! `crates/material/tests/block_surface.rs`, which CI runs.
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Pixels, Render,
    SharedString, Styled as _, TestAppContext, VisualTestContext, Window, div, hsla, px,
};

use super::{
    MarkdownExtensions, SelectionFormat, TextView, TextViewState,
    format::markdown,
    node::{BlockNode, NodeContext},
};

fn extensions() -> MarkdownExtensions {
    MarkdownExtensions::default()
        .parser_revision(1)
        .block_surface("summary", |_, _| {
            div()
                .bg(hsla(0.6, 0.5, 0.9, 1.))
                .rounded(px(20.))
                .p(px(20.))
                .child(div().h(px(30.)).child("Summary"))
        })
}

fn parsed(source: &str, extensions: MarkdownExtensions) -> Vec<BlockNode> {
    let mut cx = NodeContext {
        markdown_extensions: Arc::new(extensions),
        ..NodeContext::default()
    };
    let document = markdown::parse(source, &mut cx).unwrap();
    document.blocks.as_ref().clone()
}

fn surface(block: &BlockNode) -> (&SharedString, &[BlockNode]) {
    match block {
        BlockNode::Surface { name, children, .. } => (name, children),
        block => panic!("not a surface: {block:?}"),
    }
}

const DOCUMENT: &str = "before\n\n<!-- surface:summary -->\n\n# inside one\n\ninside **two**\n\n- three\n\n<!-- /surface -->\n\nafter";
const WITHOUT_FENCES: &str = "before\n\n# inside one\n\ninside **two**\n\n- three\n\nafter";

#[test]
fn the_blocks_between_fences_become_one_block() {
    let blocks = parsed(DOCUMENT, extensions());
    assert_eq!(blocks.len(), 3);
    let (name, children) = surface(&blocks[1]);
    assert_eq!(name.as_ref(), "summary");
    assert_eq!(children.len(), 3);
    let span = blocks[1].span().unwrap();
    assert!(DOCUMENT[span.start..span.end].starts_with("<!-- surface:summary -->"));
    assert!(DOCUMENT[span.start..span.end].ends_with("<!-- /surface -->"));
}

#[test]
fn fences_need_no_blank_lines() {
    let blocks = parsed(
        "<!-- surface:summary -->\ninside\n<!-- /surface -->\nafter",
        extensions(),
    );
    assert_eq!(blocks.len(), 2);
    assert_eq!(surface(&blocks[0]).1.len(), 1);
}

#[test]
fn without_a_registration_nothing_is_gathered() {
    // Exactly what a comment has always parsed to.
    let plain = parsed(DOCUMENT, MarkdownExtensions::default());
    assert_eq!(plain.len(), 7);
    assert!(
        !plain
            .iter()
            .any(|block| matches!(block, BlockNode::Surface { .. }))
    );
    // Only a registered name opens one, and a closing fence with nothing open
    // is a comment.
    let other = MarkdownExtensions::default().block_surface("other", |_, _| div());
    assert_eq!(parsed(DOCUMENT, other), plain);
}

#[test]
fn a_document_without_fences_parses_the_same_with_or_without_a_registration() {
    assert_eq!(
        parsed(WITHOUT_FENCES, extensions()),
        parsed(WITHOUT_FENCES, MarkdownExtensions::default())
    );
}

#[test]
fn fences_are_only_for_the_top_level() {
    let source = "> <!-- surface:summary -->\n>\n> quoted\n\nafter";
    let blocks = parsed(source, extensions());
    assert_eq!(blocks.len(), 2);
    assert!(matches!(blocks[0], BlockNode::Blockquote { .. }));
}

#[test]
fn the_end_of_the_text_or_another_fence_ends_a_surface() {
    let blocks = parsed(
        "<!-- surface:summary -->\n\none\n\n<!-- surface:summary -->\n\ntwo",
        extensions(),
    );
    assert_eq!(blocks.len(), 2);
    assert_eq!(surface(&blocks[0]).1.len(), 1);
    assert_eq!(surface(&blocks[1]).1.len(), 1);
}

#[test]
fn registered_surfaces_are_part_of_the_parser_configuration() {
    let plain = MarkdownExtensions::default().parser_revision(1);
    assert!(!plain.has_same_parser_configuration(&extensions()));
    // Rebuilt during rendering, as a view's extensions commonly are.
    assert!(extensions().has_same_parser_configuration(&extensions()));
}

struct Root {
    view: Entity<TextViewState>,
    extensions: MarkdownExtensions,
    width: Pixels,
    scrollable: bool,
}

impl Render for Root {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(self.width)
            .h(px(600.))
            .text_size(px(16.))
            .child(crate::TextSelectionLayer)
            .child(
                TextView::new(&self.view)
                    .scrollable(self.scrollable)
                    .selection_format(SelectionFormat::Source)
                    .markdown_extensions(self.extensions.clone()),
            )
    }
}

fn settle(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

fn window<'a>(
    source: &str,
    extensions: MarkdownExtensions,
    scrollable: bool,
    cx: &'a mut TestAppContext,
) -> (Entity<Root>, &'a mut VisualTestContext) {
    cx.update(crate::init);
    let (root, cx) = cx.add_window_view(|_, cx| Root {
        view: cx.new(|cx| TextViewState::markdown(source, cx)),
        extensions,
        width: px(500.),
        scrollable,
    });
    let cx: &mut VisualTestContext = cx;
    settle(cx);
    (root, cx)
}

fn view(root: &Entity<Root>, cx: &mut VisualTestContext) -> Entity<TextViewState> {
    root.read_with(cx, |root, _| root.view.clone())
}

#[gpui::test]
fn the_rendered_text_does_not_change(cx: &mut TestAppContext) {
    let (with, cx) = window(DOCUMENT, extensions(), false, cx);
    let view = view(&with, cx);
    let (rendered, plain) = view.read_with(cx, |state, _| {
        (
            state.rendered_text().as_str().to_string(),
            state.parsed_content.document.text(),
        )
    });
    let mut plain_cx = NodeContext::default();
    let fenceless = markdown::parse(WITHOUT_FENCES, &mut plain_cx).unwrap();
    assert_eq!(rendered, fenceless.text());
    assert_eq!(rendered, plain);
    assert!(!rendered.contains("surface"));
}

#[gpui::test]
fn a_select_all_copies_the_source_without_fences(cx: &mut TestAppContext) {
    let (root, cx) = window(DOCUMENT, extensions(), false, cx);
    let view = view(&root, cx);
    settle(cx);
    let copied = view.update(cx, |state, cx| {
        state.select_all(cx);
        state.selected_text()
    });
    assert!(!copied.contains("surface"), "{copied}");
    let squeezed: Vec<_> = copied
        .split("\n\n")
        .filter(|part| !part.is_empty())
        .collect();
    assert_eq!(
        squeezed,
        [
            "before",
            "# inside one",
            "inside **two**",
            "- three",
            "after"
        ]
    );
}

#[gpui::test]
fn a_stream_builds_the_same_surface_a_whole_parse_does(cx: &mut TestAppContext) {
    let (root, cx) = window("before", extensions(), true, cx);
    let view = view(&root, cx);
    for chunk in [
        "\n\n<!-- surface:summary -->",
        "\n\n# inside one\n\ninside **two",
        "**\n\n- three",
        "\n\n<!-- /surface -->",
        "\n\nafter",
    ] {
        view.update(cx, |state, cx| state.push_str(chunk, cx));
        settle(cx);
    }
    view.read_with(cx, |state, _| {
        assert_eq!(state.parsed_content.document.source.as_ref(), DOCUMENT);
        assert_eq!(
            state.parsed_content.document.blocks.as_ref(),
            &parsed(DOCUMENT, extensions())
        );
        assert_eq!(state.list_state.item_count(), 3);
    });
}

#[gpui::test]
fn an_empty_surface_draws_nothing(cx: &mut TestAppContext) {
    let (root, cx) = window(
        "<!-- surface:summary -->\n\n<!-- /surface -->\n\nafter",
        extensions(),
        true,
        cx,
    );
    let view = view(&root, cx);
    settle(cx);
    view.read_with(cx, |state, _| {
        let surface = state.list_state.bounds_for_item(0).unwrap();
        assert_eq!(surface.size.height, px(0.));
    });
}

#[gpui::test]
fn a_new_container_needs_no_new_parse(cx: &mut TestAppContext) {
    let renders = Arc::new(AtomicUsize::new(0));
    let counted = |renders: Arc<AtomicUsize>| {
        MarkdownExtensions::default()
            .parser_revision(1)
            .block_surface("summary", move |_, _| {
                renders.fetch_add(1, Ordering::Relaxed);
                div()
            })
    };
    let (root, cx) = window(DOCUMENT, counted(renders.clone()), false, cx);
    let view = view(&root, cx);
    let document = view.read_with(cx, |state, _| state.parsed_content.document.blocks.clone());
    let before = renders.load(Ordering::Relaxed);
    assert!(before > 0);
    root.update(cx, |root, cx| {
        // As a theme change does: a rebuilt registration with the same name.
        root.extensions = counted(renders.clone());
        cx.notify();
    });
    settle(cx);
    assert!(renders.load(Ordering::Relaxed) > before);
    view.read_with(cx, |state, _| {
        assert!(Arc::ptr_eq(
            &document,
            &state.parsed_content.document.blocks
        ));
    });
}
