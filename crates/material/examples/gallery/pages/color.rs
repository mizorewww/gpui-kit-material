//! Color, like md3.io's Color style page: the reference theme, mode,
//! contrast and palette that repaint the whole Gallery, then every color
//! role grouped by family, each swatch with its name, hex value and the
//! WCAG contrast of the text role painted on it. A seed color control
//! generates a theme from any color instead, following the same mode and
//! contrast controls, until a reference theme or palette is chosen again.
use super::prelude::*;
use gpui_base::input::{InputEvent, InputState};
use gpui_kit_material::color::dynamic::{SeedScheme, SpecVersion, Variant};
use gpui_kit_material::color::palette::ALL as PALETTES;
use gpui_kit_material::color::{Scheme, themes};
use gpui_kit_material::theme;

pub struct Page {
    page: PageCx,
    /// The seed as `0xRRGGBB`, kept while a reference theme is painted.
    pub(crate) seed: u32,
    pub(crate) variant: Variant,
    pub(crate) spec: SpecVersion,
    /// Whether the seed is what is painted.
    pub(crate) active: bool,
    pub(crate) input: Entity<InputState>,
    /// The hex field holds text that is not a color.
    pub(crate) invalid: bool,
    /// The mode and contrast the painted seed scheme was generated for.
    applied: (ThemeMode, Contrast),
    /// The choice a change of means someone picked another theme.
    reference: (Option<ThemeId>, PaletteId),
    /// A next-frame callback is pending for the theme transition.
    driving: bool,
    _subscriptions: [Subscription; 2],
}

const CONTROL_ROWS: usize = 5;
const SEED_ROW: usize = 4;
const DEFAULT_SEED: u32 = 0x6750a4;
/// Starting points; any color can be typed. The first is the M3 baseline seed.
const PRESETS: [(&str, u32); 8] = [
    ("Violet", DEFAULT_SEED),
    ("Red", 0xb3261e),
    ("Orange", 0xe8710a),
    ("Yellow", 0xf9ab00),
    ("Green", 0x386a20),
    ("Teal", 0x006a6a),
    ("Blue", 0x0b57d0),
    ("Gray", 0x76777a),
];
const VARIANTS: [(&str, &str, Variant); 9] = [
    ("tonal-spot", "Tonal spot", Variant::TonalSpot),
    ("neutral", "Neutral", Variant::Neutral),
    ("vibrant", "Vibrant", Variant::Vibrant),
    ("expressive", "Expressive", Variant::Expressive),
    ("rainbow", "Rainbow", Variant::Rainbow),
    ("fruit-salad", "Fruit salad", Variant::FruitSalad),
    ("monochrome", "Monochrome", Variant::Monochrome),
    ("fidelity", "Fidelity", Variant::Fidelity),
    ("content", "Content", Variant::Content),
];
const SPECS: [(&str, &str, SpecVersion); 2] = [
    ("2021", "2021 spec", SpecVersion::Spec2021),
    ("2025", "2025 spec", SpecVersion::Spec2025),
];
const SWATCH_MIN_WIDTH: f32 = 168.;
const SWATCH_GAP: f32 = 8.;

/// The role families, in md3.io's order. Every reference role appears once.
pub(crate) const GROUPS: [(&str, &[&str]); 8] = [
    (
        "Primary",
        &[
            "primary",
            "on-primary",
            "primary-container",
            "on-primary-container",
        ],
    ),
    (
        "Secondary",
        &[
            "secondary",
            "on-secondary",
            "secondary-container",
            "on-secondary-container",
        ],
    ),
    (
        "Tertiary",
        &[
            "tertiary",
            "on-tertiary",
            "tertiary-container",
            "on-tertiary-container",
        ],
    ),
    (
        "Error",
        &["error", "on-error", "error-container", "on-error-container"],
    ),
    (
        "Fixed",
        &[
            "primary-fixed",
            "primary-fixed-dim",
            "on-primary-fixed",
            "on-primary-fixed-variant",
            "secondary-fixed",
            "secondary-fixed-dim",
            "on-secondary-fixed",
            "on-secondary-fixed-variant",
            "tertiary-fixed",
            "tertiary-fixed-dim",
            "on-tertiary-fixed",
            "on-tertiary-fixed-variant",
        ],
    ),
    (
        "Surface",
        &[
            "surface-dim",
            "surface",
            "surface-bright",
            "surface-container-lowest",
            "surface-container-low",
            "surface-container",
            "surface-container-high",
            "surface-container-highest",
            "surface-variant",
            "on-surface",
            "on-surface-variant",
        ],
    ),
    (
        "Inverse, outline and scrim",
        &[
            "inverse-surface",
            "inverse-on-surface",
            "inverse-primary",
            "outline",
            "outline-variant",
            "shadow",
            "scrim",
        ],
    ),
    (
        "Status",
        &[
            "success",
            "on-success",
            "warning",
            "on-warning",
            "info",
            "on-info",
        ],
    ),
];

