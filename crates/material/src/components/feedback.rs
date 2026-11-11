//! Status and notice surfaces: M3 badges, status chips, banners and
//! cards (the snackbar lives in `snackbar.rs`).
use crate::color::ColorRole::{self, *};
use crate::icons::IconName;
use crate::theme::color;
use crate::tokens::{elevation, layout::*, shape::*, state, typography::*};

use super::InteractionPreview;
use gpui::{prelude::*, *};
use std::rc::Rc;

use super::text::accessible_text;

/// The tone of a [`notice_banner`].
#[non_exhaustive]
pub enum NoticeKind {
    Info,
    /// Something needs attention, but nothing has failed.
    Warning,
    Danger,
}

/// A notice's container, ink, icon and role. A warning reads like a status: it
/// does not interrupt a screen reader the way a failure's alert does.
fn notice_tone(kind: NoticeKind) -> (ColorRole, ColorRole, IconName, Role) {
    match kind {
        NoticeKind::Info => (
            ColorRole::SECONDARY_CONTAINER,
            ColorRole::PRIMARY,
            IconName::Info,
            Role::Status,
        ),
        NoticeKind::Warning => (
            ColorRole::WARNING_BG,
            ColorRole::WARNING,
            IconName::Warning,
            Role::Status,
        ),
        NoticeKind::Danger => (
            ColorRole::ERROR_CONTAINER,
            ColorRole::ERROR,
            IconName::Error,
            Role::Alert,
        ),
    }
}

/// A page-level notice: tinted surface, status icon and wrapping text.
/// Callers append the notice's actions, which wrap below the text when narrow.
///
/// The icon centres on the text's first line, at whatever size the caller sets the
/// text, so a notice of several lines reads from its icon; icon and text together
/// centre on the actions. Debug selectors `{id}-icon` and `{id}-text` name the two.
pub fn notice_banner(
    id: impl Into<ElementId>,
    kind: NoticeKind,
    text: impl Into<SharedString>,
) -> Div {
    let id = id.into();
    let (bg, tone, icon, role) = notice_tone(kind);
    div()
        .flex()
        .flex_wrap()
        .w_full()
        .min_w_0()
        .items_center()
        .gap(rem(12.))
        .p(rem(12.))
        .rounded(RADIUS_SMALL)
        .bg(color(bg))
        .child(
            div()
                .flex()
                .items_start()
                .flex_1()
                .flex_basis(rem(200.) + ICON_SLOT + rem(12.))
                .min_w_0()
                .gap(rem(12.))
                .child(first_line_icon(&id, icon, tone))
                .child(
                    accessible_text(id.clone(), text)
                        .debug_selector({
                            let id = id.clone();
                            move || format!("{id}-text")
                        })
                        .role(role)
                        .flex_1()
                        .min_w_0()
                        .whitespace_normal(),
                ),
        )
}

/// The notice's icon in a slot exactly one line of the inherited text tall: a hidden
/// strut of no width holds one letter, whose line box sets the height whatever the
/// text system, and the icon centres on it, overflowing a line shorter than itself.
fn first_line_icon(id: &ElementId, icon: IconName, tone: ColorRole) -> Div {
    div()
        .relative()
        .flex_shrink_0()
        .w(ICON_SLOT)
        .child(
            div()
                .w_0()
                .overflow_hidden()
                .invisible()
                .whitespace_nowrap()
                .child("M"),
        )
        .child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .flex_shrink_0()
                        .size(ICON_SLOT)
                        .debug_selector({
                            let id = id.clone();
                            move || format!("{id}-icon")
                        })
                        .child(icon.icon().size(ICON_SLOT).color(color(tone))),
                ),
        )
}

/// The tone of a [`status_chip`]; text always pairs with its tinted
/// background.
pub enum StatusKind {
    Success,
    Warning,
    Danger,
    Progress,
    Neutral,
    /// An identity rather than a status, e.g. "Default".
    Tonal,
}

/// Former name of [`StatusKind`]: these are status labels, not M3 badges.
#[deprecated(note = "status labels are not M3 badges: use `StatusKind` with `status_chip`")]
pub type BadgeKind = StatusKind;

/// A status label chip: a tinted pill with an optional status icon. Icon,
/// padding and label belong to one container; callers append the label.
/// For the M3 notification badge, see [`small_badge`] and [`large_badge`].
pub fn status_chip(kind: StatusKind) -> Div {
    let (text, bg, icon) = match kind {
        StatusKind::Success => (SUCCESS, SUCCESS_BG, Some(IconName::CheckCircle)),
        StatusKind::Warning => (WARNING, WARNING_BG, Some(IconName::Warning)),
        StatusKind::Danger => (ERROR, ERROR_CONTAINER, Some(IconName::Error)),
        // A clock would read as waiting; work in progress needs no icon.
        StatusKind::Progress => (PRIMARY, SECONDARY_CONTAINER, None),
        // Neutral is a plain text chip without a status icon, on the
        // highest surface container: a step above the tile in light and dark
        // palettes alike (INSET sinks below the surface in dark ones).
        StatusKind::Neutral => (ON_SURFACE_VARIANT, SURFACE_CONTAINER_HIGHEST, None),
        StatusKind::Tonal => (ON_SECONDARY_CONTAINER, SECONDARY_CONTAINER, None),
    };
    let mut view = div()
        .flex()
        .items_center()
        .flex_shrink_0()
        .gap(rem(4.))
        .min_h(rem(24.))
        .px(rem(10.))
        .py(rem(3.))
        .rounded(RADIUS_SMALL)
        .bg(color(bg))
        .text_color(color(text))
        .text_size(TEXT_AUX)
        .font_weight(FontWeight::MEDIUM)
        .whitespace_nowrap();
    if let Some(icon) = icon {
        view = view.child(icon.icon().size(STATUS_ICON));
    }
    view
}

/// The status icon shares the chip's text ruler.
const STATUS_ICON: Rems = rem(16.);

