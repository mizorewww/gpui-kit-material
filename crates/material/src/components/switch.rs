//! M3 switch composed over GPUI Base's switch behavior.
//! Reference playground: <https://md3.io/components/switch/>.
use super::InteractionPreview;
use super::button::ButtonFocusRing;
use crate::{
    color::ColorRole,
    icons::IconName,
    motion::{self, MotionToken},
    theme::color,
    tokens::{state, typography::rem},
};
use gpui::{prelude::*, *};
use std::rc::Rc;

/// Controlled M3 switch: a 52×32 track whose handle moves and resizes on the
/// fast spatial spring (16dp unselected, 24dp selected or with an icon, 28dp
/// while pressed), with a 40dp state layer, the per-state handle colors and
/// optional check/close icons inside the handle.
#[derive(gpui::IntoElement)]
pub struct MaterialSwitch {
    id: ElementId,
    label: SharedString,
    show_label: bool,
    supporting: Option<SharedString>,
    label_position: LabelPosition,
    required: bool,
    checked: bool,
    disabled: bool,
    error: bool,
    icons: SwitchIcons,
    preview: Option<InteractionPreview>,
    on_change: Option<SwitchChange>,
}
type SwitchChange = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// Which states show an icon inside the handle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwitchIcons {
    /// No icons: a 16dp unselected and 24dp selected handle.
    #[default]
    None,
    /// A check on the selected handle only.
    Selected,
    /// A check when selected and a close mark when not; both handles are
    /// 24dp.
    Both,
}

impl SwitchIcons {
    fn shows(self, checked: bool) -> bool {
        match self {
            Self::None => false,
            Self::Selected => checked,
            Self::Both => true,
        }
    }
}

impl MaterialSwitch {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            show_label: false,
            supporting: None,
            label_position: LabelPosition::Start,
            required: false,
            checked: false,
            disabled: false,
            error: false,
            icons: SwitchIcons::None,
            preview: None,
            on_change: None,
        }
    }
    /// Render a static specimen in one forced interaction state, painting
    /// what the live state paints: the hover, focus or press state layer and
    /// handle color, the pressed handle and the focus ring. The specimen
    /// leaves the tab order and ignores live hover and press. See
    /// [`InteractionPreview`].
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
    /// Show the label beside the switch, filling the available width.
    /// Pressing it toggles the switch.
    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }
    /// Supporting text under a visible label.
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }
    /// Which side a visible label sits on (default start).
    pub fn label_position(mut self, position: LabelPosition) -> Self {
        self.label_position = position;
        self
    }
    /// Mark the choice as required: the label and accessible name gain the
    /// required mark. Enforcing it belongs to the form that owns the value.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Paint the error colors (md3.io's invalid switch, e.g. a required
    /// setting left off): error replaces primary on a selected switch and
    /// the outline on an unselected one. A disabled switch ignores it.
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }
    /// Show icons inside the handle.
    pub fn icons(mut self, icons: SwitchIcons) -> Self {
        self.icons = icons;
        self
    }
    pub fn on_change(mut self, f: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}
/// The switch's track, in dp: 52×32 with a 2dp outline. The handle must
/// stay inside the outline, so its travel is bounded by the track's inner
/// edges.
const TRACK_WIDTH: f32 = 52.;
const TRACK_HEIGHT: f32 = 32.;
const TRACK_OUTLINE: f32 = 2.;
/// The handle: 16dp unselected, 24dp selected or holding an icon, 28dp while
/// pressed, inside a 40dp state layer. Icons are 16dp.
const HANDLE_UNSELECTED: f32 = 16.;
const HANDLE_SELECTED: f32 = 24.;
const HANDLE_PRESSED: f32 = 28.;
const STATE_LAYER: f32 = 40.;
const ICON: f32 = 16.;
/// The handle's center, from the track's outer leading edge, at rest
/// unselected and selected.
const CENTER_UNSELECTED: f32 = 16.;
const CENTER_SELECTED: f32 = 36.;

/// The handle's resting size.
fn handle_size(checked: bool, icon: bool, pressed: bool) -> f32 {
    if pressed {
        HANDLE_PRESSED
    } else if checked || icon {
        HANDLE_SELECTED
    } else {
        HANDLE_UNSELECTED
    }
}

