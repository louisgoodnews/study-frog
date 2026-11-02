#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Item status on screen: the menu on the item page, labels on list rows and a stack's roll-up.

mod common;

use common::views::{assert_all_named, harness, lib_item, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::status::Status;
use studyfrog_core::uid::Uid;
use studyfrog_md::status::status_of;
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::edit_form::apply_form;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::{EditAction, Msg};
use studyfrog_ui::state::{ItemRow, Selection};

const CARD: &str =
    "---\nid: 0000000000000000000000000A\nkind: card\ntitle: Bonjour\n---\nq\n???\na\n";

fn editing(text: &str) -> Model {
    let mut m = open_model();
    let doc = studyfrog_md::parse::parse_doc(text).unwrap();
    let loaded = LoadedDoc {
        path: "french/a.md".into(),
        doc,
        hash: 1,
    };
    apply(&mut m, Msg::DocumentLoaded(Uid(0xA), Ok(loaded)));
    m
}

fn row(n: u128, title: &str, status: Status) -> ItemRow {
    ItemRow {
        uid: Uid(n),
        title: title.into(),
        kind: ItemKind::Card,
        path: format!("french/{n}.md"),
        due: None,
        status,
    }
}

#[test]
fn the_status_is_chosen_from_a_menu_and_saved_in_the_front_matter() {
    let mut h = harness(editing(CARD));
    h.run_steps(2);
    assert!(h.query_by_label("Item status").is_some());
    h.get_by_label("Item status").click();
    h.run_steps(2);
    h.get_by_label("Complete").click();
    h.run_steps(2);
    let e = h.state().0.state.editor.as_ref().unwrap();
    assert_eq!(e.form.status, Some(Status::Done));
    let saved = apply_form(&e.doc, &e.form).unwrap();
    assert_eq!(status_of(&saved), Some(Status::Done));
    assert_all_named(&h, "item page with the status menu");
}

#[test]
fn choosing_automatic_removes_the_key_and_an_untouched_unknown_word_stays() {
    let mut m = editing(
        "---\nid: 0000000000000000000000000A\nkind: card\ntitle: C\nstatus: done\n---\nq\n???\na\n",
    );
    apply(&mut m, Msg::Edit(EditAction::Status(None)));
    let e = m.state.editor.as_ref().unwrap();
    let saved = apply_form(&e.doc, &e.form).unwrap();
    assert!(
        studyfrog_md::render::render_doc(&saved)
            .unwrap()
            .find("status")
            .is_none()
    );

    let odd = editing(
        "---\nid: 0000000000000000000000000A\nkind: card\ntitle: C\nstatus: finished\n---\nq\n???\na\n",
    );
    let e = odd.state.editor.as_ref().unwrap();
    assert_eq!(e.form.status, None);
    let kept = apply_form(&e.doc, &e.form).unwrap();
    assert!(
        studyfrog_md::render::render_doc(&kept)
            .unwrap()
            .contains("status: finished")
    );
}

#[test]
fn the_menu_says_what_the_history_makes_of_an_item_without_a_status() {
    use studyfrog_ui::view::status_ui::{menu_text, summary_text};
    assert_eq!(menu_text(Some(Status::Done), Some(Status::New)), "Complete");
    assert_eq!(
        menu_text(None, Some(Status::Learning)),
        "Automatic (Learning)"
    );
    assert_eq!(menu_text(None, None), "Automatic");
    assert_eq!(summary_text(&[(Status::New, 0), (Status::Done, 0)]), None);
    assert_eq!(
        summary_text(&[(Status::Learning, 2), (Status::Paused, 1)]).as_deref(),
        Some("2 Learning, 1 On hold")
    );
}

#[test]
fn list_rows_show_each_items_status_and_the_stack_shows_its_roll_up() {
    let mut m = open_model();
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    m.state.app.stack_tabs = vec!["french:list".into()];
    let items = vec![
        row(1, "Bonjour", Status::Learning),
        row(2, "Au revoir", Status::Done),
    ];
    apply(&mut m, Msg::ItemsLoaded(Ok(items)));
    let (mut one, mut two) = (lib_item(1, "Bonjour"), lib_item(2, "Au revoir"));
    (one.row.status, two.row.status) = (Status::Learning, Status::Done);
    let rows = studyfrog_ui::library_ui::LibraryMsg::Loaded(Ok(vec![one, two]));
    apply(&mut m, Msg::Library(rows));
    let counts = vec![
        (Status::New, 1),
        (Status::Learning, 3),
        (Status::Reviewing, 0),
        (Status::Done, 2),
        (Status::Paused, 0),
    ];
    apply(&mut m, Msg::StatusCountsLoaded(Ok(counts)));
    let mut h = harness(m);
    h.run_steps(2);
    assert!(h.query_by_label("Learning").is_some() && h.query_by_label("Complete").is_some());
    assert!(
        h.query_by_label("Status of its items: 1 Not studied yet, 3 Learning, 2 Complete")
            .is_some(),
        "the roll-up lists only the statuses that have items"
    );
    assert_all_named(&h, "stack with statuses");
}

#[test]
fn a_failed_count_clears_the_roll_up_and_selecting_a_stack_asks_for_it() {
    let mut m = open_model();
    let fx = apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    assert!(
        fx.iter()
            .any(|e| matches!(e, studyfrog_ui::effect::Effect::LoadStatusCounts(_)))
    );
    apply(&mut m, Msg::StatusCountsLoaded(Ok(vec![(Status::New, 4)])));
    apply(&mut m, Msg::StatusCountsLoaded(Err("disk".into())));
    assert!(m.state.browser.status_counts.is_empty());
}
