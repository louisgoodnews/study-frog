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

fn type_and_add(m: &mut Model, name: &str) -> Vec<Effect> {
    apply(m, Msg::Subjects(SubjectsMsg::TeacherDraft(name.into())));
    apply(m, Msg::Subjects(SubjectsMsg::AddTeacher))
}

#[test]
fn a_teacher_with_a_comma_is_added_and_can_be_given_to_a_subject() {
    let mut m = model();
    let fx = type_and_add(&mut m, "Prof. Dr. Künkelmann, Harry");
    let [Effect::SaveSubjects(book)] = &fx[..] else {
        panic!("{fx:?}")
    };
    assert_eq!(book.teachers, ["Rivera", "Prof. Dr. Künkelmann, Harry"]);
    assert_eq!(m.state.subjects.teacher_draft, "");
    let fx = apply(
        &mut m,
        Msg::Subjects(SubjectsMsg::SetTeacher(
            "Maths".into(),
            Some("Prof. Dr. Künkelmann, Harry".into()),
        )),
    );
    assert!(matches!(&fx[..], [Effect::SaveSubjects(b)]
        if b.subjects[0].teacher.as_deref() == Some("Prof. Dr. Künkelmann, Harry")));
}

#[test]
fn a_name_that_cannot_be_stored_is_refused_at_the_field_and_the_text_stays() {
    let mut m = model();
    for (name, why) in [("Rivera", "twice"), ("Two\nLines", "one line")] {
        let fx = type_and_add(&mut m, name);
        assert!(fx.is_empty(), "nothing is saved for {name:?}");
        assert_eq!(m.state.subjects.teacher_draft, name, "the typed text stays");
        let shown = m.state.subjects.teacher_error.clone().unwrap();
        assert!(shown.contains(why), "{shown}");
        assert!(m.state.notice.is_none(), "no notice in the bell");
    }
    apply(
        &mut m,
        Msg::Subjects(SubjectsMsg::TeacherDraft("Chen".into())),
    );
    assert!(
        m.state.subjects.teacher_error.is_none(),
        "typing clears the message"
    );
}

#[test]
fn a_subject_name_is_checked_before_saving_too() {
    let mut m = model();
    apply(
        &mut m,
        Msg::Subjects(SubjectsMsg::SubjectDraft("maths".into())),
    );
    let fx = apply(&mut m, Msg::Subjects(SubjectsMsg::AddSubject));
    assert!(fx.is_empty());
    assert!(
        m.state
            .subjects
            .subject_error
            .as_ref()
            .unwrap()
            .contains("twice")
    );
    assert_eq!(m.state.subjects.subject_draft, "maths");
}

#[test]
fn the_notice_shows_the_error_text_as_given_so_a_load_is_not_called_a_save() {
    let mut m = model();
    apply(
        &mut m,
        Msg::Subjects(SubjectsMsg::Loaded(Err(
            "Could not load the subjects: gone".into(),
        ))),
    );
    assert_eq!(
        m.state.notice.as_ref().unwrap().text,
        "Could not load the subjects: gone"
    );
}
