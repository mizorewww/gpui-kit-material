//! Status and notice surfaces: M3 badges, status chips, banners,
//! cards and snackbars.
use crate::color::ColorRole::{self, *};
use crate::icons::IconName;
use crate::theme::color;
use crate::tokens::{elevation, layout::*, shape::*, state, typography::*};
use gpui::{prelude::*, *};
use std::rc::Rc;

use super::focus::{RingPlacement, RingedButton};
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
/// rounded content clipping there remains unsupported. Interactive card states
/// and their animations are outside this static surface's scope.
///
/// The outline paints above content without participating in layout.
pub struct CardSurface {
    inner: Div,
    outline: Option<ColorRole>,
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
        }
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
                rgba(0x00000000),
                px(1.),
                outline,
                BorderStyle::default(),
            ));
        }
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

/// M3 snackbars sit at elevation level 3.
const SNACKBAR_ELEVATION: u8 = 3;

type SnackbarHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// M3 snackbar: a brief status message on the inverse surface, with an
/// optional text action and close button. The caller places it (usually
/// bottom-leading, 16px from the window edges) and removes it when done.
#[derive(gpui::IntoElement)]
pub struct Snackbar {
    id: ElementId,
    message: SharedString,
    action: Option<(SharedString, SnackbarHandler)>,
    close: Option<(SharedString, SnackbarHandler)>,
}

impl Snackbar {
    pub fn new(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            message: message.into(),
            action: None,
            close: None,
        }
    }

    /// A text button after the message, e.g. "Undo".
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), Rc::new(handler)));
        self
    }

    /// A close icon button, announced with `label`.
    pub fn close(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.close = Some((label.into(), Rc::new(handler)));
        self
    }
}

/// A button on the inverse surface, with its state layer in `fg` and the
/// shared keyboard focus ring.
fn inverse_button(id: ElementId, fg: Rgba, handler: SnackbarHandler) -> RingedButton {
    let bg = color(INVERSE_SURFACE);
    RingedButton::new(id, RingPlacement::Outer(Corners::all(RADIUS_FULL)))
        .map(move |button| {
            button
                .flex()
                .items_center()
                .justify_center()
                .flex_shrink_0()
                .h(rem(40.))
                .rounded_full()
                .text_color(fg)
                .hover(move |s| s.bg(state::hovered(bg, fg)))
                .active(move |s| s.bg(state::pressed(bg, fg)))
        })
        .on_click(move |event, window, cx| handler(event, window, cx))
}

impl RenderOnce for Snackbar {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id;
        let child = |name: &'static str| ElementId::NamedChild(id.clone().into(), name.into());
        gpui_base::h_flex()
            .id(id.clone())
            .items_center()
            .min_h(rem(48.))
            .max_w(rem(560.))
            .pl(rem(16.))
            .pr(rem(if self.action.is_some() || self.close.is_some() {
                4.
            } else {
                16.
            }))
            .gap(rem(4.))
            .rounded(RADIUS_EXTRA_SMALL)
            .bg(color(INVERSE_SURFACE))
            .text_color(color(INVERSE_ON_SURFACE))
            .text_size(TEXT_BODY)
            .shadow(elevation::level(SNACKBAR_ELEVATION))
            .child(
                accessible_text(child("message"), self.message)
                    .role(Role::Status)
                    .flex_1()
                    .min_w_0()
                    .py(rem(14.))
                    .whitespace_normal(),
            )
            .when_some(self.action, |bar, (label, handler)| {
                bar.child(
                    inverse_button(child("action"), color(INVERSE_PRIMARY), handler)
                        .when(cfg!(test), |b| {
                            b.debug_selector(|| "snackbar-action".into())
                        })
                        .px(rem(12.))
                        .font_weight(FontWeight::MEDIUM)
                        .child(label),
                )
            })
            .when_some(self.close, |bar, (label, handler)| {
                bar.child(
                    inverse_button(child("close"), color(INVERSE_ON_SURFACE), handler)
                        .when(cfg!(test), |b| b.debug_selector(|| "snackbar-close".into()))
                        .accessibility_label(label)
                        .w(rem(40.))
                        .child(IconName::Close.icon().size(ICON_SLOT)),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // The glob import of `gpui` shadows the built-in `#[test]` that
    // `#[gpui::test]` expands to.
    use core::prelude::v1::test;
    use gpui::{Context, Modifiers, TestAppContext};

    struct Harness {
        buttons: bool,
        actions: usize,
        closes: usize,
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let snackbar = Snackbar::new("saved", "Icon saved");
            let snackbar = if self.buttons {
                snackbar
                    .action("Undo", cx.listener(|this, _, _, _| this.actions += 1))
                    .close("Dismiss", cx.listener(|this, _, _, _| this.closes += 1))
            } else {
                snackbar
            };
            div().size_full().child(snackbar)
        }
    }

    fn open(
        cx: &mut TestAppContext,
        buttons: bool,
    ) -> (Entity<Harness>, &mut gpui::VisualTestContext) {
        let (view, cx) = cx.add_window_view(move |_, _| Harness {
            buttons,
            actions: 0,
            closes: 0,
        });
        cx.run_until_parked();
        (view, cx)
    }

    #[gpui::test]
    fn action_and_close_report_their_own_clicks(cx: &mut TestAppContext) {
        let (view, cx) = open(cx, true);
        let action = cx.debug_bounds("snackbar-action").expect("action").center();
        let close = cx.debug_bounds("snackbar-close").expect("close").center();
        cx.simulate_click(action, Modifiers::default());
        cx.simulate_click(close, Modifiers::default());
        cx.simulate_click(close, Modifiers::default());
        view.read_with(cx, |h, _| {
            assert_eq!(h.actions, 1);
            assert_eq!(h.closes, 2);
        });
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

    #[test]
    fn snackbar_uses_m3_elevation_level_3() {
        assert_eq!(SNACKBAR_ELEVATION, 3);
        assert_eq!(elevation::level(SNACKBAR_ELEVATION), elevation::level3());
    }

    #[gpui::test]
    fn snackbar_buttons_show_the_ring_only_for_keyboard_focus(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let (_, cx) = open(cx, true);
        let action = cx.debug_bounds("snackbar-action").expect("action");
        cx.simulate_click(action.center(), Modifiers::default());
        redraw(cx);
        assert!(
            crate::components::focus::test_rings::painted("action").is_none(),
            "pointer focus must not show the ring"
        );
        press_key(cx);
        assert_eq!(
            crate::components::focus::test_rings::painted("action"),
            Some(action.dilate(px(5.))),
            "keyboard focus shows the 3dp ring 2dp outside the button"
        );
    }

    pub(crate) fn redraw(cx: &mut gpui::VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }

    /// A key press switches the window to keyboard modality.
    pub(crate) fn press_key(cx: &mut gpui::VisualTestContext) {
        let keystroke =
            cx.update(|_, cx| gpui::Keystroke::parse_with_os("a", cx.platform_info().os).unwrap());
        cx.simulate_event(gpui::KeyDownEvent {
            keystroke,
            is_held: false,
            prefer_character_input: false,
        });
        redraw(cx);
    }

    #[gpui::test]
    fn a_plain_snackbar_has_no_buttons(cx: &mut TestAppContext) {
        let (_, cx) = open(cx, false);
        assert!(cx.debug_bounds("snackbar-action").is_none());
        assert!(cx.debug_bounds("snackbar-close").is_none());
    }
}
