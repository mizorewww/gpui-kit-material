//! M3 Expressive toolbars: docked and floating, standard and vibrant,
//! horizontal and vertical, optionally paired with a FAB.
//!
//! - [`Toolbar`] is the container. A floating toolbar is a 64dp
//!   full-radius pill on surface-container (standard) or primary-container
//!   (vibrant) at elevation level 3; a docked toolbar is a 64dp full-width
//!   band with 16dp side padding and its items spread across it. Items are
//!   caller elements, usually [`ToolbarButton`]s.
//! - [`Toolbar::visible`] hides and shows the toolbar on the default spatial
//!   spring, the M3 scroll behavior: a floating toolbar slides off its edge
//!   (a paired FAB stays), a docked one collapses its height. Spatial motion
//!   completes at once under reduced motion.
//! - [`ToolbarButton`] is the kit's toolbar building block: a 40dp icon or
//!   label button in a 48dp target, optionally toggleable, painted in the
//!   toolbar's color scheme. Pressing morphs its corners from full to 8dp on
//!   the fast spatial spring; selection cross-fades container and ink on the
//!   fast effects spring and swaps to the filled icon.
//!
//! Behavior is GPUI Base's: the container is a Base [`gpui_base::Toolbar`]
//! (toolbar role, Left/Right roving focus over any focusable item), and each
//! button is a Base [`gpui_base::Button`] (button role, pointer, Enter and
//! Space activation, pressed state for toggles).
//!
//! A vertical toolbar sets Base's axis, so Up/Down rove focus and assistive
//! technology hears a vertical orientation.
use super::InteractionPreview;
use super::button::{ButtonFocusRing, ButtonRipple};
use super::navigation::{NavIcon, ripple_corners, type_role};
use crate::color::ColorRole;
use crate::icons::Icon;
use crate::motion::{self, MotionToken};
use crate::theme::{blend, color};
use crate::tokens::{
    elevation, state,
    typography::{TypeRole, rem},
};
use gpui::{prelude::FluentBuilder as _, *};
use std::rc::Rc;

/// Toolbar container height (horizontal) or width (vertical).
pub const TOOLBAR_SIZE: f32 = 64.;
/// Space between a floating toolbar and the edge it hides past.
pub const TOOLBAR_MARGIN: f32 = 16.;
/// Space between a floating toolbar and its paired FAB.
pub const TOOLBAR_FAB_GAP: f32 = 8.;

/// Item target, container (pill) height and pressed corner radius.
const TARGET: f32 = 48.;
const PILL: f32 = 40.;
const PRESSED_RADIUS: f32 = 8.;
const ICON: f32 = 24.;
const LABEL_ICON: f32 = 20.;
const LABEL_PADDING: f32 = 16.;
const LABEL_GAP: f32 = 8.;
/// Floating: gap between items and padding inside the pill.
const FLOATING_GAP: f32 = 4.;
const FLOATING_PADDING: f32 = 8.;
/// Docked: side padding and minimum gap between items.
const DOCKED_PADDING: f32 = 16.;
const DOCKED_GAP: f32 = 8.;

/// Container color scheme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolbarColor {
    /// Surface container with on-surface-variant items.
    #[default]
    Standard,
    /// Primary container with on-primary-container items.
    Vibrant,
}

impl ToolbarColor {
    pub const ALL: [Self; 2] = [Self::Standard, Self::Vibrant];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Vibrant => "Vibrant",
        }
    }
    /// The container role.
    pub const fn container(self) -> ColorRole {
        match self {
            Self::Standard => ColorRole::SURFACE_CONTAINER,
            Self::Vibrant => ColorRole::PRIMARY_CONTAINER,
        }
    }
}

/// Where the toolbar sits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolbarConfiguration {
    /// A pill floating over content, elevated.
    #[default]
    Floating,
    /// A full-width band at the bottom of the window.
    Docked,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolbarOrientation {
    #[default]
    Horizontal,
    /// Floating only: a column on the leading or trailing edge.
    Vertical,
}

/// How far a hidden floating toolbar travels: its own size and its margin,
/// down past the bottom edge (horizontal) or across the trailing edge
/// (vertical).
pub(crate) const EXIT_OFFSET: f32 = TOOLBAR_SIZE + TOOLBAR_MARGIN;

/// The paint of one toolbar button for the current motion frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ButtonPaint {
    pub container: Rgba,
    pub ink: Rgba,
    pub layer: Rgba,
}

