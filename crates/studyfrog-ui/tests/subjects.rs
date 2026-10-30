#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Teachers and subjects on the settings screen.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_config::subjects::{Subject, SubjectBook};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;
use studyfrog_ui::subjects_ui::SubjectsMsg;

fn model() -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Settings;
    let book = SubjectBook {
        teachers: vec!["Rivera".into()],
        subjects: vec![Subject {
            name: "Maths".into(),
            teacher: Some("Rivera".into()),
        }],
    };
    apply(&mut m, Msg::Subjects(SubjectsMsg::Loaded(Ok((book, None)))));
    m
}

#[test]
fn the_settings_list_teachers_and_subjects_with_their_teacher() {
    let mut h = harness(model());
    h.run_steps(2);
    for text in [
        "Subjects and teachers",
        "Rivera",
        "Maths",
        "Remove teacher Rivera",
        "Remove subject Maths",
        "Teacher of Maths",
    ] {
        assert!(h.query_all_by_label(text).count() >= 1, "missing {text}");
    }
    assert_all_named(&h, "subjects section");
}

#[test]
fn adding_and_removing_store_the_whole_book() {
    let mut m = model();
    apply(
        &mut m,
        Msg::Subjects(SubjectsMsg::TeacherDraft("Chen".into())),
    );
    let fx = apply(&mut m, Msg::Subjects(SubjectsMsg::AddTeacher));
    let [Effect::SaveSubjects(book)] = &fx[..] else {
        panic!("{fx:?}")
    };
    assert_eq!(book.teachers, ["Rivera", "Chen"]);
    apply(
        &mut m,
        Msg::Subjects(SubjectsMsg::SubjectDraft("Physics".into())),
    );
    apply(
        &mut m,
        Msg::Subjects(SubjectsMsg::SubjectTeacher(Some("Chen".into()))),
    );
    let fx = apply(&mut m, Msg::Subjects(SubjectsMsg::AddSubject));
    let [Effect::SaveSubjects(book)] = &fx[..] else {
        panic!("{fx:?}")
    };
    assert_eq!(
        book.subjects[1],
        Subject {
            name: "Physics".into(),
            teacher: Some("Chen".into())
        }
    );
    let fx = apply(
        &mut m,
        Msg::Subjects(SubjectsMsg::SetTeacher("Maths".into(), None)),
    );
    assert!(matches!(&fx[..], [Effect::SaveSubjects(b)] if b.subjects[0].teacher.is_none()));
    assert!(
        apply(&mut m, Msg::Subjects(SubjectsMsg::AddTeacher)).is_empty(),
        "an empty name is refused"
    );
    assert!(
        m.state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("teacher's name")
    );
}

#[test]
fn a_failed_save_is_reported_and_the_old_book_stays() {
    let mut m = model();
    apply(
        &mut m,
        Msg::Subjects(SubjectsMsg::Loaded(Err("disk full".into()))),
    );
    assert!(m.state.notice.as_ref().unwrap().text.contains("disk full"));
    assert_eq!(m.state.subjects.book.teachers, ["Rivera"]);
}

fn open_doc(m: &mut Model, path: &str, text: &str) {
    use studyfrog_core::uid::Uid;
    use studyfrog_service::document::LoadedDoc;
    let doc = studyfrog_md::parse::parse_doc(text).unwrap();
    apply(
        m,
        Msg::DocumentLoaded(
            Uid(0xA),
            Ok(LoadedDoc {
                path: path.into(),
                doc,
                hash: 1,
            }),
        ),
    );
}

#[test]
fn a_stack_page_picks_subjects_and_saves_them_in_its_front_matter() {
    let mut m = model();
    open_doc(
        &mut m,
        "french/_stack.md",
        "---\nid: 0000000000000000000000000A\nkind: stack\ntitle: French\n---\n",
    );
    let mut h = harness(m);
    h.run_steps(2);
    h.get_by_label("Subject: Maths").click();
    h.run_steps(2);
    let e = h.state().0.state.editor.as_ref().unwrap();
    assert_eq!(e.form.subjects, ["Maths"]);
    let saved = studyfrog_ui::edit_form::apply_form(&e.doc, &e.form).unwrap();
    assert_eq!(
        studyfrog_md::extra::extra_value(&saved, "subjects").as_deref(),
        Some("Maths")
    );
    h.get_by_label("Subject: Maths").click();
    h.run_steps(2);
    let e = h.state().0.state.editor.as_ref().unwrap();
    let saved = studyfrog_ui::edit_form::apply_form(&e.doc, &e.form).unwrap();
    assert_eq!(studyfrog_md::extra::extra_value(&saved, "subjects"), None);
    assert_all_named(&h, "stack page with subjects");
}

#[test]
fn an_item_page_shows_the_subjects_and_teacher_of_its_stack() {
    let mut m = model();
    m.state.vault.as_mut().unwrap().overview.stacks[0].subjects = vec!["Maths".into()];
    let folder = m.state.vault.as_ref().unwrap().overview.stacks[0]
        .folder
        .clone();
    open_doc(
        &mut m,
        &format!("{folder}/c.md"),
        "---\nid: 0000000000000000000000000A\nkind: card\ntitle: C\n---\nq\n???\na\n",
    );
    let mut h = harness(m);
    h.run_steps(2);
    assert!(h.query_by_label("Subjects (from the stack)").is_some());
    assert!(h.query_by_label("Maths, taught by Rivera").is_some());
}