pub(crate) fn role(name: &str) -> ColorRole {
    let index = themes::REFERENCE_ROLE_NAMES
        .iter()
        .position(|candidate| *candidate == name)
        .unwrap_or_else(|| panic!("{name} is a reference role"));
    themes::REFERENCE_ROLES[index]
}

fn exists(name: &str) -> bool {
    themes::REFERENCE_ROLE_NAMES.contains(&name)
}

/// The role whose text a swatch of `name` carries: its "on" partner, or
/// for surfaces and lines the text that sits on them.
pub(crate) fn text_role(name: &str) -> String {
    let partner = match name {
        "inverse-surface" | "inverse-primary" => "inverse-on-surface".to_owned(),
        "inverse-on-surface" => "inverse-surface".to_owned(),
        "outline" | "outline-variant" | "shadow" | "scrim" => "surface".to_owned(),
        _ if name.ends_with("-fixed-dim") => {
            format!("on-{}", name.trim_end_matches("-dim"))
        }
        _ if name.starts_with("on-") => {
            let base = name.trim_start_matches("on-").trim_end_matches("-variant");
            let full = name.trim_start_matches("on-");
            if exists(full) { full } else { base }.to_owned()
        }
        "surface-variant" => "on-surface-variant".to_owned(),
        _ if name.starts_with("surface") => "on-surface".to_owned(),
        _ => format!("on-{name}"),
    };
    assert!(exists(&partner), "{name} pairs with {partner}");
    partner
}

/// `#RRGGBB` of a painted color.
pub(crate) fn hex(color: Rgba) -> String {
    let channel = |c: f32| (c.clamp(0., 1.) * 255.).round() as u8;
    format!(
        "#{:02X}{:02X}{:02X}",
        channel(color.r),
        channel(color.g),
        channel(color.b)
    )
}

