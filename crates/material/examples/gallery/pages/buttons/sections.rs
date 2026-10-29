//! The Buttons page's further kit sections: icon button, split button, FAB
//! menu, FAB and button groups. Each function renders one body row (a board
//! band or a demo); the page owns rows, state and listeners.
use super::super::prelude::*;
use gpui_kit_material::icons::{Icon, Symbol};
use std::sync::OnceLock;

/// The kit's unselected toggle icon: the outlined `stars` symbol.
fn star_outline() -> Icon {
    static PATH: OnceLock<String> = OnceLock::new();
    Icon::new(SharedString::new_static(
        PATH.get_or_init(|| {
            Symbol::named("stars")
                .expect("stars belongs to the Material Symbols catalog")
                .icon()
                .asset_path()
        })
        .as_str(),
    ))
}

/// Kit captions for the interaction-state columns.
pub(super) const STATES: [InteractionPreview; 5] = InteractionPreview::ALL;
/// FAB specimens have no disabled state.
pub(super) const FAB_STATES: [InteractionPreview; 4] = [
    InteractionPreview::Enabled,
    InteractionPreview::Hovered,
    InteractionPreview::Focused,
    InteractionPreview::Pressed,
];
const FAB_STATE_NAMES: [&str; 4] = ["Enabled", "Hovered", "Focused", "Pressed"];

pub(super) const ICON_VARIANTS: [(IconButtonVariant, &str); 4] = [
    (IconButtonVariant::Standard, "Standard"),
    (IconButtonVariant::Filled, "Filled"),
    (IconButtonVariant::Tonal, "Tonal"),
    (IconButtonVariant::Outlined, "Outlined"),
];
const ICON_TOGGLE_CAPTIONS: [&str; 4] = [
    "Standard - toggleable",
    "Filled - toggleable",
    "Tonal - toggleable",
    "Outlined - toggleable",
];
/// Side labels by shape, width and (for toggles) selection.
const ICON_SIDE: [[&str; 3]; 2] = [
    ["Round\nNarrow", "Round\nDefault", "Round\nWide"],
    ["Square\nNarrow", "Square\nDefault", "Square\nWide"],
];
const ICON_TOGGLE_SIDE: [[[&str; 2]; 3]; 2] = [
    [
        ["Round\nNarrow\nUnselected", "Round\nNarrow\nSelected"],
        ["Round\nDefault\nUnselected", "Round\nDefault\nSelected"],
        ["Round\nWide\nUnselected", "Round\nWide\nSelected"],
    ],
    [
        ["Square\nNarrow\nUnselected", "Square\nNarrow\nSelected"],
        ["Square\nDefault\nUnselected", "Square\nDefault\nSelected"],
        ["Square\nWide\nUnselected", "Square\nWide\nSelected"],
    ],
];
const SIZE_NAMES: [&str; 5] = ["XSmall", "Small", "Medium", "Large", "XLarge"];

/// The icon-button bands of one board: shapes × widths (× selection).
pub(super) const ICON_BANDS: usize = 6;
pub(super) const ICON_TOGGLE_BANDS: usize = 12;

/// One band of an icon-button board: every size × state for one shape and
/// width, optionally as a toggle in one selection.
pub(super) fn icon_matrix(variant: usize, band: usize, toggle: Option<bool>) -> AnyElement {
    let (kind, caption) = ICON_VARIANTS[variant];
    let (shape, width, bands) = match toggle {
        None => (band / 3, band % 3, ICON_BANDS),
        Some(_) => (band / 6, band / 2 % 3, ICON_TOGGLE_BANDS),
    };
    let side = match toggle {
        None => ICON_SIDE[shape][width],
        Some(selected) => ICON_TOGGLE_SIDE[shape][width][usize::from(selected)],
    };
    let base =
        (variant * ICON_TOGGLE_BANDS + band) * 25 + if toggle.is_some() { 10_000 } else { 0 };
    board(("icon-board", base))
        .band(Band::of(band, bands))
        .caption(match toggle {
            None => caption,
            Some(_) => ICON_TOGGLE_CAPTIONS[variant],
        })
        .side_label(side)
        .columns(STATES.len())
        .when(band == 0, |board| {
            board.column_labels(STATES.map(InteractionPreview::name))
        })
        .cells(
            ButtonSize::ALL
                .into_iter()
                .enumerate()
                .flat_map(move |(size_index, size)| {
                    STATES
                        .into_iter()
                        .enumerate()
                        .map(move |(state_index, state)| {
                            IconButton::new(
                                ("icon-specimen", base + size_index * 5 + state_index),
                                if toggle.is_some() {
                                    star_outline()
                                } else {
                                    spec::star_icon()
                                },
                                "Star",
                            )
                            .selected_icon(spec::star_icon())
                            .variant(kind)
                            .button_size(size)
                            .width(IconButtonWidth::ALL[width])
                            .shape(ButtonShape::ALL[shape])
                            .toggle(toggle.is_some())
                            .selected(toggle == Some(true))
                            .preview_state(state)
                        })
                }),
        )
        .into_any_element()
}

