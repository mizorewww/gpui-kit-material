//! Button forms other than the labelled button, sharing [`MaterialButton`]'s
//! Base activation, press/hover/focus tracking, state layer, ripple, focus
//! ring and motion: icon buttons, FABs, extended FABs and split-button
//! segments. Each form supplies only its geometry and, where its colors are
//! not a [`ButtonVariant`]'s, a [`Palette`].
//!
//! Dimensions are the M3 Expressive tokens (m3e.kyralabs.dev; Figma
//! "Material 3 Design Kit", Buttons page), in design px.
use super::*;

/// The width of an icon button relative to its height.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum IconButtonWidth {
    Narrow,
    #[default]
    Default,
    Wide,
}

impl IconButtonWidth {
    pub const ALL: [Self; 3] = [Self::Narrow, Self::Default, Self::Wide];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Narrow => "narrow",
            Self::Default => "default",
            Self::Wide => "wide",
        }
    }
}

/// FAB container sizes. Extended FABs use the same three heights from
/// `Regular` up; `Small` extends like `Regular`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FabSize {
    /// 40dp (baseline small FAB).
    Small,
    /// 56dp.
    #[default]
    Regular,
    /// 80dp (M3 Expressive medium FAB).
    Medium,
    /// 96dp.
    Large,
}

impl FabSize {
    pub const ALL: [Self; 4] = [Self::Small, Self::Regular, Self::Medium, Self::Large];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Regular => "regular",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }
    /// Container side (and extended height).
    pub(crate) const fn container(self) -> f32 {
        match self {
            Self::Small => 40.,
            Self::Regular => 56.,
            Self::Medium => 80.,
            Self::Large => 96.,
        }
    }
    pub(crate) const fn radius(self) -> f32 {
        match self {
            Self::Small => 12.,
            Self::Regular => 16.,
            Self::Medium => 20.,
            Self::Large => 28.,
        }
    }
    pub(crate) const fn icon(self) -> f32 {
        match self {
            Self::Small | Self::Regular => 24.,
            Self::Medium => 28.,
            Self::Large => 36.,
        }
    }
    /// Extended: (height, horizontal padding, icon-label gap, label role).
    pub(crate) const fn extended(self) -> (f32, f32, f32, TypeRole) {
        match self {
            Self::Small | Self::Regular => (56., 16., 8., TypeRole::TitleMedium),
            Self::Medium => (80., 26., 12., TypeRole::TitleLarge),
            Self::Large => (96., 28., 16., TypeRole::HeadlineSmall),
        }
    }
    /// The extended form's height: `Small` extends like `Regular`.
    pub(crate) const fn extended_size(self) -> Self {
        match self {
            Self::Small => Self::Regular,
            other => other,
        }
    }
}

/// Which segment of a split button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Segment {
    Leading,
    Trailing,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Form {
    #[default]
    Label,
    Icon(IconButtonWidth),
    Fab(FabSize),
    /// `expansion` is the animated 0 (collapsed FAB) … 1 (extended) amount.
    ExtendedFab {
        size: FabSize,
        expansion: f32,
    },
    Segment(Segment),
}

/// Container and content roles of one selection state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Roles {
    pub container: Option<ColorRole>,
    pub content: ColorRole,
}

/// Colors and elevation for forms whose colors are not a [`ButtonVariant`]'s
/// (the standard icon button, FAB color styles, FAB menu parts).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Palette {
    pub rest: Roles,
    /// Toggle-selected roles; `None` keeps `rest`.
    pub selected: Option<Roles>,
    /// Disabled specimens keep a translucent container.
    pub disabled_container: bool,
    /// (resting, hovered) elevation levels.
    pub elevation: (u8, u8),
}

impl Palette {
    pub(crate) fn targets(
        &self,
        disabled: bool,
        toggle: bool,
        selected: bool,
    ) -> (CssColor, CssColor, CssColor) {
        use ColorRole as C;
        if disabled {
            return (
                if self.disabled_container {
                    CssColor::alpha_role(C::ON_SURFACE, 0.10)
                } else {
                    CssColor::transparent()
                },
                CssColor::alpha_role(C::ON_SURFACE, state::DISABLED_CONTENT),
                CssColor::transparent(),
            );
        }
        let roles = match self.selected {
            Some(roles) if toggle && selected => roles,
            _ => self.rest,
        };
        (
            roles
                .container
                .map_or_else(CssColor::transparent, CssColor::role),
            CssColor::role(roles.content),
            CssColor::transparent(),
        )
    }

    pub(crate) fn shadow_level(&self, disabled: bool, hovered: bool, active: bool) -> u8 {
        if disabled {
            0
        } else if hovered && !active {
            self.elevation.1
        } else {
            self.elevation.0
        }
    }
}

