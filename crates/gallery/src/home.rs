//! The Gallery's home: the Overview page's lead, in the spirit of md3.io's
//! landing page. A hero with M3 Expressive display type and a cluster of
//! library shapes that morph when pressed, the ways in (components, styles,
//! examples), three tiles of real, working components (the theme and mode
//! the whole Gallery paints, a few live controls, and how to add the crate),
//! then the heading over the overview's component groups.
//!
//! Everything here is drawn by the library's own components and shapes with
//! the application's color roles. It is its own view: a press, toggle or
//! drag here renders only it, and nothing animates unless asked to.
use super::catalog::{self, CATALOG};
use super::playground::{CODE_FONT, symbol};
use super::spec::type_role;
use super::*;
use gpui_base::slider::SliderState;
use gpui_kit_material::color::Scheme;

/// Content narrower than this stacks the hero's shapes under its text.
const HERO_SIDE_BY_SIDE: f32 = 760.;
/// The hero's shapes, side by side with the text: a 2×2 cluster of this side.
const HERO_SHAPE: f32 = 120.;
const HERO_SHAPE_GAP: f32 = 16.;
/// The largest a stacked hero shape grows.
const HERO_SHAPE_STACKED: f32 = 72.;
/// A tile never gets narrower than this before the tiles wrap.
const TILE_MIN_WIDTH: f32 = 280.;
const TILE_GAP: f32 = 20.;
/// M3's extra-large corner (28dp), for the hero and the tiles.
const HERO_RADIUS: f32 = 28.;
const TILE_RADIUS: f32 = 24.;
const SWATCH: f32 = 44.;
/// The content width, in design px, below which the install line may break
/// inside the crate's name: the tile's padding, the line and its copy button
/// no longer fit side by side.
const INSTALL_UNBROKEN: f32 = 300.;

/// What the hero's four shapes cycle through when pressed, each slot from
/// its own starting point.
const SHAPE_CYCLE: [ShapeKind; 8] = [
    ShapeKind::Cookie9Sided,
    ShapeKind::Flower,
    ShapeKind::Pill,
    ShapeKind::SoftBurst,
    ShapeKind::Arch,
    ShapeKind::Clover4Leaf,
    ShapeKind::Gem,
    ShapeKind::Sunny,
];
/// Each hero shape's fill.
const SHAPE_ROLES: [ColorRole; 4] = [
    ColorRole::PRIMARY,
    ColorRole::TERTIARY_CONTAINER,
    ColorRole::SECONDARY,
    ColorRole::TERTIARY,
];

/// The themes the "Make it yours" tile offers, with their names.
const SWATCH_THEMES: [(ThemeId, &str); 8] = [
    (ThemeId::Baseline, "Baseline"),
    (ThemeId::Vibrant, "Vibrant"),
    (ThemeId::Expressive, "Expressive"),
    (ThemeId::Ocean, "Ocean"),
    (ThemeId::Forest, "Forest"),
    (ThemeId::Sunset, "Sunset"),
    (ThemeId::Spring, "Spring"),
    (ThemeId::Monochrome, "Monochrome"),
];

const CHIPS: [&str; 3] = ["Vegan", "Spicy", "Gluten-free"];
const TOGGLES: [(&str, &str); 3] = [
    ("favorite", "Favorite"),
    ("bookmark", "Bookmark"),
    ("notifications", "Notify me"),
];
const INSTALL: &str = "cargo add gpui-kit-material";
/// [`INSTALL`] as shown: word joiners keep a narrow tile from breaking the
/// crate's name at its hyphens.
const INSTALL_SHOWN: &str = "cargo add gpui\u{2060}-\u{2060}kit\u{2060}-\u{2060}material";
/// How long the copy button says "Copied" before it offers to copy again:
/// Kit's `Clipboard` feedback time (`gpui-component/src/clipboard.rs`).
pub(super) const COPIED_FOR: std::time::Duration = std::time::Duration::from_secs(2);

