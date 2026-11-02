//! The focus ring and ripple every interactive component paints, whichever feature
//! group it is in; so this module, unlike the button that first needed them, is always
//! compiled.
mod focus_ring;
pub(crate) mod ripple;
mod ripple_clip;
pub(crate) use focus_ring::ButtonFocusRing;
pub(crate) use ripple::ButtonRipple;
#[cfg(test)]
pub(crate) use ripple::{last_wave_corners, ripple_writes, waves_painted};
