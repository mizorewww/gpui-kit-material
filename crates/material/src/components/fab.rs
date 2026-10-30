//! M3 Expressive floating action buttons: the FAB (small, regular, medium,
//! large) and the extended FAB (icon and label; regular, medium, large) with
//! its expand/collapse animation.
//!
//! Both are [`MaterialButton`]s in a FAB form, so activation, focus, the
//! state layer, ripple, focus ring and elevation transitions (level 3 at
//! rest, 4 on hover) are the button's.
use super::{
    ButtonIconPosition, FabSize, InteractionPreview, MaterialButton,
    button::{Form, Palette, Roles},
};
use crate::{
    color::ColorRole,
    icons::Icon,
    motion::{self, MotionToken},
    tokens::typography::{BASE_PX, TypeRole, rem},
};
use gpui::{prelude::*, *};

/// FAB color styles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FabColor {
    /// Surface container high with a primary icon (baseline surface FAB).
    Surface,
    #[default]
    PrimaryContainer,
    SecondaryContainer,
    TertiaryContainer,
    Primary,
    Secondary,
    Tertiary,
}

impl FabColor {
    pub const ALL: [Self; 7] = [
        Self::Surface,
        Self::PrimaryContainer,
        Self::SecondaryContainer,
        Self::TertiaryContainer,
        Self::Primary,
        Self::Secondary,
        Self::Tertiary,
    ];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Surface => "surface",
            Self::PrimaryContainer => "primary container",
            Self::SecondaryContainer => "secondary container",
            Self::TertiaryContainer => "tertiary container",
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Tertiary => "tertiary",
        }
    }
    /// (container, content) roles.
    pub const fn roles(self) -> (ColorRole, ColorRole) {
        use ColorRole as C;
        match self {
            Self::Surface => (C::SURFACE_CONTAINER_HIGH, C::PRIMARY),
            Self::PrimaryContainer => (C::PRIMARY_CONTAINER, C::ON_PRIMARY_CONTAINER),
            Self::SecondaryContainer => (C::SECONDARY_CONTAINER, C::ON_SECONDARY_CONTAINER),
            Self::TertiaryContainer => (C::TERTIARY_CONTAINER, C::ON_TERTIARY_CONTAINER),
            Self::Primary => (C::PRIMARY, C::ON_PRIMARY),
            Self::Secondary => (C::SECONDARY, C::ON_SECONDARY),
            Self::Tertiary => (C::TERTIARY, C::ON_TERTIARY),
        }
    }
    pub(crate) const fn palette(self, elevation: (u8, u8)) -> Palette {
        let (container, content) = self.roles();
        Palette {
            rest: Roles {
                container: Some(container),
                content,
            },
            selected: None,
            disabled_container: true,
            elevation,
        }
    }
}

/// FAB elevation: level 3 at rest, focused and pressed; 4 when hovered.
pub(crate) const FAB_ELEVATION: (u8, u8) = (3, 4);
/// Lowered FAB elevation: level 1 at rest, 2 when hovered.
pub(crate) const LOWERED_ELEVATION: (u8, u8) = (1, 2);

/// The (resting, hovered) elevation of a FAB, lowered or not.
pub(crate) const fn fab_elevation(lowered: bool) -> (u8, u8) {
    if lowered {
        LOWERED_ELEVATION
    } else {
        FAB_ELEVATION
    }
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

macro_rules! fab_builders {
    () => {
        pub fn size(mut self, size: FabSize) -> Self {
            self.size = size;
            self
        }
        pub fn color(mut self, color: FabColor) -> Self {
            self.color = color;
            self
        }
        /// The lowered FAB: elevation level 1 (2 on hover) instead of 3
        /// (4), for surfaces where the FAB should not float as high.
        pub fn lowered(mut self, lowered: bool) -> Self {
            self.lowered = lowered;
            self
        }
        /// Disabled: on-surface at 10% container and 38% content, no
        /// elevation, no activation.
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
        pub fn on_click(
            mut self,
            callback: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        ) -> Self {
            self.on_click = Some(Box::new(callback));
            self
        }
    };
}

/// A square FAB holding one icon. `label` names it for assistive technology.
#[derive(gpui::IntoElement)]
pub struct Fab {
    id: ElementId,
    icon: Icon,
    label: SharedString,
    size: FabSize,
    color: FabColor,
    lowered: bool,
    disabled: bool,
    preview: Option<InteractionPreview>,
    focus: Option<FocusHandle>,
    on_click: Option<ClickHandler>,
}

impl Fab {
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl Into<Icon>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            label: label.into(),
            size: FabSize::Regular,
            color: FabColor::PrimaryContainer,
            lowered: false,
            disabled: false,
            preview: None,
            focus: None,
            on_click: None,
        }
    }
    fab_builders!();
}

fn finish(
    mut button: MaterialButton,
    preview: Option<InteractionPreview>,
    focus: Option<FocusHandle>,
    on_click: Option<ClickHandler>,
) -> MaterialButton {
    if let Some(preview) = preview {
        button = button.preview_state(preview);
    }
    if let Some(focus) = &focus {
        button = button.track_focus(focus);
    }
    if let Some(callback) = on_click {
        button = button.on_click(callback);
    }
    button
}

