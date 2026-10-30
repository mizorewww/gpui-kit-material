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
pub enum NoticeKind {
    Info,
    Danger,
}

/// A page-level notice: tinted surface, status icon and wrapping text.
/// Callers append the notice's actions, which wrap below the text when narrow.
pub fn notice_banner(
    id: impl Into<ElementId>,
    kind: NoticeKind,
    text: impl Into<SharedString>,
) -> Div {
    let (bg, tone, icon, role) = match kind {
        NoticeKind::Info => (
            ColorRole::SECONDARY_CONTAINER,
            ColorRole::PRIMARY,
            IconName::Info,
            Role::Status,
        ),
        NoticeKind::Danger => (
            ColorRole::ERROR_CONTAINER,
            ColorRole::ERROR,
            IconName::Error,
            Role::Alert,
        ),
    };
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
        .child(icon.icon().size(ICON_SLOT).color(color(tone)))
        .child(
            accessible_text(id, text)
                .role(role)
                .flex_1()
                .flex_basis(rem(200.))
                .min_w_0()
                .whitespace_normal(),
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

/// A status label; same as [`status_chip`], kept under its earlier name.
#[deprecated(
    note = "a status chip, not an M3 badge: use `status_chip` (M3 badges are `small_badge` and `large_badge`)"
)]
pub fn badge(kind: StatusKind) -> Div {
    status_chip(kind)
}

/// A fact beside a label in the status chip's size, outlined rather than
/// filled: e.g. the engine a tile uses.
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

/// The three M3 card variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardVariant {
    /// Surface-container-low with level-1 elevation and no outline.
    Elevated,
    /// Surface-container-highest, flat.
    Filled,
    /// Surface with a 1dp outline-variant edge, flat.
    Outlined,
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
    }
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
            focus_ring: false,
        }
    }

    /// Render a fixed interaction state as a specimen: the on-surface state
    /// layer, the state's elevation and, when focused, the focus ring.
    pub fn state(mut self, state: CardState) -> Self {
        let container = color(card_style(self.variant).container);
        self.inner = self
            .inner
            .bg(state::layer(
                container,
                color(ON_SURFACE),
                state.layer_opacity(),
            ))
            .shadow(elevation::level(card_elevation(self.variant, state)));
        self.focus_ring = state == CardState::Focused;
        self
    }

    /// [`Self::state`] for one of the shared interaction previews.
    pub fn preview_state(self, preview: InteractionPreview) -> Self {
        self.state(preview.into())
    }

    /// A card the pointer can act on: hovering lays the 8% on-surface
    /// state layer over it and raises it to its hovered elevation. The
    /// caller supplies the action (and focus) on the content.
    pub fn interactive(mut self) -> Self {
        let container = color(card_style(self.variant).container);
        let hover = state::hovered(container, color(ON_SURFACE));
        let shadow = elevation::level(card_elevation(self.variant, CardState::Hovered));
        self.inner = self
            .inner
            .hover(move |style| style.bg(hover).shadow(shadow.clone()));
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
            // The kit's focus indicator: 3dp secondary, 2dp outside.
            let radius = px(f32::from(RADIUS_CARD) + 5.);
            window.paint_quad(quad(
                bounds.dilate(px(5.)),
                radius,
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
/// reduced motion). The accessible label names the action.
#[derive(gpui::IntoElement)]
pub struct MaterialCard {
    id: ElementId,
    variant: CardVariant,
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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use super::button::{ButtonFocusRing, ButtonRipple};
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
        let container = color(card_style(self.variant).container);
        let group: SharedString = format!("material-card-{id:?}").into();

        let mut surface = CardSurface::new(self.variant)
            .w_full()
            .children(self.children);
        surface = match (self.preview, dragging) {
            (_, true) => surface.state(CardState::Dragged),
            (Some(state), _) => surface.state(state),
            (None, false) if pressed => surface.state(CardState::Pressed),
            (None, false) if clickable => {
                let hover = state::hovered(container, color(ON_SURFACE));
                let hovered = elevation::level(card_elevation(self.variant, CardState::Hovered));
                surface.group_hover(group.clone(), move |style| {
                    style.bg(hover).shadow(hovered.clone())
                })
            }
            _ => surface,
        };

        let body: AnyElement = if let Some(on_click) = self.on_click.clone().filter(|_| clickable) {
            let focus = window
                .use_keyed_state((id.clone(), "card-focus"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone();
            let click_drag = drag.clone();
            let button = gpui_base::Button::new((id.clone(), "card-button"))
                .group(group)
                .track_focus(&focus)
                .w_full()
                .rounded(RADIUS_CARD)
                .cursor_pointer()
                .when_some(self.label.clone(), |button, label| {
                    button.accessibility_label(label)
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
                super::navigation::ripple_corners(RADIUS_CARD),
                Edges::all(px(0.).into()),
                color(ON_SURFACE),
                window,
                cx,
            );
            ButtonFocusRing::with_corners(ripple, focus, true, Corners::all(RADIUS_CARD))
                .into_any_element()
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

    pub(crate) fn redraw(cx: &mut gpui::VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }
}