/// Icon button tokens per size: (icon, narrow, default, wide widths).
pub(crate) const fn icon_button_metrics(size: ButtonSize) -> (f32, [f32; 3]) {
    match size {
        ButtonSize::ExtraSmall => (20., [28., 32., 40.]),
        ButtonSize::Small => (24., [32., 40., 52.]),
        ButtonSize::Medium => (24., [48., 56., 72.]),
        ButtonSize::Large => (32., [64., 96., 128.]),
        ButtonSize::ExtraLarge => (40., [104., 136., 184.]),
    }
}

pub(crate) const fn icon_button_width(size: ButtonSize, width: IconButtonWidth) -> f32 {
    let (_, widths) = icon_button_metrics(size);
    widths[match width {
        IconButtonWidth::Narrow => 0,
        IconButtonWidth::Default => 1,
        IconButtonWidth::Wide => 2,
    }]
}

/// Split-button segment tokens per size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SegmentMetrics {
    /// Inner corners at rest, hovered/focused and pressed.
    pub inner: f32,
    pub inner_hovered: f32,
    pub inner_pressed: f32,
    /// The leading segment's padding next to the trailing one.
    pub leading_inner_padding: f32,
    /// The trailing segment's width and chevron size.
    pub trailing_width: f32,
    pub trailing_icon: f32,
}

pub(crate) const fn segment_metrics(size: ButtonSize) -> SegmentMetrics {
    let (inner, inner_hovered, inner_pressed, leading_inner_padding, trailing_width, trailing_icon) =
        match size {
            ButtonSize::ExtraSmall => (4., 8., 8., 10., 34., 22.),
            ButtonSize::Small => (4., 12., 8., 12., 44., 22.),
            ButtonSize::Medium => (4., 12., 12., 24., 56., 26.),
            ButtonSize::Large => (8., 20., 16., 48., 90., 38.),
            ButtonSize::ExtraLarge => (12., 20., 20., 64., 124., 50.),
        };
    SegmentMetrics {
        inner,
        inner_hovered,
        inner_pressed,
        leading_inner_padding,
        trailing_width,
        trailing_icon,
    }
}

/// Resolved box geometry of one render, in design px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FormGeometry {
    pub height: f32,
    pub width: Option<f32>,
    pub min_width: f32,
    pub padding: (f32, f32),
    pub gap: f32,
    pub icon: f32,
    pub typography: TypeRole,
    pub outline: f32,
}

impl Form {
    /// Whether the resting/pressed radius comes from the button size table
    /// (labelled and icon buttons) rather than the form.
    pub(crate) fn sized_shape(self) -> bool {
        matches!(self, Form::Label | Form::Icon(_))
    }

    pub(crate) fn geometry(
        self,
        size: ButtonSize,
        variant: ButtonVariant,
        icon: Option<ButtonIconPosition>,
        direction: ButtonDirection,
    ) -> FormGeometry {
        let g = size.geometry();
        let label = FormGeometry {
            height: g.height,
            width: None,
            min_width: 58.,
            padding: horizontal_padding(size, variant, icon, direction),
            gap: g.gap,
            icon: size.icon_size(),
            typography: g.typography,
            outline: g.outline,
        };
        match self {
            Form::Label => label,
            Form::Icon(width) => FormGeometry {
                width: Some(icon_button_width(size, width)),
                min_width: 0.,
                padding: (0., 0.),
                gap: 0.,
                icon: icon_button_metrics(size).0,
                ..label
            },
            Form::Fab(fab) => FormGeometry {
                height: fab.container(),
                width: Some(fab.container()),
                min_width: 0.,
                padding: (0., 0.),
                gap: 0.,
                icon: fab.icon(),
                typography: TypeRole::LabelLarge,
                outline: 0.,
            },
            Form::ExtendedFab {
                size: fab,
                expansion,
            } => {
                let fab = fab.extended_size();
                let (height, padding, gap, typography) = fab.extended();
                let t = expansion.clamp(0., 1.);
                // Collapsed, the icon sits centered in a square FAB.
                let collapsed = (height - fab.icon()) / 2.;
                let pad = collapsed + (padding - collapsed) * t;
                FormGeometry {
                    height,
                    width: None,
                    min_width: height,
                    padding: (pad, pad),
                    gap: gap * t,
                    icon: fab.icon(),
                    typography,
                    outline: 0.,
                }
            }
            Form::Segment(Segment::Leading) => {
                let inner = segment_metrics(size).leading_inner_padding;
                let (start, _) = horizontal_padding(size, variant, icon, ButtonDirection::Ltr);
                FormGeometry {
                    min_width: 0.,
                    padding: if direction == ButtonDirection::Rtl {
                        (inner, start)
                    } else {
                        (start, inner)
                    },
                    ..label
                }
            }
            Form::Segment(Segment::Trailing) => {
                let metrics = segment_metrics(size);
                FormGeometry {
                    width: Some(metrics.trailing_width),
                    min_width: 0.,
                    padding: (0., 0.),
                    gap: 0.,
                    icon: metrics.trailing_icon,
                    ..label
                }
            }
        }
    }

