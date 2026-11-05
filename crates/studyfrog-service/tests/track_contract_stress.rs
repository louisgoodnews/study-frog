#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Empirical challenge & stress tests for `studyfrog-service::track` contract across all item kinds.

use std::fs;
use std::path::PathBuf;

use studyfrog_calendar::session::{Ends, EventKind, Repeat};
use studyfrog_core::civil::date_new;
use studyfrog_core::priority::Priority;
use studyfrog_core::uid::Uid;
use studyfrog_md::parse::parse_doc;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::create::IdSource;
use studyfrog_service::create_more::{NewItem, NewKind};
use studyfrog_service::planning::list_sessions;
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
    random: [42; 10],
};

fn vault(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!(
        "studyfrog-track-stress-{label}-{}",
        std::process::id()
    ));
    fs::create_dir_all(dir.join("deck")).unwrap();
    let text = format!(
        "---\nid: {}\nkind: stack\ntitle: Deck\n---\n",
        format_ulid(Uid(1))
    );
    fs::write(dir.join("deck/_stack.md"), text).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

fn item<'a>(
    kind: NewKind,
    title: &'a str,
    parent: Option<&'a str>,
    priority: Option<Priority>,
) -> NewItem<'a> {
    NewItem {
        source: None,
        kind,
        title,
        text: "Front or description body text",
        answer: "Back or reference answer",
        tags: &[],
        parent,
        priority,
        difficulty: None,
        also_in: None,
    }
}

fn ids(n: u8) -> IdSource {
    let mut r = [n; 10];
    r[0] = n;
    IdSource { random: r, ..IDS }
}

#[test]
fn all_four_kinds_with_due_dates_create_deadline_sessions_and_link_stacks() {
    let (_d, mut s) = vault("all-kinds");
    let test_date = date_new(2027, 5, 20).unwrap();
    let cases = [
        (NewKind::Stack, "SubStack", None, "Due: SubStack"),
        (NewKind::Card, "CardTitle", Some("deck"), "Due: CardTitle"),
        (NewKind::Note, "NoteTitle", Some("deck"), "Due: NoteTitle"),
        (NewKind::Question, "QTitle", Some("deck"), "Due: QTitle"),
    ];

    for (i, (kind, title, parent, expected_due_title)) in cases.iter().enumerate() {
        let due = DueDate {
            date: test_date,
            minute: 600 + (i as u16 * 30),
            reminders: vec![1440, 60, 15],
        };
        let new_item = item(*kind, title, *parent, Some(Priority::High));
        let t = create_tracked(&mut s, &new_item, Some(&due), ids((i + 1) as u8)).unwrap();
        assert!(t.warnings.is_empty(), "{:?}", t.warnings);

        let sessions = list_sessions(&s).unwrap();
        let session = sessions
            .iter()
            .find(|sess| sess.title == *expected_due_title)
            .unwrap();
        assert_eq!(session.kind, EventKind::Deadline);
        assert_eq!(session.start.date, test_date);
        assert_eq!(session.start.minute, due.minute);
        assert_eq!(session.duration_min, 15);
        assert_eq!(session.repeat, Repeat::Never);
        assert_eq!(session.ends, Ends::Forever);
        assert_eq!(session.reminders, vec![15, 60, 1440]);
        assert!(session.stack.is_some());
        if *parent == Some("deck") {
            assert_eq!(session.stack, Some(Uid(1)));
        }
    }
}

#[test]
fn reminder_deduplication_sorting_and_empty_list_handling() {
    let (_d, mut s) = vault("reminders-stress");
    let test_date = date_new(2027, 8, 15).unwrap();

    let due_dups = DueDate {
        date: test_date,
        minute: 540,
        reminders: vec![1440, 0, 60, 1440, 0, 30],
    };
    let card = item(NewKind::Card, "DupsCard", Some("deck"), None);
    let t1 = create_tracked(&mut s, &card, Some(&due_dups), ids(10)).unwrap();
    assert!(t1.warnings.is_empty(), "{:?}", t1.warnings);

    let s1 = list_sessions(&s)
        .unwrap()
        .into_iter()
        .find(|x| x.title == "Due: DupsCard")
        .unwrap();
    assert_eq!(s1.reminders, vec![0, 30, 60, 1440]);

    let due_empty = DueDate {
        date: test_date,
        minute: 540,
        reminders: vec![],
    };
    let note = item(NewKind::Note, "EmptyRemindersNote", Some("deck"), None);
    let t2 = create_tracked(&mut s, &note, Some(&due_empty), ids(11)).unwrap();
    assert!(t2.warnings.is_empty(), "{:?}", t2.warnings);

    let s2 = list_sessions(&s)
        .unwrap()
        .into_iter()
        .find(|x| x.title == "Due: EmptyRemindersNote")
        .unwrap();
    assert!(s2.reminders.is_empty());
}

#[test]
fn none_due_date_does_not_create_deadline_session() {
    let (_d, mut s) = vault("no-due");
    let q = item(NewKind::Question, "NoDueQuestion", Some("deck"), None);
    let t = create_tracked(&mut s, &q, None, ids(20)).unwrap();
    assert!(t.warnings.is_empty());
    assert!(list_sessions(&s).unwrap().is_empty());
}

#[test]
fn priority_is_preserved_in_file_for_all_creatable_kinds() {
    let (d, mut s) = vault("priorities");
    let kinds = [
        (NewKind::Stack, "PStack", None, "pstack/_stack.md"),
        (NewKind::Card, "PCard", Some("deck"), "deck/pcard.md"),
        (NewKind::Note, "PNote", Some("deck"), "deck/pnote.md"),
        (NewKind::Question, "PQ", Some("deck"), "deck/pq.md"),
    ];

    for (i, (kind, title, parent, rel_path)) in kinds.iter().enumerate() {
        let new_item = item(*kind, title, *parent, Some(Priority::Highest));
        let t = create_tracked(&mut s, &new_item, None, ids((30 + i) as u8)).unwrap();
        assert!(t.warnings.is_empty());

        let raw = fs::read_to_string(d.0.join(rel_path)).unwrap();
        let doc = parse_doc(&raw).unwrap();
        assert_eq!(
            studyfrog_md::extra::extra_value(&doc, "priority").as_deref(),
            Some("highest")
        );
    }
}

#[test]
fn excessive_reminders_exceeding_max_yields_warning_without_failing_item() {
    let (_d, mut s) = vault("excessive-reminders");
    let excessive_due = DueDate {
        date: date_new(2027, 9, 1).unwrap(),
        minute: 540,
        reminders: vec![10, 20, 30, 40, 50, 60],
    };
    let card = item(NewKind::Card, "TooManyReminders", Some("deck"), None);
    let t = create_tracked(&mut s, &card, Some(&excessive_due), ids(40)).unwrap();
    assert_eq!(t.warnings.len(), 1);
    assert!(t.warnings[0].contains("the due date was not added to the calendar"));
    assert!(list_sessions(&s).unwrap().is_empty());
}
