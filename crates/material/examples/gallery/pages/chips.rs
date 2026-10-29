//! Chips: live chip groups, then input, assist, filter and suggestion chips
//! in every state, flat and elevated, and the status chips.
use super::prelude::*;

pub struct Page {
    page: PageCx,
    /// Live input chips still present, as indices into `INPUTS`.
    inputs: Vec<usize>,
    /// Live filter chips' selection, by index.
    filters: [bool; 6],
    /// The selected live suggestion.
    suggestion: Option<usize>,
}

/// The kit's chip state rows: (forced state, dragged, row label).
const STATES: [(InteractionPreview, bool, &str); 6] = [
    (InteractionPreview::Enabled, false, "Enabled"),
    (InteractionPreview::Hovered, false, "Hovered"),
    (InteractionPreview::Focused, false, "Focused"),
    (InteractionPreview::Pressed, false, "Pressed"),
    (InteractionPreview::Enabled, true, "Dragged"),
    (InteractionPreview::Disabled, false, "Disabled"),
];

const LABEL: &str = "Label";
/// The state rows' labels for unselected or selected chips.
fn state_labels(selected: bool) -> [&'static str; 6] {
    if selected {
        [
            "Enabled · selected",
            "Hovered · selected",
            "Focused · selected",
            "Pressed · selected",
            "Dragged · selected",
            "Disabled · selected",
        ]
    } else {
        STATES.map(|(_, _, name)| name)
    }
}
const FILTERS: [&str; 6] = ["Breakfast", "Lunch", "Dinner", "Snacks", "Vegan", "Quick"];
const SUGGESTIONS: [&str; 4] = ["Sounds good", "On my way", "Call me later", "Thanks!"];
const INPUTS: [&str; 5] = ["Ada", "Grace", "Alan", "Edsger", "Barbara"];

/// Body rows.
const GROUPS: usize = 0;
const INPUT: usize = 1; // two bands
const ASSIST: usize = 3;
const FILTER: usize = 4; // four bands: flat unselected/selected, elevated
const SUGGESTION: usize = 8; // two bands
const STATUS: usize = 10;

/// A round placeholder avatar: the person's initial on a tonal container.
fn avatar(initial: &'static str) -> Div {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(color(ColorRole::PRIMARY_CONTAINER))
        .text_color(color(ColorRole::ON_PRIMARY_CONTAINER))
        .text_size(rem(12.))
        .child(initial)
}

/// A labelled line of the chip-group demo.
fn group_line(id: &'static str, label: &'static str, chips: impl IntoElement) -> Div {
    div()
        .flex()
        .items_center()
        .gap(rem(16.))
        .w_full()
        .min_w_0()
        .child(
            div()
                .w(rem(120.))
                .flex_shrink_0()
                .child(spec::small_label(id, label)),
        )
        .child(chips)
}