/// A swatch's scheme: `theme`'s standard contrast in `mode`. Reference
/// schemes are fixed data, so each is built once and never invalidated.
fn swatch_scheme(slot: usize, mode: ThemeMode) -> Scheme {
    type Schemes = [[Option<Scheme>; 2]; SWATCH_THEMES.len()];
    thread_local! {
        static SCHEMES: std::cell::RefCell<Schemes> = const {
            std::cell::RefCell::new([[None; 2]; SWATCH_THEMES.len()])
        };
    }
    let dark = usize::from(mode == ThemeMode::Dark);
    SCHEMES.with(|schemes| {
        *schemes.borrow_mut()[slot][dark]
            .get_or_insert_with(|| SWATCH_THEMES[slot].0.scheme(mode, Contrast::Standard))
    })
}

/// Below this content width, in design px (a phone at a large text scale), a
/// control that cannot reflow may be wider than its row.
const FIT_WIDTH: f32 = 280.;

/// `control` as it is, or, where the content is narrower than [`FIT_WIDTH`],
/// drawn smaller if it is wider than its row. A fit fills its row, so it is
/// only used where needed.
fn fit_if_narrow(width: f32, id: &'static str, control: impl IntoElement) -> AnyElement {
    if width >= FIT_WIDTH {
        control.into_any_element()
    } else {
        spec::fit_width(id, control).into_any_element()
    }
}

/// A tile's padding at a content `width`.
fn tile_padding(width: f32) -> f32 {
    if width >= 400. { 24. } else { 16. }
}

/// The library's shape count, as the install tile states it.
fn shapes_fact() -> SharedString {
    format!("{} expressive shapes, ready to morph", ShapeKind::ALL.len()).into()
}

/// How many component pages the overview lists.
pub(crate) fn component_count() -> usize {
    CATALOG
        .iter()
        .filter(|entry| super::shell::overview_entry(entry))
        .count()
}

fn scaled<E: Styled>(element: E, scale: TypeScale) -> E {
    element
        .font_family(scale.font_family)
        .text_size(scale.font_size)
        .line_height(scale.line_height)
        .font_weight(FontWeight(scale.font_weight as f32))
}

fn text(
    id: impl Into<SharedString>,
    value: impl Into<SharedString>,
    role: TypeRole,
) -> Stateful<Div> {
    accessible_text(ElementId::Name(id.into()), value)
        .whitespace_normal()
        .map(|text| type_role(text, role))
}

pub(crate) struct HomeView {
    gallery: WeakEntity<Gallery>,
    shared: Entity<GalleryShared>,
    /// Each hero shape's place in [`SHAPE_CYCLE`].
    pub(super) shapes: [usize; 4],
    wifi: bool,
    chips: [bool; 3],
    toggles: [bool; 3],
    volume: Entity<SliderState>,
    /// Whether the install line was copied in the last [`COPIED_FOR`].
    pub(super) copied: bool,
    /// Ends [`Self::copied`]; a new copy replaces, and so restarts, it.
    copied_reset: Option<Task<()>>,
    _observe: Subscription,
}

impl HomeView {
    pub(crate) fn new(
        gallery: WeakEntity<Gallery>,
        shared: Entity<GalleryShared>,
        cx: &mut Context<Self>,
    ) -> Self {
        // The selected swatch and mode follow the Gallery's theme, whoever
        // changes it.
        let observe = cx.observe(&shared, |_, _, cx| cx.notify());
        Self {
            gallery,
            shared,
            shapes: [0, 2, 4, 6],
            wifi: true,
            chips: [true, false, false],
            toggles: [true, false, false],
            volume: cx.new(|_| {
                SliderState::new()
                    .min(0.)
                    .max(100.)
                    .step(1.)
                    .default_value(60.)
            }),
            copied: false,
            copied_reset: None,
            _observe: observe,
        }
    }

    fn open(&self, id: &'static str, cx: &mut App) {
        let Some(index) = catalog::position(id) else {
            return;
        };
        let _ = self
            .gallery
            .update(cx, |gallery, cx| gallery.navigate(index, cx));
    }

