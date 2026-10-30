//! Component playgrounds, one module per md3.io component page. Each
//! exports a `PLAYGROUND` static that `super::catalog` points its entry at.
//! See `super::playground` for the API.
pub mod badge;
pub mod button;
pub mod checkbox;
pub mod chips;
pub mod date_picker;
pub mod loading_indicator;
pub mod progress;
pub mod radio;
pub mod search;
pub mod select;
pub mod slider;
pub mod snackbar;
pub mod switch;
#[cfg(test)]
mod test_support;
pub mod text_field;
pub mod time_picker;
pub mod tooltip;

#[cfg(test)]
mod testing;

/// The common imports of playground modules.
pub mod prelude {
    #[allow(unused_imports)]
    pub(crate) use super::super::playground::{
        Config, Control, Playground, PlaygroundState, Preview, Scenario, Section, Set, Value,
        symbol,
    };
    pub(crate) use super::super::spec::type_role;
    pub use gpui::{prelude::*, *};
    pub use gpui_kit_material::prelude::*;
}
