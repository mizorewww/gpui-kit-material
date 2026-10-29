//! M3 Expressive top app bars and the bottom app bar.
//!
//! [`TopAppBar`] renders the kit's configurations ([`TopAppBarVariant`]):
//! small, center-aligned, medium flexible, large flexible and search. Each
//! holds a caller navigation icon, a title with an optional subtitle (or a
//! caller element in the title's place, e.g. a logo) and caller trailing
//! actions. Strings and actions come from the caller; the bar owns layout,
//! typography, color and motion.
//!
//! Scrolling is driven by the caller's content scroll offset
//! ([`TopAppBar::scroll_offset`], design px):
//!
//! - Any offset above zero is the scrolled-under state: the container fades
//!   from surface to surface-container (and a search bar's field from
//!   surface-container to surface-container-highest) on the default effects
//!   spring, which keeps its fade under reduced motion.
//! - A flexible bar collapses with the offset itself, from its expanded
//!   height to the 64dp collapsed one. The large title rides up with the
//!   bar's bottom edge and fades out (alpha `1 - f`) while the collapsed
//!   title fades in over the second half ([`collapsed_title_alpha`]), the
//!   two-row bar's title handoff. The collapse follows the scroll, so it
//!   needs no clock and is identical under reduced motion.
//!
//! [`BottomAppBar`] is the 80dp surface-container bar of up to four actions
//! and a trailing FAB, hidden and shown with [`BottomAppBar::visible`] on the
//! default spatial spring (at once under reduced motion).
//!
//! Semantics: a top app bar is a banner landmark whose title is a heading; a
//! bottom app bar is a GPUI Base toolbar (toolbar role, Left/Right roving
//! focus among its actions).
use super::InteractionPreview;
use super::focus::{RingPlacement, RingedButton};
use super::navigation::type_role;
use crate::color::ColorRole;
use crate::motion::{self, MotionToken};
use crate::theme::{blend, color};
use crate::tokens::{
    state,
    typography::{TypeRole, rem},
};
use gpui::{prelude::FluentBuilder as _, *};
use std::rc::Rc;

/// Height of a single-row (and of a collapsed flexible) top app bar.
pub const TOP_APP_BAR_HEIGHT: f32 = 64.;
/// Height of the bottom app bar.
pub const BOTTOM_APP_BAR_HEIGHT: f32 = 80.;

/// Icon target and the bar's edge padding.
const TARGET: f32 = 48.;
const EDGE: f32 = 4.;
/// The flexible bar's top row and the space under its large title.
const TOP_ROW: f32 = 56.;
const TITLE_GAP: f32 = 8.;
const TITLE_BOTTOM: f32 = 12.;
const TITLE_INSET: f32 = 16.;
/// Search field height and the space between it and the icons.
const SEARCH_HEIGHT: f32 = 56.;
const SEARCH_INSET: f32 = 8.;
const SEARCH_PADDING: f32 = 20.;

/// The kit's top app bar configurations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TopAppBarVariant {
    /// Title after the navigation icon, 64dp.
    #[default]
    Small,
    /// Title centered across the bar, 64dp.
    CenterAligned,
    /// A headline-medium title under the icons that collapses into a small
    /// bar on scroll; 112dp expanded (136dp with a subtitle).
    MediumFlexible,
    /// A display-small title under the icons that collapses into a small
    /// bar on scroll; 120dp expanded (152dp with a subtitle).
    LargeFlexible,
    /// A search field between the navigation icon and the actions, 64dp.
    Search,
}

