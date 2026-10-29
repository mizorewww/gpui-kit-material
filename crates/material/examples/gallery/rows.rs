//! Lazily constructed, variable-height Gallery rows. The shared GPUI list
//! measures only the viewport; Kit Component List requires precomputed sizes,
//! while MessageScroller imposes log/tail semantics inappropriate for this page.
use super::*;

impl Gallery {
    pub(super) fn sync_button_transitions(&self, rendered: &[usize], cx: &App) {
        #[cfg(test)]
        self.button_sync_rows.borrow_mut().clear();
        // Run after list prepaint, when measurements are current. Fast can
        // carry visible rows without calling the builder; current intersection
        // protects those too. Builder membership includes focused offscreen rows.
        let participating = |row| {
            rendered.contains(&row)
                || self.list.bounds_for_item(row).is_some_and(|bounds| {
                    let overlap = bounds.intersect(&self.list.viewport_bounds());
                    overlap.size.width > px(0.) && overlap.size.height > px(0.)
                })
        };
        let effect_row = physical_row(EFFECT_SAMPLE_ROW, self.color_columns);
        let toggle_row = physical_row(TOGGLE_SHAPE_ROWS, self.color_columns);
        let loading_row = physical_row(LOADING_SIZE_ROWS, self.color_columns);
        for (index, variant) in ButtonVariant::ALL.into_iter().enumerate() {
            if participating(effect_row) {
                continue;
            }
            #[cfg(test)]
            self.button_sync_rows.borrow_mut().insert(effect_row);
            self.button_effect_playbacks[index].sync(
                ButtonTransitionInputs {
                    variant: if variant == ButtonVariant::Filled && self.button_effects_tonal {
                        ButtonVariant::Tonal
                    } else {
                        variant
                    },
                    disabled: self.button_effects_disabled,
                    active: self.button_active,
                    ripple: self.button_ripple,
                    ..Default::default()
                },
                cx,
            );
        }
        for (size_index, size) in ButtonSize::ALL.into_iter().enumerate() {
            if participating(loading_row + size_index) {
                continue;
            }
            for icon_index in 0..3 {
                #[cfg(test)]
                self.button_sync_rows
                    .borrow_mut()
                    .insert(loading_row + size_index);
                self.button_loading_playbacks[size_index * 3 + icon_index].sync(
                    ButtonTransitionInputs {
                        variant: ButtonVariant::ALL[self.button_loading_variant],
                        size,
                        disabled: self.button_effects_disabled || self.button_loading,
                        ripple: self.button_ripple,
                        ..Default::default()
                    },
                    cx,
                );
            }
        }
        for (shape_index, shape) in ButtonShape::ALL.into_iter().enumerate() {
            if participating(toggle_row + shape_index) {
                continue;
            }
            for (variant_index, variant) in ButtonVariant::ALL.into_iter().enumerate() {
                let index = shape_index * 5 + variant_index;
                #[cfg(test)]
                self.button_sync_rows
                    .borrow_mut()
                    .insert(toggle_row + shape_index);
                self.button_toggle_playbacks[index].sync(
                    ButtonTransitionInputs {
                        variant,
                        shape,
                        size: self.button_toggle_size,
                        toggle: true,
                        selected: self.button_selections[index],
                        disabled: self.button_effects_disabled,
                        ripple: self.button_ripple,
                        ..Default::default()
                    },
                    cx,
                );
            }
        }
    }
}

pub(super) struct RowSpec {
    pub gap: f32,
    pub section: Option<&'static str>,
    pub interactive: bool,
    /// Names a row other code addresses, so its index follows the table.
    #[allow(
        dead_code,
        reason = "read only during const evaluation by anchored_row"
    )]
    pub anchor: Option<&'static str>,
}
pub(super) const ROWS: &[RowSpec] = &[
    RowSpec {
        gap: 0.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 28.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 28.0,
        section: Some("themes"),
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 28.0,
        section: Some("motion"),
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 28.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 28.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: Some("buttons"),
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: true,
        anchor: Some("button-effect-samples"),
    },
    RowSpec {
        gap: 16.0,
        section: Some("toggles"),
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: Some("button-toggle-shapes"),
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: Some("loading"),
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: false,
        anchor: Some("loading-click-count"),
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: Some("button-loading-sizes"),
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: Some("icons"),
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: Some("labels"),
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 16.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 28.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 28.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 28.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 28.0,
        section: Some("feedback"),
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: true,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 28.0,
        section: Some("colors"),
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: false,
        anchor: Some("color-swatches"),
    },
    RowSpec {
        gap: 28.0,
        section: Some("typography"),
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 28.0,
        section: Some("dividers"),
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 20.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 28.0,
        section: Some("chips"),
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 28.0,
        section: Some("lists"),
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 28.0,
        section: Some("window"),
        interactive: false,
        anchor: None,
    },
    RowSpec {
        gap: 12.0,
        section: None,
        interactive: true,
        anchor: None,
    },
];

