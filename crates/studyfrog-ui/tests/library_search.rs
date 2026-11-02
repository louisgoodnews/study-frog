#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The search bar: Standard fields, the query box, errors, suggestions and the way between them.

mod common;

use common::views::{assert_all_named, harness, lib_item, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::status::Status;
use studyfrog_ui::library_search::{SearchMode, SearchMsg};
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;

fn model() -> Model {
    let mut m = open_model();
    let rows: Vec<_> = (0..12)
        .map(|i| {
            let mut r = lib_item(i as u128 + 1, &format!("Item {i}"));
            r.row.kind = [ItemKind::Note, ItemKind::Card, ItemKind::Question][i % 3];
            r.row.status = [Status::New, Status::Learning, Status::Done][i % 3];
            r
        })
        .collect();
    apply(&mut m, Msg::Library(LibraryMsg::Loaded(Ok(rows))));
    m
}

fn send(m: &mut Model, s: SearchMsg) {
    apply(m, Msg::Library(LibraryMsg::Search(s)));
}

fn shown(m: &Model) -> usize {
    m.state.library.shown.len()
}

#[test]
fn standard_fields_narrow_the_list_together() {
    let mut m = model();
    assert_eq!(shown(&m), 12);
    send(&mut m, SearchMsg::Kind(Some(ItemKind::Card)));
    assert_eq!(shown(&m), 4);
    send(&mut m, SearchMsg::Status(Some(Status::Learning)));
    assert_eq!(shown(&m), 4, "cards are every third, learning too");
    send(&mut m, SearchMsg::Status(Some(Status::Done)));
    assert_eq!(shown(&m), 0);
    send(&mut m, SearchMsg::Status(None));
    apply(&mut m, Msg::SearchEdited("Item 7".into()));
    assert_eq!(
        shown(&m),
        1,
        "Item 7 is the only card whose title has it: {:?}",
        m.state.library.shown
    );
}

#[test]
fn query_mode_narrows_shows_errors_and_keeps_the_last_result() {
    let mut m = model();
    send(&mut m, SearchMsg::Mode(true));
    assert_eq!(m.state.library.search.mode, SearchMode::Query);
    send(&mut m, SearchMsg::Query("type = card".into()));
    assert_eq!(shown(&m), 4);
    assert!(m.state.library.query_error.is_none());
    send(&mut m, SearchMsg::Query("type = ".into()));
    let e = m.state.library.query_error.clone().expect("an error");
    assert!(e.column >= 1 && !e.message.is_empty());
    assert_eq!(
        shown(&m),
        4,
        "the last good result stays while the text is wrong"
    );
    send(
        &mut m,
        SearchMsg::Query("status = done ORDER BY title DESC".into()),
    );
    assert_eq!(shown(&m), 4);
    assert!(m.state.library.query_error.is_none());
}

#[test]
fn standard_becomes_a_query_and_back() {
    let mut m = model();
    apply(&mut m, Msg::SearchEdited("Item".into()));
    send(&mut m, SearchMsg::Kind(Some(ItemKind::Note)));
    send(&mut m, SearchMsg::Mode(true));
    let text = m.state.library.search.query.clone();
    assert!(
        text.contains("title ~ Item") && text.contains("type = note"),
        "{text}"
    );
    assert_eq!(shown(&m), 4);

    send(
        &mut m,
        SearchMsg::Query("type = question AND status = done".into()),
    );
    send(&mut m, SearchMsg::Mode(false));
    let s = &m.state.library.search;
    assert_eq!(
        (s.mode, s.kind, s.status),
        (
            SearchMode::Standard,
            Some(ItemKind::Question),
            Some(Status::Done)
        )
    );
    assert_eq!(m.state.browser.search, "");

    send(&mut m, SearchMsg::Mode(true));
    send(
        &mut m,
        SearchMsg::Query("type = card OR type = note".into()),
    );
    send(&mut m, SearchMsg::Mode(false));
    assert!(
        m.state.notice.is_some(),
        "a query the fields cannot show says so"
    );
    assert_eq!(
        m.state.library.search.kind,
        Some(ItemKind::Question),
        "and the fields stay"
    );
}

#[test]
fn a_suggestion_is_inserted_into_the_query() {
    let mut m = model();
    send(&mut m, SearchMsg::Mode(true));
    send(&mut m, SearchMsg::Query("fam".into()));
    send(&mut m, SearchMsg::Complete(0));
    assert_eq!(m.state.library.search.query, "familiarity ");
}

#[test]
fn the_bar_is_on_screen_in_both_modes_and_named_for_assistive_technology() {
    let mut h = harness(model());
    h.run_steps(3);
    for name in [
        "Standard",
        "Query",
        "Search titles",
        "Type",
        "Status",
        "Priority",
    ] {
        assert!(h.query_by_label(name).is_some(), "{name}");
    }
    assert_all_named(&h, "standard bar");
    h.get_by_label("Query").click();
    h.run_steps(3);
    assert!(h.state().0.state.library.search.mode == SearchMode::Query);
    assert!(
        h.query_by_label("Query text").is_some() && h.query_by_label("Search titles").is_none()
    );
    send(&mut h.state_mut().0, SearchMsg::Query("status = ".into()));
    h.run_steps(3);
    assert!(
        h.query_by_label_contains("Column ").is_some(),
        "the error is shown under the box"
    );
}
