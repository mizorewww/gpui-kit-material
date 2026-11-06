# An input writes nothing as it paints

Source: the vendored `gpui-base` (`SOURCE.json`).
Files: `src/input/base/element.rs`, doc comments in `src/input/base/state.rs`.
Tests: `drawing_an_input_leaves_the_view_around_it_clean` (GPUI Fast only) and the
Gallery's `window_tests::scrolling_the_content_renders_neither_the_gallery_nor_the_navigation`.
Upstream PR: the `material/input-paint-no-write` branch.

The input stored its layout, bounds and scroll state in its entity at the end of
paint, an update on every paint. Under GPUI Fast a view drawn again from the last
frame around a view built again is checked before the nested view paints, so that
update read as a change to what the outer view read, and the outer view was built
again on the next frame. In the Gallery a wheel over any page with an input built
the Gallery and its navigation again on the frame after every wheel event.

The same update now runs at the end of prepaint, where what a nested view writes is
part of drawing the view around it. The values stored, the comparison (here the one
of `input-move-patch.md`, which moved with the block) and the notification on changed
geometry are unchanged. It replaces a local Fast patch, dropped because it stays out
of upstream, that took what a nested view writes while painting for the spliced
view's own write.
