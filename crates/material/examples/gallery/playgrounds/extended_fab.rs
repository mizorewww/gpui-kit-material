//! Extended FAB playground: md3.io's Extended FAB configuration (color,
//! size, width, position, lowered elevation, icon, text, icon position,
//! collapsed, disabled) on the real `ExtendedFab`. Activating the preview
//! collapses or expands it, as scrolling would in an app.
use super::fab::{COLOR_OPTIONS, COLORS, POSITIONS, stage};
use super::prelude::*;

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Compose",
            values: &[("position", Set::Option("bottom right"))],
        },
        Scenario {
            name: "Check out",
            values: &[
                ("color", Set::Option("primary")),
                ("icon", Set::Option("shopping_cart")),
                ("text", Set::Text("Check out")),
            ],
        },
        Scenario {
            name: "Publish",
            values: &[
                ("color", Set::Option("tertiary")),
                ("icon", Set::Option("arrow_upward")),
                ("text", Set::Text("Publish")),
                ("icon_position", Set::Option("end")),
            ],
        },
        Scenario {
            name: "New task",
            values: &[
                ("color", Set::Option("secondary container")),
                ("size", Set::Option("medium")),
                ("icon", Set::Option("add")),
                ("text", Set::Text("New task")),
                ("position", Set::Option("bottom left")),
            ],
        },
        Scenario {
            name: "Find flights",
            values: &[
                ("color", Set::Option("tertiary container")),
                ("size", Set::Option("large")),
                ("width", Set::Option("fluid")),
                ("icon", Set::Option("flight")),
                ("text", Set::Text("Find flights")),
                ("position", Set::Option("bottom right")),
            ],
        },
        Scenario {
            name: "Save draft",
            values: &[
                ("color", Set::Option("secondary")),
                ("icon", Set::Option("")),
                ("text", Set::Text("Save draft")),
                ("lowered", Set::Bool(true)),
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
                Control::Segmented {
                    id: "size",
                    label: "Size",
                    options: &["small", "medium", "large"],
                    default: 0,
                },
                Control::Segmented {
                    id: "width",
                    label: "Width",
                    options: &["fixed", "fluid"],
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
                    icons: &[
                        "add",
                        "edit",
                        "favorite",
                        "download",
                        "send",
                        "shopping_cart",
                        "arrow_upward",
                        "flight",
                        "",
                    ],
                    default: 1,
                },
                Control::Text {
                    id: "text",
                    label: "Text",
                    default: "Compose",
                },
                Control::Segmented {
                    id: "icon_position",
                    label: "Icon position",
                    options: &["start", "end"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "collapsed",
                    label: "Collapsed",
                    default: false,
                },
                Control::Switch {
                    id: "disabled",
                    label: "Disabled",
                    default: false,
                },
            ],
        },
    ],
    preview,
};

fn preview(
    config: &Config,
    preview: &mut Preview,
    _: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let icon = config.option("icon");
    let text = config.text("text");
    let id = "playground-extended-fab";
    let fab = if icon.is_empty() {
        ExtendedFab::label_only(id, text)
    } else {
        ExtendedFab::new(id, symbol(icon, 24.), text)
    };
    let collapsed = config.bool("collapsed");
    let collapse = preview.on_bool("collapsed", cx);
    let fluid = config.option("width") == "fluid";
    let fab = fab
        .color(COLORS[config.choice("color")])
        .size([FabSize::Regular, FabSize::Medium, FabSize::Large][config.choice("size")])
        .fluid(fluid)
        .lowered(config.bool("lowered"))
        .icon_position(if config.option("icon_position") == "end" {
            ButtonIconPosition::Trailing
        } else {
            ButtonIconPosition::Leading
        })
        .expanded(!collapsed)
        .disabled(config.bool("disabled"))
        .on_click(move |_, window, cx| {
            if !icon.is_empty() {
                collapse(&!collapsed, window, cx)
            }
        });
    stage(
        config.option("position"),
        div()
            .flex()
            .when(fluid, |slot| slot.w_full())
            .debug_selector(|| "playground-extended-fab".into())
            .child(fab),
    )
    .debug_selector(|| "playground-stage".into())
    .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::test_support::open;
    use super::*;
    use core::prelude::v1::test;

    #[gpui::test]
    fn every_scenario_renders(cx: &mut TestAppContext) {
        open(&PLAYGROUND, cx).renders_every_scenario("playground-extended-fab");
    }

    #[gpui::test]
    fn clicking_collapses_and_expands_the_fab(cx: &mut TestAppContext) {
        let mut h = open(&PLAYGROUND, cx);
        let expanded = h.bounds("playground-extended-fab").size;
        assert_eq!(expanded.height, px(56.));
        assert!(expanded.width > px(100.), "{expanded:?}");
        h.click_on("playground-extended-fab");
        assert!(h.config().bool("collapsed"));
        assert_eq!(
            h.bounds("playground-extended-fab").size,
            size(px(56.), px(56.))
        );
        h.click_on("playground-extended-fab");
        assert!(!h.config().bool("collapsed"));
        assert_eq!(h.bounds("playground-extended-fab").size, expanded);
    }

    #[gpui::test]
    fn width_and_icon_controls_change_the_fab(cx: &mut TestAppContext) {
        let mut h = open(&PLAYGROUND, cx);
        let stage = h.bounds("playground-stage");
        h.set("width", Value::Choice(1));
        assert_eq!(
            h.bounds("playground-extended-fab").size.width,
            stage.size.width
        );
        h.set("width", Value::Choice(0));
        // Without an icon the FAB cannot collapse.
        h.set("icon", Value::Choice(8));
        h.set("collapsed", Value::Bool(true));
        assert!(h.bounds("playground-extended-fab").size.width > px(56.));
    }
}
