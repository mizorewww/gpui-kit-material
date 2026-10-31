//! M3 connected button group with roving keyboard focus: a required single
//! selection (radio or tab semantics), an optional single or multiple
//! selection (toggle buttons), or plain actions.
//!
//! Each option is its own button 2px from its neighbours; a selected one
//! morphs to a full pill on the accent while its inner corners open, on the
//! fast spatial spring. Groups lay out horizontally or vertically, with
//! density steps, equal widths, and labels shown always, only on selected
//! options (revealed with the selection) or never (icon options).
use super::{ButtonDensity, ButtonShape, ButtonSize, ButtonVariant, InteractionPreview};
use crate::color::ColorRole;
use crate::icons::Icon;
use crate::theme::{blend, color};
use crate::tokens::state;
use crate::tokens::typography::{TypeRole, rem};
use gpui::prelude::FluentBuilder as _;
use gpui::*;
use std::{collections::BTreeMap, rc::Rc};

use super::roving::{Change, Orientation, Roving, select_value};

/// How a connected group selects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GroupSelection {
    /// Plain actions: no option stays selected.
    None,
    /// At most one option (exactly one with [`ConnectedButtonGroup::required`]).
    #[default]
    Single,
    /// Any number of options.
    Multi,
}

/// When an option with an icon shows its label.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GroupLabels {
    #[default]
    Always,
    /// Only while selected, revealed and hidden with the selection.
    Selected,
    /// Never: icon-only options (still labelled for assistive technology).
    Never,
}

type ToggleHandler = Rc<dyn Fn(&SharedString, bool, &mut Window, &mut App)>;
type ActivateHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

#[derive(Clone)]
struct OptionItem {
    value: SharedString,
    label: SharedString,
}

/// `options` without repeated values, keeping each value's first occurrence,
/// and whether there were none to drop.
fn unique_options(options: impl IntoIterator<Item = OptionItem>) -> (Vec<OptionItem>, bool) {
    let mut seen = std::collections::BTreeSet::new();
    let mut unique = true;
    let options = options
        .into_iter()
        .filter(|option| {
            let first = seen.insert(option.value.clone());
            unique &= first;
            first
        })
        .collect();
    (options, unique)
}

#[derive(gpui::IntoElement)]
pub struct ConnectedButtonGroup {
    id: ElementId,
    label: SharedString,
    value: Option<SharedString>,
    options: Vec<OptionItem>,
    icons: BTreeMap<SharedString, Icon>,
    full_width: bool,
    segmented: bool,
    label_text_size: Option<Rems>,
    height: Option<Rems>,
    tabs: bool,
    provided_focus: Vec<FocusHandle>,
    disabled: bool,
    on_change: Option<Change>,
    reveal_in: Option<ScrollHandle>,
    previews: BTreeMap<SharedString, InteractionPreview>,
    variant: ButtonVariant,
    size: Option<ButtonSize>,
    shape: ButtonShape,
    selection: GroupSelection,
    required: bool,
    values: Vec<SharedString>,
    on_toggle: Option<ToggleHandler>,
    on_activate: Option<ActivateHandler>,
    vertical: bool,
    density: ButtonDensity,
    equal_widths: bool,
    labels: GroupLabels,
}
impl ConnectedButtonGroup {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: None,
            options: Vec::new(),
            icons: BTreeMap::new(),
            full_width: false,
            segmented: false,
            label_text_size: None,
            height: None,
            tabs: false,
            provided_focus: Vec::new(),
            disabled: false,
            on_change: None,
            reveal_in: None,
            previews: BTreeMap::new(),
            variant: ButtonVariant::Tonal,
            size: None,
            shape: ButtonShape::Round,
            selection: GroupSelection::Single,
            required: true,
            values: Vec::new(),
            on_toggle: None,
            on_activate: None,
            vertical: false,
            density: ButtonDensity::Default,
            equal_widths: false,
            labels: GroupLabels::Always,
        }
    }
    /// Single (default), multiple or no selection.
    pub fn selection(mut self, selection: GroupSelection) -> Self {
        self.selection = selection;
        self
    }
    /// Whether a single selection must keep one option selected (default,
    /// radio semantics). Optional selections are toggle buttons whose
    /// changes arrive through `on_toggle`.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
    /// The selected values of a multiple or optional selection.
    pub fn selected_values<V: Into<SharedString>>(
        mut self,
        values: impl IntoIterator<Item = V>,
    ) -> Self {
        self.values = values.into_iter().map(Into::into).collect();
        self
    }
    /// The requested selection of one option of a toggle group (multiple,
    /// or an optional single selection).
    pub fn on_toggle(
        mut self,
        callback: impl Fn(&SharedString, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Rc::new(callback));
        self
    }
    /// An activated option of a group without selection.
    pub fn on_activate(
        mut self,
        callback: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(Rc::new(callback));
        self
    }
    /// Stack the options vertically (connected top to bottom).
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }
    /// Shorten the options by the density's step.
    pub fn density(mut self, density: ButtonDensity) -> Self {
        self.density = density;
        self
    }
    /// Give every option the widest one's width (horizontal groups that
    /// fit their content; vertical options always share the column).
    pub fn equal_widths(mut self, equal: bool) -> Self {
        self.equal_widths = equal;
        self
    }
    /// When options with icons show their labels.
    pub fn labels(mut self, labels: GroupLabels) -> Self {
        self.labels = labels;
        self
    }
    /// The toggle-button colors the options use: tonal by default (the
    /// kit's connected group), or filled, elevated or outlined.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    /// An M3 Expressive button size (XS–XL): the options take that button's
    /// height, padding, label type and icon, and the size's inner corners.
    /// Without it the group keeps its compact 40px default.
    pub fn button_size(mut self, size: ButtonSize) -> Self {
        self.size = Some(size);
        self
    }
    /// Round (default) or square ends: square ends of unselected options
    /// take the inner corner radius. The selected option is a full pill.
    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }
    /// The options as `(value, label)` pairs. Values identify an option's
    /// focus and selection state, so they must be unique: a repeated value
    /// is a caller bug (a debug assertion), and release builds keep only its
    /// first occurrence.
    pub fn options<K: Into<SharedString>, V: Into<SharedString>>(
        mut self,
        options: impl IntoIterator<Item = (K, V)>,
    ) -> Self {
        let (options, unique) =
            unique_options(options.into_iter().map(|(value, label)| OptionItem {
                value: value.into(),
                label: label.into(),
            }));
        debug_assert!(unique, "ConnectedButtonGroup option values must be unique");
        self.options = options;
        self
    }
    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }
    /// Fill the parent's width in equal options, each at least as wide as its
    /// label; options wrap to another row only when their labels cannot fit.
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }
    /// Connected segments that fill the parent, with persistent option icons.
    /// Radio/tab semantics, focus navigation and pointer feedback are unchanged.
    pub fn segmented(mut self) -> Self {
        self.segmented = true;
        self.full_width = true;
        self
    }
    /// Override option text for a page-specific typography role.
    /// Override the option height (48px segmented, 40px otherwise), e.g. for
    /// a page's view tabs.
    pub fn height(mut self, height: Rems) -> Self {
        self.height = Some(height);
        self
    }
    pub fn label_text_size(mut self, size: Rems) -> Self {
        self.label_text_size = Some(size);
        self
    }
    /// Navigation uses the same surfaces, with tab semantics and optional leading layout.
    pub fn tabs(mut self) -> Self {
        self.tabs = true;
        self
    }
    pub fn focus_handles(mut self, handles: impl IntoIterator<Item = FocusHandle>) -> Self {
        self.provided_focus = handles.into_iter().collect();
        self
    }
    /// Attach an icon by option value; this may be called before or after `options`.
    pub fn icon(mut self, value: impl Into<SharedString>, icon: impl Into<Icon>) -> Self {
        self.icons.insert(value.into(), icon.into());
        self
    }
    pub fn reveal_in(mut self, scroll: ScrollHandle) -> Self {
        self.reveal_in = Some(scroll);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Render the option with `value` as a static specimen in one forced
    /// interaction state, painting what that live state paints (its hover or
    /// press layer, focus ring or disabled colors). A group with any preview
    /// is a specimen: it leaves the tab order, and its other options ignore
    /// live hover and press. See [`InteractionPreview`].
    pub fn preview_state(
        mut self,
        value: impl Into<SharedString>,
        state: InteractionPreview,
    ) -> Self {
        self.previews.insert(value.into(), state);
        self
    }
    pub fn on_change(
        mut self,
        callback: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(callback));
        self
    }
}

/// The resolved paint of one option for the current motion frame.
#[derive(Clone, Copy, Debug, PartialEq)]
struct OptionPaint {
    background: Rgba,
    foreground: Rgba,
    hover: Rgba,
    press: Rgba,
    left_radius: Pixels,
    right_radius: Pixels,
}

/// Inner corners between neighbours of the compact default group.
const INNER_RADIUS: f32 = 8.;
const GAP: f32 = 2.;

