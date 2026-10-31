//! Pull to refresh playground: m3e's sample on the real `PullToRefresh`. Pull
//! the list down (touch, trackpad or mouse drag) or use the app bar's Refresh
//! button, the keyboard path: both start the same simulated refresh, which adds
//! items when it finishes.
use super::prelude::*;
use std::rc::Rc;
use std::time::Duration;

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Loading indicator",
            values: &[("indicator", Set::Option("loading indicator"))],
        },
        Scenario {
            name: "Refreshing",
            values: &[("refreshing", Set::Bool(true))],
        },
        Scenario {
            name: "Disabled",
            values: &[("disabled", Set::Bool(true))],
        },
    ],
    sections: &[
        Section {
            title: "Appearance",
            controls: &[Control::Segmented {
                id: "indicator",
                label: "Indicator",
                options: &["default", "loading indicator"],
                default: 0,
            }],
        },
        Section {
            title: "Content",
            controls: &[Control::Text {
                id: "label",
                label: "Accessible label",
                default: "Refreshing items",
            }],
        },
        Section {
            title: "Behavior",
            controls: &[
                Control::Switch {
                    id: "refreshing",
                    label: "Refreshing",
                    default: false,
                },
                Control::Switch {
                    id: "disabled",
                    label: "Disabled",
                    default: false,
                },
                Control::Switch {
                    id: "mouse_drag",
                    label: "Mouse drag pulls",
                    default: true,
                },
                Control::Slider {
                    id: "threshold",
                    label: "Threshold",
                    min: 40.,
                    max: 160.,
                    step: 4.,
                    default: 80.,
                },
                Control::Slider {
                    id: "duration",
                    label: "Refresh takes (s)",
                    min: 1.,
                    max: 8.,
                    step: 1.,
                    default: 2.,
                },
            ],
        },
    ],
    preview,
};

/// Starts a refresh: the app bar's button and a released pull share it.
type StartRefresh = Rc<dyn Fn(&mut Window, &mut App)>;

/// The sample's list.
struct Feed {
    count: usize,
}

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let feed = preview.local("feed", window, cx, |_, _| Feed { count: 15 });
    let count = feed.read(cx).count;
    let refreshing = config.bool("refreshing");
    let duration = Duration::from_secs_f32(config.number("duration"));
    let set_refreshing = Rc::new(preview.setter("refreshing", cx));
    let start: StartRefresh = Rc::new(move |window, cx| {
        if refreshing {
            return;
        }
        set_refreshing(Value::Bool(true), window, cx);
        let set_refreshing = set_refreshing.clone();
        let feed = feed.clone();
        window
            .spawn(cx, async move |cx| {
                cx.background_executor().timer(duration).await;
                let _ = cx.update(|window, cx| {
                    feed.update(cx, |feed, cx| {
                        feed.count += 5;
                        cx.notify();
                    });
                    set_refreshing(Value::Bool(false), window, cx);
                });
            })
            .detach();
    });
    let on_button = start.clone();
    let indicator = if config.option("indicator") == "default" {
        PullToRefreshIndicator::Default
    } else {
        PullToRefreshIndicator::LoadingIndicator
    };
    let mut list = PullToRefresh::new("playground-pull", config.text("label"))
        .refreshing(refreshing)
        .disabled(config.bool("disabled"))
        .indicator(indicator)
        .threshold(config.number("threshold"))
        .mouse_drag(config.bool("mouse_drag"))
        .on_refresh(move |window, cx| start(window, cx));
    for index in (1..=count).rev() {
        list = list.child(
            ListItem::new(
                SharedString::from(format!("pull-item-{index}")),
                SharedString::from(format!("Item {index}")),
            )
            .container(ColorRole::SURFACE),
        );
    }
    gpui_base::v_flex()
        .debug_selector(|| "playground-pull-to-refresh".into())
        .w_full()
        .max_w(rem(420.))
        .h(rem(480.))
        .rounded(rem(16.))
        .overflow_hidden()
        .border_1()
        .border_color(color(ColorRole::OUTLINE_VARIANT))
        .bg(color(ColorRole::SURFACE))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .h(rem(64.))
                .px(rem(16.))
                .flex_shrink_0()
                .child(
                    div()
                        .map(|text| type_role(text, TypeRole::TitleLarge))
                        .text_color(color(ColorRole::ON_SURFACE))
                        .child("Items"),
                )
                .child(
                    div().debug_selector(|| "pull-refresh-button".into()).child(
                        IconButton::new("pull-refresh", symbol("refresh", 24.), "Refresh")
                            .on_click(move |_, window, cx| on_button(window, cx)),
                    ),
                ),
        )
        .child(div().flex_1().min_h_0().child(list))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use core::prelude::v1::test;
    use gpui::{Modifiers, MouseButton};

    #[gpui::test]
    fn every_scenario_renders(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        render_every_scenario(&state, &mut cx);
        render_every_option(&state, &mut cx, &["indicator"]);
        assert!(cx.debug_bounds("playground-pull-to-refresh").is_some());
    }

    #[gpui::test]
    fn the_refresh_button_refreshes_and_adds_items(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        let button = element_bounds(&mut cx, "pull-refresh-button").center();
        click(&mut cx, button);
        assert!(config(&state, &mut cx).bool("refreshing"));
        assert!(cx.debug_bounds("pull-to-refresh-indicator").is_some());
        cx.executor().advance_clock(Duration::from_secs(3));
        settle_motion(&mut cx);
        assert!(!config(&state, &mut cx).bool("refreshing"));
        assert!(cx.debug_bounds("pull-to-refresh-indicator").is_none());
    }

    #[gpui::test]
    fn a_mouse_pull_past_the_threshold_starts_the_same_refresh(cx: &mut gpui::TestAppContext) {
        let (state, mut cx) = open(&PLAYGROUND, cx);
        let frame = element_bounds(&mut cx, "pull-to-refresh-content");
        let start = gpui::point(frame.center().x, frame.top() + gpui::px(20.));
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        draw(&mut cx);
        let end = gpui::point(start.x, start.y + gpui::px(260.));
        cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
        draw(&mut cx);
        cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
        draw(&mut cx);
        assert!(config(&state, &mut cx).bool("refreshing"));
    }
}
