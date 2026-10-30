//! M3 dialogs: basic (icon, headline, supporting text, optional content such
//! as a list, actions) and full-screen.
//!
//! Visuals follow the Figma "Material 3 Design Kit" `Basic dialog` /
//! `List dialog` and m3e.kyralabs.dev: a surface-container-high container
//! with 28dp corners at level 3, 24dp padding and 16dp gaps, a 24dp
//! secondary icon above a headline-small headline (centered when the icon is
//! present), body-medium on-surface-variant supporting text, and trailing
//! text actions 8dp apart. A full-screen dialog is a surface page with a
//! close button, title-large headline and a confirming action.
//!
//! Behavior is GPUI Base's modal [`gpui_base::Dialog`] host: the dialog role,
//! the focus trap (Tab cycles inside), Escape and scrim dismissal. Material
//! adds what the host leaves to callers: focus moves into the dialog when it
//! opens and returns to the previously focused element when it closes, and
//! the open/close motion. The container rises into place on the default
//! spatial spring while it and the 32% scrim fade on the default effects
//! spring; a full-screen dialog slides up from the bottom edge. Reduced
//! motion finishes the movement immediately and keeps the fades. Closing
//! plays the same channels in reverse and keeps the host (and its focus
//! trap) until they settle; the content then leaves the tree.
use crate::{
    color::ColorRole,
    icons::{Icon, IconName},
    motion::{self, MotionToken},
    theme::color,
    tokens::{
        elevation, state,
        typography::{TypeRole, rem},
    },
};
use gpui::{prelude::*, *};
use std::rc::Rc;

/// The basic dialog's container width in the design kit; M3 allows 280–560dp.
pub const DIALOG_WIDTH: f32 = 312.;
const DIALOG_MIN_WIDTH: f32 = 280.;
const DIALOG_MAX_WIDTH: f32 = 560.;
const DIALOG_RADIUS: Pixels = px(28.);
/// How far a basic dialog rises while it opens, in dp.
const DIALOG_RISE: f32 = 24.;
/// The scrim's opacity over the content behind a modal surface.
pub const SCRIM_OPACITY: f32 = 0.32;

type Dismiss = Rc<dyn Fn(&mut Window, &mut App)>;

/// An M3 dialog. Without [`MaterialDialog::modal`] it renders its container
/// in place (a specimen); with it, it is a modal overlay over the window.
#[derive(gpui::IntoElement)]
pub struct MaterialDialog {
    id: ElementId,
    icon: Option<Icon>,
    headline: Option<SharedString>,
    label: Option<SharedString>,
    supporting: Option<SharedString>,
    content: Vec<AnyElement>,
    actions: Vec<AnyElement>,
    divider: bool,
    width: f32,
    full_screen: Option<FullScreen>,
    modal: Option<bool>,
    on_dismiss: Option<Dismiss>,
}

/// The full-screen dialog's header: its close button's accessible name and
/// the confirming action.
struct FullScreen {
    close_label: SharedString,
    confirm: Option<AnyElement>,
}

impl MaterialDialog {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            headline: None,
            label: None,
            supporting: None,
            content: Vec::new(),
            actions: Vec::new(),
            divider: false,
            width: DIALOG_WIDTH,
            full_screen: None,
            modal: None,
            on_dismiss: None,
        }
    }
    /// The hero icon above the headline; the headline then centers.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    pub fn headline(mut self, headline: impl Into<SharedString>) -> Self {
        self.headline = Some(headline.into());
        self
    }
    /// The modal dialog's accessible name when it differs from (or there is
    /// no) headline.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }
    /// Content under the text, full width (e.g. a list of items).
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content.push(content.into_any_element());
        self
    }
    /// An action button; actions sit at the trailing edge in call order.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
    /// An outline-variant divider between the content and the actions.
    pub fn divider(mut self, divider: bool) -> Self {
        self.divider = divider;
        self
    }
    /// The basic dialog's width in dp, clamped to M3's 280–560dp.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.clamp(DIALOG_MIN_WIDTH, DIALOG_MAX_WIDTH);
        self
    }
    /// Make this a full-screen dialog. `close_label` names its close button;
    /// `confirm` is the header's action (e.g. a text button).
    pub fn full_screen(
        mut self,
        close_label: impl Into<SharedString>,
        confirm: Option<AnyElement>,
    ) -> Self {
        self.full_screen = Some(FullScreen {
            close_label: close_label.into(),
            confirm,
        });
        self
    }
    /// Present the dialog modally over the window while `open`.
    pub fn modal(mut self, open: bool) -> Self {
        self.modal = Some(open);
        self
    }
    /// Called when the person dismisses the dialog: Escape, the scrim, or a
    /// full-screen dialog's close button. The caller closes it.
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
}

