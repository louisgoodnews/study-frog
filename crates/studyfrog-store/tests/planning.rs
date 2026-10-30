#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Sessions and the board in SQLite.

use rusqlite::Connection;
use studyfrog_board::edit::add_card;
use studyfrog_board::model::{Board, MoveMode, board_default, card_new};
use studyfrog_board::moves::push;
use studyfrog_calendar::session::{Ends, Repeat, Session};
use studyfrog_core::civil::{LocalTime, date_new};
use studyfrog_core::uid::Uid;
use studyfrog_store::board_store::{load_board, save_board};
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::migrate;
use studyfrog_store::schema::MIGRATIONS;
use studyfrog_store::sessions::{delete_session, load_sessions, save_session};

fn db() -> Connection {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    conn
}

fn session(id: u128, repeat: Repeat, ends: Ends) -> Session {
    Session {
        id: Uid(id),
        title: format!("Session {id} \u{1F438}"),
        stack: Some(Uid(77)),
        start: LocalTime {
            date: date_new(2026, 11, 3).unwrap(),
            minute: 8 * 60 + 15,
        },
        duration_min: 45,
        repeat,
        ends,
        cancelled: vec![
            date_new(2026, 11, 10).unwrap(),
            date_new(2026, 11, 17).unwrap(),
        ],
        notes: "bring the deck\nand coffee".into(),
    }
}

#[test]
fn every_repeat_and_end_variant_round_trips_in_start_order() {
    let mut conn = db();
    let all = [
        session(1, Repeat::Never, Ends::Forever),
        session(
            2,
            Repeat::Daily { every: 3 },
            Ends::On(date_new(2027, 1, 31).unwrap()),
        ),
        session(
            3,
            Repeat::Weekly {
                every: 2,
                days: 0b0101010,
            },
            Ends::After(12),
        ),
        session(4, Repeat::Monthly { every: 1 }, Ends::Forever),
    ];
    for s in all.iter().rev() {
        save_session(&mut conn, s).unwrap();
    }
    let mut expected = all.to_vec();
    expected.sort_by_key(|s| (s.start, s.id));
    assert_eq!(load_sessions(&conn).unwrap(), expected);
}

#[test]
fn saving_again_replaces_and_deleting_removes_everything_of_it() {
    let mut conn = db();
    let mut s = session(1, Repeat::Never, Ends::Forever);
    save_session(&mut conn, &s).unwrap();
    s.title = "Renamed".into();
    s.stack = None;
    s.cancelled.clear();
    save_session(&mut conn, &s).unwrap();
    assert_eq!(load_sessions(&conn).unwrap(), [s.clone()]);
    let cancelled: i64 = conn
        .query_row("SELECT COUNT(*) FROM session_cancelled", [], |r| r.get(0))
        .unwrap();
    assert_eq!(cancelled, 0, "replacing rewrites the cancelled dates");

    save_session(&mut conn, &session(2, Repeat::Never, Ends::Forever)).unwrap();
    assert!(delete_session(&conn, Uid(2)).unwrap() && !delete_session(&conn, Uid(2)).unwrap());
    let left: i64 = conn
        .query_row("SELECT COUNT(*) FROM session_cancelled", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        (load_sessions(&conn).unwrap().len(), left),
        (1, 0),
        "its cancelled dates went with it"
    );
}

#[test]
fn the_database_refuses_nonsense_sessions() {
    let conn = db();
    let bad = |sql: &str| conn.execute(sql, []).is_err();
    let base = "INSERT INTO session (id, title, start_date, start_minute, duration_min, repeat_kind, repeat_every, repeat_days, ends_kind)";
    assert!(!bad(&format!(
        "{base} VALUES ('A', 't', '2026-01-01', 60, 30, 'never', 1, 0, 'forever')"
    )));
    assert!(bad(&format!(
        "{base} VALUES ('B', 't', '2026-01-01', 1500, 30, 'never', 1, 0, 'forever')"
    )));
    assert!(bad(&format!(
        "{base} VALUES ('C', 't', '2026-01-01', 60, 0, 'never', 1, 0, 'forever')"
    )));
    assert!(bad(&format!(
        "{base} VALUES ('D', 't', '2026-01-01', 60, 30, 'hourly', 1, 0, 'forever')"
    )));
    assert!(bad(&format!(
        "{base} VALUES ('E', 't', '2026-01-01', 60, 30, 'weekly', 1, 200, 'forever')"
    )));
}

fn sample_board() -> Board {
    let mut b = board_default([Uid(1), Uid(2), Uid(3), Uid(4)]);
    for i in 0..5 {
        b = add_card(
            &b,
            Uid(1),
            card_new(Uid(100 + i), &format!("task {i}")),
            MoveMode::Strict,
        )
        .unwrap();
    }
    b = push(&b, Uid(102), MoveMode::Strict).unwrap();
    b = push(&b, Uid(100), MoveMode::Strict).unwrap();
    add_card(
        &b,
        Uid(3),
        studyfrog_board::model::Card {
            stack: Some(Uid(9)),
            points: Some(5),
            notes: "n\nm".into(),
            ..card_new(Uid(200), "linked")
        },
        MoveMode::Strict,
    )
    .unwrap()
}

#[test]
fn the_board_round_trips_with_columns_limits_cards_and_order() {
    let mut conn = db();
    assert_eq!(load_board(&conn).unwrap(), None, "nothing saved yet");
    let b = sample_board();
    save_board(&mut conn, &b).unwrap();
    assert_eq!(load_board(&conn).unwrap(), Some(b.clone()));

    let moved = push(&b, Uid(200), MoveMode::Strict).unwrap();
    save_board(&mut conn, &moved).unwrap();
    assert_eq!(
        load_board(&conn).unwrap(),
        Some(moved),
        "saving replaces the previous board entirely"
    );
    let cards: i64 = conn
        .query_row("SELECT COUNT(*) FROM board_card", [], |r| r.get(0))
        .unwrap();
    assert_eq!(cards, 6);
}
