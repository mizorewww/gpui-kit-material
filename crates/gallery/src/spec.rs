//! Design-kit page presentation: section headers, specimen boards and
//! their captions, modelled on the component pages of Google's Figma
//! "Material 3 Design Kit". The Gallery shows them on each component's
//! Specs tab and as the body of Styles and Examples pages, one virtual row
//! per page row; the shell frames each row as a slice of its card (see
//! `super::page`).
//!
//! Boards are dashed specimen frames. A board taller than a comfortable row
//! is split into bands, one per virtual row: [`Band::Top`], any number of
//! [`Band::Middle`] and a [`Band::Bottom`] draw one continuous outline.
use gpui::{prelude::*, *};
use gpui_kit_material::icons::{Icon, Symbol};
use gpui_kit_material::prelude::*;
use std::sync::OnceLock;

/// The default space above a body row (see `GalleryPage::row_gap`).
pub const ROW_GAP: f32 = 24.;
/// The space between a page's header and its first specimen.
pub const HEADER_TO_BODY: f32 = 48.;

/// The Gallery's page background (md3.io's page behind its cards).
pub fn page_background() -> Rgba {
    color(ColorRole::SURFACE)
}

/// Apply an M3 type role's family, size, line height and weight.
pub fn type_role<E: Styled>(element: E, role: TypeRole) -> E {
    let scale = role.scale();
    element
        .font_family(scale.font_family)
        .text_size(scale.font_size)
        .line_height(scale.line_height)
        .font_weight(FontWeight(scale.font_weight as f32))
}

/// The kit's star placeholder icon (`stars`, filled), resolved once.
pub fn star_icon() -> Icon {
    static PATH: OnceLock<String> = OnceLock::new();
    Icon::new(SharedString::new_static(
        PATH.get_or_init(|| {
            Symbol::named("stars")
                .expect("stars belongs to the Material Symbols catalog")
                .icon()
                .filled(true)
                .asset_path()
        })
        .as_str(),
    ))
}

/// Static description of a page or of a sub-section of one.
#[derive(Clone, Copy, Debug)]
pub struct PageInfo {
    /// Stable id: navigation, `--section <id>` captures and element ids.
    pub id: &'static str,
    /// The kit's page title.
    pub title: &'static str,
    /// The kit's one-paragraph description.
    pub description: &'static str,
    /// The m3.material.io guideline the "See design guideline" link opens.
    pub guideline: Option<&'static str>,
}

/// A per-page element id, e.g. `page-title-buttons`.
fn named(prefix: &str, id: &str) -> ElementId {
    ElementId::Name(format!("{prefix}-{id}").into())
}

/// The link label of every header.
const GUIDELINE_LINK: &str = "See design guideline";

fn guideline_link(id: &'static str, url: &'static str, size: TypeRole) -> impl IntoElement {
    FocusRing::new(
        named("guideline-ring", id),
        gpui_base::Button::new(named("guideline", id))
            .role(Role::Link)
            .accessibility_label(GUIDELINE_LINK)
            .cursor_pointer()
            .text_color(color(ColorRole::PRIMARY))
            .underline()
            .map(|link| type_role(link, size))
            .font_weight(FontWeight::NORMAL)
            .child(GUIDELINE_LINK)
            .on_click(move |_, _, cx| cx.open_url(url)),
    )
}

/// A sub-section header inside a page (e.g. "Icon button" on Buttons): a
/// headline title, its guideline link and description, over a divider.
pub fn section_header(info: &PageInfo) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .items_start()
        .gap(rem(12.))
        .pt(rem(24.))
        .child(
            accessible_text(named("section-title", info.id), info.title)
                .role(Role::Heading)
                .map(|title| type_role(title, TypeRole::DisplaySmall))
                .text_color(color(ColorRole::ON_SURFACE)),
        )
        .when_some(info.guideline, |column, url| {
            column.child(guideline_link(info.id, url, TypeRole::BodyLarge))
        })
        .child(
            accessible_text(named("section-description", info.id), info.description)
                .max_w(rem(720.))
                .whitespace_normal()
                .map(|text| type_role(text, TypeRole::BodyLarge))
                .text_color(color(ColorRole::ON_SURFACE)),
        )
        .child(
            div()
                .w_full()
                .pt(rem(12.))
                .child(MaterialDivider::new(named("section-divider", info.id))),
        )
}

/// A small specimen label (column and side labels): the kit's label-medium.
pub fn small_label(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Stateful<Div> {
    accessible_text(id, text)
        .map(|text| type_role(text, TypeRole::LabelMedium))
        .text_color(color(ColorRole::ON_SURFACE))
}

/// Which slice of a dashed outline a board band draws.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Band {
    /// A board in one row.
    #[default]
    Whole,
    /// The first band: top edge and corners, and the board's caption.
    Top,
    /// A band between the first and last: side edges only.
    Middle,
    /// The last band: bottom edge and corners.
    Bottom,
}

impl Band {
    /// The band of `index` among `count` bands.
    pub fn of(index: usize, count: usize) -> Self {
        match (index, count) {
            (_, 0 | 1) => Band::Whole,
            (0, _) => Band::Top,
            (i, n) if i + 1 == n => Band::Bottom,
            _ => Band::Middle,
        }
    }
    fn has_top(self) -> bool {
        matches!(self, Band::Whole | Band::Top)
    }
    fn has_bottom(self) -> bool {
        matches!(self, Band::Whole | Band::Bottom)
    }
    /// The row gap a page returns for a row holding this band: bands after
    /// the first continue the outline without a gap.
    pub fn row_gap(self) -> f32 {
        if self.has_top() { ROW_GAP } else { 0. }
    }
}

