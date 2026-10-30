//! M3 sheets: side sheets (standard and modal) and bottom sheets (standard
//! and modal, with a drag handle, drag-to-dismiss and snapping).
//!
//! Visuals follow the Figma "Material 3 Design Kit" `Side Sheet` and
//! `Bottom sheet` components: a standard side sheet is the surface with an
//! outline-variant divider on its leading edge; a modal side sheet is
//! surface-container-low with 16dp corners over the 32% scrim. Its header
//! holds an optional back button, a title-large on-surface-variant headline
//! and a close button; actions sit under an outline-variant divider. A bottom
//! sheet is surface-container-low with 28dp top corners and a 32×4 outline
//! drag handle in a 36dp header; standard bottom sheets float at level 1,
//! modal ones over the scrim.
//!
//! Modal behavior (focus trap, Escape, scrim dismissal) is GPUI Base's
//! [`gpui_base::Sheet`] host; Material adds focus into and back out of the
//! sheet and the motion. Sheets enter and leave on the default spatial
//! spring with fades on the default effects spring; a closed sheet's content
//! leaves the tree once the close settles, so it offers no tab stops or
//! accessibility nodes off screen.
use super::dialog::{OverlayFrame, header_icon_button, modal_focus, overlay_frame, scrim};
use crate::{
    color::ColorRole,
    icons::{Icon, IconName, Style, Symbol},
    motion::{self, MotionToken},
    theme::color,
    tokens::{
        elevation,
        typography::{TypeRole, rem},
    },
};
use gpui::{prelude::*, *};
use std::{rc::Rc, sync::OnceLock};

#[cfg(not(target_family = "wasm"))]
use std::time::Instant;
#[cfg(target_family = "wasm")]
use web_time::Instant;

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// The modal side sheet's corners and the bottom sheet's top corners.
const SIDE_SHEET_RADIUS: Pixels = px(16.);
const BOTTOM_SHEET_RADIUS: Pixels = px(28.);
/// M3 caps a bottom sheet's width on large windows.
const BOTTOM_SHEET_MAX_WIDTH: f32 = 640.;
/// Pointer travel before a press on a bottom sheet becomes a drag.
const DRAG_SLOP: f32 = 4.;

fn typed<E: Styled>(element: E, role: TypeRole) -> E {
    let scale = role.scale();
    element
        .font_family(scale.font_family)
        .text_size(scale.font_size)
        .line_height(scale.line_height)
        .font_weight(FontWeight(scale.font_weight as f32))
}

fn arrow_back() -> Icon {
    static PATH: OnceLock<String> = OnceLock::new();
    Icon::new(SharedString::new_static(
        PATH.get_or_init(|| {
            Symbol::named("arrow_back")
                .expect("a Material Symbol")
                .icon()
                .style(Style::Rounded)
                .asset_path()
        })
        .as_str(),
    ))
}

/// Whether a sheet has finished closing: closed, with its reveal settled.
fn sheet_hidden(open: bool, reveal: f32) -> bool {
    !open && reveal == 0.
}

/// An M3 side sheet. By default it is a nonmodal standard sheet docked to
/// its parent's trailing edge that leaves the main content interactive;
/// [`MaterialSideSheet::modal`] presents it over the window instead, and
/// [`MaterialSideSheet::specimen`] renders it in place. Its controlled
/// visibility retargets the spatial spring, including on close.
#[derive(gpui::IntoElement)]
pub struct MaterialSideSheet {
    id: ElementId,
    open: bool,
    modal: bool,
    specimen: bool,
    width: Rems,
    headline: Option<SharedString>,
    label: Option<SharedString>,
    back: Option<(SharedString, Handler)>,
    close_label: Option<SharedString>,
    actions: Vec<AnyElement>,
    on_dismiss: Option<Handler>,
    children: Vec<AnyElement>,
}

