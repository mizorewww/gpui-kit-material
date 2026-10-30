//! Type scale in rems. 14px is 1rem at the 100% text scale.
//!
//! Reference priority is Figma Material 3 Design Kit > md3.io >
//! m3e.kyralabs.dev, per value, and the kit defines the whole type family:
//! sizes, line heights, weights, tracking, typefaces and the emphasized
//! styles all come from its local text styles `M3/<group>/<size>` and
//! `M3/<group>/<size>-emphasized` (Styles page, file key
//! `ozddDB2Z1v51f1IFZlUuJ2`), which are bound to its `Typescale` and
//! `Font theme` variable collections (mode Baseline). Fixture:
//! `crates/material/tests/fixtures/figma-typography.json`.
//!
//! The kit's Baseline font theme sets both the brand and plain faces to
//! Roboto, with Regular/Medium/SemiBold as its regular/medium/bold weights
//! (so the emphasized title-medium, title-small and label styles are 600).
//! md3.io agrees on every baseline metric but puts title-medium and
//! title-small on the brand face (`tests/fixtures/md3-typography.json`);
//! m3e's tracking,
//! 700 emphasized weights and Google Sans Flex
//! (`tests/fixtures/m3e-typography.json`) are outranked and kept only as
//! secondary evidence. Applications can resolve [`TypefaceRole`] to any
//! family.
use gpui::{Rems, rems};

/// The px size of 1rem at the 100% text scale.
pub const BASE_PX: f32 = 14.;

/// Express a 100%-scale px size in rems.
pub const fn rem(px: f32) -> Rems {
    rems(px / BASE_PX)
}

// Legacy size constants. Each is defined from the M3 role it maps to where
// the sizes agree; prefer `TypeRole::<role>.scale()` in new code so line
// height and weight come with the size.

/// Supporting and auxiliary text: body-small (12px).
pub const TEXT_AUX: Rems = TypeRole::BodySmall.scale().font_size;
/// Body and control labels: body-medium (14px).
pub const TEXT_BODY: Rems = TypeRole::BodyMedium.scale().font_size;
/// Long-form reading: body-large (16px).
pub const TEXT_READER: Rems = TypeRole::BodyLarge.scale().font_size;
/// Card and dialog titles: title-large (22px).
pub const TEXT_TITLE: Rems = TypeRole::TitleLarge.scale().font_size;
/// Section headings on a page: title-large (22px).
pub const TEXT_SECTION: Rems = TypeRole::TitleLarge.scale().font_size;
/// Page titles use the reference headline-small role (24px / 32px).
pub const TEXT_HEADLINE: Rems = TypeRole::HeadlineSmall.scale().font_size;
pub const HEADLINE_LINE_HEIGHT: Rems = TypeRole::HeadlineSmall.scale().line_height;
/// Component token: the label of a regular tile and of a choice tile
/// ([`crate::components::TileScale::Regular`], [`crate::components::ChoiceTile`]),
/// one step larger than navigation: body-large (16px).
pub const TEXT_SETTINGS_BODY: Rems = TypeRole::BodyLarge.scale().font_size;
/// Component token: the supporting text of a regular tile and of a choice
/// tile: body-medium (14px).
pub const TEXT_SETTINGS_AUX: Rems = TEXT_BODY;

/// The M3 role each legacy size constant maps to, by constant name.
pub const LEGACY_ROLES: [(&str, TypeRole); 8] = [
    ("TEXT_AUX", TypeRole::BodySmall),
    ("TEXT_BODY", TypeRole::BodyMedium),
    ("TEXT_READER", TypeRole::BodyLarge),
    ("TEXT_TITLE", TypeRole::TitleLarge),
    ("TEXT_SECTION", TypeRole::TitleLarge),
    ("TEXT_HEADLINE", TypeRole::HeadlineSmall),
    ("TEXT_SETTINGS_BODY", TypeRole::BodyLarge),
    ("TEXT_SETTINGS_AUX", TypeRole::BodyMedium),
];

/// A role in the reference Material type scale.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypeRole {
    DisplayLarge,
    DisplayMedium,
    DisplaySmall,
    HeadlineLarge,
    HeadlineMedium,
    HeadlineSmall,
    TitleLarge,
    TitleMedium,
    TitleSmall,
    BodyLarge,
    BodyMedium,
    BodySmall,
    LabelLarge,
    LabelMedium,
    LabelSmall,
}

/// Independently customizable reference typeface families.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypefaceRole {
    Brand,
    Plain,
}

/// Reference typography metrics at 100% text scale.
///
/// Sizes and tracking are in rems so they follow the application's text scale.
/// `letter_spacing` paints through GPUI's text style, for text elements. Kit's
/// editors shape their own lines and do not apply it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeScale {
    pub typeface: TypefaceRole,
    /// Default family; applications can resolve `typeface` to a custom family.
    pub font_family: &'static str,
    pub font_size: Rems,
    pub line_height: Rems,
    pub letter_spacing: Rems,
    pub font_weight: u16,
}

