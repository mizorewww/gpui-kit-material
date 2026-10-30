//! Dialog playground: md3.io's Dialog configuration (size, animation,
//! dividers, title, subtitle, body, actions and their alignment, open, close
//! button, scrim and Escape dismissal) on the real `MaterialDialog`, opened
//! from a button and presented over the preview's screen.
//!
//! md3.io's "Dialog" content preset is the Scenario select.
use super::prelude::*;

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Location permission",
            values: &[
                ("title", Set::Text("Allow location access?")),
                ("subtitle", Set::Text("Weather")),
                (
                    "body",
                    Set::Text(
                        "Weather uses your location to show local forecasts and severe weather alerts.",
                    ),
                ),
                ("animation", Set::Option("slide up")),
                ("close", Set::Bool(false)),
                ("open", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Discard draft",
            values: &[
                ("title", Set::Text("Discard draft?")),
                (
                    "body",
                    Set::Text("Your message hasn't been sent. Discarding removes it permanently."),
                ),
                ("close", Set::Bool(false)),
                ("scrim", Set::Bool(false)),
                ("animation", Set::Option("fade")),
                ("open", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Out of stock",
            values: &[
                ("title", Set::Text("Out of stock")),
                ("subtitle", Set::Text("Wireless headphones")),
                (
                    "body",
                    Set::Text(
                        "This item is sold out right now. We can email you as soon as it's back in stock, usually within two weeks.",
                    ),
                ),
                ("size", Set::Option("medium")),
                ("dividers", Set::Bool(true)),
                ("alignment", Set::Option("space between")),
                ("open", Set::Bool(true)),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Select {
                    id: "size",
                    label: "Size",
                    options: &["small", "medium", "large", "fullwidth", "fullscreen"],
                    default: 0,
                },
                Control::Select {
                    id: "animation",
                    label: "Animation",
                    options: &["scale", "slide up", "slide down", "fade"],
                    default: 0,
                },
                Control::Switch {
                    id: "dividers",
                    label: "Dividers",
                    default: false,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Text {
                    id: "title",
                    label: "Title",
                    default: "Save your changes?",
                },
                Control::Text {
                    id: "subtitle",
                    label: "Subtitle",
                    default: "",
                },
                Control::Text {
                    id: "body",
                    label: "Body",
                    default: "Keep your changes before leaving this page, or discard them.",
                },
                Control::Switch {
                    id: "actions",
                    label: "Show actions",
                    default: true,
                },
                Control::Select {
                    id: "alignment",
                    label: "Action alignment",
                    options: &["right", "left", "center", "space between"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "open",
                    label: "Open",
                    default: false,
                },
                Control::Switch {
                    id: "close",
                    label: "Close button",
                    default: true,
                },
                Control::Switch {
                    id: "scrim",
                    label: "Dismiss on scrim",
                    default: true,
                },
                Control::Switch {
                    id: "escape",
                    label: "Dismiss with Escape",
                    default: true,
                },
            ],
        },
    ],
    preview,
};

/// The dismissing and confirming actions' labels for a title.
fn action_labels(title: &str) -> (&'static str, &'static str) {
    match title {
        "Save your changes?" => ("Discard", "Save"),
        "Allow location access?" => ("Deny", "Allow"),
        "Discard draft?" => ("Cancel", "Discard"),
        "Out of stock" => ("Close", "Notify me"),
        _ => ("Cancel", "OK"),
    }
}

/// The height of a preview's screen, in design px: the canvas grows with its
/// content, so the screen brings its own height.
const SCREEN_HEIGHT: f32 = 480.;

/// The preview's screen: the surface a contained overlay presents over.
pub(super) fn screen(id: &'static str) -> Div {
    div()
        .debug_selector(move || id.into())
        .relative()
        .w_full()
        .h(rem(SCREEN_HEIGHT))
        .overflow_hidden()
        .bg(color(ColorRole::SURFACE_CONTAINER_LOWEST))
        .border_1()
        .border_color(color(ColorRole::OUTLINE_VARIANT))
}

/// The tonal button opening the preview's overlay.
pub(super) fn open_button(
    id: &'static str,
    label: &'static str,
    open: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .child(
            material_button(id, ButtonVariant::Tonal)
                .debug_selector(move || id.into())
                .child(label)
                .on_click(open),
        )
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    _: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let set_open = preview.setter("open", cx);
    let set_open = std::rc::Rc::new(set_open);
    let title = config.text("title");
    let (dismiss_label, confirm_label) = action_labels(&title);
    let mut dialog = MaterialDialog::new("playground-dialog")
        .size(DialogSize::ALL[config.choice("size")])
        .animation(DialogAnimation::ALL[config.choice("animation")])
        .dividers(config.bool("dividers"))
        .action_alignment(DialogActionAlignment::ALL[config.choice("alignment")])
        .dismiss_on_scrim(config.bool("scrim"))
        .dismiss_on_escape(config.bool("escape"))
        .contained(true)
        .modal(config.bool("open"))
        .on_dismiss({
            let set_open = set_open.clone();
            move |window, cx| set_open(Value::Bool(false), window, cx)
        });
    if !title.is_empty() {
        dialog = dialog.headline(title.clone());
    } else {
        dialog = dialog.label(confirm_label);
    }
    for (id, apply) in [
        (
            "subtitle",
            MaterialDialog::subtitle as fn(MaterialDialog, SharedString) -> MaterialDialog,
        ),
        ("body", MaterialDialog::supporting_text),
    ] {
        let text = config.text(id);
        if !text.is_empty() {
            dialog = apply(dialog, text);
        }
    }
    if config.bool("close") || config.option("size") == "fullscreen" {
        dialog = dialog.close_button("Close dialog");
    }
    if config.bool("actions") {
        // Either action closes the dialog.
        for (id, label, variant) in [
            ("dialog-dismiss", dismiss_label, ButtonVariant::Text),
            ("dialog-confirm", confirm_label, ButtonVariant::Text),
        ] {
            let set_open = set_open.clone();
            dialog = dialog.action(
                material_button(id, variant)
                    .debug_selector(move || id.into())
                    .child(label)
                    .on_click(move |_, window, cx| set_open(Value::Bool(false), window, cx)),
            );
        }
    }
    screen("playground-dialog")
        .child(open_button("dialog-open", "Open dialog", {
            let set_open = set_open.clone();
            move |_, window, cx| set_open(Value::Bool(true), window, cx)
        }))
        .child(dialog)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::nav_harness::Harness;
    use super::*;
    use core::prelude::v1::test;
    use gpui::TestAppContext;

    #[gpui::test]
    fn every_scenario_renders_the_screen(cx: &mut TestAppContext) {
        let mut h = Harness::open(cx, &PLAYGROUND);
        h.render_every_scenario(|h, scenario| {
            assert!(h.bounds("playground-dialog").is_some(), "{scenario}");
            let open = h.config().bool("open");
            // Opened scenarios present their dialog with its actions.
            h.settle();
            assert_eq!(h.bounds("dialog-confirm").is_some(), open, "{scenario}");
        });
    }

    #[gpui::test]
    fn the_button_opens_and_actions_escape_and_the_scrim_dismiss(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let mut h = Harness::open(cx, &PLAYGROUND);
        assert!(h.bounds("dialog-confirm").is_none());
        h.click("dialog-open");
        assert!(h.config().bool("open"));
        h.advance(1000);
        let confirm = h.expect("dialog-confirm");
        // The dialog presents over the screen, not the window.
        let screen = h.expect("playground-dialog");
        assert!(
            screen.contains(&confirm.origin),
            "{confirm:?} in {screen:?}"
        );
        h.click("dialog-confirm");
        assert!(!h.config().bool("open"), "an action closes it");

        h.click("dialog-open");
        h.advance(1000);
        h.keys("escape");
        assert!(!h.config().bool("open"), "Escape dismisses");

        h.set("escape", Value::Bool(false));
        h.click("dialog-open");
        h.advance(1000);
        h.keys("escape");
        assert!(h.config().bool("open"), "Escape is turned off");

        // The scrim (the screen's corner, outside the dialog) dismisses.
        let corner = screen.origin + gpui::point(gpui::px(8.), gpui::px(8.));
        h.set("scrim", Value::Bool(false));
        h.click_at(corner);
        assert!(h.config().bool("open"), "the scrim is turned off");
        h.set("scrim", Value::Bool(true));
        h.click_at(corner);
        assert!(!h.config().bool("open"), "the scrim dismisses");
    }

    #[gpui::test]
    fn the_close_button_dismisses_and_fullscreen_fills_the_screen(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let mut h = Harness::open(cx, &PLAYGROUND);
        h.set("size", Value::Choice(4));
        h.set("open", Value::Bool(true));
        h.advance(1000);
        let screen = h.expect("playground-dialog");
        let confirm = h.expect("dialog-confirm");
        assert!(
            confirm.bottom() > screen.bottom() - gpui::px(80.),
            "the full-screen dialog's actions sit at the screen's bottom"
        );
        let close = h.expect("dialog-close");
        h.click_at(close.center());
        assert!(!h.config().bool("open"));
    }
}
