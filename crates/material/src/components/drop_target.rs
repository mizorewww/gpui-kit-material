//! A region that takes dropped files: a dashed outline around the caller's
//! content (an upload icon, a label, a button to choose a file instead),
//! which turns to the primary role under a state layer while files are
//! dragged over it.
//!
//! M3 has no drop zone component. The look composes what it does define:
//! the outline role for an edge that separates without emphasis (M3 color
//! roles, "Outline"), the primary role for the one active element, and a
//! primary state layer at the 8% hover opacity for an element a pointer is
//! over, which is what a drag over a target is (M3 states; Figma kit State
//! Layers variables, `state::HOVER`). Material defines no dashed stroke;
//! the dash is the convention desktop and web upload regions use to mark a
//! place to drop rather than a container, and both it and its 2dp width are
//! the kit's choice. GPUI's dash pattern scales with the stroke, a dash of
//! twice its width and a gap of one, so the 2dp stroke draws 4dp dashes 2dp
//! apart; at M3's 1dp outline width they would read as a dotted hairline.
//! Corners are the medium shape.
//!
//! The drag-over look follows GPUI's own drag state for external files
//! (`drag_over::<ExternalPaths>`), which GPUI clears when the files leave
//! the window, the drag is cancelled or they are dropped, so it cannot be
//! left behind. Content that changes with it uses
//! `group_drag_over::<ExternalPaths>(DROP_TARGET_GROUP, ..)`.
//!
//! A drop target is never the only way in: dragging needs a pointer, so
//! give it a button that opens a file picker too. The caller handles the
//! drop (`on_drop::<ExternalPaths>`) and names the target; components ship
//! no strings.
use crate::color::ColorRole::{self, *};
use crate::theme::color;
use crate::tokens::{shape::RADIUS_MEDIUM, state};
use gpui::{prelude::*, *};

/// The width of a drop target's dashed outline.
pub const DROP_TARGET_STROKE: Pixels = px(2.);

/// The hover group a drop target's content follows with
/// `group_drag_over::<ExternalPaths>`; the innermost target answers.
pub const DROP_TARGET_GROUP: &str = "material-drop-target";

/// The page's own ground: a drop target paints nothing at rest.
const CLEAR: Rgba = Rgba {
    r: 0.,
    g: 0.,
    b: 0.,
    a: 0.,
};

/// A drop target's outline role and the opacity of the primary state layer
/// over it: the outline at rest, the primary role under a hover-opacity
/// layer while files are dragged over it.
fn drop_target_paint(active: bool) -> (ColorRole, f32) {
    if active {
        (PRIMARY, state::HOVER)
    } else {
        (OUTLINE, 0.)
    }
}

/// A region that accepts dropped files, named `label` for assistive
/// technology (a group). It shows the dragged-over look while files are
/// dragged over it; `active` forces that look, for a target shown only
/// during a drag (a window-wide overlay) or a specimen. It lays its
/// children out in a centered column on a transparent ground (the page's),
/// with the medium shape; the caller's size, padding, gap and corners
/// refine it.
pub fn drop_target(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    active: bool,
) -> Stateful<Div> {
    let (rest, _) = drop_target_paint(active);
    let (over, layer) = drop_target_paint(true);
    let over_fill = state::layer(CLEAR, color(PRIMARY), layer);
    let over_edge = color(over);
    div()
        .id(id)
        .group(DROP_TARGET_GROUP)
        .role(Role::Group)
        .aria_label(label.into())
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .rounded(RADIUS_MEDIUM)
        .border(DROP_TARGET_STROKE)
        .border_dashed()
        .border_color(color(rest))
        .when(active, |target| target.bg(over_fill))
        .drag_over::<ExternalPaths>(move |style, _, _, _| {
            style.border_color(over_edge).bg(over_fill)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::{Context, TestAppContext, Window};

    #[test]
    fn the_outline_turns_primary_under_a_layer_while_dragged_over() {
        assert_eq!(drop_target_paint(false), (OUTLINE, 0.));
        assert_eq!(drop_target_paint(true), (PRIMARY, state::HOVER));
        assert_eq!(state::HOVER, 0.08);
        assert_eq!(DROP_TARGET_STROKE, px(2.));
    }

    struct Target(bool);
    impl Render for Target {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .p(px(20.))
                .child(drop_target("drop", "Files", self.0).w(px(200.)).h(px(120.)))
        }
    }

    /// The target's edge style, width in dp, edge color and fill.
    fn look(cx: &mut VisualTestContext) -> (BorderStyle, f32, Hsla, Background) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let scale = window.scale_factor();
            let quads = window.painted_quads();
            let target = |q: &&Quad| q.bounds.size.width.0 == 200. * scale;
            // The engine may paint the fill and the edge as separate quads.
            let edge = quads
                .iter()
                .filter(target)
                .find(|q| q.border_widths.top.0 > 0.)
                .expect("the target's edge");
            let fill = quads
                .iter()
                .filter(target)
                .map(|q| q.background)
                .find(|b| !b.is_transparent())
                .unwrap_or_default();
            (
                edge.border_style,
                edge.border_widths.top.0 / scale,
                edge.border_color,
                fill,
            )
        })
    }

    fn over_fill() -> Background {
        solid_background(state::layer(CLEAR, color(PRIMARY), state::HOVER))
    }

    #[gpui::test]
    fn files_dragged_over_and_away_light_and_clear_the_target(cx: &mut TestAppContext) {
        let (_, cx) = cx.add_window_view(|_, _| Target(false));
        cx.run_until_parked();
        let (style, width, ink, fill) = look(cx);
        assert_eq!((style, width), (BorderStyle::Dashed, 2.));
        assert_eq!(ink, Hsla::from(color(OUTLINE)));
        assert!(fill.is_transparent(), "at rest the page shows through");

        let paths = ExternalPaths(vec![std::path::PathBuf::from("/tmp/a.txt")].into());
        cx.simulate_event(FileDropEvent::Entered {
            position: point(px(100.), px(80.)),
            paths,
        });
        let (style, width, ink, fill) = look(cx);
        assert_eq!((style, width), (BorderStyle::Dashed, 2.));
        assert_eq!(ink, Hsla::from(color(PRIMARY)), "files over it");
        assert_eq!(fill, over_fill());

        // The files leave the window: GPUI ends the drag and the look with it.
        cx.simulate_event(FileDropEvent::Exited);
        let (_, _, ink, fill) = look(cx);
        assert_eq!(ink, Hsla::from(color(OUTLINE)), "files gone");
        assert!(fill.is_transparent());
    }

    #[gpui::test]
    fn a_forced_target_shows_the_dragged_over_look(cx: &mut TestAppContext) {
        let (_, cx) = cx.add_window_view(|_, _| Target(true));
        cx.run_until_parked();
        let (style, width, ink, fill) = look(cx);
        assert_eq!((style, width), (BorderStyle::Dashed, 2.));
        assert_eq!(ink, Hsla::from(color(PRIMARY)));
        assert_eq!(fill, over_fill());
    }
}