/// The board outline: a 1px dashed line in a translucent primary (the kit
/// draws component sets in its own purple; this keeps it theme-aware).
pub fn board_outline() -> Rgba {
    color(ColorRole::PRIMARY).opacity(0.5)
}
const BOARD_RADIUS: f32 = 5.;
const BOARD_PADDING: f32 = 20.;
const CELL_GAP: f32 = 16.;
/// Wide enough for side labels like "Center-aligned hero" to wrap between
/// words rather than inside one.
const SIDE_GUTTER: f32 = 96.;

/// A dashed specimen board, or one band of a board split across rows.
///
/// Cells fill rows of `columns` in a grid whose columns are as wide as
/// their widest specimen, with spare width spread around them (or are
/// `column_width` design px when set). Specimens never overlap or wrap.
///
/// Boards never scroll sideways: when the columns do not fit the row, the
/// board wraps them into stacked groups of adjacent columns, each with its
/// own column labels and row labels (see [`wrap_columns`]). Measured column
/// widths are remembered per board (see `MEASURES`); a board drawn for the
/// first time shows one group, clipped, for that frame only.
///
/// `caption` appears centered above the top band; `column_labels` title the
/// columns inside the top band and inside every group of a wrapped band;
/// `row_labels` title each cell row at its leading edge (e.g. interaction
/// states); `side_label` sits outside the band's leading edge, level with
/// its first cell row (the kit's "Round"/"Square"). Give every band of one
/// board the same columns, widths and labels, and if any band has a side
/// label, give each a side label (possibly empty).
#[derive(gpui::IntoElement)]
pub struct Board {
    id: ElementId,
    band: Band,
    columns: usize,
    column_width: Option<f32>,
    caption: Option<SharedString>,
    column_labels: Vec<SharedString>,
    row_labels: Vec<SharedString>,
    side_label: Option<SharedString>,
    cells: Vec<AnyElement>,
}

pub fn board(id: impl Into<ElementId>) -> Board {
    Board {
        id: id.into(),
        band: Band::Whole,
        columns: 1,
        column_width: None,
        caption: None,
        column_labels: Vec::new(),
        row_labels: Vec::new(),
        side_label: None,
        cells: Vec::new(),
    }
}

impl Board {
    pub fn band(mut self, band: Band) -> Self {
        self.band = band;
        self
    }
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns.max(1);
        self
    }
    pub fn column_width(mut self, width: f32) -> Self {
        self.column_width = Some(width);
        self
    }
    pub fn caption(mut self, caption: impl Into<SharedString>) -> Self {
        self.caption = Some(caption.into());
        self
    }
    pub fn column_labels<S: Into<SharedString>>(
        mut self,
        labels: impl IntoIterator<Item = S>,
    ) -> Self {
        self.column_labels = labels.into_iter().map(Into::into).collect();
        self
    }
    pub fn row_labels<S: Into<SharedString>>(
        mut self,
        labels: impl IntoIterator<Item = S>,
    ) -> Self {
        self.row_labels = labels.into_iter().map(Into::into).collect();
        self
    }
    pub fn side_label(mut self, label: impl Into<SharedString>) -> Self {
        self.side_label = Some(label.into());
        self
    }
    pub fn cell(mut self, cell: impl IntoElement) -> Self {
        self.cells.push(cell.into_any_element());
        self
    }
    pub fn cells<E: IntoElement>(mut self, cells: impl IntoIterator<Item = E>) -> Self {
        self.cells
            .extend(cells.into_iter().map(IntoElement::into_any_element));
        self
    }
}

const ROW_LABEL_WIDTH: f32 = 88.;

/// Measured board geometry, in px, keyed by each board's global element id.
#[derive(Clone, Default)]
struct Measure {
    /// The max-content width of each column, then of the row labels.
    columns: Vec<f32>,
    /// The width the band's frame may take.
    available: f32,
}

thread_local! {
    /// Measured board geometry. Each board records its columns and room on
    /// every prepaint and owns its entry's invalidation: when a new
    /// measurement changes the board's column groups (first draw, window
    /// resize, text scale or font changes), it re-renders the view that drew
    /// it once. Entries are a few floats per board; the map is cleared if it
    /// ever grows past `MEASURE_LIMIT` boards.
    static MEASURES: std::cell::RefCell<std::collections::HashMap<GlobalElementId, Measure>> =
        Default::default();
}
const MEASURE_LIMIT: usize = 4096;

#[cfg(test)]
thread_local! {
    /// Measured boards whose frame was drawn wider than their row, and demos
    /// whose content was drawn past their row's edge.
    pub static OVERFLOWS: std::cell::RefCell<Vec<String>> = const {
        std::cell::RefCell::new(Vec::new())
    };
    /// The smallest scale a measured board drew a group at, with the board.
    pub static BOARD_SCALES: std::cell::RefCell<Vec<(String, f32)>> = const {
        std::cell::RefCell::new(Vec::new())
    };
}