impl TopAppBarVariant {
    pub const ALL: [Self; 5] = [
        Self::CenterAligned,
        Self::Search,
        Self::Small,
        Self::MediumFlexible,
        Self::LargeFlexible,
    ];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::CenterAligned => "Small-centered",
            Self::MediumFlexible => "Medium flexible",
            Self::LargeFlexible => "Large flexible",
            Self::Search => "Search",
        }
    }
    pub const fn is_flexible(self) -> bool {
        matches!(self, Self::MediumFlexible | Self::LargeFlexible)
    }
    /// The title and subtitle roles of the bar's (expanded) title.
    pub const fn title_roles(self) -> (TypeRole, TypeRole) {
        match self {
            Self::MediumFlexible => (TypeRole::HeadlineMedium, TypeRole::LabelLarge),
            Self::LargeFlexible => (TypeRole::DisplaySmall, TypeRole::TitleMedium),
            Self::Small | Self::CenterAligned | Self::Search => {
                (TypeRole::TitleLarge, TypeRole::LabelMedium)
            }
        }
    }
    /// Space between the title and its subtitle.
    pub const fn subtitle_gap(self) -> f32 {
        match self {
            Self::MediumFlexible => 4.,
            Self::LargeFlexible => 8.,
            Self::Small | Self::CenterAligned | Self::Search => 0.,
        }
    }
    /// Expanded height in design px.
    pub fn expanded_height(self, subtitle: bool) -> f32 {
        if !self.is_flexible() {
            return TOP_APP_BAR_HEIGHT;
        }
        let (title, sub) = self.title_roles();
        let mut text = line_height(title);
        if subtitle {
            text += line_height(sub) + self.subtitle_gap();
        }
        TOP_ROW + TITLE_GAP + text + TITLE_BOTTOM
    }
    /// How far a flexible bar has collapsed at `offset` (0 expanded, 1
    /// collapsed); single-row bars never collapse.
    pub fn collapse_fraction(self, subtitle: bool, offset: f32) -> f32 {
        let range = self.expanded_height(subtitle) - TOP_APP_BAR_HEIGHT;
        if range <= 0. {
            0.
        } else {
            (offset / range).clamp(0., 1.)
        }
    }
    /// The bar's height at `offset`.
    pub fn height(self, subtitle: bool, offset: f32) -> f32 {
        let expanded = self.expanded_height(subtitle);
        expanded - (expanded - TOP_APP_BAR_HEIGHT) * self.collapse_fraction(subtitle, offset)
    }
}

/// A role's line height in design px at the base text scale.
fn line_height(role: TypeRole) -> f32 {
    match role.scale().line_height {
        Rems(value) => value * crate::tokens::typography::BASE_PX,
    }
}

/// The collapsed title's alpha at collapse fraction `f`: hidden over the
/// first half, then fading in on an ease-in-out so it arrives as the large
/// title leaves.
pub fn collapsed_title_alpha(f: f32) -> f32 {
    let t = ((f - 0.5) * 2.).clamp(0., 1.);
    t * t * (3. - 2. * t)
}

/// Container, search-field and scrolled amounts for one frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BarPaint {
    pub container: Rgba,
    pub search: Rgba,
}

/// `scrolled` is the effects amount, 0 flat and 1 scrolled-under.
pub(crate) fn bar_paint(scrolled: f32) -> BarPaint {
    let scrolled = scrolled.clamp(0., 1.);
    BarPaint {
        container: blend(
            color(ColorRole::SURFACE),
            color(ColorRole::SURFACE_CONTAINER),
            scrolled,
        ),
        search: blend(
            color(ColorRole::SURFACE_CONTAINER),
            color(ColorRole::SURFACE_CONTAINER_HIGHEST),
            scrolled,
        ),
    }
}