impl MaterialSideSheet {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            open: false,
            modal: false,
            specimen: false,
            width: rem(320.),
            headline: None,
            label: None,
            back: None,
            close_label: None,
            actions: Vec::new(),
            on_dismiss: None,
            children: vec![],
        }
    }
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    pub fn width(mut self, width: Rems) -> Self {
        self.width = width;
        self
    }
    /// Present over the window with a scrim; Escape, the scrim and the close
    /// button dismiss it.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }
    /// Render the sheet's surface in place, without motion (a specimen).
    pub fn specimen(mut self, specimen: bool) -> Self {
        self.specimen = specimen;
        self
    }
    /// The title-large headline in the header.
    pub fn headline(mut self, headline: impl Into<SharedString>) -> Self {
        self.headline = Some(headline.into());
        self
    }
    /// The modal sheet's accessible name when it differs from (or there is
    /// no) headline.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// A leading back button named `label`.
    pub fn back(
        mut self,
        label: impl Into<SharedString>,
        on_back: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.back = Some((label.into(), Rc::new(on_back)));
        self
    }
    /// A trailing close button named `label`; it calls `on_dismiss`.
    pub fn close_button(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = Some(label.into());
        self
    }
    /// An action under the divider at the sheet's bottom, in call order.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
}

impl ParentElement for MaterialSideSheet {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

impl MaterialSideSheet {
    fn has_header(&self) -> bool {
        self.headline.is_some() || self.back.is_some() || self.close_label.is_some()
    }

    /// The header, content and actions in a column.
    fn body(self) -> Div {
        let has_header = self.has_header();
        let dismiss = self.on_dismiss.clone();
        let back = self.back;
        let header = has_header.then(|| {
            div()
                .flex()
                .items_start()
                .pt(rem(12.))
                .pr(rem(12.))
                .pb(rem(16.))
                .pl(rem(if back.is_some() { 4. } else { 24. }))
                .when_some(back, |header, (label, on_back)| {
                    header.child(header_icon_button(
                        "sheet-back",
                        arrow_back(),
                        label,
                        move |_, window, cx| on_back(window, cx),
                    ))
                })
                .child(div().flex_1().min_w_0().h(rem(40.)).pt(rem(12.)).when_some(
                    self.headline,
                    |content, headline| {
                        content.child(
                            typed(div(), TypeRole::TitleLarge)
                                .truncate()
                                .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                                .child(headline),
                        )
                    },
                ))
                .when_some(self.close_label, |header, label| {
                    header.child(header_icon_button(
                        "sheet-close",
                        IconName::Close.icon(),
                        label,
                        move |_, window, cx| {
                            if let Some(dismiss) = &dismiss {
                                dismiss(window, cx);
                            }
                        },
                    ))
                })
        });
        let has_actions = !self.actions.is_empty();
        div()
            .flex()
            .flex_col()
            .size_full()
            .when(has_actions, |body| body.pb(rem(20.)))
            .children(header)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .children(self.children),
            )
            .when(has_actions, |body| {
                body.child(
                    div()
                        .h(px(1.))
                        .w_full()
                        .bg(color(ColorRole::OUTLINE_VARIANT)),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(rem(8.))
                        .pt(rem(12.))
                        .px(rem(24.))
                        .children(self.actions),
                )
            })
    }

    /// The modal container: surface-container-low with 16dp corners.
    fn modal_surface(self, corners: Corners<Pixels>) -> Div {
        let width = self.width;
        div()
            .w(width)
            .h_full()
            .overflow_hidden()
            .rounded_tl(corners.top_left)
            .rounded_tr(corners.top_right)
            .rounded_br(corners.bottom_right)
            .rounded_bl(corners.bottom_left)
            .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
            .child(self.body())
    }

