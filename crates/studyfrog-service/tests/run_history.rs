#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The history of runs: mode, pacing, time used and answers a minute.

use studyfrog_core::event::{EventKind, RunEvent};
use studyfrog_core::pacing::Pacing;
use studyfrog_core::plan::plan_spaced;
use studyfrog_core::review::Rating;
use studyfrog_core::run::{Run, RunMode, RunState};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::run_history::{answers_per_minute, run_history};
use studyfrog_service::session::open_vault;
use studyfrog_store::runs::{append_events, insert_run};

fn run(uid: u128, mode: RunMode, pacing: Pacing, started: i64) -> Run {
    let mut plan = plan_spaced();
    plan.pacing = pacing;
    Run {
        uid: Uid(uid),
        mode,
        plan,
        queue: vec![Uid(1)],
        state: RunState::Finished,
        started_at: Some(UnixMs(started)),
    }
}

fn events(run: u128, rated_at: &[u32], end: u32) -> Vec<RunEvent> {
    let mut list = vec![(0, EventKind::RunStarted)];
    list.extend(
        rated_at
            .iter()
            .map(|t| (*t, EventKind::ItemRated(Rating::Good))),
    );
    list.push((end, EventKind::RunFinished));
    list.into_iter()
        .enumerate()
        .map(|(i, (t_ms, kind))| RunEvent {
            run: Uid(run),
            seq: i as u32,
            t_ms,
            item: matches!(kind, EventKind::ItemRated(_)).then_some(Uid(1)),
            kind,
        })
        .collect()
}

#[test]
fn each_run_lists_its_mode_pacing_time_used_and_answers_a_minute_newest_first() {
    let dir = std::env::temp_dir().join(format!("studyfrog-rhist-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    insert_run(&s.conn, &run(1, RunMode::Flip, Pacing::Free, 1_000)).unwrap();
    append_events(&s.conn, &events(1, &[10_000, 20_000], 60_000)).unwrap();
    insert_run(
        &s.conn,
        &run(2, RunMode::Answer, Pacing::Countdown(600), 2_000),
    )
    .unwrap();
    append_events(&s.conn, &events(2, &[30_000], 120_000)).unwrap();

    let history = run_history(&s.conn).unwrap();
    assert_eq!(history.iter().map(|r| r.uid.0).collect::<Vec<_>>(), [2, 1]);
    let (a, b) = (&history[0], &history[1]);
    assert_eq!(
        (a.mode, a.pacing, a.rated, a.used_ms),
        (RunMode::Answer, Pacing::Countdown(600), 1, 120_000)
    );
    assert_eq!(
        (b.mode, b.pacing, b.rated, b.used_ms),
        (RunMode::Flip, Pacing::Free, 2, 60_000)
    );
    assert_eq!(answers_per_minute(b.rated, b.used_ms), Some(2.0));
    assert_eq!(answers_per_minute(1, 0), None);
    let _ = std::fs::remove_dir_all(&dir);
}
