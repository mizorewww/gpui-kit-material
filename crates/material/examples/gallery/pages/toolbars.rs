//! Toolbars: the kit's Examples & Usage, the floating, docked and vertical
//! toolbar matrix in standard and vibrant colors, the toolbar button
//! building blocks in every state, and a live toolbar that hides as its
//! content scrolls down and returns when it scrolls up. The kit's XR
//! section is not mirrored.
use super::prelude::*;
use gpui_kit_material::components::contain_scroll;
use gpui_kit_material::icons::{Icon, Style, Symbol};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// A Material Symbol's rounded outlined and filled asset paths, resolved
/// once per name.
fn paths(name: &'static str) -> (SharedString, SharedString) {
    static PATHS: OnceLock<Mutex<HashMap<&'static str, (SharedString, SharedString)>>> =
        OnceLock::new();
    let map = PATHS.get_or_init(Default::default);
    let mut map = map.lock().expect("icon cache");
    map.entry(name)
        .or_insert_with(|| {
            let icon = Symbol::named(name)
                .expect("a Material Symbol")
                .icon()
                .style(Style::Rounded);
            (
                icon.asset_path().into(),
                icon.filled(true).asset_path().into(),
            )
        })
        .clone()
}

/// A symbol as an outlined/filled toolbar icon.
pub(super) fn nav_icon(name: &'static str) -> NavIcon {
    let (outlined, filled) = paths(name);
    NavIcon::new(outlined, filled)
}

/// A symbol's filled form as a plain icon.
pub(super) fn filled_icon(name: &'static str) -> Icon {
    Icon::new(paths(name).1)
}

/// The gallery's FAB stand-in (the FAB component lives on the Buttons
/// page): 56dp, 16dp corners, a container color with level-3 elevation.
pub(super) fn fab(
    id: impl Into<ElementId>,
    icon: &'static str,
    label: &'static str,
    container: ColorRole,
    ink: ColorRole,
) -> impl IntoElement {
    let id = id.into();
    let (container, ink) = (color(container), color(ink));
    FocusRing::new(
        id.clone(),
        gpui_base::Button::new(id)
            .accessibility_label(label)
            .size(rem(56.))
            .flex_shrink_0()
            .rounded(rem(16.))
            .bg(container)
            .shadow(gpui_kit_material::tokens::elevation::level(3))
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(move |style| style.bg(gpui_kit_material::tokens::state::hovered(container, ink)))
            .active(move |style| {
                style.bg(gpui_kit_material::tokens::state::pressed(container, ink))
            })
            .child(filled_icon(icon).size(rem(24.)).color(ink)),
    )
}

/// An inert specimen icon button.
fn specimen(id: impl Into<ElementId>, icon: &'static str, scheme: ToolbarColor) -> ToolbarButton {
    ToolbarButton::icon(id, nav_icon(icon), icon)
        .color(scheme)
        .preview_state(InteractionPreview::Enabled)
}

const SHARE: [&str; 3] = ["share", "chat", "download"];
const STATES: [InteractionPreview; 5] = InteractionPreview::ALL;

/// The building-block bands of one color: icon button, toggleable icon
/// button, toggleable label button.
const BLOCKS: [&str; 3] = ["Icon button", "Icon button toggleable", "Button toggleable"];

pub struct Page {
    demo: Entity<ToolbarDemo>,
    /// Examples & Usage toggles.
    raised: bool,
    format: [bool; 3],
    tab: usize,
}