    /// The standard container: the surface with a leading divider.
    fn standard_surface(self) -> Div {
        div()
            .size_full()
            .flex()
            .bg(color(ColorRole::SURFACE))
            .child(
                div()
                    .w(px(1.))
                    .h_full()
                    .flex_shrink_0()
                    .bg(color(ColorRole::OUTLINE_VARIANT)),
            )
            .child(div().flex_1().min_w_0().h_full().child(self.body()))
    }
}

impl RenderOnce for MaterialSideSheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.specimen {
            let width = self.width;
            let id = self.id.clone();
            return if self.modal {
                div()
                    .id(id)
                    .h_full()
                    .child(self.modal_surface(Corners::all(SIDE_SHEET_RADIUS)))
                    .into_any_element()
            } else {
                div()
                    .id(id)
                    .w(width)
                    .h_full()
                    .child(self.standard_surface())
                    .into_any_element()
            };
        }
        if self.modal {
            let id = self.id.clone();
            let open = self.open;
            let frame = overlay_frame(&id, open, window, cx);
            let focus = modal_focus(&id, open, window, cx);
            if frame.settled_closed(open) {
                return div().into_any_element();
            }
            let width = self.width.to_pixels(window.rem_size());
            let dismiss = self.on_dismiss.clone().filter(|_| open);
            let name = self.label.clone().or_else(|| self.headline.clone());
            let surface = div()
                .id(id)
                .role(Role::Dialog)
                .when_some(name, |surface, name| surface.aria_label(name))
                .absolute()
                .top_0()
                .bottom_0()
                .right(width * (frame.spatial - 1.))
                .opacity(frame.opacity())
                .occlude()
                .child(self.modal_surface(Corners {
                    top_left: SIDE_SHEET_RADIUS,
                    bottom_left: SIDE_SHEET_RADIUS,
                    ..Corners::default()
                }));
            return modal_host(focus, frame, dismiss, surface).into_any_element();
        }
        let reveal = motion::value(
            (self.id.clone(), "reveal"),
            if self.open { 1. } else { 0. },
            window,
            cx,
        );
        let progress = reveal.clamp(0., 1.);
        let width = self.width.to_pixels(window.rem_size());
        let hidden = sheet_hidden(self.open, reveal);
        let id = self.id.clone();
        div()
            .id(id)
            .absolute()
            .top_0()
            .bottom_0()
            .right(width * (progress - 1.))
            .w(width)
            .opacity(progress)
            // Once the close has settled, the sheet's content leaves the tree,
            // so it offers no tab stops or accessibility nodes off screen. The
            // spring lands exactly on its target; an overshoot does not.
            .when(!hidden, |sheet| sheet.child(self.standard_surface()))
            .into_any_element()
    }
}

/// Base's modal sheet host over the window, with the Material scrim.
fn modal_host(
    focus: FocusHandle,
    frame: OverlayFrame,
    dismiss: Option<Handler>,
    surface: impl IntoElement,
) -> impl IntoElement {
    let escape = dismiss.clone();
    deferred(
        ModalSheetHost {
            focus,
            fade: frame.fade,
            dismiss: escape,
            surface: surface.into_any_element(),
        }
        .into_any_element(),
    )
    .with_priority(10)
}

#[derive(gpui::IntoElement)]
struct ModalSheetHost {
    focus: FocusHandle,
    fade: f32,
    dismiss: Option<Handler>,
    surface: AnyElement,
}

impl RenderOnce for ModalSheetHost {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let dismiss = self.dismiss;
        gpui_base::Sheet::new(cx)
            .focus_handle(self.focus)
            .overlay_closable(dismiss.is_some())
            .on_close(move |_, window, cx| {
                if let Some(dismiss) = &dismiss {
                    dismiss(window, cx);
                }
            })
            .overlay(scrim(self.fade))
            .surface(self.surface)
    }
}

/// An in-progress drag of a bottom sheet, in window pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Drag {
    start: f32,
    offset: f32,
    moved: bool,
}

