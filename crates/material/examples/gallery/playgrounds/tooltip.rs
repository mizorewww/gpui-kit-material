//! Tooltip playground: md3.io's Tooltip configuration (variant, position,
//! target, text, visible, show on hover/focus, show and hide delays) on the
//! real `MaterialTooltip`. Hover or focus the target to show it.
//!
//! md3.io's `default` variant is the plain tooltip, as in M3. The Figma
//! kit's rich tooltip adds a subhead and an action, offered as the extra
//! Subhead and Action options; the action dismisses a pinned tooltip.
use super::prelude::*;
use gpui_base::{Align, Placement};
use std::time::Duration;

const POSITIONS: &[&str] = &[
    "top",
    "right",
    "bottom",
    "left",
    "top start",
    "top end",
    "right start",
    "right end",
    "bottom start",
    "bottom end",
    "left start",
    "left end",
];
const TARGETS: &[&str] = &["Favorite", "Upload", "Present now"];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Upload",
            values: &[
                ("target", Set::Option("Upload")),
                ("text", Set::Text("Upload files")),
                ("position", Set::Option("top")),
            ],
        },
        Scenario {
            name: "Present now",
            values: &[
                ("variant", Set::Option("rich")),
                ("target", Set::Option("Present now")),
                ("subhead", Set::Text("Present to everyone")),
                (
                    "text",
                    Set::Text("Share your screen, a window or a tab with the people in this call."),
                ),
                ("action", Set::Text("Learn more")),
                ("position", Set::Option("bottom start")),
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
                    options: &["default", "plain", "rich"],
                    default: 0,
                },
                Control::Select {
                    id: "position",
                    label: "Position",
                    options: POSITIONS,
                    default: 2,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "target",
                    label: "Target",
                    options: TARGETS,
                    default: 0,
                },
                Control::Text {
                    id: "text",
                    label: "Text",
                    default: "Save to favorites",
                },
                Control::Text {
                    id: "subhead",
                    label: "Subhead (rich)",
                    default: "Favorites",
                },
                Control::Text {
                    id: "action",
                    label: "Action (rich)",
                    default: "",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "visible",
                    label: "Visible",
                    default: false,
                },
                Control::Switch {
                    id: "hover",
                    label: "Show on hover",
                    default: true,
                },
                Control::Switch {
                    id: "focus",
                    label: "Show on focus",
                    default: true,
                },
                Control::Slider {
                    id: "show_delay",
                    label: "Show delay (ms)",
                    min: 0.,
                    max: 2000.,
                    step: 50.,
                    default: 300.,
                },
                Control::Slider {
                    id: "hide_delay",
                    label: "Hide delay (ms)",
                    min: 0.,
                    max: 2000.,
                    step: 50.,
                    default: 100.,
                },
            ],
        },
    ],
    preview,
};

/// A position option's side and alignment.
fn placement(position: &str) -> (Placement, Align) {
    let mut words = position.split(' ');
    let side = match words.next() {
        Some("top") => Placement::Top,
        Some("right") => Placement::Right,
        Some("left") => Placement::Left,
        _ => Placement::Bottom,
    };
    let align = match words.next() {
        Some("start") => Align::Start,
        Some("end") => Align::End,
        _ => Align::Center,
    };
    (side, align)
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    _: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let text = config.text("text");
    let target = config.option("target");
    let trigger = match target {
        "Present now" => MaterialButton::new("playground-tooltip-target")
            .variant(ButtonVariant::Filled)
            .icon(symbol("present_to_all", 20.))
            .child(target)
            .into_any_element(),
        _ => {
            let icon = if target == "Upload" {
                "upload"
            } else {
                "favorite"
            };
            IconButton::new("playground-tooltip-target", symbol(icon, 24.), target)
                .variant(IconButtonVariant::Tonal)
                .into_any_element()
        }
    };
    let trigger = div()
        .debug_selector(|| "playground-tooltip-target".into())
        .child(trigger);
    let mut tooltip = if config.option("variant") == "rich" {
        let mut rich = MaterialTooltip::rich("playground-tooltip", text);
        let subhead = config.text("subhead");
        if !subhead.is_empty() {
            rich = rich.subhead(subhead);
        }
        let action = config.text("action");
        if !action.is_empty() {
            let dismiss = preview.setter("visible", cx);
            rich = rich.action(
                MaterialButton::new("playground-tooltip-action")
                    .variant(ButtonVariant::Text)
                    .debug_selector(|| "playground-tooltip-action".into())
                    .child(action)
                    .on_click(move |_, window, cx| dismiss(Value::Bool(false), window, cx)),
            );
        }
        rich
    } else {
        MaterialTooltip::plain("playground-tooltip", text)
    };
    let (side, align) = placement(config.option("position"));
    let millis = |id| Duration::from_millis(config.number(id).max(0.) as u64);
    tooltip = tooltip
        .placement(side)
        .align(align)
        .show_on_hover(config.bool("hover"))
        .show_on_focus(config.bool("focus"))
        .show_delay(millis("show_delay"))
        .hide_delay(millis("hide_delay"));
    if config.bool("visible") {
        tooltip = tooltip.force_visible(true);
    }
    div()
        .debug_selector(|| "playground-tooltip".into())
        .child(tooltip.trigger(trigger))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn positions_map_to_a_side_and_alignment() {
        assert!(matches!(placement("top"), (Placement::Top, Align::Center)));
        assert!(matches!(
            placement("right start"),
            (Placement::Right, Align::Start)
        ));
        assert!(matches!(
            placement("left end"),
            (Placement::Left, Align::End)
        ));
        assert!(matches!(
            placement("bottom"),
            (Placement::Bottom, Align::Center)
        ));
    }

    #[gpui::test]
    fn every_scenario_and_position_renders(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        render_every_scenario(&state, &mut cx);
        set(&state, &mut cx, "visible", Value::Bool(true));
        render_every_option(&state, &mut cx, &["variant", "target"]);
        set(&state, &mut cx, "visible", Value::Bool(true));
        render_every_option(&state, &mut cx, &["position"]);
    }

    #[gpui::test]
    fn the_rich_action_dismisses_a_pinned_tooltip(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        cx.update(|window, cx| state.update(cx, |state, cx| state.select_scenario(2, window, cx)));
        set(&state, &mut cx, "visible", Value::Bool(true));
        settle_motion(&mut cx);
        let action = element_bounds(&mut cx, "playground-tooltip-action");
        click(&mut cx, action.center());
        assert!(!config(&state, &mut cx).bool("visible"));
    }

    #[gpui::test]
    fn hover_shows_the_tooltip_after_the_configured_delay(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        cx.update(|_, cx| cx.set_reduce_motion(true));
        set(&state, &mut cx, "variant", Value::Choice(2));
        set(&state, &mut cx, "action", Value::Text("Learn more".into()));
        set(&state, &mut cx, "show_delay", Value::Number(1000.));
        let target = element_bounds(&mut cx, "playground-tooltip-target");
        cx.simulate_mouse_move(target.center(), None, gpui::Modifiers::none());
        settle(&mut cx);
        cx.executor().advance_clock(Duration::from_millis(900));
        settle(&mut cx);
        assert!(
            cx.debug_bounds("playground-tooltip-action").is_none(),
            "not before the show delay"
        );
        cx.executor().advance_clock(Duration::from_millis(200));
        settle(&mut cx);
        assert!(
            cx.debug_bounds("playground-tooltip-action").is_some(),
            "shown after the show delay"
        );
    }
}
