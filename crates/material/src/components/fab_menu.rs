//! M3 Expressive FAB menu: a FAB that opens a stack of related actions.
//!
//! Opening morphs the FAB into the round close button (container → solid
//! color, 16dp → full corners, icon turning into the close icon) while the
//! items, end-aligned above it, expand from the FAB's size into labelled
//! pills, nearest first, on the default spatial spring. Closing reverses
//! the stagger from the farthest item. Under reduced motion every change
//! finishes at once.
//!
//! The toggle and items are [`MaterialButton`]s, so activation, focus,
//! state layers, ripples and focus rings are the button's. The caller owns
//! `open`; Escape inside the menu requests closing.
use super::{
    FabColor, InteractionPreview, MaterialButton,
    button::{Form, Palette, Roles},
    fab::{FAB_ELEVATION, revealed_label},
};
use crate::{
    icons::{Icon, IconName},
    motion::{self, MotionToken},
    tokens::typography::{TypeRole, rem},
};
use gpui::{prelude::*, *};
use std::{rc::Rc, time::Duration};

/// The FAB menu color styles: the closed FAB and the items use the
/// container color, the close button the solid color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FabMenuColor {
    #[default]
    Primary,
    Secondary,
    Tertiary,
}

impl FabMenuColor {
    pub const ALL: [Self; 3] = [Self::Primary, Self::Secondary, Self::Tertiary];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Tertiary => "tertiary",
        }
    }
    /// (container style, solid style).
    const fn styles(self) -> (FabColor, FabColor) {
        match self {
            Self::Primary => (FabColor::PrimaryContainer, FabColor::Primary),
            Self::Secondary => (FabColor::SecondaryContainer, FabColor::Secondary),
            Self::Tertiary => (FabColor::TertiaryContainer, FabColor::Tertiary),
        }
    }
    fn item_palette(self) -> Palette {
        let (container, content) = self.styles().0.roles();
        Palette {
            rest: Roles {
                container: Some(container),
                content,
            },
            selected: None,
            disabled_container: true,
            elevation: (0, 0),
        }
    }
    fn toggle_palette(self, open: bool) -> Palette {
        let (closed, opened) = self.styles();
        if open { opened } else { closed }.palette(FAB_ELEVATION)
    }
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type ToggleHandler = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// One action of a FAB menu. Rendered on its own (e.g. a building-block
/// specimen) it is the open item in its `color`; inside a menu it takes the
/// menu's color.
#[derive(gpui::IntoElement)]
pub struct FabMenuItem {
    id: ElementId,
    icon: Icon,
    label: SharedString,
    color: FabMenuColor,
    preview: Option<InteractionPreview>,
    on_click: Option<ClickHandler>,
}

impl FabMenuItem {
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl Into<Icon>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            label: label.into(),
            color: FabMenuColor::Primary,
            preview: None,
            on_click: None,
        }
    }
    /// The color of a stand-alone item (a menu's items take its color).
    pub fn color(mut self, color: FabMenuColor) -> Self {
        self.color = color;
        self
    }
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
    pub fn on_click(
        mut self,
        callback: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(callback));
        self
    }
}

impl FabMenuItem {
    /// The item at a 0 … 1 `expansion` from FAB size to labelled pill.
    fn button(self, expansion: f32, window: &Window) -> MaterialButton {
        let label = revealed_label(&self.label, TypeRole::TitleMedium, expansion, window);
        let mut button = MaterialButton::new(self.id)
            .form(Form::FabMenuItem { expansion })
            .palette(self.color.item_palette())
            .icon(self.icon)
            .accessibility_label(self.label)
            .children(label)
            .opacity(expansion.clamp(0., 1.));
        if let Some(state) = self.preview {
            button = button.preview_state(state);
        }
        if let Some(callback) = self.on_click {
            button = button.on_click(move |event, window, cx| callback(event, window, cx));
        }
        button
    }
}

impl RenderOnce for FabMenuItem {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        self.button(1., window)
    }
}

#[derive(gpui::IntoElement)]
pub struct FabMenu {
    id: ElementId,
    icon: Icon,
    close_icon: Icon,
    label: SharedString,
    color: FabMenuColor,
    open: bool,
    items: Vec<FabMenuItem>,
    preview: Option<InteractionPreview>,
    on_toggle: Option<ToggleHandler>,
}

/// Delay between neighbouring items' entrances and exits.
const STAGGER: Duration = Duration::from_millis(30);
/// Space between the items and the button, and between items.
const BUTTON_GAP: f32 = 8.;
const ITEM_GAP: f32 = 4.;

impl FabMenu {
    /// `icon` is the closed FAB's; `label` names the toggle for assistive
    /// technology.
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl Into<Icon>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            close_icon: IconName::Close.into(),
            label: label.into(),
            color: FabMenuColor::Primary,
            open: false,
            items: Vec::new(),
            preview: None,
            on_toggle: None,
        }
    }
    pub fn close_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.close_icon = icon.into();
        self
    }
    pub fn color(mut self, color: FabMenuColor) -> Self {
        self.color = color;
        self
    }
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    pub fn item(mut self, item: FabMenuItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = FabMenuItem>) -> Self {
        self.items.extend(items);
        self
    }
    /// Render the toggle as a static specimen in one interaction state.
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
    /// The requested open state (toggle activation, Escape).
    pub fn on_toggle(mut self, callback: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(callback));
        self
    }
}

/// The entrance (exit) delay of item `index` of `count`, counted from the
/// top: opening starts nearest the FAB, closing farthest from it.
fn stagger(index: usize, count: usize, open: bool) -> Duration {
    let steps = if open { count - 1 - index } else { index };
    STAGGER * steps as u32
}

