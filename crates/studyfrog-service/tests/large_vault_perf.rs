#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stdout)] // a measurement: prints its timings
//! Measurements on a large vault (STORY-0177): the first-review lookup that starts a new card's
//! difficulty, and the rating of a new card with it on and off. Ignored by default, because it writes
//! a vault of `STUDYFROG_LARGE_N` cards (default 20,000) with ten reviews each in the temp directory.
//! Run: `cargo test -p studyfrog-service --test large_vault_perf -- --ignored --nocapture`.

use std::fs;
use std::time::Instant;

use rusqlite::params;
use studyfrog_core::event::{EventKind as RunEventKind, RunEvent};
use studyfrog_core::plan::ScheduleEffect;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunState;
use studyfrog_core::settings::{VaultSettings, vault_settings_default};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_rehearsal::input::StepOutput;
use studyfrog_service::rehearse::{RatingTiming, StepContext, record_step};
use studyfrog_service::schedule::observed_difficulties;
use studyfrog_service::session::open_vault;
use studyfrog_store::first_timing::first_review_timings;

const DAY: i64 = 86_400_000;
const START: i64 = 1_700_000_000_000;
const REVIEWS_PER_ITEM: i64 = 10;
/// Cards without reviews, rated as first answers in the measurement (half with the switch on, half off).
const NEW_CARDS: usize = 40;

fn large_n() -> usize {
    std::env::var("STUDYFROG_LARGE_N")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20_000)
}

/// A vault of `n` cards in one stack, each with `REVIEWS_PER_ITEM` reviews written straight to the
/// table (the rating path would be too slow to build it).
fn build(n: usize) -> std::path::PathBuf {
    let untimed = std::env::var("STUDYFROG_LARGE_UNTIMED").is_ok();
    let dir = std::env::temp_dir().join(format!(
        "studyfrog-large-{n}{}",
        if untimed { "-untimed" } else { "" }
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("s")).unwrap();
    fs::write(
        dir.join("s/_stack.md"),
        format!(
            "---\nid: {}\nkind: stack\ntitle: S\n---\n",
            format_ulid(Uid(1))
        ),
    )
    .unwrap();
    // The new cards rated below exist as files: a rating only reaches the lookup inside a stack.
    for i in 0..n + NEW_CARDS {
        let body = format!(
            "---\nid: {}\nkind: card\ntitle: C{i}\n---\nq {i}\n???\na {i}\n",
            format_ulid(Uid(1_000 + i as u128))
        );
        fs::write(dir.join(format!("s/c{i}.md")), body).unwrap();
    }
    let (session, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let mut conn = session.conn;
    let tx = conn.transaction().unwrap();
    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO review (item_uid, rating, at_ms, elapsed_days, answer_ms, expected_ms) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .unwrap();
        for i in 0..n {
            let uid = format_ulid(Uid(1_000 + i as u128));
            for k in 0..REVIEWS_PER_ITEM {
                stmt.execute(params![
                    uid,
                    (k % 4 + 1),
                    START + k * DAY + i as i64 * 1_000,
                    if k == 0 { 0.0 } else { 1.0 },
                    if untimed {
                        None
                    } else {
                        Some(3_000 + (i % 7) as i64 * 500)
                    },
                    4_000,
                ])
                .unwrap();
            }
        }
    }
    tx.commit().unwrap();
    dir
}

fn rate_new(conn: &mut rusqlite::Connection, item: Uid, settings: &VaultSettings, now: i64) {
    // Each new card gets its own run, so the run events do not collide.
    let run = item;
    let out = StepOutput {
        events: vec![RunEvent {
            run,
            seq: 0,
            t_ms: 0,
            item: Some(item),
            kind: RunEventKind::ItemRated(Rating::Good),
        }],
        latency: None,
    };
    let ctx = StepContext {
        run,
        effect: ScheduleEffect::UpdateSchedule,
        state: RunState::Running,
        timing: Some(RatingTiming {
            answer_ms: 3_000,
            expected_ms: 4_000,
        }),
    };
    record_step(conn, &ctx, &out, UnixMs(now), settings).unwrap();
}

#[test]
#[ignore]
fn a_large_vault_keeps_the_starting_difficulty_cheap() {
    let n = large_n();
    let t = Instant::now();
    let dir = build(n);
    println!(
        "build {n} cards, {} reviews: {:?}",
        n * REVIEWS_PER_ITEM as usize,
        t.elapsed()
    );

    let t = Instant::now();
    let (session, _) = open_vault(dir.to_str().unwrap()).unwrap();
    println!("open (index) {:?}", t.elapsed());
    drop(session);

    let (mut session, _) = open_vault(dir.to_str().unwrap()).unwrap();
    for _ in 0..3 {
        let t = Instant::now();
        let rows = first_review_timings(&session.conn).unwrap();
        println!(
            "first_review_timings: {} rows in {:?}",
            rows.len(),
            t.elapsed()
        );
    }
    let t = Instant::now();
    let observed = observed_difficulties(&session.conn).unwrap();
    println!(
        "observed_difficulties: {} in {:?}",
        observed.len(),
        t.elapsed()
    );

    let on = vault_settings_default();
    let mut off = vault_settings_default();
    off.calibrated_start = false;
    let half = NEW_CARDS / 2;
    for (label, settings, first) in [
        ("calibrated on", &on, n),
        ("calibrated off", &off, n + half),
    ] {
        let t = Instant::now();
        for k in 0..half {
            let item = Uid(1_000 + (first + k) as u128);
            rate_new(
                &mut session.conn,
                item,
                settings,
                START + 90 * DAY + k as i64,
            );
        }
        println!(
            "{half} new-card first ratings, {label}: {:?} total",
            t.elapsed()
        );
    }
    let stored =
        studyfrog_store::schedule::load_mem_state(&session.conn, Uid(1_000 + n as u128)).unwrap();
    println!("a new card's stored starting state (calibrated on): {stored:?}");
    let _ = fs::remove_dir_all(&dir);
}
