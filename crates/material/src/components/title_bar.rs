//! A title bar the application draws in its own colors.
//!
//! The system title bar is made transparent and the window's content draws
//! the bar, so it always matches the page beneath it in every palette. macOS
//! keeps its traffic lights, centred in the bar; Windows and Linux get
//! [`CaptionButtons`]. The bar moves the window when dragged and zooms it on a
//! double click, the way each platform's own title bar does. Controls placed
//! in it take their presses with [`title_bar_control`].
use super::caption::{Caption, CaptionButtons, CaptionLabels, CloseRequest};
use gpui::{
    AnyElement, App, Bounds, InteractiveElement as _, IntoElement, MouseButton, ParentElement,
    Pixels, Point, RenderOnce, SharedString, Stateful, StatefulInteractiveElement as _,
    StyleRefinement, Styled, TitlebarOptions, Window, WindowBounds, WindowControlArea,
    WindowDecorations, WindowOptions, div, point, px,
};
use std::rc::Rc;

/// The macOS traffic lights: three 14px buttons, 6px apart.
const TRAFFIC_LIGHT: f32 = 14.;
const TRAFFIC_LIGHT_GAP: f32 = 6.;
/// The lights sit at most this far from the window's leading edge.
const TRAFFIC_LIGHT_MAX_INSET: f32 = 20.;
/// Open space between the lights, or the window edge, and the bar's content.
const CONTENT_GAP: f32 = 16.;

/// Where the traffic lights go in a bar `height` tall: centred vertically,
/// and as far from the leading edge as from the top, up to 20px.
pub fn traffic_light_position(height: Pixels) -> Point<Pixels> {
    let inset = ((f32::from(height) - TRAFFIC_LIGHT) / 2.).max(0.);
    point(px(inset.min(TRAFFIC_LIGHT_MAX_INSET)), px(inset))
}

/// Window options for a window whose content draws a [`TitleBar`] `height`
/// tall, centred on the main display at `window_size`. Extend them with
/// struct update syntax for anything else the window needs.
///
/// The application owns title-bar drags: otherwise AppKit also treats the bar
/// as a native title bar, zooming twice on a double click and delaying every
/// click in it. Linux windows use client-side decorations.
pub fn title_bar_window_options(
    title: impl Into<SharedString>,
    height: Pixels,
    window_size: gpui::Size<Pixels>,
    cx: &App,
) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            window_size,
            cx,
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some(title.into()),
            appears_transparent: true,
            traffic_light_position: Some(traffic_light_position(height)),
        }),
        app_owns_titlebar_drag: true,
        window_decorations: Some(WindowDecorations::Client),
        ..Default::default()
    }
}

/// Where the bar's own content starts: after the macOS traffic lights, or a
/// 16px inset where the platform puts its buttons at the trailing edge.
pub fn title_bar_leading(height: Pixels, window: &Window) -> f32 {
    leading_for(cfg!(target_os = "macos") && !window.is_fullscreen(), height)
}

/// The insets a bar leaves for the platform's buttons, leading and trailing,
/// with its caption style forced or detected. A forced Windows or Linux style
/// has its buttons at the trailing edge, so it leaves no room for traffic
/// lights, which real Windows and Linux windows never have.
fn insets(height: Pixels, caption: Option<Caption>, window: &Window) -> (f32, f32) {
    match caption {
        Some(style @ (Caption::Windows | Caption::Linux)) => (
            leading_for(false, height),
            CaptionButtons::trailing_for_style(style, window),
        ),
        Some(style) => (
            title_bar_leading(height, window),
            CaptionButtons::trailing_for_style(style, window),
        ),
        None => (
            title_bar_leading(height, window),
            CaptionButtons::trailing(window),
        ),
    }
}

/// The width a bar `height` tall leaves for its children in `width`, with its
/// caption style forced or detected, so a crowded bar can decide what to show.
pub fn title_bar_room(
    width: Pixels,
    height: Pixels,
    caption: Option<Caption>,
    window: &Window,
) -> Pixels {
    let (leading, trailing) = insets(height, caption, window);
    width - px(leading + trailing)
}

fn leading_for(traffic_lights: bool, height: Pixels) -> f32 {
    if traffic_lights {
        f32::from(traffic_light_position(height).x)
            + 3. * TRAFFIC_LIGHT
            + 2. * TRAFFIC_LIGHT_GAP
            + CONTENT_GAP
    } else {
        CONTENT_GAP
    }
}

