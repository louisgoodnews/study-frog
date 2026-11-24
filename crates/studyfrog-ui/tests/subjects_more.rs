#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Teachers and subjects on the settings screen.
use common::views::open_model;
use studyfrog_config::subjects::{Subject, SubjectBook};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::effect::EffectVault;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;
use studyfrog_ui::subjects_ui::SubjectsMsg;

mod common;

fn model() -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Settings;
    m.state.app.settings_tab = Some("subjects".into());
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

fn type_and_add(m: &mut Model, name: &str) -> Vec<Effect> {
    apply(m, Msg::Subjects(SubjectsMsg::TeacherDraft(name.into())));
    apply(m, Msg::Subjects(SubjectsMsg::AddTeacher))
}

#[test]
fn a_teacher_with_a_comma_is_added_and_can_be_given_to_a_subject() {
    let mut m = model();
    let fx = type_and_add(&mut m, "Prof. Dr. Künkelmann, Harry");
    let [Effect::Vault(EffectVault::SaveSubjects(book))] = &fx[..] else {
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
    assert!(
        matches!(&fx[..], [Effect::Vault(EffectVault::SaveSubjects(b))]
        if b.subjects[0].teacher.as_deref() == Some("Prof. Dr. Künkelmann, Harry"))
    );
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
