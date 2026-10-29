//! Design-kit page presentation: the page header, specimen boards and their
//! captions, modelled on the component pages of Google's Figma "Material 3
//! Design Kit".
//!
//! Every page is one rounded light panel (the kit's section background)
//! split across virtual rows. A row's content comes from its page module;
//! the shell supplies the panel slice around it (see `super::page`).
//!
//! Boards are dashed specimen frames. A board taller than a comfortable row
//! is split into bands, one per virtual row: [`Band::Top`], any number of
//! [`Band::Middle`] and a [`Band::Bottom`] draw one continuous outline.
use gpui::{prelude::*, *};
use gpui_kit_material::icons::{Icon, Symbol};
use gpui_kit_material::prelude::*;
use std::sync::OnceLock;

/// The widest a page panel grows.
pub const PAGE_MAX_WIDTH: f32 = 1240.;
/// The kit header's padding inside the panel, and the panel's side padding.
pub const PANEL_PADDING: f32 = 52.;
pub const PANEL_PADDING_COMPACT: f32 = 16.;
/// The kit's section corner radius.
pub const PANEL_RADIUS: f32 = 28.;
/// Space around each page panel.
pub const PAGE_GAP: f32 = 24.;
pub const PAGE_GAP_COMPACT: f32 = 8.;
/// The default space above a body row (see `GalleryPage::row_gap`).
pub const ROW_GAP: f32 = 24.;
/// The space between the header's divider and the first specimen.
pub const HEADER_TO_BODY: f32 = 48.;

