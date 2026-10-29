//! Semantic color roles and their derivation from a source palette.
//!
//! The role names are the ones course2md's desktop shipped with. Each role
//! documents the Material 3 role it stands for; the full M3 role set
//! (surface containers, outline variants, tertiary, fixed roles) replaces
//! them in the M3 Expressive migration (see `docs/ROADMAP.md`).
use crate::{Color, PaletteId};

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
    /// Window ground: M3 surface container, the canvas with a trace of accent.
    CANVAS,
    /// Tiles, cards and lists: M3 surface (lifted in dark palettes).
    SURFACE,
    /// M3 surface container highest: unselected segments, filled search
    /// bars, tonal icon buttons.
    SURFACE_HIGH,
    /// A recessed area: sidebars, table heads.
    INSET,
    /// M3 on surface.
    INK,
    /// M3 on surface variant: supporting text and quiet icons.
    GRAY,
    /// Tertiary text and hovered scrollbar thumbs.
    FAINT,
    /// M3 outline variant: dividers.
    HAIRLINE,
    /// A card's tonal edge, softer than the hairline.
    CARD_LINE,
    /// M3 outline: unselected radios, inputs, scrollbar thumbs.
    CONTROL,
    /// M3 primary.
    ACCENT,
    /// M3 on primary.
    ON_PRIMARY,
    /// M3 secondary container: tonal selection.
    SECONDARY_CONTAINER,
    /// M3 on secondary container.
    ON_SECONDARY_CONTAINER,
    SUCCESS,
    SUCCESS_BG,
    WARNING,
    WARNING_BG,
    /// M3 error.
    DANGER,
    /// M3 error container (a light tint).
    DANGER_BG,
    /// Find-in-page matches.
    FIND_HIGHLIGHT,
    FIND_CURRENT,
    FIND_CURRENT_LINE,
    /// Text selection in inputs.
    SELECTION,
    /// Selected long-form text: translucent accent over whatever holds it.
    READING_SELECTION,
);

/// The number of roles in a [`Scheme`].
pub const ROLE_COUNT: usize = ColorRole::ALL.len();

/// One value for every [`ColorRole`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scheme {
    colors: [Color; ROLE_COUNT],
    dark: bool,
}

impl Scheme {
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
            ColorRole::CANVAS => canvas,
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
            ColorRole::SURFACE_HIGH => {
                if dark {
                    hex(p.surface).blend(hex(p.text), 0.13)
                } else {
                    canvas.blend(hex(p.text), 0.06)
                }
            }
            ColorRole::INSET => hex(p.inset),
            ColorRole::INK => hex(p.text),
            ColorRole::GRAY => hex(p.muted),
            ColorRole::FAINT => hex(p.subtle),
            ColorRole::HAIRLINE => hex(p.border),
            // M3 cards are separated by tone rather than a full-strength outline.
            ColorRole::CARD_LINE => hex(p.surface).blend(hex(p.border), 0.55),
            ColorRole::CONTROL => hex(p.control),
            ColorRole::ACCENT => hex(p.accent),
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
            ColorRole::DANGER => hex(p.danger),
            ColorRole::DANGER_BG => tint(p.danger, 0.12),
            ColorRole::SELECTION => hex(p.selection),
            // Selected text: the accent laid over whatever surface holds the
            // text, so it stays visible on tinted panels.
            ColorRole::READING_SELECTION => hex(p.accent).alpha(if dark { 0.42 } else { 0.30 }),
            ColorRole::FIND_HIGHLIGHT => tint(p.warning, 0.16),
            ColorRole::FIND_CURRENT => tint(p.warning, 0.32),
            ColorRole::FIND_CURRENT_LINE => hex(p.warning),
        });
        Self { colors, dark }
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

    #[test]
    fn layers_stay_distinct_and_text_reads_against_surfaces() {
        for id in ALL {
            let scheme = Scheme::from_palette(id);
            assert_eq!(scheme.is_dark(), id.is_dark());
            let (canvas, surface, high) = (
                scheme.get(ColorRole::CANVAS),
                scheme.get(ColorRole::SURFACE),
                scheme.get(ColorRole::SURFACE_HIGH),
            );
            assert_ne!(canvas, surface, "{}", id.name());
            assert_ne!(surface, high, "{}", id.name());
            let ink = scheme.get(ColorRole::INK).luminance();
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
