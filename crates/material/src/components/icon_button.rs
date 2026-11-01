//! M3 Expressive icon button: standard, filled, tonal and outlined; XS–XL;
//! narrow, default and wide; round or square; optional toggle selection.
//!
//! The button is a [`MaterialButton`] in its icon form, so activation,
//! focus, keyboard, toggle semantics, the state layer, ripple, focus ring,
//! pressed-shape morph and selected-shape morph (round ⇄ square on the fast
//! spatial spring) are the labelled button's. Filled, tonal and outlined use
//! the button variants' toggle colors; the standard icon button has no
//! container and turns primary when selected.
use super::{
    ButtonShape, ButtonSize, ButtonTone, ButtonVariant, IconButtonWidth, InteractionPreview,
    MaterialButton,
    button::{Form, Palette, Roles},
};
use crate::{color::ColorRole, icons::Icon};
use gpui::{prelude::*, *};

/// Icon button color styles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum IconButtonVariant {
    /// No container: on-surface-variant, primary when selected.
    #[default]
    Standard,
    Filled,
    Tonal,
    Outlined,
}

impl IconButtonVariant {
    pub const ALL: [Self; 4] = [Self::Standard, Self::Filled, Self::Tonal, Self::Outlined];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Filled => "filled",
            Self::Tonal => "tonal",
            Self::Outlined => "outlined",
        }
    }
    /// The button variant whose colors this style shares; the standard
    /// style paints its own palette.
    fn button_variant(self) -> ButtonVariant {
        match self {
            Self::Standard => ButtonVariant::Text,
            Self::Filled => ButtonVariant::Filled,
            Self::Tonal => ButtonVariant::Tonal,
            Self::Outlined => ButtonVariant::Outlined,
        }
    }
}

const STANDARD: Palette = Palette {
    rest: Roles {
        container: None,
        content: ColorRole::ON_SURFACE_VARIANT,
    },
    selected: Some(Roles {
        container: None,
        content: ColorRole::PRIMARY,
    }),
    disabled_container: false,
    elevation: (0, 0),
};

type ChangeHandler = Box<dyn Fn(bool, &ClickEvent, &mut Window, &mut App)>;
type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// An icon-only button. `label` names it for assistive technology.
#[derive(gpui::IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: Icon,
    selected_icon: Option<Icon>,
    label: SharedString,
    variant: IconButtonVariant,
    tone: ButtonTone,
    size: ButtonSize,
    width: IconButtonWidth,
    shape: ButtonShape,
    toggle: bool,
    selected: bool,
    disabled: bool,
    preview: Option<InteractionPreview>,
    focus: Option<FocusHandle>,
    tab_stop: Option<bool>,
    on_click: Option<ClickHandler>,
    on_change: Option<ChangeHandler>,
    style: StyleRefinement,
}

impl IconButton {
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl Into<Icon>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            selected_icon: None,
            label: label.into(),
            variant: IconButtonVariant::Standard,
            tone: ButtonTone::Standard,
            size: ButtonSize::Small,
            width: IconButtonWidth::Default,
            shape: ButtonShape::Round,
            toggle: false,
            selected: false,
            disabled: false,
            preview: None,
            focus: None,
            tab_stop: None,
            on_click: None,
            on_change: None,
            style: StyleRefinement::default(),
        }
    }
    /// Which roles a filled, tonal or outlined container and its icon paint with.
    pub fn tone(mut self, tone: ButtonTone) -> Self {
        self.tone = tone;
        self
    }
    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn button_size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }
    pub fn width(mut self, width: IconButtonWidth) -> Self {
        self.width = width;
        self
    }
    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }
    /// Controlled toggle semantics (a pressed/unpressed toggle button).
    pub fn toggle(mut self, toggle: bool) -> Self {
        self.toggle = toggle;
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    /// The icon shown while selected, typically the filled symbol.
    pub fn selected_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.selected_icon = Some(icon.into());
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Render as a static specimen in one interaction state.
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }
    pub fn tab_stop(mut self, stop: bool) -> Self {
        self.tab_stop = Some(stop);
        self
    }
    pub fn on_click(
        mut self,
        callback: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(callback));
        self
    }
    /// The requested selection of a toggle icon button.
    pub fn on_change(
        mut self,
        callback: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub(crate) fn into_button(self) -> MaterialButton {
        let selected = self.toggle && self.selected;
        let icon = match self.selected_icon {
            Some(icon) if selected => icon,
            _ => self.icon,
        };
        let mut button = MaterialButton::new(self.id)
            .variant(self.variant.button_variant())
            .form(Form::Icon(self.width))
            .tone(self.tone)
            .button_size(self.size)
            .shape(self.shape)
            .icon(icon)
            .accessibility_label(self.label)
            .toggle(self.toggle)
            .selected(self.selected)
            .disabled(self.disabled);
        if self.variant == IconButtonVariant::Standard {
            button = button.palette(STANDARD);
        }
        if let Some(preview) = self.preview {
            button = button.preview_state(preview);
        }
        if let Some(focus) = &self.focus {
            button = button.track_focus(focus);
        }
        if let Some(stop) = self.tab_stop {
            button = button.tab_stop(stop);
        }
        if let Some(callback) = self.on_click {
            button = button.on_click(callback);
        }
        if let Some(callback) = self.on_change {
            button = button.on_change(callback);
        }
        *button.style() = self.style;
        button
    }
}

impl From<IconButton> for MaterialButton {
    fn from(button: IconButton) -> Self {
        button.into_button()
    }
}

impl Styled for IconButton {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for IconButton {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.into_button()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ButtonSize, ButtonVariant, IconButton, IconButtonVariant, IconButtonWidth, STANDARD,
    };
    use crate::{color::ColorRole, icons::IconName};
    use gpui::{
        Context, InteractiveElement as _, IntoElement, ParentElement as _, Render, Styled as _,
        TestAppContext, Window, div, px,
    };

    struct Harness {
        variant: IconButtonVariant,
        size: ButtonSize,
        width: IconButtonWidth,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().flex().items_start().p(px(40.)).child(
                div()
                    .flex()
                    .debug_selector(|| "icon-button-slot".into())
                    .child(
                        IconButton::new("icon", IconName::Check, "Check")
                            .variant(self.variant)
                            .button_size(self.size)
                            .width(self.width),
                    ),
            )
        }
    }

    #[gpui::test]
    fn icon_buttons_take_their_token_box(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        for (size, width, expected) in [
            (ButtonSize::ExtraSmall, IconButtonWidth::Narrow, (28., 32.)),
            (ButtonSize::Small, IconButtonWidth::Default, (40., 40.)),
            (ButtonSize::Medium, IconButtonWidth::Wide, (72., 56.)),
            (ButtonSize::ExtraLarge, IconButtonWidth::Wide, (184., 136.)),
        ] {
            let (_, cx) = cx.add_window_view(|_, _| Harness {
                variant: IconButtonVariant::Outlined,
                size,
                width,
            });
            cx.run_until_parked();
            let bounds = cx
                .debug_bounds("icon-button-slot")
                .expect("the icon button renders");
            assert_eq!(
                (
                    f32::from(bounds.size.width).round(),
                    f32::from(bounds.size.height).round()
                ),
                expected,
                "{size:?} {width:?}"
            );
        }
    }

    #[core::prelude::v1::test]
    fn standard_style_has_no_container_and_selects_primary() {
        assert_eq!(STANDARD.rest.container, None);
        assert_eq!(STANDARD.selected.unwrap().content, ColorRole::PRIMARY);
        assert_eq!(
            IconButtonVariant::Filled.button_variant(),
            ButtonVariant::Filled
        );
    }
}
