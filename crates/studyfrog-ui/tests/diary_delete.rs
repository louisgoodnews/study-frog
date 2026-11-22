#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Deleting a diary entry asks first (the entry editor's delete confirmation). Pure update, no I/O.

use studyfrog_core::uid::Uid;
use studyfrog_ui::diary_msg::DiaryMsg;
use studyfrog_ui::diary_types::DiaryEffect;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::State;
use studyfrog_ui::update::update;

fn send(state: State, m: DiaryMsg) -> (State, Vec<Effect>) {
    update(state, Msg::Diary(m))
}

#[test]
fn asking_to_delete_sends_nothing_until_it_is_confirmed() {
    let (s, fx) = send(State::default(), DiaryMsg::AskDeleteEntry(Uid(5)));
    assert!(fx.is_empty(), "asking deletes nothing");
    assert_eq!(s.diary.delete_pending, Some(Uid(5)));

    let (s, fx) = send(s, DiaryMsg::DeleteEntry(Uid(5)));
    assert_eq!(fx, vec![Effect::Diary(DiaryEffect::DeleteEntry(Uid(5)))]);
    assert_eq!(s.diary.delete_pending, None, "the question is answered");
}

#[test]
fn keeping_the_entry_clears_the_question_and_deletes_nothing() {
    let (s, _) = send(State::default(), DiaryMsg::AskDeleteEntry(Uid(5)));
    let (s, fx) = send(s, DiaryMsg::KeepEntry);
    assert!(fx.is_empty());
    assert_eq!(s.diary.delete_pending, None);
}

#[test]
fn closing_the_editor_clears_a_pending_question() {
    let (s, _) = send(State::default(), DiaryMsg::AskDeleteEntry(Uid(5)));
    let (s, fx) = send(s, DiaryMsg::CloseEditor);
    assert!(fx.is_empty());
    assert_eq!(s.diary.delete_pending, None);
}
