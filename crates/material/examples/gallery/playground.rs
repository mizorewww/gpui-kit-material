//! Declarative component playgrounds, modelled on md3.io's component pages:
//! a configuration card of controls (grouped in sections, with scenario
//! presets and Reset) driving a live preview of the real component.
//!
//! # Writing a playground
//!
//! A playground is one static in `playgrounds/<component>.rs`:
//!
//! ```ignore
//! pub static PLAYGROUND: Playground = Playground {
//!     scenarios: &[Scenario {
//!         name: "Save",
//!         values: &[("text", Set::Text("Save")), ("variant", Set::Option("tonal"))],
//!     }],
//!     sections: &[Section {
//!         title: "Appearance",
//!         controls: &[
//!             Control::Select { id: "variant", label: "Variant", options: &["filled", "tonal"], default: 0 },
//!             Control::Switch { id: "disabled", label: "Disabled", default: false },
//!         ],
//!     }],
//!     preview,
//! };
//!
//! fn preview(config: &Config, preview: &mut Preview, window: &mut Window, cx: &mut Context<PlaygroundState>) -> AnyElement {
//!     MaterialButton::new("button")
//!         .variant(match config.option("variant") { "tonal" => ButtonVariant::Tonal, _ => ButtonVariant::Filled })
//!         .disabled(config.bool("disabled"))
//!         .child(config.text("text"))
//!         .into_any_element()
//! }
//! ```
//!
//! then set the catalog entry's `playground` to `Some(&playgrounds::<component>::PLAYGROUND)`.
//!
//! The preview reads the configuration through [`Config`]'s typed getters
//! (`option`, `choice`, `bool`, `number`, `text`; an unknown id panics, and
//! the catalog tests render every playground with every scenario). It writes
//! back through [`Preview`]: `on_bool(id, cx)` / `setter(id, cx)` for
//! component callbacks (so clicking the preview's switch flips the
//! "Checked" control), `input`/`textarea` for editors bound two-way to a
//! `Control::Text`, `changed` to apply a configuration change to such an
//! editor once, and `local` for other preview-only state. Configuration
//! survives navigation: each page keeps its [`PlaygroundState`].
use super::page::GalleryShared;
use super::shell::{LeadView, ShellLayout, panes_stacked, preview_canvas};
use super::spec::type_role;
use gpui::{prelude::*, *};
use gpui_base::input::{InputEvent, InputState, TextareaState};
use gpui_base::slider::{SliderEvent, SliderState};
use gpui_kit_material::icons::{Icon, Symbol};
use gpui_kit_material::prelude::*;
use std::any::Any;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

/// One configuration control. Option values double as their labels, as on
/// md3.io (e.g. `"filled"`, `"XS"`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Control {
    /// An exposed dropdown menu.
    Select {
        id: &'static str,
        label: &'static str,
        options: &'static [&'static str],
        default: usize,
    },
    /// A connected button group.
    Segmented {
        id: &'static str,
        label: &'static str,
        options: &'static [&'static str],
        default: usize,
    },
    /// A row of icon toggle buttons; options are Material Symbol names and
    /// `""` means none.
    Icon {
        id: &'static str,
        label: &'static str,
        icons: &'static [&'static str],
        default: usize,
    },
    Switch {
        id: &'static str,
        label: &'static str,
        default: bool,
    },
    Slider {
        id: &'static str,
        label: &'static str,
        min: f32,
        max: f32,
        step: f32,
        default: f32,
    },
    /// A single-line text field.
    Text {
        id: &'static str,
        label: &'static str,
        default: &'static str,
    },
}

impl Control {
    pub const fn id(&self) -> &'static str {
        match *self {
            Control::Select { id, .. }
            | Control::Segmented { id, .. }
            | Control::Icon { id, .. }
            | Control::Switch { id, .. }
            | Control::Slider { id, .. }
            | Control::Text { id, .. } => id,
        }
    }
    pub const fn label(&self) -> &'static str {
        match *self {
            Control::Select { label, .. }
            | Control::Segmented { label, .. }
            | Control::Icon { label, .. }
            | Control::Switch { label, .. }
            | Control::Slider { label, .. }
            | Control::Text { label, .. } => label,
        }
    }
    /// The option strings of a choice control.
    pub fn options(&self) -> Option<&'static [&'static str]> {
        match *self {
            Control::Select { options, .. } | Control::Segmented { options, .. } => Some(options),
            Control::Icon { icons, .. } => Some(icons),
            _ => None,
        }
    }
    fn default_value(&self) -> Value {
        match *self {
            Control::Select { default, .. }
            | Control::Segmented { default, .. }
            | Control::Icon { default, .. } => Value::Choice(default),
            Control::Switch { default, .. } => Value::Bool(default),
            Control::Slider { default, .. } => Value::Number(default),
            Control::Text { default, .. } => Value::Text(SharedString::new_static(default)),
        }
    }
    /// The value `set` assigns to this control, if it fits.
    fn value_of(&self, set: Set) -> Option<Value> {
        match (set, self.options()) {
            (Set::Option(option), Some(options)) => options
                .iter()
                .position(|candidate| *candidate == option)
                .map(Value::Choice),
            (Set::Bool(value), None) if matches!(self, Control::Switch { .. }) => {
                Some(Value::Bool(value))
            }
            (Set::Number(value), None) if matches!(self, Control::Slider { .. }) => {
                Some(Value::Number(value))
            }
            (Set::Text(value), None) if matches!(self, Control::Text { .. }) => {
                Some(Value::Text(SharedString::new_static(value)))
            }
            _ => None,
        }
    }
}

