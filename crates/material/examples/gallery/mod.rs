//! Every extracted component on one page, in every built-in palette.
//!
//! `cargo run -p gpui-kit-material --example gallery`
#[cfg(not(target_family = "wasm"))]
pub mod capture;

use gpui::{prelude::*, *};
use gpui_kit_material::color::palette::ALL as PALETTES;
use gpui_kit_material::color::themes::{self, Contrast, ThemeId, ThemeMode};
use gpui_kit_material::icons::{Assets, IconName};
use gpui_kit_material::motion;
use gpui_kit_material::prelude::*;
use gpui_kit_material::theme;

// Reveal only the newly focused control, not the entire long gallery page.
trait GalleryReveal: IntoElement + Sized {
    fn reveal(self, id: impl Into<ElementId>, scroll: &ScrollHandle) -> RevealFocus {
        RevealFocus::new(id, self, scroll.clone()).inline()
    }
}
impl<T: IntoElement> GalleryReveal for T {}

const GALLERY_SECTIONS: [(&str, &str); 10] = [
    ("themes", "Themes"),
    ("motion", "Motion"),
    ("buttons", "Buttons"),
    ("toggles", "Toggles"),
    ("loading", "Loading"),
    ("icons", "Icons"),
    ("labels", "Labels"),
    ("feedback", "Feedback"),
    ("colors", "Colors"),
    ("typography", "Typography"),
];

