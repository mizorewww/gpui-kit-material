//! An element's bounds, reported as it is prepainted, for code that measures it.
//!
//! GPUI's `on_children_prepainted` hands a child's bounds out too, but GPUI Fast
//! cannot tell what is done with them, so it keeps every scroll container around
//! such a listener off its scroll layer: a virtual list holding one renders all
//! its visible rows on every frame of a scroll. This element takes no layout of
//! its own and does not do that.
//!
//! It is not called for a row a layer keeps, which is prepainted no more than it
//! is drawn. Use it for what only changes when the row is drawn again: a size, or
//! an offset within the row. A focus change refreshes the window, so what happens
//! on one is seen too. An absolute position in the window can be stale.
use gpui::*;

/// `child`, calling `on_prepaint` with its bounds once it is prepainted.
pub fn prepaint_bounds(
    child: impl IntoElement,
    on_prepaint: impl FnOnce(Bounds<Pixels>, &mut Window, &mut App) + 'static,
) -> PrepaintBounds {
    PrepaintBounds {
        child: child.into_any_element(),
        on_prepaint: Some(Box::new(on_prepaint)),
    }
}

type OnPrepaint = Box<dyn FnOnce(Bounds<Pixels>, &mut Window, &mut App)>;

/// See [`prepaint_bounds`].
pub struct PrepaintBounds {
    child: AnyElement,
    on_prepaint: Option<OnPrepaint>,
}

impl IntoElement for PrepaintBounds {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for PrepaintBounds {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.prepaint(window, cx);
        if let Some(on_prepaint) = self.on_prepaint.take() {
            on_prepaint(bounds, window, cx);
        }
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::prepaint_bounds;
    use core::prelude::v1::test;
    use gpui::{prelude::*, *};
    use std::{cell::Cell, rc::Rc};

    struct Measured {
        seen: Rc<Cell<Option<Bounds<Pixels>>>>,
        inner: Rc<Cell<bool>>,
    }
    impl Render for Measured {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let (seen, inner) = (self.seen.clone(), self.inner.clone());
            let prepainted = inner.clone();
            div().p(px(10.)).child(prepaint_bounds(
                div().w(px(30.)).h(px(20.)).child(
                    canvas(move |_, _, _| prepainted.set(true), |_, _, _, _| {}).size_full(),
                ),
                move |bounds, _, _| {
                    assert!(inner.get(), "reported after the child's prepaint");
                    seen.set(Some(bounds));
                },
            ))
        }
    }

    #[gpui::test]
    fn it_reports_its_childs_bounds_once_that_is_prepainted(cx: &mut TestAppContext) {
        let seen = Rc::new(Cell::new(None));
        let inner = Rc::new(Cell::new(false));
        let (_, cx) = cx.add_window_view(|_, _| Measured {
            seen: seen.clone(),
            inner,
        });
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let bounds = seen.get().expect("reported");
        assert_eq!(bounds.origin, point(px(10.), px(10.)));
        assert_eq!(bounds.size, size(px(30.), px(20.)));
    }

    /// `on_children_prepainted` in a component keeps every scroll container
    /// around it off its layer, so a list of them never stops rendering rows.
    #[test]
    fn components_a_list_may_hold_do_not_keep_it_off_its_layer() {
        for (name, source) in [
            ("tabs.rs", include_str!("tabs.rs")),
            ("focus.rs", include_str!("focus.rs")),
            ("menu.rs", include_str!("menu.rs")),
        ] {
            let production = source.split("#[cfg(test)]\nmod tests").next().unwrap();
            assert!(
                !production.contains(concat!(".on_children", "_prepainted(")),
                "{name} should use prepaint_bounds"
            );
        }
    }
}
