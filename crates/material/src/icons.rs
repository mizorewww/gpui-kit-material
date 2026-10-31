//! Material Symbols rendered through Longbridge Kit's Component Icon.
//!
//! The compatibility names below select Google symbols; the full catalog and
//! variable axes are available through [`Symbol`], [`SymbolIcon`] and [`Axes`].
use gpui::{
    AbsoluteLength, App, AssetSource, Hsla, IntoElement, RenderOnce, SharedString, Styled,
    Transformation, Window, size,
};
pub use gpui_material_symbols::{Asset, Axes, Icon as SymbolIcon, MaterialSymbols, Style, Symbol};
use std::sync::atomic::{AtomicBool, Ordering};
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
        if path == PENDING_PATH {
            return Ok(Some(Cow::Borrowed(PENDING_SVG)));
        }
        match self.inner.load(path) {
            // A page that downloads its fonts draws empty icons until they arrive,
            // not an error for every icon in every frame.
            Err(error)
                if matches!(
                    error.downcast_ref::<gpui_material_symbols::Error>(),
                    Some(gpui_material_symbols::Error::NotLoaded(_))
                ) =>
            {
                Ok(None)
            }
            result => result,
        }
    }
    fn list(&self, prefix: &str) -> gpui::Result<Vec<SharedString>> {
        self.inner.list(prefix)
    }
}

/// What an icon draws while the page is still downloading its font: a dot in the
/// icon's own color and slot. A separate path, so that the glyph drawn later is
/// not the placeholder's cached sprite.
const PENDING_PATH: &str = "material-symbols/pending.svg";
const PENDING_SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="4" fill-opacity="0.4"/></svg>"#;

static SYMBOLS_PENDING: AtomicBool = AtomicBool::new(false);

/// Whether the symbol fonts are still on their way, for a page that downloads
/// them: until it says otherwise, icons draw a placeholder instead of nothing.
/// Change it before a window redraw, which this does not cause.
pub fn set_symbols_pending(pending: bool) {
    SYMBOLS_PENDING.store(pending, Ordering::Relaxed);
}

/// Material's 20px default slot and legacy builder API over Component Icon.
#[derive(IntoElement, Clone)]
pub struct Icon {
    inner: gpui_kit::component::Icon,
    /// Mirrors under right-to-left layout.
    directional: bool,
    rotation: Option<gpui::Radians>,
}

/// Symbols that point along the reading direction, so they mirror with it.
/// The Material guidance flips arrows and navigation marks; clocks, media
/// controls and most objects keep their shape.
fn points_along_reading(name: &str) -> bool {
    matches!(
        name,
        "arrow_back"
            | "arrow_back_ios"
            | "arrow_back_ios_new"
            | "arrow_forward"
            | "arrow_forward_ios"
            | "arrow_left_alt"
            | "arrow_right_alt"
            | "chevron_backward"
            | "chevron_forward"
            | "chevron_left"
            | "chevron_right"
            | "east"
            | "first_page"
            | "format_indent_decrease"
            | "format_indent_increase"
            | "input"
            | "keyboard_arrow_left"
            | "keyboard_arrow_right"
            | "keyboard_tab"
            | "last_page"
            | "login"
            | "logout"
            | "menu_open"
            | "navigate_before"
            | "navigate_next"
            | "open_in_new"
            | "output"
            | "read_more"
            | "redo"
            | "reply"
            | "reply_all"
            | "send"
            | "short_text"
            | "subject"
            | "toc"
            | "trending_flat"
            | "undo"
            | "west"
    )
}
impl Icon {
    pub fn new(path: impl Into<SharedString>) -> Self {
        Self {
            inner: gpui_kit::component::Icon::default()
                .path(path)
                .size(crate::tokens::layout::ICON_SLOT),
            directional: false,
            rotation: None,
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
        self.rotation = Some(gpui::radians(radians));
        self.inner = self.inner.rotate(gpui::radians(radians));
        self
    }
    /// Mirror under right-to-left layout. Symbols that point along the reading
    /// direction already do; use this for a caller's own asset, or `false` to
    /// keep a symbol as drawn.
    pub fn directional(mut self, directional: bool) -> Self {
        self.directional = directional;
        self
    }
}
impl From<IconName> for Icon {
    fn from(name: IconName) -> Self {
        Self::new(name.path()).directional(points_along_reading(name.symbol().name()))
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
            directional: points_along_reading(icon.symbol().name()),
            inner: icon.component().size(crate::tokens::layout::ICON_SLOT),
            rotation: None,
        }
    }
}
impl RenderOnce for Icon {
    fn render(mut self, window: &mut Window, _: &mut App) -> impl IntoElement {
        if SYMBOLS_PENDING.load(Ordering::Relaxed) {
            self.inner = self.inner.path(PENDING_PATH);
        }
        if self.directional && window.layout_direction().is_rtl() {
            let flip = Transformation::scale(size(-1., 1.));
            let flip = match self.rotation {
                Some(rotation) => flip.with_rotation(rotation),
                None => flip,
            };
            return self.inner.transform(flip);
        }
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_pending_placeholder_is_an_svg_of_its_own() {
        let assets = Assets::new();
        let bytes = assets.load(PENDING_PATH).unwrap().unwrap();
        assert!(bytes.starts_with(b"<svg"));
        // Never the path of a real symbol, whose sprite would be kept.
        assert!(SymbolIcon::from_asset_path(PENDING_PATH).is_err());
    }

    #[test]
    fn only_symbols_along_the_reading_direction_mirror() {
        for name in ["arrow_back", "chevron_right", "send", "undo"] {
            assert!(points_along_reading(name), "{name} points along reading");
            assert!(Symbol::named(name).is_some(), "{name} is a real symbol");
        }
        for name in ["check", "search", "close", "schedule"] {
            assert!(!points_along_reading(name), "{name} keeps its shape");
        }
        assert!(Icon::from(IconName::Check).rotate(1.).rotation.is_some());
    }
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