thread_local! {
    /// The horizontal extent of what each [`FitWidth`] being prepainted drew,
    /// innermost last: its content and the items of its demo rows.
    static EXTENTS: std::cell::RefCell<Vec<(Pixels, Pixels)>> = const {
        std::cell::RefCell::new(Vec::new())
    };
    /// Each [`FitWidth`]'s scale, for the width and rem it was found at. Each
    /// owns its entry's invalidation as a board does: a measurement that
    /// changes the scale re-renders the view that drew it once. Cleared if it
    /// ever grows past `MEASURE_LIMIT` entries.
    static FITS: std::cell::RefCell<std::collections::HashMap<GlobalElementId, Fit>> =
        Default::default();
}

/// What a [`FitWidth`] found: the scale its content fits at, in a window of
/// `viewport` at `rem`. A new window size or rem starts again from the full
/// size. Otherwise it shrinks when its content overflows, and grows back as
/// far as the content would fit, judged from its width at the current scale,
/// so content that changed (a smaller specimen picked) is drawn at its size
/// again. Content whose row is sized by the content itself
/// never looks as if it would fit, so it settles instead of trading size with
/// its row.
#[derive(Clone, Copy)]
struct Fit {
    step: FitStep,
    viewport: Size<Pixels>,
    rem: Pixels,
}

/// The scale a [`FitWidth`] draws at and what its last frames found, apart
/// from the window it was found in.
#[derive(Clone, Copy, Debug, PartialEq)]
struct FitStep {
    scale: f32,
    /// The last frame overflowed: one that overflows again shrinks. A single
    /// frame's overflow can be a preview that sizes itself on its next frame.
    /// Content still overflowing at the least scale keeps it, so that it
    /// stays settled rather than asking for a frame on every render.
    overflowed: bool,
    /// Whether the content may go back to the full size, see [`Regrow`].
    regrow: Regrow,
}

/// Growing estimates the content's full width from its width at a smaller
/// scale, which content that does not scale in proportion can get wrong. A
/// try that overflows again blocks the next until the content's width at its
/// scale changes, so a wrong estimate cannot loop.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Regrow {
    Allowed,
    /// A larger scale was tried from an estimate; overflowing again blocks it.
    Trying,
    /// A try overflowed; the width to measure at the new scale is still to come.
    Failed,
    /// Blocked while the content is this wide at its scale.
    Blocked(f32),
}

/// The least growth worth another frame.
const REGROW_STEP: f32 = 1.02;

impl FitStep {
    const FULL: Self = Self {
        scale: 1.,
        overflowed: false,
        regrow: Regrow::Allowed,
    };

    /// The step after a frame drew the content `width` px wide in a row
    /// `available` px wide, never shrinking below `least`; and whether that
    /// frame needs another, drawn at the new scale.
    fn next(self, width: f32, available: f32, least: f32) -> (Self, bool) {
        let scale = self.scale;
        let overflowing = width > available + 1.;
        if overflowing {
            if !self.overflowed {
                // Confirm on the next frame.
                let step = Self {
                    overflowed: true,
                    ..self
                };
                return (step, true);
            }
            let next = fit_scale_down_to(width / scale, available, least).min(scale);
            if next < scale {
                let regrow = match self.regrow {
                    Regrow::Trying => Regrow::Failed,
                    _ => Regrow::Allowed,
                };
                let step = Self {
                    scale: next,
                    overflowed: false,
                    regrow,
                };
                return (step, true);
            }
            // Nothing smaller to draw it at: it stays clipped, and settled.
            return (self, false);
        }
        let regrow = match self.regrow {
            Regrow::Trying => Regrow::Allowed,
            Regrow::Failed => Regrow::Blocked(width),
            Regrow::Blocked(at) if (width - at).abs() > 1. => Regrow::Allowed,
            regrow => regrow,
        };
        let settled = Self {
            scale,
            overflowed: false,
            regrow,
        };
        // The content would fit at a larger scale, the full size if it can:
        // try it. Content in proportion to its scale lands at its fit at once;
        // content with parts that do not scale looks wider than it is, so it
        // gets there in steps.
        let fits = fit_scale_down_to(width / scale, available, least);
        if regrow == Regrow::Allowed && fits > scale * REGROW_STEP {
            let step = Self {
                scale: fits,
                overflowed: false,
                regrow: Regrow::Trying,
            };
            return (step, true);
        }
        (settled, false)
    }
}

/// Records how far `bounds` reach in the [`FitWidth`] being prepainted.
fn note_extent(bounds: Bounds<Pixels>) {
    EXTENTS.with(|extents| {
        if let Some((left, right)) = extents.borrow_mut().last_mut() {
            *left = (*left).min(bounds.left());
            *right = (*right).max(bounds.right());
        }
    });
}

/// Content that is drawn at a smaller rem when it is wider than its row: a
/// specimen of fixed reference width, or a demo row's item, on a phone or at
/// a large text scale. Content that reflows never needs it. Content that
/// overflows on two frames in a row is drawn at the scale that fits on the
/// next.
#[derive(gpui::IntoElement)]
pub(crate) struct FitWidth {
    id: ElementId,
    content: AnyElement,
    centered: bool,
}