    /// The resting radius of forms with their own shape.
    pub(crate) fn fixed_radius(self) -> Option<f32> {
        match self {
            Form::Fab(fab) => Some(fab.radius()),
            Form::ExtendedFab { size, .. } => Some(size.extended_size().radius()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[core::prelude::v1::test]
    fn icon_button_widths_follow_the_expressive_tokens() {
        assert_eq!(
            ButtonSize::ALL.map(|size| icon_button_width(size, IconButtonWidth::Default)),
            [32., 40., 56., 96., 136.],
            "default icon buttons are square"
        );
        assert_eq!(
            icon_button_width(ButtonSize::Small, IconButtonWidth::Narrow),
            32.
        );
        assert_eq!(
            icon_button_width(ButtonSize::ExtraLarge, IconButtonWidth::Wide),
            184.
        );
        let g = Form::Icon(IconButtonWidth::Wide).geometry(
            ButtonSize::Medium,
            ButtonVariant::Filled,
            Some(ButtonIconPosition::Leading),
            ButtonDirection::Ltr,
        );
        assert_eq!(
            (g.height, g.width, g.padding, g.icon),
            (56., Some(72.), (0., 0.), 24.)
        );
    }

    #[core::prelude::v1::test]
    fn fab_sizes_and_extended_expansion() {
        assert_eq!(FabSize::ALL.map(FabSize::container), [40., 56., 80., 96.]);
        assert_eq!(FabSize::ALL.map(FabSize::radius), [12., 16., 20., 28.]);
        let geometry = |expansion| {
            Form::ExtendedFab {
                size: FabSize::Medium,
                expansion,
            }
            .geometry(
                ButtonSize::Small,
                ButtonVariant::Filled,
                None,
                ButtonDirection::Ltr,
            )
        };
        let collapsed = geometry(0.);
        assert_eq!(
            collapsed.padding,
            (26., 26.),
            "a centered 28dp icon in 80dp"
        );
        assert_eq!(collapsed.gap, 0.);
        assert_eq!(collapsed.padding.0 * 2. + collapsed.icon, 80.);
        let extended = geometry(1.);
        assert_eq!((extended.padding, extended.gap), ((26., 26.), 12.));
        assert_eq!(geometry(0.5).gap, 6.);
    }

    #[core::prelude::v1::test]
    fn standard_palette_selects_primary_and_drops_container_when_disabled() {
        let palette = Palette {
            rest: Roles {
                container: None,
                content: ColorRole::ON_SURFACE_VARIANT,
            },
            selected: Some(Roles {
                container: None,
                content: ColorRole::PRIMARY,
            }),
            disabled_container: false,
            elevation: (0, 0),
        };
        let resolve = |(bg, fg, _): (CssColor, CssColor, CssColor)| (bg.resolve(), fg.resolve());
        let (bg, fg) = resolve(palette.targets(false, true, true));
        assert_eq!(bg.a, 0.);
        assert_eq!(fg, crate::theme::color(ColorRole::PRIMARY));
        let (_, fg) = resolve(palette.targets(false, false, true));
        assert_eq!(
            fg,
            crate::theme::color(ColorRole::ON_SURFACE_VARIANT),
            "selection needs toggle semantics"
        );
        let (bg, fg) = resolve(palette.targets(true, true, true));
        assert_eq!(bg.a, 0.);
        let ink = crate::theme::color(ColorRole::ON_SURFACE);
        assert!((fg.a - ink.a * state::DISABLED_CONTENT).abs() < 1e-4);
    }

    #[core::prelude::v1::test]
    fn fab_elevation_rises_on_hover_only() {
        let palette = Palette {
            rest: Roles {
                container: Some(ColorRole::PRIMARY_CONTAINER),
                content: ColorRole::ON_PRIMARY_CONTAINER,
            },
            selected: None,
            disabled_container: true,
            elevation: (3, 4),
        };
        assert_eq!(palette.shadow_level(false, false, false), 3);
        assert_eq!(palette.shadow_level(false, true, false), 4);
        assert_eq!(palette.shadow_level(false, true, true), 3);
        assert_eq!(palette.shadow_level(true, true, false), 0);
    }

    #[core::prelude::v1::test]
    fn split_segments_share_the_button_height() {
        for size in ButtonSize::ALL {
            let leading = Form::Segment(Segment::Leading).geometry(
                size,
                ButtonVariant::Filled,
                Some(ButtonIconPosition::Leading),
                ButtonDirection::Ltr,
            );
            let trailing = Form::Segment(Segment::Trailing).geometry(
                size,
                ButtonVariant::Filled,
                None,
                ButtonDirection::Ltr,
            );
            assert_eq!(leading.height, trailing.height);
            assert_eq!(
                leading.padding.1,
                segment_metrics(size).leading_inner_padding
            );
            let m = segment_metrics(size);
            assert!(m.inner < m.inner_pressed && m.inner_pressed <= m.inner_hovered);
        }
    }
}
