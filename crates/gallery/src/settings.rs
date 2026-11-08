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
    /// The theme mode's owner: the dark theme switch is here for a compact bar
    /// without room for its toggle.
    shared: Option<Entity<GalleryShared>>,
}

impl Render for Settings {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("gallery-settings-dialog")
            .debug_selector(|| "gallery-settings-dialog".into())
            .flex()
            .flex_col()
            .gap(px(12.))
            .when_some(self.shared.clone(), |content, shared| {
                let dark = shared.read(cx).painted_mode() == ThemeMode::Dark;
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
    }
}

#[cfg(test)]
mod tests {
    use super::Settings;
    use gpui::{AppContext as _, TestAppContext};

    #[gpui::test]
    fn the_dark_theme_switch_shows_with_the_shared_settings(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let shared = cx.new(|_| super::GalleryShared::default());
        let (_, cx) = cx.add_window_view(|_, _| Settings {
            shared: Some(shared),
        });
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert!(cx.debug_bounds("gallery-settings-dark").is_some());
    }
}
