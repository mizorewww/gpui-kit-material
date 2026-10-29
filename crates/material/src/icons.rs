//! The kit's own glyphs and an [`Icon`] element that paints any SVG asset
//! in the current text color.
//!
//! Components use only the glyphs listed in [`IconName`]: Lucide outline
//! icons (ISC, `assets/icons/LICENSE-lucide`) and window caption glyphs drawn
//! for this kit on a 10px grid. Applications keep their own icon sets and
//! pass any [`Icon`] into component slots. The Material Symbols migration
//! replaces these glyphs (see `docs/ROADMAP.md`).
use gpui::{
    AbsoluteLength, App, AssetSource, Hsla, IntoElement, RenderOnce, SharedString, Styled,
    Transformation, Window, prelude::*, svg,
};
use std::borrow::Cow;

macro_rules! icons {
    ($($name:ident => $file:literal;)+) => {
        /// A glyph shipped with the kit.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum IconName { $($name),+ }

        impl IconName {
            /// The asset path [`Assets`] serves this glyph at.
            pub fn path(self) -> &'static str {
                match self { $(Self::$name => concat!("material-kit/icons/", $file, ".svg")),+ }
            }
        }

        const ICONS: &[(&str, &[u8])] = &[
            $((
                concat!("material-kit/icons/", $file, ".svg"),
                include_bytes!(concat!("../assets/icons/", $file, ".svg")),
            ),)+
        ];
    };
}

icons! {
    Check => "check";
    CheckCircle => "circle-check";
    ChevronDown => "chevron-down";
    ChevronRight => "chevron-right";
    ChevronUp => "chevron-up";
    Close => "close";
    Error => "circle-x";
    Info => "info";
    LoaderCircle => "loader-circle";
    Warning => "triangle-alert";
    CaptionClose => "caption-close";
    CaptionMaximize => "caption-maximize";
    CaptionMinimize => "caption-minimize";
    CaptionRestore => "caption-restore";
}

/// Serves the kit's glyphs under `material-kit/`, then defers to `fallback`.
/// Wrap the application's own asset source with it:
/// `Application::new().with_assets(Assets::with(MyAssets))`.
pub struct Assets<F = ()> {
    fallback: F,
}

impl Assets<()> {
    /// Only the kit's glyphs.
    pub fn new() -> Self {
        Self { fallback: () }
    }

    pub fn with<F: AssetSource>(fallback: F) -> Assets<F> {
        Assets { fallback }
    }
}

impl Default for Assets<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: AssetSource> AssetSource for Assets<F> {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        if let Some((_, bytes)) = ICONS.iter().find(|(name, _)| *name == path) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        self.fallback.load(path)
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let mut paths = self.fallback.list(path)?;
        paths.extend(
            ICONS
                .iter()
                .filter(|(name, _)| name.starts_with(path))
                .map(|(name, _)| SharedString::from(*name)),
        );
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}

/// An SVG glyph painted in its text color (inherited unless set), 20px
/// (`ICON_SLOT`) by default.
#[derive(IntoElement, Clone)]
pub struct Icon {
    path: SharedString,
    size: AbsoluteLength,
    color: Option<Hsla>,
    rotation: f32,
}

impl Icon {
    /// Any SVG asset the application's asset source serves.
    pub fn new(path: impl Into<SharedString>) -> Self {
        Self {
            path: path.into(),
            size: crate::tokens::layout::ICON_SLOT.into(),
            color: None,
            rotation: 0.,
        }
    }

    pub fn size(mut self, size: impl Into<AbsoluteLength>) -> Self {
        self.size = size.into();
        self
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Rotate by `radians` (spinners, turning chevrons).
    pub fn rotate(mut self, radians: f32) -> Self {
        self.rotation = radians;
        self
    }
}

impl From<IconName> for Icon {
    fn from(name: IconName) -> Self {
        Icon::new(name.path())
    }
}

impl IconName {
    pub fn icon(self) -> Icon {
        self.into()
    }
}

impl RenderOnce for Icon {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        let color = self.color.unwrap_or_else(|| window.text_style().color);
        svg()
            .path(self.path)
            .flex_shrink_0()
            .size(self.size)
            .text_color(color)
            .when(self.rotation != 0., |svg| {
                svg.with_transformation(Transformation::rotate(gpui::radians(self.rotation)))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_is_served_and_listed() {
        let assets = Assets::new();
        for (path, bytes) in ICONS {
            let loaded = assets.load(path).unwrap().unwrap();
            assert_eq!(&*loaded, *bytes);
            assert!(std::str::from_utf8(bytes).unwrap().contains("<svg"));
        }
        assert_eq!(
            assets.list("material-kit/icons/").unwrap().len(),
            ICONS.len()
        );
        assert!(assets.load("icons/other.svg").unwrap().is_none());
        assert!(IconName::Check.path().ends_with("check.svg"));
    }
}