/// A titled group of controls (md3.io's Appearance, Content, Behavior…).
#[derive(Clone, Copy, Debug)]
pub struct Section {
    pub title: &'static str,
    pub controls: &'static [Control],
}

/// A scenario's value for one control.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Set {
    /// A choice control's option string.
    Option(&'static str),
    Bool(bool),
    Number(f32),
    Text(&'static str),
}

/// A named preset: the defaults with these values applied.
#[derive(Clone, Copy, Debug)]
pub struct Scenario {
    pub name: &'static str,
    pub values: &'static [(&'static str, Set)],
}

/// Renders the preview canvas content from the configuration.
pub type PreviewFn =
    fn(&Config, &mut Preview, &mut Window, &mut Context<PlaygroundState>) -> AnyElement;

/// A component's playground. See the module documentation.
pub struct Playground {
    /// Presets after "Default" (the controls' defaults).
    pub scenarios: &'static [Scenario],
    pub sections: &'static [Section],
    pub preview: PreviewFn,
}

impl Playground {
    pub fn controls(&self) -> impl Iterator<Item = &'static Control> + use<> {
        let sections: &'static [Section] = self.sections;
        sections.iter().flat_map(|section| section.controls.iter())
    }
    #[cfg_attr(target_family = "wasm", allow(dead_code))]
    pub fn control(&self, id: &str) -> Option<&'static Control> {
        self.controls().find(|control| control.id() == id)
    }
}

/// A control's current value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// The index into a choice control's options.
    Choice(usize),
    Bool(bool),
    Number(f32),
    Text(SharedString),
}

impl Value {
    /// Parse `text` as a value for `control` (capture `--set id=value`).
    #[cfg_attr(target_family = "wasm", allow(dead_code))]
    pub fn parse(control: &Control, text: &str) -> Option<Value> {
        match control {
            Control::Switch { .. } => match text {
                "true" | "on" | "1" => Some(Value::Bool(true)),
                "false" | "off" | "0" => Some(Value::Bool(false)),
                _ => None,
            },
            Control::Slider { min, max, .. } => text
                .parse::<f32>()
                .ok()
                .filter(|value| (*min..=*max).contains(value))
                .map(Value::Number),
            Control::Text { .. } => Some(Value::Text(text.to_owned().into())),
            _ => control
                .options()?
                .iter()
                .position(|option| *option == text)
                .map(Value::Choice),
        }
    }
}

/// The playground's current configuration, read by the preview.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    values: Vec<(&'static Control, Value)>,
}

impl Config {
    pub fn defaults(playground: &Playground) -> Self {
        Self {
            values: playground
                .controls()
                .map(|control| (control, control.default_value()))
                .collect(),
        }
    }
    /// The defaults with `scenario` applied.
    pub fn scenario(playground: &Playground, scenario: &Scenario) -> Self {
        let mut config = Self::defaults(playground);
        for (id, set) in scenario.values {
            let entry = config.entry_mut(id);
            let value = entry
                .0
                .value_of(*set)
                .unwrap_or_else(|| panic!("scenario {} sets {id} to {set:?}", scenario.name));
            entry.1 = value;
        }
        config
    }
    fn entry(&self, id: &str) -> &(&'static Control, Value) {
        self.values
            .iter()
            .find(|(control, _)| control.id() == id)
            .unwrap_or_else(|| panic!("no playground control {id}"))
    }
    fn entry_mut(&mut self, id: &str) -> &mut (&'static Control, Value) {
        self.values
            .iter_mut()
            .find(|(control, _)| control.id() == id)
            .unwrap_or_else(|| panic!("no playground control {id}"))
    }
    // Playground API: previews may read a value without its kind.
    #[allow(dead_code)]
    pub fn value(&self, id: &str) -> &Value {
        &self.entry(id).1
    }
    /// Replace a value; false when it was already `value`.
    pub fn set(&mut self, id: &str, value: Value) -> bool {
        let entry = self.entry_mut(id);
        if entry.1 == value {
            return false;
        }
        entry.1 = value;
        true
    }
    /// A choice control's selected index.
    pub fn choice(&self, id: &str) -> usize {
        match self.entry(id) {
            (_, Value::Choice(index)) => *index,
            (control, value) => panic!("{} is not a choice: {value:?}", control.id()),
        }
    }
    /// A choice control's selected option string.
    pub fn option(&self, id: &str) -> &'static str {
        let (control, _) = self.entry(id);
        control.options().expect("a choice control")[self.choice(id)]
    }
    pub fn bool(&self, id: &str) -> bool {
        match self.entry(id) {
            (_, Value::Bool(value)) => *value,
            (control, value) => panic!("{} is not a switch: {value:?}", control.id()),
        }
    }
    pub fn number(&self, id: &str) -> f32 {
        match self.entry(id) {
            (_, Value::Number(value)) => *value,
            (control, value) => panic!("{} is not a slider: {value:?}", control.id()),
        }
    }
    pub fn text(&self, id: &str) -> SharedString {
        match self.entry(id) {
            (_, Value::Text(value)) => value.clone(),
            (control, value) => panic!("{} is not text: {value:?}", control.id()),
        }
    }
}

