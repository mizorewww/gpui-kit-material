//! The page framework: each design-kit page is a self-contained module with
//! its own state entity and rows. The Gallery builds a page when it is
//! first shown and lists only the current page's rows: a component's Specs
//! tab, a Styles/Examples page body, or (for components without a
//! playground) the live demos of its preview.
//!
//! # Adding a page
//!
//! 1. Create `pages/<page>.rs` with a state struct implementing
//!    [`GalleryPage`]: its [`PageInfo`] (`INFO`), its body row count and a
//!    `render_row` for each body row. The shell renders the kit header as the
//!    page's first row and frames all rows in the page's panel.
//! 2. List it in `pages/mod.rs` (`PAGES`, in Figma order) and reference it
//!    from a `catalog::CATALOG` entry's `spec`. Navigation, `--section <id>`
//!    captures and keyboard traversal follow automatically.
//!
//! Pages keep all of their state in their own entity. Listeners created
//! with `cx.listener` in `render_row` update the page; `cx.notify()` on the
//! page re-renders the visible Gallery rows while the page is shown. Call [`PageRows::remeasure`]
//! for rows whose height a change alters while they may be offscreen.
//! Gallery-wide settings (theme, text scale, motion scheme, snackbar) live in
//! [`GalleryShared`], reachable through [`PageCx::shared`].
//!
//! Do not notify from `prepare` or `render_row`: the Gallery observes every
//! page, so that would schedule another frame each frame.
use gpui::{prelude::*, *};
use gpui_kit_material::color::themes::{Contrast, ThemeId, ThemeMode};
use gpui_kit_material::motion;
use gpui_kit_material::prelude::PaletteId;
use std::{cell::RefCell, ops::RangeInclusive, rc::Rc};

pub use super::spec::PageInfo;

/// Gallery-wide settings and overlays any page may change. The Gallery
/// observes this entity: theme, mode, contrast, palette and scale changes
/// start the animated theme transition; others re-render the Gallery.
pub struct GalleryShared {
    pub palette: PaletteId,
    /// `None` paints `palette` instead of a reference theme.
    pub reference_theme: Option<ThemeId>,
    pub reference_mode: ThemeMode,
    /// A theme generated from a seed color, painted in `reference_mode` and
    /// `contrast` in place of the reference theme or palette.
    pub seed: Option<gpui_kit_material::color::dynamic::SeedScheme>,
    /// The live preview's own theme and mode (md3.io's preview controls). `None`
    /// follows the application.
    pub preview_theme: Option<ThemeId>,
    pub preview_mode: Option<ThemeMode>,
    /// Lays the live preview out right to left.
    pub preview_rtl: bool,
    pub contrast: Contrast,
    /// The text scale, as the option value ("1", "1.25", …).
    pub scale: SharedString,
    pub motion_scheme: motion::MotionScheme,
    /// Shows the Gallery's sample snackbar.
    pub snackbar: bool,
}

impl Default for GalleryShared {
    fn default() -> Self {
        Self {
            palette: PaletteId::Paper,
            reference_theme: Some(ThemeId::Baseline),
            reference_mode: ThemeMode::Light,
            seed: None,
            preview_theme: None,
            preview_mode: None,
            preview_rtl: false,
            contrast: Contrast::Standard,
            scale: "1".into(),
            motion_scheme: motion::MotionScheme::Expressive,
            snackbar: false,
        }
    }
}

