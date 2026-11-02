#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Summary of a stack and changing a stack's status.

use std::fs;

use studyfrog_core::review::{Rating, Review};
use studyfrog_core::status::Status;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::session::open_vault;
use studyfrog_service::set_status::set_status;
use studyfrog_service::stack_summary::{Happened, stack_summary};
use studyfrog_store::schedule::insert_review_timed;

const DAY: i64 = 86_400_000;
const NOW: i64 = 1_800_000_000_000;

fn ms_text(ms: i64) -> String {
    studyfrog_md::rfc3339::format_rfc3339(UnixMs(ms)).unwrap()
}

#[test]
fn the_summary_counts_the_week_the_spread_the_feed_and_the_substacks() {
    let dir = std::env::temp_dir().join(format!("studyfrog-ssum-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("french/verbs")).unwrap();
    fs::create_dir_all(dir.join("maths")).unwrap();
    let stack = |p: &str, id: char, t: &str| {
        fs::write(
            dir.join(p),
            format!("---\nid: 0000000000000000000000000{id}\nkind: stack\ntitle: {t}\n---\n"),
        )
        .unwrap();
    };
    stack("french/_stack.md", '1', "French");
    stack("french/verbs/_stack.md", '2', "Verbs");
    stack("maths/_stack.md", '3', "Maths");
    let item = |p: &str, id: char, kind: &str, extra: &str, created: i64, updated: i64| {
        let text = format!(
            "---\nid: 0000000000000000000000000{id}\nkind: {kind}\ntitle: T{id}\ncreated: \"{}\"\nupdated: \"{}\"\n{extra}---\nq\n???\na\n",
            ms_text(created),
            ms_text(updated)
        );
        fs::write(dir.join(p), text).unwrap();
    };
    item(
        "french/a.md",
        'A',
        "card",
        "status: done\npriority: high\n",
        NOW - 30 * DAY,
        NOW - 2 * DAY,
    );
    item("french/b.md", 'B', "card", "", NOW - DAY, NOW - DAY);
    item(
        "french/verbs/c.md",
        'C',
        "note",
        "priority: low\n",
        NOW - 40 * DAY,
        NOW - 40 * DAY,
    );
    item("maths/d.md", 'D', "card", "", NOW - DAY, NOW - DAY);
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let r = Review {
        item: Uid(0xC),
        rating: Rating::Good,
        at: UnixMs(NOW - 3 * DAY),
        elapsed_days: 1.0,
    };
    insert_review_timed(&s.conn, &r, None).unwrap();

    let sum = stack_summary(&s.conn, "french", UnixMs(NOW)).unwrap();
    assert_eq!(
        (sum.completed, sum.updated, sum.created),
        (1, 2, 1),
        "{sum:?}"
    );
    assert_eq!(sum.types, [1, 2, 0], "the sub-stack's note counts");
    assert_eq!(sum.statuses.iter().sum::<usize>(), 3);
    assert_eq!(sum.statuses[3], 1, "one done");
    assert_eq!(sum.priorities.iter().sum::<usize>(), 3);
    assert!(sum.activity.windows(2).all(|w| w[0].at_ms >= w[1].at_ms));
    assert!(
        sum.activity
            .iter()
            .any(|a| a.what == Happened::Rehearsed && a.title == "TC")
    );
    assert_eq!(sum.substacks.len(), 1);
    let verbs = &sum.substacks[0];
    assert_eq!(
        (verbs.title.as_str(), verbs.total, verbs.done),
        ("Verbs", 1, 0)
    );
    assert!(verbs.percent.is_some());
    let none = stack_summary(&s.conn, "nothing", UnixMs(NOW)).unwrap();
    assert!(none.activity.is_empty() && none.substacks.is_empty());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_stack_can_be_given_a_status_and_back_to_automatic() {
    let dir = std::env::temp_dir().join(format!("studyfrog-sstat-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("s")).unwrap();
    fs::write(
        dir.join("s/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: S\n---\n",
    )
    .unwrap();
    let (mut s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    set_status(&mut s, Uid(1), Some(Status::Paused), UnixMs(NOW)).unwrap();
    assert!(
        fs::read_to_string(dir.join("s/_stack.md"))
            .unwrap()
            .contains("status: paused")
    );
    set_status(&mut s, Uid(1), None, UnixMs(NOW)).unwrap();
    assert!(
        !fs::read_to_string(dir.join("s/_stack.md"))
            .unwrap()
            .contains("status:")
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_tree_lists_every_nested_stack_with_its_depth_and_next_due_date() {
    use studyfrog_calendar::session::{Ends, EventKind, Repeat, Session};
    use studyfrog_core::civil::{LocalTime, date_from_days, date_to_days};
    let dir = std::env::temp_dir().join(format!("studyfrog-stree-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    for (p, id, t) in [
        ("a", '1', "A"),
        ("a/b", '2', "B"),
        ("a/b/c", '3', "C"),
        ("a/d", '4', "D"),
    ] {
        fs::create_dir_all(dir.join(p)).unwrap();
        fs::write(
            dir.join(p).join("_stack.md"),
            format!("---\nid: 0000000000000000000000000{id}\nkind: stack\ntitle: {t}\n---\n"),
        )
        .unwrap();
    }
    let (mut s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let day = date_to_days(studyfrog_core::civil::date_from_days(NOW.div_euclid(DAY))) + 9;
    let exam = Session {
        id: Uid(900),
        title: "Exam".into(),
        stack: Some(Uid(3)),
        start: LocalTime {
            date: date_from_days(day),
            minute: 540,
        },
        duration_min: 60,
        repeat: Repeat::Never,
        ends: Ends::Forever,
        cancelled: Vec::new(),
        notes: String::new(),
        kind: EventKind::Exam,
        reminders: Vec::new(),
    };
    studyfrog_store::sessions::save_session(&mut s.conn, &exam).unwrap();
    let sum = stack_summary(&s.conn, "a", UnixMs(NOW)).unwrap();
    let shape: Vec<(&str, usize)> = sum
        .tree
        .iter()
        .map(|t| (t.title.as_str(), t.depth))
        .collect();
    assert_eq!(shape, [("B", 1), ("C", 2), ("D", 1)], "tree order");
    assert_eq!(
        sum.substacks.len(),
        2,
        "the summary lists only the direct ones"
    );
    assert_eq!(sum.tree[1].due_day, Some(day));
    assert_eq!(sum.tree[0].due_day, None);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn goals_are_saved_in_the_stack_file_and_survive_a_reopen() {
    use studyfrog_service::goals::{load_goals, save_goals};
    let dir = std::env::temp_dir().join(format!("studyfrog-goals-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("s")).unwrap();
    fs::write(
        dir.join("s/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: S\n---\nAbout this stack.\n",
    )
    .unwrap();
    fs::write(
        dir.join("s/n.md"),
        "---\nid: 0000000000000000000000000A\nkind: note\ntitle: N\n---\nbody\n",
    )
    .unwrap();
    {
        let (mut s, _) = open_vault(dir.to_str().unwrap()).unwrap();
        assert_eq!(load_goals(&s, Uid(1)).unwrap(), "");
        save_goals(&mut s, Uid(1), "- pass\n- [ ] practise", UnixMs(NOW)).unwrap();
        assert!(save_goals(&mut s, Uid(1), "# Heading", UnixMs(NOW)).is_err());
        assert!(
            save_goals(&mut s, Uid(0xA), "- x", UnixMs(NOW)).is_err(),
            "a note has no goals"
        );
    }
    let text = fs::read_to_string(dir.join("s/_stack.md")).unwrap();
    assert!(
        text.contains("About this stack.") && text.contains("## Goals\n\n- pass\n- [ ] practise"),
        "{text}"
    );
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    assert_eq!(load_goals(&s, Uid(1)).unwrap(), "- pass\n- [ ] practise");
    let _ = fs::remove_dir_all(&dir);
}
