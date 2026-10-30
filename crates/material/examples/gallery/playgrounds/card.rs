//! Card playground: md3.io's Card configuration (variant, image, ratio and
//! position, title, subtitle, body, actions, clickable, draggable) on the
//! real `MaterialCard`. md3.io's Content "Card" preset select is the
//! Scenario select. Clicking a clickable card counts the clicks below it;
//! a draggable card follows the pointer and springs home on release.
use super::prelude::*;

const VARIANTS: [CardVariant; 3] = [
    CardVariant::Elevated,
    CardVariant::Filled,
    CardVariant::Outlined,
];
/// The card's width in design px, as md3.io's preview.
const WIDTH: f32 = 340.;

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Concert tour",
            values: &[
                ("variant", Set::Option("filled")),
                ("ratio", Set::Option("4:3")),
                ("title", Set::Text("Northern Lights tour")),
                ("subtitle", Set::Text("Twelve cities this summer")),
                (
                    "body",
                    Set::Text("Tickets go on sale Friday. Members get early access."),
                ),
            ],
        },
        Scenario {
            name: "Showtime tickets",
            values: &[
                ("variant", Set::Option("outlined")),
                ("image", Set::Bool(false)),
                ("title", Set::Text("Two tickets, row F")),
                ("subtitle", Set::Text("Tonight at 8:00 PM")),
                (
                    "body",
                    Set::Text("Doors open at 7:30. Show this card at the entrance."),
                ),
                ("clickable", Set::Bool(true)),
            ],
        },
        Scenario {
            name: "Podcast episode",
            values: &[
                ("variant", Set::Option("elevated")),
                ("ratio", Set::Option("1:1")),
                ("position", Set::Option("bottom")),
                ("title", Set::Text("Slow mornings")),
                ("subtitle", Set::Text("Episode 42 · 38 min")),
                ("body", Set::Text("Small rituals for a calmer start.")),
                ("actions", Set::Bool(false)),
                ("clickable", Set::Bool(true)),
                ("draggable", Set::Bool(true)),
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
                    options: &["elevated", "filled", "outlined"],
                    default: 0,
                },
                Control::Switch {
                    id: "image",
                    label: "Show image",
                    default: true,
                },
                Control::Segmented {
                    id: "ratio",
                    label: "Image ratio",
                    options: &["16:9", "4:3", "1:1"],
                    default: 0,
                },
                Control::Segmented {
                    id: "position",
                    label: "Image position",
                    options: &["top", "bottom"],
                    default: 0,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Text {
                    id: "title",
                    label: "Title",
                    default: "A little time outside",
                },
                Control::Text {
                    id: "subtitle",
                    label: "Subtitle",
                    default: "Find your next escape",
                },
                Control::Text {
                    id: "body",
                    label: "Body",
                    default: "Take the scenic route. There is always something new to discover.",
                },
                Control::Switch {
                    id: "actions",
                    label: "Show actions",
                    default: true,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "clickable",
                    label: "Clickable",
                    default: false,
                },
                Control::Switch {
                    id: "draggable",
                    label: "Draggable",
                    default: false,
                },
            ],
        },
    ],
    preview,
};

/// Clicks on the clickable card.
#[derive(Default)]
struct Clicks(usize);

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let title = config.text("title");
    let subtitle = config.text("subtitle");
    let body = config.text("body");
    let clicks = preview.local("clicks", window, cx, |_, _| Clicks::default());
    let count = clicks.read(cx).0;

    let mut text = div().flex().flex_col().p(rem(16.)).gap(rem(16.));
    let mut heading = div().flex().flex_col();
    if !title.is_empty() {
        heading = heading.child(
            accessible_text("card-title", title.clone())
                .map(|text| type_role(text, TypeRole::TitleLarge))
                .text_color(color(ColorRole::ON_SURFACE)),
        );
    }
    if !subtitle.is_empty() {
        heading = heading.child(
            accessible_text("card-subtitle", subtitle)
                .map(|text| type_role(text, TypeRole::BodyMedium))
                .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
        );
    }
    text = text.child(heading);
    if !body.is_empty() {
        text = text.child(
            accessible_text("card-body", body)
                .map(|text| type_role(text, TypeRole::BodyMedium))
                .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
        );
    }
    if config.bool("actions") {
        text = text.child(
            div()
                .flex()
                .justify_end()
                .gap(rem(8.))
                .child(
                    MaterialButton::new("card-explore")
                        .variant(ButtonVariant::Text)
                        // The card's own click is a separate action.
                        .on_click(|_, _, cx| cx.stop_propagation())
                        .child("Explore"),
                )
                .child(
                    MaterialButton::new("card-save")
                        .variant(ButtonVariant::Tonal)
                        .on_click(|_, _, cx| cx.stop_propagation())
                        .child("Save"),
                ),
        );
    }
    let image = config.bool("image").then(|| {
        let ratio = match config.option("ratio") {
            "4:3" => 3. / 4.,
            "1:1" => 1.,
            _ => 9. / 16.,
        };
        scene("card-scene").w_full().h(rem(WIDTH * ratio))
    });
    let bottom = config.option("position") == "bottom";

    let mut card = MaterialCard::new("playground-card", VARIANTS[config.choice("variant")])
        .w(rem(WIDTH))
        .draggable(config.bool("draggable"));
    if config.bool("clickable") {
        let label = if title.is_empty() {
            SharedString::new_static("Card")
        } else {
            title
        };
        card = card.accessibility_label(label).on_click(move |_, _, cx| {
            clicks.update(cx, |clicks, cx| {
                clicks.0 += 1;
                cx.notify();
            })
        });
    }
    card = match image {
        Some(image) if bottom => card.child(text).child(image),
        Some(image) => card.child(image).child(text),
        None => card.child(text),
    };
    // Tall cards (a square image) scroll inside the preview.
    div()
        .id("card-preview")
        .max_h_full()
        .overflow_y_scroll()
        .p(rem(8.))
        .flex()
        .flex_col()
        .items_center()
        .gap(rem(16.))
        .child(
            div()
                .debug_selector(|| "playground-card".into())
                .child(card),
        )
        .when(config.bool("clickable"), |column| {
            column.child(
                div()
                    .debug_selector(move || format!("card-clicks-{count}"))
                    .child(
                        accessible_text(
                            "card-clicks",
                            SharedString::from(format!("Card clicked {count} times")),
                        )
                        .map(|text| type_role(text, TypeRole::BodySmall))
                        .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                    ),
            )
        })
        .into_any_element()
}

