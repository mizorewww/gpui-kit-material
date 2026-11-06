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

pub use gpui_fps as fps_hud;
pub use gpui_kit_material::init as material_init;
