#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Time zones, sessions and the board through the service, including a restart.

use std::fs;
use std::path::PathBuf;

use studyfrog_board::model::{MoveMode, find_card, over_limit};
use studyfrog_board::ops::BoardOp;
use studyfrog_calendar::session::{Ends, Repeat, Session};
use studyfrog_core::civil::{LocalTime, date_new};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::create::IdSource;
use studyfrog_service::kanban::{apply_board_op, board_or_default};
use studyfrog_service::planning::{
    cancel_occurrence, delete_session_checked, list_sessions, save_session_checked,
};
use studyfrog_service::session::{ServiceError, open_vault};
use studyfrog_service::timezone::{day_window_at, local_offset_minutes};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str) -> Dir {
    let dir = std::env::temp_dir().join(format!("studyfrog-plan-{label}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    Dir(dir)
}

fn ids(n: u8) -> IdSource {
    IdSource {
        now_ms: 1_800_000_000_000 + u64::from(n),
        random: [n; 10],
    }
}

#[test]
fn a_local_day_is_found_from_the_offset_at_each_instant() {
    let ms = |s: &str| studyfrog_md::rfc3339::parse_rfc3339(s).unwrap();
    let utc = day_window_at(ms("2026-06-10T15:00:00Z"), |_| 0);
    assert_eq!(
        (utc.start, utc.end),
        (ms("2026-06-10T00:00:00Z"), ms("2026-06-11T00:00:00Z"))
    );

    // UTC+2: the local day of 23:30 UTC already belongs to the next date.
    let east = day_window_at(ms("2026-06-10T23:30:00Z"), |_| 120);
    assert_eq!(east.start, ms("2026-06-10T22:00:00Z"));
    assert_eq!(east.end, ms("2026-06-11T22:00:00Z"));

    // Spring forward: +1 before 01:00 UTC on 2026-03-29, +2 from then on (Central Europe).
    let change = ms("2026-03-29T01:00:00Z");
    let offset = move |at: UnixMs| if at < change { 60 } else { 120 };
    let day = day_window_at(ms("2026-03-29T12:00:00Z"), offset);
    assert_eq!(
        (day.end.0 - day.start.0) / 3_600_000,
        23,
        "the clock change makes that day 23 hours long"
    );
    let next = day_window_at(ms("2026-03-30T12:00:00Z"), offset);
    assert_eq!(next.start, day.end, "consecutive days tile without a gap");
}

#[test]
fn the_system_offset_is_plausible() {
    let off = local_offset_minutes(UnixMs(1_800_000_000_000));
    assert!((-14 * 60..=14 * 60).contains(&off), "{off}");
}

fn session(id: u128) -> Session {
    Session {
        id: Uid(id),
        title: "Review".into(),
        stack: None,
        start: LocalTime {
            date: date_new(2026, 11, 3).unwrap(),
            minute: 600,
        },
        duration_min: 30,
        repeat: Repeat::Weekly {
            every: 1,
            days: 0b0000101,
        },
        ends: Ends::Forever,
        cancelled: vec![],
        notes: String::new(),
        kind: Default::default(),
        reminders: Vec::new(),
    }
}

#[test]
fn sessions_are_validated_saved_cancelled_deleted_and_survive_a_restart() {
    let d = vault("sessions");
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let reason = |r: Result<_, ServiceError>| match r {
        Err(ServiceError::Invalid(m)) => m,
        other => panic!("expected a refusal: {other:?}"),
    };
    assert!(
        reason(save_session_checked(
            &mut s,
            Session {
                title: " ".into(),
                ..session(1)
            }
        ))
        .contains("title")
    );
    assert!(
        reason(save_session_checked(
            &mut s,
            Session {
                duration_min: 0,
                ..session(1)
            }
        ))
        .contains("duration")
    );
    assert!(
        reason(save_session_checked(
            &mut s,
            Session {
                ends: Ends::After(0),
                ..session(1)
            }
        ))
        .contains("occurrences")
    );
    assert!(
        list_sessions(&s).unwrap().is_empty(),
        "refused sessions are not stored"
    );

    save_session_checked(&mut s, session(1)).unwrap();
    save_session_checked(
        &mut s,
        Session {
            title: "Second".into(),
            start: LocalTime {
                date: date_new(2026, 12, 1).unwrap(),
                minute: 0,
            },
            ..session(2)
        },
    )
    .unwrap();
    let after = cancel_occurrence(&mut s, Uid(1), date_new(2026, 11, 10).unwrap()).unwrap();
    assert_eq!(after[0].cancelled, [date_new(2026, 11, 10).unwrap()]);
    assert_eq!(
        cancel_occurrence(&mut s, Uid(1), date_new(2026, 11, 10).unwrap()).unwrap()[0]
            .cancelled
            .len(),
        1,
        "cancelling twice is harmless"
    );

    drop(s); // restart
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let all = list_sessions(&s).unwrap();
    assert_eq!(
        all.iter().map(|x| x.title.as_str()).collect::<Vec<_>>(),
        ["Review", "Second"]
    );
    assert_eq!(delete_session_checked(&mut s, Uid(2)).unwrap().len(), 1);
    assert!(matches!(
        delete_session_checked(&mut s, Uid(2)),
        Err(ServiceError::Invalid(_))
    ));
}

#[test]
fn the_board_is_created_once_changed_through_commands_and_survives_a_restart() {
    let d = vault("board");
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let first = board_or_default(&mut s, ids(1)).unwrap();
    assert_eq!(
        first
            .columns
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["Backlog", "Doing", "Review", "Done"]
    );
    assert_eq!(
        board_or_default(&mut s, ids(99)).unwrap(),
        first,
        "later calls read the stored board, not new ids"
    );

    let backlog = first.columns[0].id;
    let mut board = first;
    for (i, title) in ["one", "two", "three", "four"].into_iter().enumerate() {
        board = apply_board_op(
            &mut s,
            &BoardOp::AddCard {
                column: backlog,
                title: title.into(),
            },
            ids(10 + i as u8),
        )
        .unwrap();
    }
    let cards: Vec<Uid> = board.columns[0].cards.iter().map(|c| c.id).collect();
    assert_eq!(cards.len(), 4);
    for id in &cards[..3] {
        apply_board_op(&mut s, &BoardOp::Push(*id, MoveMode::Strict), ids(50)).unwrap();
    }
    let refused = apply_board_op(&mut s, &BoardOp::Push(cards[3], MoveMode::Strict), ids(50));
    assert!(
        matches!(&refused, Err(ServiceError::Invalid(m)) if m.contains("\"Doing\" is full") && m.contains('3')),
        "{refused:?}"
    );
    let forced =
        apply_board_op(&mut s, &BoardOp::Push(cards[3], MoveMode::Force), ids(50)).unwrap();
    assert_eq!(over_limit(&forced).len(), 1);

    drop(s); // restart
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let again = board_or_default(&mut s, ids(7)).unwrap();
    assert_eq!(again, forced, "the board is exactly as it was left");
    assert_eq!(find_card(&again, cards[3]).map(|p| p.0), Some(1));
}
