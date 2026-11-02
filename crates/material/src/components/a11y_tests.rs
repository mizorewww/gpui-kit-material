//! The accessibility tree applications publish: names, states and values
//! of list rows, overlays, carousel items and progress, read from the tree
//! GPUI Fast builds when assistive technology is active.
use super::*;
use gpui::{
    Context, Entity, IntoElement, Render, SharedString, TestAppContext, Window, div, prelude::*, px,
};
use gpui_base::slider::SliderState;
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
            .child(
                status_fact_chip("fact", "Checked: passed", StatusKind::Success).child("Checked"),
            )
            .child(
                Stepper::new("stepper", "Setup progress")
                    .steps(["Source", "Service", "Account"])
                    .current(1)
                    .state_labels("Finished", "Current", "Upcoming"),
            )
            .child(
                ExposedDropdown::new("language", "Language")
                    .show_label(false)
                    .required(true)
                    .options(["English", "Chinese"])
                    .selected(Some(1)),
            )
            .child(
                ExposedDropdown::new("model", "Model")
                    .menu(MaterialMenu::new("models").item(MaterialMenuItem::new("Deep")))
                    .value("Deep"),
            )
            .child(
                MaterialCard::new("toggle-card", CardVariant::Outlined)
                    .selected(true)
                    .accessibility_label("Toggle card")
                    .on_click(|_, _, _| {}),
            )
            .child(
                CardGroup::new("card-group", "Card choices")
                    .card(
                        MaterialCard::new("choice-a", CardVariant::Outlined)
                            .selected(false)
                            .accessibility_label("First choice"),
                    )
                    .card(
                        MaterialCard::new("choice-b", CardVariant::Outlined)
                            .selected(true)
                            .accessibility_label("Second choice"),
                    ),
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
        // The exposed dropdown's field draws through Kit's component theme.
        gpui_kit::init(cx);
        crate::init(cx);
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

    // A selectable card is a toggle button by default…
    let toggled = |node: &Value| {
        node["toggled"]
            .as_str()
            .is_some_and(|t| t.eq_ignore_ascii_case("true"))
    };
    let card = labelled(&all, "button", "Toggle card", &tree);
    assert!(toggled(card), "{card}");
    // …and in a card group a radio button with its place in the set.
    labelled(&all, "radioGroup", "Card choices", &tree);
    for (position, name, chosen) in [(1, "First choice", false), (2, "Second choice", true)] {
        let node = labelled(&all, "radioButton", name, &tree);
        assert_eq!(toggled(node), chosen, "{node}");
        assert_eq!(node["position_in_set"], position, "{node}");
        assert_eq!(node["size_of_set"], 2, "{node}");
    }

    // Overlays are named by the caller's headline or label.
    labelled(&all, "dialog", "Discard draft?", &tree);
    labelled(&all, "dialog", "Details", &tree);
    labelled(&all, "dialog", "Filters", &tree);
    // A dropdown that draws no label is still named by it, required mark included,
    // and announces its value.
    let language = all
        .iter()
        .find(|node| {
            node["role"]
                .as_str()
                .is_some_and(|r| r.eq_ignore_ascii_case("comboBox"))
                && node["label"]
                    .as_str()
                    .is_some_and(|l| l.starts_with("Language"))
        })
        .unwrap_or_else(|| panic!("no comboBox named Language in {tree}"));
    assert_ne!(
        language["label"], "Language",
        "the required mark: {language}"
    );
    assert_eq!(language["value"], "Chinese", "{language}");

    // A time picker is named for the mode it is in, here the input.
    labelled(&all, "dialog", "Enter time", &tree);

    // A dropdown with the caller's menu announces the value it shows.
    let model = labelled(&all, "comboBox", "Model", &tree);
    assert_eq!(model["value"], "Deep", "{model}");

    // A status fact says in words what its color says.
    labelled(&all, "label", "Checked: passed", &tree);

    // A stepper is a list of its steps, each with its name, state and place in the flow.
    labelled(&all, "list", "Setup progress", &tree);
    for (position, step, state, current) in [
        (1, "Source", "Finished", false),
        (2, "Service", "Current", true),
        (3, "Account", "Upcoming", false),
    ] {
        let node = labelled(&all, "listItem", step, &tree);
        assert_eq!(node["position_in_set"], position, "{node}");
        assert_eq!(node["size_of_set"], 3, "{node}");
        assert_eq!(node["description"], state, "{node}");
        assert_eq!(node["selected"], current, "{node}");
    }

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

struct Sliders(Entity<SliderState>, Entity<SliderState>);
impl Render for Sliders {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(400.))
            .child(MaterialSlider::new(&self.0, "Volume"))
            .child(MaterialSlider::new(&self.1, "Price"))
    }
}

#[gpui::test]
fn sliders_report_a_usable_step_and_range_thumbs_their_bounds(cx: &mut TestAppContext) {
    const SLIDER_MARKER: &str = "material-a11y-slider-verified";
    if std::env::var("GPUI_A11Y_ACTIVE").as_deref() != Ok("1") {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "components::a11y_tests::sliders_report_a_usable_step_and_range_thumbs_their_bounds",
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
        assert!(
            stdout.contains(SLIDER_MARKER),
            "the child ran no assertions"
        );
        return;
    }
    cx.update(|cx| {
        gpui_base::init(cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(|window, cx| {
        window.set_rem_size(px(crate::tokens::typography::BASE_PX));
        // Two continuous sliders (step 0): a single one and a range.
        Sliders(
            cx.new(|_| SliderState::new().step(0.).max(200.).default_value(50.)),
            cx.new(|_| {
                SliderState::new()
                    .step(0.)
                    .max(200.)
                    .default_value((40., 120.))
            }),
        )
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

    // A continuous slider reports the keyboard's step, a hundredth of its range, not 0.
    let volume = labelled(&all, "slider", "Volume", &tree);
    assert_eq!(volume["numeric_value_step"], 2.0, "{volume}");

    // A range is two sliders, its thumbs, in a named group that has no value or
    // actions of its own.
    let group = labelled(&all, "group", "Price", &tree);
    assert!(
        group.get("numeric_value").is_none_or(Value::is_null),
        "{group}"
    );
    let actions = group["on_action"].as_array().cloned().unwrap_or_default();
    assert!(
        !actions
            .iter()
            .any(|action| action == "Increment" || action == "Decrement"),
        "{group}"
    );

    // Each range thumb is bounded by the other.
    let price: Vec<&Value> = all
        .iter()
        .filter(|node| {
            node["label"] == "Price"
                && node["role"]
                    .as_str()
                    .is_some_and(|r| r.eq_ignore_ascii_case("slider"))
        })
        .collect();
    assert_eq!(price.len(), 2, "only the thumbs are sliders: {tree}");
    let start: Vec<&&Value> = price
        .iter()
        .filter(|node| node["numeric_value"] == 40.0)
        .collect();
    assert_eq!(start.len(), 1, "{tree}");
    assert_eq!(start[0]["min_numeric_value"], 0.0, "{}", start[0]);
    assert_eq!(start[0]["max_numeric_value"], 120.0, "{}", start[0]);
    let end = price
        .iter()
        .find(|node| node["numeric_value"] == 120.0 && node["min_numeric_value"] == 40.0)
        .unwrap_or_else(|| panic!("no end thumb bounded by the start in {tree}"));
    assert_eq!(end["max_numeric_value"], 200.0, "{end}");
    for node in price {
        assert_eq!(node["numeric_value_step"], 2.0, "{node}");
    }
    println!("{SLIDER_MARKER}");
}