/// The paint of an outlined status chip: text, container, edge, and the icon with its color.
fn outlined_chip_paint(
    kind: StatusKind,
) -> (
    ColorRole,
    Option<ColorRole>,
    ColorRole,
    Option<(IconName, ColorRole)>,
) {
    match kind {
        StatusKind::Success => (
            ON_SURFACE,
            None,
            OUTLINE_VARIANT,
            Some((IconName::Check, SUCCESS)),
        ),
        StatusKind::Warning => (
            WARNING,
            Some(WARNING_BG),
            WARNING,
            Some((IconName::Warning, WARNING)),
        ),
        StatusKind::Danger => (
            ERROR,
            Some(ERROR_CONTAINER),
            ERROR,
            Some((IconName::Error, ERROR)),
        ),
        StatusKind::Progress | StatusKind::Tonal => (
            ON_SECONDARY_CONTAINER,
            Some(SECONDARY_CONTAINER),
            SECONDARY_CONTAINER,
            None,
        ),
        StatusKind::Neutral => (ON_SURFACE, None, OUTLINE_VARIANT, None),
    }
}

/// One of several facts in a row, such as the checks of a diagnosis or the steps of a task.
/// A row of [`status_chip`]s reads as many results; here the row stays quiet: a fine or
/// neutral fact is outlined, with a colored icon where it is fine. Facts that need attention
/// (warning, danger) and work under way (progress, tonal) are filled. `name` is its
/// accessible name, usually the label with what the color says, as a color alone does not
/// reach assistive technology. Callers append the label, and a progress spinner where one
/// applies. For a plain fact with no status, see [`outlined_status_chip`].
pub fn status_fact_chip(
    id: impl Into<ElementId>,
    name: impl Into<SharedString>,
    kind: StatusKind,
) -> Stateful<Div> {
    fact_chip(kind).id(id).role(Role::Label).aria_label(name)
}

/// The chip of [`status_fact_chip`], without its id or accessible name.
#[deprecated(note = "use `status_fact_chip`, which takes the id and accessible name")]
pub fn status_chip_outlined(kind: StatusKind) -> Div {
    fact_chip(kind)
}

fn fact_chip(kind: StatusKind) -> Div {
    let (text, container, edge, icon) = outlined_chip_paint(kind);
    let mut view = div()
        .flex()
        .items_center()
        .flex_shrink_0()
        .gap(rem(4.))
        .min_h(rem(24.))
        .px(rem(10.))
        .py(rem(3.))
        .rounded(RADIUS_SMALL)
        .border_1()
        .border_color(color(edge))
        .text_color(color(text))
        .text_size(TEXT_AUX)
        .font_weight(FontWeight::MEDIUM)
        .whitespace_nowrap();
    if let Some(container) = container {
        view = view.bg(color(container));
    }
    if let Some((icon, ink)) = icon {
        view = view.child(icon.icon().size(STATUS_ICON).color(color(ink)));
    }
    view
}

/// A status label; same as [`status_chip`], kept under its earlier name.
#[deprecated(
    note = "a status chip, not an M3 badge: use `status_chip` (M3 badges are `small_badge` and `large_badge`)"
)]
pub fn badge(kind: StatusKind) -> Div {
    status_chip(kind)
}

/// A fact beside a label in the status chip's size, outlined rather than
/// filled: e.g. the engine a tile uses. It has no status of its own; for facts that
/// do, see [`status_fact_chip`].
pub fn outlined_status_chip(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
) -> Stateful<Div> {
    accessible_text(id, label)
        .flex_shrink_0()
        .px(rem(10.))
        .py(rem(4.))
        .rounded(RADIUS_SMALL)
        .border_1()
        .border_color(color(OUTLINE_VARIANT))
        .text_size(TEXT_AUX)
        .line_height(rem(16.))
        .font_weight(FontWeight::MEDIUM)
        .text_color(color(ON_SURFACE_VARIANT))
        .whitespace_nowrap()
}

