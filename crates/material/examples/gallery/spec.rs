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
    /// Measured boards whose frame was drawn wider than their row.
    pub static OVERFLOWS: std::cell::RefCell<Vec<String>> = const {
        std::cell::RefCell::new(Vec::new())
    };
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

/// The scale that fits `needed` px into `available` px: 1 when it fits,
/// else a little under the ratio (borders do not scale), never below 0.4.
pub fn fit_scale(needed: f32, available: f32) -> f32 {
    if needed <= available + 0.5 || needed <= 0. {
        1.
    } else {
        (available / needed * 0.97).clamp(0.4, 1.)
    }
}

/// Lays out, prepaints and paints its child at another rem size: the
/// child's rem-based geometry and type scale with it.
struct RemScaled {
    rem: Pixels,
    child: AnyElement,
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
        let columns = self.columns;
        let rem_size = window.rem_size();
        let to_px = |design: f32| f32::from(rem(design).to_pixels(rem_size));
        let gap = to_px(CELL_GAP);
        // The row-label column: at least `ROW_LABEL_WIDTH`, wider for a
        // longer label (labels never break inside a word).
        let min_label = to_px(ROW_LABEL_WIDTH);
        let lead_of = move |label: f32| {
            if labelled_rows {
                label.max(min_label) + gap
            } else {
                0.
            }
        };
        // The frame's padding and dashed border around the grid.
        let chrome = 2. * to_px(BOARD_PADDING) + 2.;
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
                    fit_scale(needed, measure.available - chrome)
                }
                None => 1.,
            })
            .collect();
        let wrapped = groups.len() > 1;
        // Until the board is measured, keep its first frame inside its row.
        let unmeasured = measured.is_none();

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
            let grid_columns = span + usize::from(labelled_rows);
            let mut items: Vec<AnyElement> = Vec::new();
            // Which board column each grid item measures, if any.
            let mut item_columns: Vec<Option<usize>> = Vec::new();
            if show_column_labels {
                if labelled_rows {
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
                if labelled_rows {
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
            let widths = frame_widths.clone();
            let scale = scales[group_index];
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
                .on_children_prepainted(move |bounds, _, _| {
                    let mut widths = widths.borrow_mut();
                    for (bounds, column) in bounds.iter().zip(&item_columns) {
                        if let Some(column) = column {
                            // Record unscaled widths, so scaling does not
                            // feed back into the grouping.
                            let width = f32::from(bounds.size.width) / scale;
                            widths[*column] = widths[*column].max(width);
                        }
                    }
                })
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
            .px(rem(BOARD_PADDING))
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
        let has_side_label = self.side_label.is_some();
        // The frame's drawn width, recorded by tests to catch overflow.
        let drawn_width = std::rc::Rc::new(std::cell::Cell::new(0f32));
        let rendered = groups;
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
                    .when_some(self.side_label, |row, label| {
                        row.child(
                            div()
                                .w(rem(SIDE_GUTTER))
                                .flex_shrink_0()
                                .pt(rem(if band.has_top() {
                                    BOARD_PADDING
                                } else {
                                    CELL_GAP / 2.
                                }))
                                .pr(rem(8.))
                                .child(small_label("side-label", label).whitespace_normal()),
                        )
                    })
                    .child(
                        div()
                            .flex_1()
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
                    )
                    // Runs after the grids' listeners: record this frame's
                    // measurements and regroup on the next frame if they
                    // change the groups.
                    .on_children_prepainted(move |bounds, _, cx| {
                        let holder = bounds.get(usize::from(has_side_label));
                        let Some(holder) = holder else { return };
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
                        let changed = regrouped != rendered;
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
                    }),
            )
    }
}

/// The content width inside a spec card, in design px, for pages that lay
/// out a window-dependent number of columns.
pub fn content_width(window: &Window) -> f32 {
    super::shell::spec_content_width(window)
}

/// A free-form specimen row: wrapped items with the board cell gap, for
/// live demos that are not a state matrix.
pub fn demo_row() -> Div {
    div()
        .w_full()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(rem(CELL_GAP))
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
        .child(content)
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
        assert_eq!(super::fit_scale(10_000., 200.), 0.4);
    }

    #[test]
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
