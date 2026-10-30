//! Bottom sheet playground: md3.io's Bottom sheet configuration (standard or
//! modal, drag handle, peek height, maximum width, title, body, state, scrim
//! and Escape dismissal) on the real `MaterialBottomSheet`, opened from a
//! button on the preview's screen. Dragging the sheet or activating its
//! handle writes the State control back.
//!
//! md3.io's "Sheet" content preset is the Scenario select.
use super::dialog::{open_button, screen};
use super::prelude::*;

const DETENTS: [BottomSheetDetent; 3] = [
    BottomSheetDetent::Hidden,
    BottomSheetDetent::Partial,
    BottomSheetDetent::Expanded,
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Photo sharing",
            values: &[
                ("title", Set::Text("Share photo")),
                (
                    "body",
                    Set::Text("Send to people nearby, copy a link, or post it to an album."),
                ),
                ("state", Set::Option("partial")),
            ],
        },
        Scenario {
            name: "Song options",
            values: &[
                ("title", Set::Text("Midnight City")),
                (
                    "body",
                    Set::Text(
                        "Add to queue, go to the album, share, or download for offline listening.",
                    ),
                ),
                ("peek", Set::Number(0.)),
                ("state", Set::Option("expanded")),
            ],
        },
        Scenario {
            name: "Music player",
            values: &[
                ("variant", Set::Option("standard")),
                ("title", Set::Text("Now playing")),
                (
                    "body",
                    Set::Text("Drag up for the full player: lyrics, queue and sound settings."),
                ),
                ("peek", Set::Number(96.)),
                ("state", Set::Option("partial")),
            ],
        },
        Scenario {
            name: "File filters",
            values: &[
                ("title", Set::Text("Filter files")),
                (
                    "body",
                    Set::Text(
                        "Show documents, spreadsheets, images or videos modified in the last month.",
                    ),
                ),
                ("width", Set::Number(480.)),
                ("scrim", Set::Bool(false)),
                ("state", Set::Option("expanded")),
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
                    options: &["standard", "modal"],
                    default: 1,
                },
                Control::Switch {
                    id: "handle",
                    label: "Drag handle",
                    default: true,
                },
            ],
        },
        Section {
            title: "Layout",
            controls: &[
                Control::Slider {
                    id: "peek",
                    label: "Peek height",
                    min: 0.,
                    max: 320.,
                    step: 8.,
                    default: 120.,
                },
                Control::Slider {
                    id: "width",
                    label: "Maximum width",
                    min: 320.,
                    max: 640.,
                    step: 16.,
                    default: 640.,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Text {
                    id: "title",
                    label: "Title",
                    default: "Plan your visit",
                },
                Control::Text {
                    id: "body",
                    label: "Body",
                    default: "Find a new trail, take in the view from the ridge, and check opening hours before you go.",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Select {
                    id: "state",
                    label: "State",
                    options: &["hidden", "partial", "expanded"],
                    default: 0,
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

fn preview(
    config: &Config,
    preview: &mut Preview,
    _: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let set_state = std::rc::Rc::new(preview.setter("state", cx));
    let detent = DETENTS[config.choice("state")];
    let peek = config.number("peek");
    let title = config.text("title");
    let mut sheet = MaterialBottomSheet::new("playground-bottom-sheet")
        .modal(config.option("variant") == "modal")
        .max_width(rem(config.number("width")))
        .dismiss_on_scrim(config.bool("scrim"))
        .dismiss_on_escape(config.bool("escape"))
        .contained(true)
        .detent(detent)
        .label(if title.is_empty() {
            SharedString::new_static("Bottom sheet")
        } else {
            title.clone()
        })
        .on_detent_change({
            let set_state = set_state.clone();
            move |detent, window, cx| {
                let index = DETENTS.iter().position(|d| *d == detent).unwrap_or(0);
                set_state(Value::Choice(index), window, cx)
            }
        })
        .child(
            div()
                .debug_selector(|| "bottom-sheet-body".into())
                .flex()
                .flex_col()
                .gap(rem(8.))
                .px(rem(24.))
                .pb(rem(24.))
                .when(!config.bool("handle"), |body| body.pt(rem(24.)))
                .when(!title.is_empty(), |body| {
                    body.child(
                        div()
                            .map(|text| type_role(text, TypeRole::TitleLarge))
                            .text_color(color(ColorRole::ON_SURFACE))
                            .child(title.clone()),
                    )
                })
                .child(
                    div()
                        .map(|text| type_role(text, TypeRole::BodyMedium))
                        .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                        .child(config.text("body")),
                )
                .child(
                    div().pt(rem(16.)).flex().justify_end().child(
                        material_button("bottom-sheet-done", ButtonVariant::Filled)
                            .debug_selector(|| "bottom-sheet-done".into())
                            .child("Done")
                            .on_click({
                                let set_state = set_state.clone();
                                move |_, window, cx| set_state(Value::Choice(0), window, cx)
                            }),
                    ),
                ),
        );
    if peek > 0. {
        sheet = sheet.partial_height(rem(peek));
    }
    if config.bool("handle") {
        sheet = sheet.drag_handle("Drag handle");
    }
    let hidden = detent == BottomSheetDetent::Hidden;
    let label = if hidden {
        "Open bottom sheet"
    } else {
        "Close bottom sheet"
    };
    screen("playground-bottom-sheet")
        .child(open_button(
            "bottom-sheet-open",
            label,
            move |_, window, cx| {
                // Opening shows the peek (the partial detent) when there is one.
                let to = match (hidden, peek > 0.) {
                    (false, _) => 0,
                    (true, true) => 1,
                    (true, false) => 2,
                };
                set_state(Value::Choice(to), window, cx)
            },
        ))
        .child(sheet)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::harness::Harness;
    use super::*;
    use core::prelude::v1::test;
    use gpui::{MouseButton, TestAppContext, point, px};

    #[gpui::test]
    fn every_scenario_renders_the_screen(cx: &mut TestAppContext) {
        let mut h = Harness::open(cx, &PLAYGROUND);
        h.render_every_scenario(|h, scenario| {
            assert!(h.bounds("playground-bottom-sheet").is_some(), "{scenario}");
            let open = h.config().option("state") != "hidden";
            h.advance(1000);
            assert_eq!(h.bounds("bottom-sheet-body").is_some(), open, "{scenario}");
        });
    }

    #[gpui::test]
    fn the_sheet_opens_at_its_peek_expands_and_drags_closed(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let mut h = Harness::open(cx, &PLAYGROUND);
        h.click("bottom-sheet-open");
        assert_eq!(h.config().option("state"), "partial");
        h.advance(1000);
        let screen = h.expect("playground-bottom-sheet");
        let body = h.expect("bottom-sheet-body");
        // 120dp of the sheet peeks above the screen's bottom edge.
        let peek = screen.bottom() - body.top();
        assert!(peek > px(40.) && peek < px(120.), "peek {peek:?}");

        // The handle expands the sheet and the State control follows.
        h.click("drag-handle");
        assert_eq!(h.config().option("state"), "expanded");
        h.advance(1000);
        let expanded = h.expect("bottom-sheet-body");
        assert!(expanded.top() < body.top());
        assert!(
            (expanded.bottom() - screen.bottom()).abs() <= px(2.),
            "{expanded:?}"
        );

        // Dragging the handle down past the bottom dismisses it.
        let handle = h.expect("drag-handle").center();
        let end = point(handle.x, screen.bottom() - px(2.));
        h.cx.simulate_mouse_down(handle, MouseButton::Left, Default::default());
        h.settle();
        h.cx.simulate_mouse_move(end, Some(MouseButton::Left), Default::default());
        h.settle();
        h.cx.simulate_mouse_up(end, MouseButton::Left, Default::default());
        h.settle();
        assert_eq!(h.config().option("state"), "hidden");

        // The State control drives the sheet, and Escape dismisses it.
        h.set("state", Value::Choice(2));
        h.advance(1000);
        assert!(h.bounds("bottom-sheet-body").is_some());
        h.keys("escape");
        assert_eq!(h.config().option("state"), "hidden");
    }
}