impl TypeRole {
    pub const ALL: [Self; 15] = [
        Self::DisplayLarge,
        Self::DisplayMedium,
        Self::DisplaySmall,
        Self::HeadlineLarge,
        Self::HeadlineMedium,
        Self::HeadlineSmall,
        Self::TitleLarge,
        Self::TitleMedium,
        Self::TitleSmall,
        Self::BodyLarge,
        Self::BodyMedium,
        Self::BodySmall,
        Self::LabelLarge,
        Self::LabelMedium,
        Self::LabelSmall,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::DisplayLarge => "display-large",
            Self::DisplayMedium => "display-medium",
            Self::DisplaySmall => "display-small",
            Self::HeadlineLarge => "headline-large",
            Self::HeadlineMedium => "headline-medium",
            Self::HeadlineSmall => "headline-small",
            Self::TitleLarge => "title-large",
            Self::TitleMedium => "title-medium",
            Self::TitleSmall => "title-small",
            Self::BodyLarge => "body-large",
            Self::BodyMedium => "body-medium",
            Self::BodySmall => "body-small",
            Self::LabelLarge => "label-large",
            Self::LabelMedium => "label-medium",
            Self::LabelSmall => "label-small",
        }
    }

    /// The kit's `Static/<Role>/Font` variables: display, headline and
    /// title-large use the brand face; title-medium, title-small, body and
    /// label the plain.
    pub const fn typeface(self) -> TypefaceRole {
        match self {
            Self::TitleMedium
            | Self::TitleSmall
            | Self::BodyLarge
            | Self::BodyMedium
            | Self::BodySmall
            | Self::LabelLarge
            | Self::LabelMedium
            | Self::LabelSmall => TypefaceRole::Plain,
            _ => TypefaceRole::Brand,
        }
    }

    /// The role's metrics (size, line height and tracking in px at 100%).
    pub const fn scale(self) -> TypeScale {
        let (size, height, tracking, weight) = match self {
            Self::DisplayLarge => (57.0, 64.0, -0.25, 400),
            Self::DisplayMedium => (45.0, 52.0, 0.0, 400),
            Self::DisplaySmall => (36.0, 44.0, 0.0, 400),
            Self::HeadlineLarge => (32.0, 40.0, 0.0, 400),
            Self::HeadlineMedium => (28.0, 36.0, 0.0, 400),
            Self::HeadlineSmall => (24.0, 32.0, 0.0, 400),
            Self::TitleLarge => (22.0, 28.0, 0.0, 400),
            Self::TitleMedium => (16.0, 24.0, 0.15, 500),
            Self::TitleSmall => (14.0, 20.0, 0.1, 500),
            Self::BodyLarge => (16.0, 24.0, 0.5, 400),
            Self::BodyMedium => (14.0, 20.0, 0.25, 400),
            Self::BodySmall => (12.0, 16.0, 0.4, 400),
            Self::LabelLarge => (14.0, 20.0, 0.1, 500),
            Self::LabelMedium => (12.0, 16.0, 0.5, 500),
            Self::LabelSmall => (11.0, 16.0, 0.5, 500),
        };
        self.metrics(size, height, tracking, weight)
    }

    /// The role's M3 Expressive emphasized style (the kit's
    /// `M3/<group>/<size>-emphasized`): the same face, size, line height and
    /// tracking with the `Weight-emphasized` weight, Medium (500) for regular
    /// roles and SemiBold (600) for medium ones.
    pub const fn emphasized(self) -> TypeScale {
        let base = self.scale();
        TypeScale {
            font_weight: if base.font_weight == 400 { 500 } else { 600 },
            ..base
        }
    }

    const fn metrics(self, size: f32, height: f32, tracking: f32, weight: u16) -> TypeScale {
        TypeScale {
            typeface: self.typeface(),
            font_family: "Roboto",
            font_size: rem(size),
            line_height: rem(height),
            letter_spacing: rem(tracking),
            font_weight: weight,
        }
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;

    #[test]
    fn legacy_sizes_are_their_mapped_roles() {
        let sizes = [
            TEXT_AUX,
            TEXT_BODY,
            TEXT_READER,
            TEXT_TITLE,
            TEXT_SECTION,
            TEXT_HEADLINE,
            TEXT_SETTINGS_BODY,
            TEXT_SETTINGS_AUX,
        ];
        for (size, (name, role)) in sizes.into_iter().zip(LEGACY_ROLES) {
            assert_eq!(size, role.scale().font_size, "{name}");
        }
    }

    #[test]
    fn all_roles_match_the_figma_kit_text_styles() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/figma-typography.json"))
                .unwrap();
        assert_eq!(fixture["font_theme"]["brand"], "Roboto");
        assert_eq!(fixture["font_theme"]["plain"], "Roboto");
        for (key, style) in [
            ("roles", TypeRole::scale as fn(TypeRole) -> TypeScale),
            ("emphasized", TypeRole::emphasized),
        ] {
            let roles = fixture[key].as_array().unwrap();
            assert_eq!(roles.len(), TypeRole::ALL.len());
            for role in TypeRole::ALL {
                let expected = roles.iter().find(|row| row["name"] == role.name()).unwrap();
                assert_matches(style(role), expected, role.name());
            }
        }
    }

    fn assert_matches(scale: TypeScale, expected: &serde_json::Value, name: &str) {
        assert_eq!(
            scale.typeface,
            if expected["typeface"] == "brand" {
                TypefaceRole::Brand
            } else {
                TypefaceRole::Plain
            },
            "{name} typeface"
        );
        assert_eq!(
            scale.font_family,
            expected["font_family"].as_str().unwrap(),
            "{name} family"
        );
        assert_eq!(
            scale.font_weight,
            expected["font_weight"].as_u64().unwrap() as u16
        );
        for (actual, key) in [
            (scale.font_size, "font_size"),
            (scale.line_height, "line_height"),
            (scale.letter_spacing, "letter_spacing"),
        ] {
            assert_eq!(
                actual,
                rem(expected[key].as_f64().unwrap() as f32),
                "{name} {key}"
            );
        }
    }
}
