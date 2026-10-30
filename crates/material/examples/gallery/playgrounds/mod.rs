//! Component playgrounds, one module per md3.io component page. Each
//! exports a `PLAYGROUND` static that `super::catalog` points its entry at.
//! See `super::playground` for the API.
pub mod button;
pub mod button_group;
pub mod extended_fab;
pub mod fab;
pub mod fab_menu;
pub mod icon_button;
pub mod split_button;
pub mod switch;
#[cfg(test)]
mod test_support;
pub mod text_field;

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
