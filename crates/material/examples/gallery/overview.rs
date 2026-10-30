//! The component overview, like md3.io's /components/: per navigation
//! group, a grid of cards, each a small live thumbnail of the real
//! component over its name and description; clicking a card opens its page.
//!
//! Thumbnails are specimens: every interactive part is in
//! `InteractionPreview::Enabled` (out of the tab order, no hover), nothing
//! repeats an animation, and the card itself is the one link and keyboard
//! stop. The grid computes its column count from the content width, so it
//! reflows instead of scrolling sideways.
use super::catalog::{CATALOG, Group, Kind};
use super::playground::symbol;
use super::spec::{self, type_role};
use super::*;
use gpui_base::slider::SliderState;
use gpui_kit_material::icons::{Icon, Symbol};

/// The narrowest a card grows before the grid drops a column.
const CARD_MIN_WIDTH: f32 = 320.;
const CARD_GAP: f32 = 20.;
const CARD_RADIUS: f32 = 16.;
/// The thumbnail area's height.
const THUMBNAIL_HEIGHT: f32 = 180.;

const STILL: InteractionPreview = InteractionPreview::Enabled;

/// Cards per row for a grid `width` design px wide.
pub(crate) fn columns(width: f32) -> usize {
    (((width + CARD_GAP) / (CARD_MIN_WIDTH + CARD_GAP)).floor() as usize).max(1)
}

fn icon(name: &str) -> Icon {
    symbol(name, 24.)
}

fn row() -> Div {
    div().flex().items_center().justify_center().gap(rem(12.))
}

