//! Avatars: a person's image, monogram or placeholder in a circle.
//!
//! Geometry and color come from the Figma Material 3 Design Kit's "Avatars"
//! page (`Generic avatar` component set): a 40dp circle on
//! primary-container with on-primary-container content. The monogram is
//! title-medium (16/24, +0.1 tracking); the placeholder is the kit's person
//! glyph (28×26 in 40dp) and the selected style a 24dp check. Other sizes
//! scale every proportion with the diameter. Image avatars fill the circle
//! (cover), over the container color while the image loads.
//!
//! Avatars are not interactive: the caller wraps one in a button when it
//! acts (e.g. an account menu). The caller supplies the accessible name.
use crate::{
    color::ColorRole,
    icons::{Icon, Symbol},
    theme::color,
    tokens::{
        state,
        typography::{TypeRole, rem},
    },
};
use gpui::{prelude::*, *};

/// The kit's avatar diameter in design px.
pub const AVATAR_SIZE: f32 = 40.;

/// What an avatar shows.
// A short-lived builder value consumed by one render, like `ListLeading`;
// boxing the icon would only complicate callers.
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum AvatarContent {
    /// The kit's "Avatar" style: the person placeholder glyph.
    Placeholder,
    /// The kit's "Monogram" style: one or two initials.
    Monogram(SharedString),
    /// Any Material Symbol, drawn like the placeholder.
    Icon(Icon),
    /// A user image, cropped to the circle.
    Image(ImageSource),
}

/// A circular avatar. See the module documentation.
#[derive(gpui::IntoElement)]
pub struct Avatar {
    id: ElementId,
    label: SharedString,
    content: AvatarContent,
    size: f32,
    selected: bool,
    disabled: bool,
}

impl Avatar {
    /// A placeholder avatar named `label` for assistive technology.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            content: AvatarContent::Placeholder,
            size: AVATAR_SIZE,
            selected: false,
            disabled: false,
        }
    }
    pub fn content(mut self, content: AvatarContent) -> Self {
        self.content = content;
        self
    }
    /// Show initials (the first two characters are drawn).
    pub fn monogram(self, initials: impl Into<SharedString>) -> Self {
        self.content(AvatarContent::Monogram(initials.into()))
    }
    pub fn icon(self, icon: impl Into<Icon>) -> Self {
        self.content(AvatarContent::Icon(icon.into()))
    }
    pub fn image(self, source: impl Into<ImageSource>) -> Self {
        self.content(AvatarContent::Image(source.into()))
    }
    /// The diameter in design px (rems at the base text scale). Nonfinite or
    /// non-positive sizes use the kit's 40.
    pub fn size(mut self, size: f32) -> Self {
        self.size = if size.is_finite() && size > 0. {
            size
        } else {
            AVATAR_SIZE
        };
        self
    }
    /// The kit's "Check" style, e.g. a selected item in a list.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    /// Paint the M3 disabled container and content (on-surface at 12% and
    /// 38%). Images fade to 38%.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The circle's and the content's colors.
    pub fn colors(&self) -> (Rgba, Rgba) {
        if self.disabled {
            let on_surface = color(ColorRole::ON_SURFACE);
            (
                on_surface.opacity(state::DISABLED_CONTAINER),
                on_surface.opacity(state::DISABLED_CONTENT),
            )
        } else {
            (
                color(ColorRole::PRIMARY_CONTAINER),
                color(ColorRole::ON_PRIMARY_CONTAINER),
            )
        }
    }
}

/// The initials a monogram draws: at most two characters, uppercased.
pub fn monogram_initials(text: &str) -> SharedString {
    text.trim()
        .chars()
        .filter(|c| !c.is_whitespace())
        .take(2)
        .flat_map(char::to_uppercase)
        .collect::<String>()
        .into()
}

fn symbol(name: &str) -> Icon {
    Icon::from(Symbol::named(name).expect("a Material Symbol in the pinned catalog"))
}

impl RenderOnce for Avatar {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (container, ink) = self.colors();
        let size = self.size;
        // Every proportion scales with the diameter from the 40dp kit frame.
        let scale = size / AVATAR_SIZE;
        let glyph = |icon: Icon, design: f32| icon.size(rem(design * scale)).color(ink);
        let content: AnyElement = if self.selected {
            glyph(symbol("check"), 24.).into_any_element()
        } else {
            match self.content {
                AvatarContent::Placeholder => glyph(symbol("person"), 28.).into_any_element(),
                AvatarContent::Icon(icon) => glyph(icon, 24.).into_any_element(),
                AvatarContent::Monogram(text) => {
                    let type_scale = TypeRole::TitleMedium.scale();
                    div()
                        .font_family(type_scale.font_family)
                        .font_weight(FontWeight(type_scale.font_weight as f32))
                        .text_size(rem(16. * scale))
                        .line_height(rem(24. * scale))
                        .text_color(ink)
                        .child(monogram_initials(&text))
                        .into_any_element()
                }
                AvatarContent::Image(source) => img(source)
                    .size_full()
                    .object_fit(ObjectFit::Cover)
                    .rounded_full()
                    .when(self.disabled, |image| {
                        image.opacity(state::DISABLED_CONTENT)
                    })
                    .into_any_element(),
            }
        };
        div()
            .id(self.id)
            .role(Role::Image)
            .aria_label(self.label)
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .size(rem(size))
            .rounded_full()
            .overflow_hidden()
            .bg(container)
            .child(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn monograms_draw_two_uppercase_initials() {
        assert_eq!(monogram_initials("a"), "A");
        assert_eq!(monogram_initials(" jd "), "JD");
        assert_eq!(monogram_initials("j d x"), "JD");
        assert_eq!(monogram_initials("ßa"), "SSA");
        assert_eq!(monogram_initials(""), "");
    }

    #[test]
    fn sizes_fall_back_to_the_kit_diameter() {
        assert_eq!(Avatar::new("a", "A").size, AVATAR_SIZE);
        assert_eq!(Avatar::new("a", "A").size(56.).size, 56.);
        assert_eq!(Avatar::new("a", "A").size(0.).size, AVATAR_SIZE);
        assert_eq!(Avatar::new("a", "A").size(f32::NAN).size, AVATAR_SIZE);
    }

    #[test]
    fn colors_follow_primary_container_and_disabled_roles() {
        let (container, ink) = Avatar::new("a", "A").colors();
        assert_eq!(container, color(ColorRole::PRIMARY_CONTAINER));
        assert_eq!(ink, color(ColorRole::ON_PRIMARY_CONTAINER));
        let (container, ink) = Avatar::new("a", "A").disabled(true).colors();
        let on_surface = color(ColorRole::ON_SURFACE);
        assert_eq!(container, on_surface.opacity(state::DISABLED_CONTAINER));
        assert_eq!(ink, on_surface.opacity(state::DISABLED_CONTENT));
    }
}
