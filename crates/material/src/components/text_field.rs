//! M3 text fields: filled and outlined.
//!
//! Reference: <https://m3.material.io/components/text-fields/specs> and the
//! Material 3 Design Kit "Text fields" page. Editing, selection, clipboard,
//! IME, the context menu and text accessibility belong to Kit's
//! `gpui_kit::component::input::Input` (single line) and `Textarea` (multi
//! line), rendered without their own chrome (`appearance(false)`). This module
//! owns the Material container, active indicator or outline (with the notch
//! the floating label opens), the label's resting ↔ floating motion, icons,
//! prefix/suffix, supporting text and the character counter.
//!
//! The label floats on the fast spatial token and its color, the indicator
//! weight and the notch follow focus on the fast effects token, so both
//! retarget from wherever they are when focus changes again and honor reduced
//! motion through [`crate::motion`].
use super::InteractionPreview;
use super::labeled::required_label;
use crate::{
    color::ColorRole,
    motion::{self, MotionToken},
    theme::{blend, color},
    tokens::{
        shape::RADIUS_EXTRA_SMALL,
        state,
        typography::{TypeRole, rem},
    },
};
use gpui::{prelude::*, *};
use gpui_base::input::{InputBaseState, InputModeKind, InputState, TextareaState};
use gpui_kit::component::{
    Sizable as _, Size,
    input::{Input, Textarea},
};

/// Container height of a single-line field.
pub const TEXT_FIELD_HEIGHT: f32 = 56.;
/// The default specimen width of the design kit.
pub const TEXT_FIELD_WIDTH: f32 = 210.;
/// Horizontal content padding without an icon on that side.
const PADDING: f32 = 16.;
/// Horizontal padding beside a leading or trailing icon.
const ICON_PADDING: f32 = 12.;
const ICON_SIZE: f32 = 24.;
/// Gap between an icon and the text.
const ICON_GAP: f32 = 16.;
/// The floating label's top inside a filled container.
const FILLED_FLOAT_TOP: f32 = 8.;
/// The input line's top in a filled field with a label.
const FILLED_INPUT_TOP: f32 = 24.;
/// Space the outline notch keeps on each side of the floating label.
const NOTCH_PADDING: f32 = 4.;
/// The focused indicator and outline weight (the design kit's 3dp stroke).
const FOCUS_INDICATOR: f32 = 3.;

/// md3.io's text field densities. The design kit draws one 56dp field;
/// compact is density −2 (48dp, the M3 density scale's 4dp steps, as the
/// baseline menu's compact items), with the filled label and input moved up
/// 4dp each so the floating label keeps its 4dp gap to the input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextFieldDensity {
    #[default]
    Default,
    Compact,
}

impl TextFieldDensity {
    pub const ALL: [Self; 2] = [Self::Default, Self::Compact];
    /// The single-line container height in dp.
    pub const fn height(self) -> f32 {
        match self {
            Self::Default => TEXT_FIELD_HEIGHT,
            Self::Compact => TEXT_FIELD_HEIGHT - 8.,
        }
    }
    /// How far the filled label and input move up from their 56dp places.
    const fn lift(self) -> f32 {
        (TEXT_FIELD_HEIGHT - self.height()) / 2.
    }
}

/// The two M3 text field styles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextFieldVariant {
    #[default]
    Filled,
    Outlined,
}

impl TextFieldVariant {
    pub const ALL: [Self; 2] = [Self::Filled, Self::Outlined];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Filled => "Filled",
            Self::Outlined => "Outlined",
        }
    }
}

/// What a single-line field edits: md3.io's type options. The type masks a
/// password and filters what can be typed; platforms without a soft keyboard
/// have nothing further to switch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextFieldInputType {
    #[default]
    Text,
    Password,
    Email,
    Number,
    Tel,
    Url,
}