impl GalleryShared {
    pub fn target_scheme(&self) -> gpui_kit_material::color::Scheme {
        if let Some(seed) = self.seed_scheme(self.reference_mode) {
            return seed;
        }
        self.reference_theme.map_or_else(
            || gpui_kit_material::color::Scheme::from_palette(self.palette),
            |id| id.scheme(self.reference_mode, self.contrast),
        )
    }
    fn seed_scheme(&self, mode: ThemeMode) -> Option<gpui_kit_material::color::Scheme> {
        let seed = self.seed?;
        Some(
            seed.dark(mode == ThemeMode::Dark)
                .contrast(super::pages::color::contrast_level(self.contrast))
                .scheme(),
        )
    }
    /// The preview's scheme, when it differs from the application's. A mode of
    /// its own keeps the application's seed, if one is painted.
    pub fn preview_scheme(&self) -> Option<gpui_kit_material::color::Scheme> {
        if self.preview_theme.is_none() && self.preview_mode.is_none() {
            return None;
        }
        let mode = self.preview_mode.unwrap_or(self.reference_mode);
        if self.preview_theme.is_none()
            && let Some(seed) = self.seed_scheme(mode)
        {
            return Some(seed);
        }
        let theme = self
            .preview_theme
            .or(self.reference_theme)
            .unwrap_or(ThemeId::Baseline);
        Some(theme.scheme(mode, self.contrast))
    }
    /// The live preview's overrides, as a [`Scope`]. With none set it changes nothing.
    pub fn preview_scope(&self) -> gpui_kit_material::scope::Scope {
        let mut scope = gpui_kit_material::scope::Scope::new("preview-scope");
        if let Some(scheme) = self.preview_scheme() {
            scope = scope.scheme(scheme);
        }
        if self.preview_rtl {
            scope = scope.direction(gpui_kit_material::scope::LayoutDirection::Rtl);
        }
        scope
    }
    pub fn text_scale(&self) -> f32 {
        self.scale.parse().unwrap_or(1.)
    }
}

/// Where a page's rows sit in the Gallery list. Only the current page's
/// shown rows are in the list; the shell maps them when it shows the page
/// (a page's Specs tab may show a slice of its rows).
#[derive(Clone)]
pub struct PageRows {
    list: ListState,
    /// The list index of each page row, `None` while not in the list.
    shown: Rc<RefCell<Vec<Option<usize>>>>,
}

impl PageRows {
    pub(super) fn new(list: ListState) -> Self {
        Self {
            list,
            shown: Default::default(),
        }
    }
    /// Record which list row shows each page row (empty: none).
    pub(super) fn show(&self, rows: impl IntoIterator<Item = (usize, usize)>) {
        let mut shown = self.shown.borrow_mut();
        shown.clear();
        for (row, index) in rows {
            if shown.len() <= row {
                shown.resize(row + 1, None);
            }
            shown[row] = Some(index);
        }
    }
    /// The list row of body row `row`, if it is in the list.
    pub fn global(&self, row: usize) -> Option<usize> {
        self.shown.borrow().get(row).copied().flatten()
    }
    /// Invalidate the cached heights of the shown body `rows`.
    pub fn remeasure(&self, rows: RangeInclusive<usize>) {
        for row in rows {
            if let Some(index) = self.global(row) {
                self.list.remeasure_items(index..index + 1);
            }
        }
    }
    /// The Gallery list, e.g. for `RevealFocus::in_list`.
    pub fn list(&self) -> &ListState {
        &self.list
    }
}

/// What a page receives when the Gallery creates it.
#[derive(Clone)]
pub struct PageCx {
    pub shared: Entity<GalleryShared>,
    pub rows: PageRows,
}

/// Capture-time state overrides (`--active`, `--disabled`, …) for pages
/// with live demos of those states.
#[derive(Clone, Copy, Debug, Default)]
pub struct CaptureState {
    pub active: bool,
    pub disabled: bool,
    pub selected: bool,
    pub loading: bool,
    pub ripple: bool,
}

