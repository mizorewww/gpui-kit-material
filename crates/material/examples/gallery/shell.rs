//! The Gallery shell: the navigation, the current page's rows (md3.io's
//! page header, the Live preview | Specs card beside the configuration
//! card, overview cards, spec boards) and keyboard traversal across the
//! virtual rows. The GPUI list measures only the viewport; Kit Component
//! List requires precomputed sizes, while MessageScroller imposes log/tail
//! semantics inappropriate for these pages.
use super::catalog::{CATALOG, Group, Kind};
use super::playground::CompactSelect;
use super::spec::type_role;
use super::*;
use std::sync::OnceLock;

/// The widest the page content grows, in design px.
const CONTENT_MAX_WIDTH: f32 = 1240.;
const GUTTER: f32 = 40.;
const GUTTER_COMPACT: f32 = 12.;
const NAV_WIDTH: f32 = 248.;
const CARD_RADIUS: f32 = 16.;
const CONFIG_WIDTH: f32 = 340.;
/// The card header: tabs, theme select and mode.
const CARD_HEADER: f32 = 72.;
const MIN_PANE: f32 = 460.;
/// The height of the page header above the panes, in design px.
const LEAD_HEADER: f32 = 196.;
const COMPACT_CANVAS: f32 = 320.;
/// A spec card's side padding.
const SPEC_PADDING: f32 = 32.;
const SPEC_PADDING_COMPACT: f32 = 16.;

/// md3.io's theme order, with the select's labels.
pub(crate) const THEMES: [(ThemeId, &str); 20] = [
    (ThemeId::Baseline, "Baseline"),
    (ThemeId::Neutral, "Neutral"),
    (ThemeId::Vibrant, "Vibrant"),
    (ThemeId::Expressive, "Expressive"),
    (ThemeId::Fidelity, "Fidelity"),
    (ThemeId::Content, "Content"),
    (ThemeId::Monochrome, "Monochrome"),
    (ThemeId::Rainbow, "Rainbow"),
    (ThemeId::FruitSalad, "Fruit salad"),
    (ThemeId::Ocean, "Ocean"),
    (ThemeId::Forest, "Forest"),
    (ThemeId::Spring, "Spring"),
    (ThemeId::Sunset, "Sunset"),
    (ThemeId::Autumn, "Autumn"),
    (ThemeId::Desert, "Desert"),
    (ThemeId::Summer, "Summer"),
    (ThemeId::Brownbeige, "Brownbeige"),
    (ThemeId::Sageivory, "Sageivory"),
    (ThemeId::Tealcaramel, "Tealcaramel"),
    (ThemeId::Highcontrast, "Highcontrast"),
];

fn card_background() -> Rgba {
    color(ColorRole::SURFACE_CONTAINER_LOW)
}
fn canvas_background() -> Rgba {
    color(ColorRole::SURFACE)
}
fn outline() -> Rgba {
    color(ColorRole::OUTLINE_VARIANT)
}

/// The width inside a spec card, in design px.
pub(super) fn spec_content_width(window: &Window) -> f32 {
    let compact = is_compact(window);
    let (nav, gutter, padding) = if compact {
        (0., GUTTER_COMPACT, SPEC_PADDING_COMPACT)
    } else {
        (NAV_WIDTH, GUTTER, SPEC_PADDING)
    };
    (design_width(window) - nav - 2. * gutter).min(CONTENT_MAX_WIDTH) - 2. * (padding + 1.)
}

/// The component groups the overview lists, in navigation order.
pub(super) fn overview_groups() -> impl Iterator<Item = Group> {
    let mut last = None;
    CATALOG.iter().filter_map(move |entry| {
        let group = entry.group;
        let new = matches!(entry.kind, Kind::Component { .. }) && last != Some(group);
        last = Some(group);
        new.then_some(group)
    })
}

