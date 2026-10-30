#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Stacks past their due date are not suggested.

use std::fs;
use std::path::PathBuf;

use studyfrog_calendar::session::{Ends, EventKind, Repeat, Session};
use studyfrog_core::civil::{LocalTime, add_days, date_from_days};
use studyfrog_core::plan::plan_spaced;
use studyfrog_core::run::RunMode;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_rehearsal::queue::candidate::budget_unlimited;
use studyfrog_service::day::day_from;
use studyfrog_service::home::home_summary;
use studyfrog_service::planning::save_session_checked;
use studyfrog_service::queue::{NewRun, preview_queue};
use studyfrog_service::session::{VaultSession, current_settings, open_vault};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const DAY: i64 = 86_400_000;
const START: i64 = 1_800_000_000_000 - (1_800_000_000_000 % DAY);

fn vault(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!("studyfrog-expiry-{label}-{}", std::process::id()));
    fs::create_dir_all(dir.join("deck")).unwrap();
    let id = |n: u128| format_ulid(Uid(n));
    fs::write(
        dir.join("deck/_stack.md"),
        format!("---\nid: {}\nkind: stack\ntitle: Deck\n---\n", id(1)),
    )
    .unwrap();
    for n in [10, 11] {
        let text = format!(
            "---\nid: {}\nkind: card\ntitle: C{n}\n---\nq\n???\na\n",
            id(n)
        );
        fs::write(dir.join(format!("deck/c{n}.md")), text).unwrap();
    }
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

fn deadline(s: &mut VaultSession, days_from_today: i64) {
    let today = date_from_days(START / DAY);
    let entry = Session {
        id: Uid(500),
        title: "Due: Deck".into(),
        stack: Some(Uid(1)),
        start: LocalTime {
            date: add_days(today, days_from_today),
            minute: 540,
        },
        duration_min: 15,
        repeat: Repeat::Never,
        ends: Ends::Forever,
        cancelled: Vec::new(),
        notes: String::new(),
        kind: EventKind::Deadline,
        reminders: Vec::new(),
    };
    save_session_checked(s, entry).unwrap();
}

fn queue_len(s: &VaultSession, only_deck: bool) -> usize {
    let mut plan = plan_spaced();
    if only_deck {
        plan.filter.stacks = vec![Uid(1)];
    }
    let req = NewRun {
        plan,
        mode: RunMode::Flip,
        uid: Uid(0),
        now: UnixMs(START + 12 * 3_600_000),
        day: day_from(UnixMs(START)),
        global_budget: budget_unlimited(),
    };
    preview_queue(&s.conn, req, &current_settings(s)).unwrap()
}

#[test]
fn a_stack_past_its_due_date_is_not_suggested_but_can_still_be_chosen() {
    let (_d, mut s) = vault("queue");
    assert_eq!(queue_len(&s, false), 2);
    deadline(&mut s, 3);
    assert_eq!(queue_len(&s, false), 2, "a future due date changes nothing");
    deadline(&mut s, -1);
    assert_eq!(queue_len(&s, false), 0, "yesterday's deadline has passed");
    assert_eq!(
        queue_len(&s, true),
        2,
        "picking the stack by hand still works"
    );
    deadline(&mut s, 5);
    assert_eq!(
        queue_len(&s, false),
        2,
        "moving the due date brings it back"
    );
}

#[test]
fn the_home_page_does_not_count_an_expired_stack() {
    let (_d, mut s) = vault("home");
    deadline(&mut s, -2);
    let h = home_summary(
        &s,
        UnixMs(START + 12 * 3_600_000),
        0,
        day_from(UnixMs(START)),
    )
    .unwrap();
    assert_eq!((h.new_items, h.ready_today), (0, 0));
}
