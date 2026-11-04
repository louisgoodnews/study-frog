#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0218: an item held by two stacks is remembered by each on its own (the owner's Flashcard 100 example: stack A
//! schedules it 5 weeks out, stack B 3 weeks).

mod common;

use std::fs;

use common::{DAY, Env, NOW, build, drive, script, started};
use studyfrog_core::memory::MemState;
use studyfrog_core::plan::plan_spaced;
use studyfrog_core::review::Rating::{Again, Good};
use studyfrog_core::scheduler::Scheduler;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_store::due_dates::load_dues_in;
use studyfrog_store::reindex::reindex;
use studyfrog_store::reviews::load_reviews_timed_in;
use studyfrog_store::schedule::{StoredMem, load_mem_state_in, save_mem_state_in};
use studyfrog_store::schedule_stacks::load_stack_schedules;

const A: Uid = Uid(1);
const B: Uid = Uid(2);
const CARD: Uid = Uid(100);

/// Stacks A and B; the card sits in A and is referenced by B. B keeps a higher retention, so it asks for shorter steps.
fn two_stacks() -> Env {
    let mut env = common::env_with(None);
    let id = |n: u128| format_ulid(Uid(n));
    let w = |p: &str, t: String| fs::write(env.dir.join(p), t).unwrap();
    fs::create_dir_all(env.dir.join("b")).unwrap();
    w(
        "b/_stack.md",
        format!("---\nid: {}\nkind: stack\ntitle: Beta\n---\n", id(2)),
    );
    w("b/_stack.toml", "request_retention = 0.97\n".to_owned());
    w(
        "deck/shared.md",
        format!(
            "---\nid: {}\nkind: card\ntitle: Shared\nreferenced_by: [{}]\n---\nq\n???\na\n",
            id(100),
            id(2)
        ),
    );
    reindex(&mut env.conn, &env.dir).unwrap();
    env
}

fn plan_over(stacks: &[Uid]) -> studyfrog_core::plan::RunPlan {
    let mut plan = plan_spaced();
    plan.filter.stacks = stacks.to_vec();
    plan.filter.kinds = vec![];
    plan
}

fn rate_card_in(env: &mut Env, run: u128, stacks: &[Uid], rating: studyfrog_core::review::Rating) {
    let mut r = build(env, plan_over(stacks), run, NOW);
    r.queue.retain(|u| *u == CARD);
    assert_eq!(r.queue, [CARD], "the card is in the run");
    let mut session = started(env, r);
    drive(env, &mut session, NOW, script(&[rating]));
}

fn due(env: &Env, stack: Uid) -> Option<UnixMs> {
    load_mem_state_in(&env.conn, (CARD, stack))
        .unwrap()
        .map(|m| m.due)
}

