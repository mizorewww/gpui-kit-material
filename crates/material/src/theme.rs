//! The painted scheme: what [`color`] returns on the current frame.
//!
//! A palette change does not snap: the painted scheme blends from wherever
//! it is toward the new target on the slow effects spring, and a further
//! change retargets from the current paint. Reduced motion paints the
//! target at once.
use crate::color::{Color, ColorRole, PaletteId, Scheme, ThemePreferences};
use crate::motion::SLOW_EFFECTS;
use gpui::{App, Rgba, Window, WindowAppearance};
use std::{cell::RefCell, time::Instant};

struct Paint {
    target: Option<PaletteId>,
    from: Scheme,
    current: Scheme,
    started: Instant,
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

/// Linear interpolation between two GPUI colors; see [`Color::blend`].
pub fn blend(from: Rgba, to: Rgba, t: f32) -> Rgba {
    if t >= 1. {
        return to;
    }
    let t = t.max(0.);
    Rgba {
        r: from.r + (to.r - from.r) * t,
        g: from.g + (to.g - from.g) * t,
        b: from.b + (to.b - from.b) * t,
        a: from.a + (to.a - from.a) * t,
    }
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
pub fn apply(preference: &ThemePreferences, window: &mut Window, cx: &mut App) -> bool {
    apply_palette(preference.resolve(system_dark(cx)), window, cx)
}

/// Paint one palette, transitioning from the current paint.
pub fn apply_palette(id: PaletteId, window: &mut Window, cx: &mut App) -> bool {
    let target = Scheme::from_palette(id);
    let reduce_motion = cx.reduce_motion();
    let (changed, running) = PAINT.with(|paint| {
        let mut paint = paint.borrow_mut();
        let now = Instant::now();
        if paint.target.is_none() {
            paint.target = Some(id);
            paint.current = target;
            paint.from = target;
            return (true, false);
        }
        if paint.target != Some(id) {
            paint.from = paint.current;
            paint.target = Some(id);
            paint.started = now;
        }
        let t = if reduce_motion {
            1.
        } else {
            (now.duration_since(paint.started).as_secs_f32()
                / SLOW_EFFECTS.duration().as_secs_f32())
            .min(1.)
        };
        let next = paint.from.blend(&target, SLOW_EFFECTS.ease()(t));
        let changed = next != paint.current;
        paint.current = next;
        (changed, t < 1.)
    });
    if running {
        window.request_animation_frame();
    }
    changed
}

/// Set the text scale: 14px is 1rem at 100%, and every rem token follows it.
pub fn apply_scale(scale: f32, window: &mut Window) {
    window.set_rem_size(gpui::px(crate::tokens::typography::BASE_PX * scale));
}
