//! Scoped theme and layout direction: a subtree painting with its own scheme,
//! laying out in its own direction, or both.
//!
//! The application paints one scheme and has one direction ([`crate::theme`],
//! [`set_app_direction`]), a window can override the direction
//! ([`set_window_direction`]), and a [`Scope`] overrides either for a subtree.
//!
//! GPUI draws in three phases, so a scope is an element that re-enters itself
//! around each of them: request_layout (which renders every `RenderOnce`
//! descendant and view), prepaint (lists render their items here) and paint
//! (canvases read colors here). Draws a descendant defers, such as popovers,
//! run after the scope has left the stack, so Material's overlays wrap what they
//! defer in [`current_scope`], which brings it back in.
//!
//! Inside a scope [`crate::theme::color`], [`crate::theme::scheme`] and
//! [`crate::theme::is_dark`] answer from its scheme, and Kit's theme globals
//! (`cx.theme()` and the Base projection) hold its projection, so Kit controls
//! follow. A change of scheme blends on the same effects spring as the
//! application's, per scope, and reduced motion lands it at once.
//!
//! # Limits
//!
//! A scope reaches what draws through it. These do not, and use the
//! application's scheme and direction:
//!
//! - Kit overlays opened on its `Root` (Kit's dialog, sheet and notification
//!   layers) and Kit's own tooltips, which draw outside the subtree. Material's
//!   dialogs, sheets, menus and tooltips draw through the subtree and are
//!   covered.
//! - What Kit defers itself reads its colors while rendering, which is in the
//!   scope, but anything it reads while painting is not.
//! - Drag previews, unless the preview is built with [`current_scope`].
//! - Under GPUI Fast, a view two or more views inside a scheme scope can be rebuilt
//!   on its own by a notification, outside the scope's phases. Views directly
//!   inside rebuild with the view holding the scope, because the scope's global
//!   swap marks that view's dependency on Kit's theme as changed.
//! - Kit's theme is projected again when the scheme or text size changes. Another
//!   field changed on Kit's global reaches the subtree at its next scheme change.
//! - Kit's scrollbar styles, which Kit builds privately from its global theme.
//! - An element built before the scope is entered reads the outer scheme for
//!   any color it computed while being built. Build through [`Scope::child_with`]
//!   or [`ResolvedScope::build`]. Material components compute colors when they
//!   render, so passing one to [`Scope::child`] is fine.
use crate::color::Scheme;
use crate::motion::{self, MotionToken};
use crate::theme;
use gpui::*;
use gpui_base::Theme as BaseTheme;
use gpui_kit::component::Theme as ComponentTheme;
use std::cell::RefCell;
use std::rc::Rc;

pub use gpui::LayoutDirection;

/// Whether `window` is laying out right to left here. Components read this
/// while rendering, for the few things layout mirroring cannot reach: painted
/// paths, pointer math and arrow keys.
pub fn is_rtl(window: &Window) -> bool {
    window.layout_direction().is_rtl()
}

/// Set the application's direction. Windows that have not chosen their own
/// follow it and redraw; new windows start in it.
pub fn set_app_direction(direction: LayoutDirection, cx: &mut App) {
    cx.set_layout_direction(direction);
}

/// Set one window's direction, overriding the application's.
pub fn set_window_direction(direction: LayoutDirection, window: &mut Window) {
    window.set_layout_direction(direction);
}

/// An arrow key's logical effect here: `1` toward the end of the reading
/// direction, `-1` toward its start. Other keys answer `None`.
pub fn horizontal_arrow(key: &str, rtl: bool) -> Option<i32> {
    let forward = match key {
        "right" => 1,
        "left" => -1,
        _ => return None,
    };
    Some(if rtl { -forward } else { forward })
}

type BuildChild = Box<dyn FnOnce(&mut Window, &mut App) -> AnyElement>;

/// A subtree's overrides. Set a scheme, a direction or both.
#[derive(gpui::IntoElement)]
pub struct Scope {
    id: ElementId,
    scheme: Option<Scheme>,
    direction: Option<LayoutDirection>,
    child: Option<AnyElement>,
    build: Option<BuildChild>,
}

impl Scope {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            scheme: None,
            direction: None,
            child: None,
            build: None,
        }
    }
    /// Paint the subtree with `scheme`, transitioning when it changes.
    pub fn scheme(mut self, scheme: Scheme) -> Self {
        self.scheme = Some(scheme);
        self
    }
    /// Lay out and paint the subtree in `direction`.
    pub fn direction(mut self, direction: LayoutDirection) -> Self {
        self.direction = Some(direction);
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
            let paint = window.use_keyed_state((self.id.clone(), "scope-paint"), cx, |_, _| {
                ScopePaint::new(target)
            });
            let now = cx.background_executor().now();
            let reduce_motion = cx.reduce_motion();
            let effects = motion::curve(MotionToken::SlowEffects);
            let (current, running) = paint.update(cx, |paint, _| {
                let running = paint.paint.advance(target, now, reduce_motion, effects).1;
                (paint.paint.current, running)
            });
            if running {
                window.request_animation_frame();
            }
            let kit = paint.update(cx, |paint, cx| paint.projection(current, cx));
            Kit {
                scheme: current,
                projection: kit,
            }
        });
        // Cached views inside the scope do not know it changed, so a change
        // redraws them, once per step of a transition.
        let key = (scheme.as_ref().map(|kit| kit.scheme), self.direction);
        let drawn = window.use_keyed_state((self.id.clone(), "scope-key"), cx, |_, _| key);
        if *drawn.read(cx) != key {
            drawn.update(cx, |drawn, _| *drawn = key);
            window.on_next_frame(|window, _| window.refresh());
        }
        ResolvedScope {
            frame: Rc::new(Frame {
                outer: CURRENT.with(|current| current.borrow().clone()),
                scheme,
                direction: self.direction,
            }),
        }
    }
}

