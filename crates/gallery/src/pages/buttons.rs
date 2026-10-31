//! Buttons: the kit's full button and toggle-button state matrices, live
//! transition, toggle, progress and icon demos, and the page's other
//! sections (icon button, split button, FAB menu, FAB, button groups).
use super::prelude::*;
use std::sync::OnceLock;

mod sections;
use sections::{
    EXTENDED_SIZES, FAB_SIZES, GROUP_VARIANTS, ICON_BANDS, ICON_TOGGLE_BANDS, ICON_VARIANTS,
    SPLIT_VARIANTS,
};

/// The kit's variant order and captions.
const VARIANTS: [(ButtonVariant, &str); 5] = [
    (ButtonVariant::Filled, "Filled"),
    (ButtonVariant::Tonal, "Tonal"),
    (ButtonVariant::Outlined, "Outline"),
    (ButtonVariant::Elevated, "Elevated"),
    (ButtonVariant::Text, "Text"),
];
/// Text buttons have no toggle form.
const TOGGLE_VARIANTS: usize = 4;

const ICON_BUTTON: PageInfo = PageInfo {
    id: "icon-button",
    title: "Icon button",
    description: "Icon buttons help people take minor actions with one tap.",
    guideline: Some("https://m3.material.io/components/icon-buttons/overview"),
};
const SPLIT_BUTTON: PageInfo = PageInfo {
    id: "split-button",
    title: "Split button",
    description: "Split buttons open a menu to give people more options related to an action.",
    guideline: Some("https://m3.material.io/components/split-button/overview"),
};
const FAB_MENU: PageInfo = PageInfo {
    id: "fab-menu",
    title: "FAB menu",
    description: "The floating action button (FAB) menu opens from a FAB to display multiple related actions.",
    guideline: Some("https://m3.material.io/components/fab-menu/overview"),
};
const FAB: PageInfo = PageInfo {
    id: "fab",
    title: "FAB",
    description: "Floating action buttons (FABs) help people take primary actions.",
    guideline: Some("https://m3.material.io/components/floating-action-button/overview"),
};
const BUTTON_GROUPS: PageInfo = PageInfo {
    id: "button-groups",
    title: "Button groups",
    description: "Button groups organize buttons and add interactions between them.",
    guideline: Some("https://m3.material.io/components/button-groups/overview"),
};

/// The page's further sections, in the kit's order.
const SECTIONS: [&PageInfo; 5] = [&ICON_BUTTON, &SPLIT_BUTTON, &FAB_MENU, &FAB, &BUTTON_GROUPS];

/// One body row of the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Row {
    /// A band of a button board: one shape, every size × state.
    Matrix {
        variant: usize,
        shape: usize,
    },
    /// A band of a toggle board: one shape and selection.
    Toggle {
        variant: usize,
        shape: usize,
        selected: bool,
    },
    EffectControls,
    EffectSamples,
    ToggleSize,
    ToggleShape(usize),
    LoadingControls,
    LoadingHelp,
    LoadingCount,
    LoadingSize(usize),
    Icons(usize),
    Section(usize),
    /// A band of an icon-button board (`toggle`: its selection).
    IconMatrix {
        variant: usize,
        band: usize,
        toggle: Option<bool>,
    },
    IconDemo,
    /// A band of a split-button board: one size and variant.
    Split {
        size: usize,
        variant: usize,
    },
    SplitDemo,
    FabMenuColors,
    FabMenuCounts,
    FabMenuBlocks,
    FabMenuDemo,
    /// A band of the FAB (or extended FAB) board: one size.
    Fab {
        size: usize,
        extended: bool,
    },
    FabDemo,
    StandardGroups {
        shape: usize,
        variant: usize,
    },
    StandardGroupDemo,
    ConnectedCounts,
    ConnectedSizes(usize),
    GroupStates,
    GroupDemo,
}

