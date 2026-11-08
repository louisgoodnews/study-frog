#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! "Extract / Sync cards" on a note's page (STORY-0110): the button, the preview and the sync with the
//! real worker.

mod common;

use common::flow::send;
use common::views::{assert_all_named, harness, open_model};
use common::{run_to_idle, temp_dir};
use egui_kittest::kittest::Queryable;
use studyfrog_core::uid::Uid;
use studyfrog_md::parse::parse_doc;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::library_bulk::LibraryEffect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::recall_types::{RecallEffect, RecallMsg};

const BODY: &str = "# Cells\n- Mitosis :: cell division\nQ: What splits?\nA: The cell\n";

fn note_text() -> String {
    format!(
        "---\nid: {}\nkind: note\ntitle: Lecture\n---\n{BODY}",
        format_ulid(Uid(5))
    )
}

fn editing() -> Model {
    let mut m = open_model();
    let loaded = LoadedDoc {
        path: "bio/lecture.md".into(),
        doc: parse_doc(&note_text()).unwrap(),
        hash: 1,
    };
    apply(&mut m, Msg::DocumentLoaded(Uid(5), Ok(loaded)));
    m
}

#[test]
fn a_note_with_inline_cards_offers_the_sync_and_asks_for_a_plan() {
    let mut h = harness(editing());
    h.run_steps(2);
    h.get_by_label("Extract / Sync 2 card(s)…").click();
    h.run_steps(2);
    let panel = &h.state().0.state.recall.extract;
    assert_eq!(panel.note, Some(Uid(5)));
    assert!(panel.busy);
    let mut m = editing();
    let fx = apply(
        &mut m,
        Msg::Recall(RecallMsg::ExtractPreview(Uid(5), BODY.into())),
    );
    assert_eq!(
        fx,
        [Effect::Library(LibraryEffect::Recall(
            RecallEffect::PlanExtract(Uid(5), BODY.into())
        ))]
    );
}

#[test]
fn the_preview_lists_new_items_and_the_sync_writes_them_once() {
    let dir = temp_dir("extract-flow");
    let bio = dir.path.join("bio");
    std::fs::create_dir_all(&bio).unwrap();
    let stack = format!(
        "---\nid: {}\nkind: stack\ntitle: Bio\n---\n",
        format_ulid(Uid(1))
    );
    std::fs::write(bio.join("_stack.md"), stack).unwrap();
    std::fs::write(bio.join("lecture.md"), note_text()).unwrap();
    let mut w = worker_new(None);
    let mut m = Model::default();
    m.state.app.tutorial_offered = true;
    let path = dir.path.display().to_string();
    run_to_idle(&mut m, &mut w, vec![Effect::OpenVault(path)]);
    send(&mut m, &mut w, Msg::OpenItem(Uid(5)));
    let preview = RecallMsg::ExtractPreview(Uid(5), BODY.into());
    send(&mut m, &mut w, Msg::Recall(preview.clone()));
    let plan = m.state.recall.extract.plan.clone().expect("planned");
    assert_eq!(plan.entries.len(), 2);
    let mut h = harness(m);
    h.run_steps(3);
    for label in ["Card: Mitosis", "Card: What splits?", "Sync cards"] {
        assert!(h.query_by_label(label).is_some(), "missing {label}");
    }
    assert_all_named(&h, "extract panel");
    let mut m = std::mem::take(&mut h.state_mut().0);
    send(&mut m, &mut w, Msg::Recall(RecallMsg::ExtractSync));
    assert_eq!(m.state.recall.extract.done, Some((2, 0)));
    assert!(m.state.library.undo.is_some(), "the sync can be undone");
    let count = || std::fs::read_dir(&bio).unwrap().count();
    assert_eq!(count(), 4);
    let mut h = harness(m);
    h.run_steps(3);
    assert!(h.query_by_label("Undo the sync").is_some());
    let mut m = std::mem::take(&mut h.state_mut().0);
    send(&mut m, &mut w, Msg::Recall(preview));
    let again = m.state.recall.extract.plan.clone().unwrap();
    assert!(again.entries.iter().all(|e| e.existing.is_some()));
    send(&mut m, &mut w, Msg::Recall(RecallMsg::ExtractSync));
    assert_eq!(count(), 4, "no duplicates");
}
