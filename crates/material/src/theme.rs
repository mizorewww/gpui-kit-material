//! The painted scheme: what [`color`] returns on the current frame.
//!
//! A palette change does not snap: the painted scheme blends from wherever
//! it is toward the new target on the slow effects spring, and a further
//! change retargets from the current paint. Reduced motion paints the
//! target at once.
use crate::color::{Color, ColorRole, PaletteId, Scheme, ThemePreferences};
use crate::motion::{self, MotionToken};
use gpui::{App, Global, Hsla, Pixels, Rgba, Window, WindowAppearance};
use gpui_kit::component::{Theme as ComponentTheme, ThemeMode as ComponentThemeMode};
use std::cell::RefCell;
#[cfg(not(target_family = "wasm"))]
use std::time::Instant;
#[cfg(target_family = "wasm")]
use web_time::Instant;

struct Paint {
    target: Option<Scheme>,
    from: Scheme,
    current: Scheme,
    started: Instant,
    motion: motion::MotionCurve,
}

thread_local! {
    // GPUI renders application chrome and overlays on its UI thread, so a
    // thread-local paint keeps role lookups context-free without globals
    // that need `&App`.
    static PAINT: RefCell<Paint> = RefCell::new(Paint {
        target: None,
        from: Scheme::from_palette(PaletteId::Paper),
        current: Scheme::from_palette(PaletteId::Paper),
        started: Instant::now(),
        motion: motion::MotionScheme::Expressive.resolve(MotionToken::SlowEffects),
    });
}

/// Convert a kit color to GPUI's.
pub fn rgba(color: Color) -> Rgba {
    Rgba {
        r: color.r,
        g: color.g,
        b: color.b,
        a: color.a,
    }
}

/// The current paint of a role.
pub fn color(role: ColorRole) -> Rgba {
    rgba(PAINT.with(|paint| paint.borrow().current.get(role)))
}

/// Convert a GPUI color to the kit's.
fn kit_color(color: Rgba) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: color.a,
    }
}

/// Linear interpolation between two GPUI colors: [`Color::blend`], the one
/// implementation, applied to GPUI's color type.
pub fn blend(from: Rgba, to: Rgba, t: f32) -> Rgba {
    rgba(kit_color(from).blend(kit_color(to), t))
}

/// The current paint as a whole, e.g. to mirror it into another toolkit's theme.
pub fn scheme() -> Scheme {
    PAINT.with(|paint| paint.borrow().current)
}

/// Whether the painted scheme is a dark one. Mid-transition this follows
/// whichever palette the paint is closer to.
pub fn is_dark() -> bool {
    PAINT.with(|paint| paint.borrow().current.is_dark())
}

/// Whether the system currently asks for a dark appearance.
pub fn system_dark(cx: &App) -> bool {
    matches!(
        cx.window_appearance(),
        WindowAppearance::Dark | WindowAppearance::VibrantDark
    )
}

/// Paint `preference` resolved against the system appearance. Call at the
/// start of each root render; it schedules frames while a transition runs.
/// Returns whether the paint changed this frame, so an application can
/// mirror the scheme into other toolkits only when needed.
///
/// This render-time path does not update the Component theme.
#[deprecated(
    note = "render-time path that leaves the Component theme stale; use `init_component_theme` \
            and `advance_component_theme` with `preference.resolve(system_dark(cx))`, or \
            `apply_scheme` for Base-only rendering"
)]
pub fn apply(preference: &ThemePreferences, window: &mut Window, cx: &mut App) -> bool {
    apply_scheme(
        Scheme::from_palette(preference.resolve(system_dark(cx))),
        window,
        cx,
    )
}

/// Paint one palette, transitioning from the current paint.
///
/// This render-time path does not update the Component theme.
#[deprecated(
    note = "render-time path that leaves the Component theme stale; use `init_component_theme` \
            and `advance_component_theme` with `Scheme::from_palette(id)`, or \
            `apply_scheme(Scheme::from_palette(id), ..)` for Base-only rendering"
)]
pub fn apply_palette(id: PaletteId, window: &mut Window, cx: &mut App) -> bool {
    apply_scheme(Scheme::from_palette(id), window, cx)
}

/// Apply a complete custom semantic scheme with the same animated transitions.
pub fn apply_scheme(target: Scheme, window: &mut Window, cx: &mut App) -> bool {
    let (changed, running) = advance_paint(target, cx);
    if running {
        window.request_animation_frame();
    }
    changed
}