impl Page {
    fn floating(&self, id: &'static str, icons: &[&'static str], scheme: ToolbarColor) -> Toolbar {
        Toolbar::new(id, "Actions")
            .color(scheme)
            .items(icons.iter().enumerate().map(|(index, icon)| {
                ToolbarButton::icon((id, index), nav_icon(icon), *icon).color(scheme)
            }))
    }

    fn examples(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let tertiary = (
            ColorRole::TERTIARY_CONTAINER,
            ColorRole::ON_TERTIARY_CONTAINER,
        );
        let raise = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.raised = !this.raised;
            cx.notify();
        });
        let tabs = ["Photos", "Memories", "Library"];
        let tab_icons = ["photo", "auto_awesome", "photo_library"];
        let left = div()
            .flex()
            .flex_col()
            .items_start()
            .gap(rem(24.))
            .child(
                self.floating("ex-share", &SHARE, ToolbarColor::Standard)
                    .fab(fab("ex-share-fab", "add", "Add", tertiary.0, tertiary.1)),
            )
            .child(
                self.floating(
                    "ex-mail",
                    &["archive", "delete", "mark_email_unread"],
                    ToolbarColor::Standard,
                )
                .fab(fab("ex-mail-fab", "reply", "Reply", tertiary.0, tertiary.1)),
            )
            .child(
                Toolbar::new("ex-call", "Call")
                    .item(ToolbarButton::icon(
                        "ex-call-share",
                        nav_icon("share"),
                        "Share",
                    ))
                    .item(ToolbarButton::icon(
                        "ex-call-chat",
                        nav_icon("chat"),
                        "Chat",
                    ))
                    .item(
                        ToolbarButton::icon("ex-call-hand", nav_icon("front_hand"), "Raise hand")
                            .toggle(true)
                            .selected(self.raised)
                            .on_click(raise),
                    )
                    .item(ToolbarButton::icon(
                        "ex-call-more",
                        nav_icon("more_vert"),
                        "More",
                    ))
                    .fab(fab(
                        "ex-call-end",
                        "call_end",
                        "End call",
                        ColorRole::ERROR_CONTAINER,
                        ColorRole::ON_ERROR_CONTAINER,
                    )),
            )
            .child(
                Toolbar::new("ex-tabs", "Sections").items((0..3).map(|index| {
                    let select = cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.tab = index;
                        cx.notify();
                    });
                    let button = ToolbarButton::label(("ex-tab", index), tabs[index])
                        .toggle(true)
                        .selected(self.tab == index)
                        .on_click(select);
                    if index == self.tab {
                        button.leading_icon(nav_icon(tab_icons[index]))
                    } else {
                        button
                    }
                })),
            )
            .child(
                Toolbar::new("ex-compose", "Compose")
                    .item(ToolbarButton::icon(
                        "ex-compose-share",
                        nav_icon("share"),
                        "Share",
                    ))
                    .item(ToolbarButton::icon(
                        "ex-compose-chat",
                        nav_icon("chat"),
                        "Chat",
                    ))
                    .item(
                        material_button("ex-compose-add", ButtonVariant::Filled)
                            .icon(filled_icon("add"))
                            .accessibility_label("Add"),
                    ),
            );
        let docked = |id: &'static str| {
            Toolbar::new(id, "Actions").configuration(ToolbarConfiguration::Docked)
        };
        let right = div()
            .w(rem(412.))
            .flex()
            .flex_col()
            .gap(rem(24.))
            .child(
                docked("ex-docked-share")
                    .item(ToolbarButton::icon(
                        "ex-d1-share",
                        nav_icon("share"),
                        "Share",
                    ))
                    .item(ToolbarButton::icon("ex-d1-chat", nav_icon("chat"), "Chat"))
                    .item(div().flex_1())
                    .item(ToolbarButton::icon(
                        "ex-d1-download",
                        nav_icon("download"),
                        "Download",
                    )),
            )
            .child(
                docked("ex-docked-steps")
                    .item(material_button("ex-d2-back", ButtonVariant::Text).child("Back"))
                    .item(material_button("ex-d2-next", ButtonVariant::Filled).child("Next"))
                    .item(ToolbarButton::icon(
                        "ex-d2-download",
                        nav_icon("download"),
                        "Download",
                    )),
            )
            .child(
                docked("ex-docked-nav")
                    .item(ToolbarButton::icon(
                        "ex-d3-back",
                        nav_icon("arrow_back"),
                        "Back",
                    ))
                    .item(ToolbarButton::icon(
                        "ex-d3-forward",
                        nav_icon("arrow_forward"),
                        "Forward",
                    ))
                    .item(div().flex_1())
                    .item(
                        material_button("ex-d3-add", ButtonVariant::Filled)
                            .shape(ButtonShape::Square)
                            .icon(filled_icon("add"))
                            .accessibility_label("Add"),
                    ),
            )
            .child(
                docked("ex-docked-mail").color(ToolbarColor::Vibrant).items(
                    ["archive", "delete", "mark_email_unread"]
                        .into_iter()
                        .enumerate()
                        .map(|(index, icon)| {
                            ToolbarButton::icon(("ex-d4", index), nav_icon(icon), icon)
                                .color(ToolbarColor::Vibrant)
                        }),
                ),
            );
        let vertical = Toolbar::new("ex-history", "History")
            .orientation(ToolbarOrientation::Vertical)
            .items(
                [("undo", "Undo"), ("redo", "Redo"), ("add", "Add")]
                    .into_iter()
                    .enumerate()
                    .map(|(index, (icon, label))| {
                        ToolbarButton::icon(("ex-history", index), nav_icon(icon), label)
                    }),
            );
        let format = Toolbar::new("ex-format", "Format")
            .color(ToolbarColor::Vibrant)
            .items(
                [
                    ("format_bold", "Bold"),
                    ("format_italic", "Italic"),
                    ("format_underlined", "Underline"),
                ]
                .into_iter()
                .enumerate()
                .map(|(index, (icon, label))| {
                    let toggle = cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.format[index] = !this.format[index];
                        cx.notify();
                    });
                    ToolbarButton::icon(("ex-format", index), nav_icon(icon), label)
                        .color(ToolbarColor::Vibrant)
                        .toggle(true)
                        .selected(self.format[index])
                        .on_click(toggle)
                }),
            );
        let meeting = Toolbar::new("ex-meeting", "Meeting")
            .color(ToolbarColor::Vibrant)
            .item(
                ToolbarButton::icon("ex-meeting-video", nav_icon("videocam_off"), "Camera off")
                    .color(ToolbarColor::Vibrant),
            )
            .item(
                ToolbarButton::icon("ex-meeting-mic", nav_icon("mic"), "Microphone")
                    .color(ToolbarColor::Vibrant),
            )
            .item(
                ToolbarButton::icon("ex-meeting-hand", nav_icon("front_hand"), "Raise hand")
                    .color(ToolbarColor::Vibrant)
                    .toggle(true)
                    .selected(true),
            );
        spec::demo(
            "toolbar-examples",
            "Examples & Usage",
            div()
                .flex()
                .flex_col()
                .gap(rem(32.))
                // Room for the floating toolbars' shadows inside the row.
                .pb(rem(12.))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap(rem(48.))
                        .child(left)
                        .child(right)
                        .child(vertical),
                )
                .child(spec::demo_row().gap(rem(48.)).child(format).child(meeting)),
        )
        .into_any_element()
    }

    fn matrix(&self) -> AnyElement {
        let row = |configuration: ToolbarConfiguration, orientation: ToolbarOrientation| {
            ToolbarColor::ALL.map(move |scheme| {
                let id = (
                    "toolbar-matrix",
                    configuration as usize * 4 + orientation as usize * 2 + scheme as usize,
                );
                let toolbar = Toolbar::new(id, "Actions")
                    .configuration(configuration)
                    .orientation(orientation)
                    .color(scheme)
                    .items(SHARE.iter().enumerate().map(move |(index, icon)| {
                        specimen((id.0, id.1 * 4 + index), icon, scheme)
                    }));
                if configuration == ToolbarConfiguration::Docked {
                    div().w(rem(352.)).child(toolbar).into_any_element()
                } else {
                    toolbar.into_any_element()
                }
            })
        };
        board("toolbar-matrix")
            .caption("Toolbar")
            .columns(2)
            .column_width(384.)
            .column_labels(ToolbarColor::ALL.map(ToolbarColor::name))
            .row_labels(["Floating", "Docked", "Floating - Vertical"])
            .cells(
                row(
                    ToolbarConfiguration::Floating,
                    ToolbarOrientation::Horizontal,
                )
                .into_iter()
                .chain(row(
                    ToolbarConfiguration::Docked,
                    ToolbarOrientation::Horizontal,
                ))
                .chain(row(
                    ToolbarConfiguration::Floating,
                    ToolbarOrientation::Vertical,
                )),
            )
            .into_any_element()
    }

    fn blocks(&self, index: usize) -> AnyElement {
        let scheme = ToolbarColor::ALL[index / BLOCKS.len()];
        let block = index % BLOCKS.len();
        let toggles: &[bool] = if block == 0 { &[false] } else { &[false, true] };
        let id = ("toolbar-block", index);
        board(id)
            .band(Band::of(block, BLOCKS.len()))
            .caption(if block == 0 {
                match scheme {
                    ToolbarColor::Standard => "Building blocks · Standard",
                    ToolbarColor::Vibrant => "Building blocks · Vibrant",
                }
            } else {
                ""
            })
            .side_label(BLOCKS[block])
            .columns(STATES.len())
            .column_width(112.)
            .when(block == 0, |board| {
                board.column_labels(STATES.map(InteractionPreview::name))
            })
            .row_labels(if block == 0 {
                vec!["Enabled"]
            } else {
                vec!["Unselected", "Selected"]
            })
            .cells(toggles.iter().flat_map(move |&selected| {
                STATES.into_iter().enumerate().map(move |(column, state)| {
                    let cell = (id.1 * 2 + usize::from(selected)) * 8 + column;
                    let button = if block == 2 {
                        ToolbarButton::label(("toolbar-block-cell", cell), "Label")
                            .leading_icon(nav_icon("stars"))
                    } else {
                        ToolbarButton::icon(("toolbar-block-cell", cell), nav_icon("stars"), "Star")
                    };
                    button
                        .color(scheme)
                        .toggle(block > 0)
                        .selected(selected)
                        .preview_state(state)
                })
            }))
            .into_any_element()
    }
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "toolbars",
        title: "Toolbars",
        description: "Toolbars display frequently used actions relevant to the current page.",
        guideline: Some("https://m3.material.io/components/toolbars/overview"),
    };

    fn new(_: PageCx, _: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            demo: cx.new(|_| ToolbarDemo::default()),
            raised: true,
            format: [true, false, false],
            tab: 0,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        3 + 2 * BLOCKS.len()
    }

    fn row_gap(&self, row: usize) -> f32 {
        match row {
            0 => spec::HEADER_TO_BODY,
            row if (2..2 + 2 * BLOCKS.len()).contains(&row) => {
                Band::of((row - 2) % BLOCKS.len(), BLOCKS.len()).row_gap()
            }
            _ => spec::ROW_GAP,
        }
    }

    fn interactive(&self, row: usize) -> bool {
        row == 0 || row == 2 + 2 * BLOCKS.len()
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match row {
            0 => self.examples(cx),
            1 => self.matrix(),
            row if row < 2 + 2 * BLOCKS.len() => self.blocks(row - 2),
            _ => spec::demo("toolbar-demo", "Toolbar on scroll", self.demo.clone())
                .into_any_element(),
        }
    }
}

