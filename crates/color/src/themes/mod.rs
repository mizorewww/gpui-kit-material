//! Exact preset colors, including medium/high contrast overrides.
//!
//! Reference priority is Figma Material 3 Design Kit > md3.io >
//! m3e.kyralabs.dev, per value. The kit's `M3` variable collection
//! (`Schemes/*`, file key `ozddDB2Z1v51f1IFZlUuJ2`, collection
//! `VariableCollectionId:54778:406`, modes Light/Dark × standard/medium/high
//! contrast) defines the baseline scheme, so [`ThemeId::Baseline`] takes every
//! role the kit defines from it (fixture
//! `tests/fixtures/figma-baseline-colors.json`). The kit defines no other md3.io
//! theme and no info/success/warning roles; those come from md3.io's
//! browser-resolved Sass at c910072f3fc5b660276731d8a935d7186f288364 (fixture
//! `tests/fixtures/md3-colors.json`). These presets are not a dynamic HCT color
//! generator. See LICENSE-material.
use crate::{ColorRole, Scheme};
mod data;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Contrast {
    #[default]
    Standard,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum ThemeId {
    Autumn,
    #[default]
    Baseline,
    Brownbeige,
    Content,
    Desert,
    Expressive,
    Fidelity,
    Forest,
    FruitSalad,
    Highcontrast,
    Monochrome,
    Neutral,
    Ocean,
    Rainbow,
    Sageivory,
    Spring,
    Summer,
    Sunset,
    Tealcaramel,
    Vibrant,
}
pub const ALL: [ThemeId; 20] = [
    ThemeId::Autumn,
    ThemeId::Baseline,
    ThemeId::Brownbeige,
    ThemeId::Content,
    ThemeId::Desert,
    ThemeId::Expressive,
    ThemeId::Fidelity,
    ThemeId::Forest,
    ThemeId::FruitSalad,
    ThemeId::Highcontrast,
    ThemeId::Monochrome,
    ThemeId::Neutral,
    ThemeId::Ocean,
    ThemeId::Rainbow,
    ThemeId::Sageivory,
    ThemeId::Spring,
    ThemeId::Summer,
    ThemeId::Sunset,
    ThemeId::Tealcaramel,
    ThemeId::Vibrant,
];
pub const REFERENCE_ROLES: [ColorRole; 52] = [
    ColorRole::ERROR,
    ColorRole::ERROR_CONTAINER,
    ColorRole::INFO,
    ColorRole::INVERSE_ON_SURFACE,
    ColorRole::INVERSE_PRIMARY,
    ColorRole::INVERSE_SURFACE,
    ColorRole::ON_ERROR,
    ColorRole::ON_ERROR_CONTAINER,
    ColorRole::ON_INFO,
    ColorRole::ON_PRIMARY,
    ColorRole::ON_PRIMARY_CONTAINER,
    ColorRole::ON_PRIMARY_FIXED,
    ColorRole::ON_PRIMARY_FIXED_VARIANT,
    ColorRole::ON_SECONDARY,
    ColorRole::ON_SECONDARY_CONTAINER,
    ColorRole::ON_SECONDARY_FIXED,
    ColorRole::ON_SECONDARY_FIXED_VARIANT,
    ColorRole::ON_SUCCESS,
    ColorRole::ON_SURFACE,
    ColorRole::ON_SURFACE_VARIANT,
    ColorRole::ON_TERTIARY,
    ColorRole::ON_TERTIARY_CONTAINER,
    ColorRole::ON_TERTIARY_FIXED,
    ColorRole::ON_TERTIARY_FIXED_VARIANT,
    ColorRole::ON_WARNING,
    ColorRole::OUTLINE,
    ColorRole::OUTLINE_VARIANT,
    ColorRole::PRIMARY,
    ColorRole::PRIMARY_CONTAINER,
    ColorRole::PRIMARY_FIXED,
    ColorRole::PRIMARY_FIXED_DIM,
    ColorRole::SCRIM,
    ColorRole::SECONDARY,
    ColorRole::SECONDARY_CONTAINER,
    ColorRole::SECONDARY_FIXED,
    ColorRole::SECONDARY_FIXED_DIM,
    ColorRole::SHADOW,
    ColorRole::SUCCESS,
    ColorRole::SURFACE,
    ColorRole::SURFACE_BRIGHT,
    ColorRole::SURFACE_CONTAINER,
    ColorRole::SURFACE_CONTAINER_HIGH,
    ColorRole::SURFACE_CONTAINER_HIGHEST,
    ColorRole::SURFACE_CONTAINER_LOW,
    ColorRole::SURFACE_CONTAINER_LOWEST,
    ColorRole::SURFACE_DIM,
    ColorRole::SURFACE_VARIANT,
    ColorRole::TERTIARY,
    ColorRole::TERTIARY_CONTAINER,
    ColorRole::TERTIARY_FIXED,
    ColorRole::TERTIARY_FIXED_DIM,
    ColorRole::WARNING,
];
pub const REFERENCE_ROLE_NAMES: [&str; 52] = [
    "error",
    "error-container",
    "info",
    "inverse-on-surface",
    "inverse-primary",
    "inverse-surface",
    "on-error",
    "on-error-container",
    "on-info",
    "on-primary",
    "on-primary-container",
    "on-primary-fixed",
    "on-primary-fixed-variant",
    "on-secondary",
    "on-secondary-container",
    "on-secondary-fixed",
    "on-secondary-fixed-variant",
    "on-success",
    "on-surface",
    "on-surface-variant",
    "on-tertiary",
    "on-tertiary-container",
    "on-tertiary-fixed",
    "on-tertiary-fixed-variant",
    "on-warning",
    "outline",
    "outline-variant",
    "primary",
    "primary-container",
    "primary-fixed",
    "primary-fixed-dim",
    "scrim",
    "secondary",
    "secondary-container",
    "secondary-fixed",
    "secondary-fixed-dim",
    "shadow",
    "success",
    "surface",
    "surface-bright",
    "surface-container",
    "surface-container-high",
    "surface-container-highest",
    "surface-container-low",
    "surface-container-lowest",
    "surface-dim",
    "surface-variant",
    "tertiary",
    "tertiary-container",
    "tertiary-fixed",
    "tertiary-fixed-dim",
    "warning",
];
impl ThemeId {
    pub fn name(self) -> &'static str {
        match self {
            Self::Autumn => "autumn",
            Self::Baseline => "baseline",
            Self::Brownbeige => "brownbeige",
            Self::Content => "content",
            Self::Desert => "desert",
            Self::Expressive => "expressive",
            Self::Fidelity => "fidelity",
            Self::Forest => "forest",
            Self::FruitSalad => "fruit-salad",
            Self::Highcontrast => "highcontrast",
            Self::Monochrome => "monochrome",
            Self::Neutral => "neutral",
            Self::Ocean => "ocean",
            Self::Rainbow => "rainbow",
            Self::Sageivory => "sageivory",
            Self::Spring => "spring",
            Self::Summer => "summer",
            Self::Sunset => "sunset",
            Self::Tealcaramel => "tealcaramel",
            Self::Vibrant => "vibrant",
        }
    }
    pub fn scheme(self, mode: ThemeMode, contrast: Contrast) -> Scheme {
        let mode_index = match mode {
            ThemeMode::Light => 0,
            ThemeMode::Dark => 1,
        };
        let contrast_index = match contrast {
            Contrast::Standard => 0,
            Contrast::Medium => 1,
            Contrast::High => 2,
        };
        Scheme::from_reference(
            &data::COLORS[self as usize][mode_index][contrast_index],
            mode == ThemeMode::Dark,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, scheme::ROLE_COUNT};

    fn mode_and_contrast(row: &serde_json::Value) -> (ThemeMode, Contrast) {
        let mode = match row["mode"].as_str().unwrap() {
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            _ => panic!("invalid mode"),
        };
        let contrast = match row["contrast"].as_str().unwrap() {
            "standard" => Contrast::Standard,
            "medium" => Contrast::Medium,
            "high" => Contrast::High,
            _ => panic!("invalid contrast"),
        };
        (mode, contrast)
    }

    fn hex(value: &serde_json::Value) -> Color {
        Color::hex(u32::from_str_radix(&value.as_str().unwrap()[1..], 16).unwrap())
    }

    fn figma_baseline() -> serde_json::Value {
        serde_json::from_str(include_str!(
            "../../tests/fixtures/figma-baseline-colors.json"
        ))
        .unwrap()
    }

    #[test]
    fn baseline_matches_the_figma_kit_variables_in_every_mode() {
        let reference = figma_baseline();
        let rows = reference["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 6);
        let mut defined = 0;
        for row in rows {
            assert_eq!(row["theme"], "baseline");
            let (mode, contrast) = mode_and_contrast(row);
            let scheme = ThemeId::Baseline.scheme(mode, contrast);
            assert_eq!(scheme.is_dark(), mode == ThemeMode::Dark);
            for (role, name) in REFERENCE_ROLES.into_iter().zip(REFERENCE_ROLE_NAMES) {
                let Some(expected) = row["colors"].get(name) else {
                    continue;
                };
                defined += 1;
                assert_eq!(
                    scheme.get(role),
                    hex(expected),
                    "Figma {} {role:?}",
                    row["figma_mode"]
                );
            }
        }
        // Every reference role except info/success/warning and their on-roles.
        assert_eq!(defined, 6 * 46);
    }

    #[test]
    fn every_reference_configuration_matches_browser_resolved_css() {
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/md3-colors.json")).unwrap();
        let rows = reference["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 120);
        assert_eq!(REFERENCE_ROLES.len(), 52);
        let figma = figma_baseline();
        let mut seen = std::collections::BTreeSet::new();
        for row in rows {
            let name = row["theme"].as_str().unwrap();
            let mode_name = row["mode"].as_str().unwrap();
            let contrast_name = row["contrast"].as_str().unwrap();
            assert!(seen.insert((name, mode_name, contrast_name)));
            let id = ALL.into_iter().find(|id| id.name() == name).unwrap();
            let (mode, contrast) = mode_and_contrast(row);
            let scheme = id.scheme(mode, contrast);
            assert_eq!(scheme.is_dark(), mode == ThemeMode::Dark);
            assert_eq!(row["colors"].as_object().unwrap().len(), 52);
            // The Figma kit outranks md3.io for every baseline role it defines.
            let kit = (id == ThemeId::Baseline).then(|| {
                figma["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|kit| kit["mode"] == mode_name && kit["contrast"] == contrast_name)
                    .unwrap()
            });
            for (role, name) in REFERENCE_ROLES.into_iter().zip(REFERENCE_ROLE_NAMES) {
                if kit.is_some_and(|kit| kit["colors"].get(name).is_some()) {
                    continue;
                }
                let expected = hex(&row["colors"][name]);
                assert_eq!(
                    scheme.get(role),
                    expected,
                    "{id:?} {mode:?} {contrast:?} {role:?}"
                );
            }
        }
    }

    #[test]
    #[allow(
        deprecated,
        reason = "checks that the deprecated aliases keep sharing storage with their roles"
    )]
    fn aliases_share_storage_and_reference_roles_are_not_duplicated() {
        let unique = REFERENCE_ROLES
            .into_iter()
            .map(|r| r as usize)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique.len(), 52);
        assert_eq!(
            ROLE_COUNT,
            52 + 10 + 14,
            "52 reference roles, ten documented application extensions and seven tint pairs"
        );
        for (old, new) in [
            (ColorRole::ACCENT, ColorRole::PRIMARY),
            (ColorRole::INK, ColorRole::ON_SURFACE),
            (ColorRole::CANVAS, ColorRole::SURFACE_CONTAINER),
            (
                ColorRole::SURFACE_HIGH,
                ColorRole::SURFACE_CONTAINER_HIGHEST,
            ),
            (ColorRole::GRAY, ColorRole::ON_SURFACE_VARIANT),
            (ColorRole::CONTROL, ColorRole::OUTLINE),
            (ColorRole::HAIRLINE, ColorRole::OUTLINE_VARIANT),
            (ColorRole::DANGER, ColorRole::ERROR),
            (ColorRole::DANGER_BG, ColorRole::ERROR_CONTAINER),
        ] {
            assert_eq!(old, new);
            let scheme = ThemeId::Baseline
                .scheme(ThemeMode::Light, Contrast::Standard)
                .with_role(old, Color::hex(0x123456));
            assert_eq!(scheme.get(new), Color::hex(0x123456));
        }
    }

    #[test]
    fn standard_fixed_roles_are_mode_independent_except_the_reference_highcontrast_preset() {
        for theme in ALL.into_iter().filter(|id| *id != ThemeId::Highcontrast) {
            let light = theme.scheme(ThemeMode::Light, Contrast::Standard);
            let dark = theme.scheme(ThemeMode::Dark, Contrast::Standard);
            for (role, name) in REFERENCE_ROLES
                .into_iter()
                .zip(REFERENCE_ROLE_NAMES)
                .filter(|(_, name)| name.contains("fixed"))
            {
                assert_eq!(light.get(role), dark.get(role), "{theme:?}: {name}");
            }
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn reference_theme_ids_roundtrip_with_the_site_names() {
        for id in ALL {
            let json = serde_json::to_string(&id).unwrap();
            assert_eq!(json, format!("\"{}\"", id.name()));
            assert_eq!(serde_json::from_str::<ThemeId>(&json).unwrap(), id);
        }
    }
}
