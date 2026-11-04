#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0219: familiarity and difficulty for each (item, stack), averaged on the item, read from the database only.

mod common;

use std::fs;

use common::{Env, NOW, build, drive, script, started};
use studyfrog_core::plan::plan_spaced;
use studyfrog_core::review::Rating::{Again, Easy};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::fam_curve::item_curve;
use studyfrog_service::fam_refs::{average_over, overall, reference_scores};
use studyfrog_service::fam_report::familiarity_report;
use studyfrog_store::reindex::reindex;

const A: Uid = Uid(1);
const B: Uid = Uid(2);
const CARD: Uid = Uid(100);

fn two_stacks() -> Env {
    let mut env = common::env_with(None);
    let id = |n: u128| format_ulid(Uid(n));
    fs::create_dir_all(env.dir.join("b")).unwrap();
    let w = |p: &str, t: String| fs::write(env.dir.join(p), t).unwrap();
    w(
        "b/_stack.md",
        format!("---\nid: {}\nkind: stack\ntitle: Beta\n---\n", id(2)),
    );
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

fn rate_in(env: &mut Env, run: u128, stacks: &[Uid], rating: studyfrog_core::review::Rating) {
    let mut plan = plan_spaced();
    plan.filter.stacks = stacks.to_vec();
    plan.order = studyfrog_core::plan::Order::Sequential;
    let mut r = build(env, plan, run, NOW);
    r.queue.retain(|u| *u == CARD);
    let mut s = started(env, r);
    drive(env, &mut s, NOW, script(&[rating]));
}

fn value(env: &Env, stack: Uid) -> f32 {
    let at = UnixMs(NOW.0 + 1000);
    let all = reference_scores(&env.conn, at).unwrap();
    all.iter()
        .find(|s| s.uid == CARD && s.stack == stack)
        .unwrap()
        .fam
        .unwrap()
        .value
}

#[test]
fn each_stack_counts_only_the_reviews_it_made() {
    let mut env = two_stacks();
    rate_in(&mut env, 1, &[A], Easy);
    rate_in(&mut env, 2, &[B], Again);
    let (a, b) = (value(&env, A), value(&env, B));
    assert!(a > b, "A saw an easy rating and B a lapse: {a} > {b}");
    let scores = reference_scores(&env.conn, UnixMs(NOW.0 + 1000)).unwrap();
    let mine: Vec<_> = scores.iter().filter(|s| s.uid == CARD).collect();
    assert_eq!(mine.len(), 2);
    assert!(
        mine[0].primary && !mine[1].primary,
        "the own stack comes first"
    );
    assert_eq!((mine[0].reviews, mine[1].reviews), (1, 1));
}

#[test]
fn the_item_shows_the_average_and_one_line_per_reference() {
    let mut env = two_stacks();
    rate_in(&mut env, 1, &[A], Easy);
    rate_in(&mut env, 2, &[B], Again);
    let now = UnixMs(NOW.0 + 1000);
    let curve = item_curve(&env.conn, CARD, now).unwrap();
    assert_eq!(curve.references.len(), 2);
    let (avg, difficulty) = overall(&curve.references);
    let expected = average_over([value(&env, A), value(&env, B)]).unwrap();
    assert!((avg.unwrap() - expected).abs() < 1e-5);
    let (da, db) = (
        curve.references[0].difficulty.unwrap(),
        curve.references[1].difficulty.unwrap(),
    );
    assert!((difficulty.unwrap() - (da + db) / 2.0).abs() < 1e-5);
    assert!(curve.references.iter().all(|r| r.due_ms.is_some()));
    assert_eq!(curve.references[0].title, "Deck");
}

#[test]
fn a_stack_report_uses_the_stacks_own_value_and_nothing_is_written_to_the_file() {
    let mut env = two_stacks();
    rate_in(&mut env, 1, &[A], Easy);
    rate_in(&mut env, 2, &[B], Again);
    let report = familiarity_report(&env.conn, UnixMs(NOW.0 + 1000)).unwrap();
    let percent = |folder: &str| {
        report
            .stacks
            .iter()
            .find(|g| g.folder == folder)
            .unwrap()
            .percent
    };
    assert!(
        percent("deck") > percent("b"),
        "{:?} vs {:?}",
        percent("deck"),
        percent("b")
    );
    let text = fs::read_to_string(env.dir.join("deck/shared.md")).unwrap();
    assert!(
        !text.contains("familiarity") && !text.contains("difficulty"),
        "{text}"
    );
}
