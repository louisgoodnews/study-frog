#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0290: a session starts from a stack, a note and a PDF and is stored; refused without text or a teacher; refused without consent.

mod classroom_common;

use classroom_common::*;

use std::cell::RefCell;

use studyfrog_core::classroom::{Participant, SourceKind, SourceRef};
use studyfrog_core::uid::Uid;
use studyfrog_service::classroom_context::Lecture;
use studyfrog_service::classroom_turns::start_session;
use studyfrog_service::classroom_turns::{advance_segment, simulated_question, user_message};
use studyfrog_service::create::IdSource;

#[test]
fn a_session_starts_from_a_stack_a_note_and_a_pdf_and_is_stored() {
    let (_d, s) = vault("start");
    let id = start(&s);
    let session = stored(&s, id);
    assert_eq!(session.title, "Cell lecture");
    assert_eq!(session.sources, sources());
    assert_eq!(session.participants.len(), 4);
    assert!(session.turns.is_empty());
}

#[test]
fn a_session_is_refused_without_text_to_teach_from_or_without_a_teacher() {
    let (_d, s) = vault("refuse");
    let ids = || IdSource {
        now_ms: 1_000,
        random: [4; 10],
    };
    let missing = [SourceRef {
        kind: SourceKind::Stack,
        uid: Uid(99),
    }];
    assert!(start_session(&s, ids(), "x", &missing, people()).is_err());
    assert!(start_session(&s, ids(), "x", &[], people()).is_err());
    let no_teacher = vec![Participant::SimulatedStudent {
        name: "Ada".to_owned(),
    }];
    assert!(start_session(&s, ids(), "x", &sources(), no_teacher).is_err());
}

#[test]
fn a_remote_provider_without_consent_is_never_called_and_stores_nothing() {
    let (_d, s) = vault("consent");
    let id = start(&s);
    let p = remote(false);
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    assert!(advance_segment(&lecture, &mut fake(&log), AT).is_err());
    assert!(simulated_question(&lecture, &mut fake(&log), AT, 1).is_err());
    assert!(user_message(&lecture, &mut fake(&log), AT, 3, "hi").is_err());
    assert!(log.borrow().is_empty());
    assert!(stored(&s, id).turns.is_empty());
}