impl RenderOnce for Fab {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        finish(
            MaterialButton::new(self.id)
                .form(Form::Fab(self.size))
                .palette(self.color.palette(fab_elevation(self.lowered)))
                .disabled(self.disabled)
                .icon(self.icon)
                .accessibility_label(self.label),
            self.preview,
            self.focus,
            self.on_click,
        )
    }
}

/// An extended FAB: icon and visible label. `expanded(false)` collapses it
/// to a FAB of the same height, the label clipping away while the padding
/// and icon-label gap close on the default spatial spring (reduced motion
/// jumps). A label-only FAB ([`ExtendedFab::label_only`]) has nothing to
/// collapse to and stays extended.
#[derive(gpui::IntoElement)]
pub struct ExtendedFab {
    id: ElementId,
    icon: Option<Icon>,
    icon_position: ButtonIconPosition,
    label: SharedString,
    size: FabSize,
    color: FabColor,
    lowered: bool,
    disabled: bool,
    fluid: bool,
    expanded: bool,
    preview: Option<InteractionPreview>,
    focus: Option<FocusHandle>,
    on_click: Option<ClickHandler>,
}

impl ExtendedFab {
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl Into<Icon>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            icon: Some(icon.into()),
            ..Self::label_only(id, label)
        }
    }
    /// An extended FAB showing only its label.
    pub fn label_only(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            icon_position: ButtonIconPosition::Leading,
            label: label.into(),
            size: FabSize::Regular,
            color: FabColor::PrimaryContainer,
            lowered: false,
            disabled: false,
            fluid: false,
            expanded: true,
            preview: None,
            focus: None,
            on_click: None,
        }
    }
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
    /// The icon before (default) or after the label.
    pub fn icon_position(mut self, position: ButtonIconPosition) -> Self {
        self.icon_position = position;
        self
    }
    /// Fill the parent's width (content centered) instead of fitting the
    /// content.
    pub fn fluid(mut self, fluid: bool) -> Self {
        self.fluid = fluid;
        self
    }
    fab_builders!();

    /// The extension the FAB animates toward: collapsing needs an icon.
    fn target_expansion(&self) -> f32 {
        if self.expanded || self.icon.is_none() {
            1.
        } else {
            0.
        }
    }
}

