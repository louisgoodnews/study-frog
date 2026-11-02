#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The planned reviews of a real vault: planning, moving without touching memory, warnings and undo.

use std::fs;
use std::path::PathBuf;

use studyfrog_calendar::session::{Ends, EventKind, Repeat, Session};
use studyfrog_core::civil::{LocalTime, date_from_days};
use studyfrog_core::memory::{MemState, Sm2State};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::review_plan::{move_reviews, plan_reviews, undo_move};
use studyfrog_service::session::open_vault;
use studyfrog_service::timezone::{local_day_of, local_day_start};
use studyfrog_store::schedule::{
    StoredMem, StoredSm2, load_mem_state, save_mem_state, save_sm2_state,
};
use studyfrog_store::sessions::save_session;

const DAY: i64 = 86_400_000;
const NOW: i64 = 1_800_000_000_000;

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str, stack_extra: &str) -> Dir {
    let dir = std::env::temp_dir().join(format!("studyfrog-rplan-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("french")).unwrap();
    fs::create_dir_all(dir.join("maths")).unwrap();
    let w = |p: &str, t: String| fs::write(dir.join(p), t).unwrap();
    w(
        "french/_stack.md",
        format!(
            "---\nid: 00000000000000000000000001\nkind: stack\ntitle: French\n{stack_extra}---\n"
        ),
    );
    w(
        "maths/_stack.md",
        "---\nid: 00000000000000000000000002\nkind: stack\ntitle: Maths\n---\n".into(),
    );
    for (id, folder, name, extra) in [
        ('A', "french", "a", ""),
        ('B', "french", "b", "status: done\n"),
        ('C', "french", "c", ""),
        ('D', "maths", "d", ""),
    ] {
        w(
            &format!("{folder}/{name}.md"),
            format!(
                "---\nid: 0000000000000000000000000{id}\nkind: card\ntitle: Item {name}\n{extra}---\nQ\n???\nA\n"
            ),
        );
    }
    Dir(dir)
}

fn fsrs(due: i64) -> StoredMem {
    StoredMem {
        state: MemState {
            stability: 12.0,
            difficulty: 5.0,
        },
        due: UnixMs(due),
        last_review: UnixMs(NOW - 5 * DAY),
    }
}

fn today() -> i64 {
    local_day_of(UnixMs(NOW))
}

fn exam(day: i64) -> Session {
    Session {
        id: Uid(900),
        title: "French exam".into(),
        stack: Some(Uid(1)),
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
    }
}

fn schedule(s: &mut studyfrog_service::session::VaultSession) {
    for (id, days) in [(0xA, 3), (0xB, 3), (0xC, 5), (0xD, 3)] {
        save_mem_state(&s.conn, Uid(id), &fsrs(NOW + days * DAY)).unwrap();
    }
}

#[test]
fn entries_group_by_stack_and_day_and_skip_done_items_and_paused_stacks() {
    let d = vault("group", "");
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    schedule(&mut s);
    let got = plan_reviews(&s, (today(), today() + 30)).unwrap();
    let shape: Vec<_> = got
        .iter()
        .map(|e| {
            (
                e.day - today(),
                e.stack.map(|u| u.0),
                e.items.len(),
                e.stack_size,
            )
        })
        .collect();
    assert_eq!(
        shape,
        [(3, Some(1), 1, 3), (3, Some(2), 1, 1), (5, Some(1), 1, 3)],
        "b is done and so left out"
    );

    let d = vault("paused", "status: paused\n");
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    schedule(&mut s);
    let got = plan_reviews(&s, (today(), today() + 30)).unwrap();
    assert!(
        got.iter().all(|e| e.stack == Some(Uid(2))),
        "a paused stack plans nothing: {got:?}"
    );
}

#[test]
fn a_move_changes_only_the_due_date_and_can_be_undone() {
    let d = vault("move", "");
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    schedule(&mut s);
    save_sm2_state(
        &s.conn,
        Uid(0xC),
        &StoredSm2 {
            state: Sm2State {
                ease: 2.5,
                repetitions: 2,
                interval_days: 6,
            },
            due: UnixMs(NOW + 5 * DAY),
            last_review: UnixMs(NOW - DAY),
        },
    )
    .unwrap();
    let before = load_mem_state(&s.conn, Uid(0xA)).unwrap().unwrap();

    let moved = move_reviews(
        &s,
        &[Uid(0xA), Uid(0xC), Uid(0xE)],
        today() + 9,
        UnixMs(NOW),
    )
    .unwrap();
    assert_eq!(moved.moved, 2, "an item with no schedule is not moved");
    assert!(moved.warning.is_none());
    let after = load_mem_state(&s.conn, Uid(0xA)).unwrap().unwrap();
    assert_eq!(after.due, local_day_start(today() + 9));
    assert_eq!(
        (after.state, after.last_review),
        (before.state, before.last_review),
        "memory untouched"
    );
    let got = plan_reviews(&s, (today(), today() + 30)).unwrap();
    assert!(
        got.iter()
            .any(|e| e.day == today() + 9 && e.items.len() == 2),
        "{got:?}"
    );

    undo_move(&s, &moved.undo).unwrap();
    assert_eq!(load_mem_state(&s.conn, Uid(0xA)).unwrap().unwrap(), before);
    assert_eq!(
        load_mem_state(&s.conn, Uid(0xC)).unwrap().unwrap().due,
        UnixMs(NOW + 5 * DAY)
    );
}

#[test]
fn a_moved_item_is_not_due_until_its_day_and_a_move_past_the_exam_warns() {
    let d = vault("warn", "");
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    schedule(&mut s);
    save_session(&mut s.conn, &exam(today() + 6)).unwrap();

    let ok = move_reviews(&s, &[Uid(0xA)], today() + 6, UnixMs(NOW)).unwrap();
    assert!(
        ok.warning.is_none(),
        "on the day of the exam is not past it"
    );
    let late = move_reviews(&s, &[Uid(0xA), Uid(0xD)], today() + 7, UnixMs(NOW)).unwrap();
    let w = late.warning.expect("a warning");
    assert!(w.contains("French") && !w.contains("Maths"), "{w}");
    let due = load_mem_state(&s.conn, Uid(0xA)).unwrap().unwrap().due;
    assert!(due.0 > NOW, "not due before its new day");
}
