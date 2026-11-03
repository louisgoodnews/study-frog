#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The highlighter (TASK-0214): the languages of the owner's list, plain for the unknown, never a panic.

use proptest::prelude::*;
use studyfrog_md::blocks::{Block, parse_blocks};
use studyfrog_md::highlight::{Kind, Token, highlight};
use studyfrog_md::highlight_langs::LANGS;

fn kinds(info: &str, code: &str) -> Vec<(Kind, String)> {
    let mut at = 0;
    highlight(info, code)
        .into_iter()
        .map(|Token { kind, len }| {
            let part = code[at..at + len].to_owned();
            at += len;
            (kind, part)
        })
        .collect()
}

fn has(parts: &[(Kind, String)], kind: Kind, text: &str) -> bool {
    parts.iter().any(|(k, t)| *k == kind && t.contains(text))
}

#[test]
fn python_has_keywords_strings_comments_numbers_and_triple_quotes() {
    let code = "def f(x):  # note\n    return \"a\" + 'b' + 42\n\"\"\"doc\nmore\"\"\"\n";
    let p = kinds("python", code);
    assert!(has(&p, Kind::Keyword, "def") && has(&p, Kind::Keyword, "return"));
    assert!(has(&p, Kind::Comment, "# note"));
    assert!(has(&p, Kind::String, "\"a\"") && has(&p, Kind::String, "'b'"));
    assert!(has(&p, Kind::Number, "42"));
    assert!(
        has(&p, Kind::String, "\"\"\"doc\nmore\"\"\""),
        "a triple-quoted string spans lines"
    );
}

#[test]
fn every_language_of_the_owners_list_is_known() {
    for name in [
        "python", "r", "rust", "php", "java", "toml", "html", "json", "csv", "tsv", "cpp",
    ] {
        assert!(LANGS.iter().any(|l| l.names.contains(&name)), "{name}");
    }
    let rust = kinds("rust", "fn main() { let x = 1; /* c */ } // end\n");
    assert!(has(&rust, Kind::Keyword, "fn") && has(&rust, Kind::Keyword, "let"));
    assert!(has(&rust, Kind::Comment, "/* c */") && has(&rust, Kind::Comment, "// end"));
    assert!(has(
        &kinds("java", "public class A { int n = 0x1F; }"),
        Kind::Number,
        "0x1F"
    ));
    assert!(has(
        &kinds("cpp", "#include <a>\nint main() {}"),
        Kind::Keyword,
        "int"
    ));
    assert!(has(
        &kinds("php", "<?php echo $a; # hi\n"),
        Kind::Comment,
        "# hi"
    ));
    assert!(has(
        &kinds("r", "f <- function(x) TRUE # r\n"),
        Kind::Keyword,
        "function"
    ));
}

#[test]
fn data_formats_have_their_own_rules() {
    let toml = kinds(
        "toml",
        "[server]\nport = 8080\nname = \"x\"  # c\nok = true\n",
    );
    assert!(has(&toml, Kind::Keyword, "[server]") && has(&toml, Kind::Number, "8080"));
    assert!(
        has(&toml, Kind::String, "\"x\"")
            && has(&toml, Kind::Comment, "# c")
            && has(&toml, Kind::Keyword, "true")
    );
    let json = kinds("json", "{\"a\": [1, 2.5, null, false], \"b\": \"s\"}");
    assert!(
        has(&json, Kind::String, "\"a\"")
            && has(&json, Kind::Number, "2.5")
            && has(&json, Kind::Keyword, "null")
    );
    let csv = kinds("csv", "name,qty\ncat,3\n");
    assert!(has(&csv, Kind::Keyword, "name,qty") && has(&csv, Kind::Number, "3"));
    let html = kinds("html", "<!-- c --><a href=\"x\">hi</a>");
    assert!(has(&html, Kind::Comment, "<!-- c -->") && has(&html, Kind::Keyword, "<a"));
    assert!(has(&html, Kind::String, "\"x\"") && has(&html, Kind::Keyword, "</a"));
    assert!(
        !has(&html, Kind::Keyword, "hi"),
        "text between tags is plain"
    );
}

#[test]
fn an_unknown_or_missing_language_gives_no_tokens_and_a_fence_carries_them() {
    assert!(highlight("", "fn x() {}").is_empty() && highlight("brainfuck", "+++").is_empty());
    assert!(
        highlight("PYTHON title=\"a\"", "def").len() == 1,
        "the first word, in any case"
    );
    let blocks = parse_blocks("```python\nreturn 1\n```\n\n```\nreturn 1\n```\n");
    let [Block::Code { tokens: a, .. }, Block::Code { tokens: b, .. }] = blocks.as_slice() else {
        panic!()
    };
    assert!(!a.is_empty() && b.is_empty());
}

#[test]
fn a_five_thousand_line_block_is_tokenized_quickly() {
    let line = "def f(x):  # c\n    return \"s\" + 12 * x\n";
    let code = line.repeat(2500);
    let started = std::time::Instant::now();
    let tokens = highlight("python", &code);
    assert!(
        started.elapsed().as_millis() < 500,
        "{:?}",
        started.elapsed()
    );
    assert_eq!(tokens.iter().map(|t| t.len).sum::<usize>(), code.len());
}

proptest! {
    #[test]
    fn tokens_always_cover_the_text_on_character_boundaries(code in "(?s).{0,200}", pick in 0usize..11) {
        let lang = LANGS[pick % LANGS.len()].names[0];
        let tokens = highlight(lang, &code);
        let mut at = 0;
        for t in &tokens {
            prop_assert!(t.len > 0);
            at += t.len;
            prop_assert!(code.is_char_boundary(at));
        }
        prop_assert_eq!(at, code.len());
    }
}
