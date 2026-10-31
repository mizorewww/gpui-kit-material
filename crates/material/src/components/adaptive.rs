//! Window size classes and a size-aware layout element, shared by the
//! navigation suite and the canonical layouts.
//!
//! Source: neither the Figma kit nor md3.io defines size classes, so the
//! breakpoints are m3e's (`packages/m3e-motion/src/window-size-class.ts`,
//! from androidx `WindowSizeClass` at
//! `19991f2c0858aadfcb442ce02b2c8e4743c96898`, Apache-2.0: `LICENSE-androidx` in
//! this directory). Width: 0 / 600 / 840 / 1200 / 1600 dp. Height: 0 / 480 / 900 dp. A logical GPUI pixel is a dp here, and
//! the classes ignore the text scale: they describe the window, not its text.
use gpui::{prelude::*, *};

/// A width bucket, smallest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WindowWidthClass {
    /// Below 600dp.
    Compact,
    /// 600dp to 839dp.
    Medium,
    /// 840dp to 1199dp.
    Expanded,
    /// 1200dp to 1599dp.
    Large,
    /// 1600dp and wider.
    ExtraLarge,
}

impl WindowWidthClass {
    /// The class's lower bound in dp.
    pub const fn min_width(self) -> f32 {
        match self {
            Self::Compact => 0.,
            Self::Medium => 600.,
            Self::Expanded => 840.,
            Self::Large => 1200.,
            Self::ExtraLarge => 1600.,
        }
    }

    /// The class of a width in dp. A non-finite width is compact.
    pub fn of(width: f32) -> Self {
        if width >= 1600. {
            Self::ExtraLarge
        } else if width >= 1200. {
            Self::Large
        } else if width >= 840. {
            Self::Expanded
        } else if width >= 600. {
            Self::Medium
        } else {
            Self::Compact
        }
    }
}

/// A height bucket, smallest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WindowHeightClass {
    /// Below 480dp.
    Compact,
    /// 480dp to 899dp.
    Medium,
    /// 900dp and taller.
    Expanded,
}

impl WindowHeightClass {
    /// The class of a height in dp. A non-finite height is compact.
    pub fn of(height: f32) -> Self {
        if height >= 900. {
            Self::Expanded
        } else if height >= 480. {
            Self::Medium
        } else {
            Self::Compact
        }
    }
}

/// The size class of a window or container.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowSizeClass {
    /// The width bucket.
    pub width: WindowWidthClass,
    /// The height bucket.
    pub height: WindowHeightClass,
}

impl WindowSizeClass {
    /// The class of a `width` x `height` dp area.
    pub fn of(width: f32, height: f32) -> Self {
        Self {
            width: WindowWidthClass::of(width),
            height: WindowHeightClass::of(height),
        }
    }

    /// The class of a measured size.
    pub fn of_size(size: Size<Pixels>) -> Self {
        Self::of(f32::from(size.width), f32::from(size.height))
    }

    /// The class of the window's content area.
    pub fn of_window(window: &Window) -> Self {
        Self::of_size(window.viewport_size())
    }
}

type Builder = Box<dyn FnOnce(Bounds<Pixels>, &mut Window, &mut App) -> AnyElement>;

/// An element that fills its parent and builds its content once its size is
/// known, in the same frame.
///
/// GPUI has no container queries, and `NavigationBar::adaptive`'s measure and
/// redraw costs a frame at the wrong layout. Building during prepaint avoids
/// both. The parent must give it a definite size: its height never comes from
/// its content.
pub struct SizedContent {
    id: ElementId,
    builder: Option<Builder>,
}

/// Build content from the bounds this element is given. See [`SizedContent`].
pub fn sized_content(
    id: impl Into<ElementId>,
    builder: impl FnOnce(Bounds<Pixels>, &mut Window, &mut App) -> AnyElement + 'static,
) -> SizedContent {
    SizedContent {
        id: id.into(),
        builder: Some(Box::new(builder)),
    }
}

impl IntoElement for SizedContent {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for SizedContent {
    type RequestLayoutState = ();
    type PrepaintState = Option<AnyElement>;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, None, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyElement> {
        let builder = self.builder.take()?;
        let mut child = builder(bounds, window, cx);
        child.prepaint_as_root(bounds.origin, bounds.size.into(), window, cx);
        Some(child)
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        child: &mut Option<AnyElement>,
        window: &mut Window,
        cx: &mut App,
    ) {
        if let Some(child) = child {
            child.paint(window, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_breakpoints_are_the_references_lower_bounds() {
        use WindowWidthClass::*;
        for (width, class) in [
            (0., Compact),
            (599.9, Compact),
            (600., Medium),
            (839.9, Medium),
            (840., Expanded),
            (1199.9, Expanded),
            (1200., Large),
            (1599.9, Large),
            (1600., ExtraLarge),
        ] {
            assert_eq!(WindowWidthClass::of(width), class, "{width}");
            assert!(class.min_width() <= width);
        }
        assert_eq!(WindowWidthClass::of(f32::NAN), Compact);
        assert_eq!(WindowHeightClass::of(479.), WindowHeightClass::Compact);
        assert_eq!(WindowHeightClass::of(480.), WindowHeightClass::Medium);
        assert_eq!(WindowHeightClass::of(900.), WindowHeightClass::Expanded);
    }

    struct Harness;
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .size(px(500.))
                .child(sized_content("sized", |bounds, _, _| {
                    div()
                        .debug_selector(|| "built".into())
                        .w(bounds.size.width / 2.)
                        .h(bounds.size.height)
                        .into_any_element()
                }))
        }
    }

    #[gpui::test]
    fn content_is_built_from_the_given_size_in_the_first_frame(cx: &mut gpui::TestAppContext) {
        let (_, cx) = cx.add_window_view(|_, _| Harness);
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let built = cx.debug_bounds("built").expect("built in the first frame");
        assert_eq!(built.size, size(px(250.), px(500.)));
    }
}
