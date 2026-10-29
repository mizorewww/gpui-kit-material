//! Caption buttons for a window that draws its own title bar.
//!
//! macOS keeps its native traffic lights in the bar's leading edge. Windows
//! has nothing of its own left once the system title bar is hidden, and
//! Linux under client-side decorations neither, so [`CaptionButtons`] draws
//! minimize, maximize or restore, and close at the trailing edge, once for
//! the whole window.
//!
//! They paint and hit-test above pages and dialogs: GPUI Kit dialogs are
//! deferred layers (priority 10 and up) whose backdrop is itself a drag
//! area, so the buttons are a deferred layer of a higher priority, the way
//! the system's own buttons stay usable over an app's modal content. Menus,
//! popovers (`gpui_base::POPUP_PRIORITY`, 100 and up) and tooltips stay
//! above them: where an open menu reaches the top of the window, a press
//! there chooses its item.
//!
//! On Windows the buttons are window control areas: GPUI answers the
//! system's non-client hit test with `HTMINBUTTON` / `HTMAXBUTTON` /
//! `HTCLOSE` for them, so Windows itself minimizes, maximizes (with the snap
//! layouts flyout) and closes. On Linux the buttons act themselves.
use crate::color::ColorRole::{self, *};
use crate::icons::IconName;
use crate::theme::{blend, color};
use gpui::{
    AnyElement, App, Decorations, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, Pixels, RenderOnce, Rgba, SharedString, StatefulInteractiveElement as _,
    Styled, Window, WindowControlArea, div, px, rgb,
};
use std::rc::Rc;

/// Which caption buttons the window needs from the application.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Caption {
    /// The system draws them: macOS traffic lights, or a Linux window
    /// manager's server-side title bar.
    System,
    /// Windows: 46×32px buttons along the top of the bar's trailing edge.
    Windows,
    /// Linux under client-side decorations: round buttons, as GNOME and KDE
    /// draw them.
    Linux,
}

impl Caption {
    /// The caption this window needs on the current platform.
    pub fn detect(window: &Window) -> Self {
        if cfg!(target_os = "windows") {
            Caption::Windows
        } else if cfg!(any(target_os = "linux", target_os = "freebsd"))
            && matches!(window.window_decorations(), Decorations::Client { .. })
        {
            Caption::Linux
        } else {
            Caption::System
        }
    }
}

/// Windows caption buttons are 46×32px at 100% display scaling, as Windows
/// 11 draws them, top-aligned whatever the bar's height. Logical pixels
/// follow the display scaling; the text scale does not change them.
const WINDOWS_BUTTON_WIDTH: f32 = 46.;
const WINDOWS_BUTTON_HEIGHT: f32 = 32.;
/// Above every dialog layer, below popups and tooltips.
const CAPTION_PRIORITY: usize = 50;
/// A Linux caption button's target, the round shape inside it, the gap
/// between two of them and the space after the last.
const LINUX_BUTTON: f32 = 32.;
const LINUX_SHAPE: f32 = 24.;
const LINUX_GAP: f32 = 4.;
const LINUX_END: f32 = 12.;
/// The glyphs are drawn on a 10px grid.
const GLYPH: f32 = 10.;

/// The close button turns the platform's red with white glyphs on Windows,
/// whatever the palette.
fn close_hover() -> Rgba {
    rgb(0xc42b1c)
}

fn close_pressed() -> Rgba {
    Rgba {
        a: 0.9,
        ..rgb(0xc42b1c)
    }
}

/// One caption button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptionButton {
    Minimize,
    Maximize,
    Restore,
    Close,
}

impl CaptionButton {
    fn id(self) -> &'static str {
        match self {
            Self::Minimize => "caption-minimize",
            Self::Maximize => "caption-maximize",
            Self::Restore => "caption-restore",
            Self::Close => "caption-close",
        }
    }

    fn glyph(self) -> IconName {
        match self {
            Self::Minimize => IconName::CaptionMinimize,
            Self::Maximize => IconName::CaptionMaximize,
            Self::Restore => IconName::CaptionRestore,
            Self::Close => IconName::CaptionClose,
        }
    }

    fn area(self) -> WindowControlArea {
        match self {
            Self::Minimize => WindowControlArea::Min,
            Self::Maximize | Self::Restore => WindowControlArea::Max,
            Self::Close => WindowControlArea::Close,
        }
    }
}