/// A snap after release: the residual offset eases from `from` to 0.
#[derive(Clone, Copy, Debug)]
struct Settle {
    from: f32,
    started: Instant,
}

struct BottomSheetState {
    height: Pixels,
    /// Whether `height` is the sheet's measured height yet (it starts as
    /// the viewport's).
    measured: bool,
    expanded: bool,
    drag: Option<Drag>,
    settle: Option<Settle>,
}

/// Where a released bottom sheet goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SnapTarget {
    Expanded,
    Partial,
    Dismiss,
}

/// The rest position nearest to `position` (px below fully expanded) for a
/// sheet `height` tall, with an optional partial detent showing `partial`.
pub(crate) fn snap_target(position: f32, height: f32, partial: Option<f32>) -> SnapTarget {
    let mut best = (SnapTarget::Expanded, position.abs());
    if let Some(partial) = partial.filter(|&p| p < height) {
        let at = height - partial;
        if (position - at).abs() < best.1 {
            best = (SnapTarget::Partial, (position - at).abs());
        }
    }
    if (position - height).abs() < best.1 {
        best = (SnapTarget::Dismiss, 0.);
    }
    best.0
}

/// An M3 bottom sheet. Standard sheets dock to their parent's bottom edge
/// and leave the content interactive; modal ones present over the window.
/// Dragging the sheet down past its nearest rest dismisses it; otherwise it
/// snaps to the expanded or partial detent.
#[derive(gpui::IntoElement)]
pub struct MaterialBottomSheet {
    id: ElementId,
    open: Option<bool>,
    modal: bool,
    label: Option<SharedString>,
    drag_handle: Option<SharedString>,
    partial: Option<Rems>,
    on_dismiss: Option<Handler>,
    children: Vec<AnyElement>,
}

impl MaterialBottomSheet {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            open: None,
            modal: false,
            label: None,
            drag_handle: None,
            partial: None,
            on_dismiss: None,
            children: Vec::new(),
        }
    }
    /// Present the sheet while `open`. Without it the sheet renders in place
    /// (a specimen).
    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }
    /// The modal sheet's accessible name.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// Show the drag handle; `label` is its accessible name. Activating it
    /// toggles between the partial and expanded detents.
    pub fn drag_handle(mut self, label: impl Into<SharedString>) -> Self {
        self.drag_handle = Some(label.into());
        self
    }
    /// A partially expanded detent showing `height` of the sheet; the sheet
    /// opens there and expands when dragged up.
    pub fn partial_height(mut self, height: Rems) -> Self {
        self.partial = Some(height);
        self
    }
    /// Called when a drag, Escape or the scrim dismisses the sheet.
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
}

impl ParentElement for MaterialBottomSheet {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

fn drag_handle_bar() -> Div {
    div()
        .w(rem(32.))
        .h(rem(4.))
        .rounded_full()
        .bg(color(ColorRole::OUTLINE))
}

impl MaterialBottomSheet {
    /// The sheet surface. `drag_layer` sits under the handle and content;
    /// the content blocks it, so drags start from the handle row and the
    /// sheet's own background, never from text, sliders or buttons inside.
    fn surface(
        modal: bool,
        drag_layer: Option<AnyElement>,
        handle: Option<AnyElement>,
        children: Vec<AnyElement>,
    ) -> Div {
        div()
            .relative()
            .w_full()
            .max_w(rem(BOTTOM_SHEET_MAX_WIDTH))
            .flex()
            .flex_col()
            .items_center()
            .rounded_t(BOTTOM_SHEET_RADIUS)
            .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
            .when(!modal, |sheet| sheet.shadow(elevation::level1()))
            .children(drag_layer)
            .children(handle)
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .block_mouse_except_scroll()
                    .children(children),
            )
    }
}

