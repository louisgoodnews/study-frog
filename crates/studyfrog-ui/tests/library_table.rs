#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The library table on screen: headings that sort, the count, the column menu, and ten thousand rows.

mod common;

use std::time::Instant;

use common::views::{assert_all_named, harness, lib_item, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::status::Status;
use studyfrog_ui::library_ui::{Column, LibraryMsg};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;

fn model_with(n: usize) -> Model {
    let mut m = open_model();
    let rows: Vec<_> = (0..n)
        .map(|i| {
            let mut r = lib_item(i as u128 + 1, &format!("Item number {i}"));
            r.row.status = [
                Status::New,
                Status::Learning,
                Status::Reviewing,
                Status::Done,
            ][i % 4];
            r.row.reviews = (i % 30) as u32;
            r.row.due_day = (i % 3 != 0).then_some(20_000 + (i % 90) as i64);
            r
        })
        .collect();
    apply(&mut m, Msg::Library(LibraryMsg::Loaded(Ok(rows))));
    m
}

#[test]
fn the_table_shows_the_headings_the_count_and_the_first_rows() {
    let mut h = harness(model_with(30));
    h.run_steps(3);
    for name in [
        "Sort by Title",
        "Sort by Type",
        "Sort by Status",
        "Sort by Priority",
        "Sort by Due",
    ] {
        assert!(h.query_by_label(name).is_some(), "{name}");
    }
    assert!(h.query_by_label("30 of 30 items").is_some());
    assert!(h.query_by_label("Item number 0").is_some());
    assert_all_named(&h, "library table");
}

#[test]
fn clicking_a_heading_sorts_by_it_and_marks_the_direction() {
    let mut h = harness(model_with(30));
    h.run_steps(3);
    h.get_by_label("Sort by Title").click();
    h.run_steps(3);
    assert_eq!(h.state().0.state.library.sort.len(), 1);
    assert!(
        h.query_by_label("Sort by Title, sorted ascending")
            .is_some()
    );
    h.get_by_label("Sort by Title, sorted ascending").click();
    h.run_steps(3);
    assert!(
        h.query_by_label("Sort by Title, sorted descending")
            .is_some()
    );
}

#[test]
fn the_column_menu_adds_a_column_and_the_table_shows_it() {
    let mut h = harness(model_with(30));
    h.run_steps(3);
    assert!(h.query_by_label("Sort by Familiarity").is_none());
    h.get_by_label("Columns").click();
    h.run_steps(3);
    h.get_by_label("Familiarity").click();
    h.run_steps(3);
    assert!(
        h.state()
            .0
            .state
            .library
            .columns
            .contains(&Column::Familiarity)
    );
    assert!(h.query_by_label("Sort by Familiarity").is_some());
}

#[test]
fn an_empty_list_says_why() {
    let mut empty = harness(model_with(0));
    empty.run_steps(2);
    assert!(empty.query_by_label("No items here yet.").is_some());
    let mut m = model_with(5);
    apply(&mut m, Msg::SearchEdited("nothing like this".into()));
    let mut none = harness(m);
    none.run_steps(2);
    assert!(
        none.query_by_label("No items match. Clear the search?")
            .is_some()
    );
}

#[test]
fn ten_thousand_rows_are_drawn_within_the_frame_budget_because_only_the_visible_ones_are_built() {
    let mut h = harness(model_with(10_000));
    h.run_steps(3);
    assert!(h.query_by_label("10000 of 10000 items").is_some());
    let frames = 20;
    let start = Instant::now();
    for _ in 0..frames {
        h.run_steps(1);
    }
    let per_frame = start.elapsed().as_secs_f64() * 1000.0 / f64::from(frames);
    println!("10000 rows: {per_frame:.2} ms per frame (debug build)");
    assert!(per_frame < 250.0, "{per_frame} ms per frame");
    let rows_built = h
        .query_all_by_role(eframe::egui::accesskit::Role::Link)
        .count();
    assert!(
        rows_built < 100,
        "{rows_built} rows were built for a screen of a few dozen"
    );
}
