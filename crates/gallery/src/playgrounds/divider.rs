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
            title: "Content",
            controls: &[Control::Text {
                id: "subhead",
                label: "Subhead",
                default: "",
            }],
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
    code,
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
    let subhead = config.text("subhead");
    let divider = if subhead.is_empty() {
        divider
    } else {
        divider.subhead(subhead)
    };
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

fn code(config: &Config) -> String {
    const VARIANT_NAMES: [&str; 3] = ["FullWidth", "Inset", "MiddleInset"];
    const COLOR_NAMES: [&str; 4] = ["OUTLINE_VARIANT", "OUTLINE", "PRIMARY", "SECONDARY"];
    let variant = config.choice("variant");
    let (thickness, start, end) = (
        config.number("thickness"),
        config.number("start"),
        config.number("end"),
    );
    // Insets apply to the inset variants, which start 16dp in and end 0dp
    // (inset) or 16dp (middle inset) in by default.
    let default_end = if variant == 2 { 16. } else { 0. };
    let subhead = config.text("subhead");
    let color = config.choice("color");
    chain(
        "MaterialDivider::new(\"divider\")",
        [
            (config.option("orientation") == "vertical")
                .then(|| "orientation(DividerOrientation::Vertical)".into()),
            (variant != 0).then(|| format!("variant(DividerVariant::{})", VARIANT_NAMES[variant])),
            (thickness != 1.).then(|| format!("thickness(px({thickness:?}))")),
            (color != 0).then(|| format!("color_role(ColorRole::{})", COLOR_NAMES[color])),
            (variant != 0 && start != 16.).then(|| format!("inset_start(rem({start:?}))")),
            (variant != 0 && end != default_end).then(|| format!("inset_end(rem({end:?}))")),
            (!subhead.is_empty()).then(|| format!("subhead({})", lit(&subhead))),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::super::super::playground::{Config, Value};
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

    #[test]
    fn the_code_follows_the_configuration() {
        let mut config = Config::defaults(&PLAYGROUND);
        assert_eq!(super::code(&config), "MaterialDivider::new(\"divider\")");
        config.set("variant", Value::Choice(1));
        let code = super::code(&config);
        assert!(code.contains(".variant(DividerVariant::Inset)"), "{code}");
        assert!(code.contains(".inset_end(rem(16.0))"), "{code}");
        assert!(!code.contains(".inset_start("));
    }
}
