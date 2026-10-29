//! Gallery settings hosted by Kit's dialog, including its focus trap and close handling.
use gpui::{prelude::*, *};
use gpui_kit::component::WindowExt as _;
use gpui_kit_material::components::MaterialSwitch;

pub(super) fn open(window: &mut Window, cx: &mut App) {
    // Allocate once per opening, not inside the dialog's repeated builder.
    let content = cx.new(|_| Settings { error: None });
    window.open_dialog(cx, move |dialog, _, _| {
        dialog.title("Settings").w(px(420.)).child(content.clone())
    });
}

struct Settings {
    error: Option<SharedString>,
}

impl Settings {
    fn change_vsync(&mut self, enabled: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.error = match window.set_vsync(enabled) {
            Err(error) => Some(format!("Could not change vertical sync: {error}").into()),
            Ok(()) => {
                let expected = if enabled {
                    VsyncStatus::Enabled
                } else {
                    VsyncStatus::Disabled
                };
                (window.vsync_status() != expected).then(|| {
                    "The platform did not confirm the requested vertical sync setting.".into()
                })
            }
        };
        // No optimistic checked state: render reads the backend after every request.
        cx.notify();
    }
}

impl Render for Settings {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = window.vsync_status();
        let unsupported = status == VsyncStatus::Unsupported;
        let explanation = match status {
            VsyncStatus::Enabled => "On. Frame presentation is synchronized with the display.",
            VsyncStatus::Disabled => "Off. Frame presentation synchronization is disabled.",
            VsyncStatus::Unsupported if cfg!(target_family = "wasm") => {
                "Controlled by the browser. This application cannot disable vertical sync."
            }
            VsyncStatus::Unsupported => "This platform does not expose vertical sync control.",
        };
        div()
            .id("gallery-settings-dialog")
            .debug_selector(|| "gallery-settings-dialog".into())
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .debug_selector(|| "gallery-settings-vsync".into())
                    .flex()
                    .items_center()
                    .justify_between()
                    .child("Vertical sync")
                    .child(
                        MaterialSwitch::new("gallery-settings-vsync-switch", "Vertical sync")
                            .checked(status == VsyncStatus::Enabled)
                            .disabled(unsupported)
                            .on_change(cx.listener(|this, enabled: &bool, window, cx| {
                                this.change_vsync(*enabled, window, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .debug_selector(|| "gallery-settings-vsync-status".into())
                    .text_sm()
                    .child(explanation),
            )
            .when_some(self.error.clone(), |content, error| {
                content.child(
                    div()
                        .id("gallery-settings-error")
                        .debug_selector(|| "gallery-settings-error".into())
                        .role(accesskit::Role::Alert)
                        .child(error),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::Settings;
    use gpui::{TestAppContext, VsyncStatus};

    #[gpui::test]
    fn unsupported_setting_reports_failure_without_a_fake_enabled_state(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let (settings, cx) = cx.add_window_view(|_, _| Settings { error: None });
        cx.update(|window, cx| {
            assert_eq!(window.vsync_status(), VsyncStatus::Unsupported);
            settings.update(cx, |settings, cx| settings.change_vsync(true, window, cx));
            assert!(settings.read(cx).error.is_some());
            assert_eq!(window.vsync_status(), VsyncStatus::Unsupported);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert!(cx.debug_bounds("gallery-settings-error").is_some());
        assert!(cx.debug_bounds("gallery-settings-vsync").is_some());
    }
}
