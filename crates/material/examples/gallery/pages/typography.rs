//! Typography, like md3.io's Typography style page: the fifteen type scale
//! roles as live samples of editable text, in their regular and M3
//! Expressive emphasized styles, with the text scale the whole Gallery
//! follows.
use super::prelude::*;
use gpui_base::input::{InputEvent, InputState};
use gpui_kit_material::tokens::typography::TypeScale;

const SAMPLE: &str = "The quick brown fox jumps over the lazy dog";
const STYLES: [(&str, &str); 3] = [
    ("regular", "Regular"),
    ("emphasized", "Emphasized"),
    ("both", "Both"),
];

pub struct Page {
    page: PageCx,
    pub(crate) sample: Entity<InputState>,
    /// The sample's text. Rows read this rather than the input: Kit's Input writes its
    /// state each time it renders, which would rebuild every row that read the state.
    text: SharedString,
    /// Which styles each role shows: "regular", "emphasized" or "both".
    pub(crate) style: SharedString,
    _subscription: Subscription,
}

/// Apply one role's metrics.
fn scaled<E: Styled>(element: E, scale: TypeScale) -> E {
    element
        .font_family(scale.font_family)
        .text_size(scale.font_size)
        .line_height(scale.line_height)
        .font_weight(FontWeight(scale.font_weight as f32))
}

/// "57 / 64 · 400": size, line height and weight in px at 100%.
pub(crate) fn metrics(scale: TypeScale) -> String {
    let px = |rems: Rems| (rems.0 * BASE_PX).round();
    format!(
        "{} / {} · {}",
        px(scale.font_size),
        px(scale.line_height),
        scale.font_weight
    )
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "typography",
        title: "Typography",
        description: "The 15 type scale roles as live samples you can edit, regular and emphasized, and the text size the whole Gallery follows.",
        guideline: Some("https://m3.material.io/styles/typography/overview"),
    };

    fn new(page: PageCx, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let sample = cx.new(|cx| InputState::new(window, cx).default_value(SAMPLE));
        let subscription = cx.subscribe(&sample, |this: &mut Self, sample, event, cx| {
            if matches!(event, InputEvent::Change) {
                this.text = sample.read(cx).value();
                // Longer text wraps onto more lines.
                this.page.rows.remeasure(1..=TypeRole::ALL.len());
                cx.notify();
            }
        });
        Self {
            page,
            text: SAMPLE.into(),
            sample,
            style: "both".into(),
            _subscription: subscription,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        1 + TypeRole::ALL.len()
    }

    fn row_gap(&self, row: usize) -> f32 {
        match row {
            0 => spec::HEADER_TO_BODY,
            1 => 40.,
            _ => 24.,
        }
    }

    fn interactive(&self, row: usize) -> bool {
        row == 0
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let list = self.page.rows.list().clone();
        if row == 0 {
            let shared = self.page.shared.clone();
            return div()
                .flex()
                .flex_wrap()
                .items_end()
                .gap(rem(24.))
                .child(spec::demo(
                    "scale-demo",
                    "Text size",
                    ConnectedButtonGroup::new("scale", "Text size")
                        .options([
                            ("1", "100%"),
                            ("1.25", "125%"),
                            ("1.5", "150%"),
                            ("2", "200%"),
                        ])
                        .selected(self.page.shared.read(cx).scale.clone())
                        .on_change(move |value: &SharedString, _, cx| {
                            shared.update(cx, |shared, cx| {
                                shared.scale = value.clone();
                                cx.notify();
                            })
                        })
                        .reveal("reveal-scale", &list),
                ))
                .child(spec::demo(
                    "type-style-demo",
                    "Style",
                    ConnectedButtonGroup::new("type-style", "Type style")
                        .options(STYLES)
                        .selected(self.style.clone())
                        .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                            this.style = value.clone();
                            this.page.rows.remeasure(1..=TypeRole::ALL.len());
                            cx.notify();
                        }))
                        .reveal("reveal-type-style", &list),
                ))
                .child(
                    div()
                        .debug_selector(|| "type-sample-input".into())
                        .w(rem(360.))
                        .child(
                            TextField::new("type-sample", &self.sample)
                                .variant(TextFieldVariant::Outlined)
                                .label("Sample text")
                                .w_full(),
                        )
                        .reveal("reveal-type-sample", &list),
                )
                .into_any_element();
        }
        let index = row - 1;
        let role = TypeRole::ALL[index];
        let text: SharedString = if self.text.is_empty() {
            SAMPLE.into()
        } else {
            self.text.clone()
        };
        let regular = self.style != "emphasized";
        let emphasized = self.style != "regular";
        let sample = |id: &'static str, scale: TypeScale| {
            scaled(accessible_text((id, index), text.clone()), scale)
                .w_full()
                .whitespace_normal()
                .text_color(color(ColorRole::ON_SURFACE))
        };
        div()
            .flex()
            .flex_wrap()
            .gap(rem(24.))
            .pb(rem(24.))
            .border_b_1()
            .border_color(color(ColorRole::OUTLINE_VARIANT))
            .child(
                gpui_base::v_flex()
                    .w(rem(200.))
                    .gap(rem(2.))
                    .child(
                        spec::small_label(("type-label", index), role.name())
                            .text_color(color(ColorRole::ON_SURFACE)),
                    )
                    .child(
                        spec::small_label(("type-metrics", index), metrics(role.scale()))
                            .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                    ),
            )
            .child(
                gpui_base::v_flex()
                    .flex_1()
                    .min_w(rem(240.))
                    .gap(rem(8.))
                    .when(regular, |column| {
                        column.child(sample("type-sample", role.scale()))
                    })
                    .when(emphasized, |column| {
                        column.child(sample("type-emphasized", role.emphasized()))
                    }),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn metrics_read_in_px_at_the_base_scale() {
        assert_eq!(metrics(TypeRole::DisplayLarge.scale()), "57 / 64 · 400");
        assert_eq!(metrics(TypeRole::LabelSmall.scale()), "11 / 16 · 500");
        assert_eq!(metrics(TypeRole::TitleMedium.emphasized()), "16 / 24 · 600");
    }
}