/// An outlined status label; same as [`outlined_status_chip`], kept under its
/// earlier name.
#[deprecated(note = "a status chip, not an M3 badge: use `outlined_status_chip`")]
pub fn outlined_badge(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Stateful<Div> {
    outlined_status_chip(id, label)
}

/// The M3 small badge: a 6dp dot. It carries no text, so it does not follow
/// the text scale.
pub const SMALL_BADGE_SIZE: Pixels = px(6.);
/// The M3 large badge: at least 16dp square, around label-small text.
pub const LARGE_BADGE_SIZE: Rems = rem(16.);
/// Horizontal padding inside a large badge (4dp).
pub const LARGE_BADGE_PADDING: Rems = rem(4.);
/// The container and label roles of both M3 badges.
pub const BADGE_CONTAINER: ColorRole = ERROR;
pub const BADGE_LABEL: ColorRole = ON_ERROR;

/// M3 small badge: a full-round error dot that marks new content on an icon.
/// The caller positions it (6dp from the icon's top-trailing corner).
pub fn small_badge() -> Div {
    div()
        .flex_shrink_0()
        .size(SMALL_BADGE_SIZE)
        .rounded(RADIUS_FULL)
        .bg(color(BADGE_CONTAINER))
}

/// M3 large badge: a count or short label on an error pill, label-small on
/// on-error, at least 16dp square and full-round. The caller positions it
/// (12dp across and 14dp down from the icon's top-trailing corner) and
/// supplies its text, e.g. "3" or "999+".
pub fn large_badge(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Stateful<Div> {
    let scale = TypeRole::LabelSmall.scale();
    accessible_text(id, label)
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .h(LARGE_BADGE_SIZE)
        .min_w(LARGE_BADGE_SIZE)
        .px(LARGE_BADGE_PADDING)
        .rounded(RADIUS_FULL)
        .bg(color(BADGE_CONTAINER))
        .text_color(color(BADGE_LABEL))
        .text_size(scale.font_size)
        .line_height(scale.line_height)
        .font_weight(FontWeight(scale.font_weight as f32))
        .whitespace_nowrap()
}

/// The container and label roles of a [`media_label`]: the inverse surface, as a plain
/// tooltip and a slider's value label use, reads over any picture in every scheme.
pub const MEDIA_LABEL_CONTAINER: ColorRole = INVERSE_SURFACE;
pub const MEDIA_LABEL_TEXT: ColorRole = INVERSE_ON_SURFACE;

/// A short fact laid over a picture, such as a video's duration on its cover or a
/// frame's time on a screenshot: label-medium on the inverse surface, small corners,
/// one line. The caller positions it, usually 8dp in from a bottom corner.
pub fn media_label(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Stateful<Div> {
    let scale = TypeRole::LabelMedium.scale();
    accessible_text(id, label)
        .flex()
        .flex_shrink_0()
        .items_center()
        .px(rem(8.))
        .py(rem(2.))
        .rounded(RADIUS_SMALL)
        .bg(color(MEDIA_LABEL_CONTAINER))
        .text_color(color(MEDIA_LABEL_TEXT))
        .text_size(scale.font_size)
        .line_height(scale.line_height)
        .font_weight(FontWeight(scale.font_weight as f32))
        .whitespace_nowrap()
}

/// The three M3 card variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CardVariant {
    /// Surface-container-low with level-1 elevation and no outline.
    Elevated,
    /// Surface-container-highest, flat.
    Filled,
    /// Surface with a 1dp outline-variant edge, flat.
    Outlined,
    /// Surface, flat, no edge: for a page whose ground is a step below its surface,
    /// where the tone separates the card by itself.
    Surface,
}

/// The resolved paint of a card variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CardStyle {
    container: ColorRole,
    elevation: u8,
    outline: Option<ColorRole>,
}

fn card_style(variant: CardVariant) -> CardStyle {
    match variant {
        CardVariant::Elevated => CardStyle {
            container: SURFACE_CONTAINER_LOW,
            elevation: 1,
            outline: None,
        },
        CardVariant::Filled => CardStyle {
            container: SURFACE_CONTAINER_HIGHEST,
            elevation: 0,
            outline: None,
        },
        CardVariant::Outlined => CardStyle {
            container: SURFACE,
            elevation: 0,
            outline: Some(OUTLINE_VARIANT),
        },
        CardVariant::Surface => CardStyle {
            container: SURFACE,
            elevation: 0,
            outline: None,
        },
    }
}

// A selected card. M3 cards can be checked, but neither the Figma kit,
// md3.io nor m3e.kyralabs.dev gives the checked state's values; these come
// from Material Components for Android's M3 card (`Widget.Material3.CardView`
// and its `res/color` selectors), the one reference that does.

/// The secondary layer over a selected card's container: MDC Android's
/// `m3_card_foreground_color.xml` lays `colorSecondary` at 16% over a checked
/// card of every variant.
pub const CARD_SELECTED_LAYER: f32 = 0.16;

/// The selected card's check icon: MDC Android's `ic_mtrl_checked_circle`,
/// 24dp, 8dp from the top-end corner (`checkedIconSize`, `checkedIconMargin`).
const CARD_CHECK_SIZE: f32 = 24.;
const CARD_CHECK_MARGIN: f32 = 8.;

/// The edge of a card: `selected` turns an outlined card's edge to the
/// secondary role (MDC Android's `m3_card_stroke_color.xml`, `state_checked`),
/// at its unchanged 1dp width; the other variants have no edge to turn.
fn card_outline(variant: CardVariant, selected: bool) -> Option<ColorRole> {
    card_style(variant)
        .outline
        .map(|outline| if selected { SECONDARY } else { outline })
}

/// The color of a card's state layer in `state`. A selected card's hover,
/// focus and drag layers are secondary, and its press on-surface-variant,
/// since a press returns it to unselected (MDC Android's
/// `m3_card_ripple_color.xml`). An unselected card keeps the Figma kit's
/// on-surface card states even when it can be selected, where MDC Android
/// presses a checkable card in secondary: the kit's card states come from
/// the Figma kit, which outranks MDC, and only the checked state, which the
/// Figma kit lacks, comes from MDC.
fn card_layer_content(selected: bool, state: CardState) -> ColorRole {
    match (selected, state) {
        (true, CardState::Pressed) => ON_SURFACE_VARIANT,
        (true, _) => SECONDARY,
        (false, _) => ON_SURFACE,
    }
}

/// A card's container under its selection layer, when selected.
fn card_container(variant: CardVariant, selected: bool) -> Rgba {
    let container = color(card_style(variant).container);
    if selected {
        state::layer(container, color(SECONDARY), CARD_SELECTED_LAYER)
    } else {
        container
    }
}

/// The filled check-circle a selected card shows at its top-end corner, in
/// the secondary role (MDC Android tints it with the checked stroke color).
fn card_check() -> Div {
    static PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let path = PATH.get_or_init(|| {
        IconName::CheckCircle
            .symbol()
            .icon()
            .filled(true)
            .asset_path()
    });
    div()
        .absolute()
        .top(rem(CARD_CHECK_MARGIN))
        .right(rem(CARD_CHECK_MARGIN))
        .size(rem(CARD_CHECK_SIZE))
        .when(cfg!(test), |check| {
            check.debug_selector(|| "material-card-check".into())
        })
        .child(
            crate::icons::Icon::new(SharedString::new_static(path.as_str()))
                .size(rem(CARD_CHECK_SIZE))
                .color(color(SECONDARY)),
        )
}

/// Compatibility helper padding from the reference's 16px content examples.
/// The reference card surface itself has no padding.
pub const CARD_PADDING: Rems = rem(16.);

/// Static Material card surface, without implicit padding or full width.
///
/// Fast's native overflow handling clips children to the rounded shape. The
/// upstream backend currently provides rectangular overflow clipping only;
/// rounded content clipping there remains unsupported. [`Self::state`] and
/// [`Self::preview_state`] render the kit's interaction states;
/// [`Self::interactive`] adds the live hover state (its elevation change is
/// immediate, not animated).
///
/// The outline paints above content without participating in layout.
pub struct CardSurface {
    inner: Div,
    outline: Option<ColorRole>,
    variant: CardVariant,
    selected: bool,
    /// A specimen's fixed state.
    state: Option<CardState>,
    /// Whether hovering lays a state layer over it.
    hover: bool,
    /// Whether the check icon has joined the children.
    checked: bool,
    /// A specimen's forced focus: the 3dp secondary ring 2dp outside.
    focus_ring: bool,
}
impl CardSurface {
    pub fn new(variant: CardVariant) -> Self {
        let style = card_style(variant);
        Self {
            inner: gpui_base::v_flex()
                .bg(color(style.container))
                .text_color(color(ON_SURFACE))
                .overflow_hidden()
                .rounded(RADIUS_CARD)
                .shadow(elevation::level(style.elevation)),
            outline: style.outline,
            variant,
            selected: false,
            state: None,
            hover: false,
            checked: false,
            focus_ring: false,
        }
    }

    /// Show the card selected (M3's checked card, after MDC Android): a 16%
    /// secondary layer over its container, a secondary edge on an outlined
    /// card and a secondary check-circle icon at its top-end corner. It
    /// combines with [`Self::state`] and [`Self::interactive`] in any order.
    /// Each of the three repaints the card's container, so a background the
    /// caller set before them (`.bg(..)`) is replaced; set one after.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self.outline = card_outline(self.variant, selected);
        self.restyle()
    }

    /// Render a fixed interaction state as a specimen: the state layer, the
    /// state's elevation and, when focused, the focus ring.
    pub fn state(mut self, state: CardState) -> Self {
        self.state = Some(state);
        self.focus_ring = state == CardState::Focused;
        self.restyle()
    }

    /// [`Self::state`] for one of the shared interaction previews.
    pub fn preview_state(self, preview: InteractionPreview) -> Self {
        self.state(preview.into())
    }

    /// A card the pointer can act on: hovering lays the 8% on-surface
    /// state layer over it and raises it to its hovered elevation. The
    /// caller supplies the action (and focus) on the content.
    pub fn interactive(mut self) -> Self {
        self.hover = true;
        self.restyle()
    }

    /// Paint the container, elevation and hover layer from the selection,
    /// fixed state and hover together, so their builders compose in any
    /// order.
    fn restyle(mut self) -> Self {
        let container = card_container(self.variant, self.selected);
        let state = self.state.unwrap_or_default();
        self.inner = self
            .inner
            .bg(state::layer(
                container,
                color(card_layer_content(self.selected, state)),
                state.layer_opacity(),
            ))
            .shadow(elevation::level(card_elevation(self.variant, state)));
        if self.hover {
            let hover = state::hovered(
                container,
                color(card_layer_content(self.selected, CardState::Hovered)),
            );
            let shadow = elevation::level(card_elevation(self.variant, CardState::Hovered));
            self.inner = self
                .inner
                .hover(move |style| style.bg(hover).shadow(shadow.clone()));
        }
        self
    }
}