pub(super) const SPLIT_VARIANTS: [(ButtonVariant, &str); 4] = [
    (ButtonVariant::Filled, "Filled"),
    (ButtonVariant::Tonal, "Tonal"),
    (ButtonVariant::Elevated, "Elevated"),
    (ButtonVariant::Outlined, "Outlined"),
];
const SPLIT_COLUMNS: [&str; 6] = [
    "Enabled",
    "Hovered",
    "Focused",
    "Pressed",
    "Menu open",
    "Disabled",
];

/// One band of a split-button board: one size and variant. The first cell
/// row previews each state on the trailing segment, the second on the
/// leading one.
pub(super) fn split_matrix(size: usize, variant: usize) -> AnyElement {
    let (kind, name) = SPLIT_VARIANTS[variant];
    let button_size = ButtonSize::ALL[size];
    let base = (size * 4 + variant) * 12;
    let specimen = move |index: usize| {
        SplitButton::new(("split-specimen", base + index), "Label", "More options")
            .icon(spec::star_icon())
            .variant(kind)
            .button_size(button_size)
    };
    let interactive = [
        InteractionPreview::Hovered,
        InteractionPreview::Focused,
        InteractionPreview::Pressed,
    ];
    let mut cells: Vec<AnyElement> = Vec::with_capacity(12);
    for (row, part) in [SplitButtonPart::Trailing, SplitButtonPart::Leading]
        .into_iter()
        .enumerate()
    {
        let index = |column: usize| row * 6 + column;
        cells.push(if row == 0 {
            specimen(index(0))
                .preview_state(part, InteractionPreview::Enabled)
                .into_any_element()
        } else {
            div().into_any_element()
        });
        for (offset, state) in interactive.into_iter().enumerate() {
            cells.push(
                specimen(index(offset + 1))
                    .preview_state(part, state)
                    .into_any_element(),
            );
        }
        cells.push(if row == 0 {
            specimen(index(4))
                .open(true)
                .preview_state(part, InteractionPreview::Enabled)
                .into_any_element()
        } else {
            div().into_any_element()
        });
        cells.push(if row == 0 {
            specimen(index(5))
                .preview_state(part, InteractionPreview::Disabled)
                .into_any_element()
        } else {
            div().into_any_element()
        });
    }
    board(("split-board", size * 4 + variant))
        .band(Band::of(variant, SPLIT_VARIANTS.len()))
        .caption(SIZE_NAMES[size])
        .side_label(name)
        .columns(SPLIT_COLUMNS.len())
        .row_labels(["Trailing", "Leading"])
        .when(variant == 0, |board| board.column_labels(SPLIT_COLUMNS))
        .cells(cells)
        .into_any_element()
}

const MENU_LABELS: [&str; 6] = ["First", "Second", "Third", "Fourth", "Fifth", "Sixth"];

fn specimen_menu(id: usize, color: FabMenuColor, items: usize) -> FabMenu {
    FabMenu::new(("fab-menu-specimen", id), IconName::Close, "Close menu")
        .color(color)
        .open(true)
        .preview_state(InteractionPreview::Enabled)
        .items((0..items).map(move |index| {
            FabMenuItem::new(
                ("fab-menu-specimen-item", id * 8 + index),
                spec::star_icon(),
                MENU_LABELS[index],
            )
            .preview_state(InteractionPreview::Enabled)
        }))
}

/// The open FAB menu in each color style.
pub(super) fn fab_menu_colors() -> AnyElement {
    board("fab-menu-colors")
        .caption("FAB Menu")
        .columns(3)
        .column_labels([
            "Primary Container",
            "Secondary Container",
            "Tertiary Container",
        ])
        .cells(
            FabMenuColor::ALL
                .into_iter()
                .enumerate()
                .map(|(index, color)| specimen_menu(index, color, 3)),
        )
        .into_any_element()
}

