//! App bars: the kit's top app bar configurations (flat and scrolled
//! under), titles with subtitles, the bottom app bar, and a live demo whose
//! top app bar collapses as a small list scrolls under it while the bottom
//! app bar hides. The Gallery's own window title bar follows. The kit's XR
//! section is not mirrored.
use super::prelude::*;
use super::toolbars::{fab, nav_icon};
use gpui_kit_material::components::contain_scroll;
use gpui_kit_material::icons::{Icon, Style, Symbol};
use std::sync::OnceLock;

/// The kit's image placeholder (triangle, circle and square).
fn placeholder_icon() -> Icon {
    static PATH: OnceLock<SharedString> = OnceLock::new();
    Icon::new(
        PATH.get_or_init(|| {
            Symbol::named("category")
                .expect("a Material Symbol")
                .icon()
                .style(Style::Rounded)
                .filled(true)
                .asset_path()
                .into()
        })
        .clone(),
    )
}

/// An icon button for a bar; `lead` paints the navigation icon's
/// on-surface ink.
fn action(
    id: impl Into<ElementId>,
    icon: &'static str,
    label: &'static str,
    lead: bool,
) -> ToolbarButton {
    let button = ToolbarButton::icon(id, nav_icon(icon), label);
    if lead {
        button.ink(ColorRole::ON_SURFACE)
    } else {
        button
    }
}

/// An inert specimen icon button.
fn inert(
    id: impl Into<ElementId>,
    icon: &'static str,
    label: &'static str,
    lead: bool,
) -> ToolbarButton {
    action(id, icon, label, lead).preview_state(InteractionPreview::Enabled)
}

/// The kit's 32dp avatar placeholder in a 48dp target.
fn avatar() -> impl IntoElement {
    div()
        .size(rem(48.))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .size(rem(32.))
                .rounded_full()
                .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    placeholder_icon()
                        .size(rem(16.))
                        .color(color(ColorRole::OUTLINE_VARIANT)),
                ),
        )
}

/// The kit's thumbnail placeholder in a small bar's title slot.
fn thumbnail() -> impl IntoElement {
    div()
        .flex_1()
        .h(rem(40.))
        .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
        .flex()
        .items_center()
        .justify_center()
        .child(
            placeholder_icon()
                .size(rem(24.))
                .color(color(ColorRole::OUTLINE_VARIANT)),
        )
}

/// Specimen width in a two-column board: narrower than the kit's 412dp
/// frame so both columns fit the Gallery panel without wrapping.
const SPECIMEN: f32 = 360.;

const CONFIGURATIONS: [&str; 6] = [
    "Small-centered",
    "Search",
    "Small-image",
    "Small",
    "Medium flexible",
    "Large flexible",
];

/// One kit configuration as a 360dp specimen, flat or scrolled under (the
/// kit shows flexible bars expanded in both; their collapse is the live
/// demo's).
fn specimen(configuration: usize, scrolled: bool) -> impl IntoElement {
    let index = configuration * 2 + usize::from(scrolled);
    let id = ("app-bar-specimen", index);
    let menu = || inert(("app-bar-menu", index), "menu", "Menu", true);
    let back = || inert(("app-bar-back", index), "arrow_back", "Back", true);
    let search = || inert(("app-bar-search", index), "search", "Search", false);
    let bar = match configuration {
        0 => TopAppBar::new(id, TopAppBarVariant::CenterAligned)
            .title("Label")
            .navigation_icon(menu())
            .action(avatar()),
        1 => TopAppBar::new(id, TopAppBarVariant::Search)
            .search_placeholder("Search product")
            .preview_state(InteractionPreview::Enabled)
            .navigation_icon(menu())
            .action(avatar()),
        2 => TopAppBar::new(id, TopAppBarVariant::Small)
            .title_content(thumbnail())
            .navigation_icon(back())
            .action(search()),
        3 => TopAppBar::new(id, TopAppBarVariant::Small)
            .title("Label")
            .navigation_icon(back())
            .action(search()),
        4 => TopAppBar::new(id, TopAppBarVariant::MediumFlexible)
            .title("Label")
            .navigation_icon(back())
            .action(search()),
        _ => TopAppBar::new(id, TopAppBarVariant::LargeFlexible)
            .title("Label")
            .navigation_icon(back())
            .action(search()),
    };
    div().w(rem(SPECIMEN)).child(bar.scrolled_under(scrolled))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    Configurations,
    Subtitles,
    Bottom,
    Demo,
    Window,
}

