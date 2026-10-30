#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Starting a run and reading item content.

use std::fs;
use std::path::PathBuf;

use studyfrog_core::plan::{Order, RunPlan, plan_sequential, plan_shuffled};
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::content::load_item_content;
use studyfrog_service::create::IdSource;
use studyfrog_service::rehearse::start_run;
use studyfrog_service::session::{ServiceError, open_vault};
use studyfrog_store::runs::load_run;

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str) -> Dir {
    let dir = std::env::temp_dir().join(format!("studyfrog-start-{label}-{}", std::process::id()));
    fs::create_dir_all(dir.join("s")).unwrap();
    let w = |p: &str, t: &str| fs::write(dir.join(p), t).unwrap();
    w(
        "s/_stack.md",
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: S\n---\n",
    );
    w(
        "s/a.md",
        "---\nid: 0000000000000000000000000A\nkind: card\ntitle: A\n---\nfront A\n???\nback A\n",
    );
    w(
        "s/b.md",
        "---\nid: 0000000000000000000000000B\nkind: question\ntitle: B\n---\nWhy?\n???\nBecause.\n",
    );
    w(
        "s/c.md",
        "---\nid: 0000000000000000000000000C\nkind: note\ntitle: Note C\n---\nbody of C\n",
    );
    Dir(dir)
}

fn day_of(now: UnixMs) -> studyfrog_service::day::DayWindow {
    studyfrog_service::day::day_from(UnixMs(now.0 - now.0.rem_euclid(86_400_000)))
}

fn ids(n: u8) -> IdSource {
    IdSource {
        now_ms: 1_800_000_000_000,
        random: [n; 10],
    }
}

#[test]
fn content_is_read_from_the_file_found_through_the_index() {
    let d = vault("content");
    let (session, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let c = |id: u128| load_item_content(&session, Uid(id)).unwrap();
    let card = c(0xA);
    assert_eq!(
        (
            card.prompt.as_str(),
            card.answer.as_str(),
            card.title.as_str()
        ),
        ("front A", "back A", "A")
    );
    let q = c(0xB);
    assert_eq!((q.prompt.as_str(), q.answer.as_str()), ("Why?", "Because."));
    let n = c(0xC);
    assert_eq!(
        (n.prompt.as_str(), n.answer.as_str()),
        ("Note C", "body of C")
    );
    assert!(matches!(
        load_item_content(&session, Uid(999)),
        Err(ServiceError::Invalid(_))
    ));
}

#[test]
fn a_started_run_is_stored_with_a_seed_a_queue_and_the_idle_state() {
    let d = vault("start");
    let (session, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let now = UnixMs(1_800_000_000_000);
    let run = start_run(
        &session,
        plan_sequential(),
        RunMode::Answer,
        now,
        day_of(now),
        ids(7),
    )
    .unwrap();
    assert_eq!(run.queue, [Uid(0xA), Uid(0xB), Uid(0xC)]);
    assert_eq!((run.state, run.mode), (RunState::Idle, RunMode::Answer));
    assert_eq!(load_run(&session.conn, run.uid).unwrap().unwrap(), run);

    let a = start_run(
        &session,
        plan_shuffled(0),
        RunMode::Flip,
        now,
        day_of(now),
        ids(1),
    )
    .unwrap();
    assert_ne!(
        a.plan.seed, 0,
        "the seed comes from the random bytes and is stored with the run"
    );
    assert_eq!(
        load_run(&session.conn, a.uid).unwrap().unwrap().plan.seed,
        a.plan.seed
    );

    let clash = RunPlan {
        order: Order::Shuffled,
        ..plan_shuffled(0)
    };
    let again = start_run(&session, clash, RunMode::Flip, now, day_of(now), ids(1));
    assert!(
        matches!(again, Err(ServiceError::Store(_))),
        "the same id is refused, never overwritten"
    );
}
