//! M3 snackbar: a brief message on the inverse surface with an optional
//! text action and close affordance. Reference: the Figma kit's Snackbar
//! set (single/two-line, action, close, longer action) and
//! <https://m3e.kyralabs.dev/components/snackbar/>.
use super::InteractionPreview;
use super::focus::{RingPlacement, RingedButton};
use super::text::accessible_text;
use crate::color::ColorRole::*;
use crate::icons::IconName;
use crate::motion::{self, MotionToken};
use crate::theme::color;
use crate::tokens::{elevation, shape::*, state, typography::*};
use gpui::{prelude::*, *};
use std::rc::Rc;
use std::time::Duration;

/// M3 snackbars sit at elevation level 3.
const SNACKBAR_ELEVATION: u8 = 3;
/// The close affordance: a 24dp icon in a 40dp target inside a 48dp box.
pub const SNACKBAR_CLOSE_ICON: Rems = rem(24.);
const BUTTON_HEIGHT: f32 = 40.;
const CLOSE_BOX: f32 = 48.;
/// How far a snackbar rises as it enters or sinks as it leaves.
const TRAVEL: f32 = 12.;
/// A placed snackbar's distance from the edges of its ancestor.
const EDGE: f32 = 16.;

type SnackbarHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type DismissHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// How long an open snackbar stays before asking to be dismissed. md3.io's
/// short is 4 seconds and its long 10.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SnackbarDuration {
    Short,
    Long,
    /// Stays until the caller closes it.
    #[default]
    Indefinite,
    Custom(Duration),
}

impl SnackbarDuration {
    pub const SHORT: Duration = Duration::from_secs(4);
    pub const LONG: Duration = Duration::from_secs(10);
    /// The time before [`Snackbar::on_dismiss`] runs, if it ever does.
    pub const fn timeout(self) -> Option<Duration> {
        match self {
            Self::Short => Some(Self::SHORT),
            Self::Long => Some(Self::LONG),
            Self::Indefinite => None,
            Self::Custom(duration) => Some(duration),
        }
    }
}

/// Where a placed snackbar sits along the bottom of its positioned ancestor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SnackbarPosition {
    #[default]
    Start,
    Center,
    End,
}

/// The auto-dismiss countdown: what it was armed for, a generation that
/// cancels stale timers, whether the pointer pauses it, and the latest handler.
#[derive(Default)]
struct AutoDismiss {
    armed: Option<(bool, SnackbarDuration, bool)>,
    generation: u64,
    hovered: bool,
    on_dismiss: Option<DismissHandler>,
}

/// M3 snackbar.
///
/// One line grows to two by wrapping; [`Self::longer_action`] moves the
/// action below the text. The message is a status node (announced politely
/// by platforms that treat status as a live region). It rises and fades in
/// when mounted; with [`Self::open`]`(false)` it sinks and fades out, then
/// renders nothing, so the caller can keep it mounted until the exit
/// settles. The caller supplies every string and places it, or asks for a
/// [`Self::position`] along the bottom of its positioned ancestor.
/// [`Self::duration`] asks for dismissal through [`Self::on_dismiss`], which
/// also answers Escape; the pointer resting on it pauses the countdown.
#[derive(gpui::IntoElement)]
pub struct Snackbar {
    id: ElementId,
    message: SharedString,
    action: Option<(SharedString, SnackbarHandler)>,
    close: Option<(SharedString, SnackbarHandler)>,
    longer_action: bool,
    open: bool,
    duration: SnackbarDuration,
    position: Option<SnackbarPosition>,
    on_dismiss: Option<DismissHandler>,
    selector: Option<SharedString>,
    preview: Option<InteractionPreview>,
}

