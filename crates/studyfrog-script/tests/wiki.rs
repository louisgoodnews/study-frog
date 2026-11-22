#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The FrogScript reference (STORY-0171). The drift tests read the source of the language itself, so a
//! new keyword or standard-library function without an entry fails here.

use std::collections::BTreeSet;

use studyfrog_script::wiki::{CATEGORIES, Category, label, search, topics};

/// The names an `=> ` arm of a match starts with, for lines indented by `indent` spaces.
fn quoted_names(source: &str, indent: usize, arrow: &str) -> BTreeSet<String> {
    let pad = " ".repeat(indent);
    source
        .lines()
        .filter(|l| l.starts_with(&pad) && !l.starts_with(&format!("{pad} ")))
        .filter(|l| l.contains(arrow))
        .filter_map(|l| {
            let head = l.trim_start().strip_prefix('"')?;
            Some(head.split('"').next()?.to_owned())
        })
        .collect()
}

#[test]
fn every_standard_library_function_has_an_entry() {
    let names = quoted_names(include_str!("../src/eval_stdlib.rs"), 8, "=>");
    assert!(names.len() >= 12, "the scan found the functions: {names:?}");
    for name in names {
        assert!(
            topics()
                .iter()
                .any(|t| t.name == name && t.category == Category::BuiltinFunctions),
            "no built-in entry for the function {name}"
        );
    }
}

#[test]
fn every_keyword_has_an_entry() {
    let keywords: BTreeSet<String> = include_str!("../src/token.rs")
        .lines()
        .filter(|l| l.contains("=> Some(Token::"))
        .filter_map(|l| {
            let head = l.trim_start().strip_prefix('"')?;
            Some(head.split('"').next()?.to_owned())
        })
        .collect();
    assert!(
        keywords.len() >= 20,
        "the scan found the keywords: {keywords:?}"
    );
    for kw in keywords {
        assert!(
            topics().iter().any(|t| t.name == kw),
            "no entry for the keyword {kw}"
        );
    }
}

#[test]
fn every_category_has_topics_and_the_six_headings_are_listed() {
    let labels: Vec<&str> = CATEGORIES.iter().map(|c| label(*c)).collect();
    assert_eq!(
        labels,
        [
            "Syntax",
            "Operators",
            "Control Flow",
            "Built-in Functions",
            "Vault Objects",
            "Lifecycle Events"
        ]
    );
    for c in CATEGORIES {
        assert!(
            topics().iter().any(|t| t.category == c),
            "{} has no topics",
            label(c)
        );
    }
}

#[test]
fn names_are_unique_and_every_entry_is_complete() {
    let mut seen = BTreeSet::new();
    for t in topics() {
        assert!(
            seen.insert((t.name, label(t.category))),
            "{} listed twice",
            t.name
        );
        assert!(
            !t.signature.is_empty() && !t.returns.is_empty(),
            "{}",
            t.name
        );
        assert!(t.summary.len() > 20, "{} needs a sentence", t.name);
        assert!(!t.example.is_empty(), "{} needs an example", t.name);
    }
}

#[test]
fn search_finds_by_name_return_type_or_category_ignoring_case() {
    let names = |q: &str| search(q).iter().map(|t| t.name).collect::<Vec<_>>();
    assert!(names("LEN").contains(&"len"), "by name, any case");
    assert!(names("bool").contains(&"tag"), "by return type");
    assert!(names("vault objects").contains(&"vault"), "by category");
    assert!(
        names("   ").len() == topics().len(),
        "an empty query finds everything"
    );
    assert!(names("zzz-no-such-topic").is_empty());
}