/// A Material Symbol (outlined) at `size` design px.
pub fn symbol(name: &str, size: f32) -> Icon {
    Icon::from(
        Symbol::named(name)
            .unwrap_or_else(|| panic!("{name} is not a Material Symbol"))
            .icon(),
    )
    .size(rem(size))
}

/// Preview-only state, kept per playground across frames and navigation.
#[derive(Default)]
pub struct PreviewStore {
    inputs: HashMap<&'static str, Entity<InputState>>,
    textareas: HashMap<&'static str, Entity<TextareaState>>,
    locals: HashMap<&'static str, Box<dyn Any>>,
    memos: HashMap<&'static str, u64>,
    hint: PaneHint,
    subscriptions: Vec<Subscription>,
}

/// How a preview asks the shell to arrange the panes, in this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaneHint {
    /// Stack the configuration card under the preview even in a wide window.
    pub stacked: bool,
    /// The card height the preview needs, in design px, if it is more than
    /// the window's.
    pub height: Option<f32>,
    /// Use the canvas's whole width: no padding around the preview.
    pub flush: bool,
}

/// What a preview uses to write back to its playground. See the module
/// documentation.
pub struct Preview<'a> {
    config: &'a Config,
    store: &'a mut PreviewStore,
}

impl Preview<'_> {
    /// A callback setting control `id` to the callback's value, e.g. a
    /// switch's `on_change` bound to its "Checked" control.
    pub fn on_bool(
        &self,
        id: &'static str,
        cx: &mut Context<PlaygroundState>,
    ) -> impl Fn(&bool, &mut Window, &mut App) + 'static {
        let state = cx.weak_entity();
        move |value: &bool, window: &mut Window, cx: &mut App| {
            let _ = state.update(cx, |state, cx| {
                state.set_value(id, Value::Bool(*value), window, cx)
            });
        }
    }

    /// A callback setting control `id` to any value.
    pub fn setter(
        &self,
        id: &'static str,
        cx: &mut Context<PlaygroundState>,
    ) -> impl Fn(Value, &mut Window, &mut App) + 'static {
        let state = cx.weak_entity();
        move |value: Value, window: &mut Window, cx: &mut App| {
            let _ = state.update(cx, |state, cx| state.set_value(id, value, window, cx));
        }
    }

    /// A single-line editor bound two-way to the `Control::Text` `id`:
    /// typing updates the control, and control changes replace its text.
    pub fn input(
        &mut self,
        id: &'static str,
        window: &mut Window,
        cx: &mut Context<PlaygroundState>,
    ) -> Entity<InputState> {
        if let Some(input) = self.store.inputs.get(id) {
            return input.clone();
        }
        let value = self.config.text(id);
        let input = cx.new(|cx| InputState::new(window, cx).default_value(value));
        self.store.subscriptions.push(cx.subscribe_in(
            &input,
            window,
            move |state, input, event, window, cx| {
                if matches!(event, InputEvent::Change) {
                    let text = input.read(cx).value();
                    state.set_value(id, Value::Text(text), window, cx);
                }
            },
        ));
        self.store.inputs.insert(id, input.clone());
        input
    }

    /// A multi-line editor bound two-way to the `Control::Text` `id`.
    pub fn textarea(
        &mut self,
        id: &'static str,
        window: &mut Window,
        cx: &mut Context<PlaygroundState>,
    ) -> Entity<TextareaState> {
        if let Some(input) = self.store.textareas.get(id) {
            return input.clone();
        }
        let value = self.config.text(id);
        let input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .auto_grow(1, 5)
                .default_value(value)
        });
        self.store.subscriptions.push(cx.subscribe_in(
            &input,
            window,
            move |state, input, event, window, cx| {
                if matches!(event, InputEvent::Change) {
                    let text = input.read(cx).value();
                    state.set_value(id, Value::Text(text), window, cx);
                }
            },
        ));
        self.store.textareas.insert(id, input.clone());
        input
    }

    /// Whether `value` differs from the last value recorded under `key`
    /// (true the first time): apply configuration to stateful editors once
    /// per change instead of every frame.
    pub fn changed(&mut self, key: &'static str, value: impl std::hash::Hash) -> bool {
        use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher};
        let hash = BuildHasherDefault::<DefaultHasher>::default().hash_one(value);
        self.store.memos.insert(key, hash) != Some(hash)
    }

    /// Ask the shell to arrange the panes this way for this frame.
    pub fn pane_hint(&mut self, hint: PaneHint) {
        self.store.hint = hint;
    }

    /// Preview-only state `T` under `key`, created by `init` on first use.
    /// Its notifications re-render the playground.
    // Playground API for previews with state beyond their controls.
    #[allow(dead_code)]
    pub fn local<T: 'static>(
        &mut self,
        key: &'static str,
        window: &mut Window,
        cx: &mut Context<PlaygroundState>,
        init: impl FnOnce(&mut Window, &mut Context<T>) -> T,
    ) -> Entity<T> {
        if let Some(entity) = self
            .store
            .locals
            .get(key)
            .and_then(|local| local.downcast_ref::<Entity<T>>())
        {
            return entity.clone();
        }
        let entity = cx.new(|cx| init(window, cx));
        self.store
            .subscriptions
            .push(cx.observe(&entity, |_, _, cx| cx.notify()));
        self.store.locals.insert(key, Box::new(entity.clone()));
        entity
    }
}

