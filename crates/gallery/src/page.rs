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
//!
//! # How a row is built
//!
//! A row needs the page's `Context` for its listeners, and `render_row` may keep
//! what it builds in the page (playback handles, lazily made entities). Building a
//! row through [`Entity::update`] would count as writing the page while the window
//! draws, and under GPUI Fast every other retained row that read the page would be
//! built again on the next frame, each one writing it in turn. So a row is built
//! the way the engine renders a view: the page is the [`Render`] view, rendered in
//! place with [`gpui::View::render`] for the row [`render_row_view`] asks for. A
//! view's own render is drawing it, not changing it; a notify while it renders
//! still marks it changed, so what the page notifies about rebuilds every row that
//! read it, and nothing else does. The page's `Render` is [`page_views!`]'s.
use gpui::{prelude::*, *};
use gpui_kit_material::color::Appearance;
use gpui_kit_material::color::themes::{Contrast, ThemeId, ThemeMode};
use gpui_kit_material::motion;
use gpui_kit_material::prelude::PaletteId;
use std::{
    cell::{Cell, RefCell},
    ops::RangeInclusive,
    rc::Rc,
};

pub use super::spec::PageInfo;

/// Gallery-wide settings and overlays any page may change. The Gallery
/// observes this entity: theme, mode, contrast, palette and scale changes
/// start the animated theme transition; others re-render the Gallery.
pub struct GalleryShared {
    pub palette: PaletteId,
    /// `None` paints `palette` instead of a reference theme.
    pub reference_theme: Option<ThemeId>,
    pub reference_mode: ThemeMode,
    /// Whether the mode follows the system's light or dark appearance
    /// ([`Appearance::System`], until the user picks one) or keeps the user's.
    pub appearance: Appearance,
    /// A theme generated from a seed color, painted in `reference_mode` and
    /// `contrast` in place of the reference theme or palette.
    pub seed: Option<gpui_kit_material::color::dynamic::SeedScheme>,
    /// The live preview's own theme and mode (md3.io's preview controls). `None`
    /// follows the application.
    pub preview_theme: Option<ThemeId>,
    pub preview_mode: Option<ThemeMode>,
    pub contrast: Contrast,
    /// The text scale, as the option value ("1", "1.25", …).
    pub scale: SharedString,
    pub motion_scheme: motion::MotionScheme,
    /// Shows the Gallery's sample snackbar.
    pub snackbar: bool,
    scheme_memo: Cell<SchemeMemo>,
}

impl Default for GalleryShared {
    fn default() -> Self {
        Self {
            palette: PaletteId::Paper,
            reference_theme: Some(ThemeId::Baseline),
            reference_mode: ThemeMode::Light,
            appearance: Appearance::System,
            seed: None,
            preview_theme: None,
            preview_mode: None,
            contrast: Contrast::Standard,
            scale: "1".into(),
            motion_scheme: motion::MotionScheme::Expressive,
            snackbar: false,
            scheme_memo: Default::default(),
        }
    }
}

/// What a scheme is built from: the palette and theme, seed, mode and contrast.
type SchemeSource = (
    PaletteId,
    Option<ThemeId>,
    Option<gpui_kit_material::color::dynamic::SeedScheme>,
    ThemeMode,
    Contrast,
);
/// The application's and the preview's last schemes. Harmonizing a scheme's tints costs
/// about 85k instructions, and a preview with its own mode asks twice per frame it draws, so
/// each is kept until what it was built from changes, which the key says.
type SchemeMemo = [Option<(SchemeSource, gpui_kit_material::color::Scheme)>; 2];