impl TextFieldInputType {
    pub const ALL: [Self; 6] = [
        Self::Text,
        Self::Password,
        Self::Email,
        Self::Number,
        Self::Tel,
        Self::Url,
    ];
    /// Whether the value is painted as bullets and kept off the clipboard.
    pub const fn masked(self) -> bool {
        matches!(self, Self::Password)
    }
    /// Whether `text` is an acceptable value or the start of one. Partial
    /// input passes ("-", "1.", "1e-"), so completeness stays the form's check.
    pub fn accepts(self, text: &str) -> bool {
        match self {
            Self::Text | Self::Password => true,
            Self::Email | Self::Url => !text.chars().any(char::is_whitespace),
            Self::Tel => text
                .chars()
                .all(|c| c.is_ascii_digit() || "+-().#* ".contains(c)),
            Self::Number => {
                let body = text.strip_prefix(['-', '+']).unwrap_or(text);
                let (mantissa, exponent) = match body.split_once(['e', 'E']) {
                    Some((mantissa, exponent)) => (mantissa, Some(exponent)),
                    None => (body, None),
                };
                let digits = |part: &str| part.chars().all(|c| c.is_ascii_digit());
                let mantissa_ok = match mantissa.split_once('.') {
                    Some((whole, fraction)) => digits(whole) && digits(fraction),
                    None => digits(mantissa),
                };
                let exponent_ok = exponent.is_none_or(|exponent| {
                    !mantissa.is_empty()
                        && digits(exponent.strip_prefix(['-', '+']).unwrap_or(exponent))
                });
                mantissa_ok && exponent_ok
            }
        }
    }
}

enum FieldInput {
    Single(Entity<InputState>),
    Multi(Entity<TextareaState>),
    /// A specimen's fixed value, painted where the editor paints its text.
    Static(SharedString),
}

/// An M3 text field over a Kit input state.
#[derive(gpui::IntoElement)]
pub struct TextField {
    id: ElementId,
    input: FieldInput,
    variant: TextFieldVariant,
    density: TextFieldDensity,
    label: Option<SharedString>,
    placeholder: Option<SharedString>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    prefix: Option<SharedString>,
    suffix: Option<SharedString>,
    supporting: Option<SharedString>,
    max_length: Option<usize>,
    error: bool,
    disabled: bool,
    read_only: bool,
    required: bool,
    input_type: Option<TextFieldInputType>,
    float_label: bool,
    preview: Option<InteractionPreview>,
    style: StyleRefinement,
}

impl TextField {
    /// A single-line field editing `state`.
    pub fn new(id: impl Into<ElementId>, state: &Entity<InputState>) -> Self {
        Self::with_input(id, FieldInput::Single(state.clone()))
    }

    /// A multi-line field editing `state`; it grows with its rows.
    pub fn multi_line(id: impl Into<ElementId>, state: &Entity<TextareaState>) -> Self {
        Self::with_input(id, FieldInput::Multi(state.clone()))
    }

    /// A non-editable specimen showing `value` (empty for none), for boards
    /// that render many fields in forced states without an editor each.
    pub fn specimen(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self::with_input(id, FieldInput::Static(value.into()))
    }

