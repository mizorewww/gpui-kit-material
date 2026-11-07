//! Scoped theme: a subtree painting with its own scheme.
//!
//! The application paints one scheme ([`crate::theme`]), and a [`Scope`]
//! overrides it for a subtree.
//!
//! GPUI draws in three phases, so a scope is an element that re-enters itself
//! around each of them: request_layout (which renders every `RenderOnce`
//! descendant and view), prepaint (lists render their items here) and paint
//! (canvases read colors here). Draws a descendant defers, such as popovers,
//! run after the scope has left the stack, so Material's overlays wrap what they
//! defer in [`current_scope`], which brings it back in.
//!
//! Inside a scope [`crate::theme::color`], [`crate::theme::scheme`] and
//! [`crate::theme::is_dark`] answer from its scheme. A change of scheme blends on
//! the same effects spring as the application's, per scope, and reduced motion
//! lands it at once.
//!
//! # Limits
//!
//! A scope reaches what paints with Material's roles and draws through it. These
//! use the application's scheme:
//!
//! - Kit controls. A scope leaves Kit's theme globals (`cx.theme()` and Base's)
//!   alone, so a Kit control inside one, Kit's `Root` overlays, its tooltips and
//!   its scrollbar styles paint with the application's theme. Giving them the
//!   scope's would change a global for part of a draw, and every way GPUI offers
//!   to change one notifies its observers and, under GPUI Fast, rebuilds every
//!   view that read it. Material's text inputs draw through Kit's input and give
//!   it their scoped caret and selection colors themselves. Material's dialogs,
//!   sheets, menus and tooltips draw through the subtree and are covered.
//! - Drag previews, unless the preview is built with [`current_scope`]. A
//!   [`ChipSet`](crate::components::ChipSet)'s are.
//! - An element built before the scope is entered reads the outer scheme for
//!   any color it computed while being built. Build through [`Scope::child_with`]
//!   or [`ResolvedScope::build`]. Material components compute colors when they
//!   render, so passing one to [`Scope::child`] is fine.
use crate::color::Scheme;
use crate::motion::{self, MotionToken};
use crate::theme;
use gpui::*;
use std::cell::RefCell;
use std::rc::Rc;

/// A horizontal arrow key's effect: `1` for right (forward), `-1` for left
/// (back). Other keys answer `None`.
pub fn horizontal_arrow(key: &str) -> Option<i32> {
    match key {
        "right" => Some(1),
        "left" => Some(-1),
        _ => None,
    }
}

type BuildChild = Box<dyn FnOnce(&mut Window, &mut App) -> AnyElement>;

/// A subtree's scheme override.
///
/// # Example
///
/// A dark card inside a light window:
///
/// ```
/// use gpui_kit::{div, prelude::*};
/// use gpui_kit_material::color::themes::{Contrast, ThemeId, ThemeMode};
/// use gpui_kit_material::prelude::*;
///
/// let card = Scope::new("dark-card")
///     .scheme(ThemeId::Baseline.scheme(ThemeMode::Dark, Contrast::Standard))
///     .child(div().bg(color(ColorRole::SURFACE)).child("Hello"));
/// ```
#[derive(gpui::IntoElement)]
pub struct Scope {
    id: ElementId,
    scheme: Option<Scheme>,
    child: Option<AnyElement>,
    build: Option<BuildChild>,
}

