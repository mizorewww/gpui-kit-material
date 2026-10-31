//! Navigation suite playground: m3e's configuration (fixed or adaptive type,
//! the expanded form and where it starts, hide and show, primary action) on the
//! real `NavigationSuite`. The preview is smaller than the window it stands
//! for, so sliders supply the window's size unless "Follow window" is on.
use super::prelude::*;
use gpui_kit_material::icons::Symbol;
use std::sync::OnceLock;

const LABELS: &[&str] = &["Home", "Search", "Mail", "Settings"];
const ICONS: [&str; 4] = ["home", "search", "mail", "settings"];
const TYPES: &[&str] = &[
    "adaptive",
    "bar",
    "bar medium",
    "rail",
    "rail expanded",
    "drawer",
    "none",
];
const EXPANDED_FROM: &[&str] = &["medium", "expanded", "large", "extra large", "never"];

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Compact window",
            values: &[("width", Set::Number(360.))],
        },
        Scenario {
            name: "Medium window",
            values: &[("width", Set::Number(800.))],
        },
        Scenario {
            name: "Large window",
            values: &[("width", Set::Number(1300.))],
        },
        Scenario {
            name: "Short window",
            values: &[("width", Set::Number(900.)), ("height", Set::Number(400.))],
        },
        Scenario {
            name: "Expanded rail",
            values: &[
                ("width", Set::Number(1300.)),
                ("expanded", Set::Option("rail")),
            ],
        },
        Scenario {
            name: "Primary action",
            values: &[("action", Set::Bool(true))],
        },
    ],
    sections: &[
        Section {
            title: "Window",
            controls: &[
                Control::Switch {
                    id: "follow",
                    label: "Follow window",
                    default: false,
                },
                Control::Slider {
                    id: "width",
                    label: "Width",
                    min: 320.,
                    max: 1800.,
                    step: 20.,
                    default: 800.,
                },
                Control::Slider {
                    id: "height",
                    label: "Height",
                    min: 320.,
                    max: 1200.,
                    step: 20.,
                    default: 800.,
                },
            ],
        },
        Section {
            title: "Navigation",
            controls: &[
                Control::Select {
                    id: "type",
                    label: "Type",
                    options: TYPES,
                    default: 0,
                },
                Control::Segmented {
                    id: "expanded",
                    label: "Expanded form",
                    options: &["drawer", "rail"],
                    default: 0,
                },
                Control::Select {
                    id: "expanded_from",
                    label: "Expands from",
                    options: EXPANDED_FROM,
                    default: 2,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "selected",
                    label: "Selected",
                    options: LABELS,
                    default: 0,
                },
                Control::Switch {
                    id: "badges",
                    label: "Badges",
                    default: false,
                },
                Control::Switch {
                    id: "action",
                    label: "Primary action",
                    default: false,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[Control::Switch {
                id: "hidden",
                label: "Hide navigation",
                default: false,
            }],
        },
    ],
    preview,
};

fn icons() -> &'static [NavIcon] {
    static ICONS_CACHE: OnceLock<Vec<NavIcon>> = OnceLock::new();
    ICONS_CACHE.get_or_init(|| {
        ICONS
            .iter()
            .map(|name| {
                NavIcon::symbol(Symbol::named(name).unwrap_or_else(|| panic!("{name} is a symbol")))
            })
            .collect()
    })
}

fn fixed_type(name: &str) -> Option<NavigationSuiteType> {
    match name {
        "bar" => Some(NavigationSuiteType::BarCompact),
        "bar medium" => Some(NavigationSuiteType::BarMedium),
        "rail" => Some(NavigationSuiteType::RailCollapsed),
        "rail expanded" => Some(NavigationSuiteType::RailExpanded),
        "drawer" => Some(NavigationSuiteType::Drawer),
        "none" => Some(NavigationSuiteType::None),
        _ => None,
    }
}

fn expanded_from(name: &str) -> Option<WindowWidthClass> {
    match name {
        "medium" => Some(WindowWidthClass::Medium),
        "expanded" => Some(WindowWidthClass::Expanded),
        "large" => Some(WindowWidthClass::Large),
        "extra large" => Some(WindowWidthClass::ExtraLarge),
        _ => None,
    }
}

