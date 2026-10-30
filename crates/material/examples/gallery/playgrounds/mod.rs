//! Component playgrounds, one module per md3.io component page. Each
//! exports a `PLAYGROUND` static that `super::catalog` points its entry at.
//! See `super::playground` for the API.
pub mod bottom_app_bar;
pub mod bottom_sheet;
pub mod button;
pub mod card;
pub mod carousel;
pub mod dialog;
pub mod divider;
#[cfg(test)]
pub mod harness;
pub mod list;
pub mod menu;
pub mod navigation_bar;
pub mod navigation_drawer;
pub mod navigation_rail;
pub mod side_sheet;
pub mod switch;
pub mod tabs;
pub mod text_field;
pub mod toolbar;
pub mod top_app_bar;

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
