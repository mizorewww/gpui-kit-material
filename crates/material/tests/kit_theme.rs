//! Exercise the real Kit Root ordering, not only the theme projection helper.
extern crate gpui_kit as gpui;

use gpui_kit::{
    AppContext, Context, IntoElement, Pixels, Render, SharedString, TestAppContext, Window,
    WindowOptions, component::Theme, div, prelude::*, px,
};
use gpui_kit_material::{
    color::{ColorRole, PaletteId, Scheme},
    theme,
};
use std::{cell::RefCell, rc::Rc};

struct Snapshot {
    rem: Pixels,
    font: SharedString,
    material: Scheme,
    component_background: gpui_kit::Hsla,
}

struct Content {
    frames: Rc<RefCell<Vec<Snapshot>>>,
}

impl Render for Content {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.frames.borrow_mut().push(Snapshot {
            rem: window.rem_size(),
            font: window.text_style().font_family.clone(),
            material: theme::scheme(),
            component_background: Theme::global(cx).colors.background,
        });
        div()
            .size_full()
            .child(gpui_kit::component::button::Button::new("kit-button").label("Action"))
    }
}

#[gpui_kit::test]
fn kit_root_and_material_content_share_initial_and_preframe_theme(cx: &mut TestAppContext) {
    let frames = Rc::new(RefCell::new(Vec::new()));
    let initial = Scheme::from_palette(PaletteId::Ink);
    let (window, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
            // Matches gallery startup after capture overrides and before Root creation.
            assert!(theme::init_component_theme(initial, 1.5, cx));
            cx.new(|_| Content {
                frames: frames.clone(),
            })
        })
        .unwrap()
    });
    assert!(window.downcast::<gpui_kit::base::Root>().is_some());
    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    assert!(!frames.borrow().is_empty());
    for frame in frames.borrow().iter() {
        assert_eq!(frame.rem, px(21.));
        assert_eq!(frame.font.as_ref(), "Roboto");
        assert_eq!(frame.material, initial);
        assert_eq!(
            frame.component_background,
            theme::rgba(initial.get(ColorRole::SURFACE)).into()
        );
    }

    frames.borrow_mut().clear();
    let next = Scheme::from_palette(PaletteId::Paper);
    cx.update_window(window, |_, window, cx| {
        cx.set_reduce_motion(true);
        assert!(!theme::advance_component_theme(next, 2., window, cx));
        window.draw(cx).clear(cx);
    })
    .unwrap();
    assert!(!frames.borrow().is_empty());
    for frame in frames.borrow().iter() {
        assert_eq!(frame.rem, px(28.));
        assert_eq!(frame.font.as_ref(), "Roboto");
        assert_eq!(frame.material, next);
        assert_eq!(
            frame.component_background,
            theme::rgba(next.get(ColorRole::SURFACE)).into()
        );
    }
}

struct MaterialOnlyChild {
    paints: Rc<RefCell<Vec<gpui_kit::Rgba>>>,
}
impl Render for MaterialOnlyChild {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        // Deliberately do not read Component Theme: Material's paint is TLS.
        let primary = theme::color(ColorRole::PRIMARY);
        self.paints.borrow_mut().push(primary);
        div().size(px(80.)).bg(primary).child(
            gpui_kit_material::components::MaterialButton::new("cached-material").child("Material"),
        )
    }
}
struct CachedMaterialContent {
    child: gpui_kit::Entity<MaterialOnlyChild>,
}
impl Render for CachedMaterialContent {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(
            self.child
                .clone()
                .cached(gpui_kit::StyleRefinement::default().size(px(80.))),
        )
    }
}

#[gpui_kit::test]
fn cached_material_child_receives_projected_theme_before_synchronous_draw(cx: &mut TestAppContext) {
    let paints = Rc::new(RefCell::new(Vec::new()));
    let initial = Scheme::from_palette(PaletteId::Paper);
    let (window, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_reduce_motion(true);
        gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
            theme::init_component_theme(initial, 1., cx);
            let child = cx.new(|_| MaterialOnlyChild {
                paints: paints.clone(),
            });
            cx.new(|_| CachedMaterialContent { child })
        })
        .unwrap()
    });
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    let count = paints.borrow().len();
    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    assert_eq!(
        paints.borrow().len(),
        count,
        "prove the Material child is retained"
    );

    // Only a projected role changes: Root's inherited text style, mode, font,
    // rem and background remain unchanged and cannot incidentally invalidate it.
    let next = initial.with_role(
        ColorRole::PRIMARY,
        gpui_kit_material::color::Color::hex(0x123456),
    );
    cx.update_window(window, |_, window, cx| {
        assert!(!theme::advance_component_theme(next, 1., window, cx));
        assert_eq!(
            Theme::global(cx).colors.primary,
            theme::rgba(next.get(ColorRole::PRIMARY)).into()
        );
        window.draw(cx).clear(cx);
        assert_eq!(
            *paints.borrow().last().unwrap(),
            theme::rgba(next.get(ColorRole::PRIMARY)),
            "immediate draw must not replay stale Material paint while Component refresh is queued"
        );
    })
    .unwrap();
    cx.run_until_parked();
    let count = paints.borrow().len();
    cx.update_window(window, |_, window, cx| {
        assert!(!theme::advance_component_theme(next, 1., window, cx));
        window.draw(cx).clear(cx);
    })
    .unwrap();
    assert_eq!(
        paints.borrow().len(),
        count,
        "unchanged theme must retain the child"
    );

    // Also exercise the ordinary event boundary: Component's queued refresh
    // must remain sufficient after effects are flushed, without a caller notify.
    cx.update_window(window, |_, window, cx| {
        assert!(!theme::advance_component_theme(initial, 1., window, cx));
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    assert_eq!(
        *paints.borrow().last().unwrap(),
        theme::rgba(initial.get(ColorRole::PRIMARY))
    );
}