/// The live demo: a feed whose floating toolbar and FAB, or docked
/// toolbar, hide while it scrolls down and return when it scrolls up. Its
/// own entity, so scrolling re-renders only the demo.
pub struct ToolbarDemo {
    scroll: ScrollHandle,
    last: Pixels,
    visible: bool,
    scheme: ToolbarColor,
    docked: bool,
}

impl Default for ToolbarDemo {
    fn default() -> Self {
        Self {
            scroll: ScrollHandle::new(),
            last: px(0.),
            visible: true,
            scheme: ToolbarColor::Standard,
            docked: false,
        }
    }
}

const FEED: usize = 24;

impl Render for ToolbarDemo {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Scroll direction picks visibility: down hides, up shows.
        let offset = -self.scroll.offset().y;
        if offset > self.last + px(1.) {
            self.visible = false;
        } else if offset + px(1.) < self.last || offset <= px(0.) {
            self.visible = true;
        }
        self.last = offset;
        let scheme = self.scheme;
        let option = |id: &'static str, label: &'static str, selected: bool| {
            ToolbarButton::label(id, label)
                .toggle(true)
                .selected(selected)
        };
        let controls = spec::demo_row()
            .child(Toolbar::new("toolbar-demo-color", "Color").items(
                ToolbarColor::ALL.into_iter().map(|choice| {
                    option(
                        match choice {
                            ToolbarColor::Standard => "toolbar-demo-standard",
                            ToolbarColor::Vibrant => "toolbar-demo-vibrant",
                        },
                        choice.name(),
                        scheme == choice,
                    )
                    .on_click(cx.listener(
                        move |this, _: &ClickEvent, _, cx| {
                            this.scheme = choice;
                            cx.notify();
                        },
                    ))
                }),
            ))
            .child(
                Toolbar::new("toolbar-demo-configuration", "Configuration").items(
                    [
                        (false, "toolbar-demo-floating", "Floating"),
                        (true, "toolbar-demo-docked", "Docked"),
                    ]
                    .into_iter()
                    .map(|(docked, id, label)| {
                        option(id, label, self.docked == docked).on_click(cx.listener(
                            move |this, _: &ClickEvent, _, cx| {
                                this.docked = docked;
                                this.visible = true;
                                cx.notify();
                            },
                        ))
                    }),
                ),
            );
        let feed = contain_scroll(
            div()
                .id("toolbar-demo-feed")
                .size_full()
                .overflow_y_scroll(),
            &self.scroll,
        )
        .on_scroll(cx.listener(|_, _, _, cx| cx.notify()))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(rem(12.))
                .p(rem(16.))
                .pb(rem(96.))
                .children((0..FEED).map(|index| {
                    div()
                        .h(rem(72.))
                        .flex_shrink_0()
                        .rounded(rem(12.))
                        .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
                        .px(rem(16.))
                        .flex()
                        .items_center()
                        .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                        .child(SharedString::from(format!("Item {}", index + 1)))
                })),
        );
        let toolbar = Toolbar::new("toolbar-demo-toolbar", "Actions")
            .color(scheme)
            .visible(self.visible)
            .items(SHARE.iter().enumerate().map(|(index, icon)| {
                ToolbarButton::icon(("toolbar-demo-item", index), nav_icon(icon), *icon)
                    .color(scheme)
            }));
        let frame = div()
            .relative()
            .w(rem(412.))
            .h(rem(420.))
            .flex()
            .flex_col()
            .rounded(rem(16.))
            .overflow_hidden()
            .border_1()
            .border_color(color(ColorRole::OUTLINE_VARIANT))
            .bg(color(ColorRole::SURFACE))
            .child(div().flex_1().min_h_0().child(feed));
        let frame = if self.docked {
            frame.child(
                toolbar
                    .configuration(ToolbarConfiguration::Docked)
                    .into_any_element(),
            )
        } else {
            frame.child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .bottom(rem(TOOLBAR_MARGIN))
                    .flex()
                    .justify_center()
                    .child(toolbar.fab(fab(
                        "toolbar-demo-fab",
                        "add",
                        "Add",
                        ColorRole::TERTIARY_CONTAINER,
                        ColorRole::ON_TERTIARY_CONTAINER,
                    ))),
            )
        };
        div()
            .flex()
            .flex_col()
            .gap(rem(16.))
            .child(controls)
            .child(frame)
    }
}
