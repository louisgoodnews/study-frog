#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! FrogScript's editor help (STORY-0150): colours, bracket matching, the first error, completions.

use studyfrog_script::editor::{
    BUILTINS, Part, completions, error_of, highlight, line_range, matching_bracket, with_completion,
};
use studyfrog_script::error::RuntimeError;
use studyfrog_script::eval_stdlib::call_native_fn;
use studyfrog_script::log_stream::LogBuffer;

fn part_at(src: &str, needle: &str) -> Part {
    let at = src.find(needle).unwrap();
    highlight(src)
        .into_iter()
        .find(|(r, _)| r.contains(&at))
        .map(|(_, p)| p)
        .unwrap()
}

#[test]
fn every_built_in_name_is_a_function_the_interpreter_knows() {
    for name in BUILTINS {
        let mut logs = LogBuffer::default();
        let r = call_native_fn(name, &[], &mut logs, 0, None);
        assert!(
            !matches!(r, Err(RuntimeError::UnknownFunction { .. })),
            "{name} is listed as a built-in but the interpreter does not know it"
        );
    }
}

#[test]
fn the_text_is_coloured_by_what_each_piece_is() {
    let src = "from stack \"French\" where type = card  # the cards\nprint(len(\"x\") + 3)";
    assert_eq!(part_at(src, "from"), Part::Keyword);
    assert_eq!(part_at(src, "\"French\""), Part::Text);
    assert_eq!(part_at(src, "# the cards"), Part::Comment);
    assert_eq!(part_at(src, "print"), Part::Builtin);
    assert_eq!(part_at(src, "3)"), Part::Number);
    // The pieces cover the whole text, in order, without gaps.
    let pieces = highlight(src);
    assert_eq!(pieces.first().unwrap().0.start, 0);
    assert_eq!(pieces.last().unwrap().0.end, src.len());
    assert!(pieces.windows(2).all(|w| w[0].0.end == w[1].0.start));
}

#[test]
fn a_bracket_matches_its_partner_across_nesting_and_skips_text() {
    let src = "print(len(\"(\") + (1))";
    let open = src.find("print(").unwrap() + "print".len();
    assert_eq!(
        matching_bracket(src, open),
        Some(src.len() - 1),
        "the outer ( matches the last )"
    );
    // The bracket just before the cursor counts too.
    let close = src.len() - 1;
    assert_eq!(matching_bracket(src, close + 1), Some(open));
    assert_eq!(matching_bracket("x = 1", 0), None);
}

#[test]
fn the_first_error_gives_its_line() {
    let src = "x = 1\ny = \"open\nprint(x)";
    let e = error_of(src).unwrap();
    assert_eq!(e.line, Some(2), "{e:?}");
    assert!(
        e.message.contains("unclosed") || e.message.contains("string"),
        "{}",
        e.message
    );
    assert_eq!(error_of("print(1)"), None, "a valid script has no error");
    assert_eq!(line_range("ab\ncd", 2), Some(3..5));
}

#[test]
fn completions_follow_the_word_being_typed() {
    let src = "pr";
    assert_eq!(
        completions(src, 2),
        vec!["print".to_owned(), "priority".to_owned()]
    );
    assert_eq!(
        completions("vault.st", 8),
        vec!["stack".to_owned(), "stacks".to_owned()]
    );
    assert_eq!(
        completions("", 0),
        Vec::<String>::new(),
        "nothing typed, nothing offered"
    );
    assert!(
        completions("x = 1", 5).is_empty(),
        "a finished word offers nothing more"
    );
    assert!(completions("vault.", 6).contains(&"tags".to_owned()));
}

#[test]
fn a_completion_replaces_the_word_before_the_cursor() {
    let (text, at) = with_completion("x = pri", 7, "priority");
    assert_eq!(text, "x = priority");
    assert_eq!(at, 12);
    let (text, _) = with_completion("vault.st", 8, "stacks");
    assert_eq!(text, "vault.stacks");
}
