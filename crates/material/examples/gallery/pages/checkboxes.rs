//! Checkboxes: selected, indeterminate and unselected, with and without
//! error, in every interaction state, and a live tri-state group.
use super::prelude::*;

pub struct Page {
    page: PageCx,
    /// The live group's children.
    items: [bool; 3],
    /// The required checkbox, in error until checked.
    terms: bool,
}

/// The board's rows, in the kit's order: (state, error, row label).
const TYPES: [(CheckboxState, bool, &str); 6] = [
    (CheckboxState::Checked, false, "Selected"),
    (CheckboxState::Indeterminate, false, "Indeterminate"),
    (CheckboxState::Unchecked, false, "Unselected"),
    (CheckboxState::Checked, true, "Error selected"),
    (CheckboxState::Indeterminate, true, "Error indeterminate"),
    (CheckboxState::Unchecked, true, "Error unselected"),
];

const ITEMS: [&str; 3] = ["Photos", "Documents", "Music"];

/// The parent of a group: checked when all children are, unchecked when
/// none are, indeterminate otherwise.
fn parent_state(items: &[bool]) -> CheckboxState {
    match items.iter().filter(|&&item| item).count() {
        0 => CheckboxState::Unchecked,
        n if n == items.len() => CheckboxState::Checked,
        _ => CheckboxState::Indeterminate,
    }
}

fn labelled(id: &'static str, label: &'static str, checkbox: impl IntoElement) -> Div {
    div()
        .flex()
        .items_center()
        .gap(rem(4.))
        .child(checkbox)
        .child(accessible_text(id, label).text_color(color(ColorRole::ON_SURFACE)))
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "checkboxes",
        title: "Checkboxes",
        description: "Checkboxes allow users to select one or more items from a set and can be used to turn an option on or off. They’re a kind of selection control that helps users make a choice from a set of options.",
        guideline: Some("https://m3.material.io/components/checkbox/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            page,
            items: [true, false, false],
            terms: false,
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
            return board("checkbox-states")
                .caption("Checkboxes")
                .columns(5)
                .column_width(72.)
                .column_labels(InteractionPreview::ALL.map(InteractionPreview::name))
                .row_labels(TYPES.map(|(_, _, label)| label))
                .cells(
                    TYPES
                        .into_iter()
                        .enumerate()
                        .flat_map(|(kind, (state, error, _))| {
                            InteractionPreview::ALL.into_iter().enumerate().map(
                                move |(index, preview)| {
                                    MaterialCheckbox::new(
                                        ("checkbox-state", kind * 5 + index),
                                        preview.name(),
                                    )
                                    .state(state)
                                    .error(error)
                                    .preview_state(preview)
                                },
                            )
                        }),
                )
                .into_any_element();
        }
        let list = self.page.rows.list().clone();
        let parent = MaterialCheckbox::new("all-files", "All files")
            .state(parent_state(&self.items))
            .on_change(cx.listener(|this, next: &CheckboxState, _, cx| {
                this.items = [*next == CheckboxState::Checked; 3];
                cx.notify();
            }))
            .reveal("reveal-all-files", &list);
        let mut column =
            div()
                .flex()
                .flex_col()
                .child(labelled("all-files-label", "All files", parent));
        for (index, label) in ITEMS.into_iter().enumerate() {
            let checkbox = MaterialCheckbox::new(("file-kind", index), label)
                .checked(self.items[index])
                .on_change(cx.listener(move |this, next: &CheckboxState, _, cx| {
                    this.items[index] = *next == CheckboxState::Checked;
                    cx.notify();
                }))
                .reveal(("reveal-file-kind", index), &list);
            column = column.child(
                labelled(
                    ["photos-label", "documents-label", "music-label"][index],
                    label,
                    checkbox,
                )
                .pl(rem(32.)),
            );
        }
        let required = MaterialCheckbox::new("terms", "Accept the terms")
            .checked(self.terms)
            .error(!self.terms)
            .on_change(cx.listener(|this, next: &CheckboxState, _, cx| {
                this.terms = *next == CheckboxState::Checked;
                cx.notify();
            }))
            .reveal("reveal-terms", &list);
        spec::demo(
            "live-checkboxes",
            "Checkboxes in use",
            spec::demo_row()
                .items_start()
                .gap(rem(48.))
                .child(column)
                .child(labelled("terms-label", "Accept the terms", required))
                .child(labelled(
                    "disabled-label",
                    "Unavailable",
                    MaterialCheckbox::new("disabled-checkbox", "Unavailable")
                        .checked(true)
                        .disabled(true),
                )),
        )
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_parent_reflects_its_children() {
        assert_eq!(parent_state(&[false; 3]), CheckboxState::Unchecked);
        assert_eq!(parent_state(&[true; 3]), CheckboxState::Checked);
        assert_eq!(
            parent_state(&[true, false, false]),
            CheckboxState::Indeterminate
        );
    }
}
