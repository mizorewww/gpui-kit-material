//! M3 primary and secondary tabs, fixed or scrollable.
//!
//! - Primary tabs head a whole view: label and icon in primary when active,
//!   a 3dp primary indicator with rounded top corners as wide as the tab's
//!   content, and icon-over-label tabs 64dp tall (48dp otherwise).
//! - Secondary tabs sit inside a content area: label in on-surface when
//!   active, icon beside label, and a 2dp primary indicator across the whole
//!   tab, 48dp tall.
//!
//! Fixed tabs share the width equally; scrollable tabs hug their content
//! after the M3 52dp leading inset and scroll horizontally, revealing the
//! focused or selected tab. The indicator slides and resizes between tabs on
//! the default spatial spring (retargeting from wherever it is), while label
//! and icon colors cross-fade on the fast effects spring.
//!
//! Behavior is GPUI Base's `Tab`/`Tabs` (tab list and tab roles, selection,
//! position in set, pointer and Enter/Space activation); arrow keys, Home
//! and End rove focus through the shared [`super::roving`] helper, and
//! activation is manual unless [`MaterialTabs::auto_activate`] is set.
use super::interaction::{ButtonFocusRing, ButtonRipple};
use super::navigation::{NavBadge, badged_icon, type_role};
use super::ripple::ripple_corners;
use super::roving::{self, Change, Orientation, Roving};
use super::{InteractionPreview, large_badge, small_badge};
use crate::color::ColorRole;
use crate::icons::Icon;
use crate::motion::{self, MotionToken};
use crate::theme::{blend, color};
use crate::tokens::{
    state,
    typography::{TypeRole, rem},
};
use gpui::{prelude::FluentBuilder as _, *};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

/// The two M3 tab styles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsVariant {
    #[default]
    Primary,
    Secondary,
}

/// One tab: a unique value, the caller's label (also its accessible name
/// when only the icon shows), an optional icon and badge.
#[derive(Clone)]
pub struct TabItem {
    value: SharedString,
    label: SharedString,
    icon: Option<Icon>,
    show_label: bool,
    badge: NavBadge,
    disabled: bool,
    preview: Option<InteractionPreview>,
}

impl TabItem {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            show_label: true,
            badge: NavBadge::None,
            disabled: false,
            preview: None,
        }
    }
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    /// Show only the icon; the label remains the accessible name.
    pub fn icon_only(mut self) -> Self {
        self.show_label = false;
        self
    }
    /// A badge on the icon, or after the label of a tab without an icon.
    pub fn badge(mut self, badge: NavBadge) -> Self {
        self.badge = badge;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Render this tab as a static specimen in one forced interaction state.
    /// A tab row holding a previewed tab is a specimen and leaves the tab
    /// order. See [`InteractionPreview`].
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
    pub fn value(&self) -> &SharedString {
        &self.value
    }
    pub fn label(&self) -> &SharedString {
        &self.label
    }
    fn is_disabled(&self) -> bool {
        self.disabled || self.preview.is_some_and(InteractionPreview::is_disabled)
    }
}

/// M3 tabs over a caller-controlled selection.
///
/// # Example
///
/// ```
/// use gpui_kit::{SharedString, Window, prelude::*};
/// use gpui_kit_material::{icons::Symbol, prelude::*};
///
/// struct Trip {
///     tab: SharedString,
/// }
///
/// impl Render for Trip {
///     fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
///         MaterialTabs::new("tabs", "Trip sections")
///             .items([
///                 TabItem::new("flights", "Flights").icon(Symbol::FLIGHT),
///                 TabItem::new("hotels", "Hotels").icon(Symbol::HOTEL),
///                 TabItem::new("cars", "Cars").badge(NavBadge::Small),
///             ])
///             .selected(self.tab.clone())
///             .on_change(cx.listener(|this, value: &SharedString, _, cx| {
///                 this.tab = value.clone();
///                 cx.notify();
///             }))
///     }
/// }
/// ```
#[derive(IntoElement)]
pub struct MaterialTabs {
    id: ElementId,
    label: SharedString,
    variant: TabsVariant,
    scrollable: bool,
    auto_activate: bool,
    divider: bool,
    items: Vec<TabItem>,
    selected: Option<SharedString>,
    on_change: Option<Change>,
    container: ColorRole,
}