#[cfg(test)]
thread_local! {
    pub(crate) static PAINTED: std::cell::RefCell<std::collections::BTreeMap<String, (BarPaint, f32)>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// An M3 Expressive top app bar.
#[derive(IntoElement)]
pub struct TopAppBar {
    id: ElementId,
    variant: TopAppBarVariant,
    title: Option<SharedString>,
    subtitle: Option<SharedString>,
    title_content: Option<AnyElement>,
    centered: bool,
    navigation: Option<AnyElement>,
    actions: Vec<AnyElement>,
    scroll_offset: f32,
    scrolled_under: Option<bool>,
    search_placeholder: SharedString,
    on_search: Option<ClickHandler>,
    preview: Option<InteractionPreview>,
}

impl TopAppBar {
    pub fn new(id: impl Into<ElementId>, variant: TopAppBarVariant) -> Self {
        Self {
            id: id.into(),
            variant,
            title: None,
            subtitle: None,
            title_content: None,
            centered: variant == TopAppBarVariant::CenterAligned,
            navigation: None,
            actions: Vec::new(),
            scroll_offset: 0.,
            scrolled_under: None,
            search_placeholder: SharedString::default(),
            on_search: None,
            preview: None,
        }
    }
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }
    /// Supporting text under the title.
    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
    /// A caller element in the title's place, e.g. a logo or image. A
    /// text title, if also set, stays the bar's accessible heading.
    pub fn title_content(mut self, content: impl IntoElement) -> Self {
        self.title_content = Some(content.into_any_element());
        self
    }
    /// Center a flexible bar's titles (center-aligned bars always are).
    pub fn centered(mut self, centered: bool) -> Self {
        self.centered = centered || self.variant == TopAppBarVariant::CenterAligned;
        self
    }
    /// The leading navigation control, e.g. a menu or back icon button.
    pub fn navigation_icon(mut self, icon: impl IntoElement) -> Self {
        self.navigation = Some(icon.into_any_element());
        self
    }
    /// A trailing action, e.g. an icon button or avatar.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
    pub fn actions<E: IntoElement>(mut self, actions: impl IntoIterator<Item = E>) -> Self {
        self.actions
            .extend(actions.into_iter().map(IntoElement::into_any_element));
        self
    }
    /// How far the content under the bar has scrolled, in design px.
    /// Override the scrolled-under state the offset implies, e.g. for a bar
    /// over content the caller knows is scrolled, or a static specimen. A
    /// flexible bar still collapses with the offset alone.
    pub fn scrolled_under(mut self, scrolled: bool) -> Self {
        self.scrolled_under = Some(scrolled);
        self
    }
    fn is_scrolled_under(&self) -> bool {
        self.scrolled_under.unwrap_or(self.scroll_offset > 0.)
    }
    pub fn scroll_offset(mut self, offset: f32) -> Self {
        self.scroll_offset = offset.max(0.);
        self
    }
    /// The search field's placeholder (search bars).
    pub fn search_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.search_placeholder = placeholder.into();
        self
    }
    /// Activating the search field, e.g. to open a search view.
    pub fn on_search(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_search = Some(Rc::new(handler));
        self
    }
    /// Render the search field as a static specimen in one forced
    /// interaction state. See [`InteractionPreview`].
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }

    fn title_block(
        &self,
        id: &'static str,
        roles: (TypeRole, TypeRole),
        gap: f32,
        heading: bool,
    ) -> Option<Div> {
        let title = self.title.clone()?;
        let centered = self.centered;
        let text = |text: SharedString, role: TypeRole, ink: ColorRole| {
            div()
                .w_full()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .when(centered, |line| line.text_center())
                .text_color(color(ink))
                .map(|line| type_role(line, role))
                .child(text)
        };
        let block = div()
            .id(id)
            .min_w_0()
            .flex()
            .flex_col()
            .gap(rem(gap))
            .when(centered, |block| block.items_center())
            .when(heading, |block| block.role(Role::Heading).aria_label(title))
            .when_some(self.title.clone(), |block, title| {
                block.child(text(title, roles.0, ColorRole::ON_SURFACE))
            })
            .when_some(self.subtitle.clone(), |block, subtitle| {
                block.child(text(subtitle, roles.1, ColorRole::ON_SURFACE_VARIANT))
            });
        Some(div().min_w_0().child(block))
    }
}

