# Width change on a `measure_all` list

Source: the published Apache-2.0 `gpui-pre` 0.3.12 package (`SOURCE.json`), the
same in `vendor/gpui-fast`. File: `src/elements/list.rs`.

**Why.** On a width change, upstream's `List::prepaint` drops every item's height
and re-arms `measure_all`, so `layout_all_items` lays out every item in that
frame. Kit's `TextView` lists its blocks with `measure_all`, and a reader resized
over a long document re-wrapped all of it in every frame the width changed:
p90 51 ms per frame for 79k characters, 190–250 ms for 317k (course2md #42).

**What.** A list that measures all its items and had measured them keeps each
item's height as an estimate, and in the frame with the new width lays out only
the items it shows plus its overdraw, as a list of visible items does. The
estimates of the others are scaled by how much the items measured this frame grew
or shrank together, so the list's height, and the scrollbar, follow the reflow
(wrapped text grows as the width shrinks; fixed-height rows do not, which the
width alone cannot tell). Later frames measure the rest in order, 4 ms of layout
per frame and at least one item, requesting a frame until none is left, then one
more. The item at the scroll top keeps the same fraction of itself above the
viewport, as `remeasure` keeps it.

A frame budget rather than one full measurement once the width has settled: every
frame stays bounded, the heights converge without a long frame after the resize,
and a list is exact within frames of its last width change. The scroll position is
relative to the item at its top, so measuring the items above it moves the
scrollbar, not the content; `scroll_to` lands on its item at once.
`scroll_to_reveal_item` of an item below the viewport would otherwise place it from
estimated heights and leave it there (a reveal that stops once the item is partly
shown, as `TextView`'s, never corrects it): while a deferred measurement is under
way it is kept until the next prepaint, which lays out the item and those before it,
back from it, until they fill the viewport, and puts the item at the bottom edge
from those exact heights.

Since 0.3.12 (and gpui-fast 0.1.14) upstream lays items out inside the list's padding
and border, and forgets heights when that inner width changes; the patch keys on the
same inner width and measures deferred and revealed items at it.

Unchanged: the first measurement, `reset`, `remeasure` and `remeasure_items` still
measure every item in one frame (a full measurement due also ends a deferred
one), and a list of visible items still forgets its heights on a width change.
Measurements that are not ready yet are retried by a full measurement, as before.

Tests: `test_width_change_measures_only_around_the_viewport_first`,
`test_width_change_converges_to_exact_heights`,
`test_width_change_keeps_the_scroll_anchor`,
`test_scroll_to_item_after_width_change`,
`test_remeasure_during_deferred_measurement_measures_all`,
`test_reveal_into_estimated_heights_after_width_change`,
`test_width_change_twice_before_convergence`,
`test_splice_during_deferred_measurement`,
`test_width_change_on_bottom_aligned_list`,
`test_width_change_to_one_pixel_and_back`,
`test_width_change_to_zero_with_fixed_rows`,
`test_deferred_measurement_ends_with_unready_items`,
`test_wheel_during_deferred_measurement`, in both engines.

Behavior only; no API is added. Consumers need this patch for long `TextView`s to
resize smoothly: `docs/ARCHITECTURE.md`, Consumer patch set. Not yet offered
upstream.
