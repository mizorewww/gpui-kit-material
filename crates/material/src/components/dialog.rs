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
//! md3.io's dialog options are available too: a subtitle (body-medium
//! on-surface-variant, 16dp under the headline), [`DialogSize`]s, a close
//! button in the header, dividers above and below a scrolling body, the
//! actions' [`DialogActionAlignment`], the [`DialogAnimation`], and whether
//! the scrim and Escape dismiss.
//!
//! Behavior is GPUI Base's modal [`gpui_base::Dialog`] host: the dialog role,
//! the focus trap (Tab cycles inside), Escape and scrim dismissal. Material
//! adds what the host leaves to callers: focus moves into the dialog when it
//! opens and returns to the previously focused element when it closes, and
//! the open/close motion. By default the container slides into place from
//! 24dp above on the default spatial spring while it and the 32% scrim fade
//! on the default effects spring; a full-screen dialog slides up from the
//! bottom edge. Reduced motion finishes the movement immediately and keeps
//! the fades. Closing plays the same channels in reverse and keeps the host
//! (and its focus trap) until they settle; the content then leaves the tree.
//!
//! A [`MaterialDialog::contained`] dialog presents over its positioned parent
//! instead of the whole window (e.g. a device frame in a preview): the host,
//! scrim and full-screen surface take the parent's bounds.
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
/// md3.io's large dialog width, for large windows.
const DIALOG_LARGE_WIDTH: f32 = 720.;
/// The margin a dialog keeps from its container's edges.
const DIALOG_MARGIN: f32 = 24.;
const DIALOG_RADIUS: Pixels = px(28.);
/// How far a basic dialog slides while it opens, in dp.
const DIALOG_RISE: f32 = 24.;
/// The scale a scaling dialog opens from.
const DIALOG_SCALE_FROM: f32 = 0.8;
/// The scrim's opacity over the content behind a modal surface.
pub const SCRIM_OPACITY: f32 = 0.32;

type Dismiss = Rc<dyn Fn(&mut Window, &mut App)>;

/// A dialog's size (md3.io): the basic widths, or a full-screen page.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogSize {
    /// The design kit's basic dialog ([`MaterialDialog::width`], 312dp by
    /// default).
    #[default]
    Small,
    /// M3's widest basic dialog, 560dp.
    Medium,
    /// 720dp, for large windows.
    Large,
    /// The container's width less 24dp margins.
    FullWidth,
    /// A full-screen dialog: a header with the close button and headline.
    FullScreen,
}

impl DialogSize {
    pub const ALL: [Self; 5] = [
        Self::Small,
        Self::Medium,
        Self::Large,
        Self::FullWidth,
        Self::FullScreen,
    ];
}

/// How a dialog enters and leaves (md3.io). Every animation fades; reduced
/// motion keeps only the fade.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogAnimation {
    /// Grows from 80% around its center. GPUI has no element scale
    /// transform, so the container's clip grows while its content stays at
    /// full size.
    Scale,
    /// Rises 24dp into place.
    SlideUp,
    /// Descends 24dp into place.
    #[default]
    SlideDown,
    /// Fades only.
    Fade,
}

impl DialogAnimation {
    pub const ALL: [Self; 4] = [Self::Scale, Self::SlideUp, Self::SlideDown, Self::Fade];
}

/// Where a dialog's actions sit in their row (md3.io).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogActionAlignment {
    /// The trailing edge (M3's default).
    #[default]
    End,
    Start,
    Center,
    /// The first action at the start, the last at the end.
    SpaceBetween,
}

impl DialogActionAlignment {
    pub const ALL: [Self; 4] = [Self::End, Self::Start, Self::Center, Self::SpaceBetween];
}

/// An M3 dialog. Without [`MaterialDialog::modal`] it renders its container
/// in place (a specimen); with it, it is a modal overlay over the window.
#[derive(gpui::IntoElement)]
pub struct MaterialDialog {
    id: ElementId,
    icon: Option<Icon>,
    headline: Option<SharedString>,
    subtitle: Option<SharedString>,
    label: Option<SharedString>,
    supporting: Option<SharedString>,
    content: Vec<AnyElement>,
    actions: Vec<AnyElement>,
    divider: bool,
    dividers: bool,
    width: f32,
    size: DialogSize,
    animation: DialogAnimation,
    alignment: DialogActionAlignment,
    close_label: Option<SharedString>,
    full_screen: Option<FullScreen>,
    modal: Option<bool>,
    contained: bool,
    dismiss_on_scrim: bool,
    dismiss_on_escape: bool,
    on_dismiss: Option<Dismiss>,
}

