//! Color, like md3.io's Color style page: the reference theme, mode,
//! contrast and palette that repaint the whole Gallery, then every color
//! role grouped by family, each swatch with its name, hex value and the
//! WCAG contrast of the text role painted on it.
use super::prelude::*;
use gpui_kit_material::color::palette::ALL as PALETTES;
use gpui_kit_material::color::themes;

pub struct Page {
    page: PageCx,
}

const CONTROL_ROWS: usize = 4;
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

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "color",
        title: "Color",
        description: "Every color role in each theme, light and dark, with contrast checks. Pick a theme, mode, contrast or palette to repaint the whole Gallery.",
        guideline: Some("https://m3.material.io/styles/color/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self { page }
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
                "Theme",
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
