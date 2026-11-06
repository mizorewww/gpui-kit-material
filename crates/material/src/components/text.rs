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
///
/// The size, line height and weight are on the returned `Div`, which the text
/// inherits, so a caller's `text_size`, `line_height` and `font_weight` reach
/// the text: a page that needs a larger headline (M3 type scale, Headline
/// medium 28/36) sets them on the heading. This is not a whole type role: the
/// font family and letter spacing stay the heading's.
pub fn page_heading(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .items_center()
        .min_w_0()
        .text_size(TEXT_HEADLINE)
        .line_height(HEADLINE_LINE_HEIGHT)
        .font_weight(FontWeight::NORMAL)
        .child(
            accessible_text(id, title)
                .min_w_0()
                .whitespace_normal()
                .role(Role::Heading),
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

#[cfg(test)]
mod heading_tests {
    use super::*;
    use crate::tokens::typography::BASE_PX;
    use core::prelude::v1::test;
    use gpui::{Context, Render, TestAppContext, VisualTestContext, Window};

    struct Headings;
    impl Render for Headings {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(BASE_PX));
            let heading = |name: &'static str, heading: Div| {
                div()
                    .flex()
                    .debug_selector(move || name.into())
                    .child(heading)
            };
            let medium = TypeRole::HeadlineMedium.scale();
            // Each heading as wide as its text.
            div()
                .flex()
                .flex_col()
                .items_start()
                .child(heading("default", page_heading("default", "Library")))
                .child(heading(
                    "headline-medium",
                    page_heading("headline-medium", "Library")
                        .text_size(medium.font_size)
                        .line_height(medium.line_height),
                ))
        }
    }

    #[gpui::test]
    fn a_callers_type_size_reaches_the_heading_text(cx: &mut TestAppContext) {
        let (_, cx) = cx.add_window_view(|_, _| Headings);
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let bounds = |cx: &mut VisualTestContext, name| cx.debug_bounds(name).unwrap().size;
        let (default, medium) = (bounds(cx, "default"), bounds(cx, "headline-medium"));
        // Headline small is 24/32, headline medium 28/36.
        assert_eq!(default.height, px(32.));
        assert_eq!(medium.height, px(36.));
        assert!(
            medium.width > default.width,
            "28px text is wider than 24px: {default:?} and {medium:?}"
        );
    }
}

/// Apply an M3 type role's family, size, line height and weight.
pub(crate) fn type_style<E: Styled>(element: E, role: TypeRole) -> E {
    let scale = role.scale();
    element
        .font_family(scale.font_family)
        .text_size(scale.font_size)
        .line_height(scale.line_height)
        .font_weight(FontWeight(scale.font_weight as f32))
}