/// A scope's scheme animation, with the Kit theme projected from it.
struct ScopePaint {
    paint: theme::Paint,
    projection: Option<(Scheme, Pixels, Rc<RefCell<KitThemes>>)>,
}

impl ScopePaint {
    fn new(initial: Scheme) -> Self {
        Self {
            paint: theme::Paint::at(initial),
            projection: None,
        }
    }

    /// Kit's themes for `scheme`. Rebuilt when the scheme changes, or when the
    /// application's text scale does.
    fn projection(&mut self, scheme: Scheme, cx: &App) -> Option<Rc<RefCell<KitThemes>>> {
        let component = cx.try_global::<ComponentTheme>()?;
        let font_size = component.font_size;
        if let Some((built, size, themes)) = &self.projection
            && *built == scheme
            && *size == font_size
        {
            return Some(themes.clone());
        }
        let mut component = component.clone();
        theme::project_scheme(&mut component, scheme);
        let mut base = BaseTheme::global(cx);
        base.appearance = if scheme.is_dark() {
            gpui_base::ThemeAppearance::Dark
        } else {
            gpui_base::ThemeAppearance::Light
        };
        base.tokens = component.semantic_tokens();
        let themes = Rc::new(RefCell::new(KitThemes { component, base }));
        self.projection = Some((scheme, font_size, themes.clone()));
        Some(themes)
    }
}

struct KitThemes {
    component: ComponentTheme,
    base: BaseTheme,
}

thread_local! {
    /// The scope being drawn, for [`current_scope`] and nested scopes.
    static CURRENT: RefCell<Option<Rc<Frame>>> = const { RefCell::new(None) };
}

/// Exchange Kit's theme globals with a scope's. This notifies their observers,
/// which is only Base's caret blink: it compares its policy, which a scope leaves
/// unchanged, and moves on, so nothing redraws. A test holds that.
fn swap_kit_themes(themes: &RefCell<KitThemes>, cx: &mut App) {
    let mut themes = themes.borrow_mut();
    let themes = &mut *themes;
    std::mem::swap(cx.global_mut::<ComponentTheme>(), &mut themes.component);
    if cx.has_global::<BaseTheme>() {
        std::mem::swap(cx.global_mut::<BaseTheme>(), &mut themes.base);
    }
}

struct Kit {
    scheme: Scheme,
    projection: Option<Rc<RefCell<KitThemes>>>,
}

/// What one scope changes while something draws inside it.
struct Frame {
    /// The enclosing scope, for draws deferred out of both.
    outer: Option<Rc<Frame>>,
    scheme: Option<Kit>,
    direction: Option<LayoutDirection>,
}

impl Frame {
    fn enter<R>(
        self: &Rc<Self>,
        window: &mut Window,
        cx: &mut App,
        f: impl FnOnce(&mut Window, &mut App) -> R,
    ) -> R {
        let previous = self.scheme.as_ref().map(|kit| {
            if let Some(projection) = &kit.projection {
                swap_kit_themes(projection, cx);
            }
            theme::replace_scoped_scheme(Some(kit.scheme))
        });
        let direction = self.direction.unwrap_or_else(|| window.layout_direction());
        let outer = CURRENT.with(|current| current.replace(Some(self.clone())));
        let result = window.with_layout_direction(direction, |window| f(window, cx));
        CURRENT.with(|current| current.replace(outer));
        if let (Some(kit), Some(previous)) = (&self.scheme, previous) {
            theme::replace_scoped_scheme(previous);
            if let Some(projection) = &kit.projection {
                swap_kit_themes(projection, cx);
            }
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
        // and swapping Kit's themes twice would put the outer one back.
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
        let previous = self.frame.scheme.as_ref().map(|kit| {
            if let Some(projection) = &kit.projection {
                swap_kit_themes(projection, cx.borrow_mut());
            }
            theme::replace_scoped_scheme(Some(kit.scheme))
        });
        let direction = self
            .frame
            .direction
            .unwrap_or_else(|| window.layout_direction());
        let outer = CURRENT.with(|current| current.replace(Some(self.frame.clone())));
        let result = window.with_layout_direction(direction, |window| f(window, cx));
        CURRENT.with(|current| current.replace(outer));
        if let (Some(kit), Some(previous)) = (&self.frame.scheme, previous) {
            theme::replace_scoped_scheme(previous);
            if let Some(projection) = &kit.projection {
                swap_kit_themes(projection, cx.borrow_mut());
            }
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

#[cfg(test)]
mod tests;
