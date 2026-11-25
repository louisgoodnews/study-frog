//! Choice blocks: single and multiple, the reported errors, and option order (STORY-0220).

use studyfrog_md::choice::{ChoiceError, correct_indices, is_multiple, parse_choice};

const SINGLE: &str = "Which organelle makes ATP?\n- [ ] Ribosome\n- [x] Mitochondrion\n- [ ] Golgi apparatus\n";

#[test]
fn single_correct_option_is_single_choice() {
    let block = parse_choice(SINGLE).unwrap();
    assert_eq!(block.prompt, "Which organelle makes ATP?");
    assert_eq!(block.options.len(), 3);
    assert_eq!(block.options.iter().filter(|o| o.correct).count(), 1);
    assert!(!is_multiple(&block));
    assert_eq!(correct_indices(&block), vec![1]);
}

#[test]
fn two_correct_options_is_multiple_select() {
    let body = "Which are mammals?\n- [x] Whale\n- [ ] Shark\n- [x] Bat\n";
    let block = parse_choice(body).unwrap();
    assert!(is_multiple(&block));
    assert_eq!(correct_indices(&block), vec![0, 2]);
}

#[test]
fn all_options_correct_is_multiple_select() {
    let body = "Pick the primes.\n- [x] 2\n- [x] 3\n- [x] 5\n";
    let block = parse_choice(body).unwrap();
    assert!(is_multiple(&block));
    assert_eq!(correct_indices(&block), vec![0, 1, 2]);
}

#[test]
fn uppercase_x_marks_a_correct_option() {
    let block = parse_choice("Q?\n- [X] yes\n- [ ] no\n").unwrap();
    assert!(block.options[0].correct);
    assert!(!block.options[1].correct);
}

#[test]
fn no_correct_option_is_reported() {
    let err = parse_choice("Q?\n- [ ] a\n- [ ] b\n").unwrap_err();
    assert_eq!(err, ChoiceError::NoCorrect);
}

#[test]
fn body_without_options_is_not_a_choice_block() {
    assert_eq!(
        parse_choice("Just a sentence about cells.").unwrap_err(),
        ChoiceError::NoOptions
    );
    assert_eq!(parse_choice("").unwrap_err(), ChoiceError::NoOptions);
}

#[test]
fn ordinary_list_is_not_a_choice_block() {
    let body = "Steps:\n- first thing\n- second thing\n";
    assert_eq!(parse_choice(body).unwrap_err(), ChoiceError::NoOptions);
}

#[test]
fn missing_prompt_is_reported() {
    assert_eq!(
        parse_choice("- [x] a\n- [ ] b\n").unwrap_err(),
        ChoiceError::NoPrompt
    );
}

#[test]
fn empty_option_is_reported() {
    assert_eq!(
        parse_choice("Q?\n- [x]\n- [ ] b\n").unwrap_err(),
        ChoiceError::EmptyOption
    );
}

#[test]
fn text_after_the_list_is_reported() {
    let body = "Q?\n- [x] a\n- [ ] b\nand then more prose\n";
    assert_eq!(parse_choice(body).unwrap_err(), ChoiceError::TextAfterOptions);
}

#[test]
fn blank_lines_between_options_are_ignored() {
    let block = parse_choice("Q?\n\n- [x] a\n\n- [ ] b\n").unwrap();
    assert_eq!(block.options.len(), 2);
}

#[test]
fn option_order_and_text_are_kept() {
    let body = "Order?\n- [ ] Zeta  with   spaces\n- [x] Alpha\n- [ ] Mid *emphasis*\n";
    let block = parse_choice(body).unwrap();
    let texts: Vec<&str> = block.options.iter().map(|o| o.text.as_str()).collect();
    assert_eq!(texts, vec!["Zeta  with   spaces", "Alpha", "Mid *emphasis*"]);
    let flags: Vec<bool> = block.options.iter().map(|o| o.correct).collect();
    assert_eq!(flags, vec![false, true, false]);
}

#[test]
fn multiline_prompt_is_kept_before_the_list() {
    let body = "First line.\nSecond line.\n- [x] a\n- [ ] b\n";
    let block = parse_choice(body).unwrap();
    assert_eq!(block.prompt, "First line.\nSecond line.");
}
