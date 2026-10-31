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
        .items_center()
        .child(
            div()
                .debug_selector(|| "example-device-scale".into())
                .h(rem(SCALE_LABEL))
                .flex()
                .items_center()
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
