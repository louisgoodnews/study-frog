#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Roadmap tab: rows, bars, the axis, zoom, and 1000 rows inside the budget.

mod common;

use std::time::Instant;

use common::views::{assert_all_named, harness, lib_item, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::status::Status;
use studyfrog_service::library_rows::LibraryItem;
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::stack_roadmap::{Zoom, axis, bar, px_per_day, rows, visible};
use studyfrog_ui::stack_view_ui::StackViewMsg;
use studyfrog_ui::state::Selection;

const TODAY: i64 = 20_000;

fn item(n: u128, title: &str, created: i64, due: Option<i64>, chain: &[&str]) -> LibraryItem {
    let mut r = lib_item(n, title);
    r.row.created_day = Some(created);
    r.row.due_day = due;
    r.row.status = Status::Learning;
    r.row.stacks = chain.iter().map(|s| (*s).to_owned()).collect();
    r
}

fn model(items: Vec<LibraryItem>) -> Model {
    let mut m = open_model();
    m.state.app.stack_tabs = vec!["french:roadmap".into()];
    let v = m.state.vault.as_mut().unwrap();
    let mut verbs = common::views::stack("Verbs", "french/verbs", 0, 1);
    verbs.parent = Some("french".into());
    v.overview.stacks.insert(1, verbs);
    apply(&mut m, Msg::Library(LibraryMsg::Loaded(Ok(items))));
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    m
}

#[test]
fn rows_hold_the_stacks_inside_then_the_items_and_bars_span_first_to_next_date() {
    let fr = ["french", "French"];
    let verbs = ["french", "French", "french/verbs", "Verbs"];
    let m = model(vec![
        item(1, "Late", TODAY - 5, Some(TODAY + 10), &fr),
        item(2, "Early", TODAY - 20, None, &fr),
        item(3, "Aller", TODAY - 8, Some(TODAY + 30), &verbs),
        item(4, "Other", TODAY, None, &["maths", "Maths"]),
    ]);
    let stacks = &m.state.vault.as_ref().unwrap().overview.stacks;
    let list = rows(&m.state.library, stacks, "french", TODAY);
    let shape: Vec<(&str, usize, i64, i64)> = list
        .iter()
        .map(|r| (r.title.as_str(), r.depth, r.start, r.end))
        .collect();
    assert_eq!(
        shape,
        [
            ("Verbs", 1, TODAY - 8, TODAY + 30),
            ("Early", 0, TODAY - 20, TODAY - 20),
            ("Late", 0, TODAY - 5, TODAY + 10),
        ],
        "the verbs stack spans its item; its item is not listed again; a missing due date is a one-day bar"
    );
    let (from, to) = axis(&list, TODAY);
    assert!(from < TODAY - 20 && to > TODAY + 30);
    let (x, w) = bar(&list[2], from, px_per_day(Zoom::Weeks));
    assert_eq!(x, (list[2].start - from) as f32 * 10.0);
    assert_eq!(w, 16.0 * 10.0);
    assert_eq!(visible(100, 30.0, 0.0, 90.0), 0..4);
    assert_eq!(visible(3, 30.0, 600.0, 90.0), 3..3);
}

#[test]
fn the_tab_names_every_bar_zooms_and_opens_what_is_clicked() {
    let fr = ["french", "French"];
    let mut h = harness(model(vec![item(
        1,
        "Bonjour",
        TODAY - 3,
        Some(TODAY + 4),
        &fr,
    )]));
    h.run_steps(4);
    for z in ["Today", "Weeks", "Months", "Quarters"] {
        assert!(h.query_all_by_label(z).count() >= 1, "missing zoom {z}");
    }
    assert!(
        h.query_by_label_contains("Item Bonjour, Learning")
            .is_some()
    );
    assert!(h.query_by_label_contains("Stack Verbs").is_some());
    assert_all_named(&h, "roadmap");
    h.get_by_label("Quarters").click();
    h.run_steps(3);
    assert_eq!(h.state().0.state.library.stack_view.zoom, Zoom::Quarters);
    h.get_by_label_contains("Item Bonjour").click();
    h.run_steps(3);
    assert!(
        h.state()
            .1
            .iter()
            .any(|e| matches!(e, studyfrog_ui::effect::Effect::LoadDocument(_))),
        "{:?}",
        h.state().1
    );
    let _ = StackViewMsg::Zoom(Zoom::Today);
}

#[test]
fn a_thousand_rows_lay_out_and_draw_inside_the_budget() {
    let fr = ["french", "French"];
    let items: Vec<LibraryItem> = (0..1000)
        .map(|i| {
            item(
                i as u128 + 1,
                &format!("Item {i}"),
                TODAY - (i % 200) as i64,
                Some(TODAY + (i % 90) as i64),
                &fr,
            )
        })
        .collect();
    let m = model(items);
    let stacks = m.state.vault.as_ref().unwrap().overview.stacks.clone();
    let start = Instant::now();
    let list = rows(&m.state.library, &stacks, "french", TODAY);
    let _ = axis(&list, TODAY);
    assert!(
        start.elapsed().as_millis() < 50,
        "layout of 1000 rows took {:?}",
        start.elapsed()
    );
    assert_eq!(list.len(), 1001);

    let mut h = harness(m);
    let start = Instant::now();
    h.run_steps(4);
    let per_frame = start.elapsed().as_millis() / 4;
    assert!(
        per_frame < 150,
        "a frame with 1000 rows took {per_frame} ms in a debug build"
    );
}

#[test]
fn the_roadmap_stretches_over_the_room_there_is_instead_of_a_few_pixels() {
    use egui_kittest::kittest::NodeT;
    let fr = ["french", "French"];
    let mut h = harness(model(vec![item(
        1,
        "Bonjour",
        TODAY - 3,
        Some(TODAY + 4),
        &fr,
    )]));
    h.run_steps(6);
    let b = h
        .get_by_label_contains("Item Bonjour")
        .accesskit_node()
        .bounding_box()
        .unwrap();
    // Two days at ten pixels a day were 20 pixels; a short axis is now stretched over the window.
    assert!(b.x1 - b.x0 > 60.0, "bar is {} px wide", b.x1 - b.x0);
    assert!(b.y1 - b.y0 >= 16.0, "bar is {} px high", b.y1 - b.y0);
}