// Both the Base-only renderer and the Component pre-frame driver use one sampler.
fn advance_paint(target: Scheme, cx: &App) -> (bool, bool) {
    let reduce_motion = cx.reduce_motion();
    let effects = motion::curve(MotionToken::SlowEffects);
    PAINT.with(|paint| {
        let mut paint = paint.borrow_mut();
        let now = cx.background_executor().now();
        if paint.target.is_none() {
            paint.target = Some(target);
            paint.current = target;
            paint.from = target;
            paint.started = now;
            paint.motion = effects;
            return (true, false);
        }
        if paint.target != Some(target) {
            paint.from = paint.current;
            paint.target = Some(target);
            paint.started = now;
            paint.motion = effects;
        }
        if reduce_motion {
            let changed = paint.current != target;
            paint.current = target;
            paint.from = target;
            paint.started = now;
            return (changed, false);
        }
        let t = (now.duration_since(paint.started).as_secs_f32()
            / paint.motion.duration().as_secs_f32())
        .min(1.);
        let next = paint.from.blend(&target, paint.motion.ease()(t));
        let changed = next != paint.current;
        paint.current = next;
        (changed, paint.from != target && t < 1.)
    })
}

/// Set the text scale: 14px is 1rem at 100%, and every rem token follows it.
///
/// This leaves the Component theme's font size behind the window's rem size.
#[deprecated(
    note = "leaves the Component theme's font size stale; pass the scale to \
            `init_component_theme` and `advance_component_theme`"
)]
pub fn apply_scale(scale: f32, window: &mut Window) {
    set_rem_scale(scale, window);
}

// The single authority for the window's rem size: both the legacy
// `apply_scale` path and `advance_component_theme` resolve the text scale
// here. Returns whether the rem size changed.
fn set_rem_scale(scale: f32, window: &mut Window) -> bool {
    let font_size = scale_pixels(scale);
    let changed = window.rem_size() != font_size;
    if changed {
        window.set_rem_size(font_size);
    }
    changed
}

// Explicit opt-in: existing Base-only callers never require a Component global.
struct ComponentThemeBridge;
impl Global for ComponentThemeBridge {}

fn scale_pixels(scale: f32) -> Pixels {
    let scale = if scale.is_finite() && scale > 0. {
        scale
    } else {
        1.
    };
    let pixels = crate::tokens::typography::BASE_PX * scale;
    gpui::px(if pixels.is_finite() {
        pixels
    } else {
        crate::tokens::typography::BASE_PX
    })
}

fn component_font_family() -> &'static str {
    // Follow the currently implemented Material type scale. A future reference
    // font migration must update the scale and font assets together.
    crate::tokens::typography::TypeRole::BodyMedium
        .scale()
        .font_family
}

fn component_color(painted: Scheme, role: ColorRole) -> Hsla {
    rgba(painted.get(role)).into()
}

// Only generic semantic colors belong to this bridge. Full M3 role storage,
// state-layer recipes, motion, spacing and elevation remain Material-owned.
macro_rules! component_role_projection {
    ($($field:ident => $role:ident),* $(,)?) => {
        fn component_projection_matches(theme: &ComponentTheme, painted: Scheme, scale: f32) -> bool {
            theme.font_size == scale_pixels(scale)
                && theme.font_family.as_ref() == component_font_family()
                $(&& theme.colors.$field == component_color(painted, ColorRole::$role)
                  && theme.tokens.$field == component_color(painted, ColorRole::$role).into())*
        }

        fn write_component_projection(theme: &mut ComponentTheme, painted: Scheme, scale: f32) {
            theme.font_size = scale_pixels(scale);
            theme.font_family = component_font_family().into();
            $(let color = component_color(painted, ColorRole::$role);
              theme.colors.$field = color;
              theme.tokens.$field = color.into();)*
        }
    };
}

component_role_projection! {
    background => SURFACE,
    foreground => ON_SURFACE,
    popover => SURFACE_CONTAINER_HIGH,
    popover_foreground => ON_SURFACE,
    primary => PRIMARY,
    primary_foreground => ON_PRIMARY,
    secondary => SECONDARY_CONTAINER,
    secondary_foreground => ON_SECONDARY_CONTAINER,
    muted => SURFACE_CONTAINER,
    muted_foreground => ON_SURFACE_VARIANT,
    accent => TERTIARY_CONTAINER,
    accent_foreground => ON_TERTIARY_CONTAINER,
    danger => ERROR,
    danger_foreground => ON_ERROR,
    border => OUTLINE_VARIANT,
    input => OUTLINE,
    ring => PRIMARY,
}