pub(crate) fn rows() -> &'static [Row] {
    static ROWS: OnceLock<Vec<Row>> = OnceLock::new();
    ROWS.get_or_init(|| {
        let mut rows = Vec::new();
        for variant in 0..VARIANTS.len() {
            for shape in 0..ButtonShape::ALL.len() {
                rows.push(Row::Matrix { variant, shape });
            }
            if variant < TOGGLE_VARIANTS {
                for shape in 0..ButtonShape::ALL.len() {
                    for selected in [false, true] {
                        rows.push(Row::Toggle {
                            variant,
                            shape,
                            selected,
                        });
                    }
                }
            }
        }
        rows.extend([Row::EffectControls, Row::EffectSamples, Row::ToggleSize]);
        rows.extend((0..ButtonShape::ALL.len()).map(Row::ToggleShape));
        rows.extend([Row::LoadingControls, Row::LoadingHelp, Row::LoadingCount]);
        rows.extend((0..ButtonSize::ALL.len()).map(Row::LoadingSize));
        rows.extend((0..ButtonSize::ALL.len()).map(Row::Icons));
        // Icon button: plain boards, then toggle boards, per variant.
        rows.push(Row::Section(0));
        rows.push(Row::IconDemo);
        for variant in 0..ICON_VARIANTS.len() {
            rows.extend((0..ICON_BANDS).map(|band| Row::IconMatrix {
                variant,
                band,
                toggle: None,
            }));
        }
        for variant in 0..ICON_VARIANTS.len() {
            rows.extend((0..ICON_TOGGLE_BANDS).map(|band| Row::IconMatrix {
                variant,
                band,
                toggle: Some(band % 2 == 1),
            }));
        }
        rows.push(Row::Section(1));
        rows.push(Row::SplitDemo);
        for size in 0..ButtonSize::ALL.len() {
            rows.extend((0..SPLIT_VARIANTS.len()).map(|variant| Row::Split { size, variant }));
        }
        rows.extend([
            Row::Section(2),
            Row::FabMenuDemo,
            Row::FabMenuColors,
            Row::FabMenuCounts,
            Row::FabMenuBlocks,
            Row::Section(3),
            Row::FabDemo,
        ]);
        rows.extend((0..FAB_SIZES.len()).map(|size| Row::Fab {
            size,
            extended: false,
        }));
        rows.extend((0..EXTENDED_SIZES.len()).map(|size| Row::Fab {
            size,
            extended: true,
        }));
        rows.push(Row::Section(4));
        rows.push(Row::StandardGroupDemo);
        for shape in 0..ButtonShape::ALL.len() {
            rows.extend(
                (0..GROUP_VARIANTS.len()).map(|variant| Row::StandardGroups { shape, variant }),
            );
        }
        rows.push(Row::ConnectedCounts);
        rows.extend((0..ButtonShape::ALL.len()).map(Row::ConnectedSizes));
        rows.extend([Row::GroupStates, Row::GroupDemo]);
        rows
    })
}

/// The body row of `row`.
pub(crate) fn row_of(row: Row) -> usize {
    rows()
        .iter()
        .position(|candidate| *candidate == row)
        .expect("the Buttons page lays out this row")
}

pub struct Page {
    page: PageCx,
    pub(crate) effects_disabled: bool,
    effects_tonal: bool,
    pub(crate) ripple: bool,
    pub(crate) active: bool,
    pub(crate) selections: [bool; 10],
    toggle_size: ButtonSize,
    pub(crate) loading: bool,
    progress: bool,
    loading_variant: usize,
    pub(crate) loading_clicks: usize,
    loading_playbacks: [ButtonTransitionPlayback; 15],
    effect_playbacks: [ButtonTransitionPlayback; 5],
    toggle_playbacks: [ButtonTransitionPlayback; 10],
    progress_playbacks: [ButtonProgressPlayback; 15],
    progress_inputs: Option<(bool, bool, bool)>,
    density: SharedString,
    icon_selections: [bool; 4],
    split_open: bool,
    split_clicks: usize,
    fab_menu_open: bool,
    fab_extended: bool,
    #[cfg(test)]
    pub(crate) sync_rows: std::collections::BTreeSet<usize>,
}

/// A kit specimen: "Label" with the star icon.
fn specimen(id: impl Into<ElementId>, variant: ButtonVariant) -> MaterialButton {
    MaterialButton::new(id)
        .variant(variant)
        .icon(spec::star_icon())
        .child("Label")
}