const fn same(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// The source row carrying `anchor`; a missing anchor fails compilation.
const fn anchored_row(anchor: &str) -> usize {
    let mut row = 0;
    while row < ROWS.len() {
        if let Some(name) = ROWS[row].anchor
            && same(name, anchor)
        {
            return row;
        }
        row += 1;
    }
    panic!("Gallery row anchor is missing from ROWS")
}

/// The samples driven by `button_effect_playbacks`.
pub(super) const EFFECT_SAMPLE_ROW: usize = anchored_row("button-effect-samples");
/// One row per [`ButtonShape`], driven by `button_toggle_playbacks`.
pub(super) const TOGGLE_SHAPE_ROWS: usize = anchored_row("button-toggle-shapes");
const TOGGLE_SHAPE_LAST: usize = TOGGLE_SHAPE_ROWS + ButtonShape::ALL.len() - 1;
/// One row per [`ButtonSize`], driven by `button_loading_playbacks`.
pub(super) const LOADING_SIZE_ROWS: usize = anchored_row("button-loading-sizes");
const LOADING_SIZE_LAST: usize = LOADING_SIZE_ROWS + ButtonSize::ALL.len() - 1;
/// The one source row that expands to as many physical rows as color columns need.
const COLOR_SOURCE_ROW: usize = anchored_row("color-swatches");
/// The count of loading sample activations.
const CLICK_COUNT_ROW: usize = anchored_row("loading-click-count");

impl Gallery {
    pub(super) fn render_row_content(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let scroll = &self.list.clone();
        let color_start = index.saturating_sub(COLOR_SOURCE_ROW) * self.color_columns;
        let index = source_row(index, self.color_columns);
        // A listener on this row remeasures only the rows its change resizes.
        let row = index;
        match index {
            0 => {  (page_heading("title", "gpui-kit-material")).into_any_element() },
            1 => {  (supporting_info(
                        "intro",
                        "Explore Material themes, motion and components.",
                    )).into_any_element() },
            2 => {  (section_title("reference-theme", "Material themes")).into_any_element() },
            3 => {  (RevealFocus::in_list("reveal-row-reference_themes", ConnectedButtonGroup::new("reference-themes", "Material themes")

            .options(themes::ALL.map(|id| (id.name(), id.name())))
            .selected(self.reference_theme.map(ThemeId::name).unwrap_or(""))
            .full_width()
            .on_change(cx.listener(|this, name: &SharedString, window, cx| {
                this.reference_theme = themes::ALL
                    .into_iter()
                    .find(|id| id.name() == name.as_ref());
                this.prepare_theme(window, cx);
                cx.notify();
            })), self.list.clone())).into_any_element() },
            4 => {  (RevealFocus::in_list("reveal-row-mode", ConnectedButtonGroup::new("reference-mode", "Material theme mode")

            .options([("light", "Light"), ("dark", "Dark")])
            .selected(if self.reference_mode == ThemeMode::Dark {
                "dark"
            } else {
                "light"
            })
            .on_change(cx.listener(|this, name: &SharedString, window, cx| {
                this.reference_mode = if name == "dark" {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                };
                this.reference_theme.get_or_insert(ThemeId::Baseline);
                this.prepare_theme(window, cx);
                cx.notify();
            })), self.list.clone())).into_any_element() },
            5 => {  (RevealFocus::in_list("reveal-row-contrast", ConnectedButtonGroup::new("reference-contrast", "Material contrast")

            .options([
                ("standard", "Standard contrast"),
                ("medium", "Medium contrast"),
                ("high", "High contrast"),
            ])
            .selected(match self.contrast {
                Contrast::Standard => "standard",
                Contrast::Medium => "medium",
                Contrast::High => "high",
            })
            .on_change(cx.listener(|this, name: &SharedString, window, cx| {
                this.contrast = match name.as_ref() {
                    "high" => Contrast::High,
                    "medium" => Contrast::Medium,
                    _ => Contrast::Standard,
                };
                this.reference_theme.get_or_insert(ThemeId::Baseline);
                this.prepare_theme(window, cx);
                cx.notify();
            })), self.list.clone())).into_any_element() },
            6 => {  (section_title("motion", "Motion")).into_any_element() },
            7 => {  (RevealFocus::in_list("reveal-row-motion_controls", ConnectedButtonGroup::new("motion-scheme", "Motion scheme")

            .options([("expressive", "Expressive"), ("standard", "Standard")])
            .selected(match self.motion_scheme {
                motion::MotionScheme::Expressive => "expressive",
                motion::MotionScheme::Standard => "standard",
            })
            .on_change(cx.listener(|this, value: &SharedString, window, cx| {
                this.motion_scheme = if value == "standard" {
                    motion::MotionScheme::Standard
                } else {
                    motion::MotionScheme::Expressive
                };
                this.prepare_theme(window, cx);
                cx.notify();
            })), self.list.clone())).into_any_element() },
            8 => {  (material_button("play-motion", ButtonVariant::Tonal)
                                    .child("Toggle motion preview")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.motion_target = !this.motion_target;
                                        cx.notify();
                                    })).reveal("reveal-play-motion",scroll)).into_any_element() },
            9..=14 => { let sample_index = index - 9; (([
            ("Fast spatial", motion::MotionToken::FastSpatial, false),
            (
                "Default spatial",
                motion::MotionToken::DefaultSpatial,
                false,
            ),
            ("Slow spatial", motion::MotionToken::SlowSpatial, false),
            ("Fast effects", motion::MotionToken::FastEffects, true),
            ("Default effects", motion::MotionToken::DefaultEffects, true),
            ("Slow effects", motion::MotionToken::SlowEffects, true),
        ]
        .into_iter()
        .enumerate()
        .nth(sample_index).map(|(index, (label, _token, effects))| {
            let progress = self.motion_values[index];
            gpui_base::h_flex()
                .gap(rem(12.))
                .child(div().w(rem(160.)).child(label))
                .child(
                    div().w(rem(200.)).h(rem(24.)).child(
                        div()
                            .size(rem(24.))
                            .rounded_full()
                            .bg(color(ColorRole::ACCENT))
                            .when(effects, |view| view.opacity(progress))
                            .when(!effects, |view| view.ml(rem(progress * 160.))),
                    ),
                )
        })).expect("motion row")).into_any_element() },
            15 => {  (section_title("groups", "Connected button groups")).into_any_element() },
            16 => {  (RevealFocus::in_list("reveal-row-palettes", ConnectedButtonGroup::new("palette", "Palette")

            .options(PALETTES.map(|id| (id.name(), id.name())))
            .selected(if self.reference_theme.is_none() {
                self.palette.name()
            } else {
                ""
            })
            .full_width()
            .on_change(cx.listener(|this, name: &SharedString, window, cx| {
                if let Some(id) = PALETTES.into_iter().find(|id| id.name() == name.as_ref()) {
                    this.palette = id;
                    this.reference_theme = None;
                    this.prepare_theme(window, cx);
                    cx.notify();
                }
            })), self.list.clone())).into_any_element() },
            17 => {  (RevealFocus::in_list("reveal-row-scale", ConnectedButtonGroup::new("scale", "Text size")

            .options([
                ("1", "100%"),
                ("1.25", "125%"),
                ("1.5", "150%"),
                ("2", "200%"),
            ])
            .selected(self.scale.clone())
            .on_change(cx.listener(|this, value: &SharedString, window, cx| {
                this.scale = value.clone();
                this.prepare_theme(window, cx);
                cx.notify();
            })), self.list.clone())).into_any_element() },
            18 => {  (RevealFocus::in_list("reveal-row-density", ConnectedButtonGroup::new("density", "Density")

            .segmented()
            .options([
                ("auto", "Automatic"),
                ("dense", "Dense"),
                ("roomy", "Roomy"),
            ])
            .icon("auto", IconName::Info)
            .icon("dense", IconName::Check)
            .icon("roomy", IconName::Warning)
            .selected(self.density.clone())
            .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                this.density = value.clone();
                cx.notify();
            })), self.list.clone())).into_any_element() },
            19 => {  (section_title("controls", "Material controls")).into_any_element() },
            20 => {  (SearchField::new(&self.search).reveal("reveal-search",scroll)).into_any_element() },
            21 => {  (MaterialSwitch::new("example-switch", "Enable notifications")
                                    .checked(self.switch)
                                    .on_change(cx.listener(|this, checked: &bool, _, cx| {
                                        this.switch = *checked;
                                        cx.notify();
                                    })).reveal("reveal-switch",scroll)).into_any_element() },
            22 => {  (MaterialSlider::new(&self.slider, "Expressive slider").reveal("reveal-slider",scroll)).into_any_element() },
            23 => {  (MaterialSlider::new(&self.slider, "Classic slider")
                                    .appearance(SliderAppearance::Classic).reveal("reveal-slider-classic",scroll)).into_any_element() },
            24 => {  (section_title("buttons", "Buttons")).into_any_element() },
            25..=34 => { let sample_index = index - 25; ({let shape=ButtonShape::ALL[sample_index / 5]; (ButtonSize::ALL.into_iter().nth(sample_index % 5).map(move |size| {
                                        gpui_base::v_flex()
                                            .gap(rem(8.))
                                            .child(accessible_text(
                                                ("button-label", sample_index),
                                                SharedString::from(format!(
                                                    "{} / {}",
                                                    shape.name(),
                                                    size.name()
                                                )),
                                            ))
                                            .child(
                                                gpui_base::h_flex()
                                                    .flex_wrap()
                                                    .gap(rem(12.))
                                                    .children(ButtonVariant::ALL.into_iter().enumerate().map(
                                                        move |(variant_index, variant)| {
                                                            let sample = sample_index * 5 + variant_index;
                                                            MaterialButton::new(("button", sample))
                                                    .variant(variant)
                                                    .button_size(size)
                                                    .shape(shape)
                                                    .child(variant.name())
                                                    .reveal(("reveal-button", sample),scroll)
                                                        },
                                                    )),
                                            )
                                    })).expect("button size row")}).into_any_element() },
            35 => {  (section_title("button-effects", "Button transitions")).into_any_element() },
            36 => {  (gpui_base::h_flex()
                    .flex_wrap()
                    .gap(rem(12.))
                    .child(
                        material_button("toggle-button-active", ButtonVariant::Outlined)
                            .child(if self.button_active {
                                "Release active samples"
                            } else {
                                "Keep samples active"
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.button_active = !this.button_active;
                                this.remeasure_rows(row..=row); cx.notify();
                            }))
                            .reveal("reveal-toggle-button-active", scroll),
                    )
                    .child(
                        material_button("toggle-button-disabled", ButtonVariant::Outlined)
                            .child(if self.button_effects_disabled {
                                "Enable samples"
                            } else {
                                "Disable samples"
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.button_effects_disabled = !this.button_effects_disabled;
                                this.remeasure_rows(row..=row); cx.notify();
                            }))
                            .reveal("reveal-toggle-button-disabled", scroll),
                    )
                    .child(
                        material_button("toggle-button-variant", ButtonVariant::Outlined)
                            .child("Switch filled / tonal")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.button_effects_tonal = !this.button_effects_tonal;
                                cx.notify();
                            }))
                            .reveal("reveal-toggle-button-variant", scroll),
                    )
                    .child(
                        material_button("toggle-button-ripple", ButtonVariant::Outlined)
                            .child(if self.button_ripple {
                                "Turn ripple off"
                            } else {
                                "Turn ripple on"
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.button_ripple = !this.button_ripple;
                                this.remeasure_rows(row..=row); cx.notify();
                            }))
                            .reveal("reveal-toggle-button-ripple", scroll),
                    )).into_any_element() },
            EFFECT_SAMPLE_ROW => {  (gpui_base::h_flex().debug_selector(|| "button-effect-samples".into()).flex_wrap().gap(rem(12.)).children(
                ButtonVariant::ALL.into_iter().enumerate().map(|(sample_index, variant)| {
                    material_button(
                        ("effect-sample", sample_index),
                        if variant == ButtonVariant::Filled && self.button_effects_tonal {
                            ButtonVariant::Tonal
                        } else {
                            variant
                        },
                    )
                    .transition_playback(self.button_effect_playbacks[sample_index].clone())
                    .disabled(self.button_effects_disabled)
                    .active_state(self.button_active)
                    .ripple(self.button_ripple)
                    .icon(IconName::Check)
                    .child(variant.name())
                    .reveal(
                        ("reveal-effect-sample", sample_index),
                        scroll,
                    )
                }),
            )).into_any_element() },
            38 => {  (section_title("button-toggles", "Toggle buttons")).into_any_element() },
            39 => {  (RevealFocus::in_list("reveal-toggle-size", ConnectedButtonGroup::new("toggle-size", "Toggle button size")

                    .options(ButtonSize::ALL.map(|size| (size.name(), size.name())))
                    .selected(self.button_toggle_size.name())
                    .on_change(cx.listener(|this, name: &SharedString, _, cx| {
                        if let Some(size) = ButtonSize::ALL
                            .into_iter()
                            .find(|size| size.name() == name.as_ref())
                        {
                            this.button_toggle_size = size;
                            this.remeasure_rows(TOGGLE_SHAPE_ROWS..=TOGGLE_SHAPE_LAST); cx.notify();
                        }
                    })), self.list.clone())).into_any_element() },
            TOGGLE_SHAPE_ROWS..=TOGGLE_SHAPE_LAST => { let sample_index = index - TOGGLE_SHAPE_ROWS; ((ButtonShape::ALL
                    .into_iter()
                    .enumerate()
                    .nth(sample_index).map(|(shape_index, shape)| {
                        gpui_base::v_flex()
                            .when(sample_index == 0, |row| row.debug_selector(|| "button-toggle-shapes".into()))
                            .gap(rem(12.))
                            .child(accessible_text(
                                ("toggle-shape", shape_index),
                                shape.name(),
                            ))
                            .child(gpui_base::h_flex().flex_wrap().gap(rem(16.)).children(
                                ButtonVariant::ALL.into_iter().enumerate().map(
                                    |(variant_index, variant)| {
                                        let index = shape_index * 5 + variant_index;
                                        MaterialButton::new(("toggle", index))
                                        .variant(variant)
                                        .shape(shape)
                                        .button_size(self.button_toggle_size)
                                        .toggle(true)
                                        .selected(self.button_selections[index])
                                        .transition_playback(self.button_toggle_playbacks[index].clone())
                                        .disabled(self.button_effects_disabled)
                                        .ripple(self.button_ripple)
                                        .icon(IconName::Check)
                                        .child(variant.name())
                                        .on_change({
                                            let entity = cx.entity().downgrade();
                                            move |selected, _, _, cx| {
                                                let _ = entity.update(cx, |this, cx| {
                                                    this.button_selections[index] = selected;
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .reveal(
                                            ("reveal-toggle", index),
                                            scroll,
                                        )
                                    },
                                ),
                            ))
                    })).expect("gallery row")).into_any_element() },
            42 => {  (section_title("button-loading", "Button progress and loading")).into_any_element() },
            43 => { let loading_variant = ButtonVariant::ALL[self.button_loading_variant]; (gpui_base::h_flex().flex_wrap().gap(rem(12.))
                .child(material_button("toggle-loading", ButtonVariant::Outlined)
                    .child(if self.button_loading { "Stop loading" } else { "Start loading" })
                    .on_click(cx.listener(move |this, _, _, cx| { this.button_loading = !this.button_loading; this.remeasure_rows(row..=row); this.remeasure_rows(LOADING_SIZE_ROWS..=LOADING_SIZE_LAST); cx.notify(); }))
                    .reveal("reveal-toggle-loading", scroll))
                .child(material_button("toggle-progress", ButtonVariant::Outlined)
                    .child(if self.button_progress { "Hide progress" } else { "Show progress" })
                    .on_click(cx.listener(move |this, _, _, cx| { this.button_progress = !this.button_progress; this.remeasure_rows(row..=row); this.remeasure_rows(LOADING_SIZE_ROWS..=LOADING_SIZE_LAST); cx.notify(); }))
                    .reveal("reveal-toggle-progress", scroll))
                .child(material_button("loading-variant", ButtonVariant::Outlined)
                    .child(SharedString::from(format!("Variant: {}", loading_variant.name())))
                    .on_click(cx.listener(move |this, _, _, cx| { this.button_loading_variant = (this.button_loading_variant + 1) % ButtonVariant::ALL.len(); this.remeasure_rows(row..=row); this.remeasure_rows(LOADING_SIZE_ROWS..=LOADING_SIZE_LAST); cx.notify(); }))
                    .reveal("reveal-loading-variant", scroll))).into_any_element() },
            44 => {  (accessible_text("loading-help", "Progress stays interactive. Loading disables the samples and changes their labels.")).into_any_element() },
            CLICK_COUNT_ROW => {  (accessible_text("loading-click-count", SharedString::from(format!("Sample activations: {}", self.button_loading_clicks))).debug_selector(|| "loading-click-count".into())).into_any_element() },
            LOADING_SIZE_ROWS..=LOADING_SIZE_LAST => { let loading_variant = ButtonVariant::ALL[self.button_loading_variant];let sample_index = index - LOADING_SIZE_ROWS; ((ButtonSize::ALL.into_iter().nth(sample_index).map(|size| {
                gpui_base::v_flex().when(sample_index == 0, |row| row.debug_selector(|| "button-loading-sizes".into())).gap(rem(12.))
                    .child(accessible_text(("loading-size", sample_index), size.name()))
                    .child(gpui_base::h_flex().flex_wrap().gap(rem(16.)).children((0..3).map(|icon_position| {
                        MaterialButton::new(("loading", sample_index * 3 + icon_position))
                            .variant(loading_variant).button_size(size)
                            .when(icon_position > 0, |button| button.icon(IconName::Check))
                            .when(icon_position == 2, |button| button.icon_position(ButtonIconPosition::Trailing))
                            .disabled(self.button_effects_disabled).ripple(self.button_ripple)
                            .progress(self.button_progress, "Operation progress")
                            .loading(self.button_loading, "Operation loading")
                            .transition_playback(self.button_loading_playbacks[sample_index * 3 + icon_position].clone())
                            .progress_playback(self.button_progress_playbacks[sample_index * 3 + icon_position].clone())
                            .child(if self.button_loading { "Saving…" } else { "Save changes" })
                            .on_click(cx.listener(|this, _, _, cx| this.count_loading_activation(cx)))
                            .reveal(("reveal-loading", sample_index * 3 + icon_position), scroll)
                    })))
            })).expect("gallery row")).into_any_element() },
            51 => {  (section_title("button-icons", "Button icons")).into_any_element() },
            52..=56 => { let sample_index = index - 52; ((ButtonSize::ALL.into_iter().nth(sample_index).map(|size| {
                gpui_base::h_flex().flex_wrap().gap(rem(12.)).children(
                    [
                        (
                            "Leading",
                            ButtonIconPosition::Leading,
                            ButtonDirection::Ltr,
                            ButtonVariant::Filled,
                        ),
                        (
                            "Trailing",
                            ButtonIconPosition::Trailing,
                            ButtonDirection::Ltr,
                            ButtonVariant::Filled,
                        ),
                        (
                            "RTL leading",
                            ButtonIconPosition::Leading,
                            ButtonDirection::Rtl,
                            ButtonVariant::Text,
                        ),
                        (
                            "RTL trailing",
                            ButtonIconPosition::Trailing,
                            ButtonDirection::Rtl,
                            ButtonVariant::Text,
                        ),
                    ]
                    .into_iter()
                    .enumerate()
                    .map(move |(position_index, (label, position, direction, variant))| {
                        let sample = sample_index * 4 + position_index;
                        material_button(
                            ("icon", sample),
                            variant,
                        )
                        .button_size(size)
                        .icon(IconName::Check)
                        .icon_position(position)
                        .direction(direction)
                        .child(label)
                        .reveal(
                            ("reveal-icon", sample),
                            scroll,
                        )
                    }),
                )
            })).expect("gallery row")).into_any_element() },
            57 => {  (section_title("text-labels", "Text labels")).into_any_element() },
            58..=60 => { let sample_index = index - 58; (([
                    TypeRole::LabelLarge,
                    TypeRole::LabelMedium,
                    TypeRole::LabelSmall,
                ]
                .into_iter()
                .nth(sample_index).map(|role| {
                    let spec = role.scale();
                    accessible_text(
                        ("label-role", sample_index),
                        role.name(),
                    )
                    .font_family(spec.font_family)
                    .text_size(spec.font_size)
                    .line_height(spec.line_height)
                    .font_weight(FontWeight(spec.font_weight as f32))
                })).expect("gallery row")).into_any_element() },
            61 => {  (gpui_base::h_flex().flex_wrap().gap(rem(12.)).children(
                                ButtonVariant::ALL.into_iter().enumerate().map(|(variant_index, variant)| {
                                    material_button(
                                        ("disabled", variant_index),
                                        variant,
                                    )
                                    .disabled(true)
                                    .icon(IconName::Check)
                                    .child(variant.name())
                                }),
                            )).into_any_element() },
            62 => {  (div()
                            .relative()
                            .h(rem(220.))
                            .p(px(8.))
                            .overflow_hidden()
                            .child(
                                material_button("open-sheet", ButtonVariant::Tonal)
                                    .child("Toggle side sheet")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.sheet = !this.sheet;
                                        cx.notify();
                                    })).reveal("reveal-open-sheet",scroll),
                            )
                            .child(
                                MaterialSideSheet::new("gallery-sheet")
                                    .open(self.sheet)
                                    .width(rem(280.))
                                    .child(
                                        div().p(rem(20.)).child("Standard side sheet").child(
                                            material_button("close-sheet", ButtonVariant::Text)
                                                .child("Close")
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.sheet = false;
                                                    cx.notify();
                                                })).reveal("reveal-close-sheet",scroll),
                                        ),
                                    ),
                            )).into_any_element() },
            63 => {  ({        let engines = ["Local engine", "Cloud service", "Unavailable"];
        let mut choices = TileGroup::new("engines", "Choice tiles");
        for (index, title) in engines.into_iter().enumerate() {
            let tile = ChoiceTile::new(
                ("engine", index),
                index,
                engines.len(),
                title,
                self.engine == index,
            )
            .tinted_icon(
                IconName::Info.icon(),
                [Tint::Blue, Tint::Teal, Tint::Orange][index],
                px(40.),
            )
            .description(match index {
                0 => "Runs on this computer",
                1 => "Sends audio to the configured service",
                _ => "Shown disabled, with its status at full strength",
            })
            .status((index == 2).then(|| badge(BadgeKind::Warning).child("Missing")))
            .enabled(index != 2)
            .render(window, cx)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.engine = index;
                cx.notify();
            }));
            choices = choices.tile(move |_| tile.into_any_element());
        }