impl Snackbar {
    pub fn new(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            message: message.into(),
            action: None,
            close: None,
            longer_action: false,
            open: true,
            duration: SnackbarDuration::Indefinite,
            position: None,
            on_dismiss: None,
            selector: None,
            preview: None,
        }
    }

    /// A text button after the message, e.g. "Undo".
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), Rc::new(handler)));
        self
    }

    /// A close icon button, announced with `label`.
    pub fn close(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.close = Some((label.into(), Rc::new(handler)));
        self
    }

    /// Place a long action label on its own row below the message.
    pub fn longer_action(mut self, longer: bool) -> Self {
        self.longer_action = longer;
        self
    }

    /// Controlled visibility: `false` plays the exit, then hides.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// How long it stays open before [`Self::on_dismiss`] runs.
    pub fn duration(mut self, duration: SnackbarDuration) -> Self {
        self.duration = duration;
        self
    }

    /// Place it along the bottom of the nearest positioned ancestor, 16dp from
    /// the edges, instead of leaving placement to the caller.
    pub fn position(mut self, position: SnackbarPosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Called when the duration runs out or Escape is pressed inside it. The
    /// caller owns visibility and answers with [`Self::open`]`(false)`.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    /// Name the surface for layout tests.
    pub fn debug_name(mut self, name: impl Into<SharedString>) -> Self {
        self.selector = Some(name.into());
        self
    }

    /// Show the action and close affordance in one forced state (a
    /// static specimen; the buttons leave the tab order).
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
}

/// A button on the inverse surface, with its state layer in `fg` and the
/// shared keyboard focus ring.
fn inverse_button(
    id: ElementId,
    fg: Rgba,
    handler: SnackbarHandler,
    preview: Option<InteractionPreview>,
) -> RingedButton {
    let clear = Rgba { a: 0., ..fg };
    let (hover, press) = (state::hovered(clear, fg), state::pressed(clear, fg));
    let focused = state::layer(clear, fg, state::FOCUS);
    let button = RingedButton::new(id, RingPlacement::Outer(Corners::all(RADIUS_FULL)))
        .map(move |button| {
            button
                .flex()
                .items_center()
                .justify_center()
                .flex_shrink_0()
                .h(rem(BUTTON_HEIGHT))
                .rounded_full()
                .text_color(fg)
                .when(preview == Some(InteractionPreview::Focused), |b| {
                    b.bg(focused)
                })
        })
        .state_layers(hover, press)
        .ripple(fg)
        .on_click(move |event, window, cx| handler(event, window, cx));
    match preview {
        Some(state) => button.preview_state(state),
        None => button,
    }
}

/// Keep the countdown in line with the snackbar. Each show, duration change or
/// hover end restarts it; anything else leaves a running timer alone.
fn sync_auto_dismiss(
    state: &Entity<AutoDismiss>,
    open: bool,
    duration: SnackbarDuration,
    on_dismiss: Option<DismissHandler>,
    window: &mut Window,
    cx: &mut App,
) {
    let hovered = state.read(cx).hovered;
    let key = (open, duration, hovered);
    let restart = state.read(cx).armed != Some(key);
    let generation = state.update(cx, |state, _| {
        state.on_dismiss = on_dismiss;
        if restart {
            state.armed = Some(key);
            state.generation += 1;
        }
        state.generation
    });
    if !restart || !open || hovered {
        return;
    }
    let Some(delay) = duration.timeout() else {
        return;
    };
    let state = state.downgrade();
    window
        .spawn(cx, async move |cx| {
            cx.background_executor().timer(delay).await;
            let _ = cx.update(|window, cx| {
                let Some(state) = state.upgrade() else {
                    return;
                };
                let state = state.read(cx);
                if state.generation == generation
                    && let Some(on_dismiss) = state.on_dismiss.clone()
                {
                    on_dismiss(window, cx);
                }
            });
        })
        .detach();
}

/// Whether a closing snackbar has finished leaving.
fn hidden(open: bool, shown: f32) -> bool {
    !open && shown <= 0.
}