/// The scenario select's extra entries.
const DEFAULT_SCENARIO: &str = "Default";
const CUSTOM_SCENARIO: &str = "Custom";
/// The configuration card's content width in design px.
// Playground API: previews may size content to the card.
#[allow(dead_code)]
pub const PANEL_CONTENT_WIDTH: f32 = 300.;

/// One playground's live state: its configuration, the configuration
/// card's editors and the preview's state. Created when its page is first
/// shown and kept while the Gallery runs.
pub struct PlaygroundState {
    pub playground: &'static Playground,
    pub config: Config,
    /// The selected scenario: 0 is "Default", `1..` the playground's
    /// scenarios, `None` a custom configuration.
    pub scenario: Option<usize>,
    /// The open select: a control id, or "" for the scenario select.
    open: Option<&'static str>,
    /// Whether the configuration last drew in the compact layout.
    compact_controls: bool,
    texts: HashMap<&'static str, Entity<InputState>>,
    sliders: HashMap<&'static str, Entity<SliderState>>,
    store: PreviewStore,
    pub scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl PlaygroundState {
    pub fn new(
        playground: &'static Playground,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let config = Config::defaults(playground);
        let mut texts = HashMap::new();
        let mut sliders = HashMap::new();
        let mut subscriptions = Vec::new();
        for control in playground.controls() {
            match *control {
                Control::Text { id, default, .. } => {
                    let input = cx.new(|cx| InputState::new(window, cx).default_value(default));
                    subscriptions.push(cx.subscribe_in(
                        &input,
                        window,
                        move |this: &mut Self, input, event, window, cx| {
                            if matches!(event, InputEvent::Change) {
                                let text = input.read(cx).value();
                                this.set_value(id, Value::Text(text), window, cx);
                            }
                        },
                    ));
                    texts.insert(id, input);
                }
                Control::Slider {
                    id,
                    min,
                    max,
                    step,
                    default,
                    ..
                } => {
                    let slider = cx.new(|_| {
                        // Raise the maximum first: setting a minimum above
                        // the default maximum (100) would clamp min > max.
                        SliderState::new()
                            .max(max)
                            .min(min)
                            .step(step)
                            .default_value(default)
                    });
                    subscriptions.push(cx.subscribe_in(
                        &slider,
                        window,
                        move |this: &mut Self, _, event: &SliderEvent, window, cx| {
                            if let SliderEvent::Change(value) = event {
                                this.set_value(id, Value::Number(value.start()), window, cx);
                            }
                        },
                    ));
                    sliders.insert(id, slider);
                }
                _ => {}
            }
        }
        Self {
            playground,
            config,
            scenario: Some(0),
            open: None,
            compact_controls: false,
            texts,
            sliders,
            store: PreviewStore::default(),
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    /// Set control `id`, mirror it into its editors and mark the
    /// configuration custom.
    pub fn set_value(
        &mut self,
        id: &str,
        value: Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.config.set(id, value) {
            self.scenario = None;
            self.sync_editors(window, cx);
            cx.notify();
        }
    }

    /// Restore the defaults (md3.io's Reset).
    pub fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.select_scenario(0, window, cx);
    }

    /// Apply scenario `index` (0 is Default).
    pub fn select_scenario(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.config = match index.checked_sub(1) {
            None => Config::defaults(self.playground),
            Some(scenario) => match self.playground.scenarios.get(scenario) {
                Some(scenario) => Config::scenario(self.playground, scenario),
                // "Custom" keeps the current configuration.
                None => return,
            },
        };
        self.scenario = Some(index);
        self.open = None;
        self.sync_editors(window, cx);
        cx.notify();
    }

    /// Set several controls at once (capture `--set`).
    pub fn apply(
        &mut self,
        values: &[(&'static str, Value)],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        for (id, value) in values {
            self.set_value(id, value.clone(), window, cx);
        }
    }

    /// Push the configuration into every editor showing a different value.
    fn sync_editors(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for (control, value) in &self.config.values {
            let id = control.id();
            match value {
                Value::Text(text) => {
                    let editors = self
                        .texts
                        .get(id)
                        .into_iter()
                        .chain(self.store.inputs.get(id));
                    for input in editors {
                        if input.read(cx).value() != *text {
                            let text = text.clone();
                            input.update(cx, |input, cx| input.set_value(text, window, cx));
                        }
                    }
                    if let Some(input) = self.store.textareas.get(id)
                        && input.read(cx).value() != *text
                    {
                        let text = text.clone();
                        input.update(cx, |input, cx| input.set_value(text, window, cx));
                    }
                }
                Value::Number(number) => {
                    if let Some(slider) = self.sliders.get(id)
                        && slider.read(cx).value().start() != *number
                    {
                        let number = *number;
                        slider.update(cx, |slider, cx| slider.set_value(number, window, cx));
                    }
                }
                _ => {}
            }
        }
    }

    /// The preview canvas content.
    pub fn render_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let preview_fn = self.playground.preview;
        self.store.hint = PaneHint::default();
        let mut preview = Preview {
            config: &self.config,
            store: &mut self.store,
        };
        preview_fn(&self.config, &mut preview, window, cx)
    }

    /// The preview-only state under `key`, once the preview has built it.
    #[cfg(test)]
    pub fn local_entity<T: 'static>(&self, key: &'static str) -> Option<Entity<T>> {
        self.store
            .locals
            .get(key)
            .and_then(|local| local.downcast_ref::<Entity<T>>())
            .cloned()
    }

    /// The panes' arrangement the last rendered preview asked for.
    pub fn pane_hint(&self) -> PaneHint {
        self.store.hint
    }

    fn scenario_names(&self) -> impl Iterator<Item = &'static str> + use<> {
        let scenarios: &'static [Scenario] = self.playground.scenarios;
        std::iter::once(DEFAULT_SCENARIO)
            .chain(scenarios.iter().map(|scenario| scenario.name))
            .chain(std::iter::once(CUSTOM_SCENARIO))
    }