/// Handle leading offset (from the track's outer edge) for a selection
/// amount `t` and a presented handle `size`. Both follow the fast spatial
/// spring, whose overshoot is kept, bounded only so the handle never crosses
/// the outline.
fn handle_left(t: f32, size: f32) -> f32 {
    let t = if t.is_finite() { t } else { 0. };
    let size = size.clamp(0., HANDLE_PRESSED);
    let center = CENTER_UNSELECTED + (CENTER_SELECTED - CENTER_UNSELECTED) * t;
    (center - size / 2.)
        .max(TRACK_OUTLINE)
        .min(TRACK_WIDTH - TRACK_OUTLINE - size)
}

/// The paint of a switch in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
struct SwitchPaint {
    track: Rgba,
    outline: Rgba,
    handle: Rgba,
    /// The handle while hovered, focused or pressed.
    handle_active: Rgba,
    icon: Rgba,
    /// The state layer's content color.
    layer: Rgba,
}

/// Disabled colors follow the Figma kit's tokens literally, through
/// [`state`]: a disabled container (track, outline) is its role at
/// [`state::DISABLED_CONTAINER`] and disabled content (handle, icon) is its
/// role at [`state::DISABLED_CONTENT`], composited over whatever is beneath
/// rather than pre-mixed into a surface. The one exception is the token's
/// opaque surface handle of a selected disabled switch.
///
/// An error switch swaps its accent for the error roles, as the text field
/// does: error / on-error / error-container / on-error-container where the
/// selected switch uses the primary family, and an error outline and
/// handle (on-error-container while active) on an unselected one.
fn switch_paint(checked: bool, disabled: bool, error: bool) -> SwitchPaint {
    let role = color;
    let container = |r| role(r).opacity(state::DISABLED_CONTAINER);
    if error && !disabled {
        return if checked {
            SwitchPaint {
                track: role(ColorRole::ERROR),
                outline: role(ColorRole::ERROR),
                handle: role(ColorRole::ON_ERROR),
                handle_active: role(ColorRole::ERROR_CONTAINER),
                icon: role(ColorRole::ON_ERROR_CONTAINER),
                layer: role(ColorRole::ERROR),
            }
        } else {
            SwitchPaint {
                track: role(ColorRole::SURFACE_CONTAINER_HIGHEST),
                outline: role(ColorRole::ERROR),
                handle: role(ColorRole::ERROR),
                handle_active: role(ColorRole::ON_ERROR_CONTAINER),
                icon: role(ColorRole::SURFACE_CONTAINER_HIGHEST),
                layer: role(ColorRole::ERROR),
            }
        };
    }
    match (checked, disabled) {
        (true, false) => SwitchPaint {
            track: role(ColorRole::PRIMARY),
            outline: role(ColorRole::PRIMARY),
            handle: role(ColorRole::ON_PRIMARY),
            handle_active: role(ColorRole::PRIMARY_CONTAINER),
            icon: role(ColorRole::ON_PRIMARY_CONTAINER),
            layer: role(ColorRole::PRIMARY),
        },
        (false, false) => SwitchPaint {
            track: role(ColorRole::SURFACE_CONTAINER_HIGHEST),
            outline: role(ColorRole::OUTLINE),
            handle: role(ColorRole::OUTLINE),
            handle_active: role(ColorRole::ON_SURFACE_VARIANT),
            icon: role(ColorRole::SURFACE_CONTAINER_HIGHEST),
            layer: role(ColorRole::ON_SURFACE),
        },
        (true, true) => SwitchPaint {
            track: container(ColorRole::ON_SURFACE),
            outline: Rgba {
                a: 0.,
                ..role(ColorRole::ON_SURFACE)
            },
            handle: role(ColorRole::SURFACE),
            handle_active: role(ColorRole::SURFACE),
            icon: state::disabled(role(ColorRole::ON_SURFACE)),
            layer: role(ColorRole::ON_SURFACE),
        },
        (false, true) => {
            let handle = state::disabled(role(ColorRole::ON_SURFACE));
            SwitchPaint {
                track: container(ColorRole::SURFACE_CONTAINER_HIGHEST),
                outline: container(ColorRole::ON_SURFACE),
                handle,
                handle_active: handle,
                icon: state::disabled(role(ColorRole::SURFACE_CONTAINER_HIGHEST)),
                layer: role(ColorRole::ON_SURFACE),
            }
        }
    }
}