struct Gallery {
    palette: PaletteId,
    reference_theme: Option<ThemeId>,
    reference_mode: ThemeMode,
    contrast: Contrast,
    scale: SharedString,
    density: SharedString,
    engine: usize,
    details: bool,
    scroll: ScrollHandle,
    navigation_scroll: ScrollHandle,
    section_anchors: [ScrollAnchor; 10],
    selected_section: usize,
    search: Entity<gpui_base::input::InputState>,
    slider: Entity<gpui_base::slider::SliderState>,
    switch: bool,
    sheet: bool,
    motion_scheme: motion::MotionScheme,
    motion_target: bool,
    button_effects_disabled: bool,
    button_effects_tonal: bool,
    button_ripple: bool,
    button_active: bool,
    button_selections: [bool; 10],
    button_toggle_size: ButtonSize,
    button_loading: bool,
    button_progress: bool,
    button_loading_variant: usize,
    button_loading_clicks: usize,
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        motion::apply_scheme(self.motion_scheme);
        if let Some(theme_id) = self.reference_theme {
            theme::apply_scheme(
                theme_id.scheme(self.reference_mode, self.contrast),
                window,
                cx,
            );
        } else {
            theme::apply_palette(self.palette, window, cx);
        }
        theme::apply_scale(self.scale.parse().unwrap_or(1.), window);
        let compact = is_compact(window);
        let scroll = &self.scroll;
        let palettes = ConnectedButtonGroup::new("palette", "Palette")
            .reveal_in(scroll.clone())
            .options(PALETTES.map(|id| (id.name(), id.name())))
            .selected(if self.reference_theme.is_none() {
                self.palette.name()
            } else {
                ""
            })
            .full_width()
            .on_change(cx.listener(|this, name: &SharedString, _, cx| {
                if let Some(id) = PALETTES.into_iter().find(|id| id.name() == name.as_ref()) {
                    this.palette = id;
                    this.reference_theme = None;
                    cx.notify();
                }
            }));
        let reference_themes = ConnectedButtonGroup::new("reference-themes", "Material themes")
            .reveal_in(scroll.clone())
            .options(themes::ALL.map(|id| (id.name(), id.name())))
            .selected(self.reference_theme.map(ThemeId::name).unwrap_or(""))
            .full_width()
            .on_change(cx.listener(|this, name: &SharedString, _, cx| {
                this.reference_theme = themes::ALL
                    .into_iter()
                    .find(|id| id.name() == name.as_ref());
                cx.notify();
            }));
        let mode = ConnectedButtonGroup::new("reference-mode", "Material theme mode")
            .reveal_in(scroll.clone())
            .options([("light", "Light"), ("dark", "Dark")])
            .selected(if self.reference_mode == ThemeMode::Dark {
                "dark"
            } else {
                "light"
            })
            .on_change(cx.listener(|this, name: &SharedString, _, cx| {
                this.reference_mode = if name == "dark" {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                };
                this.reference_theme.get_or_insert(ThemeId::Baseline);
                cx.notify();
            }));
        let contrast = ConnectedButtonGroup::new("reference-contrast", "Material contrast")
            .reveal_in(scroll.clone())
            .options([
                ("standard", "Standard contrast"),
                ("medium", "Medium contrast"),
                ("high", "High contrast"),
            ])
            .selected(match self.contrast {
                Contrast::Standard => "standard",
                Contrast::Medium => "medium",
                Contrast::High => "high",
            })
            .on_change(cx.listener(|this, name: &SharedString, _, cx| {
                this.contrast = match name.as_ref() {
                    "high" => Contrast::High,
                    "medium" => Contrast::Medium,
                    _ => Contrast::Standard,
                };
                this.reference_theme.get_or_insert(ThemeId::Baseline);
                cx.notify();
            }));
        let color_swatches = themes::REFERENCE_ROLES
            .into_iter()
            .zip(themes::REFERENCE_ROLE_NAMES)
            .map(|(role, name)| {
                gpui_base::v_flex()
                    .w(rem(166.))
                    .gap(rem(4.))
                    .child(div().h(rem(32.)).w_full().rounded(rem(8.)).bg(color(role)))
                    .child(
                        accessible_text(SharedString::from(format!("color-{name}")), name)
                            .text_size(TEXT_AUX),
                    )
            });
        let button_effects = gpui_base::v_flex()
            .gap(rem(12.))
            .child(section_title("button-effects", "Button transitions"))
            .child(
                gpui_base::h_flex()
                    .flex_wrap()
                    .gap(rem(12.))
                    .child(
                        material_button("toggle-button-active", ButtonVariant::Outlined)
                            .child(if self.button_active {
                                "Release active samples"
                            } else {
                                "Keep samples active"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.button_active = !this.button_active;
                                cx.notify();
                            }))
                            .reveal("reveal-toggle-button-active", scroll),
                    )
                    .child(
                        material_button("toggle-button-disabled", ButtonVariant::Outlined)
                            .child(if self.button_effects_disabled {
                                "Enable samples"
                            } else {
                                "Disable samples"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.button_effects_disabled = !this.button_effects_disabled;
                                cx.notify();
                            }))
                            .reveal("reveal-toggle-button-disabled", scroll),
                    )
                    .child(
                        material_button("toggle-button-variant", ButtonVariant::Outlined)
                            .child("Switch filled / tonal")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.button_effects_tonal = !this.button_effects_tonal;
                                cx.notify();
                            }))
                            .reveal("reveal-toggle-button-variant", scroll),
                    )
                    .child(
                        material_button("toggle-button-ripple", ButtonVariant::Outlined)
                            .child(if self.button_ripple {
                                "Turn ripple off"
                            } else {
                                "Turn ripple on"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.button_ripple = !this.button_ripple;
                                cx.notify();
                            }))
                            .reveal("reveal-toggle-button-ripple", scroll),
                    ),
            )
            .child(gpui_base::h_flex().flex_wrap().gap(rem(12.)).children(
                ButtonVariant::ALL.into_iter().map(|variant| {
                    material_button(
                        SharedString::from(format!("effect-sample-{variant:?}")),
                        if variant == ButtonVariant::Filled && self.button_effects_tonal {
                            ButtonVariant::Tonal
                        } else {
                            variant
                        },
                    )
                    .disabled(self.button_effects_disabled)
                    .active_state(self.button_active)
                    .ripple(self.button_ripple)
                    .icon(IconName::Check)
                    .child(variant.name())
                    .reveal(
                        SharedString::from(format!("reveal-effect-sample-{variant:?}")),
                        scroll,
                    )
                }),
            ));
        let button_toggles = gpui_base::v_flex()
            .id("gallery-section-toggles")
            .anchor_scroll(Some(self.section_anchors[3].clone()))
            .gap(rem(16.))
            .child(section_title("button-toggles", "Toggle buttons"))
            .child(
                ConnectedButtonGroup::new("toggle-size", "Toggle button size")
                    .reveal_in(scroll.clone())
                    .options(ButtonSize::ALL.map(|size| (size.name(), size.name())))
                    .selected(self.button_toggle_size.name())
                    .on_change(cx.listener(|this, name: &SharedString, _, cx| {
                        if let Some(size) = ButtonSize::ALL
                            .into_iter()
                            .find(|size| size.name() == name.as_ref())
                        {
                            this.button_toggle_size = size;
                            cx.notify();
                        }
                    })),
            )
            .children(
                ButtonShape::ALL
                    .into_iter()
                    .enumerate()
                    .map(|(shape_index, shape)| {
                        gpui_base::v_flex()
                            .gap(rem(12.))
                            .child(accessible_text(
                                SharedString::from(format!("toggle-shape-{shape:?}")),
                                shape.name(),
                            ))
                            .child(gpui_base::h_flex().flex_wrap().gap(rem(16.)).children(
                                ButtonVariant::ALL.into_iter().enumerate().map(
                                    |(variant_index, variant)| {
                                        let index = shape_index * 5 + variant_index;
                                        MaterialButton::new(SharedString::from(format!(
                                            "toggle-{shape:?}-{variant:?}"
                                        )))
                                        .variant(variant)
                                        .shape(shape)
                                        .button_size(self.button_toggle_size)
                                        .toggle(true)
                                        .selected(self.button_selections[index])
                                        .disabled(self.button_effects_disabled)
                                        .ripple(self.button_ripple)
                                        .icon(IconName::Check)
                                        .child(variant.name())
                                        .on_change({
                                            let entity = cx.entity().downgrade();
                                            move |selected, _, _, cx| {
                                                let _ = entity.update(cx, |this, cx| {
                                                    this.button_selections[index] = selected;
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .reveal(
                                            SharedString::from(format!(
                                                "reveal-toggle-{shape:?}-{variant:?}"
                                            )),
                                            scroll,
                                        )
                                    },
                                ),
                            ))
                    }),
            );
        let loading_variant = ButtonVariant::ALL[self.button_loading_variant];
        let button_loading = gpui_base::v_flex().id("gallery-section-loading").anchor_scroll(Some(self.section_anchors[4].clone())).gap(rem(16.))
            .child(section_title("button-loading", "Button progress and loading"))
            .child(gpui_base::h_flex().flex_wrap().gap(rem(12.))
                .child(material_button("toggle-loading", ButtonVariant::Outlined)
                    .child(if self.button_loading { "Stop loading" } else { "Start loading" })
                    .on_click(cx.listener(|this, _, _, cx| { this.button_loading = !this.button_loading; cx.notify(); }))
                    .reveal("reveal-toggle-loading", scroll))
                .child(material_button("toggle-progress", ButtonVariant::Outlined)
                    .child(if self.button_progress { "Hide progress" } else { "Show progress" })
                    .on_click(cx.listener(|this, _, _, cx| { this.button_progress = !this.button_progress; cx.notify(); }))
                    .reveal("reveal-toggle-progress", scroll))
                .child(material_button("loading-variant", ButtonVariant::Outlined)
                    .child(SharedString::from(format!("Variant: {}", loading_variant.name())))
                    .on_click(cx.listener(|this, _, _, cx| { this.button_loading_variant = (this.button_loading_variant + 1) % ButtonVariant::ALL.len(); cx.notify(); }))
                    .reveal("reveal-loading-variant", scroll)))
            .child(accessible_text("loading-help", "Progress stays interactive. Loading disables the samples and changes their labels."))
            .child(accessible_text("loading-click-count", SharedString::from(format!("Sample activations: {}", self.button_loading_clicks))))
            .children(ButtonSize::ALL.into_iter().map(|size| {
                gpui_base::v_flex().gap(rem(12.))
                    .child(accessible_text(SharedString::from(format!("loading-size-{size:?}")), size.name()))
                    .child(gpui_base::h_flex().flex_wrap().gap(rem(16.)).children((0..3).map(|icon_position| {
                        MaterialButton::new(SharedString::from(format!("loading-{size:?}-{icon_position}")))
                            .variant(loading_variant).button_size(size)
                            .when(icon_position > 0, |button| button.icon(IconName::Check))
                            .when(icon_position == 2, |button| button.icon_position(ButtonIconPosition::Trailing))
                            .disabled(self.button_effects_disabled).ripple(self.button_ripple)
                            .progress(self.button_progress, "Operation progress")
                            .loading(self.button_loading, "Operation loading")
                            .child(if self.button_loading { "Saving…" } else { "Save changes" })
                            .on_click(cx.listener(|this, _, _, cx| { this.button_loading_clicks += 1; cx.notify(); }))
                            .reveal(SharedString::from(format!("reveal-loading-{size:?}-{icon_position}")), scroll)
                    })))
            }));
        let button_icons = gpui_base::v_flex()
            .id("gallery-section-icons")
            .anchor_scroll(Some(self.section_anchors[5].clone()))
            .gap(rem(16.))
            .child(section_title("button-icons", "Button icons"))
            .children(ButtonSize::ALL.into_iter().map(|size| {
                gpui_base::h_flex().flex_wrap().gap(rem(12.)).children(
                    [
                        (
                            "Leading",
                            ButtonIconPosition::Leading,
                            ButtonDirection::Ltr,
                            ButtonVariant::Filled,
                        ),
                        (
                            "Trailing",
                            ButtonIconPosition::Trailing,
                            ButtonDirection::Ltr,
                            ButtonVariant::Filled,
                        ),
                        (
                            "RTL leading",
                            ButtonIconPosition::Leading,
                            ButtonDirection::Rtl,
                            ButtonVariant::Text,
                        ),
                        (
                            "RTL trailing",
                            ButtonIconPosition::Trailing,
                            ButtonDirection::Rtl,
                            ButtonVariant::Text,
                        ),
                    ]
                    .into_iter()
                    .map(move |(label, position, direction, variant)| {
                        material_button(
                            SharedString::from(format!("icon-{size:?}-{label}")),
                            variant,
                        )
                        .button_size(size)
                        .icon(IconName::Check)
                        .icon_position(position)
                        .direction(direction)
                        .child(label)
                        .reveal(
                            SharedString::from(format!("reveal-icon-{size:?}-{label}")),
                            scroll,
                        )
                    }),
                )
            }));
        let text_labels = gpui_base::v_flex()
            .id("gallery-section-labels")
            .anchor_scroll(Some(self.section_anchors[6].clone()))
            .gap(rem(16.))
            .child(section_title("text-labels", "Text labels"))
            .children(
                [
                    TypeRole::LabelLarge,
                    TypeRole::LabelMedium,
                    TypeRole::LabelSmall,
                ]
                .into_iter()
                .map(|role| {
                    let spec = role.scale();
                    accessible_text(
                        SharedString::from(format!("label-role-{}", role.name())),
                        role.name(),
                    )
                    .font_family(spec.font_family)
                    .text_size(spec.font_size)
                    .line_height(spec.line_height)
                    .font_weight(FontWeight(spec.font_weight as f32))
                }),
            );
        let typography_samples = TypeRole::ALL.into_iter().map(|role| {
            let spec = role.scale();
            gpui_base::v_flex()
                .gap(rem(4.))
                .child(
                    accessible_text(
                        SharedString::from(format!("type-label-{}", role.name())),
                        role.name(),
                    )
                    .text_size(TEXT_AUX)
                    .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                )
                .child(
                    accessible_text(
                        SharedString::from(format!("type-sample-{}", role.name())),
                        "The quick brown fox jumps over the lazy dog",
                    )
                    .whitespace_normal()
                    .font_family(spec.font_family)
                    .font_weight(FontWeight(spec.font_weight as f32))
                    .text_size(spec.font_size)
                    .line_height(spec.line_height),
                )
        });
        let scale = ConnectedButtonGroup::new("scale", "Text size")
            .reveal_in(scroll.clone())
            .options([
                ("1", "100%"),
                ("1.25", "125%"),
                ("1.5", "150%"),
                ("2", "200%"),
            ])
            .selected(self.scale.clone())
            .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                this.scale = value.clone();
                cx.notify();
            }));
        let density = ConnectedButtonGroup::new("density", "Density")
            .reveal_in(scroll.clone())
            .segmented()
            .options([
                ("auto", "Automatic"),
                ("dense", "Dense"),
                ("roomy", "Roomy"),
            ])
            .icon("auto", IconName::Info)
            .icon("dense", IconName::Check)
            .icon("roomy", IconName::Warning)
            .selected(self.density.clone())
            .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                this.density = value.clone();
                cx.notify();
            }));

        let engines = ["Local engine", "Cloud service", "Unavailable"];
        let mut choices = TileGroup::new("engines", "Choice tiles");
        for (index, title) in engines.into_iter().enumerate() {
            let tile = ChoiceTile::new(
                SharedString::from(format!("engine-{index}")),
                index,
                engines.len(),
                title,
                self.engine == index,
            )
            .tinted_icon(
                IconName::Info.icon(),
                [Tint::Blue, Tint::Teal, Tint::Orange][index],
                px(40.),
            )
            .description(match index {
                0 => "Runs on this computer",
                1 => "Sends audio to the configured service",
                _ => "Shown disabled, with its status at full strength",
            })
            .status((index == 2).then(|| badge(BadgeKind::Warning).child("Missing")))
            .enabled(index != 2)
            .render(window, cx)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.engine = index;
                cx.notify();
            }));
            choices = choices.tile(move |_| tile.into_any_element());
        }

        let details = self.details;
        let motion_controls = ConnectedButtonGroup::new("motion-scheme", "Motion scheme")
            .reveal_in(scroll.clone())
            .options([("expressive", "Expressive"), ("standard", "Standard")])
            .selected(match self.motion_scheme {
                motion::MotionScheme::Expressive => "expressive",
                motion::MotionScheme::Standard => "standard",
            })
            .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                this.motion_scheme = if value == "standard" {
                    motion::MotionScheme::Standard
                } else {
                    motion::MotionScheme::Expressive
                };
                cx.notify();
            }));
        let motion_rows = [
            ("Fast spatial", motion::MotionToken::FastSpatial, false),
            (
                "Default spatial",
                motion::MotionToken::DefaultSpatial,
                false,
            ),
            ("Slow spatial", motion::MotionToken::SlowSpatial, false),
            ("Fast effects", motion::MotionToken::FastEffects, true),
            ("Default effects", motion::MotionToken::DefaultEffects, true),
            ("Slow effects", motion::MotionToken::SlowEffects, true),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (label, token, effects))| {
            let progress = motion::token_value(
                SharedString::from(format!("motion-preview-{index}")),
                if self.motion_target { 1. } else { 0. },
                token,
                window,
                cx,
            );
            gpui_base::h_flex()
                .gap(rem(12.))
                .child(div().w(rem(160.)).child(label))
                .child(
                    div().w(rem(200.)).h(rem(24.)).child(
                        div()
                            .size(rem(24.))
                            .rounded_full()
                            .bg(color(ColorRole::ACCENT))
                            .when(effects, |view| view.opacity(progress))
                            .when(!effects, |view| view.ml(rem(progress * 160.))),
                    ),
                )
        })
        .collect::<Vec<_>>();
        let toggle = cx.listener(|this, _, _, cx| {
            this.details = !this.details;
            cx.notify();
        });
        let settings = TileGroup::new("tiles", "Connected tiles")
            .tile(move |outer| {
                TileScale::Regular
                    .button("details", outer)
                    .on_click(toggle)
                    .child(
                        TileScale::Regular
                            .row()
                            .child(TileScale::Regular.text(
                                "details-text",
                                "Show details",
                                Some("A tile that discloses content in place".into()),
                            ))
                            .child(disclosure_chevron(details)),
                    )
                    .into_any_element()
            })
            .detail(motion::disclosure(
                "details-content",
                details,
                div()
                    .px(ITEM_INSET)
                    .pb(rem(16.))
                    .child(tile_info("details-info", "Details stay inside their tile.")),
            ))
            .child(
                div()
                    .px(ITEM_INSET)
                    .py(rem(16.))
                    .child(TileScale::Regular.text(
                        "plain",
                        "A content tile",
                        Some("Its corners follow its place in the group".into()),
                    )),
            );

        let feedback = gpui_base::v_flex()
            .id("gallery-section-feedback")
            .anchor_scroll(Some(self.section_anchors[7].clone()))
            .gap(rem(12.))
            .child(section_title("feedback", "Feedback"))
            .child(
                gpui_base::h_flex()
                    .flex_wrap()
                    .gap(rem(8.))
                    .child(badge(BadgeKind::Success).child("Ready"))
                    .child(badge(BadgeKind::Warning).child("Needs attention"))
                    .child(badge(BadgeKind::Danger).child("Failed"))
                    .child(badge(BadgeKind::Progress).child("Working"))
                    .child(badge(BadgeKind::Neutral).child("Queued"))
                    .child(badge(BadgeKind::Tonal).child("Default"))
                    .child(outlined_badge("outlined", "Outlined")),
            )
            .child(notice_banner(
                "notice-info",
                NoticeKind::Info,
                "An informational notice wraps its text when narrow.",
            ))
            .child(notice_banner(
                "notice-danger",
                NoticeKind::Danger,
                "A failure stays visible with its recovery action.",
            ))
            .child(info_callout(
                "callout",
                "Standalone guidance sits in its own low-emphasis panel.",
            ))
            .child(error_text("error", "Error text is announced as an alert."))
            .child(
                card_surface()
                    .gap(rem(12.))
                    .child(semantic_label(
                        "progress-label",
                        "Linear progress",
                        icon_spinner("label-spinner", ICON_SLOT, cx),
                    ))
                    .child(progress("measured", 0.62, window, cx))
                    .child(indeterminate_progress("indeterminate", cx))
                    .child(accessible_text("circular-label", "Circular progress"))
                    .child(
                        gpui_base::h_flex()
                            .gap(rem(16.))
                            .child(circular_progress(
                                "circular-animated",
                                "Loading",
                                sample_circular_progress("circular-animated-clock", window, cx),
                                None,
                                size(rem(24.), rem(24.)),
                                1.,
                            ))
                            .children([0., 0.62, 1.].into_iter().enumerate().map(
                                |(index, fraction)| {
                                    circular_progress(
                                        SharedString::from(format!("circular-measured-{index}")),
                                        "Progress",
                                        CircularProgressFrame::determinate(fraction),
                                        Some(fraction),
                                        size(rem(24.), rem(24.)),
                                        1.,
                                    )
                                },
                            )),
                    ),
            );

        let navigation = div()
            .id("gallery-navigation")
            .role(Role::Navigation)
            .aria_label("Gallery sections")
            .flex_shrink_0()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.navigation_scroll)
            .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
            .when(compact, |nav| nav.w_full().max_h(rem(160.)).border_b_1())
            .when(!compact, |nav| nav.w(rem(176.)).h_full().border_r_1())
            .border_color(color(ColorRole::OUTLINE_VARIANT))
            .child(
                gpui_base::v_flex()
                    .p(rem(12.))
                    .pt(rem(32.))
                    .gap(rem(12.))
                    .child(
                        accessible_text("gallery-navigation-title", "Browse components")
                            .text_size(TEXT_AUX),
                    )
                    .child(
                        gpui_base::v_flex()
                            .gap(rem(8.))
                            .when(compact, |items| items.flex_row().flex_wrap())
                            .children(GALLERY_SECTIONS.into_iter().enumerate().map(
                                |(index, (id, label))| {
                                    MaterialButton::new(SharedString::from(format!(
                                        "navigate-{id}"
                                    )))
                                    .variant(ButtonVariant::Tonal)
                                    .button_size(ButtonSize::ExtraSmall)
                                    .toggle(true)
                                    .selected(self.selected_section == index)
                                    .toggle_on_click(false)
                                    .child(label)
                                    .when(!compact, |button| button.w_full().justify_start())
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.selected_section = index;
                                        this.section_anchors[index].scroll_to(window, cx);
                                        cx.notify();
                                    }))
                                    .reveal(
                                        SharedString::from(format!("reveal-navigation-{id}")),
                                        &self.navigation_scroll,
                                    )
                                },
                            )),
                    ),
            );
        let content = div()
            .id("gallery-content")
            .flex_1()
            .min_w_0()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .bg(color(ColorRole::CANVAS))
            .text_color(color(ColorRole::INK))
            .font_family("Roboto")
            .child(
                gpui_base::v_flex()
                    .w_full()
                    .max_w(rem(760.))
                    .mx_auto()
                    .px(px(if compact { 16. } else { 24. }))
                    .py(rem(32.))
                    .gap(rem(28.))
                    .child(page_heading("title", "gpui-kit-material"))
                    .child(supporting_info(
                        "intro",
                        "Explore Material themes, motion and components.",
                    ))
                    .child(
                        gpui_base::v_flex()
                            .gap(rem(12.))
                            .id("gallery-section-themes").anchor_scroll(Some(self.section_anchors[0].clone()))
                            .child(section_title("reference-theme", "Material themes"))
                            .child(reference_themes)
                            .child(mode)
                            .child(contrast),
                    )
                    .child(
                        gpui_base::v_flex()
                            .gap(rem(12.))
                            .id("gallery-section-motion").anchor_scroll(Some(self.section_anchors[1].clone()))
                            .child(section_title("motion", "Motion"))
                            .child(motion_controls)
                            .child(
                                material_button("play-motion", ButtonVariant::Tonal)
                                    .child("Toggle motion preview")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.motion_target = !this.motion_target;
                                        cx.notify();
                                    })).reveal("reveal-play-motion",scroll),
                            )
                            .children(motion_rows),
                    )
                    .child(
                        gpui_base::v_flex()
                            .gap(rem(12.))
                            .child(section_title("groups", "Connected button groups"))
                            .child(palettes)
                            .child(scale)
                            .child(density),
                    )
                    .child(
                        gpui_base::v_flex()
                            .gap(rem(16.))
                            .child(section_title("controls", "Material controls"))
                            .child(SearchField::new(&self.search).reveal("reveal-search",scroll))
                            .child(
                                MaterialSwitch::new("example-switch", "Enable notifications")
                                    .checked(self.switch)
                                    .on_change(cx.listener(|this, checked: &bool, _, cx| {
                                        this.switch = *checked;
                                        cx.notify();
                                    })).reveal("reveal-switch",scroll),
                            )
                            .child(MaterialSlider::new(&self.slider, "Expressive slider").reveal("reveal-slider",scroll))
                            .child(
                                MaterialSlider::new(&self.slider, "Classic slider")
                                    .appearance(SliderAppearance::Classic).reveal("reveal-slider-classic",scroll),
                            )
                            .child(section_title("buttons", "Buttons").anchor_scroll(Some(self.section_anchors[2].clone())))
                            .children(ButtonShape::ALL.into_iter().map(|shape| {
                                gpui_base::v_flex().gap(rem(16.)).children(
                                    ButtonSize::ALL.into_iter().map(move |size| {
                                        gpui_base::v_flex()
                                            .gap(rem(8.))
                                            .child(accessible_text(
                                                SharedString::from(format!(
                                                    "button-label-{shape:?}-{size:?}"
                                                )),
                                                SharedString::from(format!(
                                                    "{} / {}",
                                                    shape.name(),
                                                    size.name()
                                                )),
                                            ))
                                            .child(
                                                gpui_base::h_flex()
                                                    .flex_wrap()
                                                    .gap(rem(12.))
                                                    .children(ButtonVariant::ALL.into_iter().map(
                                                        move |variant| {
                                                            MaterialButton::new(SharedString::from(
                                                        format!(
                                                            "button-{shape:?}-{size:?}-{variant:?}"
                                                        ),
                                                    ))
                                                    .variant(variant)
                                                    .button_size(size)
                                                    .shape(shape)
                                                    .child(variant.name())
                                                    .reveal(SharedString::from(format!("reveal-button-{shape:?}-{size:?}-{variant:?}")),scroll)
                                                        },
                                                    )),
                                            )
                                    }),
                                )
                            }))
                            .child(button_effects)
                            .child(button_toggles)
                            .child(button_loading)
                            .child(button_icons)
                            .child(text_labels)
                            .child(gpui_base::h_flex().flex_wrap().gap(rem(12.)).children(
                                ButtonVariant::ALL.into_iter().map(|variant| {
                                    material_button(
                                        SharedString::from(format!("disabled-{variant:?}")),
                                        variant,
                                    )
                                    .disabled(true)
                                    .icon(IconName::Check)
                                    .child(variant.name())
                                }),
                            )),
                    )
                    .child(
                        div()
                            .relative()
                            .h(rem(220.))
                            .p(px(8.))
                            .overflow_hidden()
                            .child(
                                material_button("open-sheet", ButtonVariant::Tonal)
                                    .child("Toggle side sheet")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.sheet = !this.sheet;
                                        cx.notify();
                                    })).reveal("reveal-open-sheet",scroll),
                            )
                            .child(
                                MaterialSideSheet::new("gallery-sheet")
                                    .open(self.sheet)
                                    .width(rem(280.))
                                    .child(
                                        div().p(rem(20.)).child("Standard side sheet").child(
                                            material_button("close-sheet", ButtonVariant::Text)
                                                .child("Close")
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.sheet = false;
                                                    cx.notify();
                                                })).reveal("reveal-close-sheet",scroll),
                                        ),
                                    ),
                            ),
                    )
                    .child(choices.reveal("reveal-choices",scroll))
                    .child(settings.reveal("reveal-settings",scroll))
                    .child(feedback)
                    .child(
                        gpui_base::v_flex()
                            .gap(rem(12.))
                            .id("gallery-section-colors").anchor_scroll(Some(self.section_anchors[8].clone()))
                            .child(section_title("reference-colors", "Material color roles"))
                            .child(
                                gpui_base::h_flex()
                                    .flex_wrap()
                                    .gap(rem(12.))
                                    .children(color_swatches),
                            ),
                    )
                    .child(
                        gpui_base::v_flex()
                            .gap(rem(20.))
                            .id("gallery-section-typography").anchor_scroll(Some(self.section_anchors[9].clone()))
                            .child(section_title("type-scale", "Typography"))
                            .children(typography_samples),
                    ),
            );
        div()
            .id("gallery")
            .relative()
            .size_full()
            .flex()
            .flex_row()
            .when(compact, |layout| layout.flex_col())
            .bg(color(ColorRole::CANVAS))
            .text_color(color(ColorRole::INK))
            .font_family("Roboto")
            .child(navigation)
            .child(content)
            .child(CaptionButtons::new(app_bar_height(window)))
    }
}

