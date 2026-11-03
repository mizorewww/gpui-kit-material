//! The Examples page's playground: the example app on the selected device.
use super::super::pages::examples::app::{Body, Device, ExamplesApp, fit};
use super::super::playground::PaneHint;
use super::super::shell::{ShellLayout, flush_preview_area, preview_area, preview_card_height};
use super::prelude::*;

/// Below this scale a device's touch targets are too small to use, as a Tablet
/// or Desktop is on a phone (Phone, the default device, fits one).
const USABLE_SCALE: f32 = 0.6;
/// The height of the line saying so, in design px.
const SCALE_LABEL: f32 = 32.;

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[
        Scenario {
            name: "Phone messages",
            values: &[
                ("device", Set::Option("Phone")),
                ("screen", Set::Option("Messaging")),
            ],
        },
        Scenario {
            name: "Tablet library",
            values: &[
                ("device", Set::Option("Tablet")),
                ("screen", Set::Option("Library")),
            ],
        },
        Scenario {
            name: "Desktop gallery",
            values: &[
                ("device", Set::Option("Desktop")),
                ("screen", Set::Option("Gallery")),
            ],
        },
    ],
    sections: &[Section {
        title: "Example",
        controls: &[
            Control::Segmented {
                id: "device",
                label: "Device",
                options: &Device::NAMES,
                default: 0,
            },
            Control::Select {
                id: "screen",
                label: "Screen",
                options: &Body::NAMES,
                default: 0,
            },
        ],
    }],
    preview,
    code: Some(code),
};

fn preview(
    config: &Config,
    preview: &mut Preview,
    window: &mut Window,
    cx: &mut Context<PlaygroundState>,
) -> AnyElement {
    let device = Device::ALL[config.choice("device")];
    let body = Body::ALL[config.choice("screen")];
    let outer = device.outer();
    // On a phone the device fills the card, its frame standing in for padding.
    let flush = ShellLayout::of(window).compact();
    let area = if flush {
        flush_preview_area
    } else {
        preview_area
    };
    let fit = fit(outer, area(window, false), area(window, true));
    // Too small to use by touch: it is a picture of that device, and says so.
    let scaled = fit.scale < USABLE_SCALE;
    preview.pane_hint(PaneHint {
        stacked: fit.stacked,
        height: Some(preview_card_height(
            outer.1 * fit.scale + if scaled { SCALE_LABEL } else { 0. },
            window,
        )),
        flush,
        fits_itself: true,
    });
    let playground = cx.weak_entity();
    let app = preview.local("example-app", window, cx, move |window, cx| {
        ExamplesApp::new(playground, window, cx)
    });
    let shown = DeviceShown {
        device,
        body,
        scale: fit.scale,
    };
    let view = preview.local("example-device", window, cx, move |_, _| DeviceView {
        app,
        shown,
    });
    // Not `view.read`: what this render reads, it must render again for, and the
    // device's own notifications are what this view keeps from the Gallery.
    if preview.changed(
        "example-shown",
        (device.index(), body.index(), fit.scale.to_bits()),
    ) {
        view.update(cx, |view, cx| {
            view.shown = shown;
            cx.notify();
        });
    }
    // The device is its own view: scrolling and tapping inside it notify the
    // app, which only this view reads, so the Gallery around it is left alone.
    let scaled_rem = window.rem_size() * fit.scale;
    let device = AnyView::from(view).cached(
        StyleRefinement::default()
            .w(scaled_rem * outer.0 / BASE_PX)
            .h(scaled_rem * outer.1 / BASE_PX),
    );
    if !scaled {
        return device.into_any_element();
    }
    gpui_base::v_flex()
        .w_full()
        .items_center()
        .child(
            // Wraps where a large text scale leaves it no room on one line.
            div()
                .debug_selector(|| "example-device-scale".into())
                .w_full()
                .min_h(rem(SCALE_LABEL))
                .flex()
                .items_center()
                .justify_center()
                .whitespace_normal()
                .text_center()
                .map(|text| type_role(text, TypeRole::LabelMedium))
                .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                .child(SharedString::from(format!(
                    "Shown at {}%. Use a wider window to try it.",
                    (fit.scale * 100.).round()
                ))),
        )
        .child(device)
        .into_any_element()
}

