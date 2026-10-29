//! Text roles that carry their accessibility semantics.
use crate::color::ColorRole;
use crate::icons::{Icon, IconName};
use crate::theme::color;
use crate::tokens::{layout::*, shape::RADIUS_CARD, typography::*};
use gpui::{prelude::*, *};

/// Text on a leaf element with a label role. GPUI's accessibility bridge
/// does not expose plain `Div` text, and keeping labels on leaves stops an
/// ancestor from replacing its controls' accessibility.
pub fn accessible_text(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Stateful<Div> {
    let value = value.into();
    let label = spoken(&value);
    div()
        .id(id)
        .role(Role::Label)
        .aria_label(label)
        .child(value)
}

/// The accessible name of displayed text. Word joiners only steer line
/// wrapping; a screen reader gets the plain words so its pacing matches the
/// text's meaning.
fn spoken(value: &SharedString) -> SharedString {
    if value.contains('\u{2060}') {
        SharedString::from(value.replace('\u{2060}', ""))
    } else {
        value.clone()
    }
}

/// Failure feedback as one accessible alert form. Callers chain their own
/// size and wrapping.
pub fn error_text(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Stateful<Div> {
    accessible_text(id, message)
        .role(Role::Alert)
        .text_color(color(ColorRole::ERROR))
}

/// An icon column and a medium-weight text column. The slot also takes a
/// live spinner, keeping state changes on the same axis.
pub fn semantic_label(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    icon: impl IntoElement,
) -> Div {
    div()
        .flex()
        .items_center()
        .min_w_0()
        .gap(ICON_GAP)
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(ICON_SLOT)
                .flex_shrink_0()
                .child(icon),
        )
        .child(
            accessible_text(id, label)
                .min_w_0()
                .whitespace_normal()
                .text_size(TEXT_BODY)
                .font_weight(FontWeight::MEDIUM),
        )
}

/// Short supporting text inside an existing item boundary. The item
/// supplies the surface; a second frame would compete with it. The info
/// icon sits in a label's icon slot, so the text starts on the label ruler.
pub fn supporting_info(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .w_full()
        .min_w_0()
        .items_start()
        .gap(ICON_GAP)
        .child(
            div()
                .flex()
                .justify_center()
                .w(ICON_SLOT)
                .h(rem(18.))
                .items_center()
                .flex_shrink_0()
                .child(
                    IconName::Info
                        .icon()
                        .size(rem(16.))
                        .color(color(ColorRole::ON_SURFACE_VARIANT)),
                ),
        )
        .child(
            accessible_text(id, text)
                .flex_1()
                .min_w_0()
                .whitespace_normal()
                .text_size(TEXT_AUX)
                .line_height(rem(18.))
                .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
        )
}

/// Standalone guidance with its own low-emphasis boundary. Inside an item,
/// use [`supporting_info`].
pub fn info_callout(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .w_full()
        .min_w_0()
        .items_start()
        .gap(rem(10.))
        .px(rem(16.))
        .py(rem(12.))
        .rounded(RADIUS_CARD)
        .bg(color(ColorRole::SURFACE_CONTAINER_HIGHEST))
        .child(
            div().mt(rem(1.)).child(
                IconName::Info
                    .icon()
                    .size(rem(18.))
                    .color(color(ColorRole::ON_SURFACE_VARIANT)),
            ),
        )
        .child(
            accessible_text(id, text)
                .flex_1()
                .min_w_0()
                .whitespace_normal()
                .text_size(TEXT_AUX)
                .line_height(rem(18.))
                .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
        )
}

/// A page title: text alone in the regular-weight headline role. A title
/// longer than its column wraps rather than running past the edge.
pub fn page_heading(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Div {
    div().flex().items_center().min_w_0().child(
        accessible_text(id, title)
            .min_w_0()
            .whitespace_normal()
            .role(Role::Heading)
            .text_size(TEXT_HEADLINE)
            .line_height(HEADLINE_LINE_HEIGHT)
            .font_weight(FontWeight::NORMAL),
    )
}

/// An M3 list section title: a quiet medium-weight label above the group
/// it names, on the group's text inset.
pub fn section_title(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Stateful<Div> {
    accessible_text(id, title)
        .role(Role::Heading)
        .text_size(TEXT_BODY)
        .line_height(rem(20.))
        .font_weight(FontWeight::MEDIUM)
        .text_color(color(ColorRole::ON_SURFACE_VARIANT))
}

/// A section title with a leading icon.
pub fn section_heading(
    id: impl Into<ElementId>,
    icon: impl Into<Icon>,
    title: impl Into<SharedString>,
) -> Div {
    div()
        .flex()
        .min_w_0()
        .items_center()
        .gap(ICON_GAP)
        .text_color(color(ColorRole::ON_SURFACE_VARIANT))
        .child(icon.into().size(rem(18.)))
        .child(section_title(id, title))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn spoken_text_drops_word_joiners_only() {
        let joined = SharedString::from("Text\u{2060}to\u{2060}speech");
        assert_eq!(spoken(&joined).as_ref(), "Texttospeech");
        let plain = SharedString::from("Plain words");
        assert_eq!(spoken(&plain), plain);
    }
}
