use super::*;

fn row(text: &str, correct: bool) -> Row {
    Row {
        text: text.to_owned(),
        correct,
    }
}

#[test]
fn open_body_is_the_prompt_unchanged() {
    assert_eq!(
        create_body("What is 2+2?", AnswerType::Open),
        "What is 2+2?"
    );
}

#[test]
fn single_template_has_one_wrong_and_one_correct_empty_option() {
    assert_eq!(
        create_body("Pick one", AnswerType::Single),
        "Pick one\n\n- [ ] \n- [x] "
    );
}

#[test]
fn multiple_template_has_two_correct_empty_options() {
    assert_eq!(
        create_body("Pick all", AnswerType::Multiple),
        "Pick all\n\n- [x] \n- [x] "
    );
}

#[test]
fn prompt_with_options_keeps_its_options() {
    let body = "Q\n\n- [ ] a\n- [x] b";
    assert_eq!(create_body(body, AnswerType::Multiple), body);
}

#[test]
fn rendered_options_are_read_back_as_the_same_options() {
    let rows = vec![row("Ribosome", false), row("Mitochondrion", true)];
    let body = render("Which organelle makes ATP?", &rows);
    let block = block_of(&body).expect("the rendered body is a choice");
    assert_eq!(rows_of(&block), rows);
}

#[test]
fn an_empty_prompt_renders_the_options_alone() {
    assert_eq!(render("", &[row("a", true)]), "- [x] a");
}

#[test]
fn zero_correct_options_do_not_agree_with_any_choice_type() {
    let none = vec![row("a", false), row("b", false)];
    assert!(!agrees(AnswerType::Single, &none));
    assert!(!agrees(AnswerType::Multiple, &none));
}

#[test]
fn one_correct_option_agrees_with_single_only() {
    let one = vec![row("a", true), row("b", false)];
    assert!(agrees(AnswerType::Single, &one));
    assert!(!agrees(AnswerType::Multiple, &one));
}

#[test]
fn open_text_has_no_options() {
    assert!(agrees(AnswerType::Open, &[]));
    assert!(!agrees(AnswerType::Open, &[row("a", true)]));
}

#[test]
fn open_text_confirmation_applies_only_to_choice_bodies() {
    assert!(open_text_needs_confirm("Q\n\n- [ ] a\n- [x] b"));
    assert!(!open_text_needs_confirm("Just a prompt"));
}

#[test]
fn open_text_body_drops_the_option_markers() {
    assert_eq!(open_text_body("Q\n\n- [ ] a\n- [x] b"), "Q");
}

#[test]
fn answer_type_names_round_trip() {
    for t in [AnswerType::Open, AnswerType::Single, AnswerType::Multiple] {
        assert_eq!(answer_type_named(answer_type_name(t)), Some(t));
    }
    assert_eq!(answer_type_named("essay"), None);
    assert_eq!(stated_type(None), AnswerType::Open);
}