impl MaterialTabs {
    /// `label` is the tab list's accessible name.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: TabsVariant::Primary,
            scrollable: false,
            auto_activate: false,
            divider: true,
            items: Vec::new(),
            selected: None,
            on_change: None,
            container: ColorRole::SURFACE,
        }
    }
    /// The tab bar's container (default: surface, the kit's). Tabs set in a
    /// surface of another role take that role, so that they blend into it.
    pub fn container(mut self, container: ColorRole) -> Self {
        self.container = container;
        self
    }
    pub fn variant(mut self, variant: TabsVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Content-width tabs that scroll, for many or long tabs.
    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }
    /// Select a tab as soon as arrow keys focus it (otherwise Enter or Space
    /// selects the focused tab).
    pub fn auto_activate(mut self, auto: bool) -> Self {
        self.auto_activate = auto;
        self
    }
    /// The full-width outline-variant divider under the tabs (on by
    /// default); the indicator draws over it.
    pub fn divider(mut self, divider: bool) -> Self {
        self.divider = divider;
        self
    }
    /// The tabs. Values must be unique.
    pub fn items(mut self, items: impl IntoIterator<Item = TabItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }
    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.selected = Some(value.into());
        self
    }
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

/// The M3 leading inset of scrollable tabs.
const SCROLL_INSET: f32 = 52.;
const TAB_PADDING: f32 = 16.;

/// A side of a tab: the kit's padding, which grows to center the content in a
/// wide tab and gives way before the label does in a narrow one, so that a label
/// that fits the tab is never cut off. A scrollable tab has no share of a width
/// to fit into: it is as wide as its content and the whole padding, which a flex
/// basis alone adds nothing to.
fn tab_padding(scrollable: bool) -> Div {
    if scrollable {
        return div().w(rem(TAB_PADDING)).flex_shrink_0();
    }
    div()
        .flex_basis(rem(TAB_PADDING))
        .flex_grow(1.)
        .flex_shrink(1000.)
}

/// Container height: icon-over-label primary tabs are 64dp, all others 48dp.
pub(crate) fn tabs_height(variant: TabsVariant, items: &[TabItem]) -> f32 {
    let stacked = variant == TabsVariant::Primary
        && items.iter().any(|item| item.icon.is_some())
        && items.iter().any(|item| item.show_label);
    if stacked { 64. } else { 48. }
}

/// Indicator thickness: 3dp primary, 2dp secondary.
pub(crate) const fn indicator_height(variant: TabsVariant) -> f32 {
    match variant {
        TabsVariant::Primary => 3.,
        TabsVariant::Secondary => 2.,
    }
}

/// The resolved paint of one tab for the current motion frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TabPaint {
    pub content: Rgba,
    pub hover: Rgba,
    pub press: Rgba,
}

pub(crate) fn tab_paint(
    variant: TabsVariant,
    selected: bool,
    fade: f32,
    disabled: bool,
) -> TabPaint {
    if disabled {
        return TabPaint {
            content: state::disabled(color(ColorRole::ON_SURFACE)),
            hover: Rgba::default(),
            press: Rgba::default(),
        };
    }
    let active = match variant {
        TabsVariant::Primary => color(ColorRole::PRIMARY),
        TabsVariant::Secondary => color(ColorRole::ON_SURFACE),
    };
    let layer = if variant == TabsVariant::Primary && selected {
        color(ColorRole::PRIMARY)
    } else {
        color(ColorRole::ON_SURFACE)
    };
    TabPaint {
        content: blend(
            color(ColorRole::ON_SURFACE_VARIANT),
            active,
            fade.clamp(0., 1.),
        ),
        hover: layer.opacity(state::HOVER),
        press: layer.opacity(state::PRESS),
    }
}

/// Whether measured spans differ by more than rounding. The indicator drawn from
/// a measurement can move a tab's rounded edge by a fraction of a pixel, and a
/// redraw for that would flip it back: a loop of frames, at some scales forever.
fn spans_moved(old: &[(f32, f32)], new: &[(f32, f32)]) -> bool {
    old.len() != new.len()
        || old.iter().zip(new).any(|(old, new)| {
            (old.0 - new.0).abs() > SPAN_TOLERANCE || (old.1 - new.1).abs() > SPAN_TOLERANCE
        })
}
const SPAN_TOLERANCE: f32 = 1.;

