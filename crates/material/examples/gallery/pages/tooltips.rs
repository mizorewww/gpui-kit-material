//! Tooltips: the kit's rich tooltips (with and without actions) and plain
//! tooltips (single and multi line), and live tooltips on hover and focus.
use super::prelude::*;
use gpui_kit_material::icons::{Icon, Style, Symbol};
use std::sync::OnceLock;

fn symbol(index: usize) -> Icon {
    const NAMES: [&str; 3] = ["favorite", "share", "info"];
    static PATHS: OnceLock<[String; 3]> = OnceLock::new();
    let paths = PATHS.get_or_init(|| {
        NAMES.map(|name| {
            Symbol::named(name)
                .expect("a Material Symbol")
                .icon()
                .style(Style::Rounded)
                .asset_path()
        })
    });
    Icon::new(SharedString::new_static(paths[index].as_str()))
}

const RICH_TEXT: &str = "Supporting line text lorem ipsum dolor sit amet, consectetur";
const MULTI_LINE: &str = "Supporting text Body text string goes here psum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt";

fn action(id: (&'static str, usize), live: bool) -> MaterialButton {
    let button = material_button(id, ButtonVariant::Text).child("Action");
    if live {
        button
    } else {
        button.preview_state(InteractionPreview::Enabled)
    }
}

/// A 40dp standard icon button for the live demo.
fn icon_button(id: &'static str, icon: Icon, label: &'static str) -> impl IntoElement {
    let ink = color(ColorRole::ON_SURFACE_VARIANT);
    FocusRing::new(
        (id, 0usize),
        gpui_base::Button::new(id)
            .accessibility_label(label)
            .size(rem(40.))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(move |style| style.bg(ink.opacity(0.08)))
            .active(move |style| style.bg(ink.opacity(0.10)))
            .child(icon.size(rem(24.)).color(ink)),
    )
}

pub struct Page {
    page: PageCx,
    forced: bool,
}

const RICH: usize = 0;
const PLAIN: usize = 1;
const LIVE: usize = 2;

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "tooltips",
        title: "Tooltips",
        description: "Tooltips display brief labels or messages.",
        guideline: Some("https://m3.material.io/components/tooltips/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            page,
            forced: false,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        LIVE + 1
    }

    fn interactive(&self, row: usize) -> bool {
        row == LIVE
    }

    fn apply_capture(&mut self, state: CaptureState, cx: &mut Context<Self>) {
        // `--active` shows the live tooltips for captures.
        self.forced = state.active;
        cx.notify();
    }

    fn render_row(&mut self, row: usize, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
        match row {
            RICH => board("rich-tooltips")
                .caption("Rich tooltip")
                .columns(1)
                .column_width(RICH_TOOLTIP_WIDTH)
                .row_labels(["With actions", "Without actions"])
                .cell(
                    MaterialTooltip::rich("rich-specimen-actions", RICH_TEXT)
                        .subhead("Title")
                        .action(action(("rich-action", 0), false))
                        .action(action(("rich-action", 1), false)),
                )
                .cell(MaterialTooltip::rich("rich-specimen", RICH_TEXT).subhead("Title"))
                .into_any_element(),
            PLAIN => board("plain-tooltips")
                .caption("Plain tooltip")
                .columns(1)
                .column_width(PLAIN_TOOLTIP_MAX_WIDTH + 40.)
                .row_labels(["Single line", "Multi line"])
                .cell(MaterialTooltip::plain("plain-single", "Supporting text"))
                .cell(MaterialTooltip::plain("plain-multi", MULTI_LINE))
                .into_any_element(),
            _ => {
                let list = self.page.rows.list().clone();
                let forced = self.forced;
                let force = |tip: MaterialTooltip| {
                    if forced { tip.force_visible(true) } else { tip }
                };
                spec::demo(
                    "tooltip-demo",
                    "Tooltips in use · hover or focus",
                    spec::demo_row()
                        .pt(rem(56.))
                        .pb(rem(160.))
                        .gap(rem(48.))
                        .child(force(
                            MaterialTooltip::plain("live-favorite", "Add to favorites")
                                .trigger(icon_button("favorite", symbol(0), "Favorite")),
                        ))
                        .child(force(
                            MaterialTooltip::plain("live-share", "Share").trigger(icon_button(
                                "share",
                                symbol(1),
                                "Share",
                            )),
                        ))
                        .child(force(
                            MaterialTooltip::rich(
                                "live-rich",
                                "Rich tooltips explain a feature and can offer actions.",
                            )
                            .subhead("Learn more")
                            .action(action(("live-rich-action", 0), true))
                            .trigger(icon_button(
                                "info",
                                symbol(2),
                                "About tooltips",
                            )),
                        ))
                        .reveal("reveal-tooltip-demo", &list),
                )
                .into_any_element()
            }
        }
    }
}
