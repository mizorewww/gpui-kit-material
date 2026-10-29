//! Every extracted component on one page, in every built-in palette.
//!
//! `cargo run -p gpui-kit-material --example gallery`
use gpui::{prelude::*, *};
use gpui_kit_material::color::palette::ALL as PALETTES;
use gpui_kit_material::icons::{Assets, IconName};
use gpui_kit_material::motion;
use gpui_kit_material::prelude::*;
use gpui_kit_material::theme;

struct Gallery {
    palette: PaletteId,
    scale: SharedString,
    density: SharedString,
    engine: usize,
    details: bool,
    scroll: ScrollHandle,
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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

        div()
            .id("gallery")
            .size_full()
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
                            .child(section_title("groups", "Connected button groups"))
                            .child(palettes)
                            .child(scale)
                            .child(density),
                    )
                    .child(choices)
                    .child(settings)
                    .child(feedback),
            )
            .child(CaptionButtons::new(app_bar_height(window)))
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(Assets::new())
        .run(|cx: &mut App| {
            gpui_base::init(cx);
            gpui_kit_material::init(cx);
            let bounds = Bounds::centered(None, size(px(960.), px(900.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|_| Gallery {
                        palette: PaletteId::Paper,
                        scale: "1".into(),
                        density: "auto".into(),
                        engine: 0,
                        details: false,
                        scroll: ScrollHandle::new(),
                    })
                },
            )
            .unwrap();
            cx.activate(true);
        });
}