    /// The configuration card's body: the scenario select, then each
    /// section's controls.
    pub fn render_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let compact = ShellLayout::of(window) == ShellLayout::Compact;
        self.compact_controls = compact;
        let custom = 1 + self.playground.scenarios.len();
        let scenario = CompactSelect::new("scenario", "Scenario")
            .options(self.scenario_names().map(SharedString::new_static))
            .selected(self.scenario.unwrap_or(custom))
            .open(self.open == Some(""))
            .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                this.open = open.then_some("");
                cx.notify();
            }))
            .on_select(cx.listener(|this, index: &usize, window, cx| {
                this.select_scenario(*index, window, cx);
            }));
        let mut body = div().flex().flex_col().child(
            div()
                .flex()
                .flex_col()
                .gap(rem(6.))
                .px(rem(20.))
                .pt(rem(16.))
                .pb(rem(16.))
                .debug_selector(|| "playground-scenario".into())
                .child(control_row(compact, control_label("Scenario"), scenario))
                .child(
                    accessible_text(
                        "scenario-supporting",
                        "The options this playground starts with.",
                    )
                    .whitespace_normal()
                    .map(|text| type_role(text, TypeRole::BodySmall))
                    .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                ),
        );
        for (index, section) in self.playground.sections.iter().enumerate() {
            body = body.child(self.render_section(index, section, cx));
        }
        body.into_any_element()
    }

    fn render_section(
        &self,
        index: usize,
        section: &'static Section,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(("playground-section", index))
            .flex()
            .flex_col()
            .gap(rem(8.))
            .px(rem(20.))
            .pt(rem(16.))
            .pb(rem(16.))
            .border_t_1()
            .border_color(color(ColorRole::OUTLINE_VARIANT))
            .child(
                // md3.io's small uppercase section heading.
                accessible_text(("section-title", index), section.title.to_uppercase())
                    .role(Role::Heading)
                    .pb(rem(4.))
                    .map(|text| type_role(text, TypeRole::LabelMedium))
                    .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
            )
            .children(
                section
                    .controls
                    .iter()
                    .map(|control| self.render_control(control, cx)),
            )
    }

    /// One md3.io configuration row: the label on the leading side and a
    /// compact control on the trailing side. A control too wide for the
    /// card wraps below its label instead of scrolling.
    fn render_control(&self, control: &'static Control, cx: &mut Context<Self>) -> AnyElement {
        let compact = self.compact_controls;
        let id = control.id();
        let label = control.label();
        let element_id = ElementId::Name(SharedString::new_static(id));
        let row = div()
            .id(element_id.clone())
            .debug_selector(move || format!("playground-control-{id}"));
        match *control {
            Control::Select { options, .. } => row
                .child(control_row(
                    compact,
                    control_label(label),
                    CompactSelect::new("select", label)
                        .options(options.iter().copied().map(SharedString::new_static))
                        .selected(self.config.choice(id))
                        .open(self.open == Some(id))
                        .on_open_change(cx.listener(move |this, open: &bool, _, cx| {
                            this.open = open.then_some(id);
                            cx.notify();
                        }))
                        .on_select(cx.listener(move |this, index: &usize, window, cx| {
                            this.open = None;
                            this.set_value(id, Value::Choice(*index), window, cx);
                            cx.notify();
                        })),
                ))
                .into_any_element(),
            Control::Segmented { options, .. } => row
                .child(control_row(
                    compact,
                    control_label(label),
                    ConnectedButtonGroup::new("segmented", label)
                        .button_size(ButtonSize::ExtraSmall)
                        .options(options.iter().map(|option| (*option, *option)))
                        .selected(options[self.config.choice(id)])
                        .on_change(cx.listener(move |this, value: &SharedString, window, cx| {
                            if let Some(index) =
                                options.iter().position(|option| *option == value.as_ref())
                            {
                                this.set_value(id, Value::Choice(index), window, cx);
                            }
                        })),
                ))
                .into_any_element(),
            Control::Icon { icons, .. } => {
                let selected = self.config.choice(id);
                row.child(control_row(
                    compact,
                    control_label(label),
                    div()
                        .id("icon-options")
                        .role(Role::RadioGroup)
                        .aria_label(label)
                        .flex()
                        .flex_wrap()
                        .justify_end()
                        .gap(rem(2.))
                        .children(icons.iter().enumerate().map(|(index, icon)| {
                            let (name, accessible) = if icon.is_empty() {
                                ("block", "None")
                            } else {
                                (*icon, *icon)
                            };
                            IconButton::new(("icon-option", index), symbol(name, 20.), accessible)
                                .variant(IconButtonVariant::Standard)
                                .button_size(ButtonSize::ExtraSmall)
                                .toggle(true)
                                .selected(index == selected)
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.set_value(id, Value::Choice(index), window, cx);
                                }))
                        })),
                ))
                .into_any_element()
            }
            Control::Switch { .. } => row
                .child(control_row(
                    compact,
                    control_label(label),
                    MaterialSwitch::new("switch", label)
                        .checked(self.config.bool(id))
                        .on_change(cx.listener(move |this, checked: &bool, window, cx| {
                            this.set_value(id, Value::Bool(*checked), window, cx);
                        })),
                ))
                .into_any_element(),
            Control::Slider { .. } => {
                let slider = self.sliders[id].clone();
                let value = self.config.number(id);
                row.flex()
                    .flex_col()
                    .gap(rem(2.))
                    .child(control_row(
                        compact,
                        control_label(label),
                        accessible_text("slider-value", format!("{value}"))
                            .map(|text| type_role(text, TypeRole::LabelLarge))
                            .text_color(color(ColorRole::ON_SURFACE_VARIANT)),
                    ))
                    .child(
                        MaterialSlider::new(&slider, label)
                            .id("slider")
                            .size(SliderSize::XSmall),
                    )
                    .into_any_element()
            }
            Control::Text { .. } => row
                .child(control_row(
                    compact,
                    control_label(label),
                    compact_input("text", label, compact, &self.texts[id]),
                ))
                .into_any_element(),
        }
    }
}