fn typed<E: Styled>(element: E, role: TypeRole) -> E {
    let scale = role.scale();
    element
        .font_family(scale.font_family)
        .text_size(scale.font_size)
        .line_height(scale.line_height)
        .font_weight(FontWeight(scale.font_weight as f32))
}

/// The open/close presentation of a modal surface at spatial progress
/// `spatial` and fade `fade`, both 0 (closed) to 1 (open).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OverlayFrame {
    pub spatial: f32,
    pub fade: f32,
}

impl OverlayFrame {
    /// Whether a closing overlay has settled and may leave the tree.
    pub(crate) fn settled_closed(self, open: bool) -> bool {
        !open && self.spatial == 0. && self.fade <= 0.
    }
    pub(crate) fn opacity(self) -> f32 {
        self.fade.clamp(0., 1.)
    }
}

/// Sample the open/close channels of a modal overlay keyed by `id`.
pub(crate) fn overlay_frame(
    id: &ElementId,
    open: bool,
    window: &mut Window,
    cx: &mut App,
) -> OverlayFrame {
    let target = if open { 1. } else { 0. };
    OverlayFrame {
        spatial: motion::token_value(
            (id.clone(), "overlay-spatial"),
            target,
            MotionToken::DefaultSpatial,
            window,
            cx,
        ),
        fade: motion::token_value(
            (id.clone(), "overlay-fade"),
            target,
            MotionToken::DefaultEffects,
            window,
            cx,
        ),
    }
}

/// Focus that moves into a modal surface when it opens and returns to the
/// previously focused element when it closes. Base's host traps focus but
/// leaves both moves to the application.
pub(crate) struct ModalFocus {
    pub focus: FocusHandle,
    previous: Option<FocusHandle>,
    shown: bool,
}

/// The focus handle of the modal surface keyed by `id`, after applying the
/// open/close focus moves for this frame.
pub(crate) fn modal_focus(
    id: &ElementId,
    open: bool,
    window: &mut Window,
    cx: &mut App,
) -> FocusHandle {
    let state = window.use_keyed_state((id.clone(), "modal-focus"), cx, |_, cx| ModalFocus {
        focus: cx.focus_handle(),
        previous: None,
        shown: false,
    });
    let shown = state.read(cx).shown;
    if open && !shown {
        let previous = window.focused(cx);
        state.update(cx, |state, _| {
            state.shown = true;
            state.previous = previous;
        });
        let target = state.read(cx).focus.clone();
        // The host's focus handle is attached during this frame's paint.
        window.defer(cx, move |window, cx| target.focus(window, cx));
    } else if !open && shown {
        let previous = state.update(cx, |state, _| {
            state.shown = false;
            state.previous.take()
        });
        if let Some(previous) = previous {
            window.defer(cx, move |window, cx| previous.focus(window, cx));
        }
    }
    state.read(cx).focus.clone()
}

/// The modal scrim at the overlay's fade.
pub(crate) fn scrim(fade: f32) -> Div {
    // The scrim blocks the pointer (and wheel) from the content behind, so
    // the page under a modal surface cannot scroll its owner away.
    div()
        .absolute()
        .inset_0()
        .occlude()
        .bg(color(ColorRole::SCRIM).opacity(SCRIM_OPACITY * fade.clamp(0., 1.)))
}

/// A 40dp standard icon button (on-surface-variant) for overlay headers.
pub(crate) fn header_icon_button(
    id: impl Into<ElementId>,
    icon: Icon,
    label: SharedString,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let ink = color(ColorRole::ON_SURFACE_VARIANT);
    let clear = Rgba { a: 0., ..ink };
    let id = id.into();
    div()
        .size(rem(48.))
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .child(super::FocusRing::new(
            ElementId::NamedChild(std::sync::Arc::new(id.clone()), "ring".into()),
            gpui_base::Button::new(id)
                .accessibility_label(label)
                .size(rem(40.))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(move |style| style.bg(state::hovered(clear, ink)))
                .active(move |style| style.bg(state::pressed(clear, ink)))
                .child(icon.size(rem(24.)).color(ink))
                .on_click(on_click),
        ))
}

