//! Semantic color roles and their derivation from a source palette.
//!
//! Reference themes use the complete md3.io color role set. Compatibility
//! constants map older names onto the same role storage; application-specific
//! status containers and selection colors remain explicit extensions.
use crate::{Color, PaletteId, dynamic::blend::harmonize};

macro_rules! roles {
    ($($(#[$doc:meta])* $name:ident),+ $(,)?) => {
        /// A semantic color role. Components paint with roles, never with a
        /// palette's stored colors.
        #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[repr(usize)]
        pub enum ColorRole { $($(#[$doc])* $name),+ }

        impl ColorRole {
            /// Every role, in declaration order.
            pub const ALL: &[ColorRole] = &[$(ColorRole::$name),+];
        }
    }
}

roles!(
    ERROR,
    ERROR_CONTAINER,
    INFO,
    INVERSE_ON_SURFACE,
    INVERSE_PRIMARY,
    INVERSE_SURFACE,
    ON_ERROR,
    ON_ERROR_CONTAINER,
    ON_INFO,
    ON_PRIMARY,
    ON_PRIMARY_CONTAINER,
    ON_PRIMARY_FIXED,
    ON_PRIMARY_FIXED_VARIANT,
    ON_SECONDARY,
    ON_SECONDARY_CONTAINER,
    ON_SECONDARY_FIXED,
    ON_SECONDARY_FIXED_VARIANT,
    ON_SUCCESS,
    ON_SURFACE,
    ON_SURFACE_VARIANT,
    ON_TERTIARY,
    ON_TERTIARY_CONTAINER,
    ON_TERTIARY_FIXED,
    ON_TERTIARY_FIXED_VARIANT,
    ON_WARNING,
    OUTLINE,
    OUTLINE_VARIANT,
    PRIMARY,
    PRIMARY_CONTAINER,
    PRIMARY_FIXED,
    PRIMARY_FIXED_DIM,
    SCRIM,
    SECONDARY,
    SECONDARY_CONTAINER,
    SECONDARY_FIXED,
    SECONDARY_FIXED_DIM,
    SHADOW,
    SUCCESS,
    SURFACE,
    SURFACE_BRIGHT,
    SURFACE_CONTAINER,
    SURFACE_CONTAINER_HIGH,
    SURFACE_CONTAINER_HIGHEST,
    SURFACE_CONTAINER_LOW,
    SURFACE_CONTAINER_LOWEST,
    SURFACE_DIM,
    SURFACE_VARIANT,
    TERTIARY,
    TERTIARY_CONTAINER,
    TERTIARY_FIXED,
    TERTIARY_FIXED_DIM,
    WARNING,
    INSET,
    FAINT,
    CARD_LINE,
    SUCCESS_BG,
    WARNING_BG,
    FIND_HIGHLIGHT,
    FIND_CURRENT,
    FIND_CURRENT_LINE,
    SELECTION,
    READING_SELECTION,
    /// The container of [`TintFamily::Orange`].
    TINT_ORANGE,
    /// The icon on [`ColorRole::TINT_ORANGE`].
    ON_TINT_ORANGE,
    /// The container of [`TintFamily::Blue`].
    TINT_BLUE,
    /// The icon on [`ColorRole::TINT_BLUE`].
    ON_TINT_BLUE,
    /// The container of [`TintFamily::Teal`].
    TINT_TEAL,
    /// The icon on [`ColorRole::TINT_TEAL`].
    ON_TINT_TEAL,
    /// The container of [`TintFamily::Amber`].
    TINT_AMBER,
    /// The icon on [`ColorRole::TINT_AMBER`].
    ON_TINT_AMBER,
    /// The container of [`TintFamily::Green`].
    TINT_GREEN,
    /// The icon on [`ColorRole::TINT_GREEN`].
    ON_TINT_GREEN,
    /// The container of [`TintFamily::Pink`].
    TINT_PINK,
    /// The icon on [`ColorRole::TINT_PINK`].
    ON_TINT_PINK,
    /// The container of [`TintFamily::Purple`].
    TINT_PURPLE,
    /// The icon on [`ColorRole::TINT_PURPLE`].
    ON_TINT_PURPLE,
);

/// A hue family for telling categories apart, in the manner of Material's custom
/// colors: each family's design color is harmonized toward the scheme's primary,
/// and its container and content are derived from the scheme's surface and text.
/// A scheme's own container roles cannot do this: the source palettes derive all of
/// them from one accent, so they would all be one color.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TintFamily {
    Orange,
    Blue,
    Teal,
    Amber,
    Green,
    Pink,
    Purple,
}

impl TintFamily {
    pub const ALL: [Self; 7] = [
        Self::Orange,
        Self::Blue,
        Self::Teal,
        Self::Amber,
        Self::Green,
        Self::Pink,
        Self::Purple,
    ];

    /// The family's design color, before harmonizing.
    const fn design_color(self) -> u32 {
        match self {
            Self::Orange => 0xd9692b,
            Self::Blue => 0x2f6fe0,
            Self::Teal => 0x1b8ea6,
            Self::Amber => 0xc79a12,
            Self::Green => 0x2f9a52,
            Self::Pink => 0xc7428a,
            Self::Purple => 0x7a5cc7,
        }
    }

    /// The family's container role and the content role on it.
    pub const fn roles(self) -> (ColorRole, ColorRole) {
        match self {
            Self::Orange => (ColorRole::TINT_ORANGE, ColorRole::ON_TINT_ORANGE),
            Self::Blue => (ColorRole::TINT_BLUE, ColorRole::ON_TINT_BLUE),
            Self::Teal => (ColorRole::TINT_TEAL, ColorRole::ON_TINT_TEAL),
            Self::Amber => (ColorRole::TINT_AMBER, ColorRole::ON_TINT_AMBER),
            Self::Green => (ColorRole::TINT_GREEN, ColorRole::ON_TINT_GREEN),
            Self::Pink => (ColorRole::TINT_PINK, ColorRole::ON_TINT_PINK),
            Self::Purple => (ColorRole::TINT_PURPLE, ColorRole::ON_TINT_PURPLE),
        }
    }

    /// The container and content over `surface`, with `text` as the scheme's text.
    /// The mix keeps the content's contrast on the container in light and dark.
    fn derive(self, surface: Color, text: Color, primary: Color, dark: bool) -> (Color, Color) {
        let hue = Color::argb(harmonize(
            0xff00_0000 | self.design_color(),
            primary.to_argb(),
        ));
        (
            surface.blend(hue, if dark { 0.30 } else { 0.24 }),
            hue.blend(text, if dark { 0.5 } else { 0.45 }),
        )
    }
}

// Compatibility names share storage with their semantic M3 role. Application
// extensions above remain independent; they are not claimed as M3 scheme roles.
impl ColorRole {
    /// Legacy name for [`ColorRole::SURFACE_CONTAINER`], the window ground.
    #[deprecated(note = "use `ColorRole::SURFACE_CONTAINER`")]
    pub const CANVAS: Self = Self::SURFACE_CONTAINER;
    /// Legacy name for [`ColorRole::SURFACE_CONTAINER_HIGHEST`]. Despite the
    /// name, this is the *highest* container, not
    /// [`ColorRole::SURFACE_CONTAINER_HIGH`].
    #[deprecated(note = "use `ColorRole::SURFACE_CONTAINER_HIGHEST`")]
    pub const SURFACE_HIGH: Self = Self::SURFACE_CONTAINER_HIGHEST;
    /// Legacy name for [`ColorRole::ON_SURFACE`], primary text and icons.
    #[deprecated(note = "use `ColorRole::ON_SURFACE`")]
    pub const INK: Self = Self::ON_SURFACE;
    /// Legacy name for [`ColorRole::ON_SURFACE_VARIANT`], secondary text
    /// and icons.
    #[deprecated(note = "use `ColorRole::ON_SURFACE_VARIANT`")]
    pub const GRAY: Self = Self::ON_SURFACE_VARIANT;
    /// Legacy name for [`ColorRole::OUTLINE_VARIANT`], decorative dividers.
    #[deprecated(note = "use `ColorRole::OUTLINE_VARIANT`")]
    pub const HAIRLINE: Self = Self::OUTLINE_VARIANT;
    /// Legacy name for [`ColorRole::OUTLINE`], control boundaries such as
    /// outlined fields and unselected checkboxes.
    #[deprecated(note = "use `ColorRole::OUTLINE`")]
    pub const CONTROL: Self = Self::OUTLINE;
    /// Legacy name for [`ColorRole::PRIMARY`].
    #[deprecated(note = "use `ColorRole::PRIMARY`")]
    pub const ACCENT: Self = Self::PRIMARY;
    /// Legacy name for [`ColorRole::ERROR`].
    #[deprecated(note = "use `ColorRole::ERROR`")]
    pub const DANGER: Self = Self::ERROR;
    /// Legacy name for [`ColorRole::ERROR_CONTAINER`].
    #[deprecated(note = "use `ColorRole::ERROR_CONTAINER`")]
    pub const DANGER_BG: Self = Self::ERROR_CONTAINER;
}

/// The number of roles in a [`Scheme`].
pub const ROLE_COUNT: usize = ColorRole::ALL.len();

/// One value for every [`ColorRole`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scheme {
    colors: [Color; ROLE_COUNT],
    dark: bool,
}

impl Scheme {
    /// Override one semantic role in an application-defined scheme.
    pub fn with_role(mut self, role: ColorRole, color: Color) -> Self {
        self.colors[role as usize] = color;
        self
    }

    /// Derive every role from a built-in palette.
    pub fn from_palette(id: PaletteId) -> Self {
        let p = id.colors();
        let dark = id.is_dark();
        let hex = Color::hex;
        let tint = |foreground: u32, amount| hex(p.surface).blend(hex(foreground), amount);
        // The window ground is M3's surface container: the palette's canvas
        // with a trace of its accent, so surface tiles read as a layer above it.
        let canvas = hex(p.canvas).blend(hex(p.accent), if dark { 0.03 } else { 0.045 });
        let colors = std::array::from_fn(|index| match ColorRole::ALL[index] {
            ColorRole::SURFACE_CONTAINER => canvas,
            // Dark palettes' surfaces sit only a step above their canvas; lift
            // them as M3 lifts dark surface containers, so tiles and cards
            // read as layers.
            ColorRole::SURFACE => {
                if dark {
                    hex(p.surface).blend(hex(p.text), 0.045)
                } else {
                    hex(p.surface)
                }
            }
            // Dark palettes raise surfaces by lightening them, so their
            // highest container builds on the surface rather than the
            // darker ground.
            ColorRole::SURFACE_CONTAINER_HIGHEST => {
                if dark {
                    hex(p.surface).blend(hex(p.text), 0.13)
                } else {
                    canvas.blend(hex(p.text), 0.06)
                }
            }
            ColorRole::INSET => hex(p.inset),
            ColorRole::ON_SURFACE => hex(p.text),
            ColorRole::ON_SURFACE_VARIANT => hex(p.muted),
            ColorRole::FAINT => hex(p.subtle),
            ColorRole::OUTLINE_VARIANT => hex(p.border),
            // M3 cards are separated by tone rather than a full-strength outline.
            ColorRole::CARD_LINE => hex(p.surface).blend(hex(p.border), 0.55),
            ColorRole::OUTLINE => hex(p.control),
            ColorRole::PRIMARY => hex(p.accent),
            ColorRole::ON_PRIMARY => hex(p.on_accent),
            // Container roles derive from each palette's own accent and text,
            // so every preset keeps its identity.
            ColorRole::SECONDARY_CONTAINER => tint(p.accent, if dark { 0.26 } else { 0.24 }),
            ColorRole::ON_SECONDARY_CONTAINER => {
                hex(p.text).blend(hex(p.accent), if dark { 0.30 } else { 0.40 })
            }
            ColorRole::SUCCESS => hex(p.success),
            ColorRole::SUCCESS_BG => tint(p.success, 0.12),
            ColorRole::WARNING => hex(p.warning),
            ColorRole::WARNING_BG => tint(p.warning, 0.12),
            ColorRole::ERROR => hex(p.danger),
            ColorRole::ERROR_CONTAINER => tint(p.danger, 0.12),
            ColorRole::SELECTION => hex(p.selection),
            // Selected text: the accent laid over whatever surface holds the
            // text, so it stays visible on tinted panels.
            ColorRole::READING_SELECTION => hex(p.accent).alpha(if dark { 0.42 } else { 0.30 }),
            ColorRole::FIND_HIGHLIGHT => tint(p.warning, 0.16),
            ColorRole::FIND_CURRENT => tint(p.warning, 0.32),
            ColorRole::FIND_CURRENT_LINE => hex(p.warning),
            // Legacy presets predate full M3 schemes. These are adapters from
            // their existing colors, not seed-generated HCT schemes.
            ColorRole::PRIMARY_CONTAINER | ColorRole::TERTIARY_CONTAINER => tint(p.accent, 0.24),
            ColorRole::ON_PRIMARY_CONTAINER | ColorRole::ON_TERTIARY_CONTAINER => hex(p.text),
            ColorRole::SECONDARY | ColorRole::TERTIARY => hex(p.accent),
            ColorRole::ON_SECONDARY | ColorRole::ON_TERTIARY => hex(p.on_accent),
            ColorRole::PRIMARY_FIXED | ColorRole::SECONDARY_FIXED | ColorRole::TERTIARY_FIXED => {
                hex(p.accent).blend(hex(0xffffff), 0.8)
            }
            ColorRole::PRIMARY_FIXED_DIM
            | ColorRole::SECONDARY_FIXED_DIM
            | ColorRole::TERTIARY_FIXED_DIM => hex(p.accent).blend(hex(0xffffff), 0.6),
            ColorRole::ON_PRIMARY_FIXED
            | ColorRole::ON_SECONDARY_FIXED
            | ColorRole::ON_TERTIARY_FIXED => hex(p.accent).blend(hex(0), 0.8),
            ColorRole::ON_PRIMARY_FIXED_VARIANT
            | ColorRole::ON_SECONDARY_FIXED_VARIANT
            | ColorRole::ON_TERTIARY_FIXED_VARIANT => hex(p.accent).blend(hex(0), 0.4),
            ColorRole::SURFACE_DIM => hex(p.canvas),
            ColorRole::SURFACE_BRIGHT => {
                if dark {
                    hex(p.surface).blend(hex(p.text), 0.18)
                } else {
                    hex(p.surface)
                }
            }
            ColorRole::SURFACE_VARIANT | ColorRole::SURFACE_CONTAINER_HIGH => {
                tint(p.text, if dark { 0.10 } else { 0.06 })
            }
            ColorRole::SURFACE_CONTAINER_LOWEST => {
                if dark {
                    hex(p.canvas)
                } else {
                    hex(p.surface)
                }
            }
            ColorRole::SURFACE_CONTAINER_LOW => tint(p.text, 0.025),
            ColorRole::INVERSE_SURFACE => hex(p.text),
            ColorRole::INVERSE_ON_SURFACE => hex(p.surface),
            ColorRole::INVERSE_PRIMARY => hex(p.accent).blend(hex(p.surface), 0.4),
            ColorRole::ON_ERROR => hex(p.on_accent),
            ColorRole::ON_ERROR_CONTAINER => hex(p.danger),
            ColorRole::ON_SUCCESS | ColorRole::ON_WARNING => hex(p.on_accent),
            ColorRole::INFO => hex(p.accent),
            ColorRole::ON_INFO => hex(p.on_accent),
            ColorRole::SHADOW | ColorRole::SCRIM => hex(0),
            // `with_tints` below derives these from the finished roles.
            ColorRole::TINT_ORANGE
            | ColorRole::ON_TINT_ORANGE
            | ColorRole::TINT_BLUE
            | ColorRole::ON_TINT_BLUE
            | ColorRole::TINT_TEAL
            | ColorRole::ON_TINT_TEAL
            | ColorRole::TINT_AMBER
            | ColorRole::ON_TINT_AMBER
            | ColorRole::TINT_GREEN
            | ColorRole::ON_TINT_GREEN
            | ColorRole::TINT_PINK
            | ColorRole::ON_TINT_PINK
            | ColorRole::TINT_PURPLE
            | ColorRole::ON_TINT_PURPLE => hex(0),
        });
        Self { colors, dark }.with_tints()
    }

    /// Derive every [`TintFamily`]'s roles from the scheme's surface, text and primary.
    fn with_tints(mut self) -> Self {
        let (surface, text, primary) = (
            self.get(ColorRole::SURFACE),
            self.get(ColorRole::ON_SURFACE),
            self.get(ColorRole::PRIMARY),
        );
        for family in TintFamily::ALL {
            let (container, content) = family.derive(surface, text, primary, self.dark);
            let (container_role, content_role) = family.roles();
            self = self
                .with_role(container_role, container)
                .with_role(content_role, content);
        }
        self
    }

    /// Construct an exact reference scheme and derive only the kit-specific
    /// extensions (selection, find highlights, and status containers).
    pub(crate) fn from_reference(colors: &[u32; 52], dark: bool) -> Self {
        let mut scheme = Self {
            colors: [Color::hex(0); ROLE_COUNT],
            dark,
        };
        for (role, value) in crate::themes::REFERENCE_ROLES.iter().zip(colors) {
            scheme.colors[*role as usize] = Color::hex(*value);
        }
        let surface = scheme.get(ColorRole::SURFACE);
        let accent = scheme.get(ColorRole::PRIMARY);
        let warning = scheme.get(ColorRole::WARNING);
        scheme
            .with_role(
                ColorRole::INSET,
                scheme.get(ColorRole::SURFACE_CONTAINER_LOWEST),
            )
            .with_role(ColorRole::FAINT, scheme.get(ColorRole::OUTLINE))
            .with_role(ColorRole::CARD_LINE, scheme.get(ColorRole::OUTLINE_VARIANT))
            .with_role(
                ColorRole::SUCCESS_BG,
                surface.blend(scheme.get(ColorRole::SUCCESS), 0.12),
            )
            .with_role(ColorRole::WARNING_BG, surface.blend(warning, 0.12))
            .with_role(ColorRole::FIND_HIGHLIGHT, surface.blend(warning, 0.16))
            .with_role(ColorRole::FIND_CURRENT, surface.blend(warning, 0.32))
            .with_role(ColorRole::FIND_CURRENT_LINE, warning)
            .with_role(
                ColorRole::SELECTION,
                scheme.get(ColorRole::PRIMARY_CONTAINER),
            )
            .with_role(
                ColorRole::READING_SELECTION,
                accent.alpha(if dark { 0.42 } else { 0.30 }),
            )
            .with_tints()
    }

    pub fn get(&self, role: ColorRole) -> Color {
        self.colors[role as usize]
    }

    /// Whether the scheme was derived from a dark palette.
    pub fn is_dark(&self) -> bool {
        self.dark
    }

    /// Every role interpolated toward `to`; used for palette transitions.
    /// The result takes `to`'s darkness once it is more than halfway there.
    pub fn blend(&self, to: &Scheme, t: f32) -> Scheme {
        Scheme {
            colors: std::array::from_fn(|i| self.colors[i].blend(to.colors[i], t)),
            dark: if t < 0.5 { self.dark } else { to.dark },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::ALL;

    /// Every scheme a family can land in: the palettes, a reference theme and a seed.
    fn every_kind_of_scheme() -> Vec<(String, Scheme)> {
        let mut schemes: Vec<_> = ALL
            .into_iter()
            .map(|id| (id.name().to_owned(), Scheme::from_palette(id)))
            .collect();
        for dark in [false, true] {
            let mode = if dark {
                crate::themes::ThemeMode::Dark
            } else {
                crate::themes::ThemeMode::Light
            };
            schemes.push((
                format!("baseline {dark}"),
                crate::themes::ThemeId::Baseline.scheme(mode, crate::themes::Contrast::Standard),
            ));
            schemes.push((
                format!("seed {dark}"),
                crate::dynamic::SeedScheme::new(0x6750a4)
                    .dark(dark)
                    .scheme(),
            ));
        }
        schemes
    }

    #[test]
    fn tint_families_stay_apart_and_their_icons_read_in_every_scheme() {
        for (name, scheme) in every_kind_of_scheme() {
            let surface = scheme.get(ColorRole::SURFACE);
            for (index, family) in TintFamily::ALL.into_iter().enumerate() {
                let (container, content) = family.roles();
                let (container, content) = (scheme.get(container), scheme.get(content));
                assert_ne!(container, surface, "{name} {family:?}");
                assert!(
                    contrast(container, content) >= 3.,
                    "{name} {family:?}: {}",
                    contrast(container, content)
                );
                for other in &TintFamily::ALL[index + 1..] {
                    assert_ne!(
                        container,
                        scheme.get(other.roles().0),
                        "{name}: {family:?} and {other:?}"
                    );
                }
            }
        }
    }

    fn contrast(a: Color, b: Color) -> f32 {
        let (a, b) = (a.luminance() + 0.05, b.luminance() + 0.05);
        a.max(b) / a.min(b)
    }

    #[test]
    fn layers_stay_distinct_and_text_reads_against_surfaces() {
        for id in ALL {
            let scheme = Scheme::from_palette(id);
            assert_eq!(scheme.is_dark(), id.is_dark());
            let (canvas, surface, high) = (
                scheme.get(ColorRole::SURFACE_CONTAINER),
                scheme.get(ColorRole::SURFACE),
                scheme.get(ColorRole::SURFACE_CONTAINER_HIGHEST),
            );
            assert_ne!(canvas, surface, "{}", id.name());
            assert_ne!(surface, high, "{}", id.name());
            let ink = scheme.get(ColorRole::ON_SURFACE).luminance();
            assert_eq!(
                ink > surface.luminance(),
                id.is_dark(),
                "{}: dark palettes have light text",
                id.name()
            );
        }
    }

    #[test]
    fn blend_reaches_its_target() {
        let paper = Scheme::from_palette(PaletteId::Paper);
        let ink = Scheme::from_palette(PaletteId::Ink);
        assert_eq!(paper.blend(&ink, 0.), paper);
        assert_eq!(paper.blend(&ink, 1.), ink);
        assert!(paper.blend(&ink, 0.6).is_dark());
    }
}

#[cfg(test)]
mod custom_scheme_tests {
    use super::*;
    #[test]
    fn overriding_one_role_preserves_other_roles_and_appearance() {
        let before = Scheme::from_palette(PaletteId::Paper);
        let after = before.with_role(ColorRole::PRIMARY, Color::hex(0x0b57d0));
        assert_eq!(after.get(ColorRole::PRIMARY), Color::hex(0x0b57d0));
        for role in ColorRole::ALL.iter().filter(|r| **r != ColorRole::PRIMARY) {
            assert_eq!(before.get(*role), after.get(*role));
        }
        assert_eq!(before.is_dark(), after.is_dark());
    }
}