impl RenderOnce for TopAppBar {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let variant = self.variant;
        let subtitle = self.subtitle.is_some();
        let scrolled = motion::token_value(
            (self.id.clone(), "app-bar-scrolled"),
            if self.is_scrolled_under() { 1. } else { 0. },
            MotionToken::DefaultEffects,
            window,
            cx,
        );
        let paint = bar_paint(scrolled);
        let f = variant.collapse_fraction(subtitle, self.scroll_offset);
        #[cfg(test)]
        PAINTED.with(|painted| {
            painted
                .borrow_mut()
                .insert(format!("{:?}", self.id), (paint, f))
        });
        let height = variant.height(subtitle, self.scroll_offset);
        let leading = self.navigation.take();
        let has_leading = leading.is_some();
        let actions = std::mem::take(&mut self.actions);
        let action_count = actions.len();
        let trailing = div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_end()
            .children(actions);
        let leading = leading.map(|icon| div().flex_shrink_0().child(icon));
        let content = self.title_content.take();
        let small_roles = TopAppBarVariant::Small.title_roles();
        // Space from the bar's edge to a small title without a navigation
        // icon: the 16dp title inset.
        let lead_gap = if has_leading {
            EDGE
        } else {
            TITLE_INSET - EDGE
        };

        let bar = div()
            .id(self.id.clone())
            .role(Role::Banner)
            .when(cfg!(test), |bar| {
                bar.debug_selector(|| "top-app-bar".into())
            })
            .relative()
            .w_full()
            .flex_shrink_0()
            .overflow_hidden()
            .h(rem(height))
            .bg(paint.container);

        match variant {
            TopAppBarVariant::Small | TopAppBarVariant::Search => {
                let middle = if variant == TopAppBarVariant::Search {
                    let preview = self.preview;
                    let ink = color(ColorRole::ON_SURFACE_VARIANT);
                    let on_search = self.on_search.clone();
                    let scale = TypeRole::BodyLarge.scale();
                    div().flex_1().min_w_0().mx(rem(SEARCH_INSET)).child(
                        RingedButton::new(
                            (self.id.clone(), "search"),
                            RingPlacement::Outer(Corners::all(
                                window.rem_size() * rem(SEARCH_HEIGHT / 2.).0,
                            )),
                        )
                        .accessibility_label(self.search_placeholder.clone())
                        .map(|field| {
                            field
                                .w_full()
                                .h(rem(SEARCH_HEIGHT))
                                .px(rem(SEARCH_PADDING))
                                .flex()
                                .items_center()
                                .justify_start()
                                .rounded_full()
                                .bg(paint.search)
                                .text_color(ink)
                                .text_size(scale.font_size)
                                .line_height(scale.line_height)
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .child(self.search_placeholder.clone())
                                .when(on_search.is_some(), |field| field.cursor_pointer())
                                .when_some(on_search, |field, handler| {
                                    field.on_click(move |event, window, cx| {
                                        handler(event, window, cx)
                                    })
                                })
                        })
                        .state_layers(
                            state::hovered(paint.search, ink),
                            state::pressed(paint.search, ink),
                        )
                        .when_some(preview, |field, preview| field.preview_state(preview)),
                    )
                } else {
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .items_center()
                        .ml(rem(lead_gap))
                        .mr(rem(EDGE))
                        .children(content)
                        .children(self.title_block("app-bar-title", small_roles, 0., true))
                };
                bar.flex()
                    .items_center()
                    .px(rem(EDGE))
                    .children(leading)
                    .child(middle)
                    .child(trailing)
                    .into_any_element()
            }
            TopAppBarVariant::CenterAligned => {
                // The title centers on the bar, clear of the wider side.
                let side = EDGE + TARGET * (usize::from(has_leading).max(action_count)) as f32;
                bar.flex()
                    .items_center()
                    .px(rem(EDGE))
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .px(rem(side.max(TITLE_INSET)))
                            .children(content)
                            .children(self.title_block("app-bar-title", small_roles, 0., true)),
                    )
                    .children(leading)
                    .child(div().flex_1())
                    .child(trailing)
                    .into_any_element()
            }
            TopAppBarVariant::MediumFlexible | TopAppBarVariant::LargeFlexible => {
                let top_alpha = collapsed_title_alpha(f);
                let top_title = (top_alpha > 0.001)
                    .then(|| self.title_block("app-bar-collapsed-title", small_roles, 0., false))
                    .flatten()
                    .map(|title| title.opacity(top_alpha));
                let expanded_title = self
                    .title_block(
                        "app-bar-title",
                        variant.title_roles(),
                        variant.subtitle_gap(),
                        true,
                    )
                    .map(|title| title.w_full().opacity(1. - f));
                let top_row = div()
                    .relative()
                    .flex()
                    .flex_shrink_0()
                    .items_start()
                    .h(rem(TOP_ROW))
                    .pt(rem(EDGE * 2.))
                    .px(rem(EDGE))
                    .children(leading)
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .h(rem(TARGET))
                            .flex()
                            .items_center()
                            .when(self.centered, |middle| middle.justify_center())
                            .ml(rem(lead_gap))
                            .mr(rem(EDGE))
                            .children(top_title),
                    )
                    .child(trailing);
                let area = div()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .justify_end()
                    .px(rem(TITLE_INSET))
                    .pb(rem(TITLE_BOTTOM))
                    .children(content.map(|content| div().opacity(1. - f).child(content)))
                    .children(expanded_title);
                bar.flex()
                    .flex_col()
                    .child(top_row)
                    .child(area)
                    .into_any_element()
            }
        }
    }
}

