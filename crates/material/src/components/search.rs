//! M3 search: the search bar and the search view (docked and full-screen).
//!
//! Reference: <https://m3.material.io/components/search/specs> and the
//! Material 3 Design Kit "Search" page. Editing, selection, clipboard, IME and
//! the input's accessibility belong to Kit's `component::input::Input`, drawn
//! without its own chrome; suggestions are Base buttons with the shared M3
//! focus ring. This module owns the pill container, state layers, slots and
//! the view's expand/collapse motion (default spatial size, fast effects fade,
//! retargeting mid-flight and honoring reduced motion via [`crate::motion`]).
use super::InteractionPreview;
use super::focus::{RingPlacement, RingedButton};
use super::text_field::type_style;
use crate::{
    color::ColorRole,
    motion::{self, MotionToken},
    theme::color,
    tokens::{
        shape::RADIUS_MEDIUM,
        state,
        typography::{TypeRole, rem},
    },
};
use gpui::{prelude::*, *};
use gpui_base::input::InputState;
use gpui_kit::component::{Sizable as _, Size, input::Input};
use std::rc::Rc;

/// How a search bar sits on the screen. The M3 bar suits a page on the plain surface; on a
/// page ground, a field among tiles reads better as one of them or as a quiet well.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SearchFieldStyle {
    /// The M3 search bar: surface-container-high, no edge.
    #[default]
    Standard,
    /// Surface with a 1dp outline-variant edge: one more tile on the page ground.
    Surface,
    /// Surface-container-highest, no edge: a well, among filled controls.
    Tonal,
}

impl SearchFieldStyle {
    pub const ALL: [Self; 3] = [Self::Standard, Self::Surface, Self::Tonal];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Surface => "Surface",
            Self::Tonal => "Tonal",
        }
    }
    const fn container(self) -> ColorRole {
        match self {
            Self::Standard => SEARCH_CONTAINER,
            Self::Surface => ColorRole::SURFACE,
            Self::Tonal => ColorRole::SURFACE_CONTAINER_HIGHEST,
        }
    }
}

/// The search bar container: surface-container-high (not the highest).
pub(crate) const SEARCH_CONTAINER: ColorRole = ColorRole::SURFACE_CONTAINER_HIGH;
/// Search bar height.
pub const SEARCH_BAR_HEIGHT: f32 = 56.;
/// One suggestion row (a middle-aligned two-line list item).
pub const SEARCH_SUGGESTION_HEIGHT: f32 = 64.;
/// The gap between a docked bar and its results.
const DOCKED_GAP: f32 = 2.;
/// A 48dp touch target around each 24dp icon slot.
const TARGET: f32 = 48.;
/// The full-screen view's margin around its bar.
const FULL_SCREEN_MARGIN: f32 = 12.;

enum BarInput {
    Live(Entity<InputState>),
    /// A specimen: `(text, is_placeholder)`.
    Static(SharedString, bool),
}

/// The M3 search bar: a 56dp pill on surface-container-high with a leading
/// icon, the query and up to two trailing icons or an avatar.
#[derive(gpui::IntoElement)]
pub struct SearchField {
    id: ElementId,
    input: BarInput,
    leading: Option<AnyElement>,
    trailing: Vec<AnyElement>,
    disabled: bool,
    field_style: SearchFieldStyle,
    preview: Option<InteractionPreview>,
    style: StyleRefinement,
}

impl SearchField {
    /// A live bar editing `state`. Its placeholder is the state's placeholder.
    pub fn new(state: &Entity<InputState>) -> Self {
        Self::with_input(
            ElementId::from(("search-field", state.entity_id())),
            BarInput::Live(state.clone()),
        )
    }

    /// A non-editable specimen showing `text`, painted as hint text when
    /// `placeholder` is true.
    pub fn specimen(
        id: impl Into<ElementId>,
        text: impl Into<SharedString>,
        placeholder: bool,
    ) -> Self {
        Self::with_input(id.into(), BarInput::Static(text.into(), placeholder))
    }