/// Tab and content spans, in px from the first tab's leading edge, measured
/// after layout. The indicator follows these on the next frame.
#[derive(Default)]
struct TabGeometry {
    tabs: Vec<(f32, f32)>,
    contents: Vec<(f32, f32)>,
    /// Absolute content spans written by this frame's probes.
    probed: Vec<(f32, f32)>,
    /// Each tab's bounds, written by this frame's probes.
    slots: Vec<Bounds<Pixels>>,
    /// Where the sliding indicator is.
    indicator: motion::TokenTransitionState<Bounds<Pixels>>,
}

/// The indicator's target for a tab's span along the row.
fn indicator_target((left, width): (f32, f32), inset: Pixels) -> Bounds<Pixels> {
    Bounds::new(point(inset + px(left), px(0.)), size(px(width), px(0.)))
}

#[cfg(test)]
thread_local! {
    pub(crate) static INDICATOR: std::cell::Cell<Option<(f32, f32)>> =
        const { std::cell::Cell::new(None) };
}

impl RenderOnce for MaterialTabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let variant = self.variant;
        let primary = variant == TabsVariant::Primary;
        let count = self.items.len();
        let height = tabs_height(variant, &self.items);
        let stacked = height > 48.;
        let scrollable = self.scrollable;

        let handles_state = window.use_keyed_state((self.id.clone(), "tab-focus"), cx, |_, _| {
            BTreeMap::<SharedString, FocusHandle>::new()
        });
        roving::keep_focus_in_handles(&handles_state, &self.items, |item| &item.value, window, cx);
        // Only read while the tabs stay the same: a write as the row is built marks what
        // read this state as changed.
        let known = {
            let handles = handles_state.read(cx);
            (handles.len() == count)
                .then(|| {
                    self.items
                        .iter()
                        .map(|item| handles.get(&item.value).cloned())
                        .collect::<Option<Rc<[FocusHandle]>>>()
                })
                .flatten()
        };
        let handles: Rc<[FocusHandle]> = known.unwrap_or_else(|| {
            handles_state
                .update(cx, |handles, cx| {
                    roving::sync_handles(
                        handles,
                        &self.items,
                        |item| &item.value,
                        || cx.focus_handle(),
                    )
                })
                .into()
        });
        let scroll = window
            .use_keyed_state((self.id.clone(), "tab-scroll"), cx, |_, _| {
                ScrollHandle::new()
            })
            .read(cx)
            .clone();
        // What is measured as the row prepaints is kept beside the entity, not written to it:
        // a write while drawing reads as a change to whatever views hold this row, and the
        // row notifies, below, only when what it measured moved something on screen.
        let geometry_state =
            window.use_keyed_state((self.id.clone(), "tab-geometry"), cx, |_, _| {
                Rc::new(RefCell::new(TabGeometry::default()))
            });
        let geometry = geometry_state.read(cx).clone();
        // Each tab's probe overwrites its slot in prepaint; only the row's size
        // changes it here, so a settled row is not written while drawing.
        if geometry.borrow().probed.len() != count {
            let mut geometry = geometry.borrow_mut();
            geometry.probed.resize(count, (0., 0.));
            geometry.slots.resize(count, Bounds::default());
        }

        let selected = self
            .items
            .iter()
            .position(|item| Some(&item.value) == self.selected.as_ref());
        let specimen = self.items.iter().any(|item| item.preview.is_some());
        let skip: Rc<[bool]> = self.items.iter().map(TabItem::is_disabled).collect();
        let select = self
            .on_change
            .clone()
            .filter(|_| self.auto_activate)
            .map(|on_change| {
                roving::select_value(self.items.iter().map(|item| item.value.clone()), on_change)
            });
        let roving = Roving::new(handles.clone(), Orientation::Row)
            .skip(skip)
            .selected(selected)
            .select_on_focus(select)
            .reveal(scrollable.then(|| scroll.clone()));
        let entry = if specimen {
            None
        } else {
            roving.tab_stop(window)
        };

        // The sliding indicator, once this tab row has been measured.
        let measured = {
            let geometry = geometry.borrow();
            selected
                .filter(|_| geometry.tabs.len() == count && geometry.contents.len() == count)
                .map(|index| {
                    if primary {
                        geometry.contents[index]
                    } else {
                        geometry.tabs[index]
                    }
                })
        };
        let rem_size = window.rem_size();
        let inset = if scrollable {
            rem(SCROLL_INSET).to_pixels(rem_size)
        } else {
            px(0.)
        };
        let thickness = rem(indicator_height(variant));
        let indicator_bar = move || {
            div()
                .absolute()
                .bottom_0()
                .h(thickness)
                .bg(color(ColorRole::PRIMARY))
                .when(primary, |bar| bar.rounded_t(thickness))
        };
        let sliding = measured.map(|(left, width)| {
            let indicator = geometry.borrow().indicator.clone();
            let at = indicator.follow(
                indicator_target((left, width), inset),
                MotionToken::DefaultSpatial,
                window,
                cx,
            );
            #[cfg(test)]
            INDICATOR
                .with(|cell| cell.set(Some((f32::from(at.origin.x), f32::from(at.size.width)))));
            indicator_bar().left(at.origin.x).w(at.size.width)
        });
        let fallback = sliding.is_none();

        let mut lane = div()
            .id((self.id.clone(), "tab-lane"))
            .relative()
            .w_full()
            .h_full()
            .flex()
            .flex_row()
            .when(scrollable, |lane| {
                lane.overflow_x_scroll()
                    .track_scroll(&scroll)
                    .pl(rem(SCROLL_INSET))
            });

        for (index, item) in self.items.into_iter().enumerate() {
            let is_selected = selected == Some(index);
            let preview = item.preview;
            let disabled = item.is_disabled();
            let enabled = !disabled;
            let tab_id = ElementId::NamedChild(self.id.clone().into(), item.value.clone());
            let fade = motion::token_value(
                (tab_id.clone(), "tab-fade"),
                if is_selected { 1. } else { 0. },
                MotionToken::FastEffects,
                window,
                cx,
            );
            let paint = tab_paint(variant, is_selected, fade, disabled);
            let focus = handles[index]
                .clone()
                .tab_stop(entry == Some(index) && enabled);

            let label = item.show_label.then(|| {
                div()
                    .when(cfg!(test), |label| {
                        label.debug_selector(move || format!("tab-label-{index}"))
                    })
                    .min_w_0()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .text_color(paint.content)
                    .map(|label| type_role(label, TypeRole::TitleSmall))
                    .child(item.label.clone())
            });
            let icon = item
                .icon
                .clone()
                .map(|icon| badged_icon(icon.color(paint.content), &item.badge));
            let inline_badge = match (&item.badge, item.icon.is_some()) {
                (NavBadge::Small, false) => Some(small_badge().into_any_element()),
                (NavBadge::Large(text), false) => {
                    Some(large_badge("tab-badge", text.clone()).into_any_element())
                }
                _ => None,
            };
            let probe_geometry = geometry.clone();
            let probe = canvas(
                move |bounds, _, _| {
                    let probed = (f32::from(bounds.left()), f32::from(bounds.size.width));
                    if let Some(slot) = probe_geometry.borrow_mut().probed.get_mut(index) {
                        *slot = probed;
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0();
            let content = div()
                .relative()
                .h_full()
                .min_w_0()
                .flex()
                .items_center()
                .justify_center()
                .map(|content| {
                    if stacked {
                        content.flex_col().gap(rem(2.))
                    } else {
                        content.flex_row().gap(rem(8.))
                    }
                })
                .children(icon)
                .children(label)
                .children(inline_badge)
                .child(probe)
                .when(fallback && primary && is_selected, |content| {
                    content.child(indicator_bar().left_0().right_0())
                });

            let click_focus = focus.clone();
            let on_change = self.on_change.clone();
            let value = item.value.clone();
            let reveal = scrollable.then(|| scroll.clone());
            let tab = gpui_base::Tab::new(tab_id.clone())
                .selected(is_selected)
                .disabled(disabled)
                .accessibility_label(item.label.clone())
                .set_position(index + 1, count)
                .track_focus(&focus)
                .relative()
                .h_full()
                .flex()
                .flex_row()
                .map(|tab| {
                    if scrollable {
                        tab.flex_shrink_0()
                    } else {
                        tab.flex_1().min_w_0()
                    }
                })
                .map(|tab| match preview {
                    _ if disabled => tab,
                    None => tab
                        .cursor_pointer()
                        .hover(move |style| style.bg(paint.hover))
                        .active(move |style| style.bg(paint.press)),
                    Some(InteractionPreview::Hovered) => tab.bg(paint.hover),
                    Some(InteractionPreview::Pressed) => tab.bg(paint.press),
                    Some(_) => tab,
                })
                .when(cfg!(test), |tab| {
                    tab.debug_selector(move || format!("material-tab-{index}"))
                })
                .child(tab_padding(scrollable))
                .child(content)
                .child(tab_padding(scrollable))
                .when(fallback && !primary && is_selected, |tab| {
                    tab.child(indicator_bar().left_0().right_0())
                })
                .when(preview.is_none(), |tab| {
                    tab.on_click(move |_, window, cx| {
                        click_focus.focus(window, cx);
                        if let Some(scroll) = &reveal {
                            scroll.scroll_to_item(index);
                        }
                        if !is_selected && let Some(on_change) = &on_change {
                            on_change(&value, window, cx);
                        }
                    })
                });
            let ink = if primary && is_selected {
                color(ColorRole::PRIMARY)
            } else {
                color(ColorRole::ON_SURFACE)
            };
            let ripple = ButtonRipple::new(
                tab,
                (tab_id, "tab-ripple").into(),
                preview.is_none(),
                enabled,
                ripple_corners(px(0.)),
                Edges::all(px(0.).into()),
                ink,
                window,
                cx,
            );
            let ring = ButtonFocusRing::with_corners(
                ripple,
                focus,
                enabled,
                Corners::all(rem(8.).to_pixels(rem_size)),
            )
            .inward()
            .forced(preview == Some(InteractionPreview::Focused));
            let slot_geometry = geometry.clone();
            lane = lane.child(super::prepaint_bounds(
                div()
                    .h_full()
                    .flex()
                    .map(|slot| {
                        if scrollable {
                            slot.flex_shrink_0()
                        } else {
                            slot.flex_1().min_w_0()
                        }
                    })
                    .child(ring),
                move |bounds, _, _| {
                    if let Some(slot) = slot_geometry.borrow_mut().slots.get_mut(index) {
                        *slot = bounds;
                    }
                },
            ));
        }
        let lane = {
            let geometry = geometry.clone();
            let geometry_state = geometry_state.clone();
            // Once every tab has reported, as the lane finishes prepainting.
            super::prepaint_bounds(lane.children(sliding), move |_, window, cx| {
                // Offsets from the first tab.
                let mut current = geometry.borrow_mut();
                let bounds = &current.slots;
                let Some(first) = bounds.first().map(|b| f32::from(b.left())) else {
                    return;
                };
                let tabs: Vec<(f32, f32)> = bounds
                    .iter()
                    .take(count)
                    .map(|b| (f32::from(b.left()) - first, f32::from(b.size.width)))
                    .collect();
                let contents: Vec<(f32, f32)> = current
                    .probed
                    .iter()
                    .map(|(left, width)| (*left - first, *width))
                    .collect();
                let first = current.tabs.is_empty();
                let moved = !first
                    && (spans_moved(&current.tabs, &tabs)
                        || spans_moved(&current.contents, &contents));
                // The first measure changes nothing on screen: the row drew its indicator
                // under the selected tab. So the indicator only starts from there, and the
                // view holding the row, a list of many, is not built again for it.
                if first && let Some(index) = selected {
                    let span = if primary {
                        contents.get(index)
                    } else {
                        tabs.get(index)
                    };
                    if let Some(span) = span {
                        current.indicator.sample(
                            indicator_target(*span, inset),
                            MotionToken::DefaultSpatial,
                            window,
                            cx,
                        );
                    }
                }
                current.tabs = tabs;
                current.contents = contents;
                // Draw the indicator at the measured place: the state's
                // notification reaches the view holding this row, not the whole
                // window. Only a real move does: layout rounding alone would
                // redraw forever (see `spans_moved`).
                if moved {
                    let measured = geometry_state.clone();
                    window.defer(cx, move |_, cx| {
                        measured.update(cx, |_, cx| cx.notify());
                    });
                }
            })
        };
        let list = gpui_base::Tabs::new(self.id)
            .aria_label(self.label)
            .relative()
            .w_full()
            .h(rem(height))
            .bg(color(self.container))
            // The full-width divider under the tabs; the indicator covers it.
            .when(self.divider, |list| {
                list.child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .h(px(1.))
                        .bg(color(ColorRole::OUTLINE_VARIANT))
                        .when(cfg!(test), |line| {
                            line.debug_selector(|| "tabs-divider".into())
                        }),
                )
            })
            .child(lane);
        roving.install(list)
    }
}

#[cfg(test)]
#[path = "tabs_tests.rs"]
mod tests;