/// The page background behind the panels, and the panel (the kit's
/// "Section background").
pub fn page_background() -> Rgba {
    color(ColorRole::SURFACE_DIM)
}
pub fn panel_background() -> Rgba {
    color(ColorRole::SURFACE_CONTAINER_LOW)
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

/// The Material logo mark of the kit's headers: a circle around a square
/// holding a "V", stroked in on-surface. Drawn, not an image asset.
pub fn logo_mark() -> impl IntoElement {
    let ink = color(ColorRole::ON_SURFACE);
    div()
        .relative()
        .flex_shrink_0()
        .size(rem(40.))
        .rounded_full()
        .border_1()
        .border_color(ink)
        .child(
            div()
                .absolute()
                .left(rem(5.))
                .top(rem(5.))
                .size(rem(28.))
                .border_1()
                .border_color(ink),
        )
        .child(
            canvas(
                |_, _, _| {},
                move |bounds, _, window, _| {
                    let inset = bounds.size.width * (6. / 40.);
                    let side = bounds.size.width - inset * 2.;
                    let origin = bounds.origin + point(inset, inset);
                    let mut path = PathBuilder::stroke(px(1.));
                    path.move_to(origin);
                    path.line_to(origin + point(side / 2., side));
                    path.line_to(origin + point(side, px(0.)));
                    if let Ok(path) = path.build() {
                        window.paint_path(path, ink);
                    }
                },
            )
            .absolute()
            .inset_0(),
        )
}

/// The decorative primary-container shape behind a header's top end.
fn header_shape() -> impl IntoElement {
    div()
        .absolute()
        .top(rem(-120.))
        .right(rem(-160.))
        .w(rem(560.))
        .h(rem(520.))
        .rounded_full()
        .bg(color(ColorRole::PRIMARY_CONTAINER).opacity(0.3))
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

/// The kit's page header: logo mark, display title, guideline link,
/// description and a full-width divider. The shell renders it as each
/// page's first row.
pub fn page_header(info: &PageInfo) -> impl IntoElement {
    div()
        .relative()
        .w_full()
        .pt(rem(PANEL_PADDING))
        .child(header_shape())
        .child(
            div()
                .relative()
                .flex()
                .flex_col()
                .gap(rem(60.))
                .child(logo_mark())
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_start()
                        .gap(rem(24.))
                        .child(
                            accessible_text(named("page-title", info.id), info.title)
                                .role(Role::Heading)
                                .font_family("Roboto")
                                .text_size(rem(80.))
                                .line_height(rem(94.))
                                .font_weight(FontWeight::NORMAL)
                                .text_color(color(ColorRole::ON_SURFACE)),
                        )
                        .when_some(info.guideline, |column, url| {
                            column.child(guideline_link(info.id, url, TypeRole::HeadlineSmall))
                        })
                        .child(
                            accessible_text(named("page-description", info.id), info.description)
                                .max_w(rem(720.))
                                .whitespace_normal()
                                .font_family("Roboto")
                                .text_size(rem(18.))
                                .line_height(rem(24.))
                                .text_color(color(ColorRole::ON_SURFACE)),
                        ),
                )
                .child(MaterialDivider::new(named("page-divider", info.id))),
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
const SIDE_GUTTER: f32 = 72.;

/// A dashed specimen board, or one band of a board split across rows.
///
/// Cells fill rows of `columns` in a grid whose columns are at least as
/// wide as their widest specimen and share spare width equally (or are
/// `column_width` design px when set). Specimens never overlap or wrap; a
/// band wider than its row scrolls horizontally inside the row. `caption` appears centered above the top band;
/// `column_labels` title the columns inside the top band; `row_labels` title
/// each cell row at its leading edge (e.g. interaction states); `side_label`
/// sits outside the band's leading edge, level with its first cell row (the
/// kit's "Round"/"Square"). Give every band of one board the same columns,
/// widths and labels layout, and if any band has a side label, give each a
/// side label (possibly empty).
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

impl IntoElement for Board {
    type Element = AnyElement;
    fn into_element(self) -> AnyElement {
        let band = self.band;
        let radius = rem(BOARD_RADIUS);
        let width = self.column_width;
        let labelled_rows = !self.row_labels.is_empty();
        let columns = self.columns + usize::from(labelled_rows);
        // One grid per band: every column is at least as wide as its widest
        // specimen (min-content) and shares any spare width equally, so
        // specimens never overlap or wrap; a band wider than the page
        // scrolls horizontally instead.
        let cell = move |content: AnyElement| {
            div()
                .flex()
                .items_center()
                .justify_center()
                .when_some(width, |cell, width| cell.w(rem(width)))
                .child(content)
        };
        let row_label = |label: Option<AnyElement>| {
            div()
                .w(rem(ROW_LABEL_WIDTH))
                .flex()
                .items_center()
                .children(label)
                .into_any_element()
        };
        let mut items: Vec<AnyElement> = Vec::new();
        if band.has_top() && !self.column_labels.is_empty() {
            if labelled_rows {
                items.push(row_label(None));
            }
            items.extend(
                self.column_labels
                    .into_iter()
                    .enumerate()
                    .map(|(index, label)| {
                        cell(
                            small_label(("column-label", index), label)
                                .whitespace_normal()
                                .text_center()
                                .into_any_element(),
                        )
                        .into_any_element()
                    }),
            );
        }
        let mut row_labels = self.row_labels.into_iter();
        let mut cells = self.cells.into_iter().peekable();
        let mut index: usize = 0;
        while cells.peek().is_some() {
            if labelled_rows {
                let label = row_labels
                    .next()
                    .map(|label| small_label(("row-label", index), label).into_any_element());
                items.push(row_label(label));
            }
            let mut taken = 0;
            for content in cells.by_ref().take(self.columns) {
                items.push(cell(content).into_any_element());
                taken += 1;
            }
            // Keep the next row's label in the first column.
            items.extend((taken..self.columns).map(|_| div().into_any_element()));
            index += 1;
        }
        let grid = div()
            .w_full()
            .grid()
            .grid_cols_min_content(columns as u16)
            .items_center()
            .gap_x(rem(CELL_GAP))
            .gap_y(rem(CELL_GAP))
            .children(items);
        let frame = div()
            .flex_shrink_0()
            .when(width.is_none(), |frame| frame.min_w(relative(1.)))
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
            .child(grid);
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
                                .child(small_label("side-label", label)),
                        )
                    })
                    .child(
                        div()
                            .id("board-scroll")
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_row()
                            .overflow_x_scroll()
                            .restrict_scroll_to_axis()
                            .child(frame),
                    ),
            )
            .into_any_element()
    }
}

/// The content width inside a page panel, in design px, for pages that lay
/// out a window-dependent number of columns.
pub fn content_width(window: &Window) -> f32 {
    let compact = is_compact(window);
    let available = design_width(window) - if compact { 0. } else { 176. };
    let (gap, padding) = if compact {
        (PAGE_GAP_COMPACT, PANEL_PADDING_COMPACT)
    } else {
        (PAGE_GAP, PANEL_PADDING)
    };
    available.min(PAGE_MAX_WIDTH) - 2. * (gap + padding)
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
    use super::Band;
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
}
