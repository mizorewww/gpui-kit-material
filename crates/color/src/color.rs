/// An sRGB color with straight alpha, each channel in `0.0..=1.0`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// An opaque color from `0xRRGGBB`.
    pub const fn hex(rgb: u32) -> Self {
        Self::hexa((rgb << 8) | 0xff)
    }

    /// A color from `0xRRGGBBAA`.
    pub const fn hexa(rgba: u32) -> Self {
        Self {
            r: ((rgba >> 24) & 0xff) as f32 / 255.,
            g: ((rgba >> 16) & 0xff) as f32 / 255.,
            b: ((rgba >> 8) & 0xff) as f32 / 255.,
            a: (rgba & 0xff) as f32 / 255.,
        }
    }

    /// The same color at `alpha` opacity.
    pub const fn alpha(self, alpha: f32) -> Self {
        Self { a: alpha, ..self }
    }

    /// Linear interpolation toward `to`; `t` is clamped to `0.0..=1.0`, and the ends are exact.
    ///
    /// State layers, tonal containers and palette transitions all use this
    /// one blend, so a hovered selected control resolves over its current
    /// container instead of stacking translucent fills.
    pub fn blend(self, to: Self, t: f32) -> Self {
        // The ends are exact, so a finished transition paints its target.
        if t >= 1. {
            return to;
        }
        let t = t.max(0.);
        Self {
            r: self.r + (to.r - self.r) * t,
            g: self.g + (to.g - self.g) * t,
            b: self.b + (to.b - self.b) * t,
            a: self.a + (to.a - self.a) * t,
        }
    }

    /// Relative luminance (WCAG 2.x / sRGB): Rec. 709 weights on the
    /// linearized channels, so mid gray `#808080` is about 0.216, not 0.5.
    pub fn luminance(self) -> f32 {
        fn linear(channel: f32) -> f32 {
            if channel <= 0.04045 {
                channel / 12.92
            } else {
                ((channel + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * linear(self.r) + 0.7152 * linear(self.g) + 0.0722 * linear(self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn hex_and_blend() {
        let black = Color::hex(0x000000);
        let white = Color::hex(0xffffff);
        assert_eq!(white.a, 1.);
        assert_eq!(black.blend(white, 0.5).r, 0.5);
        assert_eq!(black.blend(white, 2.), white, "t is clamped");
        assert_eq!(Color::hexa(0x10141680).a, 128. / 255.);
        assert!(white.luminance() > black.luminance());
    }

    #[test]
    fn luminance_linearizes_srgb() {
        assert_eq!(Color::hex(0xffffff).luminance(), 1.);
        assert_eq!(Color::hex(0x000000).luminance(), 0.);
        // WCAG: #808080 is 0.2159, far darker than its stored 0.502.
        let gray = Color::hex(0x808080).luminance();
        assert!((gray - 0.2159).abs() < 0.0005, "{gray}");
        // The low segment is linear: #0A0A0A (0.0392) / 12.92.
        let dark = Color::hex(0x0a0a0a).luminance();
        assert!((dark - 10. / 255. / 12.92).abs() < 1e-6, "{dark}");
        // Pure green carries the largest weight.
        assert!((Color::hex(0x00ff00).luminance() - 0.7152).abs() < 1e-6);
    }
}
