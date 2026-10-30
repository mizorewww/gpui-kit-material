//! Text fields: the design kit's eight boards (filled and outlined, with and
//! without leading and trailing icons) in every state and text
//! configuration, then live fields with prefix/suffix, counters, errors and
//! multiple lines.
use super::prelude::*;
use gpui_base::input::{InputState, TextareaState};
use gpui_kit_material::icons::{Icon, Symbol};

/// A Material Symbol at the 24dp icon size of text fields.
pub(crate) fn symbol(name: &str, filled: bool) -> Icon {
    Icon::from(
        Symbol::named(name)
            .expect("a Material Symbols catalog name")
            .icon()
            .filled(filled),
    )
    .size(rem(24.))
}

/// The kit's state columns, in its order.
const STATES: [(InteractionPreview, bool); 5] = [
    (InteractionPreview::Enabled, false),
    (InteractionPreview::Focused, false),
    (InteractionPreview::Hovered, false),
    (InteractionPreview::Enabled, true),
    (InteractionPreview::Disabled, false),
];
const STATE_NAMES: [&str; 5] = ["Enabled", "Focused", "Hovered", "Error", "Disabled"];
/// The kit's text configurations: (row label, value, float label).
const CONFIGS: [(&str, &str, bool); 3] = [
    ("Input text", "Input", true),
    ("Label text", "", false),
    ("Placeholder text", "", true),
];
/// The kit's boards: (variant, leading icon, trailing icon).
const BOARDS: [(TextFieldVariant, bool, bool); 8] = [
    (TextFieldVariant::Filled, false, true),
    (TextFieldVariant::Filled, true, true),
    (TextFieldVariant::Filled, false, false),
    (TextFieldVariant::Filled, true, false),
    (TextFieldVariant::Outlined, false, true),
    (TextFieldVariant::Outlined, true, true),
    (TextFieldVariant::Outlined, false, false),
    (TextFieldVariant::Outlined, true, false),
];

pub struct Page {
    name: Entity<InputState>,
    amount: Entity<InputState>,
    email: Entity<InputState>,
    outlined_name: Entity<InputState>,
    outlined_amount: Entity<InputState>,
    outlined_email: Entity<InputState>,
    bio: Entity<TextareaState>,
    outlined_bio: Entity<TextareaState>,
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "text-fields",
        title: "Text fields",
        description: "Text fields let users enter text into a UI.",
        guideline: Some("https://m3.material.io/components/text-fields/overview"),
    };

    fn new(_: PageCx, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut input = |value: &str| {
            let value = SharedString::from(value.to_owned());
            cx.new(|cx| InputState::new(window, cx).default_value(value))
        };
        let name = input("");
        let amount = input("");
        let email = input("not-an-email");
        let outlined_name = input("");
        let outlined_amount = input("");
        let outlined_email = input("not-an-email");
        let bio = cx.new(|cx| TextareaState::new(window, cx).auto_grow(1, 5));
        let outlined_bio = cx.new(|cx| TextareaState::new(window, cx).auto_grow(1, 5));
        // Errors and counters follow the text; observing keeps them current.
        for state in [
            &name,
            &amount,
            &email,
            &outlined_name,
            &outlined_amount,
            &outlined_email,
        ] {
            cx.observe(state, |_, _, cx| cx.notify()).detach();
        }
        for state in [&bio, &outlined_bio] {
            cx.observe(state, |_, _, cx| cx.notify()).detach();
        }
        Self {
            name,
            amount,
            email,
            outlined_name,
            outlined_amount,
            outlined_email,
            bio,
            outlined_bio,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        BOARDS.len() + 3
    }

    fn interactive(&self, row: usize) -> bool {
        row >= BOARDS.len()
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        if let Some(&(variant, leading, trailing)) = BOARDS.get(row) {
            return specimen_board(row, variant, leading, trailing);
        }
        match row - BOARDS.len() {
            0 => self.live_row("live-filled", TextFieldVariant::Filled, cx),
            1 => self.live_row("live-outlined", TextFieldVariant::Outlined, cx),
            _ => spec::demo(
                "live-multi-line",
                "Multi-line fields with character counters",
                spec::demo_row()
                    .items_start()
                    .child(
                        TextField::multi_line("bio", &self.bio)
                            .label("Biography")
                            .supporting_text("Grows up to five lines")
                            .max_length(120)
                            .error(self.bio.read(cx).value().chars().count() > 120)
                            .w(rem(320.)),
                    )
                    .child(
                        TextField::multi_line("outlined-bio", &self.outlined_bio)
                            .outlined()
                            .label("Biography")
                            .supporting_text("Grows up to five lines")
                            .max_length(120)
                            .error(self.outlined_bio.read(cx).value().chars().count() > 120)
                            .w(rem(320.)),
                    ),
            )
            .into_any_element(),
        }
    }
}

