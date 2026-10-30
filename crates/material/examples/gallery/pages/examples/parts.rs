//! Building blocks shared by the example screens: device frames, the kit's
//! image placeholder, cached Material Symbols and the recurring section
//! layouts (headers, tile rows, media cards and feature items). Everything
//! interactive here is a crate component; the rest is sample content.
use super::super::prelude::*;
use gpui_kit_material::icons::{Icon, Style, Symbol};
use std::{cell::RefCell, collections::HashMap};

/// The kit's phone frame width and the web frame's size, in dp.
pub const PHONE_WIDTH: f32 = 412.;
pub const WEB_WIDTH: f32 = 905.;
pub const WEB_HEIGHT: f32 = 680.;
/// The kit draws every example frame inside an 8dp outline-variant stroke.
const FRAME_STROKE: f32 = 8.;

thread_local! {
    static PATHS: RefCell<HashMap<(&'static str, bool), SharedString>> =
        RefCell::new(HashMap::new());
    static NAV_ICONS: RefCell<HashMap<&'static str, NavIcon>> = RefCell::new(HashMap::new());
}

/// A rounded Material Symbol, outlined or filled. Paths resolve once per
/// thread, so rendering a symbol again does not allocate.
pub fn symbol(name: &'static str, filled: bool) -> Icon {
    let path = PATHS.with(|paths| {
        paths
            .borrow_mut()
            .entry((name, filled))
            .or_insert_with(|| {
                Symbol::named(name)
                    .expect("a Material Symbol")
                    .icon()
                    .style(Style::Rounded)
                    .filled(filled)
                    .asset_path()
                    .into()
            })
            .clone()
    });
    Icon::new(path)
}

pub fn icon(name: &'static str) -> Icon {
    symbol(name, false)
}

pub fn filled(name: &'static str) -> Icon {
    symbol(name, true)
}

/// A symbol in its navigation (outlined / filled) pair, resolved once.
pub fn nav_icon(name: &'static str) -> NavIcon {
    NAV_ICONS.with(|icons| {
        icons
            .borrow_mut()
            .entry(name)
            .or_insert_with(|| NavIcon::symbol(Symbol::named(name).expect("a Material Symbol")))
            .clone()
    })
}

/// A per-screen element id: `k` is the screen, `i` the element within it.
pub fn eid(name: &'static str, k: usize, i: usize) -> ElementId {
    (name, k * 1000 + i).into()
}

/// Text in an M3 type role and color role.
pub fn text(role: TypeRole, ink: ColorRole, content: impl Into<SharedString>) -> Div {
    spec::type_role(div(), role)
        .text_color(color(ink))
        .child(content.into())
}

/// One line of text, ellipsized at its container's width.
pub fn line(role: TypeRole, ink: ColorRole, content: impl Into<SharedString>) -> Div {
    text(role, ink, content)
        .w_full()
        .min_w_0()
        .whitespace_nowrap()
        .overflow_hidden()
        .text_ellipsis()
}

/// The kit's image placeholder: its shapes artwork (triangle, flower and
/// square) on surface-container-high. The caller sizes and rounds it.
/// Stands in for every photo; the Gallery ships no image assets.
pub fn art(glyph: f32) -> Div {
    div()
        .flex_shrink_0()
        .overflow_hidden()
        .flex()
        .items_center()
        .justify_center()
        .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
        .child(
            filled("interests")
                .size(rem(glyph))
                .color(color(ColorRole::OUTLINE_VARIANT)),
        )
}

/// The device stroke around a screen, rounded `radius` inside.
fn stroke(id: ElementId, radius: f32, screen: Div) -> Stateful<Div> {
    div()
        .id(id)
        .flex_shrink_0()
        .p(rem(FRAME_STROKE))
        .rounded(rem(radius + FRAME_STROKE))
        .bg(color(ColorRole::OUTLINE_VARIANT))
        .child(screen.rounded(rem(radius)).overflow_hidden())
}

/// A 412dp phone: status bar, the screen's content and the gesture bar
/// (`gesture` is its background, matching the navigation bar above it).
/// `edge_to_edge` content starts under the status bar (a hero image).
pub fn phone(
    id: impl Into<ElementId>,
    content: impl IntoElement,
    gesture: ColorRole,
    edge_to_edge: bool,
) -> Stateful<Div> {
    stroke(
        id.into(),
        28.,
        div()
            .relative()
            .w(rem(PHONE_WIDTH))
            .flex()
            .flex_col()
            .bg(color(ColorRole::SURFACE))
            .when(!edge_to_edge, |screen| screen.child(status_bar()))
            .child(content)
            .child(gesture_bar(gesture))
            .when(edge_to_edge, |screen| {
                screen.child(status_bar().absolute().top_0().left_0().right_0())
            }),
    )
}

/// A 905×680 browser window: the browser bar over `body`.
pub fn browser(id: impl Into<ElementId>, body: impl IntoElement) -> Stateful<Div> {
    stroke(
        id.into(),
        18.,
        div()
            .w(rem(WEB_WIDTH))
            .h(rem(WEB_HEIGHT))
            .flex()
            .flex_col()
            .bg(color(ColorRole::SURFACE))
            .child(browser_bar())
            .child(div().flex_1().min_h_0().flex().child(body)),
    )
}

/// The kit's Android status bar: time, camera cutout and system icons.
fn status_bar() -> Div {
    let ink = color(ColorRole::ON_SURFACE);
    div()
        .relative()
        .flex_shrink_0()
        .h(rem(52.))
        .px(rem(24.))
        .pb(rem(10.))
        .flex()
        .items_end()
        .justify_between()
        .child(text(TypeRole::BodyMedium, ColorRole::ON_SURFACE, "9:30"))
        .child(
            div()
                .flex()
                .items_center()
                .gap(rem(2.))
                .child(filled("signal_wifi_4_bar").size(rem(17.)).color(ink))
                .child(filled("signal_cellular_4_bar").size(rem(17.)).color(ink))
                .child(filled("battery_full").size(rem(17.)).color(ink)),
        )
        .child(
            div()
                .absolute()
                .top(rem(14.))
                .left_0()
                .right_0()
                .flex()
                .justify_center()
                .child(div().size(rem(24.)).rounded_full().bg(ink)),
        )
}

/// The gesture navigation handle.
fn gesture_bar(background: ColorRole) -> Div {
    div()
        .flex_shrink_0()
        .h(rem(24.))
        .flex()
        .items_center()
        .justify_center()
        .bg(color(background))
        .child(
            div()
                .w(rem(108.))
                .h(rem(4.))
                .rounded(rem(12.))
                .bg(color(ColorRole::ON_SURFACE)),
        )
}

/// The kit's browser bar: navigation glyphs, the address field, the
/// profile monogram and the overflow glyph. Decoration, not controls.
fn browser_bar() -> Div {
    let glyph = |name: &'static str| {
        icon(name)
            .size(rem(20.))
            .color(color(ColorRole::ON_SURFACE_VARIANT))
    };
    div()
        .flex_shrink_0()
        .h(rem(68.))
        .px(rem(16.))
        .flex()
        .items_center()
        .gap(rem(16.))
        .border_b_1()
        .border_color(color(ColorRole::OUTLINE_VARIANT))
        .child(
            div()
                .flex()
                .gap(rem(12.))
                .child(glyph("arrow_back"))
                .child(glyph("arrow_forward"))
                .child(glyph("refresh")),
        )
        .child(
            div()
                .flex_1()
                .h(rem(36.))
                .px(rem(16.))
                .rounded_full()
                .bg(color(ColorRole::SURFACE_CONTAINER))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(rem(8.))
                        .child(
                            filled("lock")
                                .size(rem(15.))
                                .color(color(ColorRole::ON_SURFACE_VARIANT)),
                        )
                        .child(text(
                            TypeRole::BodyLarge,
                            ColorRole::ON_SURFACE,
                            "www.url.com",
                        )),
                )
                .child(glyph("star")),
        )
        .child(
            div()
                .size(rem(28.))
                .rounded_full()
                .bg(color(ColorRole::OUTLINE))
                .flex()
                .items_center()
                .justify_center()
                .child(text(TypeRole::TitleSmall, ColorRole::SURFACE, "M")),
        )
        .child(glyph("more_vert"))
}