/// The size-dependent geometry of one group, in design px.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Metrics {
    height: f32,
    /// Corners between neighbours.
    inner: f32,
    /// Unselected outer ends: `None` is a full half-height round.
    outer: Option<f32>,
    /// Horizontal padding of filling options, inside the 2px border.
    padding: f32,
    /// The same of content-sized options.
    content_padding: f32,
    icon: f32,
    /// Between an option's icon and label.
    icon_gap: f32,
    text: Option<TypeRole>,
    /// Options stack top to bottom.
    vertical: bool,
}

impl Metrics {
    /// M3 Expressive connected group (Figma kit): each option is that
    /// size's button, padding and icon gap included, so with the border an
    /// option's content sits where the button's does; inner corners are
    /// XS 4, S 8, M 8, L 16, XL 20.
    fn of(size: ButtonSize, shape: ButtonShape) -> Self {
        let (height, inner, padding, icon, icon_gap, text) = match size {
            ButtonSize::ExtraSmall => (32., 4., 12., 20., 4., TypeRole::LabelLarge),
            ButtonSize::Small => (40., 8., 16., 20., 8., TypeRole::LabelLarge),
            ButtonSize::Medium => (56., 8., 24., 24., 8., TypeRole::TitleMedium),
            ButtonSize::Large => (96., 16., 48., 32., 12., TypeRole::HeadlineSmall),
            ButtonSize::ExtraLarge => (136., 20., 64., 40., 16., TypeRole::HeadlineLarge),
        };
        Self {
            height,
            inner,
            // Square groups round their ends like their inner corners.
            outer: (shape == ButtonShape::Square).then_some(inner),
            padding: padding - 2.,
            content_padding: padding - 2.,
            icon,
            icon_gap,
            text: Some(text),
            vertical: false,
        }
    }
    fn compact(height: f32) -> Self {
        Self {
            height,
            inner: INNER_RADIUS,
            outer: None,
            padding: 12.,
            content_padding: 14.,
            icon: 18.,
            icon_gap: 8.,
            text: None,
            vertical: false,
        }
    }
}

/// M3 connected button group: each option is its own button. `selection` is the
/// option's animated selection amount (0 unselected, 1 selected), which morphs
/// both colour and the inner corners toward a full pill.
#[cfg(test)]
fn option_paint(
    selection: f32,
    index: usize,
    count: usize,
    height: Pixels,
    disabled: bool,
) -> OptionPaint {
    option_paint_with(
        selection,
        index,
        count,
        height,
        disabled,
        ButtonVariant::Tonal,
        Metrics::compact(f32::from(height)),
    )
}

fn option_paint_with(
    selection: f32,
    index: usize,
    count: usize,
    height: Pixels,
    disabled: bool,
    variant: ButtonVariant,
    metrics: Metrics,
) -> OptionPaint {
    let t = selection.clamp(0., 1.);
    let full = height / 2.;
    let scale = height / px(metrics.height);
    let morph = |rest: Pixels| rest + (full - rest) * t;
    let inner = morph(px(metrics.inner) * scale);
    let outer = morph(metrics.outer.map_or(full, |outer| px(outer) * scale));
    let left_radius = if index == 0 { outer } else { inner };
    let right_radius = if index + 1 == count { outer } else { inner };
    if disabled {
        // M3 disabled: on-surface at 12% container and 38% content. The pill
        // shape still tells which option is current.
        let ink = color(ColorRole::ON_SURFACE);
        return OptionPaint {
            background: ink.opacity(state::DISABLED_CONTAINER),
            foreground: state::disabled(ink),
            hover: ink.opacity(state::DISABLED_CONTAINER),
            press: ink.opacity(state::DISABLED_CONTAINER),
            left_radius,
            right_radius,
        };
    }
    // The toggle button's own unselected and selected colors.
    let (rest_bg, rest_fg) = super::button::toggle_colors(variant, false);
    let (on_bg, on_fg) = super::button::toggle_colors(variant, true);
    let background = blend(rest_bg, on_bg, t);
    let foreground = blend(rest_fg, on_fg, t);
    OptionPaint {
        background,
        foreground,
        // One state layer in the content colour, resolved over the current
        // container instead of stacking translucent fills, so the selected
        // accent stays under hover and press.
        hover: state::hovered(background, foreground),
        press: state::pressed(background, foreground),
        left_radius,
        right_radius,
    }
}

#[cfg(test)]
thread_local! {
    static PAINTED: std::cell::RefCell<BTreeMap<String, OptionPaint>> =
        const { std::cell::RefCell::new(BTreeMap::new()) };
}

/// Containers stay behind the interactive item. Hover/press cannot erase selection.
fn item_style<T: Styled + StatefulInteractiveElement + gpui::prelude::FluentBuilder>(
    item: T,
    paint: OptionPaint,
    height: Pixels,
    fill: bool,
    disabled: bool,
    preview: Option<InteractionPreview>,
    metrics: Metrics,
) -> T {
    item.relative()
        .flex()
        .items_center()
        .justify_center()
        // Content-width options may shrink and ellipsize; filling options keep
        // their automatic minimum, the width of their whole label.
        .when(!fill, |item| item.min_w_0())
        .h(height)
        .min_h(height)
        // With the 2px focus border: 16px to the content of a content-sized
        // option; 14px in equal shares. That is also the floor for a label
        // wider than its share, so a long label at a large text scale never
        // meets the pill's edge: the options wrap to another row first.
        .px(rem(if fill {
            metrics.padding
        } else {
            metrics.content_padding
        }))
        .py_0()
        .map(|item| {
            // The first option's outer corners lead the main axis.
            if metrics.vertical {
                item.rounded_t(paint.left_radius)
                    .rounded_b(paint.right_radius)
            } else {
                item.rounded_l(paint.left_radius)
                    .rounded_r(paint.right_radius)
            }
        })
        // A transparent 2px border keeps the content inset; the focus ring is
        // the shared one, drawn by `focus_ring`.
        .border_2()
        .border_color(gpui::transparent_black())
        .bg(paint.background)
        .text_color(paint.foreground)
        .map(|item| match metrics.text {
            Some(role) => {
                let scale = role.scale();
                item.font_family(scale.font_family)
                    .text_size(scale.font_size)
                    .letter_spacing(scale.letter_spacing)
                    .line_height(scale.line_height)
                    .font_weight(FontWeight(scale.font_weight as f32))
            }
            None => item
                .text_size(crate::tokens::typography::TEXT_BODY)
                .font_weight(FontWeight::MEDIUM),
        })
        // Content-width options keep the default shrink so a narrow parent
        // ellipsizes labels instead of overflowing. Filling options are equal
        // shares of the row (a zero basis), floored at their label's own
        // width: a long label ("Speech recognition") takes what it needs and
        // never truncates, and the others split the rest equally.
        .when(fill, |item| item.flex_1())
        .map(|item| match preview {
            _ if disabled => item,
            None => item
                .hover(move |style| style.bg(paint.hover))
                .active(move |style| style.bg(paint.press)),
            Some(InteractionPreview::Hovered) => item.bg(paint.hover),
            Some(InteractionPreview::Pressed) => item.bg(paint.press),
            Some(_) => item,
        })
}
/// The shared M3 keyboard focus ring around one option. Options sit 2dp
/// apart, so the ring uses the M3 inner offset: an outer ring would be
/// covered by the next option.
fn focus_ring(
    option: impl IntoElement,
    focus: &FocusHandle,
    disabled: bool,
    forced: bool,
    paint: OptionPaint,
    vertical: bool,
) -> super::button::ButtonFocusRing {
    let (start, end) = (paint.left_radius, paint.right_radius);
    super::button::ButtonFocusRing::with_corners(
        option,
        focus.clone(),
        !disabled,
        if vertical {
            Corners {
                top_left: start,
                top_right: start,
                bottom_right: end,
                bottom_left: end,
            }
        } else {
            Corners {
                top_left: start,
                top_right: end,
                bottom_right: end,
                bottom_left: start,
            }
        },
    )
    .inward()
    .forced(forced)
}