impl GalleryShared {
    fn memoized_scheme(
        &self,
        slot: usize,
        source: SchemeSource,
        build: impl FnOnce() -> gpui_kit_material::color::Scheme,
    ) -> gpui_kit_material::color::Scheme {
        let mut memo = self.scheme_memo.get();
        if let Some((held, scheme)) = memo[slot]
            && held == source
        {
            return scheme;
        }
        let scheme = build();
        memo[slot] = Some((source, scheme));
        self.scheme_memo.set(memo);
        scheme
    }
    pub fn target_scheme(&self) -> gpui_kit_material::color::Scheme {
        let source = (
            self.palette,
            self.reference_theme,
            self.seed,
            self.reference_mode,
            self.contrast,
        );
        self.memoized_scheme(0, source, || {
            if let Some(seed) = self.seed_scheme(self.reference_mode) {
                return seed;
            }
            self.reference_theme.map_or_else(
                || gpui_kit_material::color::Scheme::from_palette(self.palette),
                |id| id.scheme(self.reference_mode, self.contrast),
            )
        })
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
    /// its own keeps the application's seed, if one is painted, or its palette's
    /// family.
    pub fn preview_scheme(&self) -> Option<gpui_kit_material::color::Scheme> {
        if self.preview_theme.is_none() && self.preview_mode.is_none() {
            return None;
        }
        let mode = self.preview_mode.unwrap_or(self.painted_mode());
        let theme = self.preview_theme.or(self.reference_theme);
        let seed = self.seed.filter(|_| self.preview_theme.is_none());
        let palette = palette_for(self.palette, mode);
        let source = (palette, theme, seed, mode, self.contrast);
        Some(self.memoized_scheme(1, source, || {
            if seed.is_some()
                && let Some(seed) = self.seed_scheme(mode)
            {
                return seed;
            }
            theme.map_or_else(
                || gpui_kit_material::color::Scheme::from_palette(palette),
                |theme| theme.scheme(mode, self.contrast),
            )
        }))
    }
    /// Whether the application paints `palette` rather than a reference theme
    /// or a seed.
    pub fn paints_palette(&self) -> bool {
        self.reference_theme.is_none() && self.seed.is_none()
    }
    /// The mode of the scheme the application paints. A palette is light or
    /// dark of itself, whatever `reference_mode` says.
    pub fn painted_mode(&self) -> ThemeMode {
        if self.paints_palette() {
            if self.palette.is_dark() {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            }
        } else {
            self.reference_mode
        }
    }
    /// Paint the application light or dark, keeping what it paints: a palette
    /// turns into its family's other mode, a theme or seed changes its mode.
    pub fn set_dark(&mut self, dark: bool) {
        let mode = if dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        self.reference_mode = mode;
        if self.paints_palette() {
            self.palette = palette_for(self.palette, mode);
        }
    }
    /// The user picked light or dark: paint it, and stop following the system.
    pub fn choose_dark(&mut self, dark: bool) {
        self.appearance = if dark {
            Appearance::Dark
        } else {
            Appearance::Light
        };
        self.set_dark(dark);
    }
    /// The system's appearance is (now) dark or light: paint it unless the user
    /// chose a mode. Returns whether the painted mode changed.
    pub fn follow_system(&mut self, system_dark: bool) -> bool {
        let mode = if system_dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        if self.appearance != Appearance::System || self.painted_mode() == mode {
            return false;
        }
        self.set_dark(system_dark);
        true
    }
    /// The live preview's overrides, as a [`Scope`]. With none set it changes nothing.
    pub fn preview_scope(&self) -> gpui_kit_material::scope::Scope {
        let mut scope = gpui_kit_material::scope::Scope::new("preview-scope");
        if let Some(scheme) = self.preview_scheme() {
            scope = scope.scheme(scheme);
        }
        scope
    }
    pub fn text_scale(&self) -> f32 {
        self.scale.parse().unwrap_or(1.)
    }
}

/// The palette of `palette`'s family in `mode`: itself when it is already
/// that light or dark.
pub fn palette_for(palette: PaletteId, mode: ThemeMode) -> PaletteId {
    let dark = mode == ThemeMode::Dark;
    if palette.is_dark() == dark {
        return palette;
    }
    match palette {
        PaletteId::Paper => PaletteId::Ink,
        PaletteId::Ink => PaletteId::Paper,
        PaletteId::NordSnow => PaletteId::Nord,
        PaletteId::Nord => PaletteId::NordSnow,
        PaletteId::TokyoDay => PaletteId::TokyoNight,
        PaletteId::TokyoNight => PaletteId::TokyoDay,
        PaletteId::CatppuccinLatte => PaletteId::CatppuccinMocha,
        PaletteId::CatppuccinFrappe
        | PaletteId::CatppuccinMacchiato
        | PaletteId::CatppuccinMocha => PaletteId::CatppuccinLatte,
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

/// One design-kit page. Its [`Render`] builds the row asked for: see
/// [`page_views!`].
pub trait GalleryPage: Render + Sized + 'static {
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
    /// to forward controlled changes to playbacks of unmounted rows. A
    /// forwarded transition that runs requests the next frame.
    fn after_layout(
        &mut self,
        _visible: &dyn Fn(usize) -> bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }
    fn apply_capture(&mut self, _state: CaptureState, _cx: &mut Context<Self>) {}
}

/// A page behind its entity, without its type.
pub(super) trait AnyPage {
    fn row_count(&self, window: &Window, cx: &App) -> usize;
    fn row_gap(&self, row: usize, cx: &App) -> f32;
    fn interactive(&self, row: usize, cx: &App) -> bool;
    fn render_row(&self, row: usize, window: &mut Window, cx: &mut App) -> AnyElement;
    fn prepare(&self, window: &mut Window, cx: &mut App);
    fn after_layout(&self, visible: &dyn Fn(usize) -> bool, window: &mut Window, cx: &mut App);
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
        render_row_view(self, row, window, cx)
    }
    fn prepare(&self, window: &mut Window, cx: &mut App) {
        if P::PREPARES {
            self.update(cx, |page, cx| page.prepare(window, cx))
        }
    }
    fn observes_layout(&self) -> bool {
        P::OBSERVES_LAYOUT
    }
    fn after_layout(&self, visible: &dyn Fn(usize) -> bool, window: &mut Window, cx: &mut App) {
        self.update(cx, |page, cx| page.after_layout(visible, window, cx))
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

thread_local! {
    /// The row the page rendered next builds: see [`render_row_view`].
    static REQUESTED_ROW: Cell<Option<usize>> = const { Cell::new(None) };
}

/// Body row `row` of `page`, built as the engine renders a view (see the
/// module's "How a row is built").
pub(super) fn render_row_view<P: GalleryPage>(
    page: &Entity<P>,
    row: usize,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    REQUESTED_ROW.set(Some(row));
    render_view(page, window, cx)
}

/// `entity` rendered in place, as the engine renders a view, for what its
/// `Render` was asked to build. Upstream GPUI has no retained subtrees: an
/// update is all a render is.
pub(super) fn render_view<T: Render>(
    entity: &Entity<T>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    #[cfg(feature = "gpui-fast")]
    return gpui::View::render(entity.clone(), window, cx).into_any_element();
    #[cfg(not(feature = "gpui-fast"))]
    entity.update(cx, |view, cx| view.render(window, cx).into_any_element())
}

/// What a page's [`Render`] builds: the row [`render_row_view`] asked for. A
/// page rendered any other way (as a window's root) builds nothing.
pub fn render_requested_row<P: GalleryPage>(
    page: &mut P,
    window: &mut Window,
    cx: &mut Context<P>,
) -> AnyElement {
    match REQUESTED_ROW.take() {
        Some(row) => page.render_row(row, window, cx),
        None => div().into_any_element(),
    }
}

/// Gives each page type its [`Render`], which builds the row asked for.
macro_rules! page_views {
    ($($page:ty),* $(,)?) => {$(
        impl gpui::Render for $page {
            fn render(
                &mut self,
                window: &mut gpui::Window,
                cx: &mut gpui::Context<Self>,
            ) -> impl gpui::IntoElement {
                $crate::page::render_requested_row(self, window, cx)
            }
        }
    )*};
}
pub(crate) use page_views;

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

/// The rule rows and playground parts are built by: building from an entity as
/// its view draws it, which leaves what read the entity valid, unless the build
/// notifies; an update would build the readers again.
// Fast retains views: upstream builds every view again on each draw.
#[cfg(all(test, feature = "gpui-fast"))]
mod build_tests {
    use super::*;
    use core::prelude::v1::test;

    struct Shared {
        notify: bool,
    }
    impl Render for Shared {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.notify {
                cx.notify();
            }
            div().child("built")
        }
    }
    struct Builder {
        shared: Entity<Shared>,
        update: bool,
    }
    impl Render for Builder {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.update {
                self.shared.update(cx, |shared, cx| {
                    shared.render(window, cx).into_any_element()
                })
            } else {
                render_view(&self.shared, window, cx)
            }
        }
    }
    struct Reader {
        shared: Entity<Shared>,
        renders: Rc<Cell<usize>>,
    }
    impl Render for Reader {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.renders.set(self.renders.get() + 1);
            div().child(if self.shared.read(cx).notify {
                "a"
            } else {
                "b"
            })
        }
    }
    struct Root(Entity<Builder>, Entity<Reader>);
    impl Render for Root {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().child(self.0.clone()).child(self.1.clone())
        }
    }

    /// How many times the reader renders over frames that build the builder again.
    fn reader_renders(cx: &mut gpui::TestAppContext, update: bool, notify: bool) -> usize {
        let renders = Rc::new(Cell::new(0));
        let mut builder = None;
        let window = cx.add_window({
            let renders = renders.clone();
            let builder = &mut builder;
            move |_, cx| {
                let shared = cx.new(|_| Shared { notify });
                let built = cx.new(|_| Builder {
                    shared: shared.clone(),
                    update,
                });
                *builder = Some(built.clone());
                Root(built, cx.new(|_| Reader { shared, renders }))
            }
        });
        let builder = builder.unwrap();
        let draw = |cx: &mut gpui::TestAppContext| {
            cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
                .unwrap();
        };
        draw(cx);
        let before = renders.get();
        for _ in 0..3 {
            cx.update(|cx| cx.notify(builder.entity_id()));
            draw(cx);
        }
        renders.get() - before
    }

    #[gpui::test]
    fn building_as_a_view_leaves_readers_valid_unless_it_notifies(cx: &mut gpui::TestAppContext) {
        assert_eq!(
            reader_renders(cx, false, false),
            0,
            "a view's build is drawing it"
        );
        assert!(reader_renders(cx, true, false) > 0, "an update is a write");
        assert!(reader_renders(cx, false, true) > 0, "a notify is a change");
    }
}