/// Open menus with two to six items.
pub(super) fn fab_menu_counts() -> AnyElement {
    board("fab-menu-counts")
        .columns(5)
        .column_labels(["2 items", "3 items", "4 items", "5 items", "6 items"])
        .cells((2..=6).map(|items| {
            div().h_full().flex().items_end().child(specimen_menu(
                10 + items,
                FabMenuColor::Primary,
                items,
            ))
        }))
        .into_any_element()
}

/// Building blocks: the close button and an item in each state and color.
pub(super) fn fab_menu_blocks() -> AnyElement {
    board("fab-menu-blocks")
        .caption("Building Blocks")
        .columns(8)
        .column_labels(FAB_STATE_NAMES.into_iter().chain(FAB_STATE_NAMES))
        .row_labels(["Primary", "Secondary", "Tertiary"])
        .cells(
            FabMenuColor::ALL
                .into_iter()
                .enumerate()
                .flat_map(|(row, color)| {
                    let buttons = FAB_STATES
                        .into_iter()
                        .enumerate()
                        .map(move |(column, state)| {
                            FabMenu::new(
                                ("fab-menu-block", row * 4 + column),
                                IconName::Close,
                                "Close menu",
                            )
                            .color(color)
                            .open(true)
                            .preview_state(state)
                            .into_any_element()
                        });
                    let items = FAB_STATES
                        .into_iter()
                        .enumerate()
                        .map(move |(column, state)| {
                            FabMenuItem::new(
                                ("fab-menu-item-specimen", row * 4 + column),
                                spec::star_icon(),
                                "Label",
                            )
                            .color(color)
                            .preview_state(state)
                            .into_any_element()
                        });
                    buttons.chain(items).collect::<Vec<_>>()
                }),
        )
        .into_any_element()
}

/// The kit's FAB color order (the surface style follows, from baseline).
pub(super) const FAB_COLORS: [(FabColor, &str); 7] = [
    (FabColor::PrimaryContainer, "Primary Container"),
    (FabColor::SecondaryContainer, "Secondary Container"),
    (FabColor::TertiaryContainer, "Tertiary Container"),
    (FabColor::Primary, "Primary"),
    (FabColor::Secondary, "Secondary"),
    (FabColor::Tertiary, "Tertiary"),
    (FabColor::Surface, "Surface"),
];
pub(super) const FAB_SIZES: [(FabSize, &str); 4] = [
    (FabSize::Regular, "FABs"),
    (FabSize::Medium, "Medium FABs"),
    (FabSize::Large, "Large FABs"),
    (FabSize::Small, "Small FABs"),
];
pub(super) const EXTENDED_SIZES: [(FabSize, &str); 3] = [
    (FabSize::Regular, "Extended FAB"),
    (FabSize::Medium, "Medium"),
    (FabSize::Large, "Large"),
];

/// One FAB board band: one size, every color × state. `extended` boards
/// show the extended FAB.
pub(super) fn fab_matrix(size: usize, extended: bool) -> AnyElement {
    let (fab_size, caption, bands) = if extended {
        let (size, caption) = EXTENDED_SIZES[size];
        (size, caption, EXTENDED_SIZES.len())
    } else {
        let (size, caption) = FAB_SIZES[size];
        (size, caption, FAB_SIZES.len())
    };
    let base = size * 64 + if extended { 1000 } else { 0 };
    board(("fab-board", base))
        .band(Band::of(size, bands))
        .caption(if extended { "Extended FAB" } else { "FAB" })
        .side_label(caption)
        .columns(FAB_STATES.len())
        .row_labels(FAB_COLORS.map(|(_, name)| name))
        .when(size == 0, |board| board.column_labels(FAB_STATE_NAMES))
        .cells(
            FAB_COLORS
                .into_iter()
                .enumerate()
                .flat_map(move |(row, (color, _))| {
                    FAB_STATES
                        .into_iter()
                        .enumerate()
                        .map(move |(column, state)| {
                            let id = base + row * 4 + column;
                            if extended {
                                ExtendedFab::new(("fab-specimen", id), spec::star_icon(), "Label")
                                    .size(fab_size)
                                    .color(color)
                                    .preview_state(state)
                                    .into_any_element()
                            } else {
                                Fab::new(("fab-specimen", id), spec::star_icon(), "Star")
                                    .size(fab_size)
                                    .color(color)
                                    .preview_state(state)
                                    .into_any_element()
                            }
                        })
                }),
        )
        .into_any_element()
}