    fn with_input(id: ElementId, input: BarInput) -> Self {
        Self {
            id,
            input,
            leading: None,
            trailing: Vec::new(),
            disabled: false,
            field_style: SearchFieldStyle::Standard,
            preview: None,
            style: StyleRefinement::default(),
        }
    }

    /// How the bar sits on the screen. See [`SearchFieldStyle`].
    pub fn field_style(mut self, style: SearchFieldStyle) -> Self {
        self.field_style = style;
        self
    }

    /// A disabled bar takes no focus, input or hover, and fades its content
    /// (icons, query and placeholder) to on-surface at 38%.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
    /// The leading icon or icon button (menu, back arrow, search).
    pub fn leading(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }
    /// Append a trailing icon, icon button or avatar.
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing.push(element.into_any_element());
        self
    }
    /// Render a static specimen in one forced interaction state. See
    /// [`InteractionPreview`].
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
}

impl Styled for SearchField {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

fn alpha(mut c: Rgba, a: f32) -> Rgba {
    c.a *= a;
    c
}

/// A 48dp target holding one 24dp slot.
fn target(child: AnyElement, ink: Rgba) -> Div {
    div()
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .size(rem(TARGET))
        .text_color(ink)
        .child(child)
}

impl RenderOnce for SearchField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let on_surface = color(ColorRole::ON_SURFACE);
        let container = color(self.field_style.container());
        let edged = self.field_style == SearchFieldStyle::Surface;
        let disabled = self.disabled || self.preview.is_some_and(InteractionPreview::is_disabled);
        let preview = self.preview.filter(|_| !disabled);
        let live = preview.is_none() && !disabled;
        let fade = if disabled {
            state::DISABLED_CONTENT
        } else {
            1.
        };
        let layer = preview.map_or(0., InteractionPreview::layer_opacity);
        // Group hover resolves to the nearest painting ancestor of this name,
        // so a static name stays scoped to this bar without formatting one
        // per frame.
        let group = SharedString::new_static("material-search-bar");
        let focus = match &self.input {
            BarInput::Live(state) => Some(state.read(cx).focus_handle(cx)),
            BarInput::Static(..) => None,
        };
        let content: AnyElement = match self.input {
            BarInput::Live(state) => Input::new(&state)
                .appearance(false)
                .disabled(disabled)
                .with_size(Size::XSmall)
                .p_0()
                .h(rem(24.))
                .text_size(TypeRole::BodyLarge.scale().font_size)
                .line_height(TypeRole::BodyLarge.scale().line_height)
                .into_any_element(),
            BarInput::Static(text, placeholder) => div()
                .overflow_hidden()
                .whitespace_nowrap()
                .when(placeholder, |this| {
                    this.text_color(color(ColorRole::ON_SURFACE_VARIANT))
                })
                .child(text)
                .into_any_element(),
        };
        let has_leading = self.leading.is_some();
        let trailing_count = self.trailing.len();
        let mut bar = div()
            .id(self.id.clone())
            .debug_selector(|| "search-bar".into())
            .group(group.clone())
            .relative()
            .flex()
            .items_center()
            .w_full()
            .min_w_0()
            .h(rem(SEARCH_BAR_HEIGHT))
            .px(rem(4.))
            .gap(rem(4.))
            .rounded_full()
            .bg(container)
            .when(edged, |this| {
                this.border(px(1.))
                    .border_color(color(ColorRole::OUTLINE_VARIANT))
            })
            .text_color(on_surface)
            .map(|this| type_style(this, TypeRole::BodyLarge))
            .when(live, |this| this.cursor_text())
            // The state layer, under the content.
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .rounded_full()
                    .bg(alpha(on_surface, layer))
                    .when(live, |this| {
                        this.group_hover(group.clone(), move |s| {
                            s.bg(alpha(on_surface, state::HOVER))
                        })
                    }),
            )
            .when(live, |this| {
                this.active(move |s| s.bg(state::pressed(container, on_surface)))
            })
            .children(
                self.leading
                    .map(|icon| target(icon, color(ColorRole::ON_SURFACE)).opacity(fade)),
            )
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_w_0()
                    .opacity(fade)
                    .when(!has_leading, |this| this.pl(rem(12.)))
                    .when(trailing_count == 0, |this| this.pr(rem(12.)))
                    .child(content),
            )
            .children(
                self.trailing
                    .into_iter()
                    .map(|icon| target(icon, color(ColorRole::ON_SURFACE_VARIANT)).opacity(fade)),
            );
        if let Some(focus) = focus.filter(|_| live) {
            bar = bar.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                focus.focus(window, cx);
            });
        }
        let _ = window;
        bar.style().refine(&self.style);
        bar
    }
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// One search suggestion: a 64dp row with an optional leading element (icon
/// or 40dp avatar), a label and optional supporting text.
pub struct SearchSuggestion {
    label: SharedString,
    supporting: Option<SharedString>,
    leading: Option<AnyElement>,
    on_click: Option<ClickHandler>,
}