#[cfg(test)]
thread_local! {
    pub(crate) static BOTTOM_SHOWN: std::cell::RefCell<std::collections::BTreeMap<String, f32>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

/// The M3 bottom app bar: actions and an optional trailing FAB.
#[derive(IntoElement)]
pub struct BottomAppBar {
    id: ElementId,
    label: SharedString,
    actions: Vec<AnyElement>,
    fab: Option<AnyElement>,
    visible: bool,
}

impl BottomAppBar {
    /// `label` names the bar's toolbar for assistive technology.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            actions: Vec::new(),
            fab: None,
            visible: true,
        }
    }
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
    pub fn actions<E: IntoElement>(mut self, actions: impl IntoIterator<Item = E>) -> Self {
        self.actions
            .extend(actions.into_iter().map(IntoElement::into_any_element));
        self
    }
    pub fn fab(mut self, fab: impl IntoElement) -> Self {
        self.fab = Some(fab.into_any_element());
        self
    }
    /// Show or hide the bar, e.g. hidden while scrolling down.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

impl RenderOnce for BottomAppBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let shown = motion::token_value(
            (self.id.clone(), "bottom-app-bar-visible"),
            if self.visible { 1. } else { 0. },
            MotionToken::DefaultSpatial,
            window,
            cx,
        );
        #[cfg(test)]
        BOTTOM_SHOWN.with(|map| map.borrow_mut().insert(format!("{:?}", self.id), shown));
        let gone = !self.visible && shown <= 0.001;
        div()
            .when(cfg!(test), |bar| {
                bar.debug_selector(|| "bottom-app-bar".into())
            })
            .w_full()
            .flex_shrink_0()
            .overflow_hidden()
            .h(rem(BOTTOM_APP_BAR_HEIGHT * shown.max(0.)))
            .child(
                gpui_base::Toolbar::new(self.id)
                    .aria_label(self.label)
                    .w_full()
                    .h(rem(BOTTOM_APP_BAR_HEIGHT))
                    .flex()
                    .items_center()
                    .pl(rem(EDGE))
                    .pr(rem(TITLE_INSET))
                    .bg(color(ColorRole::SURFACE_CONTAINER))
                    .when(!gone, |bar| {
                        bar.child(
                            div()
                                .flex_1()
                                .flex()
                                .items_center()
                                .gap(rem(TITLE_GAP))
                                .children(self.actions),
                        )
                        .children(self.fab)
                    }),
            )
    }
}

#[cfg(test)]
#[path = "app_bar_tests.rs"]
mod tests;