/// The kit's standard groups are unselected toggle buttons.
pub(super) const GROUP_VARIANTS: [(ButtonVariant, IconButtonVariant, &str); 3] = [
    (ButtonVariant::Filled, IconButtonVariant::Filled, "Filled"),
    (ButtonVariant::Tonal, IconButtonVariant::Tonal, "Tonal"),
    (
        ButtonVariant::Outlined,
        IconButtonVariant::Outlined,
        "Outlined",
    ),
];
const GROUP_SIDE: [[&str; 3]; 2] = [
    ["Round\nFilled", "Round\nTonal", "Round\nOutlined"],
    ["Square\nFilled", "Square\nTonal", "Square\nOutlined"],
];

/// One band of the standard button group board: one shape and variant,
/// each size as an icon-button group and a labelled group. These are live
/// groups: pressing one shows the expressive width interaction.
pub(super) fn standard_groups(shape: usize, variant: usize) -> AnyElement {
    let (button, icon, _) = GROUP_VARIANTS[variant];
    let shape_value = ButtonShape::ALL[shape];
    let band = shape * GROUP_VARIANTS.len() + variant;
    board(("standard-group-board", band))
        .band(Band::of(band, 2 * GROUP_VARIANTS.len()))
        .caption("Standard button group")
        .side_label(GROUP_SIDE[shape][variant])
        .columns(2)
        .row_labels(SIZE_NAMES)
        .when(band == 0, |board| {
            board.column_labels(["Icon buttons", "Buttons"])
        })
        .cells(
            ButtonSize::ALL
                .into_iter()
                .enumerate()
                .flat_map(move |(size_index, size)| {
                    let base = (band * 5 + size_index) * 8;
                    let icons =
                        ButtonGroup::new(("standard-icon-group", base), "Icon button group")
                            .button_size(size)
                            .items((0..4).map(move |index| {
                                IconButton::new(
                                    ("standard-icon-item", base + index),
                                    star_outline(),
                                    "Star",
                                )
                                .variant(icon)
                                .toggle(true)
                                .button_size(size)
                                .shape(shape_value)
                            }));
                    let labels = ButtonGroup::new(("standard-label-group", base), "Button group")
                        .button_size(size)
                        .items((0..4).map(move |index| {
                            MaterialButton::new(("standard-label-item", base + index))
                                .variant(button)
                                .toggle(true)
                                .button_size(size)
                                .shape(shape_value)
                                .icon(spec::star_icon())
                                .child("Label")
                        }));
                    [icons.into_any_element(), labels.into_any_element()]
                }),
        )
        .into_any_element()
}

const CONNECTED_SIDE: [&str; 2] = ["Round", "Square"];

/// The connected group in every size for one shape.
pub(super) fn connected_sizes(shape: usize) -> AnyElement {
    let shape_value = ButtonShape::ALL[shape];
    board(("connected-size-board", shape))
        .band(Band::of(shape, 2))
        .caption("Connected button group")
        .side_label(CONNECTED_SIDE[shape])
        .columns(1)
        .row_labels(SIZE_NAMES)
        .cells(
            ButtonSize::ALL
                .into_iter()
                .enumerate()
                .map(move |(index, size)| {
                    ConnectedButtonGroup::new(
                        ("connected-size", shape * 5 + index),
                        "Connected group",
                    )
                    .options([("first", "Label"), ("second", "Label"), ("third", "Label")])
                    .icon("first", spec::star_icon())
                    .icon("second", spec::star_icon())
                    .icon("third", spec::star_icon())
                    .button_size(size)
                    .shape(shape_value)
                    .selected("first")
                    .preview_state("first", InteractionPreview::Enabled)
                }),
        )
        .into_any_element()
}

/// Connected groups of two to five buttons.
pub(super) fn connected_counts() -> AnyElement {
    const VALUES: [(&str, &str); 5] = [
        ("1", "Label"),
        ("2", "Label"),
        ("3", "Label"),
        ("4", "Label"),
        ("5", "Label"),
    ];
    board("connected-count-board")
        .caption("Connected button group")
        .columns(4)
        .column_labels(["2 buttons", "3 buttons", "4 buttons", "5 buttons"])
        .cells((2..=5).map(|count| {
            let mut group =
                ConnectedButtonGroup::new(("connected-count", count), "Connected group")
                    .options(VALUES.into_iter().take(count))
                    .button_size(ButtonSize::ExtraSmall)
                    .selected("1")
                    .preview_state("1", InteractionPreview::Enabled);
            for (value, _) in VALUES.into_iter().take(count) {
                group = group.icon(value, spec::star_icon());
            }
            group
        }))
        .into_any_element()
}
