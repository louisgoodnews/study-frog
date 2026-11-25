use studyfrog_log::{Level, name, parse_level};

#[test]
fn levels_order_from_most_to_least_severe() {
    assert!(Level::Error < Level::Warn);
    assert!(Level::Warn < Level::Info);
    assert!(Level::Info < Level::Debug);
    assert!(Level::Debug < Level::Trace);
}

#[test]
fn a_level_is_enabled_when_it_is_at_or_above_the_minimum_in_severity() {
    let minimum = Level::Info;
    assert!(Level::Error <= minimum);
    assert!(Level::Warn <= minimum);
    assert!(Level::Info <= minimum);
    assert!(!(Level::Debug <= minimum));
    assert!(!(Level::Trace <= minimum));
}

#[test]
fn names_are_upper_case() {
    assert_eq!(name(Level::Error), "ERROR");
    assert_eq!(name(Level::Trace), "TRACE");
}

#[test]
fn parsing_ignores_case_and_surrounding_spaces() {
    assert_eq!(parse_level("  WARNING "), Some(Level::Warn));
    assert_eq!(parse_level("debug"), Some(Level::Debug));
    assert_eq!(parse_level("Trace"), Some(Level::Trace));
    assert_eq!(parse_level("verbose"), None);
    assert_eq!(parse_level(""), None);
}