    fn with_input(id: impl Into<ElementId>, input: FieldInput) -> Self {
        Self {
            id: id.into(),
            input,
            variant: TextFieldVariant::Filled,
            density: TextFieldDensity::Default,
            label: None,
            placeholder: None,
            leading: None,
            trailing: None,
            prefix: None,
            suffix: None,
            supporting: None,
            max_length: None,
            error: false,
            disabled: false,
            read_only: false,
            required: false,
            input_type: None,
            float_label: false,
            preview: None,
            style: StyleRefinement::default(),
        }
    }

    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }
    /// The container density (default 56dp; compact 48dp).
    pub fn density(mut self, density: TextFieldDensity) -> Self {
        self.density = density;
        self
    }
    pub fn outlined(self) -> Self {
        self.variant(TextFieldVariant::Outlined)
    }
    /// The label: inside the field at rest, floating above the text while
    /// the field is focused or populated. It is also the accessible name.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// Hint text shown in an empty field while its label floats (or when it
    /// has no label).
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }
    /// Keep the label floating even when empty and unfocused, so the
    /// placeholder shows at rest (the design kit's "Placeholder text").
    pub fn float_label(mut self, float: bool) -> Self {
        self.float_label = float;
        self
    }
    /// A 24dp leading icon; it inherits the field's icon color.
    pub fn leading_icon(mut self, icon: impl IntoElement) -> Self {
        self.leading = Some(icon.into_any_element());
        self
    }
    /// A 24dp trailing icon or icon button; painted in the error color while
    /// the field is in error.
    pub fn trailing_icon(mut self, icon: impl IntoElement) -> Self {
        self.trailing = Some(icon.into_any_element());
        self
    }
    /// Text before the input (e.g. a currency sign), shown with the input.
    pub fn prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }
    /// Text after the input (e.g. a unit), shown with the input.
    pub fn suffix(mut self, suffix: impl Into<SharedString>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }
    /// Supporting text below the field; with [`Self::error`] it is the
    /// error message.
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }
    /// Show a `count/max` character counter at the end of the supporting row.
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// The value can be focused, selected and copied but not edited. The
    /// field keeps its enabled appearance.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    /// Mark the field as required: the label and accessible name gain the
    /// required mark. Enforcing it belongs to the form that owns the value.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
    /// Mask and filter a single-line field for this type. This replaces the
    /// state's validator; leave it unset to keep a caller's own.
    pub fn input_type(mut self, input_type: TextFieldInputType) -> Self {
        self.input_type = Some(input_type);
        self
    }
    /// Render a static specimen in one forced interaction state (Enabled,
    /// Hovered, Focused or Disabled; Pressed paints as Focused, which is
    /// what pressing a field produces). See [`InteractionPreview`].
    pub fn preview_state(mut self, state: InteractionPreview) -> Self {
        self.preview = Some(state);
        self
    }
}

impl Styled for TextField {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// Which visual state the field paints.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct FieldState {
    pub focused: bool,
    pub hovered: bool,
    pub error: bool,
    pub disabled: bool,
}

/// The roles a field paints in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FieldPaint {
    pub container: Rgba,
    /// Filled: the active indicator. Outlined: the outline.
    pub indicator: Rgba,
    pub indicator_width: f32,
    pub label: Rgba,
    pub input: Rgba,
    pub placeholder: Rgba,
    pub leading: Rgba,
    pub trailing: Rgba,
    pub supporting: Rgba,
    pub state_layer: Rgba,
    /// The text caret (`md.comp.*-text-field.caret.color`, error in error).
    pub caret: Rgba,
}

fn alpha(mut c: Rgba, a: f32) -> Rgba {
    c.a *= a;
    c
}