/// An option of an equal-width group: at least `widest` wide, reporting its
/// own width. Other options pass through unwrapped.
fn equal_slot(
    option: impl IntoElement,
    width: &Rc<std::cell::Cell<f32>>,
    equal: bool,
    widest: f32,
) -> AnyElement {
    if !equal {
        return option.into_any_element();
    }
    let width = width.clone();
    div()
        .relative()
        .flex()
        // A column stretches the option across the slot.
        .flex_col()
        .items_stretch()
        .flex_shrink_0()
        .when(widest > 0., |slot| slot.min_w(px(widest)))
        .child(option)
        .child(
            canvas(
                move |bounds, window, _| {
                    let measured = f32::from(bounds.size.width);
                    if (width.get() - measured).abs() > 0.5 {
                        width.set(measured);
                        // The other options take the new width next frame.
                        window.request_animation_frame();
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_full(),
        )
        .into_any_element()
}

/// The press wave over an option, clipped to its corners along the group's axis.
fn option_ripple(
    option: impl IntoElement,
    index: usize,
    paint: &OptionPaint,
    vertical: bool,
) -> super::Ripple {
    let (start, end) = (paint.left_radius, paint.right_radius);
    super::Ripple::new(
        ElementId::NamedInteger("option-ripple".into(), index as u64),
        option,
    )
    .corners(if vertical {
        Corners {
            top_left: start,
            top_right: start,
            bottom_right: end,
            bottom_left: end,
        }
    } else {
        Corners {
            top_left: start,
            top_right: end,
            bottom_right: end,
            bottom_left: start,
        }
    })
    .ink(paint.foreground)
}

/// The focus handle of each option, created on first sight (see
/// [`super::roving::sync_handles`]).
fn sync_handles(
    handles: &mut BTreeMap<SharedString, FocusHandle>,
    options: &[OptionItem],
    create: impl FnMut() -> FocusHandle,
) -> Vec<FocusHandle> {
    super::roving::sync_handles(handles, options, |option| &option.value, create)
}

impl RenderOnce for ConnectedButtonGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.disabled;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| {
            BTreeMap::<SharedString, FocusHandle>::new()
        });
        let handles = if self.provided_focus.len() == self.options.len() {
            self.provided_focus.clone()
        } else {
            state.update(cx, |handles, cx| {
                sync_handles(handles, &self.options, || cx.focus_handle())
            })
        };
        let radio = self.tabs || (self.selection == GroupSelection::Single && self.required);
        let toggles = !radio && self.selection != GroupSelection::None;
        let selected = self
            .options
            .iter()
            .position(|option| Some(&option.value) == self.value.as_ref());
        let is_checked = |index: usize, value: &SharedString| match self.selection {
            _ if radio => selected == Some(index),
            GroupSelection::None => false,
            _ => self.values.contains(value) || self.value.as_ref() == Some(value),
        };
        let specimen = !self.previews.is_empty();
        let roving = Roving::new(handles.clone(), Orientation::Both)
            .rtl(crate::scope::is_rtl(window))
            .disabled(disabled)
            .selected(if radio { selected } else { None })
            .select_on_focus(self.on_change.clone().filter(|_| radio).map(|on_change| {
                select_value(
                    self.options.iter().map(|option| option.value.clone()),
                    on_change,
                )
            }));
        let entry = if specimen {
            None
        } else {
            roving.tab_stop(window)
        };
        let count = self.options.len();
        let mut metrics = match self.size {
            Some(size) => Metrics::of(size, self.shape),
            None => Metrics::compact(if self.segmented { 48. } else { 40. }),
        };
        metrics.height = (metrics.height + self.density.offset()).max(0.);
        let vertical = self.vertical;
        metrics.vertical = vertical;
        let equal = self.equal_widths && !vertical && !self.full_width;
        // A slot measures no less than the widest width it is given, so what
        // makes an option's own width change has to start the measure over.
        let measured_for = equal.then(|| {
            use std::hash::{Hash, Hasher};
            let mut hash = std::collections::hash_map::DefaultHasher::new();
            f32::from(window.rem_size()).to_bits().hash(&mut hash);
            for (index, option) in self.options.iter().enumerate() {
                option.label.hash(&mut hash);
                if self.labels == GroupLabels::Selected {
                    is_checked(index, &option.value).hash(&mut hash);
                }
            }
            hash.finish()
        });
        let widths = window
            .use_keyed_state((self.id.clone(), "equal-widths"), cx, |_, _| {
                (None::<u64>, Vec::<Rc<std::cell::Cell<f32>>>::new())
            })
            .update(cx, |(signature, widths), _| {
                widths.resize_with(self.options.len(), Rc::default);
                if !equal || *signature != measured_for {
                    // Re-measure from natural widths when equal widths return.
                    widths.iter().for_each(|width| width.set(0.));
                }
                *signature = measured_for;
                widths.clone()
            });
        // Equal options share the widest measured width, which holds while
        // the group stays equal.
        let widest = if equal {
            widths.iter().map(|width| width.get()).fold(0., f32::max)
        } else {
            0.
        };
        let item_height = match self.height {
            Some(height) => height.to_pixels(window.rem_size()),
            None => rem(metrics.height).to_pixels(window.rem_size()),
        };
        let fill = self.full_width;
        let icon_size = rem(metrics.icon);
        let variant = self.variant;
        let selection_root: std::sync::Arc<ElementId> =
            std::sync::Arc::new((self.id.clone(), "selection").into());
        let debug_kind = if fill { "full" } else { "content" };
        let mut lane = div()
            .flex()
            .map(|lane| {
                if vertical {
                    lane.flex_col().items_stretch()
                } else {
                    lane.flex_row().items_center().w_full().min_w_0()
                }
            })
            .gap(px(GAP))
            // When even the labels' own widths exceed the parent (large text
            // in a narrow pane), options wrap to another row rather than
            // clipping their labels.
            .when(fill && !vertical, |lane| lane.flex_wrap())
            .when(cfg!(test), |lane| {
                lane.debug_selector(|| "full-choice-lane".into())
            });
        for (index, option) in self.options.into_iter().enumerate() {
            let checked = is_checked(index, &option.value);
            let focus = handles[index].clone().tab_stop(entry == Some(index));
            let callback = self.on_change.clone();
            let value = option.value.clone();
            let label = option.label.clone();
            // Each option animates its own selection amount; a rapid click
            // retargets from the current presentation value.
            let selection = crate::motion::selection_value(
                ElementId::NamedChild(selection_root.clone(), value.clone()),
                if checked { 1. } else { 0. },
                window,
                cx,
            );
            let forced = self.previews.get(&value).copied();
            // A group-level disabled state wins; a forced one disables only
            // its own option.
            let option_disabled = disabled || forced.is_some_and(|p| p.is_disabled());
            // Every option of a specimen ignores live hover and press.
            let preview = forced.or(specimen.then_some(InteractionPreview::Enabled));
            let forced_focus = forced == Some(InteractionPreview::Focused);
            let paint = option_paint_with(
                selection,
                index,
                count,
                item_height,
                option_disabled,
                variant,
                metrics,
            );
            #[cfg(test)]
            PAINTED.with(|painted| {
                painted
                    .borrow_mut()
                    .insert(format!("{debug_kind}-{index}"), paint)
            });
            let content = gpui_base::h_flex()
                .when(!fill, |content| content.max_w_full().min_w_0())
                .gap(rem(metrics.icon_gap))
                .items_center()
                // Text-only options center their label; no empty icon slot.
                .when_some(self.icons.get(&value).cloned(), |content, icon| {
                    content.child(
                        div()
                            .w(icon_size)
                            .h(icon_size)
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(cfg!(test), |slot| {
                                slot.debug_selector(move || {
                                    format!("{debug_kind}-choice-icon-{index}")
                                })
                            })
                            .child(icon.size(icon_size)),
                    )
                })
                .map(|content| {
                    let has_icon = self.icons.contains_key(&value);
                    match self.labels {
                        GroupLabels::Never if has_icon => content,
                        GroupLabels::Selected if has_icon => {
                            let role = metrics.text.unwrap_or(TypeRole::LabelLarge);
                            let gap = metrics.icon_gap * selection.clamp(0., 1.);
                            content.gap(rem(gap)).children(super::fab::revealed_label(
                                &label, role, selection, window,
                            ))
                        }
                        _ => content.child(
                            div()
                                .when(!fill, |label| label.min_w_0().text_ellipsis())
                                .whitespace_nowrap()
                                .when(cfg!(test), |label| {
                                    label.debug_selector(move || {
                                        format!("{debug_kind}-choice-label-{index}")
                                    })
                                })
                                .child(label.clone()),
                        ),
                    }
                });
            if self.tabs {
                let click_focus = focus.clone();
                let tab = item_style(
                    gpui_base::Tab::new(value.clone()),
                    paint,
                    item_height,
                    fill,
                    option_disabled,
                    preview,
                    metrics,
                )
                .when_some(self.label_text_size, |item, size| item.text_size(size))
                .tab_stop(entry == Some(index))
                .selected(checked)
                .disabled(option_disabled)
                .accessibility_label(label)
                .set_position(index + 1, count)
                .track_focus(&focus)
                .child(content)
                .when(cfg!(test), |tab| {
                    tab.debug_selector(move || format!("{debug_kind}-choice-option-{index}"))
                })
                .on_click(move |_, window, cx| {
                    click_focus.focus(window, cx);
                    if !checked && let Some(callback) = &callback {
                        callback(&value, window, cx);
                    }
                });
                let tab = option_ripple(tab, index, &paint, vertical)
                    .disabled(option_disabled)
                    .specimen(preview.is_some());
                let ring = focus_ring(tab, &focus, option_disabled, forced_focus, paint, vertical);
                lane = lane.child(equal_slot(ring, &widths[index], equal, widest));
            } else if toggles || self.selection == GroupSelection::None {
                let click_focus = focus.clone();
                let on_toggle = self.on_toggle.clone();
                let on_activate = self.on_activate.clone();
                let option = if toggles {
                    item_style(
                        gpui_base::Button::new(value.clone()),
                        paint,
                        item_height,
                        fill,
                        option_disabled,
                        preview,
                        metrics,
                    )
                    .when_some(self.label_text_size, |item, size| item.text_size(size))
                    .aria_toggled(if checked {
                        Toggled::True
                    } else {
                        Toggled::False
                    })
                    .tab_stop(entry == Some(index))
                    .disabled(option_disabled)
                    .accessibility_label(label)
                    .track_focus(&focus)
                    .child(content)
                    .when(cfg!(test), |item| {
                        item.debug_selector(move || format!("{debug_kind}-choice-option-{index}"))
                    })
                    .on_click(move |_, window, cx| {
                        click_focus.focus(window, cx);
                        if let Some(callback) = &on_toggle {
                            callback(&value, !checked, window, cx);
                        }
                    })
                    .into_any_element()
                } else {
                    item_style(
                        gpui_base::Button::new(value.clone()),
                        paint,
                        item_height,
                        fill,
                        option_disabled,
                        preview,
                        metrics,
                    )
                    .when_some(self.label_text_size, |item, size| item.text_size(size))
                    .tab_stop(entry == Some(index))
                    .disabled(option_disabled)
                    .accessibility_label(label)
                    .track_focus(&focus)
                    .child(content)
                    .when(cfg!(test), |item| {
                        item.debug_selector(move || format!("{debug_kind}-choice-option-{index}"))
                    })
                    .on_click(move |_, window, cx| {
                        click_focus.focus(window, cx);
                        if let Some(callback) = &on_activate {
                            callback(&value, window, cx);
                        }
                    })
                    .into_any_element()
                };
                let option = option_ripple(option, index, &paint, vertical)
                    .disabled(option_disabled)
                    .specimen(preview.is_some());
                let ring = focus_ring(
                    option,
                    &focus,
                    option_disabled,
                    forced_focus,
                    paint,
                    vertical,
                );
                lane = lane.child(equal_slot(ring, &widths[index], equal, widest));
            } else {
                let click_focus = focus.clone();
                let radio = item_style(
                    gpui_base::Radio::new(value.clone()),
                    paint,
                    item_height,
                    fill,
                    option_disabled,
                    preview,
                    metrics,
                )
                .when_some(self.label_text_size, |item, size| item.text_size(size))
                .checked(checked)
                .tab_stop(entry == Some(index))
                .disabled(option_disabled)
                .accessibility_label(label)
                .set_position(index + 1, count)
                .track_focus(&focus)
                .child(content)
                .when(cfg!(test), |radio| {
                    radio.debug_selector(move || format!("{debug_kind}-choice-option-{index}"))
                })
                .on_change(move |_, _, window, cx| {
                    click_focus.focus(window, cx);
                    if !checked && let Some(callback) = &callback {
                        callback(&value, window, cx);
                    }
                });
                let radio = option_ripple(radio, index, &paint, vertical)
                    .disabled(option_disabled)
                    .specimen(preview.is_some());
                let ring = focus_ring(
                    radio,
                    &focus,
                    option_disabled,
                    forced_focus,
                    paint,
                    vertical,
                );
                lane = lane.child(equal_slot(ring, &widths[index], equal, widest));
            }
        }
        let group = div()
            .id(self.id.clone())
            .role(if self.tabs {
                Role::TabList
            } else if radio {
                Role::RadioGroup
            } else {
                Role::Group
            })
            .aria_label(self.label)
            .flex()
            .min_w_0()
            .max_w_full()
            .when(fill, |group| group.w_full())
            .when(cfg!(test), |group| {
                group.debug_selector(|| "full-choice-track".into())
            })
            .child(lane);
        let group = roving.install(group);
        if let Some(scroll) = self.reveal_in {
            let reveal_id = ElementId::from((self.id.clone(), "choice-reveal"));
            let reveal = super::RevealFocus::new(reveal_id, group, scroll);
            if fill { reveal } else { reveal.inline() }.into_any_element()
        } else {
            group.into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ConnectedButtonGroup, OptionItem, OptionPaint, PAINTED, option_paint, state, sync_handles,
        unique_options,
    };

    #[core::prelude::v1::test]
    fn repeated_option_values_keep_their_first_occurrence() {
        let item = |value: &str, label: &str| OptionItem {
            value: value.into(),
            label: label.into(),
        };
        let (options, unique) =
            unique_options([item("a", "A"), item("b", "B"), item("a", "Again")]);
        assert!(!unique);
        let kept: Vec<_> = options
            .iter()
            .map(|o| (o.value.as_ref(), o.label.as_ref()))
            .collect();
        assert_eq!(kept, [("a", "A"), ("b", "B")]);
        assert!(unique_options([item("a", "A"), item("b", "B")]).1);
    }

    #[gpui::test]
    fn focus_handles_follow_the_current_options(cx: &mut TestAppContext) {
        let item = |value: &str| OptionItem {
            value: value.into(),
            label: value.to_string().into(),
        };
        let mut map = std::collections::BTreeMap::new();
        let first = cx.update(|cx| {
            sync_handles(&mut map, &[item("a"), item("b"), item("c")], || {
                cx.focus_handle()
            })
        });
        assert_eq!(map.len(), 3);
        let second =
            cx.update(|cx| sync_handles(&mut map, &[item("c"), item("a")], || cx.focus_handle()));
        assert_eq!(map.len(), 2, "the removed option's handle is pruned");
        assert!(!map.contains_key("b"));
        assert_eq!(
            second,
            [first[2].clone(), first[0].clone()],
            "kept handles survive"
        );
    }

    #[core::prelude::v1::test]
    #[cfg_attr(debug_assertions, should_panic(expected = "must be unique"))]
    fn repeated_option_values_are_rejected_in_debug_builds() {
        let group = ConnectedButtonGroup::new("dupes", "Dupes").options([("a", "A"), ("a", "B")]);
        assert_eq!(group.options.len(), 1);
    }

    #[core::prelude::v1::test]
    fn state_layers_come_from_the_shared_tokens() {
        for selection in [0., 0.5, 1.] {
            let paint = option_paint(selection, 1, 3, px(40.), false);
            assert_eq!(
                paint.hover,
                state::hovered(paint.background, paint.foreground)
            );
            assert_eq!(
                paint.press,
                state::pressed(paint.background, paint.foreground)
            );
        }
        let disabled = option_paint(1., 1, 3, px(40.), true);
        let ink = crate::theme::color(ColorRole::ON_SURFACE);
        assert_eq!(disabled.foreground, state::disabled(ink));
        assert_eq!(disabled.background.a, ink.a * state::DISABLED_CONTAINER);
        assert_eq!(
            disabled.hover, disabled.background,
            "no state layer when disabled"
        );
    }
    use crate::color::ColorRole;
    use crate::icons::IconName;
    use gpui::{
        AppContext, Bounds, Context, FontWeight, IntoElement, Modifiers, ParentElement as _,
        Pixels, Render, SharedString, Styled as _, TestAppContext, TextRun, VisualTestContext,
        Window, div, font, px,
    };
    use std::time::Duration;

    struct RovingHarness {
        handles: [gpui::FocusHandle; 5],
        selected: usize,
        disabled: bool,
        changes: usize,
        /// Without a change callback arrows only move focus.
        manual: bool,
    }

    impl Render for RovingHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            use gpui::InteractiveElement as _;
            div()
                .w(px(600.))
                .tab_group()
                .child(
                    gpui_base::Button::new("before")
                        .track_focus(&self.handles[0])
                        .child("Before"),
                )
                .child({
                    let group = ConnectedButtonGroup::new("roving", "Choices")
                        .options([("0", "First"), ("1", "Second"), ("2", "Third")])
                        .focus_handles(self.handles[1..4].iter().cloned())
                        .selected(self.selected.to_string())
                        .disabled(self.disabled);
                    if self.manual {
                        group
                    } else {
                        group.on_change(cx.listener(|this, value: &SharedString, _, cx| {
                            this.selected = value.parse().unwrap();
                            this.changes += 1;
                            cx.notify();
                        }))
                    }
                })
                .child(
                    gpui_base::Button::new("after")
                        .track_focus(&self.handles[4])
                        .child("After"),
                )
        }
    }

    fn press_roving_key(cx: &mut VisualTestContext, key: &str) {
        let keystroke =
            cx.update(|_, cx| gpui::Keystroke::parse_with_os(key, cx.platform_info().os).unwrap());
        cx.simulate_event(gpui::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(gpui::KeyUpEvent { keystroke });
        draw_choice(cx);
    }

    #[gpui::test]
    fn tab_traversal_uses_only_selected_radio_and_tracks_arrow_changes(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            crate::components::roving::init(cx);
            cx.set_reduce_motion(true);
        });
        for disabled in [false, true] {
            let handles: [gpui::FocusHandle; 5] =
                cx.update(|cx| std::array::from_fn(|_| cx.focus_handle()));
            let controls = cx.new(|_| RovingHarness {
                handles: handles.clone(),
                selected: 1,
                disabled,
                changes: 0,
                manual: false,
            });
            let (_, cx) =
                cx.add_window_view(|window, cx| gpui_base::Root::new(controls.clone(), window, cx));
            cx.update(|window, cx| {
                window.activate_window();
                window.draw(cx).clear(cx);
                handles[0].focus(window, cx);
                window.draw(cx).clear(cx);
            });
            if disabled {
                for (key, expected) in [("tab", 4), ("shift-tab", 0)] {
                    press_roving_key(cx, key);
                    cx.update(|window, cx| {
                        assert_eq!(window.focused(cx).as_ref(), Some(&handles[expected]))
                    });
                }
                cx.update(|_, cx| {
                    assert_eq!(controls.read(cx).selected, 1);
                    assert_eq!(controls.read(cx).changes, 0);
                });
                continue;
            }
            // Enter at the selected middle option, leave the entire group in
            // one Tab, and re-enter at selection when traversing backwards.
            for (key, expected) in [
                ("tab", 2),
                ("tab", 4),
                ("shift-tab", 2),
                ("shift-tab", 0),
                ("tab", 2),
                ("right", 3),
            ] {
                press_roving_key(cx, key);
                cx.update(|window, cx| {
                    assert_eq!(
                        window.focused(cx).as_ref(),
                        Some(&handles[expected]),
                        "{key}"
                    )
                });
            }
            cx.update(|_, cx| {
                assert_eq!(controls.read(cx).selected, 2);
                assert_eq!(controls.read(cx).changes, 1);
            });
            for (key, expected) in [
                ("tab", 4),
                ("shift-tab", 3),
                ("right", 1),
                ("tab", 4),
                ("shift-tab", 1),
            ] {
                press_roving_key(cx, key);
                cx.update(|window, cx| {
                    assert_eq!(
                        window.focused(cx).as_ref(),
                        Some(&handles[expected]),
                        "{key}"
                    )
                });
            }
            cx.update(|_, cx| {
                assert_eq!(controls.read(cx).selected, 0);
                assert_eq!(controls.read(cx).changes, 2);
            });
        }
    }

    #[gpui::test]
    fn the_tab_stop_follows_focus_without_a_change_callback(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            crate::components::roving::init(cx);
            cx.set_reduce_motion(true);
        });
        let handles: [gpui::FocusHandle; 5] =
            cx.update(|cx| std::array::from_fn(|_| cx.focus_handle()));
        let controls = cx.new(|_| RovingHarness {
            handles: handles.clone(),
            selected: 1,
            disabled: false,
            changes: 0,
            manual: true,
        });
        let (_, cx) =
            cx.add_window_view(|window, cx| gpui_base::Root::new(controls.clone(), window, cx));
        cx.update(|window, cx| {
            window.activate_window();
            window.draw(cx).clear(cx);
            handles[0].focus(window, cx);
            window.draw(cx).clear(cx);
        });
        // Arrows only move focus; Shift+Tab and Tab then leave the group
        // from wherever focus is, and re-entry lands on the selection.
        for (key, expected) in [
            ("tab", 2),
            ("right", 3),
            ("shift-tab", 0),
            ("tab", 2),
            ("left", 1),
            ("tab", 4),
        ] {
            press_roving_key(cx, key);
            cx.update(|window, cx| {
                assert_eq!(
                    window.focused(cx).as_ref(),
                    Some(&handles[expected]),
                    "{key}"
                )
            });
        }
        cx.update(|_, cx| assert_eq!(controls.read(cx).changes, 0));
    }

    struct FullWidthHarness {
        count: usize,
        selected: usize,
        width: Pixels,
        changes: usize,
        disabled: bool,
    }

    struct ContentWidthHarness {
        selected: SharedString,
        changes: usize,
    }

    const MODEL_LABELS: [&str; 3] = ["Qwen3-ASR 1.7B", "Qwen3-ASR 0.6B", "Whisper"];

    struct ModelWidthHarness {
        selected: SharedString,
    }

    impl Render for ModelWidthHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().w(px(600.)).font_family(".SystemUIFont").child(
                ConnectedButtonGroup::new("model-width-choice", "Recognition model")
                    .options(MODEL_LABELS.map(|label| (label, label)))
                    .selected(self.selected.clone())
                    .on_change(cx.listener(|this, selected: &SharedString, _, cx| {
                        this.selected = selected.clone();
                        cx.notify();
                    })),
            )
        }
    }

    impl Render for ContentWidthHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().w(px(600.)).child(
                ConnectedButtonGroup::new("content-width-choice", "Speech service")
                    .options([
                        ("auto", "Auto"),
                        ("local", "Whisper"),
                        ("remote", "Cloud transcription service"),
                    ])
                    .selected(self.selected.clone())
                    .on_change(cx.listener(|this, next: &SharedString, _, cx| {
                        this.selected = next.clone();
                        this.changes += 1;
                        cx.notify();
                    })),
            )
        }
    }

    impl Render for FullWidthHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().w(self.width).child(
                ConnectedButtonGroup::new("full-width-choice", "文字大小")
                    .options(
                        (0..self.count)
                            .map(|index| (index.to_string(), format!("{}%", 100 + 25 * index))),
                    )
                    .selected(self.selected.to_string())
                    .full_width()
                    .disabled(self.disabled)
                    .on_change(cx.listener(|this, next: &SharedString, _, cx| {
                        this.selected = next.parse().unwrap();
                        this.changes += 1;
                        cx.notify();
                    })),
            )
        }
    }

    struct SegmentedHarness {
        selected: usize,
        changes: usize,
    }

    impl Render for SegmentedHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().w(px(360.)).child(
                ConnectedButtonGroup::new("connected-choice", "文字来源")
                    .options([("0", "自动"), ("1", "仅字幕"), ("2", "语音识别")])
                    .icon("0", IconName::Info)
                    .icon("1", IconName::Check)
                    .icon("2", IconName::Warning)
                    .segmented()
                    .selected(self.selected.to_string())
                    .on_change(cx.listener(|this, next: &SharedString, _, cx| {
                        this.selected = next.parse().unwrap();
                        this.changes += 1;
                        cx.notify();
                    })),
            )
        }
    }

    fn painted(name: &str) -> OptionPaint {
        PAINTED.with(|painted| {
            *painted
                .borrow()
                .get(name)
                .unwrap_or_else(|| panic!("no paint for {name}"))
        })
    }

    fn token(role: ColorRole) -> gpui::Rgba {
        crate::theme::color(role)
    }

    /// Blended endpoints may differ from a token in the last float bit.
    fn same(a: gpui::Rgba, b: gpui::Rgba) -> bool {
        [a.r - b.r, a.g - b.g, a.b - b.b, a.a - b.a]
            .iter()
            .all(|d| d.abs() < 1e-4)
    }

    /// Selected: accent container, on-primary content and a full pill.
    fn assert_selected(name: &str, height: Pixels) {
        let paint = painted(name);
        assert!(
            same(paint.background, token(ColorRole::SECONDARY)),
            "{name}: {:?}",
            paint
        );
        assert!(
            same(paint.foreground, token(ColorRole::ON_SECONDARY)),
            "{name}: {:?}",
            paint
        );
        assert_eq!(paint.left_radius, height / 2., "{name}");
        assert_eq!(paint.right_radius, height / 2., "{name}");
        // A state layer resolves over the selected container, never replaces it.
        assert!(!same(paint.hover, token(ColorRole::SECONDARY_CONTAINER)));
        assert!(!same(paint.hover, paint.background));
    }

    /// Unselected: neutral container, ink content; only outer ends are round.
    fn assert_unselected(name: &str, index: usize, count: usize, height: Pixels) {
        let paint = painted(name);
        assert_eq!(
            paint.background,
            token(ColorRole::SECONDARY_CONTAINER),
            "{name}"
        );
        assert!(
            same(paint.foreground, token(ColorRole::ON_SECONDARY_CONTAINER)),
            "{name}: {:?}",
            paint
        );
        let left = if index == 0 { height / 2. } else { px(8.) };
        let right = if index + 1 == count {
            height / 2.
        } else {
            px(8.)
        };
        assert_eq!(paint.left_radius, left, "{name}");
        assert_eq!(paint.right_radius, right, "{name}");
    }

    struct SizedHarness(crate::components::ButtonSize);
    impl Render for SizedHarness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(crate::tokens::typography::BASE_PX));
            div().flex().items_start().child(
                ConnectedButtonGroup::new("connected-sized", "Sized")
                    .options([("0", "One"), ("1", "Two")])
                    .icon("0", IconName::Check)
                    .button_size(self.0)
                    .selected("1"),
            )
        }
    }

    /// An option of a sized group is that size's button: the kit's padding and
    /// icon-label gap (`tests/fixtures/figma-button.json`).
    #[gpui::test]
    fn a_sized_option_has_its_buttons_padding_and_gap(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        use crate::components::ButtonSize as Size;
        for (size, padding, gap) in [
            (Size::ExtraSmall, 12., 4.),
            (Size::Small, 16., 8.),
            (Size::Medium, 24., 8.),
            (Size::Large, 48., 12.),
            (Size::ExtraLarge, 64., 16.),
        ] {
            let (_, cx) = cx.add_window_view(move |_, _| SizedHarness(size));
            draw_choice(cx);
            let option = choice_bounds(cx, "content-choice-option-0");
            let icon = choice_bounds(cx, "content-choice-icon-0");
            let label = choice_bounds(cx, "content-choice-label-0");
            assert_eq!(icon.left() - option.left(), px(padding), "{size:?} padding");
            assert_eq!(label.left() - icon.right(), px(gap), "{size:?} gap");
        }
    }

    #[gpui::test]
    fn connected_segments_fit_their_labels_and_keep_them_still_on_selection(
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let (view, cx) = cx.add_window_view(|_, _| SegmentedHarness {
            selected: 0,
            changes: 0,
        });
        draw_choice(cx);
        let labels = [
            "full-choice-label-0",
            "full-choice-label-1",
            "full-choice-label-2",
        ];
        let before = labels.map(|selector| choice_bounds(cx, selector));
        let icons = [
            "full-choice-icon-0",
            "full-choice-icon-1",
            "full-choice-icon-2",
        ];
        let icon_before = icons.map(|selector| choice_bounds(cx, selector));
        let options = OPTION_SELECTORS[..3]
            .iter()
            .map(|selector| choice_bounds(cx, selector))
            .collect::<Vec<_>>();
        let lane = choice_bounds(cx, "full-choice-lane");
        for index in 0..3 {
            // Each label follows its icon; the cluster is centered in a 48px button.
            assert_eq!(options[index].size.height, px(48.));
            // Each option holds its whole label.
            assert!(before[index].right() <= options[index].right());
            assert!(icon_before[index].right() <= before[index].left());
            let content_center = (icon_before[index].left() + before[index].right()) / 2.;
            assert!((content_center - options[index].center().x).abs() <= px(0.5));
            assert!((icon_before[index].center().y - options[index].center().y).abs() <= px(0.5));
            assert!((before[index].center().y - options[index].center().y).abs() <= px(0.5));
        }
        // Labels that fit an equal share get equal options.
        for option in &options[1..] {
            assert!((option.size.width - options[0].size.width).abs() <= px(0.5));
        }
        // Separate buttons with a 2px gap that together fill the parent.
        for pair in options.windows(2) {
            assert!((pair[1].left() - pair[0].right() - px(2.)).abs() <= px(0.5));
        }
        assert!((options[0].left() - lane.left()).abs() < px(0.5));
        assert!((options[2].right() - lane.right()).abs() < px(0.5));
        assert_eq!(lane.size.width, px(360.));
        assert_selected("full-0", px(48.));
        assert_unselected("full-1", 1, 3, px(48.));
        assert_unselected("full-2", 2, 3, px(48.));
        // The narrowest options lane must show the full label.
        let required = cx.update(|window, _| {
            let mut font = window.text_style().font();
            font.weight = FontWeight::MEDIUM;
            window
                .text_system()
                .shape_line(
                    "语音识别".into(),
                    px(14.),
                    &[TextRun {
                        len: "语音识别".len(),
                        font,
                        color: gpui::black(),
                        background_color: None,
                        underline: None,
                        strikethrough: None,
                    }],
                    None,
                )
                .width
                .ceil()
        });
        assert!(
            before[2].size.width >= required,
            "label bounds: {:?}, needed: {:?}",
            before[2],
            required
        );
        cx.simulate_click(options[2].center(), Modifiers::default());
        draw_choice(cx);
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).selected, 2);
            assert_eq!(view.read(cx).changes, 1);
        });
        // Reduce motion: the new selection is fully drawn on the changed frame.
        assert_selected("full-2", px(48.));
        assert_unselected("full-0", 0, 3, px(48.));
        for (selector, bounds) in labels.into_iter().zip(before) {
            assert_eq!(choice_bounds(cx, selector), bounds);
        }
        for (selector, bounds) in icons.into_iter().zip(icon_before) {
            assert_eq!(choice_bounds(cx, selector), bounds);
        }
        cx.simulate_click(options[2].center(), Modifiers::default());
        cx.update(|_, cx| assert_eq!(view.read(cx).changes, 1));
    }

    struct LongLabelHarness {
        width: Pixels,
    }

    const LONG_LABELS: [&str; 3] = ["Auto", "Subtitles", "Speech recognition"];

    impl Render for LongLabelHarness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().w(self.width).child(
                ConnectedButtonGroup::new("long-label-choice", "Text source")
                    .options(LONG_LABELS.map(|label| (label, label)))
                    .icon("Auto", IconName::Info)
                    .icon("Subtitles", IconName::Check)
                    .icon("Speech recognition", IconName::Warning)
                    .segmented()
                    .selected("Auto"),
            )
        }
    }

    /// Equal options are floored at their label: a label wider than an equal
    /// share keeps its whole width, the others share the rest equally, and
    /// when even the labels cannot fit the options wrap instead of clipping.
    #[gpui::test]
    fn filling_options_never_truncate_a_label_wider_than_its_share(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| LongLabelHarness { width: px(480.) });
        draw_choice(cx);
        let required = cx.update(|window, _| {
            let mut font = window.text_style().font();
            font.weight = FontWeight::MEDIUM;
            LONG_LABELS.map(|label| {
                window
                    .text_system()
                    .shape_line(
                        label.into(),
                        px(14.),
                        &[TextRun {
                            len: label.len(),
                            font: font.clone(),
                            ..Default::default()
                        }],
                        None,
                    )
                    .width
                    .ceil()
            })
        });
        let labels = [
            "full-choice-label-0",
            "full-choice-label-1",
            "full-choice-label-2",
        ];
        let check = |cx: &mut VisualTestContext| {
            for (index, selector) in labels.into_iter().enumerate() {
                let label = choice_bounds(cx, selector);
                let option = choice_bounds(cx, OPTION_SELECTORS[index]);
                assert!(
                    label.size.width + px(0.5) >= required[index],
                    "{selector} truncated"
                );
                assert!(label.right() <= option.right() + px(0.5));
            }
        };
        check(cx);
        let options = [0, 1, 2].map(|index| choice_bounds(cx, OPTION_SELECTORS[index]));
        assert!(options[2].size.width > options[0].size.width);
        assert!((options[1].size.width - options[0].size.width).abs() <= px(0.5));
        // Too narrow for all three labels on one row: they wrap.
        cx.update(|_, cx| {
            view.update(cx, |view, cx| {
                view.width = px(220.);
                cx.notify();
            })
        });
        draw_choice(cx);
        check(cx);
        let wrapped = [0, 1, 2].map(|index| choice_bounds(cx, OPTION_SELECTORS[index]));
        assert!(wrapped[2].top() > wrapped[0].top());
    }

    const OPTION_SELECTORS: [&str; 4] = [
        "full-choice-option-0",
        "full-choice-option-1",
        "full-choice-option-2",
        "full-choice-option-3",
    ];

    fn choice_bounds(cx: &mut VisualTestContext, name: &'static str) -> Bounds<Pixels> {
        cx.debug_bounds(name)
            .unwrap_or_else(|| panic!("missing {name}"))
    }

    fn draw_choice(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }

    #[gpui::test]
    fn content_width_model_labels_keep_their_rounded_text_width(cx: &mut TestAppContext) {
        let (_, cx) = cx.add_window_view(|_, _| ModelWidthHarness {
            selected: MODEL_LABELS[0].into(),
        });
        draw_choice(cx);
        let required_widths = cx.update(|window, _| {
            let mut label_font = font(".SystemUIFont");
            label_font.weight = FontWeight::MEDIUM;
            MODEL_LABELS.map(|label| {
                window
                    .text_system()
                    .shape_line(
                        label.into(),
                        px(14.),
                        &[TextRun {
                            len: label.len(),
                            font: label_font.clone(),
                            ..Default::default()
                        }],
                        None,
                    )
                    .width
                    .ceil()
            })
        });
        let option_selectors = [
            "content-choice-option-0",
            "content-choice-option-1",
            "content-choice-option-2",
        ];
        let label_selectors = [
            "content-choice-label-0",
            "content-choice-label-1",
            "content-choice-label-2",
        ];
        let before = option_selectors.map(|selector| choice_bounds(cx, selector));
        // The native GPUI test window uses a 2x device scale. Check actual
        // laid-out labels, including the slightly longer 0.6B option.
        for index in 0..MODEL_LABELS.len() {
            let label = choice_bounds(cx, label_selectors[index]);
            assert!(
                label.size.width >= required_widths[index],
                "{} was narrowed below its full shaped width",
                MODEL_LABELS[index]
            );
            // Content-sized buttons: 16px from each edge to the label.
            let leading = label.left() - before[index].left();
            let trailing = before[index].right() - label.right();
            assert!((leading - px(16.)).abs() <= px(1.), "leading {leading:?}");
            assert!(
                (trailing - px(16.)).abs() <= px(1.),
                "trailing {trailing:?}"
            );
            assert_eq!(before[index].size.height, px(40.));
        }
        cx.simulate_click(before[1].center(), Modifiers::default());
        draw_choice(cx);
        for index in 0..MODEL_LABELS.len() {
            assert_eq!(choice_bounds(cx, option_selectors[index]), before[index]);
            assert!(choice_bounds(cx, label_selectors[index]).size.width >= required_widths[index]);
        }
    }

    #[gpui::test]
    fn content_width_choices_morph_in_place_and_settle(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| ContentWidthHarness {
            selected: "auto".into(),
            changes: 0,
        });
        draw_choice(cx);
        let selectors = [
            "content-choice-option-0",
            "content-choice-option-1",
            "content-choice-option-2",
        ];
        let labels = [
            "content-choice-label-0",
            "content-choice-label-1",
            "content-choice-label-2",
        ];
        let before = selectors.map(|selector| choice_bounds(cx, selector));
        let label_before = labels.map(|selector| choice_bounds(cx, selector));
        // Each option is sized to its own label and the group hugs the start.
        assert!(before[2].size.width > before[0].size.width + px(40.));
        assert!((before[1].left() - before[0].right() - px(2.)).abs() < px(0.5));
        assert_selected("content-0", px(40.));
        cx.simulate_click(before[2].center(), Modifiers::default());
        draw_choice(cx);
        // The click frame starts from the current presentation, not the endpoint.
        assert_eq!(
            painted("content-2").background,
            token(ColorRole::SECONDARY_CONTAINER)
        );
        cx.executor().advance_clock(Duration::from_millis(60));
        draw_choice(cx);
        let entering = painted("content-2");
        let leaving = painted("content-0");
        assert!(entering.right_radius == px(20.) && entering.left_radius > px(8.));
        assert!(
            entering.left_radius < px(20.),
            "an intermediate shape is drawn"
        );
        assert!(leaving.right_radius > px(8.) && leaving.right_radius < px(20.));
        assert!(!same(entering.background, token(ColorRole::SECONDARY)));
        for index in 0..3 {
            assert_eq!(choice_bounds(cx, selectors[index]), before[index]);
            assert_eq!(choice_bounds(cx, labels[index]), label_before[index]);
        }
        cx.executor().advance_clock(
            crate::motion::curve(crate::motion::MotionToken::FastSpatial).duration(),
        );
        draw_choice(cx);
        assert_selected("content-2", px(40.));
        assert_unselected("content-0", 0, 3, px(40.));
        cx.simulate_mouse_move(before[2].center(), None, Modifiers::default());
        draw_choice(cx);
        assert_eq!(choice_bounds(cx, selectors[2]), before[2]);
        cx.update(|window, cx| {
            assert_eq!(view.read(cx).changes, 1);
            window.simulate_next_frame(cx);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(window.simulate_next_frame(cx), 0);
        });
    }

    fn assert_equal_options(cx: &mut VisualTestContext, count: usize, width: Pixels) {
        let track = choice_bounds(cx, "full-choice-track");
        let lane = choice_bounds(cx, "full-choice-lane");
        assert_eq!(track.size.width, width);
        assert_eq!(lane.size.width, width);
        let share = (width - px(2.) * (count - 1) as f32) / count as f32;
        for selector in &OPTION_SELECTORS[..count] {
            let option = choice_bounds(cx, selector);
            assert!((option.size.width - share).abs() <= px(0.5));
            assert_eq!(option.size.height, px(40.));
        }
    }

    fn exercise_full_width_motion(cx: &mut TestAppContext, count: usize) {
        let (view, cx) = cx.add_window_view(|_, _| FullWidthHarness {
            count,
            selected: 0,
            width: px(420.),
            changes: 0,
            disabled: false,
        });
        draw_choice(cx);
        assert_equal_options(cx, count, px(420.));
        let last = format!("full-{}", count - 1);
        assert_selected("full-0", px(40.));
        assert_unselected(&last, count - 1, count, px(40.));
        let options = (0..count)
            .map(|index| choice_bounds(cx, OPTION_SELECTORS[index]))
            .collect::<Vec<_>>();
        let destination = options[count - 1];

        cx.simulate_click(destination.center(), Modifiers::default());
        cx.update(|window, cx| {
            assert_eq!(
                view.read(cx).selected,
                count - 1,
                "click is accepted immediately"
            );
            assert_eq!(view.read(cx).changes, 1);
            window.draw(cx).clear(cx);
        });
        assert_eq!(painted(&last).left_radius, px(8.));
        cx.simulate_click(destination.center(), Modifiers::default());
        cx.update(|_, cx| assert_eq!(view.read(cx).changes, 1, "same choice does not save twice"));

        cx.executor().advance_clock(Duration::from_millis(60));
        draw_choice(cx);
        let middle = painted(&last);
        assert!(middle.left_radius > px(8.) && middle.left_radius < px(20.));
        assert!(!same(
            middle.background,
            token(ColorRole::SECONDARY_CONTAINER)
        ));
        assert!(!same(middle.background, token(ColorRole::SECONDARY)));
        for (index, before) in options.iter().enumerate() {
            assert_eq!(choice_bounds(cx, OPTION_SELECTORS[index]), *before);
        }
        draw_choice(cx);
        assert_eq!(
            painted(&last),
            middle,
            "ordinary redraw must not restart motion"
        );

        // Reverse before settling, then resize without advancing time. The
        // shape continues from its current value instead of jumping.
        cx.simulate_click(options[0].center(), Modifiers::default());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(painted(&last).left_radius, middle.left_radius);
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.width = px(284.);
                cx.notify();
            });
            window.draw(cx).clear(cx);
        });
        assert_equal_options(cx, count, px(284.));
        cx.executor().advance_clock(Duration::from_millis(30));
        draw_choice(cx);
        let returning = painted(&last);
        assert!(returning.left_radius < middle.left_radius);
        assert!(returning.left_radius >= px(8.));

        cx.executor().advance_clock(
            crate::motion::curve(crate::motion::MotionToken::FastSpatial).duration(),
        );
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(
                window.simulate_next_frame(cx),
                0,
                "settled control must stop scheduling frames"
            );
        });
        assert_selected("full-0", px(40.));
        assert_unselected(&last, count - 1, count, px(40.));
    }

    #[gpui::test]
    fn full_width_choices_morph_inside_current_geometry_and_settle(cx: &mut TestAppContext) {
        for count in [3, 4] {
            exercise_full_width_motion(cx, count);
        }
    }

    #[gpui::test]
    fn full_width_choices_respect_reduce_motion_on_the_changed_frame(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let (view, cx) = cx.add_window_view(|_, _| FullWidthHarness {
            count: 4,
            selected: 0,
            width: px(420.),
            changes: 0,
            disabled: false,
        });
        draw_choice(cx);
        let destination = choice_bounds(cx, "full-choice-option-3");
        cx.simulate_click(destination.center(), Modifiers::default());
        cx.update(|window, cx| {
            assert_eq!(view.read(cx).selected, 3);
            window.draw(cx).clear(cx);
            assert_eq!(window.simulate_next_frame(cx), 0);
        });
        assert_selected("full-3", px(40.));
        assert_unselected("full-0", 0, 4, px(40.));
        assert_equal_options(cx, 4, px(420.));
    }

    #[gpui::test]
    fn arrow_keys_move_focus_and_selection_once_per_step(cx: &mut TestAppContext) {
        cx.update(|cx| {
            cx.set_reduce_motion(true);
            crate::components::roving::init(cx);
        });
        let (view, cx) = cx.add_window_view(|_, _| FullWidthHarness {
            count: 3,
            selected: 0,
            width: px(420.),
            changes: 0,
            disabled: false,
        });
        draw_choice(cx);
        // Clicking the selected option focuses it without a change.
        let first = choice_bounds(cx, "full-choice-option-0").center();
        cx.simulate_click(first, Modifiers::default());
        draw_choice(cx);
        cx.update(|_, cx| assert_eq!(view.read(cx).changes, 0));
        cx.simulate_keystrokes("right");
        draw_choice(cx);
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).selected, 1);
            assert_eq!(view.read(cx).changes, 1);
        });
        cx.simulate_keystrokes("left left");
        draw_choice(cx);
        cx.update(|_, cx| {
            assert_eq!(
                view.read(cx).selected,
                2,
                "previous wraps to the last option"
            );
            assert_eq!(view.read(cx).changes, 3);
        });
        cx.simulate_keystrokes("end");
        cx.update(|_, cx| assert_eq!(view.read(cx).changes, 3, "already last"));
        cx.simulate_keystrokes("home");
        draw_choice(cx);
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).selected, 0);
            assert_eq!(view.read(cx).changes, 4);
        });
        assert_selected("full-0", px(40.));
    }

    #[gpui::test]
    fn disabled_choices_use_disabled_roles_and_ignore_clicks(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| FullWidthHarness {
            count: 3,
            selected: 1,
            width: px(420.),
            changes: 0,
            disabled: true,
        });
        draw_choice(cx);
        let third = choice_bounds(cx, "full-choice-option-2").center();
        cx.simulate_click(third, Modifiers::default());
        draw_choice(cx);
        cx.update(|_, cx| assert_eq!(view.read(cx).changes, 0));
        let ink = token(ColorRole::ON_SURFACE);
        for index in 0..3 {
            let paint = painted(&format!("full-{index}"));
            assert_eq!(paint.background, ink.opacity(0.12));
            assert_eq!(paint.foreground, ink.opacity(0.38));
            assert_eq!(paint.hover, paint.background, "no hover layer");
        }
        // The current option keeps its pill shape.
        assert_eq!(painted("full-1").left_radius, px(20.));
    }
}

