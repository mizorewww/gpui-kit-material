//! Radio button: the radio indicator, choice tiles in every state, and a
//! live radio group of choice tiles.
use super::prelude::*;

pub struct Page {
    page: PageCx,
    pub(crate) engine: usize,
}

/// The live choice group's row.
pub(crate) const CHOICE_ROW: usize = 2;

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "radio-button",
        title: "Radio button",
        description: "Radio buttons let people select one option from a set of options.",
        guideline: Some("https://m3.material.io/components/radio-button/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self { page, engine: 0 }
    }

    fn row_count(&self, _: &Window) -> usize {
        3
    }

    fn interactive(&self, row: usize) -> bool {
        row == CHOICE_ROW
    }

    fn render_row(
        &mut self,
        row: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match row {
            0 => board("radio-indicators")
                .caption("Radio indicator")
                .columns(2)
                .column_width(120.)
                .column_labels(["Selected", "Unselected"])
                .row_labels(["Enabled", "Disabled"])
                .cells(
                    [(true, true), (false, true), (true, false), (false, false)]
                        .map(|(selected, enabled)| choice_radio(selected, enabled)),
                )
                .into_any_element(),
            1 => board("choice-tile-states")
                .caption("Choice tile")
                .columns(5)
                .column_labels(InteractionPreview::ALL.map(InteractionPreview::name))
                .row_labels(["Unselected", "Selected"])
                .cells(
                    [false, true]
                        .into_iter()
                        .flat_map(|selected| {
                            InteractionPreview::ALL
                                .into_iter()
                                .enumerate()
                                .map(move |(index, state)| (selected, index, state))
                        })
                        .map(|(selected, index, state)| {
                            ChoiceTile::new(
                                ("choice-state", index + if selected { 5 } else { 0 }),
                                0,
                                1,
                                "Label",
                                selected,
                            )
                            .preview_state(state)
                            .render(window, cx)
                        }),
                )
                .into_any_element(),
            _ => {
                let engines = ["Local engine", "Cloud service", "Unavailable"];
                // A radio group: Tab enters at the selection, arrows, Home and
                // End move and select among the available choices.
                let mut choices = ChoiceGroup::new("engines", "Choice tiles").on_change(
                    cx.listener(|this, index: &usize, _, cx| {
                        this.engine = *index;
                        cx.notify();
                    }),
                );
                for (index, title) in engines.into_iter().enumerate() {
                    choices = choices.tile(
                        ChoiceTile::new(
                            ("engine", index),
                            index,
                            engines.len(),
                            title,
                            self.engine == index,
                        )
                        .tinted_icon(
                            IconName::Info.icon(),
                            [Tint::Blue, Tint::Teal, Tint::Orange][index],
                            px(40.),
                        )
                        .description(match index {
                            0 => "Runs on this computer",
                            1 => "Sends audio to the configured service",
                            _ => "Shown disabled, with its status at full strength",
                        })
                        .status(
                            (index == 2).then(|| status_chip(StatusKind::Warning).child("Missing")),
                        )
                        .enabled(index != 2),
                    );
                }
                spec::demo(
                    "choice-group",
                    "Choice tiles in a radio group",
                    choices.reveal("reveal-choices", self.page.rows.list()),
                )
                .into_any_element()
            }
        }
    }
}