/// The settle progress at `elapsed` seconds on the default spatial curve.
fn settle_progress(elapsed: f32, reduced: bool) -> f32 {
    if reduced {
        return 1.;
    }
    let curve = motion::curve(MotionToken::DefaultSpatial);
    let duration = curve.duration().as_secs_f32().max(f32::EPSILON);
    if elapsed >= duration {
        1.
    } else {
        curve.ease()(elapsed / duration)
    }
}

impl RenderOnce for MaterialBottomSheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let modal = self.modal;
        let Some(open) = self.open else {
            let handle = self.drag_handle.map(|_| {
                div()
                    .w_full()
                    .flex()
                    .justify_center()
                    .py(rem(16.))
                    .child(drag_handle_bar())
                    .into_any_element()
            });
            return div()
                .id(self.id)
                .w_full()
                .h_full()
                .flex()
                .flex_col()
                .justify_end()
                .items_center()
                .child(Self::surface(modal, None, handle, self.children).flex_1())
                .into_any_element();
        };
        let id = self.id.clone();
        let frame = overlay_frame(&id, open, window, cx);
        let focus = modal.then(|| modal_focus(&id, open, window, cx));
        let state = window.use_keyed_state((id.clone(), "bottom-sheet"), cx, |window, _| {
            BottomSheetState {
                height: window.viewport_size().height,
                measured: false,
                expanded: false,
                drag: None,
                settle: None,
            }
        });
        if frame.settled_closed(open) {
            state.update(cx, |state, _| {
                state.expanded = false;
                state.drag = None;
                state.settle = None;
            });
            return div().into_any_element();
        }
        let reduced = cx.reduce_motion();
        let partial = self
            .partial
            .map(|partial| f32::from(partial.to_pixels(window.rem_size())));
        let (height, measured, expanded, drag, settle) = {
            let s = state.read(cx);
            (
                f32::from(s.height),
                s.measured,
                s.expanded,
                s.drag,
                s.settle,
            )
        };
        let visible = match partial {
            Some(partial) if !expanded => partial.min(height),
            _ => height,
        };
        let rest = height - visible;
        let residual = match (drag, settle) {
            (Some(drag), _) if drag.moved => drag.offset,
            (_, Some(settle)) => {
                let t = settle_progress(settle.started.elapsed().as_secs_f32(), reduced);
                if t >= 1. {
                    state.update(cx, |state, _| state.settle = None);
                    0.
                } else {
                    window.request_animation_frame();
                    settle.from * (1. - t)
                }
            }
            _ => 0.,
        };
        // Upward drags stop at the expanded detent.
        let position = (rest + residual).max(0.);
        let offset = position + (1. - frame.spatial) * height;
        let dismiss = self.on_dismiss.clone().filter(|_| open);

        let handle = self.drag_handle.map(|label| {
            let toggle = state.clone();
            div()
                .w_full()
                .flex()
                .justify_center()
                .child(
                    gpui_base::Button::new("drag-handle")
                        .accessibility_label(label)
                        .w(rem(48.))
                        .h(rem(36.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_grab()
                        .child(drag_handle_bar())
                        .on_click(move |_, window, cx| {
                            if partial.is_some() {
                                toggle.update(cx, |state, _| {
                                    state.expanded = !state.expanded;
                                    state.settle = None;
                                });
                                window.refresh();
                            }
                        }),
                )
                .into_any_element()
        });

        let measure = state.clone();
        let tracker = state.clone();
        let release_dismiss = dismiss.clone();
        let dragging = drag.is_some();
        let starter = state.clone();
        let drag_layer = canvas(
            |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
            move |_, hitbox, window, _| {
                let starter = starter.clone();
                window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                    if phase != DispatchPhase::Bubble
                        || event.button != MouseButton::Left
                        || !hitbox.is_hovered(window)
                    {
                        return;
                    }
                    let y = f32::from(event.position.y);
                    starter.update(cx, |state, _| {
                        state.drag = Some(Drag {
                            start: y,
                            offset: 0.,
                            moved: false,
                        });
                        state.settle = None;
                    });
                    window.refresh();
                });
            },
        )
        .absolute()
        .size_full()
        .into_any_element();
        let sheet = div()
            .id(id.clone())
            .when(modal, |sheet| {
                sheet
                    .role(Role::Dialog)
                    .when_some(self.label, |sheet, label| sheet.aria_label(label))
            })
            .absolute()
            .left_0()
            .right_0()
            .bottom(px(-offset))
            .flex()
            .justify_center()
            .opacity(frame.opacity())
            // Until its height is measured the sheet would sit at the
            // viewport-height guess; the next frame places it at its detent.
            .when(!measured, |sheet| sheet.invisible())
            .occlude()
            .child(
                Self::surface(modal, Some(drag_layer), handle, self.children).child(
                    canvas(
                        |_, _, _| {},
                        move |_, _, window, _| {
                            if !dragging {
                                return;
                            }
                            let moving = tracker.clone();
                            window.on_mouse_event(
                                move |event: &MouseMoveEvent, phase, window, cx| {
                                    if phase != DispatchPhase::Bubble {
                                        return;
                                    }
                                    moving.update(cx, |state, _| {
                                        if let Some(drag) = state.drag.as_mut() {
                                            let delta = f32::from(event.position.y) - drag.start;
                                            drag.moved |= delta.abs() > DRAG_SLOP;
                                            drag.offset = delta;
                                        }
                                    });
                                    window.refresh();
                                },
                            );
                            let releasing = tracker.clone();
                            let dismiss = release_dismiss.clone();
                            window.on_mouse_event(move |_: &MouseUpEvent, phase, window, cx| {
                                if phase != DispatchPhase::Bubble {
                                    return;
                                }
                                let Some(drag) = releasing.update(cx, |state, _| state.drag.take())
                                else {
                                    return;
                                };
                                window.refresh();
                                if !drag.moved {
                                    return;
                                }
                                let position = (rest + drag.offset).max(0.);
                                let target = snap_target(position, height, partial);
                                let new_rest = match target {
                                    SnapTarget::Expanded => 0.,
                                    SnapTarget::Partial => height - partial.unwrap_or(height),
                                    SnapTarget::Dismiss => height,
                                };
                                releasing.update(cx, |state, _| {
                                    state.expanded = target == SnapTarget::Expanded;
                                    let from = position
                                        - if target == SnapTarget::Dismiss {
                                            rest
                                        } else {
                                            new_rest
                                        };
                                    state.settle = Some(Settle {
                                        from,
                                        started: Instant::now(),
                                    });
                                });
                                if target == SnapTarget::Dismiss
                                    && let Some(dismiss) = &dismiss
                                {
                                    dismiss(window, cx);
                                }
                            });
                        },
                    )
                    .absolute()
                    .size_full(),
                ),
            )
            .child(
                // Measure the sheet's height for the next frame's offsets.
                canvas(
                    move |bounds, window, cx| {
                        let height = bounds.size.height;
                        let stale = {
                            let state = measure.read(cx);
                            !state.measured || state.height != height
                        };
                        if stale {
                            measure.update(cx, |state, _| {
                                state.height = height;
                                state.measured = true;
                            });
                            // This frame used the old height (`refresh` is
                            // ignored while drawing): draw again at the detent.
                            window.request_animation_frame();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .top_0()
                .left_0()
                .w(px(1.))
                .h_full(),
            );

        match focus {
            Some(focus) => modal_host(focus, frame, dismiss, sheet).into_any_element(),
            None => div()
                .absolute()
                .inset_0()
                .overflow_hidden()
                .child(sheet)
                .into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::{Context, TestAppContext, VisualTestContext};

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
    }

    struct SheetHarness(bool);
    impl Render for SheetHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().relative().size_full().overflow_hidden().child(
                MaterialSideSheet::new("sheet")
                    .open(self.0)
                    .width(rem(100.))
                    .child(div().debug_selector(|| "sheet-body".into()).size_full()),
            )
        }
    }
    #[gpui::test]
    fn reduced_motion_sheet_opens_and_closes_outside_viewport(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let (view, cx) = cx.add_window_view(|_, _| SheetHarness(false));
        draw(cx);
        assert!(cx.debug_bounds("sheet-body").is_none());
        view.update(cx, |v, cx| {
            v.0 = true;
            cx.notify();
        });
        draw(cx);
        let open = cx.debug_bounds("sheet-body").unwrap();
        let viewport = cx.update(|window, _| window.viewport_size());
        assert_eq!(open.right(), viewport.width, "an open sheet meets the edge");
        view.update(cx, |v, cx| {
            v.0 = false;
            cx.notify();
        });
        draw(cx);
        assert!(
            cx.debug_bounds("sheet-body").is_none(),
            "a settled closed sheet renders no content"
        );
    }

    #[test]
    fn the_sheet_keeps_its_content_until_the_close_settles() {
        assert!(
            !sheet_hidden(true, 0.),
            "opening starts from 0 with content"
        );
        assert!(
            !sheet_hidden(false, 0.4),
            "closing animates its content out"
        );
        assert!(!sheet_hidden(false, -0.01), "an overshoot is still moving");
        assert!(sheet_hidden(false, 0.));
    }

    struct FocusSheetHarness {
        open: bool,
        focus: FocusHandle,
    }
    impl Render for FocusSheetHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().relative().size_full().child(
                MaterialSideSheet::new("focus-sheet").open(self.open).child(
                    div()
                        .id("sheet-control")
                        .track_focus(&self.focus)
                        .debug_selector(|| "sheet-control".into())
                        .size(px(20.)),
                ),
            )
        }
    }

    #[gpui::test]
    fn a_closed_sheet_offers_no_tab_stops(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let (view, cx) = cx.add_window_view(|_, cx| FocusSheetHarness {
            open: true,
            focus: cx.focus_handle(),
        });
        draw(cx);
        assert!(cx.debug_bounds("sheet-control").is_some());
        view.update(cx, |v, cx| {
            v.open = false;
            cx.notify();
        });
        draw(cx);
        assert!(cx.debug_bounds("sheet-control").is_none());
        cx.update(|window, cx| {
            window.focus_next(cx);
            let focus = view.read(cx).focus.clone();
            assert!(!focus.is_focused(window));
        });
    }

    #[test]
    fn released_sheets_snap_to_the_nearest_rest_or_dismiss() {
        // A 400px sheet with a 200px partial detent (rest at 200 below).
        assert_eq!(snap_target(30., 400., Some(200.)), SnapTarget::Expanded);
        assert_eq!(snap_target(180., 400., Some(200.)), SnapTarget::Partial);
        assert_eq!(snap_target(330., 400., Some(200.)), SnapTarget::Dismiss);
        // Without a partial detent, past half the height dismisses.
        assert_eq!(snap_target(190., 400., None), SnapTarget::Expanded);
        assert_eq!(snap_target(210., 400., None), SnapTarget::Dismiss);
        // A partial detent taller than the sheet is ignored.
        assert_eq!(snap_target(250., 400., Some(500.)), SnapTarget::Dismiss);
    }

    #[test]
    fn reduced_motion_settles_at_once() {
        assert_eq!(settle_progress(0., true), 1.);
        assert!(settle_progress(0., false) < 1.);
        assert_eq!(settle_progress(10., false), 1.);
    }

    struct BottomHarness {
        open: bool,
        dismissals: usize,
    }
    impl Render for BottomHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().relative().size(px(400.)).child(
                MaterialBottomSheet::new("bottom")
                    .open(self.open)
                    .drag_handle("Drag handle")
                    .on_dismiss({
                        let this = cx.entity().downgrade();
                        move |_, cx| {
                            let _ = this.update(cx, |this, cx| {
                                this.dismissals += 1;
                                this.open = false;
                                cx.notify();
                            });
                        }
                    })
                    .child(div().debug_selector(|| "bottom-body".into()).h(px(200.))),
            )
        }
    }

    #[gpui::test]
    fn dragging_a_bottom_sheet_down_dismisses_it(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| BottomHarness {
            open: true,
            dismissals: 0,
        });
        draw(cx);
        draw(cx);
        let body = cx.debug_bounds("bottom-body").unwrap();
        assert_eq!(body.bottom(), px(400.), "an open sheet meets the bottom");
        // Drags start from the handle row above the content.
        let start = point(body.left() + px(20.), body.top() - px(10.));
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        draw(cx);
        cx.simulate_mouse_move(
            point(start.x, start.y + px(60.)),
            Some(MouseButton::Left),
            Modifiers::default(),
        );
        draw(cx);
        let dragged = cx.debug_bounds("bottom-body").unwrap();
        assert_eq!(dragged.top() - body.top(), px(60.), "the sheet follows");
        cx.simulate_mouse_up(
            point(start.x, start.y + px(60.)),
            MouseButton::Left,
            Modifiers::default(),
        );
        draw(cx);
        assert_eq!(view.read_with(cx, |v, _| v.dismissals), 0, "snapped back");
        assert_eq!(cx.debug_bounds("bottom-body").unwrap().top(), body.top());
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        draw(cx);
        cx.simulate_mouse_move(
            point(start.x, start.y + px(180.)),
            Some(MouseButton::Left),
            Modifiers::default(),
        );
        draw(cx);
        cx.simulate_mouse_up(
            point(start.x, start.y + px(180.)),
            MouseButton::Left,
            Modifiers::default(),
        );
        draw(cx);
        assert_eq!(view.read_with(cx, |v, _| v.dismissals), 1);
        // Reduced motion keeps the fade; the content leaves once it settles.
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(1));
        draw(cx);
        draw(cx);
        assert!(cx.debug_bounds("bottom-body").is_none());
    }

    #[gpui::test]
    fn pressing_content_does_not_drag_the_sheet(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| BottomHarness {
            open: true,
            dismissals: 0,
        });
        draw(cx);
        draw(cx);
        let body = cx.debug_bounds("bottom-body").unwrap();
        let start = body.center();
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        draw(cx);
        let end = point(start.x, start.y + px(180.));
        cx.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::default());
        draw(cx);
        assert_eq!(
            cx.debug_bounds("bottom-body").unwrap().top(),
            body.top(),
            "a press on the content (text, a slider) moved the sheet"
        );
        cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        draw(cx);
        assert_eq!(view.read_with(cx, |v, _| v.dismissals), 0);
    }

    struct PartialHarness;
    impl Render for PartialHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().relative().size(px(400.)).child(
                MaterialBottomSheet::new("partial")
                    .open(true)
                    .partial_height(rem(100.))
                    .child(div().debug_selector(|| "partial-body".into()).h(px(300.))),
            )
        }
    }

    #[gpui::test]
    fn a_bottom_sheet_opens_at_its_measured_detent(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (_, cx) = cx.add_window_view(|window, _| {
            window.set_rem_size(px(crate::tokens::typography::BASE_PX));
            PartialHarness
        });
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        // The first frame guessed the viewport height; it asks for another.
        let frames = cx.update(|window, cx| window.simulate_next_frame(cx));
        assert!(frames > 0, "the measured height was stored silently");
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let body = cx.debug_bounds("partial-body").unwrap();
        // 100dp of the sheet shows above the bottom edge.
        assert_eq!(body.top(), px(300.), "{body:?}");
        let frames = cx.update(|window, cx| window.simulate_next_frame(cx));
        assert_eq!(frames, 0, "a settled sheet requests nothing");
    }
}
