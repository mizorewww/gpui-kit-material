//! Switch: every interaction state, selected and not, and a live switch.
use super::prelude::*;

pub struct Page {
    page: PageCx,
    checked: bool,
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "switch",
        title: "Switch",
        description: "Switches toggle the selection of an item on or off.",
        guideline: Some("https://m3.material.io/components/switch/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            page,
            checked: false,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        2
    }

    fn interactive(&self, row: usize) -> bool {
        row == 1
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        if row == 0 {
            return board("switch-states")
                .caption("Switch")
                .columns(5)
                .column_width(96.)
                .column_labels(InteractionPreview::ALL.map(InteractionPreview::name))
                .row_labels(["Selected", "Unselected"])
                .cells([true, false].into_iter().flat_map(|checked| {
                    InteractionPreview::ALL
                        .into_iter()
                        .enumerate()
                        .map(move |(index, state)| {
                            MaterialSwitch::new(
                                ("switch-state", index + if checked { 0 } else { 5 }),
                                state.name(),
                            )
                            .checked(checked)
                            .preview_state(state)
                        })
                }))
                .into_any_element();
        }
        spec::demo(
            "live-switch",
            "Switch in use",
            div()
                .flex()
                .items_center()
                .gap(rem(16.))
                .child(accessible_text("switch-label", "Enable notifications"))
                .child(
                    MaterialSwitch::new("example-switch", "Enable notifications")
                        .checked(self.checked)
                        .on_change(cx.listener(|this, checked: &bool, _, cx| {
                            this.checked = *checked;
                            cx.notify();
                        }))
                        .reveal("reveal-switch", self.page.rows.list()),
                ),
        )
        .into_any_element()
    }
}