impl MaterialDialog {
    fn basic_surface(self) -> Stateful<Div> {
        let centered = self.icon.is_some();
        let text = div()
            .flex()
            .flex_col()
            .gap(rem(16.))
            .pt(rem(24.))
            .px(rem(24.))
            .when(centered, |text| text.items_center())
            .when_some(self.icon, |text, icon| {
                text.child(icon.size(rem(24.)).color(color(ColorRole::SECONDARY)))
            })
            .when_some(self.headline, |text, headline| {
                text.child(
                    typed(div(), TypeRole::HeadlineSmall)
                        .w_full()
                        .when(centered, |headline| headline.text_center())
                        .text_color(color(ColorRole::ON_SURFACE))
                        .child(headline),
                )
            })
            .when_some(self.supporting, |text, supporting| {
                text.child(
                    typed(div(), TypeRole::BodyMedium)
                        .w_full()
                        .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                        .child(supporting),
                )
            });
        let has_content = !self.content.is_empty();
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .flex_shrink_0()
            .w(rem(self.width))
            .overflow_hidden()
            .rounded(DIALOG_RADIUS)
            .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
            .shadow(elevation::level3())
            .child(text)
            .when(has_content, |surface| {
                surface.child(div().pt(rem(16.)).flex().flex_col().children(self.content))
            })
            .when(self.divider, |surface| {
                surface.child(
                    div().pt(rem(16.)).child(
                        div()
                            .h(px(1.))
                            .w_full()
                            .bg(color(ColorRole::OUTLINE_VARIANT)),
                    ),
                )
            })
            .child(
                div()
                    .flex()
                    .justify_end()
                    .items_center()
                    .gap(rem(8.))
                    .pt(rem(20.))
                    .pb(rem(20.))
                    .pl(rem(8.))
                    .pr(rem(24.))
                    .children(self.actions),
            )
    }

    fn full_screen_surface(mut self, dismiss: Option<Dismiss>) -> Stateful<Div> {
        let full = self.full_screen.take().expect("a full-screen dialog");
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .size_full()
            .bg(color(ColorRole::SURFACE))
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(rem(56.))
                    .pl(rem(4.))
                    .pr(rem(12.))
                    .gap(rem(4.))
                    .child(header_icon_button(
                        "dialog-close",
                        IconName::Close.icon(),
                        full.close_label,
                        move |_, window, cx| {
                            if let Some(dismiss) = &dismiss {
                                dismiss(window, cx);
                            }
                        },
                    ))
                    .child(
                        typed(div(), TypeRole::TitleLarge)
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_color(color(ColorRole::ON_SURFACE))
                            .children(self.headline),
                    )
                    .children(full.confirm),
            )
            .child(
                div()
                    .id("dialog-body")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .gap(rem(16.))
                    .px(rem(24.))
                    .when_some(self.supporting, |body, supporting| {
                        body.child(
                            typed(div(), TypeRole::BodyMedium)
                                .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                                .child(supporting),
                        )
                    })
                    .children(self.content),
            )
    }
}