/// M3 text field colors (`md.comp.{filled,outlined}-text-field.*`).
pub(crate) fn field_paint(variant: TextFieldVariant, s: FieldState) -> FieldPaint {
    use ColorRole as C;
    let filled = variant == TextFieldVariant::Filled;
    let on_surface = color(C::ON_SURFACE);
    let variant_ink = color(C::ON_SURFACE_VARIANT);
    if s.disabled {
        let content = state::disabled(on_surface);
        return FieldPaint {
            container: if filled {
                alpha(on_surface, 0.04)
            } else {
                alpha(on_surface, 0.)
            },
            indicator: if filled {
                content
            } else {
                alpha(on_surface, state::DISABLED_CONTAINER)
            },
            indicator_width: 1.,
            label: content,
            input: content,
            placeholder: content,
            leading: content,
            trailing: content,
            supporting: content,
            state_layer: alpha(on_surface, 0.),
            caret: color(C::PRIMARY),
        };
    }
    let (rest, hover) = if filled {
        (variant_ink, on_surface)
    } else {
        (color(C::OUTLINE), on_surface)
    };
    let indicator = match (s.error, s.focused, s.hovered) {
        (true, false, true) => color(C::ON_ERROR_CONTAINER),
        (true, _, _) => color(C::ERROR),
        (false, true, _) => color(C::PRIMARY),
        (false, false, true) => hover,
        _ => rest,
    };
    let label = match (s.error, s.focused, s.hovered) {
        (true, false, true) => color(C::ON_ERROR_CONTAINER),
        (true, _, _) => color(C::ERROR),
        (false, true, _) => color(C::PRIMARY),
        (false, false, true) if !filled => on_surface,
        _ => variant_ink,
    };
    FieldPaint {
        container: if filled {
            color(C::SURFACE_CONTAINER_HIGHEST)
        } else {
            alpha(on_surface, 0.)
        },
        indicator,
        indicator_width: if s.focused { FOCUS_INDICATOR } else { 1. },
        label,
        input: on_surface,
        placeholder: variant_ink,
        leading: variant_ink,
        trailing: match (s.error, s.hovered && !s.focused) {
            (true, true) => color(C::ON_ERROR_CONTAINER),
            (true, false) => color(C::ERROR),
            _ => variant_ink,
        },
        supporting: if s.error {
            color(C::ERROR)
        } else {
            variant_ink
        },
        state_layer: if filled && s.hovered {
            alpha(on_surface, state::HOVER)
        } else {
            alpha(on_surface, 0.)
        },
        caret: if s.error {
            color(C::ERROR)
        } else {
            color(C::PRIMARY)
        },
    }
}

/// Label geometry in design px: `(left, top, font size, line height)` for a
/// float progress `t` (0 resting, 1 floating).
#[cfg(test)]
pub(crate) fn label_geometry(
    variant: TextFieldVariant,
    leading: bool,
    t: f32,
) -> (f32, f32, f32, f32) {
    density_label_geometry(variant, TextFieldDensity::Default, leading, t)
}

/// The label geometry (see `label_geometry`) in a field of `density`.
pub(crate) fn density_label_geometry(
    variant: TextFieldVariant,
    density: TextFieldDensity,
    leading: bool,
    t: f32,
) -> (f32, f32, f32, f32) {
    let rest_left = if leading {
        ICON_PADDING + ICON_SIZE + ICON_GAP
    } else {
        PADDING
    };
    let (float_left, float_top) = match variant {
        TextFieldVariant::Filled => (rest_left, FILLED_FLOAT_TOP - density.lift()),
        // Centered on the top outline, at the content's leading edge.
        TextFieldVariant::Outlined => (PADDING, -8.),
    };
    let rest_top = (density.height() - 24.) / 2.;
    let large = TypeRole::BodyLarge.scale();
    let small = TypeRole::BodySmall.scale();
    let lerp = |a: f32, b: f32| a + (b - a) * t;
    (
        lerp(rest_left, float_left),
        lerp(rest_top, float_top),
        lerp(large.font_size.0, small.font_size.0) * 14.,
        lerp(large.line_height.0, small.line_height.0) * 14.,
    )
}

/// The outline's notch `(start, width)` in design px for a label `label_px`
/// wide at the floating size, opened by `t`.
pub(crate) fn notch(label_px: f32, t: f32) -> (f32, f32) {
    let t = t.clamp(0., 1.);
    let full = label_px + 2. * NOTCH_PADDING;
    let center = PADDING - NOTCH_PADDING + full / 2.;
    let width = full * t;
    (center - width / 2., width)
}

/// Apply an M3 type role's family, size, line height and weight.
pub(crate) fn type_style<E: Styled>(element: E, role: TypeRole) -> E {
    let scale = role.scale();
    element
        .font_family(scale.font_family)
        .text_size(scale.font_size)
        .line_height(scale.line_height)
        .font_weight(FontWeight(scale.font_weight as f32))
}