impl Page {
    fn live_row(&self, id: &'static str, variant: TextFieldVariant, cx: &App) -> AnyElement {
        let outlined = variant == TextFieldVariant::Outlined;
        let (name, amount, email) = if outlined {
            (
                &self.outlined_name,
                &self.outlined_amount,
                &self.outlined_email,
            )
        } else {
            (&self.name, &self.amount, &self.email)
        };
        let invalid = {
            let value = email.read(cx).value();
            !value.is_empty() && !value.contains('@')
        };
        spec::demo(
            id,
            if outlined {
                "Outlined text fields in use"
            } else {
                "Filled text fields in use"
            },
            spec::demo_row()
                .items_start()
                .child(
                    TextField::new((id, 0usize), name)
                        .variant(variant)
                        .label("Name")
                        .placeholder("First and last")
                        .supporting_text("As it appears on your ID")
                        .leading_icon(symbol("person", false))
                        .w(rem(260.)),
                )
                .child(
                    TextField::new((id, 1usize), amount)
                        .variant(variant)
                        .label("Amount")
                        .prefix("$")
                        .suffix("USD")
                        .supporting_text("Prefix and suffix")
                        .w(rem(220.)),
                )
                .child(
                    TextField::new((id, 2usize), email)
                        .variant(variant)
                        .label("Email")
                        .trailing_icon(if invalid {
                            symbol("error", true)
                        } else {
                            symbol("mail", false)
                        })
                        .error(invalid)
                        .supporting_text(if invalid {
                            "Enter an address with @"
                        } else {
                            "We never share it"
                        })
                        .max_length(40)
                        .w(rem(260.)),
                ),
        )
        .into_any_element()
    }
}

fn specimen_board(
    index: usize,
    variant: TextFieldVariant,
    leading: bool,
    trailing: bool,
) -> AnyElement {
    let caption = format!(
        "{} · {} leading icon · {} trailing icon",
        variant.name(),
        if leading { "with" } else { "no" },
        if trailing { "with" } else { "no" },
    );
    let cells = CONFIGS.iter().enumerate().flat_map(|(config_ix, config)| {
        STATES
            .iter()
            .enumerate()
            .map(move |(state_ix, &(state, error))| {
                let (_, value, float) = *config;
                let mut field = TextField::specimen(
                    (
                        "text-field-specimen",
                        index * 100 + config_ix * 10 + state_ix,
                    ),
                    value,
                )
                .variant(variant)
                .label("Label")
                .float_label(float)
                .supporting_text("Supporting text")
                .error(error)
                .preview_state(state);
                if config_ix == 2 {
                    field = field.placeholder("Placeholder");
                }
                if leading {
                    field = field.leading_icon(symbol("search", false));
                }
                if error {
                    field = field.trailing_icon(symbol("error", true));
                } else if trailing {
                    field = field.trailing_icon(symbol("cancel", false));
                }
                field
            })
    });
    board(("text-field-board", index))
        .caption(caption)
        .columns(5)
        .column_width(210.)
        .column_labels(STATE_NAMES)
        .row_labels(CONFIGS.map(|(label, ..)| label))
        .cells(cells)
        .into_any_element()
}