    /// Scroll the overview to its first component group, and take keyboard
    /// focus there: the Gallery focuses the group's first card once its row
    /// is drawn, as Tab does into a row that was off-screen.
    fn browse(&self, cx: &mut App) {
        let _ = self.gallery.update(cx, |gallery, cx| {
            gallery.list.scroll_to(ListOffset {
                item_ix: 1,
                offset_in_item: px(0.),
            });
            gallery.pending_focus = Some((1, false));
            cx.notify();
        });
    }

    /// Copy the install line and say so for [`COPIED_FOR`]. The reset is one
    /// timer, not a frame loop: once it fires, nothing here draws again.
    fn copy_install(&mut self, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(INSTALL.to_string()));
        self.copied = true;
        self.copied_reset = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(COPIED_FOR).await;
            let _ = this.update(cx, |this, cx| {
                this.copied = false;
                this.copied_reset = None;
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn render_hero(&self, width: f32, cx: &mut Context<Self>) -> Div {
        let wide = width >= HERO_SIDE_BY_SIDE;
        let padding = if width >= 600. {
            48.
        } else if width >= 400. {
            24.
        } else {
            16.
        };
        // A phone's actions take M3's small buttons, so that two share a row.
        let action_size = if width >= 600. {
            ButtonSize::Medium
        } else {
            ButtonSize::Small
        };
        // The display type steps down with the room, so that its longest word
        // fits a phone at a large text scale.
        let title = if wide {
            TypeRole::DisplayLarge
        } else if width >= 480. {
            TypeRole::DisplayMedium
        } else if width >= 320. {
            TypeRole::DisplaySmall
        } else if width >= 220. {
            TypeRole::HeadlineMedium
        } else {
            TypeRole::HeadlineSmall
        };
        let headline = |line: &'static str, role: ColorRole| {
            div()
                .whitespace_normal()
                .map(|text| scaled(text, title.emphasized()))
                .text_color(color(role))
                .child(line)
        };
        let actions = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(rem(12.))
            .pt(rem(8.))
            .child(fit_if_narrow(
                width,
                "home-browse-fit",
                MaterialButton::new("home-browse")
                    .button_size(action_size)
                    .icon(symbol("arrow_downward", 24.))
                    .icon_position(ButtonIconPosition::Trailing)
                    .debug_selector(|| "home-browse".into())
                    .child("Browse components")
                    .on_click(cx.listener(|this, _, _, cx| this.browse(cx))),
            ))
            .child(fit_if_narrow(
                width,
                "home-styles-fit",
                MaterialButton::new("home-styles")
                    .variant(ButtonVariant::Tonal)
                    .button_size(action_size)
                    .debug_selector(|| "home-styles".into())
                    .child("Explore styles")
                    .on_click(cx.listener(|this, _, _, cx| this.open("color", cx))),
            ))
            .child(fit_if_narrow(
                width,
                "home-examples-fit",
                MaterialButton::new("home-examples")
                    .variant(ButtonVariant::Outlined)
                    .button_size(action_size)
                    .debug_selector(|| "home-examples".into())
                    .child("See examples")
                    .on_click(cx.listener(|this, _, _, cx| this.open("examples", cx))),
            ));
        let copy = gpui_base::v_flex()
            .flex_1()
            .min_w_0()
            .gap(rem(16.))
            .child(
                text(
                    "home-eyebrow",
                    "MATERIAL 3 EXPRESSIVE FOR GPUI",
                    TypeRole::LabelLarge,
                )
                .text_color(color(ColorRole::PRIMARY)),
            )
            .child(
                div()
                    .id("home-title")
                    .role(Role::Heading)
                    .aria_label("Expressive by design. Native by default.")
                    .child(headline("Expressive by design.", ColorRole::ON_SURFACE))
                    .child(headline("Native by default.", ColorRole::PRIMARY)),
            )
            .child(
                text(
                    "home-pitch",
                    "Material 3 components for Rust apps built on GPUI, drawn on the GPU \
                     on the desktop and in the browser. Pick one, tune it, and see every \
                     state.",
                    if wide {
                        TypeRole::TitleLarge
                    } else {
                        TypeRole::BodyLarge
                    },
                )
                .max_w(rem(620.))
                .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
            )
            .child(actions);
        // Side by side, a 2×2 cluster; stacked, one row that fits the card.
        let room = width - 2. * padding;
        let stacked_size = ((room - 3. * 12.) / 4.).clamp(8., HERO_SHAPE_STACKED);
        let shape = |slot: usize, size: f32| {
            let kind = SHAPE_CYCLE[(self.shapes[slot]) % SHAPE_CYCLE.len()];
            gpui_base::Button::new(("home-shape", slot))
                .accessibility_label(SharedString::from(format!("Morph shape {}", slot + 1)))
                .debug_selector(move || format!("home-shape-{slot}"))
                .rounded(rem(16.))
                .cursor_pointer()
                .child(
                    MaterialShape::new(("home-shape-fill", slot), kind)
                        .color(SHAPE_ROLES[slot])
                        .size(size)
                        .rotation(if slot % 2 == 1 { 15. } else { 0. })
                        .animated(true),
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.shapes[slot] = (this.shapes[slot] + 1) % SHAPE_CYCLE.len();
                    cx.notify();
                }))
        };
        let shapes = if wide {
            gpui_base::v_flex()
                .flex_shrink_0()
                .gap(rem(HERO_SHAPE_GAP))
                .child(
                    div()
                        .flex()
                        .gap(rem(HERO_SHAPE_GAP))
                        .child(shape(0, HERO_SHAPE))
                        .child(shape(1, HERO_SHAPE)),
                )
                .child(
                    div()
                        .flex()
                        .gap(rem(HERO_SHAPE_GAP))
                        .child(shape(2, HERO_SHAPE))
                        .child(shape(3, HERO_SHAPE)),
                )
        } else {
            div()
                .flex()
                .justify_center()
                .gap(rem(12.))
                .children((0..4).map(|slot| shape(slot, stacked_size)))
        };
        div()
            .debug_selector(|| "home-hero".into())
            .flex()
            .when(wide, |hero| hero.flex_row().items_center().gap(rem(48.)))
            .when(!wide, |hero| hero.flex_col().gap(rem(32.)))
            .p(rem(padding))
            .rounded(rem(HERO_RADIUS))
            .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
            .child(copy)
            .child(shapes)
    }

    fn tile(id: &'static str, title: &'static str, body: &'static str, width: f32) -> Div {
        gpui_base::v_flex()
            .debug_selector(move || format!("home-tile-{id}"))
            .flex_1()
            // The tiles wrap before one gets narrower than this, but none
            // asks for more than the content's width.
            .min_w(rem(TILE_MIN_WIDTH.min(width)))
            .gap(rem(16.))
            .p(rem(tile_padding(width)))
            .rounded(rem(TILE_RADIUS))
            .border_1()
            .border_color(color(ColorRole::OUTLINE_VARIANT))
            .bg(color(ColorRole::SURFACE_CONTAINER_LOWEST))
            .child(
                gpui_base::v_flex()
                    .gap(rem(4.))
                    .child(
                        text(format!("home-tile-title-{id}"), title, TypeRole::TitleLarge)
                            .role(Role::Heading)
                            .text_color(color(ColorRole::ON_SURFACE)),
                    )
                    .child(
                        text(format!("home-tile-body-{id}"), body, TypeRole::BodyMedium)
                            .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                    ),
            )
    }

    /// The theme swatches and the light/dark choice, for the whole Gallery.
    fn render_theme_tile(&self, width: f32, cx: &mut Context<Self>) -> Div {
        let shared = self.shared.read(cx);
        let mode = shared.painted_mode();
        let current = shared.reference_theme.filter(|_| shared.seed.is_none());
        let swatches = div().flex().flex_wrap().gap(rem(8.)).children(
            SWATCH_THEMES
                .into_iter()
                .enumerate()
                .map(|(index, (theme, name))| {
                    let selected = current == Some(theme);
                    let scope = gpui_kit_material::scope::Scope::new(("home-swatch-scope", index))
                        .scheme(swatch_scheme(index, mode))
                        .child(
                            div()
                                .relative()
                                .size(rem(SWATCH))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    // The chosen theme's swatch morphs to a
                                    // cookie and takes a check.
                                    MaterialShape::new(
                                        ("home-swatch", index),
                                        if selected {
                                            ShapeKind::Cookie9Sided
                                        } else {
                                            ShapeKind::Circle
                                        },
                                    )
                                    .color(ColorRole::PRIMARY)
                                    .size(SWATCH)
                                    .animated(true),
                                )
                                .when(selected, |swatch| {
                                    swatch.child(
                                        div()
                                            .absolute()
                                            .inset_0()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                symbol("check", 20.)
                                                    .color(color(ColorRole::ON_PRIMARY)),
                                            ),
                                    )
                                }),
                        );
                    let ring = color(ColorRole::SECONDARY);
                    gpui_base::Button::new(("home-swatch-button", index))
                        .accessibility_label(name)
                        .selected(selected)
                        .debug_selector(move || format!("home-swatch-{index}"))
                        .rounded_full()
                        .cursor_pointer()
                        // A constant ring that only focus colors, so focus
                        // moves nothing.
                        .border_2()
                        .border_color(gpui::transparent_black())
                        .focus_visible(move |style| style.border_color(ring))
                        .child(scope)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.shared.update(cx, |shared, cx| {
                                shared.seed = None;
                                shared.reference_theme = Some(theme);
                                cx.notify();
                            });
                        }))
                }),
        );
        let shared = self.shared.clone();
        Self::tile(
            "theme",
            "Make it yours",
            "Repaint the whole Gallery with a Material theme, in light or dark.",
            width,
        )
        .child(swatches)
        .child(fit_if_narrow(
            width,
            "home-mode-fit",
            ConnectedButtonGroup::new("home-mode", "Theme mode")
                .options([("light", "Light"), ("dark", "Dark")])
                .selected(if mode == ThemeMode::Dark {
                    "dark"
                } else {
                    "light"
                })
                .on_change(move |value: &SharedString, _, cx| {
                    top_bar::set_dark(&shared, value.as_ref() == "dark", cx)
                }),
        ))
    }

    /// A few controls to touch, each holding its state here.
    fn render_live_tile(&self, width: f32, cx: &mut Context<Self>) -> Div {
        let label = |id: &'static str, value: &'static str| {
            text(id, value, TypeRole::BodyLarge).text_color(color(ColorRole::ON_SURFACE))
        };
        Self::tile(
            "live",
            "Try it live",
            "Real components, not pictures. Press, drag and toggle them.",
            width,
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap(rem(16.))
                .child(label("home-wifi-label", "Wi-Fi"))
                .child(
                    MaterialSwitch::new("home-wifi", "Wi-Fi")
                        .checked(self.wifi)
                        .icons(SwitchIcons::Selected)
                        .on_change(cx.listener(|this, checked: &bool, _, cx| {
                            this.wifi = *checked;
                            cx.notify();
                        })),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(rem(8.))
                .child(symbol("volume_up", 24.).color(color(ColorRole::ON_SURFACE_VARIANT)))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(MaterialSlider::new(&self.volume, "Volume").id("home-volume")),
                ),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(rem(8.))
                .children(CHIPS.into_iter().enumerate().map(|(index, chip)| {
                    div()
                        .debug_selector(move || format!("home-chip-{index}"))
                        .child(
                            FilterChip::new(("home-chip", index), chip)
                                .selected(self.chips[index])
                                .on_toggle(cx.listener(move |this, selected: &bool, _, cx| {
                                    this.chips[index] = *selected;
                                    cx.notify();
                                })),
                        )
                })),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(rem(8.))
                .children(
                    TOGGLES
                        .into_iter()
                        .enumerate()
                        .map(|(index, (icon, name))| {
                            let view = cx.weak_entity();
                            div()
                                .debug_selector(move || format!("home-toggle-{index}"))
                                .child(
                                    IconButton::new(
                                        ("home-toggle", index),
                                        symbol(icon, 24.),
                                        name,
                                    )
                                    .variant(IconButtonVariant::Tonal)
                                    .toggle(true)
                                    .selected(self.toggles[index])
                                    .on_change(
                                        move |selected, _, _, cx| {
                                            let _ = view.update(cx, |this, cx| {
                                                this.toggles[index] = selected;
                                                cx.notify();
                                            });
                                        },
                                    ),
                                )
                        }),
                ),
        )
    }

    fn render_install_tile(&self, width: f32, cx: &mut Context<Self>) -> Div {
        let copied = self.copied;
        let fact = |index: usize, icon: &'static str, value: SharedString| {
            div()
                .flex()
                .items_center()
                .gap(rem(8.))
                .child(symbol(icon, 20.).color(color(ColorRole::PRIMARY)))
                .child(
                    text(format!("home-fact-{index}"), value, TypeRole::BodyMedium)
                        .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                )
        };
        Self::tile(
            "install",
            "Add it to your app",
            "One crate on top of GPUI Kit. Desktop and web share the same code.",
            width,
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(rem(4.))
                .pl(rem(16.))
                .pr(rem(4.))
                .py(rem(4.))
                .rounded(rem(12.))
                .bg(color(ColorRole::SURFACE_CONTAINER_HIGHEST))
                .child(
                    accessible_text(
                        "home-install",
                        // Where the crate's name cannot fit on a line, it
                        // breaks rather than reaching past the tile.
                        if width >= INSTALL_UNBROKEN {
                            INSTALL_SHOWN
                        } else {
                            INSTALL
                        },
                    )
                    .flex_1()
                    .min_w_0()
                    .whitespace_normal()
                    .font_family(CODE_FONT)
                    .text_size(rem(14.))
                    .line_height(rem(20.))
                    .text_color(color(ColorRole::ON_SURFACE)),
                )
                .child(
                    div().debug_selector(|| "home-install-copy".into()).child(
                        IconButton::new(
                            "home-install-copy",
                            symbol(if copied { "check" } else { "content_copy" }, 24.),
                            if copied { "Copied" } else { "Copy" },
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.copy_install(cx))),
                    ),
                ),
        )
        .child(
            gpui_base::v_flex()
                .gap(rem(8.))
                .child(fact(
                    0,
                    "desktop_windows",
                    "macOS, Windows and Linux".into(),
                ))
                .child(fact(1, "language", "The web, through WebAssembly".into()))
                .child(fact(2, "interests", shapes_fact())),
        )
    }

    /// The heading over the component groups.
    fn render_library_heading(&self) -> Div {
        gpui_base::v_flex()
            .gap(rem(4.))
            .pt(rem(40.))
            .pb(rem(24.))
            .child(
                text(
                    "home-library",
                    "Browse the library",
                    TypeRole::HeadlineMedium,
                )
                .role(Role::Heading)
                .debug_selector(|| "home-library".into())
                .text_color(color(ColorRole::ON_SURFACE)),
            )
            .child(
                text(
                    "home-library-body",
                    format!(
                        "{} components in {} groups. Every card opens a live playground.",
                        component_count(),
                        super::shell::overview_groups().count()
                    ),
                    TypeRole::BodyLarge,
                )
                .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
            )
    }
}

impl Render for HomeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let width = super::shell::page_content_width(window);
        gpui_base::v_flex()
            .debug_selector(|| "page-overview".into())
            .w_full()
            .gap(rem(TILE_GAP))
            .child(self.render_hero(width, cx))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(rem(TILE_GAP))
                    .child(self.render_theme_tile(width, cx))
                    .child(self.render_live_tile(width, cx))
                    .child(self.render_install_tile(width, cx)),
            )
            .child(self.render_library_heading())
    }
}