const ROWS: [Row; 5] = [
    Row::Configurations,
    Row::Subtitles,
    Row::Bottom,
    Row::Demo,
    Row::Window,
];

fn rows_where(keep: fn(Row) -> bool) -> Vec<usize> {
    (0..ROWS.len()).filter(|row| keep(ROWS[*row])).collect()
}
/// The top app bar's rows on its Specs tab.
pub fn top_bar_rows() -> Vec<usize> {
    rows_where(|row| !matches!(row, Row::Bottom))
}
/// The bottom app bar's rows on its Specs tab.
pub fn bottom_bar_rows() -> Vec<usize> {
    rows_where(|row| matches!(row, Row::Bottom | Row::Demo))
}

const BOTTOM_ICONS: [(&str, &str); 4] = [
    ("search", "Search"),
    ("delete", "Delete"),
    ("archive", "Archive"),
    ("forward", "Forward"),
];

pub struct Page {
    page: PageCx,
    demo: Entity<AppBarDemo>,
}

impl Page {
    fn configurations(&self) -> AnyElement {
        board("app-bar-configurations")
            .caption("App bar")
            .columns(2)
            .column_width(SPECIMEN)
            .column_labels(["Flat", "On scroll"])
            .row_labels(CONFIGURATIONS)
            .cells((0..CONFIGURATIONS.len()).flat_map(|configuration| {
                [false, true].map(move |scrolled| specimen(configuration, scrolled))
            }))
            .into_any_element()
    }

    fn subtitles(&self) -> AnyElement {
        let variants = [
            TopAppBarVariant::Small,
            TopAppBarVariant::MediumFlexible,
            TopAppBarVariant::LargeFlexible,
        ];
        board("app-bar-subtitles")
            .caption("Title and subtitle")
            .columns(2)
            .column_width(SPECIMEN)
            .column_labels(["Left aligned", "Centered"])
            .row_labels(["Small", "Medium flexible", "Large flexible"])
            .cells(variants.into_iter().enumerate().flat_map(|(row, variant)| {
                [false, true].map(move |centered| {
                    let index = row * 2 + usize::from(centered);
                    let variant = if variant == TopAppBarVariant::Small && centered {
                        TopAppBarVariant::CenterAligned
                    } else {
                        variant
                    };
                    div().w(rem(SPECIMEN)).child(
                        TopAppBar::new(("app-bar-subtitle", index), variant)
                            .title("Label")
                            .subtitle("Supporting text")
                            .centered(centered)
                            .navigation_icon(inert(
                                ("app-bar-subtitle-back", index),
                                "arrow_back",
                                "Back",
                                true,
                            ))
                            .action(inert(
                                ("app-bar-subtitle-search", index),
                                "search",
                                "Search",
                                false,
                            )),
                    )
                })
            }))
            .into_any_element()
    }

    fn bottom(&self) -> AnyElement {
        let labels = [true, false].into_iter().flat_map(|with_fab| {
            (1..=4).rev().map(move |count: usize| {
                SharedString::from(format!(
                    "{count} icon{}{}",
                    if count == 1 { "" } else { "s" },
                    if with_fab { " + FAB" } else { "" }
                ))
            })
        });
        board("bottom-app-bars")
            .caption("Bottom app bar")
            .columns(1)
            .column_width(412.)
            .row_labels(labels.collect::<Vec<_>>())
            .cells(
                [true, false]
                    .into_iter()
                    .enumerate()
                    .flat_map(|(group, with_fab)| {
                        (1..=4).rev().map(move |count: usize| {
                            let index = group * 4 + count;
                            let bar = BottomAppBar::new(("bottom-app-bar", index), "Actions")
                                .actions(BOTTOM_ICONS[..count].iter().enumerate().map(
                                    move |(n, (icon, label))| {
                                        inert(
                                            ("bottom-app-bar-action", index * 4 + n),
                                            icon,
                                            label,
                                            false,
                                        )
                                    },
                                ));
                            let bar = if with_fab {
                                bar.fab(fab(
                                    ("bottom-app-bar-fab", index),
                                    "add",
                                    "Add",
                                    ColorRole::SECONDARY_CONTAINER,
                                    ColorRole::ON_SECONDARY_CONTAINER,
                                ))
                            } else {
                                bar
                            };
                            div().w(rem(412.)).child(bar)
                        })
                    }),
            )
            .into_any_element()
    }

