#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic

//! The application log: line formats, rotation by size and reading the end.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use studyfrog_service::logging::{
    LogEntry, LogLevel, json_line, log_append, rotated_name, rotation_plan, tail_lines, text_line,
};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn temp_dir(label: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("studyfrog-{label}-{}-{n}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn entry(message: &str) -> LogEntry {
    LogEntry {
        at_ms: 1_762_560_000_123,
        level: LogLevel::Error,
        source: "notice".to_owned(),
        message: message.to_owned(),
    }
}

#[test]
fn a_text_line_has_the_time_the_level_the_source_and_one_line_of_message() {
    let line = text_line(&entry("Could not save\nthe note"));
    assert_eq!(
        line,
        "2025-11-08T00:00:00.123Z ERROR notice: Could not save\\nthe note"
    );
}

#[test]
fn a_json_line_is_one_object_with_the_message_unchanged() {
    let value: serde_json::Value = serde_json::from_str(&json_line(&entry("a \"b\"\nc"))).unwrap();
    assert_eq!(value["level"], "error");
    assert_eq!(value["source"], "notice");
    assert_eq!(value["message"], "a \"b\"\nc");
    assert_eq!(value["at_ms"], 1_762_560_000_123_i64);
}

#[test]
fn the_rotation_plan_shifts_the_older_copies_first() {
    let base = PathBuf::from("/l/studyfrog.log");
    assert_eq!(
        rotation_plan(&base, 3),
        vec![
            (rotated_name(&base, 1), rotated_name(&base, 2)),
            (base.clone(), rotated_name(&base, 1)),
        ]
    );
}

#[test]
fn a_full_log_is_rotated_and_only_the_kept_files_remain() {
    let dir = temp_dir("log-rotate");
    for i in 0..30 {
        log_append(&dir, &entry(&format!("line {i:02} of the log")), 200).unwrap();
    }
    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "console.jsonl",
            "console.jsonl.1",
            "console.jsonl.2",
            "studyfrog.log",
            "studyfrog.log.1",
            "studyfrog.log.2",
        ]
    );
    let newest = tail_lines(&dir.join("studyfrog.log"), 1).unwrap();
    assert!(newest[0].ends_with("line 29 of the log"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_tail_is_the_last_lines_oldest_first_and_a_missing_file_has_none() {
    let dir = temp_dir("log-tail");
    for i in 0..5 {
        log_append(&dir, &entry(&format!("m{i}")), 1 << 20).unwrap();
    }
    let lines = tail_lines(&dir.join("studyfrog.log"), 2).unwrap();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].ends_with("m3") && lines[1].ends_with("m4"));
    assert!(tail_lines(&dir.join("absent.log"), 2).unwrap().is_empty());
    let _ = fs::remove_dir_all(&dir);
}