/// `content` in a [`FitWidth`] that keeps it at its leading edge.
pub(crate) fn fit_width(id: impl Into<ElementId>, content: impl IntoElement) -> FitWidth {
    FitWidth {
        id: id.into(),
        content: content.into_any_element(),
        centered: false,
    }
}

impl FitWidth {
    /// Center the content in the row, as a preview canvas does.
    pub(crate) fn centered(mut self) -> Self {
        self.centered = true;
        self
    }
}

impl RenderOnce for FitWidth {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        let key = window.with_global_id(self.id.clone(), |id, _| id.clone());
        let rem = window.rem_size();
        let viewport = window.viewport_size();
        let view = window.current_view();
        let found = FITS.with(|fits| fits.borrow().get(&key).copied());
        let stored = found
            .filter(|fit| fit.rem == rem && fit.viewport == viewport)
            .map(|fit| fit.step);
        let step = stored.unwrap_or(FitStep::FULL);
        let scale = step.scale;
        let content = if scale < 1. {
            RemScaled::new(rem * scale, self.content).into_any_element()
        } else {
            self.content
        };
        let start = prepaint_bounds(div(), |_, _, _| {
            EXTENTS.with(|extents| extents.borrow_mut().push((px(f32::MAX), px(f32::MIN))))
        });
        let content = prepaint_bounds(content, |bounds, _, _| note_extent(bounds));
        let holder = div()
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .when(self.centered, |holder| holder.items_center())
            .child(start)
            .child(content);
        prepaint_bounds(holder, move |row, _, cx| {
            let Some((left, right)) = EXTENTS.with(|extents| extents.borrow_mut().pop()) else {
                return;
            };
            let available = f32::from(row.size.width);
            let width = f32::from(right - left);
            let (next, redraw) = step.next(width, available, min_fit_scale(rem));
            // Nothing smaller to draw it at: it is clipped.
            #[cfg(test)]
            if next.overflowed && step.overflowed && next.scale >= scale {
                OVERFLOWS.with(|overflows| {
                    overflows
                        .borrow_mut()
                        .push(format!("{key}: {width:?} wide in {available:?} at {scale}"))
                });
            }
            // Only a change is written.
            if stored != Some(next) {
                FITS.with(|fits| {
                    let mut fits = fits.borrow_mut();
                    if fits.len() >= MEASURE_LIMIT && !fits.contains_key(&key) {
                        fits.clear();
                    }
                    fits.insert(
                        key,
                        Fit {
                            step: next,
                            viewport,
                            rem,
                        },
                    );
                });
            }
            if redraw {
                cx.defer(move |cx| cx.notify(view));
            }
        })
    }
}

/// Split columns of `widths` px into runs of adjacent columns that each fit
/// `available` px, given `lead` px before each run's first column (row
/// labels) and `gap` px between columns. Every run holds at least one
/// column; a column wider than `available` gets a run of its own.
pub fn wrap_columns(
    widths: &[f32],
    lead: f32,
    gap: f32,
    available: f32,
) -> Vec<std::ops::Range<usize>> {
    let mut runs = Vec::new();
    let mut start = 0;
    let mut used = lead;
    for (index, width) in widths.iter().enumerate() {
        let needed = if index == start { *width } else { gap + width };
        // Half a pixel of slack absorbs layout rounding.
        if index > start && used + needed > available + 0.5 {
            runs.push(start..index);
            start = index;
            used = lead + width;
        } else {
            used += needed;
        }
    }
    if start < widths.len() || runs.is_empty() {
        runs.push(start..widths.len());
    }
    runs
}

/// The least a board's group shrinks to at the 100% text scale. Four
/// extra-large buttons in a row need it on a phone.
const MIN_FIT_SCALE: f32 = 0.15;

/// The least a group is drawn at as a share of the 100% text size, whatever
/// the text scale: a larger scale draws the same specimen wider in the same
/// window, with wider gutters, and it still has to fit rather than be clipped.
/// Extra-large button groups on a 320px phone at 200% need it.
const MIN_DRAWN_SCALE: f32 = 0.1;

/// The least a group shrinks to at `rem`.
pub(crate) fn min_fit_scale(rem: Pixels) -> f32 {
    (MIN_DRAWN_SCALE * BASE_PX / f32::from(rem)).min(MIN_FIT_SCALE)
}

/// The scale that fits `needed` px into `available` px: 1 when it fits,
/// else a little under the ratio (borders do not scale), never below
/// [`MIN_FIT_SCALE`].
#[cfg(test)]
pub fn fit_scale(needed: f32, available: f32) -> f32 {
    fit_scale_down_to(needed, available, MIN_FIT_SCALE)
}

/// [`fit_scale`], never below `least`.
fn fit_scale_down_to(needed: f32, available: f32, least: f32) -> f32 {
    if needed <= available + 0.5 || needed <= 0. {
        1.
    } else {
        (available / needed * 0.97).clamp(least, 1.)
    }
}

/// Lays out, prepaints and paints its child at another rem size: the
/// child's rem-based geometry and type scale with it.
pub(crate) struct RemScaled {
    rem: Pixels,
    child: AnyElement,
}

impl RemScaled {
    pub(crate) fn new(rem: Pixels, child: AnyElement) -> Self {
        Self { rem, child }
    }
}

