#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0200: the editor saves by itself, token by token, and before anything that leaves it.

mod common;

use std::fs;

use common::{run_to_idle, temp_dir};
use studyfrog_core::uid::Uid;
use studyfrog_ui::editor::is_dirty;
use studyfrog_ui::editor_save::{SAVE_AFTER_MS, token_done};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::{EditAction, Msg};
use studyfrog_ui::state::Screen;

const ALPHA: &str = "---\nid: 0000000000000000000000000A\nkind: note\ntitle: Alpha\n---\nbody\n";
const BETA: &str = "---\nid: 0000000000000000000000000B\nkind: note\ntitle: Beta\n---\nother\n";
const T0: i64 = 1_800_000_000_000;

fn open() -> (Model, Worker, common::TempDir) {
    let dir = temp_dir("autosave");
    fs::create_dir_all(dir.path.join("s")).unwrap();
    let stack = "---\nid: 00000000000000000000000001\nkind: stack\ntitle: S\n---\n";
    fs::write(dir.path.join("s/_stack.md"), stack).unwrap();
    fs::write(dir.path.join("s/alpha.md"), ALPHA).unwrap();
    fs::write(dir.path.join("s/beta.md"), BETA).unwrap();
    let (mut m, mut w) = (Model::default(), worker_new(None));
    let fx = apply(&mut m, Msg::OpenRecent(dir.path.display().to_string()));
    run_to_idle(&mut m, &mut w, fx);
    m.state.wall_ms = T0;
    let fx = apply(&mut m, Msg::OpenItem(Uid(0xA)));
    run_to_idle(&mut m, &mut w, fx);
    (m, w, dir)
}

fn type_text(m: &mut Model, text: &str) -> Vec<Effect> {
    apply(m, Msg::Edit(EditAction::Field(0, text.into())))
}

fn on_disk(dir: &common::TempDir) -> String {
    fs::read_to_string(dir.path.join("s/alpha.md")).unwrap()
}

fn saved_toasts(m: &Model) -> Vec<String> {
    let titles = m.state.toasts.iter().map(|t| t.title.clone());
    titles.filter(|t| t.starts_with("Saved ")).collect()
}

fn saves(fx: &[Effect]) -> usize {
    fx.iter()
        .filter(|e| matches!(e, Effect::SaveDocument { .. }))
        .count()
}

#[test]
fn a_token_is_finished_by_a_whitespace_or_line_break_typed_into_the_text() {
    assert!(token_done("hello", "hello "));
    assert!(token_done("hello", "hello\n"));
    assert!(token_done("a c", "a b c"), "a space typed in the middle");
    assert!(!token_done("hello", "hello w"));
    assert!(
        !token_done("hello ", "hello"),
        "a deletion finishes nothing"
    );
    assert!(!token_done("same", "same"));
    assert!(token_done("", " "));
    assert!(!token_done("ä", "äö"), "multi-byte characters are fine");
}

#[test]
fn typing_a_word_saves_when_the_space_after_it_is_typed_and_not_before() {
    let (mut m, mut w, dir) = open();
    assert_eq!(saves(&type_text(&mut m, "body wor")), 0);
    assert!(is_dirty(m.state.editor.as_ref().unwrap()));
    let fx = type_text(&mut m, "body word ");
    assert_eq!(saves(&fx), 1);
    run_to_idle(&mut m, &mut w, fx);
    assert!(on_disk(&dir).contains("body word"));
    let e = m.state.editor.as_ref().unwrap();
    assert!(!is_dirty(e) && e.saves == 1 && e.dirty_since.is_none());
}

#[test]
fn text_without_a_whitespace_is_saved_after_five_seconds_and_a_pause_alone_saves_nothing() {
    let (mut m, mut w, dir) = open();
    type_text(&mut m, "body wor");
    m.state.wall_ms = T0 + SAVE_AFTER_MS - 1;
    assert_eq!(
        saves(&apply(&mut m, Msg::Tick)),
        0,
        "a short pause does not save"
    );
    assert!(!on_disk(&dir).contains("wor"));
    m.state.wall_ms = T0 + SAVE_AFTER_MS;
    let fx = apply(&mut m, Msg::Tick);
    assert_eq!(saves(&fx), 1);
    run_to_idle(&mut m, &mut w, fx);
    assert!(on_disk(&dir).contains("body wor"));
}