#[cfg(test)]
mod mode_tests {
    use super::{ConnectedButtonGroup, GroupLabels, GroupSelection};
    use crate::components::{ButtonDensity, ButtonSize};
    use crate::icons::IconName;
    use gpui::{
        Context, InteractiveElement as _, IntoElement, Modifiers, ParentElement as _, Render,
        SharedString, Styled as _, TestAppContext, VisualTestContext, Window, div, px,
    };
    use std::{cell::RefCell, rc::Rc};

    #[derive(Default)]
    struct Harness {
        selection: Option<GroupSelection>,
        required: bool,
        vertical: bool,
        density: ButtonDensity,
        equal: bool,
        labels: GroupLabels,
        values: Vec<SharedString>,
        events: Rc<RefCell<Vec<String>>>,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            let events = self.events.clone();
            let activated = self.events.clone();
            let view = cx.entity().downgrade();
            div().flex().items_start().p(px(40.)).child(
                div().flex().debug_selector(|| "group".into()).child(
                    ConnectedButtonGroup::new("group", "Format")
                        .button_size(ButtonSize::Small)
                        .options([("b", "Bold"), ("i", "Italic"), ("u", "Underline wide")])
                        .icon("b", IconName::Check)
                        .icon("i", IconName::Check)
                        .icon("u", IconName::Check)
                        .selection(self.selection.unwrap_or_default())
                        .required(self.required)
                        .vertical(self.vertical)
                        .density(self.density)
                        .equal_widths(self.equal)
                        .labels(self.labels)
                        .selected_values(self.values.clone())
                        .on_toggle(move |value: &SharedString, on, _, cx| {
                            events.borrow_mut().push(format!("{value}={on}"));
                            let _ = view.update(cx, |this, cx| {
                                if on {
                                    this.values.push(value.clone());
                                } else {
                                    this.values.retain(|v| v != value);
                                }
                                cx.notify();
                            });
                        })
                        .on_activate(move |value, _, _| {
                            activated.borrow_mut().push(format!("{value}!"))
                        }),
                ),
            )
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        for _ in 0..3 {
            cx.executor()
                .advance_clock(std::time::Duration::from_millis(800));
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
        }
    }

