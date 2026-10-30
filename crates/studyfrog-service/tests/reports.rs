#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Reports from a real vault's history.

use std::fs;
use std::path::PathBuf;

use studyfrog_core::event::Difficulty;
use studyfrog_core::review::{Rating, Review};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_report::simulate::{SimConfig, simulate_history};
use studyfrog_service::reports::{FORECAST_DAYS, build_reports};
use studyfrog_service::schedule::apply_rating;
use studyfrog_service::session::open_vault;
use studyfrog_srs::params::params_default;
use studyfrog_store::schedule::{insert_review, save_mem_state};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const DAY: i64 = 86_400_000;
const T0: i64 = 1_800_000_000_000;

fn vault(label: &str) -> Dir {
    let dir = std::env::temp_dir().join(format!("studyfrog-rep-{label}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let text = format!(
        "---\nid: {}\nkind: card\ntitle: C\n---\nq\n???\na\n",
        format_ulid(Uid(10))
    );
    fs::write(dir.join("c.md"), text).unwrap();
    Dir(dir)
}

fn event(
    s: &studyfrog_service::session::VaultSession,
    seq: u32,
    t: u32,
    item: Option<u128>,
    kind: &str,
    payload: Option<&str>,
) {
    s.conn
        .execute(
            "INSERT INTO run_event (run_uid, seq, t_ms, item_uid, kind, payload) VALUES (?6, ?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![seq, t, item.map(|i| format_ulid(Uid(i))), kind, payload, format_ulid(Uid(500))],
        )
        .unwrap();
}

#[test]
fn a_small_history_gives_every_report_and_a_simulated_backtest() {
    let d = vault("small");
    let (s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let params = params_default();
    let mut prev = None;
    for (rating, day) in [(Rating::Good, 0), (Rating::Good, 2), (Rating::Again, 9)] {
        let (r, mem) = apply_rating(
            prev.as_ref(),
            Uid(10),
            rating,
            UnixMs(T0 + day * DAY),
            &params,
        );
        insert_review(&s.conn, &r).unwrap();
        save_mem_state(&s.conn, Uid(10), &mem).unwrap();
        prev = Some(mem);
    }
    for (seq, t, item, kind, payload) in [
        (0, 0, None, "run_started", None),
        (1, 0, Some(10), "item_shown", None),
        (2, 1_000, Some(10), "item_marked", Some("difficult")),
        (3, 4_000, Some(10), "item_flipped", None),
        (4, 5_000, Some(10), "item_rated", Some("3")),
    ] {
        event(&s, seq, t, item, kind, payload);
    }

    let r = build_reports(&s, UnixMs(T0 + 10 * DAY), 0).unwrap();
    assert_eq!(r.reviews, 3);
    let spaced: u32 = r.retention.iter().map(|b| b.reviews).sum();
    let recalled: u32 = r.retention.iter().map(|b| b.recalled).sum();
    assert_eq!(
        (spaced, recalled),
        (2, 1),
        "two spaced reviews, one of them rated Again"
    );
    assert_eq!(r.latency.unwrap().p50, 4_000);
    assert_eq!(r.by_mark.len(), 1);
    assert_eq!(
        (r.by_mark[0].mark, r.by_mark[0].stats.n),
        (Some(Difficulty::Difficult), 1)
    );
    assert_eq!(r.forecast.len(), FORECAST_DAYS);
    assert!(r.forecast.iter().all(|x| *x >= 0.0));
    assert!(
        r.forecast.iter().sum::<f32>() > 0.0,
        "the scheduled item comes back"
    );
    assert!(
        r.backtest.simulated && r.backtest.error.is_some(),
        "too little history: simulated"
    );
}

#[test]
fn a_long_history_is_backtested_on_itself() {
    let d = vault("long");
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let history = simulate_history(
        &SimConfig {
            seed: 3,
            days: 150,
            new_per_day: 12,
            miss_percent: 10,
        },
        &params_default(),
    );
    let tx = s.conn.transaction().unwrap();
    let mut last = std::collections::HashMap::new();
    for r in &history {
        let elapsed = last
            .insert(r.item, r.day)
            .map_or(0.0, |d| (r.day - d) as f32);
        let review = Review {
            item: Uid(u128::from(r.item) + 100),
            rating: r.rating,
            at: UnixMs(T0 + r.day * DAY),
            elapsed_days: elapsed,
        };
        insert_review(&tx, &review).unwrap();
    }
    tx.commit().unwrap();
    let r = build_reports(&s, UnixMs(T0 + 150 * DAY), 0).unwrap();
    assert_eq!(r.reviews, history.len());
    assert!(!r.backtest.simulated, "enough real history");
    let e = r.backtest.error.unwrap();
    assert!(e.weekly_error < 0.4, "{e:?}");
    assert!(r.retention.iter().any(|b| b.reviews > 100));
}

#[test]
fn an_empty_vault_reports_nothing_without_failing() {
    let d = vault("empty");
    let (s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let r = build_reports(&s, UnixMs(T0), 0).unwrap();
    assert_eq!((r.reviews, r.latency, r.by_mark.len()), (0, None, 0));
    assert_eq!(r.forecast, vec![0.0; FORECAST_DAYS]);
    assert!(r.retention.iter().all(|b| b.reviews == 0));
}
