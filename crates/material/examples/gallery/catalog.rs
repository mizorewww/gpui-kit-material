//! The Gallery's navigation catalog, organized like md3.io: top-level
//! sections (Components, Styles, Examples), component groups in md3.io's
//! order, and for each entry its playground and the design-kit page (or
//! slice of one) its Specs tab shows.
//!
//! # Adding a component playground
//!
//! Write `playgrounds/<component>.rs` exporting a
//! [`Playground`](super::playground::Playground) static and point the
//! entry's `playground` at it (the one registry line). See
//! `super::playground` for the API.
use super::pages::{self, PAGES};
use super::playground::Playground;
use super::playgrounds;
use std::sync::OnceLock;

/// The top app bar's sections.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopSection {
    Components,
    Styles,
    Examples,
}

impl TopSection {
    pub const ALL: [Self; 3] = [Self::Components, Self::Styles, Self::Examples];
    pub const fn title(self) -> &'static str {
        match self {
            Self::Components => "Components",
            Self::Styles => "Styles",
            Self::Examples => "Examples",
        }
    }
    pub const fn id(self) -> &'static str {
        match self {
            Self::Components => "components",
            Self::Styles => "styles",
            Self::Examples => "examples-section",
        }
    }
}

/// A navigation group heading, in md3.io's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    /// md3.io's "Components" group: the overview.
    Library,
    Actions,
    SelectionInput,
    Navigation,
    Containment,
    Communication,
    Styles,
    Examples,
}

impl Group {
    pub const fn title(self) -> &'static str {
        match self {
            Self::Library => "Components",
            Self::Actions => "Actions",
            Self::SelectionInput => "Selection & input",
            Self::Navigation => "Navigation",
            Self::Containment => "Containment",
            Self::Communication => "Communication",
            Self::Styles => "Styles",
            Self::Examples => "Examples",
        }
    }
    pub const fn section(self) -> TopSection {
        match self {
            Self::Styles => TopSection::Styles,
            Self::Examples => TopSection::Examples,
            _ => TopSection::Components,
        }
    }
}

/// What an entry's page shows.
#[derive(Clone, Copy)]
pub enum Kind {
    /// The component overview: a card per component.
    Overview,
    /// A component page: a Live preview | Specs card and a configuration
    /// card. Without a playground, the preview shows the spec page's live
    /// demos.
    Component {
        playground: Option<&'static Playground>,
    },
    /// A foundation or examples page: the spec page's rows as the body.
    Document,
}

/// The design-kit page an entry's Specs tab shows, and optionally which of
/// its rows (for kit pages that hold several components).
#[derive(Clone, Copy)]
pub struct SpecRef {
    /// A `pages::PAGES` id.
    pub page: &'static str,
    /// The page's rows for this entry; `None` shows every row.
    pub rows: Option<fn() -> Vec<usize>>,
}

const fn spec(page: &'static str) -> Option<SpecRef> {
    Some(SpecRef { page, rows: None })
}
const fn slice(page: &'static str, rows: fn() -> Vec<usize>) -> Option<SpecRef> {
    Some(SpecRef {
        page,
        rows: Some(rows),
    })
}

/// One navigation entry and page.
#[derive(Clone, Copy)]
pub struct Entry {
    /// Stable id: navigation, `--section <id>` captures and element ids.
    pub id: &'static str,
    pub title: &'static str,
    pub group: Group,
    /// The page's one-line description.
    pub description: &'static str,
    pub kind: Kind,
    pub spec: Option<SpecRef>,
}

const fn component(
    id: &'static str,
    title: &'static str,
    group: Group,
    description: &'static str,
    spec: Option<SpecRef>,
    playground: Option<&'static Playground>,
) -> Entry {
    Entry {
        id,
        title,
        group,
        description,
        kind: Kind::Component { playground },
        spec,
    }
}

const fn document(
    id: &'static str,
    title: &'static str,
    group: Group,
    description: &'static str,
) -> Entry {
    Entry {
        id,
        title,
        group,
        description,
        kind: Kind::Document,
        spec: spec(id),
    }
}

use Group::*;