fn read_value<M: InputModeKind>(state: &Entity<InputBaseState<M>>, cx: &App) -> SharedString {
    state.read(cx).value()
}

fn focus_handle<M: InputModeKind>(state: &Entity<InputBaseState<M>>, cx: &App) -> FocusHandle {
    state.read(cx).focus_handle(cx)
}

/// The width of `text` at the floating label's size, in design px.
fn label_width(text: &SharedString, window: &Window) -> f32 {
    let scale = TypeRole::BodySmall.scale();
    let mut style = window.text_style();
    style.font_family = scale.font_family.into();
    style.font_weight = FontWeight(scale.font_weight as f32);
    let rem = window.rem_size();
    let size = rem * scale.font_size.0;
    let run = style.to_run(text.len());
    let line = window
        .text_system()
        .shape_line(text.clone(), size, &[run], None);
    // Back to design px at the base rem, so it composes with `rem()`.
    line.width.as_f32() / rem.as_f32() * 14.
}

/// Bring a live input into line with `input_type`, once per change of type.
fn apply_input_type(
    id: &ElementId,
    state: &Entity<InputState>,
    input_type: TextFieldInputType,
    window: &mut Window,
    cx: &mut App,
) {
    let applied = window.use_keyed_state((id.clone(), "input-type"), cx, |_, _| None);
    if *applied.read(cx) == Some(input_type) {
        return;
    }
    applied.update(cx, |applied, _| *applied = Some(input_type));
    state.update(cx, |state, cx| {
        state.set_masked(input_type.masked(), window, cx);
        state.set_validator(move |text, _| input_type.accepts(text), cx);
    });
}

impl RenderOnce for TextField {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        if self.required {
            self.label = self.label.map(|label| required_label(&label, true));
        }
        if let (FieldInput::Single(state), Some(input_type)) = (&self.input, self.input_type) {
            apply_input_type(&id, state, input_type, window, cx);
        }
        let (value, focus) = match &self.input {
            FieldInput::Single(state) => (read_value(state, cx), Some(focus_handle(state, cx))),
            FieldInput::Multi(state) => (read_value(state, cx), Some(focus_handle(state, cx))),
            FieldInput::Static(value) => (value.clone(), None),
        };
        let multi_line = matches!(self.input, FieldInput::Multi(_));
        let preview = self.preview;
        let disabled = self.disabled || preview.is_some_and(InteractionPreview::is_disabled);
        let focused = match preview {
            Some(p) => p.is_focused() || p.is_pressed(),
            None => !disabled && focus.as_ref().is_some_and(|f| f.is_focused(window)),
        };
        let hovered_preview = preview.is_some_and(InteractionPreview::is_hovered);
        let populated = !value.is_empty();
        let has_label = self.label.is_some();
        let floating = focused || populated || self.float_label || !has_label;
        let variant = self.variant;
        let filled = variant == TextFieldVariant::Filled;
        let leading = self.leading.is_some();
        let density = self.density;
        let height = density.height();

        // Motion: float on the fast spatial token, focus emphasis on fast effects.
        let float_t = motion::token_value(
            (id.clone(), "label-float"),
            if floating { 1. } else { 0. },
            MotionToken::FastSpatial,
            window,
            cx,
        );
        let focus_t = motion::token_value(
            (id.clone(), "focus"),
            if focused { 1. } else { 0. },
            MotionToken::FastEffects,
            window,
            cx,
        )
        .clamp(0., 1.);