/// Accessible names of the caption buttons. The kit has no locale of its
/// own; applications pass their translations.
#[derive(Clone, Debug)]
pub struct CaptionLabels {
    pub minimize: SharedString,
    pub maximize: SharedString,
    pub restore: SharedString,
    pub close: SharedString,
}

impl Default for CaptionLabels {
    fn default() -> Self {
        Self {
            minimize: "Minimize".into(),
            maximize: "Maximize".into(),
            restore: "Restore".into(),
            close: "Close".into(),
        }
    }
}

impl CaptionLabels {
    fn get(&self, button: CaptionButton) -> SharedString {
        match button {
            CaptionButton::Minimize => self.minimize.clone(),
            CaptionButton::Maximize => self.maximize.clone(),
            CaptionButton::Restore => self.restore.clone(),
            CaptionButton::Close => self.close.clone(),
        }
    }
}

/// The buttons in order, as far as the window and its window manager allow.
fn buttons(caption: Caption, window: &Window) -> Vec<CaptionButton> {
    if caption == Caption::System || window.is_fullscreen() {
        return Vec::new();
    }
    let (minimize, maximize) = match caption {
        // Windows itself acts on these buttons.
        Caption::Windows => (true, true),
        _ => {
            let supported = window.window_controls();
            (supported.minimize, supported.maximize)
        }
    };
    let zoom = if window.is_maximized() {
        CaptionButton::Restore
    } else {
        CaptionButton::Maximize
    };
    [
        minimize.then_some(CaptionButton::Minimize),
        maximize.then_some(zoom),
        Some(CaptionButton::Close),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn width_of(caption: Caption, count: usize) -> f32 {
    match (caption, count) {
        (_, 0) | (Caption::System, _) => 0.,
        (Caption::Windows, count) => WINDOWS_BUTTON_WIDTH * count as f32,
        (Caption::Linux, count) => {
            LINUX_BUTTON * count as f32 + LINUX_GAP * (count - 1) as f32 + LINUX_END
        }
    }
}

fn trailing_for(caption_width: f32) -> f32 {
    if caption_width > 0. {
        caption_width + 8.
    } else {
        16.
    }
}

type CloseRequest = Rc<dyn Fn(&mut Window, &mut App) -> bool>;

/// The caption buttons over the trailing end of a title bar, above any
/// dialog, or nothing where the system draws them. Render it once per
/// window, as a child of the root.
#[derive(IntoElement)]
pub struct CaptionButtons {
    height: Pixels,
    labels: CaptionLabels,
    caption: Option<Caption>,
    on_close: Option<CloseRequest>,
}

impl CaptionButtons {
    /// Buttons for a title bar `height` tall.
    pub fn new(height: Pixels) -> Self {
        Self {
            height,
            labels: CaptionLabels::default(),
            caption: None,
            on_close: None,
        }
    }

    pub fn labels(mut self, labels: CaptionLabels) -> Self {
        self.labels = labels;
        self
    }

    /// Force a caption style instead of detecting it, e.g. for screenshots.
    pub fn caption(mut self, caption: Caption) -> Self {
        self.caption = Some(caption);
        self
    }

    /// Asked before the Linux close button removes the window; return
    /// `false` to keep it (e.g. to save first). Windows routes close through
    /// the system, which the application observes as a window close.
    pub fn on_close(mut self, request: impl Fn(&mut Window, &mut App) -> bool + 'static) -> Self {
        self.on_close = Some(Rc::new(request));
        self
    }

    /// The width the buttons take from the title bar's trailing edge.
    pub fn width(window: &Window) -> f32 {
        let caption = Caption::detect(window);
        width_of(caption, buttons(caption, window).len())
    }

    /// Where the title bar's own content should end: before the buttons with
    /// a little open space, or a 16px inset where the system draws them.
    pub fn trailing(window: &Window) -> f32 {
        trailing_for(Self::width(window))
    }
}

impl RenderOnce for CaptionButtons {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        let caption = self.caption.unwrap_or_else(|| Caption::detect(window));
        let buttons = buttons(caption, window);
        if buttons.is_empty() {
            return div().into_any_element();
        }
        let glyph_color = color(if window.is_window_active() {
            ON_SURFACE
        } else {
            ON_SURFACE_VARIANT
        });
        let row = gpui_base::h_flex()
            .id("window-caption")
            .text_color(glyph_color);
        let row = match caption {
            Caption::Windows => row
                .h(px(WINDOWS_BUTTON_HEIGHT).min(self.height))
                .children(buttons.into_iter().map(|b| windows_button(b, &self.labels))),
            _ => row
                .h(self.height)
                .items_center()
                .pr(px(LINUX_END))
                .gap(px(LINUX_GAP))
                .children(
                    buttons
                        .into_iter()
                        .map(|button| linux_button(button, &self.labels, self.on_close.clone())),
                ),
        };
        caption_layer(row, window)
    }
}

/// Makes a control inside a title bar take the press itself. The whole bar
/// is a drag area: on Windows a press there arrives as a caption press, and
/// one no element stops goes on to the system's move-window loop (and
/// maximizes on a double click). Empty bar space still moves the window.
pub fn title_bar_control<E: gpui::InteractiveElement>(element: E) -> E {
    element.on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
}

/// The row at the window's top-right corner, after every dialog layer.
fn caption_layer(row: impl IntoElement, window: &Window) -> AnyElement {
    let trailing = gpui::point(window.viewport_size().width, px(0.));
    gpui::deferred(
        gpui::anchored()
            .anchor(gpui::Anchor::TopRight)
            .position(trailing)
            .child(row),
    )
    .with_priority(CAPTION_PRIORITY)
    .into_any_element()
}

/// A Windows caption button: a window control area, so the system acts on
/// it. It occludes the drag area behind it.
fn windows_button(button: CaptionButton, labels: &CaptionLabels) -> gpui::Stateful<gpui::Div> {
    let close = button == CaptionButton::Close;
    let ink = color(ON_SURFACE);
    let (hover, pressed) = if close {
        (close_hover(), close_pressed())
    } else {
        (Rgba { a: 0.08, ..ink }, Rgba { a: 0.10, ..ink })
    };
    let on_close = rgb(0xffffff);
    div()
        .id(button.id())
        .role(gpui::Role::Button)
        .aria_label(labels.get(button))
        .occlude()
        .window_control_area(button.area())
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .w(px(WINDOWS_BUTTON_WIDTH))
        .h_full()
        .hover(move |style| {
            let style = style.bg(hover);
            if close {
                style.text_color(on_close)
            } else {
                style
            }
        })
        .active(move |style| {
            let style = style.bg(pressed);
            if close {
                style.text_color(on_close)
            } else {
                style
            }
        })
        .child(button.glyph().icon().size(px(GLYPH)))
}

/// A Linux caption button: GPUI does not hit-test window controls there, so
/// the button acts itself. As libadwaita draws them, every button, close
/// included, takes the same neutral hover and press.
fn linux_button(
    button: CaptionButton,
    labels: &CaptionLabels,
    on_close: Option<CloseRequest>,
) -> gpui::Stateful<gpui::Div> {
    let group = button.id();
    let shape: ColorRole = SURFACE_CONTAINER_HIGHEST;
    let (hover, pressed) = (
        blend(color(shape), color(ON_SURFACE), 0.08),
        blend(color(shape), color(ON_SURFACE), 0.10),
    );
    div()
        .id(button.id())
        .group(group)
        .role(gpui::Role::Button)
        .aria_label(labels.get(button))
        .occlude()
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .size(px(LINUX_BUTTON))
        .child(
            div()
                .id("shape")
                .flex()
                .items_center()
                .justify_center()
                .size(px(LINUX_SHAPE))
                .rounded_full()
                .bg(color(shape))
                .group_hover(group, move |style| style.bg(hover))
                .group_active(group, move |style| style.bg(pressed))
                .child(button.glyph().icon().size(px(GLYPH))),
        )
        .on_mouse_down(MouseButton::Left, |_, window, cx| {
            window.prevent_default();
            cx.stop_propagation();
        })
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            match button {
                CaptionButton::Minimize => window.minimize_window(),
                CaptionButton::Maximize | CaptionButton::Restore => window.zoom_window(),
                CaptionButton::Close => {
                    if on_close.as_ref().is_none_or(|request| request(window, cx)) {
                        window.remove_window();
                    }
                }
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Context, Modifiers, Render, TestAppContext};

    /// A modal dialog as gpui-component lays it out: a deferred layer at
    /// priority 10 whose full-window backdrop takes every press.
    struct DialogHarness {
        backdrop_presses: usize,
        menu_presses: usize,
    }

    impl Render for DialogHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let backdrop = div()
                .id("backdrop")
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .occlude()
                .window_control_area(WindowControlArea::Drag)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, _| this.backdrop_presses += 1),
                );
            // A tall menu clamped to the window's top-right corner, over the
            // close button, as gpui-component's popups place it.
            let menu = div()
                .id("menu")
                .absolute()
                .top(px(8.))
                .right(px(8.))
                .w(px(20.))
                .h(px(200.))
                .occlude()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, _| this.menu_presses += 1),
                );
            let labels = CaptionLabels::default();
            let row = gpui_base::h_flex().h(px(WINDOWS_BUTTON_HEIGHT)).children(
                [
                    CaptionButton::Minimize,
                    CaptionButton::Maximize,
                    CaptionButton::Close,
                ]
                .map(|button| windows_button(button, &labels)),
            );
            div()
                .size_full()
                .child(gpui::deferred(backdrop).with_priority(10))
                .child(caption_layer(row, window))
                .child(gpui::deferred(menu).with_priority(gpui_base::POPUP_PRIORITY))
        }
    }

    #[gpui::test]
    fn caption_buttons_take_presses_over_a_dialog_backdrop(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| DialogHarness {
            backdrop_presses: 0,
            menu_presses: 0,
        });
        cx.run_until_parked();
        let width = cx.update(|window, _| f32::from(window.viewport_size().width));
        // Close (where the menu leaves it), maximize and minimize, from the
        // trailing edge.
        for offset in [36., 69., 115.] {
            cx.simulate_click(
                gpui::point(px(width - offset), px(16.)),
                Modifiers::default(),
            );
        }
        cx.update(|_, cx| assert_eq!(view.read(cx).backdrop_presses, 0));
        // Beside them the dialog's backdrop still takes the press.
        cx.simulate_click(gpui::point(px(width - 160.), px(16.)), Modifiers::default());
        // Below the 32px buttons, too.
        cx.simulate_click(gpui::point(px(width - 69.), px(40.)), Modifiers::default());
        cx.update(|_, cx| assert_eq!(view.read(cx).backdrop_presses, 2));
        // An open menu over a caption button takes the press itself.
        cx.simulate_click(gpui::point(px(width - 18.), px(16.)), Modifiers::default());
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).menu_presses, 1);
            assert_eq!(view.read(cx).backdrop_presses, 2);
        });
    }

    #[test]
    fn windows_buttons_take_the_system_width_and_leave_open_space_before_them() {
        assert_eq!(width_of(Caption::Windows, 3), 138.);
        assert_eq!(trailing_for(width_of(Caption::Windows, 3)), 146.);
    }

    #[test]
    fn a_system_title_bar_keeps_the_ordinary_inset() {
        assert_eq!(width_of(Caption::System, 3), 0.);
        assert_eq!(trailing_for(0.), 16.);
    }

    #[test]
    fn linux_buttons_follow_what_the_window_manager_offers() {
        // Close alone on a tiling compositor, all three otherwise.
        assert_eq!(width_of(Caption::Linux, 1), 44.);
        assert_eq!(width_of(Caption::Linux, 3), 32. * 3. + 4. * 2. + 12.);
    }
}
