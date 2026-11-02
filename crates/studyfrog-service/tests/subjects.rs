#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Teachers and subjects of a vault, and what stacks inherit.

use std::fs;

use studyfrog_config::subjects::{Subject, SubjectBook};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::session::open_vault;
use studyfrog_service::subjects::{
    load_subjects, save_subjects, stack_subjects, subjects_with_teachers,
};

fn write(dir: &std::path::Path, rel: &str, n: u128, extra: &str) {
    let text = format!(
        "---\nid: {}\nkind: stack\ntitle: S{n}\n{extra}---\n",
        format_ulid(Uid(n))
    );
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[test]
fn the_book_is_saved_checked_and_loaded_and_a_removed_teacher_clears_the_subject() {
    let dir = std::env::temp_dir().join(format!("studyfrog-subjects-{}", std::process::id()));
    write(&dir, "a/_stack.md", 1, "subjects: Maths, Physics\n");
    write(&dir, "a/b/_stack.md", 2, "");
    write(&dir, "a/c/_stack.md", 3, "subjects: Art\n");
    write(&dir, "z/_stack.md", 4, "");
    let (session, info) = open_vault(dir.to_str().unwrap()).unwrap();
    assert_eq!(load_subjects(&session), (SubjectBook::default(), None));

    let book = SubjectBook {
        teachers: vec!["Rivera".into(), "Chen".into()],
        subjects: vec![
            Subject {
                name: "Maths".into(),
                teacher: Some("Rivera".into()),
            },
            Subject {
                name: "Physics".into(),
                teacher: Some("Chen".into()),
            },
        ],
    };
    assert_eq!(save_subjects(&session, book.clone()).unwrap(), book);
    assert_eq!(load_subjects(&session).0, book);
    let mut fewer = book.clone();
    fewer.teachers.retain(|t| t != "Chen");
    let saved = save_subjects(&session, fewer).unwrap();
    assert_eq!(saved.subjects[1].teacher, None);
    let mut bad = book.clone();
    bad.subjects.push(Subject {
        name: "maths".into(),
        teacher: None,
    });
    assert!(save_subjects(&session, bad).is_err());

    let stacks = &info.overview.stacks;
    assert_eq!(stack_subjects(stacks, "a"), ["Maths", "Physics"]);
    assert_eq!(
        stack_subjects(stacks, "a/b"),
        ["Maths", "Physics"],
        "inherited from the stack above"
    );
    assert_eq!(stack_subjects(stacks, "a/c"), ["Art"], "its own wins");
    assert!(stack_subjects(stacks, "z").is_empty());
    let shown = subjects_with_teachers(&book, &["Maths".into(), "Art".into()]);
    assert_eq!(
        shown,
        [
            ("Maths".to_owned(), Some("Rivera".to_owned())),
            ("Art".to_owned(), None)
        ]
    );
    fs::write(dir.join("subjects.toml"), "oops = 1").unwrap();
    assert!(
        load_subjects(&session)
            .1
            .unwrap()
            .contains("subjects.toml is not used")
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_professor_with_a_comma_can_teach_a_subject_in_a_stack_and_survives_a_reload() {
    let dir = std::env::temp_dir().join(format!("studyfrog-commas-{}", std::process::id()));
    let prof = "Prof. Dr. Künkelmann, Harry";
    let subject = "Test theory, basics";
    let list = format!("subjects: [\"{subject}\", Maths]\n");
    write(&dir, "a/_stack.md", 1, &list);
    let (session, info) = open_vault(dir.to_str().unwrap()).unwrap();
    let stacks = &info.overview.stacks;
    assert_eq!(stack_subjects(stacks, "a"), [subject, "Maths"]);

    let book = SubjectBook {
        teachers: vec![prof.into()],
        subjects: vec![
            Subject {
                name: subject.into(),
                teacher: Some(prof.into()),
            },
            Subject {
                name: "Maths".into(),
                teacher: None,
            },
        ],
    };
    assert_eq!(save_subjects(&session, book.clone()).unwrap(), book);
    let (loaded, problem) = load_subjects(&session);
    assert_eq!((loaded.clone(), problem), (book, None));
    let shown = subjects_with_teachers(&loaded, &stack_subjects(stacks, "a"));
    assert_eq!(shown[0], (subject.to_owned(), Some(prof.to_owned())));
    let _ = fs::remove_dir_all(&dir);
}