        let base = FieldState {
            focused: false,
            hovered: hovered_preview,
            error: self.error,
            disabled,
        };
        let rest = field_paint(variant, base);
        let active = field_paint(
            variant,
            FieldState {
                focused: true,
                ..base
            },
        );
        let mix = |a: Rgba, b: Rgba| blend(a, b, focus_t);
        let indicator = mix(rest.indicator, active.indicator);
        // The outline's weight steps with focus; its color crossfades.
        let outline_width = if focused {
            active.indicator_width
        } else {
            rest.indicator_width
        };
        let label_color = mix(rest.label, active.label);
        let hover_paint = field_paint(
            variant,
            FieldState {
                hovered: true,
                ..base
            },
        );
        let live_hover = preview.is_none() && !disabled && !focused;
        // Group hover resolves to the nearest painting ancestor of this name,
        // so a static name stays scoped to this field without formatting one
        // per frame.
        let group = SharedString::new_static("material-text-field");

        let mut container = div()
            .id(id.clone())
            .group(group.clone())
            .relative()
            .flex()
            .w_full()
            .min_h(rem(height))
            .when(!multi_line, |this| this.h(rem(height)))
            .when(multi_line, |this| this.items_start())
            .when(!multi_line, |this| this.items_center())
            .map(|this| match variant {
                TextFieldVariant::Filled => this
                    .rounded_t(rem(RADIUS_EXTRA_SMALL.as_f32()))
                    .bg(rest.container),
                TextFieldVariant::Outlined => this.rounded(rem(RADIUS_EXTRA_SMALL.as_f32())),
            })
            .when(!disabled, |this| this.cursor_text());
        if let Some(focus) = focus.clone().filter(|_| !disabled && preview.is_none()) {
            container = container.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                focus.focus(window, cx);
            });
        }

        // The filled hover state layer.
        if filled {
            container = container.child(
                div()
                    .absolute()
                    .inset_0()
                    .rounded_t(rem(RADIUS_EXTRA_SMALL.as_f32()))
                    .bg(rest.state_layer)
                    .when(live_hover, |this| {
                        let layer = hover_paint.state_layer;
                        this.group_hover(group.clone(), move |s| s.bg(layer))
                    }),
            );
            // The resting 1dp indicator, and the 3dp focus indicator fading
            // in over it (material-web's two-layer active indicator).
            container = container
                .child(
                    div()
                        .debug_selector(|| "text-field-indicator".into())
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .h(px(rest.indicator_width))
                        .bg(rest.indicator)
                        .when(live_hover, |this| {
                            let ink = hover_paint.indicator;
                            this.group_hover(group.clone(), move |s| s.bg(ink))
                        }),
                )
                .when(focus_t > 0., |this| {
                    this.child(
                        div()
                            .debug_selector(|| "text-field-focus-indicator".into())
                            .absolute()
                            .left_0()
                            .right_0()
                            .bottom_0()
                            .h(px(active.indicator_width))
                            .bg(active.indicator)
                            .opacity(focus_t),
                    )
                });
        } else {
            let label_px = self
                .label
                .as_ref()
                .map(|label| label_width(label, window))
                .unwrap_or(0.);
            let (start, width) = if has_label {
                notch(label_px, float_t)
            } else {
                (0., 0.)
            };
            container = container.children(outline(
                start,
                width,
                indicator,
                outline_width,
                live_hover.then_some((group.clone(), hover_paint.indicator)),
            ));
        }

        let pad_left = if leading { ICON_PADDING } else { PADDING };
        let pad_right = if self.trailing.is_some() {
            ICON_PADDING
        } else {
            PADDING
        };
        let input_top = if filled && has_label {
            FILLED_INPUT_TOP - density.lift()
        } else {
            (height - 24.) / 2.
        };
        let input_bottom = height - 24. - input_top;
        let show_content = floating || !has_label;
        let show_placeholder = !populated && show_content;

        let icon_slot = |icon: AnyElement, ink: Rgba| {
            div()
                .flex()
                .flex_shrink_0()
                .items_center()
                .justify_center()
                .size(rem(ICON_SIZE))
                .when(multi_line, |this| this.mt(rem((height - 24.) / 2.)))
                .text_color(ink)
                .child(icon)
        };
        let affix = |text: SharedString| {
            type_style(div().flex_shrink_0(), TypeRole::BodyLarge)
                .text_color(rest.placeholder)
                .child(text)
        };
        // Kit's input paints the theme caret (primary through the theme
        // bridge); the field's own caret follows its error state.
        let caret = Some(Hsla::from(active.caret));
        match &self.input {
            FieldInput::Single(state) => state.update(cx, |s, _| s.set_caret_color(caret)),
            FieldInput::Multi(state) => state.update(cx, |s, _| s.set_caret_color(caret)),
            FieldInput::Static(_) => {}
        }
        // Kit's editors align physically; they do not mirror with layout.
        let rtl = crate::scope::is_rtl(window);
        let editor: AnyElement = match &self.input {
            FieldInput::Single(state) => Input::new(state)
                .appearance(false)
                .with_size(Size::XSmall)
                .disabled(disabled)
                .readonly(self.read_only)
                .when(rtl, |this| this.text_right())
                .when_some(self.label.clone(), |this, label| this.aria_label(label))
                .p_0()
                .h(rem(24.))
                .text_size(TypeRole::BodyLarge.scale().font_size)
                .line_height(TypeRole::BodyLarge.scale().line_height)
                .into_any_element(),
            FieldInput::Multi(state) => Textarea::new(state)
                .appearance(false)
                .with_size(Size::XSmall)
                .disabled(disabled)
                .readonly(self.read_only)
                .when(rtl, |this| this.text_right())
                .when_some(self.label.clone(), |this, label| this.aria_label(label))
                .p_0()
                // Kit insets multi-line text by the XSmall input padding.
                .mx(px(-4.))
                .text_size(TypeRole::BodyLarge.scale().font_size)
                .line_height(TypeRole::BodyLarge.scale().line_height)
                .into_any_element(),
            FieldInput::Static(value) => div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .child(if self.input_type.is_some_and(TextFieldInputType::masked) {
                    SharedString::from("\u{2022}".repeat(value.chars().count()))
                } else {
                    value.clone()
                })
                .into_any_element(),
        };
        let placeholder = self.placeholder.clone().filter(|_| show_placeholder);
        let input_row = div()
            .relative()
            .flex()
            .flex_1()
            .min_w_0()
            .when(!multi_line, |this| this.items_center().h(rem(24.)))
            .when(multi_line, |this| this.items_start())
            .gap(rem(2.))
            .map(|this| type_style(this, TypeRole::BodyLarge))
            .text_color(rest.input)
            .children(self.prefix.clone().filter(|_| show_content).map(affix))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_w_0()
                    // Kit paints the editor's text in its own foreground, so
                    // a disabled editor fades as a whole: on-surface at 38%.
                    .child(
                        div()
                            .when(disabled, |this| {
                                this.text_color(color(ColorRole::ON_SURFACE))
                                    .opacity(state::DISABLED_CONTENT)
                            })
                            .child(editor),
                    )
                    .when_some(placeholder, |this, placeholder| {
                        this.child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .right_0()
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_color(rest.placeholder)
                                .child(placeholder),
                        )
                    }),
            )
            .children(self.suffix.clone().filter(|_| show_content).map(affix));

        container = container.child(
            div()
                .relative()
                .flex()
                .w_full()
                .when(!multi_line, |this| this.items_center().h_full())
                .when(multi_line, |this| this.items_start())
                .pl(rem(pad_left))
                .pr(rem(pad_right))
                .gap(rem(ICON_GAP))
                .children(self.leading.map(|icon| icon_slot(icon, rest.leading)))
                .child(
                    div()
                        .flex()
                        .flex_1()
                        .min_w_0()
                        .pt(rem(input_top))
                        .pb(rem(input_bottom))
                        .child(input_row),
                )
                .children(self.trailing.map(|icon| {
                    icon_slot(icon, mix(rest.trailing, active.trailing)).when(live_hover, |this| {
                        let ink = hover_paint.trailing;
                        this.group_hover(group.clone(), move |s| s.text_color(ink))
                    })
                })),
        );

        if let Some(label) = self.label.clone() {
            let (left, top, size, line) =
                density_label_geometry(variant, density, leading, float_t);
            container = container.child(
                div()
                    .debug_selector(|| "text-field-label".into())
                    .absolute()
                    .left(rem(left))
                    .top(rem(top))
                    .max_w(relative(1.))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .font_family(TypeRole::BodyLarge.scale().font_family)
                    .text_size(rem(size))
                    .line_height(rem(line))
                    .text_color(label_color)
                    // An error field's label hovers to on-error-container,
                    // as its hovered preview paints.
                    .when(live_hover, |this| {
                        let ink = hover_paint.label;
                        this.group_hover(group.clone(), move |s| s.text_color(ink))
                    })
                    .child(label),
            );
        }

        let counter = self
            .max_length
            .map(|max| SharedString::from(format!("{}/{max}", value.chars().count())));
        let supporting_row = (self.supporting.is_some() || counter.is_some()).then(|| {
            type_style(div(), TypeRole::BodySmall)
                .flex()
                .justify_between()
                .gap(rem(16.))
                .px(rem(PADDING))
                .pt(rem(4.))
                .text_color(rest.supporting)
                .child(
                    div()
                        .debug_selector(|| "text-field-supporting".into())
                        .min_w_0()
                        .flex_1()
                        .children(self.supporting.clone()),
                )
                .children(counter.map(|counter| div().flex_shrink_0().child(counter)))
        });

        let mut outer = div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .w(rem(TEXT_FIELD_WIDTH))
            .child(container)
            .children(supporting_row);
        outer.style().refine(&self.style);
        outer
    }
}

