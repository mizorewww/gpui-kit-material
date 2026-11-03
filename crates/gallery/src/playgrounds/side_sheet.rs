//! Side sheet playground: md3.io's Side sheet configuration (standard or
//! modal, start or end edge, width, title, body, open, close button, scrim
//! and Escape dismissal) on the real `MaterialSideSheet`, opened from a
//! button on the preview's screen.
//!
//! md3.io's "Sheet" content preset is the Scenario select.
use super::dialog::{open_button, screen};
use super::prelude::*;

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Photo info",
            values: &[
                ("variant", Set::Option("standard")),
                ("title", Set::Text("Photo info")),
                (
                    "body",
                    Set::Text(
                        "Taken on Pixel 9 Pro, 24 mm, f/1.7, 1/120 s, ISO 125. Golden Gate Park, San Francisco.",
                    ),
                ),
                ("open", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Search filters",
            values: &[
                ("position", Set::Option("start")),
                ("title", Set::Text("Filters")),
                (
                    "body",
                    Set::Text("Narrow results by date, file type, owner and location."),
                ),
                ("width", Set::Number(280.)),
                ("open", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Appearance settings",
            values: &[
                ("title", Set::Text("Appearance")),
                (
                    "body",
                    Set::Text(
                        "Choose a theme, text size and contrast level. Changes apply right away.",
                    ),
                ),
                ("width", Set::Number(360.)),
                ("scrim", Set::Bool(false)),
                ("open", Set::Bool(true)),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[Control::Segmented {
                id: "variant",
                label: "Variant",
                options: &["standard", "modal"],
                default: 1,
            }],
        },
        Section {
            title: "Layout",
            controls: &[
                Control::Segmented {
                    id: "position",
                    label: "Position",
                    options: &["start", "end"],
                    default: 1,
                },
                Control::Slider {
                    id: "width",
                    label: "Width",
                    min: 256.,
                    max: 400.,
                    step: 8.,
                    default: 320.,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Text {
                    id: "title",
                    label: "Title",
                    default: "Details",
                },
                Control::Text {
                    id: "body",
                    label: "Body",
                    default: "A place for useful context, related actions and supporting details.",
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
    code: Some(code),
};

fn preview(
    config: &Config,
    preview: &mut Preview,
    _: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let set_open = std::rc::Rc::new(preview.setter("open", cx));
    let open = config.bool("open");
    let title = config.text("title");
    let body = config.text("body");
    let mut sheet = MaterialSideSheet::new("playground-side-sheet")
        .modal(config.option("variant") == "modal")
        .edge(if config.option("position") == "start" {
            SideSheetEdge::Start
        } else {
            SideSheetEdge::End
        })
        .width(rem(config.number("width")))
        .dismiss_on_scrim(config.bool("scrim"))
        .dismiss_on_escape(config.bool("escape"))
        .contained(true)
        .open(open)
        .label(if title.is_empty() {
            SharedString::new_static("Side sheet")
        } else {
            title.clone()
        })
        .on_dismiss({
            let set_open = set_open.clone();
            move |window, cx| set_open(Value::Bool(false), window, cx)
        })
        .child(
            div()
                .debug_selector(|| "side-sheet-body".into())
                .px(rem(24.))
                .map(|text| type_role(text, TypeRole::BodyMedium))
                .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                .child(body),
        )
        .action(
            material_button("side-sheet-save", ButtonVariant::Filled)
                .debug_selector(|| "side-sheet-save".into())
                .child("Save")
                .on_click({
                    let set_open = set_open.clone();
                    move |_, window, cx| set_open(Value::Bool(false), window, cx)
                }),
        )
        .action(
            material_button("side-sheet-cancel", ButtonVariant::Outlined)
                .child("Cancel")
                .on_click({
                    let set_open = set_open.clone();
                    move |_, window, cx| set_open(Value::Bool(false), window, cx)
                }),
        );
    if !title.is_empty() {
        sheet = sheet.headline(title);
    }
    if config.bool("close") {
        sheet = sheet.close_button("Close sheet");
    }
    let label = if open {
        "Close side sheet"
    } else {
        "Open side sheet"
    };
    screen("playground-side-sheet")
        .child(open_button(
            "side-sheet-open",
            label,
            move |_, window, cx| set_open(Value::Bool(!open), window, cx),
        ))
        .child(sheet)
        .into_any_element()
}

fn code(config: &Config) -> String {
    const CLOSE: &str = "cx.listener(|this, _: &ClickEvent, _, cx| {\n    this.sheet_open = false;\n    cx.notify();\n})";
    let title = config.text("title");
    let width = config.number("width");
    let action = |id: &str, variant: &str, label: &str| {
        Some(format!(
            "action({})",
            chain(
                format!("material_button({}, ButtonVariant::{variant})", lit(id)),
                [
                    Some(format!("child({})", lit(label))),
                    Some(format!("on_click({CLOSE})")),
                ],
            )
        ))
    };
    let sheet = chain(
        "MaterialSideSheet::new(\"sheet\")",
        [
            Some("open(self.sheet_open)".into()),
            (config.option("variant") == "modal").then(|| "modal(true)".into()),
            (config.option("position") == "start").then(|| "edge(SideSheetEdge::Start)".into()),
            (width != 320.).then(|| format!("width(rem({width:?}))")),
            if title.is_empty() {
                Some("label(\"Side sheet\")".into())
            } else {
                Some(format!("headline({})", lit(&title)))
            },
            config
                .bool("close")
                .then(|| "close_button(\"Close sheet\")".into()),
            (!config.bool("scrim")).then(|| "dismiss_on_scrim(false)".into()),
            (!config.bool("escape")).then(|| "dismiss_on_escape(false)".into()),
            Some(
                "on_dismiss({\n    let view = cx.entity();\n    move |_, cx| {\n        view.update(cx, |this, cx| {\n            this.sheet_open = false;\n            cx.notify();\n        })\n    }\n})"
                    .into(),
            ),
            Some(format!(
                "child(div().px(rem(24.)).child({}))",
                lit(&config.text("body"))
            )),
            action("save", "Filled", "Save"),
            action("cancel", "Outlined", "Cancel"),
        ],
    );
    let open = chain(
        "material_button(\"open-sheet\", ButtonVariant::Tonal)",
        [
            Some("child(\"Open side sheet\")".into()),
            Some(
                "on_click(cx.listener(|this, _: &ClickEvent, _, cx| {\n    this.sheet_open = true;\n    cx.notify();\n}))"
                    .into(),
            ),
        ],
    );
    chain(
        "div()",
        [
            Some("relative()".into()),
            Some("size_full()".into()),
            Some(format!("child({open})")),
            Some(format!("child({sheet})")),
        ],
    )
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
            assert!(h.bounds("playground-side-sheet").is_some(), "{scenario}");
            let open = h.config().bool("open");
            h.advance(1000);
            assert_eq!(h.bounds("side-sheet-body").is_some(), open, "{scenario}");
        });
    }

    #[gpui::test]
    fn the_sheet_opens_on_either_edge_and_dismisses(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let mut h = Harness::open(cx, &PLAYGROUND);
        h.click("side-sheet-open");
        assert!(h.config().bool("open"));
        h.advance(1000);
        let screen = h.expect("playground-side-sheet");
        let body = h.expect("side-sheet-body");
        assert!(
            body.right() > screen.center().x && body.right() <= screen.right(),
            "an end sheet sits at the screen's right: {body:?} in {screen:?}"
        );
        h.keys("escape");
        assert!(!h.config().bool("open"), "Escape dismisses");

        h.set("position", Value::Choice(0));
        h.set("open", Value::Bool(true));
        h.advance(1000);
        let body = h.expect("side-sheet-body");
        assert!(body.left() < screen.center().x, "a start sheet: {body:?}");
        // The scrim (the screen's far side) dismisses.
        h.click_at(gpui::point(
            screen.right() - gpui::px(8.),
            screen.center().y,
        ));
        assert!(!h.config().bool("open"), "the scrim dismisses");

        // A standard sheet leaves the screen interactive; its close button
        // dismisses it.
        h.set("variant", Value::Choice(0));
        h.set("open", Value::Bool(true));
        h.advance(1000);
        h.click("sheet-close");
        assert!(!h.config().bool("open"), "the close button dismisses");
    }

    #[test]
    fn the_code_follows_the_configuration() {
        let mut config = Config::defaults(&PLAYGROUND);
        let code = super::code(&config);
        assert!(code.starts_with("div()"), "{code}");
        assert!(code.contains("MaterialSideSheet::new(\"sheet\")"));
        assert!(code.contains(".headline(\"Details\")"));
        assert!(!code.contains(".edge("));
        assert!(!code.contains(".width("));
        config.set("position", Value::Choice(0));
        config.set("width", Value::Number(280.));
        let code = super::code(&config);
        assert!(code.contains(".edge(SideSheetEdge::Start)"));
        assert!(code.contains(".width(rem(280.0))"));
    }
}