/// A configuration row: `label` leading, `control` trailing, vertically
/// centred at md3.io's 36dp row height, or a 48dp touch target's in the
/// compact layout; the control wraps below the label when the two do not fit
/// side by side.
fn control_row(compact: bool, label: impl IntoElement, control: impl IntoElement) -> Div {
    div()
        .w_full()
        .min_h(rem(if compact { TOUCH_TARGET } else { 36. }))
        .flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap_x(rem(12.))
        .gap_y(rem(4.))
        .child(div().flex_shrink_0().child(label))
        .child(div().flex_1().min_w_0().flex().justify_end().child(control))
}

fn control_label(label: &'static str) -> impl IntoElement {
    accessible_text("control-label", label)
        .map(|text| type_role(text, TypeRole::LabelLarge))
        .text_color(color(ColorRole::ON_SURFACE))
}

/// The width of compact selects and inputs in the configuration card.
pub const COMPACT_CONTROL_WIDTH: f32 = 180.;
const COMPACT_CONTROL_HEIGHT: f32 = 32.;
/// What a finger needs: the height of a compact layout's control and its row.
const TOUCH_CONTROL_HEIGHT: f32 = 40.;
const TOUCH_TARGET: f32 = 48.;

/// The compact control frame md3.io's configuration uses: a 32dp outlined
/// box with an 8dp corner, taller where the window is a phone's.
fn compact_frame(compact: bool) -> Div {
    div()
        .h(rem(if compact {
            TOUCH_CONTROL_HEIGHT
        } else {
            COMPACT_CONTROL_HEIGHT
        }))
        .px(rem(10.))
        .flex()
        .items_center()
        .gap(rem(4.))
        .rounded(rem(8.))
        .border_1()
        .border_color(color(ColorRole::OUTLINE_VARIANT))
        .bg(color(ColorRole::SURFACE_CONTAINER_LOWEST))
        .map(|frame| type_role(frame, TypeRole::BodyMedium))
        .text_color(color(ColorRole::ON_SURFACE))
}

/// A compact single-line editor on Kit's input, in the compact frame.
fn compact_input(
    id: &'static str,
    label: &'static str,
    compact: bool,
    state: &Entity<InputState>,
) -> impl IntoElement {
    use gpui_kit::component::{Sizable as _, Size, input::Input};
    compact_frame(compact)
        .id(id)
        .w(rem(COMPACT_CONTROL_WIDTH))
        .child(
            Input::new(state)
                .appearance(false)
                .with_size(Size::XSmall)
                .aria_label(label)
                .p_0()
                .w_full()
                .text_size(TypeRole::BodyMedium.scale().font_size),
        )
}