impl SearchSuggestion {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            supporting: None,
            leading: None,
            on_click: None,
        }
    }
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }
    pub fn leading(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// The row; only a `live` (expanded) view's rows take focus or clicks,
    /// so a collapsing view's fading rows cannot be activated.
    fn render(self, id: ElementId, live: bool) -> AnyElement {
        let on_surface = color(ColorRole::ON_SURFACE);
        let clear = alpha(on_surface, 0.);
        let label = self.label.clone();
        let mut row = RingedButton::new(id, RingPlacement::Inner(Corners::default()))
            .state_layers(
                alpha(on_surface, state::HOVER),
                alpha(on_surface, state::PRESS),
            )
            .ripple(on_surface)
            .accessibility_label(label)
            .tab_stop(live)
            .map(|button| {
                button
                    .flex()
                    .items_center()
                    .w_full()
                    .h(rem(SEARCH_SUGGESTION_HEIGHT))
                    .pl(rem(16.))
                    .pr(rem(24.))
                    .gap(rem(16.))
                    .bg(clear)
                    .cursor_pointer()
            })
            .children(self.leading.map(|leading| {
                div()
                    .flex()
                    .flex_shrink_0()
                    .items_center()
                    .justify_center()
                    .min_w(rem(24.))
                    .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                    .child(leading)
            }))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .child(
                        type_style(div(), TypeRole::BodyLarge)
                            .text_color(on_surface)
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .child(self.label),
                    )
                    .children(self.supporting.map(|text| {
                        type_style(div(), TypeRole::BodyMedium)
                            .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .child(text)
                    })),
            );
        if let Some(handler) = self.on_click.filter(|_| live) {
            row = row.on_click(move |event, window, cx| handler(event, window, cx));
        }
        row.into_any_element()
    }
}

/// Where a search view shows its results.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SearchViewLayout {
    /// Results drop below the bar in a container of their own.
    #[default]
    Docked,
    /// The view covers its whole area, the bar inset at the top.
    FullScreen,
}

/// How a search view sets its results apart from the bar (md3.io's Style).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SearchViewStyle {
    /// M3 Expressive: results in a container of their own, 2dp below a
    /// docked bar (full-screen: under the inset pill).
    #[default]
    Contained,
    /// The baseline view: bar and results share one surface (28dp corners
    /// when docked, edge to edge full-screen) split by a 1dp outline
    /// divider.
    Divided,
}