/// The interaction states of a card, including the drag the shared
/// [`InteractionPreview`] does not cover.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CardState {
    #[default]
    Enabled,
    Hovered,
    Focused,
    Pressed,
    Dragged,
}

impl CardState {
    /// Every state in the kit's order.
    pub const ALL: [Self; 5] = [
        Self::Enabled,
        Self::Hovered,
        Self::Focused,
        Self::Pressed,
        Self::Dragged,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Hovered => "Hovered",
            Self::Focused => "Focused",
            Self::Pressed => "Pressed",
            Self::Dragged => "Dragged",
        }
    }

    /// The on-surface state layer's opacity.
    pub const fn layer_opacity(self) -> f32 {
        match self {
            Self::Enabled => 0.,
            Self::Hovered => state::HOVER,
            Self::Focused => state::FOCUS,
            Self::Pressed => state::PRESS,
            Self::Dragged => state::DRAG,
        }
    }
}

impl From<InteractionPreview> for CardState {
    fn from(preview: InteractionPreview) -> Self {
        match preview {
            InteractionPreview::Hovered => Self::Hovered,
            InteractionPreview::Focused => Self::Focused,
            InteractionPreview::Pressed => Self::Pressed,
            InteractionPreview::Enabled | InteractionPreview::Disabled => Self::Enabled,
        }
    }
}

/// The kit's elevation for each card variant and state: hovering raises a
/// card one level, dragging to level 4 (elevated) or 3; a filled card stays
/// at level 1 while focused or pressed.
fn card_elevation(variant: CardVariant, state: CardState) -> u8 {
    use CardState::*;
    match (variant, state) {
        (CardVariant::Elevated, Hovered) => 2,
        (CardVariant::Elevated, Dragged) => 4,
        (CardVariant::Elevated, _) => 1,
        (_, Dragged) => 3,
        (_, Hovered) => 1,
        (CardVariant::Filled, Focused | Pressed) => 1,
        _ => 0,
    }
}
impl Styled for CardSurface {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}
impl ParentElement for CardSurface {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}
impl InteractiveElement for CardSurface {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}
impl IntoElement for CardSurface {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for CardSurface {
    type RequestLayoutState = <Div as Element>::RequestLayoutState;
    type PrepaintState = <Div as Element>::PrepaintState;
    fn id(&self) -> Option<ElementId> {
        Element::id(&self.inner)
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        Element::source_location(&self.inner)
    }
    fn a11y_role(&self) -> Option<accesskit::Role> {
        self.inner.a11y_role()
    }
    fn write_a11y_info(&self, node: &mut accesskit::Node) {
        self.inner.write_a11y_info(node);
    }
    fn a11y_synthetic_children(
        &mut self,
        prepaint: &mut Self::PrepaintState,
        builder: &mut A11ySubtreeBuilder,
    ) {
        self.inner.a11y_synthetic_children(prepaint, builder);
    }
    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        if self.selected && !self.checked {
            // Last, so it draws over the content.
            self.checked = true;
            self.inner.extend([card_check().into_any_element()]);
        }
        self.inner.request_layout(id, inspector, window, cx)
    }
    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        state: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.inner
            .prepaint(id, inspector, bounds, state, window, cx)
    }
    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        state: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let style = self
            .inner
            .interactivity()
            .compute_style(id, prepaint.as_ref(), window, cx);
        self.inner
            .paint(id, inspector, bounds, state, prepaint, window, cx);
        if let Some(role) = self.outline
            && style.visibility != Visibility::Hidden
            && style.display != Display::None
        {
            let radii = style
                .corner_radii
                .to_pixels(window.rem_size())
                .clamp_radii_for_quad_size(bounds.size);
            // Match GPUI's local multiplication; paint_quad applies ancestor
            // opacity. Neither path clamps the local opacity here.
            let mut outline: Hsla = color(role).into();
            outline.a *= style.opacity.unwrap_or(1.);
            window.paint_quad(quad(
                bounds,
                radii,
                transparent_black(),
                px(1.),
                outline,
                BorderStyle::default(),
            ));
        }
        if self.focus_ring {
            // The kit's focus indicator: 3dp secondary, 2dp outside, around
            // the surface's own corners.
            let radii = style
                .corner_radii
                .to_pixels(window.rem_size())
                .clamp_radii_for_quad_size(bounds.size);
            let out = |radius: Pixels| radius + px(5.);
            window.paint_quad(quad(
                bounds.dilate(px(5.)),
                Corners {
                    top_left: out(radii.top_left),
                    top_right: out(radii.top_right),
                    bottom_right: out(radii.bottom_right),
                    bottom_left: out(radii.bottom_left),
                },
                transparent_black(),
                px(3.),
                color(SECONDARY),
                BorderStyle::default(),
            ));
        }
    }
}