pub fn run(application: gpui::Application, initialize: impl FnOnce(&mut App) + 'static) {
    let application = application.with_assets(Assets::new());
    application.run(move |cx: &mut App| {
        initialize(cx);
        #[cfg(target_family = "wasm")]
        cx.text_system()
            .add_fonts(vec![std::borrow::Cow::Borrowed(include_bytes!(
                "fonts/IBMPlexSans-Regular.ttf"
            ))])
            .expect("Failed to load the gallery font");
        cx.text_system()
            .add_fonts(vec![
                std::borrow::Cow::Borrowed(include_bytes!("fonts/roboto/Roboto-Regular.ttf")),
                std::borrow::Cow::Borrowed(include_bytes!("fonts/roboto/Roboto-Medium.ttf")),
            ])
            .expect("Failed to load the reference Roboto fonts");
        gpui_base::init(cx);
        gpui_kit_material::init(cx);
        let viewport = size(px(960.), px(900.));
        #[cfg(not(target_family = "wasm"))]
        let viewport = capture::window_size(cx).unwrap_or(viewport);
        let bounds = Bounds::centered(None, viewport, cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let gallery = cx.new(|cx| {
                    let search = cx.new(|cx| {
                        gpui_base::input::InputState::new(window, cx)
                            .placeholder("Search components")
                    });
                    let slider = cx.new(|_| {
                        gpui_base::slider::SliderState::new()
                            .min(0.)
                            .max(100.)
                            .default_value(40.)
                    });
                    cx.observe(&slider, |_, _, cx| cx.notify()).detach();
                    let scroll = ScrollHandle::new();
                    let section_anchors =
                        std::array::from_fn(|_| ScrollAnchor::for_handle(scroll.clone()));
                    Gallery {
                        search,
                        slider,
                        switch: false,
                        sheet: false,
                        motion_scheme: motion::MotionScheme::Expressive,
                        motion_target: false,
                        button_effects_disabled: false,
                        button_effects_tonal: false,
                        button_ripple: true,
                        button_active: false,
                        button_selections: [false; 10],
                        button_toggle_size: ButtonSize::Small,
                        button_loading: false,
                        button_progress: false,
                        button_loading_variant: 0,
                        button_loading_clicks: 0,
                        palette: PaletteId::Paper,
                        reference_theme: Some(ThemeId::Baseline),
                        reference_mode: ThemeMode::Light,
                        contrast: Contrast::Standard,
                        scale: "1".into(),
                        density: "auto".into(),
                        engine: 0,
                        details: false,
                        scroll,
                        section_anchors,
                        navigation_scroll: ScrollHandle::new(),
                        selected_section: 0,
                    }
                });
                #[cfg(not(target_family = "wasm"))]
                capture::install(&gallery, window, cx);
                cx.new(|cx| gpui_base::Root::new(gallery, window, cx))
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
