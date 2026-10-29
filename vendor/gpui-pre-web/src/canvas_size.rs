//! Keep CSS layout dimensions independent of the canvas backing-store pixels.
//! Device-pixel observer boxes can disagree with emulated DPR in Chromium.
//! Preserve precise native pixel rounding when consistent; otherwise use CSS × DPR.
pub(crate) fn canvas_dimensions(
    css_width: f64,
    css_height: f64,
    dpr: f64,
    device_box: Option<(f64, f64)>,
) -> (u32, u32, f32, f32) {
    let expected = (css_width * dpr, css_height * dpr);
    let compatible = |(width, height): (f64, f64)| {
        width.is_finite()
            && height.is_finite()
            && width.fract() == 0.
            && height.fract() == 0.
            && width >= 0.
            && height >= 0.
            && (width - expected.0).abs() <= 1.
            && (height - expected.1).abs() <= 1.
            && (css_width != 0. || width == 0.)
            && (css_height != 0. || height == 0.)
    };
    // ResizeObserverSize uses inline/block axes, whereas contentRect uses x/y.
    let physical = device_box
        .and_then(|size| {
            if compatible(size) {
                Some(size)
            } else if compatible((size.1, size.0)) {
                Some((size.1, size.0))
            } else {
                None
            }
        })
        .unwrap_or((expected.0.round(), expected.1.round()));
    (
        physical.0 as u32,
        physical.1 as u32,
        css_width as f32,
        css_height as f32,
    )
}

#[cfg(test)]
mod tests {
    use super::canvas_dimensions;
    #[test]
    fn emulated_dpr_does_not_halve_the_logical_viewport() {
        assert_eq!(
            canvas_dimensions(960., 720., 2., Some((960., 720.))),
            (1920, 1440, 960., 720.)
        );
        assert_eq!(
            canvas_dimensions(960., 720., 1.25, Some((960., 720.))),
            (1200, 900, 960., 720.)
        );
    }
    #[test]
    fn accurate_native_rounding_does_not_move_css_layout() {
        assert_eq!(
            canvas_dimensions(100.4, 80.4, 1.25, Some((125., 101.))),
            (125, 101, 100.4, 80.4)
        );
        assert_eq!(
            canvas_dimensions(100.4, 80.4, 1.25, None),
            (126, 101, 100.4, 80.4)
        );
        assert_eq!(
            canvas_dimensions(100.4, 80.4, 1.25, Some((101., 125.))),
            (125, 101, 100.4, 80.4)
        );
    }
    #[test]
    fn missing_or_invalid_boxes_and_hidden_canvas_use_css_size() {
        for device in [
            None,
            Some((f64::NAN, 30.)),
            Some((-1., 30.)),
            Some((999., 999.)),
        ] {
            assert_eq!(canvas_dimensions(20., 30., 2., device), (40, 60, 20., 30.));
        }
        assert_eq!(
            canvas_dimensions(0., 0., 2., Some((1., 1.))),
            (0, 0, 0., 0.)
        );
    }
}