    fn setup(cx: &mut TestAppContext, harness: Harness) -> &mut VisualTestContext {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (_, cx) = cx.add_window_view(move |_, _| harness);
        draw(cx);
        cx
    }

    fn option(cx: &mut VisualTestContext, index: usize) -> gpui::Bounds<gpui::Pixels> {
        let name: &'static str = Box::leak(format!("content-choice-option-{index}").into());
        cx.debug_bounds(name).unwrap()
    }

    #[gpui::test]
    fn multiple_selection_toggles_each_option(cx: &mut TestAppContext) {
        let events = Rc::new(RefCell::new(Vec::new()));
        let cx = setup(
            cx,
            Harness {
                selection: Some(GroupSelection::Multi),
                events: events.clone(),
                ..Harness::default()
            },
        );
        for index in [0, 2, 0] {
            let at = option(cx, index).center();
            cx.simulate_click(at, Modifiers::none());
            draw(cx);
        }
        assert_eq!(*events.borrow(), ["b=true", "u=true", "b=false"]);
    }

    #[gpui::test]
    fn an_optional_single_selection_can_be_cleared(cx: &mut TestAppContext) {
        let events = Rc::new(RefCell::new(Vec::new()));
        let cx = setup(
            cx,
            Harness {
                selection: Some(GroupSelection::Single),
                required: false,
                values: vec!["i".into()],
                events: events.clone(),
                ..Harness::default()
            },
        );
        let at = option(cx, 1).center();
        cx.simulate_click(at, Modifiers::none());
        draw(cx);
        assert_eq!(*events.borrow(), ["i=false"]);
    }