/// Why a [`MaterialCard`] is moving: the pointer holding it, or the
/// spring carrying it home after release.
#[derive(Default)]
struct CardDrag {
    /// Pointer position and card offset when the press began, while held.
    grab: Option<(Point<Pixels>, Point<Pixels>)>,
    /// The card's displacement from its place.
    offset: Point<Pixels>,
    /// Whether the current or last press moved the card (it then is not a
    /// click).
    moved: bool,
    /// Bumped on every release, keying the return animation.
    releases: u64,
    /// Where the last release left the card.
    released_at: Point<Pixels>,
}

/// A [`MaterialCard`]'s action.
type CardClick = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// The distance a press must travel before it drags the card.
const CARD_DRAG_SLOP: Pixels = px(4.);

/// A Material card that can act and move: the [`CardSurface`] of its
/// variant, optionally a GPUI Base button (`on_click`: hover and press
/// state layers, the kit's hover elevation, ripple, keyboard activation
/// and the focus ring) and optionally draggable (the dragged state layer
/// and elevation while the pointer carries it; on release it springs back
/// to its place on the default spatial spring, or returns at once under
/// reduced motion). The accessible label names the action. A selectable
/// card ([`Self::selected`]) shows and announces whether it is selected.
///
/// A caller's rounding (`.rounded(..)` and the per-corner forms) is the
/// card's shape: its surface, state layers, ripple and focus ring all follow
/// it. The surface fills the card, even one the caller sizes, and clips its
/// content to that shape, so the card needs no clip of its own; one asked of it (`.overflow_hidden()`) is left
/// to the surface, so it cannot cut the focus ring drawn outside the card.
///
/// # Example
///
/// ```
/// use gpui_kit::{div, prelude::*};
/// use gpui_kit_material::prelude::*;
///
/// let trip = MaterialCard::new("trip", CardVariant::Outlined)
///     .accessibility_label("Trip to Lisbon")
///     .on_click(|_, _window, _cx| println!("Open the trip"))
///     .child(div().p(rem(16.)).child("Trip to Lisbon"));
/// let pinned = MaterialCard::new("note", CardVariant::Filled).draggable(true);
/// # let _ = (trip, pinned);
/// ```
#[derive(gpui::IntoElement)]
pub struct MaterialCard {
    id: ElementId,
    variant: CardVariant,
    /// Whether it is selected, for a card that can be; `None` for one that
    /// cannot.
    selected: Option<bool>,
    selection: CardSelection,
    /// A [`CardGroup`]'s focus handle and whether the card is its tab stop.
    focus: Option<(FocusHandle, bool)>,
    label: Option<SharedString>,
    on_click: Option<CardClick>,
    draggable: bool,
    preview: Option<CardState>,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl MaterialCard {
    pub fn new(id: impl Into<ElementId>, variant: CardVariant) -> Self {
        Self {
            id: id.into(),
            variant,
            selected: None,
            selection: CardSelection::Toggle,
            focus: None,
            label: None,
            on_click: None,
            draggable: false,
            preview: None,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// Make the card one action (Base button semantics).
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// The action's accessible name.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Make the card selectable, and say whether it is selected (M3's
    /// checked card, after MDC Android; see [`CardSurface::selected`]). Its
    /// action, from [`Self::on_click`], changes it; a card without one shows
    /// the state but announces none, having nothing to change it with.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// What selecting the card means to assistive technology:
    /// [`CardSelection::Toggle`] (the default) or one choice of several,
    /// [`CardSelection::Radio`]. A [`CardGroup`] sets the radio meaning and
    /// adds the radio group's roving focus.
    pub fn selection_role(mut self, selection: CardSelection) -> Self {
        self.selection = selection;
        self
    }

    /// Let the pointer pick the card up and move it.
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    /// Render a fixed interaction state as a specimen.
    pub fn state(mut self, state: CardState) -> Self {
        self.preview = Some(state);
        self
    }

    /// Whether this card acts.
    pub fn is_clickable(&self) -> bool {
        self.on_click.is_some()
    }
}

impl Styled for MaterialCard {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for MaterialCard {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for MaterialCard {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use super::interaction::{ButtonFocusRing, ButtonRipple};
        // The caller's rounding is the card's shape: the surface, its state
        // layers, ripple and focus ring take it, not just the outer element.
        let rounding = std::mem::take(&mut self.style.corner_radii);
        // The surface clips its content to that shape. A clip the caller asks
        // of the card goes no further, so it never cuts the focus ring, which
        // is drawn outside the card.
        self.style.overflow = Default::default();
        let card = AbsoluteLength::from(RADIUS_CARD);
        let mut shape = Corners {
            top_left: card,
            top_right: card,
            bottom_right: card,
            bottom_left: card,
        };
        for (corner, asked) in [
            (&mut shape.top_left, rounding.top_left),
            (&mut shape.top_right, rounding.top_right),
            (&mut shape.bottom_right, rounding.bottom_right),
            (&mut shape.bottom_left, rounding.bottom_left),
        ] {
            if let Some(asked) = asked {
                *corner = asked;
            }
        }
        let corners = shape.to_pixels(window.rem_size());
        let id = self.id;
        let drag =
            window.use_keyed_state((id.clone(), "card-drag"), cx, |_, _| CardDrag::default());
        let (grabbed, offset, releases, released_at) = {
            let drag = drag.read(cx);
            (
                drag.grab.is_some(),
                drag.offset,
                drag.releases,
                drag.released_at,
            )
        };
        let dragging = grabbed && drag.read(cx).moved;
        let clickable = self.on_click.is_some() && self.preview.is_none();
        let pressed = clickable && grabbed && !dragging;
        let selected = self.selected.unwrap_or(false);
        let container = card_container(self.variant, selected);
        let hover_ink = color(card_layer_content(selected, CardState::Hovered));
        let press_ink = color(card_layer_content(selected, CardState::Pressed));
        let group: SharedString = format!("material-card-{id:?}").into();

        // The surface fills the card, so a card the caller sizes is the
        // surface's size, and the surface's clip keeps taller content in it.
        let mut surface = CardSurface::new(self.variant)
            .selected(selected)
            .w_full()
            .flex_grow_1()
            .min_h_0()
            .rounded_tl(corners.top_left)
            .rounded_tr(corners.top_right)
            .rounded_br(corners.bottom_right)
            .rounded_bl(corners.bottom_left)
            .children(self.children);
        surface = match (self.preview, dragging) {
            (_, true) => surface.state(CardState::Dragged),
            (Some(state), _) => surface.state(state),
            (None, false) if pressed => surface.state(CardState::Pressed),
            (None, false) if clickable => {
                let hover = state::hovered(container, hover_ink);
                let hovered = elevation::level(card_elevation(self.variant, CardState::Hovered));
                surface.group_hover(group.clone(), move |style| {
                    style.bg(hover).shadow(hovered.clone())
                })
            }
            _ => surface,
        };

        let body: AnyElement = if let Some(on_click) = self.on_click.clone().filter(|_| clickable) {
            let (focus, tab_stop) = self.focus.clone().unwrap_or_else(|| {
                let focus = window
                    .use_keyed_state((id.clone(), "card-focus"), cx, |_, cx| cx.focus_handle())
                    .read(cx)
                    .clone();
                (focus, true)
            });
            let click_drag = drag.clone();
            let button = gpui_base::Button::new((id.clone(), "card-button"))
                .group(group)
                .track_focus(&focus)
                .tab_stop(tab_stop)
                .w_full()
                .flex()
                .flex_col()
                .flex_grow_1()
                .min_h_0()
                .rounded_tl(corners.top_left)
                .rounded_tr(corners.top_right)
                .rounded_br(corners.bottom_right)
                .rounded_bl(corners.bottom_left)
                .cursor_pointer()
                .when_some(self.label.clone(), |button, label| {
                    button.accessibility_label(label)
                })
                .when_some(self.selected, |button, selected| {
                    let toggled = if selected {
                        accesskit::Toggled::True
                    } else {
                        accesskit::Toggled::False
                    };
                    match self.selection {
                        CardSelection::Toggle => button.aria_toggled(toggled),
                        CardSelection::Radio { index, count } => button
                            .role(Role::RadioButton)
                            .aria_toggled(toggled)
                            .aria_selected(selected)
                            .aria_position_in_set(index + 1)
                            .aria_size_of_set(count),
                    }
                })
                .child(surface)
                .on_click(move |event, window, cx| {
                    // A press that carried the card away is a drag, not a click.
                    if !click_drag.read(cx).moved {
                        on_click(event, window, cx);
                    }
                });
            let ripple = ButtonRipple::new(
                button,
                (id.clone(), "card-ripple").into(),
                true,
                true,
                shape,
                Edges::all(px(0.).into()),
                press_ink,
                window,
                cx,
            );
            ButtonFocusRing::with_corners(ripple, focus, true, corners).into_any_element()
        } else {
            surface.into_any_element()
        };

        let draggable = self.draggable && self.preview.is_none();
        let mut root = div()
            .id(id.clone())
            .relative()
            .flex()
            .flex_col()
            .child(body);
        root.style().refine(&self.style);
        if draggable {
            root = root.cursor(if dragging {
                CursorStyle::ClosedHand
            } else {
                CursorStyle::OpenHand
            });
        }
        if draggable || clickable {
            // The press: the pressed state of an action, and a drag's grab.
            let grab = drag.clone();
            root = root.on_mouse_down(MouseButton::Left, move |event, window, cx| {
                grab.update(cx, |drag, _| {
                    drag.grab = Some((event.position, drag.offset));
                    drag.moved = false;
                });
                window.refresh();
            });
        }
        if grabbed {
            // While held, follow the pointer anywhere in the window.
            let follow = drag.clone();
            root = root.child(
                canvas(
                    |_, _, _| (),
                    move |_, _, window, _| {
                        let moving = follow.clone();
                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                            if phase != DispatchPhase::Capture {
                                return;
                            }
                            moving.update(cx, |drag, _| {
                                let Some((start, from)) = drag.grab else {
                                    return;
                                };
                                let delta = event.position - start;
                                if !draggable {
                                    return;
                                }
                                if drag.moved
                                    || delta.x.abs() > CARD_DRAG_SLOP
                                    || delta.y.abs() > CARD_DRAG_SLOP
                                {
                                    drag.moved = true;
                                    drag.offset = from + delta;
                                }
                            });
                            window.refresh();
                        });
                        let dropping = follow.clone();
                        window.on_mouse_event(move |_: &MouseUpEvent, phase, window, cx| {
                            if phase != DispatchPhase::Capture {
                                return;
                            }
                            dropping.update(cx, |drag, _| {
                                drag.grab = None;
                                if drag.moved {
                                    drag.releases += 1;
                                    drag.released_at = drag.offset;
                                }
                                drag.offset = Point::default();
                            });
                            // The click this release may complete reads
                            // `moved`; forget it once the event is handled.
                            let settled = dropping.clone();
                            window.defer(cx, move |_, cx| {
                                settled.update(cx, |drag, _| drag.moved = false);
                            });
                            window.refresh();
                        });
                    },
                )
                .absolute()
                .size_0(),
            );
        }
        let lifted = dragging || grabbed;
        if lifted && offset != Point::default() {
            return root.left(offset.x).top(offset.y).into_any_element();
        }
        if releases > 0 && released_at != Point::default() && !cx.reduce_motion() {
            let curve = crate::motion::curve(crate::motion::MotionToken::DefaultSpatial);
            let ease = curve.ease();
            return root
                .with_animation(
                    ElementId::NamedChild(
                        std::sync::Arc::new(id),
                        format!("card-return-{releases}").into(),
                    ),
                    Animation::new(curve.duration()),
                    move |root, t| {
                        let remaining = 1. - ease(t);
                        root.left(released_at.x * remaining)
                            .top(released_at.y * remaining)
                    },
                )
                .into_any_element();
        }
        root.into_any_element()
    }
}

/// What selecting a [`MaterialCard`] means to assistive technology.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardSelection {
    /// A card that is on or off by itself, as in a multiple selection: a
    /// toggle button.
    #[default]
    Toggle,
    /// One choice of `count`, the `index`th (from 0): a radio button. Put
    /// the cards in a [`CardGroup`] for the radio group and its keyboard.
    Radio { index: usize, count: usize },
}

type CardGroupChange = Rc<dyn Fn(&usize, &mut Window, &mut App)>;

/// A single choice among selectable [`MaterialCard`]s: a radio group named
/// by its label, with roving focus as a `ChoiceGroup` has. Tab
/// enters at the selected card; arrow keys, Home and End move focus among
/// the cards and select them. Each card is a radio button with its place
/// in the set. The caller lays the cards out (`Styled`: a wrapping row, a
/// grid) and keeps the selection.
#[derive(gpui::IntoElement)]
pub struct CardGroup {
    id: ElementId,
    label: SharedString,
    cards: Vec<MaterialCard>,
    on_change: Option<CardGroupChange>,
    style: StyleRefinement,
}

impl CardGroup {
    /// `label` is the radio group's accessible name.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            cards: Vec::new(),
            on_change: None,
            style: StyleRefinement::default(),
        }
    }

    /// Append a choice; give it [`MaterialCard::selected`]. Its action is
    /// the group's.
    pub fn card(mut self, card: MaterialCard) -> Self {
        self.cards.push(card);
        self
    }

    /// Called with the index of a newly chosen card.
    pub fn on_change(mut self, f: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl Styled for CardGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for CardGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use super::roving::{Orientation, Roving, Select, index_handles};
        let count = self.cards.len();
        let focus = index_handles((self.id.clone(), "card-group-focus"), count, window, cx);
        let selected = self
            .cards
            .iter()
            .position(|card| card.selected == Some(true));
        let on_select = self.on_change.clone().map(|on_change| {
            Rc::new(move |index: usize, window: &mut Window, cx: &mut App| {
                on_change(&index, window, cx)
            }) as Select
        });
        let roving = Roving::new(focus.clone(), Orientation::Both)
            .selected(selected)
            .select_on_focus(on_select);
        let entry = roving.tab_stop(window);
        let mut group = div()
            .id(self.id)
            .role(Role::RadioGroup)
            .aria_label(self.label);
        group.style().refine(&self.style);
        let cards = self.cards.into_iter().enumerate().map(|(index, card)| {
            let chosen = card.selected == Some(true);
            let handle = focus[index].clone();
            let on_change = self.on_change.clone();
            let card = MaterialCard {
                selected: Some(chosen),
                selection: CardSelection::Radio { index, count },
                focus: Some((handle.clone(), entry == Some(index))),
                ..card
            };
            card.on_click(move |_, window, cx| {
                handle.focus(window, cx);
                if !chosen && let Some(on_change) = &on_change {
                    on_change(&index, window, cx);
                }
            })
        });
        roving.install(group).children(cards)
    }
}