/// The toggle icon for a 0 (FAB icon) … 1 (close icon) morph amount: the
/// outgoing icon turns a quarter away as the incoming one turns in.
fn toggle_icon(icon: Icon, close: Icon, morph: f32) -> Icon {
    let quarter = std::f32::consts::FRAC_PI_2;
    if morph >= 0.5 {
        close.rotate(-(1. - morph) * quarter)
    } else {
        icon.rotate(morph * quarter)
    }
}

impl RenderOnce for FabMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let open = self.open;
        let count = self.items.len();
        let curve = motion::curve(MotionToken::DefaultSpatial);
        let mut column = div().flex().flex_col().items_end().gap(rem(ITEM_GAP));
        let mut any_item = false;
        for (index, item) in self.items.into_iter().enumerate() {
            let expansion = gpui_base::transition(
                ElementId::from((item.id.clone(), "fab-menu-expansion")),
                if open { 1f32 } else { 0. },
                gpui_base::Transition::new(curve.duration())
                    .ease(curve.ease())
                    .delay(stagger(index, count, open)),
                window,
                cx,
            );
            // A settled closed menu lays out no items.
            if !open && expansion <= 0. {
                continue;
            }
            any_item = true;
            column = column.child(
                item.color(self.color)
                    .button(expansion, window)
                    .tab_stop(open),
            );
        }
        let morph = motion::token_value(
            (self.id.clone(), "fab-menu-icon"),
            if open { 1. } else { 0. },
            MotionToken::FastSpatial,
            window,
            cx,
        );
        let mut toggle = MaterialButton::new((self.id.clone(), "fab-menu-toggle"))
            .form(Form::FabMenuToggle { open })
            .palette(self.color.toggle_palette(open))
            .icon(toggle_icon(self.icon, self.close_icon, morph))
            .accessibility_label(self.label)
            .aria_expanded(open);
        if let Some(state) = self.preview {
            toggle = toggle.preview_state(state);
        }
        let on_toggle = self.on_toggle;
        if let Some(callback) = on_toggle.clone() {
            toggle = toggle.on_click(move |_, window, cx| callback(&!open, window, cx));
        }
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .items_end()
            .gap(rem(BUTTON_GAP))
            .when_some(on_toggle.filter(|_| open), |menu, callback| {
                menu.on_key_down(move |event, window, cx| {
                    if event.keystroke.key == "escape" {
                        cx.stop_propagation();
                        callback(&false, window, cx);
                    }
                })
            })
            .when(any_item, |menu| menu.child(column))
            .child(toggle)
    }
}

#[cfg(test)]
mod tests {
    use super::{FabMenu, FabMenuColor, FabMenuItem, stagger};
    use crate::{color::ColorRole, icons::IconName, motion};
    use gpui::{
        Context, InteractiveElement as _, IntoElement, ParentElement as _, Render, Styled as _,
        TestAppContext, VisualTestContext, Window, div, px,
    };
    use std::time::Duration;

    #[core::prelude::v1::test]
    fn items_stagger_from_the_fab_and_close_from_the_top() {
        assert_eq!(stagger(2, 3, true), Duration::ZERO, "nearest opens first");
        assert_eq!(stagger(0, 3, true), Duration::from_millis(60));
        assert_eq!(
            stagger(0, 3, false),
            Duration::ZERO,
            "farthest closes first"
        );
        assert_eq!(stagger(2, 3, false), Duration::from_millis(60));
    }

    #[core::prelude::v1::test]
    fn the_close_button_takes_the_solid_color() {
        let closed = FabMenuColor::Tertiary.toggle_palette(false);
        let open = FabMenuColor::Tertiary.toggle_palette(true);
        assert_eq!(closed.rest.container, Some(ColorRole::TERTIARY_CONTAINER));
        assert_eq!(open.rest.container, Some(ColorRole::TERTIARY));
        assert_eq!(
            FabMenuColor::Tertiary.item_palette().rest.container,
            Some(ColorRole::TERTIARY_CONTAINER)
        );
    }

    struct Harness {
        open: bool,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            window.set_rem_size(px(14.));
            div().flex().items_start().p(px(40.)).child(
                div().flex().debug_selector(|| "menu".into()).child(
                    FabMenu::new("menu", IconName::Check, "Actions")
                        .open(self.open)
                        .items((0..3usize).map(|index| {
                            FabMenuItem::new(("item", index), IconName::Check, "Action")
                        })),
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
    fn opening_grows_the_stack_and_closing_removes_it(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_base::init(cx);
            cx.set_reduce_motion(false);
            motion::set_motion_scheme(motion::MotionScheme::Expressive);
        });
        let (view, cx) = cx.add_window_view(|_, _| Harness { open: false });
        draw(cx);
        let closed = cx.debug_bounds("menu").unwrap().size;
        assert_eq!(closed.height, px(56.));
        view.update(cx, |view, cx| {
            view.open = true;
            cx.notify();
        });
        draw(cx);
        cx.executor().advance_clock(Duration::from_secs(2));
        draw(cx);
        draw(cx);
        let open = cx.debug_bounds("menu").unwrap().size;
        assert_eq!(open.height, px(56. * 4. + 8. + 4. * 2.));
        assert!(open.width > px(56.));
        view.update(cx, |view, cx| {
            view.open = false;
            cx.notify();
        });
        draw(cx);
        cx.executor().advance_clock(Duration::from_secs(2));
        draw(cx);
        draw(cx);
        assert_eq!(cx.debug_bounds("menu").unwrap().size.height, px(56.));
    }
}
