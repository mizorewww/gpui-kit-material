//! Compile-time proof of what a downstream application sees. Nothing here runs.
use gpui_kit::profiler::FrameEvent;

/// The engine's profiler events stay upstream's: this match is exhaustive only
/// while the dev-only `frame-observation` feature is off, which is also what
/// lets `gpui-fps` (linked below) compile.
pub fn event_is_draw(event: FrameEvent) -> bool {
    match event {
        FrameEvent::Draw(_) => true,
        FrameEvent::Present(_) => false,
    }
}

/// The caret query a page uses to keep an input above an on-screen keyboard is
/// part of the engine a consumer gets, not of frame observation.
pub fn caret(
    window: &mut gpui_kit::Window,
    cx: &mut gpui_kit::App,
) -> Option<gpui_kit::Bounds<gpui_kit::Pixels>> {
    window.focused_input_selection_bounds(cx)
}

pub use gpui_fps as fps_hud;
pub use gpui_kit_material::init as material_init;
