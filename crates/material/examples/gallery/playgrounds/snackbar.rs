//! Snackbar playground: md3.io's Snackbar configuration (position,
//! message, action, dismiss label, duration, close button, visible) on the
//! real `Snackbar`, shown from a button at the bottom of the preview.
//!
//! As in m3e's snackbar host, `short` hides after 4s and `long` after 10s,
//! the timer pauses while the pointer is over the snackbar, and Escape, the
//! action and the close button dismiss it. The Figma kit's longer-action
//! layout is the extra "Action on new line" option.
use super::prelude::*;

const DURATIONS: &[&str] = &["short", "long", "indefinite"];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Email archived",
            values: &[
                ("message", Set::Text("Email archived")),
                ("action_text", Set::Text("Undo")),
                ("close", Set::Bool(false)),
                ("duration", Set::Option("short")),
                ("visible", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Saved to album",
            values: &[
                ("message", Set::Text("Saved to album")),
                ("action_text", Set::Text("View")),
                ("position", Set::Option("start")),
                ("duration", Set::Option("long")),
                ("visible", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "All changes saved",
            values: &[
                ("message", Set::Text("All changes saved")),
                ("action", Set::Bool(false)),
                ("close", Set::Bool(false)),
                ("duration", Set::Option("short")),
                ("visible", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Photo added",
            values: &[
                (
                    "message",
                    Set::Text(
                        "Photo added to your library. It can take a moment to appear in shared albums.",
                    ),
                ),
                ("action_text", Set::Text("Open album")),
                ("new_line", Set::Bool(true)),
                ("visible", Set::Bool(true)),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Layout",
            controls: &[
                Control::Segmented {
                    id: "position",
                    label: "Position",
                    options: &["start", "center", "end"],
                    default: 1,
                },
                Control::Switch {
                    id: "new_line",
                    label: "Action on new line",
                    default: false,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Text {
                    id: "message",
                    label: "Message",
                    default: "Your changes have been saved",
                },
                Control::Switch {
                    id: "action",
                    label: "Show action",
                    default: true,
                },
                Control::Text {
                    id: "action_text",
                    label: "Action text",
                    default: "Undo",
                },
                Control::Text {
                    id: "dismiss",
                    label: "Dismiss label",
                    default: "Dismiss",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Select {
                    id: "duration",
                    label: "Duration",
                    options: DURATIONS,
                    default: 2,
                },
                Control::Switch {
                    id: "close",
                    label: "Close button",
                    default: true,
                },
                Control::Switch {
                    id: "visible",
                    label: "Visible",
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
    let visible = config.bool("visible");
    let hide = preview.setter("visible", cx);
    let hide = std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
        hide(Value::Bool(false), window, cx)
    });
    let show = preview.setter("visible", cx);
    let mut snackbar = Snackbar::new("playground-snackbar", config.text("message"))
        .debug_name("playground-snackbar")
        .open(visible)
        .longer_action(config.bool("new_line"))
        .duration(match config.option("duration") {
            "short" => SnackbarDuration::Short,
            "long" => SnackbarDuration::Long,
            _ => SnackbarDuration::Indefinite,
        })
        .position(match config.option("position") {
            "start" => SnackbarPosition::Start,
            "end" => SnackbarPosition::End,
            _ => SnackbarPosition::Center,
        })
        .on_dismiss({
            let hide = hide.clone();
            move |window, cx| hide(window, cx)
        });
    if config.bool("action") {
        let hide = hide.clone();
        snackbar = snackbar.action(config.text("action_text"), move |_, window, cx| {
            hide(window, cx)
        });
    }
    if config.bool("close") {
        let hide = hide.clone();
        snackbar = snackbar.close(config.text("dismiss"), move |_, window, cx| {
            hide(window, cx)
        });
    }
    div()
        .relative()
        .size_full()
        .min_h(rem(320.))
        .flex()
        .items_center()
        .justify_center()
        .child(
            MaterialButton::new("playground-snackbar-show")
                .variant(ButtonVariant::Tonal)
                .debug_selector(|| "playground-snackbar-show".into())
                .child("Show snackbar")
                .on_click(move |_, window, cx| show(Value::Bool(true), window, cx)),
        )
        .child(snackbar)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use core::prelude::v1::test;
    use std::time::Duration;

    fn visible(state: &Entity<PlaygroundState>, cx: &mut gpui::VisualTestContext) -> bool {
        config(state, cx).bool("visible")
    }

    #[gpui::test]
    fn every_scenario_renders(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        render_every_scenario(&state, &mut cx);
        render_every_option(&state, &mut cx, &["position", "duration"]);
    }

    #[gpui::test]
    fn the_button_shows_it_and_the_action_and_close_dismiss_it(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        assert!(!visible(&state, &mut cx));
        let show = element_bounds(&mut cx, "playground-snackbar-show");
        click(&mut cx, show.center());
        assert!(visible(&state, &mut cx), "the button shows the snackbar");
        settle_motion(&mut cx);
        // The close button is the snackbar's trailing 48dp box.
        let snackbar = element_bounds(&mut cx, "playground-snackbar");
        let close = design(&mut cx, 24.);
        click(
            &mut cx,
            gpui::point(snackbar.right() - close, snackbar.center().y),
        );
        assert!(!visible(&state, &mut cx), "close dismisses");
        settle_motion(&mut cx);
        click(&mut cx, show.center());
        settle_motion(&mut cx);
        // The action sits before the close box.
        let snackbar = element_bounds(&mut cx, "playground-snackbar");
        let offset = design(&mut cx, 48. + 24.);
        click(
            &mut cx,
            gpui::point(snackbar.right() - offset, snackbar.center().y),
        );
        assert!(!visible(&state, &mut cx), "the action dismisses");
    }

    #[gpui::test]
    fn short_and_long_durations_hide_it_and_hover_pauses(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        set(&state, &mut cx, "duration", Value::Choice(0));
        set(&state, &mut cx, "visible", Value::Bool(true));
        cx.executor()
            .advance_clock(SnackbarDuration::SHORT - Duration::from_millis(100));
        draw(&mut cx);
        assert!(visible(&state, &mut cx), "still shown before 4s");
        cx.executor().advance_clock(Duration::from_millis(200));
        settle(&mut cx);
        assert!(!visible(&state, &mut cx), "short hides after 4s");

        set(&state, &mut cx, "duration", Value::Choice(1));
        set(&state, &mut cx, "visible", Value::Bool(true));
        settle_motion(&mut cx);
        // Hovering pauses the countdown.
        let snackbar = element_bounds(&mut cx, "playground-snackbar");
        cx.simulate_mouse_move(snackbar.center(), None, gpui::Modifiers::none());
        settle(&mut cx);
        cx.executor().advance_clock(SnackbarDuration::LONG * 2);
        settle(&mut cx);
        assert!(visible(&state, &mut cx), "hover pauses the timer");
        cx.simulate_mouse_move(
            gpui::point(gpui::px(1.), gpui::px(1.)),
            None,
            gpui::Modifiers::none(),
        );
        settle(&mut cx);
        cx.executor()
            .advance_clock(SnackbarDuration::LONG + Duration::from_millis(100));
        settle(&mut cx);
        assert!(
            !visible(&state, &mut cx),
            "long hides 10s after the pointer leaves"
        );

        // Indefinite stays.
        set(&state, &mut cx, "duration", Value::Choice(2));
        set(&state, &mut cx, "visible", Value::Bool(true));
        cx.executor().advance_clock(SnackbarDuration::LONG * 3);
        settle(&mut cx);
        assert!(visible(&state, &mut cx));
    }
}