choices.reveal("reveal-choices", scroll)}).into_any_element() },
            64 => {  ({ let details = self.details; let toggle = cx.listener(move |this, _, _, cx| { this.details = !this.details; this.remeasure_rows(row..=row); cx.notify(); }); let settings = TileGroup::new("tiles", "Connected tiles")
            .tile(move |outer| {
                TileScale::Regular
                    .button("details", outer)
                    .on_click(toggle)
                    .child(
                        TileScale::Regular
                            .row()
                            .child(TileScale::Regular.text(
                                "details-text",
                                "Show details",
                                Some("A tile that discloses content in place".into()),
                            ))
                            .child(disclosure_chevron(details)),
                    )
                    .into_any_element()
            })
            .detail(motion::disclosure(
                "details-content",
                details,
                div()
                    .px(ITEM_INSET)
                    .pb(rem(16.))
                    .child(tile_info("details-info", "Details stay inside their tile.")),
            ))
            .child(
                div()
                    .px(ITEM_INSET)
                    .py(rem(16.))
                    .child(TileScale::Regular.text(
                        "plain",
                        "A content tile",
                        Some("Its corners follow its place in the group".into()),
                    )),
            ); settings.reveal("reveal-settings", scroll) }).into_any_element() },
            65 => {  (section_title("feedback", "Feedback")).into_any_element() },
            66 => {  (gpui_base::h_flex()
                    .flex_wrap()
                    .gap(rem(8.))
                    .child(badge(BadgeKind::Success).child("Ready"))
                    .child(badge(BadgeKind::Warning).child("Needs attention"))
                    .child(badge(BadgeKind::Danger).child("Failed"))
                    .child(badge(BadgeKind::Progress).child("Working"))
                    .child(badge(BadgeKind::Neutral).child("Queued"))
                    .child(badge(BadgeKind::Tonal).child("Default"))
                    .child(outlined_badge("outlined", "Outlined"))).into_any_element() },
            67 => {  (notice_banner(
                "notice-info",
                NoticeKind::Info,
                "An informational notice wraps its text when narrow.",
            )).into_any_element() },
            68 => {  (notice_banner(
                "notice-danger",
                NoticeKind::Danger,
                "A failure stays visible with its recovery action.",
            )).into_any_element() },
            69 => {  (info_callout(
                "callout",
                "Standalone guidance sits in its own low-emphasis panel.",
            )).into_any_element() },
            70 => {  (error_text("error", "Error text is announced as an alert.")).into_any_element() },
            71 => {  (material_button("show-snackbar", ButtonVariant::Tonal)
                    .child("Show snackbar")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.snackbar = true;
                        cx.notify();
                    }))
                    .reveal("reveal-show-snackbar", scroll)).into_any_element() },
            72 => {  (card_surface()
                    .gap(rem(12.))
                    .child(semantic_label(
                        "progress-label",
                        "Linear progress",
                        icon_spinner_with_playback("label-spinner", ICON_SLOT, self.feedback_spinner_playback.clone(), cx),
                    ))
                    .child(progress("measured", 0.62, window, cx))
                    .child(indeterminate_progress_with_playback("indeterminate", self.feedback_linear_playback.clone(), cx))
                    .child(accessible_text("circular-label", "Circular progress"))
                    .child(
                        gpui_base::h_flex()
                            .gap(rem(16.))
                            .child(indeterminate_circular_progress_with_playback(
                                "circular-animated",
                                "Loading",
                                size(rem(24.), rem(24.)),
                                1.,
                                &self.feedback_circular_playback,
                                cx,
                            ))
                            .children([0., 0.62, 1.].into_iter().enumerate().map(
                                |(index, fraction)| {
                                    circular_progress(
                                        ("circular-measured", index),
                                        "Progress",
                                        CircularProgressFrame::determinate(fraction),
                                        Some(fraction),
                                        size(rem(24.), rem(24.)),
                                        1.,
                                    )
                                },
                            )),
                    )).into_any_element() },
            73 => {  (section_title("reference-colors", "Material color roles")).into_any_element() },
            COLOR_SOURCE_ROW => {  (gpui_base::h_flex().when(color_start == 0, |row| row.debug_selector(|| "color-swatches".into())).flex_wrap().gap(rem(12.)).children(themes::REFERENCE_ROLES
            .into_iter()
            .zip(themes::REFERENCE_ROLE_NAMES).enumerate().skip(color_start).take(self.color_columns)
            .map(|(role_index, (role, name))| {
                gpui_base::v_flex()
                    .w(rem(166.))
                    .gap(rem(4.))
                    .child(div().h(rem(32.)).w_full().rounded(rem(8.)).bg(color(role)))
                    .child(
                        accessible_text(("color", role_index), name)
                            .text_size(TEXT_AUX),
                    )
            }))).into_any_element() },
            75 => {  (section_title("type-scale", "Typography")).into_any_element() },
            76..=90 => { let sample_index = index - 76; ((TypeRole::ALL.into_iter().nth(sample_index).map(|role| {
            let spec = role.scale();
            gpui_base::v_flex()
                .gap(rem(4.))
                .child(
                    accessible_text(
                        ("type-label", sample_index),
                        role.name(),
                    )
                    .text_size(TEXT_AUX)
                    .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                )
                .child(
                    accessible_text(
                        ("type-sample", sample_index),
                        "The quick brown fox jumps over the lazy dog",
                    )
                    .whitespace_normal()
                    .font_family(spec.font_family)
                    .font_weight(FontWeight(spec.font_weight as f32))
                    .text_size(spec.font_size)
                    .line_height(spec.line_height),
                )
        })).expect("type row")).into_any_element() },
            91 => {  (section_title("dividers", "Dividers")).into_any_element() },
            92 => {  ("Horizontal: full width, start inset, both ends inset, RTL start inset").into_any_element() },
            93..=96 => { let sample_index = index - 93; (([
                    (DividerVariant::FullWidth, DividerDirection::Ltr),
                    (DividerVariant::Inset, DividerDirection::Ltr),
                    (DividerVariant::MiddleInset, DividerDirection::Ltr),
                    (DividerVariant::Inset, DividerDirection::Rtl),
                ]
                .into_iter()
                .enumerate()
                .nth(sample_index).map(|(index, (variant, direction))| {
                    div()
                        .w_full()
                        .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
                        .child(
                            MaterialDivider::new(("horizontal-divider", index))
                                .variant(variant)
                                .direction(direction),
                        )
                })).expect("gallery row")).into_any_element() },
            97 => {  ("Vertical in a stretching row: full height, start inset, both ends inset").into_any_element() },
            98 => {  (div()
                    .flex()
                    .items_stretch()
                    .h(rem(72.))
                    .gap(rem(32.))
                    .children(
                        [
                            DividerVariant::FullWidth,
                            DividerVariant::Inset,
                            DividerVariant::MiddleInset,
                        ]
                        .into_iter()
                        .enumerate()
                        .map(|(index, variant)| {
                            MaterialDivider::new(("vertical-divider", index))
                                .orientation(DividerOrientation::Vertical)
                                .variant(variant)
                        }),
                    )).into_any_element() },
            99 => {  ("Custom: primary color, 2px thickness, 24px / 8px insets").into_any_element() },
            100 => {  (MaterialDivider::new("custom-divider")
                    .variant(DividerVariant::MiddleInset)
                    .inset_start(px(24.))
                    .inset_end(px(8.))
                    .thickness(px(2.))
                    .color_role(ColorRole::PRIMARY)).into_any_element() },
            101 => section_title("chips", "Chips").into_any_element(),
            102 => {
                let label = TypeRole::LabelLarge.scale();
                let assist = |leading: bool| {
                    chip(leading)
                        .border_1()
                        .border_color(color(ColorRole::OUTLINE))
                        .text_color(color(ColorRole::ON_SURFACE))
                        .font_family(label.font_family)
                        .text_size(label.font_size)
                        .line_height(label.line_height)
                        .font_weight(FontWeight(label.font_weight as f32))
                };
                gpui_base::h_flex()
                    .flex_wrap()
                    .gap(rem(8.))
                    .child(assist(false).child("Assist chip"))
                    .child(
                        assist(true)
                            .child(
                                IconName::Info
                                    .icon()
                                    .size(rem(18.))
                                    .color(color(ColorRole::PRIMARY)),
                            )
                            .child("With leading icon"),
                    )
                    .into_any_element()
            }
            103 => section_heading("lists", IconName::Info, "Lists").into_any_element(),
            104 => {
                let items = [
                    ("Display", "Theme, contrast and text size"),
                    ("Notifications", "Alerts and sounds"),
                    ("About", "Version and licenses"),
                ];
                gpui_base::v_flex()
                    .gap(TILE_GAP)
                    .children(items.into_iter().enumerate().map(|(index, (title, support))| {
                        list_tile(index, items.len())
                            .px(ITEM_INSET)
                            .py(rem(12.))
                            .child(
                                TileScale::Regular
                                    .row()
                                    .child(TileScale::Regular.text(
                                        ("list-tile", index),
                                        title,
                                        Some(support.into()),
                                    ))
                                    .child(tile_chevron()),
                            )
                    }))
                    .into_any_element()
            }
            105 => section_title("window", "Window controls").into_any_element(),
            106 => {
                let caption = match Caption::detect(window) {
                    Caption::System => "Caption buttons: drawn by the system",
                    Caption::Windows => "Caption buttons: Windows style, drawn by the app",
                    Caption::Linux => "Caption buttons: Linux style, drawn by the app",
                };
                gpui_base::v_flex()
                    .gap(rem(12.))
                    .child(accessible_text("caption-style", caption))
                    .child(
                        // A sample bar: its control takes the press itself
                        // instead of starting a window drag.
                        gpui_base::h_flex()
                            .h(rem(56.))
                            .px(rem(16.))
                            .gap(rem(12.))
                            .items_center()
                            .rounded(RADIUS_SMALL)
                            .bg(color(ColorRole::SURFACE_CONTAINER_HIGH))
                            .child(accessible_text("sample-title", "Sample title bar"))
                            .child(div().flex_1())
                            .child(
                                title_bar_control(
                                    material_button("sample-bar-settings", ButtonVariant::Text)
                                        .button_size(ButtonSize::ExtraSmall)
                                        .child("Settings")
                                        .on_click(|_, window, cx| super::settings::open(window, cx)),
                                )
                                .reveal("reveal-sample-bar-settings", scroll),
                            ),
                    )
                    .into_any_element()
            }
            _ => unreachable!("Gallery row index comes from the static directory"),
        }
    }
}