/// The outline as three borders that leave the notch open: the leading part
/// up to the notch, the trailing part after it, and the bottom edge under it.
fn outline(
    start: f32,
    width: f32,
    ink: Rgba,
    weight: f32,
    hover: Option<(SharedString, Rgba)>,
) -> Vec<AnyElement> {
    let radius = rem(RADIUS_EXTRA_SMALL.as_f32());
    let weight = px(weight);
    let outline_hover = hover.clone();
    let hovering = move |el: Div| match outline_hover.clone() {
        Some((group, hover)) => el.group_hover(group, move |s| s.border_color(hover)),
        None => el,
    };
    let fill_hover = {
        let hover = hover.clone();
        move |el: Div| match hover.clone() {
            Some((group, hover)) => el.group_hover(group, move |s| s.bg(hover)),
            None => el,
        }
    };
    let start_piece = div()
        .debug_selector(|| "text-field-outline-start".into())
        .absolute()
        .top_0()
        .bottom_0()
        .left_0()
        .border_l(weight)
        .border_t(weight)
        .border_b(weight)
        .border_color(ink);
    if width <= 0. {
        // A whole outline: one rounded border.
        return vec![
            hovering(start_piece.right_0().border_r(weight).rounded(radius)).into_any_element(),
        ];
    }
    vec![
        hovering(start_piece.w(rem(start)).rounded_l(radius)).into_any_element(),
        hovering(
            div()
                .debug_selector(|| "text-field-outline-end".into())
                .absolute()
                .top_0()
                .bottom_0()
                .right_0()
                .left(rem(start + width))
                .border_r(weight)
                .border_t(weight)
                .border_b(weight)
                .rounded_r(radius)
                .border_color(ink),
        )
        .into_any_element(),
        fill_hover(
            div()
                .absolute()
                .bottom_0()
                .left(rem(start))
                .w(rem(width))
                .h(weight)
                .bg(ink),
        )
        .into_any_element(),
    ]
}

#[cfg(test)]
#[path = "text_field_tests.rs"]
mod tests;