    /// The Gallery's own window title bar and caption buttons, the
    /// app-drawn bar of its window.
    fn window_bar(&self, window: &mut Window) -> AnyElement {
        let shared = self.page.shared.clone();
        let caption = match Caption::detect(window) {
            Caption::System => "Caption buttons: drawn by the system",
            Caption::Windows => "Caption buttons: Windows style, drawn by the app",
            Caption::Linux => "Caption buttons: Linux style, drawn by the app",
        };
        spec::demo(
            "window-controls",
            "Window title bar",
            div()
                .flex()
                .flex_col()
                .gap(rem(12.))
                .child(accessible_text("caption-style", caption))
                .child(
                    // Inline previews of the app-drawn styles: inert and in the
                    // page's flow, not over the real title bar.
                    spec::demo_row().children(
                        [
                            (Caption::Windows, "caption-preview-windows", "Windows"),
                            (Caption::Linux, "caption-preview-linux", "Linux"),
                        ]
                        .into_iter()
                        .enumerate()
                        .map(|(index, (caption, id, name))| {
                            div()
                                .flex()
                                .flex_col()
                                .gap(rem(8.))
                                .child(spec::small_label(("caption-preview-name", index), name))
                                .child(
                                    div()
                                        .debug_selector(move || id.to_string())
                                        .flex()
                                        .rounded(RADIUS_SMALL)
                                        .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
                                        .child(
                                            CaptionButtons::new(
                                                px(48.),
                                                super::super::caption_labels(),
                                            )
                                            .caption(caption)
                                            .preview(id),
                                        ),
                                )
                        }),
                    ),
                )
                .child(
                    // A sample bar: its control takes the press itself instead
                    // of starting a window drag.
                    div()
                        .flex()
                        .h(rem(56.))
                        .px(rem(16.))
                        .gap(rem(12.))
                        .items_center()
                        .rounded(RADIUS_SMALL)
                        .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
                        .child(accessible_text("sample-title", "Sample title bar"))
                        .child(div().flex_1())
                        .child(
                            title_bar_control(
                                material_button("sample-bar-settings", ButtonVariant::Text)
                                    .button_size(ButtonSize::ExtraSmall)
                                    .child("Settings")
                                    .on_click(move |_, window, cx| {
                                        super::super::settings::open(shared.clone(), window, cx)
                                    }),
                            )
                            .reveal("reveal-sample-bar-settings", self.page.rows.list()),
                        ),
                ),
        )
        .into_any_element()
    }
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "app-bars",
        title: "App bars",
        description: "App bars display information and actions at the top of a screen, such as the page title and shortcuts to actions.",
        guideline: Some("https://m3.material.io/components/app-bars/overview"),
    };

    fn new(page: PageCx, _: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            page,
            demo: cx.new(|_| AppBarDemo::default()),
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        ROWS.len()
    }

    /// `--active` captures the live demo with its list scrolled partway, so
    /// the flexible bar shows mid-collapse and scrolled-under.
    fn apply_capture(&mut self, state: CaptureState, cx: &mut Context<Self>) {
        if state.active {
            self.demo.update(cx, |demo, cx| {
                demo.scroll.set_offset(point(px(0.), px(-40.)));
                cx.notify();
            });
        }
    }

    fn interactive(&self, row: usize) -> bool {
        matches!(ROWS[row], Row::Demo | Row::Window)
    }

    fn render_row(&mut self, row: usize, window: &mut Window, _: &mut Context<Self>) -> AnyElement {
        match ROWS[row] {
            Row::Configurations => self.configurations(),
            Row::Subtitles => self.subtitles(),
            Row::Bottom => self.bottom(),
            Row::Demo => spec::demo("app-bar-demo", "App bars on scroll", self.demo.clone())
                .into_any_element(),
            Row::Window => self.window_bar(window),
        }
    }
}