impl IntoElement for RemScaled {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for RemScaled {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let child = &mut self.child;
        let id = window.with_rem_size(Some(self.rem), |window| child.request_layout(window, cx));
        (id, ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let child = &mut self.child;
        window.with_rem_size(Some(self.rem), |window| child.prepaint(window, cx));
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let child = &mut self.child;
        window.with_rem_size(Some(self.rem), |window| child.paint(window, cx));
    }
}

impl RenderOnce for Board {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        let band = self.band;
        let radius = rem(BOARD_RADIUS);
        let width = self.column_width;
        let labelled_rows = !self.row_labels.is_empty();
        // In a narrow card each row's label goes over its cells, giving the
        // specimens the label column's width (a phone, or a large text scale).
        let narrow = content_width(window) < STACKED_LABELS_CONTENT;
        let stacked_labels = labelled_rows && narrow;
        let columns = self.columns;
        let rem_size = window.rem_size();
        let to_px = |design: f32| f32::from(rem(design).to_pixels(rem_size));
        let gap = to_px(CELL_GAP);
        let least = min_fit_scale(rem_size);
        // The row-label column: at least `ROW_LABEL_WIDTH`, wider for a
        // longer label (labels never break inside a word).
        let min_label = to_px(ROW_LABEL_WIDTH);
        let lead_of = move |label: f32| {
            if labelled_rows && !stacked_labels {
                label.max(min_label) + gap
            } else {
                0.
            }
        };
        // The frame's padding and dashed border around the grid.
        // A narrow card keeps the 100% padding at a larger text scale: the
        // specimens need the room more.
        let side_padding = if narrow {
            to_px(BOARD_PADDING).min(BOARD_PADDING)
        } else {
            to_px(BOARD_PADDING)
        };
        let chrome = 2. * side_padding + 2.;
        let key = window.with_global_id(self.id.clone(), |id, _| id.clone());
        let view = window.current_view();
        let measured = MEASURES.with(|measures| measures.borrow().get(&key).cloned());
        // Fixed column widths need no measurement (row labels still do).
        let fixed = width.map(|width| vec![to_px(width); columns]);
        let measured = measured.filter(|measure| measure.columns.len() == columns + 1);
        let groups = match &measured {
            Some(measure) => {
                let widths = fixed.as_deref().unwrap_or(&measure.columns[..columns]);
                let lead = lead_of(measure.columns[columns]);
                wrap_columns(widths, lead, gap, measure.available - chrome)
            }
            None => std::iter::once(0..columns).collect(),
        };
        // A group still wider than the row (one specimen wider than the
        // row, e.g. an extra-large button group) is drawn at a smaller rem
        // so it fits instead of being clipped.
        let scales: Vec<f32> = groups
            .iter()
            .map(|group| match &measured {
                Some(measure) => {
                    let widths = fixed.as_deref().unwrap_or(&measure.columns[..columns]);
                    let needed = lead_of(measure.columns[columns])
                        + widths[group.clone()].iter().sum::<f32>()
                        + gap * group.len().saturating_sub(1) as f32;
                    fit_scale_down_to(needed, measure.available - chrome, least)
                }
                None => 1.,
            })
            .collect();
        let wrapped = groups.len() > 1;
        // Until the board is measured, keep its first frame inside its row.
        let unmeasured = measured.is_none();
        #[cfg(test)]
        if !unmeasured {
            let least = scales.iter().copied().fold(1f32, f32::min);
            BOARD_SCALES.with(|scales| scales.borrow_mut().push((key.to_string(), least)));
        }

        let cell = move |content: AnyElement| {
            div()
                .flex()
                .items_center()
                .justify_center()
                .when_some(width, |cell, width| cell.w(rem(width)))
                .child(content)
                .into_any_element()
        };
        let row_label = |label: Option<AnyElement>| {
            div()
                .min_w(rem(ROW_LABEL_WIDTH))
                .whitespace_nowrap()
                .flex()
                .items_center()
                .children(label)
                .into_any_element()
        };
        let show_column_labels = !self.column_labels.is_empty() && (band.has_top() || wrapped);
        let column_labels = self.column_labels;
        let row_labels = self.row_labels;
        let mut cells: Vec<Option<AnyElement>> = self.cells.into_iter().map(Some).collect();
        let rows = cells.len().div_ceil(columns);

