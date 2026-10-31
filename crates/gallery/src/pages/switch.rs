//! Switch: every interaction state, selected and not, with and without
//! handle icons, and live switches.
use super::prelude::*;

pub struct Page {
    page: PageCx,
    checked: bool,
    icons: bool,
}

/// The board's rows, in the kit's order: (selected, icons, row label).
const TYPES: [(bool, SwitchIcons, &str); 4] = [
    (true, SwitchIcons::None, "Selected"),
    (true, SwitchIcons::Both, "Selected · icon"),
    (false, SwitchIcons::None, "Unselected"),
    (false, SwitchIcons::Both, "Unselected · icon"),
];

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
            icons: true,
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
                .row_labels(TYPES.map(|(_, _, label)| label))
                .cells(
                    TYPES
                        .into_iter()
                        .enumerate()
                        .flat_map(|(kind, (checked, icons, _))| {
                            InteractionPreview::ALL.into_iter().enumerate().map(
                                move |(index, state)| {
                                    MaterialSwitch::new(
                                        ("switch-state", kind * 5 + index),
                                        state.name(),
                                    )
                                    .checked(checked)
                                    .icons(icons)
                                    .preview_state(state)
                                },
                            )
                        }),
                )
                .into_any_element();
        }
        let list = self.page.rows.list().clone();
        let line = |id: &'static str, label: &'static str, switch: AnyElement| {
            div()
                .flex()
                .items_center()
                .gap(rem(16.))
                .child(accessible_text(id, label).text_color(color(ColorRole::ON_SURFACE)))
                .child(switch)
        };
        spec::demo(
            "live-switch",
            "Switch in use",
            spec::demo_row()
                .gap(rem(48.))
                .child(line(
                    "switch-label",
                    "Enable notifications",
                    MaterialSwitch::new("example-switch", "Enable notifications")
                        .checked(self.checked)
                        .icons(if self.icons {
                            SwitchIcons::Selected
                        } else {
                            SwitchIcons::None
                        })
                        .on_change(cx.listener(|this, checked: &bool, _, cx| {
                            this.checked = *checked;
                            cx.notify();
                        }))
                        .reveal("reveal-switch", &list)
                        .into_any_element(),
                ))
                .child(line(
                    "icons-label",
                    "Show icons",
                    MaterialSwitch::new("icons-switch", "Show icons")
                        .checked(self.icons)
                        .icons(SwitchIcons::Both)
                        .on_change(cx.listener(|this, checked: &bool, _, cx| {
                            this.icons = *checked;
                            cx.notify();
                        }))
                        .reveal("reveal-icons-switch", &list)
                        .into_any_element(),
                ))
                .child(line(
                    "disabled-switch-label",
                    "Unavailable",
                    MaterialSwitch::new("disabled-switch", "Unavailable")
                        .checked(true)
                        .disabled(true)
                        .into_any_element(),
                )),
        )
        .into_any_element()
    }
}
