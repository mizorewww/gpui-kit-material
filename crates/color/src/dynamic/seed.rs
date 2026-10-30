//! A seed color and style as a complete [`Scheme`] for the whole application.
//!
//! Every role material-color-utilities defines comes from it unchanged. The
//! status roles it does not define (`SUCCESS`, `WARNING`, `INFO` and their
//! on-roles) follow Material's custom-color recipe: the baseline theme's status
//! color harmonized toward the seed, at the tone of an accent, held to the same
//! contrast curves as the accents. The kit's application extensions are derived
//! as for reference themes.
use super::blend::harmonize;
use super::color::{ContrastCurve, DynamicRole, foreground_tone, with_tone_memo};
use super::contrast::ratio_of_tones;
use super::hct::Hct;
use super::palette::TonalPalette;
use super::scheme::{DynamicScheme, SpecVersion, Variant};
use crate::scheme::Scheme;
use crate::themes::{Contrast, REFERENCE_ROLES, ThemeId, ThemeMode};
use crate::{Color, ColorRole};

/// What a generated theme is made from.
///
/// ```
/// use gpui_kit_material_color::dynamic::{SeedScheme, Variant};
/// use gpui_kit_material_color::{Color, ColorRole};
///
/// let scheme = SeedScheme::new(0x6750a4).variant(Variant::TonalSpot).scheme();
/// assert_eq!(scheme.get(ColorRole::PRIMARY), Color::hex(0x65558f));
/// assert!(!scheme.is_dark());
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeedScheme {
    /// The seed as `0xRRGGBB`.
    pub seed: u32,
    /// The style.
    pub variant: Variant,
    /// Dark mode.
    pub dark: bool,
    /// -1 is minimum contrast, 0 standard, 0.5 medium, 1 high.
    pub contrast: f64,
    /// The color spec. Variants without a 2025 spec resolve as 2021.
    pub spec: SpecVersion,
}

impl SeedScheme {
    /// A light tonal-spot scheme at standard contrast on the 2021 spec.
    pub const fn new(seed: u32) -> Self {
        Self {
            seed: seed & 0xff_ffff,
            variant: Variant::TonalSpot,
            dark: false,
            contrast: 0.,
            spec: SpecVersion::Spec2021,
        }
    }

    /// Set the style.
    pub const fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the mode.
    pub const fn dark(mut self, dark: bool) -> Self {
        self.dark = dark;
        self
    }

    /// Set the contrast level, from -1 to 1.
    pub const fn contrast(mut self, contrast: f64) -> Self {
        self.contrast = contrast;
        self
    }

    /// Set the color spec.
    pub const fn spec(mut self, spec: SpecVersion) -> Self {
        self.spec = spec;
        self
    }

    /// Parse `#RRGGBB`, `RRGGBB`, `#RGB` or `RGB`.
    pub fn parse_hex(text: &str) -> Option<u32> {
        let digits = text.trim();
        let digits = digits.strip_prefix('#').unwrap_or(digits);
        if !digits.chars().all(|digit| digit.is_ascii_hexdigit()) {
            return None;
        }
        let value = u32::from_str_radix(digits, 16).ok()?;
        match digits.len() {
            6 => Some(value),
            3 => {
                let double = |nibble: u32| (nibble & 0xf) * 0x11;
                Some(double(value >> 8) << 16 | double(value >> 4) << 8 | double(value))
            }
            _ => None,
        }
    }

    /// The resolved palettes and roles.
    pub fn dynamic(&self) -> DynamicScheme {
        DynamicScheme::new(
            Hct::from_argb(0xff00_0000 | self.seed),
            self.variant,
            self.dark,
            self.contrast,
            self.spec,
        )
    }

    /// Every [`ColorRole`] of the generated theme.
    pub fn scheme(&self) -> Scheme {
        let dynamic = self.dynamic();
        let colors = with_tone_memo(|| {
            REFERENCE_ROLES.map(|role| {
                let argb = match upstream_role(role) {
                    Some(upstream) => dynamic.argb(upstream),
                    None => self.status(&dynamic, role),
                };
                argb & 0xff_ffff
            })
        });
        Scheme::from_reference(&colors, self.dark)
    }