        // Column widths measured this frame, max over every cell.
        let frame_widths = std::rc::Rc::new(std::cell::RefCell::new(vec![0f32; columns + 1]));
        let mut grids: Vec<AnyElement> = Vec::with_capacity(groups.len());
        for (group_index, group) in groups.iter().enumerate() {
            let span = group.len();
            let grid_columns = span + usize::from(labelled_rows && !stacked_labels);
            let mut items: Vec<AnyElement> = Vec::new();
            // Which board column each grid item measures, if any.
            let mut item_columns: Vec<Option<usize>> = Vec::new();
            if show_column_labels {
                if labelled_rows && !stacked_labels {
                    items.push(row_label(None));
                    item_columns.push(None);
                }
                for column in group.clone() {
                    let label = column_labels.get(column).cloned().unwrap_or_default();
                    items.push(cell(
                        small_label(("column-label", column), label)
                            .whitespace_normal()
                            .text_center()
                            .into_any_element(),
                    ));
                    item_columns.push(Some(column));
                }
            }
            for row in 0..rows {
                if stacked_labels {
                    // A line of its own over the row, across every column. Out
                    // of the flow, so a long label does not widen the columns.
                    let label = row_labels.get(row).cloned().unwrap_or_default();
                    items.push(
                        div()
                            .col_span_full()
                            .relative()
                            .h(rem(STACKED_LABEL_HEIGHT))
                            .when(cfg!(test), |label| {
                                label.debug_selector(move || format!("stacked-row-label-{row}"))
                            })
                            .child(
                                small_label(("row-label", group_index * rows + row), label)
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .right_0()
                                    .truncate(),
                            )
                            .into_any_element(),
                    );
                    item_columns.push(None);
                } else if labelled_rows {
                    let label = row_labels.get(row).map(|label| {
                        small_label(("row-label", group_index * rows + row), label.clone())
                            .into_any_element()
                    });
                    items.push(row_label(label));
                    item_columns.push(Some(columns));
                }
                for column in group.clone() {
                    match cells.get_mut(row * columns + column).and_then(Option::take) {
                        Some(content) => {
                            items.push(cell(content));
                            item_columns.push(Some(column));
                        }
                        // Keep the next row's label in the first column.
                        None => {
                            items.push(div().into_any_element());
                            item_columns.push(None);
                        }
                    }
                }
            }
            let scale = scales[group_index];
            // Each item reports its width, unscaled so that scaling does not
            // feed back into the grouping.
            let items = items.into_iter().zip(item_columns).map(|(item, column)| {
                let widths = frame_widths.clone();
                prepaint_bounds(item, move |bounds, _, _| {
                    if let Some(column) = column {
                        let width = f32::from(bounds.size.width) / scale;
                        let mut widths = widths.borrow_mut();
                        widths[column] = widths[column].max(width);
                    }
                })
            });
            let grid = div()
                .w_full()
                .grid()
                .grid_cols_max_content(grid_columns as u16)
                // Spare width goes around content-sized columns; it never
                // widens one column to match the widest. Wrapped groups pack
                // from the leading edge so their labels and columns line up.
                .map(|grid| {
                    if wrapped && labelled_rows {
                        grid.justify_start()
                    } else {
                        grid.justify_around()
                    }
                })
                .items_center()
                .gap_x(rem(CELL_GAP))
                .gap_y(rem(CELL_GAP))
                .children(items);
            grids.push(
                div()
                    .w_full()
                    .when(group_index > 0, |group| {
                        group
                            .mt(rem(CELL_GAP))
                            .pt(rem(CELL_GAP * 1.5))
                            .border_t_1()
                            .border_dashed()
                            .border_color(board_outline().opacity(0.6))
                    })
                    .map(|group| {
                        if scale < 1. {
                            group.child(RemScaled {
                                rem: rem_size * scale,
                                child: grid.into_any_element(),
                            })
                        } else {
                            group.child(grid)
                        }
                    })
                    .into_any_element(),
            );
        }

        // The band is at least as wide as its row, so spare width spreads
        // around the columns.
        let frame = div()
            .flex_shrink_0()
            .min_w(relative(1.))
            .flex()
            .flex_col()
            .px(px(side_padding))
            .pt(rem(if band.has_top() {
                BOARD_PADDING
            } else {
                CELL_GAP / 2.
            }))
            .pb(rem(if band.has_bottom() {
                BOARD_PADDING
            } else {
                CELL_GAP / 2.
            }))
            .border_dashed()
            .border_color(board_outline())
            .border_l_1()
            .border_r_1()
            .when(band.has_top(), |frame| frame.border_t_1().rounded_t(radius))
            .when(band.has_bottom(), |frame| {
                frame.border_b_1().rounded_b(radius)
            })
            .children(grids);
        // In a narrow card the side label goes over the board: the gutter
        // would take the width the board needs.
        let label_above = content_width(window) < NARROW_CONTENT;
        // The frame's drawn width, recorded by tests to catch overflow.
        let drawn_width = std::rc::Rc::new(std::cell::Cell::new(0f32));
        let rendered = groups;
        let drawn_scales = scales;
        let caption = self.caption.filter(|_| band.has_top());
        div()
            .id(self.id)
            .w_full()
            .flex()
            .flex_col()
            .when_some(caption, |column, caption| {
                column.child(
                    div().w_full().flex().justify_center().pb(rem(12.)).child(
                        accessible_text("board-caption", caption)
                            .map(|text| type_role(text, TypeRole::LabelLarge))
                            .text_color(color(ColorRole::ON_SURFACE)),
                    ),
                )
            })
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_start()
                    .when(label_above, |row| row.flex_col())
                    .when_some(self.side_label, |row, label| {
                        row.child(
                            div()
                                .when(!label_above, |gutter| gutter.w(rem(SIDE_GUTTER)))
                                .flex_shrink_0()
                                .pt(rem(if band.has_top() {
                                    BOARD_PADDING
                                } else {
                                    CELL_GAP / 2.
                                }))
                                .when(!label_above, |gutter| gutter.pr(rem(8.)))
                                .when(label_above, |gutter| gutter.pb(rem(8.)))
                                .child(small_label("side-label", label).whitespace_normal()),
                        )
                    })
                    .child(prepaint_bounds(
                        div()
                            .when(!label_above, |holder| holder.flex_1())
                            .when(label_above, |holder| holder.w_full())
                            .min_w_0()
                            .flex()
                            .flex_row()
                            .when(unmeasured, |holder| holder.overflow_hidden())
                            .when(cfg!(test), |holder| {
                                let drawn = drawn_width.clone();
                                holder.on_children_prepainted(move |frames, _, _| {
                                    if let Some(frame) = frames.first() {
                                        drawn.set(f32::from(frame.size.width));
                                    }
                                })
                            })
                            .child(frame),
                        // Runs after the grids' items: record this frame's
                        // measurements and regroup on the next frame if they
                        // change the groups.
                        move |holder, _, cx| {
                            let available = f32::from(holder.size.width);
                            #[cfg(test)]
                            if !unmeasured && drawn_width.get() > available + 1. {
                                OVERFLOWS.with(|overflows| {
                                    overflows.borrow_mut().push(format!(
                                        "{key}: {} px in {available} px",
                                        drawn_width.get()
                                    ))
                                });
                            }
                            let mut widths = frame_widths.borrow().clone();
                            if let Some(fixed) = &fixed {
                                widths[..columns].copy_from_slice(fixed);
                            }
                            let lead = lead_of(widths[columns]);
                            let regrouped =
                                wrap_columns(&widths[..columns], lead, gap, available - chrome);
                            // A new width can leave the groups alone and still overflow,
                            // until the group draws at its smaller scale. Only that
                            // redraws: chasing every change of scale would not settle,
                            // as borders and rounding don't scale with the rem.
                            let overflowing =
                                regrouped.iter().zip(&drawn_scales).any(|(group, scale)| {
                                    let needed = lead
                                        + widths[group.clone()].iter().sum::<f32>()
                                        + gap * group.len().saturating_sub(1) as f32;
                                    *scale > least && needed * scale > available - chrome + 1.
                                });
                            let changed = regrouped != rendered || overflowing;
                            MEASURES.with(|measures| {
                                let mut measures = measures.borrow_mut();
                                if measures.len() >= MEASURE_LIMIT && !measures.contains_key(&key) {
                                    measures.clear();
                                }
                                measures.insert(
                                    key.clone(),
                                    Measure {
                                        columns: widths,
                                        available,
                                    },
                                );
                            });
                            if changed {
                                cx.defer(move |cx| cx.notify(view));
                            }
                        },
                    )),
            )
    }
}