/// Compatibility appearance helper: full width and 16dp example padding.
/// Returns [`CardSurface`] instead of the earlier `Div`; Styled and
/// ParentElement chains remain supported. Use [`CardSurface::new`] for the
/// reference's unpadded surface, then supply caller-owned content spacing.
pub fn card(variant: CardVariant) -> CardSurface {
    CardSurface::new(variant).w_full().p(CARD_PADDING)
}

/// Elevated compatibility helper, retaining full width and example padding.
pub fn card_surface() -> CardSurface {
    card(CardVariant::Elevated)
}

#[cfg(test)]
#[path = "card_tests.rs"]
mod card_tests;

#[cfg(test)]
mod tests {
    use super::*;
    // The glob import of `gpui` shadows the built-in `#[test]` that
    // `#[gpui::test]` expands to.
    use core::prelude::v1::test;
    use gpui::{Context, TestAppContext};

    #[test]
    fn a_media_label_reads_in_every_palette() {
        use crate::color::{Scheme, palette};
        let contrast = |a: f32, b: f32| (a.max(b) + 0.05) / (a.min(b) + 0.05);
        for id in palette::ALL {
            let scheme = Scheme::from_palette(id);
            let ratio = contrast(
                scheme.get(MEDIA_LABEL_CONTAINER).luminance(),
                scheme.get(MEDIA_LABEL_TEXT).luminance(),
            );
            assert!(ratio >= 4.5, "{}: {ratio}", id.name());
        }
    }

