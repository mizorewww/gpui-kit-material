//! FAB playground: md3.io's FAB configuration (color, size, position,
//! lowered elevation, icon, accessible label, disabled) on the real `Fab`,
//! floating at a corner of the preview like a FAB over a screen.
use super::prelude::*;

/// md3.io's color options, in order.
pub(super) const COLORS: [FabColor; 6] = [
    FabColor::PrimaryContainer,
    FabColor::SecondaryContainer,
    FabColor::TertiaryContainer,
    FabColor::Primary,
    FabColor::Secondary,
    FabColor::Tertiary,
];
pub(super) const COLOR_OPTIONS: &[&str] = &[
    "primary container",
    "secondary container",
    "tertiary container",
    "primary",
    "secondary",
    "tertiary",
];
pub(super) const POSITIONS: &[&str] = &[
    "center",
    "bottom right",
    "bottom left",
    "top right",
    "top left",
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Compose",
            values: &[
                ("icon", Set::Option("edit")),
                ("label", Set::Text("Compose")),
                ("position", Set::Option("bottom right")),
            ],
        },
        Scenario {
            name: "Create",
            values: &[
                ("color", Set::Option("tertiary container")),
                ("size", Set::Option("large")),
                ("label", Set::Text("Create")),
            ],
        },
        Scenario {
            name: "Add",
            values: &[
                ("color", Set::Option("primary")),
                ("size", Set::Option("medium")),
                ("position", Set::Option("bottom left")),
                ("lowered", Set::Bool(true)),
                ("label", Set::Text("Add")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Select {
                    id: "color",
                    label: "Color",
                    options: COLOR_OPTIONS,
                    default: 0,
                },
                Control::Select {
                    id: "size",
                    label: "Size",
                    options: &["default", "medium", "large"],
                    default: 0,
                },
                Control::Select {
                    id: "position",
                    label: "Position",
                    options: POSITIONS,
                    default: 0,
                },
                Control::Switch {
                    id: "lowered",
                    label: "Lowered elevation",
                    default: false,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Icon {
                    id: "icon",
                    label: "Icon",
                    icons: &["add", "edit", "favorite", "download", "send"],
                    default: 0,
                },
                Control::Text {
                    id: "label",
                    label: "Accessible label",
                    default: "Create new item",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[Control::Switch {
                id: "disabled",
                label: "Disabled",
                default: false,
            }],
        },
    ],
    preview,
};

/// The preview's screen: fills the canvas and floats `child` at
/// The FAB position for a [`POSITIONS`] option. "right" is the end and "left"
/// the start, in md3.io's left-to-right wording.
pub(super) fn position(option: &str) -> FabPosition {
    match option {
        "bottom right" => FabPosition::BottomEnd,
        "bottom left" => FabPosition::BottomStart,
        "top right" => FabPosition::TopEnd,
        "top left" => FabPosition::TopStart,
        _ => FabPosition::Center,
    }
}

/// The positioned area a preview FAB places itself in.
pub(super) fn stage(fab: impl IntoElement) -> Div {
    div()
        .debug_selector(|| "playground-stage".into())
        .relative()
        .w_full()
        .flex_1()
        .min_h(rem(160.))
        .child(fab)
}

fn preview(
    config: &Config,
    _: &mut Preview,
    _: &mut Window,
    _: &mut Context<PlaygroundState>,
) -> AnyElement {
    let icon = config.option("icon");
    let label = config.text("label");
    let label = if label.is_empty() {
        SharedString::new_static(icon)
    } else {
        label
    };
    let fab = Fab::new("playground-fab", symbol(icon, 24.), label)
        .color(COLORS[config.choice("color")])
        .size([FabSize::Regular, FabSize::Medium, FabSize::Large][config.choice("size")])
        .lowered(config.bool("lowered"))
        .position(position(config.option("position")))
        .debug_name("playground-fab")
        .disabled(config.bool("disabled"));
    stage(fab).into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::harness::open;
    use super::*;
    use core::prelude::v1::test;

    #[gpui::test]
    fn every_scenario_renders(cx: &mut TestAppContext) {
        open(&PLAYGROUND, cx).renders_every_scenario("playground-fab");
    }

    #[gpui::test]
    fn size_and_position_controls_move_the_fab(cx: &mut TestAppContext) {
        let mut h = open(&PLAYGROUND, cx);
        let stage = h.bounds("playground-stage");
        let fab = h.bounds("playground-fab");
        assert_eq!(fab.size, size(px(56.), px(56.)));
        assert!((fab.center().x - stage.center().x).abs() < px(1.));
        h.set("position", Value::Choice(1));
        let fab = h.bounds("playground-fab");
        assert_eq!(fab.right(), stage.right() - px(16.));
        assert_eq!(fab.bottom(), stage.bottom() - px(16.));
        h.set("size", Value::Choice(2));
        assert_eq!(h.bounds("playground-fab").size, size(px(96.), px(96.)));
    }
}