/// The label's width in design px.
fn label_width(label: &SharedString, role: TypeRole, window: &Window) -> f32 {
    let scale = role.scale();
    let font_size = scale.font_size.to_pixels(window.rem_size());
    let run = TextRun {
        len: label.len(),
        font: Font {
            weight: FontWeight(scale.font_weight as f32),
            ..font(scale.font_family)
        },
        color: Hsla::default(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let width = window
        .text_system()
        .shape_line(label.clone(), font_size, &[run], None)
        .width;
    f32::from(width) * BASE_PX / f32::from(window.rem_size())
}

/// A label revealed to `expansion` (0 … 1) of its width: absent at 0, laid
/// out naturally at 1, and only measured and clipped while it moves.
pub(super) fn revealed_label(
    label: &SharedString,
    role: TypeRole,
    expansion: f32,
    window: &Window,
) -> Option<AnyElement> {
    (expansion > 0.).then(|| {
        let text = div().whitespace_nowrap().child(label.clone());
        if expansion >= 1. {
            text.into_any_element()
        } else {
            let width = label_width(label, role, window) * expansion;
            div()
                .overflow_hidden()
                .flex_shrink_0()
                .w(rem(width))
                .h(role.scale().line_height)
                .opacity(expansion.clamp(0., 1.))
                .child(text)
                .into_any_element()
        }
    })
}

impl RenderOnce for ExtendedFab {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let expansion = motion::token_value(
            (self.id.clone(), "fab-expansion"),
            self.target_expansion(),
            MotionToken::DefaultSpatial,
            window,
            cx,
        );
        let (_, _, _, role) = self.size.extended_size().extended();
        let label = revealed_label(&self.label, role, expansion, window);
        finish(
            MaterialButton::new(self.id)
                .form(Form::ExtendedFab {
                    size: self.size,
                    expansion,
                })
                .palette(self.color.palette(fab_elevation(self.lowered)))
                .disabled(self.disabled)
                .when_some(self.icon, |button, icon| button.icon(icon))
                .icon_position(self.icon_position)
                .accessibility_label(self.label)
                .children(label)
                .when(self.fluid, |button| button.w_full()),
            self.preview,
            self.focus,
            self.on_click,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{ExtendedFab, FAB_ELEVATION, Fab, FabColor, FabSize, fab_elevation};
    use crate::components::ButtonIconPosition;
    use crate::{color::ColorRole, icons::IconName, motion};
    use gpui::{
        Context, InteractiveElement as _, IntoElement, ParentElement as _, Render, Styled as _,
        TestAppContext, VisualTestContext, Window, div, px, size,
    };
    use std::time::Duration;

    struct Harness {
        expanded: bool,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div()
                .flex()
                .flex_col()
                .items_start()
                .p(px(40.))
                .gap(px(20.))
                .child(
                    div()
                        .flex()
                        .debug_selector(|| "fab".into())
                        .child(Fab::new("fab", IconName::Check, "Compose").size(FabSize::Large)),
                )
                .child(
                    div().flex().debug_selector(|| "extended".into()).child(
                        ExtendedFab::new("extended", IconName::Check, "Compose")
                            .size(FabSize::Medium)
                            .expanded(self.expanded),
                    ),
                )
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }

    #[gpui::test]
    fn extended_fab_collapses_to_a_square_through_intermediate_widths(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(false);
            motion::set_motion_scheme(motion::MotionScheme::Expressive);
        });
        let (view, cx) = cx.add_window_view(|_, _| Harness { expanded: true });
        draw(cx);
        let fab = cx.debug_bounds("fab").unwrap();
        assert_eq!(fab.size, size(px(96.), px(96.)));
        let extended = cx.debug_bounds("extended").unwrap();
        assert_eq!(extended.size.height, px(80.));
        assert!(extended.size.width > px(140.), "{extended:?}");
        view.update(cx, |view, cx| {
            view.expanded = false;
            cx.notify();
        });
        draw(cx);
        cx.executor().advance_clock(Duration::from_millis(80));
        draw(cx);
        let moving = cx.debug_bounds("extended").unwrap().size.width;
        assert!(
            moving < extended.size.width && moving > px(80.),
            "{moving:?}"
        );
        cx.executor().advance_clock(Duration::from_secs(2));
        draw(cx);
        draw(cx);
        let collapsed = cx.debug_bounds("extended").unwrap().size;
        assert_eq!(collapsed, size(px(80.), px(80.)));
    }

    #[core::prelude::v1::test]
    fn color_styles_pair_containers_with_their_content() {
        assert_eq!(
            FabColor::TertiaryContainer.roles(),
            (
                ColorRole::TERTIARY_CONTAINER,
                ColorRole::ON_TERTIARY_CONTAINER
            )
        );
        assert_eq!(
            FabColor::Surface.roles(),
            (ColorRole::SURFACE_CONTAINER_HIGH, ColorRole::PRIMARY)
        );
        assert_eq!(FabColor::Primary.palette(FAB_ELEVATION).elevation, (3, 4));
    }

    #[core::prelude::v1::test]
    fn lowered_fabs_float_two_levels_lower() {
        assert_eq!(fab_elevation(false), (3, 4));
        assert_eq!(fab_elevation(true), (1, 2));
        let fab = Fab::new("fab", IconName::Check, "Add").lowered(true);
        assert!(fab.lowered);
    }

    #[core::prelude::v1::test]
    fn only_a_fab_with_an_icon_collapses() {
        let collapsed = ExtendedFab::new("fab", IconName::Check, "Compose").expanded(false);
        assert_eq!(collapsed.target_expansion(), 0.);
        let label_only = ExtendedFab::label_only("fab", "Compose").expanded(false);
        assert_eq!(label_only.target_expansion(), 1.);
    }

    struct Variants {
        clicks: std::rc::Rc<std::cell::Cell<usize>>,
    }
    impl Render for Variants {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            let clicks = self.clicks.clone();
            div()
                .w(px(400.))
                .flex()
                .flex_col()
                .items_start()
                .gap(px(20.))
                .child(
                    div().flex().debug_selector(|| "end".into()).child(
                        ExtendedFab::new("end", IconName::Check, "Compose")
                            .icon_position(ButtonIconPosition::Trailing),
                    ),
                )
                .child(
                    div()
                        .flex()
                        .debug_selector(|| "label-only".into())
                        .child(ExtendedFab::label_only("label-only", "Compose").expanded(false)),
                )
                .child(
                    div()
                        .w_full()
                        .flex()
                        .debug_selector(|| "fluid".into())
                        .child(ExtendedFab::new("fluid", IconName::Check, "Compose").fluid(true)),
                )
                .child(
                    div().flex().debug_selector(|| "disabled".into()).child(
                        Fab::new("disabled", IconName::Check, "Add")
                            .disabled(true)
                            .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
                    ),
                )
        }
    }

    #[gpui::test]
    fn extended_fab_options_lay_out_and_disabled_fabs_ignore_clicks(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(true);
        });
        let clicks = std::rc::Rc::new(std::cell::Cell::new(0));
        let (_, cx) = cx.add_window_view({
            let clicks = clicks.clone();
            move |_, _| Variants { clicks }
        });
        draw(cx);
        draw(cx);
        let label_only = cx.debug_bounds("label-only").unwrap().size;
        assert_eq!(label_only.height, px(56.));
        assert!(
            label_only.width > px(56.),
            "a label-only FAB stays extended"
        );
        assert_eq!(cx.debug_bounds("fluid").unwrap().size.width, px(400.));
        let end = cx.debug_bounds("end").unwrap().size;
        assert!(end.width > px(100.));
        let disabled = cx.debug_bounds("disabled").unwrap();
        cx.simulate_click(disabled.center(), gpui::Modifiers::none());
        draw(cx);
        assert_eq!(clicks.get(), 0);
    }
}
