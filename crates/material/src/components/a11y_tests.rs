//! The accessibility tree applications publish: names, states and values
//! of list rows, overlays, carousel items and progress, read from the tree
//! GPUI Fast builds when assistive technology is active.
use super::*;
use gpui::{
    Context, Entity, IntoElement, Render, SharedString, TestAppContext, Window, div, prelude::*, px,
};
use serde_json::Value;

const MARKER: &str = "material-a11y-names-verified";

struct Screen(Entity<TimePickerState>);
impl Render for Screen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .relative()
            .size_full()
            .child(
                ListItem::new("row", "Wi-Fi")
                    .supporting("Connected")
                    .selected(true)
                    .trailing(ListTrailing::Switch(true))
                    .on_click(|_, _, _| {}),
            )
            .child(
                MaterialCarousel::new("carousel", "Photos")
                    .width(412.)
                    .items([CarouselItem::new().label("Beach").supporting("Summer")]),
            )
            .child(MaterialTooltip::plain("tip", "Save a copy").trigger(div().size(px(20.))))
            .child(progress_with_label("copy", "Copy", 0.25, window, cx))
            .child(indeterminate_progress_with_label("index", "Index", cx))
            .child(
                MaterialDialog::new("dialog")
                    .headline("Discard draft?")
                    .modal(true),
            )
            .child(
                MaterialSideSheet::new("side")
                    .modal(true)
                    .open(true)
                    .headline("Details"),
            )
            .child(TimePicker::new("time", &self.0))
            .child(
                MaterialBottomSheet::new("bottom")
                    .modal(true)
                    .open(true)
                    .label("Filters")
                    .child(div().h(px(100.))),
            )
    }
}

fn time_labels() -> TimePickerLabels {
    let s = SharedString::new_static;
    TimePickerLabels {
        select_title: s("Select time"),
        enter_title: s("Enter time"),
        hour: s("Hour"),
        minute: s("Minute"),
        am: s("AM"),
        pm: s("PM"),
        confirm: s("OK"),
        cancel: s("Cancel"),
        switch_to_input: s("Keyboard"),
        switch_to_dial: s("Dial"),
        number: std::rc::Rc::new(|n| n.to_string().into()),
        two_digits: std::rc::Rc::new(|n| format!("{n:02}").into()),
        separator: s(":"),
    }
}

/// Every node in the published tree.
fn nodes(tree: &str) -> Vec<Value> {
    fn walk(value: &Value, out: &mut Vec<Value>) {
        match value {
            Value::Object(map) => {
                if map.contains_key("role") {
                    out.push(value.clone());
                }
                map.values().for_each(|v| walk(v, out));
            }
            Value::Array(items) => items.iter().for_each(|v| walk(v, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(
        &serde_json::from_str(tree).expect("the tree is JSON"),
        &mut out,
    );
    out
}

/// The node labelled `label` with `role` (compared case-insensitively).
fn labelled<'a>(nodes: &'a [Value], role: &str, label: &str, tree: &str) -> &'a Value {
    nodes
        .iter()
        .find(|node| {
            node["label"] == label
                && node["role"]
                    .as_str()
                    .is_some_and(|r| r.eq_ignore_ascii_case(role))
        })
        .unwrap_or_else(|| panic!("no {role} named {label:?} in {tree}"))
}

#[gpui::test]
fn overlays_rows_and_items_announce_names_and_states(cx: &mut TestAppContext) {
    if std::env::var("GPUI_A11Y_ACTIVE").as_deref() != Ok("1") {
        // Fast's switch is read once per process: assert in a child process
        // rather than racing other tests over the environment.
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "components::a11y_tests::overlays_rows_and_items_announce_names_and_states",
                "--nocapture",
            ])
            .env("GPUI_A11Y_ACTIVE", "1")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "{stdout}{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(stdout.contains(MARKER), "the child ran no assertions");
        return;
    }
    cx.update(|cx| {
        gpui_base::init(cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(|window, cx| {
        window.set_rem_size(px(crate::tokens::typography::BASE_PX));
        Screen(cx.new(|cx| {
            let time = chrono::NaiveTime::from_hms_opt(7, 0, 0).unwrap();
            let mut picker = TimePickerState::new(time_labels(), time, window, cx);
            picker.toggle_mode(window, cx);
            picker
        }))
    });
    for _ in 0..3 {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
    }
    let tree = cx.update(|window, _| window.debug_a11y_tree_json().expect("accessibility on"));
    let all = nodes(&tree);

    // A list row announces its state and supporting text, not only its
    // headline.
    let row = labelled(&all, "button", "Wi-Fi", &tree);
    assert_eq!(row["selected"], true, "{row}");
    assert!(
        row["toggled"]
            .as_str()
            .is_some_and(|t| t.eq_ignore_ascii_case("true")),
        "{row}"
    );
    assert_eq!(row["description"], "Connected", "{row}");

    // Overlays are named by the caller's headline or label.
    labelled(&all, "dialog", "Discard draft?", &tree);
    labelled(&all, "dialog", "Details", &tree);
    labelled(&all, "dialog", "Filters", &tree);
    // A time picker is named for the mode it is in, here the input.
    labelled(&all, "dialog", "Enter time", &tree);

    // Carousel items are list items named by their label.
    let item = labelled(&all, "listItem", "Beach", &tree);
    assert_eq!(item["description"], "Summer", "{item}");

    // The labelled progress shorthands are named, with Base's value range.
    let copy = labelled(&all, "progressIndicator", "Copy", &tree);
    assert_eq!(copy["numeric_value"], 25.0, "{copy}");
    labelled(&all, "progressIndicator", "Index", &tree);
    // A hidden tooltip still describes its trigger.
    assert!(
        all.iter().any(|node| node["description"] == "Save a copy"),
        "{tree}"
    );
    println!("{MARKER}");
}