    #[gpui::test]
    fn a_group_without_selection_activates(cx: &mut TestAppContext) {
        let events = Rc::new(RefCell::new(Vec::new()));
        let cx = setup(
            cx,
            Harness {
                selection: Some(GroupSelection::None),
                events: events.clone(),
                ..Harness::default()
            },
        );
        let at = option(cx, 2).center();
        cx.simulate_click(at, Modifiers::none());
        draw(cx);
        assert_eq!(*events.borrow(), ["u!"]);
    }

    #[gpui::test]
    fn vertical_dense_groups_stack_shorter_options_of_one_width(cx: &mut TestAppContext) {
        let cx = setup(
            cx,
            Harness {
                required: true,
                vertical: true,
                density: ButtonDensity::Compact,
                ..Harness::default()
            },
        );
        let (first, last) = (option(cx, 0), option(cx, 2));
        assert_eq!(first.size.height, px(40. - 8.));
        assert_eq!(
            first.size.width, last.size.width,
            "options share the column"
        );
        assert_eq!(last.top(), first.top() + px(2. * (32. + 2.)));
    }

    #[gpui::test]
    fn equal_widths_take_the_widest_option(cx: &mut TestAppContext) {
        let cx = setup(
            cx,
            Harness {
                required: true,
                equal: true,
                ..Harness::default()
            },
        );
        draw(cx);
        let widths = [0, 1, 2].map(|index| option(cx, index).size.width);
        assert_eq!(widths[0], widths[2], "{widths:?}");
        assert_eq!(widths[1], widths[2], "{widths:?}");
    }