#[test]
fn only_one_save_runs_and_what_is_typed_meanwhile_is_saved_right_after() {
    let (mut m, mut w, dir) = open();
    let first = type_text(&mut m, "one ");
    assert_eq!(saves(&first), 1);
    let second = type_text(&mut m, "one two ");
    assert_eq!(saves(&second), 0, "the first save is still running");
    let mut fx = first;
    fx.extend(second);
    run_to_idle(&mut m, &mut w, fx);
    assert!(on_disk(&dir).contains("one two"), "{}", on_disk(&dir));
    assert!(!is_dirty(m.state.editor.as_ref().unwrap()));
}

#[test]
fn leaving_the_editor_or_opening_another_item_saves_first() {
    let (mut m, mut w, dir) = open();
    type_text(&mut m, "kept");
    let fx = apply(&mut m, Msg::CloseEditor);
    assert_eq!(saves(&fx), 1);
    assert_eq!(
        m.state.screen,
        Screen::Editor,
        "it leaves once the file is safe"
    );
    run_to_idle(&mut m, &mut w, fx);
    assert!(on_disk(&dir).contains("kept"));
    assert_eq!(m.state.screen, Screen::Library);

    let fx = apply(&mut m, Msg::OpenItem(Uid(0xA)));
    run_to_idle(&mut m, &mut w, fx);
    type_text(&mut m, "again");
    let fx = apply(&mut m, Msg::OpenItem(Uid(0xB)));
    run_to_idle(&mut m, &mut w, fx);
    assert!(on_disk(&dir).contains("again"));
    assert_eq!(m.state.editor.as_ref().unwrap().uid, Uid(0xB));
}

#[test]
fn a_file_changed_elsewhere_is_never_overwritten_until_the_user_chooses() {
    let (mut m, mut w, dir) = open();
    fs::write(dir.path.join("s/alpha.md"), ALPHA.replace("body", "theirs")).unwrap();
    let fx = type_text(&mut m, "mine ");
    run_to_idle(&mut m, &mut w, fx);
    let e = m.state.editor.as_ref().unwrap();
    assert!(e.conflict && e.fail.is_some() && is_dirty(e));
    assert!(on_disk(&dir).contains("theirs"));
    m.state.wall_ms = T0 + 10 * SAVE_AFTER_MS;
    assert_eq!(
        saves(&apply(&mut m, Msg::Tick)),
        0,
        "no retry on a conflict"
    );
    let fx = apply(&mut m, Msg::CloseEditor);
    assert_eq!(saves(&fx), 0);
    assert_eq!(
        m.state.screen,
        Screen::Editor,
        "it stays so nothing is lost"
    );

    let fx = apply(&mut m, Msg::KeepMineClicked);
    run_to_idle(&mut m, &mut w, fx);
    assert!(on_disk(&dir).contains("mine"));
    let e = m.state.editor.as_ref().unwrap();
    assert!(!e.conflict && e.fail.is_none() && !is_dirty(e));
}

#[test]
fn a_failed_save_is_shown_and_retried_after_the_limit() {
    let (mut m, mut w, dir) = open();
    fs::remove_file(dir.path.join("s/alpha.md")).unwrap();
    let fx = type_text(&mut m, "lost ");
    run_to_idle(&mut m, &mut w, fx);
    let e = m.state.editor.as_ref().unwrap();
    assert!(e.fail.is_some() && !e.conflict && is_dirty(e));
    m.state.wall_ms = T0 + SAVE_AFTER_MS;
    assert_eq!(saves(&apply(&mut m, Msg::Tick)), 1, "it tries again");
}

#[test]
fn leaving_after_a_save_raises_one_toast_that_goes_away_by_itself() {
    let (mut m, mut w, _dir) = open();
    let fx = type_text(&mut m, "one ");
    run_to_idle(&mut m, &mut w, fx);
    let fx = type_text(&mut m, "one two ");
    run_to_idle(&mut m, &mut w, fx);
    assert!(saved_toasts(&m).is_empty(), "no toast for each auto-save");
    let fx = apply(&mut m, Msg::CloseEditor);
    run_to_idle(&mut m, &mut w, fx);
    assert_eq!(saved_toasts(&m), vec!["Saved Alpha".to_owned()]);
    apply(&mut m, Msg::Tick);
    assert_eq!(saved_toasts(&m).len(), 1, "still there right away");
    m.state.wall_ms += studyfrog_ui::editor_leave::TOAST_MS + 1;
    apply(&mut m, Msg::Tick);
    assert!(saved_toasts(&m).is_empty());
}

#[test]
fn leaving_without_a_save_raises_no_toast() {
    let (mut m, mut w, _dir) = open();
    let fx = apply(&mut m, Msg::CloseEditor);
    run_to_idle(&mut m, &mut w, fx);
    assert!(saved_toasts(&m).is_empty());
}