/// One design-kit page.
pub trait GalleryPage: Sized + 'static {
    const INFO: PageInfo;
    fn new(page: PageCx, window: &mut Window, cx: &mut Context<Self>) -> Self;
    /// Body rows after the header. May depend on the window (e.g. columns).
    fn row_count(&self, window: &Window) -> usize;
    fn render_row(&mut self, row: usize, window: &mut Window, cx: &mut Context<Self>)
    -> AnyElement;
    /// Space above body `row` in design px (rems at the base scale).
    fn row_gap(&self, row: usize) -> f32 {
        if row == 0 {
            super::spec::HEADER_TO_BODY
        } else {
            super::spec::ROW_GAP
        }
    }
    /// Whether body `row` holds a keyboard stop. Specimen boards do not.
    fn interactive(&self, _row: usize) -> bool {
        false
    }
    /// Runs in every Gallery render while the page is shown, mounted or
    /// not: keep playback clocks and keyed animations alive offscreen.
    /// Never notify from here.
    fn prepare(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {}
    /// Whether [`Self::prepare`] does anything. Running it updates the page
    /// entity, which marks what read the page as changed, so a page without
    /// one must not pay for that on every Gallery render.
    const PREPARES: bool = false;
    /// Whether [`Self::after_layout`] does anything, for the same reason.
    const OBSERVES_LAYOUT: bool = false;
    /// Runs after the list's prepaint with which body rows are on screen,
    /// to forward controlled changes to playbacks of unmounted rows.
    fn after_layout(&mut self, _visible: &dyn Fn(usize) -> bool, _cx: &mut Context<Self>) {}
    fn apply_capture(&mut self, _state: CaptureState, _cx: &mut Context<Self>) {}
}

/// A page behind its entity, without its type.
pub(super) trait AnyPage {
    fn row_count(&self, window: &Window, cx: &App) -> usize;
    fn row_gap(&self, row: usize, cx: &App) -> f32;
    fn interactive(&self, row: usize, cx: &App) -> bool;
    fn render_row(&self, row: usize, window: &mut Window, cx: &mut App) -> AnyElement;
    fn prepare(&self, window: &mut Window, cx: &mut App);
    fn after_layout(&self, visible: &dyn Fn(usize) -> bool, cx: &mut App);
    fn observes_layout(&self) -> bool;
    fn apply_capture(&self, state: CaptureState, cx: &mut App);
    /// Re-render the Gallery when page `index` (in `PAGES`) changes while shown.
    fn observe(&self, index: usize, cx: &mut Context<super::Gallery>) -> Subscription;
    #[cfg(test)]
    fn any_entity(&self) -> AnyEntity;
}

impl<P: GalleryPage> AnyPage for Entity<P> {
    fn row_count(&self, window: &Window, cx: &App) -> usize {
        self.read(cx).row_count(window)
    }
    fn row_gap(&self, row: usize, cx: &App) -> f32 {
        self.read(cx).row_gap(row)
    }
    fn interactive(&self, row: usize, cx: &App) -> bool {
        self.read(cx).interactive(row)
    }
    fn render_row(&self, row: usize, window: &mut Window, cx: &mut App) -> AnyElement {
        self.update(cx, |page, cx| page.render_row(row, window, cx))
    }
    fn prepare(&self, window: &mut Window, cx: &mut App) {
        if P::PREPARES {
            self.update(cx, |page, cx| page.prepare(window, cx))
        }
    }
    fn observes_layout(&self) -> bool {
        P::OBSERVES_LAYOUT
    }
    fn after_layout(&self, visible: &dyn Fn(usize) -> bool, cx: &mut App) {
        self.update(cx, |page, cx| page.after_layout(visible, cx))
    }
    fn apply_capture(&self, state: CaptureState, cx: &mut App) {
        self.update(cx, |page, cx| page.apply_capture(state, cx))
    }
    fn observe(&self, index: usize, cx: &mut Context<super::Gallery>) -> Subscription {
        cx.observe(self, move |gallery, _, cx| {
            if gallery.shows_page(index) {
                gallery.counted.set(None);
                cx.notify()
            }
        })
    }
    #[cfg(test)]
    fn any_entity(&self) -> AnyEntity {
        self.clone().into()
    }
}

/// One registry entry: a page's static info and its constructor.
pub struct PageEntry {
    pub info: &'static PageInfo,
    pub(super) build: fn(PageCx, &mut Window, &mut App) -> Rc<dyn AnyPage>,
}

fn build<P: GalleryPage>(page: PageCx, window: &mut Window, cx: &mut App) -> Rc<dyn AnyPage> {
    Rc::new(cx.new(|cx| P::new(page, window, cx)))
}

/// The registry entry of page `P`.
pub const fn entry<P: GalleryPage>() -> PageEntry {
    PageEntry {
        info: &P::INFO,
        build: build::<P>,
    }
}
