//! Status and notice surfaces: badges, chips, banners, cards and snackbars.
use crate::color::ColorRole::{self, *};
use crate::icons::IconName;
use crate::theme::color;
use crate::tokens::{elevation, layout::*, shape::*, state, typography::*};
use gpui::{prelude::*, *};
use std::rc::Rc;

use super::text::accessible_text;

/// The tone of a [`notice_banner`].
pub enum NoticeKind {
    Info,
    Danger,
}

/// A page-level notice: tinted surface, status icon and wrapping text.
/// Callers append the notice's actions, which wrap below the text when narrow.
pub fn notice_banner(
    id: impl Into<ElementId>,
    kind: NoticeKind,
    text: impl Into<SharedString>,
) -> Div {
    let (bg, tone, icon, role) = match kind {
        NoticeKind::Info => (
            ColorRole::SECONDARY_CONTAINER,
            ColorRole::PRIMARY,
            IconName::Info,
            Role::Status,
        ),
        NoticeKind::Danger => (
            ColorRole::ERROR_CONTAINER,
            ColorRole::ERROR,
            IconName::Error,
            Role::Alert,
        ),
    };
    div()
        .flex()
        .flex_wrap()
        .w_full()
        .min_w_0()
        .items_center()
        .gap(rem(12.))
        .p(rem(12.))
        .rounded(RADIUS_SMALL)
        .bg(color(bg))
        .child(icon.icon().size(ICON_SLOT).color(color(tone)))
        .child(
            accessible_text(id, text)
                .role(role)
                .flex_1()
                .flex_basis(rem(200.))
                .min_w_0()
                .whitespace_normal(),
        )
}

/// Status badge kinds; text always pairs with its tinted background.
pub enum BadgeKind {
    Success,
    Warning,
    Danger,
    Progress,
    Neutral,
    /// An identity rather than a status, e.g. "Default".
    Tonal,
}

/// Icon, padding and label belong to one container; callers append the label.
pub fn badge(kind: BadgeKind) -> Div {
    let (text, bg, icon) = match kind {
        BadgeKind::Success => (SUCCESS, SUCCESS_BG, Some(IconName::CheckCircle)),
        BadgeKind::Warning => (WARNING, WARNING_BG, Some(IconName::Warning)),
        BadgeKind::Danger => (ERROR, ERROR_CONTAINER, Some(IconName::Error)),
        // A clock would read as waiting; work in progress needs no icon.
        BadgeKind::Progress => (PRIMARY, SECONDARY_CONTAINER, None),
        // Neutral is a plain text chip without a status icon, on the
        // highest surface container: a step above the tile in light and dark
        // palettes alike (INSET sinks below the surface in dark ones).
        BadgeKind::Neutral => (ON_SURFACE_VARIANT, SURFACE_CONTAINER_HIGHEST, None),
        BadgeKind::Tonal => (ON_SECONDARY_CONTAINER, SECONDARY_CONTAINER, None),
    };
    let mut view = div()
        .flex()
        .items_center()
        .flex_shrink_0()
        .gap(rem(4.))
        .min_h(rem(24.))
        .px(rem(10.))
        .py(rem(3.))
        .rounded(RADIUS_SMALL)
        .bg(color(bg))
        .text_color(color(text))
        .text_size(TEXT_AUX)
        .font_weight(FontWeight::MEDIUM)
        .whitespace_nowrap();
    if let Some(icon) = icon {
        view = view.child(icon.icon().size(px(16.)));
    }
    view
}

