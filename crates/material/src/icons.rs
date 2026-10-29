//! Material Symbols rendered through Longbridge Kit's Component Icon.
//!
//! The compatibility names below select Google symbols; the full catalog and
//! variable axes are available through [`Symbol`], [`SymbolIcon`] and [`Axes`].
use gpui::{
    AbsoluteLength, App, AssetSource, Hsla, IntoElement, RenderOnce, SharedString, Styled, Window,
};
pub use gpui_material_symbols::{Axes, Icon as SymbolIcon, MaterialSymbols, Style, Symbol};
use std::{borrow::Cow, collections::HashMap, sync::OnceLock};

mod kit_mapping;

macro_rules! icons {
    ($($name:ident => $symbol:literal;)+) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum IconName { $($name),+ }
        impl IconName {
            pub const ALL: &'static [Self] = &[$(Self::$name),+];
            pub fn symbol(self) -> Symbol {
                Symbol::named(match self { $(Self::$name => $symbol),+ })
                    .expect("Material compatibility symbol belongs to the pinned catalog")
            }
            /// Stable canonical path at Rounded, FILL=0, weight 400, optical size 24.
            pub fn path(self) -> &'static str {
                match self { $(Self::$name => {
                    static PATH: OnceLock<String> = OnceLock::new();
                    PATH.get_or_init(|| self.symbol().icon().style(Style::Rounded).asset_path()).as_str()
                }),+ }
            }
            pub fn icon(self) -> Icon { self.into() }
        }
    };
}
icons! {
    Check => "check";
    CheckCircle => "check_circle";
    ChevronDown => "keyboard_arrow_down";
    ChevronRight => "chevron_right";
    ChevronUp => "keyboard_arrow_up";
    Close => "close";
    Error => "cancel";
    Info => "info";
    LoaderCircle => "progress_activity";
    Warning => "warning";
    CaptionClose => "close";
    CaptionMaximize => "crop_square";
    CaptionMinimize => "remove";
    CaptionRestore => "filter_none";
}

/// Register the precomputed mapping for Kit's default internal icon paths.
/// User-provided raw SVG data deliberately remains under the caller's control.
pub(crate) fn init(cx: &mut App) {
    let paths: HashMap<&'static str, SharedString> = kit_mapping::mappings()
        .map(|(path, icon)| (path, icon.asset_path().into()))
        .collect();
    // Material names are also initialized before the first frame.
    for name in IconName::ALL {
        let _ = name.path();
    }
    gpui_kit::component::set_icon_resolver(cx, move |path, _| paths.get(path).cloned());
}

/// Material Symbols first, followed by caller-owned application assets.
/// With default features, symbols are embedded on both desktop and WASM.
/// Without `embedded-symbols`, supply the sibling library assets before drawing.
pub struct Assets<F = ()> {
    inner: gpui_material_symbols::WithMaterialSymbols<F>,
}
impl Assets<()> {
    pub fn new() -> Self {
        Self {
            inner: MaterialSymbols.with_fallback(()),
        }
    }
    pub fn with<F: AssetSource>(fallback: F) -> Assets<F> {
        Assets {
            inner: MaterialSymbols.with_fallback(fallback),
        }
    }
}
impl Default for Assets<()> {
    fn default() -> Self {
        Self::new()
    }
}
impl<F: AssetSource> AssetSource for Assets<F> {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        self.inner.load(path)
    }
    fn list(&self, prefix: &str) -> gpui::Result<Vec<SharedString>> {
        self.inner.list(prefix)
    }
}