#[cfg(test)]
mod scheme_tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn a_kept_scheme_follows_what_it_was_built_from() {
        let mut shared = GalleryShared::default();
        assert_eq!(shared.preview_scheme(), None);
        let expect = |theme: ThemeId, mode, contrast| theme.scheme(mode, contrast);
        assert_eq!(
            shared.target_scheme(),
            expect(ThemeId::Baseline, ThemeMode::Light, Contrast::Standard)
        );
        shared.preview_mode = Some(ThemeMode::Dark);
        for _ in 0..2 {
            assert_eq!(
                shared.preview_scheme(),
                Some(expect(
                    ThemeId::Baseline,
                    ThemeMode::Dark,
                    Contrast::Standard
                ))
            );
        }
        shared.preview_theme = Some(ThemeId::Ocean);
        assert_eq!(
            shared.preview_scheme(),
            Some(expect(ThemeId::Ocean, ThemeMode::Dark, Contrast::Standard))
        );
        shared.contrast = Contrast::High;
        assert_eq!(
            shared.preview_scheme(),
            Some(expect(ThemeId::Ocean, ThemeMode::Dark, Contrast::High))
        );
        shared.reference_mode = ThemeMode::Dark;
        assert_eq!(
            shared.target_scheme(),
            expect(ThemeId::Baseline, ThemeMode::Dark, Contrast::High)
        );
        shared.reference_theme = None;
        assert_eq!(
            shared.target_scheme(),
            gpui_kit_material::color::Scheme::from_palette(shared.palette)
        );
    }

    /// A dark palette reads as dark wherever the mode shows, and switching the
    /// mode keeps the palette's family instead of falling back to a theme.
    #[test]
    fn the_mode_follows_the_painted_palette_and_keeps_it() {
        use gpui_kit_material::color::Scheme;
        let mut shared = GalleryShared {
            palette: PaletteId::TokyoNight,
            reference_theme: None,
            ..GalleryShared::default()
        };
        assert_eq!(shared.reference_mode, ThemeMode::Light);
        assert_eq!(shared.painted_mode(), ThemeMode::Dark);
        shared.set_dark(false);
        assert_eq!(shared.reference_theme, None, "the palette stays painted");
        assert_eq!(shared.palette, PaletteId::TokyoDay);
        assert_eq!(shared.painted_mode(), ThemeMode::Light);
        shared.set_dark(true);
        assert_eq!(shared.palette, PaletteId::TokyoNight);
        // The preview's own mode keeps the palette's family too.
        shared.preview_mode = Some(ThemeMode::Light);
        assert_eq!(
            shared.preview_scheme(),
            Some(Scheme::from_palette(PaletteId::TokyoDay))
        );
        // Every palette has a family member in each mode.
        for palette in gpui_kit_material::color::palette::ALL {
            for mode in [ThemeMode::Light, ThemeMode::Dark] {
                assert_eq!(
                    palette_for(palette, mode).is_dark(),
                    mode == ThemeMode::Dark,
                    "{palette:?} in {mode:?}"
                );
            }
        }
        // A reference theme only changes its mode.
        shared.reference_theme = Some(ThemeId::Ocean);
        shared.set_dark(false);
        assert_eq!(shared.painted_mode(), ThemeMode::Light);
        assert_eq!(shared.reference_theme, Some(ThemeId::Ocean));
    }

    /// The mode follows the system appearance, at start and as it changes,
    /// until the user picks light or dark; their pick then stays.
    #[test]
    fn the_mode_follows_the_system_until_the_user_picks_one() {
        let mut shared = GalleryShared::default();
        assert_eq!(shared.appearance, Appearance::System);
        assert!(shared.follow_system(true));
        assert_eq!(shared.painted_mode(), ThemeMode::Dark);
        assert!(
            !shared.follow_system(true),
            "an unchanged appearance changes nothing"
        );
        assert!(shared.follow_system(false));
        assert_eq!(shared.painted_mode(), ThemeMode::Light);
        // A painted palette follows within its family.
        shared.reference_theme = None;
        assert!(shared.follow_system(true));
        assert_eq!(shared.palette, PaletteId::Ink);
        // The user's pick holds against later system changes, either way.
        shared.choose_dark(false);
        assert_eq!(shared.appearance, Appearance::Light);
        assert_eq!(shared.palette, PaletteId::Paper);
        assert!(!shared.follow_system(true));
        assert_eq!(shared.painted_mode(), ThemeMode::Light);
        shared.choose_dark(true);
        assert_eq!(shared.appearance, Appearance::Dark);
        assert!(!shared.follow_system(false));
        assert_eq!(shared.painted_mode(), ThemeMode::Dark);
    }
}