    #[gpui::test]
    fn selected_labels_show_only_on_selected_options(cx: &mut TestAppContext) {
        let cx = setup(
            cx,
            Harness {
                selection: Some(GroupSelection::Multi),
                labels: GroupLabels::Selected,
                values: vec!["u".into()],
                ..Harness::default()
            },
        );
        let (bare, labelled) = (option(cx, 0), option(cx, 2));
        // An unselected icon option is the icon and its padding, the border within.
        assert_eq!(bare.size.width, px(20. + 16. * 2.));
        assert!(labelled.size.width > bare.size.width + px(40.));
        let at = bare.center();
        cx.simulate_click(at, Modifiers::none());
        draw(cx);
        assert!(option(cx, 0).size.width > bare.size.width, "revealed");
    }

    struct Relabelled {
        long: bool,
    }
    impl Render for Relabelled {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            let last = if self.long {
                "A much much much longer label"
            } else {
                "Short"
            };
            div().flex().items_start().p(px(40.)).child(
                ConnectedButtonGroup::new("group", "Format")
                    .button_size(ButtonSize::Small)
                    .options([("a", "One"), ("b", last)])
                    .equal_widths(true),
            )
        }
    }

    #[gpui::test]
    fn equal_widths_shrink_with_their_labels(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| Relabelled { long: true });
        draw(cx);
        let wide = option(cx, 0).size.width;
        view.update(cx, |view, cx| {
            view.long = false;
            cx.notify();
        });
        draw(cx);
        let narrow = option(cx, 0).size.width;
        assert!(narrow < wide - px(50.), "{wide:?} then {narrow:?}");
        assert_eq!(narrow, option(cx, 1).size.width, "still equal");
    }

    struct EqualTabs;
    impl Render for EqualTabs {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().flex().items_start().p(px(40.)).child(
                ConnectedButtonGroup::new("group", "Format")
                    .button_size(ButtonSize::Small)
                    .options([("a", "One"), ("b", "A much longer label")])
                    .selected("a")
                    .tabs()
                    .equal_widths(true),
            )
        }
    }

    #[gpui::test]
    fn tabs_take_equal_widths_too(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (_, cx) = cx.add_window_view(|_, _| EqualTabs);
        draw(cx);
        assert_eq!(option(cx, 0).size.width, option(cx, 1).size.width);
    }

    struct RadioGroup;
    impl Render for RadioGroup {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().flex().items_start().p(px(40.)).child(
                ConnectedButtonGroup::new("group", "Format")
                    .button_size(ButtonSize::Small)
                    .options([("a", "One"), ("b", "Two")])
                    .selected("a")
                    .required(true),
            )
        }
    }

    struct TabGroup;
    impl Render for TabGroup {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().flex().items_start().p(px(40.)).child(
                ConnectedButtonGroup::new("group", "Format")
                    .button_size(ButtonSize::Small)
                    .options([("a", "One"), ("b", "Two")])
                    .selected("a")
                    .tabs(),
            )
        }
    }

    #[gpui::test]
    fn every_kind_of_option_ripples(cx: &mut TestAppContext) {
        use crate::components::button::waves_painted;
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (_, cx) = cx.add_window_view(|_, _| RadioGroup);
        draw(cx);
        let center = option(cx, 1).center();
        let before = waves_painted();
        cx.simulate_mouse_down(center, gpui::MouseButton::Left, Modifiers::none());
        draw(cx);
        assert!(waves_painted() > before, "a radio option");
        cx.simulate_mouse_up(center, gpui::MouseButton::Left, Modifiers::none());
        let (_, cx) = cx.cx.add_window_view(|_, _| TabGroup);
        draw(cx);
        let center = option(cx, 1).center();
        let before = waves_painted();
        cx.simulate_mouse_down(center, gpui::MouseButton::Left, Modifiers::none());
        draw(cx);
        assert!(waves_painted() > before, "a tab option");
    }
}
