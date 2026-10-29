//! M3 Expressive split button: a leading action segment and a trailing menu
//! segment, 2dp apart, in the filled, tonal, outlined and elevated styles and
//! the XS–XL button sizes.
//!
//! Each segment is a [`MaterialButton`] in its split-segment form: outer ends
//! stay fully round while the inner corners widen on hover/focus and press
//! (fast spatial spring). Opening the menu turns the trailing segment fully
//! round and rotates its chevron half a turn. The menu itself belongs to
//! the caller, which owns `open` and receives requested changes.
use super::{
    ButtonShape, ButtonSize, ButtonVariant, InteractionPreview, MaterialButton,
    button::{Form, Segment, segment_metrics},
};
use crate::{
    icons::{Icon, IconName},
    motion::{self, MotionToken},
};
use gpui::{prelude::*, *};
use std::rc::Rc;

/// One segment of a split button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SplitButtonPart {
    Leading,
    Trailing,
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type ToggleHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

#[derive(gpui::IntoElement)]
pub struct SplitButton {
    id: ElementId,
    label: SharedString,
    menu_label: SharedString,
    icon: Option<Icon>,
    variant: ButtonVariant,
    size: ButtonSize,
    open: bool,
    disabled: bool,
    preview: Option<(SplitButtonPart, InteractionPreview)>,
    on_click: Option<ClickHandler>,
    on_toggle: Option<ToggleHandler>,
}

impl SplitButton {
    /// `label` is the action's visible label; `menu_label` names the menu
    /// segment for assistive technology.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        menu_label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            menu_label: menu_label.into(),
            icon: None,
            variant: ButtonVariant::Filled,
            size: ButtonSize::Small,
            open: false,
            disabled: false,
            preview: None,
            on_click: None,
            on_toggle: None,
        }
    }
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    /// Filled, tonal, outlined or elevated (text has no split form).
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn button_size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }
    /// Whether the caller's menu is open (the trailing segment is selected).
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Render as a static specimen with `part` in one interaction state;
    /// the other segment stays at rest.
    pub fn preview_state(mut self, part: SplitButtonPart, state: InteractionPreview) -> Self {
        self.preview = Some((part, state));
        self
    }
    /// The leading action.
    pub fn on_click(
        mut self,
        callback: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(callback));
        self
    }
    /// The requested menu state when the trailing segment is activated.
    pub fn on_toggle(mut self, callback: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(callback));
        self
    }

    fn preview_of(&self, part: SplitButtonPart) -> Option<InteractionPreview> {
        self.preview.map(|(previewed, state)| {
            if previewed == part {
                state
            } else if state.is_disabled() {
                InteractionPreview::Disabled
            } else {
                InteractionPreview::Enabled
            }
        })
    }
}

/// The chevron's rotation in turns: half a turn while the menu is open.
fn chevron_turns(open: bool) -> f32 {
    if open { 0.5 } else { 0. }
}

impl RenderOnce for SplitButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let turns = motion::token_value(
            (self.id.clone(), "split-chevron"),
            chevron_turns(self.open),
            MotionToken::FastSpatial,
            window,
            cx,
        );
        let metrics = segment_metrics(self.size);
        let leading_preview = self.preview_of(SplitButtonPart::Leading);
        let trailing_preview = self.preview_of(SplitButtonPart::Trailing);
        let mut leading = MaterialButton::new((self.id.clone(), "split-leading"))
            .variant(self.variant)
            .button_size(self.size)
            .shape(ButtonShape::Round)
            .form(Form::Segment(Segment::Leading))
            .disabled(self.disabled)
            .when_some(self.icon, |button, icon| button.icon(icon))
            .child(self.label);
        if let Some(state) = leading_preview {
            leading = leading.preview_state(state);
        }
        if let Some(callback) = self.on_click {
            leading = leading.on_click(move |event, window, cx| callback(event, window, cx));
        }
        let chevron = Icon::from(IconName::ChevronDown)
            .size(crate::tokens::typography::rem(metrics.trailing_icon))
            .rotate(turns * std::f32::consts::TAU);
        let open = self.open;
        let mut trailing = MaterialButton::new((self.id.clone(), "split-trailing"))
            .variant(self.variant)
            .button_size(self.size)
            .form(Form::Segment(Segment::Trailing))
            .open(open)
            .disabled(self.disabled)
            .icon(chevron)
            .accessibility_label(self.menu_label)
            .aria_expanded(open);
        if let Some(state) = trailing_preview {
            trailing = trailing.preview_state(state);
        }
        if let Some(callback) = self.on_toggle {
            trailing = trailing.on_click(move |_, window, cx| callback(!open, window, cx));
        }
        div()
            .id(self.id)
            .role(Role::Group)
            .flex()
            .flex_row()
            .flex_shrink_0()
            .items_center()
            .gap(crate::tokens::typography::rem(2.))
            .child(leading)
            .child(trailing)
    }
}

#[cfg(test)]
mod tests {
    use super::{SplitButton, SplitButtonPart, chevron_turns};
    use crate::components::{ButtonSize, InteractionPreview};
    use gpui::{
        Context, InteractiveElement as _, IntoElement, ParentElement as _, Render, Styled as _,
        TestAppContext, VisualTestContext, Window, div, px,
    };

    struct Harness {
        size: ButtonSize,
        open: bool,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().flex().items_start().p(px(40.)).child(
                div().flex().debug_selector(|| "split".into()).child(
                    SplitButton::new("split", "Label", "More")
                        .button_size(self.size)
                        .open(self.open)
                        .preview_state(SplitButtonPart::Trailing, InteractionPreview::Enabled),
                ),
            )
        }
    }

    #[gpui::test]
    fn segments_share_the_button_height_with_a_two_dp_gap(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        for (size, height) in [
            (ButtonSize::ExtraSmall, 32.),
            (ButtonSize::Medium, 56.),
            (ButtonSize::ExtraLarge, 136.),
        ] {
            let (_, cx) = cx.add_window_view(|_, _| Harness { size, open: true });
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
            let bounds = cx.debug_bounds("split").unwrap();
            assert_eq!(bounds.size.height, px(height), "{size:?}");
            let _: &mut VisualTestContext = cx;
        }
    }

    #[core::prelude::v1::test]
    fn the_chevron_turns_half_way_while_open() {
        assert_eq!(chevron_turns(false), 0.);
        assert_eq!(chevron_turns(true), 0.5);
    }

    #[core::prelude::v1::test]
    fn a_previewed_segment_leaves_its_neighbor_at_rest() {
        let split = SplitButton::new("s", "Label", "More")
            .preview_state(SplitButtonPart::Leading, InteractionPreview::Pressed);
        assert_eq!(
            split.preview_of(SplitButtonPart::Leading),
            Some(InteractionPreview::Pressed)
        );
        assert_eq!(
            split.preview_of(SplitButtonPart::Trailing),
            Some(InteractionPreview::Enabled)
        );
        let disabled = SplitButton::new("s", "Label", "More")
            .preview_state(SplitButtonPart::Leading, InteractionPreview::Disabled);
        assert_eq!(
            disabled.preview_of(SplitButtonPart::Trailing),
            Some(InteractionPreview::Disabled)
        );
    }
}