type ExpandedChange = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// The M3 search view: a search bar that expands into a results surface.
///
/// The caller controls `expanded` (typically opening on the input's focus and
/// closing from a back button or a chosen suggestion). The results grow from
/// the bar on the default spatial token and fade on fast effects; collapsing
/// plays the same motion back and keeps the rows painted until it settles.
#[derive(gpui::IntoElement)]
pub struct SearchView {
    id: ElementId,
    bar: SearchField,
    layout: SearchViewLayout,
    view_style: SearchViewStyle,
    expanded: bool,
    expand_on_focus: bool,
    collapse_on_blur: bool,
    on_expanded_change: Option<ExpandedChange>,
    suggestions: Vec<SearchSuggestion>,
    max_visible: usize,
    style: StyleRefinement,
}

impl SearchView {
    pub fn new(id: impl Into<ElementId>, bar: SearchField) -> Self {
        Self {
            id: id.into(),
            bar,
            layout: SearchViewLayout::Docked,
            view_style: SearchViewStyle::Contained,
            expanded: false,
            expand_on_focus: false,
            collapse_on_blur: false,
            on_expanded_change: None,
            suggestions: Vec::new(),
            max_visible: 5,
            style: StyleRefinement::default(),
        }
    }
    pub fn layout(mut self, layout: SearchViewLayout) -> Self {
        self.layout = layout;
        self
    }
    /// Contained (default) or divided results. See [`SearchViewStyle`].
    pub fn view_style(mut self, style: SearchViewStyle) -> Self {
        self.view_style = style;
        self
    }
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
    /// Ask to expand when the bar's input gains focus (not for a disabled
    /// bar). The caller owns the state: see [`Self::on_expanded_change`].
    pub fn expand_on_focus(mut self, expand: bool) -> Self {
        self.expand_on_focus = expand;
        self
    }
    /// Ask to collapse on a press outside the view or on Escape inside it.
    pub fn collapse_on_blur(mut self, collapse: bool) -> Self {
        self.collapse_on_blur = collapse;
        self
    }
    /// Called with the state [`Self::expand_on_focus`] and
    /// [`Self::collapse_on_blur`] ask for.
    pub fn on_expanded_change(
        mut self,
        f: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_expanded_change = Some(Rc::new(f));
        self
    }
    pub fn suggestion(mut self, suggestion: SearchSuggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }
    pub fn suggestions(mut self, suggestions: impl IntoIterator<Item = SearchSuggestion>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }
    /// How many rows a docked view shows before it scrolls (default 5).
    pub fn max_visible(mut self, rows: usize) -> Self {
        self.max_visible = rows.max(1);
        self
    }
}

