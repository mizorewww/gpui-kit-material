//! Status and notice surfaces: badges, chips, banners and cards.
use crate::color::ColorRole::{self, *};
use crate::icons::IconName;
use crate::theme::color;
use crate::tokens::{elevation, layout::*, shape::*, typography::*};
use gpui::{prelude::*, *};

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
            ColorRole::ACCENT,
            IconName::Info,
            Role::Status,
        ),
        NoticeKind::Danger => (
            ColorRole::DANGER_BG,
            ColorRole::DANGER,
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
        BadgeKind::Danger => (DANGER, DANGER_BG, Some(IconName::Error)),
        // A clock would read as waiting; work in progress needs no icon.
        BadgeKind::Progress => (ACCENT, SECONDARY_CONTAINER, None),
        // Neutral is a plain text chip without a status icon, on the
        // highest surface container: a step above the tile in light and dark
        // palettes alike (INSET sinks below the surface in dark ones).
        BadgeKind::Neutral => (GRAY, SURFACE_HIGH, None),
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
        .text_color(color(ColorRole::GRAY))
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