impl RenderOnce for MaterialDialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let full_screen = self.full_screen.is_some();
        let Some(open) = self.modal else {
            // A specimen: the container in place.
            return if full_screen {
                self.full_screen_surface(None).into_any_element()
            } else {
                self.basic_surface().into_any_element()
            };
        };
        let id = self.id.clone();
        let frame = overlay_frame(&id, open, window, cx);
        let focus = modal_focus(&id, open, window, cx);
        if frame.settled_closed(open) {
            return div().into_any_element();
        }
        let dismiss = self.on_dismiss.clone();
        // Escape and the scrim reach the caller only while open: a closing
        // dialog keeps its host until the motion settles.
        let cancel_dismiss = dismiss.clone();
        let viewport = window.viewport_size();
        let opacity = frame.opacity();
        let name = self.label.clone().or_else(|| self.headline.clone());
        let surface = if full_screen {
            div()
                .absolute()
                .left_0()
                .w(viewport.width)
                .h(viewport.height)
                .top(viewport.height * (1. - frame.spatial))
                .opacity(opacity)
                .child(self.full_screen_surface(dismiss.filter(|_| open)))
        } else {
            div()
                .relative()
                .top(rem(-DIALOG_RISE * (1. - frame.spatial)))
                .opacity(opacity)
                .child(self.basic_surface())
        };
        gpui_base::Dialog::new(cx)
            .open(true)
            .when_some(name, |dialog, name| dialog.accessibility_label(name))
            .focus_handle(focus)
            .close_on_escape(true)
            .close_on_backdrop_press(open)
            // Enter activates the focused action; it never dismisses.
            .on_ok(|_, _, _| false)
            .on_cancel(move |_, window, cx| {
                if open && let Some(dismiss) = &cancel_dismiss {
                    dismiss(window, cx);
                }
                false
            })
            .backdrop(scrim(frame.fade))
            .popup(surface)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::{Context, Modifiers, TestAppContext, VisualTestContext};
    use std::cell::Cell;

    #[test]
    fn a_closing_overlay_settles_only_at_rest() {
        let frame = |spatial, fade| OverlayFrame { spatial, fade };
        assert!(!frame(0., 0.).settled_closed(true), "opening starts at 0");
        assert!(!frame(0.3, 0.2).settled_closed(false));
        assert!(!frame(-0.01, 0.).settled_closed(false), "overshoot moves");
        assert!(frame(0., 0.).settled_closed(false));
    }

    #[test]
    fn width_stays_within_m3_bounds() {
        assert_eq!(MaterialDialog::new("d").width(100.).width, 280.);
        assert_eq!(MaterialDialog::new("d").width(900.).width, 560.);
        assert_eq!(MaterialDialog::new("d").width, DIALOG_WIDTH);
    }

    struct Harness {
        open: bool,
        dismissals: Rc<Cell<usize>>,
        actions: Rc<Cell<usize>>,
        trigger: FocusHandle,
        action: FocusHandle,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(16.));
            let dismissals = self.dismissals.clone();
            let actions = self.actions.clone();
            div()
                .size_full()
                .child(div().id("trigger").track_focus(&self.trigger).size(px(20.)))
                .child(
                    MaterialDialog::new("dialog")
                        .headline("Title")
                        .supporting_text("Body")
                        .action(
                            gpui_base::Button::new("action")
                                .track_focus(&self.action)
                                .debug_selector(|| "dialog-action".into())
                                .size(px(40.))
                                .on_click(move |_, _, _| actions.set(actions.get() + 1)),
                        )
                        .modal(self.open)
                        .on_dismiss({
                            let this = cx.entity().downgrade();
                            move |_, cx| {
                                dismissals.set(dismissals.get() + 1);
                                let _ = this.update(cx, |this, cx| {
                                    this.open = false;
                                    cx.notify();
                                });
                            }
                        }),
                )
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
    }

    fn key(cx: &mut VisualTestContext, key: &str) {
        let keystroke =
            cx.update(|_, cx| gpui::Keystroke::parse_with_os(key, cx.platform_info().os).unwrap());
        cx.simulate_event(gpui::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(gpui::KeyUpEvent { keystroke });
        cx.run_until_parked();
    }

    #[gpui::test]
    fn focus_moves_in_traps_and_returns_and_escape_dismisses(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let dismissals = Rc::new(Cell::new(0));
        let actions = Rc::new(Cell::new(0));
        let (view, cx) = cx.add_window_view({
            let (dismissals, actions) = (dismissals.clone(), actions.clone());
            move |_, cx| Harness {
                open: false,
                dismissals,
                actions,
                trigger: cx.focus_handle(),
                action: cx.focus_handle(),
            }
        });
        draw(cx);
        cx.update(|window, cx| view.read(cx).trigger.clone().focus(window, cx));
        view.update(cx, |v, cx| {
            v.open = true;
            cx.notify();
        });
        draw(cx);
        draw(cx);
        assert!(cx.debug_bounds("dialog-action").is_some());
        cx.update(|window, cx| {
            assert!(
                !view.read(cx).trigger.is_focused(window),
                "focus moved into the dialog"
            );
        });
        // Tab stays inside: the action is the dialog's only stop.
        for _ in 0..3 {
            cx.update(|window, cx| window.focus_next(cx));
        }
        draw(cx);
        cx.update(|window, cx| assert!(view.read(cx).action.is_focused(window)));
        // Enter activates the action and does not dismiss.
        key(cx, "enter");
        draw(cx);
        assert_eq!(dismissals.get(), 0);
        key(cx, "escape");
        draw(cx);
        assert_eq!(dismissals.get(), 1);
        draw(cx);
        draw(cx);
        assert!(cx.debug_bounds("dialog-action").is_none(), "settled closed");
        cx.update(|window, cx| {
            assert!(
                view.read(cx).trigger.is_focused(window),
                "focus returned to the trigger"
            );
        });
        let _ = actions.get();
        let _ = Modifiers::default();
    }
}
