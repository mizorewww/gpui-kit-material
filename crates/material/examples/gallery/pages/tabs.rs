//! Tabs: the kit's fixed and scrollable primary and secondary tab rows, the
//! tab building blocks in every interaction state, and live tabs.
use super::prelude::*;
use gpui_kit_material::icons::{Icon, Style, Symbol};
use std::sync::OnceLock;

/// The kit's media icons (video, photos, audio) and the star placeholder.
fn icon(index: usize) -> Icon {
    static PATHS: OnceLock<[String; 4]> = OnceLock::new();
    let paths = PATHS.get_or_init(|| {
        ["videocam", "image", "music_note", "stars"].map(|name| {
            Symbol::named(name)
                .expect("a Material Symbol")
                .icon()
                .style(Style::Rounded)
                .asset_path()
        })
    });
    Icon::new(SharedString::new_static(paths[index].as_str()))
}

const MEDIA: [&str; 4] = ["Video", "Photos", "Audio", "Tab"];

/// A kit tab row: Video, Photos, Audio (and a fourth, clipped, when
/// scrollable), as a static specimen.
fn media_tabs(
    id: (&'static str, usize),
    variant: TabsVariant,
    icons: bool,
    labels: bool,
    scrollable: bool,
) -> impl IntoElement {
    let count = if scrollable { 4 } else { 3 };
    div().w(rem(338.)).overflow_hidden().child(
        MaterialTabs::new(id, "Media")
            .variant(variant)
            .scrollable(scrollable)
            .selected("Video")
            .items((0..count).map(move |index| {
                let mut item = TabItem::new(MEDIA[index], MEDIA[index])
                    .preview_state(InteractionPreview::Enabled);
                if icons {
                    item = item.icon(icon(index));
                }
                if !labels {
                    item = item.icon_only();
                }
                item
            })),
    )
}

/// The kit's rows of tab rows: (variant, icons, labels).
const ROWS: [(TabsVariant, bool, bool, &str); 5] = [
    (TabsVariant::Primary, false, true, "Primary · label"),
    (TabsVariant::Primary, true, true, "Primary · icon and label"),
    (TabsVariant::Primary, true, false, "Primary · icon"),
    (TabsVariant::Secondary, false, true, "Secondary · label"),
    (
        TabsVariant::Secondary,
        true,
        true,
        "Secondary · icon and label",
    ),
];

const STATES: [InteractionPreview; 4] = [
    InteractionPreview::Enabled,
    InteractionPreview::Hovered,
    InteractionPreview::Focused,
    InteractionPreview::Pressed,
];

/// The building-block boards, in the kit's order.
const BLOCKS: [(TabsVariant, bool, bool, &str); 5] = [
    (
        TabsVariant::Primary,
        true,
        true,
        "Primary tabs / Icon and label",
    ),
    (
        TabsVariant::Primary,
        true,
        false,
        "Primary tabs / Icon only",
    ),
    (
        TabsVariant::Primary,
        false,
        true,
        "Primary tabs / Label only",
    ),
    (
        TabsVariant::Secondary,
        false,
        true,
        "Secondary tabs / Label only",
    ),
    (
        TabsVariant::Secondary,
        true,
        true,
        "Secondary tabs / Icon and label",
    ),
];

pub struct Page {
    primary: SharedString,
    secondary: SharedString,
}

/// The live secondary tabs: many, with badges, scrolling.
fn library_items() -> &'static [TabItem] {
    static ITEMS: OnceLock<Vec<TabItem>> = OnceLock::new();
    ITEMS.get_or_init(|| {
        vec![
            TabItem::new("all", "All"),
            TabItem::new("albums", "Albums").badge(NavBadge::Large("12".into())),
            TabItem::new("artists", "Artists"),
            TabItem::new("playlists", "Playlists").badge(NavBadge::Small),
            TabItem::new("podcasts", "Podcasts"),
            TabItem::new("audiobooks", "Audiobooks"),
            TabItem::new("downloads", "Downloads"),
            TabItem::new("history", "History"),
        ]
    })
}