type IndexHandler = std::rc::Rc<dyn Fn(&usize, &mut Window, &mut App)>;
type OpenHandler = std::rc::Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// md3.io's compact select: the current option and an arrow in the compact
/// frame, opening a Material menu of the options (the library's
/// [`MenuPopup`], which owns the overlay, focus, keyboard and dismissal).
#[derive(gpui::IntoElement)]
pub struct CompactSelect {
    id: ElementId,
    label: SharedString,
    options: Vec<SharedString>,
    selected: usize,
    open: bool,
    width: f32,
    on_open_change: Option<OpenHandler>,
    on_select: Option<IndexHandler>,
}

impl CompactSelect {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            options: Vec::new(),
            selected: 0,
            open: false,
            width: COMPACT_CONTROL_WIDTH,
            on_open_change: None,
            on_select: None,
        }
    }
    pub fn options(mut self, options: impl IntoIterator<Item = SharedString>) -> Self {
        self.options = options.into_iter().collect();
        self
    }
    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = selected;
        self
    }
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    pub fn on_open_change(mut self, f: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(std::rc::Rc::new(f));
        self
    }
    pub fn on_select(mut self, f: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(std::rc::Rc::new(f));
        self
    }
}

impl RenderOnce for CompactSelect {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        let compact = ShellLayout::of(window) == ShellLayout::Compact;
        let value = self.options.get(self.selected).cloned().unwrap_or_default();
        let open = self.open;
        let toggle = self.on_open_change.clone();
        let on_surface = color(ColorRole::ON_SURFACE);
        let trigger = FocusRing::new(
            (self.id.clone(), "ring"),
            gpui_base::Button::new((self.id.clone(), "trigger"))
                .role(Role::ComboBox)
                .accessibility_label(self.label.clone())
                .aria_value(value.clone())
                .aria_expanded(open)
                .cursor_pointer()
                // A 48dp target around the frame.
                .when(compact, |trigger| {
                    trigger.py(rem((TOUCH_TARGET - TOUCH_CONTROL_HEIGHT) / 2.))
                })
                .child(
                    compact_frame(compact)
                        .w(rem(self.width))
                        .hover(move |style| style.bg(on_surface.opacity(0.08)))
                        .child(div().flex_1().min_w_0().truncate().child(value))
                        .child(
                            symbol("arrow_drop_down", 20.)
                                .color(color(ColorRole::ON_SURFACE_VARIANT)),
                        ),
                )
                .on_click(move |_, window, cx| {
                    if let Some(toggle) = toggle.as_ref() {
                        toggle(&!open, window, cx);
                    }
                }),
        );
        let selected = self.selected;
        let on_select = self.on_select.clone();
        let menu = MaterialMenu::new((self.id.clone(), "menu"))
            .width(self.width)
            .items(self.options.into_iter().enumerate().map(|(index, option)| {
                let on_select = on_select.clone();
                MaterialMenuItem::new(option)
                    .selected(index == selected)
                    .on_click(move |_, window, cx| {
                        if let Some(select) = on_select.as_ref() {
                            select(&index, window, cx);
                        }
                    })
            }));
        let change = self.on_open_change;
        MenuPopup::new(self.id, trigger, menu)
            .open(open)
            .on_open_change(move |next, window, cx| {
                if let Some(change) = change.as_ref() {
                    change(next, window, cx);
                }
            })
    }
}

/// The preview canvas as its own view. A running component animation,
/// hover or press invalidates the view being drawn; rendered inline, that
/// was the whole Gallery (its content list re-laid out the page's lead row
/// every animation frame). As a view, only the preview is rebuilt.
///
/// A preview says how to arrange the panes while it renders, which is after
/// the lead row around it has. The lead row must not read the playground state
/// for that, as the state changes every frame and the row would be built again
/// with it, so this view publishes the hint in a cell and is its invalidation
/// owner: a hint that changed draws the lead row and the panel again, in the
/// next frame.
///
/// The preview's scope (its own theme, mode and direction) is this view's too,
/// not the lead row's. A preview built again on its own would otherwise leave
/// the scope, which only applies while it is being drawn through.
pub struct PreviewView {
    state: Entity<PlaygroundState>,
    shared: Entity<GalleryShared>,
    hint: Rc<Cell<PaneHint>>,
    dependents: (WeakEntity<LeadView>, WeakEntity<PanelView>),
    _observe: [Subscription; 2],
}

impl PreviewView {
    pub fn new(
        state: Entity<PlaygroundState>,
        shared: Entity<GalleryShared>,
        hint: Rc<Cell<PaneHint>>,
        dependents: (WeakEntity<LeadView>, WeakEntity<PanelView>),
        cx: &mut Context<Self>,
    ) -> Self {
        let observe = [
            cx.observe(&state, |_, _, cx| cx.notify()),
            cx.observe(&shared, |_, _, cx| cx.notify()),
        ];
        Self {
            state,
            shared,
            hint,
            dependents,
            _observe: observe,
        }
    }
}

impl Render for PreviewView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let scope = self.shared.read(cx).preview_scope().resolve(window, cx);
        let state = self.state.clone();
        let (preview, hint) = scope.build(window, cx, |window, cx| {
            state.update(cx, |state, cx| {
                (state.render_preview(window, cx), state.pane_hint())
            })
        });
        if self.hint.replace(hint) != hint {
            let (lead, panel) = self.dependents.clone();
            cx.defer(move |cx| {
                let _ = lead.update(cx, |_, cx| cx.notify());
                let _ = panel.update(cx, |_, cx| cx.notify());
            });
        }
        let stacked = panes_stacked(window, hint);
        let canvas = scope.build(window, cx, |window, _| {
            preview_canvas(stacked, false, hint.flush, window, preview)
        });
        scope.child(canvas)
    }
}