fn sync_component_theme(painted: Scheme, scale: f32, cx: &mut App) -> bool {
    if cx.try_global::<ComponentThemeBridge>().is_none()
        || cx.try_global::<ComponentTheme>().is_none()
    {
        return false;
    }
    let mode = if painted.is_dark() {
        ComponentThemeMode::Dark
    } else {
        ComponentThemeMode::Light
    };
    let mode_changed = ComponentTheme::global(cx).mode != mode;
    if mode_changed {
        // Component loads its registered mode config after an update that changes
        // mode. Perform that step first so it cannot overwrite the M3 projection.
        ComponentTheme::update(cx, |theme| theme.mode = mode);
    }
    let changed = !component_projection_matches(ComponentTheme::global(cx), painted, scale);
    if changed {
        ComponentTheme::update(cx, |theme| {
            write_component_projection(theme, painted, scale)
        });
    }
    mode_changed || changed
}

/// Enable the Component theme bridge after `gpui_kit::init`, before opening a
/// window or returning its content builder. Initialize from the final startup
/// palette and scale, including capture overrides, so the first Root preparation
/// sees the correct colors and rem size. Returns false when Component is absent.
/// No transition is started for this initial paint. The bridge owns the sans
/// family from Material's current body type scale, its base size and projected
/// colors. The projection does not own monospace fonts or unrelated geometry;
/// Component mode configurations retain their normal behavior for those fields.
pub fn init_component_theme(initial: Scheme, scale: f32, cx: &mut App) -> bool {
    if cx.try_global::<ComponentTheme>().is_none() {
        return false;
    }
    PAINT.with(|paint| {
        let mut paint = paint.borrow_mut();
        paint.target = Some(initial);
        paint.from = initial;
        paint.current = initial;
        paint.started = cx.background_executor().now();
        paint.motion = motion::curve(MotionToken::SlowEffects);
    });
    if cx.try_global::<ComponentThemeBridge>().is_none() {
        cx.set_global(ComponentThemeBridge);
    }
    sync_component_theme(initial, scale, cx);
    true
}