impl Scope {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            scheme: None,
            child: None,
            build: None,
        }
    }
    /// Paint the subtree with `scheme`, transitioning when it changes.
    pub fn scheme(mut self, scheme: Scheme) -> Self {
        self.scheme = Some(scheme);
        self
    }
    /// The subtree, already built. See the module's limits.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }
    /// The subtree, built inside the scope.
    pub fn child_with(
        mut self,
        build: impl FnOnce(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.build = Some(Box::new(build));
        self
    }
    /// Advance the scope now, to build its subtree with a borrowed context.
    pub fn resolve(self, window: &mut Window, cx: &mut App) -> ResolvedScope {
        let scheme = self.scheme.map(|target| {
            let _ = cx.try_global::<ScopeDrawn>();
            let paint = window.use_keyed_state((self.id.clone(), "scope-paint"), cx, |_, _| {
                theme::Paint::at(target)
            });
            let now = cx.background_executor().now();
            let reduce_motion = cx.reduce_motion();
            let effects = motion::curve(MotionToken::SlowEffects);
            let (current, running) = paint.update(cx, |paint, _| {
                let running = paint.advance(target, now, reduce_motion, effects).1;
                (paint.current, running)
            });
            if running {
                window.request_animation_frame();
            }
            current
        });
        // Cached views inside the scope do not know it changed, so a change
        // redraws them, once per step of a transition.
        let key = scheme;
        let drawn = window.use_keyed_state((self.id.clone(), "scope-key"), cx, |_, _| key);
        if *drawn.read(cx) != key {
            drawn.update(cx, |drawn, _| *drawn = key);
            window.on_next_frame(|window, _| window.refresh());
        }
        ResolvedScope {
            frame: Rc::new(Frame {
                outer: CURRENT.with(|current| current.borrow().clone()),
                scheme,
            }),
        }
    }
}

thread_local! {
    /// The scope being drawn, for [`current_scope`] and nested scopes.
    static CURRENT: RefCell<Option<Rc<Frame>>> = const { RefCell::new(None) };
}

/// Read by the view holding a scheme scope and stamped by drawing it. Under GPUI
/// Fast that makes the holder rebuild whenever something nested in it does, instead
/// of being drawn from last frame around it, which would leave the nested view
/// outside the scope's phases.
#[derive(Default)]
struct ScopeDrawn;
impl Global for ScopeDrawn {}

/// What one scope changes while something draws inside it.
struct Frame {
    /// The enclosing scope, for draws deferred out of both.
    outer: Option<Rc<Frame>>,
    scheme: Option<Scheme>,
}

impl Frame {
    fn enter<R>(
        self: &Rc<Self>,
        window: &mut Window,
        cx: &mut App,
        f: impl FnOnce(&mut Window, &mut App) -> R,
    ) -> R {
        let previous = self.scheme.map(|scheme| {
            cx.set_global(ScopeDrawn);
            theme::replace_scoped_scheme(Some(scheme))
        });
        let outer = CURRENT.with(|current| current.replace(Some(self.clone())));
        let result = f(window, cx);
        CURRENT.with(|current| current.replace(outer));
        if let Some(previous) = previous {
            theme::replace_scoped_scheme(previous);
        }
        result
    }

    /// Whether this frame is being drawn now, directly or around the current one.
    fn is_current(self: &Rc<Self>) -> bool {
        let mut frame = CURRENT.with(|current| current.borrow().clone());
        while let Some(current) = frame {
            if Rc::ptr_eq(&current, self) {
                return true;
            }
            frame = current.outer.clone();
        }
        false
    }

    /// Run deferred work, which starts outside every scope, inside this one.
    fn reenter(
        self: &Rc<Self>,
        window: &mut Window,
        cx: &mut App,
        f: &mut dyn FnMut(&mut Window, &mut App),
    ) {
        // `Deferred` lays out in place: the frame may already be on the stack,
        // and entering it again would only repeat what is in force.
        if self.is_current() {
            return f(window, cx);
        }
        match self.outer.clone() {
            Some(outer) => outer.reenter(window, cx, &mut |window, cx| {
                self.enter(window, cx, |window, cx| f(window, cx))
            }),
            None => self.enter(window, cx, |window, cx| f(window, cx)),
        }
    }
}

/// A scope advanced for this frame. Build its subtree with [`Self::build`] where
/// the build needs a borrowed context, then pass that to [`Self::child`].
pub struct ResolvedScope {
    frame: Rc<Frame>,
}