impl RenderOnce for Snackbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let child = |name: &'static str| ElementId::NamedChild(id.clone().into(), name.into());
        let shown = motion::token_value(
            child("shown"),
            if self.open { 1. } else { 0. },
            MotionToken::DefaultEffects,
            window,
            cx,
        )
        .clamp(0., 1.);
        let preview = self.preview;
        let auto_dismiss =
            window.use_keyed_state(child("auto-dismiss"), cx, |_, _| AutoDismiss::default());
        if preview.is_none() {
            sync_auto_dismiss(
                &auto_dismiss,
                self.open,
                self.duration,
                self.on_dismiss.clone(),
                window,
                cx,
            );
        }
        if hidden(self.open, shown) {
            return div().id(id).hidden().into_any_element();
        }
        let label = TypeRole::LabelLarge.scale();
        let action = self.action.map(|(text, handler)| {
            inverse_button(child("action"), color(INVERSE_PRIMARY), handler, preview)
                .when(cfg!(test), |b| {
                    b.debug_selector(|| "snackbar-action".into())
                })
                .px(rem(12.))
                .text_size(label.font_size)
                .line_height(label.line_height)
                .font_weight(FontWeight(label.font_weight as f32))
                .child(text)
        });
        let close = self.close.map(|(text, handler)| {
            div()
                .flex()
                .items_center()
                .justify_center()
                .flex_shrink_0()
                .size(rem(CLOSE_BOX))
                .child(
                    inverse_button(child("close"), color(INVERSE_ON_SURFACE), handler, preview)
                        .when(cfg!(test), |b| b.debug_selector(|| "snackbar-close".into()))
                        .accessibility_label(text)
                        .w(rem(BUTTON_HEIGHT))
                        .child(IconName::Close.icon().size(SNACKBAR_CLOSE_ICON)),
                )
        });
        let has_close = close.is_some();
        let has_buttons = action.is_some() || has_close;
        let body = TypeRole::BodyMedium.scale();
        let message = accessible_text(child("message"), self.message)
            .role(Role::Status)
            .flex_1()
            .min_w_0()
            .py(rem(14.))
            .whitespace_normal();
        let buttons = div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .children(action)
            .children(close);
        let surface = div()
            .id(id.clone())
            .flex()
            .min_h(rem(48.))
            .max_w(rem(560.))
            .rounded(RADIUS_EXTRA_SMALL)
            .bg(color(INVERSE_SURFACE))
            .text_color(color(INVERSE_ON_SURFACE))
            .text_size(body.font_size)
            .line_height(body.line_height)
            .shadow(elevation::level(SNACKBAR_ELEVATION))
            .when_some(self.selector, |surface, name| {
                surface.debug_selector(move || name.to_string())
            })
            .on_hover(move |hovered, _, cx| {
                let hovered = *hovered;
                auto_dismiss.update(cx, |auto_dismiss, cx| {
                    if auto_dismiss.hovered != hovered {
                        auto_dismiss.hovered = hovered;
                        cx.notify();
                    }
                });
            })
            .when_some(self.on_dismiss, |surface, on_dismiss| {
                surface.on_key_down(move |event, window, cx| {
                    if event.keystroke.key == "escape" {
                        on_dismiss(window, cx);
                    }
                })
            });
        let surface = if self.longer_action && has_buttons {
            surface
                .flex_col()
                .gap(rem(10.))
                .child(div().flex().px(rem(16.)).child(message.pb_0()))
                .child(
                    buttons
                        .justify_end()
                        .h(rem(48.))
                        .pl(rem(8.))
                        .pr(rem(if has_close { 0. } else { 8. })),
                )
        } else {
            surface
                .items_center()
                .gap(rem(4.))
                .pl(rem(16.))
                .pr(rem(if has_close {
                    0.
                } else if has_buttons {
                    8.
                } else {
                    16.
                }))
                .child(message)
                .child(buttons)
        };
        // Rises and fades in on mount (reduced motion settles at once); the
        // controlled exit sinks and fades on the same effects spring.
        let surface = surface
            .relative()
            .top(rem(TRAVEL * (1. - shown)))
            .opacity(shown);
        let surface = if self.open {
            motion::state_enter(child("enter"), surface)
        } else {
            surface.into_any_element()
        };
        match self.position {
            None => surface,
            Some(position) => div()
                .absolute()
                .bottom(rem(EDGE))
                .left(rem(EDGE))
                .right(rem(EDGE))
                .flex()
                .map(|row| match position {
                    SnackbarPosition::Start => row.justify_start(),
                    SnackbarPosition::Center => row.justify_center(),
                    SnackbarPosition::End => row.justify_end(),
                })
                .child(surface)
                .into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::{Context, Modifiers, TestAppContext};

    struct Harness {
        buttons: bool,
        longer: bool,
        open: bool,
        actions: usize,
        closes: usize,
    }

    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            let snackbar = Snackbar::new("saved", "Icon saved")
                .longer_action(self.longer)
                .open(self.open);
            let snackbar = if self.buttons {
                snackbar
                    .action("Undo", cx.listener(|this, _, _, _| this.actions += 1))
                    .close("Dismiss", cx.listener(|this, _, _, _| this.closes += 1))
            } else {
                snackbar
            };
            div().size_full().child(
                div()
                    .w(px(344.))
                    .debug_selector(|| "bar".into())
                    .child(snackbar),
            )
        }
    }

    fn open(
        cx: &mut TestAppContext,
        buttons: bool,
    ) -> (Entity<Harness>, &mut gpui::VisualTestContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let (view, cx) = cx.add_window_view(move |_, _| Harness {
            buttons,
            longer: false,
            open: true,
            actions: 0,
            closes: 0,
        });
        cx.run_until_parked();
        (view, cx)
    }

    #[gpui::test]
    fn action_and_close_report_their_own_clicks(cx: &mut TestAppContext) {
        let (view, cx) = open(cx, true);
        let action = cx.debug_bounds("snackbar-action").expect("action").center();
        let close = cx.debug_bounds("snackbar-close").expect("close").center();
        cx.simulate_click(action, Modifiers::default());
        cx.simulate_click(close, Modifiers::default());
        cx.simulate_click(close, Modifiers::default());
        view.read_with(cx, |h, _| {
            assert_eq!(h.actions, 1);
            assert_eq!(h.closes, 2);
        });
    }

    #[gpui::test]
    fn single_line_is_48dp_and_the_close_target_is_40dp(cx: &mut TestAppContext) {
        let (_, cx) = open(cx, true);
        assert_eq!(cx.debug_bounds("bar").unwrap().size.height, px(48.));
        let close = cx.debug_bounds("snackbar-close").unwrap();
        assert_eq!(close.size, size(px(40.), px(40.)));
        assert_eq!(SNACKBAR_CLOSE_ICON, rem(24.));
    }

    #[gpui::test]
    fn a_longer_action_moves_below_the_message(cx: &mut TestAppContext) {
        let (view, cx) = open(cx, true);
        view.update(cx, |h, cx| {
            h.longer = true;
            cx.notify();
        });
        redraw(cx);
        let bar = cx.debug_bounds("bar").unwrap();
        let action = cx.debug_bounds("snackbar-action").unwrap();
        assert!(
            action.top() > bar.top() + px(40.),
            "the action sits on its own row: {bar:?} {action:?}"
        );
        assert_eq!(bar.size.height, px(14. + 20. + 10. + 48.));
    }

    #[gpui::test]
    fn closing_hides_once_the_exit_settles(cx: &mut TestAppContext) {
        let (view, cx) = open(cx, true);
        view.update(cx, |h, cx| {
            h.open = false;
            cx.notify();
        });
        redraw(cx);
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(1));
        redraw(cx);
        assert!(cx.debug_bounds("snackbar-action").is_none());
        assert!(hidden(false, 0.) && !hidden(true, 0.) && !hidden(false, 0.5));
    }

    #[test]
    fn snackbar_uses_m3_elevation_level_3() {
        assert_eq!(SNACKBAR_ELEVATION, 3);
        assert_eq!(elevation::level(SNACKBAR_ELEVATION), elevation::level3());
    }

    #[gpui::test]
    fn snackbar_buttons_show_the_ring_only_for_keyboard_focus(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let (_, cx) = open(cx, true);
        let action = cx.debug_bounds("snackbar-action").expect("action");
        cx.simulate_click(action.center(), Modifiers::default());
        redraw(cx);
        assert!(
            crate::components::focus::test_rings::painted("action").is_none(),
            "pointer focus must not show the ring"
        );
        press_key(cx);
        assert_eq!(
            crate::components::focus::test_rings::painted("action"),
            Some(action.dilate(px(5.))),
            "keyboard focus shows the 3dp ring 2dp outside the button"
        );
    }

    fn redraw(cx: &mut gpui::VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }

    /// A key press switches the window to keyboard modality.
    fn press_key(cx: &mut gpui::VisualTestContext) {
        let keystroke =
            cx.update(|_, cx| gpui::Keystroke::parse_with_os("a", cx.platform_info().os).unwrap());
        cx.simulate_event(gpui::KeyDownEvent {
            keystroke,
            is_held: false,
            prefer_character_input: false,
        });
        redraw(cx);
    }

    #[gpui::test]
    fn a_plain_snackbar_has_no_buttons(cx: &mut TestAppContext) {
        let (_, cx) = open(cx, false);
        assert!(cx.debug_bounds("snackbar-action").is_none());
        assert!(cx.debug_bounds("snackbar-close").is_none());
    }
}
