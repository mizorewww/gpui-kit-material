//! The Examples page's playground: the example app on the selected device.
use super::super::pages::examples::app::{Body, Device, ExamplesApp, fit};
use super::super::playground::PaneHint;
use super::super::shell::{preview_area, preview_card_height};
use super::prelude::*;

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
    let fit = fit(
        outer,
        preview_area(window, false),
        preview_area(window, true),
    );
    preview.pane_hint(PaneHint {
        stacked: fit.stacked,
        height: Some(preview_card_height(outer.1 * fit.scale, window)),
    });
    let playground = cx.weak_entity();
    let app = preview.local("example-app", window, cx, move |window, cx| {
        ExamplesApp::new(playground, window, cx)
    });
    app.update(cx, |app, cx| {
        app.render_device(device, body, fit.scale, window, cx)
    })
}
