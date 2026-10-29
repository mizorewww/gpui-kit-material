use super::{
    DividerContext, DividerDirection, DividerOrientation, DividerVariant, MaterialDivider,
};
use gpui::{Context, IntoElement, Render, TestAppContext, Window, div, prelude::*, px};
use serde_json::Value;

struct Probe {
    row: Value,
}
fn divider(row: &Value) -> MaterialDivider {
    let mut divider = MaterialDivider::new("divider")
        .orientation(if row["orientation"] == "vertical" {
            DividerOrientation::Vertical
        } else {
            DividerOrientation::Horizontal
        })
        .variant(match row["variant"].as_str().unwrap() {
            "inset" => DividerVariant::Inset,
            "middle-inset" => DividerVariant::MiddleInset,
            _ => DividerVariant::FullWidth,
        })
        .direction(if row["direction"] == "rtl" {
            DividerDirection::Rtl
        } else {
            DividerDirection::Ltr
        })
        .context(match row["context"].as_str().unwrap_or("plain") {
            "list" => DividerContext::List,
            "card" => DividerContext::Card,
            _ => DividerContext::Plain,
        });
    if let Some(value) = row["config"]["thickness"].as_f64() {
        divider = divider.thickness(px(value as f32));
    }
    if let Some(value) = row["config"]["insetStart"].as_f64() {
        divider = divider.inset_start(px(value as f32));
    }
    if let Some(value) = row["config"]["insetEnd"].as_f64() {
        divider = divider.inset_end(px(value as f32));
    }
    divider
}
impl Render for Probe {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14. * self.row["scale"].as_f64().unwrap_or(1.) as f32));
        let layout =
            self.row["layout"]
                .as_str()
                .unwrap_or(if self.row["orientation"] == "vertical" {
                    "row"
                } else {
                    "column"
                });
        let mut host = div().w(px(240.)).h(px(120.));
        host = match layout {
            "block" => host.block(),
            "column" => host.flex().flex_col().items_stretch(),
            "row-center" => host.flex().items_center(),
            _ => host.flex().items_stretch(),
        };
        host.child(
            divider(&self.row)
                .into_line()
                .debug_selector(|| "line".into()),
        )
    }
}
#[gpui::test]
fn reference_dimensions_and_physical_margins(cx: &mut TestAppContext) {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../conformance/reference/divider.json"
    ))
    .unwrap();
    let rows = fixture["rows"].as_array().unwrap();
    let (view, cx) = cx.add_window_view(|_, _| Probe {
        row: rows[0].clone(),
    });
    for row in rows.iter().chain(fixture["custom"].as_array().unwrap()) {
        view.update(cx, |this, cx| {
            this.row = row.clone();
            cx.notify();
        });
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let bounds = cx.debug_bounds("line").unwrap();
        assert_eq!(
            f32::from(bounds.size.width),
            row["sample"]["rect"]["width"].as_f64().unwrap() as f32,
            "{row}"
        );
        assert_eq!(
            f32::from(bounds.size.height),
            row["sample"]["rect"]["height"].as_f64().unwrap() as f32,
            "{row}"
        );
        let mut line = divider(row).styled_line();
        let margins = &line.style().margin;
        for (actual, expected) in [margins.top, margins.right, margins.bottom, margins.left]
            .into_iter()
            .zip(row["sample"]["margin"].as_array().unwrap())
        {
            let expected = expected
                .as_str()
                .unwrap()
                .trim_end_matches("px")
                .parse::<f32>()
                .unwrap();
            assert_eq!(actual, Some(px(expected).into()), "{row}");
        }
    }
}
#[test]
fn full_width_retains_insets_and_orientation_clears_old_axis() {
    let mut divider = MaterialDivider::new("divider")
        .inset_start(px(24.))
        .inset_end(px(8.))
        .variant(DividerVariant::Inset);
    assert_eq!(
        divider.styled_line().style().margin.left,
        Some(px(24.).into())
    );
    divider = divider.variant(DividerVariant::FullWidth);
    assert_eq!(
        divider.styled_line().style().margin.left,
        Some(px(0.).into())
    );
    divider = divider
        .orientation(DividerOrientation::Vertical)
        .variant(DividerVariant::MiddleInset);
    let mut line = divider.styled_line();
    assert_eq!(line.style().margin.left, Some(px(0.).into()));
    assert_eq!(line.style().margin.top, Some(px(24.).into()));
    assert_eq!(line.style().margin.bottom, Some(px(8.).into()));
}
#[test]
fn thickness_defaults_and_caller_geometry_are_preserved() {
    for value in [0., -1., f32::NAN, f32::INFINITY] {
        assert_eq!(
            MaterialDivider::new("divider")
                .thickness(px(value))
                .thickness,
            px(1.)
        );
    }
    let mut line = MaterialDivider::new("divider")
        .thickness(px(0.5))
        .h(px(3.))
        .styled_line();
    assert_eq!(line.style().size.height, Some(px(3.).into()));
}

#[gpui::test]
fn paint_resolves_live_theme_roles_and_caller_override(cx: &mut TestAppContext) {
    use crate::color::{
        ColorRole,
        themes::{Contrast, ThemeId, ThemeMode},
    };
    use gpui::rgb;
    let (_,cx)=cx.add_window_view(|_,_|Probe {row:serde_json::json!({"orientation":"horizontal","variant":"full-width","context":"plain","scale":1,"layout":"block"})});
    for (mode, expected) in [(ThemeMode::Light, 0xcac4d0), (ThemeMode::Dark, 0x49454f)] {
        cx.update(|window, cx| {
            cx.set_reduce_motion(true);
            crate::theme::apply_scheme(
                ThemeId::Baseline.scheme(mode, Contrast::Standard),
                window,
                cx,
            );
            let mut line = MaterialDivider::new("divider").styled_line();
            assert_eq!(line.style().background, Some(rgb(expected).into()));
            let mut custom = MaterialDivider::new("divider")
                .color_role(ColorRole::PRIMARY)
                .styled_line();
            assert_eq!(
                custom.style().background,
                Some(crate::theme::color(ColorRole::PRIMARY).into())
            );
            let mut refined = MaterialDivider::new("divider")
                .bg(crate::theme::color(ColorRole::ERROR))
                .styled_line();
            assert_eq!(
                refined.style().background,
                Some(crate::theme::color(ColorRole::ERROR).into())
            );
        });
    }
}
