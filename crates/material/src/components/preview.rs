//! Forced, static interaction states for specimen boards.
//!
//! Design-kit pages show every control in each of its interaction states side
//! by side. `preview_state` on an interactive component replaces its live
//! hover, focus and press tracking with one fixed [`InteractionPreview`] and
//! paints exactly what that live state paints: the same state-layer opacity
//! from [`crate::tokens::state`], the same keyboard focus ring and, for
//! `Pressed`, the pressed shape with the press layer at rest (a ripple that has
//! finished spreading covers the container at the press opacity).
//!
//! A previewed control is a specimen, not a control the person operates: it
//! leaves the tab order and ignores pointer hover and press styling, so the
//! forced state cannot be overridden by the pointer. Leaving `preview_state`
//! unset keeps every live behavior unchanged.
use crate::tokens::state;

/// One interaction state a specimen renders without live input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum InteractionPreview {
    /// At rest: no state layer, no ring.
    #[default]
    Enabled,
    /// The pointer is over the control.
    Hovered,
    /// The control has keyboard focus: focus layer (where the live control
    /// draws one) and the focus ring.
    Focused,
    /// The control is held down.
    Pressed,
    /// The control cannot be used.
    Disabled,
}

impl InteractionPreview {
    /// Every state in design-kit column order.
    pub const ALL: [Self; 5] = [
        Self::Enabled,
        Self::Hovered,
        Self::Focused,
        Self::Pressed,
        Self::Disabled,
    ];

    /// The design kit's name for the state; galleries may use it as a
    /// column label.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Hovered => "Hovered",
            Self::Focused => "Focused",
            Self::Pressed => "Pressed",
            Self::Disabled => "Disabled",
        }
    }

    pub const fn is_disabled(self) -> bool {
        matches!(self, Self::Disabled)
    }
    pub const fn is_hovered(self) -> bool {
        matches!(self, Self::Hovered)
    }
    pub const fn is_focused(self) -> bool {
        matches!(self, Self::Focused)
    }
    pub const fn is_pressed(self) -> bool {
        matches!(self, Self::Pressed)
    }

    /// The M3 state-layer opacity for this state (0 at rest and disabled).
    pub const fn layer_opacity(self) -> f32 {
        match self {
            Self::Hovered => state::HOVER,
            Self::Focused => state::FOCUS,
            Self::Pressed => state::PRESS,
            Self::Enabled | Self::Disabled => 0.,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InteractionPreview::*;
    use crate::tokens::state;

    #[test]
    fn states_use_the_state_layer_tokens_in_design_kit_order() {
        assert_eq!(
            super::InteractionPreview::ALL.map(|s| s.layer_opacity()),
            [0., state::HOVER, state::FOCUS, state::PRESS, 0.]
        );
        assert_eq!(
            super::InteractionPreview::ALL.map(|s| s.name()),
            ["Enabled", "Hovered", "Focused", "Pressed", "Disabled"]
        );
        assert!(Disabled.is_disabled() && !Pressed.is_disabled());
    }

    use crate::components::focus::test_rings;
    use crate::components::{
        AssistChip, ChoiceTile, ConnectedButtonGroup, InteractionPreview, MaterialSwitch, TileScale,
    };
    use gpui::{Context, IntoElement, Render, TestAppContext, Window, div, prelude::*, px};

    struct Specimens {
        state: InteractionPreview,
    }
    impl Render for Specimens {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            let state = self.state;
            div()
                .flex()
                .flex_col()
                .gap(px(24.))
                .p(px(24.))
                .child(AssistChip::new("specimen-chip", "Chip").preview_state(state))
                .child(
                    div().debug_selector(|| "specimen-switch".into()).child(
                        MaterialSwitch::new("specimen-switch", "Switch").preview_state(state),
                    ),
                )
                .child(
                    TileScale::Regular
                        .button("specimen-tile", [true; 4])
                        .preview_state(state)
                        .child("Tile"),
                )
                .child(
                    ChoiceTile::new("specimen-choice", 0, 1, "Choice", false)
                        .preview_state(state)
                        .render(window, cx),
                )
                .child(
                    ConnectedButtonGroup::new("specimen-group", "Group")
                        .options([("a", "A"), ("b", "B")])
                        .selected("a")
                        .preview_state("b", state),
                )
        }
    }

    fn draw(cx: &mut gpui::VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }

    #[gpui::test]
    fn forced_focus_draws_the_ring_without_keyboard_input(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let (view, cx) = cx.add_window_view(|_, _| Specimens { state: Focused });
        draw(cx);
        for part in ["specimen-chip", "specimen-tile", "specimen-choice"] {
            assert!(test_rings::painted(part).is_some(), "{part} shows its ring");
        }
        for state in [Enabled, Hovered, Pressed, Disabled] {
            view.update(cx, |view, cx| {
                view.state = state;
                cx.notify();
            });
            draw(cx);
            for part in ["specimen-chip", "specimen-tile", "specimen-choice"] {
                assert!(test_rings::painted(part).is_none(), "{part}: {state:?}");
            }
        }
    }

    #[gpui::test]
    fn specimens_are_not_tab_stops(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let (_, cx) = cx.add_window_view(|_, _| Specimens { state: Enabled });
        draw(cx);
        cx.update(|window, cx| window.focus_next(cx));
        assert!(cx.update(|window, cx| window.focused(cx)).is_none());
    }

    #[gpui::test]
    fn a_forced_press_grows_the_switch_handle(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let (view, cx) = cx.add_window_view(|_, _| Specimens { state: Pressed });
        draw(cx);
        let pressed = cx.debug_bounds("switch-handle").unwrap().size.width;
        assert_eq!(pressed, px(28.));
        view.update(cx, |view, cx| {
            view.state = Hovered;
            cx.notify();
        });
        draw(cx);
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(1));
        draw(cx);
        assert_eq!(
            cx.debug_bounds("switch-handle").unwrap().size.width,
            px(16.)
        );
    }
}
