use studyfrog_log::{Level, Record, format_line};

fn record(message: &str, fields: &[(&str, &str)]) -> Record {
    Record {
        level: Level::Info,
        target: "ui".to_string(),
        message: message.to_string(),
        fields: fields
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect(),
        unix_ms: 1500,
    }
}

#[test]
fn plain_fields_are_not_quoted() {
    let line = format_line(&record("opened stack", &[("count", "3")]));
    assert_eq!(line, "1500 INFO ui opened stack count=3");
}

#[test]
fn values_with_spaces_are_quoted() {
    let line = format_line(&record("opened", &[("folder", "My Notes")]));
    assert_eq!(line, r#"1500 INFO ui opened folder="My Notes""#);
}

#[test]
fn quotes_and_backslashes_inside_a_quoted_value_are_escaped() {
    let line = format_line(&record("x", &[("v", r#"say "hi" \o/"#)]));
    assert!(line.ends_with(r#"v="say \"hi\" \\o/""#), "{line}");
}

#[test]
fn empty_and_equals_values_are_quoted() {
    assert!(format_line(&record("x", &[("k", "")])).ends_with(r#"k="""#));
    assert!(format_line(&record("x", &[("k", "a=b")])).ends_with(r#"k="a=b""#));
}

#[test]
fn a_newline_in_the_message_is_escaped_so_the_record_stays_on_one_line() {
    let line = format_line(&record("line one\nline two", &[]));
    assert_eq!(line, r"1500 INFO ui line one\nline two");
    assert!(!line.contains('\n'));
}