    #[test]
    fn a_warning_notice_uses_the_warning_roles_and_does_not_alert() {
        let (bg, tone, icon, role) = notice_tone(NoticeKind::Warning);
        assert_eq!(bg, ColorRole::WARNING_BG);
        assert_eq!(tone, ColorRole::WARNING);
        assert_eq!(icon, IconName::Warning);
        assert_eq!(role, Role::Status);
        assert_eq!(notice_tone(NoticeKind::Danger).3, Role::Alert);
    }

    #[test]
    fn an_outlined_status_chip_fills_only_what_needs_attention() {
        let filled = |kind| super::outlined_chip_paint(kind).1.is_some();
        assert!(!filled(StatusKind::Success));
        assert!(!filled(StatusKind::Neutral));
        assert!(filled(StatusKind::Warning));
        assert!(filled(StatusKind::Danger));
    }

    #[test]
    fn card_variants_use_m3_container_roles_elevation_and_outline() {
        assert_eq!(
            card_style(CardVariant::Elevated),
            CardStyle {
                container: SURFACE_CONTAINER_LOW,
                elevation: 1,
                outline: None,
            }
        );
        assert_eq!(
            card_style(CardVariant::Filled),
            CardStyle {
                container: SURFACE_CONTAINER_HIGHEST,
                elevation: 0,
                outline: None,
            }
        );
        assert_eq!(
            card_style(CardVariant::Outlined),
            CardStyle {
                container: SURFACE,
                elevation: 0,
                outline: Some(OUTLINE_VARIANT),
            }
        );
        assert_eq!(RADIUS_CARD, px(12.));
        assert_eq!(CARD_PADDING, rem(16.));
    }