/// The component thumbnail for catalog entry `id`.
pub(crate) fn thumbnail(id: &'static str) -> AnyElement {
    let key = |part: &'static str| ElementId::Name(format!("thumb-{id}-{part}").into());
    match id {
        "button" => row()
            .child(
                MaterialButton::new(key("filled"))
                    .preview_state(STILL)
                    .child("Button"),
            )
            .child(
                MaterialButton::new(key("outlined"))
                    .variant(ButtonVariant::Outlined)
                    .preview_state(STILL)
                    .child("Button"),
            )
            .into_any_element(),
        "icon-button" => row()
            .children(
                [
                    IconButtonVariant::Standard,
                    IconButtonVariant::Filled,
                    IconButtonVariant::Tonal,
                    IconButtonVariant::Outlined,
                ]
                .into_iter()
                .enumerate()
                .map(|(index, variant)| {
                    IconButton::new(
                        ElementId::NamedInteger("thumb-icon-button".into(), index as u64),
                        icon("favorite"),
                        "Favorite",
                    )
                    .variant(variant)
                    .preview_state(STILL)
                }),
            )
            .into_any_element(),
        "button-group" => ConnectedButtonGroup::new(key("group"), "Text style")
            .options([
                ("bold", "Bold"),
                ("italic", "Italic"),
                ("underline", "Underline"),
            ])
            .selected("bold")
            .preview_state("bold", STILL)
            .into_any_element(),
        "split-button" => SplitButton::new(key("split"), "Save", "More save options")
            .preview_state(SplitButtonPart::Leading, STILL)
            .into_any_element(),
        "fab" => Fab::new(key("fab"), icon("add"), "Create")
            .preview_state(STILL)
            .into_any_element(),
        "fab-menu" => Fab::new(key("fab"), icon("edit"), "Open actions")
            .preview_state(STILL)
            .into_any_element(),
        "extended-fab" => ExtendedFab::new(key("fab"), icon("edit"), "Compose")
            .preview_state(STILL)
            .into_any_element(),
        "checkbox" => gpui_base::v_flex()
            .gap(rem(4.))
            .children(
                [
                    ("Additions", CheckboxState::Indeterminate),
                    ("Pickles", CheckboxState::Checked),
                    ("Tomato", CheckboxState::Unchecked),
                ]
                .into_iter()
                .enumerate()
                .map(|(index, (label, state))| {
                    row()
                        .justify_start()
                        .gap(rem(4.))
                        .when(index > 0, |row| row.pl(rem(24.)))
                        .child(
                            MaterialCheckbox::new(
                                ElementId::NamedInteger("thumb-checkbox".into(), index as u64),
                                label,
                            )
                            .state(state)
                            .preview_state(STILL),
                        )
                        .child(thumb_label(label))
                }),
            )
            .into_any_element(),
        "switch" => row()
            .gap(rem(24.))
            .child(thumb_label("Notifications"))
            .child(
                MaterialSwitch::new(key("switch"), "Notifications")
                    .checked(true)
                    .icons(SwitchIcons::Selected)
                    .preview_state(STILL),
            )
            .into_any_element(),
        "radio-buttons" => gpui_base::v_flex()
            .gap(rem(4.))
            .children(
                ["Daily", "Weekly"]
                    .into_iter()
                    .enumerate()
                    .map(|(index, label)| {
                        row()
                            .justify_start()
                            .gap(rem(4.))
                            .child(
                                MaterialRadio::new(
                                    ElementId::NamedInteger("thumb-radio".into(), index as u64),
                                    label,
                                )
                                .selected(index == 0)
                                .preview_state(STILL),
                            )
                            .child(thumb_label(label))
                    }),
            )
            .into_any_element(),
        "chips" => row()
            .gap(rem(8.))
            .child(
                FilterChip::new(key("filter"), "Vegan")
                    .selected(true)
                    .preview_state(STILL),
            )
            .child(AssistChip::new(key("assist"), "Directions").preview_state(STILL))
            .into_any_element(),
        "slider" => SliderThumbnail.into_any_element(),
        "text-field" => TextField::specimen(key("field"), "Ada Lovelace")
            .label("Name")
            .preview_state(STILL)
            .into_any_element(),
        "select" => ExposedDropdown::new(key("select"), "Fruit")
            .options(["Apple", "Banana", "Cherry"])
            .selected(Some(1))
            .width(200.)
            .into_any_element(),
        "search" => div()
            .w(rem(280.))
            .child(
                TextField::specimen(key("search"), "")
                    .placeholder("Search mail")
                    .leading_icon(icon("search"))
                    .preview_state(STILL),
            )
            .into_any_element(),
        "date-picker" => TextField::specimen(key("date"), "10/30/2026")
            .variant(TextFieldVariant::Outlined)
            .label("Date")
            .trailing_icon(icon("calendar_today"))
            .preview_state(STILL)
            .into_any_element(),
        "time-picker" => row()
            .gap(rem(8.))
            .child(time_box("09", true))
            .child(thumb_text(":", TypeRole::DisplayMedium))
            .child(time_box("41", false))
            .into_any_element(),
        "navigation-rail" => gpui_base::v_flex()
            .gap(rem(12.))
            .children(
                [("inbox", "Inbox", true), ("send", "Sent", false)]
                    .into_iter()
                    .map(|(name, label, selected)| nav_destination(name, label, selected)),
            )
            .into_any_element(),
        "navigation-suite" => div()
            .w(rem(300.))
            .child(
                NavigationBar::new(key("suite"), "Destinations")
                    .items(nav_items())
                    .selected("home"),
            )
            .into_any_element(),
        "canonical-layouts" => row()
            .child(
                surface(ColorRole::SURFACE_CONTAINER, 12.)
                    .w(rem(72.))
                    .h(rem(96.)),
            )
            .child(
                surface(ColorRole::SURFACE_CONTAINER_HIGHEST, 12.)
                    .w(rem(144.))
                    .h(rem(96.)),
            )
            .into_any_element(),
        "pull-to-refresh" => LoadingIndicator::new(key("pull"), "Refreshing")
            .contained(true)
            .progress(Some(0.7))
            .into_any_element(),
        "navigation-bar" => div()
            .w(rem(300.))
            .child(
                NavigationBar::new(key("bar"), "Destinations")
                    .items(nav_items())
                    .selected("home"),
            )
            .into_any_element(),
        "tabs" => div()
            .w(rem(300.))
            .child(
                MaterialTabs::new(key("tabs"), "Sections")
                    .items([
                        TabItem::new("flights", "Flights").preview_state(STILL),
                        TabItem::new("trips", "Trips").preview_state(STILL),
                        TabItem::new("explore", "Explore").preview_state(STILL),
                    ])
                    .selected("flights"),
            )
            .into_any_element(),
        "menu" => surface(ColorRole::SURFACE_CONTAINER, 8.)
            .w(rem(180.))
            .py(rem(8.))
            .children(["Cut", "Copy", "Paste"].into_iter().map(|item| {
                div()
                    .h(rem(44.))
                    .px(rem(12.))
                    .flex()
                    .items_center()
                    .child(thumb_label(item))
            }))
            .into_any_element(),
        "top-app-bar" => div()
            .w(rem(300.))
            .child(
                TopAppBar::new(key("bar"), TopAppBarVariant::Small)
                    .title("Inbox")
                    .navigation_icon(thumb_icon_button(key("menu"), "menu", "Menu"))
                    .action(thumb_icon_button(key("search"), "search", "Search"))
                    .preview_state(STILL),
            )
            .into_any_element(),
        "bottom-app-bar" => surface(ColorRole::SURFACE_CONTAINER, 0.)
            .w(rem(300.))
            .h(rem(80.))
            .px(rem(4.))
            .flex()
            .flex_row()
            .items_center()
            .children(
                [("check_box", "Tasks"), ("edit", "Edit"), ("mic", "Voice")]
                    .into_iter()
                    .enumerate()
                    .map(|(index, (name, label))| {
                        thumb_icon_button(
                            ElementId::NamedInteger("thumb-bottom-bar".into(), index as u64),
                            name,
                            label,
                        )
                    }),
            )
            .child(div().flex_1())
            .child(
                div()
                    .pr(rem(12.))
                    .child(Fab::new(key("fab"), icon("add"), "New task").preview_state(STILL)),
            )
            .into_any_element(),
        "toolbar" => surface(ColorRole::SURFACE_CONTAINER, 999.)
            .px(rem(8.))
            .h(rem(64.))
            .flex()
            .flex_row()
            .items_center()
            .gap(rem(4.))
            .children(
                [("arrow_back", "Back"), ("star", "Star"), ("share", "Share")]
                    .into_iter()
                    .enumerate()
                    .map(|(index, (name, label))| {
                        thumb_icon_button(
                            ElementId::NamedInteger("thumb-toolbar".into(), index as u64),
                            name,
                            label,
                        )
                    }),
            )
            .into_any_element(),
        "card" => surface(ColorRole::SURFACE_CONTAINER_HIGHEST, 12.)
            .w(rem(220.))
            .p(rem(16.))
            .gap(rem(8.))
            .child(thumb_text("Explore", TypeRole::TitleMedium))
            .child(thumb_text("Places near you", TypeRole::BodyMedium))
            .child(
                row().justify_end().child(
                    MaterialButton::new(key("save"))
                        .variant(ButtonVariant::Tonal)
                        .button_size(ButtonSize::ExtraSmall)
                        .preview_state(STILL)
                        .child("Save"),
                ),
            )
            .into_any_element(),
        "list" => surface(ColorRole::SURFACE, 0.)
            .w(rem(280.))
            .child(
                ListItem::new(key("one"), "Ada Lovelace")
                    .supporting("Analytical engine notes")
                    .leading(ListLeading::Monogram("A".into())),
            )
            .child(
                ListItem::new(key("two"), "Grace Hopper")
                    .supporting("Compiler draft")
                    .leading(ListLeading::Monogram("G".into())),
            )
            .into_any_element(),
        "carousel" => row()
            .gap(rem(8.))
            .children(
                [120., 72., 40.]
                    .into_iter()
                    .enumerate()
                    .map(|(index, width)| {
                        div()
                            .w(rem(width))
                            .h(rem(120.))
                            .rounded(CAROUSEL_ITEM_RADIUS)
                            .bg(color(if index == 0 {
                                ColorRole::PRIMARY_CONTAINER
                            } else {
                                ColorRole::SECONDARY_CONTAINER
                            }))
                    }),
            )
            .into_any_element(),
        "divider" => gpui_base::v_flex()
            .w(rem(240.))
            .gap(rem(12.))
            .child(thumb_label("Ideas for today"))
            .child(MaterialDivider::new(key("divider")))
            .child(thumb_label("Places to explore"))
            .into_any_element(),
        "dialog" => surface(ColorRole::SURFACE_CONTAINER_HIGH, 28.)
            .w(rem(240.))
            .p(rem(20.))
            .gap(rem(12.))
            .child(thumb_text("Discard draft?", TypeRole::TitleLarge))
            .child(
                row()
                    .justify_end()
                    .gap(rem(4.))
                    .child(text_button(key("cancel"), "Cancel"))
                    .child(text_button(key("discard"), "Discard")),
            )
            .into_any_element(),
        "bottom-sheet" => surface(ColorRole::SURFACE_CONTAINER_LOW, 0.)
            .w(rem(260.))
            .h(rem(120.))
            .rounded_t(rem(28.))
            .items_center()
            .pt(rem(16.))
            .child(drag_handle())
            .into_any_element(),
        "side-sheet" => row()
            .w(rem(260.))
            .h(rem(140.))
            .justify_end()
            .rounded(rem(12.))
            .bg(color(ColorRole::SURFACE_CONTAINER_HIGHEST))
            .overflow_hidden()
            .child(
                surface(ColorRole::SURFACE_CONTAINER_LOW, 0.)
                    .w(rem(120.))
                    .h_full()
                    .rounded_l(rem(16.))
                    .p(rem(16.))
                    .child(thumb_text("Filters", TypeRole::TitleSmall)),
            )
            .into_any_element(),
        "badge" => Badge::large(key("badge"), "3")
            .inset(8.)
            .anchor(thumb_icon_button(key("inbox"), "inbox", "Inbox, 3 new"))
            .into_any_element(),
        // A determinate flat bar settles at once: nothing repeats.
        "progress" => div()
            .w(rem(240.))
            .child(LinearProgress::new(key("progress"), "Uploading").value(Some(0.6)))
            .into_any_element(),
        "loading-indicator" => LoadingIndicator::new(key("loading"), "Loading")
            .contained(true)
            .shape(LoadingShape::SoftBurst)
            .into_any_element(),
        "snackbar" => surface(ColorRole::INVERSE_SURFACE, 4.)
            .flex_row()
            .items_center()
            .gap(rem(12.))
            .pl(rem(16.))
            .pr(rem(8.))
            .h(rem(48.))
            .child(
                thumb_text("Photo archived", TypeRole::BodyMedium)
                    .text_color(color(ColorRole::INVERSE_ON_SURFACE)),
            )
            .child(
                thumb_text("Undo", TypeRole::LabelLarge)
                    .px(rem(12.))
                    .text_color(color(ColorRole::INVERSE_PRIMARY)),
            )
            .into_any_element(),
        "tooltip" => gpui_base::v_flex()
            .items_center()
            .gap(rem(4.))
            .child(MaterialTooltip::plain(key("tooltip"), "Save to favorites"))
            .child(thumb_icon_button(key("favorite"), "favorite", "Favorite"))
            .into_any_element(),
        _ => div().into_any_element(),
    }
}