/// Material's 20px default slot and legacy builder API over Component Icon.
#[derive(IntoElement, Clone)]
pub struct Icon {
    inner: gpui_kit::component::Icon,
}
impl Icon {
    pub fn new(path: impl Into<SharedString>) -> Self {
        Self {
            inner: gpui_kit::component::Icon::default()
                .path(path)
                .size(crate::tokens::layout::ICON_SLOT),
        }
    }
    pub fn size(mut self, size: impl Into<AbsoluteLength>) -> Self {
        self.inner = self.inner.size(size.into());
        self
    }
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.inner = self.inner.text_color(color);
        self
    }
    /// Rotate by radians, using Kit's SVG transformation.
    pub fn rotate(mut self, radians: f32) -> Self {
        self.inner = self.inner.rotate(gpui::radians(radians));
        self
    }
}
impl From<IconName> for Icon {
    fn from(name: IconName) -> Self {
        Self::new(name.path())
    }
}
impl From<Symbol> for Icon {
    fn from(symbol: Symbol) -> Self {
        Self::from(symbol.icon().style(Style::Rounded))
    }
}
impl From<SymbolIcon> for Icon {
    fn from(icon: SymbolIcon) -> Self {
        Self {
            inner: icon.component().size(crate::tokens::layout::ICON_SLOT),
        }
    }
}
impl RenderOnce for Icon {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compatibility_symbols_and_kit_paths_load_from_google_catalog() {
        let assets = Assets::new();
        for name in IconName::ALL {
            let parsed = SymbolIcon::from_asset_path(name.path()).unwrap();
            assert_eq!(parsed.symbol(), name.symbol());
            assert_eq!(parsed.visual_style(), Style::Rounded);
            assert!(
                assets
                    .load(name.path())
                    .unwrap()
                    .unwrap()
                    .starts_with(b"<svg")
            );
        }
        for (_, icon) in kit_mapping::mappings() {
            assert!(assets.load(&icon.asset_path()).unwrap().is_some());
        }
        assert!(assets.load("caller/missing.svg").unwrap().is_none());
    }
    #[test]
    fn kit_default_catalog_has_no_silent_mapping_gaps() {
        use std::collections::BTreeSet;
        let expected: BTreeSet<_> = include_str!("../../../conformance/kit-default-icons.txt")
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();
        let mapped: BTreeSet<_> = kit_mapping::mappings().map(|(path, _)| path).collect();
        assert_eq!(mapped.len(), kit_mapping::mappings().count());
        let unsupported: BTreeSet<_> = kit_mapping::UNSUPPORTED_PATHS.iter().copied().collect();
        assert!(mapped.is_disjoint(&unsupported));
        assert_eq!(
            mapped.union(&unsupported).copied().collect::<BTreeSet<_>>(),
            expected
        );
        assert!(
            kit_mapping::APPROXIMATE_PATHS
                .iter()
                .all(|path| mapped.contains(path))
        );
        let filled = kit_mapping::mappings()
            .find(|(path, _)| *path == "icons/star-fill.svg")
            .unwrap()
            .1;
        assert_eq!(filled.variations().fill(), 1.);
    }

    #[test]
    fn source_preserves_fallback_and_invalid_symbol_errors() {
        struct Caller;
        impl AssetSource for Caller {
            fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
                match path {
                    "caller/known" => Ok(Some(Cow::Borrowed(b"caller bytes"))),
                    "caller/error" => Err(anyhow::anyhow!("caller error")),
                    _ => Ok(None),
                }
            }
            fn list(&self, _: &str) -> gpui::Result<Vec<SharedString>> {
                Ok(vec![])
            }
        }
        let assets = Assets::with(Caller);
        assert_eq!(
            assets.load("caller/known").unwrap().unwrap().as_ref(),
            b"caller bytes"
        );
        assert!(assets.load("caller/missing").unwrap().is_none());
        assert_eq!(
            assets.load("caller/error").unwrap_err().to_string(),
            "caller error"
        );
        assert!(assets.load("material-symbols/invalid.svg").is_err());
    }

    #[test]
    fn filled_variants_have_distinct_cache_keys_and_geometry() {
        let outline = Symbol::named("favorite").unwrap().icon();
        let filled = outline.filled(true);
        assert_ne!(outline.asset_path(), filled.asset_path());
        assert_ne!(outline.to_svg().unwrap(), filled.to_svg().unwrap());
    }
}