/// "ACTIONS / BUTTON" per entry, built once.
fn breadcrumb(entry: usize) -> SharedString {
    static CRUMBS: OnceLock<Vec<SharedString>> = OnceLock::new();
    CRUMBS.get_or_init(|| {
        CATALOG
            .iter()
            .map(|entry| {
                let group = match entry.kind {
                    Kind::Overview => "Explore the library",
                    _ => entry.group.title(),
                };
                if matches!(entry.kind, Kind::Overview) {
                    group.to_uppercase().into()
                } else {
                    format!("{} / {}", group, entry.title).to_uppercase().into()
                }
            })
            .collect()
    })[entry]
        .clone()
}

/// A full-width card with md3.io's outline.
fn card() -> Div {
    div()
        .flex()
        .flex_col()
        .bg(card_background())
        .border_1()
        .border_color(outline())
        .rounded(rem(CARD_RADIUS))
        .overflow_hidden()
}

impl Gallery {
    pub(super) fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Entries whose title contains the search query, in catalog order.
    pub(crate) fn matches(&self) -> impl Iterator<Item = usize> + '_ {
        let query = self.query.trim().to_lowercase();
        CATALOG
            .iter()
            .enumerate()
            .filter(move |(_, entry)| {
                query.is_empty() || entry.title.to_lowercase().contains(&query)
            })
            .map(|(index, _)| index)
    }

    /// Let the shown page forward controlled changes to playbacks of
    /// unmounted rows.
    pub(super) fn after_layout(&self, rendered: &[usize], cx: &mut App) {
        let Some(built) = self
            .current_page()
            .and_then(|index| self.pages[index].as_ref())
        else {
            return;
        };
        // Run after list prepaint, when measurements are current. Fast can
        // carry visible rows without calling the builder; current intersection
        // protects those too. A preview's live demos are mounted in its card.
        let viewport = self.list.viewport_bounds();
        let demos = self.preview_demos(cx);
        let visible = |row: usize| {
            if demos.as_ref().is_some_and(|demos| demos.contains(&row)) {
                return true;
            }
            built.rows.global(row).is_some_and(|global| {
                rendered.contains(&global)
                    || self.list.bounds_for_item(global).is_some_and(|bounds| {
                        let overlap = bounds.intersect(&viewport);
                        overlap.size.width > px(0.) && overlap.size.height > px(0.)
                    })
            })
        };
        built.page.after_layout(&visible, cx);
    }

    /// The spec page rows a playground-less component shows in its preview:
    /// the slice's live demos, else its first row.
    pub(super) fn preview_demos(&self, cx: &App) -> Option<Vec<usize>> {
        let entry = &self.entries[self.current];
        if !matches!(
            CATALOG[self.current].kind,
            Kind::Component { playground: None }
        ) || entry.tab != Tab::Preview
        {
            return None;
        }
        let built = self.pages[self.current_page()?].as_ref()?;
        let slice = entry.slice.as_ref()?;
        let demos: Vec<usize> = slice
            .iter()
            .copied()
            .filter(|row| built.page.interactive(*row, cx))
            .collect();
        Some(if demos.is_empty() {
            slice.first().copied().into_iter().collect()
        } else {
            demos
        })
    }

    pub(super) fn interactive_row(&self, row: usize, cx: &App) -> bool {
        match self.rows[row] {
            Row::Lead | Row::Group(_) => true,
            Row::Spec(local) => self
                .current_page()
                .and_then(|index| self.pages[index].as_ref())
                .is_some_and(|built| built.page.interactive(local, cx)),
        }
    }

    fn adjacent_interactive_row(&self, row: usize, reverse: bool, cx: &App) -> Option<usize> {
        if reverse {
            (0..row)
                .rev()
                .find(|index| self.interactive_row(*index, cx))
        } else {
            (row + 1..self.row_count()).find(|index| self.interactive_row(*index, cx))
        }
    }

    #[cfg_attr(target_family = "wasm", allow(dead_code))]
    pub(super) fn scroll_to_top(&self) {
        self.list.scroll_to(ListOffset {
            item_ix: 0,
            offset_in_item: px(0.),
        });
    }

    pub(super) fn render_navigation(&self, compact: bool, cx: &mut Context<Self>) -> Stateful<Div> {
        let searching = !self.query.trim().is_empty();
        let section = CATALOG[self.current].group.section();
        let mut groups: Vec<AnyElement> = Vec::new();
        let mut items: Vec<AnyElement> = Vec::new();
        let mut group = None;
        let flush =
            |group: Option<Group>, items: &mut Vec<AnyElement>, groups: &mut Vec<AnyElement>| {
                if let Some(group) = group
                    && !items.is_empty()
                {
                    groups.push(
                        gpui_base::v_flex()
                            .gap(rem(2.))
                            .child(
                                div().px(rem(16.)).pt(rem(12.)).pb(rem(6.)).child(
                                    accessible_text(("nav-group", group as usize), group.title())
                                        .role(Role::Heading)
                                        .map(|text| type_role(text, TypeRole::TitleSmall))
                                        .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                                ),
                            )
                            .when(compact, |column| column.flex_row().flex_wrap())
                            .children(items.drain(..))
                            .into_any_element(),
                    );
                }
            };
        for index in self.matches() {
            let entry = &CATALOG[index];
            if !searching && entry.group.section() != section {
                continue;
            }
            if group != Some(entry.group) {
                flush(group, &mut items, &mut groups);
                group = Some(entry.group);
            }
            items.push(self.nav_item(index, compact, cx).into_any_element());
        }
        flush(group, &mut items, &mut groups);
        if groups.is_empty() {
            groups.push(
                div()
                    .px(rem(16.))
                    .pt(rem(16.))
                    .child(
                        accessible_text("nav-empty", "No matching pages")
                            .map(|text| type_role(text, TypeRole::BodyMedium))
                            .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                    )
                    .into_any_element(),
            );
        }
        div()
            .id("gallery-navigation")
            .role(Role::Navigation)
            .aria_label("Gallery pages")
            .track_focus(&self.navigation_focus)
            .tab_stop(false)
            .flex_shrink_0()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.navigation_scroll)
            .bg(spec::page_background())
            .when(compact, |nav| nav.w_full().max_h(rem(180.)).border_b_1())
            .when(!compact, |nav| nav.w(rem(NAV_WIDTH)).h_full().border_r_1())
            .border_color(outline())
            .child(
                gpui_base::v_flex()
                    .px(rem(12.))
                    .py(rem(12.))
                    .gap(rem(4.))
                    .children(groups),
            )
    }

    fn nav_item(&self, index: usize, compact: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let entry = &CATALOG[index];
        let selected = index == self.current;
        let on_surface = color(ColorRole::ON_SURFACE);
        FocusRing::new(
            ("nav-ring", index),
            gpui_base::Button::new(("navigate", index))
                .role(Role::Link)
                .accessibility_label(entry.title)
                .selected(selected)
                .debug_selector(move || format!("nav-{}", entry.id))
                .flex()
                .items_center()
                .justify_start()
                .h(rem(36.))
                .px(rem(16.))
                .rounded_full()
                .cursor_pointer()
                .when(!compact, |item| item.w_full())
                .map(|item| type_role(item, TypeRole::LabelLarge))
                .when(selected, |item| {
                    item.bg(color(ColorRole::SECONDARY_CONTAINER))
                        .text_color(color(ColorRole::ON_SECONDARY_CONTAINER))
                })
                .when(!selected, |item| {
                    item.font_weight(FontWeight::NORMAL)
                        .text_color(color(ColorRole::ON_SURFACE_VARIANT))
                        .hover(move |style| style.bg(on_surface.opacity(0.08)))
                })
                .child(entry.title)
                .on_click(cx.listener(move |this, _, _, cx| this.navigate(index, cx)))
                .reveal_handle(("reveal-navigation", index), &self.navigation_scroll),
        )
    }

    pub(super) fn render_row(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let compact = is_compact(window);
        let gutter = if compact { GUTTER_COMPACT } else { GUTTER };
        let content = match self.rows[index] {
            Row::Lead => self.lead.clone().into_any_element(),
            Row::Spec(row) => self.render_spec_row(index, row, window, cx),
            Row::Group(group) => self.render_overview_group(group, window, cx),
        };
        div()
            .id(("gallery-row", index))
            .when(index == 0, |row| {
                row.debug_selector(|| "gallery-first-row".into())
            })
            .track_focus(&self.row_focus[index])
            .tab_stop(false)
            .w_full()
            .px(rem(gutter))
            .when(index == 0, |row| row.pt(rem(24.)))
            .when(index + 1 == self.rows.len(), |row| row.pb(rem(40.)))
            .child(div().w_full().max_w(rem(CONTENT_MAX_WIDTH)).child(content))
            .into_any_element()
    }

    /// md3.io's page header: breadcrumb, title and description.
    fn render_header(current: usize) -> impl IntoElement {
        let entry = &CATALOG[current];
        let id = entry.id;
        gpui_base::v_flex()
            .debug_selector(move || format!("page-{id}"))
            .gap(rem(8.))
            .pb(rem(24.))
            .child(
                accessible_text("breadcrumb", breadcrumb(current))
                    .map(|text| type_role(text, TypeRole::LabelMedium))
                    .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
            )
            .child(
                accessible_text("page-title", entry.title)
                    .role(Role::Heading)
                    .map(|text| type_role(text, TypeRole::DisplaySmall))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(color(ColorRole::ON_SURFACE)),
            )
            .child(
                accessible_text("page-description", entry.description)
                    .whitespace_normal()
                    .map(|text| type_role(text, TypeRole::BodyLarge))
                    .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
            )
    }

    /// What the lead row shows, copied out so that drawing it only reads the
    /// Gallery. Writing to the Gallery while drawing would change what the
    /// Gallery and this row depend on and defeat their retained layouts.
    fn lead_props(&self, gallery: WeakEntity<Gallery>) -> LeadProps {
        let entry = &self.entries[self.current];
        LeadProps {
            current: self.current,
            tab: entry.tab,
            shared: self.shared.clone(),
            theme_menu_open: self.theme_menu_open,
            playground: entry.playground.clone(),
            views: entry.views.clone(),
            gallery,
        }
    }

    /// A component without a playground shows its spec page's live demos,
    /// which only the Gallery can render.
    fn lead_needs_gallery(&self) -> bool {
        matches!(CATALOG[self.current].kind, Kind::Component { .. })
            && self.entries[self.current].tab == Tab::Preview
            && self.entries[self.current].views.is_none()
    }

    /// The card's header: the Live preview | Specs tabs, the theme select
    /// and the light/dark segmented control.
    fn render_card_header(props: &LeadProps, cx: &mut App) -> impl IntoElement {
        let tab = props.tab;
        let gallery = props.gallery.clone();
        let shared = props.shared.read(cx);
        let theme = shared
            .reference_theme
            .and_then(|theme| THEMES.iter().position(|(id, _)| *id == theme));
        let dark = shared.reference_mode == ThemeMode::Dark;
        let update_shared = |f: fn(&mut GalleryShared, usize)| {
            let shared = props.shared.clone();
            move |value: usize, cx: &mut App| {
                shared.update(cx, |shared, cx| {
                    f(shared, value);
                    cx.notify();
                })
            }
        };
        let select_theme = update_shared(|shared, index| {
            shared.reference_theme = Some(THEMES[index].0);
        });
        let select_mode = update_shared(|shared, dark| {
            shared.reference_mode = if dark == 1 {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            };
            shared.reference_theme.get_or_insert(ThemeId::Baseline);
        });
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(rem(8.))
            .min_h(rem(CARD_HEADER))
            .px(rem(16.))
            .py(rem(8.))
            .border_b_1()
            .border_color(outline())
            .child(
                div()
                    // Two fixed tabs with room for "Live preview" at any
                    // text scale (both labels scale with the rem).
                    .w(rem(240.))
                    .flex_shrink_0()
                    .debug_selector(|| "card-tabs".into())
                    .child(
                        MaterialTabs::new("view-tabs", "Playground view")
                            .items([
                                TabItem::new("preview", "Live preview"),
                                TabItem::new("specs", "Specs"),
                            ])
                            .selected(if tab == Tab::Specs {
                                "specs"
                            } else {
                                "preview"
                            })
                            .on_change({
                                let gallery = gallery.clone();
                                move |value: &SharedString, _, cx| {
                                    let tab = if value == "specs" {
                                        Tab::Specs
                                    } else {
                                        Tab::Preview
                                    };
                                    let _ = gallery.update(cx, |this, cx| this.set_tab(tab, cx));
                                }
                            }),
                    ),
            )
            .child(div().flex_1())
            .child(
                accessible_text("theme-label", "Theme")
                    .map(|text| type_role(text, TypeRole::LabelLarge))
                    .text_color(color(ColorRole::ON_SURFACE)),
            )
            .child(
                CompactSelect::new("preview-theme", "Theme")
                    .options(THEMES.map(|(_, label)| SharedString::new_static(label)))
                    .selected(theme.unwrap_or(0))
                    .open(props.theme_menu_open)
                    .width(116.)
                    .on_open_change({
                        let gallery = gallery.clone();
                        move |open: &bool, _, cx| {
                            let _ = gallery.update(cx, |this, cx| {
                                this.theme_menu_open = *open;
                                cx.notify();
                            });
                        }
                    })
                    .on_select({
                        let gallery = gallery.clone();
                        move |index: &usize, _, cx| {
                            let _ = gallery.update(cx, |this, cx| {
                                this.theme_menu_open = false;
                                cx.notify();
                            });
                            select_theme(*index, cx);
                        }
                    }),
            )
            .child(
                ConnectedButtonGroup::new("preview-mode", "Preview mode")
                    .button_size(ButtonSize::ExtraSmall)
                    .options([("light", "Light"), ("dark", "Dark")])
                    .selected(if dark { "dark" } else { "light" })
                    .on_change(move |value: &SharedString, _, cx| {
                        select_mode(usize::from(value == "dark"), cx)
                    }),
            )
    }

    /// The preview card beside the configuration card.
    fn render_panes(
        props: &LeadProps,
        demos: Option<AnyElement>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let compact = is_compact(window);
        let design_height =
            f32::from(window.viewport_size().height) * BASE_PX / f32::from(window.rem_size());
        let pane = (design_height - f32::from(TITLE_BAR) - LEAD_HEADER - 24.).max(MIN_PANE);
        let playground = props.playground.clone();
        let canvas = match (&props.views, demos) {
            (Some(views), _) => views.preview.clone().into_any_element(),
            (None, Some(demos)) => demos,
            (None, None) => div().into_any_element(),
        };
        let fallback = playground.is_none();
        let preview_card = card()
            .flex_1()
            .min_w_0()
            .when(!compact, |card| card.h(rem(pane)))
            .child(Self::render_card_header(props, cx))
            .child(
                div()
                    .id("preview-canvas")
                    .debug_selector(|| "preview-canvas".into())
                    .flex_1()
                    .min_h(rem(COMPACT_CANVAS))
                    .bg(canvas_background())
                    .flex()
                    .flex_col()
                    .when(!fallback, |canvas| canvas.items_center().justify_center())
                    .when(fallback, |canvas| canvas.overflow_y_scroll().p(rem(24.)))
                    .p(rem(24.))
                    .child(canvas),
            );
        let config_card = Self::render_config_card(playground, props.views.clone(), compact, pane);
        div()
            .w_full()
            .flex()
            .gap(rem(24.))
            .when(compact, |panes| panes.flex_col())
            .child(preview_card)
            .child(config_card)
            .into_any_element()
    }

    /// The live demos of a component without a playground yet.
    pub(super) fn render_demos(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let demos = self.preview_demos(cx).unwrap_or_default();
        let Some(built) = self
            .current_page()
            .and_then(|index| self.pages[index].as_ref())
        else {
            return div().into_any_element();
        };
        gpui_base::v_flex()
            .w_full()
            .gap(rem(24.))
            .children(
                demos
                    .into_iter()
                    .map(|row| built.page.render_row(row, window, cx)),
            )
            .into_any_element()
    }

    fn render_config_card(
        playground: Option<Entity<PlaygroundState>>,
        views: Option<PlaygroundViews>,
        compact: bool,
        pane: f32,
    ) -> impl IntoElement {
        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .h(rem(56.))
            .px(rem(20.))
            .border_b_1()
            .border_color(outline())
            .child(
                accessible_text("configuration-title", "Configuration")
                    .role(Role::Heading)
                    .map(|text| type_role(text, TypeRole::TitleSmall))
                    .text_color(color(ColorRole::ON_SURFACE)),
            )
            .when_some(playground.clone(), |header, state| {
                header.child(
                    MaterialButton::new("configuration-reset")
                        .variant(ButtonVariant::Text)
                        .button_size(ButtonSize::ExtraSmall)
                        .debug_selector(|| "configuration-reset".into())
                        .child("Reset")
                        .on_click(move |_, window, cx| {
                            state.update(cx, |state, cx| state.reset(window, cx))
                        }),
                )
            });
        let body = match views {
            // Fixed outer bounds beside the preview, so the panel is cached with
            // its scroller inside: the preview writes the playground state on
            // every frame, which only draws a cached view again if notified.
            Some(PlaygroundViews { panel, .. }) if !compact => AnyView::from(panel)
                .cached(StyleRefinement::default().flex_1().min_h_0().w_full())
                .into_any_element(),
            Some(PlaygroundViews { panel, .. }) => div()
                .id("configuration-body")
                .flex_1()
                .min_h_0()
                .child(panel)
                .into_any_element(),
            None => div()
                .p(rem(20.))
                .child(
                    accessible_text(
                        "configuration-pending",
                        "This playground's configuration is on its way. The preview shows the component's live demos; Specs shows every variant and state.",
                    )
                    .whitespace_normal()
                    .map(|text| type_role(text, TypeRole::BodyMedium))
                    .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                )
                .into_any_element(),
        };
        card()
            .debug_selector(|| "configuration-card".into())
            .flex_shrink_0()
            .when(!compact, |card| card.w(rem(CONFIG_WIDTH)).h(rem(pane)))
            .child(header)
            .child(body)
    }

    /// A spec page row framed as a slice of its card.
    fn render_spec_row(
        &mut self,
        index: usize,
        row: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(built) = self
            .current_page()
            .and_then(|page| self.pages[page].as_ref())
        else {
            return div().into_any_element();
        };
        let content = built.page.render_row(row, window, cx);
        let first = index == 1;
        let last = index + 1 == self.rows.len();
        // Component Specs continue the card the lead row opened; a document
        // page's card starts with its first row.
        let opens_card = first && matches!(CATALOG[self.current].kind, Kind::Document);
        let gap = if first {
            32.
        } else {
            built.page.row_gap(row, cx)
        };
        let compact = is_compact(window);
        let radius = rem(CARD_RADIUS);
        div()
            .when(first, |panel| {
                let id = CATALOG[self.current].id;
                panel.debug_selector(move || format!("specs-{id}"))
            })
            .w_full()
            .bg(card_background())
            .border_l_1()
            .border_r_1()
            .border_color(outline())
            .px(rem(if compact {
                SPEC_PADDING_COMPACT
            } else {
                SPEC_PADDING
            }))
            .pt(rem(gap))
            .when(opens_card, |panel| panel.border_t_1().rounded_t(radius))
            .when(last, |panel| {
                panel.border_b_1().rounded_b(radius).pb(rem(32.))
            })
            .child(content)
            .into_any_element()
    }

    // The public traversal API exposes the rendered tab order, not candidate
    // enumeration. Scan at most one cycle, as Base Root does for focus traps.
    pub(super) fn focus_row(
        &mut self,
        row: usize,
        reverse: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(target) = self.row_focus.get(row) else {
            return;
        };
        let target = target.clone();
        let original = window.focused(cx);
        let mut visited = Vec::new();
        loop {
            if reverse {
                window.focus_prev(cx);
            } else {
                window.focus_next(cx);
            }
            if target.contains_focused(window, cx) {
                return;
            }
            let current = window.focused(cx);
            if visited.contains(&current) {
                break;
            }
            visited.push(current);
        }
        // A controlled row may have become fully disabled. Continue through the
        // logical catalog instead of trapping focus on an empty row container.
        if let Some(original) = original {
            original.focus(window, cx);
        }
        if let Some(next) = self.adjacent_interactive_row(row, reverse, cx) {
            self.list.scroll_to(ListOffset {
                item_ix: next,
                offset_in_item: px(0.),
            });
            self.pending_focus = Some((next, reverse));
            cx.notify();
        }
    }

    pub(super) fn advance_gallery_focus(
        &mut self,
        reverse: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if gpui_base::active_focus_trap(window, cx).is_some() {
            cx.propagate();
            return;
        }
        let previous = window.focused(cx);
        let current_row = self
            .row_focus
            .iter()
            .position(|focus| focus.contains_focused(window, cx));
        let from_navigation = self.navigation_focus.contains_focused(window, cx);
        if current_row.is_none() && !from_navigation {
            cx.propagate();
            return;
        }
        if reverse {
            window.focus_prev(cx);
        } else {
            window.focus_next(cx);
        }
        if let Some(row) = current_row {
            if self.row_focus[row].contains_focused(window, cx) {
                return;
            }
        } else if self.navigation_focus.contains_focused(window, cx) {
            return;
        }
        let target = match current_row {
            Some(row) => self.adjacent_interactive_row(row, reverse, cx),
            None if reverse => (0..self.row_count())
                .rev()
                .find(|row| self.interactive_row(*row, cx)),
            None => (0..self.row_count()).find(|row| self.interactive_row(*row, cx)),
        };
        if let Some(target) = target {
            // Moving focus into the next row is only needed when it is not
            // mounted; a mounted neighbour already took focus above.
            let mounted = self
                .row_focus
                .get(target)
                .is_some_and(|focus| focus.contains_focused(window, cx));
            if mounted {
                return;
            }
            if let Some(previous) = previous {
                previous.focus(window, cx);
            }
            self.list.scroll_to(ListOffset {
                item_ix: target,
                offset_in_item: px(0.),
            });
            self.pending_focus = Some((target, reverse));
            cx.notify();
        }
    }
}