/// WCAG 2.x contrast ratio of two opaque colors.
pub(crate) fn contrast(a: Rgba, b: Rgba) -> f32 {
    fn luminance(color: Rgba) -> f32 {
        let linear = |c: f32| {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
    }
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// The WCAG level a ratio passes for text.
pub(crate) fn grade(ratio: f32) -> &'static str {
    match ratio {
        r if r >= 7. => "AAA",
        r if r >= 4.5 => "AA",
        r if r >= 3. => "AA large",
        _ => "Fails",
    }
}

/// Swatches per row at the window's current width.
fn columns(window: &Window) -> usize {
    ((spec::content_width(window) + SWATCH_GAP) / (SWATCH_MIN_WIDTH + SWATCH_GAP))
        .floor()
        .max(1.) as usize
}

fn swatch(name: &'static str) -> impl IntoElement {
    let fill = color(role(name));
    let ink_role = text_role(name);
    let ink = color(role(&ink_role));
    let ratio = contrast(fill, ink);
    let text = |text: String, type_role: TypeRole| {
        div()
            .map(|el| spec::type_role(el, type_role))
            .text_color(ink)
            .whitespace_nowrap()
            .overflow_hidden()
            .text_ellipsis()
            .child(text)
    };
    div()
        .id(name)
        .flex_1()
        .min_w_0()
        .h(rem(96.))
        .p(rem(12.))
        .flex()
        .flex_col()
        .justify_between()
        .rounded(rem(12.))
        .border_1()
        .border_color(color(ColorRole::OUTLINE_VARIANT))
        .bg(fill)
        .child(text(name.to_owned(), TypeRole::LabelLarge))
        .child(
            div()
                .flex()
                .justify_between()
                .gap(rem(8.))
                .child(text(hex(fill), TypeRole::BodySmall))
                .child(text(
                    format!("{ratio:.1}:1 {}", grade(ratio)),
                    TypeRole::LabelSmall,
                )),
        )
}

/// The contrast level of a contrast option (upstream's standard, medium, high).
pub(crate) fn contrast_level(contrast: Contrast) -> f64 {
    match contrast {
        Contrast::Standard => 0.,
        Contrast::Medium => 0.5,
        Contrast::High => 1.,
    }
}

fn hex_text(seed: u32) -> SharedString {
    format!("#{seed:06X}").into()
}

impl Page {
    fn seed_scheme(&self, mode: ThemeMode, contrast: Contrast) -> Scheme {
        SeedScheme::new(self.seed)
            .variant(self.variant)
            .spec(self.spec)
            .dark(mode == ThemeMode::Dark)
            .contrast(contrast_level(contrast))
            .scheme()
    }

    /// Paint the seed, regenerating it for the current mode and contrast.
    pub(crate) fn apply_seed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active = true;
        // The mode and contrast controls pick the baseline when no reference
        // theme is set; settling that now keeps it from reading as a new choice.
        self.page.shared.update(cx, |shared, cx| {
            if shared.reference_theme.is_none() {
                shared.reference_theme = Some(ThemeId::Baseline);
                cx.notify();
            }
        });
        let shared = self.page.shared.read(cx);
        self.applied = (shared.reference_mode, shared.contrast);
        self.reference = (shared.reference_theme, shared.palette);
        let scheme = self.seed_scheme(self.applied.0, self.applied.1);
        theme::set_scheme_override(Some(scheme));
        self.drive_theme(window, cx);
        cx.notify();
    }

    /// Paint the Gallery's reference theme or palette again.
    pub(crate) fn clear_seed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active = false;
        theme::set_scheme_override(None);
        self.drive_theme(window, cx);
        cx.notify();
    }

    /// Advance the transition. The Gallery only advances when one of its own
    /// settings changes, and a seed is not one.
    fn drive_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let shared = self.page.shared.read(cx);
        let (target, scale) = (shared.target_scheme(), shared.text_scale());
        let running = theme::advance_component_theme(target, scale, window, cx);
        if running && !self.driving {
            self.driving = true;
            let page = cx.weak_entity();
            window.on_next_frame(move |window, cx| {
                let _ = page.update(cx, |this, cx| {
                    this.driving = false;
                    this.drive_theme(window, cx);
                });
            });
        }
    }

    pub(crate) fn choose_preset(&mut self, seed: u32, window: &mut Window, cx: &mut Context<Self>) {
        self.seed = seed;
        self.invalid = false;
        // `set_value` emits no change event, so this does not apply twice.
        self.input
            .update(cx, |input, cx| input.set_value(hex_text(seed), window, cx));
        self.apply_seed(window, cx);
    }

    fn hex_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.input.read(cx).value();
        match SeedScheme::parse_hex(&text) {
            Some(seed) => {
                self.invalid = false;
                if seed != self.seed || !self.active {
                    self.seed = seed;
                    self.apply_seed(window, cx);
                } else {
                    cx.notify();
                }
            }
            None => {
                self.invalid = !text.trim().is_empty();
                cx.notify();
            }
        }
    }

    /// Follow the Gallery's other theme controls while a seed is painted.
    fn shared_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let shared = self.page.shared.read(cx);
        let reference = (shared.reference_theme, shared.palette);
        let settings = (shared.reference_mode, shared.contrast);
        if !self.active {
            self.reference = reference;
        } else if reference != self.reference {
            self.clear_seed(window, cx);
        } else if settings != self.applied {
            self.apply_seed(window, cx);
        }
    }

    fn render_seed(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let list = self.page.rows.list().clone();
        let selected_variant = VARIANTS
            .iter()
            .find(|(_, _, variant)| *variant == self.variant)
            .map_or("", |(id, _, _)| *id);
        let selected_spec = SPECS
            .iter()
            .find(|(_, _, spec)| *spec == self.spec)
            .map_or("", |(id, _, _)| *id);
        let field = TextField::new("seed-hex", &self.input)
            .label("Hex")
            .placeholder("#6750A4")
            .error(self.invalid)
            .supporting_text(if self.invalid {
                "Enter 3 or 6 hex digits"
            } else {
                "Any color"
            })
            .w(rem(180.));
        let presets =
            div()
                .flex()
                .flex_wrap()
                .gap(rem(8.))
                .children(
                    PRESETS
                        .into_iter()
                        .enumerate()
                        .map(|(index, (name, seed))| {
                            let selected = self.active && self.seed == seed;
                            gpui_base::Button::new(("seed-preset", index))
                                .accessibility_label(name)
                                .selected(selected)
                                .debug_selector(move || format!("seed-preset-{index}"))
                                .size(rem(40.))
                                .rounded_full()
                                .cursor_pointer()
                                .bg(rgb(seed))
                                .when(selected, |swatch| {
                                    swatch.border_2().border_color(color(ColorRole::ON_SURFACE))
                                })
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.choose_preset(seed, window, cx)
                                }))
                                .reveal(("reveal-seed-preset", index), &list)
                        }),
                );
        let spec_note =
            (self.spec == SpecVersion::Spec2025 && !self.variant.has_2025_spec()).then(|| {
                spec::small_label(
                    "seed-spec-note",
                    "This variant has no 2025 spec, so it resolves as 2021.",
                )
            });
        spec::demo(
            "seed-demo",
            "Seed color",
            gpui_base::v_flex()
                .debug_selector(|| "seed-control".into())
                .gap(rem(12.))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap(rem(16.))
                        .child(field.reveal("reveal-seed-hex", &list))
                        .child(presets),
                )
                .child(
                    ConnectedButtonGroup::new("seed-variant", "Seed scheme variant")
                        .options(VARIANTS.map(|(id, label, _)| (id, label)))
                        .selected(if self.active { selected_variant } else { "" })
                        .full_width()
                        .on_change(cx.listener(|this, id: &SharedString, window, cx| {
                            if let Some((_, _, variant)) =
                                VARIANTS.iter().find(|(key, _, _)| *key == id.as_ref())
                            {
                                this.variant = *variant;
                                this.apply_seed(window, cx);
                            }
                        }))
                        .reveal("reveal-seed-variant", &list),
                )
                .child(
                    ConnectedButtonGroup::new("seed-spec", "Seed color spec")
                        .options(SPECS.map(|(id, label, _)| (id, label)))
                        .selected(selected_spec)
                        .on_change(cx.listener(|this, id: &SharedString, window, cx| {
                            if let Some((_, _, spec)) =
                                SPECS.iter().find(|(key, _, _)| *key == id.as_ref())
                            {
                                this.spec = *spec;
                                this.apply_seed(window, cx);
                            }
                        }))
                        .reveal("reveal-seed-spec", &list),
                )
                .children(spec_note)
                .child(
                    div().flex().child(
                        material_button("seed-clear", ButtonVariant::Outlined)
                            .child("Use reference theme")
                            .disabled(!self.active)
                            .on_click(
                                cx.listener(|this, _, window, cx| this.clear_seed(window, cx)),
                            )
                            .reveal("reveal-seed-clear", &list),
                    ),
                ),
        )
        .into_any_element()
    }
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "color",
        title: "Color",
        description: "Every color role in each theme, light and dark, with contrast checks. Pick a theme, mode, contrast or palette, or generate a theme from a seed color, to repaint the whole Gallery.",
        guideline: Some("https://m3.material.io/styles/color/overview"),
    };

    fn new(page: PageCx, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("#6750A4")
                .default_value(hex_text(DEFAULT_SEED))
        });
        let subscriptions = [
            cx.subscribe_in(&input, window, |this, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::Change) {
                    this.hex_changed(window, cx);
                }
            }),
            cx.observe_in(&page.shared, window, |this, _, window, cx| {
                this.shared_changed(window, cx);
            }),
        ];
        let shared = page.shared.read(cx);
        let applied = (shared.reference_mode, shared.contrast);
        let reference = (shared.reference_theme, shared.palette);
        Self {
            page,
            seed: DEFAULT_SEED,
            variant: Variant::TonalSpot,
            spec: SpecVersion::Spec2021,
            active: false,
            input,
            invalid: false,
            applied,
            reference,
            driving: false,
            _subscriptions: subscriptions,
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        CONTROL_ROWS + GROUPS.len()
    }

    fn row_gap(&self, row: usize) -> f32 {
        match row {
            0 => spec::HEADER_TO_BODY,
            1..CONTROL_ROWS => spec::ROW_GAP,
            CONTROL_ROWS => 40.,
            _ => 32.,
        }
    }

    fn interactive(&self, row: usize) -> bool {
        row < CONTROL_ROWS
    }

    fn render_row(
        &mut self,
        row: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if row == SEED_ROW {
            return self.render_seed(cx);
        }
        let list = self.page.rows.list().clone();
        let seeded = self.active;
        let shared = self.page.shared.read(cx);
        let (theme, mode, contrast, palette) = (
            shared.reference_theme,
            shared.reference_mode,
            shared.contrast,
            shared.palette,
        );
        let update = |f: fn(&mut GalleryShared, &str)| {
            let shared = self.page.shared.clone();
            move |value: &SharedString, _: &mut Window, cx: &mut App| {
                shared.update(cx, |shared, cx| {
                    f(shared, value.as_ref());
                    cx.notify();
                })
            }
        };
        // Picking a theme or palette ends the seed even if it is already the
        // Gallery's choice, when `shared_changed` would see no change.
        let pick = |f: fn(&mut GalleryShared, &str)| {
            let update = update(f);
            let page = cx.weak_entity();
            move |value: &SharedString, window: &mut Window, cx: &mut App| {
                let _ = page.update(cx, |this, cx| {
                    if this.active {
                        this.clear_seed(window, cx);
                    }
                });
                update(value, window, cx);
            }
        };
        match row {
            0 => spec::demo(
                "reference-themes-demo",
                "Theme",
                ConnectedButtonGroup::new("reference-themes", "Material themes")
                    .options(themes::ALL.map(|id| (id.name(), id.name())))
                    .selected(if seeded {
                        ""
                    } else {
                        theme.map(ThemeId::name).unwrap_or("")
                    })
                    .full_width()
                    .on_change(pick(|shared, name| {
                        shared.reference_theme =
                            themes::ALL.into_iter().find(|id| id.name() == name);
                    }))
                    .reveal("reveal-reference-themes", &list),
            )
            .into_any_element(),
            1 => spec::demo(
                "mode-demo",
                "Mode",
                ConnectedButtonGroup::new("reference-mode", "Material theme mode")
                    .options([("light", "Light"), ("dark", "Dark")])
                    .selected(if mode == ThemeMode::Dark {
                        "dark"
                    } else {
                        "light"
                    })
                    .on_change(update(|shared, name| {
                        shared.reference_mode = if name == "dark" {
                            ThemeMode::Dark
                        } else {
                            ThemeMode::Light
                        };
                        shared.reference_theme.get_or_insert(ThemeId::Baseline);
                    }))
                    .reveal("reveal-mode", &list),
            )
            .into_any_element(),
            2 => spec::demo(
                "contrast-demo",
                "Contrast",
                ConnectedButtonGroup::new("reference-contrast", "Material contrast")
                    .options([
                        ("standard", "Standard contrast"),
                        ("medium", "Medium contrast"),
                        ("high", "High contrast"),
                    ])
                    .selected(match contrast {
                        Contrast::Standard => "standard",
                        Contrast::Medium => "medium",
                        Contrast::High => "high",
                    })
                    .on_change(update(|shared, name| {
                        shared.contrast = match name {
                            "high" => Contrast::High,
                            "medium" => Contrast::Medium,
                            _ => Contrast::Standard,
                        };
                        shared.reference_theme.get_or_insert(ThemeId::Baseline);
                    }))
                    .reveal("reveal-contrast", &list),
            )
            .into_any_element(),
            3 => spec::demo(
                "palette-demo",
                "Palettes",
                ConnectedButtonGroup::new("palette", "Palette")
                    .options(PALETTES.map(|id| (id.name(), id.name())))
                    .selected(if theme.is_none() && !seeded {
                        palette.name()
                    } else {
                        ""
                    })
                    .full_width()
                    .on_change(pick(|shared, name| {
                        if let Some(id) = PALETTES.into_iter().find(|id| id.name() == name) {
                            shared.palette = id;
                            shared.reference_theme = None;
                        }
                    }))
                    .reveal("reveal-palettes", &list),
            )
            .into_any_element(),
            _ => {
                let group = row - CONTROL_ROWS;
                let (title, names) = GROUPS[group];
                let columns = columns(window);
                gpui_base::v_flex()
                    .when(group == 0, |row| {
                        row.debug_selector(|| "color-swatches".into())
                    })
                    .gap(rem(SWATCH_GAP))
                    .child(
                        accessible_text(("color-group", group), title)
                            .role(Role::Heading)
                            .pb(rem(4.))
                            .map(|text| spec::type_role(text, TypeRole::TitleMedium))
                            .text_color(color(ColorRole::ON_SURFACE)),
                    )
                    .children(names.chunks(columns).map(|chunk| {
                        div()
                            .flex()
                            .gap(rem(SWATCH_GAP))
                            .children(chunk.iter().map(|name| swatch(name)))
                            .children((chunk.len()..columns).map(|_| div().flex_1().min_w_0()))
                    }))
                    .into_any_element()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui_kit_material::color::Color;

    /// A page in a window, as the Gallery builds it, with motion off so a theme
    /// change is painted at once.
    fn open(
        cx: &mut gpui::TestAppContext,
    ) -> (Entity<Page>, Entity<GalleryShared>, VisualTestContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
            cx.set_reduce_motion(true);
        });
        let shared = cx.new(|_| GalleryShared::default());
        let page_shared = shared.clone();
        let (page, cx) = cx.add_window_view(move |window, cx| {
            let list = ListState::new(0, ListAlignment::Top, px(100.));
            Page::new(
                PageCx {
                    shared: page_shared,
                    rows: super::super::super::page::PageRows::new(list),
                },
                window,
                cx,
            )
        });
        (page, shared, cx.clone())
    }

    fn painted(role: ColorRole) -> Color {
        theme::scheme().get(role)
    }

    fn primary_of(seed: u32, dark: bool, contrast: f64) -> Color {
        SeedScheme::new(seed)
            .dark(dark)
            .contrast(contrast)
            .scheme()
            .get(ColorRole::PRIMARY)
    }

    impl Render for Page {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.render_seed(cx)
        }
    }

    #[gpui::test]
    fn a_preset_paints_its_scheme_and_follows_mode_and_contrast(cx: &mut gpui::TestAppContext) {
        let (page, shared, mut cx) = open(cx);
        cx.update(|window, cx| {
            page.update(cx, |page, cx| page.choose_preset(0xb3261e, window, cx));
        });
        assert_eq!(painted(ColorRole::PRIMARY), primary_of(0xb3261e, false, 0.));
        cx.update(|_, cx| {
            assert_eq!(page.read(cx).input.read(cx).value().as_ref(), "#B3261E");
            shared.update(cx, |shared, cx| {
                shared.reference_mode = ThemeMode::Dark;
                shared.contrast = Contrast::High;
                cx.notify();
            });
        });
        cx.run_until_parked();
        assert_eq!(painted(ColorRole::PRIMARY), primary_of(0xb3261e, true, 1.));
        assert!(theme::is_dark());
    }

    #[gpui::test]
    fn the_hex_field_applies_valid_colors_and_flags_others(cx: &mut gpui::TestAppContext) {
        let (page, _, mut cx) = open(cx);
        let type_text = |text: &'static str, cx: &mut VisualTestContext| {
            cx.update(|window, cx| {
                let input = page.read(cx).input.clone();
                input.update(cx, |input, cx| {
                    input.set_value(text, window, cx);
                    cx.emit(InputEvent::Change);
                });
            });
            cx.run_until_parked();
        };
        type_text("#12", &mut cx);
        cx.update(|_, cx| {
            let page = page.read(cx);
            assert!(page.invalid && !page.active, "partial text is not applied");
        });
        type_text("#0b57d0", &mut cx);
        cx.update(|_, cx| {
            let page = page.read(cx);
            assert!(!page.invalid && page.active);
            assert_eq!(page.seed, 0x0b57d0);
        });
        assert_eq!(painted(ColorRole::PRIMARY), primary_of(0x0b57d0, false, 0.));
        // Clearing the field while typing is neither an error nor a new theme.
        type_text("", &mut cx);
        cx.update(|_, cx| {
            let page = page.read(cx);
            assert!(!page.invalid && page.active);
        });
    }

    #[gpui::test]
    fn variant_and_spec_regenerate_and_reference_themes_take_back_over(
        cx: &mut gpui::TestAppContext,
    ) {
        let (page, shared, mut cx) = open(cx);
        cx.update(|window, cx| {
            page.update(cx, |page, cx| {
                page.variant = Variant::Vibrant;
                page.spec = SpecVersion::Spec2025;
                page.apply_seed(window, cx);
            });
        });
        let expected = SeedScheme::new(DEFAULT_SEED)
            .variant(Variant::Vibrant)
            .spec(SpecVersion::Spec2025)
            .scheme();
        assert_eq!(theme::scheme(), expected);

        // Choosing another theme anywhere in the Gallery ends the seed.
        cx.update(|_, cx| {
            shared.update(cx, |shared, cx| {
                shared.reference_theme = Some(ThemeId::Forest);
                cx.notify();
            });
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert!(!page.read(cx).active));
        let forest = ThemeId::Forest.scheme(ThemeMode::Light, Contrast::Standard);
        assert_eq!(theme::scheme(), forest);

        // So does the page's own button, with nothing else to trigger a repaint.
        cx.update(|window, cx| {
            page.update(cx, |page, cx| page.apply_seed(window, cx));
        });
        assert_eq!(theme::scheme(), expected);
        cx.update(|window, cx| {
            page.update(cx, |page, cx| page.clear_seed(window, cx));
        });
        assert_eq!(theme::scheme(), forest);
    }

    #[gpui::test]
    fn the_seed_controls_render(cx: &mut gpui::TestAppContext) {
        let (_, _, mut cx) = open(cx);
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert!(cx.debug_bounds("seed-control").is_some());
        for index in 0..PRESETS.len() {
            let name = Box::leak(format!("seed-preset-{index}").into_boxed_str());
            assert!(cx.debug_bounds(name).is_some(), "preset {index}");
        }
    }

    #[test]
    fn contrast_options_are_upstreams_levels() {
        assert_eq!(contrast_level(Contrast::Standard), 0.);
        assert_eq!(contrast_level(Contrast::Medium), 0.5);
        assert_eq!(contrast_level(Contrast::High), 1.);
    }

    #[test]
    fn groups_list_every_reference_role_once() {
        let mut names: Vec<&str> = GROUPS
            .iter()
            .flat_map(|(_, names)| names.iter().copied())
            .collect();
        assert_eq!(names.len(), themes::REFERENCE_ROLE_NAMES.len());
        names.sort();
        let mut reference = themes::REFERENCE_ROLE_NAMES.to_vec();
        reference.sort();
        assert_eq!(names, reference);
    }

    #[test]
    fn every_swatch_carries_its_text_role() {
        assert_eq!(text_role("primary"), "on-primary");
        assert_eq!(text_role("on-primary-container"), "primary-container");
        assert_eq!(text_role("on-surface-variant"), "surface-variant");
        assert_eq!(text_role("on-tertiary-fixed-variant"), "tertiary-fixed");
        assert_eq!(text_role("primary-fixed-dim"), "on-primary-fixed");
        assert_eq!(text_role("surface-container-high"), "on-surface");
        assert_eq!(text_role("inverse-primary"), "inverse-on-surface");
        for name in themes::REFERENCE_ROLE_NAMES {
            text_role(name);
        }
    }

    #[test]
    fn hex_and_contrast_follow_wcag() {
        let black = Rgba {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        };
        let white = Rgba {
            r: 1.,
            g: 1.,
            b: 1.,
            a: 1.,
        };
        assert_eq!(hex(white), "#FFFFFF");
        assert_eq!(
            hex(Rgba {
                r: 0.4,
                g: 0.314,
                b: 0.643,
                a: 1.
            }),
            "#6650A4"
        );
        assert!((contrast(black, white) - 21.).abs() < 0.01);
        assert_eq!(grade(21.), "AAA");
        assert_eq!(grade(4.6), "AA");
        assert_eq!(grade(3.2), "AA large");
        assert_eq!(grade(1.5), "Fails");
    }
}