/// Every page, in navigation order.
pub static CATALOG: &[Entry] = &[
    Entry {
        id: "overview",
        title: "Overview",
        group: Library,
        description: "Try a component. Adjust the details. See every state.",
        kind: Kind::Overview,
        spec: None,
    },
    // Actions
    component(
        "button",
        "Button",
        Actions,
        "One action, many expressions: five variants and five sizes.",
        slice("buttons", button_rows),
        Some(&playgrounds::button::PLAYGROUND),
    ),
    component(
        "icon-button",
        "Icon button",
        Actions,
        "Compact actions, with a shape for every state.",
        slice("buttons", icon_button_rows),
        Some(&playgrounds::icon_button::PLAYGROUND),
    ),
    component(
        "button-group",
        "Button group",
        Actions,
        "Related actions with shared shapes and flexible selection.",
        slice("buttons", button_group_rows),
        Some(&playgrounds::button_group::PLAYGROUND),
    ),
    component(
        "split-button",
        "Split button",
        Actions,
        "One primary action, with more options close by.",
        slice("buttons", split_button_rows),
        Some(&playgrounds::split_button::PLAYGROUND),
    ),
    component(
        "fab",
        "FAB",
        Actions,
        "The most important action on a screen.",
        slice("buttons", fab_rows),
        Some(&playgrounds::fab::PLAYGROUND),
    ),
    component(
        "fab-menu",
        "FAB menu",
        Actions,
        "A FAB that opens into related actions.",
        slice("buttons", fab_menu_rows),
        Some(&playgrounds::fab_menu::PLAYGROUND),
    ),
    component(
        "extended-fab",
        "Extended FAB",
        Actions,
        "A FAB with a label, for a clearer primary action.",
        slice("buttons", extended_fab_rows),
        Some(&playgrounds::extended_fab::PLAYGROUND),
    ),
    // Selection & input
    component(
        "checkbox",
        "Checkbox",
        SelectionInput,
        "Make a choice, or represent a partial selection.",
        spec("checkboxes"),
        Some(&playgrounds::checkbox::PLAYGROUND),
    ),
    component(
        "switch",
        "Switch",
        SelectionInput,
        "Turn a setting on or off. Try labels, supporting text, and states.",
        spec("switch"),
        Some(&playgrounds::switch::PLAYGROUND),
    ),
    component(
        "radio-buttons",
        "Radio buttons",
        SelectionInput,
        "Pick exactly one option from a set.",
        spec("radio-button"),
        Some(&playgrounds::radio::PLAYGROUND),
    ),
    component(
        "chips",
        "Chips",
        SelectionInput,
        "Compact elements for input, filtering, and suggestions.",
        spec("chips"),
        Some(&playgrounds::chips::PLAYGROUND),
    ),
    component(
        "slider",
        "Slider",
        SelectionInput,
        "Choose a value or a range along a track.",
        spec("sliders"),
        Some(&playgrounds::slider::PLAYGROUND),
    ),
    component(
        "text-field",
        "Text field",
        SelectionInput,
        "Enter text with helpful context: styles, icons, and validation.",
        spec("text-fields"),
        Some(&playgrounds::text_field::PLAYGROUND),
    ),
    component(
        "select",
        "Select",
        SelectionInput,
        "Choose one option from a menu that opens from a field.",
        spec("menu"),
        Some(&playgrounds::select::PLAYGROUND),
    ),
    component(
        "search",
        "Search",
        SelectionInput,
        "Find content with a search bar and suggestions.",
        spec("search"),
        Some(&playgrounds::search::PLAYGROUND),
    ),
    component(
        "date-picker",
        "Date picker",
        SelectionInput,
        "Select a date or a range of dates.",
        spec("date-time-pickers"),
        Some(&playgrounds::date_picker::PLAYGROUND),
    ),
    component(
        "time-picker",
        "Time picker",
        SelectionInput,
        "Select a time on a dial or with the keyboard.",
        spec("date-time-pickers"),
        Some(&playgrounds::time_picker::PLAYGROUND),
    ),
    // Navigation
    component(
        "navigation-rail",
        "Navigation rail",
        Navigation,
        "Move between destinations from the side of the window.",
        spec("navigation"),
        Some(&playgrounds::navigation_rail::PLAYGROUND),
    ),
    component(
        "navigation-bar",
        "Navigation bar",
        Navigation,
        "Move between destinations along the bottom of the window.",
        spec("navigation"),
        Some(&playgrounds::navigation_bar::PLAYGROUND),
    ),
    component(
        "drawer",
        "Drawer",
        Navigation,
        "Grouped destinations in a side panel, standard or modal.",
        spec("navigation"),
        Some(&playgrounds::navigation_drawer::PLAYGROUND),
    ),
    component(
        "navigation-suite",
        "Navigation suite",
        Navigation,
        "One set of destinations as a bar, rail or drawer, by window width.",
        spec("navigation"),
        Some(&playgrounds::navigation_suite::PLAYGROUND),
    ),
    component(
        "tabs",
        "Tabs",
        Navigation,
        "Organize related content across views at the same level.",
        spec("tabs"),
        Some(&playgrounds::tabs::PLAYGROUND),
    ),
    component(
        "menu",
        "Menu",
        Navigation,
        "A list of choices on a temporary surface.",
        spec("menu"),
        Some(&playgrounds::menu::PLAYGROUND),
    ),
    component(
        "top-app-bar",
        "Top app bar",
        Navigation,
        "The title and actions at the top of a screen.",
        spec("app-bars"),
        Some(&playgrounds::top_app_bar::PLAYGROUND),
    ),
    component(
        "bottom-app-bar",
        "Bottom app bar",
        Navigation,
        "Actions along the bottom of a screen.",
        spec("app-bars"),
        Some(&playgrounds::bottom_app_bar::PLAYGROUND),
    ),
    component(
        "toolbar",
        "Toolbar",
        Navigation,
        "Frequent actions for the current page, docked or floating.",
        spec("toolbars"),
        Some(&playgrounds::toolbar::PLAYGROUND),
    ),
    // Containment
    component(
        "card",
        "Card",
        Containment,
        "Content and actions about a single subject.",
        spec("cards"),
        Some(&playgrounds::card::PLAYGROUND),
    ),
    component(
        "list",
        "List",
        Containment,
        "Continuous, vertical indexes of text and images.",
        spec("lists"),
        Some(&playgrounds::list::PLAYGROUND),
    ),
    component(
        "carousel",
        "Carousel",
        Containment,
        "A collection of items that scrolls on and off the screen.",
        spec("carousel"),
        Some(&playgrounds::carousel::PLAYGROUND),
    ),
    component(
        "divider",
        "Divider",
        Containment,
        "A thin line that groups content in lists and containers.",
        spec("dividers"),
        Some(&playgrounds::divider::PLAYGROUND),
    ),
    component(
        "dialog",
        "Dialog",
        Containment,
        "Focus on a decision: content, actions, and dismissal.",
        spec("dialogs"),
        Some(&playgrounds::dialog::PLAYGROUND),
    ),
    component(
        "bottom-sheet",
        "Bottom sheet",
        Containment,
        "Secondary content anchored to the bottom of the screen.",
        spec("sheets"),
        Some(&playgrounds::bottom_sheet::PLAYGROUND),
    ),
    component(
        "side-sheet",
        "Side sheet",
        Containment,
        "Secondary content anchored to the side of the screen.",
        spec("sheets"),
        Some(&playgrounds::side_sheet::PLAYGROUND),
    ),
    component(
        "canonical-layouts",
        "Canonical layouts",
        Containment,
        "List-detail, supporting pane and feed layouts that adapt to their width.",
        spec("examples"),
        Some(&playgrounds::canonical_layouts::PLAYGROUND),
    ),
    // Communication
    component(
        "badge",
        "Badge",
        Communication,
        "Notifications and counts on navigation items and icons.",
        spec("badges"),
        Some(&playgrounds::badge::PLAYGROUND),
    ),
    component(
        "avatar",
        "Avatar",
        Communication,
        "A person's picture, initials or placeholder, in a circle.",
        spec("avatars"),
        Some(&playgrounds::avatar::PLAYGROUND),
    ),
    component(
        "progress",
        "Progress",
        Communication,
        "Show the status of ongoing work, linear or circular.",
        spec("loading-progress"),
        Some(&playgrounds::progress::PLAYGROUND),
    ),
    component(
        "loading-indicator",
        "Loading indicator",
        Communication,
        "Show that content is on its way, for short waits.",
        spec("loading-progress"),
        Some(&playgrounds::loading_indicator::PLAYGROUND),
    ),
    component(
        "pull-to-refresh",
        "Pull to refresh",
        Communication,
        "Pull the top of a list down to refresh it.",
        spec("loading-progress"),
        Some(&playgrounds::pull_to_refresh::PLAYGROUND),
    ),
    component(
        "snackbar",
        "Snackbar",
        Communication,
        "Brief updates about app processes at the bottom of the screen.",
        spec("snackbar"),
        Some(&playgrounds::snackbar::PLAYGROUND),
    ),
    component(
        "tooltip",
        "Tooltip",
        Communication,
        "Brief labels or messages that appear on hover or focus.",
        spec("tooltips"),
        Some(&playgrounds::tooltip::PLAYGROUND),
    ),
    // Styles
    document(
        "color",
        "Color",
        Styles,
        "Every color role in each theme, light and dark, with contrast checks.",
    ),
    document(
        "typography",
        "Typography",
        Styles,
        "The 15 type scale roles as live samples you can edit.",
    ),
    document(
        "shape",
        "Shape",
        Styles,
        "The corner scale: make your components squarer or rounder.",
    ),
    document(
        "motion",
        "Motion",
        Styles,
        "Spring tokens for spatial and effects motion: play and compare them.",
    ),
    document(
        "elevation",
        "Elevation",
        Styles,
        "Shadow levels and tonal surfaces.",
    ),
    // Examples
    component(
        "examples",
        "Examples",
        Examples,
        "Complete screens built from the library's components: pick a device and a screen, and use the app.",
        spec("examples"),
        Some(&playgrounds::examples::PLAYGROUND),
    ),
];