/// The configuration card's body as its own view, so editing a control or
/// animating one (a switch, a segmented button) rebuilds only the panel and
/// the preview that observes the same state.
pub struct PanelView {
    state: Entity<PlaygroundState>,
    hint: Rc<Cell<PaneHint>>,
    _observe: Subscription,
}

impl PanelView {
    pub fn new(
        state: Entity<PlaygroundState>,
        hint: Rc<Cell<PaneHint>>,
        cx: &mut Context<Self>,
    ) -> Self {
        let observe = cx.observe(&state, |_, _, cx| cx.notify());
        Self {
            state,
            hint,
            _observe: observe,
        }
    }
}

impl Render for PanelView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let panel = self
            .state
            .update(cx, |state, cx| state.render_panel(window, cx));
        // Stacked, the card is as tall as its controls and the page scrolls.
        if panes_stacked(window, self.hint.get()) {
            return panel;
        }
        let scroll = self.state.read(cx).scroll.clone();
        div()
            .id("configuration-body")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .child(panel)
            .into_any_element()
    }
}

#[cfg(test)]
impl Render for PlaygroundState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .child(self.render_preview(window, cx))
            .child(self.render_panel(window, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    fn preview(
        _: &Config,
        _: &mut Preview,
        _: &mut Window,
        _: &mut Context<PlaygroundState>,
    ) -> AnyElement {
        div().into_any_element()
    }

    static TEST: Playground = Playground {
        scenarios: &[Scenario {
            name: "Tonal",
            values: &[
                ("variant", Set::Option("tonal")),
                ("label", Set::Text("Save")),
                ("disabled", Set::Bool(true)),
            ],
        }],
        sections: &[
            Section {
                title: "Appearance",
                controls: &[
                    Control::Select {
                        id: "variant",
                        label: "Variant",
                        options: &["filled", "tonal"],
                        default: 0,
                    },
                    Control::Slider {
                        id: "size",
                        label: "Size",
                        min: 0.,
                        max: 10.,
                        step: 1.,
                        default: 4.,
                    },
                ],
            },
            Section {
                title: "Content",
                controls: &[
                    Control::Text {
                        id: "label",
                        label: "Label",
                        default: "Button",
                    },
                    Control::Switch {
                        id: "disabled",
                        label: "Disabled",
                        default: false,
                    },
                ],
            },
        ],
        preview,
    };

    #[test]
    fn scenarios_apply_over_the_defaults() {
        let defaults = Config::defaults(&TEST);
        assert_eq!(defaults.option("variant"), "filled");
        assert_eq!(defaults.number("size"), 4.);
        assert_eq!(defaults.text("label"), "Button");
        assert!(!defaults.bool("disabled"));
        let tonal = Config::scenario(&TEST, &TEST.scenarios[0]);
        assert_eq!(tonal.option("variant"), "tonal");
        assert_eq!(tonal.text("label"), "Save");
        assert!(tonal.bool("disabled"));
        assert_eq!(tonal.number("size"), 4.);
    }

    #[test]
    fn values_parse_by_control_kind() {
        let control = |id| TEST.control(id).unwrap();
        assert_eq!(
            Value::parse(control("variant"), "tonal"),
            Some(Value::Choice(1))
        );
        assert_eq!(Value::parse(control("variant"), "bogus"), None);
        assert_eq!(
            Value::parse(control("disabled"), "true"),
            Some(Value::Bool(true))
        );
        assert_eq!(Value::parse(control("size"), "7"), Some(Value::Number(7.)));
        assert_eq!(Value::parse(control("size"), "70"), None);
        assert_eq!(
            Value::parse(control("label"), "Hi"),
            Some(Value::Text("Hi".into()))
        );
    }

    #[gpui::test]
    fn changes_mark_custom_and_reset_restores_defaults(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit_material::init(cx);
        });
        let (state, cx) = cx.add_window_view(|window, cx| PlaygroundState::new(&TEST, window, cx));
        cx.update(|window, cx| {
            state.update(cx, |state, cx| {
                state.set_value("label", Value::Text("Edited".into()), window, cx);
                assert_eq!(state.scenario, None);
                // The configuration card's editor follows.
                assert_eq!(state.texts["label"].read(cx).value(), "Edited");
                state.select_scenario(1, window, cx);
                assert_eq!(state.config, Config::scenario(&TEST, &TEST.scenarios[0]));
                assert_eq!(state.texts["label"].read(cx).value(), "Save");
                state.set_value("size", Value::Number(9.), window, cx);
                assert_eq!(state.sliders["size"].read(cx).value().start(), 9.);
                state.reset(window, cx);
                assert_eq!(state.config, Config::defaults(&TEST));
                assert_eq!(state.scenario, Some(0));
                assert_eq!(state.texts["label"].read(cx).value(), "Button");
                assert_eq!(state.sliders["size"].read(cx).value().start(), 4.);
            });
        });
    }
}