/// An application-drawn window title bar. Lay out its children with the usual
/// style methods; it adds the leading and trailing space the platform's
/// buttons need and draws [`CaptionButtons`] where the platform has none.
/// Open the window with [`title_bar_window_options`] for the same height.
///
/// The bar sets its own height and horizontal padding after the caller's
/// styles, so `.h()`, `.px()`, `.pl()` and `.pr()` on it have no effect.
#[derive(gpui::IntoElement)]
pub struct TitleBar {
    id: gpui::ElementId,
    base: Stateful<gpui::Div>,
    height: Pixels,
    labels: CaptionLabels,
    caption: Option<Caption>,
    on_close: Option<CloseRequest>,
}

impl TitleBar {
    /// A bar `height` tall at the top of the window. `labels` are the
    /// accessible names of the Windows and Linux caption buttons, in the
    /// application's language.
    pub fn new(id: impl Into<gpui::ElementId>, height: Pixels, labels: CaptionLabels) -> Self {
        let id = id.into();
        Self {
            base: div().id(id.clone()),
            id,
            height,
            labels,
            caption: None,
            on_close: None,
        }
    }

    /// Force a caption style instead of detecting it, e.g. to compare the
    /// Windows and Linux bars from another platform.
    pub fn caption(mut self, caption: Caption) -> Self {
        self.caption = Some(caption);
        self
    }

    /// Replace the accessible names of the Windows and Linux caption buttons.
    pub fn labels(mut self, labels: CaptionLabels) -> Self {
        self.labels = labels;
        self
    }

    /// Asked before the Linux close button removes the window, as
    /// [`CaptionButtons::on_close`].
    pub fn on_close(mut self, request: impl Fn(&mut Window, &mut App) -> bool + 'static) -> Self {
        self.on_close = Some(Rc::new(request));
        self
    }
}

impl Styled for TitleBar {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for TitleBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements)
    }
}

/// A press on the bar that may become a window drag once the pointer moves.
#[derive(Default)]
struct DragIntent(bool);

/// Lets the platform move the window with the pointer. Tests count the moves
/// instead: the test platform does not implement them.
fn start_window_move(window: &mut Window) {
    #[cfg(test)]
    {
        let _ = window;
        tests::MOVES.with(|moves| moves.set(moves.get() + 1));
    }
    #[cfg(not(test))]
    window.start_window_move();
}