/// The content width inside a spec card, in design px, for pages that lay
/// out a window-dependent number of columns.
pub fn content_width(window: &Window) -> f32 {
    super::shell::spec_content_width(window)
}

/// Below this content width a board's side label goes over the board.
const NARROW_CONTENT: f32 = 560.;
/// Below this content width, in design px, a board's row labels go over their
/// rows instead of beside them.
const STACKED_LABELS_CONTENT: f32 = 400.;
/// A stacked row label's line: Label medium's 16dp line height.
const STACKED_LABEL_HEIGHT: f32 = 16.;

/// A free-form specimen row: wrapped items with the board cell gap, for
/// live demos that are not a state matrix. An item wider than the row has the
/// demo around it drawn smaller (see [`FitWidth`]), which needs each item's
/// extent: the row itself is never wider than its parent.
pub fn demo_row() -> DemoRow {
    DemoRow {
        row: div()
            .w_full()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(rem(CELL_GAP)),
    }
}

/// See [`demo_row`].
pub struct DemoRow {
    row: Div,
}

impl ParentElement for DemoRow {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.row.extend(elements.into_iter().map(|item| {
            prepaint_bounds(item, |bounds, _, _| note_extent(bounds)).into_any_element()
        }));
    }
}

impl Styled for DemoRow {
    fn style(&mut self) -> &mut StyleRefinement {
        self.row.style()
    }
}

impl InteractiveElement for DemoRow {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.row.interactivity()
    }
}

impl IntoElement for DemoRow {
    type Element = Div;
    fn into_element(self) -> Div {
        self.row
    }
}

/// A captioned demo: the board caption over free-form content.
pub fn demo(
    id: impl Into<ElementId>,
    caption: impl Into<SharedString>,
    content: impl IntoElement,
) -> impl IntoElement {
    div()
        .id(id)
        .w_full()
        .flex()
        .flex_col()
        .gap(rem(12.))
        .child(
            accessible_text("demo-caption", caption)
                .role(Role::Heading)
                .map(|text| type_role(text, TypeRole::LabelLarge))
                .text_color(color(ColorRole::ON_SURFACE)),
        )
        .child(fit_width("demo-fit", content))
}

#[cfg(test)]
mod tests {
    use super::{Band, wrap_columns};
    use core::prelude::v1::test;

    #[test]
    fn oversized_groups_scale_to_fit() {
        assert_eq!(super::fit_scale(100., 200.), 1.);
        assert_eq!(super::fit_scale(200.4, 200.), 1.);
        let scale = super::fit_scale(400., 200.);
        assert!(scale < 0.5 && 400. * scale <= 200.);
        assert_eq!(super::fit_scale(10_000., 200.), super::MIN_FIT_SCALE);
    }

    use super::{FitStep, Regrow};

    /// Runs frames of content whose width at a scale is `width(scale)` in a
    /// row `available` wide until a frame asks for no other, and returns
    /// where it settled with the number of frames it asked for.
    fn settle(mut step: FitStep, width: impl Fn(f32) -> f32, available: f32) -> (FitStep, usize) {
        for asked in 0..20 {
            let (next, redraw) = step.next(width(step.scale), available, 0.1);
            step = next;
            if !redraw {
                return (step, asked);
            }
        }
        panic!("the fit never settled: {step:?}");
    }

