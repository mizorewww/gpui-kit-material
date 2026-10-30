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
//!
//! Both styles sit on the bar's vertical center, like everything else in it.
//! The glyphs are the platform's own shapes, not Material Symbols: symbols
//! have different optical sizes on their 24dp grid (`remove` is 7/12 of the
//! box, `crop_square` 3/4, and `filter_none` sits off-center), which at
//! caption size reads as a crooked row. Each glyph here fills its box
//! edge to edge and strokes on whole device pixels.
use crate::color::ColorRole::{self, *};
use crate::theme::color;
use crate::tokens::state;
use gpui::{
    AnyElement, App, Bounds, Decorations, ElementId, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement as _, PathBuilder, Pixels, RenderOnce, Rgba, SharedString,
    StatefulInteractiveElement as _, Styled, Window, WindowControlArea, canvas, div, fill, point,
    prelude::FluentBuilder as _, px, size,
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
/// 11 draws them; a taller bar centers them vertically with its content.
/// Logical pixels follow the display scaling; the text scale does not change
/// them.
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
/// The glyph box and stroke of each style: Windows draws 10px glyphs with
/// hairlines (Segoe Fluent's chrome glyphs), libadwaita 8px ones with a
/// heavier line in its round button.
const WINDOWS_GLYPH: f32 = 10.;
const WINDOWS_STROKE: f32 = 1.;
const LINUX_GLYPH: f32 = 8.;
const LINUX_STROKE: f32 = 1.5;

/// A platform color, not a Material role: Windows 11 turns a hovered
/// caption close button red (`#C42B1C`) behind a white glyph whatever the
/// app's palette, as its own title bars and WinUI 3's `TitleBar` caption
/// buttons do. Pressed keeps the red at 90% opacity.
const WINDOWS_CLOSE_RED: Rgba = Rgba {
    r: 0xc4 as f32 / 255.,
    g: 0x2b as f32 / 255.,
    b: 0x1c as f32 / 255.,
    a: 1.,
};
/// The glyph over [`WINDOWS_CLOSE_RED`].
const WINDOWS_CLOSE_GLYPH: Rgba = Rgba {
    r: 1.,
    g: 1.,
    b: 1.,
    a: 1.,
};

fn close_hover() -> Rgba {
    WINDOWS_CLOSE_RED
}

fn close_pressed() -> Rgba {
    Rgba {
        a: 0.9,
        ..WINDOWS_CLOSE_RED
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

    fn area(self) -> WindowControlArea {
        match self {
            Self::Minimize => WindowControlArea::Min,
            Self::Maximize | Self::Restore => WindowControlArea::Max,
            Self::Close => WindowControlArea::Close,
        }
    }
}

/// Accessible names of the caption buttons. The kit has no locale of its
/// own, so there is no default: applications pass their translations.
///
/// ```compile_fail
/// let _ = gpui_kit_material::components::CaptionLabels::default();
/// ```
#[derive(Clone, Debug)]
pub struct CaptionLabels {
    pub minimize: SharedString,
    pub maximize: SharedString,
    pub restore: SharedString,
    pub close: SharedString,
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

/// The buttons a preview shows: a window's full set, whatever the current
/// window offers, and none for a caption the system draws.
fn preview_buttons(caption: Caption) -> Vec<CaptionButton> {
    if caption == Caption::System {
        return Vec::new();
    }
    vec![
        CaptionButton::Minimize,
        CaptionButton::Maximize,
        CaptionButton::Close,
    ]
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

pub(super) type CloseRequest = Rc<dyn Fn(&mut Window, &mut App) -> bool>;

/// The caption buttons over the trailing end of a title bar, above any
/// dialog, or nothing where the system draws them. Render it once per
/// window, as a child of the root; a [`super::TitleBar`] renders its own, so
/// a window with one needs no other.
///
/// [`CaptionButtons::preview`] instead draws them in place, e.g. in a
/// catalog page: inert, in the page's flow, never over the real title bar.
#[derive(gpui::IntoElement)]
pub struct CaptionButtons {
    height: Pixels,
    labels: CaptionLabels,
    caption: Option<Caption>,
    on_close: Option<CloseRequest>,
    preview: Option<ElementId>,
}

impl CaptionButtons {
    /// Buttons for a title bar `height` tall, announced with `labels`.
    pub fn new(height: Pixels, labels: CaptionLabels) -> Self {
        Self {
            height,
            labels,
            caption: None,
            on_close: None,
            preview: None,
        }
    }

    /// Draw the buttons inline, where the parent lays them out, instead of
    /// pinning them to the window's top-right corner above every dialog. A
    /// preview shows minimize, maximize and close for its caption style
    /// (nothing for [`Caption::System`]) and keeps the hover and press
    /// states, but its buttons are not window control areas and do not act
    /// on the window. `id` keeps several previews in one window apart.
    pub fn preview(mut self, id: impl Into<ElementId>) -> Self {
        self.preview = Some(id.into());
        self
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

    /// [`Self::trailing`] for a forced caption style.
    pub(super) fn trailing_for_style(caption: Caption, window: &Window) -> f32 {
        trailing_for(width_of(caption, buttons(caption, window).len()))
    }
}

impl RenderOnce for CaptionButtons {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        let caption = self.caption.unwrap_or_else(|| Caption::detect(window));
        let live = self.preview.is_none();
        let buttons = if live {
            buttons(caption, window)
        } else {
            preview_buttons(caption)
        };
        if buttons.is_empty() {
            return div().into_any_element();
        }
        let glyph_color = color(if window.is_window_active() {
            ON_SURFACE
        } else {
            ON_SURFACE_VARIANT
        });
        let id = self
            .preview
            .clone()
            .unwrap_or_else(|| ElementId::from("window-caption"));
        let row = gpui_base::h_flex()
            .id(id)
            .flex_shrink_0()
            .text_color(glyph_color);
        let row = match caption {
            Caption::Windows => row.h(self.height).items_center().children(
                buttons
                    .into_iter()
                    .map(|b| windows_button(b, &self.labels, self.height, live)),
            ),
            _ => {
                row.h(self.height)
                    .items_center()
                    .pr(px(LINUX_END))
                    .gap(px(LINUX_GAP))
                    .children(buttons.into_iter().map(|button| {
                        linux_button(button, &self.labels, live, self.on_close.clone())
                    }))
            }
        };
        if live {
            caption_layer(row, window)
        } else {
            row.into_any_element()
        }
    }
}

/// Makes a control inside a title bar take the press itself. The whole bar
/// is a drag area: on Windows a press there arrives as a caption press, and
/// one no element stops goes on to the system's move-window loop (and
/// maximizes on a double click). Empty bar space still moves the window.
pub fn title_bar_control<E: gpui::InteractiveElement>(element: E) -> E {
    element.on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
}

/// The straight strokes of a glyph in a `glyph` px box, relative to the box.
/// Lines are `stroke` thick; [`CaptionButton::Close`] has none, its diagonals
/// are stroked as a path. The ink always fills the box, so it is centered in
/// the box up to one stroke's parity.
fn glyph_rects(button: CaptionButton, glyph: f32, stroke: f32) -> Vec<Bounds<Pixels>> {
    let rect = |x: f32, y: f32, width: f32, height: f32| Bounds {
        origin: point(px(x), px(y)),
        size: size(px(width), px(height)),
    };
    let frame = |x: f32, y: f32, side: f32| {
        [
            rect(x, y, side, stroke),
            rect(x, y + side - stroke, side, stroke),
            rect(x, y + stroke, stroke, side - 2. * stroke),
            rect(x + side - stroke, y + stroke, stroke, side - 2. * stroke),
        ]
    };
    match button {
        CaptionButton::Minimize => vec![rect(0., (glyph - stroke) / 2., glyph, stroke)],
        CaptionButton::Maximize => frame(0., 0., glyph).to_vec(),
        CaptionButton::Restore => {
            // The front window at the lower left, and what shows of the one
            // behind it: its top and right edges.
            let offset = (glyph * 0.2).round();
            let mut rects = frame(0., offset, glyph - offset).to_vec();
            rects.push(rect(offset, 0., glyph - offset, stroke));
            rects.push(rect(
                glyph - stroke,
                stroke,
                stroke,
                glyph - offset - stroke,
            ));
            rects
        }
        CaptionButton::Close => Vec::new(),
    }
}

/// A caption glyph in the surrounding text color, so it follows the button's
/// hover and press. Strokes stay on whole device pixels at any display scale.
fn glyph(button: CaptionButton, glyph: f32, stroke: f32) -> impl IntoElement {
    let ink_box = div()
        .when(cfg!(test), move |ink_box| {
            ink_box.debug_selector(move || format!("{}-glyph", button.id()))
        })
        .flex_shrink_0()
        .size(px(glyph));
    ink_box.child(
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let ink = window.text_style().color;
                let scale = window.scale_factor();
                let snap = |value: f32| (value * scale).round() / scale;
                let stroke = ((stroke * scale).round() / scale).max(1. / scale);
                let origin = point(
                    px(snap(f32::from(bounds.origin.x))),
                    px(snap(f32::from(bounds.origin.y))),
                );
                for rect in glyph_rects(button, glyph, stroke) {
                    let snapped = Bounds {
                        origin: point(
                            origin.x + px(snap(f32::from(rect.origin.x))),
                            origin.y + px(snap(f32::from(rect.origin.y))),
                        ),
                        size: rect.size,
                    };
                    window.paint_quad(fill(snapped, ink));
                }
                if button == CaptionButton::Close {
                    let side = px(glyph);
                    for (from, to) in [
                        (point(px(0.), px(0.)), point(side, side)),
                        (point(side, px(0.)), point(px(0.), side)),
                    ] {
                        let mut path = PathBuilder::stroke(px(stroke));
                        path.move_to(origin + from);
                        path.line_to(origin + to);
                        if let Ok(path) = path.build() {
                            window.paint_path(path, ink);
                        }
                    }
                }
            },
        )
        .size_full(),
    )
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
/// it, unless it is only a preview. It occludes the drag area behind it.
fn windows_button(
    button: CaptionButton,
    labels: &CaptionLabels,
    bar_height: Pixels,
    live: bool,
) -> gpui::Stateful<gpui::Div> {
    let close = button == CaptionButton::Close;
    let ink = color(ON_SURFACE);
    let (hover, pressed) = if close {
        (close_hover(), close_pressed())
    } else {
        let clear = Rgba { a: 0., ..ink };
        (state::hovered(clear, ink), state::pressed(clear, ink))
    };
    let on_close = WINDOWS_CLOSE_GLYPH;
    div()
        .id(button.id())
        .role(gpui::Role::Button)
        .aria_label(labels.get(button))
        .occlude()
        .when(live, |button_area| {
            button_area.window_control_area(button.area())
        })
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .w(px(WINDOWS_BUTTON_WIDTH))
        .h(px(WINDOWS_BUTTON_HEIGHT).min(bar_height))
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
        .when(cfg!(test), |area| {
            area.debug_selector(move || button.id().to_string())
        })
        .child(glyph(button, WINDOWS_GLYPH, WINDOWS_STROKE))
}

/// A Linux caption button: GPUI does not hit-test window controls there, so
/// the button acts itself, unless it is only a preview. As libadwaita draws
/// them, every button, close included, takes the same neutral hover and
/// press.
fn linux_button(
    button: CaptionButton,
    labels: &CaptionLabels,
    live: bool,
    on_close: Option<CloseRequest>,
) -> gpui::Stateful<gpui::Div> {
    let group = button.id();
    let shape: ColorRole = SURFACE_CONTAINER_HIGHEST;
    let (hover, pressed) = (
        state::hovered(color(shape), color(ON_SURFACE)),
        state::pressed(color(shape), color(ON_SURFACE)),
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
                .when(cfg!(test), |shape| {
                    shape.debug_selector(move || format!("{}-shape", button.id()))
                })
                .child(glyph(button, LINUX_GLYPH, LINUX_STROKE)),
        )
        .when(cfg!(test), |target| {
            target.debug_selector(move || button.id().to_string())
        })
        .on_mouse_down(MouseButton::Left, |_, window, cx| {
            window.prevent_default();
            cx.stop_propagation();
        })
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            if !live {
                return;
            }
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
pub(crate) mod tests {
    use super::*;
    use gpui::{Context, Modifiers, Render, TestAppContext};

    pub(crate) fn test_labels() -> CaptionLabels {
        CaptionLabels {
            minimize: "Minimize".into(),
            maximize: "Maximize".into(),
            restore: "Restore".into(),
            close: "Close".into(),
        }
    }

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
            let labels = test_labels();
            let row = gpui_base::h_flex().h(px(WINDOWS_BUTTON_HEIGHT)).children(
                [
                    CaptionButton::Minimize,
                    CaptionButton::Maximize,
                    CaptionButton::Close,
                ]
                .map(|button| windows_button(button, &labels, px(WINDOWS_BUTTON_HEIGHT), true)),
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

    /// Two previews in a page's flow, as a catalog shows them.
    struct PreviewHarness {
        page_presses: usize,
    }

    impl Render for PreviewHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .id("page")
                .size_full()
                .flex()
                .flex_col()
                .items_start()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, _| this.page_presses += 1),
                )
                .child(
                    div()
                        .debug_selector(|| "windows-preview".into())
                        .flex()
                        .child(
                            CaptionButtons::new(px(48.), test_labels())
                                .caption(Caption::Windows)
                                .preview("windows-preview"),
                        ),
                )
                .child(
                    div()
                        .debug_selector(|| "linux-preview".into())
                        .flex()
                        .child(
                            CaptionButtons::new(px(48.), test_labels())
                                .caption(Caption::Linux)
                                .on_close(|_, _| panic!("a preview must not close"))
                                .preview("linux-preview"),
                        ),
                )
                .child(
                    div().debug_selector(|| "system-preview".into()).child(
                        CaptionButtons::new(px(48.), test_labels())
                            .caption(Caption::System)
                            .preview("system-preview"),
                    ),
                )
        }
    }

    #[gpui::test]
    fn a_preview_lays_out_in_place_and_does_not_act_on_the_window(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| PreviewHarness { page_presses: 0 });
        cx.run_until_parked();
        let windows = cx.debug_bounds("windows-preview").unwrap();
        let linux = cx.debug_bounds("linux-preview").unwrap();
        let system = cx.debug_bounds("system-preview").unwrap();
        // In flow at the page's leading edge, not at the window's corner.
        assert_eq!(windows.origin, gpui::point(px(0.), px(0.)));
        assert_eq!(f32::from(windows.size.width), width_of(Caption::Windows, 3));
        assert_eq!(windows.size.height, px(48.));
        assert_eq!(linux.origin.y, windows.size.height);
        assert_eq!(f32::from(linux.size.width), width_of(Caption::Linux, 3));
        assert_eq!(system.size.height, px(0.));
        // Pressing the Linux preview's close button keeps the window open
        // and does not reach the page behind it.
        let close = gpui::point(
            linux.origin.x + px(width_of(Caption::Linux, 3) - LINUX_END - LINUX_BUTTON / 2.),
            linux.origin.y + px(24.),
        );
        cx.simulate_click(close, Modifiers::default());
        cx.update(|window, cx| {
            assert!(window.viewport_size().width > px(0.));
            assert_eq!(view.read(cx).page_presses, 0);
        });
    }

    /// Asserts one row of buttons: every button is on the bar's vertical
    /// center at the same size, and every glyph box on its button's center.
    fn assert_centered(
        cx: &mut gpui::VisualTestContext,
        (width, height): (f32, f32),
        glyph_size: f32,
        round: bool,
    ) {
        let bar = cx.debug_bounds("bar").expect("the bar is rendered");
        for button in [
            CaptionButton::Minimize,
            CaptionButton::Maximize,
            CaptionButton::Close,
        ] {
            let id = button.id();
            let selector = |suffix: &str| -> &'static str {
                Box::leak(format!("{id}{suffix}").into_boxed_str())
            };
            let target = cx.debug_bounds(id).expect("the button is rendered");
            assert_eq!(target.size, size(px(width), px(height)), "{id}");
            assert_eq!(
                target.center().y,
                bar.center().y,
                "{id} on the bar's center"
            );
            let glyph = cx.debug_bounds(selector("-glyph")).expect("a glyph");
            assert_eq!(glyph.size, size(px(glyph_size), px(glyph_size)), "{id}");
            assert_eq!(glyph.center(), target.center(), "{id}'s glyph is centered");
            if round {
                let shape = cx.debug_bounds(selector("-shape")).expect("a shape");
                assert_eq!(shape.center(), target.center(), "{id}'s shape is centered");
            }
        }
    }

    /// One caption style in a bar.
    struct BarHarness(Caption, Pixels);

    impl Render for BarHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .debug_selector(|| "bar".into())
                .flex()
                .w_full()
                .h(self.1)
                .justify_end()
                .child(
                    CaptionButtons::new(self.1, test_labels())
                        .caption(self.0)
                        .preview("preview"),
                )
        }
    }

    #[gpui::test]
    fn windows_buttons_and_glyphs_center_in_the_bar(cx: &mut TestAppContext) {
        for height in [32., 40., 48., 56., 64.] {
            let (_, cx) = cx.add_window_view(|_, _| BarHarness(Caption::Windows, px(height)));
            cx.run_until_parked();
            assert_centered(cx, (46., 32.), 10., false);
        }
    }

    #[gpui::test]
    fn linux_buttons_and_glyphs_center_in_the_bar(cx: &mut TestAppContext) {
        for height in [32., 40., 48., 56., 64.] {
            let (_, cx) = cx.add_window_view(|_, _| BarHarness(Caption::Linux, px(height)));
            cx.run_until_parked();
            assert_centered(cx, (32., 32.), 8., true);
        }
    }

    #[gpui::test]
    fn a_bar_shorter_than_the_windows_button_clamps_it(cx: &mut TestAppContext) {
        let (_, cx) = cx.add_window_view(|_, _| BarHarness(Caption::Windows, px(28.)));
        cx.run_until_parked();
        assert_centered(cx, (46., 28.), 10., false);
    }

    #[test]
    fn every_glyph_fills_its_box_symmetrically() {
        for (glyph, stroke) in [(10., 1.), (8., 2.), (8., 1.5)] {
            for button in [
                CaptionButton::Minimize,
                CaptionButton::Maximize,
                CaptionButton::Restore,
            ] {
                let rects = glyph_rects(button, glyph, stroke);
                let (mut left, mut top, mut right, mut bottom) = (glyph, glyph, 0_f32, 0_f32);
                for rect in &rects {
                    left = left.min(f32::from(rect.origin.x));
                    top = top.min(f32::from(rect.origin.y));
                    right = right.max(f32::from(rect.origin.x + rect.size.width));
                    bottom = bottom.max(f32::from(rect.origin.y + rect.size.height));
                }
                assert_eq!((left, right), (0., glyph), "{button:?} spans the box");
                assert_eq!(
                    (top + bottom) / 2.,
                    glyph / 2.,
                    "{button:?} is centered vertically"
                );
                if button != CaptionButton::Minimize {
                    assert_eq!((top, bottom), (0., glyph), "{button:?} fills the box");
                }
            }
        }
        // The X is two strokes corner to corner, centered by construction.
        assert!(glyph_rects(CaptionButton::Close, 10., 1.).is_empty());
    }

    #[test]
    fn a_preview_shows_the_full_set_whatever_the_window_offers() {
        assert_eq!(
            preview_buttons(Caption::Windows),
            [
                CaptionButton::Minimize,
                CaptionButton::Maximize,
                CaptionButton::Close
            ]
        );
        assert_eq!(preview_buttons(Caption::Linux).len(), 3);
        assert!(preview_buttons(Caption::System).is_empty());
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