/// The full-screen dialog's header: its close button's accessible name and
/// the confirming action.
struct FullScreen {
    close_label: Option<SharedString>,
    confirm: Option<AnyElement>,
}

impl MaterialDialog {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            headline: None,
            subtitle: None,
            label: None,
            supporting: None,
            content: Vec::new(),
            actions: Vec::new(),
            divider: false,
            dividers: false,
            width: DIALOG_WIDTH,
            size: DialogSize::Small,
            animation: DialogAnimation::default(),
            alignment: DialogActionAlignment::End,
            close_label: None,
            full_screen: None,
            modal: None,
            contained: false,
            dismiss_on_scrim: true,
            dismiss_on_escape: true,
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
    /// A body-medium line 16dp under the headline (md3.io).
    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
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
    /// Outline-variant dividers above and below the body (supporting text
    /// and content), for a body that scrolls (md3.io's Dividers).
    pub fn dividers(mut self, dividers: bool) -> Self {
        self.dividers = dividers;
        self
    }
    /// The basic dialog's width in dp, clamped to M3's 280–560dp.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.clamp(DIALOG_MIN_WIDTH, DIALOG_MAX_WIDTH);
        self
    }
    /// The dialog's size; [`DialogSize::Small`] uses [`Self::width`].
    pub fn size(mut self, size: DialogSize) -> Self {
        self.size = size;
        self
    }
    pub fn animation(mut self, animation: DialogAnimation) -> Self {
        self.animation = animation;
        self
    }
    pub fn action_alignment(mut self, alignment: DialogActionAlignment) -> Self {
        self.alignment = alignment;
        self
    }
    /// A close button named `label` at the header's trailing edge; it calls
    /// `on_dismiss`. A full-screen dialog shows it at its leading edge.
    pub fn close_button(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = Some(label.into());
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
            close_label: Some(close_label.into()),
            confirm,
        });
        self
    }
    /// Present the dialog modally over the window while `open`.
    pub fn modal(mut self, open: bool) -> Self {
        self.modal = Some(open);
        self
    }
    /// Present the modal dialog over its positioned parent's bounds instead
    /// of the whole window.
    pub fn contained(mut self, contained: bool) -> Self {
        self.contained = contained;
        self
    }
    /// Whether pressing the scrim dismisses the dialog (default true).
    pub fn dismiss_on_scrim(mut self, dismiss: bool) -> Self {
        self.dismiss_on_scrim = dismiss;
        self
    }
    /// Whether Escape dismisses the dialog (default true).
    pub fn dismiss_on_escape(mut self, dismiss: bool) -> Self {
        self.dismiss_on_escape = dismiss;
        self
    }
    /// Called when the person dismisses the dialog: Escape, the scrim, or a
    /// close button. The caller closes it.
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }

    /// The basic container's width in dp inside a container `available` dp
    /// wide.
    fn basic_width(&self, available: f32) -> f32 {
        let width = match self.size {
            DialogSize::Small | DialogSize::FullScreen => self.width,
            DialogSize::Medium => DIALOG_MAX_WIDTH,
            DialogSize::Large => DIALOG_LARGE_WIDTH,
            // A specimen has no container: the widest basic dialog.
            DialogSize::FullWidth if available.is_finite() => available - 2. * DIALOG_MARGIN,
            DialogSize::FullWidth => DIALOG_MAX_WIDTH,
        };
        width.min(available - 2. * DIALOG_MARGIN).max(0.)
    }
}

/// The clip a scaling dialog paints at spatial progress `spatial`, for a
/// container `size`: the clip's size and the container's offset inside it.
pub(crate) fn scale_clip(size: Size<Pixels>, spatial: f32) -> (Size<Pixels>, Point<Pixels>) {
    let scale = DIALOG_SCALE_FROM + (1. - DIALOG_SCALE_FROM) * spatial;
    let clip = size.map(|length| length * scale);
    let offset = point(
        (clip.width - size.width) / 2.,
        (clip.height - size.height) / 2.,
    );
    (clip, offset)
}