/// The live demo: a small list under a top app bar that fades to its
/// scrolled-under color and, when flexible, collapses with the list's
/// offset, over a bottom app bar that hides while the list scrolls down.
/// Its own entity, so scrolling re-renders only the demo.
pub struct AppBarDemo {
    scroll: ScrollHandle,
    last: Pixels,
    bottom_visible: bool,
    variant: TopAppBarVariant,
}

impl Default for AppBarDemo {
    fn default() -> Self {
        Self {
            scroll: ScrollHandle::new(),
            last: px(0.),
            bottom_visible: true,
            variant: TopAppBarVariant::LargeFlexible,
        }
    }
}

const DEMO_VARIANTS: [(TopAppBarVariant, &str); 5] = [
    (TopAppBarVariant::Small, "Small"),
    (TopAppBarVariant::CenterAligned, "Center"),
    (TopAppBarVariant::MediumFlexible, "Medium"),
    (TopAppBarVariant::LargeFlexible, "Large"),
    (TopAppBarVariant::Search, "Search"),
];

impl Render for AppBarDemo {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let offset = -self.scroll.offset().y;
        if offset > self.last + px(1.) {
            self.bottom_visible = false;
        } else if offset + px(1.) < self.last || offset <= px(0.) {
            self.bottom_visible = true;
        }
        self.last = offset;
        // The bar takes design px; at a larger text scale a rem is wider.
        let scale = f32::from(window.rem_size()) / gpui_kit_material::tokens::typography::BASE_PX;
        let design = f32::from(offset) / scale;
        let variant = self.variant;
        let controls = Toolbar::new("app-bar-demo-variant", "Configuration").items(
            DEMO_VARIANTS
                .into_iter()
                .enumerate()
                .map(|(index, (choice, label))| {
                    ToolbarButton::label(("app-bar-demo-variant", index), label)
                        .toggle(true)
                        .selected(variant == choice)
                        .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                            this.variant = choice;
                            this.scroll.set_offset(point(px(0.), px(0.)));
                            this.last = px(0.);
                            this.bottom_visible = true;
                            cx.notify();
                        }))
                }),
        );
        let bar = TopAppBar::new("app-bar-demo-bar", variant)
            .title("Inbox")
            .subtitle("24 messages")
            .search_placeholder("Search mail")
            .scroll_offset(design)
            .navigation_icon(action("app-bar-demo-menu", "menu", "Menu", true))
            .action(action(
                "app-bar-demo-attach",
                "attach_file",
                "Attach",
                false,
            ))
            .action(action("app-bar-demo-more", "more_vert", "More", false));
        let list = contain_scroll(
            div()
                .id("app-bar-demo-list")
                .size_full()
                .overflow_y_scroll(),
            &self.scroll,
        )
        .on_scroll(cx.listener(|_, _, _, cx| cx.notify()))
        .child(div().flex().flex_col().children((0..24).map(|index| {
            div()
                .h(rem(56.))
                .flex_shrink_0()
                .px(rem(16.))
                .flex()
                .items_center()
                .border_b_1()
                .border_color(color(ColorRole::OUTLINE_VARIANT))
                .text_color(color(ColorRole::ON_SURFACE))
                .child(SharedString::from(format!("Message {}", index + 1)))
        })));
        let bottom = BottomAppBar::new("app-bar-demo-bottom", "Actions")
            .visible(self.bottom_visible)
            .actions(
                BOTTOM_ICONS[..3]
                    .iter()
                    .enumerate()
                    .map(|(index, (icon, label))| {
                        action(("app-bar-demo-action", index), icon, label, false)
                    }),
            )
            .fab(fab(
                "app-bar-demo-fab",
                "edit",
                "Compose",
                ColorRole::SECONDARY_CONTAINER,
                ColorRole::ON_SECONDARY_CONTAINER,
            ));
        div().flex().flex_col().gap(rem(16.)).child(controls).child(
            div()
                .w(rem(412.))
                .h(rem(480.))
                .flex()
                .flex_col()
                .rounded(rem(16.))
                .overflow_hidden()
                .border_1()
                .border_color(color(ColorRole::OUTLINE_VARIANT))
                .bg(color(ColorRole::SURFACE))
                .child(bar)
                .child(div().flex_1().min_h_0().child(list))
                .child(bottom),
        )
    }
}
