//! Shared presentation adapter. Base Button remains the only owner of activation
//! and accessibility, a toggle reporting its state through `aria_toggled` because
//! Base's Toggle has no focus policy; no selection state is retained here.
use gpui::{prelude::*, *};
use std::rc::Rc;

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
pub(super) type ChangeHandler = Rc<dyn Fn(bool, &ClickEvent, &mut Window, &mut App)>;

#[derive(gpui::IntoElement)]
pub(super) struct BaseControl {
    #[cfg(test)]
    pub a11y_observer: Option<Rc<std::cell::RefCell<Vec<accesskit::Node>>>>,
    id: ElementId,
    interaction: Stateful<Div>,
    style: StyleRefinement,
    children: Vec<AnyElement>,
    disabled: bool,
    busy: bool,
    label: Option<SharedString>,
    focus: Option<FocusHandle>,
    focusable: bool,
    tab_index: isize,
    tab_stop: bool,
    on_click: Option<ClickHandler>,
    pub toggle: bool,
    pub selected: bool,
    pub toggle_on_click: bool,
    pub on_change: Option<ChangeHandler>,
}

impl BaseControl {
    pub fn new(id: ElementId) -> Self {
        Self {
            #[cfg(test)]
            a11y_observer: None,
            interaction: div().id(id.clone()),
            id,
            style: StyleRefinement::default(),
            children: vec![],
            disabled: false,
            busy: false,
            label: None,
            focus: None,
            focusable: true,
            tab_index: 0,
            tab_stop: true,
            on_click: None,
            toggle: false,
            selected: false,
            toggle_on_click: true,
            on_change: None,
        }
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn busy(mut self, value: bool) -> Self {
        self.busy = value;
        self
    }
    pub fn accessibility_label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
    }
    pub fn track_focus(mut self, value: &FocusHandle) -> Self {
        self.focus = Some(value.clone());
        self
    }
    pub fn focusable(mut self, value: bool) -> Self {
        self.focusable = value;
        self
    }
    pub fn tab_index(mut self, value: isize) -> Self {
        self.tab_index = value;
        self
    }
    pub fn tab_stop(mut self, value: bool) -> Self {
        self.tab_stop = value;
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
impl Styled for BaseControl {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl ParentElement for BaseControl {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}
impl InteractiveElement for BaseControl {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.interaction.interactivity()
    }
}
impl StatefulInteractiveElement for BaseControl {}
impl RenderOnce for BaseControl {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use gpui_base::StyledExt;
        #[cfg(test)]
        let a11y_observer = self.a11y_observer.take();
        #[cfg(not(test))]
        let a11y_observer = ();
        // Transfer caller and passive visual listeners onto the one Base root.
        let mut base = gpui_base::Button::new(self.id);
        std::mem::swap(base.interactivity(), self.interaction.interactivity());
        let busy = self.busy;
        let base = base
            .disabled(self.disabled)
            .focusable(self.focusable)
            .tab_index(self.tab_index)
            .tab_stop(self.tab_stop)
            .when_some(self.focus.as_ref(), |base, focus| base.track_focus(focus))
            .when_some(self.label, |base, label| base.accessibility_label(label))
            .children(self.children)
            .refine_style(&self.style);
        let on_click = self.on_click;
        let base = if self.toggle {
            let selected = self.selected;
            let on_change = self.on_change.filter(|_| self.toggle_on_click);
            base.aria_toggled(if selected {
                Toggled::True
            } else {
                Toggled::False
            })
            .when(on_click.is_some() || on_change.is_some(), |base| {
                base.on_click(move |event, window, cx| {
                    if let Some(callback) = &on_change {
                        callback(!selected, event, window, cx);
                    }
                    if let Some(callback) = &on_click {
                        callback(event, window, cx);
                    }
                })
            })
        } else {
            base.when_some(on_click, |base, callback| {
                base.on_click(move |event, window, cx| callback(event, window, cx))
            })
        };
        finish(base, busy, a11y_observer, window, cx)
    }
}

#[cfg(test)]
type A11yObserver = Option<Rc<std::cell::RefCell<Vec<accesskit::Node>>>>;
#[cfg(not(test))]
type A11yObserver = ();

/// Render the configured Base root once; tests also record its a11y node.
fn finish(
    base: impl RenderOnce,
    busy: bool,
    #[cfg_attr(not(test), allow(unused_variables))] observer: A11yObserver,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let element = Busy {
        inner: RenderOnce::render(base, window, cx).into_element(),
        busy,
    };
    #[cfg(test)]
    if let Some(observer) = &observer {
        let mut node = accesskit::Node::new(Role::Button);
        element.write_a11y_info(&mut node);
        observer.borrow_mut().push(node);
    }
    if busy {
        element.into_any_element()
    } else {
        element.inner.into_any_element()
    }
}

/// A control's root that reports itself busy on its accessibility node, which GPUI
/// has no setter for. It adds no layout, focus stop or node of its own. Only
/// `accesskit_windows` reads the flag today, as the UIA `busy` ARIA property.
struct Busy<E> {
    inner: E,
    busy: bool,
}

impl<E: Element> IntoElement for Busy<E> {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl<E: Element> Element for Busy<E> {
    type RequestLayoutState = E::RequestLayoutState;
    type PrepaintState = E::PrepaintState;
    fn id(&self) -> Option<ElementId> {
        self.inner.id()
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        self.inner.source_location()
    }
    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.inner.request_layout(id, inspector, window, cx)
    }
    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.inner
            .prepaint(id, inspector, bounds, layout, window, cx)
    }
    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.inner
            .paint(id, inspector, bounds, layout, prepaint, window, cx)
    }
    fn a11y_role(&self) -> Option<Role> {
        self.inner.a11y_role()
    }
    fn write_a11y_info(&self, node: &mut accesskit::Node) {
        self.inner.write_a11y_info(node);
        if self.busy {
            node.set_busy();
        }
    }
    fn a11y_synthetic_children(
        &mut self,
        prepaint: &mut Self::PrepaintState,
        builder: &mut A11ySubtreeBuilder,
    ) {
        self.inner.a11y_synthetic_children(prepaint, builder);
    }
}