/// The vertical offset in dp of a sliding dialog at `spatial`.
fn slide_offset(animation: DialogAnimation, spatial: f32) -> f32 {
    match animation {
        DialogAnimation::SlideUp => DIALOG_RISE * (1. - spatial),
        DialogAnimation::SlideDown => -DIALOG_RISE * (1. - spatial),
        DialogAnimation::Scale | DialogAnimation::Fade => 0.,
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
        .debug_selector({
            let selector = id.clone();
            move || selector.to_string()
        })
        .size(rem(48.))
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .child(super::FocusRing::new(
            ElementId::NamedChild(std::sync::Arc::new(id.clone()), "ring".into()),
            super::Ripple::new(
                ElementId::NamedChild(std::sync::Arc::new(id.clone()), "ripple".into()),
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
            )
            .round()
            .ink(ink),
        ))
}

/// The window bounds of a contained overlay's positioned parent, as
/// measured the previous frame.
pub(crate) struct ContainerBounds(Option<Bounds<Pixels>>);

/// The measured parent bounds of the contained overlay keyed by `id`.
pub(crate) fn container_state(
    id: &ElementId,
    window: &mut Window,
    cx: &mut App,
) -> Entity<ContainerBounds> {
    window.use_keyed_state((id.clone(), "overlay-container"), cx, |_, _| {
        ContainerBounds(None)
    })
}

/// Where an overlay presents: its container's bounds when contained (`None`
/// until measured), otherwise the window.
pub(crate) fn overlay_area(
    container: Option<&Entity<ContainerBounds>>,
    window: &Window,
    cx: &App,
) -> Option<Bounds<Pixels>> {
    match container {
        Some(container) => container.read(cx).0,
        None => Some(Bounds::new(Point::default(), window.viewport_size())),
    }
}

/// Lay an overlay host out over `area` instead of the window.
pub(crate) fn place_host<E: Styled>(host: E, area: Bounds<Pixels>) -> E {
    host.left(area.origin.x)
        .top(area.origin.y)
        .w(area.size.width)
        .h(area.size.height)
        // Clip a partially shown sheet to the container.
        .overflow_hidden()
}

/// Wrap a contained overlay's host with the probe measuring its parent; the
/// probe asks for another frame when the bounds change (the host used the
/// previous ones this frame).
pub(crate) fn with_container_probe(
    container: Option<Entity<ContainerBounds>>,
    host: AnyElement,
) -> AnyElement {
    let Some(container) = container else {
        return host;
    };
    div()
        .absolute()
        .inset_0()
        .child(
            canvas(
                move |bounds, window, cx| {
                    if container.read(cx).0 != Some(bounds) {
                        container.update(cx, |container, _| container.0 = Some(bounds));
                        window.request_animation_frame();
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        )
        .child(host)
        .into_any_element()
}

/// A basic dialog container's measured size, for the scale animation.
struct MeasuredSize(Option<Size<Pixels>>);

fn divider_line() -> Div {
    div()
        .h(px(1.))
        .w_full()
        .flex_shrink_0()
        .bg(color(ColorRole::OUTLINE_VARIANT))
}

impl MaterialDialog {
    /// The actions row, aligned; `None` without actions.
    fn actions_row(alignment: DialogActionAlignment, actions: Vec<AnyElement>) -> Option<Div> {
        if actions.is_empty() {
            return None;
        }
        let row = div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .gap(rem(8.))
            .pt(rem(20.))
            .pb(rem(20.));
        let row = match alignment {
            DialogActionAlignment::End => row.justify_end().pl(rem(8.)).pr(rem(24.)),
            DialogActionAlignment::Start => row.justify_start().pl(rem(24.)).pr(rem(8.)),
            DialogActionAlignment::Center => row.justify_center().px(rem(24.)),
            DialogActionAlignment::SpaceBetween => row.justify_between().px(rem(24.)),
        };
        Some(row.children(actions))
    }

    fn supporting_element(supporting: SharedString) -> Div {
        typed(div(), TypeRole::BodyMedium)
            .w_full()
            .text_color(color(ColorRole::ON_SURFACE_VARIANT))
            .child(supporting)
    }

    fn basic_surface(
        self,
        width: f32,
        dismiss: Option<Dismiss>,
        measure: Option<Entity<MeasuredSize>>,
    ) -> Stateful<Div> {
        let centered = self.icon.is_some();
        let dividers = self.dividers;
        let has_close = self.close_label.is_some();
        let supporting = self.supporting;
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
                        .when(has_close && !centered, |headline| headline.pr(rem(32.)))
                        .text_color(color(ColorRole::ON_SURFACE))
                        .child(headline),
                )
            })
            .when_some(self.subtitle, |text, subtitle| {
                text.child(Self::supporting_element(subtitle).when(centered, |s| s.text_center()))
            })
            .when(!dividers, |text| {
                text.when_some(supporting.clone(), |text, supporting| {
                    text.child(Self::supporting_element(supporting))
                })
            });
        let content = self.content;
        let has_content = !content.is_empty();
        // With dividers, the supporting text and content form the body
        // between them; otherwise the content follows the text.
        let middle = if dividers {
            Some(
                div()
                    .flex()
                    .flex_col()
                    .gap(rem(16.))
                    .py(rem(16.))
                    .when_some(supporting, |body, supporting| {
                        body.child(
                            div()
                                .px(rem(24.))
                                .child(Self::supporting_element(supporting)),
                        )
                    })
                    .children(content),
            )
        } else {
            has_content.then(|| div().pt(rem(16.)).flex().flex_col().children(content))
        };
        let actions = Self::actions_row(self.alignment, self.actions);
        let has_actions = actions.is_some();
        let close = self.close_label.map(|label| {
            div()
                .absolute()
                .top(rem(12.))
                .right(rem(12.))
                .child(header_icon_button(
                    "dialog-close",
                    IconName::Close.icon(),
                    label,
                    move |_, window, cx| {
                        if let Some(dismiss) = &dismiss {
                            dismiss(window, cx);
                        }
                    },
                ))
        });
        let mut surface = div()
            .id(self.id)
            .relative()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .w(rem(width))
            .overflow_hidden()
            .rounded(DIALOG_RADIUS)
            .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
            .shadow(elevation::level3())
            .child(text);
        match middle {
            Some(body) if dividers => {
                surface = surface
                    .child(div().pt(rem(16.)).child(divider_line()))
                    .child(body)
                    .child(divider_line());
            }
            Some(content) => surface = surface.child(content),
            None => {}
        }
        surface
            .when(self.divider && !dividers, |surface| {
                surface.child(div().pt(rem(16.)).child(divider_line()))
            })
            .children(actions)
            .when(!has_actions, |surface| surface.child(div().h(rem(24.))))
            .children(close)
            .when_some(measure, |surface, measure| {
                surface.child(
                    canvas(
                        move |bounds, window, cx| {
                            if measure.read(cx).0 != Some(bounds.size) {
                                measure.update(cx, |measure, _| measure.0 = Some(bounds.size));
                                window.request_animation_frame();
                            }
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .inset_0(),
                )
            })
    }
}

impl MaterialDialog {
    fn full_screen_surface(mut self, dismiss: Option<Dismiss>) -> Stateful<Div> {
        let full = self.full_screen.take().unwrap_or(FullScreen {
            close_label: None,
            confirm: None,
        });
        let close_label = full.close_label.or(self.close_label.take());
        let dividers = self.dividers;
        let footer = Self::actions_row(self.alignment, std::mem::take(&mut self.actions));
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .size_full()
            .bg(color(ColorRole::SURFACE))
            .child(
                div()
                    .flex()
                    .flex_shrink_0()
                    .items_center()
                    .h(rem(56.))
                    .pl(rem(if close_label.is_some() { 4. } else { 24. }))
                    .pr(rem(12.))
                    .gap(rem(4.))
                    .when_some(close_label, |header, label| {
                        header.child(header_icon_button(
                            "dialog-close",
                            IconName::Close.icon(),
                            label,
                            move |_, window, cx| {
                                if let Some(dismiss) = &dismiss {
                                    dismiss(window, cx);
                                }
                            },
                        ))
                    })
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
            .when(dividers, |page| page.child(divider_line()))
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
                    .when(dividers, |body| body.py(rem(16.)))
                    .when_some(self.subtitle, |body, subtitle| {
                        body.child(Self::supporting_element(subtitle))
                    })
                    .when_some(self.supporting, |body, supporting| {
                        body.child(Self::supporting_element(supporting))
                    })
                    .children(self.content),
            )
            .when(dividers && footer.is_some(), |page| {
                page.child(divider_line())
            })
            .children(footer)
    }
}

impl RenderOnce for MaterialDialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let full_screen = self.full_screen.is_some() || self.size == DialogSize::FullScreen;
        let Some(open) = self.modal else {
            // A specimen: the container in place.
            return if full_screen {
                self.full_screen_surface(None).into_any_element()
            } else {
                let width = self.basic_width(f32::INFINITY);
                let dismiss = self.on_dismiss.clone();
                self.basic_surface(width, dismiss, None).into_any_element()
            };
        };
        let id = self.id.clone();
        let container = self.contained.then(|| container_state(&id, window, cx));
        let frame = overlay_frame(&id, open, window, cx);
        let focus = modal_focus(&id, open, window, cx);
        if frame.settled_closed(open) {
            return with_container_probe(container, div().into_any_element());
        }
        let Some(area) = overlay_area(container.as_ref(), window, cx) else {
            // The container is measured this frame; present on the next.
            return with_container_probe(container, div().into_any_element());
        };
        let dismiss = self.on_dismiss.clone();
        // Escape and the scrim reach the caller only while open: a closing
        // dialog keeps its host until the motion settles.
        let cancel_dismiss = dismiss.clone();
        let scrim_dismiss = dismiss.clone();
        let opacity = frame.opacity();
        let name = self.label.clone().or_else(|| self.headline.clone());
        let (dismiss_on_escape, dismiss_on_scrim) = (self.dismiss_on_escape, self.dismiss_on_scrim);
        let surface = if full_screen {
            div()
                .absolute()
                .left_0()
                .w(area.size.width)
                .h(area.size.height)
                .top(area.size.height * (1. - frame.spatial))
                .opacity(opacity)
                .child(self.full_screen_surface(dismiss.filter(|_| open)))
                .into_any_element()
        } else {
            let rem_size = window.rem_size();
            let available = f32::from(area.size.width) * crate::tokens::typography::BASE_PX
                / f32::from(rem_size);
            let width = self.basic_width(available);
            let animation = self.animation;
            let scaling = animation == DialogAnimation::Scale && frame.spatial < 1.;
            let measure = (animation == DialogAnimation::Scale).then(|| {
                window.use_keyed_state((id.clone(), "dialog-size"), cx, |_, _| MeasuredSize(None))
            });
            let measured = measure.as_ref().and_then(|measure| measure.read(cx).0);
            let basic = self.basic_surface(width, dismiss.filter(|_| open), measure);
            let positioned = div()
                .relative()
                .top(rem(slide_offset(animation, frame.spatial)))
                .opacity(opacity);
            match measured.filter(|_| scaling) {
                Some(size) => {
                    let (clip, offset) = scale_clip(size, frame.spatial.max(0.));
                    positioned
                        .child(
                            div()
                                .relative()
                                .w(clip.width)
                                .h(clip.height)
                                .overflow_hidden()
                                .rounded(DIALOG_RADIUS)
                                .child(div().absolute().left(offset.x).top(offset.y).child(basic)),
                        )
                        .into_any_element()
                }
                None => positioned.child(basic).into_any_element(),
            }
        };
        let host = gpui_base::Dialog::new(cx)
            .open(true)
            .when_some(name, |dialog, name| dialog.accessibility_label(name))
            .focus_handle(focus)
            .close_on_escape(dismiss_on_escape)
            // The scrim occludes the host's backdrop layer (so the pointer
            // and wheel never reach the page behind), so it dismisses itself.
            .close_on_backdrop_press(false)
            // Enter activates the focused action; it never dismisses.
            .on_ok(|_, _, _| false)
            .on_cancel(move |_, window, cx| {
                if open && let Some(dismiss) = &cancel_dismiss {
                    dismiss(window, cx);
                }
                false
            })
            .backdrop(
                scrim(frame.fade).on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    cx.stop_propagation();
                    if open
                        && dismiss_on_scrim
                        && let Some(dismiss) = &scrim_dismiss
                    {
                        dismiss(window, cx);
                    }
                }),
            )
            .popup(surface);
        let host = if container.is_some() {
            place_host(host, area)
        } else {
            host
        };
        with_container_probe(container, host.into_any_element())
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

    #[test]
    fn sizes_pick_their_widths_within_the_container() {
        let width = |size| MaterialDialog::new("d").size(size).basic_width(1000.);
        assert_eq!(width(DialogSize::Small), DIALOG_WIDTH);
        assert_eq!(width(DialogSize::Medium), 560.);
        assert_eq!(width(DialogSize::Large), 720.);
        assert_eq!(width(DialogSize::FullWidth), 952.);
        // A narrow container caps every size at its width less margins.
        let narrow = MaterialDialog::new("d").size(DialogSize::Large);
        assert_eq!(narrow.basic_width(400.), 352.);
        // A specimen (no container) shows the widest basic dialog.
        let specimen = MaterialDialog::new("d").size(DialogSize::FullWidth);
        assert_eq!(specimen.basic_width(f32::INFINITY), 560.);
    }

    #[test]
    fn animations_move_scale_or_only_fade() {
        assert_eq!(slide_offset(DialogAnimation::SlideUp, 0.), 24.);
        assert_eq!(slide_offset(DialogAnimation::SlideDown, 0.), -24.);
        assert_eq!(slide_offset(DialogAnimation::SlideUp, 1.), 0.);
        assert_eq!(slide_offset(DialogAnimation::Fade, 0.), 0.);
        assert_eq!(slide_offset(DialogAnimation::Scale, 0.), 0.);
        let size = gpui::size(px(300.), px(200.));
        let (clip, offset) = scale_clip(size, 0.);
        assert_eq!(clip, gpui::size(px(240.), px(160.)));
        assert_eq!(offset, point(px(-30.), px(-20.)), "the clip stays centered");
        assert_eq!(scale_clip(size, 1.), (size, point(px(0.), px(0.))));
    }

    struct Contained {
        open: bool,
        scrim: bool,
        dismissals: usize,
    }
    impl Render for Contained {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(16.));
            let this = cx.entity().downgrade();
            div().size_full().pl(px(200.)).child(
                div()
                    .relative()
                    .debug_selector(|| "frame".into())
                    .w(px(500.))
                    .h(px(400.))
                    .child(
                        MaterialDialog::new("contained")
                            .headline("Title")
                            .action(
                                div()
                                    .debug_selector(|| "contained-action".into())
                                    .size(px(40.)),
                            )
                            .size(DialogSize::FullWidth)
                            .animation(DialogAnimation::Scale)
                            .close_button("Close")
                            .dismiss_on_scrim(self.scrim)
                            .contained(true)
                            .modal(self.open)
                            .on_dismiss(move |_, cx| {
                                let _ = this.update(cx, |this, cx| {
                                    this.dismissals += 1;
                                    this.open = false;
                                    cx.notify();
                                });
                            }),
                    ),
            )
        }
    }

    #[gpui::test]
    fn a_contained_dialog_presents_over_its_parent_and_dismisses(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| Contained {
            open: true,
            scrim: false,
            dismissals: 0,
        });
        for _ in 0..3 {
            draw(cx);
        }
        let frame = cx.debug_bounds("frame").unwrap();
        let action = cx.debug_bounds("contained-action").unwrap();
        assert!(frame.contains(&action.origin), "{action:?} in {frame:?}");
        // Full width: the container less 24dp margins on each side.
        let close = cx.debug_bounds("dialog-close").unwrap();
        assert!(close.right() <= frame.right() - px(24.));
        assert!(close.right() > frame.right() - px(48.));
        // The scrim does not dismiss when turned off, then does.
        let corner = frame.origin + point(px(4.), px(4.));
        cx.simulate_click(corner, Modifiers::default());
        draw(cx);
        assert_eq!(view.read_with(cx, |v, _| v.dismissals), 0);
        view.update(cx, |v, cx| {
            v.scrim = true;
            cx.notify();
        });
        draw(cx);
        cx.simulate_click(corner, Modifiers::default());
        draw(cx);
        assert_eq!(view.read_with(cx, |v, _| v.dismissals), 1);
        // The close button dismisses too.
        view.update(cx, |v, cx| {
            v.open = true;
            cx.notify();
        });
        for _ in 0..3 {
            draw(cx);
        }
        let close = cx.debug_bounds("dialog-close").unwrap();
        cx.simulate_click(close.center(), Modifiers::default());
        draw(cx);
        assert_eq!(view.read_with(cx, |v, _| v.dismissals), 2);
    }
}
