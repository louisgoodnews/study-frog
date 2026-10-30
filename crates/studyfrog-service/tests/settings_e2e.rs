#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Stack settings in action: SM-2, retention overrides, scheduler switches and daily budgets.

mod common;

use std::fs;

use common::{DAY, NOW, build, drive, env_with, reload, script, started};
use studyfrog_core::plan::{plan_sequential, plan_spaced};
use studyfrog_core::review::Rating::{Again, Easy, Good};
use studyfrog_core::scheduler::Scheduler;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_store::schedule::{load_mem_state, load_reviews, load_sm2_state};

fn sm2(env: &common::Env, id: u128) -> studyfrog_store::schedule::StoredSm2 {
    load_sm2_state(&env.conn, Uid(id)).unwrap().unwrap()
}

fn at_day(n: i64) -> UnixMs {
    UnixMs(NOW.0 + n * DAY)
}

#[test]
fn an_sm2_stack_uses_sm2_intervals_and_leaves_fsrs_state_alone() {
    let mut env = env_with(Some("scheduler = \"sm2\"\n"));
    let mut first = started(&env, build(&env, plan_spaced(), 100, NOW));
    drive(
        &mut env,
        &mut first,
        NOW,
        script(&[Good, Easy, Again, Good]),
    );

    assert_eq!(
        (sm2(&env, 11).state.interval_days, sm2(&env, 11).due),
        (1, at_day(1)),
        "SM-2: first success is 1 day"
    );
    assert_eq!(sm2(&env, 12).state.interval_days, 1);
    assert!(
        (sm2(&env, 12).state.ease - 2.6).abs() < 1e-5,
        "Easy raises the ease"
    );
    assert_eq!(
        sm2(&env, 13).state.repetitions,
        1,
        "Again reset it, the requeued Good counted once"
    );
    assert!(
        load_mem_state(&env.conn, Uid(11)).unwrap().is_none(),
        "no FSRS state for SM-2 items"
    );
    assert_eq!(load_reviews(&env.conn, Uid(13)).unwrap().len(), 2);

    // The next day the second repetition is worth 6 days.
    let mut second = started(&env, build(&env, plan_spaced(), 101, at_day(1)));
    assert_eq!(second.run.queue.len(), 3);
    drive(
        &mut env,
        &mut second,
        at_day(1),
        script(&[Good, Good, Good]),
    );
    assert_eq!(sm2(&env, 11).state.interval_days, 6);
    assert_eq!(sm2(&env, 11).due, at_day(7));
}

#[test]
fn a_stack_retention_override_lengthens_fsrs_intervals() {
    let mut env = env_with(Some("request_retention = 0.8\n"));
    let mut run = started(&env, build(&env, plan_spaced(), 100, NOW));
    drive(&mut env, &mut run, NOW, script(&[Good, Good, Good]));
    // S0(Good) = 2.3065: 2 days at 90% retention, 8 days at 80%.
    let due = load_mem_state(&env.conn, Uid(11)).unwrap().unwrap().due;
    assert_eq!(due, at_day(8));
}

#[test]
fn vault_defaults_apply_and_a_stack_can_override_them() {
    let mut env = env_with(Some("scheduler = \"fsrs\"\n"));
    env.vault.scheduler = Scheduler::Sm2; // vault default is SM-2, the stack chooses FSRS
    let mut run = started(&env, build(&env, plan_spaced(), 100, NOW));
    drive(&mut env, &mut run, NOW, script(&[Good, Good, Good]));
    assert!(load_mem_state(&env.conn, Uid(11)).unwrap().is_some());
    assert!(load_sm2_state(&env.conn, Uid(11)).unwrap().is_none());

    let mut plain = env_with(None);
    plain.vault.scheduler = Scheduler::Sm2; // no stack setting: the vault default decides
    let mut run = started(&plain, build(&plain, plan_spaced(), 100, NOW));
    drive(&mut plain, &mut run, NOW, script(&[Good, Good, Good]));
    assert!(load_sm2_state(&plain.conn, Uid(11)).unwrap().is_some());
}

#[test]
fn switching_a_stack_keeps_items_scheduled_and_the_next_review_starts_fresh_state() {
    let mut env = env_with(None);
    let mut run = started(&env, build(&env, plan_spaced(), 100, NOW));
    drive(&mut env, &mut run, NOW, script(&[Good, Good, Good])); // FSRS: due in 2 days

    fs::write(env.dir.join("deck/_stack.toml"), "scheduler = \"sm2\"\n").unwrap();
    reload(&mut env);
    assert!(
        build(&env, plan_spaced(), 101, at_day(1)).queue.is_empty(),
        "still not due"
    );
    let due = build(&env, plan_spaced(), 102, at_day(2));
    assert_eq!(
        due.queue.len(),
        3,
        "the old FSRS due date keeps the items scheduled"
    );

    let mut run = started(&env, due);
    drive(&mut env, &mut run, at_day(2), script(&[Good, Good, Good]));
    let sm2 = load_sm2_state(&env.conn, Uid(11)).unwrap().unwrap();
    assert_eq!(
        (sm2.state.repetitions, sm2.state.interval_days),
        (1, 1),
        "fresh SM-2 state"
    );
    assert_eq!(
        load_reviews(&env.conn, Uid(11)).unwrap().len(),
        2,
        "one shared history"
    );
}

#[test]
fn daily_new_limit_is_spent_per_day_and_not_by_practice_runs() {
    let mut env = env_with(Some("daily_new_limit = 2\n"));
    let first = build(&env, plan_spaced(), 100, NOW);
    assert_eq!(
        first.queue,
        [Uid(11), Uid(12)],
        "only 2 of the 3 new cards today"
    );
    let mut run = started(&env, first);
    drive(&mut env, &mut run, NOW, script(&[Good, Good]));

    let again = build(&env, plan_spaced(), 101, at_day(0));
    assert!(
        again.queue.is_empty(),
        "the allowance is spent and nothing is due yet"
    );

    let mut practice = started(&env, build(&env, plan_sequential(), 102, NOW));
    drive(&mut env, &mut practice, NOW, script(&[Good, Good, Good]));
    assert!(
        build(&env, plan_spaced(), 103, NOW).queue.is_empty(),
        "practice-only runs do not count"
    );

    assert_eq!(
        build(&env, plan_spaced(), 104, at_day(1)).queue,
        [Uid(13)],
        "a new day, the next new card"
    );
}

#[test]
fn daily_review_limit_caps_due_reviews_per_day() {
    let mut env = env_with(Some("daily_review_limit = 1\n"));
    let mut run = started(&env, build(&env, plan_spaced(), 100, NOW));
    drive(&mut env, &mut run, NOW, script(&[Good, Good, Good]));

    let day9 = build(&env, plan_spaced(), 101, at_day(9));
    assert_eq!(day9.queue.len(), 1, "three are due, one review allowed");
    let mut run = started(&env, day9);
    drive(&mut env, &mut run, at_day(9), script(&[Good]));
    assert!(
        build(&env, plan_spaced(), 102, at_day(9)).queue.is_empty(),
        "today's review allowance is spent"
    );
    assert_eq!(
        build(&env, plan_spaced(), 103, at_day(10)).queue.len(),
        1,
        "tomorrow one more"
    );
}