impl ResolvedScope {
    /// Run `f` inside the scope, to build elements that read colors as they are
    /// built. Render code passes its `Window` and `Context` through.
    pub fn build<C: std::borrow::BorrowMut<App>, R>(
        &self,
        window: &mut Window,
        cx: &mut C,
        f: impl FnOnce(&mut Window, &mut C) -> R,
    ) -> R {
        let previous = self
            .frame
            .scheme
            .map(|scheme| theme::replace_scoped_scheme(Some(scheme)));
        let outer = CURRENT.with(|current| current.replace(Some(self.frame.clone())));
        let result = f(window, cx);
        CURRENT.with(|current| current.replace(outer));
        if let Some(previous) = previous {
            theme::replace_scoped_scheme(previous);
        }
        result
    }

    /// The element that draws `child` inside the scope.
    pub fn child(&self, child: impl IntoElement) -> ScopeElement {
        ScopeElement {
            frame: self.frame.clone(),
            child: child.into_any_element(),
        }
    }
}

impl RenderOnce for Scope {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let child = self.child.take();
        let build = self.build.take();
        let resolved = self.resolve(window, cx);
        let child = match build {
            Some(build) => resolved.build(window, cx, |window, cx| build(window, cx)),
            None => child.unwrap_or_else(|| Empty.into_any_element()),
        };
        resolved.child(child)
    }
}

/// Keep `child` in the scope that is current now, for something that draws later
/// from outside the tree: a deferred popover or a drag preview. Call it while
/// rendering in the scope; the result draws correctly anywhere, and it costs
/// nothing outside a scope.
pub fn current_scope(child: impl IntoElement) -> AnyElement {
    match CURRENT.with(|current| current.borrow().clone()) {
        Some(frame) => ReplayElement {
            frame,
            child: child.into_any_element(),
        }
        .into_any_element(),
        None => child.into_any_element(),
    }
}

/// The scope that is current now, kept for something built later from outside
/// the tree, where [`current_scope`] cannot reach because nothing exists yet.
#[derive(Clone)]
pub(crate) struct CapturedScope(Option<Rc<Frame>>);

impl CapturedScope {
    pub(crate) fn now() -> Self {
        Self(CURRENT.with(|current| current.borrow().clone()))
    }

    /// `child`, drawn inside the scope that was captured.
    pub(crate) fn draw(&self, child: impl IntoElement) -> AnyElement {
        match &self.0 {
            Some(frame) => ReplayElement {
                frame: frame.clone(),
                child: child.into_any_element(),
            }
            .into_any_element(),
            None => child.into_any_element(),
        }
    }
}

/// The element a [`ResolvedScope`] wraps its subtree in.
pub struct ScopeElement {
    frame: Rc<Frame>,
    child: AnyElement,
}

impl IntoElement for ScopeElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for ScopeElement {
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
        let child = &mut self.child;
        let id = self
            .frame
            .enter(window, cx, |window, cx| child.request_layout(window, cx));
        (id, ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let child = &mut self.child;
        self.frame
            .enter(window, cx, |window, cx| child.prepaint(window, cx));
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
        let child = &mut self.child;
        self.frame
            .enter(window, cx, |window, cx| child.paint(window, cx));
    }
}

/// Draws a subtree inside a scope captured earlier.
struct ReplayElement {
    frame: Rc<Frame>,
    child: AnyElement,
}

impl IntoElement for ReplayElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for ReplayElement {
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
        let child = &mut self.child;
        let mut id = None;
        self.frame.reenter(window, cx, &mut |window, cx| {
            id = Some(child.request_layout(window, cx));
        });
        (id.expect("a scope runs its work once"), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let child = &mut self.child;
        self.frame.reenter(window, cx, &mut |window, cx| {
            child.prepaint(window, cx);
        });
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
        let child = &mut self.child;
        self.frame
            .reenter(window, cx, &mut |window, cx| child.paint(window, cx));
    }
}

#[cfg(all(test, feature = "full"))]
mod tests;