/// An app bar icon button; `lead` paints the navigation icon's on-surface
/// ink.
pub fn bar_button(
    id: ElementId,
    name: &'static str,
    label: &'static str,
    lead: bool,
) -> ToolbarButton {
    let button = ToolbarButton::icon(id, nav_icon(name), label);
    if lead {
        button.ink(ColorRole::ON_SURFACE)
    } else {
        button
    }
}

/// A section's title row: a title-large heading and a standard arrow icon
/// button, `inset` from the sides.
pub fn section_header(id: ElementId, title: &'static str, action: &'static str, inset: f32) -> Div {
    div()
        .flex_shrink_0()
        .h(rem(48.))
        .px(rem(inset))
        .flex()
        .items_center()
        .child(text(TypeRole::TitleLarge, ColorRole::ON_SURFACE, title))
        .child(IconButton::new(id, icon("arrow_forward"), action))
}

/// A horizontally scrolling row of `children`, `inset` at the leading edge.
pub fn scroll_row(id: ElementId, inset: f32, gap: f32) -> Stateful<Div> {
    div()
        .id(id)
        .w_full()
        .flex()
        .gap(rem(gap))
        .px(rem(inset))
        .overflow_x_scroll()
        .restrict_scroll_to_axis()
}

/// A 96dp artwork tile with a label under it: round (an artist) with a
/// centered label, or rounded-square (an album) with a leading label.
pub fn tile(round: bool, label: &'static str) -> Div {
    div()
        .w(rem(96.))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(rem(if round { 8. } else { 4. }))
        .child(
            art(56.)
                .size(rem(96.))
                .when(round, |art| art.rounded_full())
                .when(!round, |art| art.rounded(rem(16.))),
        )
        .child(
            line(TypeRole::LabelLarge, ColorRole::ON_SURFACE, label)
                .when(round, |label| label.text_center()),
        )
}