impl Styled for SearchView {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// The results' revealed height in design px for an expansion `t`.
pub(crate) fn results_height(rows: usize, max_visible: usize, t: f32) -> f32 {
    rows.min(max_visible) as f32 * SEARCH_SUGGESTION_HEIGHT * t.clamp(0., 1.)
}

impl RenderOnce for SearchView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        // Focus moves refresh the window, so the rising edge shows up here for
        // pointer and keyboard alike. The request waits for the frame to end.
        let on_change = self.on_expanded_change.clone();
        if let (BarInput::Live(input), true, Some(on_change)) =
            (&self.bar.input, self.expand_on_focus, &on_change)
        {
            let focused = input.read(cx).focus_handle(cx).is_focused(window);
            let last = window.use_keyed_state((id.clone(), "bar-focused"), cx, |_, _| false);
            if *last.read(cx) != focused {
                last.update(cx, |last, _| *last = focused);
                if focused && !self.expanded && !self.bar.disabled {
                    let on_change = on_change.clone();
                    window.defer(cx, move |window, cx| on_change(&true, window, cx));
                }
            }
        }
        let collapse = on_change.filter(|_| self.collapse_on_blur && self.expanded);
        let expand = motion::token_value(
            (id.clone(), "expand"),
            if self.expanded { 1. } else { 0. },
            MotionToken::DefaultSpatial,
            window,
            cx,
        );
        let fade = motion::token_value(
            (id.clone(), "fade"),
            if self.expanded { 1. } else { 0. },
            MotionToken::FastEffects,
            window,
            cx,
        )
        .clamp(0., 1.);
        let visible = self.expanded || fade > 0.001 || expand > 0.001;
        let rows = self.suggestions.len();
        let live = self.expanded;
        let divided = self.view_style == SearchViewStyle::Divided;
        let divider = |fade: f32| {
            div()
                .debug_selector(|| "search-divider".into())
                .flex_shrink_0()
                .w_full()
                .h(px(1.))
                .bg(color(ColorRole::OUTLINE))
                .opacity(fade)
        };
        let list = |suggestions: Vec<SearchSuggestion>| {
            div()
                .id((id.clone(), "results"))
                .flex()
                .flex_col()
                .w_full()
                .overflow_y_scroll()
                .children(suggestions.into_iter().enumerate().map(|(ix, s)| {
                    s.render(
                        ElementId::NamedInteger("suggestion".into(), ix as u64),
                        live,
                    )
                }))
        };
        let mut root = match self.layout {
            SearchViewLayout::Docked if divided => {
                let height = results_height(rows, self.max_visible, expand);
                let open = visible && rows > 0;
                div()
                    .id(id.clone())
                    .relative()
                    .flex()
                    .flex_col()
                    .w_full()
                    .max_w(rem(360.))
                    .rounded(rem(SEARCH_BAR_HEIGHT / 2.))
                    .overflow_hidden()
                    .when(open, |this| this.bg(color(SEARCH_CONTAINER)))
                    .child(self.bar)
                    .when(open, |this| {
                        this.child(divider(fade)).child(
                            div()
                                .debug_selector(|| "search-results".into())
                                .w_full()
                                .h(rem(height))
                                .overflow_hidden()
                                .opacity(fade)
                                .child(list(self.suggestions)),
                        )
                    })
            }
            SearchViewLayout::Docked => {
                let height = results_height(rows, self.max_visible, expand);
                div()
                    .id(id.clone())
                    .relative()
                    .flex()
                    .flex_col()
                    .w_full()
                    .max_w(rem(360.))
                    .child(self.bar)
                    .when(visible && rows > 0, |this| {
                        this.child(
                            div()
                                .debug_selector(|| "search-results".into())
                                .mt(rem(DOCKED_GAP))
                                .w_full()
                                .h(rem(height))
                                .overflow_hidden()
                                .rounded(rem(RADIUS_MEDIUM.as_f32()))
                                .bg(color(SEARCH_CONTAINER))
                                .opacity(fade)
                                .child(list(self.suggestions)),
                        )
                    })
            }
            SearchViewLayout::FullScreen => {
                // A divided view runs its header edge to edge.
                let margin = if divided {
                    0.
                } else {
                    FULL_SCREEN_MARGIN * expand.clamp(0., 1.)
                };
                div()
                    .id(id.clone())
                    .relative()
                    .flex()
                    .flex_col()
                    .size_full()
                    .overflow_hidden()
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .bg(color(ColorRole::SURFACE))
                            .opacity(fade),
                    )
                    .child(div().relative().w_full().px(rem(margin)).child(if divided {
                        self.bar.rounded_none()
                    } else {
                        self.bar
                    }))
                    .when(visible && divided, |this| this.child(divider(fade)))
                    .when(visible, |this| {
                        this.child(
                            div()
                                .debug_selector(|| "search-results".into())
                                .relative()
                                .flex_1()
                                .min_h_0()
                                .w_full()
                                .overflow_hidden()
                                .opacity(fade)
                                .child(list(self.suggestions)),
                        )
                    })
            }
        };
        root.style().refine(&self.style);
        root.when_some(collapse, |root, collapse| {
            let outside = collapse.clone();
            root.on_mouse_down_out(move |_, window, cx| outside(&false, window, cx))
                .on_key_down(move |event, window, cx| {
                    if event.keystroke.key == "escape" {
                        collapse(&false, window, cx);
                    }
                })
        })
    }
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