fn surface(role: ColorRole, radius: f32) -> Div {
    div().flex().flex_col().bg(color(role)).rounded(rem(radius))
}

fn thumb_text(text: &'static str, role: TypeRole) -> Div {
    div()
        .map(|text| type_role(text, role))
        .text_color(color(ColorRole::ON_SURFACE))
        .whitespace_nowrap()
        .child(text)
}

fn thumb_label(text: &'static str) -> Div {
    thumb_text(text, TypeRole::BodyLarge)
}

fn thumb_icon_button(id: ElementId, name: &str, label: &'static str) -> IconButton {
    IconButton::new(id, icon(name), label).preview_state(STILL)
}

fn text_button(id: ElementId, label: &'static str) -> MaterialButton {
    MaterialButton::new(id)
        .variant(ButtonVariant::Text)
        .preview_state(STILL)
        .child(label)
}

fn drag_handle() -> Div {
    div()
        .w(rem(32.))
        .h(rem(4.))
        .rounded_full()
        .bg(color(ColorRole::ON_SURFACE_VARIANT).opacity(0.4))
}

fn time_box(text: &'static str, selected: bool) -> Div {
    div()
        .w(rem(72.))
        .h(rem(64.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(rem(8.))
        .bg(color(if selected {
            ColorRole::PRIMARY_CONTAINER
        } else {
            ColorRole::SURFACE_CONTAINER_HIGHEST
        }))
        .child(
            thumb_text(text, TypeRole::DisplayMedium).text_color(color(if selected {
                ColorRole::ON_PRIMARY_CONTAINER
            } else {
                ColorRole::ON_SURFACE
            })),
        )
}

fn nav_items() -> [NavItem; 3] {
    [
        ("home", "Home"),
        ("explore", "Explore"),
        ("person", "Profile"),
    ]
    .map(|(name, label)| {
        NavItem::new(
            name,
            label,
            NavIcon::symbol(Symbol::named(name).expect("a Material Symbol")),
        )
        .preview_state(STILL)
    })
}

/// A rail destination: the active indicator pill around the icon over its
/// label.
fn nav_destination(name: &'static str, label: &'static str, selected: bool) -> Div {
    gpui_base::v_flex()
        .items_center()
        .gap(rem(4.))
        .child(
            div()
                .w(rem(56.))
                .h(rem(32.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .when(selected, |pill| {
                    pill.bg(color(ColorRole::SECONDARY_CONTAINER))
                })
                .child(icon(name).color(color(if selected {
                    ColorRole::ON_SECONDARY_CONTAINER
                } else {
                    ColorRole::ON_SURFACE_VARIANT
                }))),
        )
        .child(
            thumb_text(label, TypeRole::LabelMedium).text_color(color(if selected {
                ColorRole::SECONDARY
            } else {
                ColorRole::ON_SURFACE_VARIANT
            })),
        )
}

/// A still slider: the kit's slider at 40% with no keyboard stop. Its
/// state lives in the card while the card is mounted.
#[derive(gpui::IntoElement)]
struct SliderThumbnail;

impl RenderOnce for SliderThumbnail {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state("thumb-slider-state", cx, |_, _| {
            SliderState::new().min(0.).max(100.).default_value(40.)
        });
        div().w(rem(240.)).child(
            MaterialSlider::new(&state, "Volume")
                .id("thumb-slider")
                .preview_state(STILL),
        )
    }
}

impl Gallery {
    /// One overview group: its heading over the reflowing card grid.
    pub(super) fn render_overview_group(
        &self,
        group: Group,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let columns = columns(spec::content_width(window));
        let entries: Vec<usize> = CATALOG
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry.group == group && matches!(entry.kind, Kind::Component { .. })
            })
            .map(|(index, _)| index)
            .collect();
        let rows = entries.chunks(columns).map(|chunk| {
            div()
                .flex()
                .gap(rem(CARD_GAP))
                .children(chunk.iter().map(|index| self.overview_card(*index, cx)))
                // Keep the last row's cards in the grid's columns.
                .children(
                    (chunk.len()..columns).map(|_| div().flex_1().min_w_0().into_any_element()),
                )
        });
        gpui_base::v_flex()
            .gap(rem(16.))
            .pb(rem(40.))
            .child(
                accessible_text(
                    ("overview-group", group as usize),
                    group.title().to_uppercase(),
                )
                .role(Role::Heading)
                .map(|text| type_role(text, TypeRole::LabelLarge))
                .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
            )
            .child(gpui_base::v_flex().gap(rem(CARD_GAP)).children(rows))
            .into_any_element()
    }

    fn overview_card(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let entry = &CATALOG[index];
        let on_surface = color(ColorRole::ON_SURFACE);
        let outline = color(ColorRole::OUTLINE_VARIANT);
        let card = div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(color(ColorRole::SURFACE_CONTAINER_LOW))
            .border_1()
            .border_color(outline)
            .rounded(rem(CARD_RADIUS))
            .overflow_hidden()
            .child(
                div()
                    .h(rem(THUMBNAIL_HEIGHT))
                    .flex()
                    .items_center()
                    .justify_center()
                    .overflow_hidden()
                    .bg(color(ColorRole::SURFACE))
                    .border_b_1()
                    .border_color(outline)
                    .child(thumbnail(entry.id)),
            )
            .child(
                gpui_base::v_flex()
                    .gap(rem(6.))
                    .px(rem(20.))
                    .py(rem(16.))
                    .child(
                        div()
                            .map(|text| type_role(text, TypeRole::TitleMedium))
                            .text_color(color(ColorRole::ON_SURFACE))
                            .child(entry.title),
                    )
                    .child(
                        div()
                            .whitespace_normal()
                            .map(|text| type_role(text, TypeRole::BodyMedium))
                            .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                            .child(entry.description),
                    ),
            );
        // The link covers the card, so the thumbnail's specimens never take
        // the pointer.
        // Keyboard focus draws the M3 3dp secondary ring on the card's edge.
        let ring = color(ColorRole::SECONDARY);
        let link = gpui_base::Button::new(("overview-card", index))
            .role(Role::Link)
            .accessibility_label(entry.title)
            .debug_selector(move || format!("overview-{}", entry.id))
            .absolute()
            .inset_0()
            .rounded(rem(CARD_RADIUS))
            .cursor_pointer()
            .hover(move |style| style.bg(on_surface.opacity(0.08)))
            .focus_visible(move |style| style.border_3().border_color(ring))
            .on_click(cx.listener(move |this, _, _, cx| this.navigate(index, cx)));
        div()
            .relative()
            .flex_1()
            .min_w_0()
            .child(card)
            .child(link)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_grid_drops_columns_as_it_narrows() {
        assert_eq!(columns(1200.), 3);
        assert_eq!(columns(700.), 2);
        assert_eq!(columns(500.), 1);
        assert_eq!(columns(100.), 1);
    }
}
