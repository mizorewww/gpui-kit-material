//! Badge playground: md3.io's Badge configuration (variant, color,
//! position, label, maximum count, visible) on the real `Badge`, marking
//! the icon of a tonal icon button as md3.io's preview does.
use super::prelude::*;

const COLORS: &[&str] = &[
    "error",
    "primary",
    "secondary",
    "tertiary",
    "success",
    "warning",
    "info",
];
const POSITIONS: &[&str] = &["top right", "top left", "bottom right", "bottom left"];
const MAX_COUNTS: &[&str] = &["9", "99", "999"];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "New content",
            values: &[("variant", Set::Option("small"))],
        },
        Scenario {
            name: "Overflowing count",
            values: &[("label", Set::Text("1250")), ("max", Set::Option("999"))],
        },
        Scenario {
            name: "Primary label",
            values: &[
                ("label", Set::Text("New")),
                ("color", Set::Option("primary")),
                ("position", Set::Option("top left")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Segmented {
                    id: "variant",
                    label: "Variant",
                    options: &["small", "large"],
                    default: 1,
                },
                Control::Select {
                    id: "color",
                    label: "Color",
                    options: COLORS,
                    default: 0,
                },
                Control::Select {
                    id: "position",
                    label: "Position",
                    options: POSITIONS,
                    default: 0,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Text {
                    id: "label",
                    label: "Label",
                    default: "8",
                },
                Control::Segmented {
                    id: "max",
                    label: "Maximum count",
                    options: MAX_COUNTS,
                    default: 1,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[Control::Switch {
                id: "visible",
                label: "Visible",
                default: true,
            }],
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
    let max: u32 = config.option("max").parse().unwrap_or(99);
    let label = config.text("label");
    let small = config.option("variant") == "small";
    let visible = config.bool("visible");
    // The button's name carries what the badge says.
    let name: SharedString = match (visible, small) {
        (false, _) => "Inbox".into(),
        (true, true) => "Inbox, new messages".into(),
        (true, false) => format!("Inbox, {}", badge_label(&label, Some(max))).into(),
    };
    let badge = if small {
        Badge::small("playground-badge")
    } else {
        Badge::large("playground-badge", label)
    };
    let button = IconButton::new("playground-badge-target", symbol("inbox", 24.), name)
        .variant(IconButtonVariant::Tonal);
    div()
        .debug_selector(|| "playground-badge".into())
        .child(
            badge
                .max_count(max)
                .color(BadgeColor::ALL[config.choice("color")])
                .position(BadgePosition::ALL[config.choice("position")])
                .visible(visible)
                // The 24dp icon sits 8dp inside the 40dp button.
                .inset(8.)
                .anchor(button),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use core::prelude::v1::test;

    #[gpui::test]
    fn every_scenario_and_option_renders(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        render_every_scenario(&state, &mut cx);
        render_every_option(&state, &mut cx, &["variant", "color", "position", "max"]);
        assert!(cx.debug_bounds("playground-badge").is_some());
    }

    #[gpui::test]
    fn the_visible_switch_hides_and_shows_the_badge(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        let shown = element_bounds(&mut cx, "playground-badge");
        toggle_switch(&mut cx, "visible");
        assert!(!config(&state, &mut cx).bool("visible"));
        settle_motion(&mut cx);
        // The badge overlays the button: hiding it never moves the button.
        assert_eq!(element_bounds(&mut cx, "playground-badge"), shown);
        toggle_switch(&mut cx, "visible");
        assert!(config(&state, &mut cx).bool("visible"));
    }
}
