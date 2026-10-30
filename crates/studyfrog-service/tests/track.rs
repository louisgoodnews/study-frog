#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! What creating an item leaves on the board and the calendar, and the reminders that follow.

use std::fs;
use std::path::PathBuf;

use studyfrog_calendar::session::EventKind;
use studyfrog_core::civil::{date_new, utc_to_local};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::create::IdSource;
use studyfrog_service::create_more::{NewItem, NewKind};
use studyfrog_service::kanban::board_or_default;
use studyfrog_service::planning::list_sessions;
use studyfrog_service::reminders::take_due_reminders;
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_service::track::{DueDate, create_tracked};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const IDS: IdSource = IdSource {
    now_ms: 1_800_000_000_000,
    random: [7; 10],
};

fn vault(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!("studyfrog-track-{label}-{}", std::process::id()));
    fs::create_dir_all(dir.join("deck")).unwrap();
    let text = format!(
        "---\nid: {}\nkind: stack\ntitle: Deck\n---\n",
        format_ulid(Uid(1))
    );
    fs::write(dir.join("deck/_stack.md"), text).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

fn item<'a>(kind: NewKind, title: &'a str, parent: Option<&'a str>) -> NewItem<'a> {
    NewItem {
        kind,
        title,
        text: "front",
        answer: "back",
        tags: &[],
        parent,
    }
}

fn ids(n: u8) -> IdSource {
    IdSource {
        random: [n; 10],
        ..IDS
    }
}

#[test]
fn a_new_card_and_a_new_stack_each_get_a_card_on_the_first_board_column() {
    let (_d, mut s) = vault("board");
    let t = create_tracked(
        &mut s,
        &item(NewKind::Card, "Bonjour", Some("deck")),
        None,
        ids(1),
    )
    .unwrap();
    assert!(t.warnings.is_empty(), "{:?}", t.warnings);
    create_tracked(&mut s, &item(NewKind::Stack, "Verbs", None), None, ids(2)).unwrap();
    let board = board_or_default(&mut s, ids(3)).unwrap();
    let titles: Vec<_> = board.columns[0]
        .cards
        .iter()
        .map(|c| c.title.as_str())
        .collect();
    assert_eq!(titles, ["Card: Bonjour", "Stack: Verbs"]);
    assert_eq!(
        board.columns[0].cards[0].stack,
        Some(Uid(1)),
        "linked to its stack"
    );
}

#[test]
fn a_stack_with_a_due_date_puts_a_deadline_with_reminders_on_the_calendar() {
    let (_d, mut s) = vault("due");
    let due = DueDate {
        date: date_new(2027, 3, 1).unwrap(),
        minute: 540,
        reminders: vec![1440, 60, 60],
    };
    let t = create_tracked(
        &mut s,
        &item(NewKind::Stack, "Exam prep", None),
        Some(&due),
        ids(4),
    )
    .unwrap();
    assert!(t.warnings.is_empty(), "{:?}", t.warnings);
    let all = list_sessions(&s).unwrap();
    assert_eq!(all.len(), 1);
    let e = &all[0];
    assert_eq!(
        (e.kind, e.title.as_str(), e.reminders.clone()),
        (EventKind::Deadline, "Due: Exam prep", vec![60, 1440])
    );
    assert!(e.stack.is_some(), "linked to the stack");
}

#[test]
fn a_non_stack_ignores_a_due_date() {
    let (_d, mut s) = vault("ignore");
    let due = DueDate {
        date: date_new(2027, 3, 1).unwrap(),
        minute: 540,
        reminders: vec![0],
    };
    create_tracked(
        &mut s,
        &item(NewKind::Note, "Idea", Some("deck")),
        Some(&due),
        ids(5),
    )
    .unwrap();
    assert!(list_sessions(&s).unwrap().is_empty());
}

#[test]
fn each_reminder_comes_once_when_its_time_has_come() {
    let (_d, mut s) = vault("remind");
    let due = DueDate {
        date: date_new(2027, 3, 1).unwrap(),
        minute: 540,
        reminders: vec![60, 1440],
    };
    create_tracked(
        &mut s,
        &item(NewKind::Stack, "Exam", None),
        Some(&due),
        ids(6),
    )
    .unwrap();
    let at = |d: u8, h: i64, m: i64| {
        let day = studyfrog_core::civil::date_to_days(date_new(2027, 2, d).unwrap());
        UnixMs(day * 86_400_000 + (h * 60 + m) * 60_000)
    };
    assert!(
        take_due_reminders(&s, at(27, 8, 0), 0).unwrap().is_empty(),
        "a day and an hour early is too early"
    );
    let day_before = take_due_reminders(&s, at(28, 9, 5), 0).unwrap();
    assert_eq!(
        day_before
            .iter()
            .map(|r| r.minutes_before)
            .collect::<Vec<_>>(),
        [1440]
    );
    assert!(
        take_due_reminders(&s, at(28, 9, 10), 0).unwrap().is_empty(),
        "shown once"
    );
    let march = UnixMs(
        studyfrog_core::civil::date_to_days(date_new(2027, 3, 1).unwrap()) * 86_400_000
            + 8 * 3_600_000
            + 30 * 60_000,
    );
    let hour = take_due_reminders(&s, march, 0).unwrap();
    assert_eq!(
        hour.iter().map(|r| r.minutes_before).collect::<Vec<_>>(),
        [60]
    );
    let after = UnixMs(march.0 + 3 * 3_600_000);
    assert!(take_due_reminders(&s, after, 0).unwrap().is_empty());
    let _ = utc_to_local(after, 0);
}