fn type_name(suite_type: NavigationSuiteType) -> &'static str {
    match suite_type {
        NavigationSuiteType::BarCompact => "Navigation bar",
        NavigationSuiteType::BarMedium => "Navigation bar, horizontal items",
        NavigationSuiteType::RailCollapsed => "Navigation rail",
        NavigationSuiteType::RailExpanded => "Expanded navigation rail",
        NavigationSuiteType::Drawer => "Navigation drawer",
        NavigationSuiteType::None => "No navigation",
    }
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let selected = config.option("selected");
    let badges = config.bool("badges");
    let items = LABELS.iter().zip(icons()).map(|(label, icon)| {
        let item = NavItem::new(*label, *label, icon.clone());
        match *label {
            "Search" if badges => item.badge(NavBadge::Small),
            "Mail" if badges => item.badge(NavBadge::Large("3".into())),
            _ => item,
        }
    });
    let size_class = (!config.bool("follow"))
        .then(|| WindowSizeClass::of(config.number("width"), config.number("height")));
    let from = expanded_from(config.option("expanded_from"));
    let form = if config.option("expanded") == "rail" {
        NavigationSuiteType::RailExpanded
    } else {
        NavigationSuiteType::Drawer
    };
    let fixed = fixed_type(config.option("type"));
    let shown = fixed.unwrap_or_else(|| {
        navigation_suite_type(
            size_class.unwrap_or_else(|| WindowSizeClass::of_window(window)),
            from,
            form,
        )
    });
    let set_selected = preview.setter("selected", cx);
    let mut suite = NavigationSuite::new("playground-navigation-suite", "Main")
        .items(items)
        .selected(selected)
        .layout_type(fixed)
        .size_class(size_class)
        .expanded_from(from)
        .expanded_type(form)
        .navigation_hidden(config.bool("hidden"))
        .on_change(move |value, window, cx| {
            if let Some(index) = LABELS.iter().position(|label| *label == value.as_ref()) {
                set_selected(Value::Choice(index), window, cx);
            }
        })
        .child(
            gpui_base::v_flex()
                .size_full()
                .p(rem(24.))
                .gap(rem(8.))
                .child(
                    div()
                        .debug_selector(|| "navigation-suite-title".into())
                        .map(|text| type_role(text, TypeRole::TitleLarge))
                        .text_color(color(ColorRole::ON_SURFACE))
                        .child(selected),
                )
                .child(
                    div()
                        .map(|text| type_role(text, TypeRole::BodyMedium))
                        .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                        .child(type_name(shown)),
                ),
        );
    if config.bool("action") {
        suite = suite.primary_action(Fab::new(
            "playground-suite-fab",
            symbol("edit", 24.),
            "Compose",
        ));
    }
    div()
        .id("playground-navigation-suite-frame")
        .debug_selector(|| "playground-navigation-suite".into())
        .w_full()
        .max_w(rem(config.number("width")))
        .h(rem(480.))
        .rounded(rem(16.))
        .overflow_hidden()
        .border_1()
        .border_color(color(ColorRole::OUTLINE_VARIANT))
        .child(suite)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use core::prelude::v1::test;

    #[gpui::test]
    fn every_scenario_and_type_renders(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        render_every_scenario(&state, &mut cx);
        render_every_option(&state, &mut cx, &["type", "expanded", "expanded_from"]);
        assert!(cx.debug_bounds("playground-navigation-suite").is_some());
    }

    #[gpui::test]
    fn the_width_slider_moves_between_bar_rail_and_drawer(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        settle_motion(&mut cx);
        let nav =
            |cx: &mut gpui::VisualTestContext| element_bounds(cx, "navigation-suite-navigation");
        let rail = nav(&mut cx);
        assert!(near(rail.size.width, design(&mut cx, RAIL_COLLAPSED_WIDTH)));

        set(&state, &mut cx, "width", Value::Number(1300.));
        settle_motion(&mut cx);
        let drawer = nav(&mut cx);
        assert!(near(drawer.size.width, design(&mut cx, DRAWER_WIDTH)));

        set(&state, &mut cx, "width", Value::Number(360.));
        settle_motion(&mut cx);
        let bar = nav(&mut cx);
        let content = element_bounds(&mut cx, "navigation-suite-content");
        assert!(
            bar.top() >= content.bottom(),
            "a bar sits under the content"
        );
    }

    #[gpui::test]
    fn clicking_a_destination_writes_back_to_the_control(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        settle_motion(&mut cx);
        let id = ElementId::NamedChild(
            ElementId::from("playground-navigation-suite").into(),
            "Search".into(),
        );
        let item = cx.update(|window, _| {
            gpui_base::test_support::find(window, &[], &id)
                .expect("the rail shows the destination")
                .bounds()
                .center()
        });
        click(&mut cx, item);
        assert_eq!(config(&state, &mut cx).option("selected"), "Search");
    }

    #[gpui::test]
    fn hiding_gives_the_content_the_room(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        settle_motion(&mut cx);
        let before = element_bounds(&mut cx, "navigation-suite-content");
        toggle_switch(&mut cx, "hidden");
        settle_motion(&mut cx);
        assert!(config(&state, &mut cx).bool("hidden"));
        let after = element_bounds(&mut cx, "navigation-suite-content");
        assert!(after.size.width > before.size.width);
    }
}