/// A filled media card: artwork over a title and description.
pub fn media_card(title: &'static str, description: &'static str) -> CardSurface {
    CardSurface::new(CardVariant::Filled)
        .w(rem(132.))
        .flex_shrink_0()
        .rounded(rem(12.))
        .p(rem(8.))
        .gap(rem(8.))
        .child(art(56.).w_full().h(rem(119.)).rounded(rem(8.)))
        .child(
            div()
                .flex()
                .flex_col()
                .pr(rem(8.))
                .child(line(TypeRole::TitleSmall, ColorRole::ON_SURFACE, title))
                .child(line(
                    TypeRole::BodySmall,
                    ColorRole::ON_SURFACE_VARIANT,
                    description,
                )),
        )
}

/// A feature item: 120dp artwork beside a title, a two-line description and
/// a meta row (a starred duration and a play icon button).
pub fn feature_item(
    id: ElementId,
    title: &'static str,
    description: &'static str,
    meta: &'static str,
    play: &'static str,
) -> Div {
    div()
        .w_full()
        .h(rem(120.))
        .flex()
        .gap(rem(16.))
        .child(art(64.).size(rem(120.)).rounded(rem(16.)))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(rem(4.))
                        .child(line(TypeRole::TitleLarge, ColorRole::ON_SURFACE, title))
                        .child(
                            text(
                                TypeRole::BodyMedium,
                                ColorRole::ON_SURFACE_VARIANT,
                                description,
                            )
                            .line_clamp(2),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(rem(4.))
                                .child(
                                    filled("stars")
                                        .size(rem(20.))
                                        .color(color(ColorRole::ON_SURFACE_VARIANT)),
                                )
                                .child(text(
                                    TypeRole::BodySmall,
                                    ColorRole::ON_SURFACE_VARIANT,
                                    meta,
                                )),
                        )
                        .child(IconButton::new(id, filled("play_arrow"), play)),
                ),
        )
}

/// A chat bubble: received messages sit on surface-container-high with a
/// tight lower-leading corner, sent ones on secondary with a tight
/// lower-trailing corner, and suggested replies on secondary-container.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Bubble {
    Received,
    Sent,
    Suggested,
}

pub fn bubble(kind: Bubble, message: impl Into<SharedString>) -> Div {
    let (container, ink) = match kind {
        Bubble::Received => (
            ColorRole::SURFACE_CONTAINER_HIGH,
            ColorRole::ON_SURFACE_VARIANT,
        ),
        Bubble::Sent => (ColorRole::SECONDARY, ColorRole::ON_SECONDARY),
        Bubble::Suggested => (
            ColorRole::SECONDARY_CONTAINER,
            ColorRole::ON_SECONDARY_CONTAINER,
        ),
    };
    let (tight_leading, tight_trailing) = match kind {
        Bubble::Received => (true, false),
        Bubble::Sent | Bubble::Suggested => (false, true),
    };
    let corner = |tight: bool| rem(if tight { 8. } else { 20. });
    text(TypeRole::BodyLarge, ink, message)
        .flex_shrink_0()
        .px(rem(16.))
        .py(rem(8.))
        .bg(color(container))
        .rounded_t(rem(20.))
        .rounded_bl(corner(tight_leading))
        .rounded_br(corner(tight_trailing))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn every_example_symbol_is_in_the_catalog() {
        for name in [
            "interests",
            "signal_wifi_4_bar",
            "signal_cellular_4_bar",
            "battery_full",
            "arrow_back",
            "arrow_forward",
            "refresh",
            "lock",
            "star",
            "more_vert",
            "stars",
            "play_arrow",
            "account_circle",
            "notifications",
            "settings",
            "attach_file",
            "today",
            "bookmark",
            "pause",
            "skip_next",
            "favorite",
            "directions_walk",
            "swap_vert",
            "list",
            "add_circle",
            "mood",
            "menu",
            "search",
            "edit",
            "image",
            "photo_camera",
            "mic",
            "menu_open",
            "edit_note",
            "checklist",
        ] {
            assert!(Symbol::named(name).is_some(), "{name}");
        }
    }

    #[test]
    fn element_ids_are_unique_per_screen() {
        assert_ne!(eid("x", 0, 1), eid("x", 1, 1));
        assert_ne!(eid("x", 0, 1), eid("x", 0, 2));
    }
}