/// `fade` is the fast-effects selection amount (0 unselected, 1 selected).
/// A label button has a tonal container at rest; an icon button none.
pub(crate) fn button_paint(
    scheme: ToolbarColor,
    label: bool,
    fade: f32,
    disabled: bool,
) -> ButtonPaint {
    let on_surface = color(ColorRole::ON_SURFACE);
    if disabled {
        return ButtonPaint {
            container: on_surface.opacity(DISABLED_CONTAINER),
            ink: state::disabled(on_surface),
            layer: Rgba::default(),
        };
    }
    let fade = fade.clamp(0., 1.);
    let (rest_container, rest_ink, on_container, on_ink) = match scheme {
        ToolbarColor::Standard => (
            color(ColorRole::SURFACE_CONTAINER),
            color(ColorRole::ON_SURFACE_VARIANT),
            color(ColorRole::SECONDARY_CONTAINER),
            color(ColorRole::ON_SECONDARY_CONTAINER),
        ),
        ToolbarColor::Vibrant => (
            color(ColorRole::PRIMARY_CONTAINER),
            color(ColorRole::ON_PRIMARY_CONTAINER),
            color(ColorRole::SURFACE_CONTAINER),
            on_surface,
        ),
    };
    let rest_container = if label {
        rest_container
    } else {
        rest_container.opacity(0.)
    };
    let container = if label {
        blend(rest_container, on_container, fade)
    } else {
        on_container.opacity(fade)
    };
    let ink = blend(rest_ink, on_ink, fade);
    ButtonPaint {
        container,
        ink,
        layer: ink,
    }
}