/// A landscape illustration standing in for a photo: a sky, a sun and two
/// mountain ranges, painted in the theme's roles so it follows the scheme.
pub(crate) fn scene(id: impl Into<ElementId>) -> Stateful<Div> {
    scene_variant(id, 0)
}

/// [`scene`] in one of several palettes and ridge shapes, for galleries of
/// distinct pictures.
pub(crate) fn scene_variant(id: impl Into<ElementId>, variant: usize) -> Stateful<Div> {
    use ColorRole as R;
    const PALETTES: [[ColorRole; 4]; 4] = [
        [
            R::SECONDARY_CONTAINER,
            R::TERTIARY_CONTAINER,
            R::OUTLINE,
            R::SECONDARY,
        ],
        [
            R::PRIMARY_CONTAINER,
            R::TERTIARY_CONTAINER,
            R::PRIMARY,
            R::ON_PRIMARY_CONTAINER,
        ],
        [
            R::TERTIARY_CONTAINER,
            R::SECONDARY_CONTAINER,
            R::TERTIARY,
            R::ON_TERTIARY_CONTAINER,
        ],
        [
            R::SURFACE_CONTAINER_HIGHEST,
            R::PRIMARY_CONTAINER,
            R::OUTLINE_VARIANT,
            R::ON_SURFACE_VARIANT,
        ],
    ];
    let [sky, sun, far, near] = PALETTES[variant % PALETTES.len()].map(color);
    // Ridges and the sun drift with the variant.
    let shift = (variant as f32 * 0.37).fract() * 0.3;
    let ridges = [
        (
            far,
            [
                (0., 1.),
                (0., 0.75),
                (0.29 + shift, 0.15 + shift / 2.),
                (0.6, 0.62),
                (0.83, 0.45 - shift / 3.),
                (1., 0.6),
                (1., 1.),
            ],
        ),
        (
            near,
            [
                (0., 1.),
                (0., 0.85 - shift / 2.),
                (0.38 - shift / 2., 0.6),
                (0.7, 0.9),
                (1., 0.72),
                (1., 1.),
                (1., 1.),
            ],
        ),
    ];
    div().id(id).relative().overflow_hidden().bg(sky).child(
        canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let (w, h) = (bounds.size.width, bounds.size.height);
                let at = |x: f32, y: f32| bounds.origin + point(w * x, h * y);
                let radius = h.min(w) * 0.14;
                let center = at(0.74 - shift, 0.16) + point(px(0.), radius * 0.6);
                let mut disc = PathBuilder::fill();
                for step in 0..=32 {
                    let angle = step as f32 / 32. * std::f32::consts::TAU;
                    let point = center + point(radius * angle.cos(), radius * angle.sin());
                    if step == 0 {
                        disc.move_to(point);
                    } else {
                        disc.line_to(point);
                    }
                }
                disc.close();
                if let Ok(path) = disc.build() {
                    window.paint_path(path, sun);
                }
                for (ink, points) in &ridges {
                    let mut range = PathBuilder::fill();
                    for (index, (x, y)) in points.iter().enumerate() {
                        if index == 0 {
                            range.move_to(at(*x, *y));
                        } else {
                            range.line_to(at(*x, *y));
                        }
                    }
                    range.close();
                    if let Ok(path) = range.build() {
                        window.paint_path(path, *ink);
                    }
                }
            },
        )
        .absolute()
        .size_full(),
    )
}

#[cfg(test)]
mod tests {
    use super::super::super::playground::Value;
    use super::super::nav_harness::Harness;
    use super::PLAYGROUND;
    use gpui::TestAppContext;

    #[gpui::test]
    fn every_scenario_renders(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.render_every_scenario(|harness, _| {
            harness.expect("playground-card");
        });
    }

    #[gpui::test]
    fn image_ratio_sizes_the_card_and_clicks_count(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        let wide = harness.expect("playground-card").size.height;
        harness.set("ratio", Value::Choice(2));
        let square = harness.expect("playground-card").size.height;
        assert!(square > wide, "a 1:1 image is taller than 16:9");
        harness.set("image", Value::Bool(false));
        assert!(harness.expect("playground-card").size.height < wide);

        harness.set("clickable", Value::Bool(true));
        harness.click("playground-card");
        harness.click("playground-card");
        assert!(harness.bounds("card-clicks-2").is_some());
    }
}