/// What the device view shows.
#[derive(Clone, Copy)]
struct DeviceShown {
    device: Device,
    body: Body,
    scale: f32,
}

/// The device, with the app it runs.
struct DeviceView {
    app: Entity<ExamplesApp>,
    shown: DeviceShown,
}

impl Render for DeviceView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let DeviceShown {
            device,
            body,
            scale,
        } = self.shown;
        self.app.update(cx, |app, cx| {
            app.render_device(device, body, scale, window, cx)
        })
    }
}

// The app's frame: the screen beside or above its navigation. A screen's own
// content is the app's (`self.home(window, cx)` and the like), and the device
// frame and its scale are the Gallery's.
fn code(config: &Config) -> String {
    use super::super::pages::examples::app::{NAV, nav_value};
    let device = Device::ALL[config.choice("device")];
    let body = Body::ALL[config.choice("screen")];
    let screen = format!(
        "self.{}(window, cx)",
        Body::NAMES[body.index()].to_lowercase()
    );
    let items = NAV.iter().map(|(_, value, label, icon)| {
        format!(
            "NavItem::new({}, {}, NavIcon::symbol({}))",
            lit(value),
            lit(label),
            symbol_code(icon)
        )
    });
    let on_change = "on_change(cx.listener(|this, value: &SharedString, _, cx| {\n    this.destination = value.clone();\n    cx.notify();\n}))";
    // Screens pushed over the destinations, such as a detail, have no navigation.
    let has_nav = !nav_value(body).is_empty();
    if device == Device::Phone {
        let bar = chain(
            "NavigationBar::new(\"navigation\", \"Destinations\")",
            [
                Some(format!("items({})", array(items))),
                Some("selected(self.destination.clone())".into()),
                Some(on_change.into()),
            ],
        );
        return with_imports(chain(
            "div()",
            [
                Some("flex()".into()),
                Some("flex_col()".into()),
                Some("size_full()".into()),
                Some(format!("child(div().flex_1().min_h_0().child({screen}))")),
                has_nav.then(|| format!("child({bar})")),
            ],
        ));
    }
    let rail = chain(
        "NavigationRail::new(\"rail\", \"Destinations\")",
        [
            Some(format!("items({})", array(items))),
            Some("selected(self.destination.clone())".into()),
            Some("expanded(self.rail_expanded)".into()),
            Some(format!(
                "menu({})",
                chain(
                    format!(
                        "IconButton::new(\"rail-menu\", {}, \"Expand navigation\")",
                        symbol_code("menu")
                    ),
                    [Some(
                        "on_click(cx.listener(|this, _: &ClickEvent, _, cx| {\n    this.rail_expanded = !this.rail_expanded;\n    cx.notify();\n}))"
                            .into(),
                    )],
                )
            )),
            Some(format!(
                "fab(ExtendedFab::new(\"compose\", {}, \"Compose\").expanded(self.rail_expanded))",
                symbol_code("edit")
            )),
            Some(on_change.into()),
        ],
    );
    // Messaging lays out panes of its own, without the rail.
    with_imports(chain(
        "div()",
        [
            Some("flex()".into()),
            Some("size_full()".into()),
            (body != Body::Messaging).then(|| format!("child({rail})")),
            Some(format!("child(div().flex_1().min_w_0().child({screen}))")),
        ],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_code_follows_the_configuration() {
        let mut config = Config::defaults(&PLAYGROUND);
        let code = super::code(&config);
        assert!(
            code.starts_with("use gpui_kit_material::icons::Symbol;\n\ndiv()"),
            "{code}"
        );
        assert!(code.contains("NavigationBar::new(\"navigation\", \"Destinations\")"));
        assert!(code.contains("self.home(window, cx)"));
        assert!(!code.contains("NavigationRail::new("));
        config.set("device", Value::Choice(2));
        config.set("screen", Value::Choice(4));
        let code = super::code(&config);
        assert!(code.contains("NavigationRail::new(\"rail\", \"Destinations\")"));
        assert!(code.contains("self.library(window, cx)"));
        assert!(!code.contains("NavigationBar::new("));
    }
}