/// Hover styles resolve against this group.
const SWITCH_GROUP: &str = "material-switch";

impl RenderOnce for MaterialSwitch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus = window
            .use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let press_state = window.use_keyed_state((self.id.clone(), "pressed"), cx, |_, _| false);
        let preview = self.preview;
        let disabled = self.disabled || preview.is_some_and(|p| p.is_disabled());
        let enabled = !disabled;
        let live = enabled && preview.is_none();
        let forced = |state| enabled && preview == Some(state);
        let keyboard_focus = forced(InteractionPreview::Focused)
            || (live && focus.is_focused(window) && window.last_input_was_keyboard());
        // The release handlers are only installed while live, so a press
        // that outlives a switch to disabled or a preview ends here rather
        // than sticking once it is live again.
        if !live && *press_state.read(cx) {
            press_state.update(cx, |pressed, _| *pressed = false);
        }
        let pressed = forced(InteractionPreview::Pressed) || (live && *press_state.read(cx));
        let icon = self.icons.shows(self.checked);
        // Position and size follow separate fast spatial springs, so a press
        // during a toggle retargets only the size, from where it is.
        let t = motion::selection_value(
            (self.id.clone(), "thumb"),
            if self.checked { 1. } else { 0. },
            window,
            cx,
        );
        let target = handle_size(self.checked, icon, pressed);
        let size = if preview.is_some() {
            target
        } else {
            motion::token_value(
                (self.id.clone(), "handle-size"),
                target,
                MotionToken::FastSpatial,
                window,
                cx,
            )
        }
        .clamp(0., HANDLE_PRESSED);
        let left = handle_left(t, size);
        let paint = switch_paint(self.checked, disabled, self.error);
        let clear = Rgba {
            a: 0.,
            ..paint.layer
        };
        let (hover, press, focused) = (
            state::hovered(clear, paint.layer),
            state::pressed(clear, paint.layer),
            state::layer(clear, paint.layer, state::FOCUS),
        );
        let active_handle = keyboard_focus || pressed || forced(InteractionPreview::Hovered);
        // Positions inside the track's padding box, which starts inside the
        // outline.
        let center = left + size / 2.;
        let layer_origin = (
            center - STATE_LAYER / 2. - TRACK_OUTLINE,
            TRACK_HEIGHT / 2. - STATE_LAYER / 2. - TRACK_OUTLINE,
        );
        let glyph = if self.checked {
            IconName::Check
        } else {
            IconName::Close
        };
        let action = self.on_change;
        let release = press_state.clone();
        let release_out = press_state.clone();
        let switch = gpui_base::Switch::new(self.id)
            .track_focus(&focus)
            .accessibility_label(label)
            .checked(self.checked)
            .disabled(disabled)
            .tab_stop(preview.is_none())
            .on_change(move |value, _, window, cx| {
                if let Some(f) = &action {
                    f(&value, window, cx);
                }
            })
            .when(live, |switch| {
                switch
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        press_state.update(cx, |pressed, _| *pressed = true);
                        window.refresh();
                    })
                    .on_mouse_up(MouseButton::Left, move |_, window, cx| {
                        release.update(cx, |pressed, _| *pressed = false);
                        window.refresh();
                    })
                    .on_mouse_up_out(MouseButton::Left, move |_, window, cx| {
                        release_out.update(cx, |pressed, _| *pressed = false);
                        window.refresh();
                    })
            })
            .group(SWITCH_GROUP)
            .relative()
            .w(rem(TRACK_WIDTH))
            .h(rem(TRACK_HEIGHT))
            .rounded_full()
            .border(rem(TRACK_OUTLINE))
            .border_color(paint.outline)
            .bg(paint.track)
            .child(
                div()
                    .id("state-layer")
                    .absolute()
                    .left(rem(layer_origin.0))
                    .top(rem(layer_origin.1))
                    .size(rem(STATE_LAYER))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .when(keyboard_focus, |layer| layer.bg(focused))
                    .when(forced(InteractionPreview::Hovered), |layer| layer.bg(hover))
                    .when(pressed, |layer| layer.bg(press))
                    .when(live && !pressed && !keyboard_focus, |layer| {
                        layer.group_hover(SWITCH_GROUP, move |style| style.bg(hover))
                    })
                    .child(
                        div()
                            .id("handle")
                            .when(cfg!(test), |handle| {
                                handle.debug_selector(|| "switch-handle".into())
                            })
                            .flex()
                            .flex_shrink_0()
                            .items_center()
                            .justify_center()
                            .size(rem(size))
                            .rounded_full()
                            .bg(if active_handle {
                                paint.handle_active
                            } else {
                                paint.handle
                            })
                            .when(live, |handle| {
                                let active = paint.handle_active;
                                // A cursor gives the handle the hitbox its
                                // group hover needs.
                                handle
                                    .cursor_pointer()
                                    .group_hover(SWITCH_GROUP, move |style| style.bg(active))
                            })
                            .when(icon, |handle| {
                                handle.child(glyph.icon().size(rem(ICON)).color(paint.icon))
                            }),
                    ),
            );
        ButtonFocusRing::new(switch, focus, enabled, rem(TRACK_HEIGHT / 2.))
            .forced(forced(InteractionPreview::Focused))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_handle_stays_inside_the_outline_through_overshoot() {
        for t in [-0.3, -0.05, 0., 0.25, 0.5, 0.75, 1., 1.05, 1.3] {
            for size in [14., 16., 24., 28., 29.] {
                let left = handle_left(t, size);
                let size = f32::min(size, HANDLE_PRESSED);
                assert!(left >= TRACK_OUTLINE, "{t}: inside the leading outline");
                assert!(
                    left + size <= TRACK_WIDTH - TRACK_OUTLINE,
                    "{t}: inside the trailing outline"
                );
            }
        }
        assert_eq!(handle_left(0., 16.), 8.);
        assert_eq!(handle_left(1., 24.), 24.);
        assert_eq!(handle_left(0., 24.), 4., "an unselected icon handle");
    }

    #[test]
    fn spring_overshoot_moves_the_handle_past_rest() {
        assert!(handle_left(1.05, 24.) > handle_left(1., 24.) || handle_left(1., 24.) == 24.);
        assert!(handle_left(-0.05, 16.) < handle_left(0., 16.));
    }

    #[test]
    fn handle_sizes_follow_the_m3_spec() {
        assert_eq!(handle_size(false, false, false), 16.);
        assert_eq!(handle_size(false, true, false), 24.);
        assert_eq!(handle_size(true, false, false), 24.);
        assert_eq!(handle_size(true, true, true), 28.);
        assert_eq!(handle_size(false, false, true), 28.);
        assert!(!SwitchIcons::Selected.shows(false));
        assert!(SwitchIcons::Selected.shows(true));
        assert!(SwitchIcons::Both.shows(false));
    }

    #[test]
    fn switch_uses_m3_roles_in_every_state() {
        let c = color;
        let on = switch_paint(true, false, false);
        assert_eq!(on.track, c(ColorRole::PRIMARY));
        assert_eq!(on.handle, c(ColorRole::ON_PRIMARY));
        assert_eq!(on.handle_active, c(ColorRole::PRIMARY_CONTAINER));
        assert_eq!(on.icon, c(ColorRole::ON_PRIMARY_CONTAINER));
        let off = switch_paint(false, false, false);
        assert_eq!(off.track, c(ColorRole::SURFACE_CONTAINER_HIGHEST));
        assert_eq!(off.outline, c(ColorRole::OUTLINE));
        assert_eq!(off.handle, c(ColorRole::OUTLINE));
        assert_eq!(off.handle_active, c(ColorRole::ON_SURFACE_VARIANT));
        assert_eq!(off.icon, c(ColorRole::SURFACE_CONTAINER_HIGHEST));
        let on_surface = c(ColorRole::ON_SURFACE);
        let disabled_on = switch_paint(true, true, false);
        assert_eq!(
            disabled_on.track,
            on_surface.opacity(state::DISABLED_CONTAINER)
        );
        assert_eq!(disabled_on.handle, c(ColorRole::SURFACE));
        assert_eq!(disabled_on.icon, state::disabled(on_surface));
        let disabled_off = switch_paint(false, true, false);
        assert_eq!(disabled_off.handle, state::disabled(on_surface));
        assert_eq!(
            disabled_off.outline,
            on_surface.opacity(state::DISABLED_CONTAINER)
        );
        assert_eq!(
            disabled_off.track,
            c(ColorRole::SURFACE_CONTAINER_HIGHEST).opacity(state::DISABLED_CONTAINER)
        );
        assert_eq!(STATE_LAYER, 40.);
        assert_eq!(ICON, 16.);
    }

    #[test]
    fn an_error_switch_uses_the_error_roles_unless_disabled() {
        let c = color;
        let on = switch_paint(true, false, true);
        assert_eq!(on.track, c(ColorRole::ERROR));
        assert_eq!(on.handle, c(ColorRole::ON_ERROR));
        assert_eq!(on.handle_active, c(ColorRole::ERROR_CONTAINER));
        assert_eq!(on.icon, c(ColorRole::ON_ERROR_CONTAINER));
        assert_eq!(on.layer, c(ColorRole::ERROR));
        let off = switch_paint(false, false, true);
        assert_eq!(off.track, c(ColorRole::SURFACE_CONTAINER_HIGHEST));
        assert_eq!(off.outline, c(ColorRole::ERROR));
        assert_eq!(off.handle, c(ColorRole::ERROR));
        assert_eq!(off.handle_active, c(ColorRole::ON_ERROR_CONTAINER));
        assert_eq!(
            switch_paint(true, true, true),
            switch_paint(true, true, false)
        );
        assert_eq!(
            switch_paint(false, true, true),
            switch_paint(false, true, false)
        );
    }

    struct PressHarness(SwitchIcons, bool, bool);
    impl Render for PressHarness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().p(px(20.)).child(
                MaterialSwitch::new("press-switch", "Wi-Fi")
                    .checked(self.1)
                    .disabled(self.2)
                    .icons(self.0),
            )
        }
    }

    #[gpui::test]
    fn a_press_ended_while_disabled_does_not_stick(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| PressHarness(SwitchIcons::None, true, false));
        draw(cx);
        let rest = cx.debug_bounds("switch-handle").unwrap();
        cx.simulate_mouse_move(rest.center(), None, Modifiers::default());
        cx.simulate_mouse_down(rest.center(), MouseButton::Left, Modifiers::default());
        draw(cx);
        let set_disabled = |disabled, cx: &mut VisualTestContext| {
            view.update(cx, |view, cx| {
                view.2 = disabled;
                cx.notify();
            });
            draw(cx);
        };
        set_disabled(true, cx);
        cx.simulate_mouse_up(rest.center(), MouseButton::Left, Modifiers::default());
        draw(cx);
        set_disabled(false, cx);
        assert_eq!(
            cx.debug_bounds("switch-handle").unwrap().size,
            size(px(24.), px(24.)),
            "the handle is back at rest"
        );
    }

    #[gpui::test]
    fn pressing_grows_the_handle_to_28dp_around_its_center(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (_, cx) = cx.add_window_view(|_, _| PressHarness(SwitchIcons::None, true, false));
        draw(cx);
        let rest = cx.debug_bounds("switch-handle").unwrap();
        assert_eq!(rest.size, size(px(24.), px(24.)));
        cx.simulate_mouse_move(rest.center(), None, Modifiers::default());
        cx.simulate_mouse_down(rest.center(), MouseButton::Left, Modifiers::default());
        draw(cx);
        let pressed = cx.debug_bounds("switch-handle").unwrap();
        assert_eq!(pressed.size, size(px(28.), px(28.)));
        assert_eq!(pressed.center(), rest.center());
        cx.simulate_mouse_up(rest.center(), MouseButton::Left, Modifiers::default());
        draw(cx);
        assert_eq!(
            cx.debug_bounds("switch-handle").unwrap().size,
            size(px(24.), px(24.)),
            "releasing returns the handle to rest"
        );
    }

    #[gpui::test]
    fn an_icon_grows_the_unselected_handle_to_24dp(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let (_, cx) = cx.add_window_view(|_, _| PressHarness(SwitchIcons::Both, false, false));
        draw(cx);
        assert_eq!(
            cx.debug_bounds("switch-handle").unwrap().size,
            size(px(24.), px(24.))
        );
        let (_, cx) = cx.add_window_view(|_, _| PressHarness(SwitchIcons::Selected, false, false));
        draw(cx);
        assert_eq!(
            cx.debug_bounds("switch-handle").unwrap().size,
            size(px(16.), px(16.))
        );
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|w, cx| {
            w.refresh();
            w.draw(cx).clear(cx);
        });
    }
}