/// The kit's disabled toolbar-button container: on-surface at 10%.
const DISABLED_CONTAINER: f32 = 0.10;

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[cfg(test)]
thread_local! {
    pub(crate) static PAINTED: std::cell::RefCell<std::collections::BTreeMap<String, (ButtonPaint, f32)>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

/// A toolbar's icon or label button.
#[derive(IntoElement)]
pub struct ToolbarButton {
    id: ElementId,
    icon: Option<NavIcon>,
    label: Option<SharedString>,
    accessibility_label: SharedString,
    scheme: ToolbarColor,
    toggle: bool,
    selected: bool,
    disabled: bool,
    ink: Option<ColorRole>,
    preview: Option<InteractionPreview>,
    on_click: Option<ClickHandler>,
}

impl ToolbarButton {
    /// An icon button. `accessibility_label` names it for assistive
    /// technology; the toolbar shows no text.
    pub fn icon(
        id: impl Into<ElementId>,
        icon: NavIcon,
        accessibility_label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            icon: Some(icon),
            label: None,
            accessibility_label: accessibility_label.into(),
            scheme: ToolbarColor::Standard,
            toggle: false,
            selected: false,
            disabled: false,
            ink: None,
            preview: None,
            on_click: None,
        }
    }
    /// A label button, optionally with a leading icon ([`Self::leading_icon`]).
    pub fn label(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        Self {
            id: id.into(),
            icon: None,
            label: Some(label.clone()),
            accessibility_label: label,
            scheme: ToolbarColor::Standard,
            toggle: false,
            selected: false,
            disabled: false,
            ink: None,
            preview: None,
            on_click: None,
        }
    }
    pub fn leading_icon(mut self, icon: NavIcon) -> Self {
        self.icon = Some(icon);
        self
    }
    /// The color scheme of the toolbar holding the button.
    pub fn color(mut self, scheme: ToolbarColor) -> Self {
        self.scheme = scheme;
        self
    }
    /// Toggle semantics: the button reports its pressed state and paints
    /// the selected container while `selected`.
    pub fn toggle(mut self, toggle: bool) -> Self {
        self.toggle = toggle;
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Override the resting ink, e.g. on-surface for an app bar's
    /// navigation icon.
    pub fn ink(mut self, role: ColorRole) -> Self {
        self.ink = Some(role);
        self
    }
    /// Render a static specimen in one forced interaction state. See
    /// [`InteractionPreview`].
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

/// Whether a button is held, for the pressed shape morph.
#[derive(Default)]
struct Held(bool);

impl RenderOnce for ToolbarButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let preview = self.preview;
        let disabled = self.disabled || preview.is_some_and(InteractionPreview::is_disabled);
        let enabled = !disabled;
        let live = enabled && preview.is_none();
        let selected = self.toggle && self.selected;
        let is_label = self.label.is_some();
        let fade = motion::token_value(
            (self.id.clone(), "toolbar-fade"),
            if selected { 1. } else { 0. },
            MotionToken::FastEffects,
            window,
            cx,
        );
        let held =
            window.use_keyed_state((self.id.clone(), "toolbar-held"), cx, |_, _| Held(false));
        let pressed = match preview {
            Some(InteractionPreview::Pressed) => true,
            Some(_) => false,
            None => live && held.read(cx).0,
        };
        let press = motion::token_value(
            (self.id.clone(), "toolbar-press"),
            if pressed { 1. } else { 0. },
            MotionToken::FastSpatial,
            window,
            cx,
        );
        let mut paint = button_paint(self.scheme, is_label, fade, disabled);
        if let (Some(role), false, true) = (self.ink, disabled, fade < 0.001) {
            paint.ink = color(role);
            paint.layer = paint.ink;
        }
        let radius_px = PILL / 2. + (PRESSED_RADIUS - PILL / 2.) * press.clamp(0., 1.2);
        #[cfg(test)]
        PAINTED.with(|painted| {
            painted
                .borrow_mut()
                .insert(format!("{:?}", self.id), (paint, radius_px))
        });
        let radius = rem(radius_px);
        let focus = window
            .use_keyed_state((self.id.clone(), "toolbar-focus"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone()
            .tab_stop(live);

        let (hover, focus_layer, press_layer) = (
            paint.layer.opacity(state::HOVER),
            paint.layer.opacity(state::FOCUS),
            paint.layer.opacity(state::PRESS),
        );
        let group: SharedString = format!("toolbar-button-{:?}", self.id).into();
        let layer = div()
            .id("toolbar-state-layer")
            .absolute()
            .inset_0()
            .rounded(radius)
            .map(|layer| match preview {
                _ if disabled => layer,
                None => layer
                    .group_hover(group.clone(), move |style| style.bg(hover))
                    .group_active(group.clone(), move |style| style.bg(press_layer)),
                Some(InteractionPreview::Hovered) => layer.bg(hover),
                Some(InteractionPreview::Focused) => layer.bg(focus_layer),
                Some(InteractionPreview::Pressed) => layer.bg(press_layer),
                Some(_) => layer,
            });
        let hovered_label = is_label && preview == Some(InteractionPreview::Hovered);
        let icon = self.icon.as_ref().map(|icon| {
            let size = if is_label { LABEL_ICON } else { ICON };
            Icon::new(icon.path(selected))
                .size(rem(size))
                .color(paint.ink)
        });
        let pill = div()
            .relative()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .h(rem(PILL))
            .when(is_label, |pill| {
                pill.px(rem(LABEL_PADDING))
                    .gap(rem(LABEL_GAP))
                    .when(live, |pill| {
                        let group = group.clone();
                        pill.group_hover(group, |style| style.shadow(elevation::level(1)))
                    })
                    .when(hovered_label, |pill| pill.shadow(elevation::level(1)))
            })
            .when(!is_label, |pill| pill.w(rem(PILL)))
            .rounded(radius)
            .bg(paint.container)
            .child(layer)
            .children(icon)
            .when_some(self.label.clone(), |pill, label| {
                pill.child(
                    div()
                        .relative()
                        .whitespace_nowrap()
                        .text_color(paint.ink)
                        .map(|text| type_role(text, TypeRole::LabelLarge))
                        .child(label),
                )
            });
        let corners = window.rem_size() * radius.0;
        let ripple = ButtonRipple::new(
            pill,
            (self.id.clone(), "toolbar-ripple").into(),
            preview.is_none(),
            enabled,
            ripple_corners(corners),
            Edges::all(px(0.).into()),
            paint.layer,
            window,
            cx,
        );
        let ring =
            ButtonFocusRing::with_corners(ripple, focus.clone(), enabled, Corners::all(corners))
                .forced(preview == Some(InteractionPreview::Focused));

        let on_click = self.on_click.filter(|_| live);
        let down = held.clone();
        let up = held.clone();
        let out = held;
        gpui_base::Button::new(self.id)
            .group(group)
            .disabled(disabled)
            .tab_stop(live)
            .accessibility_label(self.accessibility_label)
            .when(self.toggle, |button| button.selected(selected))
            .track_focus(&focus)
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .h(rem(TARGET))
            .min_w(rem(TARGET))
            .when(live, |button| {
                button
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        down.update(cx, |held, cx| {
                            held.0 = true;
                            cx.notify();
                        })
                    })
                    .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                        up.update(cx, |held, cx| {
                            held.0 = false;
                            cx.notify();
                        })
                    })
                    .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                        out.update(cx, |held, cx| {
                            held.0 = false;
                            cx.notify();
                        })
                    })
            })
            .child(ring)
            .when_some(on_click, |button, handler| {
                let focus = focus.clone();
                button.on_click(move |event, window, cx| {
                    focus.focus(window, cx);
                    handler(event, window, cx)
                })
            })
    }
}

/// A toolbar: caller items in a floating pill or a docked band.
#[derive(IntoElement)]
pub struct Toolbar {
    id: ElementId,
    label: SharedString,
    configuration: ToolbarConfiguration,
    orientation: ToolbarOrientation,
    scheme: ToolbarColor,
    items: Vec<AnyElement>,
    fab: Option<AnyElement>,
    visible: bool,
}