#[test]
fn a_run_of_one_stack_leaves_the_other_stacks_schedule_alone() {
    let mut env = two_stacks();
    rate_card_in(&mut env, 1, &[A], Good);
    assert!(due(&env, A).is_some(), "A remembers the rating");
    assert_eq!(due(&env, B), None, "B has not seen the card");
    assert_eq!(
        load_reviews_timed_in(&env.conn, (CARD, A)).unwrap().len(),
        1
    );
    assert!(
        load_reviews_timed_in(&env.conn, (CARD, B))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn each_stack_schedules_the_card_by_its_own_settings_and_shows_it_on_its_own_day() {
    let mut env = two_stacks();
    rate_card_in(&mut env, 1, &[A, B], Good);
    let (da, db) = (due(&env, A).unwrap(), due(&env, B).unwrap());
    assert!(
        db.0 < da.0,
        "B asks for a higher retention, so its step is shorter: {db:?} < {da:?}"
    );
    assert_eq!(
        load_reviews_timed_in(&env.conn, (CARD, A)).unwrap().len(),
        1
    );
    assert_eq!(
        load_reviews_timed_in(&env.conn, (CARD, B)).unwrap().len(),
        1
    );

    let all = load_stack_schedules(&env.conn, |_| Scheduler::Fsrs).unwrap();
    assert_eq!(all[&(CARD, A)].due, da);
    assert_eq!(all[&(CARD, B)].due, db);

    // Later, on the day between the two: due in B only.
    let between = UnixMs((da.0 + db.0) / 2);
    let queue_at = |env: &Env, stacks: &[Uid]| {
        let mut plan = plan_over(stacks);
        plan.order = studyfrog_core::plan::Order::Due;
        build(env, plan, 9, between).queue.contains(&CARD)
    };
    assert!(!queue_at(&env, &[A]), "not yet due in A");
    assert!(queue_at(&env, &[B]), "due in B");
    assert!(queue_at(&env, &[A, B]), "due in any of them, queued once");
}

#[test]
fn rating_in_one_stack_does_not_move_the_other_and_a_set_due_date_stays_put() {
    let mut env = two_stacks();
    rate_card_in(&mut env, 1, &[A, B], Good);
    let before_b = load_mem_state_in(&env.conn, (CARD, B)).unwrap().unwrap();
    // A later run of A alone, a day after, with a lapse.
    // An early review: library order, but the schedule is still updated.
    let mut early = plan_over(&[A]);
    early.order = studyfrog_core::plan::Order::Sequential;
    let mut r = build(&env, early, 2, UnixMs(NOW.0 + DAY));
    r.queue.retain(|u| *u == CARD);
    let mut s = started(&env, r);
    drive(&mut env, &mut s, UnixMs(NOW.0 + DAY), script(&[Again]));
    assert_eq!(
        load_mem_state_in(&env.conn, (CARD, B)).unwrap().unwrap(),
        before_b
    );
    assert_eq!(
        load_reviews_timed_in(&env.conn, (CARD, A)).unwrap().len(),
        2
    );
    assert_eq!(
        load_reviews_timed_in(&env.conn, (CARD, B)).unwrap().len(),
        1
    );
    let dues = load_dues_in(&env.conn, (CARD, B)).unwrap();
    assert_eq!(dues.fsrs, Some(before_b.due.0));
}

#[test]
fn an_item_new_in_one_selected_stack_is_still_queued_as_new() {
    let mut env = two_stacks();
    rate_card_in(&mut env, 1, &[A], Good);
    let far = UnixMs(NOW.0 - 1);
    // Scheduled far ahead in A, never seen in B: a run over both still offers it, because it is new in B.
    save_mem_state_in(
        &env.conn,
        (CARD, A),
        &StoredMem {
            state: MemState {
                stability: 30.0,
                difficulty: 5.0,
            },
            due: UnixMs(NOW.0 + 30 * DAY),
            last_review: far,
        },
    )
    .unwrap();
    let mut plan = plan_over(&[A, B]);
    plan.order = studyfrog_core::plan::Order::Due;
    assert!(build(&env, plan, 7, NOW).queue.contains(&CARD));
    let mut only_a = plan_over(&[A]);
    only_a.order = studyfrog_core::plan::Order::Due;
    assert!(!build(&env, only_a, 8, NOW).queue.contains(&CARD));
}

#[test]
fn the_calendar_shows_the_card_on_each_stacks_own_day_and_a_move_changes_one_stack_only() {
    use studyfrog_service::review_plan::{move_reviews, plan_reviews};
    use studyfrog_service::session::open_vault;
    use studyfrog_service::timezone::{local_day_of, local_day_start};

    let env = two_stacks();
    let dir = env.dir.clone();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let mem = |due: i64| StoredMem {
        state: MemState {
            stability: 10.0,
            difficulty: 5.0,
        },
        due: UnixMs(due),
        last_review: UnixMs(NOW.0 - DAY),
    };
    let (in_5_weeks, in_3_weeks) = (NOW.0 + 35 * DAY, NOW.0 + 21 * DAY);
    save_mem_state_in(&s.conn, (CARD, A), &mem(in_5_weeks)).unwrap();
    save_mem_state_in(&s.conn, (CARD, B), &mem(in_3_weeks)).unwrap();
    let today = local_day_of(NOW);
    let plan = plan_reviews(&s, (today, today + 60)).unwrap();
    let day_of = |stack: Uid| {
        plan.iter()
            .find(|e| e.stack == Some(stack) && e.items.contains(&CARD))
            .map(|e| e.day)
    };
    assert_eq!(day_of(A), Some(local_day_of(UnixMs(in_5_weeks))));
    assert_eq!(day_of(B), Some(local_day_of(UnixMs(in_3_weeks))));

    move_reviews(&s, (Some(B), &[CARD]), today + 40, NOW).unwrap();
    assert_eq!(
        load_mem_state_in(&s.conn, (CARD, B)).unwrap().unwrap().due,
        local_day_start(today + 40)
    );
    let a_due = load_mem_state_in(&s.conn, (CARD, A)).unwrap().unwrap().due;
    assert_eq!(a_due, UnixMs(in_5_weeks), "A did not move");
}