/// The index of the entry with `id`.
#[cfg_attr(target_family = "wasm", allow(dead_code))]
pub fn position(id: &str) -> Option<usize> {
    CATALOG.iter().position(|entry| entry.id == id)
}

/// The `PAGES` index of each entry's spec page.
pub fn spec_page(entry: usize) -> Option<usize> {
    static INDEX: OnceLock<Vec<Option<usize>>> = OnceLock::new();
    INDEX.get_or_init(|| {
        CATALOG
            .iter()
            .map(|entry| {
                entry.spec.map(|spec| {
                    PAGES
                        .iter()
                        .position(|page| page.info.id == spec.page)
                        .unwrap_or_else(|| panic!("unknown spec page {}", spec.page))
                })
            })
            .collect()
    })[entry]
}

/// The first entry of a top section.
pub fn first_of(section: TopSection) -> usize {
    CATALOG
        .iter()
        .position(|entry| entry.group.section() == section)
        .expect("every section has an entry")
}

/// The entry a capture's `--section` names: a catalog id, or a kit page id
/// (the first entry showing that page; such captures open its Specs tab).
#[cfg_attr(target_family = "wasm", allow(dead_code))]
pub fn resolve(id: &str) -> Option<(usize, bool)> {
    if let Some(entry) = position(id) {
        return Some((entry, false));
    }
    let page = PAGES.iter().position(|page| page.info.id == id)?;
    let entry = (0..CATALOG.len()).find(|entry| spec_page(*entry) == Some(page))?;
    Some((entry, true))
}