impl Gallery {
    /// Invalidates the cached heights of source `rows` only. Rows the list
    /// lays out this frame are measured again anyway; this keeps offscreen
    /// rows whose content changed from scrolling at a stale height.
    pub(super) fn remeasure_rows(&self, rows: std::ops::RangeInclusive<usize>) {
        let start = physical_row(*rows.start(), self.color_columns);
        let end = physical_row(*rows.end(), self.color_columns);
        self.list.remeasure_items(start..end + 1);
    }

    fn count_loading_activation(&mut self, cx: &mut Context<Self>) {
        self.button_loading_clicks += 1;
        self.remeasure_rows(CLICK_COUNT_ROW..=CLICK_COUNT_ROW);
        cx.notify();
    }

    pub(super) fn scroll_to_section(&self, section: usize) {
        self.list.scroll_to(ListOffset {
            item_ix: self.section_rows[section],
            offset_in_item: px(
                ROWS[source_row(self.section_rows[section], self.color_columns)].gap
                    * f32::from(self.last_rem)
                    / gpui_kit_material::tokens::typography::BASE_PX,
            ),
        });
    }

    pub(super) fn render_row(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let child = self.render_row_content(index, window, cx);
        let source = source_row(index, self.color_columns);
        let row = div()
            .id(("gallery-row", index))
            .when(index == 0, |row| {
                row.debug_selector(|| "gallery-first-row".into())
            })
            .track_focus(&self.row_focus[index])
            .tab_stop(false)
            .w_full()
            .max_w(rem(760.))
            .px(px(if is_compact(window) { 16. } else { 24. }))
            .pt(rem(if index == 0 { 32. } else { ROWS[source].gap }))
            .when(index + 1 == row_count(self.color_columns), |row| {
                row.pb(rem(32.))
            })
            .child(child);
        // List item roots are positioned directly at the list's left edge.
        // A real flex parent supplies the same centering as the original page.
        div()
            .w_full()
            .flex()
            .justify_center()
            .child(row)
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
        let target = &self.row_focus[row];
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
        if let Some(next) = adjacent_interactive_row(source_row(row, self.color_columns), reverse) {
            let next = physical_row(next, self.color_columns);
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
            Some(row) => adjacent_interactive_row(source_row(row, self.color_columns), reverse),
            None if reverse => ROWS.iter().rposition(|row| row.interactive),
            None => ROWS.iter().position(|row| row.interactive),
        };
        if let Some(target) = target {
            let target = physical_row(target, self.color_columns);
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

fn adjacent_interactive_row(row: usize, reverse: bool) -> Option<usize> {
    if reverse {
        (0..row).rev().find(|index| ROWS[*index].interactive)
    } else {
        (row + 1..ROWS.len()).find(|index| ROWS[*index].interactive)
    }
}

#[cfg(test)]
mod tests {
    use super::{ROWS, adjacent_interactive_row};
    #[test]
    fn section_directory_is_complete_unique_and_ordered() {
        let actual: Vec<_> = ROWS.iter().filter_map(|row| row.section).collect();
        let expected: Vec<_> = super::super::GALLERY_SECTIONS
            .iter()
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(actual, expected);
        assert!(ROWS.len() > 90, "large sections must remain fine-grained");
    }
    #[test]
    fn logical_keyboard_order_crosses_noninteractive_and_unmounted_rows() {
        let interactive: Vec<_> = ROWS
            .iter()
            .enumerate()
            .filter_map(|(ix, row)| row.interactive.then_some(ix))
            .collect();
        for pair in interactive.windows(2) {
            assert_eq!(adjacent_interactive_row(pair[0], false), Some(pair[1]));
            assert_eq!(adjacent_interactive_row(pair[1], true), Some(pair[0]));
        }
        assert_eq!(adjacent_interactive_row(interactive[0], true), None);
        assert_eq!(
            adjacent_interactive_row(*interactive.last().unwrap(), false),
            None
        );
    }
}

fn color_row_count(columns: usize) -> usize {
    themes::REFERENCE_ROLES.len().div_ceil(columns)
}
pub(super) fn row_count(columns: usize) -> usize {
    ROWS.len() - 1 + color_row_count(columns)
}
pub(super) fn source_row(index: usize, columns: usize) -> usize {
    if index < COLOR_SOURCE_ROW {
        index
    } else if index < COLOR_SOURCE_ROW + color_row_count(columns) {
        COLOR_SOURCE_ROW
    } else {
        index - color_row_count(columns) + 1
    }
}
pub(super) fn physical_row(source: usize, columns: usize) -> usize {
    if source <= COLOR_SOURCE_ROW {
        source
    } else {
        source + color_row_count(columns) - 1
    }
}
pub(super) fn color_columns(window: &Window) -> usize {
    let unit = f32::from(window.rem_size()) / gpui_kit_material::tokens::typography::BASE_PX;
    let compact = is_compact(window);
    let width = f32::from(window.viewport_size().width) - if compact { 0. } else { 176. * unit };
    let available = width.min(760. * unit) - if compact { 32. } else { 48. };
    ((available + 12. * unit) / (178. * unit)).floor().max(1.) as usize
}

#[cfg(test)]
mod keyboard_tests {
    use super::super::Gallery;
    use super::{
        CLICK_COUNT_ROW, COLOR_SOURCE_ROW, EFFECT_SAMPLE_ROW, LOADING_SIZE_ROWS, TOGGLE_SHAPE_ROWS,
    };
    use gpui::{AppContext, TestAppContext, WindowHandle, px, size};

    fn draw(window: &WindowHandle<Gallery>, cx: &mut TestAppContext) {
        cx.update_window((*window).into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    }
    fn frame(window: &WindowHandle<Gallery>, cx: &mut TestAppContext) {
        cx.update_window((*window).into(), |_, window, cx| {
            window.simulate_next_frame(cx);
        })
        .unwrap();
        cx.run_until_parked();
        draw(window, cx);
    }

    #[gpui::test]
    fn button_sync_runs_after_list_and_does_not_treat_remeasurement_as_unmount(
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let window = cx.open_window(size(px(1100.), px(500.)), Gallery::new);
        window
            .update(cx, |gallery, _, _| {
                gallery.button_sync_rows.borrow_mut().insert(999);
            })
            .unwrap();
        draw(&window, cx);
        window
            .update(cx, |gallery, _, cx| {
                assert!(
                    !gallery.button_sync_rows.borrow().contains(&999),
                    "zero-size canvas prepaint must run"
                );
                assert!(
                    gallery
                        .button_sync_rows
                        .borrow()
                        .contains(&EFFECT_SAMPLE_ROW)
                );
                assert!(
                    gallery
                        .button_sync_rows
                        .borrow()
                        .contains(&LOADING_SIZE_ROWS)
                );
                gallery.list.scroll_to(gpui::ListOffset {
                    item_ix: EFFECT_SAMPLE_ROW,
                    offset_in_item: px(0.),
                });
                cx.notify();
            })
            .unwrap();
        draw(&window, cx);
        window
            .update(cx, |gallery, _, cx| {
                assert!(
                    !gallery
                        .button_sync_rows
                        .borrow()
                        .contains(&EFFECT_SAMPLE_ROW)
                );
                gallery.button_active = true;
                gallery.list.remeasure();
                cx.notify();
            })
            .unwrap();
        draw(&window, cx);
        window
            .update(cx, |gallery, _, _| {
                assert!(
                    !gallery
                        .button_sync_rows
                        .borrow()
                        .contains(&EFFECT_SAMPLE_ROW),
                    "remeasure must not inject rest targets into the visible row"
                );
            })
            .unwrap();
    }

    #[gpui::test]
    fn anchored_rows_render_the_content_their_playbacks_drive(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let window = cx.open_window(size(px(960.), px(600.)), Gallery::new);
        draw(&window, cx);
        for (row, selector) in [
            (EFFECT_SAMPLE_ROW, "button-effect-samples"),
            (TOGGLE_SHAPE_ROWS, "button-toggle-shapes"),
            (LOADING_SIZE_ROWS, "button-loading-sizes"),
            (COLOR_SOURCE_ROW, "color-swatches"),
            (CLICK_COUNT_ROW, "loading-click-count"),
        ] {
            let physical = window
                .update(cx, |gallery, _, cx| {
                    let physical = super::physical_row(row, gallery.color_columns);
                    gallery.list.scroll_to(gpui::ListOffset {
                        item_ix: physical,
                        offset_in_item: px(0.),
                    });
                    cx.notify();
                    physical
                })
                .unwrap();
            draw(&window, cx);
            let row_bounds = window
                .update(cx, |gallery, _, _| gallery.list.bounds_for_item(physical))
                .unwrap()
                .expect("anchored row is mounted");
            let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
            let content = visual
                .debug_bounds(selector)
                .unwrap_or_else(|| panic!("{selector} is not rendered"));
            assert!(
                row_bounds.contains(&content.origin),
                "{selector} must render in row {physical}: {content:?} outside {row_bounds:?}"
            );
        }
    }

    #[gpui::test]
    fn chip_list_and_window_control_sections_render_their_samples(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let window = cx.open_window(size(px(700.), px(400.)), Gallery::new);
        draw(&window, cx);
        for section in ["chips", "lists", "window"] {
            let index = super::super::GALLERY_SECTIONS
                .iter()
                .position(|(id, _)| *id == section)
                .unwrap();
            window
                .update(cx, |gallery, _, cx| {
                    gallery.scroll_to_section(index);
                    cx.notify();
                })
                .unwrap();
            draw(&window, cx);
            window
                .update(cx, |gallery, _, _| {
                    let heading = gallery.section_rows[index];
                    for row in [heading, heading + 1] {
                        let bounds = gallery.list.bounds_for_item(row);
                        assert!(
                            bounds.is_some_and(|bounds| bounds.size.height > px(0.)),
                            "{section}: row {row} must render"
                        );
                    }
                })
                .unwrap();
        }
    }

    #[gpui::test]
    fn keyboard_mounts_offscreen_row_and_returns_without_extra_tab_stop(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let window = cx.open_window(size(px(960.), px(350.)), Gallery::new);
        cx.run_until_parked();
        // A roving group contributes exactly one tab stop. The next group
        // must mount even after the page is manually scrolled far away.
        window
            .update(cx, |gallery, _, cx| {
                gallery.list.scroll_to(gpui::ListOffset {
                    item_ix: 3,
                    offset_in_item: px(0.),
                });
                cx.notify();
            })
            .unwrap();
        draw(&window, cx);
        window
            .update(cx, |gallery, window, cx| {
                gallery.focus_row(3, false, window, cx);
                assert!(gallery.row_focus[3].contains_focused(window, cx));
                gallery.scroll_to_section(10);
                cx.notify();
            })
            .unwrap();
        draw(&window, cx);
        window
            .update(cx, |gallery, window, cx| {
                assert!(
                    gallery.list.bounds_for_item(4).is_none(),
                    "next logical row must be unmounted"
                );
                gallery.advance_gallery_focus(false, window, cx);
                assert_eq!(
                    gallery.pending_focus,
                    Some((4, false)),
                    "one Tab must leave the roving group and mount the next row"
                );
            })
            .unwrap();
        draw(&window, cx);
        frame(&window, cx);
        window
            .update(cx, |gallery, window, cx| {
                assert!(
                    gallery.row_focus[4].contains_focused(window, cx),
                    "focused={:?} rows={:?} top={:?} pending={:?}",
                    window.focused(cx),
                    gallery
                        .row_focus
                        .iter()
                        .enumerate()
                        .filter_map(|(i, h)| h.contains_focused(window, cx).then_some(i))
                        .collect::<Vec<_>>(),
                    gallery.list.logical_scroll_top(),
                    gallery.pending_focus
                );
                assert!(
                    !gallery.row_focus[4].is_focused(window),
                    "no container-only tab stop"
                );
                gallery.advance_gallery_focus(true, window, cx);
            })
            .unwrap();
        draw(&window, cx);
        frame(&window, cx);
        window
            .update(cx, |gallery, window, cx| {
                assert!(gallery.row_focus[3].contains_focused(window, cx));
            })
            .unwrap();
    }

    #[gpui::test]
    fn virtual_navigation_preserves_business_and_input_entities(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let window = cx.open_window(size(px(700.), px(250.)), Gallery::new);
        cx.run_until_parked();
        let original = window
            .update(cx, |gallery, _, _| {
                gallery.button_selections[0] = true;
                gallery.details = true;
                gallery.button_loading_clicks = 9;
                (gallery.search.clone(), gallery.slider.clone())
            })
            .unwrap();
        for section in [10, 0, 4, 8, 3] {
            window
                .update(cx, |gallery, _, cx| {
                    gallery.scroll_to_section(section);
                    cx.notify();
                })
                .unwrap();
            draw(&window, cx);
        }
        window
            .update(cx, |gallery, _, _| {
                assert_eq!(gallery.search.entity_id(), original.0.entity_id());
                assert_eq!(gallery.slider.entity_id(), original.1.entity_id());
                assert!(gallery.button_selections[0]);
                assert!(gallery.details);
                assert_eq!(gallery.button_loading_clicks, 9);
            })
            .unwrap();
    }
    #[gpui::test]
    fn virtual_rows_center_at_the_page_maximum_width(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        for width in [700., 960., 1400.] {
            let window = cx.open_window(size(px(width), px(600.)), Gallery::new);
            window
                .update(cx, |_, window, _| window.set_rem_size(px(14.)))
                .unwrap();
            draw(&window, cx);
            let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
            let bounds = visual.debug_bounds("gallery-first-row").unwrap();
            let sidebar = if width < 960. { 0. } else { 176. };
            let content_width = width - sidebar;
            let row_width = content_width.min(760.);
            assert!((f32::from(bounds.size.width) - row_width).abs() < 0.01);
            assert!(
                (f32::from(bounds.origin.x) - sidebar - (content_width - row_width) / 2.).abs()
                    < 0.01,
                "width {width}: {bounds:?}"
            );
            window
                .update(cx, |gallery, _, _| {
                    let padding = if width < 960. { 32. } else { 48. };
                    let expected = ((row_width - padding + 12.) / 178.).floor() as usize;
                    assert_eq!(gallery.color_columns, expected.max(1));
                })
                .unwrap();
        }
    }

    #[gpui::test]
    fn circular_feedback_keeps_phase_and_observes_reduced_motion_offscreen(
        cx: &mut TestAppContext,
    ) {
        use gpui_kit_material::components::circular_indeterminate_frame;
        use std::time::Duration;
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let window = cx.open_window(size(px(700.), px(250.)), Gallery::new);
        draw(&window, cx);
        // An unused playback must start on the actual feedback row's first render.
        cx.executor().advance_clock(Duration::from_secs(2));
        window
            .update(cx, |gallery, _, cx| {
                gallery.list.scroll_to(gpui::ListOffset {
                    item_ix: 72,
                    offset_in_item: px(0.),
                });
                cx.notify();
            })
            .unwrap();
        draw(&window, cx);
        cx.executor().advance_clock(Duration::from_millis(150));
        window
            .update(cx, |gallery, _, cx| {
                assert!(gallery.list.bounds_for_item(72).is_some());
                assert_eq!(
                    gallery.feedback_circular_playback.sample(cx),
                    circular_indeterminate_frame(150.),
                    "the mounted row must use the Gallery-owned playback"
                );
                gallery.scroll_to_section(10);
                cx.notify();
            })
            .unwrap();
        draw(&window, cx);
        cx.executor().advance_clock(Duration::from_millis(350));
        window
            .update(cx, |gallery, _, cx| {
                assert!(gallery.list.bounds_for_item(72).is_none());
                cx.set_reduce_motion(true);
            })
            .unwrap();
        draw(&window, cx);
        window
            .update(cx, |_, _, cx| {
                // Do not sample while reduced: that could hide a missing
                // offscreen observation in Gallery::render.
                cx.set_reduce_motion(false);
            })
            .unwrap();
        draw(&window, cx);
        cx.executor().advance_clock(Duration::from_secs(1));
        window
            .update(cx, |gallery, _, cx| {
                gallery.list.scroll_to(gpui::ListOffset {
                    item_ix: 72,
                    offset_in_item: px(0.),
                });
                cx.notify();
            })
            .unwrap();
        draw(&window, cx);
        window
            .update(cx, |gallery, _, cx| {
                assert!(gallery.list.bounds_for_item(72).is_some());
                assert_eq!(
                    gallery.feedback_circular_playback.sample(cx),
                    circular_indeterminate_frame(500.),
                    "remounting must preserve the reduced-motion stop"
                );
            })
            .unwrap();
    }

    #[gpui::test]
    fn motion_previews_keep_phase_across_virtual_unmount_and_offscreen_retarget(
        cx: &mut TestAppContext,
    ) {
        use gpui_kit_material::motion::{MotionScheme, MotionToken};
        use std::time::Duration;
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let window = cx.open_window(size(px(700.), px(250.)), Gallery::new);
        draw(&window, cx);
        window
            .update(cx, |gallery, _, cx| {
                gallery.scroll_to_section(10);
                cx.notify();
            })
            .unwrap();
        draw(&window, cx);
        window
            .update(cx, |gallery, _, cx| {
                for row in 9..=14 {
                    assert!(gallery.list.bounds_for_item(row).is_none());
                }
                assert_eq!(gallery.motion_values, [0.; 6]);
                gallery.motion_target = true;
                cx.notify();
            })
            .unwrap();
        draw(&window, cx);
        cx.executor().advance_clock(Duration::from_millis(100));
        draw(&window, cx);
        let ongoing = window
            .update(cx, |gallery, _, _| {
                let tokens = [
                    MotionToken::FastSpatial,
                    MotionToken::DefaultSpatial,
                    MotionToken::SlowSpatial,
                    MotionToken::FastEffects,
                    MotionToken::DefaultEffects,
                    MotionToken::SlowEffects,
                ];
                for (index, token) in tokens.into_iter().enumerate() {
                    let curve = MotionScheme::Expressive.resolve(token);
                    let expected = curve.ease()(0.1 / curve.duration().as_secs_f32());
                    assert!(
                        (gallery.motion_values[index] - expected).abs() < 0.002,
                        "channel {index}: {} != {expected}",
                        gallery.motion_values[index]
                    );
                }
                gallery.motion_values
            })
            .unwrap();
        window
            .update(cx, |gallery, _, cx| {
                gallery.list.scroll_to(gpui::ListOffset {
                    item_ix: 9,
                    offset_in_item: px(0.),
                });
                cx.notify();
            })
            .unwrap();
        draw(&window, cx);
        window
            .update(cx, |gallery, _, cx| {
                assert!(
                    gallery.list.bounds_for_item(9).is_some(),
                    "preview must really remount"
                );
                assert_eq!(
                    gallery.motion_values, ongoing,
                    "remount must not restart or snap"
                );
                gallery.scroll_to_section(10);
                gallery.motion_target = false;
                cx.notify();
            })
            .unwrap();
        draw(&window, cx);
        window
            .update(cx, |gallery, _, _| {
                assert_eq!(
                    gallery.motion_values, ongoing,
                    "reversal starts at the sampled value"
                );
            })
            .unwrap();
        cx.executor().advance_clock(Duration::from_secs(5));
        draw(&window, cx);
        window
            .update(cx, |gallery, _, _| {
                assert_eq!(gallery.motion_values, [0.; 6], "offscreen reversal settles");
            })
            .unwrap();
    }
}