impl Toolbar {
    /// `label` names the toolbar for assistive technology.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            configuration: ToolbarConfiguration::Floating,
            orientation: ToolbarOrientation::Horizontal,
            scheme: ToolbarColor::Standard,
            items: Vec::new(),
            fab: None,
            visible: true,
        }
    }
    pub fn configuration(mut self, configuration: ToolbarConfiguration) -> Self {
        self.configuration = configuration;
        self
    }
    /// Vertical applies to floating toolbars; a docked toolbar stays
    /// horizontal.
    pub fn orientation(mut self, orientation: ToolbarOrientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn color(mut self, scheme: ToolbarColor) -> Self {
        self.scheme = scheme;
        self
    }
    pub fn item(mut self, item: impl IntoElement) -> Self {
        self.items.push(item.into_any_element());
        self
    }
    pub fn items<E: IntoElement>(mut self, items: impl IntoIterator<Item = E>) -> Self {
        self.items
            .extend(items.into_iter().map(IntoElement::into_any_element));
        self
    }
    /// A FAB beside (horizontal) or below (vertical) a floating toolbar. It
    /// stays when the toolbar hides.
    pub fn fab(mut self, fab: impl IntoElement) -> Self {
        self.fab = Some(fab.into_any_element());
        self
    }
    /// Show or hide the toolbar, e.g. from the scroll direction: hidden on
    /// scrolling down, shown on scrolling up.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    fn vertical(&self) -> bool {
        self.configuration == ToolbarConfiguration::Floating
            && self.orientation == ToolbarOrientation::Vertical
    }
}

#[cfg(test)]
thread_local! {
    pub(crate) static SHOWN: std::cell::RefCell<std::collections::BTreeMap<String, f32>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

impl RenderOnce for Toolbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let vertical = self.vertical();
        let docked = self.configuration == ToolbarConfiguration::Docked;
        let shown = motion::token_value(
            (self.id.clone(), "toolbar-visible"),
            if self.visible { 1. } else { 0. },
            MotionToken::DefaultSpatial,
            window,
            cx,
        );
        #[cfg(test)]
        SHOWN.with(|shown_map| {
            shown_map
                .borrow_mut()
                .insert(format!("{:?}", self.id), shown)
        });
        let gone = !self.visible && shown <= 0.001;
        let container = color(self.scheme.container());

        let bar = gpui_base::Toolbar::new(self.id.clone())
            .axis(if vertical {
                Axis::Vertical
            } else {
                Axis::Horizontal
            })
            .aria_label(self.label)
            .when(cfg!(test), |bar| bar.debug_selector(|| "toolbar".into()))
            .flex()
            .flex_shrink_0()
            .items_center()
            .bg(container)
            .map(|bar| {
                if docked {
                    bar.w_full()
                        .h(rem(TOOLBAR_SIZE))
                        .px(rem(DOCKED_PADDING))
                        .gap(rem(DOCKED_GAP))
                        .justify_between()
                } else if vertical {
                    bar.flex_col()
                        .w(rem(TOOLBAR_SIZE))
                        .py(rem(FLOATING_PADDING))
                        .gap(rem(FLOATING_GAP))
                        .rounded_full()
                        .shadow(elevation::level(3))
                } else {
                    bar.h(rem(TOOLBAR_SIZE))
                        .px(rem(FLOATING_PADDING))
                        .gap(rem(FLOATING_GAP))
                        .rounded_full()
                        .shadow(elevation::level(3))
                }
            })
            .when(!gone, |bar| bar.children(self.items));

        let bar = div()
            .flex()
            .when(docked, |wrapper| wrapper.w_full())
            .child(bar);

        if docked {
            // Collapse from the top edge; items ride down with it.
            return div()
                .w_full()
                .flex_shrink_0()
                .overflow_hidden()
                .h(rem(TOOLBAR_SIZE * shown.max(0.)))
                .child(bar)
                .into_any_element();
        }
        let offset = rem(EXIT_OFFSET * (1. - shown));
        let sliding = div()
            .relative()
            .when(gone, |bar| bar.invisible())
            .map(|bar| {
                if vertical {
                    bar.left(offset)
                } else {
                    bar.top(offset)
                }
            })
            .child(bar);
        match self.fab {
            None => sliding.into_any_element(),
            Some(fab) => div()
                .flex()
                .items_center()
                .gap(rem(TOOLBAR_FAB_GAP))
                .when(vertical, |pair| pair.flex_col())
                .child(sliding)
                .child(fab)
                .into_any_element(),
        }
    }
}

#[cfg(test)]
#[path = "toolbar_tests.rs"]
mod tests;
