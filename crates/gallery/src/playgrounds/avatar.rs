//! Avatar playground: the kit's generic avatar styles (placeholder,
//! monogram, check) plus an icon avatar, at any size, on the real `Avatar`
//! beside a name as in a list or account chip.
use super::prelude::*;

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Monogram",
            values: &[
                ("style", Set::Option("monogram")),
                ("initials", Set::Text("JD")),
            ],
        },
        Scenario {
            name: "Selected",
            values: &[("selected", Set::Bool(true))],
        },
        Scenario {
            name: "Large icon",
            values: &[
                ("style", Set::Option("icon")),
                ("icon", Set::Option("pets")),
                ("size", Set::Number(72.)),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Segmented {
                    id: "style",
                    label: "Style",
                    options: &["avatar", "monogram", "icon"],
                    default: 0,
                },
                Control::Slider {
                    id: "size",
                    label: "Size",
                    min: 24.,
                    max: 96.,
                    step: 4.,
                    default: 40.,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Text {
                    id: "initials",
                    label: "Initials",
                    default: "A",
                },
                Control::Icon {
                    id: "icon",
                    label: "Icon",
                    icons: &["person", "face", "pets", "star"],
                    default: 1,
                },
                Control::Text {
                    id: "name",
                    label: "Name",
                    default: "Ali Connors",
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "selected",
                    label: "Selected",
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
    let name = config.text("name");
    let selected = config.bool("selected");
    let avatar = Avatar::new("avatar", name.clone())
        .size(config.number("size"))
        .selected(selected)
        .disabled(config.bool("disabled"));
    let avatar = match config.option("style") {
        "monogram" => avatar.monogram(config.text("initials")),
        "icon" => avatar.icon(symbol(config.option("icon"), 24.)),
        _ => avatar,
    };
    // Clicking the row toggles selection, as a list's avatar does.
    let toggle = preview.on_bool("selected", cx);
    FocusRing::new(
        "avatar-row-ring",
        gpui_base::Button::new("avatar-row")
            .accessibility_label(name.clone())
            .selected(selected)
            .flex()
            .items_center()
            .gap(rem(16.))
            .px(rem(16.))
            .py(rem(12.))
            .rounded(rem(16.))
            .cursor_pointer()
            .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
            .child(avatar)
            .child(
                div()
                    .map(|text| type_role(text, TypeRole::BodyLarge))
                    .text_color(color(ColorRole::ON_SURFACE))
                    .child(name),
            )
            .on_click(move |_, window, cx| toggle(&!selected, window, cx)),
    )
    .into_any_element()
}
