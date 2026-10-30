#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The home summary from a real vault's history.

use std::fs;
use std::path::PathBuf;

use studyfrog_calendar::session::{Ends, Repeat, Session};
use studyfrog_core::civil::{LocalTime, utc_to_local};
use studyfrog_core::review::Rating;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::day::day_from;
use studyfrog_service::home::home_summary;
use studyfrog_service::planning::save_session_checked;
use studyfrog_service::schedule::apply_rating;
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_srs::params::params_default;
use studyfrog_store::schedule::{insert_review, save_mem_state};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const DAY: i64 = 86_400_000;
const NOON: i64 = 1_800_000_000_000 - (1_800_000_000_000 % DAY) + 12 * 3_600_000;

fn vault(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!("studyfrog-home-{label}-{}", std::process::id()));
    fs::create_dir_all(dir.join("deck")).unwrap();
    let id = |n: u128| format_ulid(Uid(n));
    fs::write(
        dir.join("deck/_stack.md"),
        format!("---\nid: {}\nkind: stack\ntitle: Deck\n---\n", id(1)),
    )
    .unwrap();
    for n in [10, 11, 12] {
        let text = format!(
            "---\nid: {}\nkind: card\ntitle: C{n}\n---\nq\n???\na\n",
            id(n)
        );
        fs::write(dir.join(format!("deck/c{n}.md")), text).unwrap();
    }
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

fn review(s: &VaultSession, item: u128, at: i64) {
    let (r, mem) = apply_rating(None, Uid(item), Rating::Good, UnixMs(at), &params_default());
    insert_review(&s.conn, &r).unwrap();
    save_mem_state(&s.conn, Uid(item), &mem).unwrap();
}

#[test]
fn the_summary_counts_due_and_new_items_per_stack_and_what_is_ready_today() {
    let (_d, s) = vault("counts");
    review(&s, 10, NOON - 30 * DAY); // long overdue
    review(&s, 11, NOON - 100); // reviewed a moment ago: not due
    let now = UnixMs(NOON);
    let h = home_summary(&s, now, 0, day_from(UnixMs(NOON - 12 * 3_600_000))).unwrap();
    assert_eq!(
        (h.due_now, h.new_items),
        (1, 1),
        "C10 is due, C12 was never reviewed"
    );
    assert_eq!(h.stacks.len(), 1);
    assert_eq!(
        (h.stacks[0].title.as_str(), h.stacks[0].due, h.stacks[0].new),
        ("Deck", 1, 1)
    );
    assert_eq!(h.ready_today, 2, "the due item plus the new one");
    assert_eq!(h.hour, 12);
    assert_eq!(h.recent_items.len(), 3);
}

#[test]
fn reviews_today_the_streak_and_recent_runs_are_reported() {
    let (_d, s) = vault("activity");
    for (item, ago) in [(10, 2 * DAY), (11, DAY), (12, 3_600_000)] {
        review(&s, item, NOON - ago);
    }
    s.conn
        .execute(
            "INSERT INTO run (uid, mode, state, started_at_ms) VALUES (?1, 'flip', 'finished', ?2)",
            rusqlite::params![format_ulid(Uid(500)), NOON - 7_200_000],
        )
        .unwrap();
    for (seq, kind) in [(0, "item_rated"), (1, "item_rated"), (2, "item_skipped")] {
        s.conn.execute("INSERT INTO run_event (run_uid, seq, t_ms, item_uid, kind, payload) VALUES (?1, ?2, 0, NULL, ?3, NULL)", rusqlite::params![format_ulid(Uid(500)), seq, kind]).unwrap();
    }
    let day = day_from(UnixMs(NOON - 12 * 3_600_000));
    let h = home_summary(&s, UnixMs(NOON), 0, day).unwrap();
    assert_eq!(h.reviews_today, 1, "only the review an hour ago is today");
    assert_eq!(h.streak_days, 3, "two days ago, yesterday and today");
    assert_eq!(h.recent_runs.len(), 1);
    let run = &h.recent_runs[0];
    assert_eq!(
        (run.rated, run.skipped, run.state.as_str()),
        (2, 1, "finished")
    );
}

