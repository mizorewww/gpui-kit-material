//! Navigation rail playground: md3.io's Navigation rail configuration
//! (standard or modal layout, expanded, expanded width, hide collapsed,
//! destinations, selection, badges, menu button, ripple) on the real
//! `NavigationRail` beside a content pane. The menu button expands and
//! collapses the rail; a modal expanded rail floats over a scrim that,
//! like Escape, collapses it.
use super::super::playground::PaneHint;
use super::super::shell::{panes_stacked, preview_area};
use super::prelude::*;
use gpui_kit_material::icons::Symbol;
use std::sync::OnceLock;

/// md3.io's destination sets, named as its Destinations select names them.
const SETS: &[&str] = &["Default", "Timer", "Expanded clock"];
/// Every destination label of every set; a label outside the current set
/// selects nothing.
const LABELS: &[&str] = &[
    "Inbox",
    "Favorites",
    "Sent",
    "Alarm",
    "Clock",
    "Timer",
    "Stopwatch",
];

/// The least a standard rail leaves its content, in design px.
const MIN_CONTENT: f32 = 200.;

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Timer",
            values: &[
                ("destinations", Set::Option("Timer")),
                ("selected", Set::Option("Timer")),
            ],
        },
        Scenario {
            name: "Expanded clock",
            values: &[
                ("destinations", Set::Option("Expanded clock")),
                ("selected", Set::Option("Clock")),
                ("expanded", Set::Bool(true)),
                ("badges", Set::Bool(false)),
            ],
        },
    ],
    sections: &[
        Section {
            title: "Layout",
            controls: &[
                Control::Segmented {
                    id: "layout",
                    label: "Layout",
                    options: &["standard", "modal"],
                    default: 0,
                },
                Control::Switch {
                    id: "expanded",
                    label: "Expanded",
                    default: false,
                },
                Control::Slider {
                    id: "expanded_width",
                    label: "Expanded width",
                    min: 220.,
                    max: 360.,
                    step: 4.,
                    default: 280.,
                },
                Control::Switch {
                    id: "hide_collapsed",
                    label: "Hide collapsed",
                    default: false,
                },
            ],
        },
        Section {
            title: "Content",
            controls: &[
                Control::Select {
                    id: "destinations",
                    label: "Destinations",
                    options: SETS,
                    default: 0,
                },
                Control::Select {
                    id: "selected",
                    label: "Selected",
                    options: LABELS,
                    default: 0,
                },
                Control::Switch {
                    id: "badges",
                    label: "Badges",
                    default: true,
                },
                Control::Switch {
                    id: "menu",
                    label: "Menu button",
                    default: true,
                },
            ],
        },
        Section {
            title: "Behavior",
            controls: &[Control::Switch {
                id: "ripple",
                label: "Ripple",
                default: true,
            }],
        },
    ],
    preview,
};

/// One destination: label, Material Symbol and badge.
struct Destination {
    label: &'static str,
    symbol: &'static str,
    badge: Option<&'static str>,
}

const MAIL: &[Destination] = &[
    Destination {
        label: "Inbox",
        symbol: "inbox",
        badge: Some("8"),
    },
    Destination {
        label: "Favorites",
        symbol: "favorite",
        badge: None,
    },
    Destination {
        label: "Sent",
        symbol: "send",
        badge: None,
    },
];

const CLOCK: &[Destination] = &[
    Destination {
        label: "Alarm",
        symbol: "alarm",
        badge: Some(""),
    },
    Destination {
        label: "Clock",
        symbol: "schedule",
        badge: None,
    },
    Destination {
        label: "Timer",
        symbol: "timer",
        badge: Some("2"),
    },
    Destination {
        label: "Stopwatch",
        symbol: "avg_pace",
        badge: None,
    },
];