/// Advance Material paint and its enabled Component projection before Root
/// rendering. Call from theme/scale events and next-frame callbacks, not from
/// `Render`. Returns whether the animation needs another frame; the caller owns
/// scheduling and should retain only one pending callback. This function never
/// schedules an animation itself. Base-only callers remain supported.
pub fn advance_component_theme(
    target: Scheme,
    scale: f32,
    window: &mut Window,
    cx: &mut App,
) -> bool {
    let (changed, running) = advance_paint(target, cx);
    sync_component_theme(scheme(), scale, cx);
    let scale_changed = set_rem_scale(scale, window);
    if changed || scale_changed {
        // Material colors are thread-local, so cached Material-only entities do
        // not observe Component's global. Its queued refresh handles ordinary
        // event boundaries; this also makes synchronous draws see current paint.
        // This refresh is the single invalidation owner for a theme or scale
        // step: callers need not also notify their view for it.
        window.refresh();
    }
    running
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Context, IntoElement, Render, TestAppContext, div};
    use std::time::Duration;

    #[test]
    fn component_projection_keeps_unowned_tokens_and_uses_one_rem_baseline() {
        let mut theme = ComponentTheme::default();
        theme.font_family = "Caller Font".into();
        theme.mono_font_family = "Caller Mono".into();
        theme.radius = gpui::px(13.);
        let unowned = theme.colors.chart_1;
        let painted = Scheme::from_palette(PaletteId::Paper)
            .with_role(ColorRole::PRIMARY_FIXED, Color::hex(0x123456));
        write_component_projection(&mut theme, painted, 1.5);
        assert!(component_projection_matches(&theme, painted, 1.5));
        assert_eq!(theme.font_size, gpui::px(21.));
        assert_eq!(theme.font_family.as_ref(), component_font_family());
        assert_eq!(theme.mono_font_family.as_ref(), "Caller Mono");
        assert_eq!(theme.radius, gpui::px(13.));
        assert_eq!(theme.colors.chart_1, unowned);
        assert_eq!(painted.get(ColorRole::PRIMARY_FIXED), Color::hex(0x123456));
        // Both representations are managed, including a pre-existing mismatched token.
        theme.tokens.primary = Hsla::default().into();
        assert!(!component_projection_matches(&theme, painted, 1.5));
        write_component_projection(&mut theme, painted, 1.5);
        assert!(component_projection_matches(&theme, painted, 1.5));
    }

    #[test]
    fn blend_matches_the_kit_color_blend() {
        let from = Color::hex(0x204060);
        let to = Color::hexa(0xffeeddcc);
        for t in [-1., 0., 0.3, 1., 2., f32::NAN] {
            assert_eq!(blend(rgba(from), rgba(to), t), rgba(from.blend(to, t)));
        }
    }

    struct ScaleHarness;
    impl Render for ScaleHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }

    #[gpui::test]
    #[allow(
        deprecated,
        reason = "checks that the deprecated `apply_scale` still shares the rem authority"
    )]
    fn legacy_and_component_paths_share_one_rem_authority(cx: &mut TestAppContext) {
        let (_, cx) = cx.add_window_view(|_, _| ScaleHarness);
        cx.update(|window, cx| {
            apply_scale(1.5, window);
            assert_eq!(window.rem_size(), gpui::px(21.));
            apply_scale(f32::NAN, window);
            assert_eq!(window.rem_size(), gpui::px(14.));
            advance_component_theme(Scheme::from_palette(PaletteId::Paper), 1.25, window, cx);
            assert_eq!(window.rem_size(), gpui::px(17.5));
            assert!(
                !set_rem_scale(1.25, window),
                "an equal scale is not a change"
            );
            assert!(set_rem_scale(-2., window));
            assert_eq!(window.rem_size(), scale_pixels(1.));
        });
    }

    #[gpui::test]
    fn component_bridge_is_explicit_and_idempotent_across_modes(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let initial = Scheme::from_palette(PaletteId::Paper);
            assert!(!init_component_theme(initial, 1., cx));
            assert!(cx.try_global::<ComponentTheme>().is_none());
            gpui_kit::init(cx);
            assert!(init_component_theme(initial, 1., cx));
            assert_eq!(scheme(), initial);
            assert!(!sync_component_theme(initial, 1., cx));
            for palette in [PaletteId::Ink, PaletteId::Paper] {
                let target = Scheme::from_palette(palette);
                assert!(sync_component_theme(target, 1.25, cx));
                let theme = ComponentTheme::global(cx);
                assert_eq!(theme.mode.is_dark(), target.is_dark());
                assert!(component_projection_matches(theme, target, 1.25));
                assert_eq!(theme.font_family.as_ref(), component_font_family());
                assert!(!sync_component_theme(target, 1.25, cx));
            }
        });
    }

    struct PaletteHarness {
        palette: PaletteId,
        motion: motion::MotionScheme,
    }
    impl Render for PaletteHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            motion::set_motion_scheme(self.motion);
            apply_scheme(Scheme::from_palette(self.palette), window, cx);
            div()
        }
    }

    #[gpui::test]
    fn palette_transition_keeps_its_curve_when_motion_scheme_changes(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| PaletteHarness {
            palette: PaletteId::Paper,
            motion: motion::MotionScheme::Expressive,
        });
        cx.update(|window, cx| {
            cx.set_reduce_motion(true);
            window.refresh();
            window.draw(cx).clear(cx);
            cx.set_reduce_motion(false);
            view.update(cx, |view, _| view.palette = PaletteId::Ink);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.executor().advance_clock(Duration::from_millis(100));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let before = scheme();
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.motion = motion::MotionScheme::Standard);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            scheme(),
            before,
            "changing a motion preference must not jump color"
        );
        cx.update(|window, cx| {
            view.update(cx, |view, _| view.palette = PaletteId::Paper);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            scheme(),
            before,
            "a reversed palette starts at current paint"
        );
        cx.executor().advance_clock(Duration::from_millis(50));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_ne!(scheme(), before);
        cx.update(|window, cx| {
            cx.set_reduce_motion(true);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(scheme(), Scheme::from_palette(PaletteId::Paper));
        cx.update(|window, cx| {
            cx.set_reduce_motion(false);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            scheme(),
            Scheme::from_palette(PaletteId::Paper),
            "restoring motion must not resume the completed fade"
        );
        cx.executor().advance_clock(Duration::from_millis(50));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(scheme(), Scheme::from_palette(PaletteId::Paper));
        motion::set_motion_scheme(motion::MotionScheme::Expressive);
    }
}
