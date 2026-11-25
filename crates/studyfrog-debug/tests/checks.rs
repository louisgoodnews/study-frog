//! Debug checks and counters (studyfrog-debug).
use studyfrog_debug::{check, count, counts, truncate_chars};

#[test]
fn check_returns_its_condition_and_never_panics() {
    assert!(check("debug-check", true, "fine"));
    assert!(!check(
        "debug-check",
        false,
        "expected to fail; this is logged at Error"
    ));
}

#[test]
fn counts_are_sorted_by_name_and_add_up() {
    count("debug-zeta");
    count("debug-alpha");
    count("debug-alpha");

    let ours: Vec<(String, u64)> = counts()
        .into_iter()
        .filter(|(name, _)| name.starts_with("debug-"))
        .collect();
    assert_eq!(
        ours,
        vec![
            ("debug-alpha".to_string(), 2),
            ("debug-zeta".to_string(), 1),
        ]
    );

    let all = counts();
    assert!(all.windows(2).all(|pair| pair[0].0 < pair[1].0));
}

#[test]
fn long_dump_text_is_cut_to_exactly_the_limit_with_an_ellipsis() {
    let long = "a".repeat(2500);
    let cut = truncate_chars(&long, 2000);
    assert_eq!(cut.chars().count(), 2000);
    assert!(cut.ends_with('\u{2026}'));
}

#[test]
fn short_text_and_characters_are_counted_not_bytes() {
    assert_eq!(truncate_chars("short", 2000), "short");
    assert_eq!(
        truncate_chars("\u{e4}\u{e4}\u{e4}\u{e4}\u{e4}", 3),
        "\u{e4}\u{e4}\u{2026}"
    );
}