/// A set's destinations with resolved icons, built once (resolving symbol
/// paths allocates).
fn icons(set: &'static [Destination]) -> &'static [NavIcon] {
    static MAIL_ICONS: OnceLock<Vec<NavIcon>> = OnceLock::new();
    static CLOCK_ICONS: OnceLock<Vec<NavIcon>> = OnceLock::new();
    let cell = if std::ptr::eq(set, MAIL) {
        &MAIL_ICONS
    } else {
        &CLOCK_ICONS
    };
    cell.get_or_init(|| {
        set.iter()
            .map(|destination| {
                NavIcon::symbol(
                    Symbol::named(destination.symbol)
                        .unwrap_or_else(|| panic!("{} is a symbol", destination.symbol)),
                )
            })
            .collect()
    })
}

fn items(set: &'static [Destination], badges: bool) -> impl Iterator<Item = NavItem> {
    set.iter().zip(icons(set)).map(move |(destination, icon)| {
        let item = NavItem::new(destination.label, destination.label, icon.clone());
        match destination.badge {
            Some("") if badges => item.badge(NavBadge::Small),
            Some(text) if badges => item.badge(NavBadge::Large(text.into())),
            _ => item,
        }
    })
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let set = match config.option("destinations") {
        "Default" => MAIL,
        _ => CLOCK,
    };
    let sections = config.option("destinations") == "Expanded clock";
    let modal = config.option("layout") == "modal";
    let expanded = config.bool("expanded");
    let hide = config.bool("hide_collapsed");
    let badges = config.bool("badges");
    let selected = config.option("selected");
    let width = config.number("expanded_width");
    let ripple = config.bool("ripple");
    let set_expanded = preview.setter("expanded", cx);
    let set_selected = preview.setter("selected", cx);
    let select = move |value: &SharedString, window: &mut Window, cx: &mut App| {
        if let Some(index) = LABELS.iter().position(|label| *label == value.as_ref()) {
            set_selected(Value::Choice(index), window, cx);
        }
    };
    let select = std::rc::Rc::new(select);
    let toggle = std::rc::Rc::new(set_expanded);
    let menu = |id: &'static str, open: bool| {
        let toggle = toggle.clone();
        IconButton::new(
            id,
            symbol(if open { "menu_open" } else { "menu" }, 24.),
            if open {
                "Collapse navigation"
            } else {
                "Expand navigation"
            },
        )
        .on_click(move |_, window, cx| toggle(Value::Bool(!open), window, cx))
    };
    let rail = |id: &'static str, open: bool| {
        let select = select.clone();
        let mut rail = NavigationRail::new(id, "Main navigation")
            .items(items(set, badges))
            .selected(selected)
            .expanded(open)
            .expanded_width(width)
            .hide_collapsed(hide)
            .ripple(ripple)
            .on_change(move |value, window, cx| select(value, window, cx));
        if sections {
            rail = rail.section(0, "Time").section(2, "Tools");
        }
        if config.bool("menu") {
            rail = rail.menu(menu(
                if id == "playground-rail" {
                    "rail-menu"
                } else {
                    "modal-rail-menu"
                },
                open,
            ));
        }
        rail
    };
    // Where an expanded standard rail would leave its content no usable pane, it
    // overlays the content instead, as a modal one does.
    let room = preview_area(window, panes_stacked(window, PaneHint::default())).0;
    let overlays = modal || room < width + MIN_CONTENT;
    let in_flow = rail("playground-rail", expanded && !overlays);
    let overlay = (overlays && expanded).then(|| {
        let collapse = toggle.clone();
        let dismiss = toggle.clone();
        div()
            .absolute()
            .inset_0()
            .child(navigation_scrim(
                "playground-rail-scrim",
                move |window, cx| collapse(Value::Bool(false), window, cx),
            ))
            .child(
                div().absolute().top_0().bottom_0().left_0().child(
                    rail("playground-modal-rail", true)
                        .modal(true)
                        .on_dismiss(move |window, cx| dismiss(Value::Bool(false), window, cx)),
                ),
            )
    });
    let title = set
        .iter()
        .find(|destination| destination.label == selected)
        .map_or("", |destination| destination.label);
    let content = div()
        .debug_selector(|| "playground-rail-content".into())
        .flex_1()
        .min_w_0()
        .h_full()
        .p(rem(24.))
        .bg(color(ColorRole::SURFACE_CONTAINER))
        .child(
            div()
                .map(|text| type_role(text, TypeRole::TitleLarge))
                .text_color(color(ColorRole::ON_SURFACE))
                .child(title),
        );
    div()
        .id("playground-navigation-rail")
        .debug_selector(|| "playground-navigation-rail".into())
        .relative()
        .w_full()
        .h_full()
        .min_h(rem(360.))
        .flex()
        .rounded(rem(16.))
        .overflow_hidden()
        .child(in_flow)
        .child(content)
        .children(overlay)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::nav_harness::Harness;
    use super::*;
    use core::prelude::v1::test;
    use gpui::TestAppContext;

    /// Enter as a full press: GPUI clicks the focused element on release.
    fn enter(harness: &mut Harness) {
        let keystroke = Keystroke::parse("enter").unwrap();
        harness.cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        harness.cx.simulate_event(KeyUpEvent { keystroke });
        harness.settle();
    }

    #[gpui::test]
    fn every_scenario_renders_the_rail(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        harness.render_every_scenario(|harness, scenario| {
            assert!(
                harness.bounds("playground-navigation-rail").is_some(),
                "scenario {scenario}"
            );
        });
        // Every layout combination renders too.
        for (layout, expanded, hide) in [
            (0, true, true),
            (1, false, true),
            (1, true, false),
            (1, false, false),
        ] {
            harness.set("layout", Value::Choice(layout));
            harness.set("expanded", Value::Bool(expanded));
            harness.set("hide_collapsed", Value::Bool(hide));
            harness.advance(2000);
            harness.expect("playground-navigation-rail");
        }
    }

    #[gpui::test]
    fn the_menu_expands_and_keyboard_selection_writes_back(cx: &mut TestAppContext) {
        let mut harness = Harness::open(cx, &PLAYGROUND);
        // Clicking a collapsed destination selects it: 44dp top padding,
        // the 48dp menu, a 40dp gap, then 64dp items 4dp apart.
        let frame = harness.expect("playground-navigation-rail");
        let favorites = frame.origin + point(px(48.), px(44. + 48. + 40. + 68. + 32.));
        harness.click_at(favorites);
        assert_eq!(harness.config().option("selected"), "Favorites");
        // The menu button expands the rail.
        harness.click_at(frame.origin + point(px(48.), px(44. + 24.)));
        assert!(harness.config().bool("expanded"));
        // Tab enters the destinations on the selection; Down and Enter
        // select the next.
        harness.cx.update(|window, cx| window.focus_next(cx));
        harness.settle();
        harness.keys("down");
        enter(&mut harness);
        assert_eq!(harness.config().option("selected"), "Sent");

        // A modal expanded rail collapses on Escape.
        harness.set("layout", Value::Choice(1));
        harness.advance(2000);
        harness.cx.update(|window, cx| window.focus_next(cx));
        harness.cx.update(|window, cx| window.focus_next(cx));
        harness.keys("escape");
        assert!(!harness.config().bool("expanded"));
    }
}