#[test]
fn todays_sessions_are_listed_with_their_stack_in_time_order() {
    let (_d, mut s) = vault("sessions");
    let today = utc_to_local(UnixMs(NOON), 0).date;
    let make = |id: u128, title: &str, minute: u16, stack: Option<Uid>| Session {
        id: Uid(id),
        title: title.into(),
        stack,
        start: LocalTime {
            date: today,
            minute,
        },
        duration_min: 30,
        repeat: Repeat::Never,
        ends: Ends::Forever,
        cancelled: Vec::new(),
        notes: String::new(),
        kind: Default::default(),
        reminders: Vec::new(),
    };
    save_session_checked(&mut s, make(1, "Evening", 19 * 60, None)).unwrap();
    save_session_checked(&mut s, make(2, "Morning drills", 8 * 60 + 30, Some(Uid(1)))).unwrap();
    let h = home_summary(&s, UnixMs(NOON), 0, day_from(UnixMs(NOON - 12 * 3_600_000))).unwrap();
    let list: Vec<(&str, u16, Option<&str>)> = h
        .sessions_today
        .iter()
        .map(|t| (t.title.as_str(), t.start_minute, t.stack.as_deref()))
        .collect();
    assert_eq!(
        list,
        [
            ("Morning drills", 510, Some("Deck")),
            ("Evening", 1140, None)
        ]
    );
}

#[test]
fn due_items_come_most_overdue_first_and_the_next_week_lists_other_days_only() {
    let (_d, mut s) = vault("due-items");
    review(&s, 10, NOON - 30 * DAY);
    review(&s, 11, NOON - 20 * DAY);
    review(&s, 12, NOON - 100); // not due yet
    let today = utc_to_local(UnixMs(NOON), 0).date;
    let on = |n: i64, id: u128, title: &str, kind| Session {
        id: Uid(id),
        title: title.into(),
        stack: None,
        start: LocalTime {
            date: studyfrog_core::civil::add_days(today, n),
            minute: 600,
        },
        duration_min: 30,
        repeat: Repeat::Never,
        ends: Ends::Forever,
        cancelled: Vec::new(),
        notes: String::new(),
        kind,
        reminders: Vec::new(),
    };
    use studyfrog_calendar::session::EventKind;
    save_session_checked(&mut s, on(0, 1, "Today", EventKind::Task)).unwrap();
    save_session_checked(&mut s, on(3, 2, "Exam", EventKind::Exam)).unwrap();
    save_session_checked(&mut s, on(9, 3, "Far away", EventKind::Deadline)).unwrap();
    let h = home_summary(&s, UnixMs(NOON), 0, day_from(UnixMs(NOON - 12 * 3_600_000))).unwrap();
    let due: Vec<&str> = h.due_items.iter().map(|i| i.title.as_str()).collect();
    assert_eq!(due, ["C10", "C11"]);
    let next: Vec<(&str, EventKind)> = h
        .upcoming
        .iter()
        .map(|u| (u.title.as_str(), u.kind))
        .collect();
    assert_eq!(
        next,
        [("Exam", EventKind::Exam)],
        "not today, not beyond a week"
    );
}

#[test]
fn an_empty_vault_has_a_quiet_summary() {
    let dir = std::env::temp_dir().join(format!("studyfrog-home-empty-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let _d = Dir(dir.clone());
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let h = home_summary(&s, UnixMs(NOON), 0, day_from(UnixMs(NOON - 12 * 3_600_000))).unwrap();
    assert_eq!(
        (h.ready_today, h.due_now, h.streak_days, h.reviews_today),
        (0, 0, 0, 0)
    );
    assert!(
        h.stacks.is_empty()
            && h.recent_items.is_empty()
            && h.recent_runs.is_empty()
            && h.sessions_today.is_empty()
    );
}