/// A fact beside a label in the badge's size, outlined rather than filled:
/// e.g. the engine a tile uses.
pub fn outlined_badge(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Stateful<Div> {
    accessible_text(id, label)
        .flex_shrink_0()
        .px(rem(10.))
        .py(rem(4.))
        .rounded(RADIUS_SMALL)
        .border_1()
        .border_color(color(ColorRole::HAIRLINE))
        .text_size(TEXT_AUX)
        .line_height(rem(16.))
        .font_weight(FontWeight::MEDIUM)
        .text_color(color(ColorRole::ON_SURFACE_VARIANT))
        .whitespace_nowrap()
}

/// M3 assist chip geometry: 32px tall with 8px corners, 8px before a leading
/// icon and 12px otherwise. Callers give it their tone and label size.
pub fn chip(leading_icon: bool) -> Div {
    gpui_base::h_flex()
        .flex_shrink_0()
        .h(rem(32.))
        .pl(rems(if leading_icon { 8. } else { 12. } / 14.))
        .pr(rem(12.))
        .gap(px(6.))
        .items_center()
        .rounded(RADIUS_SMALL)
        .whitespace_nowrap()
}

/// A standard card: surface fill, tonal edge, 16px corners and level-1
/// elevation. Callers chain their own gap and contents.
pub fn card_surface() -> Div {
    gpui_base::v_flex()
        .w_full()
        .p_4()
        .bg(color(ColorRole::SURFACE))
        .border_1()
        .border_color(color(ColorRole::CARD_LINE))
        .rounded(RADIUS_CARD)
        .shadow(elevation::level1())
}

type SnackbarHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// M3 snackbar: a brief status message on the inverse surface, with an
/// optional text action and close button. The caller places it (usually
/// bottom-leading, 16px from the window edges) and removes it when done.
#[derive(gpui::IntoElement)]
pub struct Snackbar {
    id: ElementId,
    message: SharedString,
    action: Option<(SharedString, SnackbarHandler)>,
    close: Option<(SharedString, SnackbarHandler)>,
}

impl Snackbar {
    pub fn new(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            message: message.into(),
            action: None,
            close: None,
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
}

/// A button on the inverse surface, with its state layer in `fg`.
fn inverse_button(id: ElementId, fg: Rgba, handler: SnackbarHandler) -> gpui_base::Button {
    let bg = color(INVERSE_SURFACE);
    gpui_base::Button::new(id)
        .flex()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .h(rem(40.))
        .rounded_full()
        .text_color(fg)
        .hover(move |s| s.bg(state::hovered(bg, fg)))
        .active(move |s| s.bg(state::pressed(bg, fg)))
        .on_click(move |event, window, cx| handler(event, window, cx))
}

impl RenderOnce for Snackbar {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id;
        let child = |name: &'static str| ElementId::NamedChild(id.clone().into(), name.into());
        gpui_base::h_flex()
            .id(id.clone())
            .items_center()
            .min_h(rem(48.))
            .max_w(rem(560.))
            .pl(rem(16.))
            .pr(rem(if self.action.is_some() || self.close.is_some() {
                4.
            } else {
                16.
            }))
            .gap(rem(4.))
            .rounded(RADIUS_EXTRA_SMALL)
            .bg(color(INVERSE_SURFACE))
            .text_color(color(INVERSE_ON_SURFACE))
            .text_size(TEXT_BODY)
            .shadow(elevation::popover())
            .child(
                accessible_text(child("message"), self.message)
                    .role(Role::Status)
                    .flex_1()
                    .min_w_0()
                    .py(rem(14.))
                    .whitespace_normal(),
            )
            .when_some(self.action, |bar, (label, handler)| {
                bar.child(
                    inverse_button(child("action"), color(INVERSE_PRIMARY), handler)
                        .when(cfg!(test), |b| {
                            b.debug_selector(|| "snackbar-action".into())
                        })
                        .px(rem(12.))
                        .font_weight(FontWeight::MEDIUM)
                        .child(label),
                )
            })
            .when_some(self.close, |bar, (label, handler)| {
                bar.child(
                    inverse_button(child("close"), color(INVERSE_ON_SURFACE), handler)
                        .when(cfg!(test), |b| b.debug_selector(|| "snackbar-close".into()))
                        .w(rem(40.))
                        .accessibility_label(label)
                        .child(IconName::Close.icon().size(ICON_SLOT)),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // The glob import of `gpui` shadows the built-in `#[test]` that
    // `#[gpui::test]` expands to.
    use core::prelude::v1::test;
    use gpui::{Context, Modifiers, TestAppContext};

    struct Harness {
        buttons: bool,
        actions: usize,
        closes: usize,
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let snackbar = Snackbar::new("saved", "Icon saved");
            let snackbar = if self.buttons {
                snackbar
                    .action("Undo", cx.listener(|this, _, _, _| this.actions += 1))
                    .close("Dismiss", cx.listener(|this, _, _, _| this.closes += 1))
            } else {
                snackbar
            };
            div().size_full().child(snackbar)
        }
    }

    fn open(
        cx: &mut TestAppContext,
        buttons: bool,
    ) -> (Entity<Harness>, &mut gpui::VisualTestContext) {
        let (view, cx) = cx.add_window_view(move |_, _| Harness {
            buttons,
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
    fn a_plain_snackbar_has_no_buttons(cx: &mut TestAppContext) {
        let (_, cx) = open(cx, false);
        assert!(cx.debug_bounds("snackbar-action").is_none());
        assert!(cx.debug_bounds("snackbar-close").is_none());
    }
}