    #[test]
    fn a_selected_card_takes_the_secondary_roles() {
        // MDC Android's M3 card, checked: a secondary edge on the outlined card…
        assert_eq!(
            card_outline(CardVariant::Outlined, false),
            Some(OUTLINE_VARIANT)
        );
        assert_eq!(card_outline(CardVariant::Outlined, true), Some(SECONDARY));
        for variant in [
            CardVariant::Elevated,
            CardVariant::Filled,
            CardVariant::Surface,
        ] {
            assert_eq!(card_outline(variant, true), None, "{variant:?}");
        }
        // …and a 16% secondary layer over every container, under which the
        // state layers draw in secondary, except a press, which leads back
        // to unselected and so draws in on-surface-variant.
        assert_eq!(CARD_SELECTED_LAYER, 0.16);
        assert_eq!((CARD_CHECK_SIZE, CARD_CHECK_MARGIN), (24., 8.));
        for state in CardState::ALL {
            let selected = if state == CardState::Pressed {
                ON_SURFACE_VARIANT
            } else {
                SECONDARY
            };
            assert_eq!(card_layer_content(true, state), selected, "{state:?}");
            assert_eq!(card_layer_content(false, state), ON_SURFACE, "{state:?}");
        }
        for variant in [
            CardVariant::Elevated,
            CardVariant::Filled,
            CardVariant::Outlined,
            CardVariant::Surface,
        ] {
            let container = color(card_style(variant).container);
            assert_eq!(card_container(variant, false), container);
            assert_eq!(
                card_container(variant, true),
                state::layer(container, color(SECONDARY), 0.16)
            );
        }
    }

    #[test]
    fn card_states_follow_the_kit_layers_and_elevation() {
        use CardState::*;
        let levels = |variant| CardState::ALL.map(|s| card_elevation(variant, s));
        assert_eq!(levels(CardVariant::Elevated), [1, 2, 1, 1, 4]);
        assert_eq!(levels(CardVariant::Filled), [0, 1, 1, 1, 3]);
        assert_eq!(levels(CardVariant::Outlined), [0, 1, 0, 0, 3]);
        assert_eq!(
            CardState::ALL.map(CardState::layer_opacity),
            [0., state::HOVER, state::FOCUS, state::PRESS, state::DRAG]
        );
        assert_eq!(CardState::from(InteractionPreview::Disabled), Enabled);
        assert_eq!(CardState::from(InteractionPreview::Focused), Focused);
    }

    #[test]
    fn badges_follow_m3_geometry_and_roles() {
        assert_eq!(SMALL_BADGE_SIZE, px(6.));
        assert_eq!(LARGE_BADGE_SIZE, rem(16.));
        assert_eq!(LARGE_BADGE_PADDING, rem(4.));
        assert_eq!((BADGE_CONTAINER, BADGE_LABEL), (ERROR, ON_ERROR));
    }

    #[gpui::test]
    fn large_badge_hugs_short_labels_and_grows_for_long_ones(cx: &mut TestAppContext) {
        struct Badges;
        impl Render for Badges {
            fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                window.set_rem_size(px(14.));
                div()
                    .flex()
                    .items_start()
                    .child(large_badge("one", "3").debug_selector(|| "badge-short".into()))
                    .child(large_badge("many", "999+").debug_selector(|| "badge-long".into()))
                    .child(small_badge().debug_selector(|| "badge-dot".into()))
            }
        }
        let (_, cx) = cx.add_window_view(|_, _| Badges);
        cx.run_until_parked();
        let short = cx.debug_bounds("badge-short").unwrap();
        let long = cx.debug_bounds("badge-long").unwrap();
        let dot = cx.debug_bounds("badge-dot").unwrap();
        assert_eq!(short.size.height, px(16.));
        assert_eq!(short.size.width, px(16.));
        assert_eq!(long.size.height, px(16.));
        assert!(long.size.width > px(16.));
        assert_eq!(dot.size, gpui::size(px(6.), px(6.)));
    }

    #[gpui::test]
    fn a_notice_icon_centres_on_the_first_line_at_every_text_size(cx: &mut TestAppContext) {
        struct Notices;
        impl Render for Notices {
            fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                window.set_rem_size(px(14.));
                let long = "A failure explained at length wraps onto several lines, and its \
                            icon stays beside the first of them, however many follow it.";
                div()
                    .flex()
                    .flex_col()
                    // Wide enough for the short notice's action beside its text.
                    .w(px(400.))
                    .children(
                        [("small", 12., 16.), ("body", 14., 20.), ("large", 24., 32.)].map(
                            |(id, size, line)| {
                                notice_banner(id, NoticeKind::Danger, long)
                                    .text_size(px(size))
                                    .line_height(px(line))
                            },
                        ),
                    )
                    .child(
                        notice_banner("short", NoticeKind::Info, "Short")
                            .line_height(px(20.))
                            .child(
                                div()
                                    .w(px(40.))
                                    .h(px(40.))
                                    .debug_selector(|| "action".into()),
                            ),
                    )
            }
        }
        let (_, cx) = cx.add_window_view(|_, _| Notices);
        cx.run_until_parked();
        let centre = |bounds: Bounds<Pixels>| bounds.origin.y + bounds.size.height / 2.;
        for (id, icon, text, line) in [
            ("small", "small-icon", "small-text", 16.),
            ("body", "body-icon", "body-text", 20.),
            ("large", "large-icon", "large-text", 32.),
        ] {
            let icon = cx.debug_bounds(icon).unwrap();
            let text = cx.debug_bounds(text).unwrap();
            assert!(text.size.height >= px(line * 2.), "{id}: {text:?} wraps");
            let first_line = text.origin.y + px(line / 2.);
            assert!(
                (centre(icon) - first_line).abs() < px(1.),
                "{id}: icon {icon:?}, text {text:?}"
            );
            // The icon's slot is as wide as the icon, then the 12dp gap.
            let gap = text.origin.x - (icon.origin.x + icon.size.width);
            assert!((gap - px(12.)).abs() < px(0.5), "{id}: gap {gap:?}");
        }
        let short = cx.debug_bounds("short-icon").unwrap();
        let action = cx.debug_bounds("action").unwrap();
        assert!((centre(short) - centre(action)).abs() < px(1.));
    }

    pub(crate) fn redraw(cx: &mut gpui::VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }
}
