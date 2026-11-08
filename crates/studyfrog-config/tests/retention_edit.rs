#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Setting `request_retention` in a settings file without touching the rest (STORY-0106).

use studyfrog_config::retention_edit::with_request_retention;
use studyfrog_config::stack::parse_stack_settings;
use studyfrog_config::vault::parse_vault_settings;

#[test]
fn a_stack_line_is_replaced_and_comments_stay() {
    let text =
        "# my stack\nscheduler = \"fsrs\"\nrequest_retention = 0.9 # was\ndaily_new_limit = 5\n";
    let out = with_request_retention(text, 0.86, None);
    assert_eq!(
        out,
        "# my stack\nscheduler = \"fsrs\"\nrequest_retention = 0.86\ndaily_new_limit = 5\n"
    );
    assert_eq!(
        parse_stack_settings(&out).unwrap().request_retention,
        Some(0.86)
    );
    assert_eq!(
        with_request_retention("", 0.9, None),
        "request_retention = 0.9\n"
    );
}

#[test]
fn the_vault_line_goes_into_the_srs_table() {
    let text = "# vault\n\n[srs]\nscheduler = \"fsrs\"\nrequest_retention = 0.9\nmaximum_interval_days = 365\n";
    let out = with_request_retention(text, 0.84, Some("srs"));
    assert_eq!(out, text.replace("0.9", "0.84"));
    let s = parse_vault_settings(&out).unwrap();
    assert!((s.request_retention - 0.84).abs() < 1e-6);
    assert_eq!(s.maximum_interval_days, 365);
    let without = "[srs]\nscheduler = \"sm2\"\n";
    assert_eq!(
        with_request_retention(without, 0.88, Some("srs")),
        "[srs]\nrequest_retention = 0.88\nscheduler = \"sm2\"\n"
    );
    let empty = with_request_retention("# nothing yet\n", 0.92, Some("srs"));
    assert_eq!(empty, "# nothing yet\n\n[srs]\nrequest_retention = 0.92\n");
    assert!((parse_vault_settings(&empty).unwrap().request_retention - 0.92).abs() < 1e-6);
}

#[test]
fn a_key_that_only_starts_alike_is_left_alone() {
    assert_eq!(
        with_request_retention("request_retention_note = 1\n", 0.9, None),
        "request_retention_note = 1\nrequest_retention = 0.9\n"
    );
}