    /// Picking a smaller specimen after a wide one draws it at its size again,
    /// rather than at the scale the wide one needed.
    #[test]
    fn a_fit_grows_back_when_its_content_narrows() {
        let (wide, _) = settle(FitStep::FULL, |scale| 900. * scale, 300.);
        assert!(wide.scale < 0.34, "{wide:?}");
        assert!(900. * wide.scale <= 300.);
        let (narrow, asked) = settle(wide, |scale| 200. * scale, 300.);
        assert_eq!(narrow.scale, 1., "{narrow:?}");
        assert_eq!(asked, 1, "one frame to draw it at the full size");
        // Something that still needs shrinking regrows to its own fit.
        let (middle, _) = settle(wide, |scale| 450. * scale, 300.);
        assert!(middle.scale > wide.scale && 450. * middle.scale <= 300.);
    }

    /// Content still too wide at the least scale settles: later renders of
    /// the same content ask for no frame.
    #[test]
    fn content_clipped_at_the_least_scale_stays_settled() {
        let (floor, _) = settle(FitStep::FULL, |scale| 100_000. * scale, 300.);
        assert_eq!(floor.scale, 0.1);
        for _ in 0..3 {
            assert_eq!(
                floor.next(100_000. * floor.scale, 300., 0.1),
                (floor, false)
            );
        }
    }

    /// Content wider at the full size than its width at a smaller scale
    /// suggests tries the full size once, and does not loop.
    #[test]
    fn a_wrong_estimate_regrows_once() {
        // Wide at the full size, but narrow below it.
        let width = |scale: f32| if scale >= 1. { 400. } else { 250. * scale };
        let (settled, _) = settle(FitStep::FULL, width, 300.);
        assert!(settled.scale < 1.);
        assert!(matches!(settled.regrow, Regrow::Blocked(_)), "{settled:?}");
        for _ in 0..3 {
            assert_eq!(
                settled.next(width(settled.scale), 300., 0.1),
                (settled, false)
            );
        }
        // A real change of the content lifts the block.
        let (narrow, _) = settle(settled, |scale| 100. * scale, 300.);
        assert_eq!(narrow.scale, 1.);
    }

    #[test]
    #[allow(clippy::single_range_in_vec_init)] // A run really is a one-element array here.
    fn columns_wrap_into_runs_that_fit() {
        // Everything fits: one run.
        assert_eq!(wrap_columns(&[50., 50., 50.], 10., 10., 200.), [0..3]);
        // 10 + 50 + 10 + 50 = 120; the third column needs 180 > 150.
        assert_eq!(wrap_columns(&[50., 50., 50.], 10., 10., 150.), [0..2, 2..3]);
        // Each run repeats the lead.
        assert_eq!(wrap_columns(&[40.; 5], 20., 10., 110.), [0..2, 2..4, 4..5]);
        // A column wider than the room still gets a run of its own.
        assert_eq!(
            wrap_columns(&[30., 500., 30.], 0., 10., 100.),
            [0..1, 1..2, 2..3]
        );
        // No columns: one empty run.
        assert_eq!(wrap_columns(&[], 0., 10., 100.), [0..0]);
    }
    #[test]
    fn bands_continue_one_outline() {
        assert_eq!(Band::of(0, 1), Band::Whole);
        assert_eq!(
            (0..4).map(|i| Band::of(i, 4)).collect::<Vec<_>>(),
            [Band::Top, Band::Middle, Band::Middle, Band::Bottom]
        );
        assert_eq!(Band::Middle.row_gap(), 0.);
        assert_eq!(Band::Bottom.row_gap(), 0.);
        assert!(Band::Top.row_gap() > 0.);
    }

    use super::board;
    use gpui::{prelude::*, *};

    /// A board of measured (not fixed) columns in a narrow window.
    struct Narrow;
    impl Render for Narrow {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(420.)).child(
                board("narrow-board")
                    .columns(5)
                    .column_labels(["A", "B", "C", "D", "E"])
                    .row_labels(["Row"])
                    .cells((0..5).map(|index| {
                        div()
                            .debug_selector(move || format!("cell-{index}"))
                            .w(px(120.))
                            .h(px(20.))
                    })),
            )
        }
    }

    #[gpui::test]
    fn boards_wrap_columns_instead_of_overflowing(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let window = cx.open_window(size(px(800.), px(600.)), |_, _| Narrow);
        let mut cx = gpui::VisualTestContext::from_window(window.into(), cx);
        // The first frame measures; the board regroups on the next.
        for _ in 0..3 {
            cx.update_window(window.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
            cx.run_until_parked();
        }
        let cells: Vec<_> = (0..5)
            .map(|index| {
                cx.debug_bounds(Box::leak(format!("cell-{index}").into_boxed_str()))
                    .unwrap()
            })
            .collect();
        for cell in &cells {
            assert!(
                cell.right() <= px(420.),
                "a column overflows the board: {cell:?}"
            );
        }
        assert!(
            cells[4].top() > cells[0].bottom(),
            "the last column wrapped below the first"
        );
    }
}