    fn status(&self, dynamic: &DynamicScheme, role: ColorRole) -> u32 {
        let (source_role, foreground) = match role {
            ColorRole::SUCCESS => (ColorRole::SUCCESS, false),
            ColorRole::ON_SUCCESS => (ColorRole::SUCCESS, true),
            ColorRole::WARNING => (ColorRole::WARNING, false),
            ColorRole::ON_WARNING => (ColorRole::WARNING, true),
            ColorRole::ON_INFO => (ColorRole::INFO, true),
            _ => (ColorRole::INFO, false),
        };
        let baseline = ThemeId::Baseline.scheme(ThemeMode::Light, Contrast::Standard);
        let palette = TonalPalette::from_argb(harmonize(
            argb(baseline.get(source_role)),
            0xff00_0000 | self.seed,
        ));
        // The accents' tones and contrast curves in the 2021 spec.
        let surface = if self.dark {
            DynamicRole::SurfaceBright
        } else {
            DynamicRole::SurfaceDim
        };
        let mut tone = if self.dark { 80. } else { 40. };
        let needed = ContrastCurve::new(3., 4.5, 7., 7.).get(dynamic.contrast_level);
        let surface_tone = dynamic.tone(surface);
        if ratio_of_tones(surface_tone, tone) < needed {
            tone = foreground_tone(surface_tone, needed);
        }
        if !foreground {
            return palette.tone(tone);
        }
        let mut on_tone = if self.dark { 20. } else { 100. };
        let needed = ContrastCurve::new(4.5, 7., 11., 21.).get(dynamic.contrast_level);
        if ratio_of_tones(tone, on_tone) < needed {
            on_tone = foreground_tone(tone, needed);
        }
        palette.tone(on_tone)
    }
}

fn argb(color: Color) -> u32 {
    let channel = |value: f32| (value.clamp(0., 1.) * 255.).round() as u32;
    0xff00_0000 | channel(color.r) << 16 | channel(color.g) << 8 | channel(color.b)
}

/// The upstream role of a reference role; `None` for the status roles.
fn upstream_role(role: ColorRole) -> Option<DynamicRole> {
    use DynamicRole as D;
    Some(match role {
        ColorRole::ERROR => D::Error,
        ColorRole::ERROR_CONTAINER => D::ErrorContainer,
        ColorRole::INVERSE_ON_SURFACE => D::InverseOnSurface,
        ColorRole::INVERSE_PRIMARY => D::InversePrimary,
        ColorRole::INVERSE_SURFACE => D::InverseSurface,
        ColorRole::ON_ERROR => D::OnError,
        ColorRole::ON_ERROR_CONTAINER => D::OnErrorContainer,
        ColorRole::ON_PRIMARY => D::OnPrimary,
        ColorRole::ON_PRIMARY_CONTAINER => D::OnPrimaryContainer,
        ColorRole::ON_PRIMARY_FIXED => D::OnPrimaryFixed,
        ColorRole::ON_PRIMARY_FIXED_VARIANT => D::OnPrimaryFixedVariant,
        ColorRole::ON_SECONDARY => D::OnSecondary,
        ColorRole::ON_SECONDARY_CONTAINER => D::OnSecondaryContainer,
        ColorRole::ON_SECONDARY_FIXED => D::OnSecondaryFixed,
        ColorRole::ON_SECONDARY_FIXED_VARIANT => D::OnSecondaryFixedVariant,
        ColorRole::ON_SURFACE => D::OnSurface,
        ColorRole::ON_SURFACE_VARIANT => D::OnSurfaceVariant,
        ColorRole::ON_TERTIARY => D::OnTertiary,
        ColorRole::ON_TERTIARY_CONTAINER => D::OnTertiaryContainer,
        ColorRole::ON_TERTIARY_FIXED => D::OnTertiaryFixed,
        ColorRole::ON_TERTIARY_FIXED_VARIANT => D::OnTertiaryFixedVariant,
        ColorRole::OUTLINE => D::Outline,
        ColorRole::OUTLINE_VARIANT => D::OutlineVariant,
        ColorRole::PRIMARY => D::Primary,
        ColorRole::PRIMARY_CONTAINER => D::PrimaryContainer,
        ColorRole::PRIMARY_FIXED => D::PrimaryFixed,
        ColorRole::PRIMARY_FIXED_DIM => D::PrimaryFixedDim,
        ColorRole::SCRIM => D::Scrim,
        ColorRole::SECONDARY => D::Secondary,
        ColorRole::SECONDARY_CONTAINER => D::SecondaryContainer,
        ColorRole::SECONDARY_FIXED => D::SecondaryFixed,
        ColorRole::SECONDARY_FIXED_DIM => D::SecondaryFixedDim,
        ColorRole::SHADOW => D::Shadow,
        ColorRole::SURFACE => D::Surface,
        ColorRole::SURFACE_BRIGHT => D::SurfaceBright,
        ColorRole::SURFACE_CONTAINER => D::SurfaceContainer,
        ColorRole::SURFACE_CONTAINER_HIGH => D::SurfaceContainerHigh,
        ColorRole::SURFACE_CONTAINER_HIGHEST => D::SurfaceContainerHighest,
        ColorRole::SURFACE_CONTAINER_LOW => D::SurfaceContainerLow,
        ColorRole::SURFACE_CONTAINER_LOWEST => D::SurfaceContainerLowest,
        ColorRole::SURFACE_DIM => D::SurfaceDim,
        ColorRole::SURFACE_VARIANT => D::SurfaceVariant,
        ColorRole::TERTIARY => D::Tertiary,
        ColorRole::TERTIARY_CONTAINER => D::TertiaryContainer,
        ColorRole::TERTIARY_FIXED => D::TertiaryFixed,
        ColorRole::TERTIARY_FIXED_DIM => D::TertiaryFixedDim,
        _ => return None,
    })
}