/// What a browser address's hash names: a page id (or a kit page id, which opens
/// on Specs) and optionally `/specs` or `/preview`.
#[cfg(any(target_family = "wasm", test))]
pub fn parse_route(hash: &str) -> Option<(usize, Option<super::Tab>)> {
    let route = hash.strip_prefix('#').unwrap_or(hash);
    let (page, tab) = route.split_once('/').unwrap_or((route, ""));
    let (entry, kit_page) = resolve(page)?;
    let tab = match tab {
        "specs" => Some(super::Tab::Specs),
        "preview" => Some(super::Tab::Preview),
        _ => kit_page.then_some(super::Tab::Specs),
    };
    Some((entry, tab))
}

/// The hash that names `entry` on `tab`.
#[cfg(any(target_family = "wasm", test))]
pub fn route_hash(entry: usize, tab: super::Tab) -> String {
    let id = CATALOG[entry].id;
    match tab {
        super::Tab::Preview => format!("#{id}"),
        super::Tab::Specs => format!("#{id}/specs"),
    }
}

// Slices of the Buttons kit page, which holds every action component.
use pages::buttons::{Row, rows};

/// The body rows from `start` (exclusive) up to the next section header.
fn section_rows(section: Option<usize>) -> Vec<usize> {
    let all = rows();
    let start = section.map_or(0, |section| {
        all.iter()
            .position(|row| *row == Row::Section(section))
            .expect("the Buttons page has this section")
            + 1
    });
    all[start..]
        .iter()
        .take_while(|row| !matches!(row, Row::Section(_)))
        .enumerate()
        .map(|(offset, _)| start + offset)
        .collect()
}
fn button_rows() -> Vec<usize> {
    section_rows(None)
}
fn icon_button_rows() -> Vec<usize> {
    section_rows(Some(0))
}
fn split_button_rows() -> Vec<usize> {
    section_rows(Some(1))
}
fn fab_menu_rows() -> Vec<usize> {
    section_rows(Some(2))
}
fn fab_rows() -> Vec<usize> {
    section_rows(Some(3))
        .into_iter()
        .filter(|row| !matches!(rows()[*row], Row::Fab { extended: true, .. }))
        .collect()
}
fn extended_fab_rows() -> Vec<usize> {
    section_rows(Some(3))
        .into_iter()
        .filter(|row| matches!(rows()[*row], Row::Fab { extended: true, .. } | Row::FabDemo))
        .collect()
}
fn button_group_rows() -> Vec<usize> {
    section_rows(Some(4))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn catalog_follows_md3io_groups_and_resolves_every_spec_page() {
        let mut ids: Vec<_> = CATALOG.iter().map(|entry| entry.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), CATALOG.len(), "entry ids are unique");
        // Groups appear in one contiguous run each, in md3.io's order.
        let mut groups = Vec::new();
        for entry in CATALOG {
            if groups.last() != Some(&entry.group) {
                assert!(!groups.contains(&entry.group), "{:?} split", entry.group);
                groups.push(entry.group);
            }
        }
        assert_eq!(
            groups,
            [
                Library,
                Actions,
                SelectionInput,
                Navigation,
                Containment,
                Communication,
                Styles,
                Examples
            ]
        );
        for (index, entry) in CATALOG.iter().enumerate() {
            assert!(!entry.description.is_empty(), "{}", entry.id);
            assert_eq!(entry.spec.is_some(), spec_page(index).is_some());
            if let Some(SpecRef {
                rows: Some(rows), ..
            }) = entry.spec
            {
                assert!(!rows().is_empty(), "{} has spec rows", entry.id);
            }
        }
        // Every kit page is reachable from some entry.
        for (page, info) in PAGES.iter().enumerate() {
            assert!(
                (0..CATALOG.len()).any(|entry| spec_page(entry) == Some(page)),
                "{} is reachable",
                info.info.id
            );
        }
    }

    #[test]
    fn button_slices_partition_the_buttons_page_without_section_headers() {
        let all = rows();
        let mut covered: Vec<usize> = [
            button_rows(),
            icon_button_rows(),
            split_button_rows(),
            fab_menu_rows(),
            fab_rows(),
            button_group_rows(),
        ]
        .concat();
        covered.extend(
            extended_fab_rows()
                .into_iter()
                .filter(|row| matches!(all[*row], Row::Fab { extended: true, .. })),
        );
        covered.sort();
        let expected: Vec<usize> = (0..all.len())
            .filter(|row| !matches!(all[*row], Row::Section(_)))
            .collect();
        assert_eq!(covered, expected);
    }

    #[test]
    fn legacy_page_ids_resolve_to_their_specs() {
        assert_eq!(
            resolve("button"),
            Some((position("button").unwrap(), false))
        );
        assert_eq!(
            resolve("buttons"),
            Some((position("button").unwrap(), true))
        );
        assert_eq!(
            resolve("text-fields"),
            Some((position("text-field").unwrap(), true))
        );
        assert_eq!(resolve("color"), Some((position("color").unwrap(), false)));
        assert_eq!(resolve("nope"), None);
    }
}