impl RenderOnce for TitleBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let intent = window.use_keyed_state(
            gpui::ElementId::NamedChild(self.id.into(), "drag".into()),
            cx,
            |_, _| DragIntent::default(),
        );
        let press = intent.clone();
        let release = intent.clone();
        let (leading, trailing) = insets(self.height, self.caption, window);
        let mut caption = CaptionButtons::new(self.height, self.labels);
        if let Some(style) = self.caption {
            caption = caption.caption(style);
        }
        if let Some(request) = self.on_close {
            caption = caption.on_close(move |window, cx| request(window, cx));
        }
        self.base
            .flex()
            .flex_shrink_0()
            .items_center()
            .h(self.height)
            .pl(px(leading))
            .pr(px(trailing))
            // Windows hit-tests this as the caption: the system moves the
            // window and zooms on a double click.
            .window_control_area(WindowControlArea::Drag)
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                press.update(cx, |intent, _| intent.0 = true)
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                release.update(cx, |intent, _| intent.0 = false)
            })
            .on_mouse_move(move |event, window, cx| {
                let started = intent.update(cx, |intent, _| std::mem::take(&mut intent.0));
                if started && event.dragging() && !cfg!(target_os = "windows") {
                    start_window_move(window);
                }
            })
            .on_mouse_down(MouseButton::Right, |event, window, _| {
                if cfg!(any(target_os = "linux", target_os = "freebsd")) {
                    window.show_window_menu(event.position);
                }
            })
            .on_click(|event, window, _| {
                if event.click_count() != 2 {
                    return;
                }
                if cfg!(target_os = "macos") {
                    // Follows the user's "double-click a window's title bar" setting.
                    window.titlebar_double_click();
                } else if !cfg!(any(target_os = "windows", target_family = "wasm")) {
                    window.zoom_window();
                }
            })
            // Out of the flow, so the caller's gap does not reach it: the
            // buttons place themselves at the window's top-right corner.
            .child(div().absolute().child(caption))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::caption::tests::test_labels;
    use crate::components::{ButtonVariant, material_button, title_bar_control};
    use gpui::{Context, Modifiers, Render, TestAppContext, size};
    use std::cell::Cell;

    thread_local! {
        /// Window moves the bar started on this test's thread.
        pub(super) static MOVES: Cell<usize> = const { Cell::new(0) };
    }

    #[test]
    fn traffic_lights_are_centred_in_the_bar() {
        assert_eq!(traffic_light_position(px(32.)), point(px(9.), px(9.)));
        // Taller bars keep the lights near the corner, as macOS places them.
        assert_eq!(traffic_light_position(px(64.)), point(px(20.), px(25.)));
    }

    /// A Windows or Linux caption forced on another platform, as captures do,
    /// starts where those platforms do: there are no traffic lights to clear.
    #[gpui::test]
    fn a_forced_trailing_caption_has_no_traffic_light_inset(cx: &mut TestAppContext) {
        let window = cx.add_empty_window();
        window.update(|window, _| {
            let height = px(56.);
            let width = px(390.);
            assert_eq!(
                title_bar_room(width, height, Some(Caption::Windows), window),
                // 16px inset, three 46px buttons and 8px of open space.
                px(390. - 16. - 3. * 46. - 8.)
            );
            assert_eq!(
                title_bar_room(width, height, Some(Caption::System), window),
                width - px(title_bar_leading(height, window) + 16.)
            );
        });
    }

    #[test]
    fn content_starts_after_the_traffic_lights() {
        // 20px inset, three 14px lights 6px apart, then 16px of open space.
        assert_eq!(leading_for(true, px(64.)), 20. + 42. + 12. + 16.);
        assert_eq!(leading_for(false, px(64.)), 16.);
    }

    #[gpui::test]
    fn window_options_let_the_application_own_title_bar_drags(cx: &mut TestAppContext) {
        let options = cx
            .update(|cx| title_bar_window_options("Title", px(64.), size(px(800.), px(600.)), cx));
        assert!(options.app_owns_titlebar_drag);
        let titlebar = options.titlebar.expect("a title bar");
        assert!(titlebar.appears_transparent);
        assert_eq!(
            titlebar.traffic_light_position,
            Some(point(px(20.), px(25.)))
        );
    }

    struct Harness {
        clicks: usize,
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(
                TitleBar::new("bar", px(48.), test_labels())
                    .w_full()
                    .gap(px(16.))
                    .child(title_bar_control(
                        material_button("in-bar", ButtonVariant::Text)
                            .debug_selector(|| "in-bar".into())
                            .child("Action")
                            .on_click(cx.listener(|this, _, _, _| this.clicks += 1)),
                    ))
                    .child(div().flex_1())
                    .child(div().debug_selector(|| "last".into()).size(px(24.))),
            )
        }
    }

    #[gpui::test]
    fn controls_in_the_bar_take_their_clicks(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| Harness { clicks: 0 });
        cx.run_until_parked();
        let center = cx
            .debug_bounds("in-bar")
            .expect("the bar's button")
            .center();
        cx.simulate_click(center, Modifiers::default());
        // Empty bar space takes presses without reaching the control.
        cx.simulate_click(point(px(400.), px(24.)), Modifiers::default());
        view.read_with(cx, |v, _| assert_eq!(v.clicks, 1));
    }

    fn moves() -> usize {
        MOVES.with(Cell::get)
    }

    #[gpui::test]
    fn only_empty_bar_space_starts_a_window_drag(cx: &mut TestAppContext) {
        MOVES.with(|moves| moves.set(0));
        let (_, cx) = cx.add_window_view(|_, _| Harness { clicks: 0 });
        cx.run_until_parked();
        let control = cx
            .debug_bounds("in-bar")
            .expect("the bar's button")
            .center();
        let empty = point(px(400.), px(24.));
        let left = Some(MouseButton::Left);
        // Dragging from a control never moves the window.
        cx.simulate_mouse_down(control, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(control + point(px(8.), px(0.)), left, Modifiers::default());
        cx.simulate_mouse_up(control, MouseButton::Left, Modifiers::default());
        assert_eq!(moves(), 0);
        // A press and release on empty space, then a hover, does not either.
        cx.simulate_mouse_down(empty, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_up(empty, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(empty, None, Modifiers::default());
        assert_eq!(moves(), 0);
        // Dragging from empty space moves it, except where Windows does.
        cx.simulate_mouse_down(empty, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(empty + point(px(8.), px(0.)), left, Modifiers::default());
        assert_eq!(moves(), usize::from(!cfg!(target_os = "windows")));
    }

    #[gpui::test]
    fn the_caller_gap_stops_at_the_last_child(cx: &mut TestAppContext) {
        let (_, cx) = cx.add_window_view(|_, _| Harness { clicks: 0 });
        cx.run_until_parked();
        let last = cx.debug_bounds("last").expect("the last child");
        let (width, trailing) = cx.update(|window, _| {
            (
                window.viewport_size().width,
                CaptionButtons::trailing(window),
            )
        });
        assert_eq!(last.right(), width - px(trailing));
    }
}
