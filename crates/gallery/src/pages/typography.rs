//! Typography, like md3.io's Typography style page: the fifteen type scale
//! roles as live samples of editable text, in their regular and M3
//! Expressive emphasized styles, with the text scale the whole Gallery
//! follows.
use super::super::shell;
use super::prelude::*;
use gpui_base::input::{InputEvent, InputState};
use gpui_base::text::{MarkdownExtensions, SelectionFormat, TextView, TextViewState};
use gpui_kit_material::icons::{Icon, Style, Symbol};
use gpui_kit_material::tokens::typography::TypeScale;

const SAMPLE: &str = "The quick brown fox jumps over the lazy dog";
const STYLES: [(&str, &str); 3] = [
    ("regular", "Regular"),
    ("emphasized", "Emphasized"),
    ("both", "Both"),
];

/// The sample field's width, where the card has room.
const SAMPLE_WIDTH: f32 = 360.;

/// A document with a block surface: its fences hold ordinary blocks of the document.
const RICH_TEXT: &str = "A reading view is one document, so a selection and a find go across all of it.\n\n<!-- surface:summary -->\n\nA block surface draws a run of blocks inside a container: background, shape, padding and a title of its own.\n\n- Selecting, finding and revealing work inside it and across its edges.\n- Copying leaves its fences and title behind.\n\n<!-- /surface -->\n\nThe text after it continues the same document. Drag from here up into the surface.";

/// The summary surface of `RICH_TEXT`, in the current theme's roles.
fn rich_text_extensions() -> MarkdownExtensions {
    MarkdownExtensions::default().block_surface("summary", |_, _| {
        gpui_base::v_flex()
            .w_full()
            .min_w_0()
            .px(rem(24.))
            .py(rem(20.))
            .rounded(rem(20.))
            .bg(color(ColorRole::SECONDARY_CONTAINER))
            .text_color(color(ColorRole::ON_SECONDARY_CONTAINER))
            .child(
                gpui_base::h_flex()
                    .gap(rem(8.))
                    .pb(rem(12.))
                    .items_center()
                    .child(
                        Icon::new(SharedString::from(
                            Symbol::named("auto_awesome")
                                .expect("a Material Symbol")
                                .icon()
                                .style(Style::Rounded)
                                .asset_path(),
                        ))
                        .size(rem(18.))
                        .color(color(ColorRole::ON_SECONDARY_CONTAINER)),
                    )
                    .child(
                        div()
                            .text_size(rem(14.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Summary"),
                    ),
            )
    })
}

pub struct Page {
    page: PageCx,
    pub(crate) sample: Entity<InputState>,
    /// The sample's text. Rows read this rather than the input: Kit's Input writes its
    /// state each time it renders, which would rebuild every row that read the state.
    text: SharedString,
    /// Which styles each role shows: "regular", "emphasized" or "both".
    pub(crate) style: SharedString,
    rich_text: Entity<TextViewState>,
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
            rich_text: cx.new(|cx| TextViewState::markdown(RICH_TEXT, cx)),
            _subscription: subscription,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        2 + TypeRole::ALL.len()
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

    fn render_row(
        &mut self,
        row: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let list = self.page.rows.list().clone();
        if row == 0 {
            // The field and the focus reveal around it have no width to fill,
            // so its width is worked out rather than a percentage of nothing.
            let sample_width = SAMPLE_WIDTH.min(shell::spec_content_width(window));
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
                        .w(rem(sample_width))
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
        if index == TypeRole::ALL.len() {
            return gpui_base::v_flex()
                .gap(rem(8.))
                .child(
                    spec::small_label("rich-text-label", "Block surface")
                        .text_color(color(ColorRole::ON_SURFACE)),
                )
                .child(
                    TextView::new(&self.rich_text)
                        .selection_format(SelectionFormat::Source)
                        .markdown_extensions(rich_text_extensions())
                        .text_size(rem(16.))
                        .line_height(rem(24.))
                        .text_color(color(ColorRole::ON_SURFACE)),
                )
                .into_any_element();
        }
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
