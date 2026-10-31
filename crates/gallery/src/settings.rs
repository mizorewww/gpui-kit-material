//! Gallery settings hosted by Kit's dialog, including its focus trap and close handling.
use gpui::{prelude::*, *};
use gpui_kit::component::WindowExt as _;
use gpui_kit_material::color::themes::ThemeMode;
use gpui_kit_material::components::MaterialSwitch;

use super::page::GalleryShared;
use super::top_bar;

/// The dialog's width, where the window has room; otherwise the window less
/// this margin on each side.
const DIALOG_WIDTH: f32 = 420.;
const DIALOG_MARGIN: f32 = 16.;

pub(super) fn open(shared: Entity<GalleryShared>, window: &mut Window, cx: &mut App) {
    // Allocate once per opening, not inside the dialog's repeated builder.
    let content = cx.new(|cx| {
        cx.observe(&shared, |_, _, cx| cx.notify()).detach();
        Settings {
            error: None,
            shared: Some(shared),
        }
    });
    window.open_dialog(cx, move |dialog, window, _| {
        let room = window.viewport_size().width - px(2. * DIALOG_MARGIN);
        dialog
            .title("Settings")
            .w(px(DIALOG_WIDTH).min(room))
            .child(content.clone())
    });
}

struct Settings {
    error: Option<SharedString>,
    /// The theme mode's owner: the dark theme switch is here for a compact bar
    /// without room for its toggle.
    shared: Option<Entity<GalleryShared>>,
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
            .when_some(self.shared.clone(), |content, shared| {
                let dark = shared.read(cx).reference_mode == ThemeMode::Dark;
                content.child(
                    div()
                        .debug_selector(|| "gallery-settings-dark".into())
                        .flex()
                        .items_center()
                        .justify_between()
                        .child("Dark theme")
                        .child(
                            MaterialSwitch::new("gallery-settings-dark-switch", "Dark theme")
                                .checked(dark)
                                .on_change(move |dark: &bool, _, cx| {
                                    top_bar::set_dark(&shared, *dark, cx)
                                }),
                        ),
                )
            })
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
        let (settings, cx) = cx.add_window_view(|_, _| Settings {
            error: None,
            shared: None,
        });
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
