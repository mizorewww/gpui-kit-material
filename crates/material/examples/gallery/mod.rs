//! Every extracted component on one page, in every built-in palette.
//!
//! `cargo run -p gpui-kit-material --example gallery`
use gpui::{prelude::*, *};
use gpui_kit_material::color::palette::ALL as PALETTES;
use gpui_kit_material::icons::{Assets, IconName};
use gpui_kit_material::motion;
use gpui_kit_material::prelude::*;
use gpui_kit_material::theme;

/// The window title bar, fixed so the macOS traffic lights stay centred in it
/// at every text scale.
const TITLE_BAR: Pixels = px(56.);

struct Gallery {
    palette: PaletteId,
    scale: SharedString,
    density: SharedString,
    engine: usize,
    details: bool,
    scroll: ScrollHandle,
    search: Entity<gpui_base::input::InputState>,
    slider: Entity<gpui_base::slider::SliderState>,
    switch: bool,
    sheet: bool,
    snackbar: bool,
    motion_scheme: motion::MotionScheme,
    motion_target: bool,
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        motion::apply_scheme(self.motion_scheme);
        theme::apply_palette(self.palette, window, cx);
        theme::apply_scale(self.scale.parse().unwrap_or(1.), window);
        let compact = is_compact(window);
        let palettes = ConnectedButtonGroup::new("palette", "Palette")
            .options(PALETTES.map(|id| (id.name(), id.name())))
            .selected(self.palette.name())
            .full_width()
            .on_change(cx.listener(|this, name: &SharedString, _, cx| {
                if let Some(id) = PALETTES.into_iter().find(|id| id.name() == name.as_ref()) {
                    this.palette = id;
                    cx.notify();
                }
            }));
        let scale = ConnectedButtonGroup::new("scale", "Text size")
            .options([("1", "100%"), ("1.25", "125%"), ("1.5", "150%")])
            .selected(self.scale.clone())
            .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                this.scale = value.clone();
                cx.notify();
            }));
        let density = ConnectedButtonGroup::new("density", "Density")
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
                material_button("show-snackbar", ButtonVariant::Tonal)
                    .child("Show snackbar")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.snackbar = true;
                        cx.notify();
                    })),
            )
            .child(
                card_surface()
                    .gap(rem(12.))
                    .child(semantic_label(
                        "progress-label",
                        "Linear progress",
                        icon_spinner("label-spinner", ICON_SLOT, cx),
                    ))
                    .child(progress("measured", 0.62, window, cx))
                    .child(indeterminate_progress("indeterminate", cx)),
            );

        let content = div()
            .id("gallery")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .bg(color(ColorRole::CANVAS))
            .text_color(color(ColorRole::INK))
            .font_family(".SystemUIFont")
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
                        "Components extracted from course2md; M3 Expressive arrives next.",
                    ))
                    .child(
                        gpui_base::v_flex()
                            .gap(rem(12.))
                            .child(section_title("motion", "Motion"))
                            .child(motion_controls)
                            .child(
                                material_button("play-motion", ButtonVariant::Tonal)
                                    .child("Toggle motion preview")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.motion_target = !this.motion_target;
                                        cx.notify();
                                    })),
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
                            .child(SearchField::new(&self.search))
                            .child(
                                MaterialSwitch::new("example-switch", "Enable notifications")
                                    .checked(self.switch)
                                    .on_change(cx.listener(|this, checked: &bool, _, cx| {
                                        this.switch = *checked;
                                        cx.notify();
                                    })),
                            )
                            .child(MaterialSlider::new(&self.slider, "Expressive slider"))
                            .child(
                                MaterialSlider::new(&self.slider, "Classic slider")
                                    .appearance(SliderAppearance::Classic),
                            )
                            .child(
                                gpui_base::h_flex().gap(rem(8.)).children(
                                    [
                                        ("filled", ButtonVariant::Filled),
                                        ("tonal", ButtonVariant::Tonal),
                                        ("outlined", ButtonVariant::Outlined),
                                        ("text", ButtonVariant::Text),
                                    ]
                                    .map(|(label, variant)| {
                                        material_button(label, variant).child(label)
                                    }),
                                ),
                            ),
                    )
                    .child(
                        div()
                            .relative()
                            .h(rem(220.))
                            .overflow_hidden()
                            .child(
                                material_button("open-sheet", ButtonVariant::Tonal)
                                    .child("Toggle side sheet")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.sheet = !this.sheet;
                                        cx.notify();
                                    })),
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
                                                })),
                                        ),
                                    ),
                            ),
                    )
                    .child(choices)
                    .child(settings)
                    .child(feedback),
            );
        gpui_base::v_flex()
            .relative()
            .size_full()
            .bg(color(ColorRole::CANVAS))
            .text_color(color(ColorRole::INK))
            .font_family(".SystemUIFont")
            .child(
                TitleBar::new("title-bar", TITLE_BAR)
                    .gap(rem(12.))
                    .child(section_title("window-title", "gpui-kit-material")),
            )
            .child(content)
            .when(self.snackbar, |root| {
                root.child(
                    div().absolute().left(px(16.)).bottom(px(16.)).child(
                        Snackbar::new("snackbar", "Settings saved")
                            .action(
                                "Undo",
                                cx.listener(|this, _, _, cx| {
                                    this.snackbar = false;
                                    cx.notify();
                                }),
                            )
                            .close(
                                "Dismiss",
                                cx.listener(|this, _, _, cx| {
                                    this.snackbar = false;
                                    cx.notify();
                                }),
                            ),
                    ),
                )
            })
    }
}

pub fn run(application: gpui::Application) {
    application.with_assets(Assets::new()).run(|cx: &mut App| {
        #[cfg(target_family = "wasm")]
        cx.text_system()
            .add_fonts(vec![std::borrow::Cow::Borrowed(include_bytes!(
                "fonts/IBMPlexSans-Regular.ttf"
            ))])
            .expect("Failed to load the gallery font");
        gpui_base::init(cx);
        gpui_kit_material::init(cx);
        cx.open_window(
            title_bar_window_options("gpui-kit-material", TITLE_BAR, size(px(960.), px(900.)), cx),
            |window, cx| {
                cx.new(|cx| {
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
                    Gallery {
                        search,
                        slider,
                        switch: false,
                        sheet: false,
                        snackbar: false,
                        motion_scheme: motion::MotionScheme::Expressive,
                        motion_target: false,
                        palette: PaletteId::Paper,
                        scale: "1".into(),
                        density: "auto".into(),
                        engine: 0,
                        details: false,
                        scroll: ScrollHandle::new(),
                    }
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
