#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! End to end: vault -> index -> library -> queue -> engine -> persisted events and schedule.

mod common;

use common::{DAY, NOW, build, drive, env, script, started};
use studyfrog_core::plan::{plan_sequential, plan_spaced};
use studyfrog_core::review::Rating::{Again, Easy, Good};
use studyfrog_core::run::RunState;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::rehearse::recover_runs;
use studyfrog_store::runs::{load_events, load_run};
use studyfrog_store::schedule::{load_all_mem_states, load_mem_state, load_reviews};

#[test]
fn spaced_run_persists_events_and_updates_the_schedule() {
    let mut env = env();
    let run = build(&env, plan_spaced(), 100, NOW);
    assert_eq!(
        run.queue,
        [Uid(11), Uid(12), Uid(13)],
        "new items in library order"
    );
    let mut session = started(&env, run);

    // 13 is rated Again once: it comes back at the end and is then rated Good.
    let log = drive(
        &mut env,
        &mut session,
        NOW,
        script(&[Good, Easy, Again, Good]),
    );

    assert_eq!(
        load_events(&env.conn, Uid(100)).unwrap(),
        log,
        "the persisted log is the engine's log"
    );
    let stored = load_run(&env.conn, Uid(100)).unwrap().unwrap();
    assert_eq!(
        (stored.state, stored.started_at),
        (RunState::Finished, Some(NOW))
    );

    let reviews = |id| load_reviews(&env.conn, Uid(id)).unwrap();
    assert_eq!(
        (reviews(11).len(), reviews(12).len(), reviews(13).len()),
        (1, 1, 2)
    );
    assert_eq!(
        reviews(13).iter().map(|r| r.rating).collect::<Vec<_>>(),
        [Again, Good]
    );

    // FSRS-6 defaults: S0(Good) = 2.3065 -> 2 days, S0(Easy) = 8.2956 -> 8 days, and the lapse is
    // re-reviewed the same day, which keeps its interval at 1 day.
    let due = |id| load_mem_state(&env.conn, Uid(id)).unwrap().unwrap().due;
    assert_eq!(due(11), UnixMs(NOW.0 + 2 * DAY));
    assert_eq!(due(12), UnixMs(NOW.0 + 8 * DAY));
    assert_eq!(due(13), UnixMs(NOW.0 + DAY));
}

#[test]
fn items_come_back_when_due_most_overdue_first() {
    let mut env = env();
    let mut session = started(&env, build(&env, plan_spaced(), 100, NOW));
    drive(
        &mut env,
        &mut session,
        NOW,
        script(&[Good, Easy, Again, Good]),
    );

    let queue_at = |now, run| build(&env, plan_spaced(), run, now).queue;
    assert!(
        queue_at(NOW, 101).is_empty(),
        "nothing is due right after the run"
    );
    assert_eq!(queue_at(UnixMs(NOW.0 + 2 * DAY), 102), [Uid(13), Uid(11)]);
    assert_eq!(
        queue_at(UnixMs(NOW.0 + 9 * DAY), 103),
        [Uid(13), Uid(11), Uid(12)]
    );
}

#[test]
fn practice_run_records_events_but_never_touches_the_schedule() {
    let mut env = env();
    let mut first = started(&env, build(&env, plan_spaced(), 100, NOW));
    drive(
        &mut env,
        &mut first,
        NOW,
        script(&[Good, Easy, Again, Good]),
    );
    let before = load_all_mem_states(&env.conn).unwrap();
    let review_count = |env: &common::Env| -> i64 {
        env.conn
            .query_row("SELECT COUNT(*) FROM review", [], |r| r.get(0))
            .unwrap()
    };
    let reviews_before = review_count(&env);

    let later = UnixMs(NOW.0 + 30 * DAY);
    let run = build(&env, plan_sequential(), 200, later);
    assert_eq!(
        run.queue,
        [Uid(11), Uid(12), Uid(13)],
        "a practice run goes through everything"
    );
    let mut practice = started(&env, run);
    let log = drive(&mut env, &mut practice, later, script(&[Again, Good, Easy]));

    assert_eq!(load_all_mem_states(&env.conn).unwrap(), before);
    assert_eq!(review_count(&env), reviews_before);
    assert_eq!(
        load_events(&env.conn, Uid(200)).unwrap(),
        log,
        "events and latency data are still kept"
    );
    assert_eq!(
        load_run(&env.conn, Uid(200)).unwrap().unwrap().state,
        RunState::Finished
    );
}

#[test]
fn a_run_left_open_by_a_crash_is_recovered_as_aborted() {
    let mut env = env();
    let mut session = started(&env, build(&env, plan_spaced(), 100, NOW));
    drive(
        &mut env,
        &mut session,
        NOW,
        vec![(0, studyfrog_rehearsal::input::Action::Start)],
    );
    drop(session); // the app "crashes" while the run is Running

    assert_eq!(recover_runs(&mut env.conn).unwrap(), 1);
    assert_eq!(
        load_run(&env.conn, Uid(100)).unwrap().unwrap().state,
        RunState::Aborted
    );
    let log = load_events(&env.conn, Uid(100)).unwrap();
    assert_eq!(
        log.last().map(|e| e.seq),
        Some(2),
        "closed with run_aborted after started + shown"
    );
}

#[test]
fn the_browser_listing_shows_when_each_item_is_next_due() {
    use studyfrog_store::overview::list_items;
    let mut env = env();
    let before: Vec<_> = list_items(&env.conn, Some("deck"))
        .unwrap()
        .into_iter()
        .map(|i| i.due)
        .collect();
    assert_eq!(before, [None, None, None], "nothing has been reviewed yet");
    let mut run = started(&env, build(&env, plan_spaced(), 100, NOW));
    drive(&mut env, &mut run, NOW, script(&[Good, Easy, Again, Good]));
    let dues: Vec<_> = list_items(&env.conn, Some("deck"))
        .unwrap()
        .into_iter()
        .map(|i| i.due)
        .collect();
    assert_eq!(
        dues,
        [
            Some(UnixMs(NOW.0 + 2 * DAY)),
            Some(UnixMs(NOW.0 + 8 * DAY)),
            Some(UnixMs(NOW.0 + DAY))
        ]
    );
}
