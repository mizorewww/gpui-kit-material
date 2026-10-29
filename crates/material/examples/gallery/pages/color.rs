//! Color: the reference themes, mode, contrast and palettes that repaint
//! the Gallery, and every color role as a swatch.
use super::prelude::*;
use gpui_kit_material::color::palette::ALL as PALETTES;
use gpui_kit_material::color::themes;

pub struct Page {
    page: PageCx,
}

const CONTROL_ROWS: usize = 4;
const SWATCH_WIDTH: f32 = 166.;
const SWATCH_GAP: f32 = 12.;

/// Swatches per row at the window's current width.
pub fn columns(window: &Window) -> usize {
    ((spec::content_width(window) + SWATCH_GAP) / (SWATCH_WIDTH + SWATCH_GAP))
        .floor()
        .max(1.) as usize
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "color",
        title: "Color",
        description: "Color roles, schemes and the reference themes. Pick a theme, mode, contrast or palette to repaint the whole Gallery.",
        guideline: Some("https://m3.material.io/styles/color/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self { page }
    }

    fn row_count(&self, window: &Window) -> usize {
        CONTROL_ROWS + themes::REFERENCE_ROLES.len().div_ceil(columns(window))
    }

    fn row_gap(&self, row: usize) -> f32 {
        match row {
            0 => spec::HEADER_TO_BODY,
            1..CONTROL_ROWS => spec::ROW_GAP,
            CONTROL_ROWS => 40.,
            _ => SWATCH_GAP,
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
        let list = self.page.rows.list().clone();
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
        match row {
            0 => spec::demo(
                "reference-themes-demo",
                "Reference themes",
                ConnectedButtonGroup::new("reference-themes", "Material themes")
                    .options(themes::ALL.map(|id| (id.name(), id.name())))
                    .selected(theme.map(ThemeId::name).unwrap_or(""))
                    .full_width()
                    .on_change(update(|shared, name| {
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
                    .selected(if theme.is_none() { palette.name() } else { "" })
                    .full_width()
                    .on_change(update(|shared, name| {
                        if let Some(id) = PALETTES.into_iter().find(|id| id.name() == name) {
                            shared.palette = id;
                            shared.reference_theme = None;
                        }
                    }))
                    .reveal("reveal-palettes", &list),
            )
            .into_any_element(),
            _ => {
                let columns = columns(window);
                let start = (row - CONTROL_ROWS) * columns;
                div()
                    .when(start == 0, |row| {
                        row.debug_selector(|| "color-swatches".into())
                    })
                    .flex()
                    .gap(rem(SWATCH_GAP))
                    .children(
                        themes::REFERENCE_ROLES
                            .into_iter()
                            .zip(themes::REFERENCE_ROLE_NAMES)
                            .enumerate()
                            .skip(start)
                            .take(columns)
                            .map(|(index, (role, name))| {
                                div()
                                    .w(rem(SWATCH_WIDTH))
                                    .flex()
                                    .flex_col()
                                    .gap(rem(4.))
                                    .child(
                                        div()
                                            .h(rem(32.))
                                            .w_full()
                                            .rounded(rem(8.))
                                            .border_1()
                                            .border_color(color(ColorRole::OUTLINE_VARIANT))
                                            .bg(color(role)),
                                    )
                                    .child(spec::small_label(("color", index), name))
                            }),
                    )
                    .into_any_element()
            }
        }
    }
}
