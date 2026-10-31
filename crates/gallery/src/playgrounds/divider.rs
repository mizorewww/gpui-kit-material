//! Divider playground: md3.io's Divider configuration (variant,
//! orientation, thickness, color, start and end insets) on the real
//! `MaterialDivider`, drawn in a 400 x 200 frame like md3.io's preview.
use super::prelude::*;

const VARIANTS: [DividerVariant; 3] = [
    DividerVariant::FullWidth,
    DividerVariant::Inset,
    DividerVariant::MiddleInset,
];
const COLORS: [ColorRole; 4] = [
    ColorRole::OUTLINE_VARIANT,
    ColorRole::OUTLINE,
    ColorRole::PRIMARY,
    ColorRole::SECONDARY,
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Select {
                    id: "variant",
                    label: "Variant",
                    options: &["full width", "inset", "middle inset"],
                    default: 0,
                },
                Control::Segmented {
                    id: "orientation",
                    label: "Orientation",
                    options: &["horizontal", "vertical"],
                    default: 0,
                },
                Control::Slider {
                    id: "thickness",
                    label: "Thickness",
                    min: 1.,
                    max: 8.,
                    step: 1.,
                    default: 1.,
                },
                Control::Select {
                    id: "color",
                    label: "Color",
                    options: &["outline variant", "outline", "primary", "secondary"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Layout",
            controls: &[
                Control::Slider {
                    id: "start",
                    label: "Start inset",
                    min: 0.,
                    max: 64.,
                    step: 1.,
                    default: 16.,
                },
                Control::Slider {
                    id: "end",
                    label: "End inset",
                    min: 0.,
                    max: 64.,
                    step: 1.,
                    default: 16.,
                },
            ],
        },
    ],
    preview,
};

fn preview(
    config: &Config,
    _: &mut Preview,
    _: &mut Window,
    _: &mut Context<PlaygroundState>,
) -> AnyElement {
    let vertical = config.option("orientation") == "vertical";
    let divider = MaterialDivider::new("playground-divider")
        .orientation(if vertical {
            DividerOrientation::Vertical
        } else {
            DividerOrientation::Horizontal
        })
        .variant(VARIANTS[config.choice("variant")])
        .thickness(px(config.number("thickness")))
        .color_role(COLORS[config.choice("color")])
        .inset_start(rem(config.number("start")))
        .inset_end(rem(config.number("end")));
    // The line's box stretches along the frame's main axis and is as thick
    // as the line across it.
    let line = div()
        .debug_selector(|| "playground-divider".into())
        .flex()
        .items_stretch()
        .when(!vertical, |line| line.flex_col())
        .child(divider);
    div()
        .w_full()
        .max_w(rem(400.))
        .h(rem(200.))
        .flex()
        .items_stretch()
        .justify_center()
        .when(!vertical, |frame| frame.flex_col())
        .child(line)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::super::playground::Value;
    use super::super::nav_harness::Harness;
    use super::PLAYGROUND;
    use gpui::TestAppContext;

    #[gpui::test]
    fn every_scenario_renders_and_orientation_turns_the_frame(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.render_every_scenario(|harness, _| {
            harness.expect("playground-divider");
        });
        let horizontal = harness.expect("playground-divider");
        assert!(horizontal.size.width > horizontal.size.height);
        assert_eq!(horizontal.size.height, gpui::px(1.));
        harness.set("orientation", Value::Choice(1));
        harness.set("variant", Value::Choice(2));
        harness.set("thickness", Value::Number(4.));
        harness.set("color", Value::Choice(2));
        let vertical = harness.expect("playground-divider");
        assert!(vertical.size.height > vertical.size.width);
        assert_eq!(vertical.size.width, gpui::px(4.));
        assert_eq!(harness.config().option("color"), "primary");
    }
}
