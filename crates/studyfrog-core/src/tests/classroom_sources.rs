use crate::classroom::{ClassroomError, SourceKind, SourceRef};
use crate::classroom_ops::{add_source, new_session};
use crate::time::UnixMs;
use crate::uid::Uid;

fn session() -> crate::classroom::ClassroomSession {
    new_session(
        Uid(1),
        "Lecture 1",
        UnixMs(0),
        vec![crate::classroom::Participant::Teacher {
            name: "Prof".into(),
        }],
    )
    .expect("valid session")
}

#[test]
fn a_session_lists_each_vault_source_once_in_order() {
    let mut s = session();
    let stack = SourceRef {
        kind: SourceKind::Stack,
        uid: Uid(5),
    };
    let note = SourceRef {
        kind: SourceKind::Note,
        uid: Uid(5),
    };
    add_source(&mut s, stack).unwrap();
    add_source(&mut s, note).unwrap();
    assert_eq!(
        add_source(&mut s, stack),
        Err(ClassroomError::DuplicateSource)
    );
    assert_eq!(
        s.sources,
        vec![stack, note],
        "a stack and a note may share an id"
    );
}
