#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for `.studyfrog/homework.toml` parsing, validation, and deterministic rendering.

use studyfrog_config::homework::{
    ChecklistItemDto, HomeworkFileDto, HomeworkItemDto, parse_homework_toml, render_homework_toml,
    sort_homework_dtos, validate_homework_dto,
};

fn dto(id: &str, due_date: &str, due_time: &str) -> HomeworkItemDto {
    HomeworkItemDto {
        id: id.to_owned(),
        title: format!("Task {id}"),
        subject: Some("Physics".to_owned()),
        stack_uid: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_owned()),
        timetable_id: None,
        due_date: due_date.to_owned(),
        due_time: due_time.to_owned(),
        start_date: Some("2026-11-10".to_owned()),
        estimated_minutes: 45,
        priority: "high".to_owned(),
        status: "in_progress".to_owned(),
        notes: "Line 1 \"quoted\" \\ slash\nLine 2\tindented".to_owned(),
        reminders: vec![1440, 15, 1440],
        checklist: vec![
            ChecklistItemDto {
                text: "First step".to_owned(),
                done: true,
            },
            ChecklistItemDto {
                text: "Second step".to_owned(),
                done: false,
            },
        ],
    }
}

#[test]
fn empty_file_parses_to_empty_list() {
    let parsed = parse_homework_toml("").unwrap();
    assert!(parsed.homework.is_empty());
}

#[test]
fn round_trip_preserves_multiline_notes_and_optional_fields() {
    let mut minimal = dto("01ARZ3NDEKTSV4RRFFQ69G5FB1", "2026-11-20", "09:00");
    minimal.subject = None;
    minimal.stack_uid = None;
    minimal.start_date = None;
    minimal.notes.clear();
    minimal.reminders.clear();
    minimal.checklist.clear();

    let full = dto("01ARZ3NDEKTSV4RRFFQ69G5FB2", "2026-11-21", "18:30");
    let rendered = render_homework_toml(&HomeworkFileDto {
        homework: vec![full.clone(), minimal.clone()],
    });
    let reparsed = parse_homework_toml(&rendered).unwrap();
    assert_eq!(reparsed.homework.len(), 2);
    assert_eq!(reparsed.homework[0], minimal);
    assert_eq!(reparsed.homework[1].notes, full.notes);
    assert_eq!(reparsed.homework[1].reminders, vec![15, 1440]);
    assert_eq!(reparsed.homework[1].checklist, full.checklist);
}

#[test]
fn render_is_deterministic_across_input_permutations() {
    let a = dto("01ARZ3NDEKTSV4RRFFQ69G5FB1", "2026-11-20", "08:00");
    let b = dto("01ARZ3NDEKTSV4RRFFQ69G5FB2", "2026-11-20", "12:00");
    let mut b_alt = b.clone();
    b_alt.reminders = vec![15, 1440];
    let r1 = render_homework_toml(&HomeworkFileDto {
        homework: vec![b.clone(), a.clone()],
    });
    let r2 = render_homework_toml(&HomeworkFileDto {
        homework: vec![a, b_alt],
    });
    assert_eq!(r1, r2);
}

#[test]
fn invalid_inputs_and_toml_errors_are_rejected() {
    assert!(parse_homework_toml("[[homework]]\nid = \"a\"\ntitle = \"t\"\ndue_date = \"2026-11-20\"\ndue_time = \"09:00\"\nestimated_minutes = 10\npriority = \"low\"\nstatus = \"not_started\"\nextra = 1\n").is_err());

    let base = dto("01ARZ3NDEKTSV4RRFFQ69G5FB1", "2026-11-20", "09:00");
    let dup = render_homework_toml(&HomeworkFileDto {
        homework: vec![base.clone(), base.clone()],
    });
    assert!(parse_homework_toml(&dup).is_err());

    let mut cases = vec![];
    let mut bad = base.clone();
    bad.title = " \n ".to_owned();
    cases.push(bad);

    let mut bad = base.clone();
    bad.subject = Some("  ".to_owned());
    cases.push(bad);

    let mut bad = base.clone();
    bad.due_date = "2026-02-30".to_owned();
    cases.push(bad);

    let mut bad = base.clone();
    bad.due_time = "25:00".to_owned();
    cases.push(bad);

    let mut bad = base.clone();
    bad.start_date = Some("2026-11-25".to_owned());
    cases.push(bad);

    let mut bad = base.clone();
    bad.estimated_minutes = 0;
    cases.push(bad);

    let mut bad = base.clone();
    bad.priority = "critical".to_owned();
    cases.push(bad);

    let mut bad = base.clone();
    bad.status = "paused".to_owned();
    cases.push(bad);

    let mut bad = base.clone();
    bad.reminders = vec![1, 2, 3, 4, 5, 6];
    cases.push(bad);
    assert!(parse_homework_toml(
        "[[homework]]\nid = \"a\"\ntitle = \"t\"\ndue_date = \"2026-11-20\"\ndue_time = \"09:00\"\nestimated_minutes = 10\npriority = \"low\"\nstatus = \"not_started\"\nreminders = [1, 2, 3, 4, 5, 6]\n"
    ).is_err());

    let mut bad = base.clone();
    bad.reminders = vec![40_321];
    cases.push(bad);

    let mut bad = base;
    bad.checklist[0].text = " ".to_owned();
    cases.push(bad);

    for invalid in cases {
        let toml = render_homework_toml(&HomeworkFileDto {
            homework: vec![invalid],
        });
        assert!(
            parse_homework_toml(&toml).is_err(),
            "expected err for:\n{toml}"
        );
    }
}

#[test]
fn due_time_requires_strict_hhmm_and_duplicate_ids_are_case_insensitive() {
    let base = dto("01ARZ3NDEKTSV4RRFFQ69G5FB1", "2026-11-20", "09:00");
    for bad_time in ["+9:00", "09:+0", "+0:+0", "9:0", "9:00", " 09:00 "] {
        let mut bad = base.clone();
        bad.due_time = bad_time.to_owned();
        assert!(
            validate_homework_dto(&bad).is_err(),
            "expected validate_homework_dto err for {bad_time:?}"
        );
        let rendered = render_homework_toml(&HomeworkFileDto {
            homework: vec![bad],
        });
        assert!(
            parse_homework_toml(&rendered).is_err(),
            "expected parse_homework_toml err for {bad_time:?}"
        );
    }

    let mut late = dto("01ARZ3NDEKTSV4RRFFQ69G5FB2", "2026-11-20", "10:00");
    late.due_time = "10:00".to_owned();
    let mut list = vec![late, base.clone()];
    sort_homework_dtos(&mut list);
    assert_eq!(list[0].due_time, "09:00");
    assert_eq!(list[1].due_time, "10:00");

    let mut lower = base.clone();
    lower.id = base.id.to_ascii_lowercase();
    let dup_toml = render_homework_toml(&HomeworkFileDto {
        homework: vec![base, lower],
    });
    assert!(parse_homework_toml(&dup_toml).is_err());
}
