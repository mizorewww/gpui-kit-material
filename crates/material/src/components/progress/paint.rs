//! Painting primitives shared by the progress indicators: full-round bars,
//! dots and stroked centerlines with round caps added by the caller.
use gpui::*;

/// A full-round horizontal bar from `x0` to `x1`, `thickness` tall,
/// centered on `cy`.
pub(super) fn rounded_bar(
    window: &mut Window,
    x0: f32,
    x1: f32,
    cy: f32,
    thickness: f32,
    ink: Rgba,
) {
    if x1 - x0 <= 0. {
        return;
    }
    let height = thickness;
    let bounds = Bounds::new(
        point(px(x0), px(cy - height / 2.)),
        size(px(x1 - x0), px(height)),
    );
    let radius = px((height / 2.).min((x1 - x0) / 2.));
    window.paint_quad(fill(bounds, ink).corner_radii(radius));
}

/// A filled circle `diameter` across.
pub(super) fn dot(window: &mut Window, center: Point<Pixels>, diameter: f32, ink: Rgba) {
    if diameter <= 0. {
        return;
    }
    let bounds = Bounds::centered_at(center, size(px(diameter), px(diameter)));
    window.paint_quad(fill(bounds, ink).corner_radii(px(diameter / 2.)));
}

/// Stroke an open polyline `width` wide (butt ends; callers add round caps
/// with [`dot`]).
pub(super) fn stroke_polyline(
    window: &mut Window,
    points: &[Point<Pixels>],
    width: f32,
    ink: Rgba,
) {
    let Some((first, rest)) = points.split_first() else {
        return;
    };
    if rest.is_empty() {
        return;
    }
    let mut path = PathBuilder::stroke(px(width));
    path.move_to(*first);
    for point in rest {
        path.line_to(*point);
    }
    if let Ok(path) = path.build() {
        window.paint_path(path, ink);
    }
}