impl Page {
    fn groups(&self, cx: &mut Context<Self>) -> AnyElement {
        let list = self.page.rows.list().clone();
        let inputs = self.inputs.iter().map(|&index| {
            let view = cx.entity();
            let name = INPUTS[index];
            InputChip::new(("live-input", index), name)
                .avatar(avatar(&name[..1]))
                .on_remove("Remove", move |_, cx| {
                    view.update(cx, |this, cx| {
                        this.inputs.retain(|&n| n != index);
                        cx.notify();
                    })
                })
                .reveal(("reveal-live-input", index), &list)
        });
        let snackbar = |shared: Entity<GalleryShared>| {
            move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
                shared.update(cx, |shared, cx| {
                    shared.snackbar = true;
                    cx.notify();
                })
            }
        };
        let assists = ["Add to calendar", "Share", "Directions"]
            .into_iter()
            .enumerate()
            .map(|(index, label)| {
                AssistChip::new(("live-assist", index), label)
                    .icon(spec::star_icon())
                    .elevated(index == 0)
                    .on_click(snackbar(self.page.shared.clone()))
                    .reveal(("reveal-live-assist", index), &list)
            });
        let filters = FILTERS.into_iter().enumerate().map(|(index, label)| {
            FilterChip::new(("live-filter", index), label)
                .selected(self.filters[index])
                .on_toggle(cx.listener(move |this, next: &bool, _, cx| {
                    this.filters[index] = *next;
                    cx.notify();
                }))
                .reveal(("reveal-live-filter", index), &list)
        });
        let suggestions = SUGGESTIONS.into_iter().enumerate().map(|(index, label)| {
            SuggestionChip::new(("live-suggestion", index), label)
                .selected(self.suggestion == Some(index))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.suggestion = (this.suggestion != Some(index)).then_some(index);
                    cx.notify();
                }))
                .reveal(("reveal-live-suggestion", index), &list)
        });
        spec::demo(
            "chip-groups",
            "Chip groups",
            div()
                .flex()
                .flex_col()
                .gap(rem(16.))
                .child(group_line(
                    "input-group-label",
                    "Input chips",
                    spec::demo_row().gap(rem(8.)).children(inputs),
                ))
                .child(group_line(
                    "assist-group-label",
                    "Assistive chips",
                    spec::demo_row().gap(rem(8.)).children(assists),
                ))
                .child(group_line(
                    "filter-group-label",
                    "Filter chips · single row",
                    // A single row scrolls instead of wrapping.
                    div()
                        .id("filter-scroll")
                        .flex()
                        .gap(rem(8.))
                        .flex_1()
                        .min_w_0()
                        .overflow_x_scroll()
                        .children(filters),
                ))
                .child(group_line(
                    "suggestion-group-label",
                    "Suggestion chips",
                    spec::demo_row().gap(rem(8.)).children(suggestions),
                )),
        )
        .into_any_element()
    }
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "chips",
        title: "Chips",
        description: "Chips help people enter information, make selections, filter content, or trigger actions.",
        guideline: Some("https://m3.material.io/components/chips/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            page,
            inputs: (0..INPUTS.len()).collect(),
            filters: [true, false, true, false, false, false],
            suggestion: None,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        STATUS + 1
    }

    fn interactive(&self, row: usize) -> bool {
        row == GROUPS
    }

    fn row_gap(&self, row: usize) -> f32 {
        match row {
            0 => spec::HEADER_TO_BODY,
            r if (INPUT..INPUT + 2).contains(&r) => Band::of(r - INPUT, 2).row_gap(),
            r if (FILTER..FILTER + 4).contains(&r) => Band::of((r - FILTER) % 2, 2).row_gap(),
            r if (SUGGESTION..SUGGESTION + 2).contains(&r) => Band::of(r - SUGGESTION, 2).row_gap(),
            _ => spec::ROW_GAP,
        }
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match row {
            GROUPS => self.groups(cx),
            r if (INPUT..INPUT + 2).contains(&r) => {
                let selected = r == INPUT + 1;
                let offset = if selected { 100 } else { 0 };
                board(("input-chips", r))
                    .caption("Input chips")
                    .band(Band::of(r - INPUT, 2))
                    .columns(6)
                    .column_width(112.)
                    .column_labels([
                        "Label",
                        "Trailing icon",
                        "Leading icon",
                        "Both icons",
                        "Avatar",
                        "Avatar & trailing",
                    ])
                    .row_labels(state_labels(selected))
                    .cells(STATES.into_iter().enumerate().flat_map(
                        move |(state_index, (state, dragged, _))| {
                            (0..6).map(move |column| {
                                let id = offset + state_index * 6 + column;
                                let mut chip = InputChip::new(("input-state", id), LABEL)
                                    .selected(selected)
                                    .dragged(dragged)
                                    .preview_state(state);
                                if matches!(column, 2 | 3) {
                                    chip = chip.icon(spec::star_icon());
                                }
                                if matches!(column, 4 | 5) {
                                    chip = chip.avatar(avatar("A"));
                                }
                                if matches!(column, 1 | 3 | 5) {
                                    chip = chip.on_remove("Remove", |_, _| {});
                                }
                                chip
                            })
                        },
                    ))
                    .into_any_element()
            }
            ASSIST => board("assist-chips")
                .caption("Assist chips")
                .columns(4)
                .column_width(128.)
                .column_labels(["Label", "Elevated", "Icon", "Icon · elevated"])
                .row_labels(STATES.map(|(_, _, name)| name))
                .cells(STATES.into_iter().enumerate().flat_map(
                    |(state_index, (state, dragged, _))| {
                        (0..4).map(move |column| {
                            let chip =
                                AssistChip::new(("assist-state", state_index * 4 + column), LABEL)
                                    .elevated(column % 2 == 1)
                                    .dragged(dragged)
                                    .preview_state(state);
                            if column >= 2 {
                                chip.icon(spec::star_icon())
                            } else {
                                chip
                            }
                        })
                    },
                ))
                .into_any_element(),
            r if (FILTER..FILTER + 4).contains(&r) => {
                let elevated = r >= FILTER + 2;
                let selected = (r - FILTER) % 2 == 1;
                let offset = (r - FILTER) * 100;
                board(("filter-chips", r))
                    .caption(if elevated {
                        "Filter chips · elevated"
                    } else {
                        "Filter chips"
                    })
                    .band(Band::of((r - FILTER) % 2, 2))
                    .columns(4)
                    .column_width(128.)
                    .column_labels(["Label", "Trailing icon", "Leading icon", "Both icons"])
                    .row_labels(state_labels(selected))
                    .cells(STATES.into_iter().enumerate().flat_map(
                        move |(state_index, (state, dragged, _))| {
                            (0..4).map(move |column| {
                                let id = offset + state_index * 4 + column;
                                let mut chip = FilterChip::new(("filter-state", id), LABEL)
                                    .selected(selected)
                                    .elevated(elevated)
                                    .dragged(dragged)
                                    .preview_state(state);
                                if column >= 2 {
                                    chip = chip.icon(spec::star_icon());
                                }
                                if column % 2 == 1 {
                                    chip = chip.trailing_icon(IconName::ChevronDown);
                                }
                                chip
                            })
                        },
                    ))
                    .into_any_element()
            }
            r if (SUGGESTION..SUGGESTION + 2).contains(&r) => {
                let selected = r == SUGGESTION + 1;
                let offset = if selected { 100 } else { 0 };
                board(("suggestion-chips", r))
                    .caption("Suggestion chips")
                    .band(Band::of(r - SUGGESTION, 2))
                    .columns(4)
                    .column_width(128.)
                    .column_labels(["Outlined", "Elevated", "Icon · outlined", "Icon · elevated"])
                    .row_labels(state_labels(selected))
                    .cells(STATES.into_iter().enumerate().flat_map(
                        move |(state_index, (state, dragged, _))| {
                            (0..4).map(move |column| {
                                let id = offset + state_index * 4 + column;
                                let chip = SuggestionChip::new(("suggestion-state", id), LABEL)
                                    .selected(selected)
                                    .elevated(column % 2 == 1)
                                    .dragged(dragged)
                                    .preview_state(state);
                                if column >= 2 {
                                    chip.icon(spec::star_icon())
                                } else {
                                    chip
                                }
                            })
                        },
                    ))
                    .into_any_element()
            }
            _ => spec::demo(
                "status-chips",
                "Status chips",
                spec::demo_row()
                    .child(status_chip(StatusKind::Success).child("Ready"))
                    .child(status_chip(StatusKind::Warning).child("Needs attention"))
                    .child(status_chip(StatusKind::Danger).child("Failed"))
                    .child(status_chip(StatusKind::Progress).child("Working"))
                    .child(status_chip(StatusKind::Neutral).child("Queued"))
                    .child(status_chip(StatusKind::Tonal).child("Default"))
                    .child(outlined_status_chip("outlined", "Outlined")),
            )
            .into_any_element(),
        }
    }
}
