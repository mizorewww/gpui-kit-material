//! Carousel playground: md3.io's Carousel configuration (layout, corner
//! radius, item width, gap, padding, collection, captions, current slide,
//! snapping) on the real `MaterialCarousel`, with md3.io's first /
//! previous / next / last buttons and position under it. Wheel, drag and
//! the arrow keys scroll it, writing the current slide back.
use super::prelude::*;

const LAYOUTS: [CarouselLayout; 5] = [
    CarouselLayout::MultiBrowse,
    CarouselLayout::Uncontained,
    CarouselLayout::Hero,
    CarouselLayout::CenteredHero,
    CarouselLayout::FullScreen,
];
const SLIDES: &[&str] = &[
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
    "17", "18", "19", "20", "21", "22", "23",
];
const COUNT: usize = 24;

/// Each collection's 24 (label, supporting text) captions.
const PLACES: [(&str, &str); COUNT] = [
    ("Harbor Pier", "Australia"),
    ("Milky Way Fence", "Wyoming"),
    ("Golden Hills", "California"),
    ("Cliff Road", "Portugal"),
    ("Salt Flats", "Bolivia"),
    ("Fjord Morning", "Norway"),
    ("Desert Arch", "Utah"),
    ("Rice Terraces", "Bali"),
    ("Glacier Bay", "Alaska"),
    ("Lavender Rows", "Provence"),
    ("Red Canyon", "Arizona"),
    ("Lake Shore", "Slovenia"),
    ("Old Town", "Prague"),
    ("Pine Ridge", "Oregon"),
    ("Coral Reef", "Belize"),
    ("Tea Fields", "Sri Lanka"),
    ("Dune Sea", "Namibia"),
    ("Basalt Coast", "Iceland"),
    ("Cherry Lane", "Kyoto"),
    ("Volcano Rim", "Hawaii"),
    ("Mossy Falls", "Scotland"),
    ("Sunset Mesa", "New Mexico"),
    ("Island Cove", "Greece"),
    ("Aurora Camp", "Finland"),
];
const FEATURED: [&str; 8] = [
    "Night markets",
    "Coastal drives",
    "Mountain huts",
    "City gardens",
    "Hidden beaches",
    "Rooftop views",
    "River walks",
    "Snow days",
];
const CURATED: [&str; 8] = [
    "Weekend getaways",
    "Family favorites",
    "Slow travel",
    "Food trails",
    "Photo spots",
    "Quiet places",
    "Road trips",
    "Wild camping",
];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Featured collection",
            values: &[
                ("collection", Set::Option("Featured collection")),
                ("variant", Set::Option("hero")),
                ("width", Set::Number(320.)),
            ],
        },
        Scenario {
            name: "Curated lists",
            values: &[
                ("collection", Set::Option("Curated lists")),
                ("variant", Set::Option("uncontained")),
                ("width", Set::Number(200.)),
                ("radius", Set::Number(16.)),
                ("current", Set::Option("2")),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[
                Control::Select {
                    id: "variant",
                    label: "Variant",
                    options: &[
                        "multi browse",
                        "uncontained",
                        "hero",
                        "hero center",
                        "full screen",
                    ],
                    default: 0,
                },
                Control::Slider {
                    id: "radius",
                    label: "Corner radius",
                    min: 0.,
                    max: 40.,
                    step: 1.,
                    default: 28.,
                },
            ],
        },
        Section {
            title: "Layout",
            controls: &[
                Control::Slider {
                    id: "width",
                    label: "Item width",
                    min: 120.,
                    max: 400.,
                    step: 4.,
                    default: 280.,
                },
                Control::Slider {
                    id: "gap",
                    label: "Gap",
                    min: 0.,
                    max: 32.,
                    step: 1.,
                    default: 8.,
                },
                Control::Slider {
                    id: "padding",
                    label: "Padding",
                    min: 0.,
                    max: 48.,
                    step: 1.,
                    default: 16.,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "collection",
                    label: "Collection",
                    options: &["Default", "Featured collection", "Curated lists"],
                    default: 0,
                },
                Control::Switch {
                    id: "captions",
                    label: "Captions",
                    default: true,
                },
                Control::Select {
                    id: "current",
                    label: "Current slide",
                    options: SLIDES,
                    default: 0,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[Control::Switch {
                id: "snap",
                label: "Snap to items",
                default: true,
            }],
        },
    ],
    preview,
};

/// The caption of item `index` in `collection`.
fn caption(collection: usize, index: usize) -> (SharedString, SharedString) {
    match collection {
        1 => (
            FEATURED[index % 8].into(),
            format!("Volume {}", index / 8 + 1).into(),
        ),
        2 => (
            CURATED[index % 8].into(),
            format!("{} places", 6 + index % 7).into(),
        ),
        _ => (PLACES[index].0.into(), PLACES[index].1.into()),
    }
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let current = config.choice("current");
    let collection = config.choice("collection");
    let captions = config.bool("captions");
    let width = config.number("width");
    let go = |index: usize| Value::Choice(index.min(COUNT - 1));
    let items = (0..COUNT).map(|index| {
        let item = CarouselItem::new().content(
            super::card::scene_variant(("carousel-photo", index), index)
                .debug_selector(move || format!("carousel-photo-{index}"))
                .size_full(),
        );
        if captions {
            let (label, supporting) = caption(collection, index);
            item.label(label).supporting(supporting)
        } else {
            item
        }
    });
    let select = preview.setter("current", cx);
    let carousel = MaterialCarousel::new("playground-carousel", "Places")
        .layout(LAYOUTS[config.choice("variant")])
        .corner_radius(px(config.number("radius")))
        .large_item_width(width)
        .item_height(360.)
        .gap(config.number("gap"))
        .padding(config.number("padding"))
        .snap(config.bool("snap"))
        .current(current)
        .on_change(move |index, window, cx| select(go(*index), window, cx))
        .items(items);

    let mut setters = Vec::with_capacity(4);
    for _ in 0..4 {
        setters.push(preview.setter("current", cx));
    }
    let mut button =
        |id: &'static str, icon: &'static str, label: &'static str, to: usize, enabled: bool| {
            let select = setters.pop().expect("one setter per button");
            div().debug_selector(move || format!("{id}-button")).child(
                IconButton::new(id, symbol(icon, 24.), label)
                    .disabled(!enabled)
                    .on_click(move |_, window, cx| select(go(to), window, cx)),
            )
        };
    let last = COUNT - 1;
    let position = progress(
        "carousel-position",
        (current + 1) as f32 / COUNT as f32,
        window,
        cx,
    );
    let controls = div()
        .flex()
        .items_center()
        .justify_center()
        .gap(rem(8.))
        .child(button(
            "carousel-first",
            "first_page",
            "First",
            0,
            current > 0,
        ))
        .child(button(
            "carousel-previous",
            "chevron_left",
            "Previous",
            current.saturating_sub(1),
            current > 0,
        ))
        .child(div().w(rem(220.)).child(position))
        .child(button(
            "carousel-next",
            "chevron_right",
            "Next",
            current + 1,
            current < last,
        ))
        .child(button(
            "carousel-last",
            "last_page",
            "Last",
            last,
            current < last,
        ))
        .child(
            div()
                .debug_selector(move || format!("carousel-position-{current}"))
                .child(
                    accessible_text(
                        "carousel-count",
                        SharedString::from(format!("{} / {COUNT}", current + 1)),
                    )
                    .map(|text| type_role(text, TypeRole::LabelLarge))
                    .text_color(color(ColorRole::ON_SURFACE)),
                ),
        );
    div()
        .debug_selector(|| "playground-carousel".into())
        .w_full()
        .flex()
        .flex_col()
        .gap(rem(16.))
        .child(carousel)
        .child(controls)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::super::playground::Value;
    use super::super::harness::Harness;
    use super::PLAYGROUND;
    use gpui::TestAppContext;

    #[gpui::test]
    fn every_scenario_renders(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.render_every_scenario(|harness, _| {
            harness.expect("playground-carousel");
        });
    }

    #[gpui::test]
    fn buttons_and_arrow_keys_move_the_current_slide(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.click("carousel-next-button");
        assert_eq!(harness.config().choice("current"), 1);
        assert!(harness.bounds("carousel-position-1").is_some());
        harness.click("carousel-last-button");
        assert_eq!(harness.config().choice("current"), 23);
        // The keyboard moves the carousel and writes the slide back.
        harness.set("current", Value::Choice(3));
        let photo = harness.expect("carousel-photo-3");
        harness.click_at(photo.center());
        harness.keys("right");
        assert_eq!(harness.config().choice("current"), 4);
    }
}