/// What [`LeadView`] draws: a snapshot of the Gallery's state for this row.
struct LeadProps {
    current: usize,
    tab: Tab,
    shared: Entity<GalleryShared>,
    theme_menu_open: bool,
    playground: Option<Entity<PlaygroundState>>,
    views: Option<PlaygroundViews>,
    gallery: WeakEntity<Gallery>,
}

/// The lead row as its own view. A preview animation, hover or press
/// notifies the preview view, and the views around it become dirty. They are
/// spliced, keeping their layout, only if what they depend on did not change,
/// so this view reads the Gallery and never writes it while drawing. It
/// renders again when the Gallery notifies (navigation, tab, theme menu), its
/// invalidation owner.
pub(super) struct LeadView {
    gallery: WeakEntity<Gallery>,
    _observe: Subscription,
    #[cfg(test)]
    pub(super) renders: std::cell::Cell<usize>,
}

impl LeadView {
    pub(super) fn new(gallery: Entity<Gallery>, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&gallery, |_, _, cx| cx.notify());
        Self {
            gallery: gallery.downgrade(),
            _observe: observe,
            #[cfg(test)]
            renders: Default::default(),
        }
    }
}

impl Render for LeadView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        #[cfg(test)]
        self.renders.set(self.renders.get() + 1);
        let Some(gallery) = self.gallery.upgrade() else {
            return div().into_any_element();
        };
        let demos = gallery
            .read(cx)
            .lead_needs_gallery()
            .then(|| gallery.update(cx, |gallery, cx| gallery.render_demos(window, cx)));
        let props = gallery.read(cx).lead_props(self.gallery.clone());
        let body = match CATALOG[props.current].kind {
            Kind::Overview | Kind::Document => None,
            Kind::Component { .. } => Some(match props.tab {
                Tab::Preview => Gallery::render_panes(&props, demos, window, cx),
                Tab::Specs => card()
                    .rounded_b(px(0.))
                    .border_b_0()
                    .child(Gallery::render_card_header(&props, cx))
                    .into_any_element(),
            }),
        };
        gpui_base::v_flex()
            .w_full()
            .child(Gallery::render_header(props.current))
            .children(body)
            .into_any_element()
    }
}