impl Page {
    fn shape_label(shape: ButtonShape) -> &'static str {
        match shape {
            ButtonShape::Round => "Round",
            ButtonShape::Square => "Square",
        }
    }

    fn matrix(&self, variant: usize, shape: usize) -> AnyElement {
        let (kind, caption) = VARIANTS[variant];
        let shape_value = ButtonShape::ALL[shape];
        board(("button-board", variant * 2 + shape))
            .band(Band::of(shape, ButtonShape::ALL.len()))
            .caption(caption)
            .side_label(Self::shape_label(shape_value))
            .columns(InteractionPreview::ALL.len())
            .when(shape == 0, |board| {
                board.column_labels(InteractionPreview::ALL.map(InteractionPreview::name))
            })
            .cells(
                ButtonSize::ALL
                    .into_iter()
                    .enumerate()
                    .flat_map(|(size_index, size)| {
                        InteractionPreview::ALL.into_iter().enumerate().map(
                            move |(state_index, state)| {
                                let id = ((variant * 2 + shape) * 5 + size_index) * 5 + state_index;
                                specimen(("matrix", id), kind)
                                    .shape(shape_value)
                                    .button_size(size)
                                    .preview_state(state)
                            },
                        )
                    }),
            )
            .into_any_element()
    }

    fn toggle_matrix(&self, variant: usize, shape: usize, selected: bool) -> AnyElement {
        let (kind, caption) = VARIANTS[variant];
        let shape_value = ButtonShape::ALL[shape];
        let band = shape * 2 + usize::from(selected);
        board(("toggle-board", variant * 4 + band))
            .band(Band::of(band, 4))
            .caption(SharedString::from(format!("{caption} - toggleable")))
            .side_label(SharedString::from(format!(
                "{}\n{}",
                Self::shape_label(shape_value),
                if selected { "Selected" } else { "Unselected" }
            )))
            .columns(InteractionPreview::ALL.len())
            .when(band == 0, |board| {
                board.column_labels(InteractionPreview::ALL.map(InteractionPreview::name))
            })
            .cells(
                ButtonSize::ALL
                    .into_iter()
                    .enumerate()
                    .flat_map(|(size_index, size)| {
                        InteractionPreview::ALL.into_iter().enumerate().map(
                            move |(state_index, state)| {
                                let id = ((variant * 4 + band) * 5 + size_index) * 5 + state_index;
                                specimen(("toggle-matrix", id), kind)
                                    .shape(shape_value)
                                    .button_size(size)
                                    .toggle(true)
                                    .selected(selected)
                                    .preview_state(state)
                            },
                        )
                    }),
            )
            .into_any_element()
    }

    fn effect_variant(&self, variant: ButtonVariant) -> ButtonVariant {
        if variant == ButtonVariant::Filled && self.effects_tonal {
            ButtonVariant::Tonal
        } else {
            variant
        }
    }

    fn count_loading_activation(&mut self, cx: &mut Context<Self>) {
        self.loading_clicks += 1;
        let row = row_of(Row::LoadingCount);
        self.page.rows.remeasure(row..=row);
        cx.notify();
    }

    fn remeasure_loading(&self) {
        let first = row_of(Row::LoadingControls);
        let last = row_of(Row::LoadingSize(ButtonSize::ALL.len() - 1));
        self.page.rows.remeasure(first..=last);
    }
}

impl GalleryPage for Page {
    const INFO: PageInfo = PageInfo {
        id: "buttons",
        title: "Buttons",
        description: "Buttons help people take actions, such as sending an email, sharing a document, or liking a comment.",
        guideline: Some("https://m3.material.io/components/buttons/overview"),
    };

