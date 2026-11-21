#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The import destination: as the file says, the vault root, an existing stack, or a new stack.

use studyfrog_ui::state::State;
use studyfrog_ui::transfer_types::import_target;
use studyfrog_ui::transfer_ui::{TransferMsg, update};

fn send(state: State, m: TransferMsg) -> State {
    update(state, m).0
}

#[test]
fn the_destination_resolves_to_the_import_target() {
    let s = State::default();
    assert_eq!(import_target(&s.transfer), Ok(None), "as the file says");

    let s = send(s, TransferMsg::Scope(Some(String::new())));
    assert_eq!(
        import_target(&s.transfer),
        Ok(Some(String::new())),
        "the vault root"
    );

    let s = send(s, TransferMsg::Scope(Some("french".into())));
    assert_eq!(import_target(&s.transfer), Ok(Some("french".into())));
}

#[test]
fn a_new_stack_needs_a_name_and_replaces_the_scope() {
    let s = send(State::default(), TransferMsg::ChooseNewStack);
    assert!(import_target(&s.transfer).is_err(), "no name yet");
    let s = send(s, TransferMsg::NewStackName("  Spanish/verbs ".into()));
    assert_eq!(import_target(&s.transfer), Ok(Some("Spanish/verbs".into())));
    let s = send(s, TransferMsg::Scope(None));
    assert_eq!(
        import_target(&s.transfer),
        Ok(None),
        "choosing a stack or 'as the file says' leaves the new-stack choice"
    );
}

#[test]
fn an_import_without_a_name_is_refused_with_the_reason() {
    let s = send(State::default(), TransferMsg::ChooseNewStack);
    let (s, fx) = update(s, TransferMsg::Import);
    assert!(fx.is_empty());
    assert!(s.transfer.status.unwrap().contains("Name the new stack"));
    assert!(!s.transfer.busy);
}
