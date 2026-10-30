//! Component playgrounds, one module per md3.io component page. Each
//! exports a `PLAYGROUND` static that `super::catalog` points its entry at.
//! See `super::playground` for the API.
pub mod avatar;
pub mod badge;
pub mod bottom_app_bar;
pub mod bottom_sheet;
pub mod button;
pub mod button_group;
pub mod card;
pub mod carousel;
pub mod checkbox;
pub mod chips;
pub mod date_picker;
pub mod dialog;
pub mod divider;
pub mod extended_fab;
pub mod fab;
pub mod fab_menu;
pub mod icon_button;
pub mod list;
pub mod loading_indicator;
pub mod menu;
pub mod navigation_bar;
pub mod navigation_drawer;
pub mod navigation_rail;
pub mod progress;
pub mod radio;
pub mod search;
pub mod select;
pub mod side_sheet;
pub mod slider;
pub mod snackbar;
pub mod split_button;
pub mod switch;
pub mod tabs;
pub mod text_field;
pub mod time_picker;
pub mod toolbar;
pub mod tooltip;
pub mod top_app_bar;

#[cfg(test)]
mod harness;
#[cfg(test)]
mod nav_harness;
#[cfg(test)]
mod test_support;
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