impl Page {
    fn overview(&self) -> AnyElement {
        board("tabs-overview")
            .caption("Tabs")
            .columns(2)
            .column_width(338.)
            .column_labels(["Fixed", "Scrollable"])
            .cells(
                ROWS.into_iter()
                    .enumerate()
                    .flat_map(|(index, (variant, icons, labels, _))| {
                        [false, true].map(move |scrollable| {
                            media_tabs(
                                ("tabs-overview", index * 2 + usize::from(scrollable)),
                                variant,
                                icons,
                                labels,
                                scrollable,
                            )
                        })
                    }),
            )
            .into_any_element()
    }

    fn blocks(&self, block: usize) -> AnyElement {
        let (variant, icons, labels, caption) = BLOCKS[block];
        let width = if variant == TabsVariant::Secondary && icons {
            88.
        } else {
            56.
        };
        board(("tab-blocks", block))
            .band(Band::of(block, BLOCKS.len()))
            .caption(if block == 0 { "Building Blocks" } else { "" })
            .side_label(caption)
            .columns(STATES.len())
            .column_width(120.)
            .when(block == 0, |board| {
                board.column_labels(STATES.map(InteractionPreview::name))
            })
            .row_labels(["Unselected", "Selected"])
            .cells([false, true].into_iter().flat_map(move |selected| {
                STATES.into_iter().enumerate().map(move |(column, state)| {
                    let mut item = TabItem::new("tab", "Tab").preview_state(state);
                    if icons {
                        item = item.icon(icon(3));
                    }
                    if !labels {
                        item = item.icon_only();
                    }
                    div().w(rem(width + 32.)).child(
                        MaterialTabs::new(
                            (
                                "tab-block",
                                (block * 2 + usize::from(selected)) * 4 + column,
                            ),
                            "Specimen",
                        )
                        .variant(variant)
                        .items([item])
                        .when(selected, |tabs| tabs.selected("tab")),
                    )
                })
            }))
            .into_any_element()
    }

    fn live(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let primary_change = cx.listener(|this, value: &SharedString, _, cx| {
            this.primary = value.clone();
            cx.notify();
        });
        let secondary_change = cx.listener(|this, value: &SharedString, _, cx| {
            this.secondary = value.clone();
            cx.notify();
        });
        let secondary_label = library_items()
            .iter()
            .find(|item| item.value() == &self.secondary)
            .map(|item| item.label().clone())
            .unwrap_or_default();
        spec::demo(
            "tabs-demo",
            "Tabs in use",
            div()
                .w(rem(560.))
                .max_w_full()
                .flex()
                .flex_col()
                .rounded(rem(16.))
                .overflow_hidden()
                .border_1()
                .border_color(color(ColorRole::OUTLINE_VARIANT))
                .child(
                    MaterialTabs::new("demo-primary-tabs", "Media")
                        .items((0..3).map(|index| {
                            let item = TabItem::new(MEDIA[index], MEDIA[index]).icon(icon(index));
                            if index == 2 {
                                item.badge(NavBadge::Small)
                            } else {
                                item
                            }
                        }))
                        .selected(self.primary.clone())
                        .on_change(primary_change),
                )
                .child(
                    MaterialTabs::new("demo-secondary-tabs", "Library")
                        .variant(TabsVariant::Secondary)
                        .scrollable(true)
                        .items(library_items().iter().cloned())
                        .selected(self.secondary.clone())
                        .on_change(secondary_change),
                )
                .child(
                    div()
                        .h(rem(120.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(color(ColorRole::SURFACE))
                        .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                        .child(SharedString::from(format!(
                            "{} · {}",
                            self.primary, secondary_label
                        ))),
                ),
        )
        .into_any_element()
    }
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "tabs",
        title: "Tabs",
        description: "Tabs organize content across different screens and views.",
        guideline: Some("https://m3.material.io/components/tabs/overview"),
    };

    fn new(_: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            primary: "Video".into(),
            secondary: "all".into(),
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        2 + BLOCKS.len()
    }

    fn row_gap(&self, row: usize) -> f32 {
        match row {
            0 => spec::HEADER_TO_BODY,
            1 => 48.,
            row if row <= BLOCKS.len() => Band::of(row - 1, BLOCKS.len()).row_gap(),
            _ => spec::ROW_GAP,
        }
    }

    fn interactive(&self, row: usize) -> bool {
        row == 1 + BLOCKS.len()
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match row {
            0 => self.overview(),
            row if row <= BLOCKS.len() => self.blocks(row - 1),
            _ => self.live(cx),
        }
    }
}
