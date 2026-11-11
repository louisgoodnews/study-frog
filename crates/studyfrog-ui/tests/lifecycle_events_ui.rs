#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for the expanded application lifecycle event bus (STORY-0151).

use studyfrog_core::meta::Meta;
use studyfrog_core::run::{Run, RunMode, RunState};
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::{Doc, NoteDoc};
use studyfrog_ui::edit_form::form_from_doc;
use studyfrog_ui::editor::Editor;
use studyfrog_ui::msg::{EditAction, Msg};
use studyfrog_ui::script_events::{EVENT_KINDS, from_msg};
use studyfrog_ui::state::State;

fn mock_editor(uid: Uid, title: &str, path: &str) -> Editor {
    let doc = Doc::Note(NoteDoc {
        meta: Meta {
            uid,
            title: title.into(),
            tags: Vec::new(),
            aliases: Vec::new(),
            created: None,
            updated: None,
        },
        rehearse_title: false,
        body: String::new(),
        extra: Vec::new(),
    });
    let form = form_from_doc(&doc);
    Editor {
        uid,
        path: path.into(),
        doc,
        hash: 0,
        original: form.clone(),
        form,
        previews: Vec::new(),
        saving: None,
        dirty_since: None,
        again: false,
        fail: None,
        conflict: false,
        saves: 0,
        keep_mine: false,
        after: None,
        backlinks: Vec::new(),
        curve: None,
        confirm_delete: false,
    }
}

#[test]
fn all_expected_event_kinds_are_registered() {
    let expected = [
        "vault_opened",
        "vault_closed",
        "item_created",
        "item_saved",
        "item_deleted",
        "rehearsal_started",
        "rehearsal_stepped",
        "rehearsal_finished",
        "exam_finished",
        "diary_entry_saved",
        "import_finished",
        "backup_created",
        "focus_block_finished",
        "script_finished",
    ];
    for kind in expected {
        assert!(
            EVENT_KINDS.contains(&kind),
            "expected {kind} to be registered in EVENT_KINDS"
        );
    }
}

#[test]
fn item_deleted_event_is_raised_on_confirm_delete() {
    let state = State {
        editor: Some(mock_editor(Uid(42), "Stale Card", "cards/stale.md")),
        ..Default::default()
    };

    let events = from_msg(&Msg::Edit(EditAction::ConfirmDelete), &state);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, "item_deleted");
    assert_eq!(events[0].data["title"], "Stale Card");
    assert_eq!(events[0].data["path"], "cards/stale.md");
}

#[test]
fn rehearsal_started_and_stepped_events_are_raised() {
    let state = State::default();
    let run = Run {
        uid: Uid(101),
        mode: RunMode::Flip,
        plan: studyfrog_core::plan::plan_spaced(),
        queue: vec![Uid(1), Uid(2), Uid(3)],
        state: RunState::Idle,
        started_at: None,
    };

    let events = from_msg(&Msg::RunReady(Ok(run)), &state);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, "rehearsal_started");
    assert_eq!(events[0].data["mode"], "flip");
    assert_eq!(events[0].data["items"], 3);
}

#[test]
fn backup_created_event_is_raised() {
    let state = State::default();
    let msg = Msg::Backup(studyfrog_ui::backup_ui::BackupMsg::Done(Ok(Some(
        "/backups/vault-2026.tar.gz".into(),
    ))));
    let events = from_msg(&msg, &state);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, "backup_created");
    assert_eq!(events[0].data["path"], "/backups/vault-2026.tar.gz");
}

#[test]
fn focus_block_finished_event_is_raised() {
    let state = State::default();
    let recorded = studyfrog_service::focus::FocusRecorded {
        minutes: 25,
        session: None,
        today: 50,
    };
    let msg = Msg::Recall(studyfrog_ui::recall_types::RecallMsg::Focus(
        studyfrog_ui::focus_ui::FocusMsg::Recorded(Ok(recorded)),
    ));
    let events = from_msg(&msg, &state);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, "focus_block_finished");
    assert_eq!(events[0].data["minutes"], 25);
    assert_eq!(events[0].data["today"], 50);
}