    fn new(page: PageCx, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            page,
            effects_disabled: false,
            effects_tonal: false,
            ripple: true,
            active: false,
            selections: [false; 10],
            toggle_size: ButtonSize::Small,
            loading: false,
            progress: false,
            loading_variant: 0,
            loading_clicks: 0,
            loading_playbacks: std::array::from_fn(|_| ButtonTransitionPlayback::default()),
            effect_playbacks: std::array::from_fn(|_| ButtonTransitionPlayback::default()),
            toggle_playbacks: std::array::from_fn(|_| ButtonTransitionPlayback::default()),
            progress_playbacks: std::array::from_fn(|_| ButtonProgressPlayback::default()),
            progress_inputs: None,
            density: "auto".into(),
            icon_selections: [false, true, false, true],
            split_open: false,
            split_clicks: 0,
            fab_menu_open: false,
            fab_extended: true,
            #[cfg(test)]
            sync_rows: Default::default(),
        }
    }

    fn row_count(&self, _: &Window) -> usize {
        rows().len()
    }

    fn row_gap(&self, row: usize) -> f32 {
        match rows()[row] {
            _ if row == 0 => spec::HEADER_TO_BODY,
            Row::Matrix { shape, .. } => Band::of(shape, 2).row_gap() * 1.5,
            Row::Toggle {
                shape, selected, ..
            } => Band::of(shape * 2 + usize::from(selected), 4).row_gap() * 1.5,
            Row::Section(_) => 48.,
            Row::IconMatrix { band, toggle, .. } => {
                let bands = if toggle.is_some() {
                    ICON_TOGGLE_BANDS
                } else {
                    ICON_BANDS
                };
                Band::of(band, bands).row_gap() * 1.5
            }
            Row::Split { variant, .. } => Band::of(variant, SPLIT_VARIANTS.len()).row_gap() * 1.5,
            Row::Fab { size, extended } => {
                let bands = if extended {
                    EXTENDED_SIZES.len()
                } else {
                    FAB_SIZES.len()
                };
                Band::of(size, bands).row_gap() * 1.5
            }
            Row::StandardGroups { shape, variant } => {
                Band::of(
                    shape * GROUP_VARIANTS.len() + variant,
                    2 * GROUP_VARIANTS.len(),
                )
                .row_gap()
                    * 1.5
            }
            Row::ConnectedSizes(shape) => Band::of(shape, 2).row_gap() * 1.5,
            Row::EffectControls => 48.,
            Row::ToggleShape(_) | Row::LoadingSize(_) | Row::Icons(_) => 16.,
            _ => spec::ROW_GAP,
        }
    }

    fn interactive(&self, row: usize) -> bool {
        matches!(
            rows()[row],
            Row::EffectControls
                | Row::EffectSamples
                | Row::ToggleSize
                | Row::ToggleShape(_)
                | Row::LoadingControls
                | Row::LoadingSize(_)
                | Row::Icons(_)
                | Row::Section(_)
                | Row::IconDemo
                | Row::SplitDemo
                | Row::FabMenuDemo
                | Row::FabDemo
                | Row::StandardGroupDemo
                | Row::StandardGroups { .. }
                | Row::GroupDemo
        )
    }

    const PREPARES: bool = true;
    const OBSERVES_LAYOUT: bool = true;

    fn prepare(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        let inputs = (self.progress, self.loading, self.effects_disabled);
        for playback in &self.progress_playbacks {
            if self.progress_inputs != Some(inputs) {
                playback.sync(inputs.0, inputs.1, inputs.2, cx);
            }
            playback.observe_reduced_motion(cx);
        }
        self.progress_inputs = Some(inputs);
    }

    fn after_layout(&mut self, visible: &dyn Fn(usize) -> bool, cx: &mut Context<Self>) {
        #[cfg(test)]
        self.sync_rows.clear();
        let effect_row = row_of(Row::EffectSamples);
        if !visible(effect_row) {
            #[cfg(test)]
            self.sync_rows.insert(effect_row);
            for (index, (variant, _)) in VARIANTS.into_iter().enumerate() {
                self.effect_playbacks[index].sync(
                    ButtonTransitionInputs {
                        variant: self.effect_variant(variant),
                        disabled: self.effects_disabled,
                        active: self.active,
                        ripple: self.ripple,
                        ..Default::default()
                    },
                    cx,
                );
            }
        }
        for (size_index, size) in ButtonSize::ALL.into_iter().enumerate() {
            let row = row_of(Row::LoadingSize(size_index));
            if visible(row) {
                continue;
            }
            #[cfg(test)]
            self.sync_rows.insert(row);
            for icon_index in 0..3 {
                self.loading_playbacks[size_index * 3 + icon_index].sync(
                    ButtonTransitionInputs {
                        variant: VARIANTS[self.loading_variant].0,
                        size,
                        disabled: self.effects_disabled || self.loading,
                        ripple: self.ripple,
                        ..Default::default()
                    },
                    cx,
                );
            }
        }
        for (shape_index, shape) in ButtonShape::ALL.into_iter().enumerate() {
            let row = row_of(Row::ToggleShape(shape_index));
            if visible(row) {
                continue;
            }
            #[cfg(test)]
            self.sync_rows.insert(row);
            for (variant_index, (variant, _)) in VARIANTS.into_iter().enumerate() {
                let index = shape_index * 5 + variant_index;
                self.toggle_playbacks[index].sync(
                    ButtonTransitionInputs {
                        variant,
                        shape,
                        size: self.toggle_size,
                        toggle: true,
                        selected: self.selections[index],
                        disabled: self.effects_disabled,
                        ripple: self.ripple,
                        ..Default::default()
                    },
                    cx,
                );
            }
        }
    }

    fn apply_capture(&mut self, state: CaptureState, cx: &mut Context<Self>) {
        self.active = state.active;
        self.effects_disabled = state.disabled;
        self.selections = [state.selected; 10];
        self.loading = state.loading;
        self.ripple = state.ripple;
        cx.notify();
    }

    fn render_row(&mut self, row: usize, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let list = self.page.rows.list().clone();
        let rows = self.page.rows.clone();
        let this_row = row;
        match rows_table(row) {
            Row::Matrix { variant, shape } => self.matrix(variant, shape),
            Row::Toggle {
                variant,
                shape,
                selected,
            } => self.toggle_matrix(variant, shape, selected),
            Row::EffectControls => {
                let toggle = |label_on: &'static str,
                              label_off: &'static str,
                              id: &'static str,
                              on: bool,
                              change: fn(&mut Self)| {
                    let rows = rows.clone();
                    material_button(id, ButtonVariant::Outlined)
                        .child(if on { label_on } else { label_off })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            change(this);
                            rows.remeasure(this_row..=this_row + 1);
                            cx.notify();
                        }))
                        .reveal((id, 1usize), &list)
                };
                spec::demo(
                    "button-transitions",
                    "Interactive buttons",
                    spec::demo_row()
                        .child(toggle(
                            "Release active samples",
                            "Keep samples active",
                            "toggle-button-active",
                            self.active,
                            |this| this.active = !this.active,
                        ))
                        .child(toggle(
                            "Enable samples",
                            "Disable samples",
                            "toggle-button-disabled",
                            self.effects_disabled,
                            |this| this.effects_disabled = !this.effects_disabled,
                        ))
                        .child(toggle(
                            "Switch filled / tonal",
                            "Switch filled / tonal",
                            "toggle-button-variant",
                            self.effects_tonal,
                            |this| this.effects_tonal = !this.effects_tonal,
                        ))
                        .child(toggle(
                            "Turn ripple off",
                            "Turn ripple on",
                            "toggle-button-ripple",
                            self.ripple,
                            |this| this.ripple = !this.ripple,
                        )),
                )
                .into_any_element()
            }
            Row::EffectSamples => spec::demo_row()
                .debug_selector(|| "button-effect-samples".into())
                .children(VARIANTS.into_iter().enumerate().map(|(index, (variant, _))| {
                    material_button(("effect-sample", index), self.effect_variant(variant))
                        .transition_playback(self.effect_playbacks[index].clone())
                        .disabled(self.effects_disabled)
                        .active_state(self.active)
                        .ripple(self.ripple)
                        .icon(IconName::Check)
                        .child(variant.name())
                        .reveal(("reveal-effect-sample", index), &list)
                }))
                .into_any_element(),
            Row::ToggleSize => spec::demo(
                "toggle-buttons",
                "Toggle buttons",
                ConnectedButtonGroup::new("toggle-size", "Toggle button size")
                    .options(ButtonSize::ALL.map(|size| (size.name(), size.name())))
                    .selected(self.toggle_size.name())
                    .on_change(cx.listener(move |this, name: &SharedString, _, cx| {
                        if let Some(size) =
                            ButtonSize::ALL.into_iter().find(|size| size.name() == name.as_ref())
                        {
                            this.toggle_size = size;
                            let first = row_of(Row::ToggleShape(0));
                            rows.remeasure(first..=first + ButtonShape::ALL.len() - 1);
                            cx.notify();
                        }
                    }))
                    .reveal("reveal-toggle-size", &list),
            )
            .into_any_element(),
            Row::ToggleShape(shape_index) => {
                let shape = ButtonShape::ALL[shape_index];
                div()
                    .when(shape_index == 0, |row| {
                        row.debug_selector(|| "button-toggle-shapes".into())
                    })
                    .flex()
                    .flex_col()
                    .gap(rem(12.))
                    .child(spec::small_label(("toggle-shape", shape_index), shape.name()))
                    .child(spec::demo_row().children(VARIANTS.into_iter().enumerate().map(
                        |(variant_index, (variant, _))| {
                            let index = shape_index * 5 + variant_index;
                            MaterialButton::new(("toggle", index))
                                .variant(variant)
                                .shape(shape)
                                .button_size(self.toggle_size)
                                .toggle(true)
                                .selected(self.selections[index])
                                .transition_playback(self.toggle_playbacks[index].clone())
                                .disabled(self.effects_disabled)
                                .ripple(self.ripple)
                                .icon(IconName::Check)
                                .child(variant.name())
                                .on_change({
                                    let page = cx.entity().downgrade();
                                    move |selected, _, _, cx| {
                                        let _ = page.update(cx, |this, cx| {
                                            this.selections[index] = selected;
                                            cx.notify();
                                        });
                                    }
                                })
                                .reveal(("reveal-toggle", index), &list)
                        },
                    )))
                    .into_any_element()
            }
            Row::LoadingControls => {
                let variant = VARIANTS[self.loading_variant].0;
                let control = |id: &'static str, label: SharedString, change: fn(&mut Self)| {
                    material_button(id, ButtonVariant::Outlined)
                        .child(label)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            change(this);
                            this.remeasure_loading();
                            cx.notify();
                        }))
                        .reveal((id, 1usize), &list)
                };
                spec::demo(
                    "button-loading",
                    "Progress and loading",
                    spec::demo_row()
                        .child(control(
                            "toggle-loading",
                            if self.loading { "Stop loading" } else { "Start loading" }.into(),
                            |this| this.loading = !this.loading,
                        ))
                        .child(control(
                            "toggle-progress",
                            if self.progress { "Hide progress" } else { "Show progress" }.into(),
                            |this| this.progress = !this.progress,
                        ))
                        .child(control(
                            "loading-variant",
                            format!("Variant: {}", variant.name()).into(),
                            |this| this.loading_variant = (this.loading_variant + 1) % VARIANTS.len(),
                        )),
                )
                .into_any_element()
            }
            Row::LoadingHelp => accessible_text(
                "loading-help",
                "Progress stays interactive. Loading disables the samples and changes their labels.",
            )
            .into_any_element(),
            Row::LoadingCount => accessible_text(
                "loading-click-count",
                SharedString::from(format!("Sample activations: {}", self.loading_clicks)),
            )
            .debug_selector(|| "loading-click-count".into())
            .into_any_element(),
            Row::LoadingSize(size_index) => {
                let size = ButtonSize::ALL[size_index];
                let variant = VARIANTS[self.loading_variant].0;
                div()
                    .when(size_index == 0, |row| {
                        row.debug_selector(|| "button-loading-sizes".into())
                    })
                    .flex()
                    .flex_col()
                    .gap(rem(12.))
                    .child(spec::small_label(("loading-size", size_index), size.name()))
                    .child(spec::demo_row().children((0..3).map(|icon_position| {
                        let index = size_index * 3 + icon_position;
                        MaterialButton::new(("loading", index))
                            .variant(variant)
                            .button_size(size)
                            .when(icon_position > 0, |button| button.icon(IconName::Check))
                            .when(icon_position == 2, |button| {
                                button.icon_position(ButtonIconPosition::Trailing)
                            })
                            .disabled(self.effects_disabled)
                            .ripple(self.ripple)
                            .progress(self.progress, "Operation progress")
                            .loading(self.loading, "Operation loading")
                            .transition_playback(self.loading_playbacks[index].clone())
                            .progress_playback(self.progress_playbacks[index].clone())
                            .child(if self.loading { "Saving…" } else { "Save changes" })
                            .on_click(cx.listener(|this, _, _, cx| this.count_loading_activation(cx)))
                            .reveal(("reveal-loading", index), &list)
                    })))
                    .into_any_element()
            }
            Row::Icons(size_index) => {
                let size = ButtonSize::ALL[size_index];
                div()
                    .flex()
                    .flex_col()
                    .gap(rem(12.))
                    .when(size_index == 0, |column| {
                        column.child(spec::small_label("button-icons", "Icons and direction"))
                    })
                    .child(spec::demo_row().children(
                        [
                            ("Leading", ButtonIconPosition::Leading, ButtonDirection::Ltr, ButtonVariant::Filled),
                            ("Trailing", ButtonIconPosition::Trailing, ButtonDirection::Ltr, ButtonVariant::Filled),
                            ("RTL leading", ButtonIconPosition::Leading, ButtonDirection::Rtl, ButtonVariant::Text),
                            ("RTL trailing", ButtonIconPosition::Trailing, ButtonDirection::Rtl, ButtonVariant::Text),
                        ]
                        .into_iter()
                        .enumerate()
                        .map(|(position_index, (label, position, direction, variant))| {
                            let sample = size_index * 4 + position_index;
                            material_button(("icon", sample), variant)
                                .button_size(size)
                                .icon(IconName::Check)
                                .icon_position(position)
                                .direction(direction)
                                .child(label)
                                .reveal(("reveal-icon", sample), &list)
                        }),
                    ))
                    .into_any_element()
            }
            Row::Section(index) => spec::section_header(SECTIONS[index]).into_any_element(),
            Row::IconMatrix {
                variant,
                band,
                toggle,
            } => sections::icon_matrix(variant, band, toggle),
            Row::IconDemo => spec::demo(
                "icon-button-demo",
                "Toggle icon buttons",
                spec::demo_row().children(ICON_VARIANTS.into_iter().enumerate().map(
                    |(index, (variant, name))| {
                        IconButton::new(("icon-demo", index), spec::star_icon(), name)
                            .variant(variant)
                            .button_size(ButtonSize::Medium)
                            .toggle(true)
                            .selected(self.icon_selections[index])
                            .on_change({
                                let page = cx.entity().downgrade();
                                move |selected, _, _, cx| {
                                    let _ = page.update(cx, |this, cx| {
                                        this.icon_selections[index] = selected;
                                        cx.notify();
                                    });
                                }
                            })
                            .reveal(("reveal-icon-demo", index), &list)
                    },
                )),
            )
            .into_any_element(),
            Row::Split { size, variant } => sections::split_matrix(size, variant),
            Row::SplitDemo => spec::demo(
                "split-button-demo",
                "Split button",
                spec::demo_row()
                    .child(
                        SplitButton::new("split-demo", "Save", "More save options")
                            .icon(spec::star_icon())
                            .button_size(ButtonSize::Medium)
                            .open(self.split_open)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.split_clicks += 1;
                                cx.notify();
                            }))
                            .on_toggle(cx.listener(|this, open: &bool, _, cx| {
                                this.split_open = *open;
                                cx.notify();
                            }))
                            .reveal("reveal-split-demo", &list),
                    )
                    .child(accessible_text(
                        "split-demo-status",
                        SharedString::from(format!(
                            "Saved {} times · menu {}",
                            self.split_clicks,
                            if self.split_open { "open" } else { "closed" }
                        )),
                    )),
            )
            .into_any_element(),
            Row::FabMenuColors => sections::fab_menu_colors(),
            Row::FabMenuCounts => sections::fab_menu_counts(),
            Row::FabMenuBlocks => sections::fab_menu_blocks(),
            Row::FabMenuDemo => spec::demo(
                "fab-menu-demo",
                "FAB menu",
                // Room for the open stack and the FAB's shadow inside the row.
                div().h(rem(4. * 56. + 8. + 2. * 4. + 24.)).p(rem(12.)).flex().items_end().child(
                    FabMenu::new("fab-menu-demo", IconName::Check, "Actions")
                        .open(self.fab_menu_open)
                        .items(["Reply", "Forward", "Archive"].into_iter().enumerate().map(
                            |(index, label)| {
                                FabMenuItem::new(("fab-menu-demo-item", index), spec::star_icon(), label)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.fab_menu_open = false;
                                        cx.notify();
                                    }))
                            },
                        ))
                        .on_toggle(cx.listener(|this, open: &bool, _, cx| {
                            this.fab_menu_open = *open;
                            cx.notify();
                        }))
                        .reveal("reveal-fab-menu-demo", &list),
                ),
            )
            .into_any_element(),
            Row::Fab { size, extended } => sections::fab_matrix(size, extended),
            Row::FabDemo => spec::demo(
                "fab-demo",
                "Extended FAB",
                spec::demo_row()
                    .child(
                        material_button("fab-demo-toggle", ButtonVariant::Outlined)
                            .child(if self.fab_extended { "Collapse" } else { "Expand" })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.fab_extended = !this.fab_extended;
                                cx.notify();
                            }))
                            .reveal("reveal-fab-demo-toggle", &list),
                    )
                    .child(
                        ExtendedFab::new("fab-demo", spec::star_icon(), "Compose")
                            .expanded(self.fab_extended)
                            .reveal("reveal-fab-demo", &list),
                    )
                    .child(
                        ExtendedFab::new("fab-demo-medium", spec::star_icon(), "Compose")
                            .size(FabSize::Medium)
                            .color(FabColor::Tertiary)
                            .expanded(self.fab_extended)
                            .reveal("reveal-fab-demo-medium", &list),
                    ),
            )
            .into_any_element(),
            Row::StandardGroups { shape, variant } => sections::standard_groups(shape, variant),
            Row::StandardGroupDemo => spec::demo(
                "standard-group-demo",
                "Examples & usage",
                spec::demo_row()
                    .child(
                        ButtonGroup::new("usage-icons", "Playback")
                            .button_size(ButtonSize::Medium)
                            .items(["Previous", "Play", "Next", "Shuffle"].into_iter().enumerate().map(
                                |(index, label)| {
                                    IconButton::new(("usage-icon", index), spec::star_icon(), label)
                                        .variant(IconButtonVariant::Tonal)
                                        .button_size(ButtonSize::Medium)
                                        .width(if index == 1 {
                                            IconButtonWidth::Wide
                                        } else {
                                            IconButtonWidth::Default
                                        })
                                },
                            )),
                    )
                    .child(
                        ButtonGroup::new("usage-labels", "Actions")
                            .button_size(ButtonSize::Small)
                            .items(["Start", "Directions", "Share"].into_iter().enumerate().map(
                                |(index, label)| {
                                    MaterialButton::new(("usage-label", index))
                                        .variant(ButtonVariant::Tonal)
                                        .icon(spec::star_icon())
                                        .child(label)
                                },
                            )),
                    ),
            )
            .into_any_element(),
            Row::ConnectedCounts => sections::connected_counts(),
            Row::ConnectedSizes(shape) => sections::connected_sizes(shape),
            Row::GroupStates => board("button-group-states")
                .caption("Connected button group")
                .columns(1)
                .row_labels(InteractionPreview::ALL.map(InteractionPreview::name))
                .cells(InteractionPreview::ALL.into_iter().enumerate().map(|(index, state)| {
                    ConnectedButtonGroup::new(("group-state", index), "Connected button group")
                        .options([("first", "Label"), ("second", "Label"), ("third", "Label")])
                        .selected("first")
                        .preview_state("second", state)
                }))
                .into_any_element(),
            Row::GroupDemo => spec::demo(
                "button-group-demo",
                "Segmented",
                ConnectedButtonGroup::new("density", "Density")
                    .segmented()
                    .options([("auto", "Automatic"), ("dense", "Dense"), ("roomy", "Roomy")])
                    .icon("auto", IconName::Info)
                    .icon("dense", IconName::Check)
                    .icon("roomy", IconName::Warning)
                    .selected(self.density.clone())
                    .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                        this.density = value.clone();
                        cx.notify();
                    }))
                    .reveal("reveal-density", &list),
            )
            .into_any_element(),
        }
    }
}

fn rows_table(row: usize) -> Row {
    rows()[row]
}
