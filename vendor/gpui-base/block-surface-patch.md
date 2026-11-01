# Block surfaces in `TextView`

Source: the vendored `gpui-base` (`SOURCE.json`).
Files: `src/text/` (`markdown_ext.rs`, `format/markdown.rs`, `node.rs`, `document.rs`,
`state.rs`, `range_highlight.rs`, `stream_fade.rs`, `mod.rs`).
Tests: `crates/material/tests/block_surface.rs`; `src/text/block_surface_tests.rs`, plus
one test each in `node.rs` and `state.rs`, reach private state.

Block quotes were the only block with a style, and a style per view. An application
showing a note as one document, for selection and find across it, could not give one
run of blocks a panel of its own. A surface above the document or a nested `TextView`
breaks that selection and find.

`MarkdownExtensions::block_surface(name, render)` registers a surface. `render` returns
a `Div` with its background, shape, padding, text style and any leading children, read
from the theme as it renders. At the top level of a Markdown document, the blocks
between `<!-- surface:name -->` and `<!-- /surface -->` become a `BlockNode::Surface`,
appended to that `Div`.

A surface is presentation. Every walker passes through it as through a list, so
`rendered_text` is the same as without the fences and no range moves. Its fences are
not copied, in any format. Its `span` starts at the opening fence, so the stream
parser, which parses the last block again, sees the fence again.

Without a registration the parser does not look for fences and a fence is a comment,
as before. The cost is one `is_empty` per parse.

`crates/gallery`'s Typography page shows one, in the current palette.
